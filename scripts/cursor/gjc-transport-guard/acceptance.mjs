import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import {
  ACCEPTANCE_PROBES,
  CONSOLE_HUB,
  OYATIE_HUB,
  evaluateToolCall,
  hashBundleFiles,
  resetExamined,
  runAcceptanceProbes,
  assertExaminedNonZero,
} from './policy.mjs'
import { main as runProbesMain } from './run-probes.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const cli = fileURLToPath(new URL('./run-probes.mjs', import.meta.url))

test('acceptance probes are exactly 8 (6 deny + 2 allow)', () => {
  assert.equal(ACCEPTANCE_PROBES.length, 8)
  assert.equal(ACCEPTANCE_PROBES.filter((p) => p.expect === 'DENY').length, 6)
  assert.equal(ACCEPTANCE_PROBES.filter((p) => p.expect === 'ALLOW').length, 2)
})

test('runner report is examined=8 and all probes match', () => {
  const report = runAcceptanceProbes()
  assert.equal(report.examined, 8)
  assert.equal(report.failed, 0)
  assert.equal(report.exitCode, 0)
  for (const row of report.results) assert.equal(row.ok, true, row.id)
})

test('examined-zero self-report exits nonzero', () => {
  resetExamined()
  assert.throws(() => assertExaminedNonZero(), /examined=0/)
  const child = spawnSync(process.execPath, [cli, '--examined-zero'], { encoding: 'utf8' })
  assert.notEqual(child.status, 0)
  assert.match(child.stdout + child.stderr, /examined=0/)
})

test('run-probes.mjs default prints examined=8 and exits 0', () => {
  const child = spawnSync(process.execPath, [cli], { encoding: 'utf8' })
  assert.equal(child.status, 0, child.stderr)
  assert.match(child.stdout, /examined=8/)
})

test('hostile deny classes fire across both hubs', () => {
  resetExamined()
  const sibling = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: `git worktree add /Users/jasonlee/Developer/console-lane-x -b lane/x origin/main` },
  })
  assert.equal(sibling.decision, 'DENY')
  const pkill = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: 'killall git' },
  })
  assert.equal(pkill.decision, 'DENY')
  const lockRm = evaluateToolCall({
    toolName: 'write',
    cwd: OYATIE_HUB,
    input: { command: 'rm .git/HEAD.lock' },
  })
  assert.equal(lockRm.decision, 'DENY')
  const update = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: 'gh pr update-branch' },
  })
  assert.equal(update.decision, 'DENY')
  const foreign = evaluateToolCall({
    toolName: 'edit',
    cwd: CONSOLE_HUB,
    input: { content: 'please run claude -p "do it"' },
  })
  assert.equal(foreign.decision, 'DENY')
  const fast = evaluateToolCall({
    toolName: 'write',
    cwd: OYATIE_HUB,
    input: { content: 'model: grok-4-fast' },
  })
  assert.equal(fast.decision, 'DENY')
  const force = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: 'git push --force origin HEAD' },
  })
  assert.equal(force.decision, 'DENY')
})

test('safe controls allow console provision helper and oyatie sanctioned fetch+worktree', () => {
  resetExamined()
  const provision = evaluateToolCall({
    toolName: 'bash',
    cwd: `${CONSOLE_HUB}/.worktrees/lane-chore-cursor-ratchet-20260813`,
    input: { command: 'bash scripts/cursor/provision-lane-worktree.sh --kind lane demo' },
  })
  assert.equal(provision.decision, 'ALLOW')
  const oyatie = evaluateToolCall({
    toolName: 'bash',
    cwd: OYATIE_HUB,
    input: {
      command: `git fetch origin dev && git -C ${OYATIE_HUB} worktree add ${OYATIE_HUB}/.worktrees/lane-demo -b agent/demo origin/dev`,
    },
  })
  assert.equal(oyatie.decision, 'ALLOW')
})

test('force-with-lease is allowed; bare --force is not', () => {
  resetExamined()
  const leased = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: 'git push --force-with-lease origin HEAD' },
  })
  assert.equal(leased.decision, 'ALLOW')
})

test('BUNDLE-HASH matches hash-bundle.mjs output', () => {
  const hashed = hashBundleFiles(here)
  const recorded = readFileSync(join(here, 'BUNDLE-HASH'), 'utf8').trim()
  assert.equal(recorded, hashed.sha256)
  assert.match(hashed.sha256, /^[0-9a-f]{64}$/)
})

test('runProbesMain --examined-zero returns 1', () => {
  assert.equal(runProbesMain(['--examined-zero']), 1)
})
