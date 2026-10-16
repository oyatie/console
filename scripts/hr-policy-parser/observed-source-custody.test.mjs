// Machinery only: actual owner functions, private filesystem models, no tool execution.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const owner = process.env.CONSOLE_HR_CUSTODY_OWNER_MODULE ?? fileURLToPath(new URL('./observations.mjs', import.meta.url));
assert(path.isAbsolute(owner) && owner.endsWith('/scripts/hr-policy-parser/observations.mjs'));
assert.equal(fs.realpathSync(owner), owner);
const { compilerAction, soleMissingEntry } = await import(pathToFileURL(owner));
const { SOURCE, TARGET, IMPORT, hash, regular } = await import(pathToFileURL(path.join(path.dirname(owner), 'binding.mjs')));
const frozen = JSON.parse(fs.readFileSync(new URL('./fixtures/observed-source-custody-v1.json', import.meta.url)));
const realSource = regular(path.join(path.dirname(owner), '../..', SOURCE));
assert.equal(hash(realSource), frozen.source_sha256, 'frozen approved native test source must remain exact');
assert.equal(realSource.toString().split('\n')[0], IMPORT);
const owned = `hr-policy-${'1'.repeat(32)}`, foreign = `hr-policy-${'2'.repeat(32)}`;
const buildId = '00000000-0000-4000-8000-000000000001';
const clone = value => JSON.parse(JSON.stringify(value));
const reported = frozen.diagnostics[0].spans[0].file_name;

function fixture() {
  const tmp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'hr-source-custody-model-')));
  const root = path.join(tmp, 'source');
  const write = (relative, bytes) => {
    const file = path.resolve(root, relative);
    assert(file.startsWith(tmp + path.sep));
    fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, bytes); return file;
  };
  const stem = `buck-out/${owned}/art/toolchains/11a256c53bf8083e`;
  const python = `${stem}/__cpython_archive__/cpython_archive/bin/python3.13`;
  const wrapper = `buck-out/${owned}/art/prelude/rust/tools/__rustc_action__/00cc8eff58bfacd6/__rustc_action__/rustc_action.py`;
  const compiler = `${stem}/rust/__sysroot-aarch64-apple-darwin__/sysroot/bin/rustc`;
  write(python, 'MACHINERY ONLY pinned interpreter bytes; never executed');
  write(wrapper, 'MACHINERY ONLY pinned wrapper bytes; never executed');
  write(compiler, 'MACHINERY ONLY pinned compiler bytes; never executed');
  const canonical = write(SOURCE, realSource);
  const prefix = `buck-out/${owned}/art/root/backend/crates/identity/application/__${frozen.target_name}__/${frozen.source_bucket}/__srcs`;
  const source = `${prefix}/${SOURCE}`;
  const alias = (relative, target) => {
    const file = path.resolve(root, relative); assert(file.startsWith(tmp + path.sep));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    if (fs.existsSync(file) || fs.lstatSync(path.dirname(file)).isSymbolicLink()) {
      try { fs.unlinkSync(file); } catch (error) { if (error.code !== 'ENOENT') throw error; }
    }
    fs.symlinkSync(target ?? path.relative(path.dirname(file), canonical), file); return file;
  };
  const input = alias(source);
  const diagnostics = clone(frozen.diagnostics);
  const diag = 'diagnostic.json';
  write(diag, diagnostics.map(x => JSON.stringify(x)).join('\n') + '\n');
  const baseArgs = [`--diag-json=${diag}`, `--buck-target=root${TARGET}`, compiler,
    '--rustc-action-separator', source, '--test', '--crate-name=native_hr_policy_parser_contract',
    '--error-format=json', '--emit=link=absent-output'];
  let args = [...baseArgs, `--remap-path-prefix=${prefix}/=${frozen.remap_destination}`];
  write('response.args', args.join('\n') + '\n');
  const binding = { sources: [{ path: SOURCE, sha256: hash(realSource) }],
    compiler_sha256: hash(regular(path.join(root, compiler))),
    action_wrapper_sha256: [python, wrapper].map(x => hash(regular(path.join(root, x)))) };
  const row = { reason: 'build', identity: `root${TARGET} (platform) (rustc link [pic])`,
    reproducer: { executor: 'Local', details: { command: [python, wrapper, '@response.args'], env: { BUCK_BUILD_ID: buildId } } } };
  const setArgs = value => {
    assert.notDeepEqual(value, args, 'argv mutation must change its actual input');
    args = value; write('response.args', args.join('\n') + '\n');
  };
  const execute = (isolation = owned) => compilerAction(root, [row], binding, Date.now() - 1000, [buildId], isolation);
  return { tmp, root, write, alias, canonical, prefix, source, input, binding, row, baseArgs,
    get args() { return args; }, setArgs, execute, diagnostics };
}
const withFixture = body => { const f = fixture(); try { body(f); } finally { fs.rmSync(f.tmp, { recursive: true }); } };
const replaceSource = (f, source) => {
  assert.notEqual(source, f.source, 'source coordinate mutation must change the actual token');
  assert.equal(f.args.filter(x => x === f.source).length, 1);
  f.setArgs(f.args.map(x => x === f.source ? source : x));
};
const custodySnapshot = f => {
  const files = [];
  const visit = directory => {
    for (const name of fs.readdirSync(directory).sort()) {
      const file = path.join(directory, name), stat = fs.lstatSync(file), relative = path.relative(f.tmp, file);
      if (stat.isSymbolicLink()) files.push([relative, 'link', fs.readlinkSync(file)]);
      else if (stat.isDirectory()) { files.push([relative, 'directory']); visit(file); }
      else { assert(stat.isFile()); files.push([relative, 'file', hash(fs.readFileSync(file))]); }
    }
  };
  visit(f.tmp);
  return clone({ args: f.args, row: f.row, binding: f.binding, files });
};
const refuse = body => withFixture(f => {
  const before = custodySnapshot(f); body(f);
  assert.notDeepEqual(custodySnapshot(f), before, 'custody mutation must change its actual input');
  assert.throws(() => f.execute());
});

