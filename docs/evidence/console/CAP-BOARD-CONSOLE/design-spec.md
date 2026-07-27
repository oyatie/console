# CAP-BOARD-CONSOLE — design spec (dc.html extract + markdown intent)

Lane: CAP-BOARD-CONSOLE · STORY-BOARD-001 "A notice publishes to scoped audiences with acknowledgment tracking to completion." Route `/console/board`.

Sources (byte-exact design mirror `docs/design/oyatie-console/`, change-log 190):
- `Oyatie Console.dc.html` — board module config L11138–11143, generic module template L16047–16290, detail prog bar markup L4903–4908 + logic L16260–16261, rail notice section L6472–6493, rail state L9057–9061 + L16522/17023–17025, notif seed n5 L9055, ontology type L13794, viewer scr lists L10345–10352, `MOBILE_SCR` L10358, nav item L16344, `modLinkGo` L10835–10848, `MOD_SCREENS` engine merge L10985–11001, type-registry chip L8709.
- ROADMAP.md L88/99/101 (board row — benchmark Confluence·Slack, "✅ 수령확인 진행 바"), TODO.md L170/198, AGENTS.md entries 30 (진행 바), 37 (모듈 서피스 10화면), 41 (§4-19 enum), 151 (NT- 개체화·토큰 정규식), 156/160 (진실성 audit — 게시판 수령확인 e2e), 190 (커뮤니케이션 L2 판정 — 게시판 수령확인 포함).
- HANDOFF.md §1 (CommObject 공지), §3 (수령확인=법적 증빙·InboxDoc), §15 (생애주기 엔진), §16 (가드레일), §18 (온톨로지 엔진 — OT-19), §20 (CRUD 매트릭스).

## 1. Screen identity

| item | value |
|---|---|
| screen key | `board` |
| nav | 커뮤니케이션 group, label 게시판·공지, icon megaphone, **ungated** (every persona v2–v10 has `board`; mobile 7-module set includes it) |
| nav badge | unread notice count, neutral tone (`noticeUnread`) |
| bound object type | 공지 `OT-19`, code prefix `NT-` (header type chip → type card) |
| benchmark | Confluence · Slack |

## 2. Layout zones (generic module surface — MOD_SCREENS grammar, single template)

1. **Header**: title + stat bar (compact 1-row, never KPI cards) + search input (`modQ`, multi-attribute haystack: code·c1·c2·c3·st·en values) + primary action button + config strip (personal view: custom columns/stats/filter presets — §3.9.0-① direct-save whitelist, audited).
2. **List**: shared-track columns `["공지", "게시", "대상", "확인"]` (+ leading code column). All rows one track formula; J/K/Enter keyboard nav; sortable headers; enum chip click = list filter.
3. **Detail** (right split panel or object card per personal `det` config): kv pairs + typed enum chips (`en`, click = filter same value) + **수령확인 progress bar** + link chips + action buttons + 작용 자동화 chips (workflows touching the 공지 type).
4. **Comms rail** (elsewhere-owned): 공지 section — unread dot rows + 더보기 → `board` screen. Rail and main share the same objects/state (rail↔main promotion, DESIGN §4.8).

## 3. Board config (prototype seed, L11138–11143)

