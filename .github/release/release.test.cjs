const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {validateRun, selectCandidate, checkCandidate} = require('./release.cjs');
const {Rust} = require('release-please/build/src/strategies/rust');
const {DefaultVersioningStrategy} = require('release-please/build/src/versioning-strategies/default');
const {Version} = require('release-please/build/src/version');
const {TagName} = require('release-please/build/src/util/tag-name');
const {parseConventionalCommits} = require('release-please/build/src/commit');
const root = path.resolve(__dirname, '../..');
const config = JSON.parse(fs.readFileSync(path.join(root,'release-please-config.json')));
const run = {id:7, workflow_id:9, conclusion:'success', event:'push', head_branch:'main', head_sha:'merged', head_repository:{full_name:'owner/repo'}};
const candidate = () => ({path:'backend', sha:'merged', tag:new TagName(Version.parse('0.1.0'),'backend','/'), pullRequest:{number:2}});
const content = value => ({content:Buffer.from(value).toString('base64')});
const responses = () => ({
 'pulls/2':{number:2,merged:true,merge_commit_sha:'merged',base:{ref:'main'},head:{repo:{full_name:'owner/repo'}},changed_files:2},
 'pulls/2/files?per_page=100':[{filename:'backend/CHANGELOG.md'},{filename:'.release-please-manifest.json'}],
 'actions/runs/7/jobs?per_page=100':{total_count:4,jobs:['Format','Clippy','Tests (PostgreSQL)','Dependency audit'].map(name=>({name,conclusion:'success'}))},
 'contents/backend/Cargo.toml?ref=merged':content('[package]\nname="the-sara-protocol"\nversion="0.1.0"'),
 'contents/.release-please-manifest.json?ref=merged':content('{"backend":"0.1.0"}'),
 'issues/43':{state:'closed'},'compare/merged...main':{status:'ahead'},
});

test('nur derselbe main-QA-Lauf aus diesem Repository ist zulässig',()=>{
 validateRun(run,'owner/repo',9);
 for(const patch of [{conclusion:'failure'},{event:'pull_request'},{head_branch:'other'},{workflow_id:10},{head_repository:{full_name:'foreign/repo'}}]){
  assert.throws(()=>validateRun({...run,...patch},'owner/repo',9));
 }
});
test('keine alten oder anderen Komponenten veröffentlichen',()=>{
 assert.equal(selectCandidate([candidate()],'merged').sha,'merged');
 assert.equal(selectCandidate([candidate()],'newer'),null);
 assert.equal(selectCandidate([{...candidate(),path:'frontend'}],'merged'),null);
 assert.throws(()=>selectCandidate([candidate(),candidate()],'merged'));
});
test('gemergter Release, QA, Versionsdateien und Abnahme müssen stimmen',async()=>{
 const ok=responses();await checkCandidate(candidate(),run,async p=>ok[p]);
 for(const mutate of [
  r=>r['issues/43'].state='open',
  r=>r['pulls/2'].merged=false,
  r=>r['pulls/2'].merge_commit_sha='other',
  r=>r['pulls/2'].head.repo.full_name='foreign/repo',
  r=>r['pulls/2/files?per_page=100'][0].filename='backend/src/lib.rs',
  r=>r['actions/runs/7/jobs?per_page=100'].jobs[0].conclusion='failure',
  r=>r['contents/.release-please-manifest.json?ref=merged']=content('{"backend":"0.2.0"}'),
  r=>r['compare/merged...main'].status='diverged',
 ]){const r=responses();mutate(r);await assert.rejects(checkCandidate(candidate(),run,async p=>r[p]));}
});

test('Release Please aktualisiert echte Cargo-Dateien und unsere 0.x-Versionen',async()=>{
 const github={repository:{owner:'owner',repo:'repo'},getFileJson:async p=>JSON.parse(fs.readFileSync(path.join(root,p),'utf8')),getFileContentsOnBranch:async p=>{
  const parsedContent=fs.readFileSync(path.join(root,p),'utf8');return {parsedContent,content:Buffer.from(parsedContent).toString('base64'),sha:'fixture'};
 }};
 const opts={github,targetBranch:'main',path:'backend',component:'backend',includeComponentInTag:config['include-component-in-tag'],tagSeparator:config['tag-separator'],bumpMinorPreMajor:config['bump-minor-pre-major'],bumpPatchForMinorPreMajor:config['bump-patch-for-minor-pre-major']};
 opts.versioningStrategy=new DefaultVersioningStrategy(opts);
 const commits=message=>parseConventionalCommits([{sha:'abc123',message,files:['backend/src/lib.rs']}]);
 const first=await new Rust(opts).buildReleasePullRequest(commits('feat: initial API'));
 assert.equal(first.version.toString(),'0.1.0');
 const prev={tag:new TagName(Version.parse('0.1.0'),'backend','/'),sha:'old',notes:'old'};
 const next=await new Rust(opts).buildReleasePullRequest(commits('feat: compatible feature'),prev);
 assert.equal(next.version.toString(),'0.1.1');
 assert.equal(new TagName(next.version,'backend',opts.tagSeparator).toString(),'backend/v0.1.1');
 assert.deepEqual(next.updates.map(u=>u.path).sort(),['backend/CHANGELOG.md','backend/Cargo.lock','backend/Cargo.toml']);
 for(const update of next.updates.filter(u=>u.path.endsWith('.toml')||u.path.endsWith('.lock'))){
  const old=fs.readFileSync(path.join(root,update.path),'utf8');const updated=update.updater.updateContent(old);
  assert.match(updated,/version = "0.1.1"/);
  // Reversing just this package update must restore every byte, including dependencies.
  update.updater.versionsMap.set('the-sara-protocol',Version.parse('0.1.0'));
  if(update.updater.version) update.updater.version=Version.parse('0.1.0');
  assert.equal(update.updater.updateContent(updated),old);
 }
 assert.equal(opts.versioningStrategy.bump(Version.parse('0.1.0'),commits('feat!: breaking')).toString(),'0.2.0');
 assert.equal(opts.versioningStrategy.bump(Version.parse('1.0.0'),commits('feat!: breaking')).toString(),'2.0.0');
 assert.equal(config.packages.backend['release-type'],'rust');
 assert.deepEqual(Object.keys(config.packages),['backend']);
});

test('Manifest-Konfiguration wird von Release Please selbst korrekt geladen', async()=>{
 const {Manifest}=require('release-please');
 const github={repository:{owner:'owner',repo:'repo'},getFileJson:async p=>JSON.parse(fs.readFileSync(path.join(root,p),'utf8')),getFileContentsOnBranch:async p=>{
  const parsedContent=fs.readFileSync(path.join(root,p),'utf8');return {parsedContent,content:Buffer.from(parsedContent).toString('base64'),sha:'fixture'};
 }};
 const manifest=await Manifest.fromManifest(github,'main');
 assert.equal(manifest.repositoryConfig.backend.releaseType,'rust');
 assert.equal(manifest.repositoryConfig.backend.tagSeparator,'/');
 assert.equal(manifest.repositoryConfig.backend.bumpMinorPreMajor,true);
 assert.equal(manifest.repositoryConfig.backend.bumpPatchForMinorPreMajor,true);
 assert.equal(manifest.repositoryConfig.backend.initialVersion,'0.1.0');
 assert.deepEqual(manifest.releasedVersions,{});
});
