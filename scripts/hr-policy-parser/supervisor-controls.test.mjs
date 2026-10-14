// Fake-only tooling controls. No product parser, native action, or genuine RED/GREEN is exercised.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { CASES, HOST, IMPORT, LEGACY_CASES, SOURCE, TARGET, hash, regular } from './binding.mjs';
import { classify, compilerAction, hostFooter, rustDiscovery, rustExecution, soleMissingEntry } from './observations.mjs';

const fixture = () => {
  const names = [...LEGACY_CASES, 'existing::fourth_test'];
  const list = ns => ns.map(n => `${n}: test`).join('\n') + `\n\n${ns.length} tests, 0 benchmarks\n`;
  const run = ns => `running ${ns.length} tests\n` + ns.map(n => `test ${n} ... ok`).join('\n') + `\n\ntest result: ok. ${ns.length} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n`;
  const tap = green => `TAP version 13\n# Subtest: ${HOST}\n${green ? 'ok' : 'not ok'} 1 - ${HOST}\n1..1\n# tests 1\n# suites 0\n# pass ${green ? 1 : 0}\n# fail ${green ? 0 : 1}\n# cancelled 0\n# skipped 0\n# todo 0\n`;
  const sha = 'a'.repeat(64); const nonce = 'b'.repeat(64); const buildId = '00000000-0000-4000-8000-000000000001';
  const column = IMPORT.indexOf('decode_native_hr_policy_command_v1') + 1;
  const diag = { level: 'error', code: { code: 'E0432' }, message: 'unresolved import `console_identity_application::company_policy::decode_native_hr_policy_command_v1`',
    spans: [{ is_primary: true, file_name: SOURCE, line_start: 1, line_end: 1, column_start: column, column_end: column + 34,
      byte_start: column - 1, byte_end: column + 33, text: [{ text: IMPORT }] }] };
  const b = { binding_sha256: sha, candidate_head: 'c'.repeat(40), candidate_tree: 'd'.repeat(40), compiler_sha256: sha, action_wrapper_sha256: [sha, sha], sources: [{ path: SOURCE, sha256: sha }] };
  const observation = { nonce, binding_sha256: sha, candidate_head: b.candidate_head, candidate_tree: b.candidate_tree, source_pins: b.sources,
    outcome: 'MISSING_ENTRY', legacy_list: { status: 0, stdout: list(names) }, legacy_run: { status: 0, stdout: run(names) },
    legacy: { discovered: 4, executed: 4, passed: 4, ignored: 0 }, build: { status: 1 }, fresh_build_ids: [buildId],
    codec: { discovered: 0, executed: 0, passed: 0, ignored: 0 },
    compiler_actions: [{ output_sha256: null, output_magic: null, identity: `root${TARGET} (platform) (rustc test [nonpic])`, compiler_sha256: sha, source_sha256: sha,
      response_sha256: sha, diagnostic_sha256: sha, wrapper_sha256: [sha, sha], build_id: buildId, diagnostics: [diag], source_names: [SOURCE] }] };
  return { b, observation, nonce, child: { status: 1, signal: null, stdout: tap(false) }, names, list, run, tap };
};
const status = f => classify(f.observation, f.child, f.b, f.nonce, true);

