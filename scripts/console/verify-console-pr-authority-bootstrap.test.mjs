import assert from 'node:assert/strict';
import test from 'node:test';
import { AUTHORITY_PATHS, POLICY_PATH, TRUSTED_ALLOWED_SIGNER, TRUSTED_FINGERPRINT, TRUSTED_PRINCIPAL, candidateCheckPlan, fetchExactAuthorityTip, squashBindingReceipt, validatePinnedPolicy, verifyBootstrapGraph, verifyPinnedSshCommit, verifySquashBinding } from './verify-console-pr-authority-bootstrap.mjs';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';

const C = 'c'.repeat(40), T = 't'.repeat(40).replace(/t/g, 'a'), M = 'b'.repeat(40), S = 'e'.repeat(40), BASE = 'd'.repeat(40);
const policy = `${TRUSTED_ALLOWED_SIGNER}\n`;
const changes = AUTHORITY_PATHS.map((path) => ({ path, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }));
function fixture(overrides = {}) {
  const calls = []; const registry = JSON.stringify({ candidate: { sha: C } });
  const data = {
    hasCommit: () => true,
    readFile: (sha, file) => file === POLICY_PATH ? policy : registry,
    treeEntry: () => ({ mode: '100644', type: 'blob' }),
    parents: (sha) => sha === T ? [C] : sha === M ? [BASE, T] : [],
    diff: () => changes,
    tree: (sha) => sha === M || sha === T ? 'tree-t' : 'tree-c',
    sameTreeDiff: () => true,
    verifyCommit: (sha, authority) => { calls.push({ sha, authority }); return { ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: TRUSTED_FINGERPRINT }; },
    ...overrides,
  };
  return { data, calls };
}
function rejects(overrides, pattern) { const { data } = fixture(overrides); assert.throws(() => verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), pattern); }

