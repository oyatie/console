# CAP-EVALUATION-CONSOLE — backend verification (stage 3, fresh-eyes adversarial)

Date: 2026-07-24 · Verifier: independent stage-3 lane (did not author the code)
Scope: commits `07d3fb9c..52210fb4` on `claude/console-evaluation-backend-20260724`
plus three verification-pass commits (dead-code completion, audit-readback
widening, rustfmt normalization of two unformatted cross-lane mnt-app tests —
the latter verified token-identical modulo whitespace/trailing-commas).

Verdict: **PASS** — all claims in the build report were re-verified against the
actual code and re-executed tests. Two issues found and fixed during
verification (both minor, neither behavioral). Accepted deviations and carried
open items listed at the end.

## Method

Every file under `backend/crates/evaluation/**`, migration
`0190_create_evaluation.sql`, the `platform/authz` diff (`32738575`), the
integration test `backend/app/tests/evaluation_cycle_api.rs`, and both
cross-lane repair commits were read in full. All gates were re-run locally
(not trusted from the build report):

| Gate | Result |
| --- | --- |
| `cargo fmt --check` (4 evaluation crates + mnt-app test) | clean |
| `cargo clippy --all-targets -- -D warnings` (4 evaluation crates) | clean (after fix 1 below) |
| `cargo test -p mnt-evaluation-domain` | 5/5 |
| `cargo test -p mnt-evaluation-application` | 1/1 |
| `cargo test -p mnt-app --test evaluation_cycle_api` | 2/2 (real scratch DBs, full migration chain, HTTP through the mounted router as `SET ROLE mnt_rt`) |
| placeholder scan (`TODO/FIXME/unimplemented!/todo!/#[ignore]/.skip`) | zero hits in lane files |
| migration version collision scan | none (0180 → 0185 → 0190, no duplicates) |

## Findings fixed during verification

1. **Uncommitted dead-code deletion left the tree clippy-red.** The worktree
   carried an uncommitted deletion of the unreferenced
   `PgEvaluationError::kind()` helper which stranded an unused `ErrorKind`
   import — `clippy -D warnings` failed on the working tree (HEAD itself was
   green). Completed the deletion (removed the import), re-ran clippy: clean.
2. **Audit readback covered 4 of 7 mutation classes.** The story test asserted
   readback for cycle-lifecycle events, finalizations, calibrations, and
   submissions but not `evaluation.subject.added` / `evaluation.goals.replaced`
   / `evaluation.review.saved`. Widened the readback block to assert exact
   counts for all three (2 / 2 / 5), which also proves rejected requests
   (409/422) write no audit rows. Re-ran: green.

## Checklist evidence

**FORCE RLS + org policy on every new table.** All six tables
(`evaluation_cycles`, `evaluation_subjects`, `evaluation_goals`,
`evaluation_reviews`, `evaluation_evidence_links`,
`evaluation_code_counters`) get `ENABLE` + `FORCE ROW LEVEL SECURITY` and the
`org_isolation` policy (USING + WITH CHECK on
`NULLIF(current_setting('app.current_org', true), '')::uuid` — NULL compares
false, so an unarmed GUC fails closed). Org-immutability triggers on the three
UPDATE-bearing lifecycle tables; goals/evidence rows are replace-set objects
and the counter's org_id is its PK — WITH CHECK blocks re-tenanting on all of
them regardless. `mnt_rt` grants omit DELETE on all lifecycle objects.

**Test runs as mnt_rt, count-leak-free isolation.** The router pool is built
with `after_connect → SET ROLE mnt_rt` (NOSUPERUSER, NOBYPASSRLS); every HTTP
request in both tests executes as the runtime role. Cross-tenant proof: a
foreign-org ADMIN gets `total: 0` from the list endpoint (count leak), 404 on
direct cycle/subject/ledger fetches, and a raw unarmed `SELECT count(*)` on
the mnt_rt pool returns 0 while the owner pool sees the row.

**Deny-by-default authz.** Three new features registered in `feature_catalog`
and the compile-time matrix (`EvaluationRead`/`Submit` = ADMIN + EXECUTIVE +
SUPER_ADMIN, `EvaluationManage` = ADMIN + SUPER_ADMIN — the
EmployeeDirectory HR tiers); matrix exhaustiveness test updated 80 → 83 and
re-run. The submit-only path in the test is granted through the declarative
policy-role map (custom role + assignment), proving the grant-only path. An
ungranted MEMBER gets 403 with the canonical envelope on every route.
Per-subject deny-by-omission: a Submit holder who is not the assigned manager
gets 404 (read and write), never 403 — no existence leak. Unknown review kind
in the path answers 404, not a 500 or enum leak.

