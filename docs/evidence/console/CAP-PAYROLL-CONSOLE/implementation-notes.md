# CAP-PAYROLL-CONSOLE — Backend build notes (stage 2)

Companion to `design-contract.md` (the contract) and `gap-analysis.md` (the
gap table). This records the build decisions where the contract met the real
schema, plus the honest-gap register. Everything here is verifiable in code;
nothing below is design intent.

## 1. What was built

| Piece | Where |
|---|---|
| Migration 0186 (provisional) | `backend/crates/platform/db/migrations/0186_payroll_run_lifecycle.sql` — run FSM widen (+ the 0074 `calculation_enabled` CHECK widen the contract draft missed), SoD CHECK, 4 new tables, RLS FORCE + `mnt_rt` grants, no DELETE grants |
| Domain | `mnt-payroll-domain::build_line_calculation` + `VerifiedNtsTaxRow` (owned-string tax row for source-materialized figures); shared internals extracted from `build_employee_payroll_draft`, byte-identical amounts proven by test |
| Adapter | `mnt-payroll-adapter-postgres::lifecycle` — all in-tx lifecycle functions; `PayrollRunSummary/Detail` extended per contract §3 |
| REST | `mnt-payroll-rest::lifecycle` — 12 routes under `/api/v1/payroll/runs/{id}/*`; writes gated by `Feature::PayrollRunManage` (org-wide, mirror of `PayrollRunRead`); typed 409 codes; `{error:{code,message,details?}}` envelope |
| Tests | `payroll_lifecycle_rls_as_runtime_role.rs` (adapter, as `mnt_rt`); `backend/crates/payroll/rest/tests/run_lifecycle_api.rs` (full HTTP story + denial/cross-tenant/audit readback, served on an `mnt_rt` pool) |

No `backend/app/src` change was needed: the existing payroll router mount
picks the new routes up from `PAYROLL_ROUTE_PATHS`.

## 2. Contract-meets-schema decisions

1. **Calculation inputs** — the 0074 readiness rows deliberately store no won
   amounts, and no table stores NTS tax-row values. The only evidence-bearing
   store is the immutable import ledger the lines already link to
   (`payroll_draft_lines.source_data_import_row_ids` →
   `data_import_rows.canonical_row`). The calculate step therefore reads the
   figures **verbatim** from `canonical_row.payroll`:
   `monthly_gross_pay_won`, optional `pension_standard_monthly_income_won`,
   and `nts_tax_row.{table_version, monthly_income_tax_won,
   local_income_tax_won}`. A line whose flags promise a verified source but
   whose linked rows lack these keys gets blocker
   `SOURCE_AMOUNTS_NOT_MATERIALIZED` — never an estimate. This key contract is
   the ingestion charter's write-side target.
2. **Release-gate registration** — `validate_release_gate` needs a registered
   record; no dedicated table exists, and 0074 already reserved
   `payroll_draft_runs.legal_basis` for exactly this class of evidence. The
   record lives at `legal_basis.release_gate` (shape in
   `payroll_run_api.rs::register_release_gate`). Until an ops/console path
   writes it, `issue-payslips` 409s `legal_gate` — fail-closed, truthful.
3. **Close preflight checks** — mapped to what real tables can prove:
   `attendance_material` (lines with zero attendance source rows AND zero
   attendance events; blocking refs = line ids), `period_lock` (active
   `period_locks` row, domain=payroll, covering the whole period),
   `pending_leave` (soft warn; pending `leave_requests` overlapping the
   period). The prototype's `entities_closed` check has **no backing table**
   (no per-entity close object exists) and was NOT fabricated; it joins the
   honest-gap register below.
4. **Closeable states** — real 0074 rows sit in
   `STAGED/BLOCKED_LEGAL_GATE/READY_FOR_REVIEW`; all three may close, else
   live data could never enter the pipeline.
5. **Exception generation** — only from signals that exist: overtime hours on
   the roster line (→ `OVERTIME_ALLOWANCE`, warn, `amount_delta_won = NULL`
   because no verified source derives the delta) and HELD carry-forward from
   the previous run of the same `source_label` series. The other three
   taxonomy kinds are schema- and DTO-complete but are only produced when
   their source signals exist (see gaps).
