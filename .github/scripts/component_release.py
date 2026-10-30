"""Komponenten-Releases mit expliziten PR-, QA- und Versionsgrenzen."""
import argparse
import base64
from datetime import date
import json
import os
from pathlib import Path
import re
import sys
import tomllib
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import Request, urlopen

FILES = ('backend/Cargo.toml', 'backend/Cargo.lock', 'backend/CHANGELOG.md')
QA_JOBS = {'Format', 'Clippy', 'Tests (PostgreSQL)', 'Dependency audit'}
VERSION = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)'


class ReleaseError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise ReleaseError(message)


class GitHub:
    def __init__(self, repo, token):
        require(re.fullmatch(r'[\w.-]+/[\w.-]+', repo), 'Ungültiges Repository.')
        self.repo = repo
        self.token = token

    def call(self, method, path, data=None, missing=False):
        request = Request(
            f'https://api.github.com/repos/{self.repo}/{path}',
            data=json.dumps(data).encode() if data is not None else None,
            headers={'Authorization': f'Bearer {self.token}',
                     'Accept': 'application/vnd.github+json',
                     'X-GitHub-Api-Version': '2022-11-28'}, method=method,
        )
        try:
            with urlopen(request, timeout=30) as response:
                raw = response.read()
                return json.loads(raw) if raw else None
        except HTTPError as error:
            if missing and error.code == 404:
                return None
            # Never echo response bodies, credentials or supplied values.
            raise ReleaseError(f'GitHub-Anfrage fehlgeschlagen (HTTP {error.code}).') from None

    def pages(self, path):
        result = []
        for page in range(1, 101):
            separator = '&' if '?' in path else '?'
            items = self.call('GET', f'{path}{separator}per_page=100&page={page}')
            result.extend(items)
            if len(items) < 100:
                return result
        raise ReleaseError('Zu viele API-Ergebnisse.')

    def files(self, ref):
        result = {}
        for path in FILES:
            item = self.call('GET', f'contents/{path}?ref={quote(ref, safe="")}')
            result[path] = base64.b64decode(item['content']).decode()
        return result


def version_of(files):
    manifest = tomllib.loads(files[FILES[0]])['package']
    require(manifest['name'] == 'the-sara-protocol', 'Unerwartetes Backend-Paket.')
    version = manifest['version']
    require(re.fullmatch(VERSION, version), 'Nur stabile numerische Versionen werden unterstützt.')
    lock = tomllib.loads(files[FILES[1]])
    packages = [p for p in lock['package'] if p['name'] == manifest['name']]
    require(len(packages) == 1 and packages[0]['version'] == version,
            'Manifest und Lockfile stimmen nicht überein.')
    return version


def next_version(current, bump):
    require(re.fullmatch(VERSION, current), 'Ungültige Ausgangsversion.')
    major, minor, patch = map(int, current.split('.'))
    if bump == 'initial':
        require(current == '0.1.0', 'Initial ist nur für 0.1.0 vorgesehen.')
        return current
    require(bump in ('major', 'minor', 'patch'), 'Ungültiger Versionssprung.')
    if bump == 'major':
        return f'{major + 1}.0.0'
    if bump == 'minor':
        return f'{major}.{minor + 1}.0'
    return f'{major}.{minor}.{patch + 1}'


def version_files(files, target):
    current = version_of(files)
    require(re.fullmatch(VERSION, target), 'Ungültige Zielversion.')
    result = dict(files)
    # Replace only the version in the package section, never dependency versions.
    for path in FILES[:2]:
        section = r'\[package\]' if path.endswith('.toml') else r'\[\[package\]\]'
        blocks = re.split(f'(?m)(?=^{section}$)', files[path])
        count = 0
        for index, block in enumerate(blocks):
            if re.search(r'^name = "the-sara-protocol"$', block, re.M):
                blocks[index], changed = re.subn(
                    rf'^version = "{re.escape(current)}"$',
                    f'version = "{target}"', block, count=1, flags=re.M)
                count += changed
        require(count == 1, 'Paketversion konnte nicht eindeutig aktualisiert werden.')
        result[path] = ''.join(blocks)
    require(version_of(result) == target, 'Versionsaktualisierung fehlgeschlagen.')
    return result


