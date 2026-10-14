// TEST_ONLY filesystem data for the actual binder; no native tool is executed.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { mock } from 'node:test';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const sourcePaths = [
  'scripts/run-native-hr-policy-parser-contract.sh',
  'scripts/check-native-hr-policy-parser-contract.mjs',
  'scripts/check-native-hr-policy-parser-contract.test.mjs',
  'scripts/hr-policy-parser/binding.mjs', 'scripts/hr-policy-parser/native-run.mjs',
  'scripts/hr-policy-parser/observations.mjs', 'scripts/hr-policy-parser/qualify-admission.py',
  'backend/crates/identity/application/tests/native_hr_policy_parser_contract.rs',
  'backend/crates/identity/application/tests/hr_policy_fixtures/literals.rs',
  'backend/crates/identity/application/tests/hr_policy_fixtures/assert_fields.rs',
  'tools/buck2', 'tools/buck/gen_first_party.py', 'backend/crates/identity/application/BUCK',
];
export function testOnlyBindingModel(root) {
  // Read real current source, Node and host utility bytes before installing mocks.
  const snapshot = file => {
    const canonical = fs.realpathSync(file), stat = fs.lstatSync(canonical);
    if (!stat.isFile() || stat.isSymbolicLink()) throw new Error(`not a real source/tool: ${file}`);
    return { path: canonical, bytes: fs.readFileSync(canonical) };
  };
  const node = snapshot(process.execPath);
  const sources = sourcePaths.map(name => {
    const actual = snapshot(path.join(root, name));
    if (actual.path !== path.join(root, name)) throw new Error(`source is not canonical: ${name}`);
    return { path: name, sha256: hash(actual.bytes) };
  });
  const utility = name => {
    for (const directory of ['/usr/bin', '/bin']) {
      const file = path.join(directory, name);
      try {
        if (!fs.statSync(file).isFile()) continue;
        fs.accessSync(file, fs.constants.X_OK);
        return snapshot(file).bytes;
      } catch (error) {
        if (!['ENOENT', 'ENOTDIR', 'EACCES'].includes(error.code)) throw error;
      }
    }
    throw new Error(`missing real utility bytes: ${name}`);
  };
  const mkdirBytes = utility('mkdir'), tarBytes = utility('tar');
  const stem = '/TEST_ONLY/hr-command-budget';
  const entries = new Map([
    ...['python', 'git', 'dotslash', 'buck2'].map(name => [`${stem}/${name}`, {bytes: node.bytes}]),
    ['/bin/mkdir', {bytes: mkdirBytes}], ['/usr/bin/bsdtar', {bytes: tarBytes}],
    ['/usr/bin/tar', {link: 'bsdtar'}], [`${stem}/cache`, {directory: true}],
  ]);
  const absent = new Set(['/usr/bin/mkdir', '/usr/bin/dotslash', '/bin/dotslash']);
  const pin = name => ({path: `${stem}/${name}`, sha256: hash(node.bytes)});
  const utilities = {
    mkdir: {lookup_path: '/bin/mkdir', path: '/bin/mkdir', sha256: hash(mkdirBytes)},
    tar: {lookup_path: '/usr/bin/tar', path: '/usr/bin/bsdtar', sha256: hash(tarBytes)},
  };
  const binding = {
    schema: 'console.hr-policy-parser.operational-binding.v1',
    // Required parser fixture literal; this is not a review or operating receipt.
    status: 'REVIEWED_EXACT_TEST_CANDIDATE', test_only: true,
    nonexecuted_tool_roles: ['python', 'git', 'dotslash', 'buck_binary', 'compiler', 'action_wrappers'],
    cwd: root, probe: 'scripts/run-native-hr-policy-parser-contract.sh',
    contract_sha256: '95b9243256fa70e50bb645aa4c4cea400d0374e13fde2336378580344ffd832d',
    test_contract_sha256: '4b354e6c56fc3c2d06d36c488d9f7b3f3fc1f46fb57ba9e311bc5fdbd26f7d96',
    node: {path: node.path, sha256: hash(node.bytes)}, python: pin('python'), git: pin('git'),
    dotslash: pin('dotslash'), buck_binary: pin('buck2'), dotslash_cache: `${stem}/cache`,
    bootstrap_utilities: utilities, sources, timeout_ms: 12345,
    candidate_head: '1'.repeat(40), candidate_tree: '2'.repeat(40),
    compiler_sha256: hash(node.bytes), action_wrapper_sha256: [hash(node.bytes), hash(node.bytes)],
  };
  return {binding, utilities, install() {
    const methods = ['existsSync', 'accessSync', 'lstatSync', 'statSync', 'realpathSync', 'readFileSync', 'readlinkSync'];
    const original = Object.fromEntries(methods.map(name => [name, fs[name].bind(fs)]));
    const missing = file => Object.assign(new Error(`TEST_ONLY ENOENT: ${file}`), {code: 'ENOENT'});
    const stat = entry => ({isFile: () => !entry.link && !entry.directory,
      isSymbolicLink: () => Boolean(entry.link), isDirectory: () => Boolean(entry.directory),
      mode: entry.directory ? 0o40755 : entry.link ? 0o120777 : 0o100755});
    for (const name of methods) mock.method(fs, name, (file, ...args) => {
      if (absent.has(file)) {if (name === 'existsSync') return false; throw missing(file);}
      const entry = entries.get(file);
      if (!entry) return original[name](file, ...args);
      const target = entry.link ? entries.get(path.resolve(path.dirname(file), entry.link)) : entry;
      if (name === 'existsSync') return true;
      if (name === 'accessSync') return;
      if (name === 'lstatSync') return stat(entry);
      if (name === 'statSync') return stat(target);
      if (name === 'realpathSync') return entry.link ? path.resolve(path.dirname(file), entry.link) : file;
      if (name === 'readlinkSync') {
        if (entry.link) return entry.link;
        throw Object.assign(new Error('TEST_ONLY not a link'), {code: 'EINVAL'});
      }
      if (!target.bytes) throw new Error('TEST_ONLY not a regular file');
      const encoding = typeof args[0] === 'string' ? args[0] : args[0]?.encoding;
      return encoding ? target.bytes.toString(encoding) : Buffer.from(target.bytes);
    });
    return () => mock.restoreAll();
  }};
}
