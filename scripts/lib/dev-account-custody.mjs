// Local developer provisioning only. Production inventory is supplied by its
// independent operator. Never discover expected identity through finalizer TCP.
import { mkdirSync, readFileSync, writeFileSync, lstatSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { X509Certificate, createPrivateKey, randomUUID } from 'node:crypto';

const image = 'postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280';
const keys = ['ACCOUNT_CUSTODY_PG_TLS_DIR', 'ACCOUNT_CUSTODY_TARGET_ENV_FILE',
  'ACCOUNT_CUSTODY_PASSWORD_FILE', 'ACCOUNT_CUSTODY_CA_FILE'];

function exists(file) {
  try { lstatSync(file); return true; }
  catch (error) { if (error.code === 'ENOENT') return false; throw error; }
}

function privatePath(file, directory = false) {
  const stat = lstatSync(file);
  if (stat.isSymbolicLink() || (directory ? !stat.isDirectory() : !stat.isFile()) ||
      (process.getuid && stat.uid !== process.getuid()) || (stat.mode & 0o077))
    throw new Error('unsafe Account custody private path or permissions');
}

function provisionCertificate(tls, runtime) {
  const cached = spawnSync(runtime, ['image', 'inspect', '--format', '{{.Id}}', image], { encoding: 'utf8', timeout: 30_000 });
  if (cached.error || cached.status !== 0) {
    const missing = !cached.error && String(cached.stderr).split('\n').some(line =>
      ['Error: No such image: ', 'Error response from daemon: No such image: '].some(prefix => line.trim() === prefix + image));
    if (!missing) throw new Error('Account custody database image inspection failed');
    const pulled = spawnSync(runtime, ['pull', image], { encoding: 'utf8', timeout: 60_000 });
    if (pulled.error || pulled.status !== 0) throw new Error('Account custody database image preparation failed');
  }
  const owner = randomUUID();
  const name = `console-dev-tls-${owner}`;
  const run = (args, timeout = 30_000) => spawnSync(runtime, args, { encoding: 'utf8', timeout });
  const inspect = () => {
    const result = run(['container', 'inspect', '--format', '{{json .Config.Labels}}', name]);
    if (!result.error && result.status === 0) {
      const labels = JSON.parse(result.stdout);
      if (labels?.['console.dev-tls-owner'] !== owner) throw new Error('Account custody TLS container ownership mismatch');
      return true;
    }
    const absent = !result.error && result.status !== 0 &&
      String(result.stderr).split('\n').some(line =>
        ['Error: No such container: ', 'Error: No such object: ', 'Error response from daemon: No such container: ', 'Error response from daemon: No such object: '].some(prefix => line.trim() === prefix + name));
    if (!absent) throw new Error('Account custody TLS container absence unverified');
    return false;
  };
  try {
    const user = process.getuid ? ['--user', `${process.getuid()}:${process.getgid()}`] : [];
    const result = run(['run', '--rm', '--pull', 'never', '--name', name,
      '--label', `console.dev-tls-owner=${owner}`, '--network', 'none', ...user,
      '--mount', `type=bind,src=${tls},dst=/tls`, '--entrypoint', 'bash', image, '-ceu',
      'umask 077; openssl req -x509 -newkey rsa:2048 -nodes -days 30 -subj /CN=postgres -addext basicConstraints=critical,CA:FALSE -addext subjectAltName=DNS:postgres,DNS:localhost,IP:127.0.0.1 -addext extendedKeyUsage=serverAuth -keyout /tls/server.key -out /tls/server.crt'], 60_000);
    if (result.error || result.status !== 0) throw new Error('local Account custody TLS provisioning failed');
  } finally {
    if (inspect()) {
      try { run(['rm', '-f', name]); }
      finally { if (inspect()) throw new Error('Account custody TLS container cleanup failed'); }
    }
  }
}

export function prepareLocalCustody(directory, runtime, password) {
  const supplied = keys.filter((key) => process.env[key]);
  if (supplied.length) {
    if (supplied.length !== keys.length) throw new Error('supply all four Account custody transport paths');
    return null;
  }
  if (!password || /[\r\n]/.test(password)) throw new Error('invalid Account custody password');
  if (exists(directory)) privatePath(directory, true);
  else mkdirSync(directory, { recursive: true, mode: 0o700 });
  const tls = path.join(directory, 'tls');
  if (exists(tls)) privatePath(tls, true);
  else mkdirSync(tls, { mode: 0o700 });
  const key = path.join(tls, 'server.key');
  const cert = path.join(tls, 'server.crt');
  if (exists(key) !== exists(cert)) throw new Error('partial Account custody TLS certificate pair');
  if (!exists(cert)) provisionCertificate(tls, runtime);
  privatePath(key); privatePath(cert);
  try {
    const certificate = new X509Certificate(readFileSync(cert));
    if (certificate.ca || !certificate.checkPrivateKey(createPrivateKey(readFileSync(key))) ||
        !certificate.checkHost('postgres') || !certificate.checkIP('127.0.0.1') ||
        Date.parse(certificate.validFrom) > Date.now() || Date.parse(certificate.validTo) <= Date.now() + 86400_000)
      throw new Error('invalid or expired certificate');
  } catch (error) {
    throw new Error('Account custody TLS certificate/key validation failed', { cause: error });
  }
  const descriptor = path.join(directory, 'target.env');
  if (exists(descriptor)) privatePath(descriptor);
  if (!exists(descriptor)) writeFileSync(descriptor, '', { mode: 0o600, flag: 'wx' });
  const passwordFile = path.join(directory, 'password');
  if (exists(passwordFile)) privatePath(passwordFile);
  if (exists(passwordFile) && readFileSync(passwordFile, 'utf8') !== password)
    throw new Error('local Account custody password differs from retained provisioning');
  if (!exists(passwordFile)) writeFileSync(passwordFile, password, { mode: 0o600, flag: 'wx' });
  Object.assign(process.env, {
    ACCOUNT_CUSTODY_PG_TLS_DIR: tls, ACCOUNT_CUSTODY_TARGET_ENV_FILE: descriptor,
    ACCOUNT_CUSTODY_PASSWORD_FILE: passwordFile, ACCOUNT_CUSTODY_CA_FILE: path.join(tls, 'server.crt'),
    ACCOUNT_CUSTODY_RUN_UID: String(process.getuid?.() ?? 0),
    ACCOUNT_CUSTODY_RUN_GID: String(process.getgid?.() ?? 0),
  });
  return descriptor;
}

export function retainLocalCustodyInventory(descriptor, output, operator, database) {
  if (!/^[a-zA-Z_][a-zA-Z0-9_]{0,62}$/.test(operator) ||
      !/^[a-zA-Z_][a-zA-Z0-9_]{0,62}$/.test(database)) throw new Error('invalid local Account custody identity');
  const match = output.trim().match(/^([1-9][0-9]{0,19})\|([1-9][0-9]{0,9})$/);
  if (!match) throw new Error('invalid local Account custody inventory readback');
  const content = `POSTGRES_HOST=postgres\nPOSTGRES_PORT=5432\nPOSTGRES_DB=${database}\nPOSTGRES_ADMIN_USER=${operator}\nACCOUNT_CUSTODY_EXPECTED_OPERATOR=${operator}\nACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER=${match[1]}\nACCOUNT_CUSTODY_EXPECTED_DATABASE=${database}\nACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=${match[2]}\nACCOUNT_CUSTODY_EXPECTED_TLS_HOST=postgres\n`;
  privatePath(path.dirname(descriptor), true);
  privatePath(descriptor);
  const previous = readFileSync(descriptor, 'utf8');
  if (previous && previous !== content) throw new Error('local Account custody target changed; retained inventory refused');
  if (!previous) writeFileSync(descriptor, content, { mode: 0o600 });
}
