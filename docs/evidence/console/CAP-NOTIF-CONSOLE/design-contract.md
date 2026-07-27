# CAP-NOTIF-CONSOLE — design contract (API surface · DTOs · FSMs · DDL 0196)

Stage-1 contract feeding Stage-2 (backend) and Stage-3 (frontend). Everything here extends the
existing `backend/crates/notifications/*` crates; no new crate, no producer-facing breakage.

## 1. FSMs

### 1a. Notification (pure-event object — DESIGN §3.9.0-②, no approval lifecycle)
```
emit ──> unread ──(ack: mark_read | read-all | open-linked-object)──> read
              ^                                                        │
              └────────────(mark_unread — swipe toggle)────────────────┘
orthogonal:  open ──(resolve_by_link, link-equality sweep)──> resolved (resolved_at/by set)
```
- `read_at` = FIRST read timestamp, never cleared by mark_unread (forensic).
- `resolved` never implies `read` and vice versa. No hard delete (mnt_rt has no DELETE).
- Every transition = audit event: `notification.emit | notification.read |
  notification.unread | notification.read_all | notification.resolve`.

### 1b. NotificationPolicy (personal setting — §3.9.0-① direct-apply, audited)
```
absent ──(PUT upsert {scope,…,muted:true})──> active ──(DELETE)──> absent
```
Audit actions: `notification.policy_set` / `notification.policy_clear` (target = policy id).

## 2. Schema — provisional migration `0196_notification_policies_and_object_agg.sql`

> Number provisional (charter). Local head = 0180; integrator renumbers to next free before push.

```sql
-- Per-user notification routing policy: mute by scope (all | category | object).
-- Personal settings (DESIGN §3.9.0-①): direct-apply, recipient-owned, audited in code.
-- Muting suppresses ATTENTION (badge counts, realtime fan-out), never data: rows are
-- still persisted and listable. `action` is extensible ('mute' now; 'watch' later).

-- mnt-gate: audited-table notification_policies
CREATE TABLE notification_policies (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    user_id     UUID        NOT NULL,
    scope       TEXT        NOT NULL CHECK (scope IN ('all', 'category', 'object')),
    -- Required iff scope='category'; same bounds as notifications.category.
    category    TEXT        CHECK (category IS NULL OR char_length(btrim(category)) BETWEEN 1 AND 64),
    -- Required iff scope='object'; same JSONB link shape as notifications.link.
    link        JSONB       CHECK (link IS NULL OR jsonb_typeof(link) = 'object'),
    action      TEXT        NOT NULL DEFAULT 'mute' CHECK (action IN ('mute')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (scope = 'all'      AND category IS NULL AND link IS NULL) OR
        (scope = 'category' AND category IS NOT NULL AND link IS NULL) OR
        (scope = 'object'   AND category IS NULL AND link IS NOT NULL)
    ),
    UNIQUE (id, org_id),
    FOREIGN KEY (user_id, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);

-- One policy per exact target per user (upsert key).
CREATE UNIQUE INDEX idx_notification_policies_target
    ON notification_policies (org_id, user_id, action, scope,
                              COALESCE(category, ''), COALESCE(link::text, ''));

ALTER TABLE notification_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification_policies FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON notification_policies
    USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);

-- Personal settings: delete = unmute (a real removal, not a governed archive).
GRANT SELECT, INSERT, UPDATE, DELETE ON notification_policies TO mnt_rt;

-- Backs GROUP BY link (aggregate-by-object read path) and resolve-by-link sweeps.
CREATE INDEX idx_notifications_recipient_link
    ON notifications (org_id, recipient_user_id, link);
```

No `notifications` column changes. Aggregation and mute are read-path joins.

## 3. Backend contract (crate-by-crate)

### `mnt-notifications-domain`
- `NotificationPolicyScope` enum `All | Category(NotificationCategory) | Object(NotificationLink)`
  (validated; serializes to `{scope, category?, link?}`).
- Reuse existing newtypes; no changes to `NotificationLink`.

