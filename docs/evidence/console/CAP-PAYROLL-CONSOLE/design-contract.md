# CAP-PAYROLL-CONSOLE — Design Contract (API · DTO · FSM · DDL 0186)

> Buildable contract for both sides. Backend crate: `backend/crates/payroll/*` (extend, do
> not fork). Frontend module: `web/src/console/payroll/**` following the `production`
> exemplar exactly. Shared-root entries → `integration-manifest.json` (integrator-owned).

## 1. Run FSM (extends 0074 CHECK — one linear pipeline + legal gate overlay)

```
STAGED ──close-attendance──▶ ATTENDANCE_CLOSED ──calculate──▶ CALCULATING ──▶ CALCULATED
   CALCULATED ──submit (all exceptions resolved)──▶ SUBMITTED ──decision──▶ APPROVED │ REJECTED
   REJECTED ──withdraw──▶ CALCULATED            APPROVED ──schedule──▶ DISBURSEMENT_SCHEDULED
   DISBURSEMENT_SCHEDULED ──(operator attest PAID)──▶ PAID ──issue-payslips (release gate)──▶ ISSUED
   any pre-SUBMITTED ──▶ VOID (admin, reason)      BLOCKED_LEGAL_GATE / READY_FOR_REVIEW = legacy 0074 states, retained
```

Guards (all fail-closed, all audited):
- `close-attendance`: preflight receipt required — auto checks {attendance_exceptions_open=0,
  entities_closed=all, period_lock_active(domain=payroll, covering period)} + soft warns
  {pending_leave_requests} + `attest:true` by the caller. Blocking items are returned with
  object ids (fix-links).
- `calculate`: status=ATTENDANCE_CLOSED. Per line: computes only with
  `gross_pay_source_present && nts_tax_row_status='VERIFIED_SOURCE_ROW'`; otherwise line
  keeps `calculation_status='BLOCKED_LEGAL_GATE'` + `blockers[]`. Result rows are versioned
  and always `payable:false` until release-gate registration.
- `submit`: status=CALCULATED ∧ zero exceptions in OPEN.
- `decision`: status=SUBMITTED ∧ decider ≠ submitter (SoD, 409 `sod_violation`).
- `withdraw`: status=REJECTED → CALCULATED (exceptions stay resolved).
- `schedule-disbursement`: status=APPROVED.
- `issue-payslips`: status=PAID ∧ `validate_release_gate` passes for the registered
  release-gate record; else 409 `legal_gate`.

Exception FSM: `OPEN ─resolve(CONFIRM)→ CONFIRMED · ─resolve(HOLD)→ HELD` (held rows are
re-created as OPEN on the next run of the same series — carry-forward).
Disbursement FSM: `SCHEDULED → SUBMITTED_TO_BANK → PAID | FAILED` (operator-attested,
reason required on FAILED; FAILED → SCHEDULED retry allowed).

## 2. REST surface (all under `/api/v1/payroll`, tag `payroll`)

Existing (unchanged): `GET /runs`, `GET /runs/{id}`, `GET /payslips/me`.