6. **Idempotency** — every lifecycle mutation is guarded by `SELECT … FOR
   UPDATE` + a status guard, so a replay is a typed 409 rather than a second
   effect; `schedule-disbursement` additionally has the `run_id UNIQUE`
   constraint, and payslip issuance is re-drivable (inbox emits are
   dedup-keyed `payroll-run:{run}:line:{line}`, delivery links insert `ON
   CONFLICT DO NOTHING`). Matching the facilities pilot's client
   `Idempotency-Key` was deliberately not duplicated: that pattern exists for
   *creation* races on caller-named resources; every route here mutates one
   already-identified run. (`ponytail:` state-machine idempotency; add a
   client key only if a POST ever creates a caller-named resource.)
7. **Payslip acknowledgement** — `inbox_docs` CHECK
   (`inbox_docs_only_legal_confirmed`) forbids receipt confirmation on
   `kind='payslip'`, so `acknowledged_at` is truthfully `NULL` and
   `acknowledged` counts 0 until the vault grows a payslip-ack capability
   (inbox charter). The readback path is wired and live the moment it does.

## 3. Honest-gap register (unchanged L3 gaps + build-scoped additions)

1. Payable statutory calculation — stored rows stay `payable=false`; flip is
   the release-gate charter.
2. NTS 간이세액표 ingestion — nothing writes `canonical_row.payroll` yet in
   production data; until then every real line blocks truthfully.
3. Bank transfer integration — operator attestations only.
4. AP- engine linkage — `approval_ref` reserved, SoD approval is run-local.
5. Retro/소급·결근·일할·계좌 exception generation — taxonomy supported,
   producers pending their source signals (attendance retro engine, bank
   account store).
6. `entities_closed` preflight check — needs a per-entity close object.
7. Payslip ack — vault-side capability (see §2.7).

## 4. Spine repairs made in-lane (not payroll scope; flagged to integrator)

- `jwt.rs` two call sites missing the new `actor_home_org` argument (branch
  did not compile) — commit `86dfe855`.
- duplicate migration version 0170 (financial vs ontology lanes) aborting
  every `#[sqlx::test]` — financial index renumbered to 0181, commit
  `8d9cde06`.
- `mnt-logistics-adapter-postgres` `format!("{:x}", …)` on a sha2 digest that
  no longer implements `LowerHex` (blocked building `mnt-app` tests) — repo
  idiom `hex::encode` applied.
- **NOT repaired** (out of scope, actively-refactored foreign lanes):
  `mnt-facilities-rest` and `mnt-production-rest` do not compile at branch
  HEAD, so `mnt-app` (and every `backend/app/tests/*` suite) cannot build.
  The chartered `backend/app/tests/payroll_run_api.rs` therefore lives at
  `backend/crates/payroll/rest/tests/run_lifecycle_api.rs` against the exact
  router + `with_request_context` middleware the app mounts verbatim —
  identical coverage, runnable proof. Integrator may relocate it to
  `app/tests/` once those lanes heal (only the harness `app()` fn differs).

## 5. Verification

- `cargo test -p mnt-payroll-domain` — 11 green (incl. parity of
  `build_line_calculation` with the draft builder).
- `cargo test -p mnt-payroll-adapter-postgres` — RLS-as-`mnt_rt` suites green
  (0074 surfaces + new lifecycle tables; cross-org reads empty, WITH CHECK
  write rejected, cross-org mutation NotFound).
- `cargo test -p mnt-payroll-rest` — unit + HTTP suites green.
- `cargo test -p mnt-payroll-rest --test run_lifecycle_api` — full lifecycle
  story on the crate router as `mnt_rt` (ES256 chain, preflight fail-closed →
  close → calculate with real kernel figures (3,000,000 → net 2,626,698) →
  exception fail-closed submit → SoD → reject/withdraw/resubmit/approve →
  disbursement FSM → legal-gate 409 → gated issuance with vault dedup →
  audit readback), plus PBAC denial without leakage + cross-tenant
  invisibility + denied-probe no-side-effect proof.
- `cargo clippy -p mnt-payroll-{domain,adapter-postgres,rest} -- -D warnings`
  clean; `cargo fmt` applied.
