#!/usr/bin/env node
/**
 * Default-branch verifier for a console candidate C, authority tip T, and
 * GitHub synthetic merge M.  This file is trusted only because it runs from
 * the protected target branch; no PR file is executed before this check.
 */
import { execFileSync, spawnSync } from 'node:child_process';
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

export const TRUSTED_PRINCIPAL = 'jason19931225@gmail.com';
export const TRUSTED_FINGERPRINT = 'SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8';
export const TRUSTED_ALLOWED_SIGNER = `${TRUSTED_PRINCIPAL} ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAgMAp8vHS9V/9UQQVTa5FtmS9Q9fdB8I520DsZMMDTR`;
export const REGISTRY_PATH = 'docs/program/console-capability-registry.json';
export const POLICY_PATH = '.github/trust/console.allowed_signers';
export const AUTHORITY_PATHS = Object.freeze([
  'docs/program/console-capability-registry.json',
  'docs/program/console-jurisdiction-register.json',
  'docs/program/console-program-ledger.md',
]);
const SHA = /^[0-9a-f]{40}$/;
const SAFE_ENVIRONMENT_KEYS = Object.freeze(['PATH', 'SystemRoot', 'SYSTEMROOT', 'ComSpec', 'TEMP', 'TMP', 'TMPDIR', 'LANG', 'LC_ALL', 'TZ']);
const fail = (message) => { throw new Error(`console authority bootstrap: ${message}`); };
const exactSha = (value, label) => { if (!SHA.test(value ?? '')) fail(`${label} must be a lowercase 40-character SHA`); return value; };

function sanitizedGitEnvironment(source = process.env) {
  const environment = {};
  for (const key of SAFE_ENVIRONMENT_KEYS) if (source[key] !== undefined) environment[key] = source[key];
  return {
    ...environment,
    HOME: '/dev/null',
    XDG_CONFIG_HOME: '/dev/null',
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    GIT_CONFIG_SYSTEM: '/dev/null',
    GIT_TERMINAL_PROMPT: '0',
    GIT_ASKPASS: '/bin/false',
  };
}

export function validatePinnedPolicy(raw) {
  if (raw !== `${TRUSTED_ALLOWED_SIGNER}\n`) fail('C signing policy must contain exactly the pinned signer');
  return raw;
}
function candidateLocator(raw) {
  try { return exactSha(JSON.parse(raw)?.candidate?.sha, 'T registry candidate.sha'); }
  catch (error) { if (error.message?.startsWith('console authority bootstrap:')) throw error; fail('T registry is not valid JSON'); }
}
function assertSigned(status, label) {
  if (status?.ok !== true || status.principal !== TRUSTED_PRINCIPAL || status.fingerprint !== TRUSTED_FINGERPRINT) fail(`${label} is not signed by the pinned SSH authority`);
}

/** A pure seam used by hermetic tests; ops only reads Git object facts. */
function verifyAuthorityTrain(ops, authorityTipSha) {
  const T = exactSha(authorityTipSha, 'PR head');
  if (!ops.hasCommit(T)) fail('PR head object is unavailable');
  const C = candidateLocator(ops.readFile(T, REGISTRY_PATH)); // untrusted locator, never authority
  if (!ops.hasCommit(C)) fail('C object is unavailable');
  const policyEntry = ops.treeEntry(C, POLICY_PATH);
  if (policyEntry?.mode !== '100644' || policyEntry.type !== 'blob') fail('C signing policy must be a regular mode-100644 blob');
  const policy = validatePinnedPolicy(ops.readFile(C, POLICY_PATH));
  const authority = { policy, principal: TRUSTED_PRINCIPAL, fingerprint: TRUSTED_FINGERPRINT };
  assertSigned(ops.verifyCommit(C, authority), 'C');
  assertSigned(ops.verifyCommit(T, authority), 'T');
  const tipParents = ops.parents(T);
  if (!Array.isArray(tipParents) || tipParents.length !== 1 || tipParents[0] !== C) fail('T must be the direct single-parent child of C');
  const changes = ops.diff(C, T);
  if (!Array.isArray(changes) || changes.length !== AUTHORITY_PATHS.length) fail('C..T must modify exactly the three authority documents');
  const changed = new Set();
  for (const change of changes) {
    if (change.status !== 'M' || change.oldMode !== '100644' || change.newMode !== '100644' || change.oldType !== 'blob' || change.newType !== 'blob' || !AUTHORITY_PATHS.includes(change.path) || changed.has(change.path)) fail('C..T may only make regular mode-100644 modifications to exact authority documents');
    changed.add(change.path);
  }
  if (AUTHORITY_PATHS.some((entry) => !changed.has(entry))) fail('C..T authority document set is incomplete');
  return { candidateSha: C, authorityTipSha: T };
}

