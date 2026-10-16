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
// Exact adopted design a13bfbd8; additive reader controls, never a native/compiler run.
const platform = '39893c7e49283da4';
const childCoordinate = (isolation = owned, bucket = platform, target = frozen.target_name) =>
  `buck-out/${isolation}/art/root/${bucket}/backend/crates/identity/application/__${target}__/XIPL-depslink-symlinked_dirs/dirs`;
const childBytes = relative => Buffer.from(`-Ldependency=${path.posix.dirname(relative)}/0\n`);
function dependencyFixture() {
  const f = fixture();
  f.childRelative = childCoordinate(); f.childPath = f.write(f.childRelative, childBytes(f.childRelative));
  f.childToken = '@' + f.childRelative; f.setArgs([...f.args, f.childToken]);
  return f;
}
const withDependency = body => {
  const f = dependencyFixture();
  try { body(f); } finally {
    fs.rmSync(f.tmp, {recursive: true}); assert.equal(fs.existsSync(f.tmp), false, 'owned model cleanup verified');
  }
};
function replaceChild(f, relative) {
  const token = '@' + relative;
  assert.notEqual(token, f.childToken, 'child token mutation changes actual argv');
  assert.equal(f.args.filter(x => x === f.childToken).length, 1);
  f.setArgs(f.args.map(x => x === f.childToken ? token : x));
  f.childRelative = relative; f.childToken = token; f.childPath = path.resolve(f.root, relative);
}
const refuseDependency = mutate => withDependency(f => {
  const before = custodySnapshot(f); mutate(f);
  const after = custodySnapshot(f);
  assert.notDeepEqual(after, before, 'mutation must alter real argument/file custody outside rejection catcher');
  assert.throws(() => f.execute());
  assert.deepEqual(custodySnapshot(f), after, 'reader refusal cannot mutate evidence');
});
function exactAction(f, inputs) {
  const before = custodySnapshot(f), [action] = f.execute();
  assert.equal(action.source_sha256, frozen.source_sha256);
  assert.equal(action.compiler_sha256, f.binding.compiler_sha256);
  assert.deepEqual(action.wrapper_sha256, f.binding.action_wrapper_sha256);
  assert.deepEqual(action.diagnostics, frozen.diagnostics);
  assert.equal(soleMissingEntry(action.diagnostics, action.source_names), true);
  assert.equal(action.output_sha256, null);
  assert.deepEqual(action.dependency_response_inputs ?? [], inputs, 'complete finite child custody required');
  assert.deepEqual(custodySnapshot(f), before, 'reader must preserve all original inputs');
}
test('DEPENDENCY positive actual finite one-line child retains exact source diagnostics and input path/hash', () => withDependency(f => {
  assert.equal(childBytes(f.childRelative).length, 230, 'observed dependency-child shape retained');
  exactAction(f, [{path: f.childPath, sha256: hash(childBytes(f.childRelative))}]);
}));
test('DEPENDENCY positive new current platform literal records fresh bytes rather than historical digest', () => withDependency(f => {
  const oldHash = hash(childBytes(f.childRelative)), changed = childCoordinate(owned, 'abcdabcdabcdabcd');
  f.write(changed, childBytes(changed)); replaceChild(f, changed);
  assert.notEqual(hash(childBytes(changed)), oldHash, 'fresh child identity must actually differ');
  exactAction(f, [{path: f.childPath, sha256: hash(childBytes(changed))}]);
}));
test('DEPENDENCY positive zero-child materialized fixture preserves source and diagnostic contract', () => withFixture(f => exactAction(f, [])));
test('DEPENDENCY positive legacy regular source preserves zero-child behavior', () => withFixture(f => {
  f.setArgs(f.baseArgs.map(x => x === f.source ? SOURCE : x));
  const before = custodySnapshot(f), [action] = f.execute();
  assert.equal(action.source_sha256, frozen.source_sha256); assert.deepEqual(action.diagnostics, frozen.diagnostics);
  assert.deepEqual(action.dependency_response_inputs ?? [], []); assert.deepEqual(custodySnapshot(f), before);
}));
const coordinates = [
  ['absolute', f => f.childPath], ['leading-dot', f => './' + f.childRelative],
  ['normalized-parent', f => f.childRelative.replace('/XIPL-depslink-symlinked_dirs/', '/XIPL-depslink-symlinked_dirs/../XIPL-depslink-symlinked_dirs/')],
  ['foreign-isolation', f => f.childRelative.replace(owned, foreign)],
  ['stale-isolation', f => f.childRelative.replace(owned, `hr-policy-${'3'.repeat(32)}`)],
  ['invalid-isolation', f => f.childRelative.replace(owned, 'hr-policy-invalid')],
  ['uppercase-platform', f => f.childRelative.replace(platform, platform.toUpperCase())],
  ['short-platform', f => f.childRelative.replace(platform, platform.slice(1))],
  ['long-platform', f => f.childRelative.replace(platform, platform + '0')],
  ['nonhex-platform', f => f.childRelative.replace(platform, 'g'.repeat(16))],
  ['foreign-target', f => f.childRelative.replace(frozen.target_name, 'foreign-target')],
  ['extra-parent', f => f.childRelative.replace('/art/root/', '/art/root/extra/')],
  ['foreign-cell', f => f.childRelative.replace('/art/root/', '/art/foreign/')],
  ['alternate-runtime', f => f.childRelative.replace('/XIPL-depslink-symlinked_dirs/', '/other-runtime/')],
  ['alternate-file', f => f.childRelative.replace('/dirs', '/args')],
];
for (const [name, change] of coordinates) test('DEPENDENCY refuses altered coordinate ' + name, () => refuseDependency(f => {
  const relative = change(f); assert.notEqual(relative, f.childRelative);
  // Equal canonical aliases preserve original bytes; distinct coordinates have complete matching one-line files.
  if (name === 'absolute' || name === 'leading-dot' || name === 'normalized-parent') {
    assert.equal(path.resolve(f.root, relative), f.childPath);
    assert.equal(hash(fs.readFileSync(f.childPath)), hash(childBytes(f.childRelative)));
  } else f.write(relative, childBytes(relative));
  replaceChild(f, relative);
}));
test('DEPENDENCY refuses wrapper-position inclusion before compiler', () => refuseDependency(f => {
  const args = f.args.filter(x => x !== f.childToken), position = args.indexOf(f.baseArgs[2]);
  assert(position > 0); f.setArgs([...args.slice(0, position), f.childToken, ...args.slice(position)]);
  assert(f.args.indexOf(f.childToken) < f.args.indexOf('--rustc-action-separator'));
}));
for (const kind of ['same', 'different']) test('DEPENDENCY refuses duplicate inclusion ' + kind, () => refuseDependency(f => {
  const second = kind === 'same' ? f.childRelative : childCoordinate(owned, 'abcdabcdabcdabcd');
  if (kind === 'different') f.write(second, childBytes(second));
  f.setArgs([...f.args, '@' + second]); assert.equal(f.args.filter(x => x.startsWith('@')).length, 2);
}));
for (const kind of ['direct-link', 'absolute-link', 'chained-link', 'outside-link', 'parent-link', 'ancestor-link', 'missing', 'directory'])
  test('DEPENDENCY refuses noncanonical child custody ' + kind, () => refuseDependency(f => {
    const beforeHash = hash(fs.readFileSync(f.childPath));
    if (kind === 'missing') fs.unlinkSync(f.childPath);
    else if (kind === 'directory') { fs.unlinkSync(f.childPath); fs.mkdirSync(f.childPath); }
    else if (kind === 'parent-link' || kind === 'ancestor-link') {
      const parent = kind === 'parent-link' ? path.dirname(f.childPath) : path.join(f.root, `buck-out/${owned}/art/root/${platform}`);
      fs.renameSync(parent, parent + '.actual'); fs.symlinkSync(path.basename(parent) + '.actual', parent);
      assert.equal(hash(fs.readFileSync(f.childPath)), beforeHash); assert.notEqual(fs.realpathSync(f.childPath), f.childPath);
    } else {
      fs.renameSync(f.childPath, f.childPath + '.actual'); let target = path.basename(f.childPath) + '.actual';
      if (kind === 'absolute-link') target = f.childPath + '.actual';
      if (kind === 'chained-link') {
        const middle = f.write('chain-placeholder', 'temporary model'); fs.unlinkSync(middle);
        fs.symlinkSync(f.childPath + '.actual', middle); target = path.relative(path.dirname(f.childPath), middle);
      }
      if (kind === 'outside-link') target = path.relative(path.dirname(f.childPath), f.write('../outside-child', childBytes(f.childRelative)));
      fs.symlinkSync(target, f.childPath); assert.equal(hash(fs.readFileSync(f.childPath)), beforeHash);
      assert.equal(fs.lstatSync(f.childPath).isSymbolicLink(), true);
    }
  }));
