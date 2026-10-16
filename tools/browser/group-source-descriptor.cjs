'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {execFileSync} = require('node:child_process');

assert.equal(process.argv.length, 4, 'canonical checkout and fresh destination required');
const [root, destination] = process.argv.slice(2);
function canonical(value, directory) {
  assert.ok(value && !/[\r\n\0]/.test(value) && path.isAbsolute(value));
  assert.notEqual(value, path.parse(value).root);
  assert.equal(path.resolve(value), value);
  let current = value;
  for (;;) {
    assert.ok(!fs.lstatSync(current).isSymbolicLink(), 'symlink in source custody');
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  assert.equal(fs.realpathSync(value), value);
  const stat = fs.lstatSync(value);
  assert.ok(directory ? stat.isDirectory() : stat.isFile());
  if (!directory) assert.ok(stat.size > 0 && stat.size <= 1048576, 'bounded regular source required');
}
canonical(root, true);
assert.ok(destination && !/[\r\n\0]/.test(destination) && path.isAbsolute(destination));
assert.equal(path.resolve(destination), destination);
canonical(path.dirname(destination), true);
const stage = path.dirname(destination), stageStat = fs.lstatSync(stage);
assert.equal(path.basename(destination), 'group-source');
assert.match(path.basename(stage), /^console-native-browser\.[A-Za-z0-9]+$/);
assert.equal(stageStat.mode & 0o077, 0, 'private installer stage required');
if (process.getuid) assert.equal(stageStat.uid, process.getuid());
try { fs.lstatSync(destination); throw Error('fresh descriptor destination required'); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
const gitEnv = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('GIT_')));
Object.assign(gitEnv, {GIT_CONFIG_GLOBAL:'/dev/null', GIT_CONFIG_SYSTEM:'/dev/null',
  GIT_CONFIG_NOSYSTEM:'1', GIT_CONFIG_COUNT:'0', GIT_TERMINAL_PROMPT:'0', GIT_NO_REPLACE_OBJECTS:'1'});
const git = (...args) => execFileSync('git', ['-C', root, ...args], {env:gitEnv, maxBuffer:1048576});
assert.equal(git('rev-parse', '--show-toplevel').toString().trim(), root);
const candidate = git('rev-parse', '--verify', 'HEAD^{commit}').toString().trim();
assert.match(candidate, /^[0-9a-f]{40}$/);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function tracked(relative, expected) {
  const file = path.join(root, relative);
  canonical(file, false);
  const bytes = fs.readFileSync(file), object = candidate + ':' + relative;
  assert.equal(git('cat-file', '-t', object).toString().trim(), 'blob');
  const blob = git('rev-parse', object).toString().trim();
  assert.match(blob, /^[0-9a-f]{40}$/);
  assert.deepEqual(bytes, git('show', object), 'dirty authoritative source');
  if (expected) assert.equal(sha(bytes), expected, 'reviewed source identity changed');
  return {relative, bytes, blob, digest:sha(bytes)};
}
const sources = [
  ['schema_digest', 'backend/crates/platform/authz/src/group_process/process-v1.cedarschema', 'process-v1.cedarschema', 'c711017368596094ad0ff9b1123eb17573722df47f1b4abb15a31ce1ce8b1e44'],
  ['policy_digest', 'backend/crates/platform/authz/src/group_process/process-v1.cedar', 'process-v1.cedar', '2453684b70134124a8cf77f2d882fb7497d8c1498325fac1321ca7e616791ed8'],
  ['codec_contract_digest', 'ops/native-group-process/codec-contract-v1.json', 'codec-contract-v1.json', '595376f9edea8ebd5a698bd27d6f7310a470f5e5d95522d7eed7c01db8169067'],
].map(([key, relative, name, expected]) => ({key, name, ...tracked(relative, expected)}));
const metadata = tracked('ops/native-group-process/source-v1.sql', '6cda12aa3051768db0418b0b0aecb82a590ddfac0c753ef01d07e027720beea7').bytes.toString('utf8');
const engine = tracked('backend/crates/platform/authz/src/cedar_pbac/engine.rs').bytes.toString('utf8');
const literals = [...metadata.matchAll(/\$actions\$([^]*?)\$actions\$/g)];
assert.equal(literals.length, 1, 'one SQL-owned action literal required');
const actions = JSON.parse(literals[0][1]);
assert.ok(Array.isArray(actions) && actions.length === 4);
assert.equal(new Set(actions.map(action => action.key)).size, 4);
assert.equal(new Set(actions.map(action => action.action_id)).size, 4);
for (const action of actions) {
  assert.deepEqual(Object.keys(action).sort(), ['action_id', 'fields', 'key', 'revision']);
  assert.match(action.key, /^identity\.verifier\.process\.(adopt|suspend|read|receipt\.read-own)\/1$/);
  assert.match(action.action_id, /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  assert.equal(action.revision, 1);
  assert.ok(Array.isArray(action.fields) && action.fields.length > 0 && action.fields.length <= 32);
  assert.equal(new Set(action.fields).size, action.fields.length);
  for (const field of action.fields) assert.match(field, /^[a-z][a-z0-9_]{0,99}$/);
}
for (const {key, digest} of sources) {
  const sqlName = key === 'codec_contract_digest' ? 'codec_digest' : key;
  assert.ok(metadata.includes(sqlName + " bytea:=decode('" + digest + "','hex')"));
}
assert.ok(metadata.includes("'schema_id','native-group-process-v1'"));
assert.ok(metadata.includes("'registration_manifest_version',1"));
for (const [key, constant, version] of [['cedar_sdk_version','CEDAR_SDK_VERSION','4.13.0'], ['cedar_language_version','CEDAR_LANGUAGE_VERSION','4.5']]) {
  assert.ok(metadata.includes("'" + key + "','" + version + "'"));
  const declarations = [...engine.matchAll(new RegExp('^pub const ' + constant + ': &str = "([^"\\n]+)";$', 'gm'))];
  assert.equal(declarations.length, 1, 'one executable Cedar version constant required');
  assert.equal(declarations[0][1], version);
}
fs.mkdirSync(destination, {mode:0o700});
const stagedSources = path.join(destination, 'sources');
fs.mkdirSync(stagedSources, {mode:0o700});
const records = sources.map(source => {
  const staged = path.join(stagedSources, source.name);
  fs.writeFileSync(staged, source.bytes, {flag:'wx', mode:0o600});
  canonical(staged, false);
  assert.deepEqual(fs.readFileSync(staged), source.bytes);
  return {key:source.key, repository_path:source.relative, staged_path:staged, git_blob:source.blob, sha256:source.digest};
});
assert.equal(git('rev-parse', '--verify', 'HEAD^{commit}').toString().trim(), candidate, 'candidate moved');
const descriptor = {schema:'console.native_group_process_installed_test_source.v1',
  schema_id:'native-group-process-v1', source_candidate:candidate, git_root:root,
  sources:records, ...Object.fromEntries(sources.map(source => [source.key, '\\x' + source.digest])),
  cedar_sdk_version:'4.13.0', cedar_language_version:'4.5', registration_manifest_version:1,
  registered_actions:actions};
const descriptorPath = path.join(destination, 'descriptor.json');
const bytes = Buffer.from(JSON.stringify(descriptor, null, 2) + '\n');
fs.writeFileSync(descriptorPath, bytes, {flag:'wx', mode:0o600});
assert.deepEqual(fs.readFileSync(descriptorPath), bytes);
