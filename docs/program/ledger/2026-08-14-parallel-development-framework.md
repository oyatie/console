# Authority tip — parallel development framework lane

**Date:** 2026-08-14
**Kind:** authority tip ledger bound on candidate C; T adds this ledger entry only
**Base:** `dea1f91bf6c336319fc718f9d9f3eb2c2047f63c` (origin/main)
**Candidate C and Tip T:** both signed by the pinned authority (principal `jason19931225@gmail.com`, ED25519 `SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8`). C's, T's, and the squash S commit/tree SHAs are recorded in the post-merge readback ledger update: T cannot name its own SHA (its bytes are the last to freeze) and C's custody manifest names T's blob, so the identities exist only once the merge completes — recorded then, per DELIVERY.md's readback step.
**Review:** 13 `chatgpt-codex-connector` review threads (3 P1, 10 P2) reviewed 2026-08-14 and folded into this candidate; owner (Jason Lee) signs off via merge. No review finding was dismissed without a fix or an explicit recorded gap.
**Scope:** the framework lane for the remaining Console program: `docs/specs/parallel-development-framework.md` (quarry), its custody registration, the `.grok/harness/work-graph.v1.json` process extension, and the Wave A1 stale-stream measurement.
**Not product authority.** Clears no HOLD and authorizes no production, credential, compliance, payment, erase, OCI, frontend, projection, or Oyatie action.

## Summary

- Added a quarry-marked framework spec codifying the remaining-program wave plan (A: P4 stale-stream reconciliation; B: P5 canonical containment; C: CI residuals; D: 일정 projection; E: P6 §7 tracker; P∞), Bun-derived cargo-budget and git-discipline rules, the PRODUCT-boundary scope filter, failure modes, and acceptance criteria.
- Registered the spec in `docs/documentation-manifest.seed.json`; regenerated `docs/documentation-index.json`. Evidence: `node scripts/console/generate-documentation-manifest.mjs --check` → "documentation manifest OK (442 markdown files)"; `git diff --check` clean.
- Wave A1 measurement (recorded on bead `console-mrqv`): origin/main `dea1f91bf6c336319fc718f9d9f3eb2c2047f63c` is 35 commits ahead of the stale stream's merge-base `ede052d3d`; two-dot delta of the branch = 75 files (+18868/−245); the stale ledger doc is already on main under the same path. The checkout's true uncommitted residue is 6 tracked files (`backend/Cargo.lock`, the writer-ownership gate `lib.rs`, migrations 0213–0215) plus untracked custody candidates; per-file residue review is the Wave A owner's task under clean worktree resolution. **Decision direction: preserve-then-reconcile** — the stale checkout's exact bytes stay untouched as historical evidence; reconciliation proceeds in a separate bounded worktree; no destructive step is prescribed. Re-implement `console-1qw.4`/`console-1qw.5` fresh on origin/main.
- Verification recorded (doc-authority + CI-contract candidate, DELIVERY.md:25-27): `check:doc-links` OK (442 md) + doc-link tests pass; ADR gate 39 ADRs / 6 design notes + ADR tests pass; `check:doc-citations` MISSING 0; foundation gate pass (its tests run inside `check:ci-preflight`); `check:ci-preflight` gate + its 9 test files pass; `test:verify` pass; reasoning-lens tests pass; `check:doc-manifest` OK (442 md); `check:reasoning-lens-contract` OK (60 blocks); truth-ledger `STRUCTURALLY_VALID_HOLD_PRESERVED` on a synthetic merge; `npm run verify` green except the 6 Buck2 suites that are not runnable locally (no Buck2 toolchain on this workstation — they run in CI on the PR); `git diff --check` clean. Executed counts: see the per-run outputs recorded in the lane admission record.
- Fixed a CI defect surfaced by this PR: the backend job ran its `Path-class skip proof` step before `Checkout` while inheriting the job's `working-directory: backend` default, so every docs-only PR failed at job launch (bash could not start because `backend/` did not exist yet). The fix makes `Checkout` unconditional and moves it before the proof step, with the matching backend contract amendment in `scripts/check-ci-preflight.mjs` (`actionStep(0, "Checkout", ...)`).
- Process extension: 8 nodes + 5 edges added to the local `.grok/harness/work-graph.v1.json` (41 nodes / 46 edges, validated), plus Beads epics/children: `console-mrqv`, `console-wrq3` (+4 lanes), `console-7thr`, `console-2qf5`, `console-j3x7`, `console-sv28`.
- Incident (tracked as `console-8sgr`): the bd post-checkout hook wrote a stray `core.worktree` into the shared `.git/config` and clobbered 7 uncommitted files in the `lane-full-console-g002-s0b` worktree; shared config corrected; g002's committed work (`24ac6715c`, local-only) is intact.

