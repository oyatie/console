# CAP-MAINTENANCE-CONSOLE — Design Contract (API · DTO · FSM · DDL 0193)

Contract for stages 2 (backend) and 3 (frontend). Additive only; no existing route/DTO field changes except
optional additions. All mutations audited via `with_audit`; all reads branch-scoped + RLS as `mnt_rt`;
deny-by-omission (403 without leakage; list scoping never reveals other branches).

## 1. Enums (typed fields, §4-19)

```
MaintenanceType:  EMERGENCY("긴급 출동") | CORRECTIVE("교체 정비") | PREVENTIVE("예방 정비") | INSPECTION("점검")
MaintenanceCause: BREAKDOWN("고장") | RETURN_PREP("반납 준비") | SCHEDULED("정기") | INSPECTION_FINDING("점검 발견") | OTHER("기타")
SettlementStatus: DRAFT | SUBMITTED | APPROVED | VOID
SettlementLineKind: LABOR | PART | OUTSOURCE | OTHER
```
Wire form SCREAMING_SNAKE_CASE (domain_enum! macro pattern). Korean labels live only in `web/src/i18n/maintenance.ts`.

## 2. DDL — provisional migration 0193 (`0193_workorder_maintenance_console.sql`)

Number provisional per lane assignment; integrator takes next free number right before push (collision memory).

```sql
-- Typed maintenance classification (G2). Nullable: legacy rows stay unset and
-- render as absent chips; new console intake requires both (application-level).
ALTER TABLE work_orders
    ADD COLUMN maintenance_type TEXT CHECK (maintenance_type IS NULL OR maintenance_type IN
        ('EMERGENCY','CORRECTIVE','PREVENTIVE','INSPECTION')),
    ADD COLUMN maintenance_cause TEXT CHECK (maintenance_cause IS NULL OR maintenance_cause IN
        ('BREAKDOWN','RETURN_PREP','SCHEDULED','INSPECTION_FINDING','OTHER'));

CREATE INDEX idx_work_orders_maintenance_type
    ON work_orders (branch_id, maintenance_type, status)
    WHERE maintenance_type IS NOT NULL;

-- Cost settlement closing the order into cost (G3). Order-owned; finance handoff
-- is a truthful text ref until the finance-gl lane wires vouchers.
CREATE TABLE work_order_settlements (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id         UUID        NOT NULL,                     -- RLS: same arming as work_orders
    work_order_id  UUID        NOT NULL REFERENCES work_orders(id) ON DELETE RESTRICT,
    branch_id      UUID        NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    status         TEXT        NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT','SUBMITTED','APPROVED','VOID')),
    total_amount_krw BIGINT    NOT NULL DEFAULT 0 CHECK (total_amount_krw >= 0),
    voucher_ref    TEXT,                                     -- e.g. finance journal/voucher code once posted
    note           TEXT,
    created_by     UUID        NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    submitted_by   UUID        REFERENCES users(id) ON DELETE RESTRICT,
    submitted_at   TIMESTAMPTZ,
    approved_by    UUID        REFERENCES users(id) ON DELETE RESTRICT,
    approved_at    TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (work_order_id)                                    -- one live settlement per order (VOID frees it via partial index below)
);
-- replace UNIQUE above with: CREATE UNIQUE INDEX ... ON work_order_settlements(work_order_id) WHERE status <> 'VOID';
CREATE TABLE work_order_settlement_lines (
    id             UUID   PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id  UUID   NOT NULL REFERENCES work_order_settlements(id) ON DELETE CASCADE,
    kind           TEXT   NOT NULL CHECK (kind IN ('LABOR','PART','OUTSOURCE','OTHER')),
    label          TEXT   NOT NULL CHECK (label <> ''),
    amount_krw     BIGINT NOT NULL CHECK (amount_krw >= 0),
    source_ref     TEXT,                                      -- PO/IV/outsource code when known
    sort_order     INT    NOT NULL DEFAULT 0
);
-- RLS: enable + org policy identical to work_orders (copy the 0030/0031 pattern:
-- FORCE ROW LEVEL SECURITY, USING org_id = current_org(), immutable org trigger).
-- Verify as mnt_rt in sqlx tests (rls-verify-as-runtime-role memory).
```

