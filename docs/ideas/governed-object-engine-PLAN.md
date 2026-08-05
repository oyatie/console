# PLAN — Governed Object Engine: substrate + composition

**Status: PENDING APPROVAL — planning artifact. No execution authorized.**
Date: 2026-07-28 · Mode: deliberate · Source: `docs/ideas/governed-object-engine.md`

---

## 0. Charge and the corrected premise

Build the company-building-block backend: **ontology · foundry · policy**, then **organization +
employee**, then **HR + payroll**. Benchmarks: AWS Cedar (policy), Palantir Foundry (ontology/actions/
lineage). Frontend deleted; the 23KB console shell returns last as acceptance test.

**Premise correction (2026-07-28).** "Greenfield" was scoped down to *"reuse what exists but treat it
as greenfield — reimagine our work."* Direct inspection then showed the two hardest components already
exist and carry non-superuser runtime-role tests:

| Asset | Size | Evidence |
|---|---|---|
| Ontology engine (§18 model) | 15,372 LOC | `SchemaLifecycleState`, `BackingKind`, `LinkCardinality`, `ActionDispatch`, `InstanceLifecycleState`, `FieldKind` in `crates/ontology/domain/src/lib.rs` |
| Cedar partial-eval → SQL residual | 7,246 LOC | `platform/authz/src/cedar_pbac.rs` `residual` mod; `ontology/adapter-postgres/tests/instances_residual_filter_as_runtime_role.rs` |
| Audit chain | 2,325 LOC | `platform/audit-chain` |
| Cedar | pinned `4.11.2` | `platform/authz/Cargo.toml:17` |

Rewriting these would destroy the single hardest thing in the design (residual list-filtering) and
rediscover it at cost. **Therefore "reimagine" targets the substrate and the composition, not the
engine.** Confirmed by the owner.

### SUBSTRATE RESOLVED (verified 2026-07-28) — it already exists

The plan's largest unknown is answered, and the answer collapses the biggest risk. **There are two
substrates in this repo and they must not be confused:**

| | `platform/db/versioning.rs` (legacy domain) | `ontology/adapter-postgres/src/instances.rs` (engine) |
|---|---|---|
| Model | mutable rows + JSON snapshot sidecar | **append-only revisions; state is a FOLD** |
| Evidence | `capture(tx, org, id, before, after)` — a `before` only exists if you mutated in place; doc says v1 is *backfilled* | module doc `:4-10`; **`attributes` is never `UPDATE`d** — only `INSERT INTO ont_instance_revisions` |
| Only UPDATEs | content itself | `lifecycle_state`, `current_revision_id` (pointers), `valid_to` (interval close) |
| Effective-dating | none | `r.valid_from <= $2 AND (r.valid_to IS NULL OR $2 < r.valid_to)` (`:374-375`) |
| As-of | no | yes — current is `valid_to IS NULL` |
| Fixity | no | `sha2::Sha256`, genesis `prev_hash` = 64 zeros, `prev_hash`/`row_hash` chained per `(org, instance)` |
| Graph | n/a | bounded BFS over effective-dated `ont_links` (`:527`) |
| Size | 201 LOC | **1,480 LOC** |

**Because the catalog is built as ENGINE types, we inherit the good substrate and never touch the weak
one.** Event-log-first is therefore neither an addition nor a rewrite — it is *already built*.

Consequences:
- **Pre-mortem S2 ("substrate rewrite metastasizes") is MOOT.** Delete it as a risk.
- `SUBSTRATE.tsv` (P3) is no longer needed as a migration-sizing artifact.
- The project is materially smaller than planned. The reference slice is **modelling work on a working
  engine**, not substrate construction.

### What is actually new work (revised)

1. ~~Event-log-first substrate~~ — **EXISTS**. Verified above.
2. **Composition** — nothing assembles registry + instances + actions + policy into a demonstrable
   company. **This is now the whole job.** It is the gap the deleted frontend was concealing.
3. **Default catalog** — OrgUnit, Position, Person, Employment, PayRun as *engine types*, not bespoke
   crates, and explicitly not on the legacy `versioning.rs` path.
