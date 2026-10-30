import copy
import importlib.util
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('component_release', ROOT / '.github/scripts/component_release.py')
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)


def files(version='0.1.0'):
    return {
        r.FILES[0]: f'[package]\nname = "the-sara-protocol"\nversion = "{version}"\n\n[dependencies]\naxum = "0.8"\n',
        r.FILES[1]: f'version = 4\n\n[[package]]\nname = "the-sara-protocol"\nversion = "{version}"\n\n[[package]]\nname = "dependency"\nversion = "0.1.0"\n',
        r.FILES[2]: '# Backend\n\n## [Unveröffentlicht]\n\n- Neue Funktion.\n',
    }


@pytest.mark.parametrize(('current', 'bump', 'expected'), [
    ('0.1.0', 'initial', '0.1.0'), ('0.1.0', 'patch', '0.1.1'),
    ('0.1.9', 'minor', '0.2.0'), ('0.2.0', 'major', '1.0.0'),
    ('1.2.3', 'minor', '1.3.0'), ('1.2.3', 'major', '2.0.0'),
])
def test_version_policy(current, bump, expected):
    assert r.next_version(current, bump) == expected


@pytest.mark.parametrize(('current', 'bump'), [
    ('0.2.0', 'initial'), ('01.2.3', 'patch'), ('1.2.3-rc.1', 'patch'),
    ('1.2.3', 'patch; echo unsafe'),
])
def test_invalid_version_requests_fail(current, bump):
    with pytest.raises(r.ReleaseError):
        r.next_version(current, bump)


def test_version_changes_preserve_dependencies_and_history():
    original = files()
    original[r.FILES[2]] += '\n## [0.0.1] – 2026-01-01\n\n- Alt.\n'
    updated = r.prepare_files(original, '0.2.0', '2026-10-30')
    assert r.version_of(updated) == '0.2.0'
    assert 'name = "dependency"\nversion = "0.1.0"' in updated[r.FILES[1]]
    assert 'axum = "0.8"' in updated[r.FILES[0]]
    assert '## [0.0.1] – 2026-01-01\n\n- Alt.' in updated[r.FILES[2]]
    assert '## [0.2.0] – 2026-10-30\n\n- Neue Funktion.' in updated[r.FILES[2]]
    assert original == {**files(), r.FILES[2]: original[r.FILES[2]]}


@pytest.mark.parametrize('change', ['lock_mismatch', 'empty_notes', 'duplicate_version'])
def test_inconsistent_release_inputs_fail(change):
    original = files()
    if change == 'lock_mismatch':
        original[r.FILES[1]] = original[r.FILES[1]].replace('version = "0.1.0"', 'version = "9.0.0"')
    elif change == 'empty_notes':
        original[r.FILES[2]] = '# Backend\n\n## [Unveröffentlicht]\n\n_Noch keine Änderungen._\n'
    else:
        original[r.FILES[2]] += '\n## [0.2.0] – 2026-01-01\n'
    with pytest.raises(r.ReleaseError):
        r.prepare_files(original, '0.2.0', '2026-10-30')


class FakeAPI:
    repo = 'owner/repo'

    def __init__(self):
        self.old = files()
        self.current = r.prepare_files(self.old, '0.1.0', '2026-10-30')
        self.run = {'id': 7, 'workflow_id': 9, 'conclusion': 'success', 'event': 'push',
                    'head_branch': 'main', 'head_sha': 'merged',
                    'head_repository': {'full_name': self.repo}}
        self.pr = {'number': 2, 'merged_at': 'today', 'merge_commit_sha': 'merged',
                   'base': {'ref': 'main'},
                   'head': {'ref': 'release/backend/v0.1.0', 'repo': {'full_name': self.repo}},
                   'body': '<!-- sara-release:backend/v0.1.0 -->'}
        self.responses = {
            'git/ref/heads/main': {'object': {'sha': 'base'}},
            'git/commits/base': {'tree': {'sha': 'tree'}},
            'git/commits/merged': {'parents': [{'sha': 'base'}]},
            'actions/workflows/rust-qa.yml': {'id': 9},
            'issues/43': {'state': 'closed'},
            'actions/runs/7/jobs?per_page=100': {'jobs': [
                {'name': name, 'conclusion': 'success'} for name in r.QA_JOBS]},
            'compare/merged...main': {'status': 'identical'},
        }
        self.writes = []
        self.paths = list(r.FILES)

    def files(self, ref):
        return self.current if ref == 'merged' else self.old

    def pages(self, path):
        if path.startswith('commits/'):
            return [self.pr] if self.pr else []
        if path.startswith('pulls/2/files'):
            return [{'filename': path} for path in self.paths]
        return []

    def call(self, method, path, data=None, missing=False):
        if method == 'GET':
            return self.responses.get(path)
        self.writes.append((path, copy.deepcopy(data)))
        if path == 'git/trees':
            return {'sha': 'new-tree'}
        if path == 'git/commits':
            return {'sha': 'new-commit'}
        if path == 'git/refs':
            self.responses[path.replace('git/refs', 'git/ref') + '/' + data['ref'].removeprefix('refs/')] = {
                'object': {'type': 'commit', 'sha': data['sha']}}
        if path == 'releases':
            result = {'html_url': 'release-url', 'draft': False, 'prerelease': False}
            self.responses['releases/tags/backend%2Fv0.1.0'] = result
            return result
        return {'html_url': 'pr-url'}


def test_prepare_creates_only_release_files_and_dispatches_qa():
    api = FakeAPI()
    assert r.prepare(api, 'backend', 'initial', '2026-10-30') == 'pr-url'
    tree = next(data for path, data in api.writes if path == 'git/trees')
    assert {f['path'] for f in tree['tree']} == {r.FILES[2]}
    assert api.writes[-1] == ('actions/workflows/rust-qa.yml/dispatches', {'ref': 'release/backend/v0.1.0'})
    assert not any(path == 'releases' for path, _ in api.writes)


