# Wave-4 decisions (2026-07-25) — resolving the charter's §9 open list

## Founder-decided (asked and answered)

**D-0 · C-64 sequencing: DEPTH-FIRST WINS.** The authority's own post-190
directive stands; no waiver is written. Wave 4 = Phase-0 foundations + **CRM
(sales) taken to genuine production depth end-to-end**. The other 13 modules
receive ONLY their share of the Phase-0 shared fixes (window provider, code
grammar, a11y, catalog upgrade) — no per-module lanes. WMS then MES follow in
later waves. Target ~20 lanes, not 60.

**D-1 · Exposure: YES, one module to EXPOSED.** Wave 4 delivers the full
ADR-0025 evidence chain for sales (runtime proof, committed browser user-story
replay, a11y matrix, ops observation) so `EXPOSED_SCREEN_KEYS` gains one entry
*with its evidence committed*. The bar is not lowered to reach the milestone: if
the evidence does not hold, the entry does not land and we say so.

## Resolved by me (mechanical or already settled by evidence)

**D-2 · Epoch contract (charter blocker: "all 60 lanes formally inadmissible").**
The contract admitted only a rebase/cherry-pick train, but rebase IS
classifier-blocked on this spine. Amended: the integration protocol is
**plain `git merge` before push**, which is what every successful landing this
program has used (wave-1, wave-2/3, the openapi integration). Lanes are
admissible under the merge train. No lane may rebase.

**D-3 · Worktree budget: APPROVED.** 8 concurrent worktrees ≈150 GB; the volume
has ~3.0 TiB free. Not a constraint. Depth-first shrinks this anyway.

**D-4 · Window model (L-B0b 4-state provider).** Deferred as a platform
decision; wave 4 does the *narrower, verified* fix instead — L-F1 mounts
`WindowManagerProvider` in `ConsoleShell` (verified absent: it exists only in
the legacy `AppShell`, in tests, and as a nested provider in
`OntologyWorkspaceBody`), fixes the nested-provider bug, and closes the
single-pin data-loss path via a tray-restore contract. Whether to promote the
richer 4-state engine is re-decided once CRM has exercised the mounted one.

**D-5 · Dead code (projected_hours, SupportCase).** Default is DELETE, per the
no-stubs bar. A lane may keep either only by wiring it for real in the same
lane.

**D-6 · Statutory registry location.** Not on the CRM critical path; the
decision moves to the wave that takes payroll deep.

## Standing risk recorded (not scheduled this wave)

**R-1 · Payroll wage-law shallowness is a ship-blocker, not a live liability.**
연장 ×1.5 is gate-only, 야간/휴일 hours are dead columns, overlap is
structurally unrecoverable, 주휴수당 is absent, and the golden-case gate never
executes. This is criminal-exposure territory (임금체불) IF payroll ever runs
real money — but every payroll surface is DARK and nothing is exposed, so there
is no live exposure today. Binding consequence: **payroll must not be exposed,
and must not process real runs, until the L-D0/L-D4 wage-law lanes land.** That
pair opens the next wave.

**R-2 · Payroll timestamps serialize as tuples, not RFC 3339** (bare `time`
types + no `serde-human-readable`) — spec and wire disagree today on already
shipped payroll endpoints. Workspace-wide sweep needed; queued with R-1.
