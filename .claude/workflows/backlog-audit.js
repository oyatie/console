export const meta = {
  name: 'backlog-audit',
  description: 'Audit the existing codebase by capability domain, triage every open GitHub issue against the current tree, and reconcile both into one backlog — findings become beads, dead issues get closed with evidence',
  whenToUse: 'Periodically, when the issue tracker and the working backlog have drifted apart, or before planning a phase. NOT for reviewing a PR — that is lane-fanout.',
  phases: [
    { title: 'Collect', detail: 'one agent gathers raw state; classification is done in-script' },
    { title: 'Read', detail: 'ONE wave: per-domain audit + cross-cutting sweeps + issue triage, all read-only' },
    { title: 'Reconcile', detail: 'single writer: create beads, close issues, sync the two' },
  ],
}

// ---------------------------------------------------------------------------
// args = {
//   repo:        "/abs/path"        // the checkout to audit
//   ghRepo:      "owner/name"       // for gh issue operations
//   ref?:        "branch-or-sha"    // what to audit; defaults to the checkout's HEAD
//   domains?:    [{ key, crates:[...], why }]   // capability chunks; derived if omitted
//   issueBatch?: 8                  // issues per triage agent; smaller = more lanes = same wall-clock
//   apply?:      false              // false = report only. TRUE = actually mutate beads and issues.
// }
//
// WHY THIS IS SEPARATE FROM lane-fanout: that harness reviews a DIFF against a tip, which is the
// right shape for work in flight and the wrong shape for standing code. Most defects in a mature
// tree are not in any recent diff — they are in the parts nobody has looked at since they landed.
// This one reads the tree as it stands, in chunks a reviewer can actually hold.
//
// THE DISCIPLINE THIS FILE EXISTS TO ENFORCE, learned the expensive way in this programme:
//   - A finding without file:line evidence is an opinion. It does not become a bead.
//   - An issue is NEVER closed on a guess. "Probably fixed" is not a verdict; the closing comment
//     must name the commit, the test, or the code that makes it moot.
//   - Wrongly closing a real issue is far worse than leaving a stale one open, because the stale one
//     is visible and the closed one is not. When uncertain, the verdict is KEEP with a note.
//   - One writer performs every mutation. Beads and gh are shared state; concurrent writers race.
// ---------------------------------------------------------------------------

let ARGS = args
if (typeof ARGS === 'string') {
  try { ARGS = JSON.parse(ARGS) } catch (e) {
    throw new Error(`backlog-audit: args arrived as a string that is not valid JSON: ${e.message}`)
  }
}
ARGS = ARGS || {}

// An option this workflow does not read must abort rather than be silently dropped — the same rule
// lane-fanout learned from a sibling runner where an ignored option cost six lanes.
const KNOWN_ARGS = ['repo', 'ghRepo', 'ref', 'domains', 'issueBatch', 'apply']
{
  const unknown = Object.keys(ARGS).filter((k) => !KNOWN_ARGS.includes(k))
  if (unknown.length) {
    throw new Error(
      `backlog-audit: unknown option(s) ${unknown.join(', ')}. Known: ${KNOWN_ARGS.join(', ')}.`,
    )
  }
}

const REPO = ARGS.repo
const GH = ARGS.ghRepo
const REF = ARGS.ref || 'HEAD'
// `|| 8` rescues 0 and undefined and NOTHING else: -1 yields negative-stride batching that produces
// no batches at all, and 1.5 yields overlapping slices that triage the same issue twice under two
// agents. Both exit cleanly, which is what makes them worth refusing here.
const BATCH = ARGS.issueBatch === undefined ? 8 : ARGS.issueBatch
if (!Number.isInteger(BATCH) || BATCH < 1) {
  throw new Error(`backlog-audit: issueBatch must be a positive integer; got ${JSON.stringify(ARGS.issueBatch)}`)
}
const APPLY = ARGS.apply === true

if (!REPO) throw new Error('backlog-audit: args.repo is required')
if (!GH) throw new Error('backlog-audit: args.ghRepo is required (owner/name)')

