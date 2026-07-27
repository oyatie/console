# CAP-RECRUITING — Gap Analysis (backend + frontend, verified 2026-07-24)

## 1. Backend: what exists

| Fact | Evidence |
|---|---|
| **No recruiting crate** | `backend/crates/` has 34 domain crates + `platform/` — none recruiting; `grep -ri recruit backend --include=*.rs --include=*.toml` → 0 hits; `openapi.yaml` → 0 hits. |
| **Employee creation path (owning domain)** | `backend/app/src/hr.rs` — `create_employee` (`POST /api/v1/employees`, line 846): `authorize_hr_org_wide(Feature::EmployeeDirectoryManage)` → `normalize_create_employee_request` → sha256 request hash → **idempotency reservation** (`employee_create_idempotency` INSERT-then-`FOR UPDATE`, replay on match, 409 on hash mismatch) → active-branch check → INSERT `employees` + `employee_employment_profiles` inside `with_audits` (audit action `employee.create`). |
| Employee tables | `0063_create_employees.sql` (org RLS), `0172_create_employee_employment_profiles.sql` (compensation isolated from directory rows; `employee_create_idempotency` reservation table; advisory-lock trigger on employee_number). |
| Authz | `platform/authz/src/lib.rs`: deny-by-default `Feature` matrix, 6 columns `[MEMBER, RECEPTIONIST, MECHANIC, ADMIN, EXECUTIVE, SUPER_ADMIN]`. `EmployeeDirectoryRead = [D,D,D,A,A,A]`, `EmployeeDirectoryManage = [D,D,D,A,D,A]`. |
| Ontology spine | `ontology/adapter-postgres/src/seed.rs`: built-in catalog (`BUILTIN_CATALOG_VERSION 2026-07-19.1`) already ships `contract → position → posting` with `posting → employee` link **authored unresolved** (intentional null target until a governed compatibility change binds it). Posting type props: scope/fill_count/deadline, `BackingKind::Instance`. |
| Router composition | `backend/app/src/lib.rs::build_router` merges per-domain routers (`hr::router`, `console_identity_rest::router`, …) under request-context middleware. |
| Migrations | `platform/db/migrations/` — highest is `0180_service_principal_auth.sql`. **0187 is provisional**; take the next free number immediately before push (numbers collide across lanes). |

## 2. Backend gaps → owning-crate decisions

1. **New crate `backend/crates/recruiting/{domain,application,adapter-postgres,rest}`** —
   follow the `identity`/`leave` four-crate layout. `rest` self-applies the request-context
   middleware like `hr::router`; app wiring (`build_router` merge) is a one-line app change
   owned by the backend lane (app/src is not a shared collision root).
2. **Hire routes through the owning HR use-case — never a second writeback.** Decision:
   extract the transactional core of `create_employee` in `backend/app/src/hr.rs` into a
   callable `pub(crate) async fn create_employee_core(tx, org, actor, NormalizedCreateEmployeeRequest, employee_id, audit)`
   (same idempotency reservation, same tables, same audit action), and have the recruiting
   `hire` handler call it in the **same transaction** that links the applicant and increments
   the posting fill count. Idempotency key: `recruit-hire-{applicant_id}` — a retried hire
   replays the one committed employee. Recruiting code must not INSERT into
   `employees`/`employee_employment_profiles` directly.
   - Alternative rejected: HTTP call from recruiting → hr endpoint (loses transactional
     atomicity, double-audits, self-signed principal).
3. **New `Feature` rows** in `platform/authz` (deny-by-default matrix additions):
   `RecruitingRead = [D,D,D,A,A,A]`, `RecruitingManage = [D,D,D,A,D,A]` (mirror the HR
   directory pair; recruiting is HR-owned data with EXECUTIVE read-only). `hire` requires
   `RecruitingManage` **and** `EmployeeDirectoryManage` (the owning domain's gate is still
   evaluated by the reused core). Matrix edits touch `platform/authz` — small, additive,
   backend-lane-owned (not a listed collision root); coordinate if another lane adds features
   the same week.
4. **Posting→employee ontology link resolution** is a separately governed compatibility change
   (seed comment is explicit). This slice does NOT rebind the built-in catalog; the recruiting
   tables carry the `hired_employee_id` FK so the link is materializable later. Registering
   `applicant` as an ontology type is deferred with it.
