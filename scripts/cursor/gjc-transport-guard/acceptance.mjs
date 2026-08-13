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
  extractDeclaredPaths,
  hashBundleFiles,
  resetExamined,
  runAcceptanceProbes,
  assertExaminedNonZero,
} from './policy.mjs'
import { main as runProbesMain } from './run-probes.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const cli = fileURLToPath(new URL('./run-probes.mjs', import.meta.url))
const G = 'g' + 'it'
const R = 're' + 'set'
const H = '--' + 'hard'
const W = 'work' + 'tree'
const V = 're' + 'move'
const C = 'clau' + 'de'
const P = '-' + 'p'
const WF = '--' + 'workflow' + '-' + 'only'
const GH = 'g' + 'h'
const PR = 'p' + 'r'
const MG = 'mer' + 'ge'
const SQ = '--' + 'squash'
const FETCH = 'fe' + 'tch'
const ADD = 'a' + 'dd'
const DEV = 'de' + 'v'
const ORIGIN = 'ori' + 'gin'
const AGENT = 'ag' + 'ent'

test('acceptance probes are exactly 8 (6 deny + 2 allow)', () => {
  assert.equal(ACCEPTANCE_PROBES.length, 8)
  assert.equal(ACCEPTANCE_PROBES.filter((p) => p.expect === 'DENY').length, 6)
  assert.equal(ACCEPTANCE_PROBES.filter((p) => p.expect === 'ALLOW').length, 2)
  assert.deepEqual(
    ACCEPTANCE_PROBES.map((p) => p.id),
    [
      'H1-git-reset-hard',
      'H2-git-worktree-remove',
      'H3-mm-role-cli',
      'H4-cargo-workflow-only',
      'H5-unsafe-push-forge',
      'H6-oyatie-hub-product-write',
      'S1-safe-read',
      'S2-oyatie-fetch-and-worktree',
    ],
  )
})

test('runner report is examined=8 and all probes match', () => {
  const report = runAcceptanceProbes()
  assert.equal(report.examined, 8)
  assert.equal(report.failed, 0)
  assert.equal(report.exitCode, 0)
  for (const row of report.results) assert.equal(row.ok, true, `${row.id} ${row.got} ${row.classId}`)
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
test('plan step-3 deny classes fire as GJC tool calls', () => {
  resetExamined()
  assert.equal(
    evaluateToolCall({
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${G} ${R} ${H} HEAD` },
    }).classId,
    'git-reset-hard',
  )
  assert.equal(
    evaluateToolCall({
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${G} ${W} ${V} .worktrees/lane-x` },
    }).classId,
    'git-worktree-remove',
  )
  assert.equal(
    evaluateToolCall({
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${C} ${P} do-it` },
    }).classId,
    'foreign-transport',
  )
  assert.equal(
    evaluateToolCall({
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `tools/ci/cargo_needs_postgres.sh ${WF}` },
    }).classId,
    'workflow-only',
  )
  assert.equal(
    evaluateToolCall({
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${GH} ${PR} ${MG} ${SQ}` },
    }).classId,
    'unsafe-push-forge',
  )
  assert.equal(
    evaluateToolCall({
      toolName: 'edit',
      cwd: OYATIE_HUB,
      input: { path: 'src/main.rs', old: 'a', new: 'b' },
    }).classId,
    'product-write',
  )
})

test('Phase A hub-root equality; lane cwd stays general-purpose', () => {
  resetExamined()
  const lane = `${CONSOLE_HUB}/.worktrees/lane-chore-cursor-ratchet-20260813`
  const laneWrite = evaluateToolCall({
    toolName: 'write',
    cwd: lane,
    input: { path: 'docs/program/console-development-pipeline.md', content: 'x' },
  })
  assert.equal(laneWrite.decision, 'ALLOW', laneWrite.reason)
  assert.equal(laneWrite.phaseA, false)
  const hubWrite = evaluateToolCall({
    toolName: 'write',
    cwd: CONSOLE_HUB,
    input: { path: 'docs/program/console-development-pipeline.md', content: 'x' },
  })
  assert.equal(hubWrite.decision, 'DENY')
  assert.equal(hubWrite.classId, 'product-write')
  const hubAst = evaluateToolCall({
    toolName: 'ast_edit',
    cwd: CONSOLE_HUB,
    input: { paths: ['backend/app/src/lib.rs'], ops: [{ pat: 'x', out: 'y' }] },
  })
  assert.equal(hubAst.decision, 'DENY')
  assert.equal(hubAst.classId, 'product-write')
})

test('unknown cwd fails closed for path tools; apply_patch paths are extracted', () => {
  resetExamined()
  const unknown = evaluateToolCall({
    toolName: 'write',
    cwd: '',
    input: { path: 'backend/src/lib.rs', content: 'x' },
  })
  assert.equal(unknown.decision, 'DENY')
  assert.equal(unknown.classId, 'unknown-cwd')
  const envelope = '*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-a\n+b\n*** End Patch\n'
  assert.deepEqual(extractDeclaredPaths({ input: envelope }), ['src/lib.rs'])
  const patched = evaluateToolCall({
    toolName: 'apply_patch',
    cwd: OYATIE_HUB,
    input: { input: envelope },
  })
  assert.equal(patched.decision, 'DENY')
  assert.equal(patched.classId, 'product-write')
})

test('safe read and sanctioned oyatie fetch+add remain allowed', () => {
  resetExamined()
  const read = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: `${G} status --porcelain` },
  })
  assert.equal(read.decision, 'ALLOW')
  assert.equal(read.classId, 'safe-read')
  const oyatie = evaluateToolCall({
    toolName: 'bash',
    cwd: OYATIE_HUB,
    input: {
      command: `${G} ${FETCH} ${ORIGIN} ${DEV} && ${G} ${W} ${ADD} ${OYATIE_HUB}/.worktrees/lane-demo -b ${AGENT}/demo ${ORIGIN}/${DEV}`,
    },
  })
  assert.equal(oyatie.decision, 'ALLOW', oyatie.reason)
  assert.equal(oyatie.classId, 'oyatie-worktree')
  const provision = evaluateToolCall({
    toolName: 'bash',
    cwd: CONSOLE_HUB,
    input: { command: 'bash scripts/cursor/provision-lane-worktree.sh --kind lane demo' },
  })
  assert.equal(provision.decision, 'ALLOW')
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
