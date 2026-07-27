# CAP-NOTIF-CONSOLE — gap analysis (built backend/frontend vs STORY-NOTIF-001)

## A. What already exists (verified in this worktree)

### Backend — `backend/crates/notifications/{domain,application,adapter-postgres,rest}`
- **Domain**: `NotificationCategory`/`NotificationKind`/`NotificationBody` validated newtypes
  (free-form ≤64/64/2000, matching DB CHECKs); `NotificationLink` = tagged enum
  `Object{kind,id} | Screen{screen}` (JSONB wire shape).
- **Application ports**: `NotificationSink::emit` (write port producers hold — messenger
  @mentions #198, notices fan-out, workflow compensation drain already wired in
  `backend/app/src/lib.rs` 2709–3035); `NotificationNotifier` (post-commit realtime, IDs only,
  ADR-0007); `NotificationResolver::resolve_by_link` (generic detect→assign→resolve sweep).
- **Store** (`PgNotificationStore`): `emit_notification` (dedup_key at-most-once via partial
  unique index + Dedup sentinel; audited via `with_audit`; notifier fired only for new rows),
  `list` (keyset (created_at,id), clamp 1..=200, fail-closed foreign cursor), `unread_count`,
  `summary` (per-category unread), `mark_read` (NotFound for cross-user = isolation),
  `mark_all_read`, `resolve_notifications_by_link` (link-equality sweep, audited).
- **REST** (`/api/v1/me/*`, principal from JWT, never request input): GET list /
  unread-count / summary; POST {id}/read, read-all. Already in `backend/openapi/openapi.yaml`
  (tag `me`, operationIds listMyNotifications / getMyUnreadNotificationCount /
  markAllMyNotificationsRead / markMyNotificationRead / getNotificationsSummary).
- **Schema**: migrations `0099_create_notifications.sql` (RLS org_isolation FORCE, mnt_rt
  SELECT/INSERT/UPDATE no DELETE, dedup partial unique, recipient-unread index) +
  `0161_notifications_kind_and_resolution.sql` (kind, resolved_at/by, unresolved partial index).
- **Tests**: `rest/tests/api.rs` (real-router HTTP person-scoping, ES256 JWT, sqlx::test),
  `adapter-postgres/tests/notifications_rls_surfaces_as_runtime_role.rs` (mnt_rt runtime-role
  RLS proof per `rls-verify-as-runtime-role` discipline).

### Frontend
- `web/src/console/comms-rail/**` already consumes GET list + POST {id}/read (transport.ts) and
  maps `link.screen` deep-links (adapters.ts SAFE_SCREENS). Rail = summary surface.
- `web/src/console/shell/nav.ts` already declares `{ screen: "notif",
  labelKey: "console.shell.nav.notif", icon: "bell" }` (comms group, **ungated**) — but `notif`
  is NOT in `MOUNTED_SCREEN_KEYS`, not in `SCREEN_REGISTRY`, and `EXPOSED_SCREEN_KEYS`
  stays `["sales"]` (ADR-0025 — new body lands DARK).
- Exemplar conventions to copy: `web/src/console/production/**` (Screen/Route split, session
  fence re-mount key, capability projection from `useXxxConsoleAuthz` → `derive*Capabilities`,
  typed api module over `ConsoleApiClient` + `requireData`, module-owned `routeContract.ts`
  fixture, module-owned i18n file `web/src/i18n/production.ts`, plain-string classNames,
  css file per module, vitest suites per file).

## B. Story decomposition vs gaps

STORY-NOTIF-001: "Notifications **aggregate by object and channel**, **resolve to their source
objects**, and are **acknowledged, muted, or routed per user policy**."

