import assert from 'node:assert/strict';
import test from 'node:test';
import {
  RELEASE_PLEASE_BOT_EMAIL,
  RELEASE_PLEASE_BOT_NAME,
  RELEASE_PLEASE_COMMITTER_EMAIL,
  RELEASE_PLEASE_COMMITTER_NAME,
  RELEASE_PLEASE_CUSTODY_PATHS,
  RELEASE_PLEASE_PATHS,
  RELEASE_PLEASE_TRAIN_CLASS,
  assertReleasePleaseBotIdentity,
  assertReleasePleaseBotPathDiff,
  assertTrustedReleasePleasePrMeta,
  classifyReleasePleaseBotTip,
  classifyReleasePleaseSquashBinding,
  verifyReleasePleaseBotTrain,
} from './release-please-bot-candidate.mjs';

const C = 'c'.repeat(40);
const T = 'a'.repeat(40);
const M = 'b'.repeat(40);
const BASE = 'd'.repeat(40);

const botIdentity = {
  authorName: RELEASE_PLEASE_BOT_NAME,
  authorEmail: RELEASE_PLEASE_BOT_EMAIL,
  committerName: RELEASE_PLEASE_COMMITTER_NAME,
  committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
  subject: 'chore(main): release 0.3.4',
};

const releaseDiff = RELEASE_PLEASE_PATHS.map((path) => ({
  path,
  status: 'M',
  oldMode: '100644',
  newMode: '100644',
  oldType: 'blob',
  newType: 'blob',
}));

const custodyDiff = RELEASE_PLEASE_CUSTODY_PATHS.map((path) => ({
  path,
  status: 'M',
  oldMode: '100644',
  newMode: '100644',
  oldType: 'blob',
  newType: 'blob',
}));

const releaseWithCustodyDiff = [...releaseDiff, ...custodyDiff];

function ops(overrides = {}) {
  return {
    hasCommit: () => true,
    parents: (sha) => (sha === T ? [C] : sha === M ? [BASE, T] : []),
    commitIdentity: () => botIdentity,
    diff: () => releaseDiff,
    tree: (sha) => (sha === M || sha === T ? 'tree-t' : 'tree-c'),
    sameTreeDiff: () => true,
    ...overrides,
  };
}

test('accepts exact bot identity, subject, and two-file path allow-list', () => {
  assert.doesNotThrow(() => assertReleasePleaseBotIdentity(botIdentity));
  assert.doesNotThrow(() => assertReleasePleaseBotPathDiff(releaseDiff));
  assert.doesNotThrow(() => assertTrustedReleasePleasePrMeta({
    prAuthorLogin: 'github-actions[bot]',
    prHeadRef: 'release-please--branches--main--components--console',
  }));
  const classified = classifyReleasePleaseBotTip(ops(), T);
  assert.equal(classified.trainClass, RELEASE_PLEASE_TRAIN_CLASS);
  assert.equal(classified.candidateSha, C);
  assert.deepEqual(
    verifyReleasePleaseBotTrain(ops(), {
      headSha: T,
      mergeSha: M,
      prAuthorLogin: 'github-actions[bot]',
      prHeadRef: 'release-please--branches--main--components--console',
      requirePrMeta: true,
    }),
    {
      trainClass: RELEASE_PLEASE_TRAIN_CLASS,
      candidateSha: C,
      integrationTipSha: T,
      authorityTipSha: T,
      mergeSha: M,
    },
  );
});

test('accepts core release paths plus regenerated documentation custody pair', () => {
  assert.doesNotThrow(() => assertReleasePleaseBotPathDiff(releaseWithCustodyDiff));
  const classified = classifyReleasePleaseBotTip(ops({ diff: () => releaseWithCustodyDiff }), T);
  assert.equal(classified.trainClass, RELEASE_PLEASE_TRAIN_CLASS);
  assert.deepEqual(
    verifyReleasePleaseBotTrain(ops({ diff: () => releaseWithCustodyDiff }), {
      headSha: T,
      mergeSha: M,
      requirePrMeta: false,
    }),
    {
      trainClass: RELEASE_PLEASE_TRAIN_CLASS,
      candidateSha: C,
      integrationTipSha: T,
      authorityTipSha: T,
      mergeSha: M,
    },
  );
});

