import hashlib
import importlib.util
import json
from pathlib import Path
import pytest

spec = importlib.util.spec_from_file_location('verify_backend', Path(__file__).parents[1] / 'scripts/verify-backend-artifact.py')
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
SHA = 'a' * 40


def artifact(folder, arch='amd64', kind='binary'):
    content = bytearray(24)
    content[:6] = b'\x7fELF\x02\x01'
    content[18:20] = (62 if arch == 'amd64' else 183).to_bytes(2, 'little')
    file = folder / ('the-sara-protocol' if kind == 'binary' else 'image.tar')
    file.write_bytes(content)
    metadata = {'source_sha': SHA, 'architecture': arch, 'kind': kind, 'sha256': hashlib.sha256(content).hexdigest()}
    (folder / 'metadata.json').write_text(json.dumps(metadata))
    return file


@pytest.mark.parametrize('arch', ['amd64', 'arm64'])
def test_native_elf_and_image_artifacts_are_accepted(tmp_path, arch):
    for kind in ['binary', 'image']:
        file = artifact(tmp_path, arch, kind)
        assert v.verify(tmp_path, SHA, arch, kind) == file


@pytest.mark.parametrize('case', ['source', 'architecture', 'checksum', 'elf', 'kind', 'symlink'])
def test_wrong_artifacts_are_rejected_before_use(tmp_path, case):
    file = artifact(tmp_path)
    metadata = json.loads((tmp_path / 'metadata.json').read_text())
    if case == 'source': metadata['source_sha'] = 'b' * 40
    if case == 'architecture': metadata['architecture'] = 'arm64'
    if case == 'checksum': file.write_bytes(b'changed')
    if case == 'elf':
        file.write_bytes(b'not an ELF')
        metadata['sha256'] = hashlib.sha256(file.read_bytes()).hexdigest()
    if case == 'kind': metadata['kind'] = 'image'
    if case == 'symlink':
        file.rename(tmp_path / 'original')
        file.symlink_to(tmp_path / 'original')
    (tmp_path / 'metadata.json').write_text(json.dumps(metadata))
    with pytest.raises(AssertionError):
        v.verify(tmp_path, SHA, 'amd64', 'binary')