test('MACHINERY ONLY actual observed direct source alias and remap retain exact compiler custody', () => withFixture(f => {
  const [action] = f.execute();
  assert.equal(action.source_sha256, frozen.source_sha256);
  assert.equal(action.compiler_sha256, f.binding.compiler_sha256);
  assert.deepEqual(action.wrapper_sha256, f.binding.action_wrapper_sha256);
  assert(action.source_names.includes(reported), 'reported name must derive from exact authenticated remap');
  assert.deepEqual(action.diagnostics, frozen.diagnostics);
  assert.equal(action.output_sha256, null);
}));
test('MACHINERY ONLY actual Rust whole-import primary plus exact secondary missing-member span accepts', () => {
  for (const reverse of [false, true]) {
    const records = clone(frozen.diagnostics); if (reverse) records[0].spans.reverse();
    assert.equal(soleMissingEntry(records, [reported]), true);
  }
});
test('MACHINERY ONLY legacy regular source and exact member primary preserve accepted behavior', () => withFixture(f => {
  f.setArgs(f.baseArgs.map(x => x === f.source ? SOURCE : x));
  assert.equal(f.execute().length, 1);
  const records = clone(frozen.diagnostics), span = clone(records[0].spans.find(x => !x.is_primary));
  span.is_primary = true; span.label = null; span.file_name = SOURCE; records[0].spans = [span];
  assert.equal(soleMissingEntry(records, [SOURCE]), true);
}));
test('MACHINERY ONLY new source branch requires caller-owned isolation, not path or action self-report', () => {
  for (const value of [undefined, '', foreign, 'hr-policy-invalid']) withFixture(f => {
    assert.notEqual(value, owned, 'caller-isolation mutation must change the actual authority argument');
    assert.throws(() => compilerAction(f.root, [f.row], f.binding, Date.now() - 1000, [buildId], value));
  });
});
test('MACHINERY ONLY malformed stale foreign target and materialization source coordinates refuse', () => {
  const changes = [s => s.replace(owned, foreign), s => s.replace(frozen.target_name, 'foreign-target'),
    s => s.replace(frozen.source_bucket, frozen.source_bucket.toUpperCase()),
    s => s.replace(frozen.source_bucket, 'not-a-hex-bucket'), s => s.replace('/__srcs/', '/extra/__srcs/'),
    s => s.replace('/tests/native_hr_policy_parser_contract.rs', '/tests/foreign.rs'),
    s => '../outside/' + s, s => s.replace('/art/root/', '/art/foreign/')];
  for (const change of changes) refuse(f => { const source = change(f.source); f.alias(source); replaceSource(f, source); });
});
test('MACHINERY ONLY source alias rejects absolute alternate chained and outside links with equal bytes', () => {
  for (const kind of ['absolute', 'alternate', 'chain', 'outside']) refuse(f => {
    let target;
    if (kind === 'absolute') target = f.canonical;
    else if (kind === 'alternate') target = './' + path.relative(path.dirname(f.input), f.canonical);
    else if (kind === 'chain') {
      const link = path.join(f.tmp, 'chain'); fs.symlinkSync(f.canonical, link);
      target = path.relative(path.dirname(f.input), link);
    } else target = path.relative(path.dirname(f.input), f.write('../outside-source.rs', realSource));
    f.alias(f.source, target);
  });
});
test('MACHINERY ONLY materialization parents and canonical source remain nonsymlinked and frozen', () => {
  for (const kind of ['parent-link', 'canonical-parent-link', 'canonical-link', 'changed-bytes']) refuse(f => {
    if (kind === 'changed-bytes') fs.appendFileSync(f.canonical, '// changed\n');
    else if (kind === 'canonical-link') {
      fs.renameSync(f.canonical, f.canonical + '.actual'); fs.symlinkSync(path.basename(f.canonical) + '.actual', f.canonical);
    } else {
      const parent = kind === 'canonical-parent-link' ? path.dirname(f.canonical) : path.join(f.root, f.prefix);
      const alias = fs.readlinkSync(f.input); fs.renameSync(parent, parent + '.actual');
      fs.symlinkSync(path.basename(parent) + '.actual', parent);
      assert.equal(hash(fs.readFileSync(f.canonical)), frozen.source_sha256);
      assert.equal(fs.readlinkSync(f.input), alias, 'parent mutation must preserve the direct alias literal');
    }
  });
});
test('MACHINERY ONLY remap missing duplicate wrong prefix destination and extra mapping refuse', () => {
  for (const kind of ['missing', 'duplicate', 'prefix', 'destination', 'extra', 'wrong-first', 'wrong-last']) refuse(f => {
    const remap = f.args.at(-1); let value = f.args.slice(0, -1);
    assert.equal(remap, `--remap-path-prefix=${f.prefix}/=${frozen.remap_destination}`);
    const wrong = remap.replace(f.prefix, f.prefix.replace(owned, foreign));
    assert.notEqual(wrong, remap, 'foreign remap mutation must change the actual prefix');
    if (kind === 'duplicate') value.push(remap, remap);
    else if (kind === 'prefix') value.push(remap.replace(f.prefix, f.prefix.replace(owned, foreign)));
    else if (kind === 'destination') value.push(remap.replace('=backend/crates/identity/application/', '=foreign/'));
    else if (kind === 'extra') value.push(remap, '--remap-path-prefix=elsewhere/=foreign/');
    else if (kind === 'wrong-first') value.push(wrong, remap);
    else if (kind === 'wrong-last') value.push(remap, wrong);
    f.setArgs(value);
  });
});
test('MACHINERY ONLY alias exception never relaxes other regular-file custody or existing action checks', () => {
  for (const kind of ['compiler', 'wrapper', 'response', 'diagnostic', 'output']) refuse(f => {
    if (kind === 'output') f.write('absent-output', Buffer.from('7f454c46', 'hex'));
    const relative = kind === 'response' ? 'response.args' : kind === 'diagnostic' ? 'diagnostic.json'
      : kind === 'output' ? 'absent-output' : kind === 'wrapper' ? f.row.reproducer.details.command[1] : f.baseArgs[2];
    const file = path.join(f.root, relative); fs.renameSync(file, file + '.actual');
    fs.symlinkSync(path.basename(file) + '.actual', file);
  });
  for (const kind of ['build-id', 'compiler-hash', 'wrapper-hash', 'source-hash', 'foreign-executor']) refuse(f => {
    if (kind === 'build-id') f.row.reproducer.details.env.BUCK_BUILD_ID = 'stale';
    else if (kind === 'compiler-hash') f.binding.compiler_sha256 = '0'.repeat(64);
    else if (kind === 'wrapper-hash') f.binding.action_wrapper_sha256[1] = '0'.repeat(64);
    else if (kind === 'source-hash') f.binding.sources[0].sha256 = '0'.repeat(64);
    else f.row.reproducer.executor = 'Remote';
  });
  withFixture(f => {
    const file = path.join(f.root, 'diagnostic.json'), before = fs.statSync(file).mtimeMs;
    fs.utimesSync(file, new Date(1), new Date(1));
    assert.notEqual(fs.statSync(file).mtimeMs, before, 'freshness mutation must change the actual timestamp');
    assert.throws(() => f.execute());
  });
});