/** A pure seam used by hermetic tests; ops only reads Git object facts. */
export function verifyBootstrapGraph(ops, { headSha, mergeSha }) {
  const train = verifyAuthorityTrain(ops, headSha);
  const { candidateSha: C, authorityTipSha: T } = train;
  const M = exactSha(mergeSha, 'PR merge');
  if (!ops.hasCommit(M)) fail('PR merge object is unavailable');
  const mergeParents = ops.parents(M);
  if (!Array.isArray(mergeParents) || mergeParents.length !== 2 || mergeParents[1] !== T) fail('M must be a two-parent merge whose second parent is T');
  if (ops.tree(M) !== ops.tree(T) || !ops.sameTreeDiff(M, T)) fail('M tree/diff must equal T exactly');
  return Object.freeze({ candidateSha: C, integrationTipSha: T, mergeSha: M });
}

/** A pure post-merge seam. It never reads candidate, T, or S executable content. */
export function verifySquashBinding(ops, { authorityTipSha, squashSha, preMergeBaseSha }) {
  const train = verifyAuthorityTrain(ops, authorityTipSha);
  const S = exactSha(squashSha, 'squash commit');
  const B = exactSha(preMergeBaseSha, 'pre-merge base');
  if (!ops.hasCommit(S) || !ops.hasCommit(B)) fail('squash commit or pre-merge base object is unavailable');
  const squashParents = ops.parents(S);
  if (!Array.isArray(squashParents) || squashParents.length !== 1 || squashParents[0] !== B) fail('S must be a one-parent squash commit on the trusted pre-merge base');
  if (ops.tree(S) !== ops.tree(train.authorityTipSha) || !ops.sameTreeDiff(S, train.authorityTipSha)) fail('S tree/diff must equal T exactly');
  return Object.freeze({ candidateSha: train.candidateSha, authorityTipSha: train.authorityTipSha, squashSha: S, preMergeBaseSha: B });
}
export function squashBindingReceipt(binding) {
  return Object.freeze({ schema: 'console-squash-binding-v1', verdict: 'TREE_BOUND_HOLD_PRESERVED', release_disposition: 'HOLD', ...binding });
}