const SAFETY = [
  '=== READ-ONLY UNLESS TOLD OTHERWISE ===',
  APPLY
    ? 'apply=TRUE. The Reconcile phase — and ONLY that phase — may mutate beads and GitHub issues.'
    : 'apply=FALSE. NOTHING may be mutated. No bd create/close/update, no gh issue close/comment.',
  'No phase may edit source, commit, push, or open a PR. This workflow reports and files; it does',
  'not fix. A finding that is trivially fixable is still a finding — fixing it here would put an',
  'unreviewed change into a tree nobody is watching.',
  '',
  '=== EVIDENCE RULES, non-negotiable ===',
  '1. Every finding carries file:line and the command that produced it. A claim you did not run is',
  '   an opinion, and opinions do not become backlog items.',
  '2. State what you SEARCHED for as well as what you found. A null result from the wrong query has',
  '   twice become a written "not established" finding in this programme.',
  '3. Beware the greps that silently match nothing here: `git grep -E` does NOT support \\b (POSIX',
  '   ERE), and a trailing \\b makes the whole pattern match zero lines while exiting cleanly. That',
  '   produced a false-clean writer census for a table with five known writers. Validate any regex',
  '   against a case you KNOW matches before trusting a zero.',
  '4. Stale worktrees are not the tree. This checkout contains abandoned worktrees under .worktrees/',
  '   and .omx/team/**; five separate diagnostic bursts in this programme traced to them. Scope every',
  '   search to the real source roots and say which roots you used.',
].join('\n')

const FINDING_SCHEMA = {
  type: 'object',
  required: ['domain', 'findings', 'coverage'],
  properties: {
    domain: { type: 'string' },
    coverage: { type: 'string', description: 'what you actually read, and what you did NOT get to — an honest gap beats a claimed sweep' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        required: ['title', 'severity', 'evidence', 'failureScenario', 'provenByExecution'],
        properties: {
          title: { type: 'string', description: 'imperative and specific, as a backlog item should read' },
          severity: { type: 'string', enum: ['blocker', 'major', 'minor', 'nit'] },
          evidence: { type: 'string', description: 'file:line plus the command and its real output' },
          failureScenario: { type: 'string', description: 'concrete inputs or state -> wrong outcome. Not "could be unsafe".' },
          provenByExecution: { type: 'boolean', description: 'TRUE only if you RAN something and observed it. Reasoning from source is FALSE.' },
          suggestedFix: { type: 'string' },
          existingIssue: { type: 'string', description: 'an existing GitHub issue or bead this duplicates, if any — check BEFORE filing' },
        },
      },
    },
    strengths: { type: 'string', description: 'controls that are genuinely load-bearing, so a later reader does not "simplify" them away' },
  },
}

const TRIAGE_SCHEMA = {
  type: 'object',
  required: ['verdicts'],
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object',
        required: ['number', 'verdict', 'evidence'],
        properties: {
          number: { type: 'number' },
          title: { type: 'string' },
          verdict: {
            type: 'string',
            enum: ['KEEP', 'CLOSE-FIXED', 'CLOSE-OBSOLETE', 'CLOSE-DUPLICATE', 'NEEDS-OWNER'],
            description: 'KEEP when uncertain. A wrongly closed issue is invisible; a stale open one is not.',
          },
          evidence: { type: 'string', description: 'for any CLOSE: the commit, test or code that makes it moot, with file:line. "Looks done" is not evidence.' },
          duplicateOf: { type: 'number' },
          beadCandidate: { type: 'boolean', description: 'KEEP items that are real work and should exist in beads too' },
          staleness: { type: 'string', description: 'what in the issue text is now factually wrong about the tree' },
        },
      },
    },
  },
}

