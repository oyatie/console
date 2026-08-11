#!/usr/bin/env node
/**
 * Rewrite an open release-please tip so CHANGELOG custody converges.
 *
 * Release Please only edits `.release-please-manifest.json` + `CHANGELOG.md`.
 * `CHANGELOG.md` is first-party manifest evidence, so `check:doc-manifest` /
 * `check:doc-links` fail until the seed + index blob_sha rows are regenerated.
 * This script keeps a single bot tip (same subject/parent) whose parent..tip
 * diff is the release core paths plus the documentation custody pair — the
 * fail-closed set admitted by `release-please-bot-candidate.mjs`.
 *
 * Security / operability constraints:
 * - Executable code always comes from the protected checkout (this script's
 *   directory / Actions main checkout). The tip is opened only as a data
 *   worktree; its `scripts/**` are never executed.
 * - Custody bytes are exactly the protected generator's `--write` output.
 * - Rewrites require RELEASE_PLEASE_TOKEN (or CONSOLE_RELEASE_PUSH_TOKEN): a
 *   default GITHUB_TOKEN force-push does not schedule Required checks.
 *
 * Intended caller: `.github/workflows/release-please.yml` after the action, or
 * a workflow_dispatch heal. Heal also rewrites tips whose subject/parent/path
 * set are valid but author/committer identity is poisoned — still fail-closed
 * on path sprawl or wrong subject.
 */
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  RELEASE_PLEASE_BOT_EMAIL,
  RELEASE_PLEASE_BOT_NAME,
  RELEASE_PLEASE_COMMITTER_EMAIL,
  RELEASE_PLEASE_COMMITTER_NAME,
  RELEASE_PLEASE_CUSTODY_PATHS,
  RELEASE_PLEASE_HEAD_REF,
  RELEASE_PLEASE_PATHS,
  RELEASE_PLEASE_SUBJECT,
  gitCommitIdentity,
  gitRawDiff,
  releasePleaseCustodyRewritePlan,
} from './release-please-bot-candidate.mjs';

const PROTECTED_ROOT = process.cwd();
const PROTECTED_GENERATOR = join(
  dirname(fileURLToPath(import.meta.url)),
  'generate-documentation-manifest.mjs',
);
const ALLOWED = [...RELEASE_PLEASE_PATHS, ...RELEASE_PLEASE_CUSTODY_PATHS];

function run(cmd, args, opts = {}) {
  return execFileSync(cmd, args, {
    cwd: opts.cwd ?? PROTECTED_ROOT,
    encoding: 'utf8',
    stdio: opts.stdio ?? ['ignore', 'pipe', 'pipe'],
    env: opts.env ?? process.env,
  });
}

function ghJson(args) {
  const raw = run('gh', args);
  return raw.trim() ? JSON.parse(raw) : null;
}

function git(cwd, args) {
  return run('git', args, { cwd });
}

/** Discover by head-ref + release title; tip identity must still be the bot. */
function findOpenReleasePr() {
  const prs = ghJson([
    'pr', 'list',
    '--state', 'open',
    '--base', 'main',
    '--json', 'number,title,headRefName,headRefOid,author',
    '--limit', '50',
  ]) ?? [];
  const matches = prs.filter((pr) => (
    typeof pr.headRefName === 'string'
    && RELEASE_PLEASE_HEAD_REF.test(pr.headRefName)
    && RELEASE_PLEASE_SUBJECT.test(pr.title ?? '')
  ));
  if (matches.length === 0) return null;
  if (matches.length > 1) {
    throw new Error(`expected at most one open release-please PR, found ${matches.length}`);
  }
  return matches[0];
}

function pushToken() {
  return process.env.CONSOLE_RELEASE_PUSH_TOKEN
    || process.env.RELEASE_PLEASE_TOKEN
    || process.env.GH_TOKEN
    || process.env.GITHUB_TOKEN
    || '';
}

function assertPushTokenCanScheduleChecks(token) {
  const defaultActions = process.env.GITHUB_TOKEN || '';
  const usingDefaultOnly = Boolean(
    defaultActions
    && token === defaultActions
    && !process.env.RELEASE_PLEASE_TOKEN
    && !process.env.CONSOLE_RELEASE_PUSH_TOKEN,
  );
  if (usingDefaultOnly) {
    throw new Error(
      'refusing to rewrite release tip with default GITHUB_TOKEN: GitHub suppresses '
      + 'workflow triggers on that push, so Required checks would not schedule. '
      + 'Set RELEASE_PLEASE_TOKEN (or CONSOLE_RELEASE_PUSH_TOKEN) for the converge push.',
    );
  }
}

function isDirty(cwd, path) {
  try {
    git(cwd, ['diff', '--quiet', 'HEAD', '--', path]);
    return false;
  } catch {
    return true;
  }
}

