# CAP-FIELD-CONSOLE — design contract (API surface · DTOs · FSMs · DDL 0194)

> Backend contract for STORY-FIELD-001 ("customer site intake → field visit →
> check-in → work log → customer acceptance → SLA/billing"). Closure scope =
> `backend/crates/support/**` only; workorder/compliance/registry needs go through
> `gap-analysis.md` manifests. Route `/console/field`, screen key `field`.

## 1. Object model mapping (design object → backend reality)

| Design object | Backend | Owner crate | Status |
|---|---|---|---|
| 현장 ST- (CustomerSite) | `registry_sites` (name, customer_id, branch_id, address/province/city/postal, lat/lon paired CHECK, geofence_radius_m, contact) | registry | EXISTS |
| 거래처 CL- (Customer) | `registry_customers` | registry | EXISTS |
| 이슈/접수 (intake ticket) | `support_tickets` (origin INTERNAL/CUSTOMER, category, priority→SLA due_at, FSM, comments, assignee, org RLS via 0032/0035) | support | EXISTS; needs site/customer/WO link columns (0194) |
| 현장 방문 (field visit) WO- | `work_orders` (16-state FSM, site_id + customer_id + equipment_id NOT NULL, target_due_at, report fields, evidence stages) | workorder (maintenance lane) | EXISTS — manifest-only |
| 체크인/아웃 | `site_attendance_events` (ARRIVAL/DEPARTURE per user×WO×site, no coordinates, survives consent withdrawal) + `site_geofence_presence` (transient) | compliance | EXISTS — read-only projection here; write path is ping-ingest |
| 업무일지 JL- (work log) | work-order report path (`report_submitted_*`, diagnosis, action_taken, result_type) + `work_order_status_history` + evidence attachments | workorder | EXISTS — manifest-only |
| 고객 인수 (customer acceptance) | `support_ticket_acceptances` (NEW, 0194) | support | NEW |
| SLA 상태 | derived: open tickets vs `due_at` (deterministic rule §5) | support | NEW read model |

## 2. REST surface

### 2.1 Reused as-is (no change)

| Route | Method | Authz | Notes |
|---|---|---|---|
| `/api/v1/support/tickets` | GET | `Feature::Login` on representative branch; untriaged queue only for `BranchScope::All` | `TicketPage { items: TicketSummary[], next_cursor, total }`; filters status/priority/category/origin/assignee/include_untriaged; keyset `cursor`+`limit` (clamped 1..=100) |
| `/api/v1/support/tickets` | POST | `Feature::Login` on `branch_id` | `CreateInternalTicketRequest { branch_id, category, priority, title, body }` → 201 TicketSummary |
| `/api/v1/support/tickets/{id}` | GET | branch-in-scope resolution | `TicketDetail { ticket, comments }` (internal audience) |
| `/api/v1/support/tickets/{id}/assign` | POST | `Feature::AssigneeManage` on ticket branch (untriaged ⇒ cross-branch only) | `{ assignee_user_id, branch_id? }` — triage sets branch |
| `/api/v1/support/tickets/{id}/transition` | POST | `Feature::AssigneeManage` | `{ to_status }` — FSM-enforced, 409 on illegal edge |
| `/api/v1/support/tickets/{id}/comments` | POST | `Feature::WorkOrderStart` | `{ body, is_internal_note }` → 201 CommentView |
| `/api/v1/support/intake` | POST | none (public; DB fixed-window rate limit ip/device/global; `scope_org(STOREFRONT_ORG_ID)`) | 202 `{status:"received"}`; never echoes PII |
| `/api/v1/customers` | GET | registry authz | customer list (CL- upstream link) |
| `/api/v1/sites`, `/api/v1/sites/{id}` | GET/PATCH | registry authz | site master + geo/geofence edit |
| `/api/v1/work-orders*`, `/api/v1/sync`, evidence presign/confirm | — | workorder crate | visit dispatch/report/evidence — composed by frontend, not wrapped |

### 2.2 New endpoints (support crate — `backend/crates/support/rest`)

All errors use the existing envelope `{ error: { code, message } }` with the
existing kind→status mapping (validation→422, forbidden→403, not_found→404,
conflict/invalid-transition→409, internal→500; DB details never leaked).

#### A. `GET /api/v1/field/sites` — field overview (list layer)

- Authz: principal via request-context;
  `authorize(WorkOrderReadAll, representative_branch)` — the field read gate
  mirrors the shell nav gate (`OPERATIONAL_ROLES x work_order_read_all`), so the
  open-signup MEMBER tier is denied server-side (403), not just nav-hidden;
  rows confined to `BranchScope` (+ Postgres RLS as `mnt_rt`). Deny-by-omission:
  out-of-scope sites never appear in rows, counts, or totals.
- Query: `q?` (site/customer substring), `customer_id?`, `sla?` (`OK|AT_RISK|BREACHED`),
  `limit?` (clamp 1..=100), `cursor?` (keyset by `(site_name, site_id)`).
- Response `FieldSitePage { items: FieldSiteRow[], next_cursor: Option<SiteId>, total: i64 }`.

```
FieldSiteRow {
  site_id: Uuid, site_name: String, branch_id: Uuid,
  customer_id: Uuid, customer_name: String,
  address: Option<String>, latitude: Option<f64>, longitude: Option<f64>,
  open_ticket_count: i64,          // status NOT IN (RESOLVED, CLOSED)
  breached_ticket_count: i64,      // open AND due_at < now
  next_due_at: Option<Timestamp>,  // min due_at over open tickets
  active_work_order_count: i64,    // work_orders in non-terminal status for site
  last_arrival_at: Option<Timestamp>, // max site_attendance_events ARRIVAL
  sla: "OK" | "AT_RISK" | "BREACHED"  // §5 deterministic rule
}
```

- Implementation note: single SQL aggregation joining `registry_sites` ×
  `registry_customers` × `support_tickets` × `work_orders` × `site_attendance_events`
  (read-only lateral counts). Cross-crate READS of workorder/compliance tables are
  a deliberate, documented coupling (same DB, RLS-armed); WRITES/DDL to those
  tables remain with their owning lanes.

#### B. `GET /api/v1/field/sites/{id}` — site detail (object layer)

- Authz: site branch resolved, `authorize(WorkOrderReadAll, site.branch_id)`;
  404 (not 403) when the site is outside scope — no existence leak; 403 only for
  an in-scope site the principal's role may not read.
- Response:

```
FieldSiteDetail {
  site: { id, name, branch_id, customer_id, customer_name,
          address, province, city, postal_code, latitude, longitude,
          geofence_radius_m, contact_name, contact_phone },
  sla: FieldSlaSummary { state, open, breached, next_due_at,
                         resolved_within_sla_90d: i64, resolved_breached_90d: i64 },
  tickets: TicketSummary[]           // open first, then recent closed; cap 50
  work_orders: FieldWorkOrderRef[]   // { id, request_no, status, priority,
                                     //   target_due_at, report_submitted_at,
                                     //   result_type, created_at } cap 50
  attendance: FieldAttendanceEvent[] // { user_id, user_name, work_order_id,
                                     //   kind: ARRIVAL|DEPARTURE, occurred_at } cap 50
  acceptances: TicketAcceptanceView[] // §2.2-D shape, most recent first, cap 50
}
```

  `tickets`+`work_orders`+`attendance`+`acceptances` are the history layer; every
  ref is a traversable link (ticket → ticket detail, WO → workorder API, user →
  directory) satisfying ≥2 upstream (customer, contract/registry) + ≥2 downstream
  (tickets, WOs, attendance, acceptance) object links.

#### C. `POST /api/v1/support/tickets/{id}/link` — bind intake to site/visit (action layer)

- Authz: `Feature::AssigneeManage` on ticket branch (untriaged ⇒ cross-branch,
  same rule as assign; linking a site to an untriaged CUSTOMER ticket also sets
  `branch_id` from the site — this IS triage).
- Body: `{ site_id?: Uuid, work_order_id?: Uuid }` — at least one required (422).
- Validation (fail-closed): site must exist in scope; `customer_id` denormalized
  from the site; `work_order_id` must reference a WO whose `site_id` matches the
  ticket's site (409 otherwise — referential guardrail). Cleared with explicit
  `null` only on tickets not yet CLOSED.
- Response: 200 `TicketSummary` (now carrying `site_id`, `customer_id`,
  `work_order_id` — see §3 DTO extension).
- Audit: `support.ticket.linked` event with before/after.

#### D. `POST /api/v1/support/tickets/{id}/acceptance` — customer acceptance (action layer)

- Authz: `Feature::AssigneeManage` on ticket branch.
- Precondition: ticket status == RESOLVED (409 otherwise — acceptance is the
  four-eyes-style closure evidence, DESIGN §2 종결 rule: final approval ≠ closure).
- Body:

```
RecordAcceptanceRequest {
  kind: "CUSTOMER_ACCEPTED" | "CUSTOMER_DECLINED",
  channel: "IN_PERSON" | "PHONE" | "EMAIL" | "MESSENGER",   // curated enum §4-19
  accepted_by: String (1..=200 chars, trimmed non-empty),   // customer-side name
  note: Option<String> (<=2000 chars),
}
```

- Effect (single transaction):
  - insert `support_ticket_acceptances` row (append-only);
  - `CUSTOMER_ACCEPTED` ⇒ FSM transition RESOLVED→CLOSED via existing
    `transition_status` path (sets `closed_at`, fans out StatusChanged push);
  - `CUSTOMER_DECLINED` ⇒ FSM transition RESOLVED→IN_PROGRESS (reopen), note
    becomes a customer-visible comment.
- Response: 201 `TicketAcceptanceView { id, ticket_id, kind, channel, accepted_by,
  note, recorded_by_user_id, recorded_by_name, occurred_at }`.
- Audit: `support.ticket.acceptance` (+ the transition's own audit event).
  `accepted_by` is a business fact (like requester_name), not logged to traces.

## 3. DTO extensions (existing types, additive)

`TicketSummary` gains (all `Option`, additive — clients tolerate):
```
site_id: Option<SiteId>, site_name: Option<String>,
customer_id: Option<CustomerId>, customer_name: Option<String>,
work_order_id: Option<WorkOrderId>,
```
`ListTicketsQuery`/`ListTicketsRequest` gain `site_id: Option<SiteId>` filter
(field detail uses it; index provided in 0194).

New id newtypes needed in kernel-core only if absent: `SiteId`, `CustomerId`,
`WorkOrderId` already exist (used by registry/workorder crates) — reuse, no new ids.

## 4. FSMs

- **Ticket** (unchanged, `support/domain`): OPEN→IN_PROGRESS; IN_PROGRESS→ON_HOLD|RESOLVED;
  ON_HOLD→IN_PROGRESS; RESOLVED→CLOSED|IN_PROGRESS; CLOSED terminal. Acceptance
  endpoint is a *driver* of the RESOLVED→CLOSED / RESOLVED→IN_PROGRESS edges — it
  adds no new states (lazy: no parallel FSM).
- **Work order** (reference, workorder crate): 16-state; the field console renders
  its status chips read-only and links out for mutations.
- **Story state chain**: intake(OPEN) → triage/link(site set, branch set) →
  dispatch(WO- linked) → visit(WO ASSIGNED→IN_PROGRESS; ARRIVAL/DEPARTURE events)
  → work log(WO REPORT_SUBMITTED→…) → resolve(ticket RESOLVED) →
  acceptance(CLOSED | reopen) → SLA rollup (resolved_at/closed_at vs due_at).

## 5. SLA derivation (deterministic — §4-28 no-AI, §4-26 SLA≠SLO)

Per site over OPEN/IN_PROGRESS/ON_HOLD tickets:
- `BREACHED` if any `due_at < now()`;
- else `AT_RISK` if any `due_at < now() + interval '24 hours'`;
- else `OK`.
Windows come from `SlaPolicy` (support/domain — already a struct so deployments
can override; the configurable-setting-object upgrade is registered in
gap-analysis, not silently hardcoded further). Labels in UI must say **SLA**
(contractual, site/contract-scoped) — never SLO.

## 6. DDL — provisional migration `0194_field_console_support_extensions.sql`

Number 0194 is provisional (worktree HEAD = 0180; renumber to next free at
integration — openapi/migration collision protocol). Full text:

```sql
-- CAP-FIELD-CONSOLE: bind support intake to the field object chain
-- (site / customer / work order) and record customer acceptance as the
-- audited closure evidence for field visits.
--
-- support_tickets already carries org_id + FORCE RLS org_isolation (0032/0035),
-- so the new columns inherit the row policy; no new policy on that table.

-- mnt-gate: audited-table support_tickets
ALTER TABLE support_tickets
    ADD COLUMN site_id       UUID REFERENCES registry_sites(id)     ON DELETE RESTRICT,
    ADD COLUMN customer_id   UUID REFERENCES registry_customers(id) ON DELETE RESTRICT,
    ADD COLUMN work_order_id UUID REFERENCES work_orders(id)        ON DELETE RESTRICT;

-- A work-order link presupposes the site link (visit is dispatched to the
-- ticket's site); enforced app-side too (409), constraint is the backstop.
ALTER TABLE support_tickets
    ADD CONSTRAINT support_tickets_wo_requires_site
        CHECK (work_order_id IS NULL OR site_id IS NOT NULL);

-- Field console per-site queue + site filter on the ticket list.
CREATE INDEX idx_support_tickets_site
    ON support_tickets (site_id, status, created_at DESC)
    WHERE site_id IS NOT NULL;

-- Customer acceptance: append-only closure evidence per ticket.
-- Full tenant table born post-multi-tenant: org_id + FORCE RLS + immutable-org
-- trigger + composite (id, org_id) key inline (0042 pattern), and explicit
-- mnt_rt grants (0058 lesson: RLS is meaningless if the runtime role has no
-- table privilege — verify as mnt_rt, not superuser).

-- mnt-gate: audited-table support_ticket_acceptances
CREATE TABLE support_ticket_acceptances (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    org_id              UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    ticket_id           UUID        NOT NULL REFERENCES support_tickets(id) ON DELETE RESTRICT,
    kind                TEXT        NOT NULL CHECK (kind IN ('CUSTOMER_ACCEPTED', 'CUSTOMER_DECLINED')),
    channel             TEXT        NOT NULL CHECK (channel IN ('IN_PERSON', 'PHONE', 'EMAIL', 'MESSENGER')),
    -- Customer-side acknowledger; business fact like requester_name (never logged).
    accepted_by         TEXT        NOT NULL CHECK (btrim(accepted_by) <> '' AND char_length(accepted_by) <= 200),
    note                TEXT        CHECK (note IS NULL OR char_length(note) <= 2000),
    recorded_by_user_id UUID        NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    occurred_at         TIMESTAMPTZ NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, org_id)
);

ALTER TABLE support_ticket_acceptances ENABLE ROW LEVEL SECURITY;
ALTER TABLE support_ticket_acceptances FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON support_ticket_acceptances
    USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);
CREATE TRIGGER trg_support_ticket_acceptances_org_immutable
    BEFORE UPDATE ON support_ticket_acceptances
    FOR EACH ROW EXECUTE FUNCTION enforce_org_id_immutable();

CREATE INDEX idx_support_ticket_acceptances_ticket
    ON support_ticket_acceptances (ticket_id, occurred_at DESC);

-- Append-only evidence: runtime role may read and insert, never mutate/erase.
GRANT SELECT, INSERT ON support_ticket_acceptances TO mnt_rt;
REVOKE UPDATE, DELETE ON support_ticket_acceptances FROM mnt_rt;
```

## 7. Audit events (append-only, existing support_audit_event builder)

| Event | When | Snapshot |
|---|---|---|
| `support.ticket.linked` | link endpoint | before/after {site_id, customer_id, work_order_id, branch_id} |
| `support.ticket.acceptance` | acceptance endpoint | {kind, channel, ticket status edge} — `accepted_by` PII-handled like requester_name |
| existing create/assign/transition/comment events | unchanged | — |

## 8. Frontend module contract (STAGE 3 — `web/src/console/field/**`, lane-owned)

Mirror the production exemplar exactly (freshest convention):

| File | Content |
|---|---|
| `FieldScreen.tsx` | `FieldScreen` session-fence remount wrapper (sessionKey·branchId·actorId·api-fence·capabilityKey) + `FieldScreenBody`: generation/AbortController fencing, `load()` on mount/session change, loading `role="status"`, error alert+retry, denied state when `!canRead`, list pane (stat bar derived from rows + search + rows) + detail pane (kv/links/acceptance action). className = plain string literals (purity gate — no cn/clsx). |
| `fieldApi.ts` | `createFieldApi(api: ConsoleApiClient)` over generated `components["schemas"]` types: `listSites`, `getSite`, `listTickets`, `createTicket`, `assign`, `transition`, `comment`, `link`, `recordAcceptance`. `FieldApiError` with status. |
| `fieldCapabilities.ts` | `deriveFieldCapabilities(gate, branchId)` from features: `work_order_read_all`→canRead(list), `daily/… n/a`; mapping: canRead = work_order_read_all (list/detail — matches the server gate), canIntake = login, canTriage = assignee_manage (+cross-branch untriaged from projection), canComment = work_order_start, canAccept = assignee_manage. Pure projection; server re-authorizes everything. |
| `useFieldConsoleAuthz.ts` | copy of production pattern: `jwtFloorProjection` fail-closed floor → `fetchAuthzProjection` authoritative → `makePolicyGate`. |
| `routeContract.ts` | `FieldRouteContract { branchId }` + structural fixture. |
| `FieldConsoleRoute.tsx` | route adapter binding `useAuth` api/session → capabilities → screen. |
| `web/src/i18n/field.ts` | module-owned `fieldStrings` (Korean, chips/labels only — NO subtitles/captions per §4-12; note the production exemplar's `subtitle` string is a grammar violation the field module must NOT copy). |
| tests | `*.test.tsx/ts` per exemplar: route denies without grants, screen renders list/empty/denied/error, api maps errors, capabilities matrix. |

Selection/drafts survive refresh: selected site id + intake draft persisted per
module convention (localStorage keyed by session) — required by module contract.

## 9. Completion-contract checklist mapping

- List/overview layer = §2.2-A; object detail = §2.2-B; action/workflow = create/
  assign/transition/link/acceptance; history = detail's tickets+WOs+attendance+
  acceptances (+ audit events).
- ≥2 upstream links (customer, site/contract, person) + ≥2 downstream (tickets,
  WOs, attendance, acceptances) — all traversable.
- Server-enforced deny-by-omission: branch-scope confinement + RLS + 404-not-403
  for out-of-scope objects; untriaged queue cross-branch only.
- Keyboard/focus/contrast/Korean-expansion/responsive + refresh-survival owned by
  STAGE 3 build against this contract.
