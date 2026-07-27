# CAP-ORG-CONSOLE — Backend Contract (API surface · DTOs · FSM · DDL 0189)

> Stage-1 scout output. The org READ side is already served by identity/registry/hr REST;
> the org-CHANGE lifecycle (draft → preflight → SoD approval → effective-dated apply) is the
> gap and is designed here as a NEW `orgchange` crate composing the governance engine.
> Design authority: HANDOFF §15 (lifecycle engine), §16 (guardrails), DESIGN §3.9.1–3.

## 1. Existing read/read-write surface (DO NOT rebuild — consume)

All under bearer JWT + `console_platform_request_context::with_request_context` (RLS-armed org
conn). Deny model: feature-gated `authorize(...)`, fail-closed.

| Route | Op | Crate | Feature gate | DTO |
|---|---|---|---|---|
| `GET /api/v1/regions` | listRegions | identity/rest | RegionManage (org-manage) | `RegionSummary{id, name, deactivated_at?, created_at}` |
| `POST /api/v1/regions` · `PATCH/…/{id}` (update, deactivate) | | identity/rest | RegionManage | `CreateRegionCommand{name}` / `UpdateRegionCommand{name?}` / deactivate = referential guard: active branches → 409 Conflict |
| `GET /api/v1/branches` | listBranches | identity/rest | BranchManage | `BranchSummary{id, region_id, name, deactivated_at?, created_at}` |
| `POST /api/v1/branches` · `PATCH/…/{id}` (rename/move region, deactivate) | | identity/rest | BranchManage | deactivate guard: active users / non-terminal equipment → 409 |
| `GET /api/v1/hr/org-chart` | getHrOrgChart | hr read path | exec/admin read | `HrOrgChartResponse{companies[{company,total,active,units[{name,total,positions[{title,total,employees[{id,name,employee_number?,status}]}]}]}]}` — employee-derived company→org_unit→position grouping |
| `POST /api/v1/sites` · `PATCH /api/v1/sites/{id}` | createSite/updateSite | registry/rest | site features | `CreateSiteCommand`/`UpdateSiteCommand` (customer sites) |
| `GET /api/v1/users` | listUsers | identity/rest | branch-scoped | `UserPage`/`UserSummary` |
| group entity resolution | — | platform/group | GROUP_* grants | `group_member_orgs(pool, group_id, actor) → GroupMemberOrg{org_id, slug, name, status}` — the 법인 list IS the group-member org list (fail-closed empty) |

Console org-tree read = compose: group member orgs (법인 columns) × regions/branches
(identity) × hr org-chart units (teams from `employees.org_unit`) — teams have **no
first-class table** (see gap-analysis §1).

Authz floor (role-vector `[VIEWER, MEMBER, DISPATCHER, BRANCH_ADMIN, ORG_ADMIN, SUPER]`
style array in `console_platform_authz`): `RegionManage`/`BranchManage` = `[D,D,D,A,A,A]`.
New org-change features mirror this floor (see §4).

## 2. New crate: `backend/crates/orgchange/{domain,application,adapter-postgres,rest}`

Composes, does not duplicate:
- `console-governance-domain`: `evaluate_gate_chain` (Authority/SelfChecklist/FourEyes/EgressDlp,
  fail-closed), `assess_impact(Vec<Dependent{kind,id,on_delete}}) → ImpactAssessment`,
  `AuthorityEffect` mapping from Cedar `DecisionEffect`.
- `gov_approval_requests`/`gov_approvals` (migrations 0153/0158/0164): each SoD step's
  decision is recorded as a gov approval keyed by a step-specific `request_ref`
  (`org_change_approval_steps.id`) — DB CHECK `approver_id <> requested_by` gives
  self-approval-impossible for free. The step table (below) owns ordering + role binding.
- identity/registry **application commands** (pub API, no edits): `CreateBranchCommand`,
  `UpdateBranchCommand`, `DeactivateBranchCommand`, `CreateRegionCommand`,
  `UpdateRegionCommand`, `DeactivateRegionCommand`, `CreateSiteCommand`, `UpdateSiteCommand`
  — the apply executor replays the approved proposal ops through these inside one audited tx.