export function main() {
  const token = pushToken();
  if (!token) throw new Error('RELEASE_PLEASE_TOKEN, CONSOLE_RELEASE_PUSH_TOKEN, GH_TOKEN, or GITHUB_TOKEN is required');

  const pr = findOpenReleasePr();
  if (!pr) {
    console.log('converge-release-please-doc-custody: no open release-please PR; nothing to do');
    return { status: 'noop' };
  }

  run('git', ['fetch', '--no-tags', 'origin', pr.headRefName]);
  const tip = run('git', ['rev-parse', `origin/${pr.headRefName}`]).trim();
  if (tip !== pr.headRefOid) {
    throw new Error(`origin/${pr.headRefName} tip ${tip} != PR head ${pr.headRefOid}`);
  }

  const gitAt = (root, args) => run('git', ['-C', root, ...args]);
  const identity = gitCommitIdentity(PROTECTED_ROOT, tip, gitAt);
  const parent = gitAt(PROTECTED_ROOT, ['rev-parse', `${tip}^`]).trim();
  const pathChanges = gitRawDiff(PROTECTED_ROOT, parent, tip, gitAt);

  const work = mkdtempSync(join(tmpdir(), 'console-rp-custody-'));
  try {
    // Data-only tip worktree: never execute tip scripts; generator path is protected.
    run('git', ['worktree', 'add', '--detach', work, tip]);

    run('node', [PROTECTED_GENERATOR, '--write'], { cwd: work });

    const dirty = ALLOWED.filter((path) => isDirty(work, path));
    const dirtyCustodyPaths = dirty.filter((path) => RELEASE_PLEASE_CUSTODY_PATHS.includes(path));
    // Fail closed if the generator dirtied a non-custody allowlisted path.
    for (const path of dirty) {
      if (!RELEASE_PLEASE_CUSTODY_PATHS.includes(path)) {
        throw new Error(`unexpected dirty path outside custody pair: ${path}`);
      }
    }

    // Structure (subject + path allow-list) first; identity may be poisoned and
    // still rewrite. Path sprawl / wrong subject throw inside the plan helper.
    const plan = releasePleaseCustodyRewritePlan({
      identity,
      pathChanges,
      dirtyCustodyPaths,
    });
    if (plan.action === 'noop') {
      console.log(`converge-release-please-doc-custody: PR #${pr.number} custody already converged`);
      return { status: 'converged', pr: pr.number, tip };
    }

    assertPushTokenCanScheduleChecks(token);

    const subject = identity.subject;
    if (!RELEASE_PLEASE_SUBJECT.test(subject)) {
      throw new Error(`tip subject is not a release subject: ${subject}`);
    }

    git(work, ['reset', '--soft', parent]);
    git(work, ['add', '--', ...ALLOWED]);
    const staged = git(work, ['diff', '--cached', '--name-only'])
      .split('\n').map((s) => s.trim()).filter(Boolean)
      .sort();
    const expected = [...ALLOWED].sort();
    if (JSON.stringify(staged) !== JSON.stringify(expected)) {
      throw new Error(`staged set ${JSON.stringify(staged)} != allowlisted ${JSON.stringify(expected)}`);
    }

    // Match release-please / GitHub web-flow identity exactly (author bot, committer GitHub).
    // gpgsign=false: bot tips are unsigned by design; identity headers are the gate.
    run('git', ['-c', 'commit.gpgsign=false', 'commit', '--no-verify', '-m', subject], {
      cwd: work,
      env: {
        ...process.env,
        GIT_AUTHOR_NAME: RELEASE_PLEASE_BOT_NAME,
        GIT_AUTHOR_EMAIL: RELEASE_PLEASE_BOT_EMAIL,
        GIT_COMMITTER_NAME: RELEASE_PLEASE_COMMITTER_NAME,
        GIT_COMMITTER_EMAIL: RELEASE_PLEASE_COMMITTER_EMAIL,
      },
    });
    const newTip = git(work, ['rev-parse', 'HEAD']).trim();
    const healed = gitCommitIdentity(work, newTip, (root, args) => git(root, args));
    // Post-rewrite identity must be exact bot/GitHub — never leave a poisoned tip.
    if (
      healed.authorName !== RELEASE_PLEASE_BOT_NAME
      || healed.authorEmail !== RELEASE_PLEASE_BOT_EMAIL
      || healed.committerName !== RELEASE_PLEASE_COMMITTER_NAME
      || healed.committerEmail !== RELEASE_PLEASE_COMMITTER_EMAIL
    ) {
      throw new Error('rewrite produced non-bot identity; refusing to push');
    }

    // Push with the scheduling-capable token (not the Actions default).
    const remote = `https://x-access-token:${token}@github.com/${run('gh', ['repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner']).trim()}.git`;
    git(work, [
      'push',
      '--force-with-lease=refs/heads/' + pr.headRefName + ':' + tip,
      remote,
      `HEAD:refs/heads/${pr.headRefName}`,
    ]);
    console.log(
      `converge-release-please-doc-custody: PR #${pr.number} tip ${tip} -> ${newTip}`
      + ` (rewritten reason=${plan.reason})`,
    );
    return { status: 'rewritten', pr: pr.number, tip, newTip, reason: plan.reason };
  } finally {
    try { run('git', ['worktree', 'remove', '--force', work]); } catch { /* best-effort */ }
    try { rmSync(work, { recursive: true, force: true }); } catch { /* best-effort */ }
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.stack || error.message : error);
    process.exit(1);
  }
}