def prepare_files(files, target, today):
    result = version_files(files, target)
    changelog = files[FILES[2]]
    require(f'## [{target}]' not in changelog, 'Version ist bereits im Changelog enthalten.')
    marker = '## [Unveröffentlicht]\n'
    require(changelog.count(marker) == 1, 'Changelog benötigt genau einen unveröffentlichten Abschnitt.')
    prefix, rest = changelog.split(marker)
    sections = re.split(r'(?m)(?=^## \[)', rest, maxsplit=1)
    notes = sections[0].strip()
    require(notes and notes != '_Noch keine Änderungen._', 'Keine Release-Notizen vorhanden.')
    history = sections[1] if len(sections) > 1 else ''
    result[FILES[2]] = (prefix + marker + '\n_Noch keine Änderungen._\n\n'
                        + f'## [{target}] – {today}\n\n' + notes + '\n\n' + history)
    return result



def require_previous_release(api, version, base):
    tag = f'backend/v{version}'
    ref = api.call('GET', f'git/ref/tags/{tag}', missing=True)
    release = api.call('GET', 'releases/tags/' + quote(tag, safe=''), missing=True)
    require(ref and release and not release['draft'] and not release['prerelease'],
            'Zuerst die aktuelle Version veröffentlichen; erster Auftrag muss initial sein.')
    require(ref['object']['type'] == 'commit', 'Unerwarteter Vorgänger-Tag.')
    comparison = api.call('GET', f'compare/{ref["object"]["sha"]}...{base}')
    require(comparison['status'] in ('ahead', 'identical'), 'Vorgänger-Release gehört nicht zur Historie.')


def prepare(api, component, bump, today):
    require(component == 'backend', 'Frontend ist noch nicht vorhanden; keine ausführbare Frontend-Freigabe.')
    base = api.call('GET', 'git/ref/heads/main')['object']['sha']
    files = api.files(base)
    target = next_version(version_of(files), bump)
    if bump != "initial":
        require_previous_release(api, version_of(files), base)
    tag = f'backend/v{target}'
    branch = f'release/{tag}'
    require(not api.call('GET', f'git/ref/tags/{tag}', missing=True), 'Tag existiert bereits.')
    require(not api.call('GET', f'git/ref/heads/{branch}', missing=True),
            'Release-Branch existiert bereits. Vorhandenen PR prüfen; QA bei Bedarf manuell erneut starten.')
    require(not api.pages('pulls?state=open&base=main&head=' + quote(api.repo.split('/')[0] + ':' + branch, safe='')),
            'Versions-PR existiert bereits.')
    updated = prepare_files(files, target, today)
    tree = api.call('POST', 'git/trees', {
        'base_tree': api.call('GET', f'git/commits/{base}')['tree']['sha'],
        'tree': [{'path': path, 'mode': '100644', 'type': 'blob', 'content': updated[path]}
                 for path in FILES if updated[path] != files[path]],
    })['sha']
    commit = api.call('POST', 'git/commits', {
        'message': f'chore: release {tag}', 'tree': tree, 'parents': [base],
    })['sha']
    api.call('POST', 'git/refs', {'ref': f'refs/heads/{branch}', 'sha': commit})
    pr = api.call('POST', 'pulls', {
        'title': f'Release: {tag}', 'head': branch, 'base': 'main',
        'body': (f'<!-- sara-release:{tag} -->\n'
                 f'Bewusster Release-Auftrag für `{tag}`. Version und Changelog prüfen; '
                 'kein automatischer Merge. Veröffentlichung erst nach erfolgreicher Rust-QA '
                 'auf dem Merge-Commit. Für 0.1.0 muss #43 abgeschlossen sein.\n\n'
                 'Refs #44\nhttps://github.com/' + api.repo + '/issues/44'),
    })
    # Explicit dispatch bypasses suppression/approval gating of bot-created PR events.
    api.call('POST', 'actions/workflows/rust-qa.yml/dispatches', {'ref': branch})
    return pr['html_url']