### `mnt-notifications-application` (new shapes)
```rust
pub struct ListNotificationObjectGroupsQuery { recipient: UserId, unread_only: bool,
    before: Option<String /* opaque cursor */>, limit: i64 }
pub struct NotificationObjectGroup { link: NotificationLink, total: i64, unread: i64,
    categories: Vec<NotificationCategoryCount>, latest: NotificationSummary, muted: bool }
pub struct NotificationObjectGroupPage { items: Vec<NotificationObjectGroup>,
    next_cursor: Option<String> }  // opaque: base64(latest_created_at | link-json)

pub struct MarkNotificationUnreadCommand { recipient, notification_id, trace, occurred_at }

pub struct UpsertNotificationPolicyCommand { recipient, scope: NotificationPolicyScope,
    muted: bool /* v1 always true on upsert */, trace, occurred_at }
pub struct DeleteNotificationPolicyCommand { recipient, policy_id: NotificationPolicyId, trace, occurred_at }
pub struct ListNotificationPoliciesQuery { recipient: UserId }
pub struct NotificationPolicySummary { id, scope, category: Option<String>,
    link: Option<NotificationLink>, action: String, created_at }
```
- `NotificationSummary` gains `muted: bool` (computed per-row against the caller's policies;
  additive field — serde default false, non-breaking for existing consumers).
- `NotificationCountsSummary` semantics change: `total_unread`/`by_category` EXCLUDE muted
  rows; add `muted_unread: i64` so the UI can show "숨김 N" honestly (no silent data loss).
- `unread_count` likewise excludes muted (badge truth = attention truth). The raw list is
  never filtered by mute (rows annotated instead); optional `include` param stays out of v1.

### `mnt-notifications-adapter-postgres` (`PgNotificationStore`)
- `list_object_groups`: `SELECT link, COUNT(*), COUNT(*) FILTER (WHERE unread), max(created_at)…
  GROUP BY link ORDER BY max(created_at) DESC` + per-group latest row (LATERAL) + category
  breakdown (jsonb_agg or second grouped query) + `muted` via LEFT JOIN policies
  (`scope='all'` row OR category match OR link match). Keyset: `HAVING max(created_at) < $cursor_at
  OR (= AND link < $cursor_link)`; cursor is opaque to callers.
- `mark_unread`: `UPDATE … SET unread = true WHERE id = $1 AND recipient_user_id = $2`
  (read_at untouched), NotFound on miss, audited `notification.unread`.
- `unread_count`/`summary`: add `AND NOT EXISTS (policy match)` mute exclusion + muted tally.
- Policy CRUD: `upsert_policy` (INSERT … ON CONFLICT ON the target unique index DO UPDATE
  updated_at), `delete_policy` (by id + user_id — cross-user delete = NotFound), `list_policies`.
  All under `with_audit`; org from `current_org()` as today.
- `emit_notification`: after insert, evaluate mute (same match) and **skip the realtime
  notifier when muted**; row insert + audit unchanged.
- `is_muted(recipient, category, link)` helper shared by emit/list/summary paths (single
  chokepoint — DESIGN §4-19).

### `mnt-notifications-rest` (routes; all `/api/v1/me/*`, principal from JWT)
| Method/Path | op id | Req | 200 | Errors |
|---|---|---|---|---|
| GET `/api/v1/me/notifications/by-object?unread&before&limit` | listMyNotificationObjectGroups | query | NotificationObjectGroupPage | 401/503 |
| POST `/api/v1/me/notifications/{id}/unread` | markMyNotificationUnread | path Uuid | NotificationSummary | 401/404/503 |
| GET `/api/v1/me/notification-policies` | listMyNotificationPolicies | — | NotificationPolicyList | 401/503 |
| PUT `/api/v1/me/notification-policies` | upsertMyNotificationPolicy | body {scope, category?, link?} | NotificationPolicySummary | 401/422/503 |
| DELETE `/api/v1/me/notification-policies/{id}` | deleteMyNotificationPolicy | path Uuid | 204 | 401/404/503 |
Existing 5 routes unchanged (summary response gains `muted_unread`; NotificationSummary gains
`muted`). Error envelope stays `{error:{code,message}}` via `RestError`; store internals never
leak (OWASP A05). Authz: person-scoped only — no feature gate (all-employee surface; nav item
is ungated); cross-user access = 404 indistinguishable-from-absent (deny-by-omission, no leak).