function git(repo, args, options = {}) {
  const { env, ...rest } = options;
  return execFileSync('git', ['-C', repo, '-c', 'core.hooksPath=/dev/null', ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], env: sanitizedGitEnvironment(env), ...rest });
}
function gitOk(repo, args) { try { git(repo, args); return true; } catch { return false; } }
function treeEntry(repo, sha, file) { const entry = git(repo, ['ls-tree', sha, '--', file]).trim().match(/^(\d{6}) (\w+) [0-9a-f]{40}\t/); return entry ? { mode: entry[1], type: entry[2] } : null; }
function rawDiff(repo, from, to) {
  const fields = git(repo, ['diff', '--raw', '-z', '--abbrev=40', '--no-renames', '--no-ext-diff', from, to]).split('\0'); const changes = [];
  for (let index = 0; index < fields.length - 1;) {
    const header = fields[index++]; if (!header) continue;
    const match = header.match(/^:(\d{6}) (\d{6}) [0-9a-f]{40} [0-9a-f]{40} ([A-Z])$/); if (!match) fail('Git diff contains an unsupported entry');
    const [, oldMode, newMode, status] = match;
    changes.push({ path: fields[index++], status, oldMode, newMode, oldType: oldMode === '000000' ? null : 'blob', newType: newMode === '000000' ? null : 'blob' });
  }
  return changes;
}
export function verifyPinnedSshCommit(repo, sha, policy, environment = process.env) {
  const directory = mkdtempSync(path.join(tmpdir(), 'console-signers-')); const policyFile = path.join(directory, 'allowed_signers');
  try {
    writeFileSync(policyFile, policy, { mode: 0o600 }); chmodSync(policyFile, 0o600);
    const result = spawnSync('git', ['-C', repo, '-c', 'core.hooksPath=/dev/null', '-c', 'gpg.format=ssh', '-c', 'gpg.ssh.program=ssh-keygen', '-c', `gpg.ssh.allowedSignersFile=${policyFile}`, 'verify-commit', '--raw', sha], { encoding: 'utf8', env: sanitizedGitEnvironment(environment) });
    const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
    return { ok: result.status === 0, principal: output.match(/Good "git" signature for (.+?) with /)?.[1] ?? null, fingerprint: output.match(/key (SHA256:[A-Za-z0-9+/]+={0,2})/)?.[1] ?? null, command: ['gpg.format=ssh', 'gpg.ssh.program=ssh-keygen', 'gpg.ssh.allowedSignersFile=<0600-temp>'] };
  } finally { rmSync(directory, { recursive: true, force: true }); }
}
function gitOps(repo) {
  return { hasCommit: (sha) => gitOk(repo, ['cat-file', '-e', `${sha}^{commit}`]), readFile: (sha, file) => git(repo, ['show', `${sha}:${file}`]), treeEntry: (sha, file) => treeEntry(repo, sha, file), parents: (sha) => git(repo, ['show', '-s', '--format=%P', sha]).trim().split(/\s+/).filter(Boolean), diff: (from, to) => rawDiff(repo, from, to), tree: (sha) => git(repo, ['show', '-s', '--format=%T', sha]).trim(), sameTreeDiff: (left, right) => git(repo, ['diff', '--quiet', '--no-ext-diff', left, right]) === '', verifyCommit: (sha, authority) => verifyPinnedSshCommit(repo, sha, authority.policy) };
}
function safeBaseBranch(base) { return base === 'main'; }
function parseArgs(argv) {
  const result = {}; for (let index = 0; index < argv.length; index += 2) { const key = argv[index]; const value = argv[index + 1]; if (!['--pr-number', '--head', '--merge', '--base'].includes(key) || value === undefined) fail('usage: --pr-number N --head SHA --merge SHA --base branch'); result[key.slice(2)] = value; }
  if (!/^\d+$/.test(result['pr-number'] ?? '')) fail('PR number is invalid'); exactSha(result.head, 'PR head');
  // `--merge` is informational only. GitHub computes `merge_commit_sha`
  // asynchronously, so it is null on `opened` and stale on `synchronize`; the
  // authoritative value is the `refs/pull/N/merge` ref resolved at fetch time.
  // Validate it only when the event actually supplied one.
  if (result.merge !== undefined && result.merge !== '' && result.merge !== 'null') exactSha(result.merge, 'PR merge');
  if (!safeBaseBranch(result.base)) fail('PR base is outside the console foundation trust scope'); return result;
}
/**
 * Fetch the pull request's head and synthetic merge objects, and return the
 * merge SHA that GitHub actually has right now.
 *
 * `head` is compared strictly: it pins the exact reviewed code, and a mismatch
 * means the event and the refs disagree about what is being verified.
 *
 * `merge` is deliberately NOT compared to the event payload. GitHub regenerates
 * the synthetic test-merge commit asynchronously and it embeds a timestamp, so
 * its SHA changes on every recompute — including recomputes triggered by the
 * very event that starts this job. Comparing the payload's (already stale)
 * `merge_commit_sha` against the freshly fetched ref therefore fails on every
 * force-push, and on reopen, with no security benefit: the merge commit is only
 * a vehicle. Its integrity is established structurally downstream by
 * `verifyBootstrapGraph`, which requires exactly two parents with the verified
 * authority tip as parent 2. So we resolve the ref and hand that SHA onward.
 */
