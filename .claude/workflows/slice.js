export const meta = {
  name: 'slice',
  description: 'Build ONE vertical slice under Bun-rewrite discipline: parallel exploration, competing designs judged, single implementer, then two adversarial reviewers who see the DIFF ONLY. Reusable for any slice — parameterised by task, exploration areas, and owned paths.',
  whenToUse: 'Any bounded implementation slice where correctness matters more than speed: a new crate, a gate, a domain type, a migration. Not for trivial edits (a 2-line change does not need eight agents).',
  phases: [
    { title: 'Explore', detail: 'parallel readers — read the code, do not guess' },
    { title: 'Design', detail: 'competing designs, then judged and synthesised' },
    { title: 'Implement', detail: 'one implementer, coherent slice, escalation path' },
    { title: 'Prove', detail: 'two adversarial reviewers — diff only, no author reasoning' },
  ],
}

// Workflow SCRIPT, not a Node module: the runtime injects agent()/parallel()/phase()/log()/args
// and runs the body in an async context. Top-level await and the trailing return are the
// documented form — `node --check` will wrongly flag the return. Do not wrap in a function.
//
// args: {
//   task:     string   — what to build. Required.
//   context?: string   — domain facts the agents cannot infer.
//   explore?: [{label, prompt}]  — exploration areas. Defaults to one generalist reader.
//   owns?:    string   — paths the implementer owns. A COHERENT SLICE, not a file list (see below).
//   crate?:   string   — crate boundary for the work queue. Bun grouped ~16k errors BY CRATE,
//                        never by file, explicitly to prevent task fragmentation.
//   designs?: number   — competing designs before the judge. Default 2.
//   lane?:    "1".."5" — REQUIRED for anything that builds. Routes implement/prove into
//                        ~/Developer/console-lanes/lane-N, which has its own backend/target.
//                        Omitting it builds in the main checkout and contends on the build lock.
//   repo?:    string
// }
// NO NODE GLOBALS. The script body runs in a bare sandbox: `process` is not defined, so a
// `process.env.HOME` fallback is not a fallback — it is an immediate ReferenceError that kills the
// run before agent 1 starts. (`Date.now`/`Math.random` are likewise banned, as they would break
// resume.) Paths are therefore literals or `args`. Verified by the failure this line replaced.
const HOME = '/Users/jasonlee'
const A = typeof args === 'string' ? JSON.parse(args) : (args || {})
if (!A.task) throw new Error('slice: `task` is required')
const REPO = A.repo || `${HOME}/Developer/console`

// MEASURED 2026-07-28: a workflow that ran its implementer with cwd = the MAIN checkout turned a
// single-crate `cargo check` into 47 MINUTES — the log reads `Blocking waiting for file lock on
// build directory`, because the implementer and the caller contended on one `backend/target`. CI
// for the same change is 20 minutes. The bottleneck was never CI; it was a shared target dir.
//
// Anything that BUILDS therefore runs in a lane worktree, which has its own `target/`. Read-only
// exploration may use the main checkout because reads take no build lock.
const LANE = A.lane ? `${HOME}/Developer/console-lanes/lane-${A.lane}` : null
const BUILD_CWD = LANE || REPO
if (!LANE) log('WARNING: no `lane` arg — implement/prove will build in the MAIN checkout and may contend on the build lock. Pass lane: "1".."5".')
const N_DESIGNS = Math.min(Math.max(A.designs ?? 2, 1), 3)
const CRATE = A.crate || ''
const OWNS = A.owns || ''

// ── Discipline shared by every agent ────────────────────────────────────────
// Each line below was earned by a measured failure, not adopted on principle.
const DISCIPLINE = `
## Non-negotiable discipline

- **Verify by EXECUTION.** Run it; quote real output. Cite \`file:line\` of CODE, never a header
  comment — a plan premise died because a migration's header described the problem it had already
  fixed. Reasoning that feels airtight has been wrong repeatedly here.
- **A probe must be proven RED on a known-bad input before its GREEN is trusted.** Six verification
  probes were defective in one session; in one case the models were right and the *grader* was wrong.
  A probe with no demonstrated failure mode is not evidence.
- **Reproduce the original failure, not the artifact you touched.** A repair once fixed 1 of 3
  commands and reported green because it tested only its own file.
- **\`git stash\` and \`git reset\` are BANNED.** Commit or abandon. Atomic, per-file commits.
  (Bun had to amend their workflow to forbid these after agents used them to escape trouble.)
- **Write findings to disk as you go.** Agents have gone idle without ever returning a report; do not
  rely on your final message surviving.
- **\`grep\` is unreliable in this shell** (exits 1 on files it matches). Use \`awk\` for anything
  load-bearing.
- **Escalate rather than settle.** If the correct fix lies outside your slice, STOP and report it.
  Do not implement the second-best fix — an implementer once knowingly shipped a worse design because
  a briefing file-list omitted the file the real fix needed. The briefing was the defect.
`

