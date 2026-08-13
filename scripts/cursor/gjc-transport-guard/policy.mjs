#!/usr/bin/env node
/**
 * Hub-aware GJC transport policy.
 *
 * One shared source: per-repo grammar is keyed by session cwd path prefix.
 * Used by the constrained plugin hook and by the acceptance probe runner.
 * Never imports plugin host APIs.
 */

import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

export const CONSOLE_HUB = '/Users/jasonlee/Developer/console'
export const OYATIE_HUB = '/Users/jasonlee/Developer/oyatie'

/** Packed source files that constitute the installable bundle identity. */
export const BUNDLE_SOURCE_FILES = Object.freeze([
  'gajae-plugin.json',
  'policy.mjs',
  'hooks/transport-guard.ts',
])

export const DEFAULT_HUBS = Object.freeze([
  Object.freeze({
    id: 'console',
    prefixes: Object.freeze([CONSOLE_HUB]),
    allowProvisionHelper: /(?:^|[\s;&|])(?:bash\s+)?(?:\S*\/)?scripts\/cursor\/provision-lane-worktree\.sh(?:\s|$)/,
    allowWorktreeAdd: null, // compiled per-hub below
    allowFetch: null,
    worktreeBranchRe: '(?:lane|admission)/[A-Za-z0-9._/-]+',
    worktreeStartPoint: 'origin/main',
    worktreeDirRe: '[A-Za-z0-9._-]+',
  }),
  Object.freeze({
    id: 'oyatie',
    prefixes: Object.freeze([OYATIE_HUB]),
    allowProvisionHelper: null,
    allowWorktreeAdd: null,
    allowFetch: /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+fetch\s+origin\s+dev(?:\s|$)/,
    worktreeBranchRe: 'agent/[A-Za-z0-9._/-]+',
    worktreeStartPoint: 'origin/dev',
    worktreeDirRe: 'lane-[A-Za-z0-9._-]+',
  }),
])

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

export function detectHub(cwd, hubs = DEFAULT_HUBS) {
  const n = normalizePath(cwd)
  if (!n) return null
  for (const hub of hubs) {
    for (const prefix of hub.prefixes) {
      const pref = normalizePath(prefix)
      if (n === pref || n.startsWith(`${pref}/`)) return hub
    }
  }
  return null
}

function flatten(text) {
  return String(text ?? '').replace(/[\r\n]+/g, ' ')
}

function wordHas(cmd, re) {
  return re.test(cmd)
}

