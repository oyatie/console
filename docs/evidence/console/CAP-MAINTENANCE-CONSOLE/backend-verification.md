# CAP-MAINTENANCE-CONSOLE — Backend Stage-3 Adversarial Verification

Fresh-eyes verification of the stage-2 backend build (commits `0cf1662c..e0c85121`),
performed against the actual code, not the build report. Date: 2026-07-24.
Verifier changes: one test-gap closure + one manifest fix (committed on top).

## Verdict

GO. Every claim in the build report checked out against the code and the running
system. One end-to-end proof gap (one-live-settlement 409 via the partial unique
index) was found and closed; one contradictory integrator instruction in
`integration-manifest.json` was fixed. No stubs, placeholders, TODO/FIXME,
skipped tests, dead controls, or fabricated calculations anywhere in the lane.

## What was verified, with evidence

### 1. RLS — FORCE + org policy on every new table
`0193_workorder_maintenance_console.sql` arms BOTH `work_order_settlements` and
`work_order_settlement_lines` with `ENABLE`+`FORCE ROW LEVEL SECURITY`, an
`org_isolation` policy (`USING` + `WITH CHECK` on `app.current_org`), and the
`enforce_org_id_immutable` trigger — the same arming as `work_orders`. Composite
FKs `(x, org_id)` pin every reference intra-tenant. Grants are minimal:
settlements SELECT/INSERT/UPDATE, lines SELECT/INSERT only (immutable once
written, no DELETE anywhere — VOID is the correction state).

### 2. The integration test runs as the real runtime role
`backend/app/tests/maintenance_chain_api.rs` builds the router on a second pool
whose `after_connect` runs `SET ROLE mnt_rt`; the sqlx-owner pool is used ONLY
for seeding and audit readback. All four tests passed against scratch DBs
(migrations include 0193): `cargo test -p mnt-app --test maintenance_chain_api`
→ 4/4 ok (7.4s).

### 3. Tenant isolation without leakage (count-leak-free)
Test `pbac_denies_and_cross_tenant_reads_are_isolated_without_leakage`:
- outsider (other org, ADMIN) GET order / GET settlement / POST settlement → 404,
  never 403 — foreign objects do not exist for another tenant;
- outsider list → 200 with `total: 0`, empty items (no count leak);
- MEMBER (in-tenant, no features) → 403 with canonical `{error:{message}}`
  envelope and no object fields.

### 4. Deny-by-default authorization
- `Feature::CompletionReview` matrix row `[D,D,D,A,D,A]` — ADMIN/SUPER_ADMIN
  only; MEMBER is deny-everywhere except Login.
- Create: reviewer path (CompletionReview) OR mechanic path (WorkReportSubmit
  AND an assignment on that exact order, re-checked inside the audited
  transaction). Submit: creator (with WorkReportSubmit) or reviewer. Review:
  CompletionReview. Void: CompletionReview AND a built-in admin-tier role
  (`Admin|SuperAdmin|Executive`) so a custom completion_review grant alone can
  never erase a financial record. All proven by 403 assertions in the tests.

### 5. Audit event per mutation, atomically, with readback
`with_audit` (platform/db/audit_tx.rs) arms `app.current_org`, runs the mutation
closure, inserts the audit row, and commits in ONE transaction; the error path
rolls back both. create/submit/review/void each build a
`work_order_settlement.*` event (`.with_org(org)` always applied; review/void
carry before-snapshot + decision/comment/reason payloads). Readback proof:
exact per-action counts asserted from the owner pool, including replay-does-not-
double-count.

### 6. Fail-closed gates
- Settlement eligibility: order must be in REPORT_SUBMITTED/ADMIN_REVIEW/
  FINAL_COMPLETED, checked on a `FOR UPDATE`-locked order row → premature create
  asserted 409.
- RETURNED requires a non-empty comment → 422 (whitespace-only rejected).
- Void requires a non-empty reason → 422; mechanic void → 403.
- Missing/short `Idempotency-Key` → 422 before any DB write (mirrors the DB
  CHECK `char_length >= 16`, so it can never 500 at the constraint).
- Final-completion evidence interlock and approval-line guards unchanged.

### 7. Idempotency replay returns the stored outcome
Replay check reads by `(org-scoped) idempotency_key` BEFORE the audited
transaction: identical `request_hash` + same work order → the existing
settlement (asserted same id, audit count still 1); different body under the
same key → 409. The race window (two first-time creates with one key) falls
through to the `UNIQUE (org_id, idempotency_key)` index, whose 23505 maps to
Conflict/409 with a generic message (constraint name logged server-side only).

### 8. Terminal-state write races
Every settlement transition locks the row `FOR UPDATE`, then validates the pure
domain edge table (`SETTLEMENT_TRANSITIONS`); APPROVED and VOID are terminal —
`settlement_fsm.rs` proves all 16 terminal edges reject. Concurrent
approve-vs-void serializes on the row lock and the loser gets InvalidTransition
→ 409.