const CTX = `
Repo: ${REPO}
${A.context ? A.context + '\n' : ''}${CRATE ? `Crate boundary: \`${CRATE}\`. Group work BY CRATE, never by file — \`cargo check -p ${CRATE}\` is the work queue.\n` : ''}${OWNS ? `Slice you own: ${OWNS}\n` : ''}
## Task
${A.task}
${DISCIPLINE}`

// Appended to EVERY phase, exploration included.
//
// The first version excluded exploration on the reasoning that "reads take no build lock". That
// premise was refuted by observation within the hour: an explorer asked to confirm a call signature
// ran `cargo test -p console-ontology-rest --test publish_auto_create_action_as_runtime_role`
// against the MAIN checkout's Cargo.toml and held `backend/target/debug/.cargo-lock` for over seven
// minutes, blocking `npm run verify` behind `Blocking waiting for file lock on build directory`.
// Verifying a signature by executing it is exactly the discipline demanded elsewhere in this file,
// so the fix is to give exploration a lane too — not to tell it to stop running things.
const WORKDIR = `
## WORKING DIRECTORY — build here, nowhere else

**Before your first cargo command, in the same shell:**
\`\`\`bash
cd ${BUILD_CWD}
source ${REPO}/scripts/console/lane-env.sh    # RUSTC_WRAPPER=sccache, 50G ceiling
\`\`\`
sccache's cache is user-global (\`~/Library/Caches/Mozilla.sccache\`), so lanes — and other repos on
this machine — reuse each other's compiled artifacts. Without it every lane recompiles the whole
dependency graph from cold: measured \`Cache hits: 0%\` across 4,084 commands, because nothing had
ever sourced this. Confirm with \`sccache --show-stats\` after your build; hits should climb.

${LANE
  ? `This is an isolated lane worktree with its own \`backend/target\`, so your build cannot contend with
another agent's. Building in the main checkout instead is what turned a single-crate \`cargo check\`
into 47 minutes.`
  : `NOTE: no lane was assigned, so this IS the main checkout and you may contend on the build lock
with concurrent agents. Report the contention if a build stalls rather than waiting it out.`}
`

phase('Explore')

const EXPLORE_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['area', 'findings', 'exact_api', 'gotchas'],
  properties: {
    area: { type: 'string' },
    findings: { type: 'string', description: 'What EXISTS, with file:line. Concrete, not summary.' },
    exact_api: { type: 'string', description: 'Exact signatures/paths to call, copied verbatim from source.' },
    gotchas: { type: 'array', items: { type: 'string' }, description: 'What will silently break a naive implementation.' },
  },
}

const areas = A.explore?.length ? A.explore : [{
  label: 'survey',
  prompt: 'Survey what already exists for this task. The most valuable finding is that some of it is already built — assume nothing is greenfield until you have checked.',
}]

const facts = (await parallel(areas.map((a) => () =>
  agent(`${CTX}${WORKDIR}\n## YOUR EXPLORATION AREA\n${a.prompt}\n\nRead the code. Return exact APIs a caller must use — an implementer will build directly from your answer, so a vague finding becomes their wrong guess.`,
    { label: `explore:${a.label}`, phase: 'Explore', schema: EXPLORE_SCHEMA })
))).filter(Boolean)

log(`${facts.length}/${areas.length} exploration lanes returned`)

phase('Design')

const DESIGN_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['approach', 'file_layout', 'verification', 'risks'],
  properties: {
    approach: { type: 'string' },
    file_layout: { type: 'string', description: 'Exact paths and what each contains.' },
    verification: { type: 'string', description: 'How this is PROVEN to work, including what would make it fail.' },
    risks: { type: 'array', items: { type: 'string' } },
  },
}

// Competing designs, not one. Bun could skip this because a .zig reference removed
// design entirely; where no reference exists, one design is one unexamined guess.
const BIASES = [
  'BIAS: minimal surface. Fewest files, least abstraction, no framework. An interface with one implementation is a defect here.',
  'BIAS: durability. Optimise for surviving change underneath you. What breaks when the substrate moves, and what makes this immutable?',
  'BIAS: falsifiability. Optimise for the failure being LOUD and specific. How does this fail for the right reason, and how would a silent pass be detected?',
]

const designs = (await parallel(
  Array.from({ length: N_DESIGNS }, (_, i) => () =>
    agent(`${CTX}\n\nVerified exploration findings:\n${JSON.stringify(facts, null, 2)}\n\n## Design it\n${BIASES[i % BIASES.length]}`,
      { label: `design:${i + 1}`, phase: 'Design', schema: DESIGN_SCHEMA }))
)).filter(Boolean)

