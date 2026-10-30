"""Reject a mismatched same-run binary/image artifact before packaging or publishing."""
import hashlib
import json
from pathlib import Path
import re
import sys


def verify(folder, source_sha, architecture, kind):
    assert re.fullmatch(r'[0-9a-f]{40}', source_sha), 'invalid source SHA'
    assert architecture in ('amd64', 'arm64'), 'invalid architecture'
    assert kind in ('binary', 'image'), 'invalid artifact type'
    filename = 'the-sara-protocol' if kind == 'binary' else 'image.tar'
    metadata = json.loads((folder / 'metadata.json').read_text())
    assert metadata['source_sha'] == source_sha, 'wrong source commit'
    assert metadata['architecture'] == architecture, 'wrong architecture'
    assert metadata['kind'] == kind, 'wrong artifact type'
    artifact = folder / filename
    assert not artifact.is_symlink(), 'artifact must not be a symlink'
    with artifact.open('rb') as stream:
        checksum = hashlib.file_digest(stream, 'sha256').hexdigest()
    assert checksum == metadata['sha256'], 'artifact checksum mismatch'
    if kind == 'binary':
        with artifact.open('rb') as stream:
            header = stream.read(20)
        assert header[:6] == b'\x7fELF\x02\x01', 'expected 64-bit little-endian ELF'
        assert int.from_bytes(header[18:20], 'little') == {'amd64': 62, 'arm64': 183}[architecture], 'wrong ELF architecture'
    return artifact


if __name__ == '__main__':
    verify(Path(sys.argv[1]), *sys.argv[2:])