**Audit per mutation with readback.** Every mutation runs inside
`with_audits` (GUC armed before the closure, audit rows inserted in the same
transaction, rollback on error). Readback now asserts exact counts for all
seven mutation actions plus the audited person-ledger read
(`evaluation.history.viewed`, target `employee`) — 5 cycle-lifecycle events,
2 `subject.finalized`, 2 `subject.calibrated`, 3 `review.submitted`,
2 `subject.added`, 2 `goals.replaced`, 5 `review.saved`, 1 `history.viewed`.

**Fail-closed gates.** Preflight is recomputed inside the transition
transaction after `FOR UPDATE` on the cycle row; the GET report is advisory
only. Proven: open blocked on subjects-without-goals (2 blockers → 409),
start-calibration blocked on a missing manager review (blocker) while the
missing self review stays advisory, finalize blocked on an uncalibrated
subject. Submit gates: grade required (409), manager review requires ≥1
evidence link (409). Calibration: four-eyes SoD (evaluator ≠ calibrator, 409)
and reason-required-on-grade-change (409). Drafts lock once calibration
starts (409).

**Replay / write-race safety (no idempotency keys, by design).** Every
mutation takes `FOR UPDATE` on its governing rows before its guard:
transitions lock the cycle; review/goal/calibration writes lock the subject
*and* its cycle via the join lock, so a concurrent stage transition cannot
slip between guard and write; RV issuance locks the per-org counter row;
`add_subject` serializes on the cycle lock, so the duplicate-enrollment check
cannot race its own UNIQUE constraint. Replays answer 409 (re-open, resubmit,
draft-after-submit — all proven). RV codes issued deterministically
(RV-2500, RV-2501; counter readback 2502, exactly once per subject).

**Design-contract fidelity.** All 16 routes of design-contract.md §3 are
implemented with the specified feature gates, stage guards, status-code
semantics, and DTO field names (wire shapes additionally pinned by the
application-crate serde test: `SELF`/`MANAGER`, `YYYY-MM-DD` dates,
SCREAMING_SNAKE stages). Derived subject chip state (no stored column),
progress-by-unit aggregates from employee `org_unit`, default list excluding
ARCHIVED, my-tasks = OPEN cycles × assigned subjects × missing/draft reviews,
ledger = finalized entries only + audited read — all match §1/§3/§4 and are
exercised in the story test. The openapi fragment carries per-operation
`tags: [evaluation]` (kotlin OOM guard) and `EVALUATION_ROUTE_PATHS` is
exported for the drift gate.

**Rejection-class sweep.** Canonical `{error:{code,message}}` envelope on
every handler-produced error, kind-based code strings exactly like the
sibling crates; DB errors never leak sqlx strings or constraint names
(blanket `internal` mapping). No runtime-assembled SQL (one literal statement
per transition). No N+1: list/detail/task queries are single statements with
correlated subqueries or EXISTS; per-review evidence loads are bounded at 2
reviews per subject. 422 bounds validated in-handler before any DB CHECK can
fire (proven for name length and weight_pct).

## Accepted deviations (documented, not defects)

- **Blocked transitions return 409 with the joined blocker messages**, not
  the structured report the contract sketch ("409 + report") implies. The
  canonical error envelope is `{error:{code,message}}` repo-wide; the
  structured checklist is served by `GET /preflight`, which the §4-29
  checklist UI consumes. Frontend lane should read blockers from the
  preflight endpoint, not the 409 body.
- **Axum extractor rejections** (malformed JSON, duplicate/invalid query
  params, non-UUID path segments) produce axum's default rejection bodies,
  not the canonical envelope — identical to every sibling console module
  (sales, facilities, …), which use the same bare `Json`/`Query` extractors.
  A platform-wide rejection mapper would be the right fix; out of lane scope.

## Open items (carried / for the integrator)

1. Cross-lane repair commits `07d3fb9c` (0170→0185 migration renumber) and
   `aa181e45` (platform/auth jwt.rs, logistics adapter, facilities rest,
   production rest compile fixes) touch files outside this lane's ownership.
   Verified minimal and behavior-preserving; owning lanes may supersede.
2. Dark landing: mount `mnt_evaluation_rest::router` in `build_router` AND
   merge `manifests/openapi-fragment.yaml` into `backend/openapi/openapi.yaml`
   in the same change (drift gate couples them), then regenerate
   `clients/{ts,kotlin,swift}`. Migration 0190 renumber-at-consolidation if
   slots collide (inventory lane claims 0191).
3. SELF reviews are recorded by the assigned manager (`evaluator_user_id` =
   recorder) — employee self-service needs a user↔employee linkage that does
   not exist yet (carried substrate item).
4. No `Idempotency-Key` header on evaluation mutations — replay safety is
   FSM-guard + row-lock + UNIQUE-constraint based (see above); integrator
   parity review vs the logistics-style keyed endpoints stands.
5. Branch-wide `clippy -D warnings` is still not green outside this lane
   (e.g. mnt-app lib `dead_code`, facilities unused import); only lane-owned
   or lane-repaired crates were brought to green.