const spec = designs.length === 1 ? JSON.stringify(designs[0], null, 2) : await agent(
  `${CTX}\n\nCompeting designs:\n${JSON.stringify(designs, null, 2)}\n\n## JUDGE and SYNTHESISE

Pick the better spine and graft the best ideas from the others. State plainly which you chose and
what you took from the losers — a synthesis that silently drops a rival's best idea is a worse
outcome than either design alone.

Judge in this order:
1. **Is it verifiable, and does it fail LOUDLY?** A design that cannot demonstrate its own failure is
   disqualified regardless of elegance.
2. **Is it simple?** Fewest files, least abstraction.
3. **Does it survive the substrate changing?**

Return a FINAL SPECIFICATION concrete enough to implement with no further design decisions: exact
paths, exact APIs from the exploration findings, and the exact proof of correctness.`,
  { label: 'judge', phase: 'Design' })

phase('Implement')

const impl = await agent(`${CTX}${WORKDIR}

## SPECIFICATION — already judged. Implement it; do not redesign.
${spec}

Exploration findings you may rely on:
${JSON.stringify(facts, null, 2)}

## Your job
Implement it. Leave changes in the working tree, UNCOMMITTED — the caller owns landing.
${OWNS ? `You own: ${OWNS}. Touching anything else is a scope violation; escalate instead.\n` : ''}
Requirements:
1. It must COMPILE${CRATE ? ` — run \`cargo check -p ${CRATE}\` and iterate until clean` : ''}.
2. It must be PROVEN, by execution, with real output quoted.
3. If you add a Rust crate, a valid \`Cargo.toml\` lands in the SAME change — an unmatched workspace
   glob breaks the build for every lane. Never hand-edit a generated BUCK file.
4. **Report what you could NOT verify, plainly.** An unproven claim stated as fact is worse than an
   admitted gap, because it survives review.`,
  { label: 'implement', phase: 'Implement' })

phase('Prove')

const VERDICT_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['verdict', 'proven_failable', 'weakening', 'residual'],
  properties: {
    verdict: { type: 'string', enum: ['PASS', 'FAIL'] },
    proven_failable: { type: 'string', description: 'Concrete inputs you RAN that make it fail, with real output. Could not make it fail = FAIL.' },
    weakening: { type: 'string', description: 'Was any assertion deleted, loosened, skipped or made conditional? Diff it yourself.' },
    residual: { type: 'array', items: { type: 'string' }, description: 'Still vacuous or unverified after this change.' },
  },
}

// Bun's reviewers received the DIFF ONLY and never the implementer's reasoning — the author's
// narrative primes a reviewer to accept. Passing the implementer's report here (as I did in an
// earlier workflow) quietly defeats the mechanism, so these agents are told to read the diff
// themselves and are given no summary of it.
const REVIEW_BASE = `${CTX}${WORKDIR}

**READ-ONLY. No edits, no git mutations.**

An implementation was just made in this working tree. **Read the diff yourself** — \`git diff\` and
\`git status --short\`. You are deliberately NOT given the implementer's account of what they did:
their reasoning would prime you to accept it, and Bun's reviewers saw the diff alone for exactly
this reason.

Assume the change is WRONG and look hard before conceding anything. Bun's reviewers also rejected
solutions that needed a paragraph of justification to defend — a workaround that must be explained
is a defect.`

const verdicts = (await parallel([
  () => agent(`${REVIEW_BASE}

## YOUR LENS: vacuity and correctness
1. Can you make it FAIL? Construct concrete failing inputs and RUN them. If you cannot make it fail,
   that is a FAIL verdict — a check that cannot fail is not a check.
2. Would a trivial STUB satisfy this? Try to write one. Success here is the most valuable finding
   available.
3. Was any existing assertion deleted, loosened or made conditional?`,
    { label: 'prove:vacuity', phase: 'Prove', schema: VERDICT_SCHEMA }),

  () => agent(`${REVIEW_BASE}

## YOUR LENS: discipline and honesty — deliberately different from the other reviewer
1. Does it actually compile and run? Run it. Do not accept that it does.
2. Are the project's testing constraints honoured — non-superuser runtime role, disposable Postgres,
   \`--test-threads=1\`, no hand-edited generated files?
3. **Is anything claimed that was not verified?** Compare what the code proves against what any
   comment or docstring asserts. Overstatement that survives review becomes tomorrow's false premise.
4. Did it stay inside its slice? Check \`git status\` for files outside ${OWNS || 'the declared scope'}.`,
    { label: 'prove:discipline', phase: 'Prove', schema: VERDICT_SCHEMA }),
])).filter(Boolean)

const passed = verdicts.length === 2 && verdicts.every((v) => v.verdict === 'PASS')

// "Edit the process, not the outputs." When a lane produces bad work, Bun fixed the PROMPT and
// reran rather than hand-patching the diff. Emit that rerun brief instead of a bare failure, so the
// next attempt corrects the instruction rather than the symptom.
const rerun = passed ? null : {
  guidance: 'Do NOT hand-patch the diff. Fix the brief and rerun this workflow — edit the process, not the outputs.',
  failures: verdicts.filter((v) => v.verdict !== 'PASS').flatMap((v) => [v.proven_failable, v.weakening]).filter(Boolean),
  suggested_context_additions: verdicts.flatMap((v) => v.residual || []),
}

return { specification: spec, implementation: impl, verdicts, passed, rerun }
