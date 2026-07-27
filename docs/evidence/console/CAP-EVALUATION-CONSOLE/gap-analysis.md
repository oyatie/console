# CAP-EVALUATION-CONSOLE — gap analysis (verified 2026-07-24, this worktree)

## 1. Backend: NO evaluation domain exists

Verified by grep across `backend/crates/**`:
- `grep -riE "review_cycle|performance_review|appraisal|인사평가"` → zero hits.
- All `evaluation` hits are Cedar **policy evaluation** (platform/authz), not HR.
- No `backend/crates/evaluation/` directory. Migrations top out at
  `0180_service_principal_auth.sql`; slot **0190** is claimed for this lane in
  `docs/program/console-capability-registry.json` (mode `new_crate`).

Everything in the contract is greenfield: cycles, subjects, goals, reviews,
evidence links, calibration, RV- ledger codes, features, audit actions.

## 2. What the substrate already provides (reuse, do not rebuild)

| Need | Existing substrate | Where |
|---|---|---|
| Subject identity | `employees` (tenant HR rows, **not auth users**), `employee_employment_profiles` | migrations 0063, 0172; people module reads `/api/v1/employees` |
| Tenant isolation | RLS `org_isolation` on `app.current_org` GUC, `FORCE ROW LEVEL SECURITY`, grants to `console_rt` | 0172 is the copy-exact pattern |
| Audit | `with_audit(pool, AuditEvent, closure)` — mutation + append-only audit row in one tx; `AuditEvent{actor, action, target_type, target_id, org_id, before, after, trace}` | `platform/db/src/audit_tx.rs`, `kernel/core/src/audit.rs` |
| Authz | `Feature` enum + role matrix `[MEMBER, MECHANIC, RECEPTIONIST, ADMIN, EXECUTIVE, SUPER_ADMIN]`, `authorize(...)`, snake_case wire names, deny-by-default | `platform/authz/src/lib.rs` |
| REST conventions | `pub fn router(state: XxxRestState) -> Router`, exported `*_ROUTE_PATHS` consts for the openapi drift test, handler-level bound validation (422 before DB CHECK), `{ "error": { "message } }` body | `sales/rest`, `production/rest` |
| Crate shape | domain / application / adapter-postgres / rest split with `console_rt` RLS tests | `backend/crates/sales/**` (freshest full exemplar) |
| Idempotent create | per-org `idempotency_key` + `request_hash` reservation table | 0172 |

## 3. Substrate gaps and decisions

1. **Self-evaluation actor**: employees are deliberately not auth users and no
   `employees.user_id` linkage exists. Decision: `kind = SELF | MANAGER` on the
   review row; both kinds are recorded by an authenticated console user
   (`evaluator_user_id` = recorder). True subject self-service needs a
   user↔employee link — **named open item**, not simulated.
2. **Team progress**: employees carry `org_unit` (people module already
   surfaces it) — cycle progress by org_unit is derivable server-side; no new
   org tables.
3. **Audit-on-read**: design (§4.5) requires viewing another person's
   evaluation history to be itself audited. `with_audit` is mutation-shaped but
   works for the read too (the "mutation" closure is the SELECT). Only the
   person-ledger GET gets this; cycle/subject reads stay unaudited reads.
4. **RV- codes**: per-org monotone counter table (`SELECT … FOR UPDATE`) issued
   only at finalization, matching the prototype's RV-25xx/26xx seeds.
