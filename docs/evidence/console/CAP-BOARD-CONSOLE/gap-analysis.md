# CAP-BOARD-CONSOLE — gap analysis (built vs STORY-BOARD-001)

Story: "A notice publishes to scoped audiences with acknowledgment tracking to completion."

## What is ALREADY BUILT (do not rebuild)

Backend — `backend/crates/notices/{domain,application,adapter-postgres,rest}` is a complete 4-layer vertical, **mounted** in `backend/app/src/lib.rs` (route-inventory entry `notices` + `.merge(console_notices_rest::router(...))`), openapi'd (`/api/v1/notices*`, tag `notices`, schemas `NoticeSummary`/`CreateNoticeDraftRequest`/`NoticeProgress`):

- Tables (migration `0162_create_notices.sql`): `notices` (org-RLS, draft/published CHECK, title ≤300 / body ≤20000, NT- `code` nullable until publish, no DELETE grant) + `notice_receipts` (UNIQUE(notice_id, recipient_user_id), `acknowledged_at`, progress + recipient indexes, org-RLS, no DELETE).
- Authz: `Feature::NoticeManage` = `notice_manage`, role matrix `[D,D,D,A,A,A]` (ADMIN·EXECUTIVE·SUPER_ADMIN), checked via `authorize_org_wide` (requires `BranchScope::All` — a branch-scoped ADMIN is NOT a notice manager).
- Flows: create draft (manager, audited `notice.create_draft`); get/list (drafts deny-by-omission → 404/omitted for non-managers); publish (FOR UPDATE status guard, 409 if already published, `issue_code(kind="notification")` → NT-, **org-wide** recipient snapshot into `notice_receipts` in one audited insert, best-effort post-commit notification fan-out — category 공지, `NotificationLink::Object{kind:"notice"}`, dedup key); ack (recipient bound from JWT principal, idempotent `COALESCE(acknowledged_at,$3)`, 404 when not a snapshotted recipient — cross-user isolation idiom); progress (manager-only done/total).
- Tests: `rest/tests/api.rs` (real router, ES256 JWT, publish-tier 403s, draft 404 leakage checks, recipient scoping) + `adapter-postgres/tests/notices_rls_surfaces_as_runtime_role.rs` (console_rt).

Frontend — comms rail (`web/src/console/comms-rail/`) already lists published notices (`GET /api/v1/notices` in `transport.ts`; adapter maps to rail cards, currently `unread: false` hardcoded). Generic module surface exists: `web/src/console/module/config.ts` (`ModuleConfig` — MOD_SCREENS grammar; `lanes` + `prog` fields IMPLEMENTED) + one generic `ModuleScreen`. Nav already declares `{screen:"board"}` ungated in the comms group and `ko.ts` already has `console.shell.nav.board: "게시판·공지"`. No `web/src/console/board/` module exists; `board` is not in `MOUNTED_SCREEN_KEYS`/`SCREEN_REGISTRY`.

## GAPS to close (this charter)

### G1 — Scoped audiences (backend, the story's core)
`publish()` snapshots **every** active org user; 0162's own comment marks narrower scoping as future work. Design evidence NT-0628 (대상 = 대원강업 현장). The only org-membership primitive in this schema is `user_branches` (user↔branch M:N; `branches` under `regions`; "sites" here are customer sites, not employment units) → **audience granularity = org | branches** (list of branch ids). Decision: model as `notices.audience_scope ∈ {org, branches}` + `notice_audience_branches` join table; publish snapshot joins `user_branches` for scope=branches; audience frozen at publish; empty effective audience = 422 (fail-closed preflight, §4-29).

### G2 — 유형 typed enum (§4-19)
No category column/field. Add `notices.category ∈ {general, legal, hr_order, training}` (안내·법정 통지·인사명령·교육), default `general`. Powers the 유형 chip, filter drill, and (later) legal→InboxDoc routing; that passkey routing itself is the inbox lane's scope, NOT this lane's.

### G3 — Caller's own ack state (frontend cannot render 확인 column or badges without it)
List/get return no per-caller receipt state. Rail badge (`noticeUnread`), 확인 column, and the "내 수령확인" action need `my_receipt` on `NoticeSummary` (LEFT JOIN on the recipient index — no N+1). Enables mywork/rail "수령확인 필요" derivations too.

### G4 — Per-row progress for managers (list = tracking board)
`/progress` exists but only per-notice; the board list's 확인 column and "수령확인 진행" stat need done/total per row without N+1 → embed optional `progress` in `NoticeSummary` for NoticeManage callers (single grouped subquery). Keep the existing endpoint (detail refresh).

### G5 — Audience display (대상 column)
Summary has no audience fields. Add `audience_scope` + `audience_branches: [{id, name}]` to `NoticeSummary` (join `branches.name`).

### G6 — Draft editing (§3.9.0-③ draft-stage direct edit)
No update path; a typo'd draft today must be abandoned (drafts also can't be deleted — no DELETE grant). Add `PATCH /api/v1/notices/{id}` (manager, draft-only, 409 once published) covering title/body/category/audience.

### G7 — Receipts drill (history layer / "tracking to completion")
Progress alone can't answer "who is outstanding". Add manager-only `GET /api/v1/notices/{id}/receipts` (filter `acknowledged=`, keyset/limit paging, recipient display name from `users`). This is the module's history layer + a downstream traversable link (notice → 직원, matching OT-19 `수령 대상 1:N`).

### G8 — Frontend module (entire /console/board surface)
Build `web/src/console/board/**`: no-props `BoardScreenBody` (registry `ComponentType` contract) → `ModuleConfig<NoticeRow>` specialization of the ONE generic ModuleScreen (§4-18: never fork the template; `prog` field already implemented), module-owned strings file (`web/src/i18n/board.ts`, mechanism = `i18n/production.ts` exemplar), typed api module over generated client paths, capabilities from the policy gate (`notice_manage`), route/session fencing per the production exemplar. Registry/nav/ko.ts/openapi/clients edits belong to the integrator → manifest JSON in this dir.

## Explicit NON-gaps (deliberately out of scope)
- Passkey/InboxDoc legal receipt flow (HANDOFF §3) — owned by the inbox lane; board links to it (NT-0707 → 개인 수신함).
- 기록물 등재 (archive to docs registry), mail 병행 발송, AP- 연동 chips — cross-module links recorded as link chips only where the target module ships them.
- Full §3.9 archived/disposed states — the crate's one-way draft→published stands; archive belongs to the docs/records lane.
- Site-level audiences — no employment-site membership primitive exists in the schema; branch granularity is the truthful maximum today (recorded in design-contract §FSM notes).
