# CAP-DIRECTORY-CONSOLE — frontend conventions + REST contract (stage 1 scout)

## 1. Module layout convention (exemplar: `web/src/console/production/**`, freshest)

Files a directory module mirrors (ownership root for stage 3: `web/src/console/directory/**`):

| production file | role | directory analogue |
|---|---|---|
| `ProductionScreen.tsx` | pure screen: props `{api, branchId, actorId, capabilities, sessionKey}`; outer component re-mounts body via `key=sessionFence` (session/branch/actor/api-ref/capability string) so authority change resets state synchronously | `DirectoryScreen.tsx` |
| `ProductionConsoleRoute.tsx` | route adapter: `useAuth()` → api/session, authz hook → capabilities, passes `sessionKey = session?.client_session_incarnation ?? session?.access_token` | `DirectoryConsoleRoute.tsx` (if route-mounted) |
| `useProductionConsoleAuthz.ts` | fetches `fetchAuthzProjection` from `../policy/authz`, fail-closed to `jwtFloorProjection` while loading, returns `makePolicyGate(projection, projection.source === "authz")` | same pattern |
| `productionCapabilities.ts` | pure `derive*Capabilities(gate, branchId)` mapping backend feature keys → `{canRead, canX…}`; deny-by-omission (`canRead=false` renders only the denied state, no fetch) | `directoryCapabilities.ts` |
| `productionApi.ts` | typed transport: `components["schemas"][…]` from `@console/api-client-ts`, `createXxxApi(api: ConsoleApiClient)` closures, `requireData` throwing typed `XxxApiError(message, status)`; every call takes optional `AbortSignal` | `directoryApi.ts` |
| `routeContract.ts` | `interface XxxRouteContract { branchId }` + structural fixture | same |
| `production.css` | plain-class BEM-ish css using only tokens (`--sp-*`, `--surface`, `--ink`, `--teal`, `--danger-*`, `--radius-*`, `--shadow`, `--border-hairline`); focus-visible outlines; responsive `@media (max-width: 900px)` single column | `directory.css` |
| `index.ts` | public exports only | same |
| `*.test.tsx` | vitest + testing-library: denied-before-fetch, retry-on-error, keyboard activation, capability-scoped controls, backend-reconciled writes (never local success state), stale-response fencing on api/session change, cross-branch filtering | same suite shape |

Data discipline in `ProductionScreen`: generation counter + AbortController fencing for loads and
mutations; loading/`role="status"`, error/`role="alert"` + retry; empty state string; denied state
short-circuits before any fetch. i18n: module strings module `web/src/i18n/production.ts`
(`productionStrings as text`, `as const`) — **no inline Hangul in components** (check-ui-strings gate);
className = plain string literal (purity gate bans cn/clsx under `web/src/console/**`).

### Registry-mounted comms pattern (exemplar: `web/src/console/messenger/MessengerScreenBody.tsx`)