test('FAKE ONLY complete exact compiler observation maps101; full eight-case observation maps0', () => {
  const f = fixture(); assert.equal(status(f), 101);
  f.observation.outcome = 'GREEN'; f.observation.build.status = 0;
  f.observation.compiler_actions[0].diagnostics = [];
  f.observation.compiler_actions[0].output_sha256 = 'a'.repeat(64); f.observation.compiler_actions[0].output_magic = '7f454c46';
  f.observation.codec_list = { status: 0, stdout: f.list(CASES) };
  f.observation.codec_run = { status: 0, stdout: f.run(CASES) };
  f.observation.codec = { discovered: 8, executed: 8, passed: 8, ignored: 0 };
  f.child = { status: 0, signal: null, stdout: f.tap(true) }; assert.equal(status(f), 0);
  f.observation.codec_list.stdout = f.list([]); assert.equal(status(f), 126);
});
const healthy = green => {
  const f = fixture();
  if (green) {
    f.observation.outcome = 'GREEN'; f.observation.build.status = 0;
    f.observation.compiler_actions[0].diagnostics = [];
    f.observation.compiler_actions[0].output_sha256 = 'a'.repeat(64); f.observation.compiler_actions[0].output_magic = '7f454c46';
    f.observation.codec_list = { status: 0, stdout: f.list(CASES) };
    f.observation.codec_run = { status: 0, stdout: f.run(CASES) };
    f.observation.codec = { discovered: 8, executed: 8, passed: 8, ignored: 0 };
    f.child = { status: 0, signal: null, stdout: f.tap(true) };
  }
  assert.equal(status(f), green ? 0 : 101);
  return f;
};
test('FAKE ONLY actual classify rejects every contradictory duplicate host count', () => {
  for (const green of [false, true]) {
    for (const key of ['tests', 'suites', 'pass', 'fail', 'cancelled', 'skipped', 'todo']) {
      for (const count of [0, 1, 2]) {
        const f = healthy(green); f.child.stdout += `# ${key} ${count}\n`;
        assert.equal(status(f), 126); assert.throws(() => hostFooter(f.child.stdout, f.child.status));
      }
      const f = healthy(green); f.child.stdout = f.child.stdout.replace(new RegExp(`^# ${key} .*\n`, 'm'), '');
      assert.equal(status(f), 126); assert.throws(() => hostFooter(f.child.stdout, f.child.status));
    }
  }
});
test('FAKE ONLY actual classify rejects extra TAP result/plan/subtest or unknown records', () => {
  for (const green of [false, true]) {
    const f = healthy(green); f.child.stdout += '# duration_ms 1.25\n'; assert.equal(status(f), green ? 0 : 101);
    for (const extra of ['ok 2 - unreported second test', 'not ok 2 - hidden failure', '1..0', '1..2', '# Subtest: hidden', 'TAP version 13', '# tests unknown', '# duration_ms 1\n# duration_ms 2', 'unknown root record']) {
      const g = healthy(green); g.child.stdout += extra + '\n';
      assert.equal(status(g), 126); assert.throws(() => hostFooter(g.child.stdout, g.child.status));
    }
  }
});
test('FAKE ONLY actual classify rejects both trailing indentation corruptions and misplaced records', () => {
  for (const green of [false, true]) {
    for (const extra of ['  not ok 2 - hidden failure', '  # tests 0', '\tnot ok 2 - hidden failure', '  unknown record']) {
      const f = healthy(green); f.child.stdout += extra + '\n';
      assert.equal(status(f), 126); assert.throws(() => hostFooter(f.child.stdout, f.child.status));
    }
    for (const before of ['TAP version 13', `# Subtest: ${HOST}`, `${green ? 'ok' : 'not ok'} 1 - ${HOST}`, '# tests 1']) {
      const f = healthy(green); f.child.stdout = f.child.stdout.replace(before, '  not ok 2 - hidden failure\n' + before);
      assert.equal(status(f), 126); assert.throws(() => hostFooter(f.child.stdout, f.child.status));
    }
  }
});
test('FAKE ONLY declared Node TAP diagnostic block is bounded before the plan and balanced', () => {
  const diagnostic = '  ---\n  duration_ms: 1.25\n  type: test\n  error: |-\n    ordinary error text\n  ...\n';
  for (const green of [false, true]) {
    const f = healthy(green); f.child.stdout = f.child.stdout.replace('1..1\n', diagnostic + '1..1\n');
    assert.equal(status(f), green ? 0 : 101);
    for (const block of [
      diagnostic.replace('  ...\n', ''), diagnostic.replace('  ---\n', ''), diagnostic + diagnostic,
      diagnostic.replace('  duration_ms:', '  not ok 2 - hidden failure\n  duration_ms:'),
      diagnostic.replace('  type:', '  # tests 0\n  type:'), diagnostic.replace('  ...', '\t...'),
    ]) {
      const g = healthy(green); g.child.stdout = g.child.stdout.replace('1..1\n', block + '1..1\n');
      assert.equal(status(g), 126); assert.throws(() => hostFooter(g.child.stdout, g.child.status));
    }
    const g = healthy(green); g.child.stdout += diagnostic; assert.equal(status(g), 126);
  }
});
test('FAKE ONLY supporting real Node TAP success and assertion failure preserve bounded host shape', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hr-supporting-node-tap-'));
  try {
    for (const green of [false, true]) {
      const file = path.join(dir, 'host.test.mjs');
      fs.writeFileSync(file, `import test from 'node:test'; import assert from 'node:assert/strict'; test(${JSON.stringify(HOST)}, () => { ${green ? 'assert.equal(1,1);' : "assert.fail('supporting TAP format control; no compiler or product execution');"} });\n`);
      const env = {...process.env, NODE_OPTIONS:'--test-reporter=tap', NODE_PATH:''};
      delete env.NODE_TEST_CONTEXT;
      const result = spawnSync(process.execPath, ['--test', file], {encoding:'utf8', timeout:10_000,
        env});
      assert.equal(result.signal, null); assert(!result.error); assert.equal(result.status, green ? 0 : 1);
      assert(result.stdout.includes('  ---\n') && result.stdout.includes('  ...\n'));
      assert.deepEqual(hostFooter(result.stdout, result.status), {discovered:1, executed:1, passed:green ? 1 : 0, failed:green ? 0 : 1});
      const f = healthy(green); f.child.stdout = result.stdout; assert.equal(status(f), green ? 0 : 101);
    }
  } finally { fs.rmSync(dir, {recursive:true}); }
});
test('FAKE ONLY actual classify rejects all extra/failed/ignored/unknown Rust execution records', () => {
  for (const green of [false, true]) {
    for (const owner of green ? ['legacy', 'codec'] : ['legacy']) {
      const ns = owner === 'legacy' ? fixture().names : CASES;
      for (const extra of ['test hidden_failure ... FAILED', 'test hidden ... ignored', 'test hidden ... unknown', 'test hidden ... ok', 'running 0 tests', `running ${ns.length} tests`, 'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s']) {
        const f = healthy(green); const run = f.observation[`${owner}_run`]; run.stdout = run.stdout.replace('test result:', extra + '\ntest result:');
        assert.equal(status(f), 126); assert.throws(() => rustExecution(run.stdout, ns));
      }
      for (const change of [s => s.replace(' ... ok', ' ... FAILED'), s => s.replace(' ... ok', ' ... ignored'), s => s.replace(`running ${ns.length} tests`, 'running 40 tests'), s => s.replace('0 failed', '1 failed')]) {
        const f = healthy(green); const run = f.observation[`${owner}_run`]; run.stdout = change(run.stdout);
        assert.equal(status(f), 126); assert.throws(() => rustExecution(run.stdout, ns));
      }
    }
  }
});
test('FAKE ONLY actual classify rejects extra benchmark/unknown entries and duplicate discovery footers', () => {
  for (const green of [false, true]) {
    for (const owner of green ? ['legacy', 'codec'] : ['legacy']) {
      const ns = owner === 'legacy' ? fixture().names : CASES;
      for (const extra of ['hidden_benchmark: benchmark', 'hidden: unknown', 'hidden_test: test', `${ns[0]}: test`, '0 tests, 0 benchmarks', `${ns.length} tests, 0 benchmarks`]) {
        const f = healthy(green); const list = f.observation[`${owner}_list`]; list.stdout = extra + '\n' + list.stdout;
        assert.equal(status(f), 126); assert.throws(() => rustDiscovery(list.stdout, ns));
      }
      const f = healthy(green); const list = f.observation[`${owner}_list`]; list.stdout = list.stdout.replace('0 benchmarks', '1 benchmarks');
      assert.equal(status(f), 126); assert.throws(() => rustDiscovery(list.stdout, ns));
    }
  }
});
test('FAKE ONLY wrong diagnostic symbol/module/span and additional/unrelated errors refuse', () => {
  const changes = [
    d => { d.code.code = 'E0308'; }, d => { d.message = 'unresolved import `other::decode_native_hr_policy_command_v1`'; },
    d => { d.message = d.message.replace('decode_native_hr_policy_command_v1', 'missing_other'); },
    d => { d.spans[0].file_name = 'other.rs'; }, d => { d.spans[0].line_start = 2; },
    d => { d.spans[0].column_start++; }, d => { d.spans[0].byte_end++; },
    d => { d.spans[0].text[0].text = 'other source'; }, d => { d.spans[0].is_primary = false; },
  ];
  for (const change of changes) { const f = fixture(); change(f.observation.compiler_actions[0].diagnostics[0]); assert.equal(status(f), 126); }
  const f = fixture(); f.observation.compiler_actions[0].diagnostics.push({ level: 'error', code: null, message: 'dependency missing' }); assert.equal(status(f), 126);
  const g = fixture(); g.observation.compiler_actions[0].diagnostics.push({ level: 'error', code: null, message: 'aborting due to 1 previous error' }); assert.equal(status(g), 101);
});
test('FAKE ONLY stale/forged/zero-action and changed source/tool/target identities refuse', () => {
  for (const change of [
    f => { f.observation.nonce = 'stale'; }, f => { f.observation.binding_sha256 = 'bad'; },
    f => { f.observation.candidate_head = 'bad'; }, f => { f.observation.candidate_tree = 'bad'; },
    f => { f.observation.source_pins = []; }, f => { f.observation.compiler_actions = []; },
    f => { f.observation.fresh_build_ids = []; }, f => { f.observation.compiler_actions[0].source_sha256 = 'bad'; },
    f => { f.observation.compiler_actions[0].compiler_sha256 = 'bad'; }, f => { f.observation.compiler_actions[0].identity = 'other'; },
    f => { f.observation.compiler_actions[0].response_sha256 = 'bad'; }, f => { f.observation.build.status = 0; },
    f => { f.observation.build.status = 126; }, f => { f.observation.build.status = null; },
  ]) { const f = fixture(); change(f); assert.equal(status(f), 126); }
});
test('FAKE ONLY raw child failure/missing observation/footer/cleanup/interruption never maps101', () => {
  const f = fixture(); assert.equal(classify(undefined, f.child, f.b, f.nonce, true), 126);
  assert.equal(classify(f.observation, f.child, f.b, f.nonce, false), 126);
  for (const value of [0, 2, 101, 126, 127, 130, 137, -9, null]) { const g = fixture(); g.child.status = value; assert.equal(status(g), 126); }
  for (const signal of ['SIGINT', 'SIGTERM', 'SIGKILL']) { const g = fixture(); g.child.signal = signal; assert.equal(status(g), 126); }
  for (const stdout of ['', f.tap(false).replace('# tests 1', '# tests 0'), f.tap(false).replace('# cancelled 0', '# cancelled 1'), f.tap(false) + '# tests 1\n']) {
    const g = fixture(); g.child.stdout = stdout; assert.equal(status(g), 126);
  }
});
test('FAKE ONLY incomplete legacy or codec discovery/execution/ignored counts refuse', () => {
  for (const change of [
    f => { f.observation.legacy_list.status = 1; }, f => { f.observation.legacy_run.status = 1; },
    f => { f.observation.legacy_list.stdout = f.list([]); }, f => { f.observation.legacy_run.stdout = f.run(f.names.slice(1)); },
    f => { f.observation.legacy.ignored = 1; }, f => { f.observation.legacy.executed = 0; },
    f => { f.observation.codec.executed = 1; }, f => { f.observation.legacy_list.stdout = f.list(['unknown']); },
  ]) { const f = fixture(); change(f); assert.equal(status(f), 126); }
  assert.throws(() => rustDiscovery('0 tests, 0 benchmarks', []));
  assert.throws(() => rustExecution(fixture().run(CASES).replace('0 ignored', '1 ignored'), CASES));
  assert.throws(() => hostFooter('TAP version 13\n1..0\n', 0));
});
test('FAKE ONLY actual action parser checks synthetic regular response/compiler/source/diagnostic files', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hr-fake-action-'));
  try {
    const f = fixture(); const write = (name, value) => { const file = path.join(dir, name); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, value); return file; };
    write('python', 'fake pinned wrapper executable, never run'); write('rustc_action.py', 'fake wrapper, never run'); write('rustc', 'fake compiler, never run');
    write(SOURCE, IMPORT + '\n'); write('diag.json', JSON.stringify(f.observation.compiler_actions[0].diagnostics[0]) + '\n');
    write('args', `--diag-json=diag.json\n--buck-target=root${TARGET}\nrustc\n--rustc-action-separator\n${SOURCE}\n--test\n--crate-name=native_hr_policy_parser_contract\n--error-format=json\n--emit=link=fake-binary\n`);
    f.b.action_wrapper_sha256 = ['python', 'rustc_action.py'].map(name => hash(regular(path.join(dir, name))));
    f.b.compiler_sha256 = hash(regular(path.join(dir, 'rustc'))); f.b.sources[0].sha256 = hash(regular(path.join(dir, SOURCE)));
    const row = { reason: 'build', identity: `root${TARGET} (platform) (rustc test)`, reproducer: { executor: 'Local', details: { command: ['python', 'rustc_action.py', '@args'], env: { BUCK_BUILD_ID: f.observation.fresh_build_ids[0] } } } };
    assert.equal(compilerAction(dir, [row], f.b, Date.now() - 1000, f.observation.fresh_build_ids).length, 1);
    assert.throws(() => compilerAction(dir, [], f.b, 0, f.observation.fresh_build_ids));
    assert.throws(() => compilerAction(dir, [row], f.b, 0, ['stale']));
    assert.throws(() => compilerAction(dir, [row], f.b, Date.now() + 10_000, f.observation.fresh_build_ids));
    f.b.compiler_sha256 = 'bad'; assert.throws(() => compilerAction(dir, [row], f.b, 0, f.observation.fresh_build_ids));
  } finally { fs.rmSync(dir, { recursive: true }); }
});
test('FAKE ONLY direct missing/nonexecutable launcher, missing supervisor/child/tool, raw exit and killed controls refuse', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hr-fake-launch-'));
  const launcherSource = fs.readFileSync(new URL('../run-native-hr-policy-parser-contract.sh', import.meta.url));
  const env = { ...process.env, CONSOLE_HR_POLICY_NODE: fs.realpathSync(process.execPath), CONSOLE_HR_POLICY_BINDING: '/fake/reviewed', CONSOLE_HR_POLICY_BINDING_SHA256: 'a'.repeat(64) };
  const invoke = file => spawnSync(file, [], { encoding: 'utf8', shell: true, env });
  try {
    const launcher = path.join(dir, 'launcher.sh'); assert.equal(invoke(launcher).status, 127);
    fs.writeFileSync(launcher, launcherSource, { mode: 0o644 }); assert.equal(invoke(launcher).status, 126);
    fs.chmodSync(launcher, 0o755); assert.equal(invoke(launcher).status, 126);
    fs.writeFileSync(path.join(dir, 'check-native-hr-policy-parser-contract.mjs'), 'process.exit(1);\n'); assert.equal(invoke(launcher).status, 126);
    fs.writeFileSync(path.join(dir, 'check-native-hr-policy-parser-contract.test.mjs'), '// fake child never run\n'); assert.equal(invoke(launcher).status, 126);
    fs.writeFileSync(path.join(dir, 'check-native-hr-policy-parser-contract.mjs'), 'process.kill(process.pid, "SIGKILL");\n'); assert.equal(invoke(launcher).status, 126);
    assert.equal(spawnSync(launcher, [], { env: { ...env, CONSOLE_HR_POLICY_NODE: '/missing/node' } }).status, 126);
    assert.equal(spawnSync(launcher, [], { env: { ...env, CONSOLE_HR_POLICY_NODE: 'node' } }).status, 126);
  } finally { fs.rmSync(dir, { recursive: true }); }
});

