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
