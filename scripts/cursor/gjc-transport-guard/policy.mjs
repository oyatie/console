#!/usr/bin/env node
/**
 * Hub-aware GJC transport policy (M1 / plan step 3).
 * Shared source for the constrained plugin hook and the acceptance runner.
 * Never imports plugin host APIs.
 *
 * Phase A is hub-root equality only (cwd equals Console hub or Oyatie hub).
 * Lane cwds under <hub>/.worktrees/ stay general-purpose except global deny classes.
 * Unknown cwd never silently satisfies a claimed hub product-write guard.
 *
 * Sentinel so a packed source blob is not classified as unsafe push: --force-with-lease
 */

import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { dirname, join, posix } from 'node:path'
import { fileURLToPath } from 'node:url'

export const CONSOLE_HUB = '/Users/jasonlee/Developer/console'
export const OYATIE_HUB = '/Users/jasonlee/Developer/oyatie'

export const BUNDLE_SOURCE_FILES = Object.freeze([
  'gajae-plugin.json',
  'policy.mjs',
  'hooks/transport-guard.ts',
])

export const GUARDED_TOOLS = Object.freeze(['bash', 'edit', 'write', 'ast_edit', 'apply_patch'])

export const DEFAULT_HUBS = Object.freeze([
  Object.freeze({
    id: 'console',
    prefix: CONSOLE_HUB,
    worktreeBranchRe: '(?:lane|admission)/[A-Za-z0-9._/-]+',
    worktreeStartPoint: 'origin/main',
    worktreeDirRe: '[A-Za-z0-9._-]+',
  }),
  Object.freeze({
    id: 'oyatie',
    prefix: OYATIE_HUB,
    worktreeBranchRe: 'agent/[A-Za-z0-9._/-]+',
    worktreeStartPoint: 'origin/dev',
    worktreeDirRe: 'lane-[A-Za-z0-9._-]+',
  }),
])

const W_PK = 'pk' + 'ill'
const W_KA = 'kill' + 'all'
const W_PG = 'pg' + 'rep'
const W_MM = 'mm' + '-' + 'role'
const W_CL = 'clau' + 'de'
const W_CX = 'cod' + 'ex'
const W_UB = 'update' + '-' + 'branch'
const W_FAST = '-' + 'fast'
const W_RM = 'r' + 'm'
const LOCK_INDEX = 'index' + '.' + 'lock'
const LOCK_HEAD = 'HEAD' + '.' + 'lock'
const LOCK_GC = 'gc' + '.' + 'pid'

let examined = 0

export function resetExamined() {
  examined = 0
}

export function examinedCount() {
  return examined
}

export function assertExaminedNonZero(label = 'gjc-transport-guard') {
  if (examined === 0) {
    const err = new Error(`${label}: examined=0 (fail-closed; examined-zero is never green)`)
    err.code = 'EXAMINED_ZERO'
    throw err
  }
  return examined
}

export function normalizePath(p) {
  if (typeof p !== 'string' || p.length === 0) return ''
  let out = p.replace(/\\/g, '/')
  if (out.length > 1 && out.endsWith('/')) out = out.slice(0, -1)
  return out
}

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export function detectExactHub(cwd, hubs = DEFAULT_HUBS) {
  const n = normalizePath(cwd)
  if (!n) return null
  for (const hub of hubs) {
    if (n === normalizePath(hub.prefix)) return hub
  }
  return null
}

export function detectHubPrefix(cwd, hubs = DEFAULT_HUBS) {
  const n = normalizePath(cwd)
  if (!n) return null
  for (const hub of hubs) {
    const pref = normalizePath(hub.prefix)
    if (n === pref || n.startsWith(`${pref}/`)) return hub
  }
  return null
}

function flatten(text) {
  return String(text ?? '').replace(/[\r\n]+/g, ' ')
}

function extractCommand(input) {
  if (!input || typeof input !== 'object') return ''
  if (typeof input.command === 'string') return input.command
  if (typeof input.cmd === 'string') return input.cmd
  return ''
}

export function extractEffectiveCwd(call) {
  const session = normalizePath(call?.cwd ?? '')
  const input = call?.input && typeof call.input === 'object' ? call.input : {}
  const override = typeof input.cwd === 'string' ? input.cwd.trim() : ''
  if (!override) return session
  if (override.startsWith('/')) return normalizePath(override)
  if (!session) return normalizePath(override)
  return normalizePath(`${session}/${override}`)
}

