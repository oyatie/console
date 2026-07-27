# CAP-PAYROLL-CONSOLE — Backend verification (stage 3, fresh-eyes adversarial)

Verifier: independent stage-3 pass, 2026-07-24. Everything below was checked
against the actual code/DDL/tests in this worktree — not against the stage-2
build report. One defect was found and fixed (§3); all other claims held.

## 1. What was verified, with evidence

| Claim | Verdict | Evidence |
|---|---|---|
| FORCE RLS + `org_isolation` (USING + WITH CHECK on `app.current_org`) on all 4 new tables | HOLDS | `0186_payroll_run_lifecycle.sql` §6 DO-block (ENABLE + FORCE + policy per table); grants are SELECT/INSERT/UPDATE only — no DELETE anywhere |
| RLS proven as the genuine `console_rt` runtime role, count-leak-free | HOLDS | `payroll_lifecycle_rls_as_runtime_role.rs` builds its pool with `SET ROLE console_rt` (`runtime_role_pool`), seeds org A rows via the owner pool, then under org B's GUC: exceptions page `None`, disbursement `None`, delivery `None`, `COUNT(*)` on `payroll_line_calculations` = 0, cross-org `close_attendance_in_tx` = `NotFound`, WITH CHECK insert of an org-A row rejected; org A sees its own rows |
| Deny-by-default authz | HOLDS | `Feature::PayrollRunManage` matrix row `[D,D,D,A,A,A]` identical to `PayrollRunRead`; all 12 lifecycle routes call `require_run_read`/`require_run_manage` via `authorize_org_wide` (branch-scoped and built-in ADMIN denied; unit tests in both `rest/src/lib.rs` and `rest/src/lifecycle.rs` assert every denial cell). HTTP test proves MEMBER/ADMIN get the same 403 whether the run exists or not (no existence oracle) |
| Audit event per mutation, atomic, with readback proof | HOLDS | every mutation runs inside `with_audits` (verified in `platform/db/audit_tx.rs`: org GUC armed before the closure, rollback on `Err`, audit rows inserted in the same tx before commit). HTTP test reads back all 9 mutation actions from `audit_events` and proves denied probes commit zero audit rows and zero state changes |
| Fail-closed gates | HOLDS | close 409s on preflight (`period_lock` missing proven in test); calculate blocks lines truthfully (`GROSS_PAY_SOURCE_MISSING` / `NTS_TAX_ROW_UNVERIFIED` / `SOURCE_AMOUNTS_NOT_MATERIALIZED` / `SOURCE_AMOUNTS_CONFLICTING`); income tax only ever read verbatim from the linked `data_import_rows.canonical_row.payroll` (`build_line_calculation` refuses without a verified row); submit 409s `exceptions_open`; SoD enforced in code AND by the `payroll_draft_runs_sod` DB CHECK; issue-payslips 409s `legal_gate` until `legal_basis.release_gate` passes `validate_release_gate` |
| Idempotency / replay behavior | HOLDS (state-machine, as documented) | every mutation takes the run `FOR UPDATE` + a status guard, so a replay is a typed 409, never a second effect; `payroll_disbursements.run_id UNIQUE` + `ON CONFLICT DO NOTHING`; payslip issuance is re-drivable (inbox `dedup_key` `payroll-run:{run}:line:{line}` returns the existing doc — verified in `inbox/adapter-postgres::emit_inbox_doc` — and delivery links insert `ON CONFLICT DO NOTHING`); HTTP test proves resolve-replay 409 `already_resolved`, issue-replay 409, and exactly 1 vault doc |
| Canonical error envelope | HOLDS | all handlers return `{error:{code,message,details?}}` via `RestError`; typed 409 codes `preflight_blocked` (+ checks details) / `invalid_state` / `exceptions_open` / `sod_violation` / `already_resolved` / `invalid_transition` / `legal_gate` all asserted over HTTP; DB errors log server-side and return opaque `internal` |
| No N+1 on list surfaces | HOLDS | runs list = 2 queries; run detail = fixed 8 queries regardless of line count (lines paged); exceptions page = 3 fixed queries; delivery page = 2 fixed queries. The per-line queries inside `calculate` are a locked batch mutation, not a list read |
| Terminal-state write races | HOLDS | ISSUED/PAID guarded via `FOR UPDATE` + status checks; concurrent double-issue is safe (step-3 tx re-locks the run, second caller 409s; step-2 emits are dedup-keyed so no double delivery) |
| No TODO/FIXME/unimplemented!/skipped tests/fabricated data | HOLDS | `grep -rn "TODO\|FIXME\|unimplemented!\|todo!\|\.skip\|#\[ignore\]" backend/crates/payroll/` → clean; every statutory figure carries an official source URL; `total_net_won` is `None` unless every line calculated; `acknowledged_at` truthfully `NULL` (the `inbox_docs_only_legal_confirmed` CHECK in 0119 really does forbid payslip confirmation) |
| Calculation math not fabricated | HOLDS | 4-insurance amounts derive from the in-crate 2026 rate tables (floor-won ppm, checked arithmetic, pension base clamped to the effective-dated limit); income/local tax verbatim from the verified NTS source row; `build_line_calculation` proven byte-identical to `build_employee_payroll_draft`; HTTP story asserts 3,000,000 → net 2,626,698 end-to-end |
| Design-contract fidelity | HOLDS for §2 (all 12 ops, methods, authz tiers, bodies, status codes, error codes match `design-contract.md` and `manifests/openapi-fragment.yaml`); deviations in §4 below | routes cross-checked one-by-one against `rest/src/lib.rs` path constants + the fragment |

