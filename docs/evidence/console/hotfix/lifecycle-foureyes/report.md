# hf-lifecycle-foureyes — shared lifecycle maker-checker (SoD) guard

> Landed by the coordinator on the lane's behalf: the lane produced this content
> but was blocked from writing a report `.md`, and correctly did not route
> around the denial. Content is the lane's; the file is the missing artifact.

## The defect

`backend/crates/platform/db/src/lifecycle.rs::transition_lifecycle` accepted an
`actor: Option<Uuid>` and never compared it against the actor who made the
current state. Self-approval passed at the chokepoint **every domain routes
through**.

This is not a benefits defect. `backend/crates/benefit/rest/src/lib.rs:4-5`
documents that the module delegates transitions to the generic router *"so this
module cannot bypass four-eyes"* — it delegated in order to inherit a control
the router did not have. Every caller inherited the hole instead.

## Scope: derived, not assumed

Eight statements across DESIGN §3.9.1, §3.10 ③④, §3.9.3 and HANDOFF §15/§16
bind **the approver** to differ from the **기안자**. None requires two people on
every step. Counter-evidence against a blanket ban:

- §3.9.0 whitelist ③ — 초안 저장 is the author's own act.
- §3.9.0 whitelist ④ — 게시 needs the 게시 단계 권한 + audit, not a second gate.
- §3.9.2 — 승인 → 게시·발효 are sequenced distinct stages.
- DESIGN line 184 — the benefit chain carries 시행일/폐지일 **preset**, so
  `finalized→implemented` and `retiring→retired` are a date firing, not an
  approval.

`CHECKER_TRANSITIONS` therefore contains exactly the 승인 step of each chain:
`("document","submitted","approved")` and
`("benefit_catalog_item","pending","finalized")`. Everything else is
single-actor **by derivation, not omission** — including `draft→submitted`.
The constant's doc comment makes that an explicit obligation: adding an object
type to `lifecycle_transition_rules` means deciding which transition is its 승인.

The "maker" is the actor of the transition **into the current state**, keyed on
`to_state = current_state` rather than "newest row", so revision cycles resolve
correctly and two writes in one transaction cannot tie on `now()`.

## Red/green differential

Reverting the guard is structurally prevented here — a protection hook silently
restores the file, which produced a **false green** (cargo did not recompile,
0.33s). The lane took the differential from the test side instead, which is
stronger because both halves are real recorded runs of the same path:

| | Engine | Pre-fix chain walk (one unattributed actor through draft→submitted→approved→…) |
|---|---|---|
| Before | `4cabe239` | `lifecycle_walks_document_chain_and_gates_dispose` → **4 passed** — self-approval sailed through |
| After | `dfa31c1b` | same file restored verbatim → **FAILS** at `submitted→approved`: `Forbidden — 본인이 기안한 건은 승인할 수 없습니다` |

That test passed before *precisely because no control existed*.

Within-run control needing no revert: the **same actor** is refused on
`submitted→approved` and accepted on `approved→active` — same function, same
transaction shape; only `CHECKER_TRANSITIONS` membership differs.

## Vocabulary reuse

Mirrors the two existing `check_self_approval_tx` guards (workflow `:1873`,
financial `:2844`) exactly: byte-identical Korean 403, same
`anomaly.self_approval` detector, same `is_org_lead`/`SUPER_ADMIN` exemption,
recorded through the crate-local `upsert_open_finding_tx`. No new error kind, no
new dependency, no second dialect. The exemption is deliberate — without it a
one-person 법인 could never advance a document, and the chokepoint would
contradict its two siblings. Two fail-closed branches: no recorded 기안, and a
`None` actor approving a `None` 기안.

## Gates

`cargo fmt --check` clean · `cargo clippy -p mnt-platform-db --all-targets -- -D
warnings` clean, 0 warnings · suite 44/45.

The single failure,
`platform_force_migration_rejects_superuser_on_mnt_app_owned_database`
(`XX000 tuple concurrently updated`, `heapam.c:4509`), is a catalog race against
other lanes on the shared dev PG. It **passes in isolation** (verified) and
touches 0196 force-migration — unrelated to a change that creates no catalog
objects.

## Open question, not silently decided

**Should `archived→disposed` be a checker transition?** §3.9.1 is headed
«생성·변경·**폐지** 공통». Against: no source names a 기안자/승인자 pair for
폐지; disposal is already gated by legal hold + `retention_until`; and there is
no "disposal requested" state, so there is no prior maker to compare against —
the guard would either fail closed permanently or require inventing a state the
design has not specified. **Excluded.** If a `disposal_requested` state ever
lands it is a one-line `CHECKER_TRANSITIONS` entry, no guard change.

## Findings outside the lane's roots — recorded, not fixed

- **F-1 · `cedar_policy_ref` is dangling — HIGH.**
  `benefit_catalog_conditions.cedar_policy_ref` (migration 0157:116) is
  constrained only by `length ≤ 200`. Its **sole** consumer is a display SELECT
  at `benefit/adapter-postgres/src/lib.rs:1233` — never resolved, never
  evaluated, no FK. A benefit eligibility condition can name a non-existent
  Cedar policy and nothing notices: fail-**open** in a §3.10 ① authority gate.
  Cedar policy tables already exist to reference (0150/0154/0169/0170/0171); the
  fix is a referential check plus deciding that an unresolvable ref denies.
- **F-2 · `scripts/dev-up.mjs:110`** hardcodes the dev admin password as a
  fallback default. Pre-existing.
- **F-3 · Harness teardown** must prune its anonymous volumes — 707 of them
  filled the Docker VM disk mid-wave and took the shared Postgres down.
