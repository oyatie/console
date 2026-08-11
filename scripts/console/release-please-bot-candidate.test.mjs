import assert from 'node:assert/strict';
import test from 'node:test';
import {
  RELEASE_PLEASE_BOT_EMAIL,
  RELEASE_PLEASE_BOT_NAME,
  RELEASE_PLEASE_COMMITTER_EMAIL,
  RELEASE_PLEASE_COMMITTER_NAME,
  RELEASE_PLEASE_PATHS,
  RELEASE_PLEASE_TRAIN_CLASS,
  assertReleasePleaseBotIdentity,
  assertReleasePleaseBotPathDiff,
  assertTrustedReleasePleasePrMeta,
  classifyReleasePleaseBotTip,
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
    /only \.release-please-manifest\.json and CHANGELOG\.md/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff([releaseDiff[0]]),
    /must change both/,
  );
  assert.throws(
    () => assertReleasePleaseBotPathDiff(releaseDiff.map((entry) => ({ ...entry, status: 'A', oldMode: '000000' }))),
    /mode-100644 modifications/,
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
