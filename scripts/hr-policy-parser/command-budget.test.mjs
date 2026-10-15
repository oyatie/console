// Fake-only actual command-owner controls; never launches Buck or a compiler.
import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import childProcess from 'node:child_process';
import { syncBuiltinESMExports } from 'node:module';
import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { testOnlyBindingModel } from './fixtures/command-budget-v1.mjs';

const actualSpawnSync = childProcess.spawnSync;
const externalKeys = ['CONSOLE_HR_BUDGET_OWNER_MODULE', 'CONSOLE_HR_BUDGET_OWNER_SHA256',
  'CONSOLE_HR_BOOTSTRAP_FIXTURE', 'CONSOLE_HR_BOOTSTRAP_FIXTURE_SHA256'];
const external = externalKeys.some(key => process.env[key] !== undefined);
assert(!external || externalKeys.every(key => process.env[key]), 'external binding requires all four exact inputs');
const owner = process.env.CONSOLE_HR_BUDGET_OWNER_MODULE ?? fileURLToPath(new URL('./native-run.mjs', import.meta.url));
assert(owner && path.isAbsolute(owner), 'exact current owner path required');
assert(owner.endsWith('/scripts/hr-policy-parser/native-run.mjs'));
assert.equal(fs.realpathSync(owner), owner);
assert(!fs.lstatSync(owner).isSymbolicLink() && fs.lstatSync(owner).isFile());
const observedOwnerSha = crypto.createHash('sha256').update(fs.readFileSync(owner)).digest('hex');
const ownerSha = process.env.CONSOLE_HR_BUDGET_OWNER_SHA256 ?? observedOwnerSha;
assert(/^[a-f0-9]{64}$/.test(ownerSha), 'external pin or observed current source hash required');
assert.equal(observedOwnerSha, ownerSha);
const { command } = await import(pathToFileURL(owner));
const { ROOT, TARGET, LEGACY } = await import(pathToFileURL(path.join(path.dirname(owner), 'binding.mjs')));
const binding = { dotslash: { path: '/reviewed/dotslash/bin/dotslash' },
  dotslash_cache: '/reviewed/dotslash-cache', timeout_ms: 12345 };
const isolation = 'hr-policy-0123456789abcdef0123456789abcdef';

function capture(t, argv) {
  const calls = [];
  t.mock.method(childProcess, 'spawnSync', (...args) => {
    calls.push(args);
    return { status: 0, signal: null, stdout: '', stderr: '' };
  });
  syncBuiltinESMExports();
  try {
    const result = command(binding, isolation, argv);
    assert.equal(result.status, 0);
    assert.equal(calls.length, 1);
    const [executable, args, options] = calls[0];
    assert.equal(executable, path.join(ROOT, 'tools/buck2'));
    assert.equal(options.cwd, ROOT);
    assert.equal(options.timeout, 12345);
    assert.equal(options.maxBuffer, 32 * 1024 * 1024);
    assert.equal(options.env.PATH, '/usr/bin:/bin:/reviewed/dotslash/bin');
    assert.equal(options.env.DOTSLASH_CACHE, '/reviewed/dotslash-cache');
    return args;
  } finally {
    t.mock.restoreAll();
    syncBuiltinESMExports();
  }
}

test('FAKE ONLY actual target build receives explicit one-worker CLI budget', t => {
  const argv = ['build', TARGET, '--build-report', '/owned/build-report.json'];
  assert.deepEqual(capture(t, argv), ['--isolation-dir', isolation, 'build', '-j', '1', ...argv.slice(1)]);
});

test('FAKE ONLY actual legacy run receives explicit one-worker CLI budget before test separator', t => {
  const argv = ['run', LEGACY, '--', '--list'];
  assert.deepEqual(capture(t, argv), ['--isolation-dir', isolation, 'run', '-j', '1', ...argv.slice(1)]);
});

test('FAKE ONLY actual what-ran inspection keeps its command and scoped isolation unchanged', t => {
  const argv = ['log', 'what-ran', '--format', 'json', '/owned/events.pb.zst'];
  assert.deepEqual(capture(t, argv), ['--isolation-dir', isolation, ...argv]);
});

test('FAKE ONLY actual owned daemon cleanup keeps its command and scoped isolation unchanged', t => {
  assert.deepEqual(capture(t, ['kill']), ['--isolation-dir', isolation, 'kill']);
});

