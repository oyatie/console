# D4 resolved — the console rebuild charter, and the two ADRs it amends

> `Status: PENDING APPROVAL — owner decisions captured 2026-07-30; ADR numbers assigned by the integrator.`
>
> Supersedes the D4 draft in `docs/ideas/adr-adjudication.md`, which scoped D4 as an ADR-0025 amendment
> alone. Two findings enlarged it, both verified here:
>
> 1. **Leptos has no accepted-ADR home.** Zero hits for `Leptos` across `docs/decisions/`. It exists only
>    in `docs/PIVOT-2026-07-28.md` §3, which is not in `docs/decisions/` and therefore binds nothing
>    (`docs/decisions/README.md:1-2`, `:10`). This charter is where the stack decision becomes
>    authoritative.
> 2. **`ADR-0009` collides independently**, and the adjudication missed it because T12 examined gates
>    rather than the contract model. Its Decision mandates *"CI generates ts/swift/kotlin clients and
>    fails on drift (T1.9); both apps build from every release tag (T1.8 dual-build gate) … per-slice
>    sequencing ships web+Android first, iOS immediately after."* Android, iOS and the generated clients
>    are all deleted from `HEAD`.

## The decision, in one paragraph

The console rebuild is **authorized to exist and to be planned, and forbidden to be implemented** until
the ontology engine substrate is proven. The stack is **Leptos**. The internal contract is a **shared
Rust contracts crate**, not a generated client. ADR-0025's structural prescriptions — carbon-copy visual
authority, the `web/src/console/**` path and two-shell composition, and the spine boundary as enumerated
— **do not survive the pivot and are replaced**; its §7 nine-item evidence bar **does** survive and
governs every future console screen. The console's shape is to be derived from the product north star,
not inherited from the carbon-copy premise.

## Owner decisions captured

| # | Question | Decision |
|---|---|---|
| 1 | How does a Rust frontend get its contract? | **Shared Rust types via a contracts crate.** `openapi.yaml` remains the external contract; the internal one is the type system, so drift is a compile error rather than a CI diff. |
| 2 | What of ADR-0025's shape is rejected? | **The carbon-copy visual authority, the path and shell structure, and the spine boundary as enumerated** — all three. Plus a directive: design the console's best possible shape from the north star. **§7's nine-item evidence bar was not rejected and is retained.** |
| 3 | What does "backend first" gate on? | **The ontology engine, not the verticals.** |
| 4 | What is authorized now? | **Charter and deep planning only. No implementation.** |

## A1 — amends ADR-0025

**Reciprocal record owed.** `ADR-0025-carbon-copy-console-shared-platform-spine.md` frontmatter gains
`amended_by: [ADR-<A1>]`; it currently carries `amends: [ADR-0023]` and
`related: [ADR-0009, ADR-0018, ADR-0021, ADR-0022, ADR-0023]` with no `amended_by`, so this key is
created. Its index row becomes `accepted, amended`. Pre-acceptance this draft carries
`proposes_amendments_to: [ADR-0025]` and **must not** declare active `amends` (`README.md:26`).

**What is withdrawn, and why each fails rather than merely ages:**

- **The carbon-copy visual authority** — `:233`'s "one carbon-copy visual system" end state and `:70-75`'s
  prohibition on inheriting the legacy `AppShell`/shadcn styling. Both name a React design system that is
  absent from `HEAD`. A prohibition against inheriting from a deleted tree constrains nothing, and an end
  state defined as a copy of a deleted surface is unreachable.
- **The path and shell structure** — `web/src/console/**` and the two-shell composition this ADR amended
  into ADR-0023. A Leptos app is a workspace crate, so the ownership boundary must be stated in crate
  terms. The *intent* — one owner per surface, no forking of shared machinery — is retained and restated.
- **The spine boundary as enumerated** — `:59-68`'s split of shared-nonvisual from console-owned-visual.
  Its rows "generated OpenAPI types and the single typed client", "frontend policy-decision adapters" and
  "the i18n corpus and string gates" are stack-specific and describe machinery that no longer exists. The
  *principle* — the console owns its surface and reuses the platform rather than forking it — is retained.

