import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { CASES, HOST, IMPORT, LEGACY_CASES, SOURCE, TARGET, hash, pinned, regular } from './binding.mjs';

export function rustDiscovery(text, required) {
  const lines = text.split(/\r?\n/).filter(line => line !== '');
  assert(lines.length > 1);
  const footer = lines.pop();
  const names = lines.map(line => {
    const entry = line.match(/^(\S+): test$/);
    assert(entry, 'unknown discovery record or benchmark');
    return entry[1];
  });
  assert.equal(footer, `${names.length} tests, 0 benchmarks`);
  assert.equal(new Set(names).size, names.length);
  for (const name of required) assert(names.includes(name));
  return names;
}
export function rustExecution(text, names) {
  const lines = text.split(/\r?\n/).filter(line => line !== '');
  assert.equal(lines.shift(), `running ${names.length} tests`);
  const footer = lines.pop()?.match(/^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in \d+\.\d+s$/);
  assert(footer, 'missing or unknown execution footer');
  assert.deepEqual(footer.slice(1), [String(names.length), '0', '0', '0', '0']);
  const executed = lines.map(line => {
    const outcome = line.match(/^test (\S+) \.\.\. (\S+)$/);
    assert(outcome, 'unknown execution record');
    assert.equal(outcome[2], 'ok', 'unhealthy test outcome');
    return outcome[1];
  });
  assert.deepEqual(executed.sort(), [...names].sort());
  return { discovered: names.length, executed: executed.length, passed: executed.length, ignored: 0 };
}
export function hostFooter(text, exit) {
  assert([0, 1].includes(exit));
  const records = text.split(/\r?\n/).filter(line => line !== '');
  for (const expected of ['TAP version 13', `# Subtest: ${HOST}`, `${exit === 0 ? 'ok' : 'not ok'} 1 - ${HOST}`]) assert.equal(records.shift(), expected);
  // Only the declared result's balanced Node TAP diagnostic block may be indented.
  if (records[0] === '  ---') {
    records.shift();
    while (records.length && records[0] !== '  ...') {
      const line = records.shift();
      assert(/^  [A-Za-z_][A-Za-z_0-9]*:(?: |$)/.test(line) || line.startsWith('    '), 'unknown host diagnostic record');
    }
    assert.equal(records.shift(), '  ...', 'unclosed host diagnostic block');
  }
  const expected = ['1..1'];
  for (const [key, value] of Object.entries({ tests: 1, suites: 0, pass: exit === 0 ? 1 : 0, fail: exit === 0 ? 0 : 1, cancelled: 0, skipped: 0, todo: 0 })) expected.push(`# ${key} ${value}`);
  assert.deepEqual(records.slice(0, expected.length), expected);
  const extra = records.slice(expected.length);
  assert(extra.length === 0 || (extra.length === 1 && /^# duration_ms \d+(?:\.\d+)?$/.test(extra[0])), 'extra or unknown host record');
  return { discovered: 1, executed: 1, passed: exit === 0 ? 1 : 0, failed: exit === 0 ? 0 : 1 };
}
export function soleMissingEntry(diagnostics, sourceNames) {
  const errors = diagnostics.filter(d => d.level === 'error');
  const coded = errors.filter(d => d.code !== null);
  assert.equal(coded.length, 1);
  const error = coded[0];
  assert.equal(error.code.code, 'E0432');
  assert.equal(error.message, 'unresolved import `console_identity_application::company_policy::decode_native_hr_policy_command_v1`');
  const primary = error.spans.filter(s => s.is_primary);
  assert.equal(primary.length, 1);
  const span = primary[0];
  assert(sourceNames.includes(span.file_name));
  assert.equal(span.line_start, 1); assert.equal(span.line_end, 1);
  const start = IMPORT.indexOf('decode_native_hr_policy_command_v1') + 1;
  if (span.column_start === 5) {
    assert.equal(error.spans.length, 2); assert.deepEqual(error.children, []);
    const secondary = error.spans.find(s => s.is_primary === false);
    assert.equal(span.is_primary, true); assert(secondary);
    for (const [record, column, label] of [[span, 5, null],
      [secondary, start, 'no `decode_native_hr_policy_command_v1` in `company_policy`']]) {
      assert.equal(record.file_name, span.file_name);
      assert.equal(record.line_start, 1); assert.equal(record.line_end, 1);
      assert.equal(record.column_start, column); assert.equal(record.column_end, 85);
      assert.equal(record.byte_start, column - 1); assert.equal(record.byte_end, 84);
      assert.equal(record.label, label);
      assert.deepEqual(record.text, [{ text: IMPORT, highlight_start: column, highlight_end: 85 }]);
    }
  } else {
    assert.equal(span.column_start, start); assert.equal(span.column_end, start + 'decode_native_hr_policy_command_v1'.length);
    assert.equal(span.byte_start, start - 1); assert.equal(span.byte_end, span.column_end - 1);
    assert.equal(span.text.length, 1); assert.equal(span.text[0].text, IMPORT);
  }
  for (const d of diagnostics) {
    if (d === error) continue;
    assert(d.code === null && ((d.level === 'error' && d.message === 'aborting due to 1 previous error')
      || (d.level === 'failure-note' && d.message === 'For more information about this error, try `rustc --explain E0432`.')));
  }
  return true;
}
export function compilerAction(root, rows, binding, buildStart, freshBuildIds, isolation) {
  const actions = rows.filter(r => r.reason === 'build' && typeof r.identity === 'string'
    && r.identity.startsWith(`root${TARGET} `) && r.identity.includes('(rustc '));
  assert(actions.length > 0);
  const observed = [];
  for (const action of actions) {
    const details = action.reproducer?.details;
    assert.equal(action.reproducer.executor, 'Local');
    assert(freshBuildIds.includes(details.env.BUCK_BUILD_ID));
    const command = details.command;
    assert(Array.isArray(command) && command.length === 3 && command[2].startsWith('@'));
    const files = command.slice(0, 2).map(p => path.resolve(root, p));
    const response = path.resolve(root, command[2].slice(1));
    const args = regular(response).toString().trim().split('\n');
    assert(args.includes(`--buck-target=root${TARGET}`));
    const separator = args.indexOf('--rustc-action-separator'); assert(separator > 0);
    const compiler = path.resolve(root, args[separator - 1]); pinned(compiler, binding.compiler_sha256);
    let interpreter = files[0];
    if (fs.lstatSync(interpreter).isSymbolicLink()) {
      // The pinned prelude selects this direct CPython alias in the compiler's toolchain bucket.
      const coordinate = path.relative(root, compiler).match(/^buck-out\/(hr-policy-[a-f0-9]{32})\/art\/toolchains\/([a-f0-9]{16})\/rust\/__sysroot-aarch64-apple-darwin__\/sysroot\/bin\/rustc$/);
      assert(coordinate, 'unknown interpreter toolchain coordinate');
      const bin = path.join(root, 'buck-out', coordinate[1], 'art', 'toolchains', coordinate[2], '__cpython_archive__', 'cpython_archive', 'bin');
      assert.equal(interpreter, path.join(bin, 'python'));
      assert.equal(fs.readlinkSync(interpreter), 'python3.13');
      interpreter = path.join(bin, 'python3.13');
      regular(interpreter);
      assert.equal(fs.realpathSync(interpreter), interpreter);
    }
    const wrapperFiles = [interpreter, files[1]];
    for (const file of wrapperFiles) regular(file);
    assert.deepEqual(wrapperFiles.map(f => hash(regular(f))), binding.action_wrapper_sha256);
    const crateSource = path.resolve(root, args[separator + 1]);
    const sourceSha = binding.sources.find(s => s.path === SOURCE).sha256;
    let sourceBytes;
    const dependencyInputs = [];
    const sourceNames = [SOURCE, path.relative(root, crateSource), crateSource];
    if (fs.lstatSync(crateSource).isSymbolicLink()) {
      assert(typeof isolation === 'string' && /^hr-policy-[a-f0-9]{32}$/.test(isolation));
      const prefix = `buck-out/${isolation}/art/root/backend/crates/identity/application/__${TARGET.split(':')[1]}__/`;
      const suffix = `/__srcs/${SOURCE}`;
      const coordinate = path.relative(root, crateSource);
      assert.equal(args[separator + 1], coordinate);
      assert(coordinate.startsWith(prefix) && coordinate.endsWith(suffix));
      assert(/^[a-f0-9]{16}$/.test(coordinate.slice(prefix.length, -suffix.length)));
      const canonical = path.join(root, SOURCE);
      assert.equal(fs.realpathSync(path.dirname(crateSource)), path.dirname(crateSource));
      assert.equal(fs.realpathSync(canonical), canonical);
      pinned(canonical, sourceSha);
      assert.equal(fs.readlinkSync(crateSource), path.relative(path.dirname(crateSource), canonical));
      assert.equal(fs.realpathSync(crateSource), canonical);
      sourceBytes = regular(canonical);
      // Account for the wrapper's only observed response expansion without evaluating arguments.
      assert(!args.some(arg => /\$\((?:cat|abspath) /.test(arg)
        || arg.includes('${__BUILDSCRIPT_OUT_DIR__}')), 'unobserved argument evaluation');
      const includes = args.map((arg, index) => ({arg, index})).filter(({arg}) => arg.startsWith('@'));
      assert(includes.length <= 1, 'only the ordinary dependency response is supported');
      if (includes.length) {
        const {arg, index} = includes[0];
        assert(index > separator, 'dependency response must be a compiler argument');
        const relative = arg.slice(1);
        const childPrefix = `buck-out/${isolation}/art/root/`;
        const childSuffix = `/backend/crates/identity/application/__${TARGET.split(':')[1]}__/XIPL-depslink-symlinked_dirs/dirs`;
        assert(relative.startsWith(childPrefix) && relative.endsWith(childSuffix));
        assert(/^[a-f0-9]{16}$/.test(relative.slice(childPrefix.length, -childSuffix.length)));
        const child = path.resolve(root, relative);
        assert.equal(fs.realpathSync(path.dirname(child)), path.dirname(child));
        const bytes = regular(child);
        assert.equal(fs.realpathSync(child), child);
        assert(bytes.equals(Buffer.from(`-Ldependency=${path.posix.dirname(relative)}/0\n`)));
        dependencyInputs.push({path: child, sha256: hash(bytes)});
      }
      const materialization = coordinate.slice(0, -SOURCE.length);
      const destination = 'backend/crates/identity/application/';
      assert(!args.slice(0, separator).some(arg => {
        const option = arg.split('=')[0];
        return option.startsWith('--') && option.length > 2 && '--remap-cwd-prefix'.startsWith(option);
      }), 'wrapper remaps must not override source attribution');
      assert.deepEqual(args.filter(x => x.startsWith('--remap-path-prefix')),
        [`--remap-path-prefix=${materialization}=${destination}`]);
      sourceNames.push(destination + SOURCE);
    } else {
      pinned(crateSource, sourceSha); sourceBytes = regular(crateSource);
    }
    assert(args.includes('--test')); assert(args.includes('--crate-name=native_hr_policy_parser_contract'));
    assert(args.includes('--error-format=json'));
    assert(!args.some(x => x.startsWith('--test-threads') || x.includes('cargo')));
    const diagArgs = args.filter(x => x.startsWith('--diag-json=')); assert.equal(diagArgs.length, 1);
    const diagnosticPath = path.resolve(root, diagArgs[0].slice('--diag-json='.length));
    assert(fs.statSync(diagnosticPath).mtimeMs >= buildStart - 1);
    const diagnostics = regular(diagnosticPath).toString().trim().split('\n').filter(Boolean).map(x => JSON.parse(x));
    const links = args.filter(x => x.startsWith('--emit=link=')); assert.equal(links.length, 1);
    const outputPath = path.resolve(root, links[0].slice('--emit=link='.length));
    const output = fs.existsSync(outputPath) ? regular(outputPath) : null;
    const magic = output?.subarray(0, 4).toString('hex') ?? null;
    if (output) assert(['7f454c46', 'cffaedfe', 'feedfacf', 'cefaedfe', 'feedface', 'cafebabe', 'bebafeca'].includes(magic) || magic.startsWith('4d5a'));
    observed.push({ output_path: outputPath, output_sha256: output ? hash(output) : null, output_magic: magic, identity: action.identity, action, response_sha256: hash(regular(response)),
      compiler_sha256: hash(regular(compiler)), source_sha256: hash(sourceBytes),
      wrapper_sha256: wrapperFiles.map(f => hash(regular(f))), diagnostic_sha256: hash(regular(diagnosticPath)),
      diagnostics, source_names: sourceNames, build_id: details.env.BUCK_BUILD_ID,
      ...(dependencyInputs.length ? {dependency_response_inputs: dependencyInputs} : {}) });
  }
  return observed;
}
export function classify(observation, child, binding, nonce, cleanup) {
  try {
    assert.equal(child.signal, null); assert([0, 1].includes(child.status));
    assert.equal(observation.nonce, nonce); assert.equal(observation.binding_sha256, binding.binding_sha256);
    assert.equal(observation.candidate_head, binding.candidate_head); assert.equal(observation.candidate_tree, binding.candidate_tree);
    assert.deepEqual(observation.source_pins, binding.sources);
    const host = hostFooter(child.stdout, child.status);
    assert.equal(cleanup, true); assert.equal(observation.legacy.ignored, 0);
    const legacyNames = rustDiscovery(observation.legacy_list.stdout, LEGACY_CASES);
    const legacy = rustExecution(observation.legacy_run.stdout, legacyNames);
    assert.equal(observation.legacy_list.status, 0); assert.equal(observation.legacy_run.status, 0);
    assert.deepEqual(observation.legacy, legacy);
    assert(observation.compiler_actions.length > 0);
    for (const action of observation.compiler_actions) {
      assert.equal(action.compiler_sha256, binding.compiler_sha256);
      assert.equal(action.source_sha256, binding.sources.find(s => s.path === SOURCE).sha256);
      assert(observation.fresh_build_ids.includes(action.build_id));
      assert(action.identity.startsWith(`root${TARGET} `) && action.identity.includes('(rustc '));
      assert.deepEqual(action.wrapper_sha256, binding.action_wrapper_sha256);
      for (const sha of [action.response_sha256, action.diagnostic_sha256, ...action.wrapper_sha256]) assert(/^[a-f0-9]{64}$/.test(sha));
    }
    if (observation.outcome === 'MISSING_ENTRY') {
      assert.equal(child.status, 1); assert.equal(Number.isInteger(observation.build.status), true);
      assert(observation.build.status > 0 && observation.build.status < 126);
      assert.equal(observation.compiler_actions.length, 1);
      assert.deepEqual(observation.codec, { discovered: 0, executed: 0, passed: 0, ignored: 0 });
      for (const action of observation.compiler_actions) { assert.equal(action.output_sha256, null); soleMissingEntry(action.diagnostics, action.source_names); }
      assert.equal(host.failed, 1); return 101;
    }
    assert.equal(observation.outcome, 'GREEN'); assert.equal(child.status, 0); assert.equal(observation.build.status, 0);
    assert(observation.compiler_actions.every(a => !a.diagnostics.some(d => d.level === 'error') && /^[a-f0-9]{64}$/.test(a.output_sha256) && a.output_magic));
    assert.equal(observation.codec_list.status, 0); assert.equal(observation.codec_run.status, 0);
    const names = rustDiscovery(observation.codec_list.stdout, CASES); assert.deepEqual(names.sort(), [...CASES].sort());
    assert.deepEqual(rustExecution(observation.codec_run.stdout, names), { discovered: 8, executed: 8, passed: 8, ignored: 0 });
    assert.deepEqual(observation.codec, { discovered: 8, executed: 8, passed: 8, ignored: 0 });
    return 0;
  } catch { return 126; }
}