// Metadata/filesystem machinery controls only; the actual binder never launches tools.
const { loadBinding } = await import(pathToFileURL(path.join(path.dirname(owner), 'binding.mjs')));
const os = await import('node:os');
const model = testOnlyBindingModel(ROOT);
let base = model.binding;
if (external) {
  const fixture = process.env.CONSOLE_HR_BOOTSTRAP_FIXTURE;
  assert(path.isAbsolute(fixture), 'exact existing binding fixture required');
  assert.equal(fs.realpathSync(fixture), fixture);
  assert(fs.lstatSync(fixture).isFile() && !fs.lstatSync(fixture).isSymbolicLink());
  const fixtureSha = process.env.CONSOLE_HR_BOOTSTRAP_FIXTURE_SHA256;
  assert(/^[a-f0-9]{64}$/.test(fixtureSha), 'external reviewed binding fixture hash required');
  const fixtureBytes = fs.readFileSync(fixture);
  assert.equal(crypto.createHash('sha256').update(fixtureBytes).digest('hex'), fixtureSha);
  base = JSON.parse(fixtureBytes);
}
assert.equal(base.cwd, ROOT);
base.bootstrap_utilities = model.utilities;
const restoreModel = model.install();
after(restoreModel);

function withBinding(edit, body) {
  const dir = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'hr-bootstrap-binding-control-')));
  try {
    const value = structuredClone(base);
    edit(value);
    const file = path.join(dir, 'binding.json');
    const bytes = Buffer.from(JSON.stringify(value));
    fs.writeFileSync(file, bytes, {mode:0o600});
    const env = {...process.env, CONSOLE_HR_POLICY_BINDING:file,
      CONSOLE_HR_POLICY_BINDING_SHA256:crypto.createHash('sha256').update(bytes).digest('hex'), CONSOLE_HR_POLICY_NODE:value.node.path};
    body(() => loadBinding(env), value);
  } finally { fs.rmSync(dir, {recursive:true}); }
}