- Audit spine: `with_audit` (same pattern as registry adapter).

## 3. FSM (org_change_requests.status)

```
DRAFT ──preflight──▶ PRECHECKED ──submit──▶ IN_APPROVAL ──all steps approved──▶ APPROVED
  ▲                                             │ any step rejected
  └───────────── new revision ◀── REJECTED ◀────┘
APPROVED ──effectuate (kind ∈ {NEW,REORG}, now ≥ effective_date)──▶ APPLIED   [terminal]
APPROVED ──effectuate (kind = DISSOLVE)──▶ SETTLING ──all items done──▶ ARCHIVED [terminal]
DRAFT | PRECHECKED ──cancel──▶ CANCELLED  [terminal]
```

Rules (all server-enforced, fail-closed):
- `kind ∈ {NEW, REORG, DISSOLVE}` (신설/개편/폐지); immutable after submit.
- `effective_date` editable only in DRAFT/PRECHECKED. Effectuate refused (409) before
  `effective_date` (KST date, org-local) — explicit apply in slice 1, scheduler later.
- submit requires a preflight receipt with `blockers = []` computed against **current** state
  (stale receipt older than the last org mutation → re-run, 409).
- IN_APPROVAL: steps decided strictly in `step_order`; deciding out of order → 409;
  approver ≠ drafter enforced by gov_approvals CHECK + application check that the approver
  holds the step's role feature. Reject at any step → REJECTED (append-only; revision =
  new request row with `supersedes_id`).
- ARCHIVED gate: every settlement item done (참조 무결성). Archive = soft (identity/registry
  deactivate commands), never DELETE.
- Every transition = audit event + append-only `org_change_events` row (who/when/from→to/
  reason/decision), mirroring the prototype's logEvent taxonomy (§design-spec 4).

## 4. Authz (deny-by-omission, no leakage)

New `feature_catalog` keys (migration 0189) + `Feature` enum additions (platform/authz —
**hot-adjacent, goes to integration manifest**):

| feature key | floor | used by |
|---|---|---|
| `org_change_read` | `[D,D,D,A,A,A]` | list/get/preflight-view |
| `org_change_draft` | `[D,D,D,A,A,A]` | create/update draft, preflight, submit, cancel |
| `org_change_approve` | `[D,D,D,D,A,A]` | decide an approval step (plus step-role match) |
| `org_change_apply` | `[D,D,D,D,A,A]` | effectuate, settle, archive |

Out-of-scope object → 404 (not 403) — deny-by-omission; list returns only in-scope rows via
RLS `org_isolation`. UI capability projection: `canRead/canDraft/canApprove/canApply` derived
exactly like `deriveProductionCapabilities`.

## 5. REST surface (crate `orgchange/rest`, mounted by integrator in build_router)

Base `/api/v1/org-changes`, tag `orgchange` (per-domain tag mandatory — client-gen memory).