@pytest.mark.parametrize('condition', ['frontend', 'tag', 'branch'])
def test_prepare_rejects_unsupported_or_duplicate_requests_before_writes(condition):
    api = FakeAPI()
    if condition == 'tag':
        api.responses['git/ref/tags/backend/v0.1.0'] = {'object': {'sha': 'old'}}
    if condition == 'branch':
        api.responses['git/ref/heads/release/backend/v0.1.0'] = {'object': {'sha': 'old'}}
    with pytest.raises(r.ReleaseError):
        r.prepare(api, 'frontend' if condition == 'frontend' else 'backend', 'initial', '2026-10-30')
    assert not api.writes


def test_publish_uses_exact_merged_commit_and_is_idempotent():
    api = FakeAPI()
    assert r.publish(api, api.run) == 'release-url'
    assert ('git/refs', {'ref': 'refs/tags/backend/v0.1.0', 'sha': 'merged'}) in api.writes
    writes = copy.deepcopy(api.writes)
    assert r.publish(api, api.run) == 'release-url'
    assert api.writes == writes


@pytest.mark.parametrize('condition', [
    'qa_failed', 'pr_event', 'foreign_repo', 'wrong_branch', 'no_release_pr',
])
def test_non_release_runs_never_write(condition):
    api = FakeAPI()
    if condition == 'qa_failed': api.run['conclusion'] = 'failure'
    if condition == 'pr_event': api.run['event'] = 'pull_request'
    if condition == 'foreign_repo': api.run['head_repository']['full_name'] = 'other/repo'
    if condition == 'wrong_branch': api.run['head_branch'] = 'feature'
    if condition == 'no_release_pr': api.pr = None
    r.publish(api, api.run)
    assert not api.writes


@pytest.mark.parametrize('condition', [
    'open_acceptance', 'failed_job', 'wrong_workflow', 'extra_file', 'dependency_change',
    'wrong_tag', 'wrong_marker', 'not_on_main', 'bad_changelog', 'wrong_version',
])
def test_publish_rejects_invalid_release_without_writes(condition):
    api = FakeAPI()
    if condition == 'open_acceptance': api.responses['issues/43']['state'] = 'open'
    if condition == 'failed_job': api.responses['actions/runs/7/jobs?per_page=100']['jobs'][0]['conclusion'] = 'failure'
    if condition == 'wrong_workflow': api.run['workflow_id'] = 10
    if condition == 'extra_file': api.paths.append('.github/workflows/evil.yml')
    if condition == 'dependency_change': api.current[r.FILES[0]] += '\nevil = "1"\n'
    if condition == 'wrong_tag': api.responses['git/ref/tags/backend/v0.1.0'] = {'object': {'type': 'commit', 'sha': 'different'}}
    if condition == 'wrong_marker': api.pr['body'] = 'not a release request'
    if condition == 'not_on_main': api.responses['compare/merged...main']['status'] = 'diverged'
    if condition == 'bad_changelog': api.current[r.FILES[2]] += 'Unexpected change\n'
    if condition == 'wrong_version': api.current[r.FILES[0]] = api.current[r.FILES[0]].replace('0.1.0', '0.2.0')
    with pytest.raises(r.ReleaseError):
        r.publish(api, api.run)
    assert not api.writes


def test_real_manifest_and_lockfile_roundtrip():
    actual = {path: (ROOT / path).read_text() for path in r.FILES}
    changed = r.prepare_files(actual, '0.1.1', '2026-10-30')
    assert r.version_of(changed) == '0.1.1'
    reverted = r.version_files(changed, '0.1.0')
    assert all(reverted[path] == actual[path] for path in r.FILES[:2])


def test_first_release_cannot_skip_acceptance_by_requesting_patch():
    api = FakeAPI()
    with pytest.raises(r.ReleaseError, match='initial'):
        r.prepare(api, 'backend', 'patch', '2026-10-30')
    assert not api.writes


def test_followup_release_requires_published_ancestor_and_updates_versions():
    api = FakeAPI()
    api.responses['git/ref/tags/backend/v0.1.0'] = {'object': {'type': 'commit', 'sha': 'previous'}}
    api.responses['releases/tags/backend%2Fv0.1.0'] = {'draft': False, 'prerelease': False}
    api.responses['compare/previous...base'] = {'status': 'ahead'}
    assert r.prepare(api, 'backend', 'minor', '2026-10-30') == 'pr-url'
    tree = next(data for path, data in api.writes if path == 'git/trees')
    assert {f['path'] for f in tree['tree']} == set(r.FILES)
    assert all('0.2.0' in f['content'] for f in tree['tree'])


def test_partial_publication_reuses_matching_tag():
    api = FakeAPI()
    api.responses['git/ref/tags/backend/v0.1.0'] = {'object': {'type': 'commit', 'sha': 'merged'}}
    assert r.publish(api, api.run) == 'release-url'
    assert [path for path, _ in api.writes] == ['releases']


def test_failure_to_dispatch_keeps_existing_pr_and_does_not_publish():
    api = FakeAPI()
    call = api.call
    def failing(method, path, data=None, missing=False):
        if path.endswith('/dispatches'):
            raise r.ReleaseError('Dispatch nicht verfügbar.')
        return call(method, path, data, missing)
    api.call = failing
    with pytest.raises(r.ReleaseError):
        r.prepare(api, 'backend', 'initial', '2026-10-30')
    assert any(path == 'pulls' for path, _ in api.writes)
    assert not any(path == 'releases' for path, _ in api.writes)