## Remaining HOLDs / follow-ups

- All current PRODUCT/ROADMAP HOLDs remain unchanged (frontend, projection fan-out, live production, Korea compliance, OCI A1 instance, disk wipe).
- Framework PR requires human review and merge via the protected path; post-merge containment readback pending.
- Wave A destructive steps (discarding the stale checkout state) require the human owner's decision.
- `console-1qw.4`/`console-1qw.5` remain open; P4 epic close blocks on them; P5 epic `console-dgo` closes after P4.

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "implementation",
  "risk_class": "high",
  "risk_domains": [
    "release"
  ],
  "selected_lenses": [
    "Cartesian doubt",
    "Essentialism / YAGNI",
    "Chesterton's Fence",
    "Red Team",
    "Systems Thinking",
    "Operability / Day-2",
    "Blast-radius / cell-based",
    "Shared-nothing / eventual consistency",
    "Zero-trust / defense-in-depth"
  ],
  "task_fit": {
    "Cartesian doubt": "Separated the measured object facts (35-commit gap, two-dot delta of 75 files) from worktree-resolution artifacts caused by the stray shared core.worktree, and re-derived every claim from clean measurements.",
    "Essentialism / YAGNI": "Filtered the 66 open beads through the PRODUCT boundary so legacy attendance/comms/ERP/HA items stay quarry instead of being dispatched.",
    "Chesterton's Fence": "Layered Bun's cargo and git rules onto the existing cell-parallel/tip-serial machinery instead of replacing the lane-packet/admit/review gates that already caught past false-greens.",
    "Red Team": "Modeled the failure classes that recurred here (false-green gates, scope drift, unsigned tips) and the bd post-checkout hook incident as an adversary's path to clobber another lane.",
    "Systems Thinking": "Traced wave dependencies (A before B/D, C serial on the CI root) and the tip-serial mutex across ledger, lockfiles, openapi, and ci.yml.",
    "Operability / Day-2": "Recorded rollback, detection, stop conditions, and a P∞ babysit bead so the framework degrades to a visible queue rather than silent drift.",
    "Blast-radius / cell-based": "Kept every artifact additive and cell-local: one quarry spec, one in-repo work-graph, additive beads; no product code or migrations touched, and the CI fix is scoped to reordering one checkout step plus its one contract entry.",
    "Shared-nothing / eventual consistency": "Lanes stay path-disjoint with base-SHA pinning and contracts-first integration; counters move only when commits land.",
    "Zero-trust / defense-in-depth": "The candidate modifies the Required CI workflow and its executable preflight contract — the boundary that decides what runs and what counts as proof. The fix is the minimal reordering (unconditional Checkout before the skip proof) with the contract amended to match, and the verifier still pins the trusted signer, the pinned digest, and the fail-closed step semantics; no check is weakened or removed."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "The p4/canonical-ports-writer-ownership checkout is a stale partially-superseded stream: origin/main is 35 commits ahead of the merge-base and already carries the writer-ownership gate, canonical hardening, and the openapi fragment modularization under different subjects.",
    "The bd post-checkout hook wrote core.worktree into the shared .git/config and clobbered 7 uncommitted files in the lane-full-console-g002-s0b worktree; its committed work (24ac6715c) is local-only and intact.",
    "The backend job's docs-only skip-proof step ran before Checkout under the job's working-directory: backend default, failing every docs-only PR at job launch (PR #773 was the first to hit it after #770 landed).",
    "Independent review (chatgpt-codex-connector) found 13 threads: shortened base SHA, stale ADR-0030 §7 status (all six conditions are MET on main via #755; the shell restriction is the console-8nq delivery HOLD), CI mislabeled as authority, Wave D communications scope, lane-startup aggregate cargo, narrower-than-implemented tip-serial inventory, non-reproducible work-graph, over-claimed allowlist enforcement, missing Wave D migration lens record, missing Zero-trust lens on the CI boundary, missing exact identities in this ledger, and discard wording risking evidence loss. All were folded into this candidate."
  ],
  "decisions_changed_or_rejected": [
    "Rejected rebasing the stale stream commit-by-commit; measured the two-dot delta instead and adopted preserve-then-reconcile with fresh re-implementation of console-1qw.4/1qw.5 on origin/main.",
    "Rejected adding sccache for lanes until a disk census justifies it; Cargo stays on the CI rust-cache path.",
    "Rejected dismissing the bot review: every thread was either folded into the spec/ledger or recorded as an explicit L∞-PROC gap."
  ],
  "lens_set_changes": [
    "Zero-trust / defense-in-depth added after review found the CI-contract boundary change contradicted the earlier not-applicable exception."
  ]
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
