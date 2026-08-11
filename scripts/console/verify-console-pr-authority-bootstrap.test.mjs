import assert from 'node:assert/strict';
import test from 'node:test';
import { AUTHORITY_PATHS, POLICY_PATH, TRUSTED_ALLOWED_SIGNER, TRUSTED_FINGERPRINT, TRUSTED_PRINCIPAL, candidateCheckPlan, fetchExactAuthorityTip, squashBindingReceipt, validatePinnedPolicy, verifyBootstrapGraph, verifyPinnedSshCommit, verifySquashBinding } from './verify-console-pr-authority-bootstrap.mjs';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import path from 'node:path';

const C = 'c'.repeat(40), T = 't'.repeat(40).replace(/t/g, 'a'), M = 'b'.repeat(40), S = 'e'.repeat(40), BASE = 'd'.repeat(40);
// Pinned as a literal on purpose: this is the one prefix where an ADDED file is admissible, so
// the test must state it rather than echo whatever the module happens to define.
const LEDGER_DIRECTORY = 'docs/program/ledger/';
const policy = `${TRUSTED_ALLOWED_SIGNER}\n`;
const changes = AUTHORITY_PATHS.map((path) => ({ path, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }));
function fixture(overrides = {}) {
  const calls = [];
  const data = {
    hasCommit: () => true,
    // The signing policy at C is the ONLY file this gate reads. Anything else is a document the
    // pull request controls, and the gate no longer consults one to locate C.
    readFile: (sha, file) => { if (file !== POLICY_PATH) throw new Error(`unexpected read of ${file}`); return policy; },
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
function accepts(overrides) { const { data } = fixture(overrides); assert.deepEqual(verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), { candidateSha: C, integrationTipSha: T, mergeSha: M, trainClass: 'ssh-authority' }); }
const modified = (file) => ({ path: file, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' });
const addedFile = (file) => ({ path: file, status: 'A', oldMode: '000000', newMode: '100644', oldType: null, newType: 'blob' });

test('accepts signed C/T plus an unsigned synthetic merge M without executing marker', () => {
  const { data, calls } = fixture({ readFile: (sha, file) => {
    if (sha === M) throw new Error('synthetic merge executable/data was read');
    if (file !== POLICY_PATH) throw new Error(`unexpected read of ${file}`);
    return policy;
  } });
  assert.deepEqual(verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), { candidateSha: C, integrationTipSha: T, mergeSha: M, trainClass: 'ssh-authority' });
  assert.deepEqual(calls.map(({ sha }) => sha), [C, T]);
  assert.ok(calls.every(({ authority }) => authority.policy === policy && authority.principal === TRUSTED_PRINCIPAL && authority.fingerprint === TRUSTED_FINGERPRINT));
});
test('rejects unsigned C and T', () => {
  rejects({ verifyCommit: () => ({ ok: false }) }, /C is not signed/);
  rejects({ verifyCommit: (sha) => sha === C ? { ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: TRUSTED_FINGERPRINT } : { ok: false } }, /T is not signed/);
});
test('fork and Dependabot heads fail closed under the same pinned C/T policy', () => {
  for (const source of ['fork', 'dependabot']) {
    const verified = [];
    const { data } = fixture({
      verifyCommit: (sha) => {
        verified.push(sha);
        return { ok: false, principal: `${source}@example.invalid`, fingerprint: 'SHA256:untrusted' };
      },
    });
    assert.throws(
      () => verifyBootstrapGraph(data, { headSha: T, mergeSha: M }),
      /C is not signed by the pinned SSH authority/,
    );
    assert.deepEqual(verified, [C], `${source} head reached verification past an untrusted C`);
  }
});
test('rejects wrong signing key or principal', () => {
  rejects({ verifyCommit: () => ({ ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: 'SHA256:wrong' }) }, /C is not signed/);
  rejects({ verifyCommit: () => ({ ok: true, principal: 'attacker@example.invalid', fingerprint: TRUSTED_FINGERPRINT }) }, /C is not signed/);
});
test('rejects attacker self-authorized C policy and malformed policy', () => {
  rejects({ readFile: () => 'attacker@example.invalid ssh-ed25519 AAAA\n' }, /pinned signer/);
  assert.throws(() => validatePinnedPolicy(policy.trim()), /pinned signer/);
});
test('rejects policy type/mode, policy change, and product change after C', () => {
  rejects({ treeEntry: () => ({ mode: '100755', type: 'blob' }) }, /mode-100644/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: POLICY_PATH, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: 'scripts/console/validate-console-truth-ledger.mjs', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
  rejects({ diff: () => [...changes.slice(0, 2), { path: 'backend/app/src/marker.rs', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /authority documents/);
});
test('C..T may touch one authority document, and may ADD files only under the ledger directory', () => {
  // A single document is enough. "All three" was the shared-file mutex, not the allow-list.
  for (const file of AUTHORITY_PATHS) accepts({ diff: () => [modified(file)] });
  // A NEW file is status `A`, accepted under the ledger prefix …
  accepts({ diff: () => [addedFile(`${LEDGER_DIRECTORY}2026-08-01-pr-1.md`)] });
  accepts({ diff: () => [modified(`${LEDGER_DIRECTORY}0001-existing.md`)] });
  // The prefix is a FLAT directory of `.md` entries, and this is the only reader that can be
  // handed a path Git itself would never produce — the fixture supplies the diff. A bare
  // `startsWith` accepted `docs/program/ledger/../../evil`, which names a path OUTSIDE the
  // prefix, and it accepted an added `.mjs`: executable content, added by the one commit that
  // is otherwise forbidden to touch a product path.
  for (const entry of ['../../evil', '../console-capability-registry.json', 'nested/2026-08-01-pr-1.md', 'entry.mjs', 'entry.md.mjs', '.hidden.md', 'entry.txt', '', 'sub/']) {
    rejects({ diff: () => [addedFile(`${LEDGER_DIRECTORY}${entry}`)] }, /docs\/program\/ledger\//);
    rejects({ diff: () => [modified(`${LEDGER_DIRECTORY}${entry}`)] }, /docs\/program\/ledger\//);
  }
  // … and nowhere else. A near-miss sibling of the prefix is not the prefix, and the registers
  // and the legacy ledger stay modify-only: `A` on any of them is still an unsupported change.
  for (const file of ['docs/program/ledgerbook.md', 'docs/program/console-program-ledger-2.md', 'README.md', 'scripts/console/attack.mjs', ...AUTHORITY_PATHS]) {
    rejects({ diff: () => [addedFile(file)] }, /docs\/program\/ledger\//);
  }
  // The mode/type checks survive the new status.
  rejects({ diff: () => [{ ...addedFile(`${LEDGER_DIRECTORY}x.md`), newMode: '100755' }] }, /mode-100644/);
  rejects({ diff: () => [{ ...addedFile(`${LEDGER_DIRECTORY}x.md`), newMode: '120000' }] }, /mode-100644/);
  rejects({ diff: () => [{ ...modified(`${LEDGER_DIRECTORY}x.md`), status: 'D', newMode: '000000', newType: null }] }, /mode-100644/);
  // An empty C..T asserts nothing about authority at all.
  rejects({ diff: () => [] }, /at least one authority document/);
});

test('C is located from Git parentage alone; no pull-request-controlled document is read', () => {
  // The default fixture THROWS on any read other than the signing policy at C, so a plain accept
  // is the assertion: this gate cannot be steered by anything the pull request writes into a
  // file. The registry's `candidate.sha` used to be that steering wheel.
  const { data, calls } = fixture();
  assert.deepEqual(verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), { candidateSha: C, integrationTipSha: T, mergeSha: M, trainClass: 'ssh-authority' });
  assert.deepEqual(calls.map(({ sha }) => sha), [C, T]);
  // Parentage is the whole locator, so a tip with no parent or more than one has no C at all.
  rejects({ parents: (sha) => sha === T ? [] : [BASE, T] }, /direct single-parent/);
  rejects({ parents: (sha) => sha === T ? [C, BASE] : [BASE, T] }, /direct single-parent/);
  rejects({ parents: (sha) => sha === T ? ['not-a-sha'] : [BASE, T] }, /lowercase 40-character SHA/);
  // And C itself must still resolve to a real object.
  rejects({ hasCommit: (sha) => sha !== C }, /C object is unavailable/);
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
      if (sha !== C || file !== POLICY_PATH) throw new Error('untrusted executable/data was read');
      return policy;
    },
    parents: (sha) => sha === T ? [C] : sha === S ? [BASE] : [],
    tree: (sha) => sha === S || sha === T ? 'tree-t' : 'tree-c',
  });
  assert.deepEqual(verifySquashBinding(data, { authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE }), { candidateSha: C, authorityTipSha: T, squashSha: S, preMergeBaseSha: BASE, trainClass: 'ssh-authority' });
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
  // This file gates the highest-privilege script in the repository and was absent from the list,
  // so the `pull_request_target` path — the one that actually decides the merge — ran every
  // console check EXCEPT the one covering the verifier making the decision.
  assert.deepEqual(plan.commands[2][1], ['--test', 'scripts/console/validate-console-truth-ledger.test.mjs', 'scripts/console/plan-fanout.test.mjs', 'scripts/console/verify-console-authority-train.test.mjs', 'scripts/console/verify-console-pr-authority-bootstrap.test.mjs', 'scripts/console/release-please-bot-candidate.test.mjs']);
});
test('workflow separates open PR authentication from closed merged squash binding', () => {
  const workflow = readFileSync(new URL('../../.github/workflows/console-authority-bootstrap.yml', import.meta.url), 'utf8');
  // This suite is deliberately executed by the protected-target bootstrap before
  // npm install. Keep it dependency-free: exact bytes plus focused structural
  // assertions are the hermetic contract. The normal CI preflight independently
  // parses this workflow with js-yaml after the lockfile install boundary.
  assert.equal(
    createHash('sha256').update(workflow).digest('hex'),
    'ec6d1b8d96e7850f8bd433e8d878d9098980fdf4ef068823529c7b468c9af709',
  );
  assert.match(workflow, /^on:\n  pull_request_target:\n    types: \[opened, synchronize, reopened, closed\]\n    branches:\n      - main$/m);
  assert.doesNotMatch(workflow, /^  pull_request:$/m);
  assert.match(workflow, /^permissions:\n  contents: read$/m);
  const authenticateStart = workflow.indexOf('  authenticate-console-authority:');
  const squashStart = workflow.indexOf('  bind-merged-console-authority-squash:');
  assert.ok(authenticateStart >= 0 && squashStart > authenticateStart);
  const authenticate = workflow.slice(authenticateStart, squashStart);
  const squashBinding = workflow.slice(squashStart);
  assert.match(authenticate, /if: >-\n      github\.event\.action != 'closed' &&\n      github\.event\.pull_request\.base\.ref == 'main'/);
  assert.doesNotMatch(authenticate, /head\.repo|github\.repository/);
  assert.match(squashBinding, /github\.event\.pull_request\.head\.repo\.full_name == github\.repository/);
  assert.match(workflow, /github\.event\.action == 'closed'/);
  assert.match(workflow, /pull_request\.merged == true/);
  assert.match(workflow, /squash-binding/);
  assert.match(workflow, /ref: \$\{\{ github\.event\.pull_request\.merge_commit_sha \}\}/);
  assert.doesNotMatch(workflow, /ref: \$\{\{ github\.event\.pull_request\.merge_commit_sha \}\}\^/);
  assert.match(workflow, /test "\$\(git -c core\.hooksPath=\/dev\/null rev-parse HEAD\)" = "\$SQUASH_SHA"\n\s+git -c core\.hooksPath=\/dev\/null checkout --detach "\$SQUASH_SHA\^"\n\s+node scripts\/console\/verify-console-pr-authority-bootstrap\.mjs squash-binding/);
  assert.match(workflow, /PR_NUMBER: \$\{\{ github\.event\.pull_request\.number \}\}/);
  assert.match(workflow, /--pr-number "\$PR_NUMBER"/);
  assert.match(authenticate, /- name: Checkout protected target code only\n        uses: actions\/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7\n        with:\n          ref: main\n          persist-credentials: false\n          fetch-depth: 0/);
  assert.equal([...authenticate.matchAll(/^      - name:/gm)].length, 2);
  const verificationStep = authenticate.slice(authenticate.indexOf('      - name: Verify C/T authority train before candidate execution'));
  assert.match(verificationStep, /run: \|\n          node scripts\/console\/verify-console-pr-authority-bootstrap\.mjs \\/);
  assert.match(verificationStep, /--author "\$PR_AUTHOR"/);
  assert.match(verificationStep, /--head-ref "\$PR_HEAD_REF"/);
  assert.doesNotMatch(verificationStep, /checkout|npm|candidateCheckPlan|runAuthenticatedCandidateChecks/);
  const verifier = readFileSync(new URL('./verify-console-pr-authority-bootstrap.mjs', import.meta.url), 'utf8');
  const openMain = verifier.slice(verifier.indexOf('function main()'), verifier.indexOf('function squashBindingMain()'));
  const openOrder = ['fetchExactPullObjects', 'verifyBootstrapGraph', 'runAuthenticatedCandidateChecks'].map((needle) => openMain.indexOf(needle));
  assert.ok(openOrder.every((index) => index >= 0));
  assert.ok(openOrder[0] < openOrder[1] && openOrder[1] < openOrder[2]);
  const squashMain = verifier.slice(verifier.indexOf('function squashBindingMain()'));
  assert.ok(squashMain.indexOf('fetchExactAuthorityTip') < squashMain.indexOf('verifySquashBinding'));
});

test('admits release-please bot tip only with event author/ref + docs-only diff; never calls SSH verify', () => {
  const {
    RELEASE_PLEASE_BOT_EMAIL,
    RELEASE_PLEASE_BOT_NAME,
    RELEASE_PLEASE_COMMITTER_EMAIL,
    RELEASE_PLEASE_COMMITTER_NAME,
    RELEASE_PLEASE_PATHS,
    RELEASE_PLEASE_TRAIN_CLASS,
  } = requireReleasePleaseConstants();
  const releaseDiff = RELEASE_PLEASE_PATHS.map((path) => ({
    path, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob',
  }));
  const calls = [];
  const { data } = fixture({
    commitIdentity: () => ({
      authorName: RELEASE_PLEASE_BOT_NAME,
      authorEmail: RELEASE_PLEASE_BOT_EMAIL,
      committerName: RELEASE_PLEASE_COMMITTER_NAME,
      committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
      subject: 'chore(main): release 0.3.4',
    }),
    diff: () => releaseDiff,
    verifyCommit: (sha) => { calls.push(sha); return { ok: false }; },
    readFile: () => { throw new Error('release class must not read signing policy'); },
  });
  assert.deepEqual(verifyBootstrapGraph(data, {
    headSha: T,
    mergeSha: M,
    prAuthorLogin: 'github-actions[bot]',
    prHeadRef: 'release-please--branches--main--components--console',
  }), {
    candidateSha: C,
    integrationTipSha: T,
    mergeSha: M,
    trainClass: RELEASE_PLEASE_TRAIN_CLASS,
  });
  assert.deepEqual(calls, [], 'release-please class must not invoke SSH verify-commit');
  assert.throws(() => verifyBootstrapGraph(data, {
    headSha: T,
    mergeSha: M,
    prAuthorLogin: 'jason931225',
    prHeadRef: 'release-please--branches--main--components--console',
  }), /PR author/);
  assert.throws(() => verifyBootstrapGraph(fixture({
    commitIdentity: () => ({
      authorName: RELEASE_PLEASE_BOT_NAME,
      authorEmail: RELEASE_PLEASE_BOT_EMAIL,
      committerName: RELEASE_PLEASE_COMMITTER_NAME,
      committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
      subject: 'chore(main): release 0.3.4',
    }),
    diff: () => [...releaseDiff, { path: 'README.md', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }],
    verifyCommit: () => ({ ok: false }),
  }).data, {
    headSha: T,
    mergeSha: M,
    prAuthorLogin: 'github-actions[bot]',
    prHeadRef: 'release-please--branches--main--components--console',
  }), /not a release-please bot|authority documents|C is not signed/);
});

test('admits unsigned main release squash as C when it tree-binds classifiable T0; forged C still needs SSH', () => {
  const {
    RELEASE_PLEASE_BOT_EMAIL,
    RELEASE_PLEASE_BOT_NAME,
    RELEASE_PLEASE_COMMITTER_EMAIL,
    RELEASE_PLEASE_COMMITTER_NAME,
    RELEASE_PLEASE_PATHS,
  } = requireReleasePleaseConstants();
  const T0 = 'f'.repeat(40);
  const releaseDiff = RELEASE_PLEASE_PATHS.map((path) => ({
    path, status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob',
  }));
  const jurisdictionOnly = [modified('docs/program/console-jurisdiction-register.json')];
  const verified = [];
  const { data } = fixture({
    parents: (sha) => {
      if (sha === T) return [C];
      if (sha === C || sha === T0) return [BASE];
      if (sha === M) return [BASE, T];
      return [];
    },
    commitIdentity: (sha) => {
      if (sha === T0) {
        return {
          authorName: RELEASE_PLEASE_BOT_NAME,
          authorEmail: RELEASE_PLEASE_BOT_EMAIL,
          committerName: RELEASE_PLEASE_COMMITTER_NAME,
          committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
          subject: 'chore(main): release 0.3.4',
        };
      }
      return {
        authorName: RELEASE_PLEASE_BOT_NAME,
        authorEmail: RELEASE_PLEASE_BOT_EMAIL,
        committerName: RELEASE_PLEASE_COMMITTER_NAME,
        committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
        subject: 'chore(main): release 0.3.4 (#621)',
      };
    },
    diff: (from, to) => {
      if ((from === BASE && to === T0) || (from === BASE && to === C)) return releaseDiff;
      if (from === C && to === T) return jurisdictionOnly;
      return [];
    },
    tree: (sha) => {
      if (sha === C || sha === T0) return 'tree-release';
      if (sha === T || sha === M) return 'tree-t';
      return 'tree-base';
    },
    sameTreeDiff: (left, right) => (
      (left === C && right === T0)
      || (left === T0 && right === C)
      || (left === M && right === T)
      || (left === T && right === M)
    ),
    mainTip: () => C,
    isAncestor: (ancestor, descendant) => ancestor === C && descendant === C,
    releasePleasePreMergeTip: (squash) => (squash === C ? T0 : null),
    verifyCommit: (sha, authority) => {
      verified.push(sha);
      if (sha === C) return { ok: false };
      return { ok: true, principal: TRUSTED_PRINCIPAL, fingerprint: TRUSTED_FINGERPRINT };
    },
  });
  assert.deepEqual(verifyBootstrapGraph(data, { headSha: T, mergeSha: M }), {
    candidateSha: C,
    integrationTipSha: T,
    mergeSha: M,
    trainClass: 'ssh-authority',
  });
  assert.deepEqual(verified, [T], 'release squash C must skip SSH; T must still verify');

  // Forged unsigned C off main: no squash binding → SSH bar remains.
  assert.throws(() => verifyBootstrapGraph(fixture({
    mainTip: () => BASE,
    isAncestor: () => false,
    releasePleasePreMergeTip: () => T0,
    verifyCommit: () => ({ ok: false }),
  }).data, { headSha: T, mergeSha: M }), /C is not signed by the pinned SSH authority/);
});

function requireReleasePleaseConstants() {
  // Keep this suite dependency-free of a second import graph in the hermetic pin section above.
  return {
    RELEASE_PLEASE_BOT_NAME: 'github-actions[bot]',
    RELEASE_PLEASE_BOT_EMAIL: '41898282+github-actions[bot]@users.noreply.github.com',
    RELEASE_PLEASE_COMMITTER_NAME: 'GitHub',
    RELEASE_PLEASE_COMMITTER_EMAIL: 'noreply@github.com',
    RELEASE_PLEASE_PATHS: ['.release-please-manifest.json', 'CHANGELOG.md'],
    RELEASE_PLEASE_TRAIN_CLASS: 'release-please-bot',
  };
}
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
  // This resolves HEAD because the candidate is whatever commit the gate will actually
  // check out. It used to pin `28642975`, which is an ancestor of no remote ref — it
  // survives only in the local branch it was authored on. So the assertion below could
  // only ever hold in one clone, and nothing caught that because this file was wired
  // into no workflow: `git rev-parse` on an absent object prints nothing, `candidate`
  // became the empty string, and `checkout --detach ''` exits 128 with no mention of
  // the SHA that was missing. The `assert.match` is what turns that into a readable
  // failure if HEAD ever fails to resolve.
  const candidate = spawnSync('git', ['rev-parse', 'HEAD^{commit}'], { encoding: 'utf8' }).stdout.trim();
  assert.match(candidate, /^[0-9a-f]{40}$/, 'candidate commit must resolve before it can be checked out');
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
  // Proving the REAL verifier ran needs a genuinely signed commit, and this file has to be
  // runnable from a clean CI checkout, where none is reachable: main's history is unsigned
  // squash commits and a `pull_request` HEAD is GitHub's unsigned synthetic merge. It used to
  // verify `HEAD` against the pinned production policy, which is why it passed only on a
  // candidate branch and is one reason it never ran anywhere. The fixture signs its own commit
  // and pins its own key; what is under test is the Git-config sanitisation, and the pinning of
  // the production signer is asserted separately by `validatePinnedPolicy`.
  const directory = mkdtempSync(path.join(tmpdir(), 'console-hostile-git-config-'));
  const repo = path.join(directory, 'repo');
  const marker = path.join(directory, 'marker');
  const attacker = path.join(directory, 'attacker-ssh-program');
  const globalConfig = path.join(directory, 'gitconfig');
  const run = (args) => spawnSync('git', ['-C', repo, ...args], { encoding: 'utf8' });
  const principal = 'bootstrap@example.invalid';
  try {
    assert.equal(spawnSync('git', ['init', repo], { encoding: 'utf8' }).status, 0);
    const key = path.join(directory, 'signing_key');
    assert.equal(spawnSync('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', key], { encoding: 'utf8' }).status, 0);
    for (const entry of [['user.name', 'Bootstrap Test'], ['user.email', principal], ['gpg.format', 'ssh'], ['user.signingkey', key]]) assert.equal(run(['config', ...entry]).status, 0);
    writeFileSync(path.join(repo, 'base.txt'), 'base\n');
    assert.equal(run(['add', '--', 'base.txt']).status, 0);
    assert.equal(run(['commit', '-S', '-m', 'signed base']).status, 0);
    const sha = run(['rev-parse', 'HEAD']).stdout.trim();
    const publicKey = readFileSync(`${key}.pub`, 'utf8').trim().split(/\s+/).slice(0, 2).join(' ');
    const fingerprint = spawnSync('ssh-keygen', ['-lf', `${key}.pub`, '-E', 'sha256'], { encoding: 'utf8' }).stdout.trim().split(/\s+/)[1];
    writeFileSync(attacker, `#!/bin/sh\nprintf attacked > '${marker}'\nexit 1\n`, { mode: 0o700 });
    writeFileSync(globalConfig, `[gpg "ssh"]\n\tprogram = ${attacker}\n`);
    const result = verifyPinnedSshCommit(repo, sha, `${principal} ${publicKey}\n`, {
      GIT_CONFIG_GLOBAL: globalConfig,
      GIT_CONFIG_COUNT: '1',
      GIT_CONFIG_KEY_0: 'gpg.ssh.program',
      GIT_CONFIG_VALUE_0: attacker,
      HOME: directory,
      XDG_CONFIG_HOME: directory,
    });
    assert.equal(result.ok, true);
    assert.equal(result.principal, principal);
    assert.equal(result.fingerprint, fingerprint);
    assert.equal(existsSync(marker), false);
    // The production identities stay pinned constants, not values read from anywhere.
    assert.equal(TRUSTED_ALLOWED_SIGNER, `${TRUSTED_PRINCIPAL} ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAgMAp8vHS9V/9UQQVTa5FtmS9Q9fdB8I520DsZMMDTR`);
    // Pinned by VALUE, not by shape. A format-only assertion let the fingerprint be swapped for
    // any well-formed string with every suite still green — the constant is what decides which
    // key may sign the highest-privilege commits in this repository.
    assert.equal(TRUSTED_FINGERPRINT, 'SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