| Method+Path | Op id | Req body | 2xx | Errors |
|---|---|---|---|---|
| `POST /api/v1/org-changes` | createOrgChange | `CreateOrgChangeRequest` | 201 `OrgChangeDetail` | 401/403/422 |
| `GET /api/v1/org-changes?status=&kind=&limit=&offset=` | listOrgChanges | — | 200 `OrgChangePage` | 401/403 |
| `GET /api/v1/org-changes/{id}` | getOrgChange | — | 200 `OrgChangeDetail` | 401/403/404 |
| `PATCH /api/v1/org-changes/{id}` | updateOrgChangeDraft | `UpdateOrgChangeDraftRequest` (kind/effective_date/proposal/reason; DRAFT·PRECHECKED only) | 200 `OrgChangeDetail` | 401/403/404/409/422 |
| `POST /api/v1/org-changes/{id}/preflight` | preflightOrgChange | — | 200 `OrgChangePreflightReport` | 401/403/404/409 |
| `POST /api/v1/org-changes/{id}/submit` | submitOrgChange | — | 200 `OrgChangeDetail` | 401/403/404/409(blockers/stale)/422 |
| `POST /api/v1/org-changes/{id}/approval-steps/{stepId}/decision` | decideOrgChangeStep | `OrgChangeDecisionRequest{decision: APPROVED\|REJECTED, memo?}` | 200 `OrgChangeDetail` | 401/403/404/409(out-of-order/self-approval/decided) |
| `POST /api/v1/org-changes/{id}/effectuate` | effectuateOrgChange | — | 200 `OrgChangeDetail` | 401/403/404/409(before effective_date / not approved / apply conflict) |
| `POST /api/v1/org-changes/{id}/settlement-items/{itemId}/complete` | completeOrgChangeSettlementItem | `{memo?}` | 200 `OrgChangeDetail` | 401/403/404/409 |
| `POST /api/v1/org-changes/{id}/archive` | archiveOrgChange | — | 200 `OrgChangeDetail` | 401/403/404/409(unsettled) |
| `POST /api/v1/org-changes/{id}/cancel` | cancelOrgChange | `{reason}` | 200 `OrgChangeDetail` | 401/403/404/409 |

### DTOs

```
CreateOrgChangeRequest {
  kind: "NEW" | "REORG" | "DISSOLVE",
  target: { kind: "ENTITY"|"REGION"|"BRANCH"|"SITE"|"ORG_UNIT", ref: string,  // uuid or org_unit name
            label: string },
  effective_date: date,            // >= today (org-local)
  reason: string,                  // required, 1..4000
  proposal: OrgProposalOp[]        // the sandbox diff (below)
}
OrgProposalOp =                     // typed ops replayed by the apply executor, in order
  | { op:"CREATE_REGION", name }
  | { op:"RENAME_REGION", region_id, name }
  | { op:"DEACTIVATE_REGION", region_id }
  | { op:"CREATE_BRANCH", region_id, name }
  | { op:"RENAME_BRANCH", branch_id, name?, region_id? }      // rename and/or move
  | { op:"DEACTIVATE_BRANCH", branch_id }
  | { op:"CREATE_SITE", customer_id, name, ... }               // CreateSiteCommand fields
  | { op:"UPDATE_SITE", site_id, fields }
  | { op:"REASSIGN_ORG_UNIT", from_org_unit, to_org_unit, scope: {company} }  // team move/rename (employees.org_unit rewrite)
OrgChangeSummary {
  id, code,                        // display code "OC-YYYY-NNNN" (server-issued sequence)
  kind, status, target, effective_date, reason,
  headcount: int, site_count: int, team_count: int,   // impact stats snapshot
  drafted_by, created_at, updated_at, supersedes_id?
}
OrgChangeDetail = OrgChangeSummary + {
  proposal: OrgProposalOp[],
  preflight?: OrgChangePreflightReport,               // latest receipt
  approval_steps: [{ id, step_order, role_key,        // "hr"|"finance"|"legal"|"executive"
                     decision: "PENDING"|"APPROVED"|"REJECTED",
                     decided_by?, decided_at?, memo? }],
  settlement_items: [{ id, item_key, label, done, done_by?, done_at?, memo? }],
  events: [{ at, actor, action, from_status?, to_status?, reason }]
}
OrgChangePreflightReport {
  computed_at, stale: bool,
  blockers: [{ code, label, dependent_kind, count }],  // OnDelete::Restrict dependents
  warnings: [{ code, label }],                          // e.g. OPEN_APPROVALS, FREEZE_WINDOW
  headcount, dependents_total
}
OrgChangePage { items: OrgChangeSummary[], total: int64 }
```

Preflight computation (server, deterministic): dependents scan per target —
active users per branch (identity), non-terminal equipment per branch (registry), open
postings/approvals (warn-only slice 1), payroll freeze window (warn-only slice 1, see
gap-analysis §4) — folded through `assess_impact` (Restrict ⇒ blocker).