4. **Rename** — `mnt` / `maintenance` / `mnt_rt` deprecated; greenfield makes the previously
   infra-bound role rename cheap and safe for the first time.

---

## 0.5 Locked decisions (2026-07-28)

| Decision | Choice | Rationale |
|---|---|---|
| Repo root | `~/Developer/console`, remote `jason931225/console` | renamed from `maintenance` mid-session; `mnt` / `maintenance` / `mnt_rt` all deprecated |
| Frontend stack | **Leptos 0.9 (beta)**, SSR + islands/selective hydration | one language across the stack; agents refactor across the server/client seam in one change; ontology types used directly in UI with no serialization layer |
| Rejected | Next.js 16.2.12 / RSC; React Native | RSC is the more mature selective-hydration model, but this design hand-rolls every component (inline styles, hand-inlined SVG, charter bans component-library aesthetics) — so its ecosystem advantage would go uncashed. Reintroducing TS would restore the codegen tax we just deleted |
| Call boundary | Server functions **and** REST as sibling adapters over the application-layer use-cases | console uses typed server fns (no codegen); REST retained for machine/external consumers + `openapi_drift`. One set of business logic, two thin doors |
| Conformance boundary | **Application use-case layer**, not HTTP | server fns don't traverse REST; a REST-only suite would test a surface the real consumer never uses |
| Build system | **buck2 RETAINED** (decision corrected 2026-07-28) | ⚠️ **My earlier "drop buck2" advice was wrong and was given on incomplete evidence.** I judged from a working tree that was 8 commits behind; buck2 landed in those commits. Main has **761 buck2 files** — `.buckconfig`, root `BUCK`, `third-party/rust/BUCK` with reindeer fixups per crate (cedar-policy, sqlx, anyhow…), a repo-pinned `tools/buck2` launcher — and `scripts/dev-up.mjs` drives it (`BUCK2_BIN`, `buck2Version()`, `"buck2"` process mode, `dev:doctor` check). The polyglot rationale did die with the frontend, but the **remaining** one is decisive for us: `target/` lock contention across parallel worktrees is exactly the agent-fan-out coordination problem, and buck2 solves it |
| Build acceleration | **sccache** + `cargo-nextest` + profile tuning — **complementary to buck2, not a replacement** | still true that the repo has no `.cargo/config.toml` and no `[profile]` section, so cargo paths (per-crate `cargo check -p`, test runs) remain untuned and worth fixing. buck2 covers the parallel-worktree build; cargo tuning covers the inner loop |
| Test runner | `cargo-nextest` (already installed) | per-test process isolation may *fix* the `XX000` parallelism flake rather than working around it with `--test-threads=1` |
| Leptos isolation | frontend crate sits behind the contracts crate and stays **out of default workspace members** until shell work starts | confines 0.9-beta breaking changes so they cannot reach backend lanes |

### Worktree hygiene (blocking coordination defect)

**653 git worktrees currently exist; 293 registered under `/private/tmp`, which holds 890 directories.**
Backend is **128 crates across 31 domain groups** (an earlier figure of 189 counted sub-crate
directories, not crates). Bun ran their entire 64-agent
operation on **four**. Unbounded worktrees cause disk exhaustion (this repo has already lost dev
Postgres to 707 orphaned Docker volumes) and silently kill background jobs when paths move — which
happened during the `maintenance`→`console` rename this session. **A fixed, small worktree pool with
mandatory teardown is a precondition for fan-out**, not a cleanup task.

## 0.6 Design-then-expansion split (the core Bun lesson)

Bun's 64 agents were safe because every one of the 1,448 files had three things we do **not** have:
a known-correct reference (the `.zig` original), an immutable target (a language-independent TS suite
never rewritten), and a mechanical rule set (`PORTING.md`). Agents could not design the wrong thing
because they were not designing — Sumner's instruction was explicitly *"do the rewrite that looks like
we transpiled our Zig code to Rust."*

Pointing that machinery at **design** work produces N designs and a merge nightmare. So the project
splits:

| Phase | Character | Model |
|---|---|---|
| **Design** — substrate + OrgUnit end to end | serial, small, high-judgment | *not* Bun. One implementer, deep review |
| **Expansion** — Position, Person, Employment, PayRun, adapters | mechanical, repetitive | full Bun: worktree pool, crate-sharded queue, adversarial diff-only review |

