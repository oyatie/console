# CAP-ORG-CONSOLE — Gap Analysis (backend gaps · owning-crate decisions · hot-crate items)

> Stage-1 scout. HOT crates (live codex writers): `identity`, `registry` — no edits proposed
> there; anything they would need is listed in §5 for the integrator/manifest.

## 1. Teams are not first-class in the backend

Design tree = Group → 법인 → 사업장 → 팀. Backend has regions/branches (identity), customer
sites (registry), and `employees.org_unit` TEXT (hr; org-chart read groups by
company→org_unit→position, index 0085). There is **no teams table**.
**Decision**: do NOT invent one in this lane. Team rows in the console tree come from the
HR org-chart grouping (`GET /api/v1/hr/org-chart`); team-level change ops are modeled as
`REASSIGN_ORG_UNIT` proposal ops (bounded employees.org_unit rewrite at apply). A first-class
`org_units` table (+ head appointment linkage) is a registered follow-up, not this slice.

## 2. 법인(entity) list has no console REST read

Entities = member orgs of the group (`platform/group::group_member_orgs`, SECURITY DEFINER
resolver, fail-closed empty). No REST endpoint exposes this list for the org screen.
**Decision**: `orgchange/rest` exposes `GET /api/v1/org-changes/tree` is NOT needed — the
frontend composes tree reads from existing routes; but an entity-scope read
(`GET /api/v1/org-entities` → `{org_id, slug, name, status}` from group_member_orgs) is
needed for multi-법인 columns. Owning crate: **orgchange/rest** (thin read over platform/group,
no hot-crate edit). If the integrator prefers it in identity later, it moves with the
openapi tag.

## 3. Multi-role SoD chain vs single four-eyes

`gov_approvals` = one decision per `request_ref` with DB-enforced approver ≠ requester —
models ONE four-eyes gate, not the design's ordered 4-role chain (HR→재무→법무→임원).
**Decision**: orgchange owns `org_change_approval_steps` (order + role binding + denorm
decision) and records each step's decision through `gov_approvals` keyed by the step id,
inheriting the DB self-approval CHECK. Pure gate logic reused from `mnt-governance-domain`
(`evaluate_gate_chain`, `assess_impact`). No governance-crate edits.

## 4. Preflight signals that don't exist yet (warn-only in slice 1)

- **Freeze window** (급여 마감·회계 결산 중 조직 변경 제한, §3.9.1): no payroll-close/GL-close
  signal API surfaced to other crates. Slice 1 = static warning chip (as the prototype does);
  blocker upgrade needs a payroll/finance-gl read contract — registered gap.
- **Open postings / in-flight approvals** per target: recruiting/approval crates expose no
  per-branch open-count query. Slice 1 = warning chip; blocker upgrade = follow-up read.
- **Span-of-control / orphan / cycle detection**: needs the org_units follow-up (§1).
Blockers that ARE real in slice 1 (Restrict dependents via existing data): active users per
branch, non-terminal equipment per branch — same guards identity/registry deactivate already
enforce at DB level (double net).

## 5. Integrator / shared-root items (NOT edited by this lane)

Recorded in `integration-manifest.json` alongside this file:
1. `backend/openapi/openapi.yaml`: add tag `orgchange` + paths/schemas of design-contract §5
   (+ regenerate clients/{ts,kotlin,swift} — three CI drift gates).
2. `mnt_platform_authz::Feature`: add `OrgChangeRead/Draft/Approve/Apply` variants + floors +
   `as_str`/`from_str` arms (platform/authz is shared; small, mechanical).
3. `build_router` (platform-rest): mount `orgchange::rest::router` + `orgchange` route paths
   into the route-path census.
4. Migration number: **0189 is provisional** (repo head today = 0180; take next free number
   right before push — known collision hazard).
5. `web/src/console/shell/nav.ts`: `orgchart` nav item already exists (gate
   EMPLOYEE_DIRECTORY_READ); add `"orgchart"` to `MOUNTED_SCREEN_KEYS`.
6. `web/src/console/screens/registry.ts`: `orgchart: OrgChartScreenBody`.
7. `web/src/i18n/ko.ts`: nav label already present (`console.shell.nav.orgchart: "조직도"`);
   module strings go in a NEW module-owned file `web/src/i18n/orgchange.ts` (production.ts
   pattern) — no ko.ts edit expected.

## 6. Deliberate scope cuts (slice 1, named)

- Effective-date scheduler: apply is an explicit authorized CTA refused before the date
  (409), not a cron. `workflow_schedules` integration is the follow-up.
- Rejected → revision: modeled as a new request row (`supersedes_id`), not in-place reopen —
  keeps requests append-only-ish and history honest.
- Entity setup wizard's cross-surface joins (messenger channel, workflow seed, notification):
  frontend slice consumes existing module APIs where mounted; anything unmounted degrades to
  absent (never a dead control).
- Employee lifecycle events per REASSIGN_ORG_UNIT affected person: count-only in the apply
  event for slice 1; per-person events registered as follow-up.

## 7. Risks

- `hr/org-chart` grouping uses `employees.company` TEXT — mapping company strings to group
  member orgs (법인 columns) is name-based; mismatches must render as an "unmapped" column,
  never dropped silently (truthfulness bar).
- Approval role_keys (`hr/finance/legal/executive`) are fixed in slice 1; the design wants
  configurable approval matrices (DoA) eventually — table shape (role_key TEXT + step_order)
  already accommodates config later without migration.