Settlement item catalog (DISSOLVE, seeded on effectuate — the 6 design items):
`TRANSFER_EMPLOYEES / POSITIONS / COST_CENTERS / CLOSE_OPEN_DOCS / ASSETS / PAYROLL_SOCIAL_FINAL`.

## 6. DDL — provisional migration `0189_create_org_change.sql`

(Number provisional; renumber to next free at consolidation. Patterns copied from 0153/0158:
FORCE RLS `org_isolation`, `enforce_org_id_immutable`, append-only triggers, composite FK
`(user, org_id)`.)

```sql
INSERT INTO feature_catalog (feature_key) VALUES
  ('org_change_read'), ('org_change_draft'), ('org_change_approve'), ('org_change_apply')
ON CONFLICT (feature_key) DO NOTHING;

CREATE TABLE org_change_requests (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id         UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
  code           TEXT NOT NULL,                       -- OC-YYYY-NNNN, server-issued
  kind           TEXT NOT NULL CHECK (kind IN ('NEW','REORG','DISSOLVE')),
  status         TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN
                   ('DRAFT','PRECHECKED','IN_APPROVAL','APPROVED','APPLIED',
                    'SETTLING','ARCHIVED','REJECTED','CANCELLED')),
  target_kind    TEXT NOT NULL CHECK (target_kind IN ('ENTITY','REGION','BRANCH','SITE','ORG_UNIT')),
  target_ref     TEXT NOT NULL CHECK (btrim(target_ref) <> ''),
  target_label   TEXT NOT NULL CHECK (btrim(target_label) <> '' AND char_length(target_label) <= 200),
  effective_date DATE NOT NULL,
  reason         TEXT NOT NULL CHECK (btrim(reason) <> '' AND char_length(reason) <= 4000),
  proposal       JSONB NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(proposal) = 'array'),
  preflight      JSONB,                               -- latest receipt {computed_at, blockers[], warnings[], ...}
  supersedes_id  UUID,                                -- revision chain after REJECTED
  drafted_by     UUID NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (id, org_id),
  UNIQUE (org_id, code),
  FOREIGN KEY (drafted_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT,
  FOREIGN KEY (supersedes_id, org_id) REFERENCES org_change_requests(id, org_id) ON DELETE RESTRICT
);
CREATE INDEX idx_org_change_requests_status ON org_change_requests (org_id, status, created_at DESC);

-- Ordered SoD chain. Decisions live in gov_approvals keyed request_ref = this row's id,
-- giving DB-level approver<>requester. This table owns order + role binding + denorm decision.
CREATE TABLE org_change_approval_steps (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id         UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
  request_id     UUID NOT NULL,
  step_order     SMALLINT NOT NULL CHECK (step_order BETWEEN 1 AND 8),
  role_key       TEXT NOT NULL CHECK (role_key IN ('hr','finance','legal','executive')),
  decision       TEXT NOT NULL DEFAULT 'PENDING' CHECK (decision IN ('PENDING','APPROVED','REJECTED')),
  decided_by     UUID,
  decided_at     TIMESTAMPTZ,
  memo           TEXT CHECK (memo IS NULL OR char_length(memo) <= 2000),
  UNIQUE (id, org_id),
  UNIQUE (org_id, request_id, step_order),
  FOREIGN KEY (request_id, org_id) REFERENCES org_change_requests(id, org_id) ON DELETE RESTRICT,
  FOREIGN KEY (decided_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);

CREATE TABLE org_change_settlement_items (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id       UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
  request_id   UUID NOT NULL,
  item_key     TEXT NOT NULL CHECK (item_key IN ('TRANSFER_EMPLOYEES','POSITIONS','COST_CENTERS',
                 'CLOSE_OPEN_DOCS','ASSETS','PAYROLL_SOCIAL_FINAL')),
  done         BOOLEAN NOT NULL DEFAULT false,
  done_by      UUID,
  done_at      TIMESTAMPTZ,
  memo         TEXT CHECK (memo IS NULL OR char_length(memo) <= 2000),
  UNIQUE (id, org_id),
  UNIQUE (org_id, request_id, item_key),
  FOREIGN KEY (request_id, org_id) REFERENCES org_change_requests(id, org_id) ON DELETE RESTRICT,
  FOREIGN KEY (done_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);

-- Append-only transition log (audit-spine complement, powers the modal history strip).
CREATE TABLE org_change_events (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id       UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
  request_id   UUID NOT NULL,
  actor        UUID NOT NULL,
  action       TEXT NOT NULL CHECK (btrim(action) <> '' AND char_length(action) <= 80),
  from_status  TEXT,
  to_status    TEXT,
  reason       TEXT CHECK (reason IS NULL OR char_length(reason) <= 4000),
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (id, org_id),
  FOREIGN KEY (request_id, org_id) REFERENCES org_change_requests(id, org_id) ON DELETE RESTRICT,
  FOREIGN KEY (actor, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);
CREATE INDEX idx_org_change_events_request ON org_change_events (org_id, request_id, created_at);

-- FORCE RLS org_isolation + org-immutable on all four; append-only triggers on
-- org_change_events (reuse governance_append_only_record from 0153).
-- GRANT SELECT,INSERT,UPDATE on requests/steps/items; SELECT,INSERT only on events.
```