test('accepts signed C/T plus an unsigned synthetic merge M without executing marker', () => {
  const { data, calls } = fixture({ readFile: (sha, file) => {
    if (sha === M) throw new Error('synthetic merge executable/data was read');
    return file === POLICY_PATH ? policy : JSON.stringify({ candidate: { sha: C } });
  } });
  assert.deepEqual(verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), { candidateSha: C, integrationTipSha: T, mergeSha: M });
  assert.deepEqual(calls.map(({ sha }) => sha), [C, T]);
  assert.ok(calls.every(({ authority }) => authority.policy === policy && authority.principal === TRUSTED_PRINCIPAL && authority.fingerprint === TRUSTED_FINGERPRINT));
});
test('rejects unsigned C and T', () => {
  rejects({ verifyCommit: () => ({ ok: false }) }, /C is not signed/);
  rejects({ verifyCommit: (sha) => sha === C ? { ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: TRUSTED_FINGERPRINT } : { ok: false } }, /T is not signed/);
});
test('rejects wrong signing key or principal', () => {
  rejects({ verifyCommit: () => ({ ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: 'SHA256:wrong' }) }, /C is not signed/);
  rejects({ verifyCommit: () => ({ ok: true, principal: 'attacker@example.invalid', fingerprint: TRUSTED_FINGERPRINT }) }, /C is not signed/);
});
test('rejects attacker self-authorized C policy and malformed policy', () => {
  rejects({ readFile: (sha, file) => file === POLICY_PATH ? 'attacker@example.invalid ssh-ed25519 AAAA\n' : JSON.stringify({ candidate: { sha: C } }) }, /pinned signer/);
  assert.throws(() => validatePinnedPolicy(policy.trim()), /pinned signer/);
});
test('rejects policy type/mode, policy change, and product change after C', () => {
  rejects({ treeEntry: () => ({ mode: '100755', type: 'blob' }) }, /mode-100644/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: POLICY_PATH, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: 'scripts/console/validate-console-truth-ledger.mjs', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: 'backend/app/src/marker.rs', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
});
test('rejects indirect T and malformed M', () => {
  rejects({ parents: (sha) => sha === T ? [C, BASE] : [BASE, T] }, /direct single-parent/);
  rejects({ parents: (sha) => sha === T ? [C] : [BASE] }, /two-parent merge/);
  rejects({ tree: (sha) => sha === M ? 'tree-m' : 'tree-t' }, /tree\/diff/);
  rejects({ sameTreeDiff: () => false }, /tree\/diff/);
});
test('binds a one-parent squash S to signed C/T without reading T or S executable content', () => {
  const { data, calls } = fixture({
    readFile: (sha, file) => {
      if (sha === S || (sha === T && file !== 'docs/program/console-capability-registry.json')) throw new Error('untrusted executable/data was read');
      return file === POLICY_PATH ? policy : JSON.stringify({ candidate: { sha: C } });
    },
    parents: (sha) => sha === T ? [C] : sha === S ? [BASE] : [],
    tree: (sha) => sha === S || sha === T ? 'tree-t' : 'tree-c',
  });
  assert.deepEqual(verifySquashBinding(data, { authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE }), { candidateSha: C, authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE });
  assert.deepEqual(calls.map(({ sha }) => sha), [C, T]);
});
test('rejects merge, rebase, wrong-base, and tree-drift squash bindings', () => {
  const squash = (overrides) => assert.throws(() => verifySquashBinding(fixture({
    parents: (sha) => sha === T ? [C] : sha === S ? [BASE] : [],
    tree: (sha) => sha === S || sha === T ? 'tree-t' : 'tree-c',
    ...overrides,
  }).data, { authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE }), /authority bootstrap/);
  squash({ parents: (sha) => sha === T ? [C] : sha === S ? [BASE, C] : [] });
  squash({ parents: (sha) => sha === T ? [C, BASE] : sha === S ? [BASE] : [] });
  squash({ parents: (sha) => sha === T ? [C] : sha === S ? [C] : [] });
  squash({ tree: (sha) => sha === S ? 'tree-s' : sha === T ? 'tree-t' : 'tree-c' });
});
test('emits a non-release squash binding receipt that preserves HOLD', () => {
  assert.deepEqual(squashBindingReceipt({ candidateSha: C, authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE }), {
    schema: 'console-squash-binding-v1',
    verdict: 'TREE_BOUND_HOLD_PRESERVED',
    release_disposition: 'HOLD',
    candidateSha: C,
    authorityTipSha: T,
    squashSha: S,
    preMergeBaseSha: BASE,
  });
});
test('candidate compatibility fixture receives only C/T/M environment facts and planner new flags', () => {
  const plan = candidateCheckPlan(C, T, M);
  assert.deepEqual(plan.environment, {
    CONSOLE_CANDIDATE_SHA: C,
    CONSOLE_AUTHORITY_TIP_SHA: T,
    CONSOLE_SYNTHETIC_MERGE_SHA: M,
  });
  assert.deepEqual(plan.commands[1], ['node', ['scripts/console/plan-fanout.mjs', '--candidate', C, '--authority-tip', T, '--synthetic-merge', M]]);
  assert.deepEqual(plan.commands[2][1], ['--test', 'scripts/console/validate-console-truth-ledger.test.mjs', 'scripts/console/plan-fanout.test.mjs', 'scripts/console/verify-console-authority-train.test.mjs']);
});
test('workflow separates open PR authentication from closed merged squash binding', () => {
  const workflow = readFileSync(new URL('../../.github/workflows/console-authority-bootstrap.yml', import.meta.url), 'utf8');
  assert.match(workflow, /types: \[opened, synchronize, reopened, closed\]/);
  assert.match(workflow, /github\.event\.action != 'closed'/);
  assert.match(workflow, /github\.event\.action == 'closed'/);
  assert.match(workflow, /pull_request\.merged == true/);
  assert.match(workflow, /squash-binding/);
  assert.match(workflow, /ref: \$\{\{ github\.event\.pull_request\.merge_commit_sha \}\}/);
  assert.doesNotMatch(workflow, /ref: \$\{\{ github\.event\.pull_request\.merge_commit_sha \}\}\^/);
  assert.match(workflow, /test "\$\(git -c core\.hooksPath=\/dev\/null rev-parse HEAD\)" = "\$SQUASH_SHA"\n\s+git -c core\.hooksPath=\/dev\/null checkout --detach "\$SQUASH_SHA\^"\n\s+node scripts\/console\/verify-console-pr-authority-bootstrap\.mjs squash-binding/);
  assert.match(workflow, /PR_NUMBER: \$\{\{ github\.event\.pull_request\.number \}\}/);
  assert.match(workflow, /--pr-number "\$PR_NUMBER"/);
  const verifier = readFileSync(new URL('./verify-console-pr-authority-bootstrap.mjs', import.meta.url), 'utf8');
  const squashMain = verifier.slice(verifier.indexOf('function squashBindingMain()'));
  assert.ok(squashMain.indexOf('fetchExactAuthorityTip') < squashMain.indexOf('verifySquashBinding'));
});
test('fetches a deleted, non-reachable PR authority tip exactly and rejects mismatch', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'console-authority-fetch-'));
  const remote = path.join(directory, 'remote.git'), source = path.join(directory, 'source'), checkout = path.join(directory, 'checkout');
  const run = (cwd, args) => spawnSync('git', ['-C', cwd, ...args], { encoding: 'utf8' });
  try {
    assert.equal(spawnSync('git', ['init', '--bare', remote], { encoding: 'utf8' }).status, 0);
    assert.equal(spawnSync('git', ['init', source], { encoding: 'utf8' }).status, 0);
    assert.equal(run(source, ['config', 'user.name', 'Bootstrap Test']).status, 0);
    assert.equal(run(source, ['config', 'user.email', 'bootstrap@example.invalid']).status, 0);
    writeFileSync(path.join(source, 'base.txt'), 'base\n');
    assert.equal(run(source, ['add', 'base.txt']).status, 0);
    assert.equal(run(source, ['commit', '-m', 'base']).status, 0);
    const base = run(source, ['rev-parse', 'HEAD']).stdout.trim();
    assert.equal(run(source, ['remote', 'add', 'origin', remote]).status, 0);
    assert.equal(run(source, ['push', 'origin', `HEAD:refs/heads/main`]).status, 0);
    assert.equal(spawnSync('git', ['clone', '--branch', 'main', remote, checkout], { encoding: 'utf8' }).status, 0);
    writeFileSync(path.join(source, 'authority.txt'), 'tip\n');
    assert.equal(run(source, ['add', 'authority.txt']).status, 0);
    assert.equal(run(source, ['commit', '-m', 'authority tip']).status, 0);
    const tip = run(source, ['rev-parse', 'HEAD']).stdout.trim();
    assert.equal(run(source, ['push', 'origin', `HEAD:refs/pull/42/head`]).status, 0);
    assert.equal(run(source, ['reset', '--hard', base]).status, 0);
    assert.notEqual(run(checkout, ['cat-file', '-e', `${tip}^{commit}`]).status, 0);
    assert.equal(fetchExactAuthorityTip(checkout, 42, tip), tip);
    assert.equal(run(checkout, ['rev-parse', 'refs/console-squash-binding/42/head']).stdout.trim(), tip);
    assert.throws(() => fetchExactAuthorityTip(checkout, 42, base), /head ref does not match event SHA/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
test('candidate compatibility fixture executes the actual planner CLI flag contract', () => {
  const candidate = spawnSync('git', ['rev-parse', '28642975^{commit}'], { encoding: 'utf8' }).stdout.trim();
  const directory = mkdtempSync(path.join(tmpdir(), 'console-candidate-planner-'));
  try {
    assert.equal(spawnSync('git', ['worktree', 'add', '--detach', '--no-checkout', directory], { encoding: 'utf8' }).status, 0);
    assert.equal(spawnSync('git', ['-C', directory, 'checkout', '--detach', candidate], { encoding: 'utf8' }).status, 0);
    const accepted = spawnSync('node', ['scripts/console/plan-fanout.mjs', '--candidate', candidate, '--authority-tip', T, '--synthetic-merge', M], { cwd: directory, encoding: 'utf8' });
    assert.doesNotMatch(`${accepted.stdout}${accepted.stderr}`, /unknown argument/);
    const rejected = spawnSync('node', ['scripts/console/plan-fanout.mjs', '--candidate-sha', candidate, '--authority-tip-sha', T, '--synthetic-merge-sha', M], { cwd: directory, encoding: 'utf8' });
    assert.notEqual(rejected.status, 0);
    assert.match(`${rejected.stdout}${rejected.stderr}`, /unknown argument: --candidate-sha/);
  } finally {
    spawnSync('git', ['worktree', 'remove', '--force', directory], { encoding: 'utf8' });
    rmSync(directory, { recursive: true, force: true });
  }
});
test('hostile global Git config cannot replace the pinned verifier or execute its marker', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'console-hostile-git-config-'));
  const marker = path.join(directory, 'marker');
  const attacker = path.join(directory, 'attacker-ssh-program');
  const globalConfig = path.join(directory, 'gitconfig');
  const sha = spawnSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).stdout.trim();
  try {
    writeFileSync(attacker, `#!/bin/sh\nprintf attacked > '${marker}'\nexit 1\n`, { mode: 0o700 });
    writeFileSync(globalConfig, `[gpg "ssh"]\n\tprogram = ${attacker}\n`);
    const result = verifyPinnedSshCommit(process.cwd(), sha, policy, {
      GIT_CONFIG_GLOBAL: globalConfig,
      GIT_CONFIG_COUNT: '1',
      GIT_CONFIG_KEY_0: 'gpg.ssh.program',
      GIT_CONFIG_VALUE_0: attacker,
      HOME: directory,
      XDG_CONFIG_HOME: directory,
    });
    assert.equal(result.ok, true);
    assert.equal(result.principal, TRUSTED_PRINCIPAL);
    assert.equal(result.fingerprint, TRUSTED_FINGERPRINT);
    assert.equal(existsSync(marker), false);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
