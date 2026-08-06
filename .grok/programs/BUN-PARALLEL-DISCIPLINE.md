# Bun rewrite lessons — Console delivery process

Canonical process rules for multi-PR / multi-lane Console delivery.  
**Doctrine source:** [Rewriting Bun in Rust](https://bun.com/blog/bun-in-rust) — prep first, trial before fan-out, dual adversarial review, **edit the process** so agents cannot repeat the mistake.

Workflows (edit these when fixing process):

| Workflow | Role |
|----------|------|
| **`console-drive`** | **Primary Bun-style driver** — process tools + fleet repair/merge + **product implement** + process-edit + learn. Edit *this* when the drive fails as a class. |
| `program-control` | Lighter heartbeat (fleet + implement + audit); prefer `console-drive` for full implementation |
| `ultragoal` | Durable multi-story plan + activate Stop-hook loop + native `/goal` handoff |
| `ralplan` | Planner → Architect → Critic consensus before heavy execute |
| `ralph` | PRD story loop until passes + APPROVE (“boulder never stops”) |
| `learn` | Hermes-style reflection / tip / skill draft / process-upgrade promote |
| `work-manager` | Discover lanes; **enqueue every soft red/block** (no silence) |
| `implement-lane` | Claim board/beads item → **implement code** → open PR |
| `pr-babysit` | Repair → agent review → **approve then merge**; fix until approve |
| `domain-increment` | Full backend SDLC + Admit (build is capability `all`) |
| `process-upgrade` | Map a failure class → permanent control |
| `program-tick` | Legacy dual-track (fleet + product/process) |

**Primary entry:** `/workflow console-drive` — not “track reds only”.

### Hook-driven loop (Grok-native)

`.grok/hooks/ultragoal-loop.json` → `SessionStart` + `Stop` run `bin/console-hook-*`.  
While `ultragoal/active-goal.live.json.active`, Stop **blocks** and dispatches `/workflow program-control` or `ralph`.  
Pair with native `/goal <objective>`. Trust project hooks: `/hooks-trust`.

Authority remains `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md`. This file is **process**, not product.

## What "fix the process" means

**Means:** edit a **reusable workflow / harness / tool** so the next agent cannot make the same *class* of mistake without a hard fail, forced step, or explicit waiver.

| Fix class | Do this | Not this |
|-----------|---------|----------|
| **Workflow** | New phase in `domain-increment` / `open-pr-fleet`; admit gate; tip-sync step | Chat note "remember to run lens check" |
| **Tool / script** | Extend `npm run check:*` / `admit` / residual gate | One-line fix on one PR and stop |
| **Harness catalog** | Add row to `failure-classes.v1.json` + control path | One-shot hook named for a PR number |
| **Memory tip** | `.grok/memory/tips/{class}.md` after repeat ≥2 | Session-only folklore |

**Does not mean:** expanding product scope, clearing PRODUCT HOLDs, auto-merge to main (Console: **leader-only** merge), or inventing features to avoid a process gate.

When a mistake class repeats **twice**, promote a process edit (see `learning-loop.v1.json`).

## Six Bun rules (Console)

1. **Prep contract first** — admission + OWNERSHIP/allowlist before multi-agent write.
2. **One representative trial before fan-out** — do not open N domain lanes while the trial path is red.
3. **Dual split-context review** — implementer ≠ reviewer context (`domain-increment` Review phase).
4. **Fail closed on missing evidence** — no handoff without admit green; no "complete" without Required CI on exact head when claiming merge-ready.
5. **Edit process when systematic** — if agents keep doing X, change workflow/tool so X fails closed.
6. **Do not expand while trial red** — no second CI graph writer while #ci tip is red; no product fan-out without prep pack.

## Systematic mistakes → required process edits

| Class ID | Observed | Control (must live in tool/workflow) |
|----------|----------|--------------------------------------|
| `auth.unsigned-tip` | C/T not pinned SSH after rebase | Signed train checklist in handoff; authority bootstrap |
| `lens.noncanonical-json` | Hosted preflight lens fail | `check-reasoning-lens-contract` in admit for ledger paths |
| `docs.tip-blob-prebind` | Manifest ≠ tip blob | `check:doc-manifest` on tip content before push |
| `ci.verify-job-ids` | verify.mjs / preflight digests | `check:ci-preflight` + `test:verify` |
| `ci.residual-buck-growth` | New Buck wrappers in ci.yml | `check:product-buck-residual` |
| `ops.passive-wait` | Watch-only CI turns | `program-tick` / `open-pr-fleet` anti-passive; dual track |
| `ops.mid-run-push` | Push while CI in_progress cancels cone | Fleet rule: one push → wait complete |
| `ops.missed-tip-sync` | PR BEHIND after main moves | `open-pr-fleet` tip-sync report (leader restacks) |
| `ops.skip-admit` | Push without local gates | PreToolUse / admit phase in workflows |
| `ops.tip-serial-contention` | Multiple tip-writing PRs thrashing serial queue | `npm run assess:tip-contention`; tip-serial merge queue |
| `ops.multi-pr-wall-tax` | N small PRs each pay full Required CI wall | Batch pure tests; stack when tip would serialize |

## Anti-passive rule

`waiting_ci` is **not** idle. Every autonomous wake must: merge-report · fix · restack/preflight · fan-out ready work · or advance tracker.  
Unchanged WAIT narration alone is forbidden (Hindsight `autonomous-drive-no-passive-watch`).

## Autonomous merge (operator override 2026-08-06)

Policy file: `.grok/harness/autonomy-merge.v1.json`.

1. **Agent review** must produce verdict `approve` | `changes_requested` | `comment`.
2. If **not approve** → fix on branch (and/or board `review_fix`) until **approve**, up to max fix rounds; residual becomes board **blocked** with evidence — **never silent**.
3. If **approve** + Required CI green + Required Security green + mergeable/not BEHIND → **`gh pr merge --squash` without human gate**.
4. **Human supervises** and intervenes only if something looks awry, HOLD clearance is required, or production/secrets exposure.
5. Soft reds and blocks are first-class work: see **Soft reds & blocks** below.

## Soft reds & blocks (no silent drop)

Harness: `.grok/harness/lane-board.v1.json` + live board `.grok/harness/lane-board.live.json`.

**Soft reds** include (non-exhaustive): non-required CI fails, cancelled/flaky runs, BEHIND/DIRTY/CONFLICTING, auth bootstrap fail, tip prebind/unsigned, stale waiting_ci, review not approve, tip_serial contention, baseline/manifest drift, beads blocked without owner.

**Rules:**

- Every observed soft red or block **must** become a board item (`source_key` dedupe).
- `work-manager` silence_check fails closed if any observation is missing from the board.
- `pr-babysit` / `program-control` productivity audit fails if open PR issues exist with no board coverage.
- Class id: `ops.soft-red-silence` — treating a soft red as "noise" or leaving it unowned is a process defect.

## Parallelism

- **Serialize:** authority tip, `ci.yml` / preflight digests, migrations, Cargo.lock, OpenAPI faces, `docs/current/**`.
- **Parallel:** path-disjoint backend crates after prep pack; process-upgrade PRs vs product PRs when paths disjoint.
- Prefer **one tip-writing PR** at a time; batch pure-domain tests rather than N tip PRs.

## Tip-serial queue (authority + baseline)

These paths are a **single writer** across the fleet (not just concurrent git conflicts — each PR pays full CI after restack):

- `docs/documentation-manifest.seed.json` / `docs/documentation-index.json`
- `docs/program/ledger/**` (authority tip train)
- `docs/program/executed-tests-baseline.json`

**Rules:**

1. Maintain an explicit **tip-serial queue** ordered by merge readiness.
2. Do **not** open a new tip-writing PR while ≥2 tip writers are already open — stack commits or wait.
3. Pure `#[test]` hardenings across domains: **prefer one PR / stack** when tip would serialize them anyway.
4. `program-tick` must report `tip_writers` and flag `ops.tip-serial-contention`.

## CI wall tax

Hosted Required CI wall is ~25–45m. Opening N independent PRs multiplies wall cost.  
Fan-out is still correct for **path-disjoint product crates that do not share tip/baseline** — rare under current authority train. Until tip binding is less chatty, **local parallelism** (implement next stack commit while CI runs) beats **remote PR fan-out**.

