#!/usr/bin/env node
// Simulate the pull_request_target bootstrap gate EXACTLY as main runs it.
//
// The only step not reproduced is fetchExactPullObjects, which resolves
// refs/pull/N/merge from GitHub; the branch is not pushed yet. Everything that
// decides PASS/REFUSE -- signature verification against the pinned SSH policy,
// the C..T shape, and the M/T tree equality -- is the real main-branch code,
// imported from the main worktree rather than reimplemented here.
//
// usage: node simulate-main-gate.mjs <main-worktree> <T-sha> [base-sha]
//
// `base-sha` defaults to origin/main. Pass it to model the target branch AFTER
// an earlier PR in the stack has landed, which is what the expand/contract
// sequencing requires: the contract half is judged by the gate the expand half
// installed, not by the one on main today.

import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

const [mainWorktree, tipSha, baseArg] = process.argv.slice(2);
const gateUrl = new URL(`file://${path.join(mainWorktree, 'scripts/console/verify-console-pr-authority-bootstrap.mjs')}`);
const gate = await import(gateUrl.href);

const SAFE = ['PATH', 'SystemRoot', 'SYSTEMROOT', 'ComSpec', 'TEMP', 'TMP', 'TMPDIR', 'LANG', 'LC_ALL', 'TZ'];
function sanitizedGitEnvironment(source = process.env) {
  const environment = {};
  for (const key of SAFE) if (source[key] !== undefined) environment[key] = source[key];
  return { ...environment, HOME: '/dev/null', XDG_CONFIG_HOME: '/dev/null', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_SYSTEM: '/dev/null', GIT_TERMINAL_PROMPT: '0', GIT_ASKPASS: '/bin/false' };
}
const git = (repo, args) => execFileSync('git', ['-C', repo, '-c', 'core.hooksPath=/dev/null', ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], env: sanitizedGitEnvironment() });
const gitOk = (repo, args) => { try { git(repo, args); return true; } catch { return false; } };
function treeEntry(repo, sha, file) {
  const entry = git(repo, ['ls-tree', sha, '--', file]).trim().match(/^(\d{6}) (\w+) [0-9a-f]{40}\t/);
  return entry ? { mode: entry[1], type: entry[2] } : null;
}
function rawDiff(repo, from, to) {
  const fields = git(repo, ['diff', '--raw', '-z', '--abbrev=40', '--no-renames', '--no-ext-diff', from, to]).split('\0');
  const changes = [];
  for (let index = 0; index < fields.length - 1;) {
    const header = fields[index++]; if (!header) continue;
    const match = header.match(/^:(\d{6}) (\d{6}) [0-9a-f]{40} [0-9a-f]{40} ([A-Z])$/);
    if (!match) throw new Error('Git diff contains an unsupported entry');
    const [, oldMode, newMode, status] = match;
    changes.push({ path: fields[index++], status, oldMode, newMode, oldType: oldMode === '000000' ? null : 'blob', newType: newMode === '000000' ? null : 'blob' });
  }
  return changes;
}
const ops = (repo) => ({
  hasCommit: (sha) => gitOk(repo, ['cat-file', '-e', `${sha}^{commit}`]),
  readFile: (sha, file) => git(repo, ['show', `${sha}:${file}`]),
  treeEntry: (sha, file) => treeEntry(repo, sha, file),
  parents: (sha) => git(repo, ['show', '-s', '--format=%P', sha]).trim().split(/\s+/).filter(Boolean),
  diff: (from, to) => rawDiff(repo, from, to),
  tree: (sha) => git(repo, ['show', '-s', '--format=%T', sha]).trim(),
  sameTreeDiff: (left, right) => git(repo, ['diff', '--quiet', '--no-ext-diff', left, right]) === '',
  verifyCommit: (sha, authority) => gate.verifyPinnedSshCommit(repo, sha, authority.policy),
});

const T = git(mainWorktree, ['rev-parse', tipSha]).trim();
const base = git(mainWorktree, ['rev-parse', baseArg ?? 'origin/main']).trim();
// GitHub's synthetic merge: base as parent 1, the PR head as parent 2.
const M = execFileSync('git', ['-C', mainWorktree, 'commit-tree', `${T}^{tree}`, '-p', base, '-p', T, '-m', 'synthetic merge'], { encoding: 'utf8' }).trim();

console.log(`base(${baseArg ?? 'origin/main'}) = ${base}`);
console.log(`T                 = ${T}`);
console.log(`M (synthetic)     = ${M}\n`);

let graph;
try {
  graph = gate.verifyBootstrapGraph(ops(mainWorktree), { headSha: T, mergeSha: M });
  console.log('MAIN BOOTSTRAP GATE: PASS');
  console.log(JSON.stringify(graph, null, 2));
} catch (error) {
  console.log(`MAIN BOOTSTRAP GATE: REFUSED — ${error.message}`);
  process.exit(1);
}

// Main then runs the authenticated candidate checks in a worktree at C.
const plan = gate.candidateCheckPlan(graph.candidateSha, graph.integrationTipSha, graph.mergeSha);
const candidate = mkdtempSync(path.join(tmpdir(), 'console-candidate-'));
let failures = 0;
try {
  git(mainWorktree, ['worktree', 'add', '--detach', '--no-checkout', candidate]);
  git(candidate, ['checkout', '--detach', graph.candidateSha]);
  const environment = { ...sanitizedGitEnvironment(), ...plan.environment };
  for (const [binary, args] of plan.commands) {
    console.log(`\n--- ${binary} ${args.join(' ')}`);
    const status = spawnSync(binary, args, { cwd: candidate, env: environment, stdio: 'inherit' }).status;
    console.log(`--- exit ${status}`);
    if (status !== 0) failures += 1;
  }
} finally {
  try { git(mainWorktree, ['worktree', 'remove', '--force', candidate]); } catch { rmSync(candidate, { recursive: true, force: true }); }
}
console.log(`\nAUTHENTICATED CANDIDATE CHECKS: ${failures === 0 ? 'ALL PASS' : `${failures} FAILED`}`);
process.exit(failures === 0 ? 0 : 1);