Screen bodies mounted by `SCREEN_REGISTRY` are **no-prop `ComponentType`s**. A comms-family body
(directory is in the comms nav group, ungated like messenger) binds context internally:
`useAuth()` + `useActiveBranchId()` + optional `useSearchParams`, wraps the surface in
`PolicyGateProvider` with a gate built from `session.roles`/`session.feature_grants` (backend
re-authorizes every call). Directory should follow this Body shape for its registry mount and keep
the Screen pure/testable underneath (production's Screen/Route split).

## 2. Shared collision roots — exact entry shapes (READ-ONLY; integrator applies via manifest)

`web/src/console/shell/nav.ts`:
- Nav item **already exists** (comms group, ungated → visible to any authenticated session):
  `{ screen: "directory", labelKey: "console.shell.nav.directory", icon: "book" }`.
- `MOUNTED_SCREEN_KEYS` does **not** include `"directory"` → needs append (manifest).
- `EXPOSED_SCREEN_KEYS` is evidence-approved (currently `["sales"]`) — directory stays DARK until
  separately approved (ADR-0025); do not touch.

`web/src/console/screens/registry.ts`:
- Shape: `SCREEN_REGISTRY: Readonly<Record<MountedScreenKey, ComponentType>>`; entry
  `directory: DirectoryScreenBody` (import from `../directory`).

`web/src/i18n/ko.ts`: `ko.console.shell.nav.directory = "주소록"` **already present** — no change
needed for the nav label. Module strings live in a new module-owned `web/src/i18n/directory.ts`
(pattern of `i18n/production.ts`) — not a collision root.

Routing: `/console/:screen` via `screenFromConsolePath`; route key `directory` ⇒ `/console/directory`.

## 3. Existing REST contract usable by the directory (openapi.yaml = source of truth)

### 3a. General-employee surface (STORY-DIRECTORY-001 primary)

- `GET /api/messenger/members?branch_id=<uuid>&limit≤100` (tag `messenger`, operationId
  `listMessengerMembers`) — any authenticated **branch member**; returns
  `MessengerMemberListResponse { items: MessengerMemberSummary[] }`,
  `MessengerMemberSummary = { id: Uuid, display_name: string, team: string|null }`.
  Intentionally excludes phone/profile fields. Existing client:
  `web/src/console/messenger/MessengerConsoleApi.ts` (`ConsoleMessengerMember`).
- `GET /api/messenger/members/{userId}?branch_id=<uuid>` (`getMessengerMember`) — person pin-panel
  read with the design's read-audit semantics **already server-enforced**
  (`backend/crates/messenger/adapter-postgres/src/lib.rs::member_profile`):
  - non-self view records a `person.view` audit event **inside the read transaction** (evidence and
    read commit/roll back together);
  - self-view records no audit;
  - target outside the caller's branch scope → 404 with **no audit trail** (deny-by-omission);
  - proven by `messenger/adapter-postgres/tests/use_cases.rs::member_profile_records_person_view_audit_for_non_self_only`.
- `GET /api/v1/branches` (`listBranches`) — **any authenticated user** (`Feature::Login`);
  `BranchSummary { id, region_id, name, deactivated_at, created_at }` → org-placement names.
- `GET /api/v1/users/me` — self record (`UserSummary`), no branch-scope restriction.

### 3b. Management-tier surfaces (not usable for the all-employee directory)

- `GET /api/v1/employees` (`listEmployees`) — HR employee directory with `company`, `search`
  typeahead, `home_branch_review_required`, `limit≤1000/offset` paging; `EmployeePage{items,total,limit,offset}`,
  `Employee { id, company, name, employee_number?, org_unit?, worksite?, job?, position?, hire_date?,
  exit_date?, status?, leave_*?, home_branch_id?, identity_* }`. Feature `EmployeeDirectoryRead`
  matrix `[MEMBER D, RECEPTIONIST D, MECHANIC D, ADMIN A, EXECUTIVE A, SUPER_ADMIN A]`
  (`backend/crates/platform/authz/src/lib.rs`).
- `GET /api/v1/employees/{id}` (`EmployeeDetail` = employee + employment {employment_type,
  phone_e164, base_pay, currency}) — `EmployeeDirectoryManage` tier only; the ordinary directory
  must never include these fields (stated in the openapi description).
- `GET /api/v1/employees/{id}/lifecycle-events` — audited append-only ledger (person-card history
  layer for the privileged view).
- `GET /api/v1/hr/org-chart` — executive/admin read model (company › org-unit › position).
- `GET /api/v1/users` (`listUsers`) — **`Feature::UserManage` only** (admin user management);
  `UserSummary` carries employee link fields, phone, team, roles, branch_ids, account_status.
  The messenger-members endpoint exists precisely so ordinary members don't need this.

### 3c. Downstream actions already contracted

- 메시지: `POST /api/messenger/threads` (`CreateMessengerThreadRequest { branch_id, kind, member_ids,
  visibility?, title? }`) — DM reuse/creation flow already implemented in messenger frontend.
- 메일: mail module (`MailUse` feature) — compose prefill is a frontend concern.

### 3d. Frontend authz projection

`web/src/console/policy/authz.ts`: capabilities are `{feature: snake_case key, permission}` from
`GET /api/v1/policy/features`-aligned keys; `makePolicyGate(projection).allows({feature, branch,
minPermission})`. Relevant existing keys: `employee_directory_read` (management HR tier — gates the
HR "people" nav item, **not** the comms 주소록), `role_manage`, etc. The comms directory item is
ungated in nav (every persona), matching messenger.

## 4. Capability mapping proposal (stage 3 input)

- `canRead` — any authenticated session with an active branch (messenger-members tier); deny state
  only when unauthenticated/branch-less.
- `canViewPerson` — same tier; server audits non-self views (no client-side gating needed beyond
  branch scope).
- `canReadHrDirectory` — `employee_directory_read` (adds org-wide employee rows, org-unit/position
  columns, lifecycle history) — progressive enhancement of the same screen, deny-by-omission.
- `canMessage` — messenger roles (all six roles today, mirrors `MessengerScreenBody`).
- `canMail` — `MailUse`.
