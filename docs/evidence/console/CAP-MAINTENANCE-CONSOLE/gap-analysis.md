# CAP-MAINTENANCE-CONSOLE — Gap Analysis (backend vs design contract)

Baseline read: `backend/crates/workorder/{domain,application,adapter-postgres,rest}` (full),
`backend/crates/platform/db/migrations/0008_create_work_orders.sql`, `backend/crates/platform/authz`,
`backend/crates/registry/rest` (equipment timeline-graph), console exemplar `web/src/console/production/**`,
read-only `web/src/console/screens/registry.ts` + `web/src/console/shell/nav.ts`.

## 1. What the backend ALREADY covers (reuse, do not rebuild)

| Design behavior | Existing backend surface |
|---|---|
| Request intake (접수) | `POST /api/work-orders` (`WorkOrderCreate`, limited action), `PATCH /api/work-orders/{id}` intake edit; request_no `YYYYMMDD-NNN` |
| Order FSM | 16-status const transition table (`WORK_ORDER_TRANSITIONS`) with guards Open/AdminOnly/ApprovalLineComplete/FinalCompletionInterlock — richer than the 5-step design flow; the flow stepper is a projection (see design-contract §3) |
| Schedule (배정) | `PUT .../assignments` (primary+secondary, exactly-one-primary), `PATCH .../priority`, target-due + `POST .../target-change-requests` + review (maker-checker) |
| Execute (실행) | `POST .../start`, `POST .../report` (result_type, diagnosis, action_taken); mobile sync batch; evidence presign/confirm/staging + WORM replica verification; daily work plans (JL- analogue) with DRAFT→REQUESTED→APPROVED/REJECTED→FINAL_CONFIRMED |
| Close (승인·완료) | approve chain mechanic-auto→ADMIN→EXECUTIVE (SoD: pending-step approver check), `FinalCompletionInterlock` = approval line complete AND WORM-verified AFTER/REPORT evidence — fail-closed |
| Reject/cancel/archive | admin-only edges + `POST /api/v1/work-orders/{id}/reject` (memo required) |
| List/overview lens | `GET /api/v1/work-orders` returns Foundry-style `lens`: aggregates (total, p1_count, overdue_open_count, unassigned_count), facets (status, priority with drill filters), histograms (target_due_date), listograms (customers, sites) — feeds stat bar + kanban lanes + drill directly |
| Related-order traversal | `around_work_order_id` filter (same equipment OR site OR customer, org+branch-scope guarded) |
| Asset history (downstream) | `GET /api/v1/equipment/{id}/timeline-graph` (registry crate — lifecycle ribbon incl. work_order_count); `idx_work_orders_equipment` exists |
| Detail layers | `GET /api/v1/work-orders/{id}`: approval_line (with names/comments), status_history (actor/action/from→to/at), evidence (stage, WORM status), site_contact, assignments |
| Authz | Feature-gated deny-by-default (`WorkOrderReadAll`, `WorkOrderCreate`, `WorkOrderStart`, `WorkReportSubmit`, `CompletionReview`, `PriorityManage`, `AssigneeManage`, `TargetManage`, `DailyPlanRequest/Review`, `EvidenceAttach`, `OrgWideQueueTriage`), branch scope, RLS as `console_rt`, audit `with_audit` on every mutation |
| Unified inbox | `GET /api/approval-items` fan-in (`ActionInboxWorkOrderPort`) |

## 2. Gaps (design contract → additive closure; owning crate per item)