## 7. Apply executor invariants

- One transaction per effectuate: replay `proposal` ops in order through identity/registry
  application commands; any op failure → whole apply rolls back, status stays APPROVED,
  conflict surfaced as 409 with the failing op index.
- `REASSIGN_ORG_UNIT` = bounded `UPDATE employees SET org_unit = $to WHERE org_id = current
  AND company = $scope AND org_unit = $from` inside the same RLS-armed tx (append lifecycle
  event per affected employee is a follow-up; count recorded in the apply event).
- DISSOLVE effectuate does NOT deactivate yet — it opens SETTLING; the deactivate commands
  run at archive (after the referential guards themselves would pass: identity/registry
  deactivate guards are the second, DB-level net).
- Idempotency: effectuate/archive re-POST on an already-terminal request → 409 (no double
  apply); the tx takes `FOR UPDATE` on the request row.

## 8. Frontend conventions to build to (from `web/src/console/production/**`, freshest exemplar)

- Module layout: `web/src/console/orgchange/` (or extend an org module dir the frontend lane
  owns): `OrgChartScreenBody` (registry mount) + `Screen`/`Route` split (`XxxConsoleRoute` =
  authz adapter, `XxxScreen` = props-pure body with session-fence remount key).
- Authz: `useXxxConsoleAuthz()` = canonical `fetchAuthzProjection` + `jwtFloorProjection`
  fail-closed floor + `makePolicyGate`; capabilities = pure `deriveXxxCapabilities(gate,
  branch)` with typed feature union (here: `org_change_draft`/`org_change_approve`/
  `org_change_apply`/read composite).
- API module: typed `components["schemas"][...]` from `@console/api-client-ts`, class
  `XxxApiError(message, status)`, `requireData` unwrap, `AbortSignal` on every call.
- State discipline: generation counter + AbortController fences, `sessionKey` prop =
  `client_session_incarnation ?? access_token`, remount on authority change.
- i18n: module-owned strings file `web/src/i18n/<module>.ts` exporting a `const` strings
  object (pattern: `web/src/i18n/production.ts`); no inline Hangul in components; className =
  plain string literals (no cn/clsx); denied/loading/empty/error/select states all worded.
- Registry/nav (integrator-owned): nav already declares `{ screen: "orgchart", labelKey:
  "console.shell.nav.orgchart", icon: "network", gate: g(DIRECTORY_ROLES,
  [FEATURES.EMPLOYEE_DIRECTORY_READ]) }`; `ko.ts` label exists (line ~971). Needed from the
  integrator: add `"orgchart"` to `MOUNTED_SCREEN_KEYS` and `SCREEN_REGISTRY.orgchart =
  OrgChartScreenBody` (exposure stays governed by `EXPOSED_SCREEN_KEYS`, currently
  `["sales"]`). See integration-manifest.json.