// Machinery-only: files below model the pinned prelude toolchain layout; nothing executes them.
const interpreterFixture = () => {
  const tmp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'hr-fake-interpreter-alias-'))), dir = path.join(tmp, 'source'), f = fixture();
  const isolation = `buck-out/hr-policy-${'1'.repeat(32)}`, stem = `${isolation}/art/toolchains/11a256c53bf8083e`;
  const bin = `${stem}/__cpython_archive__/cpython_archive/bin`, python = `${bin}/python3.13`;
  const wrapper = `${isolation}/art/prelude/rust/tools/__rustc_action__/00cc8eff58bfacd6/__rustc_action__/rustc_action.py`;
  const compiler = `${stem}/rust/__sysroot-aarch64-apple-darwin__/sysroot/bin/rustc`;
  const write = (name, bytes) => { const file = path.join(dir, name); fs.mkdirSync(path.dirname(file), {recursive:true}); fs.writeFileSync(file, bytes); return file; };
  write(python, 'fake pinned CPython executable, never run'); write(wrapper, 'fake pinned wrapper, never run'); write(compiler, 'fake pinned compiler, never run');
  write(SOURCE, IMPORT + '\n'); write('diag.json', JSON.stringify(f.observation.compiler_actions[0].diagnostics[0]) + '\n');
  write('args', `--diag-json=diag.json\n--buck-target=root${TARGET}\n${compiler}\n--rustc-action-separator\n${SOURCE}\n--test\n--crate-name=native_hr_policy_parser_contract\n--error-format=json\n--emit=link=fake-binary\n`);
  f.b.action_wrapper_sha256 = [python, wrapper].map(name => hash(regular(path.join(dir, name))));
  f.b.compiler_sha256 = hash(regular(path.join(dir, compiler))); f.b.sources[0].sha256 = hash(regular(path.join(dir, SOURCE)));
  const row = {reason:'build', identity:`root${TARGET} (platform) (rustc test)`, reproducer:{executor:'Local', details:{command:[python, wrapper, '@args'], env:{BUCK_BUILD_ID:f.observation.fresh_build_ids[0]}}}};
  const execute = () => compilerAction(dir, [row], f.b, Date.now() - 1000, f.observation.fresh_build_ids);
  assert.equal(execute().length, 1, 'canonical regular-file positive control must pass before any alias mutation');
  const alias = (name, target) => { const file = path.join(dir, name); fs.mkdirSync(path.dirname(file), {recursive:true}); fs.symlinkSync(target, file); row.reproducer.details.command[0] = name; };
  return {...f, tmp, dir, isolation, stem, bin, python, wrapper, compiler, row, write, execute, alias};
};
const withInterpreterFixture = body => { const f = interpreterFixture(); try { body(f); } finally { fs.rmSync(f.tmp, {recursive:true}); } };

