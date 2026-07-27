# Wave-2/3 Consolidation Inventory

Integration worktree: `maintenance-worktrees/pr488-design-mirror-sync`
Spine tip at inventory time: `5fa7ac64` (`origin/codex/operational-object-runtime-progress`)
Integration branch: `wave23-consolidation-20260724` (local only, never pushed)
Date: 2026-07-24

## Why this doc exists

A concurrent codex consolidation moved the PR-488 spine mid-build. Several lane
migrations, crates, router mounts, and frontend registry entries were already
landed (some as real feature code, some as bare migration files copied to keep
the sqlx/audit migration gate green). This inventory records — per lane — what
is already represented on the tip, so the merge step reconciles instead of
blindly re-applying and regressing the live spine.

Evidence is git-derived: `git ls-tree`, `git diff HEAD <lane>`, `git log
<base>..<lane>`, and greps of `backend/app/src/lib.rs` (router mounts +
`ConfiguredRouteSurface` route-path surface) and `web/src/console/shell/nav.ts`
/ `web/src/console/screens/registry.ts`.

## Spine state at tip (facts)

**Migrations present on spine (`backend/crates/platform/db/migrations`):**
`0186_payroll_run_lifecycle`, `0187_create_recruiting`,
`0188_create_attendance_console`, `0189_employee_day_eligibility_coordination`,
`0190_create_evaluation`, `0191_create_inventory_cycle_counts`,
`0192_dispatch_gaps`, `0193_maintenance_gaps`,
`0194_harden_maintenance_history_runtime_privileges`, `0195_docs_gaps`,
`0196_platform_force_command_and_fk_closure`. Highest = **0196**; next free = **0197**.

- `0186`/`0187` were introduced by spine commit `b374fa27 fix(backend): restore
  CI audit and migration gates` as **bare migration files** — byte-identical to
  the lane copies but WITHOUT the accompanying feature code (payroll run
  lifecycle + `Feature::PayrollRunManage` are NOT on the spine; `recruiting`
  crate is ABSENT).
- `0188`/`0190`/`0191` were introduced by real feature commits (attendance /
  evaluation / inventory). `0191` is byte-identical to the inventory lane;
  `0188` and `0190` DIFFER from their lanes (codex re-implemented).
- `0189`, `0193`, `0194`, `0195`, `0196` on the spine are DIFFERENT migrations
  than the lanes' same-numbered provisional files → collisions to renumber.

**Routers mounted in `backend/app/src/lib.rs` (relevant to lanes):**
attendance, dispatch, docs (as `EVIDENCE_ROUTE_PATHS`), inventory, notices,
notifications, payroll (base), support, workorder (+ mobile). **evaluation crate
exists but is NOT mounted** (dangling codex partial). `recruiting` and
`orgchange` crates are ABSENT.

**Frontend spine state:**
- `MOUNTED_SCREEN_KEYS` (27): overview, attendance, mywork, inbox, leave,
  benefit, people, sales, consulting, finance, inventory, asset, appr, policy,
  audit, support, dashboard, laborcost, objectExplorer, ontologyManager,
  forecast, workflow, scheduled, messenger, mail, logistics, equipment.