### Tests (bar, mirroring existing suites)
- rest/tests: by-object grouping is recipient-scoped over the real router; unread toggle
  cross-user = 404; policy upsert/delete cross-user isolation; muted rows excluded from
  unread-count/summary but present+annotated in list.
- adapter tests as `mnt_rt` (runtime-role pool, `scope_org`): RLS + grants for
  `notification_policies` (INSERT/SELECT/UPDATE/DELETE as mnt_rt; cross-org invisible);
  emit-skips-notifier-when-muted; resolve sweep unaffected by mute.
- `cargo build/test -p` each of the four crates + `openapi_drift` after integrator applies spec.

## 4. Frontend contract (Stage 3 — `web/src/console/notif/**`, exemplar = production module)

- Files: `NotifScreenBody.tsx` (+ `NotifScreen` session-fence wrapper), `NotifConsoleRoute.tsx`,
  `useNotifConsoleAuthz.ts` (JWT-floor → authz projection; notif is ungated so capabilities
  reduce to authenticated-session true; keep the hook for uniformity + future covert scoping),
  `notifCapabilities.ts` (`canRead/canAck/canMute` — all self-scoped), `notifApi.ts` (typed
  over `ConsoleApiClient` + `requireData`, ops: list, byObject, unreadCount, summary, markRead,
  markUnread, readAll, policies list/upsert/delete), `routeContract.ts` (structural fixture),
  `notif.css`, `index.ts`, vitest suites per file. i18n: module-owned `web/src/i18n/notif.ts`
  (`notifStrings`) — NOT the shared `ko.ts`; no inline Hangul in components.
- Screen composition (per design-spec §1b): header (h1 + unread warn chip + 전체/미확인 segment
  + view segment 시간순/개체별 + 「모두 읽음」) · list card (token-segment rows, unread dot,
  category chip, mono time, click=ack+deep-link, swipe/secondary action=read toggle) ·
  by-object group rows (link target, total/unread, latest text, category chips, mute bell
  toggle → policy upsert/delete) · truthful loading/empty/denied/error/offline states.
  className = plain string literals; token colors only; no captions (§4-12).
- Deep-link chain: `link.object{kind,id}` → module object panel route; `link.screen` →
  `consoleScreenPath(screen)` guarded by `isExposedScreenKey` (unexposed target = row still
  ack-able, navigation withheld — fail-closed, no dead control: render as non-link text);
  reuse comms-rail `adapters.ts` mapping instead of re-implementing (§4-19 single chokepoint —
  extract shared mapper if needed within notif/comms-rail ownership).
- Rail parity: comms-rail keeps list+read; badge should move to `summary`/`unread-count`
  (mute-aware) — rail change is comms-rail-owned; recorded as integration note, not this lane's
  edit unless comms-rail is granted.

## 5. Collision-root deltas (integrator-owned; see `integration-manifest.json`)

1. `web/src/console/shell/nav.ts`: add `"notif"` to `MOUNTED_SCREEN_KEYS` (nav item already
   exists, ungated; `EXPOSED_SCREEN_KEYS` unchanged — lands DARK per ADR-0025).
2. `web/src/console/screens/registry.ts`: `notif: NotifScreenBody` (import from `../notif`).
3. `backend/openapi/openapi.yaml`: 5 new operations (§3 table; tag `me`) + `NotificationSummary.muted`,
   `NotificationCountsSummary.muted_unread`, new schemas `NotificationObjectGroup(Page)`,
   `NotificationPolicySummary`, `NotificationPolicyList`, `UpsertNotificationPolicyRequest`.
4. `clients/{ts,kotlin,swift}`: regenerate + commit (3 CI drift gates).
5. Migration renumber: 0196 → next free at consolidation time.