test('FAKE ONLY exact current-toolchain bin/python alias authenticates pinned regular python3.13 and preserves observed command', () => withInterpreterFixture(f => {
  const observed = `${f.bin}/python`; f.alias(observed, 'python3.13');
  const action = f.execute()[0];
  assert.deepEqual(action.wrapper_sha256, f.b.action_wrapper_sha256);
  assert.equal(action.action.reproducer.details.command[0], observed);
  assert.equal(action.compiler_sha256, f.b.compiler_sha256); assert.equal(action.source_sha256, f.b.sources[0].sha256);
}));
test('FAKE ONLY arbitrary/outside/undeclared interpreter aliases refuse despite matching canonical bytes', () => {
  for (const location of ['python3', 'runner', 'arbitrary', 'outside', 'other-archive']) withInterpreterFixture(f => {
    const name = location === 'arbitrary' ? 'arbitrary/python' : location === 'outside' ? '../outside/python' : location === 'other-archive' ? `${f.stem}/__other_archive__/bin/python` : `${f.bin}/${location}`;
    f.alias(name, path.join(f.dir, f.python)); assert.throws(f.execute, location);
  });
});
test('FAKE ONLY stale isolation or different toolchain bucket interpreter aliases refuse despite matching canonical bytes', () => {
  for (const changed of ['isolation', 'toolchain']) withInterpreterFixture(f => {
    const bin = changed === 'isolation' ? f.bin.replace(f.isolation, `buck-out/hr-policy-${'2'.repeat(32)}`) : f.bin.replace('11a256c53bf8083e', '22b367d64cf9194f');
    f.write(`${bin}/python3.13`, regular(path.join(f.dir, f.python))); f.alias(`${bin}/python`, 'python3.13'); assert.throws(f.execute, changed);
  });
});
test('FAKE ONLY declared interpreter alias refuses changed bytes/target, chained links and linked canonical file', () => {
  for (const changed of ['bytes', 'target', 'chain', 'canonical-link']) withInterpreterFixture(f => {
    if (changed === 'bytes') f.write(f.python, 'changed executable bytes');
    else if (changed === 'canonical-link') { fs.renameSync(path.join(f.dir, f.python), path.join(f.dir, `${f.bin}/other`)); fs.symlinkSync('other', path.join(f.dir, f.python)); }
    else if (changed === 'target') f.write(`${f.bin}/python3.12`, regular(path.join(f.dir, f.python)));
    else fs.symlinkSync('python3.13', path.join(f.dir, `${f.bin}/python3`));
    f.alias(`${f.bin}/python`, changed === 'target' ? 'python3.12' : changed === 'chain' ? 'python3' : 'python3.13'); assert.throws(f.execute, changed);
  });
});
test('FAKE ONLY interpreter alias exception never relaxes wrapper/compiler/response/source/diagnostic/output regular-file custody', () => {
  for (const slot of ['wrapper', 'compiler', 'args', SOURCE, 'diag.json', 'fake-binary']) withInterpreterFixture(f => {
    const name = slot === 'wrapper' ? f.wrapper : slot === 'compiler' ? f.compiler : slot;
    if (slot === 'fake-binary') f.write(name, Buffer.from('7f454c460001', 'hex'));
    fs.renameSync(path.join(f.dir, name), path.join(f.dir, name + '.real')); fs.symlinkSync(path.basename(name) + '.real', path.join(f.dir, name));
    assert.throws(f.execute, slot);
  });
});