// --- Collect ---------------------------------------------------------------
// One agent gathers raw facts; everything that can be decided by a rule is decided in-script.
// Measured in this programme: a per-item agent loop over 89 worktrees took many minutes and 233KB
// of transcript, while one batched shell command did the same work in 5.2 seconds. Agents are for
// judgement, not for enumeration.
phase('Collect')
const collected = await agent(
  `Gather the raw state of the backlog and the tree. Do NOT judge anything yet; this phase is a census.

REPO: ${REPO}   GH: ${GH}   REF: ${REF}

${SAFETY}

Emit, in ONE batched pass each (not a loop of small commands):
 1. Every OPEN GitHub issue: number, title, labels, author, createdAt, updatedAt, comment count, and
    the first 400 characters of the body. Use a single \`gh issue list --json\` call with --limit high
    enough to get them all, and say how many you got.
 2. Every bead: id, title, status, priority, and its dependency edges. \`bd list\` and \`bd dep\`.
 3. The crate inventory under backend/crates, with each crate's line count, so domains can be sized.
 4. Recently merged PRs (last 40) with number, title and merge commit — a closed issue often has its
    fix sitting in one of these, and that is the cheapest evidence of CLOSE-FIXED there is.
 5. The abandoned-worktree roots that must be EXCLUDED from every later search, listed explicitly.

Return raw structured data. No verdicts, no recommendations — later phases do that, and a census that
editorialises makes its own errors invisible.`,
  // A SCHEMA, not free text. The first version of this file had none, so `collected` came back as a
  // plain string and the regex below that scraped issue numbers out of it matched NOTHING. Ten
  // triage lanes were therefore never dispatched, and the run reported "0 issues triaged" as though
  // that were an answer. The census must hand the script DATA, not prose it has to parse.
  {
    label: 'collect',
    phase: 'Collect',
    schema: {
      type: 'object',
      required: ['openIssueNumbers', 'openIssueCount', 'issues', 'beads'],
      properties: {
        openIssueNumbers: { type: 'array', items: { type: 'number' }, description: 'EVERY open issue number. This drives the triage fan-out, so an omission here silently un-audits that issue.' },
        openIssueCount: { type: 'number', description: 'what gh reported, so the script can catch a truncated list' },
        issues: { type: 'array', items: { type: 'object' }, description: 'number, title, labels, author, createdAt, updatedAt, comments, body excerpt' },
        beads: { type: 'array', items: { type: 'object' } },
        crates: { type: 'array', items: { type: 'object' } },
        mergedPrs: { type: 'array', items: { type: 'object' } },
        excludedRoots: { type: 'array', items: { type: 'string' } },
      },
    },
  },
)

// --- The read fan-out ------------------------------------------------------
// EVERY PHASE BELOW IS READ-ONLY, so width is nearly free and there is no collision to fear.
// Measured in this programme: wall-clock tracks per-step latency x DEPTH and is almost insensitive
// to WIDTH — 12 agents at depth 6 took 74 min, 36 agents at depth 6 took 63 min. So the audit,
// the cross-cutting sweeps and the issue triage all run in ONE parallel block rather than in
// sequence: three independent read phases stacked serially would triple the wall-clock and buy
// nothing, because none of them feeds another. Only Reconcile, which WRITES, is serialised.
//
// Domains are narrow on purpose. A reviewer holding one bounded capability finds the interaction
// defects that live in the seam, and a smaller root means fewer files skimmed rather than read.
const DOMAINS = ARGS.domains || [
  { key: 'identity', crates: ['identity'], why: 'principal resolution and the role/feature matrix — a fail-open here is silent and total' },
  { key: 'policy-authz', crates: ['policy'], why: 'Cedar is OBSERVE-ONLY and inert in production here; find what actually enforces' },
  { key: 'governance', crates: ['governance'], why: 'four-eyes, SoD, maker-checker, effective-dating — the controls that must not be bypassable' },
  { key: 'leave', crates: ['leave'], why: '§4-31: 연차 must have NO reason field and cannot be refused. The schema currently has reason NOT NULL — verify and size it' },
  { key: 'attendance', crates: ['attendance'], why: 'the 주52 cap and break rules are statutory, not policy' },
  { key: 'orgchange-eval', crates: ['orgchange', 'evaluation'], why: 'org lifecycle and the appraisal surface labour law constrains' },
  { key: 'payroll', crates: ['payroll'], why: 'money is irreversible; period locks, rounding and the minimum-wage instrument' },
  { key: 'finance', crates: ['finance-gl', 'financial', 'benefit'], why: 'ledger integrity and benefit entitlement' },
  { key: 'ontology', crates: ['ontology'], why: 'the substrate every other domain projects through' },
  { key: 'kernel-registry', crates: ['kernel', 'registry'], why: 'shared types and the object registry everything else trusts' },
  { key: 'workflow', crates: ['workflow'], why: 'schedules, drains and outboxes — where at-least-once quietly becomes at-least-twice' },
  { key: 'comms-egress', crates: ['comms', 'messenger', 'notices'], why: 'egress: the DLP and send-gate boundary, where a leak is external and permanent' },
  { key: 'docs-inbox', crates: ['docs', 'inbox', 'notifications'], why: 'document custody and the personal surfaces §4-37 separates' },
  { key: 'ops-field', crates: ['dispatch', 'facilities', 'equipment', 'inspection', 'logistics', 'workorder', 'production'], why: 'the 70% of staff who work on a client site' },
  { key: 'platform', crates: ['platform'], why: 'db, authz, request-context, audit-chain — every tenant boundary in one crate tree' },
]

