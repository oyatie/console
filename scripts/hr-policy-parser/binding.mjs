import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
export const SOURCE = 'backend/crates/identity/application/tests/native_hr_policy_parser_contract.rs';
export const TARGET = '//backend/crates/identity/application:console-identity-application-itest-native_hr_policy_parser_contract';
export const LEGACY = '//backend/crates/identity/application:console-identity-application-unit';
export const IMPORT = 'use console_identity_application::company_policy::decode_native_hr_policy_command_v1;';
export const HOST = 'literal HR policy parser satisfies native compilation and strict codec conformance';
export const CASES = [
  'hr_policy_setup_decodes_exact_fields', 'hr_policy_B_grants_decode_without_qualification_pair',
  'hr_policy_C_grants_decode_complete_snapshot_pair', 'hr_policy_revoke_decodes_without_qualification',
  'hr_policy_protocol_header_and_flags_reject_corruption', 'hr_policy_admin_source_and_assignment_refs_reject_corruption',
  'hr_policy_qualification_snapshot_pair_rejects_corruption', 'hr_policy_all_truncations_and_trailing_bytes_reject',
];
export const LEGACY_CASES = [
  'company_policy::policy_command_tests::shared_policy_command_preserves_each_existing_codec_and_common_fields',
  'company_policy::policy_command_tests::shared_policy_command_retains_people_action_and_replay_distinctions',
  'company_policy::policy_command_tests::shared_policy_command_preserves_assignment_witnesses_for_both_protocols',
];
export const hash = data => crypto.createHash('sha256').update(data).digest('hex');
export function regular(file) {
  const st = fs.lstatSync(file);
  if (!st.isFile() || st.isSymbolicLink()) throw new Error(`not regular: ${file}`);
  return fs.readFileSync(file);
}
export function pinned(file, sha) {
  if (!/^[a-f0-9]{64}$/.test(sha) || hash(regular(file)) !== sha) throw new Error(`pin mismatch: ${file}`);
}
function firstExecutable(name, directories) {
  for (const directory of directories) {
    const file = path.join(directory, name);
    try {
      if (!fs.statSync(file).isFile()) continue;
      fs.accessSync(file, fs.constants.X_OK); return file;
    } catch (error) {
      if (!['ENOENT', 'ENOTDIR', 'EACCES'].includes(error.code)) throw error;
    }
  }
}
export function loadBinding(env = process.env) {
  const file = env.CONSOLE_HR_POLICY_BINDING;
  if (!file || !path.isAbsolute(file)) throw new Error('missing absolute root binding');
  pinned(file, env.CONSOLE_HR_POLICY_BINDING_SHA256);
  const b = JSON.parse(regular(file));
  if (b.schema !== 'console.hr-policy-parser.operational-binding.v1' || !['REVIEWED_EXACT_TEST_CANDIDATE', 'MEASURED_CURRENT_CI_INPUTS'].includes(b.status)
      || b.cwd !== ROOT || b.probe !== 'scripts/run-native-hr-policy-parser-contract.sh'
      || b.contract_sha256 !== '95b9243256fa70e50bb645aa4c4cea400d0374e13fde2336378580344ffd832d'
      || b.test_contract_sha256 !== '4b354e6c56fc3c2d06d36c488d9f7b3f3fc1f46fb57ba9e311bc5fdbd26f7d96'
      || b.node.path !== env.CONSOLE_HR_POLICY_NODE || fs.realpathSync(process.execPath) !== b.node.path
      || !Number.isSafeInteger(b.timeout_ms) || b.timeout_ms < 1000 || b.timeout_ms > 3_600_000) throw new Error('invalid operational binding');
  for (const tool of [b.node, b.python, b.git, b.dotslash, b.buck_binary]) {
    if (!path.isAbsolute(tool.path) || fs.realpathSync(tool.path) !== tool.path) throw new Error('tool path not exact canonical absolute');
    pinned(tool.path, tool.sha256); fs.accessSync(tool.path, fs.constants.X_OK);
  }
  if (firstExecutable('dotslash', ['/usr/bin', '/bin', path.dirname(b.dotslash.path)]) !== b.dotslash.path)
    throw new Error('DotSlash first executable lookup mismatch');
  const utilities = b.bootstrap_utilities;
  if (!utilities || typeof utilities !== 'object' || Array.isArray(utilities)
      || Object.keys(utilities).sort().join(',') !== 'mkdir,tar') throw new Error('invalid bootstrap utilities');
  for (const name of ['mkdir', 'tar']) {
    const tool = utilities[name];
    if (!tool || typeof tool !== 'object' || Array.isArray(tool)
        || Object.keys(tool).sort().join(',') !== 'lookup_path,path,sha256'
        || typeof tool.lookup_path !== 'string' || typeof tool.path !== 'string'
        || !path.isAbsolute(tool.path)) throw new Error('invalid bootstrap utility metadata');
    const lookup = firstExecutable(name, ['/usr/bin', '/bin']);
    if (!lookup || tool.lookup_path !== lookup) throw new Error('bootstrap utility lookup mismatch');
    if (fs.lstatSync(lookup).isSymbolicLink()) {
      if (name !== 'tar' || lookup !== '/usr/bin/tar' || tool.path !== '/usr/bin/bsdtar'
          || fs.readlinkSync(lookup) !== 'bsdtar') throw new Error('unapproved bootstrap utility alias');
    } else if (tool.path !== lookup) throw new Error('bootstrap utility canonical mismatch');
    if (fs.realpathSync(tool.path) !== tool.path) throw new Error('bootstrap utility path not canonical');
    pinned(tool.path, tool.sha256); fs.accessSync(tool.path, fs.constants.X_OK);
  }
  if (!path.isAbsolute(b.dotslash_cache) || !fs.statSync(b.dotslash_cache).isDirectory()) throw new Error('missing pinned DotSlash cache');
  if (!Array.isArray(b.sources) || b.sources.length === 0 || !b.sources.some(s => s.path === SOURCE)
      || !b.sources.some(s => s.path === 'tools/buck2') || !b.sources.some(s => s.path === 'scripts/check-native-hr-policy-parser-contract.mjs')
      || !b.sources.some(s => s.path === 'scripts/check-native-hr-policy-parser-contract.test.mjs')) throw new Error('missing source binding');
  const required = ['scripts/run-native-hr-policy-parser-contract.sh', 'scripts/hr-policy-parser/binding.mjs',
    'scripts/hr-policy-parser/native-run.mjs', 'scripts/hr-policy-parser/observations.mjs',
    'scripts/hr-policy-parser/qualify-admission.py', 'backend/crates/identity/application/tests/hr_policy_fixtures/literals.rs',
    'backend/crates/identity/application/tests/hr_policy_fixtures/assert_fields.rs',
    'tools/buck/gen_first_party.py', 'backend/crates/identity/application/BUCK'];
  if (required.some(file => !b.sources.some(s => s.path === file))) throw new Error('incomplete HR source/integration pins');
  for (const s of b.sources) {
    if (path.isAbsolute(s.path) || s.path.split('/').includes('..')) throw new Error('unsafe source pin');
    pinned(path.join(ROOT, s.path), s.sha256);
  }
  if (regular(path.join(ROOT, SOURCE)).toString().split('\n')[0] !== IMPORT) throw new Error('frozen import changed');
  if (!Array.isArray(b.action_wrapper_sha256) || b.action_wrapper_sha256.length !== 2
      || b.action_wrapper_sha256.some(x => !/^[a-f0-9]{64}$/.test(x))) throw new Error('missing actual wrapper tool identities');
  if (!/^[a-f0-9]{40}$/.test(b.candidate_head) || !/^[a-f0-9]{40}$/.test(b.candidate_tree)
      || !/^[a-f0-9]{64}$/.test(b.compiler_sha256)) throw new Error('missing candidate/compiler identities');
  return b;
}