### 9. Design-contract fidelity (design-contract.md §§1-4)
Enums, DDL (implemented stronger than the contract draft: composite tenant FKs,
idempotency columns + CHECK, `btrim` label check), settlement FSM edges, all
five routes, DTO shape (rfc3339 timestamps, snake_case), error mapping
(422/403/409/404 + `{error:{message}}`), stepper support statuses, and detail
`settlement` field all match. The backend supports exactly what the prototype
simulates for classification chips, cost settlement (정산 → 전표), asset
history (`equipment_id` filter), and lens KPIs.

### 10. No fabricated calculations
- `preventive_on_time_rate`: AVG over closed (FINAL_COMPLETED) preventive
  orders with a target AND a real status-history completion timestamp; NULL
  (never 0/100) when no basis — asserted null-before-basis, then 1.0 after a
  genuinely on-time seeded close.
- `mttr_minutes`: mean of first-IN_PROGRESS → first-REPORT_SUBMITTED spans from
  `work_order_status_history`; NULL when no span — asserted.
- Facet buckets skip NULL classification (`IS NOT NULL`) — legacy rows are
  absent chips, never a bucket; totals come from real GROUP BY counts.
- `total_amount_krw` = checked_add of line amounts (overflow → 422), lines
  validated non-empty/non-negative/labelled.

### 11. Rejection-class checks
- Repeated-query parsing: `maintenance_type`/`maintenance_type[]` + comma
  expansion, uppercased then round-tripped through the enum (unknown value →
  422), identical to the status/priority pattern.
- N+1: settlements are fetched only on the detail view (1 header + 1 lines
  query); the list stays a single query; facets are bounded (LIMIT 16); the two
  new aggregates ride the existing single aggregate query via LATERAL MINs.
- Canonical envelope everywhere (asserted on denials).

### 12. Static gates
- `cargo fmt --check` — clean (workorder crates + mnt-app tests).
- `cargo clippy -p mnt-workorder-{domain,application,adapter-postgres,rest}
  --all-targets -- -D warnings` — clean (exit 0).
- Zero TODO/FIXME/unimplemented!/todo!/#[ignore] in the lane's files.

### 13. Full test matrix (all green, 2026-07-24)
| Suite | Result |
|---|---|
| domain unit + transitions | 3/3 |
| domain settlement_fsm | 5/5 |
| application | 3/3 |
| rest unit (incl. mobile_evidence, mobile_sync) | 16+ all ok |
| adapter m2_flag_off_parity | 1/1 |
| adapter rls_read_surfaces_as_runtime_role (mnt_rt) | 10/10 |
| adapter use_cases | 6/6 |
| app maintenance_chain_api (mnt_rt router) | 4/4 |

## Findings and dispositions

1. **[FIXED — test gap]** The one-live-settlement guarantee (second draft under
   a DIFFERENT idempotency key while a live settlement exists) was enforced only
   by the partial unique index and never exercised end-to-end — the existing
   409 assertions came from the request-hash and eligibility paths. Added a
   `duplicate_live` 409 assertion to
   `pbac_denies_and_cross_tenant_reads_are_isolated_without_leakage`; re-ran →
   4/4 green.
2. **[FIXED — contradictory integrator instruction]**
   `integration-manifest.json` `shared_root_requests.new_paths` said
   `tags: ["work-orders"]` while the shipped fragment (and the manifest's own
   `tag_arbitration` field) carries `tags: [maintenance]`. Aligned the entry to
   the fragment and pointed it at the arbitration note.
3. **[ACCEPTED]** `voucher_ref` is declared, selected, and documented but never
   written — intentional truthful NULL until the finance-gl lane posts real
   vouchers (contract §2); it is not fabricated data.
4. **[ACCEPTED]** A mechanic who drafted a settlement can read it back via the
   order detail (`WorkOrderReadAll` covers MECHANIC); MEMBER remains fully
   denied. No dead read path.
5. **[NOT A LANE DEFECT — restated]** The spine base does not compile without
   the five working-tree-only fixes (platform/auth, logistics adapter,
   production rest, facilities rest, duplicate migration 0170) documented in
   `integration-manifest.json`. These remain uncommitted and outside lane
   ownership; owning lanes/integrator must land real fixes. Full `-p mnt-app`
   clippy is still blocked by other crates' pre-existing `expect_used`
   violations at base.

## Honest open items (carried to the integrator)

- Renumber provisional migration 0193 to the next free number right before
  push; resolve the base 0170 duplicate on the spine.
- Merge `manifests/openapi-fragment.yaml` into `backend/openapi/openapi.yaml`,
  arbitrate the `maintenance` vs `work-orders` tag for the five new operations
  (either satisfies the per-domain client-tagging gate), regenerate
  clients/{ts,kotlin,swift}, and pass the three drift gates.
- G5 (parts-reservation PO/IV linkage) and G6 (SLO threshold config object)
  remain deferred to the inventory/erp and governance lanes by charter.
- Stage-3 frontend (`web/src/console/maintenance/**`) is a separate lane stage.
