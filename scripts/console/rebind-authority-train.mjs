#!/usr/bin/env node
// Rebind the authority train after the candidate commit changes.
//
// The console authority train requires a two-commit branch:
//
//   C  the candidate — any change
//   T  the authority tip — C's direct single-parent child, modifying EXACTLY the
//      three authority documents, with the registers binding C's SHA
//
// Every time C is amended or rebased its SHA changes, and every reference to the
// old SHA inside the two registers must be rewritten or the truth-ledger gate
// fails with "CONSOLE_CANDIDATE_SHA must equal the authority-tip candidate SHA".
// There are ~390 such references, so doing this by hand is slow and error-prone.
// The program ledger attributes lost work across four consecutive releases to
// exactly this hand-rebuild step.
//
// Usage, from a branch whose HEAD is T and HEAD~1 is C:
//
//   node scripts/console/rebind-authority-train.mjs            # rebind only
//   node scripts/console/rebind-authority-train.mjs --amend-c  # fold staged changes into C first
//
// Refuses to run unless the branch already has the expected shape, and verifies
// the result before exiting. Never pushes.

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

const REGISTERS = [
  'docs/program/console-capability-registry.json',
  'docs/program/console-jurisdiction-register.json',
];
const LEDGER = 'docs/program/console-program-ledger.md';
const AUTHORITY_PATHS = [...REGISTERS, LEDGER];
const SHA = /^[0-9a-f]{40}$/;

const repo = process.cwd();
const git = (...args) => execFileSync('git', ['-C', repo, ...args], { encoding: 'utf8' }).trim();
const fail = (message) => {
  process.stderr.write(`rebind-authority-train: ${message}\n`);
  process.exit(1);
};

function assertShape() {
  const parents = git('rev-list', '--parents', '-n', '1', 'HEAD').split(/\s+/);
  if (parents.length !== 2) fail('HEAD must be T: a single-parent commit whose parent is C');
  const changed = git('diff', '--name-only', '--no-renames', 'HEAD~1', 'HEAD').split('\n').filter(Boolean);
  const unexpected = changed.filter((file) => !AUTHORITY_PATHS.includes(file));
  if (unexpected.length) fail(`C..T touches non-authority files: ${unexpected.join(', ')}`);
}

function currentBoundSha() {
  const registry = JSON.parse(readFileSync(REGISTERS[0], 'utf8'));
  const bound = registry?.candidate?.sha;
  if (!SHA.test(bound ?? '')) fail('capability registry has no valid candidate.sha to rebind from');
  return bound;
}

function rebind(oldSha, newSha) {
  let total = 0;
  for (const path of REGISTERS) {
    const before = readFileSync(path, 'utf8');
    const count = before.split(oldSha).length - 1;
    if (count) writeFileSync(path, before.split(oldSha).join(newSha));
    total += count;
    process.stdout.write(`  ${path}: ${count}\n`);
  }
  // source_revision is `<ref>@<sha>`; keep the ref, move the sha.
  const registryPath = REGISTERS[0];
  const registry = JSON.parse(readFileSync(registryPath, 'utf8'));
  if (typeof registry.source_revision === 'string' && registry.source_revision.includes('@')) {
    const ref = registry.source_revision.split('@')[0];
    registry.source_revision = `${ref}@${newSha}`;
    writeFileSync(registryPath, `${JSON.stringify(registry, null, 2)}\n`);
  }
  return total;
}

function main() {
  const amendC = process.argv.includes('--amend-c');
  assertShape();

  const tipMessage = git('log', '-1', '--format=%B', 'HEAD');
  const oldSha = currentBoundSha();

  // Unwind T, keeping its authority-document edits in the working tree.
  git('reset', '--soft', 'HEAD~1');
  execFileSync('git', ['-C', repo, 'restore', '--staged', ...AUTHORITY_PATHS], { stdio: 'ignore' });

  if (amendC) {
    git('commit', '--amend', '--no-edit');
  } else if (git('diff', '--cached', '--name-only') !== '') {
    fail('staged changes present but --amend-c was not passed; refusing to guess');
  }

  const newSha = git('rev-parse', 'HEAD');
  if (!SHA.test(newSha)) fail('could not resolve the candidate SHA');

  process.stdout.write(`rebinding ${oldSha.slice(0, 8)} -> ${newSha.slice(0, 8)}\n`);
  const total = oldSha === newSha ? 0 : rebind(oldSha, newSha);
  process.stdout.write(`  total references rebound: ${total}\n`);

  execFileSync('git', ['-C', repo, 'add', ...AUTHORITY_PATHS], { stdio: 'ignore' });
  execFileSync('git', ['-C', repo, 'commit', '-q', '-m', tipMessage], { stdio: 'inherit' });

  // Verify the result rather than trusting it.
  assertShape();
  const rebound = currentBoundSha();
  if (rebound !== newSha) fail(`post-rebind candidate.sha is ${rebound}, expected ${newSha}`);
  const stale = REGISTERS.filter((path) => readFileSync(path, 'utf8').includes(oldSha));
  if (oldSha !== newSha && stale.length) fail(`stale candidate SHA still present in: ${stale.join(', ')}`);

  process.stdout.write(`C = ${newSha}\nT = ${git('rev-parse', 'HEAD')}\n`);
  process.stdout.write('shape verified: T is C\'s single-parent child touching only the three authority documents\n');
}

main();