test('MACHINERY ONLY actual binder accepts authenticated normal mkdir and direct tar lookup alias', () => {
  withBinding(() => {}, (load, value) => assert.deepEqual(load().bootstrap_utilities, value.bootstrap_utilities));
});
for (const changed of ['missing', 'missing-mkdir', 'missing-tar', 'unknown', 'array', 'extra-row-field']) {
  test(`MACHINERY ONLY actual binder rejects ${changed} bootstrap utility metadata`, () => {
    withBinding(value => {
      if (changed === 'missing') delete value.bootstrap_utilities;
      else if (changed === 'missing-mkdir') delete value.bootstrap_utilities.mkdir;
      else if (changed === 'missing-tar') delete value.bootstrap_utilities.tar;
      else if (changed === 'unknown') value.bootstrap_utilities.curl = value.bootstrap_utilities.tar;
      else if (changed === 'array') value.bootstrap_utilities = Object.values(value.bootstrap_utilities);
      else value.bootstrap_utilities.tar.unreviewed_argument = '--unknown';
    }, load => assert.throws(load));
  });
}
for (const utility of ['mkdir', 'tar']) {
  test(`MACHINERY ONLY actual binder rejects changed ${utility} executable bytes`, () => {
    withBinding(value => {value.bootstrap_utilities[utility].sha256 = '0'.repeat(64);}, load => assert.throws(load));
  });
  test(`MACHINERY ONLY actual binder rejects mismatched ${utility} lookup and canonical identity`, () => {
    withBinding(value => {
      const other = value.bootstrap_utilities[utility === 'mkdir' ? 'tar' : 'mkdir'];
      Object.assign(value.bootstrap_utilities[utility], {path:other.path, sha256:other.sha256});
    }, load => assert.throws(load));
  });
  test(`MACHINERY ONLY actual binder rejects nonexecutable ${utility}`, t => {
    const access = fs.accessSync.bind(fs);
    t.mock.method(fs, 'accessSync', (file, mode) => {
      if ([base.bootstrap_utilities[utility].path, base.bootstrap_utilities[utility].lookup_path].includes(file)) throw Object.assign(new Error('machinery EACCES'), {code:'EACCES'});
      return access(file, mode);
    });
    withBinding(() => {}, load => assert.throws(load));
  });
}
for (const [utility, lookup] of [['tar', '/reviewed/dotslash/bin/tar'], ['tar', '/untrusted/ambient/tar'],
  ['tar', '/bin/tar'], ['tar', '../tar'], ['mkdir', '/reviewed/dotslash/bin/mkdir'], ['mkdir', '/untrusted/ambient/mkdir']]) {
  test(`MACHINERY ONLY actual binder rejects unapproved ${utility} lookup ${lookup}`, () => {
    withBinding(value => {value.bootstrap_utilities[utility].lookup_path = lookup;}, load => assert.throws(load));
  });
}
for (const changed of ['wrong-target', 'chain', 'linked-canonical', 'directory-canonical']) {
  test(`MACHINERY ONLY actual binder rejects tar ${changed}`, t => {
    const readlink = fs.readlinkSync.bind(fs), lstat = fs.lstatSync.bind(fs);
    if (changed === 'wrong-target' || changed === 'chain') {
      t.mock.method(fs, 'readlinkSync', file => file === '/usr/bin/tar' ? (changed === 'chain' ? 'tar-next' : '../bin/bsdtar') : readlink(file));
    } else {
      t.mock.method(fs, 'lstatSync', file => file === '/usr/bin/bsdtar' ?
        {isFile:() => changed !== 'directory-canonical', isSymbolicLink:() => changed === 'linked-canonical'} : lstat(file));
    }
    withBinding(() => {}, load => assert.throws(load));
  });
}
test('MACHINERY ONLY actual binder never selects or reads unpinned DotSlash tar shadow', t => {
  const shadow = path.join(path.dirname(base.dotslash.path), 'tar');
  const exists = fs.existsSync.bind(fs), read = fs.readFileSync.bind(fs);
  t.mock.method(fs, 'existsSync', file => file === shadow ? true : exists(file));
  t.mock.method(fs, 'readFileSync', (...args) => {
    assert.notEqual(args[0], shadow, 'unpinned lower-priority utility must never supply executable bytes');
    return read(...args);
  });
  withBinding(() => {}, (load, value) => assert.equal(load().bootstrap_utilities.tar.lookup_path, value.bootstrap_utilities.tar.lookup_path));
});
test('MACHINERY ONLY actual binder rejects higher-priority system mkdir shadow before reading its bytes', t => {
  const shadow = '/usr/bin/mkdir';
  let shadowReads = 0;
  const exists = fs.existsSync.bind(fs), access = fs.accessSync.bind(fs), lstat = fs.lstatSync.bind(fs);
  const stat = fs.statSync.bind(fs), realpath = fs.realpathSync.bind(fs), read = fs.readFileSync.bind(fs);
  const fakeStat = {isFile:() => true, isDirectory:() => false, isSymbolicLink:() => false, mode:0o100755};
  t.mock.method(fs, 'existsSync', file => file === shadow || exists(file));
  t.mock.method(fs, 'accessSync', (file, mode) => {if (file !== shadow) return access(file, mode);});
  t.mock.method(fs, 'lstatSync', file => file === shadow ? fakeStat : lstat(file));
  t.mock.method(fs, 'statSync', file => file === shadow ? fakeStat : stat(file));
  t.mock.method(fs, 'realpathSync', file => file === shadow ? shadow : realpath(file));
  t.mock.method(fs, 'readFileSync', (...args) => {
    if (args[0] === shadow) shadowReads++;
    assert.notEqual(args[0], shadow, 'unpinned first-match utility must be rejected before its bytes are read');
    return read(...args);
  });
  withBinding(() => {}, load => assert.throws(load));
  assert.equal(shadowReads, 0, 'reading an unpinned candidate must not be mistaken for correct rejection');
});
test('FAKE ONLY actual command excludes ambient PATH while retaining all budgets and isolation', t => {
  const ambient = process.env.PATH;
  try {
    process.env.PATH = '/untrusted/ambient/bin';
    assert.deepEqual(capture(t, ['run', LEGACY, '--', '--list']), ['--isolation-dir', isolation, 'run', '-j', '1', LEGACY, '--', '--list']);
  } finally {
    if (ambient === undefined) delete process.env.PATH;
    else process.env.PATH = ambient;
  }
});