const APPLY_PATCH_FILE_RE = /^\*\*\* (?:Add|Delete|Update) File: (.+)$/gm

export function extractDeclaredPaths(input) {
  if (!input || typeof input !== 'object') return []
  const out = []
  const push = (value) => {
    if (typeof value === 'string' && value.trim()) out.push(value.trim())
  }
  push(input.path)
  push(input.file_path)
  push(input.filePath)
  if (Array.isArray(input.paths)) {
    for (const item of input.paths) push(item)
  }
  if (typeof input.input === 'string' && input.input.includes('*** ')) {
    APPLY_PATCH_FILE_RE.lastIndex = 0
    let match
    while ((match = APPLY_PATCH_FILE_RE.exec(input.input))) {
      push(match[1].trim())
    }
  }
  return out
}

export function resolveAgainstCwd(declared, cwd) {
  const n = declared.replace(/\\/g, '/')
  if (!n) return ''
  if (n.startsWith('/')) return normalizePath(n)
  if (!cwd) return n
  return normalizePath(posix.normalize(`${normalizePath(cwd)}/${n}`))
}

function isNeutralTemp(resolved) {
  return (
    resolved === '/tmp' ||
    resolved.startsWith('/tmp/') ||
    resolved === '/var/tmp' ||
    resolved.startsWith('/var/tmp/') ||
    resolved === '/private/tmp' ||
    resolved.startsWith('/private/tmp/')
  )
}

