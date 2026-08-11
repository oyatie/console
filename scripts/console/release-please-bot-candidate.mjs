/**
 * Fail-closed admission class for release-please bot candidates.
 *
 * Product authority trains still require pinned SSH signatures on C and T.
 * Release Please cannot produce those signatures (GitHub web-flow / Actions at
 * best). This module admits ONLY tips that satisfy every clause below — not a
 * broad unsigned exception:
 *
 * 1. Exact github-actions[bot] author + GitHub noreply committer
 * 2. Subject `chore(<scope>): release X.Y.Z`
 * 3. Parent..tip diff is exactly `.release-please-manifest.json` + `CHANGELOG.md`
 *    (regular mode-100644 modifications, no adds/deletes/renames)
 * 4. On the pull_request_target bootstrap path: GitHub event PR author is
 *    `github-actions[bot]` and head ref matches the release-please branch pattern
 *    (commit headers alone are forgeable by a write-access collaborator)
 */

import { AUTHORITY_DIFF_ARGS } from './authority-ledger-path.mjs';

export const RELEASE_PLEASE_BOT_NAME = 'github-actions[bot]';
export const RELEASE_PLEASE_BOT_EMAIL = '41898282+github-actions[bot]@users.noreply.github.com';
export const RELEASE_PLEASE_COMMITTER_NAME = 'GitHub';
export const RELEASE_PLEASE_COMMITTER_EMAIL = 'noreply@github.com';
export const RELEASE_PLEASE_PR_AUTHORS = Object.freeze(['github-actions[bot]']);
export const RELEASE_PLEASE_HEAD_REF = /^release-please--branches--main--components--[A-Za-z0-9._-]+$/;
export const RELEASE_PLEASE_SUBJECT = /^chore\([^)]+\): release \d+\.\d+\.\d+$/;
export const RELEASE_PLEASE_PATHS = Object.freeze(['.release-please-manifest.json', 'CHANGELOG.md']);
export const RELEASE_PLEASE_TRAIN_CLASS = 'release-please-bot';

const SHA = /^[0-9a-f]{40}$/;

function fail(message) {
  throw new Error(`release-please bot candidate: ${message}`);
}

export function assertTrustedReleasePleasePrMeta({ prAuthorLogin, prHeadRef } = {}) {
  if (!RELEASE_PLEASE_PR_AUTHORS.includes(prAuthorLogin)) {
    fail('PR author must be github-actions[bot] (event payload, not commit header)');
  }
  if (typeof prHeadRef !== 'string' || !RELEASE_PLEASE_HEAD_REF.test(prHeadRef)) {
    fail('PR head ref must match release-please--branches--main--components--*');
  }
}

export function assertReleasePleaseBotIdentity(identity) {
  if (!identity || typeof identity !== 'object') fail('commit identity is unavailable');
  const { authorName, authorEmail, committerName, committerEmail, subject } = identity;
  if (authorName !== RELEASE_PLEASE_BOT_NAME || authorEmail !== RELEASE_PLEASE_BOT_EMAIL) {
    fail('tip author must be exactly github-actions[bot]');
  }
  if (committerName !== RELEASE_PLEASE_COMMITTER_NAME || committerEmail !== RELEASE_PLEASE_COMMITTER_EMAIL) {
    fail('tip committer must be exactly GitHub <noreply@github.com>');
  }
  if (typeof subject !== 'string' || !RELEASE_PLEASE_SUBJECT.test(subject)) {
    fail('tip subject must match chore(<scope>): release X.Y.Z');
  }
}

export function assertReleasePleaseBotPathDiff(changes) {
  if (!Array.isArray(changes)) fail('parent..tip diff is unavailable');
  const expected = new Set(RELEASE_PLEASE_PATHS);
  const seen = new Set();
  for (const change of changes) {
    if (!change || typeof change.path !== 'string') fail('parent..tip diff entry is malformed');
    if (!expected.has(change.path) || seen.has(change.path)) {
      fail('parent..tip may change only .release-please-manifest.json and CHANGELOG.md');
    }
    if (change.status !== 'M' || change.oldMode !== '100644' || change.newMode !== '100644') {
      fail('release paths must be regular mode-100644 modifications');
    }
    if (change.oldType !== undefined && change.oldType !== 'blob') fail('release paths must remain blobs');
    if (change.newType !== undefined && change.newType !== 'blob') fail('release paths must remain blobs');
    seen.add(change.path);
  }
  if (seen.size !== expected.size) {
    fail('parent..tip must change both .release-please-manifest.json and CHANGELOG.md');
  }
}

