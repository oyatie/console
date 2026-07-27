import { execFileSync } from 'node:child_process';
import { CONSOLE_CANDIDATE_SIGNING_AUTHORITY, verifyCommitWithCandidateSshPolicy } from './ssh-signature-policy.mjs';

const SHA = /^[0-9a-f]{40}$/;
const AUTHORITY_PATHS = new Set([
  'docs/program/console-capability-registry.json',
  'docs/program/console-jurisdiction-register.json',
  'docs/program/console-program-ledger.md',
]);

function fail(message) { throw new Error(`console authority train: ${message}`); }
function sha(value, label) { if (!SHA.test(value ?? '')) fail(`${label} must be a full lowercase Git SHA`); return value; }
function git(repoRoot, args) { return execFileSync('git', ['-C', repoRoot, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }); }
function gitSucceeds(repoRoot, args) { try { git(repoRoot, args); return true; } catch { return false; } }
function verifySigned(repoRoot, candidateSha, value, label, authority) {
  if (!gitSucceeds(repoRoot, ['cat-file', '-e', `${value}^{commit}`])) fail(`${label} SHA is unresolvable`);
  try { verifyCommitWithCandidateSshPolicy(repoRoot, candidateSha, value, authority); }
  catch (error) { fail(`${label} signature is not valid: ${error instanceof Error ? error.message : String(error)}`); }
}
function assertAuthorityDiff(repoRoot, candidateSha, authorityTipSha) {
  const fields = git(repoRoot, ['diff', '--raw', '-z', '--abbrev=40', '--no-renames', '--no-ext-diff', candidateSha, authorityTipSha]).split('\0');
  const changed = new Set();
  for (let index = 0; index < fields.length - 1;) {
    const header = fields[index++]; if (!header) continue;
    const match = header.match(/^:(\d{6}) (\d{6}) [0-9a-f]{40} [0-9a-f]{40} ([A-Z])$/);
    if (!match) fail('C..T diff contains an unsupported entry');
    const [, oldMode, newMode, status] = match; const file = fields[index++];
    if (status !== 'M' || oldMode !== '100644' || newMode !== '100644' || !AUTHORITY_PATHS.has(file) || changed.has(file)) fail('C..T may only modify the exact three regular mode-100644 authority documents');
    changed.add(file);
  }
  if (changed.size !== AUTHORITY_PATHS.size || [...AUTHORITY_PATHS].some((file) => !changed.has(file))) fail('C..T authority document set is incomplete');
}

/** Validates signed candidate C, signed authority tip T, and structural-only GitHub merge M. */
export function verifyConsoleAuthorityTrain(repoRoot, candidateSha, authorityTipSha, syntheticMergeSha, authority = CONSOLE_CANDIDATE_SIGNING_AUTHORITY) {
  sha(candidateSha, 'candidate SHA'); sha(authorityTipSha, 'authority tip SHA'); sha(syntheticMergeSha, 'synthetic merge SHA');
  verifySigned(repoRoot, candidateSha, candidateSha, 'candidate C', authority);
  verifySigned(repoRoot, candidateSha, authorityTipSha, 'authority tip T', authority);
  const tipParents = git(repoRoot, ['show', '-s', '--format=%P', authorityTipSha]).trim().split(/\s+/).filter(Boolean);
  if (tipParents.length !== 1 || tipParents[0] !== candidateSha) fail('T must be the direct single-parent child of C');
  assertAuthorityDiff(repoRoot, candidateSha, authorityTipSha);
  if (!gitSucceeds(repoRoot, ['cat-file', '-e', `${syntheticMergeSha}^{commit}`])) fail('synthetic merge M is unresolvable');
  const mergeParents = git(repoRoot, ['show', '-s', '--format=%P', syntheticMergeSha]).trim().split(/\s+/).filter(Boolean);
  if (mergeParents.length !== 2 || mergeParents[1] !== authorityTipSha) fail('M must have exactly two parents with T as parent 2');
  if (git(repoRoot, ['show', '-s', '--format=%T', syntheticMergeSha]).trim() !== git(repoRoot, ['show', '-s', '--format=%T', authorityTipSha]).trim() || !gitSucceeds(repoRoot, ['diff', '--quiet', '--no-ext-diff', syntheticMergeSha, authorityTipSha])) fail('M tree and content diff must equal T exactly');
  return Object.freeze({ candidateSha, authorityTipSha, syntheticMergeSha });
}