| Story clause | Built today | Gap (this program must close) |
|---|---|---|
| Aggregate by channel (category) | `summary` per-category unread | none (backend); full-view UI missing |
| Aggregate by object | — | **G1**: group-by-`link` read path (one row per source object: total/unread/latest/categories) + index |
| Resolve to source objects | `link` stored + rail deep-links; `resolve_by_link` sweep | **G2**: full-view deep-link chain (item/thread/screen/code + text-code fallback) is frontend-only today; sibling-read ("open object ⇒ read-mark its notifications") not exposed as an API |
| Acknowledged | mark_read / read-all (one-way) | **G3**: mark-**unread** (prototype swipe is a toggle) |
| Muted | — (prototype: `prefMuteAll` DND only, client-side) | **G4**: per-user mute policy — scope all / category / object — persisted, badge- and realtime-effective |
| Routed per user policy | hardcoded: rows + realtime always | **G5**: routing = mute policy applied at (a) unread/summary counts, (b) realtime notifier fan-out, (c) list annotation; DND(all) is the same mechanism |
| Full view UI (`/console/notif`) | rail only | **G6**: `web/src/console/notif/**` screen body (header chip/filters/모두 읽음, token-segment rows, swipe/click ack, by-object grouping, mute controls) + registry/nav/i18n wiring (manifest) |
| Watch (구독) amplification | — | **out of scope for this lane** (emit-side hook belongs to audit/ontology transition producers; registered as follow-up so the design stays compatible: watch = `notification_policies.action='watch'` extension point) |

## C. Design decisions (owning crates, mechanism)

1. **No new crate.** All gap closure extends `backend/crates/notifications/*` (domain shapes in
   `mnt-notifications-domain`/`-application`, SQL in `-adapter-postgres`, routes in `-rest`);
   DDL in `backend/crates/platform/db/migrations`. Producers keep depending only on the
   application ports (dependency arrow unchanged).
2. **Aggregation is a read path, not a schema change.** `GROUP BY link` over the existing JSONB
   column; provisional migration adds only a B-tree index
   `(org_id, recipient_user_id, link)` to back it (JSONB equality/grouping is well-defined).
3. **Mute/routing = one small personal-policy table** (`notification_policies`), evaluated at
   **read time** for counts (muting an object silences its existing unread — matches user
   expectation and the prototype's badge math) and at **emit time** only for skipping the
   realtime notifier (rows are always persisted — mute suppresses attention, never data;
   §3.9.0-② the notification object itself stays immutable-ish/append-only).
   Personal settings are §3.9.0-① direct-apply but every policy change is audited.
   DND-all = `scope='all'` row — one mechanism, no boolean special case.
4. **Ack toggle**: add `mark_unread` symmetric to `mark_read` (clears `read_at`? No — keeps
   `read_at` as "first read" forensic timestamp, sets `unread=true`; audit action
   `notification.unread`). Cross-user = NotFound, same isolation contract.
5. **Deny-by-omission unchanged**: recipient scoping in code + org RLS; policies are
   self-owned rows behind `/me/`; no admin surface in this lane.
6. **Migration number 0196 is provisional** (charter-assigned; head here is 0180; numbers
   collide across lanes — consolidation integrator takes the next free number before push, per
   `openapi-client-drift-gate` memory).
7. **OpenAPI + generated clients are collision roots** — spec deltas recorded in
   `design-contract.md` §5 and `integration-manifest.json`; integrator applies + regenerates
   ts/kotlin/swift (every operation carries a `tags:` per the client-tagging memory; these are
   `/me/*` routes → tag `me`).

## D. Module-completion-contract mapping (console-enterprise-roadmap)

- **List/overview layer** = full view rows + by-object groups + category summary chips.
- **Object detail layer** = a notification resolves INTO its source object (pointer object —
  detail is the source object's panel; by-object group expands to its notification history).
- **Action/workflow layer** = ack/toggle/read-all, mute/unmute (policy CRUD), deep-link open.
- **History layer** = read/resolved timestamps on rows; every ack/mute/emit/resolve is an
  audit event (`with_audit`) visible in the audit module.
- **≥2 upstream links**: source object (`link` → AP-/WO-/JL-/… panel), producing thread/mention.
  **≥2 downstream links**: audit events for the notification actions, mute policy object;
  screen deep-links into pay/att/etc.
- **Survivability**: filter segment + cursor + selection live in URL/search-params or module
  state per exemplar; unread state is server truth so refresh/Back is lossless by construction.