## 3. FSMs

### 3.1 Work-order FSM — unchanged
Existing 16-status table + guards is the engine. The console renders the design's 5-step **flow stepper as a
pure projection** (frontend function, single definition):

| Stepper step | Status set (cur when status ∈) |
|---|---|
| 접수 | RECEIVED, UNASSIGNED |
| 계획 · 부품 예약 | ASSIGNED, ON_HOLD, PART_WAITING (pre-start), DELAYED (pre-start) |
| 실행 | IN_PROGRESS, EQUIPMENT_IN_USE, TEMPORARY_ACTION, REVISIT_REQUIRED |
| 정산 | REPORT_SUBMITTED, ADMIN_REVIEW, or settlement in DRAFT/SUBMITTED |
| 전표 | FINAL_COMPLETED (settlement APPROVED ⇒ done; else cur) |
REJECTED/CANCELLED/ARCHIVED render as terminal chips outside the stepper.

### 3.2 Settlement FSM (new, `workorder/domain`)
```
DRAFT --submit(creator, ≥1 line, total=Σlines)--> SUBMITTED
SUBMITTED --approve(actor ≠ submitted_by, CompletionReview feature)--> APPROVED   [four-eyes, fail-closed]
SUBMITTED --return(comment required)--> DRAFT
DRAFT|SUBMITTED --void(admin, reason required)--> VOID
```
Creation precondition: order status ∈ {REPORT_SUBMITTED, ADMIN_REVIEW, FINAL_COMPLETED}. Every transition =
audit event (`work_order_settlement` target type) + status row is immutable history via audit chain.

## 4. API surface (all under existing workorder rest crate; JWT + feature authz per row)

### 4.1 Extensions to existing routes
| Route | Change | Authz |
|---|---|---|
| `GET /api/v1/work-orders` | + query `equipment_id` (uuid), `maintenance_type[]`, `maintenance_cause[]`; lens gains `facets.maintenance_type` and `aggregates.preventive_on_time_rate: number|null`, `aggregates.mttr_minutes: number|null` | WorkOrderReadAll (unchanged) |
| `POST /api/work-orders` | + body `maintenance_type` (required), `maintenance_cause` (required) — new console intake is typed; legacy mobile callers unaffected only if openapi keeps them optional ⇒ contract decision: **optional on wire, required by console UI** (fail-closed client gate + server validates enum) | WorkOrderCreate (unchanged) |
| `PATCH /api/work-orders/{id}` | + optional `maintenance_type`, `maintenance_cause` | WorkOrderEditIntake |
| `GET /api/v1/work-orders/{id}` | + `maintenance_type`, `maintenance_cause`, `settlement: SettlementSummary|null` | WorkOrderReadAll |

