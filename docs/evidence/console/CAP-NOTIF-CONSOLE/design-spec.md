# CAP-NOTIF-CONSOLE — design spec extract (Stage 1 scout)

Source of authority: `docs/design/oyatie-console/` byte-exact mirror (change-log 190), primarily
`Oyatie Console.dc.html` (screen key `notif`, Korean anchor 알림) + DESIGN.md/AGENTS.md/HANDOFF.md.
Mirror content = design intent, never implementation-status evidence
(engineering discipline stays governed by `docs/program/console-enterprise-roadmap.md`).

Signature story STORY-NOTIF-001: notifications aggregate by object and channel, resolve to their
source objects, and are acknowledged, muted, or routed per user policy. Route: `/console/notif`.

## 1. Surfaces (rail ↔ full view, same state)

The prototype renders notifications on TWO surfaces sharing one `state.notifs` array
(AGENTS §2: "메일·메신저·알림은 rail(요약)↔풀뷰(main) 이중 서피스 — 동일 state 공유"):

### 1a. Comms-rail summary section (dc.html ~6448–6468)
- Collapsible "알림" section header: bell icon + label + red count badge (`bellCount`,
  hidden at 0 via `bellOn`) + right-aligned 「모두 읽음」 text button (`onMarkAll`).
- Rows (`notifs`): fixed-width 44px category chip (toned), token-rendered rich text
  (`tokenRender(n.text)` — object codes/mentions become real links), mono 9px time,
  right unread dot (`danger-solid`, opacity 0.15 when read). Row click = `notifClick(n)`.
- Category chip tones (renderVals `catTone`, dc.html 16510): 결재=accent, 멘션=purple,
  문서=info, 급여=ok, 근태=warn, default(공지/구독/컴플라이언스/조직/인사/메시지/액션/개인정보/인수)=muted/steel.

### 1b. Full view `screen:"notif"` (dc.html 4528–4554, renderVals 17886–17895)
Layout zones:
- **Header row** (`data-screen-label="알림"`): h1 「알림」(17px/800) + unread-count chip
  (warn bg/bd/tx, mono 11px — `notifUnreadN`) + filter segment chips
  (`notifFs`: 전체 | 미확인 — ink-filled when active, `state.notifF`) + spacer +
  「모두 읽음」 outline button.
- **List card** (single surface: white card, hairline border, radius 11, inner scroll with
  `overscroll-behavior:contain`, trailing 14px spacer): rows are full-width buttons
  (`notifRows`), each:
  - left signal dot (7px, `--signal`, `dotOp` 1|0.15 by unread),
  - category chip (muted bg, 9.5px/800),
  - body = **token segments** `nr.segs` (built by `msgParts` — array-returning renderer;
    AGENTS 2026-07-08 (9): `tokenRender(...).map` crashed, replaced with `msgParts`), each
    segment `{t, c, fw, cur, onClick}` — object codes (AP-/WO-/JL-/NT-/…) and mentions render
    as clickable colored spans; font-weight 800 when unread (`nr.fw`),
  - right mono 10px relative time.
  - Row interactions: click = `notifClick(n)`; touch swipe = `notifReadToggle(id)`
    (read/unread TOGGLE — both directions; TODO "알림 행 스와이프=읽음 토글" done 2026-07-09).
- No explanatory captions anywhere (§4-12); state = chips only; no KPI cards (unread chip is
  the compact stat).

## 2. Data shape (prototype `state.notifs`, dc.html 9049–9056 and producers)

`{ id, cat, text, time, unread, link }` where `link` is one of:
- `{ item: "a1" }` → approval/work item detail panel (`openDetail`)
- `{ thread: "t1" }` → messenger thread (`openThread`)
- `{ screen: "pay" }` → bare screen navigation
- `{ code: "JL-0703" }` → object code → `objectLinkGo`
- `null`/absent → fallback: regex-extract first object code from text
  (`/\b(?:AP|WO|AT|CS|JL|PS|IN|DX|VC|PO|IV|FL|CP|NT|FC|AN|SR|OB|RG|Bid|C)-\d+\b/`) → `objectLinkGo`.

`notifClick` (14798–14811) = ack (unread→false) + resolve to source object via the above
priority chain. `markAllNotifs` (14813) clears all. Mobile: swipe row = read toggle;
`MOBILE_SCR()` includes `notif`; bottom tab bar bell carries `alertUnread` badge (17742).