## 2. Verification runs (this stage, fresh)

- `cargo fmt -p console-payroll-{domain,adapter-postgres,rest} --check` — clean.
- `cargo clippy -p console-payroll-{domain,adapter-postgres,rest} --all-targets -- -D warnings` (SQLX_OFFLINE=true) — clean.
- `cargo test -p console-payroll-domain -p console-payroll-adapter-postgres -p console-payroll-rest` against dev postgres 127.0.0.1:55432 (`#[sqlx::test]` scratch DBs) — 26 tests green after the fix (11 domain, 1+3 adapter RLS-as-`console_rt` + 1 new unit, 5 rest unit, 3 rest api, 2 run_lifecycle_api).

## 3. Finding fixed in this stage

**Nondeterministic source-figure selection (money path).** `calculate_run_in_tx`
extracted payroll figures with `find_map` over the line's linked
`data_import_rows.canonical_row` values, which are fetched with `WHERE id =
ANY($1)` — no ORDER BY, so with two payroll-bearing linked rows carrying
*different* figures the paid amount depended on arbitrary row order.
Fixed: `select_source_amounts` uses the figures only when every
payroll-bearing linked row agrees (re-import duplicates fine); differing sets
block the line truthfully with `SOURCE_AMOUNTS_CONFLICTING`. Unit-tested
(empty / non-payroll / duplicate / conflicting cases).

## 4. Adversarial notes that are NOT defects (verified intentional or exemplar-consistent)

- `payroll_line_calculations` append-only is enforced by code paths (INSERT-only
  writes, versioned) not by a DB trigger; the UPDATE grant exists solely for the
  future release-gate `payable` flip. Same posture as the 0074 tables.
- Partial payslip issuance: calculated lines whose employee has no linked user
  account are skipped (an inbox doc needs a recipient account); the delivery
  readback lists exactly which lines were delivered, and `issued` is the true
  count — nothing is claimed for skipped lines. All-lines-missing is a 409.
  Surfacing an explicit `undeliverable` list is queued as an open item.
- Query-extractor rejections (e.g. duplicated `?limit=`) return axum's plain
  400, not the JSON envelope — identical to every exemplar module; changing it
  only here would diverge.
- `calculate` returns 200 sync (contract explicitly allows "or 200 (CALCULATED
  sync)"); the transient CALCULATING state is real inside the transaction.
- VOID is in the widened status CHECK but no route produces it — the contract's
  §2 REST table defines no void op (§1's diagram mentions one); fail-closed
  until chartered, listed as an open item.
- Contract §3 defines a `LineCalculation` DTO but §2 defines no op returning
  per-line calculations; the implementation follows §2. The frontend roster
  card will need per-line amounts — open item for the frontend/integrator
  stage (extend `PayrollRunDetail.lines` or add a lines-calculations op).

## 5. Open items (carried + new, honest)

1. Integrator: merge `manifests/openapi-fragment.yaml` (12 ops) into
   `backend/openapi/openapi.yaml` + regenerate ts/kotlin/swift clients (3 drift
   gates); renumber migration 0186 at consolidation; reconcile the in-lane
   shared-root edits (`Feature::PayrollRunManage`, jwt.rs repair, logistics
   hex, 0170→0181 renumber) — all flagged in `integration-manifest.json`.
2. Pre-existing NOT repaired: `console-facilities-rest` / `console-production-rest` do
   not compile at branch HEAD → `console-app` unbuildable; relocate
   `run_lifecycle_api.rs` to `backend/app/tests/payroll_run_api.rs` once healed.
3. L3 honest gaps (implementation-notes §3): payable flip via release-gate
   charter; NTS 간이세액표/gross ingestion to `canonical_row.payroll`; bank API;
   AP- `approval_ref` linkage; RETRO/ABSENCE/PRORATION/ACCOUNT_VERIFICATION
   exception producers; `entities_closed` preflight check; payslip
   acknowledgement vault capability.
4. New (this stage): per-line calculation read surface for the frontend roster
   (§4 last note); explicit undeliverable-lines surfacing on payslip issuance;
   VOID route (admin + reason) if the design's §1 diagram is to be honored.
5. Frontend stage: `web/src/console/payroll/**` per design-contract §6;
   shared-root entries stay manifest items.
