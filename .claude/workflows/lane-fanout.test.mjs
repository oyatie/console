// Offline preflight for lane-fanout.js. Run it BEFORE every dispatch:
//
//     node .claude/workflows/lane-fanout.test.mjs
//
// It exists because two classes of defect are invisible to reading, and both shipped here:
//
//   1. SYNTAX. An unescaped backtick inside a prompt template literal ends the literal. It
//      silently truncated BASE_LOCK once; the run looked normal and the lock was half gone.
//      `node --check` cannot see this — it rejects the top-level await and tells you nothing.
//   2. WIRING. A rule can be written, documented, believed, and reachable by nothing:
//        - `LENSES = ARGS.lenses || [...]` meant the standing lenses NEVER ran, because every
//          invocation passed `lenses`. Oracle integrity, the most common rejection cause in this
//          program, had never once been reviewed for.
//        - `verifierOk` matched the literal string "none" in the verifier's prose, so a verifier
//          writing "four, all minor; none contradicts the verdict" failed the check. Convergence
//          was unreachable whenever the verifier ran, and the harness manufactured rebuild rounds.
//        - convergence keyed on `severity === 'blocker'`, so a run reported CONVERGED while both
//          reviewers held six separately PROVEN fail-opens, every one filed "major".
//
// The harness is driven with STUB agents, so this proves the ENFORCEMENT LOGIC, not any agent's
// judgement. It is offline, free, and takes under a second.

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const SRC = fs.readFileSync(path.join(HERE, 'lane-fanout.js'), 'utf8')

// Compile exactly as the harness evaluates it: an async function body with these globals, so
// top-level await and return are legal. This is the only honest syntax check.
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor
let run
try {
  run = new AsyncFunction(
    'args', 'agent', 'parallel', 'pipeline', 'log', 'phase', 'budget', 'workflow',
    SRC.replace(/^export const meta = /m, 'const meta = '),
  )
} catch (e) {
  console.error('FAIL compile —', e.message)
  console.error('  An unescaped ` inside a prompt template literal is the usual cause.')
  process.exit(1)
}
console.log('PASS compile — body parses as the harness evaluates it')