### 4.2 New settlement routes
| Route | Verb | Body → Response | Authz |
|---|---|---|---|
| `/api/v1/work-orders/{work_order_id}/settlement` | POST | `{lines:[{kind,label,amount_krw,source_ref?,sort_order?}], note?}` → SettlementSummary (creates DRAFT; 409 if live settlement exists; 409 if order not in settlement-eligible status) | CompletionReview OR WorkReportSubmit (mechanic drafts own order's settlement: must be assigned) |
| `/api/v1/work-orders/{work_order_id}/settlement` | GET | → SettlementSummary (404 when none — no existence leakage beyond order read right) | WorkOrderReadAll |
| `/api/v1/settlements/{settlement_id}/submit` | POST | `{}` → SettlementSummary | creator or CompletionReview |
| `/api/v1/settlements/{settlement_id}/review` | POST | `{decision:"APPROVED"|"RETURNED", comment?}` (comment REQUIRED for RETURNED — 422) → SettlementSummary; approver ≠ submitter (403 four-eyes) | CompletionReview |
| `/api/v1/settlements/{settlement_id}/void` | POST | `{reason}` (required) → SettlementSummary | CompletionReview + admin roles |

### 4.3 DTOs (wire, snake_case, rfc3339 timestamps)
```ts
SettlementLine     = { id, kind, label, amount_krw, source_ref: string|null, sort_order }
SettlementSummary  = { id, work_order_id, branch_id, status, total_amount_krw, voucher_ref: string|null,
                       note: string|null, lines: SettlementLine[],
                       created_by, submitted_by: string|null, submitted_at: string|null,
                       approved_by: string|null, approved_at: string|null, created_at, updated_at }
// Existing (reuse, already serialized): WorkOrderListPage{items,limit,offset,total,lens},
// WorkOrderListItem, WorkOrderDetail{...,approval_line,status_history,evidence,site_contact,assignments},
// WorkOrderObjectSetLens{aggregates,facets,histograms,listograms}
```
Errors: canonical kernel mapping (validation→422, forbidden→403, conflict→409, not-found→404); message bodies
`{error:{message}}` as production api module expects.

### 4.4 OpenAPI (integrator shared root)
All of §4.1/4.2 into `backend/openapi/openapi.yaml` with per-domain `tags: [work-orders]` (client-tagging
memory); regenerate clients/{ts,kotlin,swift}; 3 drift gates must pass.

## 5. Frontend module contract (`web/src/console/maintenance/**`, production exemplar)

```
maintenance/
  MaintenanceScreen.tsx            // session-fence remount wrapper + body (list+lanes+stat bar+detail)
  MaintenanceConsoleRoute.tsx      // binds useAuth api/session → capabilities → Screen
  useMaintenanceConsoleAuthz.ts    // canonical authz projection, JWT floor fail-closed (copy production seam)
  maintenanceCapabilities.ts       // features → {canRead, canCreate, canEditIntake, canAssign, canStart,
                                   //  canSubmitReport, canReview, canManagePriority, canManageTarget,
                                   //  canSettle, canReviewSettlement, canTriage}
                                   // from: work_order_read_all, work_order_create, work_order_edit_intake,
                                   //  assignee_manage, work_order_start, work_report_submit, completion_review,
                                   //  priority_manage, target_manage, org_wide_queue_triage
  maintenanceApi.ts                // typed over @maintenance/api-client-ts components; list/detail/create/
                                   //  assign/start/report/approve/reject/settlement CRUD; requireData error shape
  routeContract.ts                 // { branchId } fixture (structural only)
  maintenance.css                  // plain class strings `maintenance__*`; token colors only
  index.ts                         // public exports
  *.test.tsx / *.test.ts           // vitest + testing-library, module-local
web/src/i18n/maintenance.ts        // maintenanceStrings const (Korean), incl. enum label maps + status chips
```
Interaction contract: stat bar + facets from `lens` (every number drills to a filter); lanes = lens-facet
kanban (unassigned+overdue / unassigned+scheduled / assigned+active); list JK/Enter + selection surviving
refresh via URL/search params; detail = stepper projection (§3.1) + en chips (filter on click) + links
(equipment→asset screen / timeline-graph, person, settlement) + guarded actions; reject/return comment
fail-closed; denied/empty/loading/error truthful states. No inline Hangul; no cn/clsx; chips for status.

## 6. Shared-root deltas (integrator — see integration-manifest.json)
nav.ts `MOUNTED_SCREEN_KEYS` += "maintenance" · registry.ts `maintenance: MaintenanceScreenBody` ·
ko.ts `console.shell.nav.maintenance: "정비"` · openapi.yaml §4.4 · migration renumber if 0193 taken.
EXPOSED_SCREEN_KEYS untouched (module ships dark per ADR-0025).