| Op | Method/Path | Authz | Req body | 2xx resp | Errors |
|---|---|---|---|---|---|
| closeAttendance | POST `/runs/{id}/close-attendance` | PayrollRunManage (org-wide) | `{attest: true}` | 200 `PayrollRunDetail` | 401/403 · 404 · 409 `preflight_blocked` (body lists checks with `ok/warn/blocking_refs[]`) · 422 |
| getClosePreflight | GET `/runs/{id}/close-preflight` | PayrollRunRead | — | 200 `ClosePreflight` | 401/403/404 |
| calculateRun | POST `/runs/{id}/calculate` | PayrollRunManage | `{}` | 202 `PayrollRunDetail` (status CALCULATING) or 200 (CALCULATED sync) | 401/403/404 · 409 `invalid_state` |
| listExceptions | GET `/runs/{id}/exceptions` | PayrollRunRead (audited read) | — | 200 `ExceptionPage` | 401/403/404 |
| resolveException | POST `/runs/{id}/exceptions/{exId}/resolve` | PayrollRunManage | `{action: "CONFIRM"\|"HOLD", reason?}` (reason required for HOLD) | 200 `PayrollException` | 401/403/404 · 409 `already_resolved` · 422 |
| submitRun | POST `/runs/{id}/submit` | PayrollRunManage | `{}` | 200 `PayrollRunDetail` | 409 `exceptions_open` / `invalid_state` |
| decideRun | POST `/runs/{id}/decision` | PayrollRunManage | `{decision: "APPROVE"\|"REJECT", reason?}` (reason required on REJECT) | 200 `PayrollRunDetail` | 409 `sod_violation` / `invalid_state` · 422 |
| withdrawRun | POST `/runs/{id}/withdraw` | PayrollRunManage | `{}` | 200 `PayrollRunDetail` | 409 `invalid_state` |
| scheduleDisbursement | POST `/runs/{id}/schedule-disbursement` | PayrollRunManage | `{scheduled_at: RFC3339}` | 201 `Disbursement` | 409 `invalid_state` · 422 past-date |
| attestDisbursement | POST `/runs/{id}/disbursement/attest` | PayrollRunManage | `{status: "SUBMITTED_TO_BANK"\|"PAID"\|"FAILED", reason?}` | 200 `Disbursement` | 409 `invalid_transition` · 422 |
| issuePayslips | POST `/runs/{id}/issue-payslips` | PayrollRunManage | `{}` | 200 `PayslipDeliverySummary` | 409 `legal_gate` / `invalid_state` |
| getPayslipDelivery | GET `/runs/{id}/payslip-delivery` | PayrollRunRead (audited) | — | 200 `PayslipDeliverySummary` | 401/403/404 |

Error envelope: existing `{error: {code, message}}`. 404 = not-found OR other-org
(RLS-indistinguishable, deny-by-omission). All admin reads via `with_audits` (atomic);
mutations audit `payroll_run.{close,calculate,exception_resolve,submit,decide,withdraw,
disburse_schedule,disburse_attest,payslip_issue}`.

## 3. DTOs (serde structs → openapi schemas; extend `PayrollRunDetail`)

```
PayrollRunSummary  += close_receipt: Json|null, submitted_by/at, decided_by/at,
                      decision_reason, approval_ref: Uuid|null       // existing fields kept
PayrollRunDetail   += exceptions_open: i64, exceptions_total: i64,
                      calculation: RunCalcSummary|null, disbursement: Disbursement|null,
                      payslip_delivery: PayslipDeliverySummary|null
ClosePreflight      = { checks: [{key, label_ko, ok, warn, note, blocking_refs: [Uuid]}],
                        can_close: bool }
RunCalcSummary      = { version: i32, calculated_at, calculated_lines: i64,
                        blocked_lines: i64, payable: bool /* false until release gate */,
                        kernel_rate_table: str, total_net_won: i64|null /* null unless all
                        lines calculated — never a partial sum presented as a total */ }
LineCalculation     = { line_id, version, gross_won, deductions: [{code, label_ko,
                        amount_won, source_url}], total_deductions_won, net_won,
                        tax_table_version, payable: bool }
PayrollException    = { id, run_id, line_id|null, employee_id|null, employee_display_name,
                        kind: OVERTIME_ALLOWANCE|RETRO_ADJUSTMENT|ABSENCE_DEDUCTION|
                              PRORATION|ACCOUNT_VERIFICATION,
                        severity: info|warn|danger, amount_delta_won: i64|null,
                        summary_ko, detail: Json /* lines + linked_refs */,
                        linked_refs: [{kind, code, id|null}],   // AT-/WO-/person/thread
                        status: OPEN|CONFIRMED|HELD, resolved_by|at|reason,
                        carried_from_run_id: Uuid|null }
ExceptionPage       = { items, total, limit, offset }
Disbursement        = { id, run_id, scheduled_at, status: SCHEDULED|SUBMITTED_TO_BANK|
                        PAID|FAILED, attested_by|at, reason|null }
PayslipDeliverySummary = { run_id, issued: i64, acknowledged: i64,
                        items: [{line_id, employee_id, inbox_doc_id, issued_at,
                                 acknowledged_at|null}] , limit, offset, total }
```

Frontend maps: exception `severity` → chip tone (info/warn/danger), `kind` → i18n label
(연장수당·소급 인상·결근 공제·일할 계산·계좌 확인); stepper states derive from
`PayrollRunDetail.status` + `exceptions_open` exactly per design-spec §4/§5.