**What survives unchanged: §7's nine-item evidence bar** (`:125-145`). A screen counts as shipped only
with a reachable mounted body per exposed navigation state; real backend reads and mutations through the
shared contract; source-object drill-through and human-safe identifiers; server-side authorization plus
fail-closed client behaviour; required audit events and atomicity; loading, empty, denied, stale,
partial-failure and full-failure behaviour; persona-based real-backend E2E; fidelity, accessibility,
performance and console-error gates; and explicit legacy-parity coverage or an owner-approved deferral.
Incomplete navigation entries stay hidden or classified DARK and are never counted as product breadth.

This bar is the reason the amendment is narrow. Everything withdrawn is a prescription about *how* the
console is built; the bar constrains *what counts as built*, and it is the strongest defence in the
record against shipping shells. **Retaining it is what makes withdrawing the rest safe.**

**Replacement shape — to be designed, not decided here.** The console's structure is to be derived from
the product north star: an omni business-operation platform expressing a company as if it were a game
engine, intuitive surface over uncompromised depth, manageable without developers as far as honestly
possible. That design is the deep-planning work this charter authorizes and does not pre-empt.

## A2 — amends ADR-0009

**Reciprocal record owed.** `ADR-0009` gains `amended_by: [ADR-<A2>]`; index row becomes
`accepted, amended`. ADR-0012 (monorepo, "OpenAPI contract + generated clients version atomically with
consumers") gains `related` only — its atomic-versioning claim is satisfied more strongly by a compiled
dependency than by codegen, so it needs no amendment.

**Withdrawn:** ts/swift/kotlin client generation and its drift gate (T1.9); the dual-build gate requiring
both apps to build from every release tag (T1.8); and the per-slice sequencing that ships web+Android
first with iOS immediately after.

**This withdrawal is retroactive bookkeeping, not a removal — verified.** The `gen:api`, `check:ts`,
`check:kotlin` and `check:swift` scripts are already absent from `package.json`, and `android/`, `ios/`
and `clients/` are absent from `HEAD`. So the code has already diverged from the ADR, which
`README.md:12` classifies as a **governance gap, not silent supersession** — reconcilable only through a
new decision. A2 is that decision. Same class as D3's ADR-0002 correction: the record asserts machinery
that does not exist, and a reciprocal key alone would leave the false sentences standing, so **the
Decision text must be edited in place.**

**A judgement for the integrator, flagged rather than decided.** ADR-0009 is titled *"Dual-native
(Swift+Kotlin) parity strategy via single OpenAPI contract + CI parity gate."* Withdrawing the
dual-native half leaves a title describing a strategy that no longer exists. An amendment is the smaller
edit and keeps the retained contract principle in its original home; a supersede would restate that
principle in a new record and retire this one whole. **Recommendation: amend**, because the retained half
is load-bearing and cited elsewhere, and note the title's staleness in the amendment itself rather than
leaving a future reader to trip on it.

**Retained:** one utoipa-emitted `openapi.yaml` as the **single external contract**, and the principle
that parity is enforced structurally rather than by discipline.

**Added:** the internal contract is a **shared Rust contracts crate** re-exporting the request and
response types the handlers use. Structural enforcement moves from a generated-client diff to the
compiler: a breaking backend change breaks the frontend build immediately.

**The cost, stated rather than discovered.** This couples the frontend build to backend crates. That
coupling is the mechanism, not a side effect — but it means a backend change cannot land while the
frontend is red, which is a real constraint on the landing model and interacts with
`docs/ideas/lane-assembly-line.md`'s consolidation-branch shape. `PIVOT-2026-07-28.md:43` already
anticipates the isolation: Leptos lives behind a contracts crate and stays out of default workspace
members until shell work begins. That isolation is retained here as a condition, not an implementation
detail.

**Consequence for `openapi.yaml`:** it stops being the frontend's source and becomes an external
deliverable. It must therefore keep its own gate proving it still describes the served surface, since the
frontend will no longer fail when it drifts. `backend/app/tests/openapi_drift.rs` is the existing
instrument; confirm it covers this before the ts gate is deleted, or the deletion removes the only
consumer that noticed.

## The gate — what "the engine is proven" means

Owner decision 3 gates frontend implementation on the **ontology engine substrate**, not on the
HR/payroll/org verticals. Stated as checkable conditions rather than a judgement, each already measured
or named by an executed experiment:

| Condition | Status | Evidence |
|---|---|---|
| An authored type's rows are readable — policy attachment works through an audited path | **in flight** | X2 CONFIRMED a published type lists `200 OK []` until a policy is attached (`docs/ideas/experiment-x1-x2.md`); migration 0206 is the fix, in lane-1 |
| An authored relationship produces edges | **CONFIRMED, with a rule** | X1: a link type alone writes ZERO edges; **no reachable path** writes one without a property `config.link` |
| Actions on a projected type do not require a hand-written Rust closure per action | **OPEN** | `ProjectedDispatchRegistry` is a `Default`-empty `HashMap` failing closed on `NotWiredYet`; the no-code-reachable domain-write count is **0 of 15** |
| A read model exists for aggregate queries | **OPEN** | zero `CREATE MATERIALIZED VIEW` in all 205 migrations; `ont_analytics` is a formula registry with nowhere to put results |
| A real dry run exists | **OPEN** | `preflight` never calls `apply_edits`, so it reports `would_execute: true` for invalid edits |

**The risk this choice carries, recorded because it was flagged when the choice was offered.** Gating on
the engine rather than the verticals starts the frontend against a substrate whose entity model is under
active revision — the Critic returned **ITERATE, `implementation_ready: false`** with twelve blocking
items on it. So "proven" must mean *these five conditions measured green*, never "the plan was approved."
Three of the five are open, and two of those (projected dispatch, the read model) are the same defects the
Critic's B6 raised. **The gate is currently closed, and that is the correct state.**

## What this charter authorizes, and what it forbids

**Authorized now:** the rebuild's existence; Leptos as the stack; the contracts-crate contract model;
design and planning work of any depth, including the north-star console shape, information architecture,
and the surface inventory.

**Forbidden until the gate opens:** implementation. No mounted shell, no route, no component, no
navigation entry — the anti-stub rule ADR-0023 already states (*"no stubs or placeholders — when the
backend can't realize the design, the slice builds the backend"*) applies with the sequencing inverted:
the backend is built first by decision, not discovered as a blocker.

**The existing CI gate that asserts the console frontend does not exist therefore STAYS.** It is not an
obstacle to retire; under this charter it is the enforcement mechanism for "planning only", and it must be
retired deliberately, in the change that opens the gate, with a justified commit. Whoever opens it should
read `docs/ideas/experiment-results.md` first: X9 established the four-link CI wiring template, and buck2
is fully functional (X8), so the wiring is understood rather than guessed.

## Open questions this charter does not resolve

- **Does the contracts crate expose domain types or dedicated DTOs?** Re-exporting domain types couples
  the frontend to internal representations and would make an internal refactor a frontend break; dedicated
  DTOs cost a mapping layer. `ADR-0001`'s compiler-enforced clean architecture has an opinion here and it
  should be consulted before the crate is designed.
- **SSR with islands, or CSR?** `PIVOT-2026-07-28.md:38` records SSR with islands / selective hydration
  and `:41` `leptos_axum` against the existing axum 0.8.9. Neither is ratified by an accepted ADR, and
  §7's nine-item bar — particularly persona-based real-backend E2E and the console-error gate — means
  different instruments for each. This belongs in the planning work, not in the charter.
- **Leptos 0.9 is beta** (`PIVOT:38`). No ADR states what happens if it does not reach stable before the
  gate opens. Worth an explicit fallback position rather than an assumption.
- **The i18n corpus and string gates** were withdrawn with the spine enumeration but not replaced. Korean
  is the product's primary language and `scripts/check-i18n.mjs` is absent from `HEAD`. The replacement is
  planning work, but the *absence* of any string gate should be recorded as a known gap meanwhile.