// Cross-cutting sweeps read ACROSS the tree rather than down one crate. They exist because the
// worst defects in this programme were never inside one crate: a census that ran before migrations,
// a gate wired to nothing, a doc claiming a property the code had lost. No per-crate reviewer could
// have seen any of them, because each is a property of the SEAM between things.
const CROSS = [
  { key: 'x-migrations', why: 'all migrations as a sequence: RLS declared but not FORCED, a missing append-only trigger, a destructive DDL, contiguity, and any table whose properties diverge from the 0177/0213/0214 references' },
  { key: 'x-gates', why: 'every backend/ci/gates/** binary: is each REACHED by a CI step, and would it FAIL on a real violation? A gate wired to nothing is worse than no gate, because it reads as coverage' },
  { key: 'x-tenancy', why: 'the org boundary end to end: RLS FORCE, app.current_org arming, the CURRENT_ORG task-local (a bare tokio::spawn does not inherit it and the failure is ZERO ROWS, not an error), and any aggregate that merges sources and widens visibility' },
  { key: 'x-openapi', why: 'the 36k-line hand-maintained openapi.yaml against the handlers it claims to describe — a published contract the code stopped honouring is a live interop break, and two were just found' },
  { key: 'x-docs-drift', why: 'docs/** and module docs against the code: claims of the form "every / always / cannot / the N ways X can happen". Pick the load-bearing ones and check whether the code still holds them' },
  { key: 'x-compliance', why: '§4-31 labour law and §3.10 internal controls as properties of the whole tree: statutory periods and rates must be catalogue-derived with the instrument cited, never hardcoded; no reason field on 연차; no destructive delete where 보관=숨김 is required' },
  { key: 'x-supply-chain', why: 'dependency posture: unmaintained or advisory-bearing crates, git sources, licence outliers, and anything a cargo-deny ignore is silently carrying' },
]

// Issue batches are computed BEFORE the read block so triage can join the same parallel wave.
// Read the field the schema guarantees. The previous version scraped `"number": N` out of the
// stringified census with a regex; the census was prose, so it matched nothing and ten triage lanes
// were silently never dispatched.
const issueNumbers = [...new Set((collected && collected.openIssueNumbers) || [])]
  .map(Number).filter((n) => Number.isFinite(n) && n > 0).sort((a, b) => a - b)
const claimedCount = (collected && collected.openIssueCount) || 0