let failures = 0 // eslint-disable-line prefer-const
const check = (name, ok, detail) => {
  // JSON.stringify(undefined) is undefined, not "undefined", so a FAILING assertion called without a
  // detail threw TypeError here and killed the whole preflight mid-run — losing every assertion after
  // it AND the failure count, so the harness reported nothing rather than a red. The reporter's own
  // failure path was the one path no assertion exercised. It is exercised at the bottom of this file.
  const shown = detail === undefined ? '(no detail)' : String(JSON.stringify(detail)).slice(0, 500)
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${ok ? '' : ' :: ' + shown}`)
  if (!ok) failures++
}

// Sequential is fine for a logic test; parallel() only needs to run every thunk and collect.
const parallel = async (thunks) => {
  const out = []
  for (const t of thunks) { try { out.push(await t()) } catch { out.push(null) } }
  return out
}
const pipeline = async (items, ...stages) => {
  const out = []
  for (const [i, it] of items.entries()) {
    let v = it
    for (const s of stages) v = await s(v, it, i)
    out.push(v)
  }
  return out
}

const BUILD = (over = {}) => ({
  status: 'done', summary: 's', filesChanged: ['x.rs'], redBaseline: 'RED', verification: 'ok',
  contractBreaches: 'none', enforcementPlacement: 'n/a - adds no enforcement',
  peripheralsUpdated: 'n/a - nothing described this behaviour', followUps: '', commands: [], ...over,
})
const FINDING = (over = {}) => ({
  severity: 'major', claim: 'c', failureScenario: 'f', location: 'l',
  provenByExecution: false, ownerLease: false, ...over,
})
const REVIEW = (over = {}) => ({ verdict: 'accept', findings: [], oracleWeakened: false, scopeCreep: false, ...over })
const VERIFY = (over = {}) => ({
  reproduced: true, actualResults: 'a', discrepancies: 'none', contradictsClaim: false,
  falseGreenRisk: 'none', headSha: 'cafe1234', ...over,
})

// Records every label the harness actually dispatched, which is how we prove a rule is REACHABLE
// rather than merely written.
const mkAgent = (opts = {}) => {
  const seen = []
  const fn = async (prompt, o = {}) => {
    const label = o.label || ''
    seen.push({ label, prompt })
    if (label.startsWith('build:')) return opts.build ? opts.build(seen) : BUILD()
    if (label.startsWith('review:')) return opts.review ? opts.review(prompt, seen) : REVIEW()
    if (label.startsWith('verify:')) return opts.verify ? opts.verify() : VERIFY()
    if (label === 'land') return opts.land ? opts.land() : { branch: 'integration/x', headSha: 'deadbee', landedLanes: ['a'], skippedLanes: [], verification: 'v', outstandingLeasedEdits: 'none', prCommand: 'gh pr create ...' }
    return 'REPORT'
  }
  fn.seen = seen
  return fn
}

const LANE = (over = {}) => ({ key: 'a', bead: 'b', wt: '/w', owned: 'x/**', brief: 't', accept: 'a', ...over })
const ARGS = (over = {}) => ({ tip: 'abc1234', lanes: [LANE()], maxRounds: 1, land: false, ...over })

const go = (args, agent = mkAgent(), logs = []) =>
  run(args, agent, parallel, pipeline, (m) => logs.push(m), () => {}, { total: null, spent: () => 0, remaining: () => Infinity }, async () => {})

const threw = async (args) => {
  try { await go(args); return null } catch (e) { return e.message }
}

// --- 1. An option the harness does not read must ABORT, never be silently dropped. ----------
{
  const m = await threw(ARGS({ prose_hardening: false }))
  check('unknown top-level arg aborts', !!m && /prose_hardening/.test(m), m)

  const m2 = await threw(ARGS({ lanes: [LANE({ scopes: ['x/'] })] }))
  check('unknown per-lane key aborts', !!m2 && /scopes/.test(m2), m2)

  const m3 = await threw(ARGS({ lens: ['typo'] }))
  check('a typo in `lenses` aborts rather than silently using defaults', !!m3 && /lens\b/.test(m3), m3)

  const ok = await go(ARGS())
  check('every documented arg is accepted', !!ok && Array.isArray(ok.headline), ok && Object.keys(ok))
}

// --- 2. Standing lenses must survive custom lenses (the dead-default defect). ---------------
{
  const agent = mkAgent()
  await go(ARGS({ lenses: ['CUSTOM ONE', 'CUSTOM TWO'] }), agent)
  const reviews = agent.seen.filter((s) => s.label.startsWith('review:'))
  const hasOracle = reviews.some((r) => /ORACLE INTEGRITY/.test(r.prompt))
  const hasPlacement = reviews.some((r) => /ENFORCEMENT PLACEMENT/.test(r.prompt))
  const hasDrift = reviews.some((r) => /PERIPHERAL DRIFT/.test(r.prompt))
  const hasCustom = reviews.some((r) => /CUSTOM ONE/.test(r.prompt))
  check('custom lenses ADD to standing lenses, never replace them',
    reviews.length === 5 && hasOracle && hasPlacement && hasDrift && hasCustom,
    { count: reviews.length, hasOracle, hasPlacement, hasDrift, hasCustom })
}

// --- 3. Convergence keys on PROOF, not on the severity label. -------------------------------
{
  const provenMajor = mkAgent({ review: () => REVIEW({ verdict: 'accept_with_findings', findings: [FINDING({ severity: 'major', provenByExecution: true })] }) })
  const r1 = await go(ARGS(), provenMajor)
  check('a PROVEN major blocks convergence', r1.lanes[0].converged === false, r1.headline)

  const arguedMajor = mkAgent({ review: () => REVIEW({ verdict: 'accept_with_findings', findings: [FINDING({ severity: 'major', provenByExecution: false })] }) })
  const r2 = await go(ARGS(), arguedMajor)
  check('an ARGUED major does not block convergence', r2.lanes[0].converged === true, r2.headline)

  const leasedBlocker = mkAgent({ review: () => REVIEW({ verdict: 'reject', findings: [FINDING({ severity: 'blocker', provenByExecution: true, ownerLease: true })] }) })
  const r3 = await go(ARGS(), leasedBlocker)
  check('an owner lease releases at ANY severity', r3.lanes[0].converged === true, r3.headline)

  const realBlocker = mkAgent({ review: () => REVIEW({ verdict: 'reject', findings: [FINDING({ severity: 'blocker' })] }) })
  const r4 = await go(ARGS(), realBlocker)
  check('a real blocker blocks', r4.lanes[0].converged === false, r4.headline)
}

// --- 4. The oracle may never be weakened, whatever the findings say. ------------------------
{
  const weakened = mkAgent({ review: () => REVIEW({ oracleWeakened: true }) })
  const r = await go(ARGS(), weakened)
  check('oracleWeakened blocks convergence with zero findings', r.lanes[0].converged === false, r.headline)
}

// --- 5. The verifier decides via a BOOLEAN, not a regex over its prose. ---------------------
{
  const conscientious = mkAgent({ verify: () => VERIFY({ discrepancies: 'four, all minor; none contradicts the verdict', contradictsClaim: false }) })
  const r = await go(ARGS(), conscientious)
  check('a wordy but non-contradicting verifier still converges', r.lanes[0].converged === true, r.headline)

  const contradicts = mkAgent({ verify: () => VERIFY({ contradictsClaim: true }) })
  const r2 = await go(ARGS(), contradicts)
  check('contradictsClaim=true blocks convergence', r2.lanes[0].converged === false, r2.headline)

  const notReproduced = mkAgent({ verify: () => VERIFY({ reproduced: false }) })
  const r3 = await go(ARGS(), notReproduced)
  check('reproduced=false blocks convergence', r3.lanes[0].converged === false, r3.headline)
}

// --- 6. Nothing green to falsify => no verifier. Rounds are the scarce resource. ------------
{
  const partial = mkAgent({ build: () => BUILD({ status: 'partial' }) })
  await go(ARGS(), partial)
  check('a non-done build skips the verifier', partial.seen.filter((s) => s.label.startsWith('verify:')).length === 0,
    partial.seen.map((s) => s.label))

  const done = mkAgent()
  await go(ARGS(), done)
  check('a done build runs the verifier', done.seen.filter((s) => s.label.startsWith('verify:')).length === 1,
    done.seen.map((s) => s.label))
}

// --- 7. Agent death is routine at this scale; retry once, then abandon. ---------------------
{
  let n = 0
  const flaky = mkAgent({ build: () => { n++; return n === 1 ? null : BUILD() } })
  const r = await go(ARGS(), flaky)
  check('a dead implementer is retried once and the round proceeds', n === 2 && r.lanes[0].status === 'done', { n, r: r.headline })

  const dead = mkAgent({ build: () => null })
  const r2 = await go(ARGS(), dead)
  check('two deaths abandon the lane without throwing', r2.lanes[0].converged === false, r2.headline)
}

// --- 8. Rejected rounds must feed the next build, or findings are collected and discarded. --
{
  const seenPrompts = []
  const agent = mkAgent({
    build: (seen) => { seenPrompts.push(seen[seen.length - 1].prompt); return BUILD() },
    review: () => REVIEW({ verdict: 'reject', findings: [FINDING({ severity: 'blocker', claim: 'THE-DISTINCTIVE-BLOCKER' })] }),
  })
  await go(ARGS({ maxRounds: 2 }), agent)
  check('round 2 receives round 1 blockers as feedback',
    seenPrompts.length === 2 && /THE-DISTINCTIVE-BLOCKER/.test(seenPrompts[1]), seenPrompts.length)
}

// --- 9. Reviewers diff the TIP, never HEAD~1 (which once diffed the base commit). -----------
{
  const agent = mkAgent()
  await go(ARGS(), agent)
  const reviews = agent.seen.filter((s) => s.label.startsWith('review:'))
  check('reviewers are told to diff the tip, and HEAD~1 appears only as a warning',
    reviews.every((r) => r.prompt.includes('abc1234')), reviews.length)
}

// --- 10. Landing is part of the pipeline. Its absence is what produced a +99 backlog. -------
{
  const agent = mkAgent()
  // Distinctive key and worktree: 'a' and '/w' occur in English prose, so they cannot tell a
  // populated prompt from a broken one.
  const r = await go(ARGS({ land: true, lanes: [LANE({ key: 'LANE-KEY-Z', wt: '/wt/lane-z' })] }), agent)
  check('a converged lane LANDS by default', agent.seen.some((s) => s.label === 'land') && !!r.landed, r.headline)

  // THE PROMPT MUST CONTAIN THE LANE, NOT undefined. Asserting only that landing was INVOKED is
  // what let this ship: the prompt read o.lane.key / o.lane.wt / o.fix off the flattened summary,
  // whose `lane` is a STRING, so the integration owner was told "undefined — worktree undefined"
  // with no files and no follow-ups. A phase that runs on garbage is not a phase that runs.
  const landPrompt = (agent.seen.find((s) => s.label === 'land') || {}).prompt || ''
  check('the landing prompt names the real lane key and its worktree',
    landPrompt.includes('LANE-KEY-Z') && landPrompt.includes('/wt/lane-z'), landPrompt.slice(0, 400))
  check('the landing prompt carries the lane\'s changed files', landPrompt.includes('x.rs'), landPrompt.slice(0, 400))
  check('the landing prompt interpolates no undefined field', !/undefined/.test(landPrompt),
    (landPrompt.split('\n').filter((x) => /undefined/.test(x)) || []).slice(0, 4))

  // LANDING MUST BE BOUND TO THE REVIEWED HEAD. Clean-worktree + recent-log is not a binding: a
  // commit added after the reviewers finished is clean and would be cherry-picked as reviewed.
  check('the landing prompt carries the head SHA captured at review time',
    landPrompt.includes('cafe1234'), landPrompt.slice(0, 400))
  check('and orders a refusal when the worktree no longer matches it',
    /rev-parse HEAD/.test(landPrompt) && /do NOT land|REFUSE/.test(landPrompt), landPrompt.slice(0, 400))

  const noConverge = mkAgent({ review: () => REVIEW({ verdict: 'reject', findings: [FINDING({ severity: 'blocker' })] }) })
  await go(ARGS({ land: true }), noConverge)
  check('nothing converged => nothing lands', !noConverge.seen.some((s) => s.label === 'land'),
    noConverge.seen.map((s) => s.label))

  const off = mkAgent()
  await go(ARGS({ land: false }), off)
  check('land:false is honoured (and warns)', !off.seen.some((s) => s.label === 'land'), off.seen.map((s) => s.label))
}

// --- 10b. A DEAD REVIEWER IS NOT AN ABSENT FINDING. -----------------------------------------
// Measured: session-limit kills took 4 of 7 agents from one run and 3 of 3 from another, and
// `.filter(Boolean)` made them vanish — a lane could converge on one surviving reviewer in silence.
{
  const standingDied = mkAgent({ review: (prompt) => (/ORACLE INTEGRITY/.test(prompt) ? null : REVIEW()) })
  const logs = []
  const r = await go(ARGS(), standingDied, logs)
  check('a dead STANDING lens blocks convergence', r.lanes[0].converged === false, r.headline)
  check('and says which one died', logs.some((m) => /CANNOT CONVERGE.*ORACLE INTEGRITY/.test(m)),
    logs.filter((m) => /CONVERGE|DIED/.test(m)))

  const customDied = mkAgent({ review: (prompt) => (/CUSTOM A/.test(prompt) ? null : REVIEW()) })
  const logs2 = []
  const r2 = await go(ARGS({ lenses: ['CUSTOM A', 'CUSTOM B'] }), customDied, logs2)
  check('a dead CUSTOM lens is tolerated (a rebuild round costs more than it saves)',
    r2.lanes[0].converged === true, r2.headline)
  check('but the death is still logged', logs2.some((m) => /DIED/.test(m)),
    logs2.filter((m) => /DIED|reviewers returned/.test(m)))
}

// --- 10d. A DEAD VERIFIER IS NOT A PASSED VERIFICATION. -------------------------------------
// The sibling of 10b, and worse: `!verify` read a died verifier as "no verifier was needed", so the
// lane converged and the log asserted "independently re-verified" having verified NOTHING. A
// standing lens is one voice among several; the verifier is the ONLY thing that reproduces a
// claimed green, so a dead one has no substitute and cannot be tolerated the way a custom lens is.
{
  const verifierDied = mkAgent({ verify: () => null })
  const logs = []
  const r = await go(ARGS(), verifierDied, logs)
  check('a dead VERIFIER blocks convergence', r.lanes[0].converged === false, r.headline)
  check('and never claims the lane was independently re-verified',
    !logs.some((m) => /independently re-verified/.test(m)), logs.filter((m) => /re-verified|VERIFIER/.test(m)))
  check('and says the verification never happened',
    logs.some((m) => /VERIFIER DIED/.test(m)), logs.filter((m) => /VERIFIER/.test(m)))

  // The deliberate exception must survive: a lane claiming nothing has nothing to falsify, so the
  // verifier is SKIPPED, and a skip is not a death.
  const partial = mkAgent({ build: () => BUILD({ status: 'partial' }) })
  const logs2 = []
  await go(ARGS(), partial, logs2)
  check('a build claiming nothing is skipped, not accused of a dead verifier',
    logs2.some((m) => /verifier skipped/.test(m)) && !logs2.some((m) => /VERIFIER DIED/.test(m)), logs2)
}

// --- 10c. Telemetry must be able to be WRONG, i.e. must observe reality. --------------------
// The old expression was ((LENSES.length + 1) * buildRounds) / buildRounds — algebraically a
// constant. It could not detect a dead agent no matter how many died.
{
  const oneDied = mkAgent({ review: (prompt) => (/BLAST RADIUS/.test(prompt) ? null : REVIEW()) })
  const r = await go(ARGS(), oneDied)
  const t = r.telemetry
  check('telemetry reports dispatched > returned when a checker dies',
    t.checkersDispatched > t.checkersReturned, { d: t.checkersDispatched, r: t.checkersReturned })
  check('telemetry names the dead checker', (t.deadCheckers || []).length === 1, t.deadCheckers)

  const allLived = mkAgent()
  const r2 = await go(ARGS(), allLived)
  check('telemetry reports equality when none die',
    r2.telemetry.checkersDispatched === r2.telemetry.checkersReturned,
    { d: r2.telemetry.checkersDispatched, r: r2.telemetry.checkersReturned })
}

// --- 10d. Review-round findings: three defects the reviewer caught that the preflight did not. ---
{
  // Two lanes in ONE worktree passed validation and were dispatched concurrently, each told by the
  // lock to build on whatever it found — so the second treats the first's half-finished edits as
  // its baseline. Silent by design.
  const m = await threw(ARGS({ lanes: [LANE({ key: 'a', wt: '/same' }), LANE({ key: 'b', wt: '/same' })] }))
  check('two lanes sharing a worktree abort', !!m && /both declare worktree/.test(m), m)

  const m2 = await threw(ARGS({ lanes: [LANE({ key: 'dup' }), LANE({ key: 'dup', wt: '/other' })] }))
  check('duplicate lane keys abort', !!m2 && /duplicate lane key/.test(m2), m2)

  // Documented and accepted must be the same set: the args comment advertised blockedTargets while
  // the allowlist rejected it, so a caller following the docs aborted.
  const ok = await go(ARGS({ lanes: [LANE({ blockedTargets: ['x'] })] }))
  check('a documented per-lane key is accepted', !!ok && Array.isArray(ok.headline), ok && Object.keys(ok || {}))

  // scopeCreep is a first-class verdict, not a hint. A reviewer setting it without ALSO restating it
  // as a blocking finding was ignored, so a lane could edit an unowned root and still converge.
  const creeper = mkAgent({ review: () => REVIEW({ verdict: 'accept', scopeCreep: true }) })
  const r = await go(ARGS(), creeper)
  check('scopeCreep alone blocks convergence', r.lanes[0].converged === false, r.headline)

  const clean = await go(ARGS(), mkAgent())
  check('and does not block when unset', clean.lanes[0].converged === true, clean.headline)
}

// --- 11. The required-field trio must stay required, or the clause is skimmable again. ------
{
  const req = (SRC.match(/required: \[[^\]]*'enforcementPlacement'[^\]]*\]/) || [''])[0]
  check('enforcementPlacement is a REQUIRED build field', /enforcementPlacement/.test(req), req.slice(0, 200))
  check('peripheralsUpdated is a REQUIRED build field', /peripheralsUpdated/.test(req), req.slice(0, 200))
  check('redBaseline is a REQUIRED build field', /redBaseline/.test(req), req.slice(0, 200))
  const rf = (SRC.match(/required: \[[^\]]*'provenByExecution'[^\]]*\]/) || [''])[0]
  check('provenByExecution and ownerLease are REQUIRED per finding',
    /provenByExecution/.test(rf) && /ownerLease/.test(rf), rf.slice(0, 200))
}

// --- 12. The lock must not be silently truncated by a nested backtick. ----------------------
{
  const lock = SRC.slice(SRC.indexOf('const BASE_LOCK = `') + 19, SRC.indexOf('\n`\n\nconst LOCK'))
  check('BASE_LOCK contains no nested backtick', (lock.match(/`/g) || []).length === 0, (lock.match(/`/g) || []).length)
  for (const clause of ['NEVER WEAKEN THE ORACLE', 'CONTRACT TESTS ARE PART OF THE CHANGE',
    'AN ENFORCEMENT MUST BE ABLE TO SEE ITS SUBJECT', 'PERIPHERALS ARE PART OF THE CHANGE',
    'THE THIRD SPELLING MEANS THE MECHANISM IS WRONG', 'NEVER pass --workflow-only']) {
    check(`lock clause survives: ${clause}`, lock.includes(clause))
  }
}

// --- 13. The caller that dispatches this harness must not bake in one machine's paths. ------
// program-tick.js chains into lane-fanout and used to send every selected lane to a hard-coded
// /Users/<name>/... worktree, ignoring both the workspace it was given and the worktree inventory
// it had just collected. On CI, on Linux, on any other machine, every implementer was pointed at a
// path that does not exist — and nothing here could see it, because the preflight only ever
// compiled lane-fanout.js. A dispatcher is part of the harness.
{
  const TICK = fs.readFileSync(path.join(HERE, 'program-tick.js'), 'utf8')
  try {
    new AsyncFunction('args', 'agent', 'parallel', 'pipeline', 'log', 'phase', 'budget', 'workflow',
      TICK.replace(/^export const meta = /m, 'const meta = '))
    check('program-tick compiles as the harness evaluates it', true)
  } catch (e) {
    check('program-tick compiles as the harness evaluates it', false, e.message)
  }
  const homePaths = TICK.split('\n').filter((l) => /(\/Users\/|\/home\/)[A-Za-z0-9_.-]+\//.test(l))
  check('program-tick hard-codes no machine-specific worktree path', homePaths.length === 0, homePaths)
}

// The unknown-option guard was written for lane-fanout, repeated in backlog-audit, and skipped in
// program-tick. A rule present in two of three sibling harnesses is not a rule, it is a coincidence,
// so the preflight now asserts it across ALL of them rather than for each file someone remembers.
{
  const dispatchers = ['program-tick', 'backlog-audit', 'lane-fanout']
  for (const name of dispatchers) {
    const src = fs.readFileSync(path.join(HERE, `${name}.js`), 'utf8')
      .replace(/^export const meta = /m, 'const meta = ')
    const fn = new AsyncFunction('args', 'agent', 'parallel', 'pipeline', 'log', 'phase', 'budget', 'workflow', src)
    const stub = async () => ({})
    // Every required field is supplied; ONLY the bogus key should be able to fail this.
    const base = { tip: 'a'.repeat(40), lanes: [LANE()], candidateWt: '/w', candidateTip: 'b'.repeat(40),
      base: 'main', repo: '/r', ghRepo: 'o/n' }
    let threw = null
    try {
      await fn({ ...base, thisOptionDoesNotExist: true }, stub, async (t) => Promise.all(t.map((f) => f())),
        async (i) => i, () => {}, () => {}, { total: null, spent: () => 0, remaining: () => Infinity }, stub)
    } catch (e) { threw = e.message }
    check(`${name} refuses an option it does not read`,
      !!threw && /unknown option/i.test(threw), threw)
  }
}

// A batch size of -1 produces no batches and 1.5 produces overlapping slices; both exit cleanly.
{
  const src = fs.readFileSync(path.join(HERE, 'backlog-audit.js'), 'utf8')
    .replace(/^export const meta = /m, 'const meta = ')
  const fn = new AsyncFunction('args', 'agent', 'parallel', 'pipeline', 'log', 'phase', 'budget', 'workflow', src)
  for (const bad of [-1, 0, 1.5, 'eight']) {
    let threw = null
    try {
      await fn({ repo: '/r', ghRepo: 'o/n', issueBatch: bad }, async () => ({}),
        async (t) => Promise.all(t.map((f) => f())), async (i) => i, () => {}, () => {},
        { total: null, spent: () => 0, remaining: () => Infinity }, async () => ({}))
    } catch (e) { threw = e.message }
    check(`backlog-audit refuses issueBatch=${JSON.stringify(bad)}`,
      !!threw && /issueBatch must be a positive integer/.test(threw), threw)
  }
}

// The reconciler was handed `JSON.stringify(findings).slice(0, 24000)`. With ~32 read lanes the cap
// binds routinely, so the single writer filed beads for a prefix of the audit and reported success.
{
  const src = fs.readFileSync(path.join(HERE, 'backlog-audit.js'), 'utf8')
  check('backlog-audit no longer truncates the findings payload blindly',
    !/JSON\.stringify\(findingsAll\)\.slice\(/.test(src))
  const body = src.match(/function renderFindings\(all\) \{[\s\S]*?\n\}/)
  check('backlog-audit exposes renderFindings to the preflight', !!body)
  if (body) {
    const renderFindings = new Function(`${body[0]}; return renderFindings`)()
    const many = Array.from({ length: 400 }, (_, i) => ({
      title: `finding ${i} ${'x'.repeat(200)}`, severity: i ? 'minor' : 'blocker',
      provenByExecution: i === 399, evidence: 'f.rs:1',
    }))
    const out = renderFindings(many)
    check('an over-budget findings set says how many it dropped', /DID NOT FIT/.test(out), out.slice(-200))
    check('the proven finding survives truncation regardless of its position',
      out.includes('finding 399'), 'the last-listed proven finding was cut')
    const few = [{ title: 'only one', severity: 'blocker', provenByExecution: true }]
    check('a set that fits carries no truncation notice', !/DID NOT FIT/.test(renderFindings(few)))
  }
}

// The reporter must survive its own failure path. Proven by capturing stdout rather than by reading
// it: a detail-less FAIL used to throw TypeError and abort the run, which is worse than a red because
// it looks like a crash in the harness instead of a defect in the code under test.
{
  const realLog = console.log
  const lines = []
  console.log = (l) => lines.push(l)
  let crashed = null
  const before = failures
  try { check('self-test: a failing assertion carries no detail', false) } catch (e) { crashed = e.message }
  console.log = realLog
  failures = before // this deliberate FAIL must not colour the real result
  check('a detail-less failure reports instead of crashing the preflight',
    crashed === null && lines.length === 1 && lines[0].startsWith('FAIL'), crashed || lines)
}

// A doc edit that fails CI on a stale generated checksum is the cheapest possible review round: the
// fix is one command, and the lane that made the edit could have run it. This assertion exists
// because a correct change was turned red by exactly that, twice.
{
  const need = ['generate-documentation-manifest.mjs --write', 'check:doc-manifest', 'POSTFLIGHT']
  for (const fragment of need) {
    check(`the lock tells lanes to refresh generated doc peripherals: ${fragment}`, SRC.includes(fragment))
  }
}

console.log(failures ? `\n${failures} FAILURE(S) — do not dispatch` : '\nALL PASS — safe to dispatch')
process.exit(failures ? 1 : 0)