5. **No external applicant portal exists** (console has no unauthenticated principal;
   prototype's v6 candidate persona is a view-as demo scaffold per HANDOFF §0). This slice is
   the **recruiter-side pipeline**: applicants are registered by intake (typed form), and the
   prototype's self-service `candApply`/passkey-offer-inbox flows are contracted as a future
   charter. Honest gap — do not fake a candidate identity.
6. **Offer inbox/passkey receipt** (OFR- InboxDoc, 채용절차법 §4 basis) depends on the personal
   inbox + passkey engine (HANDOFF §3) which has no backend module yet — offer reply is
   recorded by the recruiter (`record-reply`) in this slice; the passkey receipt path is a
   named follow-up.
7. **Talent-pool → workforce-pool conversion** (`rcPoolPropose`) crosses into the workforce
   registry, which has no backend crate either. This slice persists the talent pool (rejected
   archive + reasons); the conversion proposal is a named follow-up gap.

## 3. Frontend: conventions extract (exemplar `web/src/console/production/**` — read in full)

File set to mirror for `web/src/console/recruiting/`:

| File | Convention captured |
|---|---|
| `index.ts` | Re-export Screen, ConsoleRoute, createApi, ROUTE_CONTRACT fixture + type. |
| `routeContract.ts` | Module-owned mount contract interface + structural fixture (no business records). |
| `useProductionConsoleAuthz.ts` | `useAuth()` → `jwtFloorProjection(session)` floor → `fetchAuthzProjection(token, signal)` authoritative → `makePolicyGate(projection, projection.source === "authz")`. Fails closed to JWT floor while loading; token-keyed. |
| `productionCapabilities.ts` | Pure `deriveXxxCapabilities(gate, branchId)` mapping **feature-grant strings** (snake_case, mirroring backend `Feature::as_str`) → boolean capability record. No role checks in the module. |
| `productionApi.ts` | `createXxxApi(api: ConsoleApiClient)` over the typed openapi client (`api.GET/POST("/path/{id}", {params, body, signal})`); `requireData` throws typed `XxxApiError(message, status)` extracting `{error:{message}}`; DTO types from `components["schemas"]`. |
| `ProductionConsoleRoute.tsx` | Route adapter: `useAuth()` for api/session, authz hook, capabilities derivation, passes `sessionKey = client_session_incarnation ?? access_token`. |
| `ProductionScreen.tsx` | **Session fence**: outer component remounts body via `key = [sessionKey, branchId, actorId, apiFenceKey(api), capabilityKey].join(":")` (WeakMap api identity). Body: generation counter + AbortController fencing on every load/mutate; denied state renders before any fetch (`canRead` gate short-circuits); errors render `role="alert"` + retry; loading `role="status"`; writes reconcile from the returned server object (`replacePlan`), never local success state; branch filter on results. Plain string-literal classNames; i18n via `import { productionStrings as text }`; zero inline Hangul. |
| `production.css` | Module-scoped BEM-ish classes (`production__panel`, `production__plan--selected`), token colors only. |
| tests | Vitest + testing-library. Canonical cases: denied-before-fetch (no GET), error→retry, keyboard activation, capability-gated controls, server-reconciled mutation, stale-response fence on api/session swap, cross-branch filtering, api-client bearer/`X-Auth-Transport` transport test, capabilities-from-grants (not roles), Route test parsing a `MeAuthzResponse` (roles + branch_scope + capabilities). |
| i18n | Module-owned `web/src/i18n/production.ts` (`as const` Korean strings, status sub-map with `unknown` fallback). Recruiting adds `web/src/i18n/recruiting.ts` — **not** the shared `ko.ts`. |

### Shared-root facts (read-only; entries are integrator-owned)

- `web/src/console/shell/nav.ts`: nav already declares
  `{ screen: "recruit", labelKey: "console.shell.nav.recruit", icon: "userPlus", gate: g(DIRECTORY_ROLES, [FEATURES.EMPLOYEE_DIRECTORY_READ]) }`
  in the HR group. `MOUNTED_SCREEN_KEYS` does **not** include `"recruit"` yet;
  `EXPOSED_SCREEN_KEYS = ["sales"]` (ADR-0025 — recruiting lands DARK, exposure needs separate
  evidence approval).
- `web/src/console/screens/registry.ts`: `SCREEN_REGISTRY: Record<MountedScreenKey, ComponentType>`
  — bodies are prop-less components (e.g. `people: PeopleWorkforceBody`). Recruiting must export
  a prop-less `RecruitingScreenBody` (route adapter resolves auth/api internally).
- `web/src/i18n/ko.ts`: `console.shell.nav.recruit: "채용"` already present (line ~970) — no
  ko.ts change needed for nav.
- `backend/openapi/openapi.yaml` + `clients/{ts,kotlin,swift}`: integrator-owned; every new
  operation needs a per-domain `tags: [recruiting]` (kotlinc OOM regression otherwise) and all
  three clients regenerated (3 independent CI drift gates).

Required shared-root entries are enumerated in `integration-manifest.json` (this directory).

## 4. Design-vs-buildable deltas (honest scope cuts, all named)

| Prototype behavior | This slice | Why |
|---|---|---|
| Candidate self-service apply (`candApply`) + v6 persona surfaces | Recruiter intake endpoint only | No external principal in console auth; view-as is demo scaffold (HANDOFF §0). |
| Offer passkey inbox receipt (OFR- legal doc) | `record-reply` by recruiter, reply deadline stored | Personal inbox/passkey engine has no backend yet (HANDOFF §3). |
| 인재풀 → 인력풀 전환 제안 | Talent pool persisted + listed; conversion = follow-up | Workforce-pool registry has no backend crate. |
| Pool posting → workforce-pool registration on confirm | Pool postings supported in schema (`POOL_DAILY`), confirm returns explicit `POOL_REGISTRATION_UNAVAILABLE` error until the registry exists | Same missing registry; never fake a registration. |
| 포지션 개체에서 공고 원클릭 생성 (DESIGN §2 backlog #4) | `position_ref` optional text on posting | Position objects exist only as ontology instances; binding is the chain follow-up. |
| Greenhouse-depth collab (scorecard 협업, 이메일 시퀀스, 소싱, 면접 키트) | Out of scope | BENCHMARK honest-gap column. |
