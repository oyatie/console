import assert from 'node:assert/strict';
import test from 'node:test';
import * as fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import vm from 'node:vm';
import * as crypto from 'node:crypto';

const sourcePath = process.env.LC07_HELPER_SOURCE ?? new URL('./dev-account-custody.mjs', import.meta.url);
const certificate = fs.readFileSync(new URL('./fixtures/account-custody-cert.pem', import.meta.url));
const privateKey = fs.readFileSync(new URL('./fixtures/account-custody-key.pem', import.meta.url));
const pinnedImage = 'postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280';
const transport = ['ACCOUNT_CUSTODY_PG_TLS_DIR','ACCOUNT_CUSTODY_TARGET_ENV_FILE','ACCOUNT_CUSTODY_PASSWORD_FILE','ACCOUNT_CUSTODY_CA_FILE'];
function fixture(t, options = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'lc07-fs-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, 'custody');
  const calls = [];
  const env = {};
  let containerName, ownerLabel, removalAttempted = false;
  const certObject = new crypto.X509Certificate(certificate);
  const clock = options.expired ? Date.parse(certObject.validTo) + 60_000 : Date.parse(certObject.validFrom) + 60_000;
  class FixtureDate extends Date { constructor(...args) { super(...(args.length ? args : [clock])); } static now() { return clock; } }
  const context = { ...fs, ...crypto, path, Date: FixtureDate,
    process: { env, getuid: process.getuid?.bind(process), getgid: process.getgid?.bind(process) },
    spawnSync: (...args) => {
      calls.push(args);
      assert.ok(args[2]?.timeout > 0 && args[2].timeout <= 60_000, 'every provisioning runtime call must be bounded');
      const argv = args[1];
      const command = argv[0] === 'container' ? argv[1] : argv[0];
      if (argv[0] === 'image' && argv[1] === 'inspect') {
        assert.equal(argv.at(-1), pinnedImage);
        assert.equal(argv[argv.indexOf('--format') + 1], '{{.Id}}');
        if (options.imageInspectTimeout) return { status:null,error:Object.assign(new Error('image inspect timeout'),{code:'ETIMEDOUT'}) };
        if (options.imageInspectError) return { status:1,stdout:'',stderr:options.imageInspectError };
        if (options.imageMissing) return { status:1,stdout:'',stderr:`Error response from daemon: No such image: ${pinnedImage}` };
        return { status:0,stdout:'sha256:'+'a'.repeat(64),stderr:'' };
      }
      if (argv[0] === 'pull') {
        assert.equal(argv.at(-1), pinnedImage);
        if (options.pullFailure) return {status:1,stdout:'',stderr:'synthetic pull failure'};
        if (options.pullTimeout) return {status:null,error:Object.assign(new Error('pull timeout'),{code:'ETIMEDOUT'})};
        return {status:0,stdout:'synthetic pulled image',stderr:''};
      }
      if (command === 'run') {
        containerName = argv[argv.indexOf('--name') + 1];
        ownerLabel = argv[argv.indexOf('--label') + 1];
        assert.ok(argv.includes('--name') && containerName, 'register named fixture before running');
        assert.ok(argv.includes('--label') && ownerLabel.includes('='), 'predeclared ownership label');
        assert.ok(ownerLabel.slice(ownerLabel.indexOf('=') + 1).length >= 12, 'unique predeclared custody token');
        assert.equal(argv[argv.indexOf('--network') + 1], 'none');
        assert.equal(argv[argv.indexOf('--pull') + 1], 'never');
        assert.equal(argv.includes('--volume'), false, 'strict bind must not create absent guest directories');
        assert.equal(argv[argv.indexOf('--mount') + 1], `type=bind,src=${path.join(directory,'tls')},dst=/tls`);
        if (options.missingGuestBind) return {status:125,stdout:'',stderr:'invalid mount config for type bind: bind source path does not exist'};
        if (options.timeout) return { status: null, error: Object.assign(new Error('timed out'), { code: 'ETIMEDOUT' }) };
        fs.writeFileSync(path.join(directory, 'tls/server.key'), privateKey, { mode: 0o600 });
        fs.writeFileSync(path.join(directory, 'tls/server.crt'), certificate, { mode: 0o600 });
        return { status: 0, stdout: '', stderr: '' };
      }
      assert.equal(argv.at(-1), containerName, 'cleanup must address only predeclared owned container');
      if (command === 'inspect' && argv.includes('--format')) {
        if (!options.timeout || removalAttempted) return { status: 1, stdout: '[]', stderr: `Error response from daemon: No such container: ${containerName}` };
        const key = ownerLabel.slice(0,ownerLabel.indexOf('='));
        return { status: 0, stdout: JSON.stringify({[key]: options.unrelatedOwner ? 'unrelated' : ownerLabel.slice(ownerLabel.indexOf('=') + 1)}), stderr: '' };
      }
      if (command === 'rm') {
        removalAttempted = true;
        assert.equal(options.unrelatedOwner, undefined, 'must never remove unrelated container');
        if (options.removeTimeout) return { status: null, error: Object.assign(new Error('lost removal response'), {code:'ETIMEDOUT'}) };
        return { status: 0, stdout: containerName, stderr: '' };
      }
      if (command === 'inspect') return { status: 1, stdout:'[]', stderr:`Error response from daemon: No such container: ${containerName}` };
      assert.fail(`unexpected runtime command ${command}`);
    },
  };
  vm.createContext(context);
  const source = fs.readFileSync(sourcePath, 'utf8').replace(/^import[\s\S]*?;\n/gm, '').replace(/^export /gm, '');
  vm.runInContext(source + '\nglobalThis.prepare = prepareLocalCustody; globalThis.retain = retainLocalCustodyInventory;', context);
  return { root, directory, calls, env, prepare: () => context.prepare(directory, 'synthetic-runtime', options.password ?? 'synthetic:pass\\value'), retain: context.retain };
}
function forgetExportedPaths(f) { for (const key of transport) delete f.env[key]; }

