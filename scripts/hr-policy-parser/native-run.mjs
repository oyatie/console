import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { ROOT, TARGET, LEGACY, CASES, LEGACY_CASES, hash, loadBinding, regular } from './binding.mjs';
import { compilerAction, rustDiscovery, rustExecution, soleMissingEntry } from './observations.mjs';

export function command(binding, isolation, argv, timeout = binding.timeout_ms) {
  const args = ['build', 'run'].includes(argv[0]) ? [argv[0], '-j', '1', ...argv.slice(1)] : argv;
  return spawnSync(path.join(ROOT, 'tools/buck2'), ['--isolation-dir', isolation, ...args], {
    cwd: ROOT, encoding: 'utf8', timeout, maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, PATH: `/usr/bin:/bin:${path.dirname(binding.dotslash.path)}`, DOTSLASH_CACHE: binding.dotslash_cache },
  });
}
export function logs(isolation) {
  const dir = path.join(ROOT, 'buck-out', isolation, 'log');
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir).filter(x => x.endsWith('_events.pb.zst')).map(x => path.join(dir, x)).sort();
}
export function inspect(binding, isolation, eventPaths, started) {
  assert(eventPaths.length > 0);
  const rows = []; const ids = [];
  for (const file of eventPaths) {
    regular(file); assert(fs.statSync(file).mtimeMs >= started - 1);
    const id = path.basename(file).match(/_([a-f0-9-]{36})_events\.pb\.zst$/)?.[1]; assert(id); ids.push(id);
    const result = command(binding, isolation, ['log', 'what-ran', '--format', 'json', file]);
    assert.equal(result.status, 0); assert.equal(result.signal, null); assert(!result.error);
    rows.push(...result.stdout.trim().split('\n').filter(Boolean).map(x => JSON.parse(x)));
  }
  return { actions: compilerAction(ROOT, rows, binding, started, ids, isolation), ids,
    logs: eventPaths.map(file => ({ file, sha256: hash(regular(file)) })) };
}
export function candidate(binding) {
  const git = args => spawnSync(binding.git.path, ['-C', ROOT, ...args], { encoding: 'utf8', timeout: 10_000 });
  for (const [args, expected] of [[['rev-parse', 'HEAD'], binding.candidate_head], [['rev-parse', 'HEAD^{tree}'], binding.candidate_tree], [['status', '--porcelain=v1', '--untracked-files=all'], '']]) {
    const result = git(args); assert.equal(result.status, 0); assert.equal(result.signal, null);
    assert.equal(result.stdout.trim(), expected);
  }
}
export function runNative() {
  const b = loadBinding(); candidate(b);
  const run = process.env.CONSOLE_HR_POLICY_RUN;
  const nonce = process.env.CONSOLE_HR_POLICY_NONCE;
  const isolation = process.env.CONSOLE_HR_POLICY_ISOLATION;
  assert(path.isAbsolute(run) && /^[a-f0-9]{64}$/.test(nonce) && /^hr-policy-[a-f0-9]{32}$/.test(isolation));
  const observation = { nonce, binding_sha256: process.env.CONSOLE_HR_POLICY_BINDING_SHA256,
    candidate_head: b.candidate_head, candidate_tree: b.candidate_tree, source_pins: b.sources,
    codec: { discovered: 0, executed: 0, passed: 0, ignored: 0 } };
  const save = () => fs.writeFileSync(path.join(run, 'observation.json'), JSON.stringify(observation));
  const call = argv => {
    const r = command(b, isolation, argv);
    assert.equal(r.signal, null); assert(!r.error);
    return { argv, status: r.status, stdout: r.stdout, stderr: r.stderr };
  };
  // Full actual legacy discovery and execution precede the target compilation.
  observation.legacy_list = call(['run', LEGACY, '--', '--list']); assert.equal(observation.legacy_list.status, 0);
  const legacyNames = rustDiscovery(observation.legacy_list.stdout, LEGACY_CASES);
  observation.legacy_run = call(['run', LEGACY]); assert.equal(observation.legacy_run.status, 0);
  observation.legacy = rustExecution(observation.legacy_run.stdout, legacyNames);
  const before = new Set(logs(isolation)); observation.build_start_ms = Date.now();
  observation.build = call(['build', TARGET, '--build-report', path.join(run, 'build-report.json')]);
  observation.build_report = regular(path.join(run, 'build-report.json')).toString();
  JSON.parse(observation.build_report);
  observation.build_report_sha256 = hash(Buffer.from(observation.build_report));
  observation.build_logs = logs(isolation).filter(x => !before.has(x));
  const evidence = inspect(b, isolation, observation.build_logs, observation.build_start_ms);
  observation.compiler_actions = evidence.actions; observation.fresh_build_ids = evidence.ids;
  observation.event_logs = evidence.logs;
  if (observation.build.status !== 0) {
    assert.equal(observation.compiler_actions.length, 1);
    for (const action of observation.compiler_actions) soleMissingEntry(action.diagnostics, action.source_names);
    observation.outcome = 'MISSING_ENTRY'; candidate(b); save();
    assert.fail('designed HR parser entry is absent at the exact native compiler boundary');
  }
  assert(observation.compiler_actions.every(a => !a.diagnostics.some(d => d.level === 'error')));
  observation.codec_list = call(['run', TARGET, '--', '--list']); assert.equal(observation.codec_list.status, 0);
  const names = rustDiscovery(observation.codec_list.stdout, CASES); assert.deepEqual(names.sort(), [...CASES].sort());
  observation.codec_run = call(['run', TARGET]); assert.equal(observation.codec_run.status, 0);
  observation.codec = rustExecution(observation.codec_run.stdout, names);
  observation.outcome = 'GREEN'; candidate(b); save();
}