// A PHASE THAT EXAMINES ZERO SUBJECTS MUST FAIL, NEVER PASS. This is the harness's own standing rule
// — "examined zero subjects MUST be a FAILURE" — applied to the harness itself, because the first run
// of this file broke exactly that way: 22 audit lanes did real work while the triage half quietly ran
// on an empty list and the headline read "0 issues triaged" as if it were a result.
// A VERIFIED empty tracker is a legitimate state, and the first version of this guard refused it.
// The false green was a census that returned NOTHING while the tracker held 78 issues; a census that
// returns nothing AND reports a count of zero is agreeing with itself, and blocking there would make
// the workflow unable to audit standing code precisely when the backlog is clean. The guard exists to
// catch a census that CONTRADICTS itself, not one that is merely empty.
if (!issueNumbers.length && claimedCount === 0) {
  log('census reports zero open issues and lists none — consistent, so triage schedules no batches')
} else if (!issueNumbers.length) {
  throw new Error(
    'backlog-audit: the census returned NO open issue numbers, so triage would examine nothing and ' +
    'report success. That is a false green, not an empty backlog. Either the repository genuinely ' +
    'has zero open issues (verify with `gh issue list --state open`), or the Collect agent did not ' +
    'populate openIssueNumbers. Fix the census rather than running a triage over nothing.',
  )
}
// A truncated list is the quieter version of the same failure: some issues get audited, the rest are
// silently dropped, and nothing in the output distinguishes that from a clean sweep.
if (claimedCount && issueNumbers.length < claimedCount) {
  throw new Error(
    `backlog-audit: census reported ${claimedCount} open issues but listed only ${issueNumbers.length} ` +
    'numbers. Triage would silently skip the remainder. Re-run Collect with a --limit high enough to ' +
    'return them all.',
  )
}

const batches = []
for (let i = 0; i < issueNumbers.length; i += BATCH) batches.push(issueNumbers.slice(i, i + BATCH))

phase('Read')
log(`read fan-out: ${DOMAINS.length} domains + ${CROSS.length} cross-cutting + ${batches.length} issue batches (${issueNumbers.length} issues) = ${DOMAINS.length + CROSS.length + batches.length} lanes, all read-only`)

const auditThunks = DOMAINS.map((d) => () =>
  agent(
    `Audit the ${d.key} capability domain of a production Korean B2B console. Find real defects.

REPO: ${REPO}   REF: ${REF}
CRATES: ${d.crates.map((c) => `backend/crates/${c}`).join(', ')}
WHY THIS DOMAIN MATTERS: ${d.why}

${SAFETY}

THE CENSUS THIS RUN COLLECTED (use it to avoid re-filing what is already tracked):
${JSON.stringify(collected).slice(0, 6000)}

WHAT TO HUNT, in descending order of value. These are the classes this programme has actually been
bitten by, so they are worth more than a generic review:

 1. FAIL-OPEN GUARDS. A check that cannot see its subject, or that exits 0 on the empty case. Ask of
    every guard: where does it run in the sequence, does its subject exist yet, and what is the
    finest distinction its data source can express? A census that runs before migrations examines
    zero rows and passes. A per-crate rule enforced by a data source that only distinguishes roles
    never draws the crate boundary.
 2. TENANT AND SCOPE BOUNDARIES. RLS declared but not FORCED; a policy that special-cases one scope
    and leaves the others open; an aggregate that widens visibility by merging sources; a read that
    fails OPEN rather than closed when no org is armed. Note that CURRENT_ORG is a tokio task-local
    and a bare tokio::spawn does not inherit it — that failure returns ZERO ROWS, not an error.
 3. SECOND WRITERS AND DUAL SOURCES OF TRUTH. Two code paths writing one table; a value stored in
    two places that can disagree; a hand-maintained list mirroring something derivable. The canonical
    tables are already gated, so look at the ones that are NOT canonical.
 4. CLAIMS WIDER THAN THE CODE. A doc, comment or test NAME asserting a universal the code does not
    hold — "every", "always", "cannot", "the three ways X can happen". Pick the load-bearing
    assertion, break the code it guards, and say whether it actually goes red.
 5. LABOUR-LAW AND COMPLIANCE GUARDRAILS (§4-31), which are correctness here and not policy taste:
    연차 must have no reason field and cannot be refused (only 시기변경 협의); no overtime glorification;
    no discriminatory recruiting fields; statutory periods and rates must be catalogue-derived with
    the instrument cited, never hardcoded constants.
 6. IRREVERSIBILITY. Hard deletes where the charter requires 보관=숨김; a destructive path reachable
    before dependent objects are settled; an egress that sends before an approval gate.

DO NOT report style, naming, or "consider extracting a helper". Do not report a defect you cannot
demonstrate. An empty findings array from an honest sweep is a GOOD result and must be reported as
one — inventing findings to look productive is the failure mode this phase must avoid.

Before filing anything, check the census for an existing issue or bead covering it and name it in
existingIssue rather than creating a duplicate.

Also report STRENGTHS: controls that are genuinely load-bearing, so a later reader does not simplify
them away. This programme has twice nearly deleted a guard that looked redundant and was not.`,
    { label: `audit:${d.key}`, phase: 'Read', schema: FINDING_SCHEMA },
  ))