test('fresh provisioning uses bounded runtime and private files; retained inventory is immutable', (t) => {
  const f = fixture(t); const descriptor = f.prepare();
  for (const item of [f.directory, path.join(f.directory, 'tls')]) assert.equal(fs.statSync(item).mode & 0o777, 0o700);
  for (const item of [descriptor, f.env.ACCOUNT_CUSTODY_PASSWORD_FILE, path.join(f.directory,'tls/server.key')]) assert.equal(fs.statSync(item).mode & 0o777, 0o600);
  assert.equal(fs.readFileSync(f.env.ACCOUNT_CUSTODY_PASSWORD_FILE,'utf8'), 'synthetic:pass\\value');
  f.retain(descriptor, '12345|16384\n', 'operator', 'console');
  const before = fs.readFileSync(descriptor);
  f.retain(descriptor, '12345|16384\n', 'operator', 'console');
  for (const [output, operator, database] of [['54321|16384','operator','console'],['12345|16385','operator','console'],['12345|16384','other','console'],['12345|16384','operator','other']]) {
    assert.throws(() => f.retain(descriptor, output, operator, database), /changed|refused/);
    assert.deepEqual(fs.readFileSync(descriptor), before);
  }
});
test('second invocation reuses certificate and password without rewriting retained inventory', (t) => {
  const f = fixture(t); const descriptor = f.prepare(); f.retain(descriptor,'12345|16384','operator','console');
  const files = [descriptor, path.join(f.directory,'password'), path.join(f.directory,'tls/server.key'),path.join(f.directory,'tls/server.crt')];
  const before = files.map(p=>fs.readFileSync(p)); forgetExportedPaths(f);
  assert.equal(f.prepare(), descriptor);
  files.forEach((p,i)=>assert.deepEqual(fs.readFileSync(p),before[i]));
  assert.equal(f.calls.filter(call=>call[1].at(-1).includes('openssl req')).length,1);
});
test('partial externally supplied transport fails before provisioning', (t) => {
  const f = fixture(t); f.env.ACCOUNT_CUSTODY_CA_FILE='/supplied/ca';
  assert.throws(f.prepare,/all four|partial|transport/); assert.equal(f.calls.length,0); assert.equal(fs.existsSync(f.directory),false);
});
test('timeout refuses publication of local operator paths', (t) => {
  const f = fixture(t,{timeout:true}); assert.throws(f.prepare,/provision|timeout/);
  assert.ok(f.calls.length >= 3); assert.equal(f.calls.at(-1)[1].at(-1), f.calls.find(c=>c[1][0]==='run')[1][f.calls.find(c=>c[1][0]==='run')[1].indexOf('--name') + 1]); for (const key of transport) assert.equal(f.env[key],undefined);
});
for(const password of ['', 'line\nbreak', 'carriage\rreturn']) test(`invalid password ${JSON.stringify(password)} rejected`, (t)=> {
  const f=fixture(t,{password});assert.throws(f.prepare,/password|credential/);
});
test('custody directory symlink is refused without modifying unrelated target', (t)=> {
  const f=fixture(t); const victim=path.join(f.root,'unrelated');fs.mkdirSync(victim,{mode:0o755});fs.symlinkSync(victim,f.directory);
  assert.throws(f.prepare,/symlink|directory|custody|unsafe/);assert.equal(fs.statSync(victim).mode & 0o777,0o755);assert.deepEqual(fs.readdirSync(victim),[]);
});
test('empty descriptor symlink is refused without writing unrelated target', (t)=> {
  const f=fixture(t);const victim=path.join(f.root,'unrelated');fs.writeFileSync(victim,'',{mode:0o600});const descriptor=path.join(f.root,'target.env');fs.symlinkSync(victim,descriptor);
  assert.throws(()=>f.retain(descriptor,'12345|16384','operator','console'),/symlink|unsafe|regular|descriptor|inventory/);assert.equal(fs.readFileSync(victim,'utf8'),'');
});
test('group-readable retained password refused rather than silently reused', (t)=> {
  const f=fixture(t);f.prepare();forgetExportedPaths(f);fs.chmodSync(path.join(f.directory,'password'),0o640);
  assert.throws(f.prepare,/permission|private|mode|unsafe/);
});
test('partial existing certificate pair is refused rather than overwriting retained identity', (t)=> {
  const f=fixture(t);f.prepare();forgetExportedPaths(f);fs.unlinkSync(path.join(f.directory,'tls/server.key'));
  const before=fs.readFileSync(path.join(f.directory,'tls/server.crt'));assert.throws(f.prepare,/certificate|TLS|partial|key/);assert.deepEqual(fs.readFileSync(path.join(f.directory,'tls/server.crt')),before);
});
test('corrupt retained certificate refuses reuse', (t)=> {
  const f=fixture(t);
  fs.mkdirSync(path.join(f.directory,'tls'),{recursive:true,mode:0o700});fs.writeFileSync(path.join(f.directory,'tls/server.key'),privateKey,{mode:0o600});fs.writeFileSync(path.join(f.directory,'tls/server.crt'),'invalid certificate',{mode:0o600});
  assert.throws(f.prepare,/certificate|TLS|expired|valid|PEM|DECODER/i);
});