/** Pure seam: ops supplies identity + parent..tip diff. */
export function classifyReleasePleaseBotTip(ops, tipSha) {
  if (!SHA.test(tipSha ?? '')) return null;
  if (!ops.hasCommit(tipSha)) return null;
  const parents = ops.parents(tipSha);
  if (!Array.isArray(parents) || parents.length !== 1 || !SHA.test(parents[0])) return null;
  const parentSha = parents[0];
  let identity;
  try { identity = ops.commitIdentity(tipSha); } catch { return null; }
  try { assertReleasePleaseBotIdentity(identity); } catch { return null; }
  let changes;
  try { changes = ops.diff(parentSha, tipSha); } catch { return null; }
  try { assertReleasePleaseBotPathDiff(changes); } catch { return null; }
  return Object.freeze({
    trainClass: RELEASE_PLEASE_TRAIN_CLASS,
    candidateSha: parentSha,
    authorityTipSha: tipSha,
    identity: Object.freeze({ ...identity }),
  });
}

/**
 * Admit a release-please bot tip as the authority tip of a structural M.
 * When `requirePrMeta` is true (bootstrap), event author/ref are mandatory.
 */
export function verifyReleasePleaseBotTrain(ops, { headSha, mergeSha, prAuthorLogin, prHeadRef, requirePrMeta = false }) {
  if (!SHA.test(headSha ?? '')) fail('PR head must be a lowercase 40-character SHA');
  if (!SHA.test(mergeSha ?? '')) fail('PR merge must be a lowercase 40-character SHA');
  const classified = classifyReleasePleaseBotTip(ops, headSha);
  if (!classified) fail('tip is not a release-please bot docs-only candidate');
  if (requirePrMeta) assertTrustedReleasePleasePrMeta({ prAuthorLogin, prHeadRef });
  if (!ops.hasCommit(mergeSha)) fail('PR merge object is unavailable');
  const mergeParents = ops.parents(mergeSha);
  if (!Array.isArray(mergeParents) || mergeParents.length !== 2 || mergeParents[1] !== headSha) {
    fail('M must be a two-parent merge whose second parent is the release tip');
  }
  if (ops.tree(mergeSha) !== ops.tree(headSha) || !ops.sameTreeDiff(mergeSha, headSha)) {
    fail('M tree/diff must equal the release tip exactly');
  }
  return Object.freeze({
    trainClass: RELEASE_PLEASE_TRAIN_CLASS,
    candidateSha: classified.candidateSha,
    integrationTipSha: headSha,
    authorityTipSha: headSha,
    mergeSha,
  });
}

/** Git-backed helpers for the authority-train / CI path (real repository). */
export function gitCommitIdentity(repoRoot, sha, git) {
  const raw = git(repoRoot, ['show', '-s', '--format=%an%x00%ae%x00%cn%x00%ce%x00%s', sha]);
  const [authorName, authorEmail, committerName, committerEmail, subject] = raw.replace(/\n$/, '').split('\0');
  return { authorName, authorEmail, committerName, committerEmail, subject };
}

export function gitRawDiff(repoRoot, from, to, git) {
  const fields = git(repoRoot, [...AUTHORITY_DIFF_ARGS, from, to]).split('\0');
  const changes = [];
  for (let index = 0; index < fields.length - 1;) {
    const header = fields[index++];
    if (!header) continue;
    const match = header.match(/^:(\d{6}) (\d{6}) [0-9a-f]{40} [0-9a-f]{40} ([A-Z])$/);
    if (!match) fail('Git diff contains an unsupported entry');
    const [, oldMode, newMode, status] = match;
    changes.push({
      path: fields[index++],
      status,
      oldMode,
      newMode,
      oldType: oldMode === '000000' ? null : 'blob',
      newType: newMode === '000000' ? null : 'blob',
    });
  }
  return changes;
}

export function gitOpsForReleasePlease(repoRoot, git, gitSucceeds) {
  return {
    hasCommit: (sha) => gitSucceeds(repoRoot, ['cat-file', '-e', `${sha}^{commit}`]),
    parents: (sha) => git(repoRoot, ['show', '-s', '--format=%P', sha]).trim().split(/\s+/).filter(Boolean),
    commitIdentity: (sha) => gitCommitIdentity(repoRoot, sha, git),
    diff: (from, to) => gitRawDiff(repoRoot, from, to, git),
    tree: (sha) => git(repoRoot, ['show', '-s', '--format=%T', sha]).trim(),
    sameTreeDiff: (left, right) => {
      try { git(repoRoot, ['diff', '--quiet', '--no-ext-diff', left, right]); return true; } catch { return false; }
    },
  };
}
