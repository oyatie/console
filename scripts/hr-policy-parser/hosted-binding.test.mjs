// TEST_ONLY hosting machinery. No Rust compiler, product target or native test runs.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { ROOT, SOURCE, hash, loadBinding, regular } from './binding.mjs';
import { candidate } from './native-run.mjs';

const owner = path.join(ROOT, 'scripts/hr-policy-parser/binding.mjs');
const node = fs.realpathSync(process.execPath);
const run = (file, args, options = {}) => spawnSync(file, args, {
  cwd: ROOT, encoding: 'utf8', timeout: 10_000, ...options,
});
const cleanResult = result => {
  assert.equal(result.status, 0); assert.equal(result.signal, null); assert.equal(result.error, undefined);
  return result.stdout.trim();
};
const git = fs.realpathSync(cleanResult(run('/usr/bin/which', ['git'])));
const python = cleanResult(run('python3', ['-B', '-I', '-c', 'import os,sys; print(os.path.realpath(sys.executable))']));
const sourcePaths = [
  'scripts/run-native-hr-policy-parser-contract.sh',
  'scripts/check-native-hr-policy-parser-contract.mjs',
  'scripts/check-native-hr-policy-parser-contract.test.mjs',
  'scripts/hr-policy-parser/binding.mjs', 'scripts/hr-policy-parser/native-run.mjs',
  'scripts/hr-policy-parser/observations.mjs', 'scripts/hr-policy-parser/qualify-admission.py',
  SOURCE, 'backend/crates/identity/application/tests/hr_policy_fixtures/literals.rs',
  'backend/crates/identity/application/tests/hr_policy_fixtures/assert_fields.rs',
  'tools/buck2', 'tools/buck/gen_first_party.py', 'backend/crates/identity/application/BUCK',
];
const host = process.platform === 'darwin' && process.arch === 'arm64' ? 'aarch64-apple-darwin'
  : process.platform === 'linux' && process.arch === 'x64' ? 'x86_64-unknown-linux-gnu'
  : process.platform === 'linux' && process.arch === 'arm64' ? 'aarch64-unknown-linux-gnu' : null;
assert(host, 'TEST_ONLY fixture needs a declared repository host');
const targets = [`toolchains//rust:sysroot-${host}`, 'toolchains//:cpython_archive', 'prelude//rust/tools:rustc_action'];