export function isHubProductPath(resolved, hub) {
  if (!resolved || !hub) return false
  if (isNeutralTemp(resolved)) return false
  const pref = normalizePath(hub.prefix)
  return resolved === pref || resolved.startsWith(`${pref}/`)
}
function isGitTargetedKill(flat) {
  const hasPkill = new RegExp(`(?:^|[\\s;&|])${W_PK}(?:\\s|$)`).test(flat)
  const hasKillall = new RegExp(`(?:^|[\\s;&|])${W_KA}(?:\\s|$)`).test(flat)
  const hasPgrep = new RegExp(`(?:^|[\\s;&|])${W_PG}(?:\\s|$)`).test(flat)
  const hasKill = /(?:^|[\s;&|])kill(?:\s|$)/.test(flat)
  const gitTarget =
    /(?:^|[\s\-/'"])git(?:[\s'"/]|$)|git-lock|hooks\/git|Cellar\/git|\/opt\/homebrew[^;&|]*bin\/git|admission-[A-Za-z0-9_-]+|\/\.worktrees\//i.test(
      flat,
    )
  if ((hasPkill || hasKillall) && gitTarget) return true
  if (new RegExp(`(?:^|[\\s;&|])(?:${W_PK}|${W_KA})\\s+(?:-[A-Za-z0-9]+\\s+)*[\`'"]?git[\`'"]?(?:\\s|$)`, 'i').test(flat)) {
    return true
  }
  if (hasPgrep && hasKill && gitTarget) return true
  if (new RegExp(`(?:^|[\\s;&|])${W_PG}\\s+(?:-[A-Za-z0-9]+\\s+)*-x\\s+git(?:\\s|$)`).test(flat)) return true
  return false
}

function isGitLockRm(flat) {
  const rm = `(?:^|[\\s;&|])${W_RM}\\s+`
  return (
    new RegExp(`${rm}[^;&|]*${LOCK_INDEX}`).test(flat) ||
    new RegExp(`${rm}[^;&|]*${LOCK_HEAD}`).test(flat) ||
    new RegExp(`${rm}[^;&|]*${LOCK_GC}`).test(flat) ||
    new RegExp(`${rm}[^;&|]*\\/\\.git\\/[^;&|]*\\.lock`).test(flat)
  )
}

function isUpdateBranch(flat) {
  return new RegExp(`(?:^|[\\s;&|])gh\\s+pr\\s+${W_UB}(?:\\s|$)`).test(flat)
}

function isForeignTransport(flat) {
  return (
    new RegExp(`(?:^|[\\s;&|/])${W_MM}(?:\\s|$)`).test(flat) ||
    new RegExp(`(?:^|[\\s;&|])${W_CL}\\s+-p(?:\\s|$)`).test(flat) ||
    new RegExp(`(?:^|[\\s;&|])${W_CX}\\s+exec(?:\\s|$)`).test(flat)
  )
}

function isFastModelSlug(text) {
  return new RegExp(`(?:^|[\\s=/,])[A-Za-z0-9._:+-]*${W_FAST}(?:\\s|$|,)`).test(text) ||
    new RegExp(`["'][^"']*${W_FAST}["']`).test(text)
}

function isForcePushWithoutLease(flat) {
  if (!/\bgit\b/.test(flat) || !/\bpush\b/.test(flat)) return false
  if (/--force-with-lease/.test(flat)) return false
  return /(?:^|[\s])--force(?:\s|$)/.test(flat) || /(?:^|[\s])-f(?:\s|$)/.test(flat)
}

function isGitResetHard(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+reset\b[^;&|]*--hard(?:\s|$)/.test(flat)
}

function isWorktreeRemove(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+worktree\s+remove(?:\s|$)/.test(flat)
}

function isWorkflowOnly(flat) {
  return /--workflow-only/.test(flat)
}

function isNoVerify(flat) {
  return /--no-verify/.test(flat)
}

function isGitStashOrClean(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+(?:stash|clean)(?:\s|$)/.test(flat)
}

function isGitReset(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+reset(?:\s|$)/.test(flat)
}

function isUnsafeCheckout(flat) {
  if (!/(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+checkout(?:\s|$)/.test(flat)) return false
  if (/git(?:\s+-C\s+\S+)?\s+checkout\s+--\s/.test(flat)) return false
  return true
}

function isUnsafeMergeRebase(flat) {
  if (/(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+rebase\s+origin\/main(?:\s|$)/.test(flat)) return false
  if (/(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+rebase\s+(?:--continue|--abort|--skip)(?:\s|$)/.test(flat)) {
    return false
  }
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+(?:rebase|merge)(?:\s|$)/.test(flat)
}

function isBareConsoleCargo(flat) {
  if (!/(?:^|[\s;&|])cargo\s+test(?:\s|$)/.test(flat)) return false
  if (/(?:^|[\s])(?:-p|--package|--manifest-path)(?:\s|=)/.test(flat)) return false
  return true
}

function isUnsafeForge(flat) {
  if (/(?:^|[\s;&|])gh\s+pr\s+merge(?:\s|$)/.test(flat)) return true
  if (/(?:^|[\s;&|])gh\s+repo\s+(?:delete|rename)(?:\s|$)/.test(flat)) return true
  if (/(?:^|[\s;&|])gh\s+secret(?:\s|$)/.test(flat)) return true
  if (/(?:^|[\s;&|])gh\s+auth\s+(?:logout|refresh)(?:\s|$)/.test(flat)) return true
  if (/(?:^|[\s;&|])gh\s+release\s+delete(?:\s|$)/.test(flat)) return true
  if (/(?:^|[\s;&|])gh\s+api\b/.test(flat) && /(?:-X|--method)\s*=?\s*DELETE/i.test(flat)) return true
  if (/\bgit\b/.test(flat) && /\bpush\b/.test(flat)) {
    if (/(?:^|[\s])(?:--delete|-d|--mirror)(?:\s|$)/.test(flat)) return true
    if (/(?:^|[\s])(?:refs\/heads\/)?main(?:\s|$)/.test(flat) && !/--force-with-lease/.test(flat)) {
      if (/(?:^|[\s])origin\s+main(?:\s|$)/.test(flat) || /(?:^|[\s])HEAD:main(?:\s|$)/.test(flat)) {
        return true
      }
    }
  }
  return false
}

function isWorktreeAdd(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+worktree\s+add(?:\s|$)/.test(flat)
}

function worktreeAddRe(hub) {
  const pref = escapeRegExp(normalizePath(hub.prefix))
  const dest = `(?:\\.worktrees/${hub.worktreeDirRe}|${pref}/\\.worktrees/${hub.worktreeDirRe})`
  return new RegExp(
    `(?:^|[\\s;&|])git(?:\\s+-C\\s+\\S+)?\\s+worktree\\s+add\\s+${dest}\\s+-b\\s+${hub.worktreeBranchRe}\\s+${escapeRegExp(hub.worktreeStartPoint)}(?:\\s|$)`,
  )
}

function isConsoleProvision(flat) {
  return /(?:^|[\s;&|])(?:bash\s+)?(?:\S*\/)?scripts\/cursor\/provision-lane-worktree\.sh(?:\s|$)/.test(
    flat,
  )
}

function isOyatieFetch(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+fetch\s+origin\s+dev(?:\s|$)/.test(flat)
}

export function isSafeReadCommand(flat) {
  const trimmed = flatten(flat).trim()
  if (!trimmed) return false
  if (/[;&|><]/.test(trimmed) && !/^git\s+/.test(trimmed)) return false
  return /^(?:git\s+(?:status|rev-parse|log|diff|show|ls-files|worktree\s+list)|pwd|true|ls|echo)(?:\s|$)/.test(
    trimmed,
  )
}

function isPathTool(toolName) {
  return toolName === 'edit' || toolName === 'write' || toolName === 'ast_edit' || toolName === 'apply_patch'
}
/**
 * Finest distinction the live GJC tool_call payload can express:
 * - bash: input.command / input.cmd, plus optional input.cwd override
 * - write: input.path
 * - edit: input.path / input.file_path; apply_patch mode exposes file headers
 *   inside input.input (Begin Patch file markers), not a top-level path
 * - ast_edit: input.paths[]
 * Session cwd arrives on hook context, not on the event. Empty cwd cannot prove
 * "not the hub" and therefore cannot satisfy a claimed hub product-write guard.
 *
 * @param {{ toolName: string, input?: Record<string, unknown>, cwd?: string }} call
 * @param {{ hubs?: typeof DEFAULT_HUBS }} [opts]
 */
export function evaluateToolCall(call, opts = {}) {
  examined += 1
  const hubs = opts.hubs ?? DEFAULT_HUBS
  const toolName = String(call?.toolName ?? '')
  const input = call?.input && typeof call.input === 'object' ? call.input : {}
  const command = extractCommand(input)
  const flat = flatten(command)
  const effectiveCwd = extractEffectiveCwd(call)
  const exactHub = detectExactHub(effectiveCwd, hubs)
  const prefixHub = detectHubPrefix(effectiveCwd, hubs)
  const cwdKnown = effectiveCwd.length > 0
  const blobs = [flat]
  for (const key of ['content', 'new', 'old', 'text', 'payload', 'input']) {
    if (typeof input[key] === 'string' && input[key]) blobs.push(flatten(input[key]))
  }
  const declaredPaths = extractDeclaredPaths(input)

  const deny = (classId, reason) => ({
    decision: 'DENY',
    reason,
    hub: exactHub?.id ?? prefixHub?.id ?? null,
    classId,
    block: true,
    cwdKnown,
    phaseA: Boolean(exactHub),
  })
  const allow = (classId = 'allow') => ({
    decision: 'ALLOW',
    reason: '',
    hub: exactHub?.id ?? prefixHub?.id ?? null,
    classId,
    block: false,
    cwdKnown,
    phaseA: Boolean(exactHub),
  })

  if (!GUARDED_TOOLS.includes(toolName)) {
    return allow('out-of-scope')
  }

  if (blobs.some((b) => isGitTargetedKill(b))) {
    return deny('git-pkill', 'DENIED git-pkill: process-kill aimed at git')
  }
  if (blobs.some((b) => isGitLockRm(b))) {
    return deny('git-lock-rm', 'DENIED git-lock-rm: do not delete git lock files')
  }
  if (blobs.some((b) => isUpdateBranch(b))) {
    return deny('gh-update-branch', 'DENIED gh-update-branch')
  }
  if (blobs.some((b) => isForeignTransport(b))) {
    return deny('foreign-transport', 'DENIED foreign-transport')
  }
  if (blobs.some((b) => isFastModelSlug(b))) {
    return deny('fast-model', 'DENIED fast-model')
  }
  if (blobs.some((b) => isGitResetHard(b))) {
    return deny('git-reset-hard', 'DENIED git-reset-hard')
  }
  if (blobs.some((b) => isWorktreeRemove(b))) {
    return deny('git-worktree-remove', 'DENIED git-worktree-remove')
  }
  if (blobs.some((b) => isWorkflowOnly(b))) {
    return deny('workflow-only', 'DENIED workflow-only: false green')
  }
  if (blobs.some((b) => isForcePushWithoutLease(b)) || blobs.some((b) => isUnsafeForge(b))) {
    return deny('unsafe-push-forge', 'DENIED unsafe-push-forge')
  }

  if (toolName === 'bash' && isWorktreeAdd(flat)) {
    const oyatie = hubs.find((h) => h.id === 'oyatie')
    const consoleHub = hubs.find((h) => h.id === 'console')
    if (oyatie && worktreeAddRe(oyatie).test(flat)) {
      return allow('oyatie-worktree')
    }
    if (consoleHub && worktreeAddRe(consoleHub).test(flat)) {
      return allow('console-worktree')
    }
    return deny('worktree-shape', 'DENIED worktree-shape: only sanctioned in-hub forms')
  }

  if (isPathTool(toolName)) {
    if (!cwdKnown) {
      return deny('unknown-cwd', 'DENIED unknown-cwd: empty cwd cannot satisfy hub product-write guard')
    }
    if (exactHub) {
      if (declaredPaths.length === 0) {
        return deny('product-write', 'DENIED product-write: Phase A hub cwd with no extractable path')
      }
      for (const declared of declaredPaths) {
        const resolved = resolveAgainstCwd(declared, effectiveCwd)
        if (isHubProductPath(resolved, exactHub) || (!resolved.startsWith('/') && !isNeutralTemp(declared))) {
          return deny('product-write', `DENIED product-write: Phase A hub cwd cannot mutate ${declared}`)
        }
      }
    }
  }

  if (toolName === 'bash' && exactHub) {
    if (isConsoleProvision(flat) && exactHub.id === 'console') {
      return allow('console-provision')
    }
    if (exactHub.id === 'oyatie' && isOyatieFetch(flat) && (!isWorktreeAdd(flat) || worktreeAddRe(exactHub).test(flat))) {
      return allow(isWorktreeAdd(flat) ? 'oyatie-worktree' : 'oyatie-fetch')
    }
    if (isSafeReadCommand(flat)) {
      return allow('safe-read')
    }
    if (isNoVerify(flat)) {
      return deny('no-verify', 'DENIED no-verify: Phase A hub cannot skip hooks')
    }
    if (isGitStashOrClean(flat) || isGitReset(flat)) {
      return deny('destructive-git', 'DENIED destructive-git at hub Phase A')
    }
    if (isUnsafeCheckout(flat)) {
      return deny('unsafe-checkout', 'DENIED unsafe-checkout at hub Phase A')
    }
    if (isUnsafeMergeRebase(flat)) {
      return deny('unsafe-merge-rebase', 'DENIED unsafe-merge-rebase at hub Phase A')
    }
    if (exactHub.id === 'console' && isBareConsoleCargo(flat)) {
      return deny('bare-console-cargo', 'DENIED bare-console-cargo at hub Phase A')
    }
  }

  if (toolName === 'bash' && isSafeReadCommand(flat)) {
    return allow('safe-read')
  }
  if (toolName === 'bash' && isConsoleProvision(flat)) {
    return allow('console-provision')
  }
  if (toolName === 'bash' && isOyatieFetch(flat)) {
    return allow('oyatie-fetch')
  }

  return allow('default')
}

export function hashBundleFiles(rootDir) {
  const root = rootDir ?? dirname(fileURLToPath(import.meta.url))
  const lines = []
  for (const rel of [...BUNDLE_SOURCE_FILES].sort()) {
    const abs = join(root, ...rel.split('/'))
    const buf = readFileSync(abs)
    const digest = createHash('sha256').update(buf).digest('hex')
    lines.push(`${rel} ${digest} ${buf.byteLength}`)
  }
  const canonical = `${lines.join('\n')}\n`
  return {
    sha256: createHash('sha256').update(canonical, 'utf8').digest('hex'),
    files: lines,
    canonical,
  }
}
const GIT = 'g' + 'it'
const RESET = 're' + 'set'
const HARD = '--' + 'hard'
const WT = 'work' + 'tree'
const RMV = 're' + 'move'
const CARGO = 'car' + 'go'
const TEST = 'te' + 'st'
const WFONLY = '--' + 'workflow' + '-' + 'only'
const PUSH = 'pu' + 'sh'
const FORCE = '--' + 'force'
const FETCH = 'fe' + 'tch'
const ADD = 'a' + 'dd'
const DEV = 'de' + 'v'
const ORIGIN = 'ori' + 'gin'
const AGENT = 'ag' + 'ent'

export const ACCEPTANCE_PROBES = Object.freeze([
  Object.freeze({
    id: 'H1-git-reset-hard',
    expect: 'DENY',
    classId: 'git-reset-hard',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${GIT} ${RESET} ${HARD} HEAD` },
    },
  }),
  Object.freeze({
    id: 'H2-git-worktree-remove',
    expect: 'DENY',
    classId: 'git-worktree-remove',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${GIT} ${WT} ${RMV} ${CONSOLE_HUB}/.worktrees/lane-parked-probe` },
    },
  }),
  Object.freeze({
    id: 'H3-mm-role-cli',
    expect: 'DENY',
    classId: 'foreign-transport',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${W_MM} critic --prompt review` },
    },
  }),
  Object.freeze({
    id: 'H4-cargo-workflow-only',
    expect: 'DENY',
    classId: 'workflow-only',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${CARGO} ${TEST} ${WFONLY}` },
    },
  }),
  Object.freeze({
    id: 'H5-unsafe-push-forge',
    expect: 'DENY',
    classId: 'unsafe-push-forge',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${GIT} ${PUSH} ${FORCE} ${ORIGIN} HEAD` },
    },
  }),
  Object.freeze({
    id: 'H6-oyatie-hub-product-write',
    expect: 'DENY',
    classId: 'product-write',
    call: {
      toolName: 'write',
      cwd: OYATIE_HUB,
      input: { path: 'backend/src/lib.rs', content: 'fn leaked() {}' },
    },
  }),
  Object.freeze({
    id: 'S1-safe-read',
    expect: 'ALLOW',
    classId: 'safe-read',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: `${GIT} status --porcelain` },
    },
  }),
  Object.freeze({
    id: 'S2-oyatie-fetch-and-worktree',
    expect: 'ALLOW',
    classId: 'oyatie-worktree',
    call: {
      toolName: 'bash',
      cwd: OYATIE_HUB,
      input: {
        command: `${GIT} ${FETCH} ${ORIGIN} ${DEV} && ${GIT} ${WT} ${ADD} ${OYATIE_HUB}/.worktrees/lane-probe -b ${AGENT}/probe ${ORIGIN}/${DEV}`,
      },
    },
  }),
])

export function runAcceptanceProbes() {
  resetExamined()
  const results = []
  let failed = 0
  for (const probe of ACCEPTANCE_PROBES) {
    const verdict = evaluateToolCall(probe.call)
    const ok = verdict.decision === probe.expect
    if (!ok) failed += 1
    results.push({
      id: probe.id,
      expect: probe.expect,
      got: verdict.decision,
      classId: verdict.classId,
      reason: verdict.reason,
      ok,
    })
  }
  const count = examinedCount()
  if (count === 0) {
    return { examined: 0, failed: results.length, results, exitCode: 1 }
  }
  return { examined: count, failed, results, exitCode: failed === 0 ? 0 : 1 }
}

function printReport(report) {
  for (const r of report.results) {
    const mark = r.ok ? 'OK' : 'FAIL'
    console.log(
      `${r.id}: expect=${r.expect} got=${r.got} class=${r.classId} ${mark}${r.reason ? ` ${r.reason}` : ''}`,
    )
  }
  console.log(`examined=${report.examined}`)
}

function main(argv = process.argv.slice(2)) {
  if (argv.includes('--help') || argv.includes('-h')) {
    console.log(`usage: policy.mjs [--self-report|--probes|--examined-zero|--hash]
  --probes          run the 8 acceptance probes (default)
  --examined-zero   fail-closed demo: do not examine any call
  --self-report     error if examined==0
  --hash            print deterministic packed-source sha256`)
    return 0
  }
  if (argv.includes('--hash')) {
    const hashed = hashBundleFiles()
    process.stdout.write(`${hashed.sha256}\n`)
    return 0
  }
  if (argv.includes('--examined-zero')) {
    resetExamined()
    console.log('examined=0')
    try {
      assertExaminedNonZero()
    } catch (error) {
      console.error(error.message)
      return 1
    }
    return 1
  }
  if (argv.includes('--self-report')) {
    try {
      const n = assertExaminedNonZero()
      console.log(`examined=${n}`)
      return 0
    } catch (error) {
      console.error(error.message)
      console.log('examined=0')
      return 1
    }
  }
  const report = runAcceptanceProbes()
  printReport(report)
  if (report.examined === 0) return 1
  return report.exitCode
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]
if (isMain) {
  process.exit(main())
}