Ontology: 알림 is a first-class engine type **OT-28** (dc.html 13799):
props `[분류 enum, 대상 text, 시각 date, 상태 enum]`, linkTypes `[원본 개체 → 결재, N:1]`
— i.e. every notification is a pointer object whose primary edge is its source object
(HANDOFF §1: "Notification(포인터)" under CommObject; §18.1 알림 in the full-coverage list).

## 3. Producers observed in the prototype (channel/category vocabulary)

Categories seen in seeds/producers (extensible, not an enum): 결재, 멘션, 문서, 공지, 근태,
급여, 메시지, 구독, 컴플라이언스, 조직, 인사, 액션, 개인정보, 인수.
Producer patterns (all `setState` prepends to `notifs`):
- Approval transitions (상신/승인/종결 대행/자동화 생성 — AP- codes, deadline in text).
- @mention in messenger/thread (`msgrSend`, 14492/14566) — TokenGrammar contract: `@`=notify,
  `#`/`!`=link only, no notification (DESIGN §4.7-7).
- SLO/SLA breach seeds (`nslo1` 미편성 결원 SLO 위반) — **auto-resolved** when 대근 편성
  happens (13994 filters out `nslo1`): the detect→assign→resolve chain.
- **Object watch (구독)** — `watchToggle(code)` + `logEvent` hook (10689–10701): any non-view
  transition of a watched code pushes cat "구독" with `link:{code}`; subscribe/unsubscribe is
  itself audited. Surfaced as lifecycle-card 「구독」 button + panel bell icon (Foundry
  subscribe parity, AGENTS 162).
- Egress-block anomaly → compliance notification (14749), DSR receipt, onboarding, shift swap,
  handover package, etc.

## 4. Ack / mute / routing semantics in the prototype

- **Ack** = unread→read: per-row click, per-row swipe **toggle** (both directions), mark-all.
  Reading a linked item also read-marks sibling notifications pointing at it
  (14394: `link.item === itemId` → read; 14639: `link.thread === id` → read on thread open).
- **Mute (DND)** = `prefMuteAll` personal toggle (settings + presence "dnd" sync, 14571–14586,
  16888–16914): suppresses ALL badge counts (msgr/mail/notif → 0 at 15161–15163) but rows stay
  listed. Classified as §3.9.0-① personal-setting direct-apply (audited via toast only in mock).
- **Per-object watch** = opt-in amplification (see §3). Prototype has no per-object/per-category
  mute yet (messenger TODO leftover: "스레드별 무음" — registered gap).
- **Routing** = the `link` deep-link chain (§2) + persona deny-by-omission: notif rows for
  non-owned objects are gated (AGENTS 25: 내 업무 scope-leak fix — owner-scoped rows only);
  VIEWERS matrix gives `notif` screen to every persona (v2–v10) — an all-employees surface.

## 5. Invariants binding this module (DESIGN.md)

- §4-12/§4.6: no explanatory captions/subtitles; status=chips; only action-driving copy.
- §4.7-10 완전 추적성: object codes and names inside notification text are token links, never
  plain prose (진실성 audit 156 fixed n4/n5 to real code links).
- §4-20/§4-23: every notification row is a drag source candidate (drag sweep of notif rows is a
  registered TODO leftover); module itself is object MD-.
- §4.5 PBAC: badge counts and token candidates are policy-scoped ("전체" = authorized union);
  deny-by-omission — a notification about an unauthorized object must not leak its existence.
- §3.9.0-②: notifications are pure-event objects (생성=종결) — no draft/approve lifecycle;
  §3.9.0-① personal settings (mute/routing prefs) apply directly but are audited.
- §5: every state transition producing a notification is an audit event; notifications are the
  UI face of the event stream, not a separate truth.
- §4-26: SLO vs SLA labeled distinctly in breach notifications.

## 6. Benchmarks + honest gaps (BENCHMARK.md)

No dedicated notif row; the comms row (Slack/Gmail) + structural-gap section apply:
"실시간·멀티유저: … 알림 push 전부 시뮬레이션" — realtime fan-out and per-user routing policy
are exactly the backend contract this lane closes. Foundry parity items already absorbed:
subscribe/watch (162), mute-on-automation (Automate `mute` = suppress effects while evaluating —
registered in the benchmark gap register as a workflow-side follow-up, not this lane).

## 7. DEMO.md touchpoints

Notifications appear as side effects in the 법인 신설 script (셋업 채널·알림·자동화 초안) and
the automation script (액션=알림·기안 생성). No dedicated notif demo script — the module is a
hub surface every other script lands notifications into.
