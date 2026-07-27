# CAP-BOARD-CONSOLE — build contract (API · DTO · FSM · DDL 0197)

Feeds STAGE 2 (backend) and STAGE 3 (frontend). Collision-root edits (openapi.yaml, clients/**, registry.ts, nav.ts, ko.ts, migration numbering) are declared in `integration-manifest.json`, not made by lanes.

## 1. FSMs

```
Notice:   draft ──publish──▶ published        (one-way; no unpublish; NT- code issued at publish;
                                               title/body/category/audience frozen at publish)
Receipt:  pending ──ack (recipient self)──▶ acknowledged   (idempotent; timestamp kept on first ack)
Completion: derived — acknowledged == total ⇒ list chip tone ok ("완료"-style), else warn ("수령확인 중")
```

Publish preflight (fail-closed, §4-29): status must be `draft` (else 409); effective audience must be non-empty (else 422 `empty audience`); audience_scope=branches requires ≥1 audience branch row.

## 2. DDL — provisional migration `0197_notice_audience_and_category.sql`
(0180 is the highest committed migration here; 0197 is this lane's reserved number — integrator renumbers to the next free slot at consolidation.)

```sql
-- Scoped audiences + typed 유형 for the notice board (STORY-BOARD-001).
-- Audience granularity is branches: user_branches is the schema's only
-- org-membership primitive (sites are customer sites, not employment units).

ALTER TABLE notices
    ADD COLUMN category TEXT NOT NULL DEFAULT 'general'
        CHECK (category IN ('general', 'legal', 'hr_order', 'training')),
    ADD COLUMN audience_scope TEXT NOT NULL DEFAULT 'org'
        CHECK (audience_scope IN ('org', 'branches'));

-- console-gate: audited-table notice_audience_branches
CREATE TABLE notice_audience_branches (
    org_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    notice_id  UUID NOT NULL,
    branch_id  UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (notice_id, branch_id),
    FOREIGN KEY (notice_id, org_id) REFERENCES notices(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (branch_id, org_id) REFERENCES branches(id, org_id) ON DELETE RESTRICT
);
CREATE INDEX idx_notice_audience_branches_branch
    ON notice_audience_branches (org_id, branch_id);

ALTER TABLE notice_audience_branches ENABLE ROW LEVEL SECURITY;
ALTER TABLE notice_audience_branches FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON notice_audience_branches
    USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);

-- DELETE granted: audience rows are replaceable while the notice is a draft
-- (application holds `SELECT … FOR UPDATE` on the notice and rejects mutation
-- once published); receipts stay the immutable record.
GRANT SELECT, INSERT, DELETE ON notice_audience_branches TO console_rt;
```

Publish snapshot (replaces the current org-wide-only insert; audited as today):

```sql
-- scope = 'org' (unchanged):
INSERT INTO notice_receipts (org_id, notice_id, recipient_user_id)
SELECT $1, $2, id FROM users WHERE org_id = $1 AND is_active = true
ON CONFLICT (notice_id, recipient_user_id) DO NOTHING;

-- scope = 'branches':
INSERT INTO notice_receipts (org_id, notice_id, recipient_user_id)
SELECT DISTINCT $1, $2, u.id
FROM users u
JOIN user_branches ub ON ub.user_id = u.id AND ub.org_id = u.org_id
JOIN notice_audience_branches nab
  ON nab.notice_id = $2 AND nab.org_id = $1 AND nab.branch_id = ub.branch_id
WHERE u.org_id = $1 AND u.is_active = true
ON CONFLICT (notice_id, recipient_user_id) DO NOTHING;
```

## 3. REST API surface (`/api/v1/notices`, tag `notices`)

Existing (unchanged semantics): 
- `GET /api/v1/notices?limit=` — published for all authenticated org members; NoticeManage additionally sees drafts. 200 `[NoticeSummary]`, 401.
- `POST /api/v1/notices` — create draft. NoticeManage. 201 `NoticeSummary`, 401/403/422.
- `GET /api/v1/notices/{id}` — draft ⇒ 404 for non-managers (deny-by-omission). 200/401/404.
- `POST /api/v1/notices/{id}/publish` — NoticeManage. 200 `NoticeSummary`, 401/403/404/409 (+ NEW 422 empty audience).
- `POST /api/v1/notices/{id}/ack` — recipient bound from JWT; 204; 404 when caller is not a snapshotted recipient (never 403 — existence isolation).
- `GET /api/v1/notices/{id}/progress` — NoticeManage. 200 `NoticeProgress`, 401/403/404.

New/changed (openapi entries via integrator manifest):
- `PATCH /api/v1/notices/{id}` — edit a DRAFT (title/body/category/audience). NoticeManage. Body `UpdateNoticeDraftRequest` (all fields optional; audience replace-whole). 200 `NoticeSummary`; 401/403; 404 (missing); 409 (already published); 422 (validation, unknown branch id in org).
- `GET /api/v1/notices/{id}/receipts?acknowledged=&limit=&offset=` — NoticeManage. 200 `NoticeReceiptPage`; 401/403/404. `acknowledged` optional bool filter (false = outstanding chase list). Default/max limit 50/200, newest-ack-first then name.
- `POST /api/v1/notices` body gains optional `category` (default `general`) and `audience` (default `{scope:"org"}`).

Authorization model (server-enforced, deny-by-omission, no leakage):
- Read published: any authenticated org member (RLS org fence + published filter).
- NoticeManage (`notice_manage`, roles [D,D,D,A,A,A] via `authorize_org_wide` — requires BranchScope::All): draft visibility, create, PATCH, publish, progress, receipts. Draft leakage rule: 404, list omission. Manager-only aggregate endpoints: 403 (feature existence is not secret).
- Ack: principal-bound recipient only; 404 otherwise.
- Audit actions (existing pattern `notice.*` via `with_audit`): `notice.create_draft`, NEW `notice.update_draft`, `notice.publish`, `notice.publish_recipients`, `notice.acknowledge`.

## 4. DTOs (openapi component schemas)

`NoticeSummary` (extended — all existing fields keep names/types):
```
id: Uuid                    code: string|null (NT-, publish-issued)
author_user_id: Uuid        title: string      body: string
status: "draft"|"published" published_at: date-time|null   created_at: Timestamp
category: "general"|"legal"|"hr_order"|"training"                      [NEW, required]
audience_scope: "org"|"branches"                                       [NEW, required]
audience_branches: [{id: Uuid, name: string}]                          [NEW, required; [] for org]
my_receipt: {acknowledged_at: date-time|null} | null                   [NEW; null = caller not a recipient]
progress: NoticeProgress | null                                        [NEW; non-null only for NoticeManage callers]
```
`CreateNoticeDraftRequest`: `{title(≤300), body(≤20000), category?, audience?: NoticeAudienceInput}`
`UpdateNoticeDraftRequest`: `{title?, body?, category?, audience?: NoticeAudienceInput}`
`NoticeAudienceInput`: `{scope: "org"|"branches", branch_ids: [Uuid] (required non-empty iff scope=branches)}`
`NoticeProgress`: `{total: int64, acknowledged: int64}` (unchanged)
`NoticeReceiptPage`: `{items: [NoticeReceipt], total: int64}`
`NoticeReceipt`: `{recipient_user_id: Uuid, display_name: string, acknowledged_at: date-time|null}`

Korean label map (frontend strings, module-owned file): category — general=안내, legal=법정 통지, hr_order=인사명령, training=교육; status — draft=초안, published=게시; completion chip — in-progress=수령확인 중(warn), complete=완료(ok tone via prog 100%).

## 5. Backend implementation notes (owning crates)

- All changes stay inside `backend/crates/notices/*` + the provisional migration; `Feature::NoticeManage` already exists — no authz crate edits.
- `NewNotice`/domain gains `NoticeCategory` enum + `NoticeAudience` VO (branches non-empty when scoped). Draft update = same validation path; publish keeps FOR UPDATE guard and adds the audience-aware snapshot + empty-audience 422.
- Summary hydration: one query with LEFT JOIN LATERAL (or grouped subqueries) for `my_receipt` and (manager) `progress` + audience branch names — no N+1; list stays ≤200 rows.
- `app/src/lib.rs` already mounts the router; new routes live inside `console_notices_rest::router` + `NOTICES_ROUTE_PATHS` (app route-inventory picks them up) — verify the app's inventory test still passes.
- Tests: extend `rest/tests/api.rs` (scoped publish snapshots only branch members; non-member gets 404 on ack + no rail row; PATCH draft 409-after-publish; receipts 403 for plain admin) + RLS-as-console_rt for `notice_audience_branches`. sqlx scratch DB via `CONSOLE_POSTGRES_DB`; run via subagent (`cargo test -p console-notices-rest -p console-notices-adapter-postgres`, fmt + clippy -D warnings).

## 6. Frontend build contract (`/console/board`, ownership root `web/src/console/board/**` + `web/src/i18n/board.ts`)

- **Body**: `BoardScreenBody` — no-props `ComponentType` (registry contract `SCREEN_REGISTRY: Record<MountedScreenKey, ComponentType>`); pulls `useAuth()` internally; session-fence remount idiom from `production/ProductionScreen.tsx` (sessionKey = `client_session_incarnation ?? access_token`, api fence, generation/AbortController, loading/error/empty/denied truthful states).
- **Surface**: a `ModuleConfig<NoticeRow>` data-only specialization of the ONE generic `ModuleScreen` (`web/src/console/module/config.ts` — §4-18 no forking; `prog` field is implemented): columns 코드(mono)/공지/게시/대상/확인; `statbar` = 게시 중 · 수령확인 진행(warn) · (manager) 초안 — every stat a filter drill; `search` haystack includes code/title/category/audience labels/status; detail kv(코드·게시·대상·유형·작성자) + prog bar (from row `progress`) + actions; `primaryAction` 공지 작성 `policy: "notice_manage"`. If ModuleConfig's current shape can't express a per-row prog detail or the ack action, extend the GENERIC contract (config.ts is shared — integrator manifest), never fork the screen.
- **Actions** (all real mutations via typed client): 수령확인 (visible iff `my_receipt && !my_receipt.acknowledged_at`; POST ack then reload row), 게시 (draft + canManage; POST publish), draft edit (PATCH), 초안 작성 (POST). Denied/absent = not rendered (deny-by-omission).
- **API module**: `boardApi.ts` over `ConsoleApiClient` generated paths (`components["schemas"]["NoticeSummary"]` etc.), `requireData` error-narrowing idiom from `productionApi.ts`.
- **Capabilities**: `deriveBoardCapabilities(gate)` — `canRead: true` (authenticated; published-only data), `canManage: gate.allows({feature: "notice_manage", branch: <sessionBranch>, minPermission: "allow"})` via `useXxxConsoleAuthz` idiom (`jwtFloorProjection` floor → authoritative `fetchAuthzProjection`).
- **i18n**: module-owned `web/src/i18n/board.ts` (`boardStrings` const, mechanism = `i18n/production.ts`); NO inline Hangul in components (check-ui-strings gate); `console.shell.nav.board` already exists in ko.ts — no shared-file edit needed.
- **Object links** (≥2 upstream + ≥2 downstream, module contract): upstream = author (person), audience branches (org objects); downstream = receipts drill (직원 1:N per OT-19), notification pointers (rail cards already map notices), plus rail 공지 section → board and notif click → board detail as inbound traversals. Cross-module chips (개인 수신함, 기록물) only where those modules are exposed.
- **A11y/UX gates**: keyboard J/K/Enter (generic ModuleScreen provides), focus management, AA contrast via tokens only, Korean text expansion, responsive narrow mode; selection + draft compose state survive refresh/Back where required.
- **DARK rule**: `EXPOSED_SCREEN_KEYS` untouched (ADR-0025); board mounts dark via registry + MOUNTED_SCREEN_KEYS (integrator).

## 7. Shared-root entries required (see integration-manifest.json)

1. `web/src/console/shell/nav.ts` — append `"board"` to `MOUNTED_SCREEN_KEYS` (nav item already present, ungated).
2. `web/src/console/screens/registry.ts` — `board: BoardScreenBody` (import from `../board`).
3. `backend/openapi/openapi.yaml` — PATCH + receipts paths, schema changes of §4 (per-domain tag `notices` already set).
4. `clients/{ts,kotlin,swift}` — regenerate + commit (3 CI drift gates).
5. Migration renumber: `0197` provisional → next free number at consolidation.
6. (only if generic contract extension needed) `web/src/console/module/config.ts` per §6.