test('retained password mismatch refuses credential rotation without overwrite', (t)=> {
  const f=fixture(t);f.prepare();forgetExportedPaths(f);const file=path.join(f.directory,'password');fs.writeFileSync(file,'different-retained-secret',{mode:0o600});
  assert.throws(f.prepare,/password|credential/);assert.equal(fs.readFileSync(file,'utf8'),'different-retained-secret');
});
test('malformed inventory capture never replaces previously frozen descriptor', (t)=> {
  const f=fixture(t);const descriptor=f.prepare();f.retain(descriptor,'12345|16384','operator','console');const before=fs.readFileSync(descriptor);
  for(const output of ['12345|16384\n67890|1','0|16384','12345|0','unexpected']) {
    assert.throws(()=>f.retain(descriptor,output,'operator','console'),/inventory|readback/);assert.deepEqual(fs.readFileSync(descriptor),before);
  }
});


test('expired certificate fails without silent renewal', (t)=> {
  const f=fixture(t,{expired:true});fs.mkdirSync(path.join(f.directory,'tls'),{recursive:true,mode:0o700});
  fs.writeFileSync(path.join(f.directory,'tls/server.key'),privateKey,{mode:0o600});fs.writeFileSync(path.join(f.directory,'tls/server.crt'),certificate,{mode:0o600});
  assert.throws(f.prepare,/certificate|TLS|expired|valid/i);assert.equal(f.calls.length,0);
});
test('different private key cannot reuse retained certificate', (t)=> {
  const f=fixture(t);fs.mkdirSync(path.join(f.directory,'tls'),{recursive:true,mode:0o700});
  const other=crypto.generateKeyPairSync('rsa',{modulusLength:2048}).privateKey.export({type:'pkcs8',format:'pem'});
  fs.writeFileSync(path.join(f.directory,'tls/server.key'),other,{mode:0o600});fs.writeFileSync(path.join(f.directory,'tls/server.crt'),certificate,{mode:0o600});
  assert.throws(f.prepare,/certificate|TLS|key|match/i);assert.equal(f.calls.length,0);
});
test('lost create response still performs owner-scoped removal and absence inspection', (t)=> {
  const f=fixture(t,{timeout:true});assert.throws(f.prepare,/provision|cleanup|timeout/i);
  const commands=f.calls.filter(c=>!['image','pull'].includes(c[1][0])).map(c=>c[1][0]==='container'?c[1][1]:c[1][0]);
  assert.deepEqual(commands,['run','inspect','rm','inspect']);
});
test('lost removal response still inspects absence and never publishes operator paths', (t)=> {
  const f=fixture(t,{timeout:true,removeTimeout:true});assert.throws(f.prepare,/provision|cleanup|timeout|remov/i);
  const commands=f.calls.filter(c=>!['image','pull'].includes(c[1][0])).map(c=>c[1][0]==='container'?c[1][1]:c[1][0]);assert.deepEqual(commands,['run','inspect','rm','inspect']);
  for(const key of transport)assert.equal(f.env[key],undefined);
});
test('unrelated ownership after lost create response refuses removal', (t)=> {
  const f=fixture(t,{timeout:true,unrelatedOwner:true});assert.throws(f.prepare,/owner|unrelated|custody|cleanup/i);
  assert.equal(f.calls.some(c=>c[1].includes('rm')),false);
});