test('rejects forged author, wrong committer, bad subject, and path sprawl', () => {
  assert.throws(
    () => assertReleasePleaseBotIdentity({ ...botIdentity, authorEmail: 'attacker@example.invalid' }),
    /tip author/,
  );
  assert.throws(
    () => assertReleasePleaseBotIdentity({ ...botIdentity, committerEmail: 'attacker@example.invalid' }),
    /tip committer/,
  );
  assert.throws(
    () => assertReleasePleaseBotIdentity({ ...botIdentity, subject: 'chore(main): bump deps' }),
    /tip subject/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff([...releaseDiff, { path: 'README.md', status: 'M', oldMode: '100644', newMode: '100644' }]),
    /documentation custody pair/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff([releaseDiff[0]]),
    /must change both/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff(releaseDiff.map((entry) => ({ ...entry, status: 'A', oldMode: '000000' }))),
    /mode-100644 modifications/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff([...releaseDiff, custodyDiff[0]]),
    /documentation custody paths must be regenerated together/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff(custodyDiff),
    /must change both/,
  );
  assert.equal(classifyReleasePleaseBotTip(ops({ commitIdentity: () => ({ ...botIdentity, authorName: 'Eve' }) }), T), null);
  assert.equal(
    classifyReleasePleaseBotTip(ops({
      diff: () => [...releaseDiff, { path: 'scripts/console/attack.mjs', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }],
    }), T),
    null,
  );
});

test('bootstrap requirePrMeta rejects human PR author and non-release head ref', () => {
  assert.throws(
    () => assertTrustedReleasePleasePrMeta({
      prAuthorLogin: 'jason931225',
      prHeadRef: 'release-please--branches--main--components--console',
    }),
    /PR author/,
  );
  assert.throws(
    () => assertTrustedReleasePleasePrMeta({
      prAuthorLogin: 'github-actions[bot]',
      prHeadRef: 'evil-branch',
    }),
    /PR head ref/,
  );
  assert.throws(
    () => verifyReleasePleaseBotTrain(ops(), {
      headSha: T,
      mergeSha: M,
      prAuthorLogin: 'jason931225',
      prHeadRef: 'release-please--branches--main--components--console',
      requirePrMeta: true,
    }),
    /PR author/,
  );
});

test('structural M checks remain fail-closed for bot tips', () => {
  assert.throws(
    () => verifyReleasePleaseBotTrain(ops({ parents: (sha) => (sha === T ? [C] : [BASE]) }), {
      headSha: T,
      mergeSha: M,
      requirePrMeta: false,
    }),
    /two-parent merge/,
  );
  assert.throws(
    () => verifyReleasePleaseBotTrain(ops({ tree: (sha) => (sha === T ? 'tree-t' : 'tree-other') }), {
      headSha: T,
      mergeSha: M,
      requirePrMeta: false,
    }),
    /tree\/diff/,
  );
});

test('admits main squash that tree-binds a classifiable release-please tip; forged off-main C stays null', () => {
  const S = 'e'.repeat(40);
  const T0 = T;
  const baseOps = {
    hasCommit: (sha) => sha === S || sha === T0 || sha === C || sha === BASE,
    parents: (sha) => (sha === S || sha === T0 ? [C] : []),
    commitIdentity: (sha) => (sha === T0
      ? botIdentity
      : { ...botIdentity, subject: 'chore(main): release 0.3.4 (#621)' }),
    diff: () => releaseDiff,
    tree: (sha) => (sha === S || sha === T0 ? 'tree-release' : 'tree-other'),
    sameTreeDiff: (left, right) => left === S && right === T0 || left === T0 && right === S,
    mainTip: () => S,
    isAncestor: (ancestor, descendant) => ancestor === S && descendant === S,
    releasePleasePreMergeTip: (squash) => (squash === S ? T0 : null),
  };
  const admitted = classifyReleasePleaseSquashBinding(baseOps, S);
  assert.equal(admitted.admittedCandidateSha, S);
  assert.equal(admitted.releasePleaseTipSha, T0);
  assert.equal(admitted.candidateSha, C);
  assert.equal(admitted.preMergeBaseSha, C);
  assert.equal(classifyReleasePleaseBotTip(baseOps, S), null, 'squash subject with (#N) must not classify as live tip');

  assert.equal(
    classifyReleasePleaseSquashBinding({
      ...baseOps,
      mainTip: () => BASE,
      isAncestor: () => false,
    }, S),
    null,
    'forged unsigned C off main must not admit',
  );
  assert.equal(
    classifyReleasePleaseSquashBinding({
      ...baseOps,
      tree: (sha) => (sha === S ? 'tree-forged' : sha === T0 ? 'tree-release' : 'tree-other'),
      sameTreeDiff: () => false,
    }, S),
    null,
    'tree drift vs T0 must not admit',
  );
  assert.equal(
    classifyReleasePleaseSquashBinding({
      ...baseOps,
      commitIdentity: (sha) => (sha === T0
        ? { ...botIdentity, authorName: 'Eve' }
        : { ...botIdentity, subject: 'chore(main): release 0.3.4 (#621)' }),
    }, S),
    null,
    'T0 that fails bot identity must not admit the squash',
  );
});