const contents = [
  ['empty', f => Buffer.alloc(0)], ['wrong-directory', f => Buffer.from('-Ldependency=foreign/0\n')],
  ['wrong-list-item', f => childBytes(f.childRelative).toString().replace('/0\n', '/1\n')],
  ['no-final-newline', f => childBytes(f.childRelative).subarray(0, -1)],
  ['CRLF', f => childBytes(f.childRelative).toString().replace('\n', '\r\n')],
  ['blank-line', f => Buffer.concat([childBytes(f.childRelative), Buffer.from('\n')])],
  ['leading-space', f => Buffer.concat([Buffer.from(' '), childBytes(f.childRelative)])],
  ['trailing-space', f => childBytes(f.childRelative).toString().replace('\n', ' \n')],
  ['second-remap', f => Buffer.concat([childBytes(f.childRelative), Buffer.from('--remap-path-prefix=foreign/=foreign/\n')])],
  ['nested-response', f => { f.write('nested-child.args', '--remap-cwd-prefix=foreign/\n'); return '@nested-child.args\n'; }],
  ['self-response', f => f.childToken + '\n'],
  ['cat-expression', f => '$(cat nested-child.args)\n'],
  ['abspath-expression', f => '$(abspath nested-child.args)\n'],
  ['OUT_DIR-sentinel', f => '-Ldependency=${__BUILDSCRIPT_OUT_DIR__}/0\n'],
  ['NUL', f => Buffer.concat([childBytes(f.childRelative), Buffer.from([0])])],
  ['BOM', f => Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), childBytes(f.childRelative)])],
  ['invalid-UTF8', f => Buffer.concat([Buffer.from([0xff]), childBytes(f.childRelative)])],
];
for (const [name, change] of contents) test('DEPENDENCY refuses nonliteral child content ' + name, () => refuseDependency(f => {
  const altered = Buffer.from(change(f)); assert.notDeepEqual(altered, childBytes(f.childRelative));
  fs.writeFileSync(f.childPath, altered);
}));
const transforms = [
  ['compiler-cat', '--extern=$(cat transformation.txt)=model.rlib', 'compiler'],
  ['compiler-abspath', '--env-set=TEST_ONLY=$(abspath transformation.txt)', 'compiler'],
  ['compiler-OUT_DIR', '--env-set=TEST_ONLY=${__BUILDSCRIPT_OUT_DIR__}', 'compiler'],
  ['wrapper-cat', '--env=TEST_ONLY=$(cat transformation.txt)', 'wrapper'],
  ['wrapper-abspath', '--env=TEST_ONLY=$(abspath transformation.txt)', 'wrapper'],
  ['wrapper-OUT_DIR', '--env=TEST_ONLY=${__BUILDSCRIPT_OUT_DIR__}', 'wrapper'],
];
for (const [name, token, position] of transforms) test('DEPENDENCY refuses declared raw transformation ' + name, () => refuseDependency(f => {
  f.write('transformation.txt', '--remap-path-prefix=foreign/=foreign/\n');
  if (position === 'compiler') f.setArgs([...f.args, token]);
  else { const at = f.args.indexOf(f.baseArgs[2]); f.setArgs([...f.args.slice(0, at), token, ...f.args.slice(at)]); }
  assert.equal(f.args.includes(token), true);
}));

// Exact independent T1 correction: the same six declared transforms also refuse without a child.
for (const [name, token, position] of transforms) test('DEPENDENCY zero-child refuses declared raw transformation ' + name, () => {
  let ownedTmp;
  try { withFixture(f => {
    ownedTmp = f.tmp; const before = custodySnapshot(f);
    f.write('transformation.txt', '--remap-path-prefix=foreign/=foreign/\n');
    if (position === 'compiler') f.setArgs([...f.args, token]);
    else { const at = f.args.indexOf(f.baseArgs[2]); f.setArgs([...f.args.slice(0, at), token, ...f.args.slice(at)]); }
    assert.equal(f.args.includes(token), true); assert.equal(f.args.filter(x => x.startsWith('@')).length, 0);
    const after = custodySnapshot(f); assert.notDeepEqual(after, before, 'actual mutation before rejection catcher');
    assert.throws(() => f.execute()); assert.deepEqual(custodySnapshot(f), after, 'reader refusal preserves evidence');
  }); } finally { assert.equal(typeof ownedTmp, 'string'); assert.equal(fs.existsSync(ownedTmp), false, 'owned zero-child model cleanup verified'); }
});
