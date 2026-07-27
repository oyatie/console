# CAP-DIRECTORY-CONSOLE frontend — Stage 3 fresh-eyes verification

Verifier: independent adversarial pass over commits `7cef9a5b..0f863349` (stage 2 build), 2026-07-24.
Scope: `web/src/console/directory/**`, `web/src/i18n/directory.ts`, frontend manifests.
Method: re-extracted the design authority section, re-derived the API contract from
`clients/ts/src/schema.d.ts` + `backend/openapi/openapi.yaml` + the messenger backend crates,
read every module file, re-ran every gate. Findings fixed in the verification commit.

## Verdict

PASS with 5 findings, all fixed and regression-tested in this stage (see "Findings"). No stub,
placeholder, TODO/FIXME, skipped test, dead control, or fabricated datum found. All gates green
(vitest 21/21, eslint, tsc, check-console-purity 402 files, check-ui-strings for this module).

## Module completion contract (9 points, point-by-point)

1. **List/overview layer** — PASS. Module-surface grammar: compact stat bar (구성원/임직원,
   clickable, mono values), 검색, 새 대화 primary action, sticky sortable header (members),
   ellipsis grid cells, chip slot (법인 on employee rows), chevron, empty state with 필터 해제.
   Two stat-switchable segments (branch roster / HR register) because no user↔employee join key
   exists (`Employee.identity_name_only_merge` forbids name merges) — documented GAP-DIR-1/-3.
2. **Object detail layer** — PASS. 인사 카드 right pane: member card via the read-audited
   endpoint (열람 기록됨 chip, 본인 self state); employee card with 사번/법인/재직 chips,
   kv rows (직책·직무·소속·근무지·입사·퇴사·기준 지점), drill chips.
3. **Action/workflow layer** — PASS. 메시지 = real `POST /api/messenger/threads {kind:"dm"}`
   (server audits `message_thread.create`) then hand-off to `/console/messenger?thread=<id>`
   (verified `MessengerScreenBody.tsx:62` consumes `thread`). Busy/error/retry states real.
4. **History layer** — PASS for employees: 이력 ledger = `GET /api/v1/employees/{id}/lifecycle-events`
   (event chip, effective date, from→to transition, comment; enum-total labels). Member cards have
   no history surface because no member-history endpoint exists — truthful omission, not a dead zone.
5. **≥2 upstream + ≥2 downstream links** — PASS with one honest qualification. Upstream: 소속
   drill (member + employee cards), 법인 drill (server-filtered `company=` reload) — both verified.
   Downstream: DM thread hand-off into messenger (cross-module, solid) + lifecycle-event ledger
   rows. Ledger rows are visible objects but not clickable — no lifecycle-event detail surface
   exists anywhere in the console to link to; a link would be a dead control.
6. **Server-enforced deny-by-omission, no leakage** — PASS. Capability projection mirrors backend
   tiers exactly (six messenger roles + branch for the roster tier; `employee_directory_read` or
   ADMIN/EXECUTIVE/SUPER_ADMIN for the register — matches `nav.ts:36,57`); zero fetches without
   capability (tested); 403 roster renders denial copy, not an error; 403/404 person reads render
   the identical no-leak blocked copy (tested); server-denied register drops the whole HR
   enhancement and (fixed this stage) clears a restored employee selection; denied ledger renders
   as absent. The privileged `GET /api/v1/employees/{id}` (`employee_directory_manage`,
   compensation+phone) is correctly NOT called by this read-tier module.
7. **Keyboard/focus/contrast/Korean-expansion/responsive** — PASS. listbox + roving tabIndex,
   ArrowUp/Down + j/k + Home/End (Enter = native button click), focus-visible outlines on all 9
   interactive classes, token colors only, flex-wrap + minmax/ellipsis + 720px media query.
   Search input accessible name fixed this stage (aria-label moved onto the `<input>`).
   Browser-level visual pass still owed at exposure review (jsdom cannot assert layout).
8. **Selection survives refresh/retry/Back** — PASS. `person` search param (`m:`/`e:`),
   restored on mount, replace-synced on change; session-fenced remount on tenant/branch/actor/api/
   capability change; stale responses fenced by request tokens (tested). Employee restore now has
   truthful loading/error card states (fixed this stage). Ceiling: an employee beyond the loaded
   register pages cannot be re-resolved after refresh (no read-tier by-id endpoint) — the
   selection is truthfully dropped rather than guessed.
9. **Truthful states throughout** — PASS. Denied-before-fetch, loading, empty+clear-filter,
   error+retry (roster, register, card, ledger, DM), blocked, busy — every visible datum is a
   real authorized response or one of these states.

## Design fidelity (re-extracted from `Oyatie Console.dc.html`)

Compared against: module-surface template (line 4555+), `MOD_SCREENS.directory` (line 11219),
`dirRows` (line 11016), `openPerson`/person-visibility policy (line 14168+).