- `EXPOSED_SCREEN_KEYS = ["sales"]` — **DO NOT TOUCH**.
- `registry.ts` bodies include InventoryScreenBody + AttendanceScreenBody
  (codex-side `../../features/attendance`, NOT the lane's `console/attendance`).
- No lane frontend branch modifies `nav.ts` or `registry.ts`; each adds only
  its `web/src/console/<module>/` body + `web/src/i18n/<module>.ts`. Wiring
  (MOUNTED_SCREEN_KEYS + registry + ko.ts nav labels) is entirely the
  integrator's job. dispatch/maintenance/field/docs/notif/board/payroll/
  recruiting/org/evaluation/directory bodies are all genuinely new.

## Per-lane decision matrix (backends)

| Lane | Spine state | Migration | Decision |
|------|-------------|-----------|----------|
| **payroll** | base `mnt_payroll_rest` mounted (418L, no run-lifecycle); `PayrollRunManage` authz ABSENT; test absent | `0186` byte-identical (bare) | **MERGE** lane — brings run lifecycle + authz + test. No migration conflict. |
| **recruiting** | crate ABSENT; not mounted | `0187` byte-identical (bare) | **MERGE** lane — new `recruiting` crate + mount + test. No migration conflict. |
| **org** | `orgchange` crate ABSENT | lane `0189_create_org_change` collides w/ spine `0189_employee_day_eligibility` | **MERGE** + renumber lane migration → next free. |
| **evaluation** | `evaluation` crate present but UNMOUNTED (codex partial); crate diverged | lane `0190_create_evaluation` DIFFERS from spine `0190` (add/add) | **MERGE lane's mount+test**; reconcile crate + migration. Trickiest — spine crate is dangling, lane wires it. |
| **inventory** | mounted + live (`INVENTORY_ROUTE_PATHS`); crate diverged from lane | `0191` byte-identical | **KEEP SPINE.** Lane backend = unmounted alternative. Do NOT merge (would fight live train). |
| **attendance** | mounted + live (`ATTENDANCE_ROUTE_PATHS`); codex body | `0188` DIFFERS | **KEEP SPINE.** Lane backend + `console/attendance` body = unmounted alternative (policy c). |
| **maintenance** | `mnt_workorder_rest` mounted (base); settlement routes absent | lane `0193_workorder_maintenance_console` collides w/ spine `0193_maintenance_gaps` | **MERGE** lane's workorder settlement additions + renumber migration. |
| **field** | `mnt_support_rest` mounted (base); field routes absent | lane `0194_field_console_support_extensions` collides w/ spine `0194_harden_maintenance` | **MERGE** + renumber. Authz: field read gate = `work_order_read_all`. |
| **docs** | `mnt_docs_rest` mounted (`EVIDENCE_ROUTE_PATHS`); retention absent | lane `0195_evidence_retention` collides w/ spine `0195_docs_gaps` | **MERGE** lane's retention additions + renumber. |
| **notif** | `mnt_notifications_rest` mounted (base); routing/toggle/agg absent | lane `0196_notification_policies_and_object_agg` collides w/ spine `0196_platform_force` | **MERGE** lane's additions + renumber. |
| **board** | `mnt_notices_rest` mounted (base); scoped-audience absent | lane `0197_notice_audience_and_category` — 0197 is FREE | **MERGE** — no renumber needed (0197 free). |
| **dispatch** | spec + `dispatch_pipeline_api` test + `0192` landed by codex | n/a (lane backend = manifest/docs only) | **Merge evidence docs only.** Consume spine-landed spec, not lane fragment. |

### Migration renumber plan (post-merge fixups, next-free block from 0197)
- board `0197_notice_audience_and_category` → **0197** (unchanged; free)
- org `0189_create_org_change` → **0198_create_org_change**
- maintenance `0193_workorder_maintenance_console` → **0199_workorder_maintenance_console**
- field `0194_field_console_support_extensions` → **0200_field_console_support_extensions**
- docs `0195_evidence_retention` → **0201_evidence_retention**
- notif `0196_notification_policies_and_object_agg` → **0202_notification_policies_and_object_agg**
- evaluation `0190_create_evaluation` → **0203_create_evaluation** (only if lane migration is merged rather than reusing spine's 0190)

(Numbers assigned sequentially; take the next truly-free number right before each
push if the spine moves again.)

## Per-lane decision matrix (frontends)

All 13 add a `web/src/console/<module>/` body + `web/src/i18n/<module>.ts`; none
touch nav/registry. All are additive + DARK (not in EXPOSED_SCREEN_KEYS).

| Lane | New dir | Screen key to mount | Notes |
|------|---------|---------------------|-------|
| payroll | console/payroll | `payroll` | wire nav+registry+ko |
| recruiting | console/recruiting | `recruit` | key is "recruit" per brief |
| attendance | console/attendance | (alt) | spine already mounts attendance (codex body). Record lane body as alternative; do NOT displace spine mount. |
| org | console/org | `orgchart` | |
| evaluation | console/evaluation | `evaluation` | |
| inventory | console/inventory | (alt) | spine already mounts inventory. Lane body = alternative. |
| dispatch | console/dispatch | `dispatch` | backend spine-landed |
| maintenance | console/maintenance | `maintenance` | |
| field | console/field | `field` | |
| docs | console/docs + console/evidence | `docs` | |
| notif | console/notif | `notif` | |
| board | console/board | `board` | |
| directory | console/directory | `directory` | FE-only lane |

## Shared-root work owed (from lane manifests)
- **platform/authz** Feature variants: `PayrollRunManage` (payroll), recruiting
  pair, attendance exception/substitution (already on spine via mounted
  attendance — verify, don't dup), orgchange, evaluation tiers, field
  (`work_order_read_all` reuse), notif, board, docs, maintenance. Apply once, dedup.
- **backend/app** router mounts + `ConfiguredRouteSurface` + `openapi_drift`
  RouteSource for: payroll gap routes, recruiting, orgchange, evaluation, field,
  notif additions, board additions, maintenance settlement. (inventory /
  attendance / dispatch / docs / notices base already mounted.)
- **openapi.yaml**: merge each lane `openapi-fragment.yaml` not already
  represented; EVERY operation needs a per-domain `tags:` entry (missing tags
  OOM kotlinc into a monolithic client). Dispatch = spine-landed spec wins.
- **nav.ts MOUNTED_SCREEN_KEYS + registry.ts + ko.ts** nav labels for: payroll,
  recruit, orgchart, evaluation, dispatch, maintenance, field, docs, notif,
  board, directory. `EXPOSED_SCREEN_KEYS` untouched.

## Cross-lane noise already on spine (expect no-op / auto-resolve on merge)
Every backend lane carries its own copies of the same spine build-fixes:
`fix(auth): actor_home_org call sites`, `fix(logistics): hex-encode fingerprint
digest`, `fix(db): renumber duplicate 0170 → 0181`. These are already present on
the spine; merges should no-op or trivially resolve them.