The design phase's job is to **manufacture Bun's three preconditions**:

| Bun had | We build |
|---|---|
| the `.zig` reference | **OrgUnit, built end to end by hand, exceptionally well** — every later domain transliterates from it, and reviewers check fidelity *against it* |
| immutable TS test suite | **conformance suite** at the use-case boundary, owned outside the lanes |
| `PORTING.md` | **`CATALOG.md`** — turns "add a domain type" into transliteration, not design |

Adopted verbatim from Bun, both hard-learned: **`git stash` / `git reset` are banned** (commit or
abandon; atomic per-file commits), and **"edit the process, not the outputs"** — when a lane produces
bad work, fix the prompt and rerun rather than hand-patching the diff.

**Deliberate divergence:** Bun rejected incremental because shim code hurts, and could afford big-bang
*because the work was mechanical and verifiable*. Ours is neither, so we invert their order —
incremental through design, big-bang through expansion.

## 1. Principles

1. **The engine is a quarry asset, not a rewrite target.** Tested code that solves a hard problem is
   kept. "Reimagine" is licence to change composition and substrate, not to relitigate solved parts.
2. **Manufacture an immutable target before parallelizing.** (See §2 driver 1 — this is where Bun's
   model does *not* transfer for free.)
3. **Crates are the shard boundary.** `cargo check -p <crate>` is the natural work queue; Rust gives
   us the file-level independence Bun got from Zig's lexical scoping.
4. **Deterministic or manual — no AI/LLM judgment.** Same input = same output, rule named in the audit.
   This is also the deliberate differentiator against Foundry's AIP.
5. **Scope is load-bearing.** Org, employee, HR, payroll. "That's it." Every scope addition is a
   decision, not a drift.

## 2. Decision drivers

1. **We have no immutable verification target, and Bun did.** Bun's whole concurrency model rested on
   a TypeScript test suite that was language-independent and never rewritten — 64 agents could aim at
   a fixed goal. **Our tests are not immutable: changing the substrate changes them.** This is the
   sharpest break in the analogy and the top risk. Mitigation is §3 P2.
2. **The collision surface just collapsed.** Deleting the frontend removed `ko.ts`, three generated
   clients, and three API-drift gates. Remaining collisions: `openapi.yaml` (one file, `include_str!`
   at `app/src/lib.rs:187` and `app/tests/openapi_drift.rs:6`), the single global migration sequence
   (`platform/db/migrations`, highest `0168`), and the type-registry seed. Parallelism is far cheaper
   than it was a week ago.
3. **The substrate decision gates the schedule.** If current state is mutable rows with history
   alongside, event-log-first is a rewrite that touches every adapter. If it is already derived,
   it is an addition. Nothing downstream can be sized until this is answered with evidence.

## 3. Prep phase (Bun's "3 hours + PR #30224") — SERIAL, before any fan-out

### P0 · Repo coherence (blocking, mechanical)
Fix before anything else so lanes have a green baseline: drop `check:api-drift:*`, `gen:api:*`,
`check:ts|kotlin|swift`, `web:*` and the ios/android gates; keep the backend/infra gates
(`check:adrs`, `check:k8s`, `check:platform-contract-drift`, migration-safety).

Verified 2026-07-28: `package.json` has already been cleaned (no `workspaces` key, no frontend
scripts) and `ci.yml` no longer references deleted trees. **Residual:** `image-release.yml` still
filters on `clients/**` and `web/**` and still resolves an `mnt-web` image digest. There is no
`release.yml`; release automation is `release-please.yml`.

### P1 · `CATALOG.md` — the PORTING.md analogue
The translation guide every lane shares: for each company building block, its engine type definition —
typed props, link types with cardinality, actions (writeback), derived analytics, lifecycle states.
Covers: OrgUnit, Position, Person, Employment, PayRun (+ the C-/OP-/PR- demand sources if retained).
Written once, referenced by all lanes; prevents five lanes inventing five shapes of "Person".

### P2 · The immutable target — `company-conformance` suite (**the load-bearing prep item**)
Bun aimed 64 agents at a test suite that could not move. We must build ours deliberately:

- A **black-box scenario suite** driven purely through the public REST surface — no internal types, no
  direct DB access — so it survives a substrate change unaltered.
- Scenario: found a company → create org units → define positions → hire people → transfer one →
  run a pay cycle → reconstruct the org as-of a past date → prove a non-privileged principal cannot
  see rows outside policy.
- This suite is written **before** the substrate work and is not edited by implementation lanes.
  A lane that wants to change it must escalate — that is the whole point.

### ~~P3 · `SUBSTRATE.tsv`~~ — **DROPPED**
Its purpose was to size a substrate migration. The substrate already exists (§0 SUBSTRATE RESOLVED),
so there is no migration to size. Removed rather than left as busywork.

### P3 (replaces the above) · Build topology — cargo and buck2 are layered, not rival

Verified 2026-07-28. Root `BUCK` states it outright: *"Rust targets are generated from the current
Cargo workspace."*

| Layer | Role |
|---|---|
| `backend/Cargo.toml` (`resolver = "3"`) | **source of truth** — members, dependency versions; drives rust-analyzer, `cargo check -p`, `cargo test` |
| **168 per-crate `BUCK` files** under `backend/` (incl. `ci/gates/*`) | buck2 targets, generated from the Cargo workspace |
| `third-party/rust/{buckify.sh, reindeer.toml}` | reindeer vendors third-party crates into `third-party/rust/BUCK` |

**Cost model — and why it reinforces the catalog decision:**

| Action | Cost |
|---|---|
| Add a domain type as an **engine type** | data in the registry — **0 crates, 0 BUCK files, 0 reindeer runs** |
| Add a domain type as a **bespoke crate** | Cargo member entry + per-crate `BUCK` + workspace-load fragility |
| Add a third-party dependency | reindeer regen of `third-party/rust/BUCK` — a serialization point |

Choosing engine types over bespoke crates was an ontology decision; it turns out to be the cheap
option for build reasons too. **Bonus from the frontend deletion:** root `BUCK` previously carved out
*"Web, Android, and iOS retain their proven native build lanes"* — those are gone, so buck2's scope
collapses to pure Rust, removing the hybrid it was apologising for.

### P4 · Reservations (the decycling)
- **Migration blocks pre-reserved per lane** — one global sequence at `0168`, no per-crate namespace,
  and numbers have collided across lanes before. Non-negotiable.
- **`openapi.yaml`: reserved tag + contiguous path block per lane**, merge-queued. Do **not** split the
  file — it is `include_str!`'d into the binary and the drift test; splitting adds a
  generated-file-in-git failure mode for no gain.
- **`backend/Cargo.toml` `members`**: pre-reserve entries. An unmatched glob fails the build **for every
  lane**, so never create a crate directory without a valid `Cargo.toml` in the same change.
- **`third-party/rust/BUCK`**: reindeer-generated. **Serialize dependency additions** — one lane owns
  the regen; others request.
- **Type-registry seed**: one owner; other lanes emit type definitions as data, never edit the seed.

### P5 · Trial run before scale
One vertical — **hire a person into a position** — through the whole machine: 1 implementer,
2 adversarial reviewers (diff only, told to assume it is wrong), 1 fixer. It exercises registry +
instance store + event log + effective dating + action dispatch + Cedar authorize/residual + audit +
as-of in a single slice. Fan-out is not authorized until it passes P2's suite.

---

## 4. Parallel execution model (from Bun)

| Bun mechanism | Ours |
|---|---|
| 4 worktrees × 16 agents = 64 | worktree per lane; concurrency capped by review throughput, not agent count |
| Errors grouped **by crate**, not file (anti-fragmentation) | `cargo check -p <crate>`; one crate active per lane, next activates when clean |
| Tests sharded by folder | `#[sqlx::test]` suites sharded by crate, `--test-threads=1` (known XX000 parallelism flake) |
| Atomic single-file commits; **`git stash`/`reset` banned** | same — Bun had to edit the workflow to forbid these |
| Reviewers get diff only, no implementer reasoning | 1 implementer + 2 adversarial reviewers + 1 fixer per lane |
| 3 consolidation points | C1 contracts+migrations landed · C2 workspace compiles + conformance green · C3 catalog complete, company demonstrable |