Matches: stat-bar grammar (label 10.5px/700/steel + mono value 13.5px/800), search box, primary
새 대화 action, sticky grid header with click-sort affordance, row grammar (mono code cell, name,
steel cells, chip slot, chevron, `draggable` with "클릭: 상세 · 드래그: 입력창에 참조 첨부"),
empty state ("검색 결과 없음" + "필터 해제 — 전체 목록"), detail pane (chips header, 14.5px title,
86px kv grid, enumerated-value drill chips "같은 값으로 목록 필터"), purple filter chip, audited
person-open semantics (design logs 열람 permit/deny; real module rides the server's `person.view`
audit + 404 deny-by-omission — confirmed in the openapi operation description).

Documented deviations (all deliberate, all backend-gap-driven, recorded in `fidelity.json`):
연락처 column + 메일 action (no ext/email in any DTO — GAP-DIR-2); single unified people list
(GAP-DIR-1/-3); 법인 count stat (GAP-DIR-4); design's stats show 임직원 1,284 / 법인 4 — built
stats are live 구성원 (loaded roster) + 임직원 (`EmployeePage.total`); cfg/시트 header buttons,
출입카드, 평가 rows, 최근 업무·KPI, 민감정보 gate, 인사 관리 모드, 휴대폰 edit (owning charters);
member rows carry no 직책/법인 (not in `MessengerMemberSummary`); employee headers not sortable
(server-paged partial list must not imply a global order).

## API contract fidelity (field level, re-derived from schema.d.ts)

- `MessengerMemberSummary { id, display_name, team }`, `MessengerMemberListResponse.items` — match.
- `listMessengerMembers` query `{ branch_id, limit }` — match; **no offset/total** → single-page
  ceiling documented in the transport (finding 5).
- `getMessengerMember` `{ path.userId, query.branch_id }`, 401/403/404 — match; openapi description
  confirms `person.view` audit on non-self reads and 404-no-audit for out-of-scope targets.
- `CreateMessengerThreadRequest { branch_id, kind, member_ids }`, kind `"dm"` valid — match.
- `listEmployees` query `{ company?, search?, limit, offset }` → `EmployeePage { items, total, limit, offset }` — match.
- `Employee` — every rendered field exists; `status` is `string | null`, unknown values fall back
  to the raw string (truthful); nullable fields render `—`.
- `EmployeeLifecycleEvent.event_type` enum = exactly `ONBOARD|OFFBOARD|TERMINATE|TRANSFER` →
  `text.event` is total; `comment` non-nullable; `effective_date` required — match.
- Error envelope `ErrorBody { error: { code, message } }` — matches `directoryApi.message()`.
- No repeated-query params, no N+1 (one list per segment + one detail read per selection).

## Findings (all fixed this stage)

1. **Stuck blank pane on denied register with restored `e:` selection** — a deep link
   `?person=e:<id>` under a server-denied register left the selection set forever and the detail
   pane empty of any state. Fixed: denied register clears employee selection (and thus the URL
   key). Regression test added.
2. **Blank pane during/after register load for a restored `e:` selection** — no loading state
   while the register page was in flight, and no state at all when the register load failed.
   Fixed: 인사 카드 로딩 state while loading; retryable card alert on register failure.
   Regression tests added (2).
3. **Search input accessible name** — `aria-label` was on the `<label>` wrapper (ignored by AT);
   the input's only name source was its placeholder. Fixed: aria-label on the input.
4. **Dead copy** — `companyFilterLabel` was defined in the i18n resource but never rendered.
   Removed (resource + i18n.json manifest).
5. **Undocumented roster ceiling** — `listMembers` fetches one page of 100 with no paging
   affordance possible (endpoint has no offset/total); the 구성원 stat counts the loaded roster.
   Same ceiling as the exemplar `composer/candidates.ts`, which documents it; now documented in
   the transport and in `fidelity.json` (GAP-DIR-5).

Reviewed and accepted as-is: the purple 법인 filter chip stays visible in the header while the
구성원 segment is active — it describes the register object space and keeps the (filtered)
임직원 stat and the chip coherent as a pair; hiding it would leave an invisible armed filter.

## Gate evidence (this stage, after fixes)

- `npx vitest run src/console/directory` — 21/21 pass
- `npx eslint src/console/directory src/i18n/directory.ts` — clean
- `npx tsc --noEmit` — exit 0
- `node scripts/check-console-purity.mjs` — 402 files clean
- `node scripts/check-ui-strings.mjs` — no directory violations; exits 1 only on the pre-existing
  committed violation `web/src/features/facilities/FacilitiesWorkflowPage.tsx` (fd93fbdd, outside
  this lane — verified via `git log` that this lane never touched it)
- `git diff --stat` vs base — 11 files, all inside ownership roots + the declared
  `web/src/i18n/directory.ts` (ko.ts, nav.ts, registry.ts, openapi, clients untouched)
- Stub scan (`TODO|FIXME|XXX|test.skip|.only|cn(|clsx`) — zero hits; Hangul in components only in
  comments; all rendered copy in the i18n resource
