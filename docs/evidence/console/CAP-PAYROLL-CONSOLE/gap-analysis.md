# CAP-PAYROLL-CONSOLE — Gap Analysis (backend vs STORY-PAYROLL-001)

> Existing surface read completely: `backend/crates/payroll/{domain,adapter-postgres,rest}`,
> migration `0074_create_payroll_readiness.sql`, authz `Feature::PayrollRunRead`,
> `/api/v1/period-locks` (PeriodLockManage), inbox vault (`console-inbox-rest`,
> `GET /api/v1/me/inbox-docs?filter=kind:payslip`). Latest migration on this branch: 0180;
> this lane's provisional number: **0186**.

## 1. What already exists (do not rebuild)

| Asset | Where | Notes |
|---|---|---|
| Readiness tables `payroll_draft_runs` / `payroll_draft_lines` (+`annual_leave_obligations`) | mig 0074 | RLS forced, org_id only (no branch_id), run FSM CHECK `STAGED/BLOCKED_LEGAL_GATE/READY_FOR_REVIEW/APPROVED/ISSUED/VOID`, `calculation_enabled` requires review states, lines store counts + `*_source_present` booleans + `nts_tax_row_status` + `blockers` — **no won amounts by design** |
| Read-only REST | `console-payroll-rest` | `GET /api/v1/payroll/runs` (list, audited `payroll_run.list_read`), `GET /runs/{id}` (audited `payroll_run.read`, miss not audited), `GET /payslips/me` (self-scoped readiness lines, never audited, empty page for unlinked accounts). Org-wide gate `authorize_org_wide(PayrollRunRead)`: built-in EXECUTIVE/SUPER_ADMIN only; ADMIN (any scope) denied; branch-scoped denied rather than widened |
| Domain kernel (W1) | `console-payroll-domain` | 2026 statutory rates (NPS/NHIS/LTC/EI, industrial-accident = external tariff, refuses to guess), pension base limits (effective-dated), minimum wage, `build_employee_payroll_draft` (requires supplied NTS tax row — **refuses to estimate income tax**), `build_severance_pay_draft` (average-vs-ordinary wage floor, fail-loud), `validate_release_gate` (rate-table version + official sources + professionally-validated golden cases + 노무사/세무사 artifact sha256) |
| Freeze windows | `/api/v1/period-locks` | domain=payroll lock/unlock, close-authority gated, date-stamped writes inside a locked window 409 — the §3.9.1 변경 동결창 |
| Payslip delivery vault | `console-inbox-rest` | Personal inbox docs, `kind: payslip`, `confirmed{by,at}` ack, passkey gate for legal docs, self-view non-audited |
| RLS test discipline | `payroll_rls_surfaces_as_runtime_role.rs` | as-`console_rt` pattern to copy for new tables |

## 2. Story-step gap table