// Cross-cutting thunks: same evidence rules, but the unit of review is a PROPERTY of the tree
// rather than a directory in it.
const crossThunks = CROSS.map((c) => () =>
  agent(
    `Sweep ONE cross-cutting property of a production Korean B2B console. This is not a per-crate
review — the unit is the property, and it is deliberately the shape a per-crate reviewer cannot see.

REPO: ${REPO}   REF: ${REF}
THE PROPERTY: ${c.key}
WHY IT IS ITS OWN LANE: ${c.why}

${SAFETY}

THE CENSUS THIS RUN COLLECTED:
${JSON.stringify(collected).slice(0, 5000)}

The worst defects in this programme were never inside one crate. A canonical-writer census sat in a
reconcile script that runs BEFORE migrations, so it examined zero tables and passed in every
automated path. A gate existed, compiled and was tested, and no CI step ever invoked it. A module doc
enumerated "the three ways X can happen" while reviewers had already proven a fourth and a fifth.
Each is a property of the SEAM between things, and no reviewer holding one directory could have found
any of them.

So: read ACROSS. Follow the property wherever it goes. Report the same evidence-bound findings the
per-domain lanes report — file:line, the command, its real output — and mark provenByExecution TRUE
only for what you actually ran.

An empty findings array from an honest sweep is a GOOD result. Say what you covered and what you did
not reach, because a claimed sweep that skipped half the tree is worse than a partial one that says so.`,
    { label: `cross:${c.key}`, phase: 'Read', schema: FINDING_SCHEMA },
  ))

// --- Triage ----------------------------------------------------------------
// Issues are batched so each agent holds a readable set and can compare within it for duplicates.
const triageThunks = batches.map((b, i) => () =>
  agent(
    `Triage GitHub issues ${b.join(', ')} in ${GH} against the CURRENT tree. Batch ${i + 1} of ${batches.length}.

REPO: ${REPO}   REF: ${REF}

${SAFETY}

FOR EACH ISSUE: read it in full (\`gh issue view <n> --comments\`), then go and LOOK at the tree.
The whole point is that issue text describes a repo that has moved on. Decide:

 KEEP            — still a real, open problem. Say what remains true.
 CLOSE-FIXED     — the code now does what the issue asked. EVIDENCE REQUIRED: the commit, the test,
                   or the file:line that implements it. Prefer a merged PR from the census.
 CLOSE-OBSOLETE  — the thing it is about no longer exists, or a decision superseded it. Name the
                   deletion or the ADR.
 CLOSE-DUPLICATE — another issue covers it. Give the number, and prefer keeping the one with more
                   evidence rather than the older one.
 NEEDS-OWNER     — real, but the next step is a human decision, not work. Say what the decision is.

THE BAR FOR CLOSING, and it is deliberately high: a wrongly closed issue is INVISIBLE, while a stale
open one is merely noise. If you cannot point at the thing that makes it moot, the verdict is KEEP
with a staleness note. "Looks done", "probably superseded" and "no longer relevant" are not evidence.

ALSO RECORD STALENESS for KEEP items: what in the issue text is now factually wrong — a renamed file,
a moved line number, a crate that no longer exists, a fixed sub-part. That is what makes an old issue
expensive to pick up, and writing it down is most of the value of this pass.

Mark beadCandidate=true for KEEP items that are real work someone should schedule, so the reconcile
phase can mirror them into the working tracker.`,
    { label: `triage:${i + 1}`, phase: 'Read', schema: TRIAGE_SCHEMA },
  ))