## 4. DDL — provisional migration `0186_payroll_run_lifecycle.sql`

```sql
-- Run lifecycle columns + widened FSM (keeps every 0074 state valid).
ALTER TABLE payroll_draft_runs
    ADD COLUMN close_receipt    JSONB NULL,
    ADD COLUMN submitted_by     UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    ADD COLUMN submitted_at     TIMESTAMPTZ NULL,
    ADD COLUMN decided_by       UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    ADD COLUMN decided_at       TIMESTAMPTZ NULL,
    ADD COLUMN decision_reason  TEXT NULL,
    ADD COLUMN approval_ref     UUID NULL;   -- future AP- engine object
ALTER TABLE payroll_draft_runs DROP CONSTRAINT payroll_draft_runs_status_check;
ALTER TABLE payroll_draft_runs ADD CONSTRAINT payroll_draft_runs_status_check CHECK (status IN
  ('STAGED','BLOCKED_LEGAL_GATE','READY_FOR_REVIEW','ATTENDANCE_CLOSED','CALCULATING',
   'CALCULATED','SUBMITTED','REJECTED','APPROVED','DISBURSEMENT_SCHEDULED','PAID','ISSUED','VOID'));
ALTER TABLE payroll_draft_runs ADD CONSTRAINT payroll_draft_runs_sod
  CHECK (decided_by IS NULL OR submitted_by IS NULL OR decided_by <> submitted_by);

CREATE TABLE payroll_line_calculations (      -- versioned draft math, append-only
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    run_id UUID NOT NULL, line_id UUID NOT NULL,
    version INTEGER NOT NULL CHECK (version >= 1),
    gross_won BIGINT NOT NULL CHECK (gross_won >= 0),
    deductions JSONB NOT NULL,                 -- [{code,label_ko,amount_won,source_url}]
    total_deductions_won BIGINT NOT NULL, net_won BIGINT NOT NULL,
    tax_table_version TEXT NOT NULL,
    payable BOOLEAN NOT NULL DEFAULT FALSE,    -- true only post release-gate registration
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, line_id, version),
    FOREIGN KEY (run_id, org_id) REFERENCES payroll_draft_runs(id, org_id) ON DELETE CASCADE
);
CREATE TABLE payroll_run_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    run_id UUID NOT NULL, line_id UUID NULL, employee_id UUID NULL,
    employee_display_name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('OVERTIME_ALLOWANCE','RETRO_ADJUSTMENT',
        'ABSENCE_DEDUCTION','PRORATION','ACCOUNT_VERIFICATION')),
    severity TEXT NOT NULL CHECK (severity IN ('info','warn','danger')),
    amount_delta_won BIGINT NULL,             -- NULL when not derivable from verified source
    summary_ko TEXT NOT NULL, detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    linked_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN','CONFIRMED','HELD')),
    resolved_by UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    resolved_at TIMESTAMPTZ NULL, resolved_reason TEXT NULL,
    carried_from_run_id UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT payroll_run_exceptions_resolution CHECK
      (status = 'OPEN' OR (resolved_by IS NOT NULL AND resolved_at IS NOT NULL)),
    CONSTRAINT payroll_run_exceptions_hold_reason CHECK
      (status <> 'HELD' OR resolved_reason IS NOT NULL),
    FOREIGN KEY (run_id, org_id) REFERENCES payroll_draft_runs(id, org_id) ON DELETE CASCADE
);
CREATE TABLE payroll_disbursements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    run_id UUID NOT NULL UNIQUE,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'SCHEDULED'
        CHECK (status IN ('SCHEDULED','SUBMITTED_TO_BANK','PAID','FAILED')),
    attested_by UUID NULL REFERENCES users(id) ON DELETE RESTRICT,
    attested_at TIMESTAMPTZ NULL, reason TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT payroll_disbursements_failed_reason CHECK (status <> 'FAILED' OR reason IS NOT NULL),
    FOREIGN KEY (run_id, org_id) REFERENCES payroll_draft_runs(id, org_id) ON DELETE CASCADE
);
CREATE TABLE payroll_payslip_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    run_id UUID NOT NULL, line_id UUID NOT NULL,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE RESTRICT,
    inbox_doc_id UUID NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, run_id, line_id),
    FOREIGN KEY (run_id, org_id) REFERENCES payroll_draft_runs(id, org_id) ON DELETE CASCADE
);
-- + indexes (org_id,run_id[,status]) per table; RLS ENABLE+FORCE + org_isolation policy
--   (copy the 0074 DO-block verbatim for the four new tables); GRANT SELECT,INSERT,UPDATE
--   TO mnt_rt. Acknowledged-at is read from the inbox doc (no duplicated ack column).
```