5. **Owning crate**: **new crate `backend/crates/evaluation/`** with the sales
   4-crate split (domain, application, adapter-postgres, rest). Feature enum
   additions go in `platform/authz` (append-only; allowed by lane
   must_not_touch, which excludes only web/**, clients/**, backend/openapi/**).
6. **Dark landing**: `backend/openapi/**` is integrator-owned, and the app's
   openapi drift gate fails if routes register in `build_router` without
   openapi.yaml. Decision: the build lane ships the crate + migration + an app
   integration test (`backend/app/tests/evaluation_cycle_api.rs`) that mounts
   `console_evaluation_rest::router(...)` **directly** against the scratch DB;
   `build_router` registration + openapi paths + client regeneration are
   deferred to the consolidation integrator via `integration-manifest.json`.

## 4. Frontend: NO evaluation module exists

- `web/src/console/evaluation/` absent (frontend lane will create it).
- `web/src/console/shell/nav.ts` **already declares** the screen:
  `{ screen: "evaluation", labelKey: "console.shell.nav.evaluation", icon:
  "circleCheck", gate: g(DIRECTORY_ROLES, [FEATURES.EMPLOYEE_DIRECTORY_READ]) }`
  in the HR group (read-only observation; DIRECTORY_ROLES = ADMIN, EXECUTIVE,
  SUPER_ADMIN). `ko.ts` already has `evaluation: "평가"`.
- `evaluation` is NOT in `MOUNTED_SCREEN_KEYS` nor `SCREEN_REGISTRY` nor
  `EXPOSED_SCREEN_KEYS` (currently `["sales"]`) — integrator adds the mount;
  exposure stays dark per ADR-0025.

### Frontend conventions (from `web/src/console/production/**`, read fully)

- File set per module: `index.ts`, `routeContract.ts`, `xxxApi.ts(+test)`,
  `xxxCapabilities.ts(+test)`, `useXxxConsoleAuthz.ts`,
  `XxxConsoleRoute.tsx(+test)`, `XxxScreen.tsx(+test)`, `xxx.css`.
- `XxxScreen` wraps the body in a **session fence** key
  (`sessionKey:branchId:actorId:apiFenceKey:capabilityKey`) so authority
  changes re-mount synchronously; body uses generation counter + AbortController
  fencing for loads and mutations; writes reconcile from the returned backend
  object, never local success state.
- `useXxxConsoleAuthz` consumes `fetchAuthzProjection` + `jwtFloorProjection`
  + `makePolicyGate` from `../policy/authz`, failing closed to the JWT floor.
- `deriveXxxCapabilities(gate, branchId)` is a pure projection to booleans;
  screens branch only on capabilities; `canRead === false` renders the denied
  state without fetching.
- API module: typed via `components["schemas"][...]` from
  `@console/api-client-ts`; `requireData` throws `XxxApiError` with the
  parsed `error.message`; every call takes an `AbortSignal`.
- i18n: module-owned `web/src/i18n/evaluation.ts` (like `production.ts`) — no
  inline Hangul in components; `ko.ts` itself is a collision root (not needed
  here — nav label exists).
- Styling: plain-string `className` (purity gate bans cn/clsx), module css file.
- Tests: vitest + testing-library; deny-before-fetch, retry, keyboard
  activation, capability-scoped controls, backend reconciliation, stale-fence,
  cross-branch filtering — mirror all seven production test shapes.

## 5. Design-vs-contract gaps to close in the builds

| Design element | Prototype status | Real contract answer |
|---|---|---|
| Cycle object + lifecycle | header caption only | `evaluation_cycles` FSM DRAFT→OPEN→CALIBRATION→FINALIZED→ARCHIVED, preflight endpoint |
| Goals | absent from prototype UI | `evaluation_goals` typed rows (metric_kind enum, weight) per subject; story requirement |
| My tasks list | seeded array | `GET /my-tasks` derived from OPEN cycles × assigned manager × missing review |
| Team progress card | seeded percentages | cycle detail aggregates by employees.org_unit |
| Scorecard evidence 3종 | hardcoded context strings | `evaluation_evidence_links` typed rows submitted with the review; MANAGER submit requires ≥1 |
| RV- issuance at submit | prototype issues at submit | design charter (§3.9 + story) places code issuance at **finalization**; submit = SUBMITTED review, finalize = RV- + ledger row (prototype simplification superseded by lifecycle charter) |
| Calibration | absent | `calibrate` action with SoD (calibrator ≠ manager evaluator) + reason on change |
| Person ledger | rvHist client state | `GET /employees/{id}/reviews` (finalized only) + audited read |
| Sensitive gate | viewer clearance check | server: `EvaluationRead` feature (management tier) + deny-by-omission 404; field classification 민감 |