| # | Story step | Exists | Gap (to close in this charter) | Owning crate decision |
|---|---|---|---|---|
| 1 | **Close gate (attendance-complete)** | Period locks; attendance records + readiness counters live in HR crate; run has no close state | Run-level close: `POST /runs/{id}/close-attendance` with §4-29 preflight receipt (auto checks: unresolved attendance exceptions = 0, per-entity close complete, active payroll period-lock covering the period; soft warn: pending leave; human attest). Persist receipt + `ATTENDANCE_CLOSED` status. Readback of blocking items with fix-links (ids of unresolved AT- exceptions) | `payroll` (adapter reads attendance/HR tables read-only in-tx, org-scoped; no new cross-crate API) |
| 2 | **Calculation trigger + status** | Kernel math is pure/in-memory; nothing persists a calculation; `calculation_enabled` flag exists | `POST /runs/{id}/calculate`: per line, compute a **draft** deduction breakdown ONLY where `gross_pay_source_present && nts_tax_row_status='VERIFIED_SOURCE_ROW'`; other lines stay uncalculated with truthful `blockers[]`. Persist versioned results in new `payroll_line_calculations` (0074 lines stay readiness-only). Run status `CALCULATING→CALCULATED` (or stays with per-line blockers). **HARD RULE honored**: no fabricated statutory output; a calculation is always labeled draft (`payable=false`) until the release gate (§1) is satisfied — the L3 gap below | `payroll` |
| 3 | **Exception queue + resolution** | Nothing | New `payroll_run_exceptions` table + typed taxonomy (OVERTIME_ALLOWANCE / RETRO_ADJUSTMENT / ABSENCE_DEDUCTION / PRORATION / ACCOUNT_VERIFICATION — the prototype's 연장수당·소급·결근 공제·일할·계좌), linked object refs, `POST .../exceptions/{id}/resolve {action: CONFIRM\|HOLD, reason}` (hold = carry to next run), audited. Exceptions are produced by the calculate step from real line deltas/blockers, never invented | `payroll` |
| 4 | **Approval routing** | `approved_by/at` columns; generic AP- workflow engine is a separate charter | `POST /runs/{id}/submit` (fail-closed: CALCULATED + all exceptions resolved), `POST /runs/{id}/withdraw` (rejected runs), `POST /runs/{id}/decision {APPROVE\|REJECT, reason}` with **SoD: decider ≠ submitter** (maker-checker), all audited. Run statuses `SUBMITTED→APPROVED\|REJECTED`. Linking to the real AP- engine object is recorded as `approval_ref` (nullable) — full engine integration deferred, documented L3 link gap | `payroll` (self-contained SoD approval; AP- engine wiring later) |
| 5 | **Disbursement scheduling** | Nothing | New `payroll_disbursements`: `POST /runs/{id}/schedule-disbursement {scheduled_at}` (guard: APPROVED), status `SCHEDULED→SUBMITTED_TO_BANK→PAID\|FAILED` advanced only by **operator-attested** transitions (audited, reason on FAILED). **No bank integration exists — statuses are truthful operator records, never simulated bank results** (L3: real transfer file/bank API) | `payroll` |
| 6 | **Payslip delivery + ack readback** | Inbox vault delivers payslip docs with `confirmed{by,at}`; nothing links runs→docs | `POST /runs/{id}/issue-payslips` — **hard-gated by `validate_release_gate`** (until a professional-validation artifact is registered this endpoint 409s `legal_gate`; truthful, no fake payslips). New `payroll_payslip_deliveries` linking run line → inbox doc; `GET /runs/{id}/payslip-delivery` aggregates delivered/acknowledged counts + per-line ack state (readback from inbox `confirmed`) | `payroll` writes docs via the inbox store crate (same DB, in-tx) |

## 3. Cross-cutting gaps

- **Authz**: new mutations need a write-tier feature `PayrollRunManage` (mirror
  `PayrollRunRead`'s row/gating exactly: org-wide `authorize_org_wide`, built-in
  EXECUTIVE/SUPER_ADMIN, ADMIN only via custom org-wide PBAC grant, branch-scoped denied).
  `Feature` enum lives in `backend/crates/platform/authz` — shared root; entry recorded in
  `integration-manifest.json`, not edited here.
- **Deny-by-omission**: run/line/exception reads already indistinguishable cross-org via
  RLS (404 == not-yours). New tables must repeat the 0074 RLS block + `console_rt` grants +
  as-runtime-role tests (BYPASSRLS masking hazard).
- **Audit**: every admin read stays audited (`with_audits` atomic pattern); every mutation
  audits actor/action/target/reason; decision + close receipts persist as JSONB evidence.
- **OpenAPI/clients**: 8 new operations must land in `backend/openapi/openapi.yaml` with a
  per-domain `tags: [payroll]` (Kotlin client OOM rule) + regenerated ts/kotlin/swift —
  integrator-owned, manifest entry provided.
- **Frontend**: `payroll` nav item exists but screen is unmounted; module build needs
  `MOUNTED_SCREEN_KEYS` + `SCREEN_REGISTRY` + i18n nav keys (manifest). Console remains
  DARK per ADR-0025 (`EXPOSED_SCREEN_KEYS` currently `["sales"]`); payroll ships mounted,
  not exposed.

## 4. L3 gaps (documented, intentionally out of scope — truthful states instead)

1. **Payable statutory calculation**: gated by `validate_release_gate` (licensed 노무사/세무사
   golden-case artifact). Until registered: calculations are draft-labeled, issuance 409s.
2. **NTS 간이세액표 ingestion**: lines without `VERIFIED_SOURCE_ROW` stay blocked with
   `blockers[]` — the API never estimates income tax.
3. **Bank transfer integration**: disbursement statuses are operator attestations.
4. **AP- workflow-engine object linkage**: SoD approval is run-local; engine `approval_ref`
   reserved.
5. **Retro/소급 recalculation engine & multi-entity consolidation**: exception types carry
   the taxonomy; automated retro math is future work (BENCHMARK honest-gap column).