test('dangling TLS key symlink refused before external target can be created', (t)=> {
  const f=fixture(t);fs.mkdirSync(path.join(f.directory,'tls'),{recursive:true,mode:0o700});
  const victim=path.join(f.root,'outside-key');fs.symlinkSync(victim,path.join(f.directory,'tls/server.key'));
  assert.throws(f.prepare,/symlink|unsafe|partial|TLS|path/i);
  assert.equal(fs.existsSync(victim),false,'provisioner followed dangling symlink outside custody');
  assert.equal(f.calls.length,0,'reject before starting provisioning runtime');
});


test('present pinned image is inspected once and never pulled', (t)=> {
  const f=fixture(t);f.prepare();assert.equal(f.calls[0][1][0],'image');
  assert.equal(f.calls.filter(c=>c[1][0]==='image').length,1);assert.equal(f.calls.some(c=>c[1][0]==='pull'),false);
});
test('exact missing pinned image pulls before isolated provisioning', (t)=> {
  const f=fixture(t,{imageMissing:true});f.prepare();
  assert.deepEqual(f.calls.slice(0,3).map(c=>c[1][0]),['image','pull','run']);
});
for(const failure of [
  {imageInspectError:'Cannot connect to the Docker daemon'},
  {imageInspectError:'Error response from daemon: No such image: unrelated:latest'},
  {imageInspectTimeout:true},
]) test(`image inspection refusal ${JSON.stringify(failure)} never pulls or provisions`, (t)=> {
  const f=fixture(t,failure);assert.throws(f.prepare,/image|inspect|runtime|unavailable/i);
  assert.equal(f.calls.length,1);assert.equal(f.calls[0][1][0],'image');
  for(const key of transport) assert.equal(f.env[key],undefined);
});
for(const failure of [{pullFailure:true},{pullTimeout:true}]) test(`pinned image pull refusal ${JSON.stringify(failure)} never provisions`, (t)=> {
  const f=fixture(t,{imageMissing:true,...failure});assert.throws(f.prepare,/pull|image/i);
  assert.deepEqual(f.calls.map(c=>c[1][0]),['image','pull']);
  for(const key of transport)assert.equal(f.env[key],undefined);
});
test('missing guest bind fails without published files and confirms named container absence', (t)=> {
  const f=fixture(t,{missingGuestBind:true});assert.throws(f.prepare,/TLS|provision|bind/i);
  assert.equal(fs.existsSync(path.join(f.directory,'tls/server.key')),false);
  assert.equal(f.calls.at(-1)[1][0],'container');assert.equal(f.calls.at(-1)[1][1],'inspect');
  for(const key of transport)assert.equal(f.env[key],undefined);
});


test('CA certificate cannot be reused as PostgreSQL TLS leaf', (t)=> {
  const f=fixture(t);fs.mkdirSync(path.join(f.directory,'tls'),{recursive:true,mode:0o700});
  const caCertificate=fs.readFileSync(new URL('./fixtures/account-custody-ca-cert.pem',import.meta.url));
  const ca=new crypto.X509Certificate(caCertificate);assert.equal(ca.ca,true);
  const now=Date.parse(new crypto.X509Certificate(certificate).validFrom)+60_000;
  assert.ok(Date.parse(ca.validFrom)<=now && Date.parse(ca.validTo)>now+86400_000,'negative CA fixture remains otherwise time-valid');
  assert.ok(ca.checkHost('postgres'));assert.ok(ca.checkIP('127.0.0.1'));
  assert.equal(ca.checkPrivateKey(crypto.createPrivateKey(privateKey)),true,'negative fixture differs in certificate constraints, not key identity');
  fs.writeFileSync(path.join(f.directory,'tls/server.key'),privateKey,{mode:0o600});fs.writeFileSync(path.join(f.directory,'tls/server.crt'),caCertificate,{mode:0o600});
  assert.throws(f.prepare,/certificate|TLS|leaf|CA|valid/i);assert.equal(f.calls.length,0);
});