// ONE parallel wave. Audit, cross-cutting and triage are mutually independent reads, so stacking
// them in three phases would multiply the wall-clock by three and buy nothing. The harness caps
// concurrency itself, so a wide list queues rather than overloads — width costs latency only when
// it exceeds the cap, and even then it degrades linearly instead of serialising.
const all = await parallel([...auditThunks, ...crossThunks, ...triageThunks])
const audits = all.slice(0, DOMAINS.length)
const crosses = all.slice(DOMAINS.length, DOMAINS.length + CROSS.length)
const triaged = all.slice(DOMAINS.length + CROSS.length)

// A dead lane is not an absent finding. Name every one, so a partial sweep can never read as a
// complete one — the same rule the lane-fanout harness learned when session limits silently took
// 4 of 7 agents from one run and 3 of 3 from another.
const auditOk = audits.filter(Boolean)
const crossOk = crosses.filter(Boolean)
const deadDomains = DOMAINS.map((d, i) => (audits[i] ? null : d.key)).filter(Boolean)
const deadCross = CROSS.map((c, i) => (crosses[i] ? null : c.key)).filter(Boolean)
const deadBatches = batches.map((b, i) => (triaged[i] ? null : `#${b[0]}-${b[b.length - 1]}`)).filter(Boolean)
const dead = [...deadDomains, ...deadCross, ...deadBatches]
log(`read: ${auditOk.length}/${DOMAINS.length} domains, ${crossOk.length}/${CROSS.length} cross-cutting, ${triaged.filter(Boolean).length}/${batches.length} issue batches`)
if (dead.length) log(`read: DIED and therefore UNAUDITED — ${dead.join(', ')}`)

const findingsAll = [...auditOk, ...crossOk]
const totalFindings = findingsAll.reduce((n, a) => n + (a.findings || []).length, 0)
const provenFindings = findingsAll.reduce((n, a) => n + (a.findings || []).filter((f) => f.provenByExecution).length, 0)
log(`read: ${totalFindings} findings, ${provenFindings} proven by execution`)

const triageOk = triaged.filter(Boolean)
const verdicts = triageOk.flatMap((t) => t.verdicts || [])
const closing = verdicts.filter((v) => String(v.verdict).startsWith('CLOSE'))
const unevidenced = closing.filter((v) => !v.evidence || v.evidence.trim().length < 40)
log(`triage: ${verdicts.length} verdict(s); ${closing.length} propose closing; ${unevidenced.length} of those lack real evidence`)
if (unevidenced.length) log(`triage: WITHHELD from closing for want of evidence: ${unevidenced.map((v) => '#' + v.number).join(', ')}`)

// --- Reconcile -------------------------------------------------------------
// A blind `.slice(0, 24000)` over the serialised findings dropped everything past the cap without a
// word, and with ~32 read lanes the cap is reached routinely — so the single writer filed beads for a
// prefix of the audit and reported success. Silent truncation reads as "covered everything". Order by
// what a reconciler must not miss (proven, then severity), and when the budget still binds, SAY what
// was cut so the omission is visible in the run rather than discovered six weeks later.
function renderFindings(all) {
  const rank = { blocker: 0, major: 1, minor: 2, nit: 3 }
  const ordered = [...all].sort((a, b) =>
    (b.provenByExecution === true) - (a.provenByExecution === true) ||
    (rank[a.severity] ?? 9) - (rank[b.severity] ?? 9))
  const kept = []
  let budget = 24000
  for (const f of ordered) {
    const line = JSON.stringify(f)
    if (line.length > budget) break
    budget -= line.length
    kept.push(f)
  }
  const dropped = ordered.length - kept.length
  return JSON.stringify(kept) + (dropped
    ? `\n\n!! ${dropped} of ${ordered.length} findings DID NOT FIT this prompt and are NOT above. They are` +
      ' the lowest-ranked ones, but they are unfiled. Your report MUST state that this run reconciled' +
      ` ${kept.length} of ${ordered.length} findings, so the omission is visible.`
    : '')
}