def release_candidate(api, run):
    if not (run.get('conclusion') == 'success' and run.get('event') == 'push'
            and run.get('head_branch') == 'main'
            and run.get('head_repository', {}).get('full_name') == api.repo):
        return None
    workflow = api.call('GET', 'actions/workflows/rust-qa.yml')
    require(run['workflow_id'] == workflow['id'], 'Unerwarteter QA-Workflow.')
    sha = run['head_sha']
    prs = api.pages(f'commits/{sha}/pulls')
    candidates = [p for p in prs if p.get('merged_at') and p.get('merge_commit_sha') == sha
                  and p.get('base', {}).get('ref') == 'main'
                  and p.get('head', {}).get('repo', {}).get('full_name') == api.repo
                  and re.fullmatch(r'release/backend/v' + VERSION, p['head']['ref'])]
    if not candidates:
        return None
    require(len(candidates) == 1, 'Release-PR ist nicht eindeutig.')
    pr = candidates[0]
    target = pr['head']['ref'].removeprefix('release/backend/v')
    tag = f'backend/v{target}'
    require(f'<!-- sara-release:{tag} -->' in (pr.get('body') or ''), 'Release-Auftrag fehlt.')
    changed = api.pages(f'pulls/{pr["number"]}/files')
    paths = {f['filename'] for f in changed}
    require(FILES[2] in paths and paths <= set(FILES), 'Release-PR enthält unerwartete Dateien.')
    commit = api.call('GET', f'git/commits/{sha}')
    require(len(commit['parents']) == 1, 'Release benötigt lineare Historie.')
    parent = commit['parents'][0]['sha']
    old = api.files(parent)
    current = api.files(sha)
    require(version_of(current) == target, 'Tag und Paketversion stimmen nicht überein.')
    require(target in {next_version(version_of(old), b) for b in ('major', 'minor', 'patch')}
            or target == version_of(old) == '0.1.0', 'Unerwarteter Versionssprung.')
    expected = version_files(old, target)
    require(all(current[p] == expected[p] for p in FILES[:2]),
            'Release darf keine Abhängigkeiten oder weitere Manifestfelder ändern.')
    match = re.search(rf'^## \[{re.escape(target)}\] – (\d{{4}}-\d{{2}}-\d{{2}})\n', current[FILES[2]], re.M)
    require(match is not None, 'Release-Abschnitt fehlt.')
    require(current == prepare_files(old, target, match[1]), 'Unerwartete Changelog-Änderung.')
    if target != version_of(old):
        require_previous_release(api, version_of(old), parent)
    if target == '0.1.0':
        require(api.call('GET', 'issues/43')['state'] == 'closed', 'Abnahme #43 ist noch offen.')
    jobs = api.call('GET', f'actions/runs/{run["id"]}/jobs?per_page=100')['jobs']
    require(all(any(j['name'] == name and j['conclusion'] == 'success' for j in jobs)
                for name in QA_JOBS), 'Erforderliche QA-Jobs sind nicht erfolgreich.')
    comparison = api.call('GET', f'compare/{sha}...main')
    require(comparison['status'] in ('ahead', 'identical'), 'Release-Commit gehört nicht zu main.')
    notes = current[FILES[2]].split(match[0], 1)[1].split('\n## [', 1)[0].strip()
    return tag, sha, notes


def publish(api, run):
    candidate = release_candidate(api, run)
    if candidate is None:
        return 'Kein Release-Auftrag für diesen Lauf.'
    tag, sha, notes = candidate
    ref = api.call('GET', f'git/ref/tags/{tag}', missing=True)
    if ref:
        require(ref['object']['type'] == 'commit' and ref['object']['sha'] == sha,
                'Vorhandener Tag zeigt auf einen anderen Stand; keine Überschreibung.')
    else:
        api.call('POST', 'git/refs', {'ref': f'refs/tags/{tag}', 'sha': sha})
    release = api.call('GET', 'releases/tags/' + quote(tag, safe=''), missing=True)
    if release:
        require(not release['draft'] and not release['prerelease'], 'Unerwarteter bestehender Release.')
        return release['html_url']
    return api.call('POST', 'releases', {
        'tag_name': tag, 'target_commitish': sha, 'name': tag, 'body': notes,
        'draft': False, 'prerelease': False, 'make_latest': 'false',
    })['html_url']


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=['prepare', 'publish'])
    args = parser.parse_args()
    api = GitHub(os.environ['GITHUB_REPOSITORY'], os.environ['GH_TOKEN'])
    event = json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text())
    if args.command == 'prepare':
        require(os.environ['GITHUB_EVENT_NAME'] == 'workflow_dispatch'
                and os.environ['GITHUB_REF'] == 'refs/heads/main', 'Start nur manuell auf main.')
        result = prepare(api, os.environ['COMPONENT'], os.environ['BUMP'], date.today().isoformat())
    else:
        require(os.environ['GITHUB_EVENT_NAME'] == 'workflow_run', 'Unerwarteter Auslöser.')
        # Fetch current API state rather than trusting a stale event conclusion.
        run = api.call('GET', f'actions/runs/{int(event["workflow_run"]["id"])}')
        result = publish(api, run)
    print(result)


if __name__ == '__main__':
    try:
        main()
    except ReleaseError as error:
        print(f"Release abgebrochen: {error}", file=sys.stderr)
        sys.exit(1)
    except (KeyError, ValueError, OSError):
        # Workflow logs expose only fixed, non-sensitive diagnostics.
        print('Release abgebrochen: Eingaben, Voraussetzungen und API-Berechtigungen prüfen.', file=sys.stderr)
        sys.exit(1)