function pin(file) {
  assert(path.isAbsolute(file)); assert.equal(fs.realpathSync(file), file);
  fs.accessSync(file, fs.constants.X_OK);
  return { path: file, sha256: hash(regular(file)) };
}
function utility(name) {
  for (const directory of ['/usr/bin', '/bin']) {
    const lookup = path.join(directory, name);
    try {
      if (!fs.statSync(lookup).isFile()) continue;
      fs.accessSync(lookup, fs.constants.X_OK);
      const file = fs.realpathSync(lookup);
      return { lookup_path: lookup, ...pin(file) };
    } catch (error) {
      if (!['ENOENT', 'ENOTDIR', 'EACCES'].includes(error.code)) throw error;
    }
  }
  throw new Error(`TEST_ONLY prerequisite missing actual ${name}`);
}
function fixture() {
  const work = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'TEST_ONLY-hr-hosted-binding-')));
  const outputs = path.join(ROOT, 'buck-out', path.basename(work));
  fs.mkdirSync(outputs, { recursive: true });
  const cache = path.join(work, 'dotslash-cache'); fs.mkdirSync(cache);
  const config = path.join(work, 'TEST_ONLY-acquisition.json');
  const transcript = path.join(work, 'TEST_ONLY-acquisition.jsonl');
  const write = (file, bytes) => {
    fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, bytes, { mode: 0o755 });
    return file;
  };
  const compiler = write(path.join(outputs, 'sysroot/bin/rustc'), 'TEST_ONLY compiler bytes; never executable native proof\n');
  const actionPython = write(path.join(outputs, 'cpython_archive/bin/python3.13'), 'TEST_ONLY action Python bytes; never executed\n');
  fs.symlinkSync('python3.13', path.join(outputs, 'cpython_archive/bin/python'));
  const actionWrapper = write(path.join(outputs, 'rustc_action.py'), 'TEST_ONLY rustc_action bytes; never executed\n');
  const buck = path.join(cache, 'TEST_ONLY-selected-buck');
  const rows = targets.map((target, index) => ({ target, output: [path.dirname(path.dirname(compiler)), path.dirname(path.dirname(actionPython)), actionWrapper][index] }));
  const value = { buck, rows, transcript };
  const save = () => fs.writeFileSync(config, JSON.stringify(value)); save();
  write(buck, `#!${node}\n// TEST_ONLY finite declared-output CLI; no build or native execution.\n` +
    `const fs=require('node:fs'); const c=JSON.parse(fs.readFileSync(${JSON.stringify(config)})); const a=process.argv.slice(2);\n` +
    `fs.appendFileSync(c.transcript,JSON.stringify({tool:'buck',argv:a})+'\\n');\n` +
    `if(a.includes('kill'))process.exit(0);\n` +
    `const r=c.rows.filter(r=>a.includes(r.target)); if(!a.includes('build')||!a.includes('--show-output')||!a.includes('--no-remote-cache')||!a.includes('-j')||a[a.indexOf('-j')+1]!=='1'||!r.length)process.exit(126);\n` +
    `for(const x of r)process.stdout.write(x.target+' '+x.output+'\\n');\n`);
  const dotslash = write(path.join(work, 'bin/dotslash'), `#!${node}\n// TEST_ONLY finite fetch CLI; no download or native execution.\n` +
    `const fs=require('node:fs'); const c=JSON.parse(fs.readFileSync(${JSON.stringify(config)})); const a=process.argv.slice(2);\n` +
    `fs.appendFileSync(c.transcript,JSON.stringify({tool:'dotslash',argv:a})+'\\n');\n` +
    `if(JSON.stringify(a)!==JSON.stringify(['--','fetch','tools/buck2']))process.exit(126); process.stdout.write(c.buck+'\\n');\n`);
  const binding = {
    schema: 'console.hr-policy-parser.operational-binding.v1', status: 'MEASURED_CURRENT_CI_INPUTS',
    cwd: ROOT, probe: 'scripts/run-native-hr-policy-parser-contract.sh',
    contract_sha256: '95b9243256fa70e50bb645aa4c4cea400d0374e13fde2336378580344ffd832d',
    test_contract_sha256: '4b354e6c56fc3c2d06d36c488d9f7b3f3fc1f46fb57ba9e311bc5fdbd26f7d96',
    node: pin(node), python: pin(python), git: pin(git), dotslash: pin(dotslash), buck_binary: pin(buck),
    dotslash_cache: cache, bootstrap_utilities: { mkdir: utility('mkdir'), tar: utility('tar') },
    candidate_head: cleanResult(run(git, ['rev-parse', 'HEAD'])),
    candidate_tree: cleanResult(run(git, ['rev-parse', 'HEAD^{tree}'])), timeout_ms: 10_000,
    sources: sourcePaths.map(name => ({ path: name, sha256: hash(regular(path.join(ROOT, name))) })),
    compiler_sha256: hash(regular(compiler)), action_wrapper_sha256: [actionPython, actionWrapper].map(file => hash(regular(file))),
  };
  const control = path.join(work, 'TEST_ONLY-positive-control.json');
  fs.writeFileSync(control, JSON.stringify(binding), { mode: 0o600 });
  const env = { ...process.env, RUNNER_TEMP: work, DOTSLASH_CACHE: cache, GITHUB_SHA: binding.candidate_head,
    GITHUB_ENV: path.join(work, 'TEST_ONLY-github-env'), CONSOLE_HR_POLICY_NODE: node,
    CONSOLE_HR_POLICY_BINDING: control, CONSOLE_HR_POLICY_BINDING_SHA256: hash(regular(control)) };
  // Positive prerequisites use real files, the real loader and the actual clean Git candidate.
  assert.equal(loadBinding(env).status, 'MEASURED_CURRENT_CI_INPUTS'); candidate(binding);
  assert.equal(cleanResult(run(dotslash, ['--', 'fetch', 'tools/buck2'], { env })), buck);
  assert.equal(cleanResult(run(buck, ['build', '-j', '1', '--no-remote-cache', ...targets, '--show-output'], { env })),
    rows.map(row => `${row.target} ${row.output}`).join('\n'));
  fs.unlinkSync(transcript);
  return { work, outputs, cache, config, transcript, value, save, binding, env, compiler, actionPython, actionWrapper,
    output: path.join(work, 'TEST_ONLY-measured-binding.json'),
    close() { fs.rmSync(outputs, { recursive: true }); fs.rmSync(work, { recursive: true }); },
  };
}
function exportCurrent(f, extra = []) {
  return run(node, [owner, '--export-ci', '--output', f.output, '--python', python, '--git', git,
    '--dotslash', f.binding.dotslash.path, '--dotslash-cache', f.cache, '--timeout-ms', '10000', ...extra], { env: f.env });
}
function refused(f, result) {
  assert.equal(result.signal, null); assert.equal(result.error, undefined);
  assert.equal(result.status, 126, 'executed owner must refuse invalid measured custody');
  assert(!fs.existsSync(f.output), 'refusal cannot leave a usable binding');
  assert(!fs.existsSync(f.env.GITHUB_ENV), 'refusal cannot export launcher environment');
}