**Local discipline:** the main session cannot run `cargo`; subagents can — compile-verify in a subagent
before push. Disposable Postgres per run, always `--rm` (707 orphaned volumes once filled the VM),
assertions as the non-superuser runtime role (superuser BYPASSRLS masks broken RLS).

**Progress tracking:** each lane reports against P2's scenario list, not against its own subjective
"done". A lane is complete when its slice of the conformance suite is green — nothing else counts.

## 5. Pre-mortem (deliberate mode)

**S1 — The conformance suite gets edited to fit the implementation.** A lane hits friction, "fixes" the
scenario, and the immutable target silently becomes mutable. Everything after that is unfalsifiable.
*Mitigation:* suite is owned outside the lanes; changes require escalation; CI diffs it separately.

**S2 — Substrate rewrite metastasizes.** Event-log-first turns out to touch every adapter; lanes half-
migrate; the tree sits broken for weeks with neither model working. *Mitigation:* P3's migration-class
column sizes this before fan-out; if the class is "rewrite" for more than a threshold of aggregates,
the substrate becomes its own serialized phase rather than a per-lane concern.

**S3 — Scope reopens.** "Building blocks of a company" quietly grows an ERP module because the engine
makes it easy. This system has already been reframed three times (FSM → conglomerate platform →
console carbon-copy → engine); the failure mode is not wrong architecture, it is scope outrunning
delivery. *Mitigation:* the four-domain boundary is an explicit gate; additions require a decision,
and the conformance suite does not grow to accommodate them.

## 6. Test plan (deliberate mode)

- **Conformance (immutable):** the P2 black-box company scenario. The definition of done.
- **Unit:** per-crate Rust tests; domain logic pure and DB-free where possible.
- **Integration:** `#[sqlx::test]` against disposable Postgres, asserted **as the runtime role**;
  RLS org-isolation proven on every new table; `--test-threads=1`.
- **Authorization:** Cedar conformance — every policy either lowers to a SQL residual or fails
  **closed** with a named untranslatable term. Negative tests: principal cannot see out-of-policy rows.
- **Temporal:** as-of reconstruction diffed against the event log; effective-dated future changes do
  not leak into present reads.
- **Contract:** `openapi_drift` (kept); `check:platform-contract-drift`.
- **Determinism:** same input = same output for every automated decision, with the rule string captured
  in the audit event (§4-28/§4-38).

## 7. Acceptance criteria

1. Repo installs and CI is green after P0.
2. `company-conformance` suite exists, is owned outside the lanes, and fails meaningfully before the
   work starts.
3. Hire-a-person vertical passes the suite, reviewed by two adversarial diff-only reviewers.
4. Org unit / Position / Person / Employment / PayRun exist as engine types per `CATALOG.md` — no
   bespoke tables.
5. As-of reconstruction of the org chart at an arbitrary past date matches the event log.
6. A non-privileged principal provably cannot read out-of-policy rows (residual, not client filtering).
7. `mnt` / `mnt_rt` naming retired; no live-infra dual-alias migration needed.

## 8. ADR

- **Decision:** Keep the ontology engine and Cedar authz; add an event-log substrate beneath them;
  build the default catalog and compose the hire-a-person vertical; rename off `mnt`.
- **Drivers:** no immutable verification target exists yet (must be manufactured); collision surface
  collapsed with the frontend deletion; substrate class gates all sizing.
- **Alternatives:** full engine rewrite — discards 22k LOC including the residual filter, the hardest
  solved problem; compose-and-rename only — ships sooner but leaves as-of/lineage on a substrate that
  was not designed for them.
- **Consequences:** two models coexist during substrate migration; the conformance suite is a hard
  dependency on all parallel work; scope boundary must be actively defended.
- **Follow-ups:** shell rebuild against the engine as final acceptance; Korean statutory ruleset as
  the expressiveness proof; cross-worktree `target/` lock contention addressed with sccache +
  per-lane `CARGO_TARGET_DIR` per §0.5 — **not** buck2, which is dropped now that every non-Rust
  surface is deleted.