function isGitTargetedKill(flat) {
  const hasPkill = /(?:^|[\s;&|])pkill(?:\s|$)/.test(flat)
  const hasKillall = /(?:^|[\s;&|])killall(?:\s|$)/.test(flat)
  const hasPgrep = /(?:^|[\s;&|])pgrep(?:\s|$)/.test(flat)
  const hasKill = /(?:^|[\s;&|])kill(?:\s|$)/.test(flat)
  const gitTarget =
    /(?:^|[\s\-/'"])git(?:[\s'"/]|$)|git-lock|hooks\/git|Cellar\/git|\/opt\/homebrew[^;&|]*bin\/git|admission-[A-Za-z0-9_-]+|\/\.worktrees\//i.test(
      flat,
    )
  if ((hasPkill || hasKillall) && gitTarget) return true
  if (/(?:^|[\s;&|])(?:pkill|killall)\s+(?:-[A-Za-z0-9]+\s+)*[`'"]?git[`'"]?(?:\s|$)/i.test(flat)) {
    return true
  }
  if (hasPgrep && hasKill && gitTarget) return true
  if (/(?:^|[\s;&|])pgrep\s+(?:-[A-Za-z0-9]+\s+)*-x\s+git(?:\s|$)/.test(flat)) return true
  return false
}

function isGitLockRm(flat) {
  return (
    /(?:^|[\s;&|])rm\s+[^;&|]*index\.lock/.test(flat) ||
    /(?:^|[\s;&|])rm\s+[^;&|]*HEAD\.lock/.test(flat) ||
    /(?:^|[\s;&|])rm\s+[^;&|]*gc\.pid/.test(flat) ||
    /(?:^|[\s;&|])rm\s+[^;&|]*\/\.git\/[^;&|]*\.lock/.test(flat)
  )
}

function isUpdateBranch(flat) {
  return /(?:^|[\s;&|])gh\s+pr\s+update-branch(?:\s|$)/.test(flat)
}

function isForeignTransport(flat) {
  return (
    /(?:^|[\s;&|/])mm-role(?:\s|$)/.test(flat) ||
    /(?:^|[\s;&|])claude\s+-p(?:\s|$)/.test(flat) ||
    /(?:^|[\s;&|])codex\s+exec(?:\s|$)/.test(flat)
  )
}

function isFastModelSlug(text) {
  return /(?:^|[\s=/,])[A-Za-z0-9._:+-]*-fast(?:\s|$|,)/.test(text) || /["'][^"']*-fast["']/.test(text)
}

function isForcePushWithoutLease(flat) {
  if (!/\bgit\b/.test(flat) || !/\bpush\b/.test(flat)) return false
  if (/--force-with-lease/.test(flat)) return false
  return /(?:^|[\s])--force(?:\s|$)/.test(flat) || /(?:^|[\s])-f(?:\s|$)/.test(flat)
}

function worktreeAddRe(hub) {
  const prefixes = hub.prefixes.map((p) => escapeRegExp(normalizePath(p)))
  const destAlt = prefixes.map((p) => `${p}/\\.worktrees/${hub.worktreeDirRe}`).join('|')
  const dest = `(?:\\.worktrees/${hub.worktreeDirRe}|${destAlt})`
  return new RegExp(
    `(?:^|[\\s;&|])git(?:\\s+-C\\s+\\S+)?\\s+worktree\\s+add\\s+${dest}\\s+-b\\s+${hub.worktreeBranchRe}\\s+${escapeRegExp(hub.worktreeStartPoint)}(?:\\s|$)`,
  )
}

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function isWorktreeAdd(flat) {
  return /(?:^|[\s;&|])git(?:\s+-C\s+\S+)?\s+worktree\s+add(?:\s|$)/.test(flat)
}

function siblingWorktreeDest(flat, hub) {
  if (!isWorktreeAdd(flat)) return false
  const m = flat.match(/worktree\s+add\s+(\S+)/)
  if (!m) return true
  const dest = normalizePath(m[1].replace(/^['"]|['"]$/g, ''))
  for (const prefix of hub.prefixes) {
    const pref = normalizePath(prefix)
    if (dest === `${pref}/.worktrees` || dest.startsWith(`${pref}/.worktrees/`)) return false
    if (dest === '.worktrees' || dest.startsWith('.worktrees/')) return false
  }
  return true
}

function extractCommand(input) {
  if (!input || typeof input !== 'object') return ''
  if (typeof input.command === 'string') return input.command
  if (typeof input.cmd === 'string') return input.cmd
  return ''
}

function extractTextBlobs(input) {
  if (!input || typeof input !== 'object') return []
  const blobs = []
  for (const key of ['command', 'cmd', 'content', 'new', 'old', 'input', 'path', 'text', 'payload']) {
    const v = input[key]
    if (typeof v === 'string' && v.length) blobs.push(v)
  }
  return blobs
}

/**
 * @param {{ toolName: string, input?: Record<string, unknown>, cwd?: string }} call
 * @param {{ hubs?: typeof DEFAULT_HUBS }} [opts]
 * @returns {{ decision: 'ALLOW' | 'DENY', reason: string, hub: string | null, classId: string | null }}
 */
export function evaluateToolCall(call, opts = {}) {
  examined += 1
  const hubs = opts.hubs ?? DEFAULT_HUBS
  const toolName = String(call?.toolName ?? '')
  const cwd = call?.cwd ?? ''
  const hub = detectHub(cwd, hubs)
  const input = call?.input && typeof call.input === 'object' ? call.input : {}
  const command = extractCommand(input)
  const flat = flatten(command)
  const blobs = [flat, ...extractTextBlobs(input).map(flatten)].filter(Boolean)
  const hay = blobs.join('\n')

  const deny = (classId, reason) => ({
    decision: 'DENY',
    reason,
    hub: hub?.id ?? null,
    classId,
    block: true,
  })
  const allow = (classId = 'allow') => ({
    decision: 'ALLOW',
    reason: '',
    hub: hub?.id ?? null,
    classId,
    block: false,
  })

  if (!['bash', 'edit', 'write'].includes(toolName)) {
    return allow('out-of-scope')
  }

  if (blobs.some((b) => isGitTargetedKill(b))) {
    return deny(
      'git-pkill',
      'DENIED git-pkill: pkill/killall/kill aimed at git (process.git-pkill-lock-race)',
    )
  }
  if (blobs.some((b) => isGitLockRm(b))) {
    return deny(
      'git-lock-rm',
      'DENIED git-lock-rm: do not rm index.lock/HEAD.lock/gc.pid/.git/**/*.lock',
    )
  }
  if (blobs.some((b) => isUpdateBranch(b))) {
    return deny('gh-update-branch', 'DENIED gh-update-branch: gh pr update-branch is forbidden')
  }
  if (blobs.some((b) => isForeignTransport(b))) {
    return deny(
      'foreign-transport',
      'DENIED foreign-transport: mm-role / claude -p / codex exec are forbidden',
    )
  }
  if (blobs.some((b) => isFastModelSlug(b))) {
    return deny('fast-model', 'DENIED fast-model: model slugs matching -fast are forbidden')
  }
  if (blobs.some((b) => isForcePushWithoutLease(b))) {
    return deny(
      'force-push',
      'DENIED force-push: git push --force requires --force-with-lease',
    )
  }

  if (toolName === 'bash' && isWorktreeAdd(flat)) {
    if (!hub) {
      return deny('worktree-shape', 'DENIED worktree-shape: cwd is not under a known hub')
    }
    if (hub.id === 'console') {
      if (hub.allowProvisionHelper && wordHas(flat, hub.allowProvisionHelper) && !isWorktreeAdd(flat)) {
        return allow('console-provision')
      }
      if (siblingWorktreeDest(flat, hub)) {
        return deny(
          'console-sibling-worktree',
          'DENIED console-sibling-worktree: worktree add must be <hub>/.worktrees/<name>',
        )
      }
      if (!worktreeAddRe(hub).test(flat)) {
        return deny(
          'console-worktree-shape',
          'DENIED console-worktree-shape: only git worktree add <hub>/.worktrees/<name> -b lane/<id>|admission/<id> origin/main (or provision-lane-worktree.sh)',
        )
      }
      return allow('console-worktree')
    } else if (hub.id === 'oyatie') {
      if (siblingWorktreeDest(flat, hub) || !worktreeAddRe(hub).test(flat)) {
        return deny(
          'oyatie-worktree-shape',
          'DENIED oyatie-worktree-shape: only git worktree add <hub>/.worktrees/lane-<id> -b agent/<id> origin/dev',
        )
      }
      return allow('oyatie-worktree')
    }
  }

  if (toolName === 'bash' && hub?.id === 'console' && hub.allowProvisionHelper && wordHas(flat, hub.allowProvisionHelper)) {
    return allow('console-provision')
  }
  if (toolName === 'bash' && hub?.id === 'oyatie' && hub.allowFetch && wordHas(flat, hub.allowFetch)) {
    // fetch is allowed; if the same command also worktree-adds, the worktree clause already ran
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

export const ACCEPTANCE_PROBES = Object.freeze([
  Object.freeze({
    id: 'H1-console-sibling-worktree',
    expect: 'DENY',
    classId: 'console-sibling-worktree',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: {
        command: `git -C ${CONSOLE_HUB} worktree add /Users/jasonlee/Developer/console-lane-hostile -b lane/hostile origin/main`,
      },
    },
  }),
  Object.freeze({
    id: 'H2-oyatie-unsanctioned-worktree',
    expect: 'DENY',
    classId: 'oyatie-worktree-shape',
    call: {
      toolName: 'bash',
      cwd: OYATIE_HUB,
      input: {
        command: `git worktree add ${OYATIE_HUB}/.worktrees/lane-hostile -b lane/hostile origin/main`,
      },
    },
  }),
  Object.freeze({
    id: 'H3-console-pkill-git',
    expect: 'DENY',
    classId: 'git-pkill',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: 'pkill -f git' },
    },
  }),
  Object.freeze({
    id: 'H4-oyatie-rm-index-lock',
    expect: 'DENY',
    classId: 'git-lock-rm',
    call: {
      toolName: 'bash',
      cwd: OYATIE_HUB,
      input: { command: `rm -f ${OYATIE_HUB}/.git/index.lock` },
    },
  }),
  Object.freeze({
    id: 'H5-console-gh-update-branch',
    expect: 'DENY',
    classId: 'gh-update-branch',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: { command: 'gh pr update-branch' },
    },
  }),
  Object.freeze({
    id: 'H6-oyatie-mm-role',
    expect: 'DENY',
    classId: 'foreign-transport',
    call: {
      toolName: 'bash',
      cwd: OYATIE_HUB,
      input: { command: 'mm-role critic --prompt "review this"' },
    },
  }),
  Object.freeze({
    id: 'S1-console-provision-helper',
    expect: 'ALLOW',
    classId: 'console-provision',
    call: {
      toolName: 'bash',
      cwd: CONSOLE_HUB,
      input: {
        command: 'bash scripts/cursor/provision-lane-worktree.sh --kind lane chore-cursor-ratchet-20260813',
      },
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
        command: `git fetch origin dev && git worktree add ${OYATIE_HUB}/.worktrees/lane-probe -b agent/probe origin/dev`,
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
    console.log(`${r.id}: expect=${r.expect} got=${r.got} class=${r.classId} ${mark}${r.reason ? ` ${r.reason}` : ''}`)
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