test('HOSTED binding prerequisites: real canonical files, declared TEST_ONLY outputs and clean Git round-trip', () => {
  const f = fixture(); try {
    assert.equal(fs.lstatSync(f.compiler).isFile(), true);
    assert.equal(fs.realpathSync(f.actionPython), f.actionPython);
    assert.notEqual(f.binding.compiler_sha256, '0'.repeat(64));
    assert.deepEqual(f.binding.action_wrapper_sha256, [f.actionPython, f.actionWrapper].map(file => hash(regular(file))));
  } finally { f.close(); }
});
test('HOSTED binding existing owner exports measured current files and exact launcher environment', () => {
  const f = fixture(); try {
    // TEST_ONLY stale metadata must not be copied or used as export authority.
    fs.writeFileSync(f.env.CONSOLE_HR_POLICY_BINDING, JSON.stringify({ ...f.binding,
      status: 'REVIEWED_EXACT_TEST_CANDIDATE', candidate_head: '0'.repeat(40), compiler_sha256: '0'.repeat(64),
      fixed_tool: { path: '/TEST_ONLY/obsolete-admission-tool' } }));
    cleanResult(exportCurrent(f));
    assert(fs.existsSync(f.output), 'executed binding owner must produce a measured current binding after positive prerequisites');
    assert.equal(fs.statSync(f.output).mode & 0o777, 0o600);
    const measured = JSON.parse(regular(f.output));
    for (const key of Object.keys(f.binding)) assert.deepEqual(measured[key], f.binding[key], key);
    for (const forbidden of ['fixed_tool', 'spec', 'spec_value', 'source_approval_pins']) assert(!Object.hasOwn(measured, forbidden));
    const emitted = Object.fromEntries(regular(f.env.GITHUB_ENV).toString().trim().split('\n').map(line => {
      const index = line.indexOf('='); assert(index > 0); return [line.slice(0, index), line.slice(index + 1)];
    }));
    assert.deepEqual(emitted, { CONSOLE_HR_POLICY_NODE: node, CONSOLE_HR_POLICY_BINDING: f.output,
      CONSOLE_HR_POLICY_BINDING_SHA256: hash(regular(f.output)) });
    const loaded = loadBinding({ ...f.env, ...emitted }); candidate(loaded);
    assert.equal(loaded.status, 'MEASURED_CURRENT_CI_INPUTS');
    const calls = regular(f.transcript).toString().trim().split('\n').map(line => JSON.parse(line));
    assert(calls.some(call => call.tool === 'dotslash'));
    assert.deepEqual([...new Set(calls.filter(call => call.tool === 'buck').flatMap(call => call.argv.filter(arg => targets.includes(arg))))].sort(), [...targets].sort());
    const qualification = run(python, [path.join(ROOT, 'scripts/hr-policy-parser/qualify-admission.py'),
      '--binding', f.output, '--binding-sha256', emitted.CONSOLE_HR_POLICY_BINDING_SHA256]);
    assert.equal(qualification.status, 126); assert.match(qualification.stderr, /not an independently reviewed exact candidate/);
  } finally { f.close(); }
});
test('HOSTED binding existing owner refuses dirty current candidate before export', () => {
  const f = fixture(); const dirty = path.join(ROOT, 'scripts/hr-policy-parser/TEST_ONLY-dirty-hosting-candidate');
  try {
    fs.writeFileSync(dirty, 'TEST_ONLY untracked source; must refuse measured clean candidate');
    assert(cleanResult(run(git, ['status', '--porcelain=v1', '--untracked-files=all'])).includes(path.basename(dirty)));
    refused(f, exportCurrent(f));
  } finally { fs.rmSync(dirty, { force: true }); f.close(); }
});
test('HOSTED binding existing owner refuses a selected executable outside owned DotSlash cache', () => {
  const f = fixture(); try {
    const outside = path.join(f.work, 'TEST_ONLY-unowned-buck'); fs.copyFileSync(f.binding.buck_binary.path, outside);
    assert.equal(fs.realpathSync(outside), outside); pin(outside);
    f.value.buck = outside; f.save(); refused(f, exportCurrent(f));
  } finally { f.close(); }
});
test('HOSTED binding existing owner refuses ambiguous declared Buck output', () => {
  const f = fixture(); try {
    f.value.rows.push({ ...f.value.rows[0] }); f.save();
    refused(f, exportCurrent(f));
  } finally { f.close(); }
});
test('HOSTED binding existing owner refuses declared output outside candidate buck-out', () => {
  const f = fixture(); try {
    const outside = path.join(f.work, 'TEST_ONLY-foreign-sysroot'); fs.mkdirSync(path.join(outside, 'bin'), { recursive: true });
    fs.copyFileSync(f.compiler, path.join(outside, 'bin/rustc')); pin(path.join(outside, 'bin/rustc'));
    f.value.rows[0].output = outside; f.save(); refused(f, exportCurrent(f));
  } finally { f.close(); }
});
test('HOSTED binding existing owner refuses nonregular compiler artifact', () => {
  const f = fixture(); try {
    const bytes = regular(f.compiler); fs.rmSync(f.compiler); const target = path.join(f.work, 'TEST_ONLY-foreign-rustc');
    fs.writeFileSync(target, bytes, { mode: 0o755 }); fs.symlinkSync(target, f.compiler);
    assert(fs.lstatSync(f.compiler).isSymbolicLink()); refused(f, exportCurrent(f));
  } finally { f.close(); }
});
test('HOSTED binding existing owner cannot promote measured metadata to reviewed admission', () => {
  const f = fixture(); try {
    refused(f, exportCurrent(f, ['--status', 'REVIEWED_EXACT_TEST_CANDIDATE']));
  } finally { f.close(); }
});