Migration number 0186 is provisional — re-check `ls backend/crates/platform/db/migrations`
immediately before push (0180 is the latest on this branch; collisions across lanes).

## 5. Authz & audit contract

- New `Feature::PayrollRunManage` (`"payroll_run_manage"`) in `platform/authz` — same
  matrix row and `authorize_org_wide` gating as `PayrollRunRead` (built-in
  EXECUTIVE/SUPER_ADMIN; ADMIN never built-in; branch-scoped denied). Shared-root change →
  manifest.
- Reads of others' data audited via `with_audits` (existing pattern); self payslip reads
  never audited. Mutations: one audit event each, action names in §2, `reason` captured
  where the API takes one, receipts (`close_receipt`, decision) persisted as JSONB.
- RLS: every new table forced + policied; tests as `mnt_rt` with `app.current_org` armed
  (extend `payroll_rls_surfaces_as_runtime_role.rs`); cross-org probes must read 0 rows and
  REST must 404.

## 6. Frontend contract (module `web/src/console/payroll/`)

Files (mirror `production/` exactly):
- `PayrollScreen.tsx` — outer session-fence remount (`key = sessionKey:branchId:actorId:
  apiFenceKey:capabilityKey`) + body. Zones per design-spec: header (title + operational
  subline + series chip + deadline chip + single CTA), 5-step stepper, roster card,
  exceptions card, totals card, schedule card. Gate/empty/loading/denied/error states are
  the truthful pre-data surface; `aria-busy`, `role="alert"` + retry on error,
  `role="status"` for loading/empty. className = plain string literals (`payroll__*`),
  no cn/clsx, no inline Hangul.
- `PayrollConsoleRoute.tsx` — `useAuth()` → api/session, `usePayrollConsoleAuthz()` →
  `derivePayrollCapabilities(authz, branchId)`.
- `usePayrollConsoleAuthz.ts` — copy of production's hook: `jwtFloorProjection` floor,
  `fetchAuthzProjection` authoritative, `makePolicyGate`.
- `payrollCapabilities.ts` — features `payroll_run_read` / `payroll_run_manage` →
  `{canRead, canManage(close/calc/resolve/submit/schedule/attest/issue), canDecide
  (=canManage; SoD enforced server-side), canReadSelf(always true — self payslips)}`.
  Deny state renders title + denied line only, zero fetches.
- `payrollApi.ts` — typed via `components["schemas"]` from `@maintenance/api-client-ts`,
  `ConsoleApiClient` GET/POST, `requireData` + `PayrollApiError`, AbortSignal on every op.
- `routeContract.ts` — `PayrollRouteContract {branchId}` + structural fixture.
- `payroll.css`, `index.ts`, i18n at `web/src/i18n/payroll.ts` (`payrollStrings`, module-
  owned file like `i18n/production.ts` — NOT `ko.ts`, which is a collision root).
- Tests: `PayrollScreen.test.tsx` (denied-before-fetch, error+retry, keyboard activation,
  session-fence remount, exception resolve flow, CTA state machine), `payrollApi.test.ts`
  (bearer + typed endpoint, param forwarding, denial surfaced), `payrollCapabilities.test.ts`.
- Persistence: `payColW` column widths + card layout per-user (localStorage, personal-view
  whitelist §3.9.0-①); selection + search survive remount via URL/query where required by
  the completion contract.

Shared roots (integrator, via `integration-manifest.json`): `MOUNTED_SCREEN_KEYS` +
`SCREEN_REGISTRY` entry `payroll: PayrollScreenBody`, i18n `ko.ts` nav keys, openapi paths +
regenerated clients, authz Feature enum. Nav item for `payroll` already exists in
`shell/nav.ts` — no nav edit needed.