| # | Gap | Design anchor | Closure | Owning crate |
|---|---|---|---|---|
| G1 | **No `equipment_id` list filter** — asset→WO history needs a precise per-asset order list (around_work_order_id OR-matches customer/site too) | SR-203 series card, FL- asset links, "고장 3회/90일" | Add optional `equipment_id` to `WorkOrderListQuery` + WHERE clause (index already exists). No DDL. | `workorder/rest` |
| G2 | **No typed maintenance classification** — 유형(긴급 출동/교체 정비/예방 정비) and 원인(고장/반납 준비/정기 …) don't exist; backend has priority + free-text symptom only. Preventive-compliance stat and lane semantics are uncomputable without it | detail en chips §4-19, stat 예방정비 준수, lanes | Migration 0193: `maintenance_type` + `maintenance_cause` TEXT CHECK enum columns (nullable — legacy rows unset), wire into create/intake-edit/list-filter/lens facets | `workorder/{domain,application,adapter-postgres,rest}` + platform/db migration |
| G3 | **No cost settlement (정산→전표)** — flow steps 정산/전표 have no backend: no cost lines, no settlement state, no finance handoff | flow stepper, VC-2604 link, "closes into cost" (story) | Migration 0193: `work_order_settlements` (+lines) with DRAFT→SUBMITTED→APPROVED FSM, four-eyes (submitter ≠ approver), only creatable from REPORT_SUBMITTED/ADMIN_REVIEW/FINAL_COMPLETED orders; `voucher_ref` TEXT (finance-gl coupling deferred — honest link field, not a fake FK) | `workorder/*` (settlement is order-owned; finance-gl integration is a later lane) |
| G4 | **Preventive-compliance + MTTR aggregates** absent from lens | stat bar, OT-13 analytics `mttr` | Extend lens aggregates: `preventive_on_time_rate` (preventive FINAL_COMPLETED ≤ target / preventive closed) + `mttr_minutes` (avg IN_PROGRESS→REPORT_SUBMITTED from status_history). Derived SQL only, no DDL. Depends on G2. | `workorder/rest` |
| G5 | **Parts-reservation link (계획·부품 예약 → PO-/IV-)** — no purchase/inventory linkage on WO | flow step 2, PO-121/IV-031 links | DEFER (cross-module): inventory crate has no REST surface in this worktree and purchase POs don't exist as backend objects. Settlement lines (G3) carry `kind=PART` + `source_ref` TEXT so parts cost still closes truthfully. Register as follow-up charter, not silently faked in UI (link chips render only when a ref exists). | future inventory/erp lane |
| G6 | **SLO config object (정비 처리 목표)** — SLA-minute thresholds are UI copy in the prototype | §4-26 | DEFER: represent urgency via existing `target_due_at` + priority P1 (overdue_open_count already in lens). A configurable SLO settings object belongs to the governance/settings lane. No hardcoded "1시간" copy in the module. | future governance lane |
| G7 | **Frontend module absent** — no `web/src/console/maintenance/**`; nav declares screen `maintenance` but `MOUNTED_SCREEN_KEYS`/`SCREEN_REGISTRY` lack it; `console.shell.nav.maintenance` label missing from `ko.ts` | route /console/maintenance | Build module per production exemplar (Screen/Route split, `useMaintenanceConsoleAuthz`, capabilities projection of the feature matrix, typed api module over generated client, routeContract, module i18n `web/src/i18n/maintenance.ts`, module css, vitest suites). Shared-root entries via `integration-manifest.json` (this dir) for the consolidation integrator. | web console (this lane, stage 3) |
| G8 | **OpenAPI drift** — new query params/endpoints/fields must land in `backend/openapi/openapi.yaml` (+ per-domain `tags:`) + regenerated ts/kotlin/swift clients (3 CI gates) | openapi-client-drift-gate memory | Integrator-owned shared root; exact deltas listed in `integration-manifest.json`. | consolidation integrator |

## 3. Deliberate non-gaps (design simulation NOT to be replicated)

- The prototype's client-side lifecycle engine (`lcSeed`/v2 revision loop) for WO- maps onto the existing
  backend FSM + status_history + audit chain; do not build a parallel document-lifecycle store for orders.
- View-as persona switcher, seed rows, simulated SLA countdowns: demo scaffolding (HANDOFF §0) — production
  uses the real principal; countdown = client rendering of `target_due_at`.
- Series SR-203 is a derived view (G1 filter + created_at trend + equipment timeline-graph), not a stored
  series object.

## 4. Sequencing for stages 2–3

1. Stage 2 (backend): G2 DDL+wiring → G1 filter → G4 lens aggregates → G3 settlement (largest; own routes) —
   each package-scoped `cargo build/test -p`, sqlx tests as `console_rt` (RLS memory), `cargo fmt`, clippy -D.
2. Stage 3 (frontend): module skeleton (authz/capabilities/api/i18n) → list+lens stat bar+lanes → detail
   (stepper projection, links, actions incl. assign/start/report/approve/reject fail-closed comment) →
   settlement panel → tests. Registry/nav/openapi/ko.ts via manifest only.
