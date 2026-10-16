import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, mkdirSync, rmSync, readFileSync, existsSync, realpathSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { gitFixtureEnvironment } from "./lib/git-fixture-environment.mjs";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");

test("live candidate passes js-test-reachability", () => {
  const r = spawnSync(process.execPath, ["scripts/check-js-test-reachability.mjs", "--json"], {
    cwd: repo,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(r.status, 0, r.stderr || r.stdout);
  const report = JSON.parse(r.stdout);
  assert.equal(report.relation, "js_test_reachability");
  assert.ok(report.suite_count >= 20);
  assert.equal(report.failures.length, 0);
  // basename-only must not be treated as wired
  for (const s of report.wired_exact) {
    assert.ok(s.endsWith(".test.mjs"));
  }
});

// Real finite Git/Node fixtures; never run a product service or native compiler.
function hermeticFixture(run = 'true', suites = ['fixture/new.test.mjs'], guard = '') {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'js-reachability-control-')));
  const env = {...gitFixtureEnvironment(), NODE_OPTIONS:'', NODE_PATH:''};
  delete env.NODE_TEST_CONTEXT;
  const write = (name, value, mode = 0o644) => {
    const file = join(root, name); mkdirSync(dirname(file), {recursive:true});
    writeFileSync(file, value, {mode}); return file;
  };
  for (const name of ['scripts/check-js-test-reachability.mjs',
    'scripts/lib/ci-workflow-executables.mjs', 'tools/ci/gate-sweep.mjs'])
    write(name, readFileSync(join(repo, name)));
  write('package.json', JSON.stringify({type:'module', scripts:{}}));
  write('docs/program/js-test-reachability-baseline.json', JSON.stringify({schema_version:1, deferred_tests:[]}));
  for (const suite of suites) write(suite, "import test from 'node:test'; test('fixture', () => {});\n");
  write('.github/workflows/fixture.yml', 'name: fixture\non: pull_request\njobs:\n  verify:\n    runs-on: ubuntu-latest\n    steps:\n      - name: actual proof\n' + guard +
    '        run: |\n' + run.split('\n').map(line => '          ' + line).join('\n') + '\n');
  const execute = (command, args, extra = {}) => spawnSync(command, args,
    {cwd:root, env:{...env, ...extra}, encoding:'utf8', timeout:10000, maxBuffer:1024*1024});
  const git = args => {const result = execute('git', args); assert.equal(result.status, 0, result.stderr);};
  git(['init', '--quiet']); git(['add', '.']);
  const gate = () => {
    git(['add', '.']);
    const result = execute(process.execPath, ['scripts/check-js-test-reachability.mjs', '--json']);
    assert.equal(result.signal, null, result.stderr); assert(!result.error, result.error?.message);
    return {...result, report:JSON.parse(result.stdout)};
  };
  return {root, write, execute, gate, cleanup:() => rmSync(root, {recursive:true})};
}
function assertDark(result, suite = 'fixture/new.test.mjs') {
  assert.equal(result.status, 1);
  assert(result.report.dark.includes(suite)); assert(!result.report.wired_exact.includes(suite));
  assert(result.report.failures.some(failure => failure.endsWith(suite)));
}
test('baseline growth fails (unregistered dark suite)', () => {
  const fixture = hermeticFixture('node --test fixture/control.test.mjs',
    ['fixture/control.test.mjs', 'fixture/new.test.mjs']);
  try {assertDark(fixture.gate());} finally {fixture.cleanup();}
});
test('hermetic reachability accepts an actual direct Node test command', () => {
  const fixture = hermeticFixture('node --test fixture/new.test.mjs');
  try {const result = fixture.gate(); assert.equal(result.status, 0); assert.deepEqual(result.report.wired_exact, ['fixture/new.test.mjs']);}
  finally {fixture.cleanup();}
});
for (const [label, run, guard] of [
  ['comment-only', '# node --test fixture/new.test.mjs\ntrue', ''],
  ['inert echo', 'echo node --test fixture/new.test.mjs', ''],
  ['disabled step', 'node --test fixture/new.test.mjs', '        if: false\n'],
]) test(`hermetic reachability refuses ${label} suite references`, () => {
  const fixture = hermeticFixture(run, undefined, guard);
  try {assertDark(fixture.gate());} finally {fixture.cleanup();}
});
for (const enabled of [true, false])
test(`hermetic reachability ${enabled ? 'follows an executed' : 'refuses a commented'} supervised child`, () => {
  const suite = 'scripts/check-native-hr-policy-parser-contract.test.mjs';
  const fixture = hermeticFixture('scripts/run-native-hr-policy-parser-contract.sh', [suite]);
  try {
    fixture.write('scripts/run-native-hr-policy-parser-contract.sh',
      readFileSync(join(repo, 'scripts/run-native-hr-policy-parser-contract.sh')), 0o755);
    fixture.write(suite, "import test from 'node:test'; import fs from 'node:fs'; test('real fixture child', () => fs.writeFileSync(process.env.TEST_ONLY_CHILD_MARKER, 'executed'));\n");
    const call = "const child = spawn(binding.node.path, ['--test', 'scripts/check-native-hr-policy-parser-contract.test.mjs'], {stdio:'inherit', env:{...process.env, NODE_OPTIONS:'--test-reporter=tap', NODE_PATH:''}});\nchild.on('close', code => {process.exitCode = code ?? 1;});\n";
    fixture.write('scripts/check-native-hr-policy-parser-contract.mjs', "import {spawn} from 'node:child_process';\nconst binding = {node:{path:process.execPath}};\n" +
      (enabled ? call : call.split('\n').map(line => '// ' + line).join('\n')));
    const marker = join(fixture.root, 'TEST_ONLY-child-executed');
    const child = fixture.execute('scripts/run-native-hr-policy-parser-contract.sh', [], {
      CONSOLE_HR_POLICY_NODE:realpathSync(process.execPath), CONSOLE_HR_POLICY_BINDING:'/TEST_ONLY/unused',
      CONSOLE_HR_POLICY_BINDING_SHA256:'1'.repeat(64), TEST_ONLY_CHILD_MARKER:marker});
    assert.equal(child.status, 0, child.stderr); assert.equal(existsSync(marker), enabled);
    if (enabled) assert.match(child.stdout, /# tests 1\n[\s\S]*# pass 1\n/);
    const result = fixture.gate();
    if (enabled) {assert.equal(result.status, 0); assert.deepEqual(result.report.wired_exact, [suite]);}
    else assertDark(result, suite);
  } finally {fixture.cleanup();}
});
test('hermetic reachability refuses an uninvoked gate-sweep manifest', () => {
  const fixture = hermeticFixture();
  try {
    fixture.write('tools/ci/gate-sweep.json', JSON.stringify({gates:[{id:'fixture', run:'node --test fixture/new.test.mjs'}]}));
    assertDark(fixture.gate());
  } finally {fixture.cleanup();}
});

function markerChild(fixture, suite) {
  const marker = join(fixture.root, 'TEST_ONLY-child-executed');
  fixture.write(suite, "import test from 'node:test'; import fs from 'node:fs'; test('real fixture child', () => fs.writeFileSync(process.env.TEST_ONLY_CHILD_MARKER, 'executed'));\n");
  assert.equal(existsSync(marker), false);
  return marker;
}
function assertMarker(result, marker, enabled, status = 0) {
  assert.equal(result.signal, null, result.stderr); assert(!result.error, result.error?.message);
  assert.equal(result.status, status, result.stderr);
  assert.equal(existsSync(marker), enabled);
  if (enabled) {
    assert.equal(readFileSync(marker, 'utf8'), 'executed');
    const stdout = result.stdout.startsWith('{') ? JSON.parse(result.stdout).host.stdout : result.stdout;
    assert.match(stdout, /# tests 1\n[\s\S]*# pass 1\n# fail 0\n[\s\S]*# skipped 0\n/);
  } else assert.doesNotMatch(result.stdout, /# tests [1-9]/);
}
for (const enabled of [true, false])
for (const shape of ['short-circuit workflow', 'literal-if workflow'])
test(`hermetic execution-boundary ${enabled ? 'accepts executing' : 'refuses inert'} ${shape}`, () => {
  const condition = enabled ? 'true' : 'false';
  const command = 'node --test fixture/new.test.mjs';
  const run = shape === 'short-circuit workflow' ? `${condition} && ${command}` : `if ${condition}; then\n  ${command}\nfi`;
  const fixture = hermeticFixture(run);
  try {
    const marker = markerChild(fixture, 'fixture/new.test.mjs');
    const child = fixture.execute('/bin/sh', ['-c', run], {
      PATH:dirname(realpathSync(process.execPath)) + ':' + process.env.PATH,
      NODE_OPTIONS:'--test-reporter=tap', TEST_ONLY_CHILD_MARKER:marker});
    assertMarker(child, marker, enabled, !enabled && shape === 'short-circuit workflow' ? 1 : 0);
    const result = fixture.gate();
    if (enabled) {assert.equal(result.status, 0); assert.deepEqual(result.report.wired_exact, ['fixture/new.test.mjs']);}
    else assertDark(result);
  } finally {fixture.cleanup();}
});

for (const enabled of [true, false])
for (const shape of ['canonical absolute truth command', 'Node value option'])
test(`hermetic argv-boundary ${enabled ? 'accepts executing' : 'refuses inert'} ${shape}`, () => {
  const run = shape === 'canonical absolute truth command'
    ? `/usr/bin/${enabled ? 'true' : 'false'} && node --test fixture/new.test.mjs`
    : `node ${enabled ? '--test-reporter=tap' : '--unknown-console-option=1'} --test fixture/new.test.mjs`;
  const fixture = hermeticFixture(run);
  try {
    const marker = markerChild(fixture, 'fixture/new.test.mjs');
    const child = fixture.execute('/bin/sh', ['-c', run], {
      PATH:dirname(realpathSync(process.execPath)) + ':' + process.env.PATH,
      NODE_OPTIONS:shape === 'Node value option' ? '' : '--test-reporter=tap', TEST_ONLY_CHILD_MARKER:marker});
    assertMarker(child, marker, enabled, enabled ? 0 : shape === 'canonical absolute truth command' ? 1 : 9);
    if (!enabled && shape === 'Node value option') assert.match(child.stderr, /bad option: --unknown-console-option=1/);
    const result = fixture.gate();
    if (enabled) {assert.equal(result.status, 0); assert.deepEqual(result.report.wired_exact, ['fixture/new.test.mjs']);}
    else assertDark(result);
  } finally {fixture.cleanup();}
});

for (const enabled of [true, false])
for (const shape of ['workflow function', 'launcher function', 'bare false owner entry'])
test(`hermetic execution-boundary ${enabled ? 'accepts executing' : 'refuses inert'} ${shape}`, () => {
  const suite = 'scripts/check-native-hr-policy-parser-contract.test.mjs';
  const launcher = 'scripts/run-native-hr-policy-parser-contract.sh';
  const supervisor = 'scripts/check-native-hr-policy-parser-contract.mjs';
  const run = shape === 'workflow function' ? `run_fixture_native() {\n  ${launcher}\n}\n${enabled ? 'run_fixture_native' : 'true'}` : launcher;
  const fixture = hermeticFixture(run, [suite]);
  try {
    let shell = readFileSync(join(repo, launcher), 'utf8');
    if (shape === 'launcher function') {
      const command = '"$CONSOLE_HR_POLICY_NODE" "$SCRIPT_DIR/check-native-hr-policy-parser-contract.mjs" || status=$?';
      assert(shell.includes(command));
      shell = shell.replace(command, `run_fixture_supervisor() {\n${command}\n}\n${enabled ? 'run_fixture_supervisor' : 'true'}`);
    }
    fixture.write(launcher, shell, 0o755);
    const marker = markerChild(fixture, suite);
    if (shape === 'bare false owner entry') {
      // Exact owner bytes, with finite TEST_ONLY dependencies that never invoke native tools.
      const source = readFileSync(join(repo, supervisor), 'utf8');
      const entry = 'if (process.argv[1] === new URL(import.meta.url).pathname) process.exitCode = await main();';
      assert(source.includes(entry));
      fixture.write(supervisor, enabled ? source : source.replace(entry, 'if (false)\n' + entry));
      fixture.write('scripts/hr-policy-parser/binding.mjs', "import fs from 'node:fs'; import crypto from 'node:crypto'; export const ROOT = process.cwd(); export const regular = file => fs.readFileSync(file); export const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex'); export const loadBinding = () => ({node:{path:process.execPath}, timeout_ms:5000});\n");
      fixture.write('scripts/hr-policy-parser/native-run.mjs', "import fs from 'node:fs'; export const candidate = () => {}; export const command = () => ({status:0, signal:null}); export const logs = () => fs.existsSync(process.env.TEST_ONLY_CHILD_MARKER) ? ['TEST_ONLY-current-run-log'] : []; export const inspect = () => ({actions:[], ids:[], logs:[]});\n");
      fixture.write('scripts/hr-policy-parser/observations.mjs', "export const classify = (_observation, child) => child.status === 0 && child.signal === null && !child.abnormal ? 0 : 126;\n");
      fixture.write(suite, "import test from 'node:test'; import fs from 'node:fs'; import path from 'node:path'; import crypto from 'node:crypto'; test('real TEST_ONLY owner child', () => { const run = process.env.CONSOLE_HR_POLICY_RUN; const report = 'TEST_ONLY-no-native-build\\n'; fs.writeFileSync(path.join(run, 'build-report.json'), report); fs.writeFileSync(path.join(run, 'observation.json'), JSON.stringify({build_start_ms:Date.now(), build_logs:['TEST_ONLY-current-run-log'], compiler_actions:[], fresh_build_ids:[], event_logs:[], build_report:report, build_report_sha256:crypto.createHash('sha256').update(report).digest('hex')})); fs.writeFileSync(process.env.TEST_ONLY_CHILD_MARKER, 'executed'); });\n");
    } else {
      fixture.write(supervisor, "import {spawn} from 'node:child_process';\nconst binding = {node:{path:process.execPath}};\nconst child = spawn(binding.node.path, ['--test', 'scripts/check-native-hr-policy-parser-contract.test.mjs'], {stdio:'inherit', env:{...process.env, NODE_OPTIONS:'--test-reporter=tap', NODE_PATH:''}});\nchild.on('close', code => {process.exitCode = code ?? 1;});\n");
    }
    const child = fixture.execute('/bin/sh', ['-c', run], {
      CONSOLE_HR_POLICY_NODE:realpathSync(process.execPath), CONSOLE_HR_POLICY_BINDING:'/TEST_ONLY/unused',
      CONSOLE_HR_POLICY_BINDING_SHA256:'1'.repeat(64), TEST_ONLY_CHILD_MARKER:marker});
    assertMarker(child, marker, enabled);
    const result = fixture.gate();
    if (enabled) {assert.equal(result.status, 0); assert.deepEqual(result.report.wired_exact, [suite]);}
    else assertDark(result, suite);
  } finally {fixture.cleanup();}
});