const spanRefuse = edit => {
  const records = clone(frozen.diagnostics); edit(records);
  assert.notDeepEqual(records, frozen.diagnostics, 'span mutation must change its actual input');
  assert.throws(() => soleMissingEntry(records, [reported]));
};
test('MACHINERY ONLY extra missing duplicated and nonboolean primary or secondary spans refuse', () => {
  const changes = [r => r[0].spans.pop(), r => r[0].spans.shift(),
    r => r[0].spans.push(clone(r[0].spans[0])), r => r[0].spans.push(clone(r[0].spans[1])),
    r => r[0].children.push({ message: 'hidden diagnostic' })];
  for (const isPrimary of [true, false, 'true', 'false', 1, 0, null]) {
    changes.push(r => { r[0].spans[0].is_primary = isPrimary; });
    changes.push(r => { r[0].spans[1].is_primary = isPrimary; });
  }
  for (const change of changes) {
    const value = clone(frozen.diagnostics); change(value);
    if (JSON.stringify(value) !== JSON.stringify(frozen.diagnostics)) spanRefuse(change);
  }
});
test('MACHINERY ONLY both actual span records bind exact file line byte column text highlight and labels', () => {
  for (const index of [0, 1]) for (const field of ['file_name', 'line_start', 'line_end', 'column_start', 'column_end', 'byte_start', 'byte_end']) {
    spanRefuse(r => { r[0].spans[index][field] = field === 'file_name' ? 'foreign.rs' : r[0].spans[index][field] + 1; });
  }
  for (const index of [0, 1]) for (const field of ['text', 'highlight_start', 'highlight_end']) {
    spanRefuse(r => { r[0].spans[index].text[0][field] = field === 'text' ? IMPORT + ' ' : r[0].spans[index].text[0][field] + 1; });
  }
  for (const index of [0, 1]) {
    spanRefuse(r => { r[0].spans[index].text = []; });
    spanRefuse(r => { r[0].spans[index].text.push(clone(r[0].spans[index].text[0])); });
    spanRefuse(r => { r[0].spans[index].label = 'unexpected'; });
    spanRefuse(r => { delete r[0].spans[index].label; });
  }
  for (const field of ['spans', 'children']) spanRefuse(r => { r[0][field] = null; });
});
test('MACHINERY ONLY wrong import member code message companions and unbound filenames refuse', () => {
  for (const change of [r => { r[0].code.code = 'E0603'; }, r => { r[0].message += ' extra'; },
    r => { r[0].spans[0].label = 'no `foreign_member` in `company_policy`'; },
    r => { r[0].spans[0].text[0].text = IMPORT.replace('decode_native_hr_policy_command_v1', 'foreign_member'); },
    r => { r.push(clone(r[0])); }, r => { r.push({ level: 'error', code: null, message: 'unrelated compiler failure' }); },
    r => { r[0].spans.forEach(s => { s.file_name = SOURCE; }); }]) spanRefuse(change);
});