// Lookup machinery only: no system paths are written and spawn stays mocked.
function withDotSlashLookup(t,directory,kind,body) {
  const shadow=path.join(directory,'dotslash'),calls=[];let shadowReads=0;
  const stat=fs.statSync.bind(fs),lstat=fs.lstatSync.bind(fs),access=fs.accessSync.bind(fs);
  const exists=fs.existsSync.bind(fs),realpath=fs.realpathSync.bind(fs),read=fs.readFileSync.bind(fs);
  const fake={isFile:()=>kind!=='nonregular',isDirectory:()=>kind==='nonregular',isSymbolicLink:()=>false,mode:kind==='nonregular'?0o40755:0o100755};
  t.mock.method(fs,'statSync',file=>file===shadow?fake:stat(file));
  t.mock.method(fs,'lstatSync',file=>file===shadow?fake:lstat(file));
  t.mock.method(fs,'existsSync',file=>file===shadow||exists(file));
  t.mock.method(fs,'realpathSync',file=>file===shadow?shadow:realpath(file));
  t.mock.method(fs,'accessSync',(file,mode)=>{
    if(file!==shadow)return access(file,mode);
    if(kind==='nonexecutable')throw Object.assign(new Error('machinery EACCES'),{code:'EACCES'});
  });
  t.mock.method(fs,'readFileSync',(...args)=>{
    if(args[0]===shadow)shadowReads++;
    assert.notEqual(args[0],shadow,'unpinned DotSlash candidate must never supply executable bytes');
    return read(...args);
  });
  t.mock.method(childProcess,'spawnSync',(...args)=>{calls.push(args);return {status:0,signal:null,stdout:'',stderr:''};});
  syncBuiltinESMExports();
  try {body(shadow,calls);assert.equal(shadowReads,0,'shadow byte-read refusal must not pass through the caught loader assertion');}
  finally {t.mock.restoreAll();syncBuiltinESMExports();}
}
function assertPinnedDotSlashDispatch(t) {
  const calls=[];
  t.mock.method(childProcess,'spawnSync',(...args)=>{calls.push(args);return {status:0,signal:null,stdout:'',stderr:''};});
  syncBuiltinESMExports();
  try {
    withBinding(()=>{},(load,value)=>{
      const loaded=load(),selectedPATH=`/usr/bin:/bin:${path.dirname(value.dotslash.path)}`;
      const first=selectedPATH.split(':').map(directory=>path.join(directory,'dotslash')).find(file=>{
        try {if(!fs.statSync(file).isFile())return false;fs.accessSync(file,fs.constants.X_OK);return true;}catch{return false;}
      });
      assert.equal(first,value.dotslash.path,'first executable is the pinned canonical DotSlash');
      assert.equal(fs.realpathSync(first),first);assert(fs.lstatSync(first).isFile()&&!fs.lstatSync(first).isSymbolicLink());
      assert.equal(crypto.createHash('sha256').update(fs.readFileSync(first)).digest('hex'),value.dotslash.sha256);
      assert(fs.readFileSync(path.join(ROOT,'tools/buck2'),'utf8').startsWith('#!/usr/bin/env dotslash\n'));
      assert.equal(command(loaded,isolation,['build',TARGET]).status,0);
      assert.equal(calls.length,1);assert.equal(calls[0][2].env.PATH,selectedPATH);
      assert.deepEqual(calls[0][1],['--isolation-dir',isolation,'build','-j','1',TARGET]);
    });
  }finally{t.mock.restoreAll();syncBuiltinESMExports();}
}
test('MACHINERY ONLY actual binder and command select current first pinned DotSlash',t=>assertPinnedDotSlashDispatch(t));
for(const directory of ['/usr/bin','/bin']) {
  test(`MACHINERY ONLY actual binder rejects higher-priority executable DotSlash ${directory}`,t=>{
    withDotSlashLookup(t,directory,'executable',(shadow,calls)=>{
      const selectedPATH=`/usr/bin:/bin:${path.dirname(base.dotslash.path)}`;
      const first=selectedPATH.split(':').map(dir=>path.join(dir,'dotslash')).find(file=>{
        try{if(!fs.statSync(file).isFile())return false;fs.accessSync(file,fs.constants.X_OK);return true;}catch{return false;}
      });
      assert.equal(first,shadow,'control actually shadows declared DotSlash');
      withBinding(()=>{},load=>assert.throws(load,'actual loader rejects first unpinned executable'));
      assert.equal(calls.length,0,'no command dispatch follows refused executable custody');
    });
  });
  for(const kind of ['nonexecutable','nonregular'])
    test(`MACHINERY ONLY actual binder skips ${kind} DotSlash lookup ${directory}`,t=>{
      withDotSlashLookup(t,directory,kind,()=>assertPinnedDotSlashDispatch(t));
    });
}

