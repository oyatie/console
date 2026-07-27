import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { verifyConsoleAuthorityTrain } from './verify-console-authority-train.mjs';

test('authority train accepts only an exact structural synthetic merge M', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'console-train-'));
  const git = (args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim();
  try {
    git(['init']); git(['config', 'user.name', 'Jason Lee']); git(['config', 'user.email', 'jason19931225@gmail.com']);
    const key = path.join(root, 'key'); execFileSync('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', key]);
    git(['config', 'gpg.format', 'ssh']); git(['config', 'user.signingkey', key]);
    const publicKey = readFileSync(`${key}.pub`, 'utf8').trim().split(/\s+/).slice(0, 2).join(' ');
    const fingerprint = execFileSync('ssh-keygen', ['-lf', `${key}.pub`, '-E', 'sha256'], { encoding: 'utf8' }).trim().split(/\s+/)[1];
    const authority = { format: 'ssh', principal: 'jason19931225@gmail.com', fingerprint };
    mkdirSync(path.join(root, '.github/trust'), { recursive: true }); mkdirSync(path.join(root, 'docs/program'), { recursive: true });
    writeFileSync(path.join(root, '.github/trust/console.allowed_signers'), `jason19931225@gmail.com ${publicKey}\n`);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'C\n');
    git(['add', '.']); git(['commit', '-S', '-m', 'C']); const C = git(['rev-parse', 'HEAD']);
    for (const file of ['console-capability-registry.json', 'console-jurisdiction-register.json', 'console-program-ledger.md']) writeFileSync(path.join(root, 'docs/program', file), 'T\n');
    git(['add', '.']); git(['commit', '-S', '-m', 'T']); const T = git(['rev-parse', 'HEAD']); const base = C;
    const M = git(['commit-tree', `${T}^{tree}`, '-p', base, '-p', T, '-m', 'M']);
    assert.doesNotThrow(() => verifyConsoleAuthorityTrain(root, C, T, M, authority));
    assert.throws(() => verifyConsoleAuthorityTrain(root, C, T, T, authority), /two parents/);
    const wrongParent = git(['commit-tree', `${T}^{tree}`, '-p', T, '-p', base, '-m', 'wrong parent']);
    assert.throws(() => verifyConsoleAuthorityTrain(root, C, T, wrongParent, authority), /parent 2/);
    writeFileSync(path.join(root, 'drift.txt'), 'drift\n'); git(['add', '.']); const driftTree = git(['write-tree']); const treeDrift = git(['commit-tree', driftTree, '-p', base, '-p', T, '-m', 'tree drift']);
    assert.throws(() => verifyConsoleAuthorityTrain(root, C, T, treeDrift, authority), /tree and content diff/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
