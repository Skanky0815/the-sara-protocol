// Standard release mechanics belong to release-please; only project gates live here.
const {GitHub, Manifest} = require('release-please');
const fs = require('node:fs');
const assert = require('node:assert/strict');
const {parseCargoManifest} = require('release-please/build/src/updaters/rust/common');
const QA = ['Format', 'Clippy', 'Tests (PostgreSQL)', 'Dependency audit'];
const ALLOWED = new Set(['backend/Cargo.toml', 'backend/Cargo.lock', 'backend/CHANGELOG.md', '.release-please-manifest.json']);

function validateRun(run, repository, workflowId) {
  assert.equal(run.workflow_id, workflowId, 'Falscher QA-Workflow');
  assert.equal(run.conclusion, 'success', 'QA nicht erfolgreich');
  assert.equal(run.event, 'push', 'Kein main-Push');
  assert.equal(run.head_branch, 'main', 'Falscher Branch');
  assert.equal(run.head_repository?.full_name, repository, 'Fremdes Repository');
}

function selectCandidate(candidates, sha) {
  const matches = candidates.filter(c => c.path === 'backend' && c.sha === sha);
  assert(matches.length <= 1, 'Mehrdeutiger Release');
  if (!matches.length) return null;
  const candidate = matches[0];
  assert.match(candidate.tag.toString(), /^backend\/v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/, 'Ungültiger Backend-Tag');
  return candidate;
}

async function checkCandidate(candidate, run, api) {
  const pr = await api(`pulls/${candidate.pullRequest.number}`);
  assert(pr.head.repo.full_name === run.head_repository.full_name, 'Fremder Release-Branch');
  assert(pr.merged && pr.merge_commit_sha === run.head_sha && pr.base.ref === 'main', 'Release-PR nicht auf diesem Commit gemergt');
  assert(pr.changed_files <= ALLOWED.size, 'Zu viele Release-Dateien');
  const files = await api(`pulls/${pr.number}/files?per_page=100`);
  assert(files.length === pr.changed_files && files.every(f => ALLOWED.has(f.filename)), 'Unerwartete Release-Dateien');
  const jobs = await api(`actions/runs/${run.id}/jobs?per_page=100`);
  assert(jobs.total_count <= 100 && QA.every(name => jobs.jobs.some(j => j.name === name && j.conclusion === 'success')), 'QA-Jobs fehlen');
  const content = await api(`contents/backend/Cargo.toml?ref=${run.head_sha}`);
  const version = parseCargoManifest(Buffer.from(content.content, 'base64').toString()).package.version;
  assert.equal(version, candidate.tag.version.toString(), 'Release-Version stimmt nicht mit Cargo überein');
  const manifest = await api(`contents/.release-please-manifest.json?ref=${run.head_sha}`);
  assert.equal(JSON.parse(Buffer.from(manifest.content, 'base64').toString()).backend, version, 'Versionsmanifest widersprüchlich');
  assert.equal((await api('issues/43')).state, 'closed', 'Abnahme #43 ist noch offen');
  assert(['ahead', 'identical'].includes((await api(`compare/${run.head_sha}...main`)).status), 'Commit nicht in main');
}

async function apiClient(repository, token, path, method = 'GET', body, optional = false) {
  const response = await fetch(`https://api.github.com/repos/${repository}/${path}`, {
    method, headers: {Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'Content-Type': 'application/json', 'X-GitHub-Api-Version': '2022-11-28'},
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (optional && response.status === 404) return null;
  assert(response.ok, `GitHub-Anfrage fehlgeschlagen (HTTP ${response.status})`);
  return response.status === 204 ? null : response.json();
}

async function main() {
  const [command] = process.argv.slice(2);
  const repository = process.env.GITHUB_REPOSITORY;
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/);
  const [owner, repo] = repository.split('/');
  const api = (path, method, body, optional) => apiClient(repository, process.env.GH_TOKEN, path, method, body, optional);
  const github = await GitHub.create({owner, repo, token: process.env.GH_TOKEN, defaultBranch: 'main'});
  const manifest = await Manifest.fromManifest(github, 'main', undefined, undefined, undefined, 'backend');
  if (command === 'prepare') {
    assert.equal(process.env.GITHUB_EVENT_NAME, 'workflow_dispatch');
    assert.equal(process.env.GITHUB_REF, 'refs/heads/main');
    assert.equal(process.env.COMPONENT, 'backend', 'Frontend noch nicht vorhanden');
    for (const pr of (await manifest.createPullRequests()).filter(Boolean)) {
      await api(`issues/${pr.number}/assignees`, 'POST', {assignees: ['Sara-Pixel42']});
      const details = await api(`pulls/${pr.number}`);
      if (!details.requested_reviewers.some(u => u.login === 'Skanky0815')) {
        await api(`pulls/${pr.number}/requested_reviewers`, 'POST', {reviewers: ['Skanky0815']});
      }
      // Release Please owns branch creation. Existing branches cannot be natively linked retrospectively.
      const issue = await api('issues/44');
      const marker = '\n\n## Aktueller Versions-PR\n';
      const link = `- PR: #${pr.number}\n- Branch: https://github.com/${repository}/tree/${pr.headBranchName}\n`;
      await api('issues/44', 'PATCH', {body: issue.body.split(marker)[0] + marker + link});
      await api('actions/workflows/rust-qa.yml/dispatches', 'POST', {ref: pr.headBranchName});
      console.log(`Versions-PR #${pr.number}; Review und QA angefordert.`);
    }
    return;
  }
  assert(['plan', 'publish'].includes(command));
  assert.equal(process.env.GITHUB_EVENT_NAME, 'workflow_run');
  const event = JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const run = await api(`actions/runs/${event.workflow_run.id}`);
  validateRun(run, repository, (await api('actions/workflows/rust-qa.yml')).id);
  const candidate = selectCandidate(await manifest.buildReleases(), run.head_sha);
  if (!candidate) { console.log('Kein ausstehender Release für diesen QA-Commit.'); return; }
  await checkCandidate(candidate, run, api);
  const tag = candidate.tag.toString();
  const ref = await api(`git/ref/tags/${tag}`, 'GET', undefined, true);
  assert(!ref || (ref.object.type === 'commit' && ref.object.sha === run.head_sha), 'Tag darf nicht überschrieben werden');
  if (command === 'plan') {
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `version=${candidate.tag.version}\nsha=${run.head_sha}\n`);
    return;
  }
  // Versioned image promotion must have succeeded in the preceding step.
  assert.equal(process.env.PROMOTED_SHA, run.head_sha);
  const existing = await api(`releases/tags/${encodeURIComponent(tag)}`, 'GET', undefined, true);
  if (!existing) await github.createRelease(candidate, {draft: false, prerelease: false});
  else assert(!existing.draft && !existing.prerelease, 'Unerwarteter vorhandener Release');
  await github.removeIssueLabels(['autorelease: pending'], candidate.pullRequest.number);
  await github.addIssueLabels(['autorelease: tagged'], candidate.pullRequest.number);
  console.log(`Release ${tag} für ${run.head_sha} veröffentlicht.`);
}

module.exports = {validateRun, selectCandidate, checkCandidate};
if (require.main === module) main().catch(error => {
  // Do not print API response bodies or credentials.
  console.error(error instanceof assert.AssertionError ? error.message : 'Release-Integration fehlgeschlagen; API-Rechte und Konfiguration prüfen.');
  process.exitCode = 1;
});