- title 게시판·공지 · primary action **공지 작성** (prototype simulates it as mail composer prefilled `전 직원 <all@kos.co.kr>` — kv "발송 · 메일 병행"; the real console's create path is a draft→publish flow).
- stats: `게시 중 4` · `수령확인 진행 1` (warn tone) — every stat is a click=filter drill (dead numbers banned).
- rows (each: code NT-, c1 title, c2 게시 date, c3 대상 audience, st status chip + tone, `en` typed enum, `kv`, `prog`, `links`):

| code | title | 게시 | 대상 (audience) | status/tone | 유형 (en) | prog | kv | links |
|---|---|---|---|---|---|---|---|---|
| NT-0708 | 7월 근무·연차촉진 안내 | 오늘 | 전 직원 | 게시 / ok | 안내 | 0/3 "촉진 대상 수령확인" | 발송=메일 병행 · 연동=촉진 1차 AP-3126 | 발송 메일→mail · 연차 촉진→leave |
| NT-0707 | 취업규칙 개정 통지 | 어제 | 전 직원 | 수령확인 중 / warn | 법정 통지 | 1,192/1,284 "수령확인" | 법정=근로기준법 §94 — 개별 수령확인 | 개인 수신함→inbox · 컴플라이언스→compliance |
| NT-0701 | 2026년 정기인사 명령 | 7/1 | 전 직원 | 게시 / ok | 인사명령 | — | 보존=영구 — 기록물 등재 | 기록물 NT-0701 (code link) |
| NT-0628 | 경비팀 안전교육 (7/8) | 6/28 | **대원강업 현장** | 완료 / neutral | 교육 | — | 일정 · 대상=경비 1반 44명 | 현장 대원강업 (node link) |

**NT-0628 is the scoped-audience evidence**: 대상 is a *scoped* audience (one site/branch), not 전 직원 — the signature story's "scoped audiences" is design-anchored, not invented.

## 4. Progress bar contract (수령확인 진행 바 — AGENTS 30, generic `prog` field)

- Row `prog: {done, total, label}` → detail renders a labeled bar: `pct = round(done/max(1,total)*100)`, width `pct%`, fill `--ok-solid` when pct ≥ 100 else `--warn-solid`, label `"{label} {done:,} / {total:,} ({pct}%)"` (e.g. "수령확인 1,192 / 1,284 (93%)").
- "Tracking to completion": 100% flips tone to ok; NT-0707's st chip is `수령확인 중`(warn) until complete.
- The generic `prog` field is reusable by any module (제네릭 — already implemented in `web/src/console/module/config.ts` as `field: {kind:"prog"}`).

## 5. Typed enum 유형 (§4-19 field-type discipline)

`en: [["유형", …]]` values observed: **안내 · 법정 통지 · 인사명령 · 교육**. Enum chip in detail → clicking filters the list to the same value; enum participates in search/J-K filter and analytics. Free-text 유형 would violate §4-19.

## 6. Ontology (OT-19 공지, dc.html L13794)

- props: `유형 enum · 게시 date · 대상 text · 수령확인 text · 상태 lifecycle`
- linkTypes: `수령 대상 → 직원 (1:N)` · `기록물 등재 → 기록물 (1:1)`
- actions: `board → 게시판·공지` (navigate)
- analytics: `ack — 수령률 = 확인 ÷ 대상`
- Type-registry entry (L8709): OT-19, stage active, steward 경영지원팀, note "NT- · 수령확인 진행 — 법정 통지 연동".

## 7. Lifecycle & governance intent

- §3.9 standard pipeline applies; a notice's operative FSM in the prototype is **draft → 게시(published) → (완료 = ack complete) → 기록물 등재(archive)**. Publishing is a 중대 액션 → §4-29 preflight (fail-closed: required fields, audience non-empty) applies. No unpublish; no hard delete (보관=숨김).
- NT- code is issued at publish (draft has none). Publish = audit event + per-recipient snapshot + notification fan-out (rail pointer per recipient, notif cat 공지, link = NT- code — n5 seed).
- **법정 통지 (legal) notices** additionally route through the personal Inbox passkey receipt flow (HANDOFF §3: `legal && !confirmed` → passkey → confirmed = legal evidence; AP- 연동). The board's 수령확인 bar for NT-0707 mirrors compliance row CP-015 (진행 1,192/1,284). Board-level ack (this lane) is the plain acknowledgment; passkey/InboxDoc is a separate owning lane (inbox) — the board links to it, never re-implements it.
- 열람/확인 all audited (§3.10-⑥); ack is recipient-scoped self-action.

## 8. Interaction grammar to honor (charter §4/§4.7)

- Every noun clickable/pinnable: NT- code chips are drag sources (`objDrag`) and token-linkable (`!NT-…` — msgParts regex includes NT since AGENTS 151); links/acts route through one `modLinkGo`-style dispatcher.
- Status = chips (게시/수령확인 중/완료), tones ok/warn/neutral; numbers/codes mono; exceptions only (0 hidden or —).
- No explanatory captions/subtitles/meta text; only action-driving copy.
- Stat bar entries and enum chips are filter drills (no dead numbers).
- Empty state = reason + next action in one line.
- Keyboard: J/K/Enter list nav; Esc closes panels; readability floor on column resize.
- Deny-by-omission: manager-only affordances (draft rows, publish button, progress/receipts) simply do not render for non-managers.