phase('Reconcile')
const reconciled = await agent(
  `You are the SINGLE WRITER for the backlog. Turn this audit into tracked work, and close what is dead.

REPO: ${REPO}   GH: ${GH}   APPLY: ${APPLY}

${SAFETY}

AUDIT + CROSS-CUTTING FINDINGS (${totalFindings} total, ${provenFindings} proven by execution):
${renderFindings(findingsAll)}

ISSUE VERDICTS (${verdicts.length}):
${JSON.stringify(verdicts).slice(0, 16000)}

WITHHELD FOR WANT OF EVIDENCE — these were proposed for closing and must NOT be closed:
${unevidenced.map((v) => `#${v.number} ${v.title || ''}`).join('\n') || '(none)'}

${dead.length ? `LANES THAT NEVER REPORTED — their scope is UNAUDITED and the summary MUST say so: ${dead.join(', ')}` : 'Every read lane reported.'}

DO, in this order:

1. DE-DUPLICATE ACROSS SOURCES before writing anything. The same defect may appear as an audit
   finding, an open issue, and an existing bead. Collapse them and say which record wins. Filing a
   fourth copy of a known problem makes the backlog worse, not better.

2. RANK. Order by severity and blast radius, with proven-by-execution ahead of argued. A tenant or
   money defect outranks a doc drift regardless of how neatly the doc drift is written up.

3. ${APPLY ? 'CREATE BEADS' : 'DRAFT BEADS (do not create — apply is false)'} for confirmed findings.
   Each bead must carry the EVIDENCE inline — file:line, the command, the observed output — so
   nobody re-derives it. A bead whose body is a restatement of its title is worthless six weeks
   later. Set priority from the ranking, and wire dependencies where one finding blocks another.

4. ${APPLY ? 'CLOSE ISSUES' : 'LIST ISSUES TO CLOSE (do not close — apply is false)'} for
   CLOSE-* verdicts that carry real evidence. The closing comment must state WHY, name the commit or
   file that makes it moot, and be written for the person who filed it — this repository's sibling
   project comments an explicit acceptance and then closes with the merged PR named, which is the
   pattern to follow. Never close silently.

5. SYNC THE TWO TRACKERS, and state the rule you applied rather than inventing one per item:
   GitHub issues are the durable, public record; beads are the working queue. So every KEEP issue
   marked beadCandidate should have a bead, and every bead representing work others should see
   should reference an issue. Report the drift you found — this run began with ${issueNumbers.length}
   open issues against a far smaller bead set, which is itself the finding.

6. REPORT: what was filed, what was closed, what was WITHHELD and why, which domains went unaudited,
   and the three things you would fix first. Be honest about coverage — a sweep that missed a domain
   and says so is worth more than one that implies completeness it does not have.`,
  { label: 'reconcile', phase: 'Reconcile' },
)

return {
  headline: [
    `${auditOk.length}/${DOMAINS.length} domains + ${crossOk.length}/${CROSS.length} cross-cutting audited`,
    `${totalFindings} findings (${provenFindings} proven)`,
    `${verdicts.length} issues triaged, ${closing.length - unevidenced.length} closable, ${unevidenced.length} withheld`,
    // Never claim APPLIED on the strength of the caller's flag alone. When the Reconcile agent dies,
    // agent() returns null and the mutations are absent — or worse, a partial prefix of them landed
    // and nothing records which. An operator reading APPLIED would take a success claim over an
    // unknown state, which is the same false-green class this workflow exists to find.
    APPLY ? (reconciled ? 'APPLIED' : 'RECONCILE DIED — mutations UNKNOWN, possibly partial; verify by hand') : 'REPORT ONLY (apply=false)',
  ],
  dead,
  audits: auditOk,
  cross: crossOk,
  verdicts,
  withheld: unevidenced.map((v) => v.number),
  reconciled,
}