// Measured verification metadata is not implementation admission authority.
test('MACHINERY ONLY actual binder accepts measured current CI inputs', () => {
  withBinding(value => {value.status = 'MEASURED_CURRENT_CI_INPUTS';},
    load => assert.equal(load().status, 'MEASURED_CURRENT_CI_INPUTS'));
});
test('MACHINERY ONLY actual binder rejects unknown input status', () => {
  withBinding(value => {value.status = 'UNKNOWN_INPUTS';}, load => assert.throws(load));
});
test('MACHINERY ONLY actual admission qualifier refuses measured and unknown inputs before lane admission', () => {
  const dir = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'hr-measured-admission-control-')));
  try {
    for (const status of ['MEASURED_CURRENT_CI_INPUTS', 'UNKNOWN_INPUTS']) {
      const bytes = Buffer.from(JSON.stringify({...base, status})), file = path.join(dir, 'binding.json');
      fs.writeFileSync(file, bytes, {mode:0o600});
      const result = actualSpawnSync('python3', [path.join(ROOT, 'scripts/hr-policy-parser/qualify-admission.py'),
        '--binding', file, '--binding-sha256', crypto.createHash('sha256').update(bytes).digest('hex')],
        {cwd:ROOT, env:{PATH:process.env.PATH}, encoding:'utf8', timeout:10000});
      assert.equal(result.status, 126); assert.equal(result.signal, null); assert.equal(result.stdout, '');
      assert.match(result.stderr, /binding is not an independently reviewed exact candidate/);
    }
  } finally { fs.rmSync(dir, {recursive:true}); }
});

for (const retain of [false, true]) {
  test(`MACHINERY ONLY filesystem fixture ${retain ? 'resource oracle detects deliberate retention' : 'bounds retained bytes and restores native functions'}`, () => {
    const result = actualSpawnSync(process.execPath, ['--expose-gc', '--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import path from 'node:path';
      import crypto from 'node:crypto';
      import { testOnlyBindingModel } from './scripts/hr-policy-parser/fixtures/command-budget-v1.mjs';
      const methods = ['existsSync', 'accessSync', 'lstatSync', 'statSync', 'realpathSync', 'readFileSync', 'readlinkSync'];
      const originals = methods.map(name => [name, Object.getOwnPropertyDescriptor(fs, name)]);
      const native = Object.getOwnPropertyDescriptor(fs.realpathSync, 'native');
      const nodeBytes = fs.statSync(fs.realpathSync(process.execPath)).size;
      assert(nodeBytes > 1024 * 1024, 'real current Node executable is the resource input');
      const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
      const model = testOnlyBindingModel(process.cwd()), restore = model.install();
      const retained = [];
      let measured;
      try {
        assert.deepEqual(Object.getOwnPropertyDescriptor(fs.realpathSync, 'native'), native);
        const source = model.binding.sources[0], file = path.join(process.cwd(), source.path);
        let bytes = fs.readFileSync(file);
        assert.equal(hash(bytes), source.sha256);
        bytes[0] ^= 1; bytes = null;
        assert.equal(hash(fs.readFileSync(file)), source.sha256, 'pass-through reads remain fresh');
        global.gc(); global.gc();
        const baseline = process.memoryUsage().arrayBuffers;
        for (let index = 0; index < 3; index++) {
          let value = fs.readFileSync(model.binding.python.path);
          assert.equal(value.byteLength, nodeBytes);
          assert.equal(hash(value), model.binding.python.sha256, 'actual current tool bytes and pin');
          value[0] ^= 1;
          if (${retain}) retained.push(value);
          value = null;
        }
        global.gc(); global.gc();
        measured = { nodeBytes, reads: 3, growth: process.memoryUsage().arrayBuffers - baseline,
          budget: 2 * nodeBytes + 1024 * 1024,
          injected: retained.reduce((total, bytes) => total + bytes.byteLength, 0) };
      } finally { restore(); }
      for (const [name, descriptor] of originals) {
        assert.equal(fs[name], descriptor.value, name + ' function identity restored');
        assert.deepEqual(Object.getOwnPropertyDescriptor(fs, name), descriptor);
      }
      assert.deepEqual(Object.getOwnPropertyDescriptor(fs.realpathSync, 'native'), native);
      process.stdout.write(JSON.stringify(measured));
    `], { cwd: ROOT, encoding: 'utf8', timeout: 15_000, maxBuffer: 1024 * 1024 });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    const measured = JSON.parse(result.stdout);
    assert.equal(measured.reads, 3);
    assert.equal(measured.injected, retain ? 3 * measured.nodeBytes : 0);
    const detail = `retained buffer growth ${measured.growth} exceeds budget ${measured.budget} for Node bytes ${measured.nodeBytes}`;
    if (retain) assert.ok(measured.growth > measured.budget, 'injected retained buffers must fail the same resource bound: ' + detail);
    else assert.ok(measured.growth <= measured.budget, detail);
  });
}