function fetchExactPullObjects(repo, number, expectedHead) {
  const namespace = `refs/console-bootstrap/${number}`;
  git(repo, ['fetch', '--no-tags', '--no-recurse-submodules', 'origin', `+refs/pull/${number}/head:${namespace}/head`, `+refs/pull/${number}/merge:${namespace}/merge`]);
  if (git(repo, ['rev-parse', `${namespace}/head`]).trim() !== expectedHead) fail('GitHub pull head ref does not match the event head SHA');
  const resolvedMerge = git(repo, ['rev-parse', `${namespace}/merge`]).trim();
  if (!/^[0-9a-f]{40}$/.test(resolvedMerge)) fail('GitHub pull merge ref is unresolvable');
  return resolvedMerge;
}
export function fetchExactAuthorityTip(repo, number, expectedHead) {
  const parsedNumber = String(number);
  if (!/^\d+$/.test(parsedNumber)) fail('PR number is invalid');
  const T = exactSha(expectedHead, 'PR head');
  const ref = `refs/console-squash-binding/${parsedNumber}/head`;
  git(repo, ['fetch', '--no-tags', '--no-recurse-submodules', 'origin', `+refs/pull/${parsedNumber}/head:${ref}`]);
  if (git(repo, ['rev-parse', ref]).trim() !== T) fail('GitHub pull head ref does not match event SHA');
  return T;
}
export function candidateCheckPlan(C, T, M) {
  return Object.freeze({
    environment: Object.freeze({ CONSOLE_CANDIDATE_SHA: C, CONSOLE_AUTHORITY_TIP_SHA: T, CONSOLE_SYNTHETIC_MERGE_SHA: M }),
    commands: Object.freeze([
      ['node', ['scripts/console/validate-console-truth-ledger.mjs']],
      ['node', ['scripts/console/plan-fanout.mjs', '--candidate', C, '--authority-tip', T, '--synthetic-merge', M]],
      ['node', ['--test', 'scripts/console/validate-console-truth-ledger.test.mjs', 'scripts/console/plan-fanout.test.mjs', 'scripts/console/verify-console-authority-train.test.mjs']],
    ]),
  });
}
function runAuthenticatedCandidateChecks(repo, C, T, M) {
  const candidate = mkdtempSync(path.join(tmpdir(), 'console-candidate-'));
  try {
    git(repo, ['worktree', 'add', '--detach', '--no-checkout', candidate]); git(repo, ['-C', candidate, 'checkout', '--detach', C]);
    const plan = candidateCheckPlan(C, T, M);
    const environment = { ...sanitizedGitEnvironment(), ...plan.environment };
    for (const [binary, args] of plan.commands) {
      if (spawnSync(binary, args, { cwd: candidate, env: environment, stdio: 'inherit' }).status !== 0) fail(`authenticated C check failed: ${binary} ${args.join(' ')}`);
    }
  } finally { try { git(repo, ['worktree', 'remove', '--force', candidate]); } catch { rmSync(candidate, { recursive: true, force: true }); } }
}
function parseSquashBindingArgs(argv) {
  const result = {}; for (let index = 0; index < argv.length; index += 2) { const key = argv[index]; const value = argv[index + 1]; if (!['--pr-number', '--head', '--squash', '--base'].includes(key) || value === undefined) fail('usage: squash-binding --pr-number N --head SHA --squash SHA --base main'); result[key.slice(2)] = value; }
  if (!/^\d+$/.test(result['pr-number'] ?? '')) fail('PR number is invalid'); exactSha(result.head, 'PR head'); exactSha(result.squash, 'squash commit'); if (!safeBaseBranch(result.base)) fail('PR base is outside the protected main trust scope'); return result;
}
function main() { const args = parseArgs(process.argv.slice(2)); const repo = process.cwd(); const mergeSha = fetchExactPullObjects(repo, args['pr-number'], args.head); const graph = verifyBootstrapGraph(gitOps(repo), { headSha: args.head, mergeSha }); runAuthenticatedCandidateChecks(repo, graph.candidateSha, graph.integrationTipSha, graph.mergeSha); process.stdout.write(`${JSON.stringify({ verdict: 'PASS', ...graph }, null, 2)}\n`); }
function squashBindingMain() {
  const args = parseSquashBindingArgs(process.argv.slice(3)); const repo = process.cwd();
  const preMergeBaseSha = git(repo, ['rev-parse', 'HEAD']).trim();
  const authorityTipSha = fetchExactAuthorityTip(repo, args['pr-number'], args.head);
  const binding = verifySquashBinding(gitOps(repo), { authorityTipSha, squashSha: args.squash, preMergeBaseSha });
  process.stdout.write(`${JSON.stringify(squashBindingReceipt(binding), null, 2)}\n`);
}
if (import.meta.url === new URL(`file://${process.argv[1]}`).href) {
  if (process.argv[2] === 'squash-binding') squashBindingMain(); else main();
}
