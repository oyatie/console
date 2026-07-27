# CAP-FIELD-CONSOLE — design spec extract (고객·현장 / field)

> Source of authority: Claude Design "B2B SaaS Console Design", byte-exact mirror
> `docs/design/oyatie-console/` (change-log 190, synced 2026-07-24). This file is an
> ENGINEERING EXTRACT of design intent for screen key `field`; it is not
> implementation-status evidence. Implementation discipline stays governed by
> `docs/program/console-enterprise-roadmap.md` (ADR-0025 — all bodies DARK until
> exposure-approved; `EXPOSED_SCREEN_KEYS` currently `["sales"]`).

## 1. Module identity

| Item | Value | Mirror anchor |
|---|---|---|
| Screen key | `field` | AGENTS §2 (모듈 서피스 10종), dc.html `MOD_SCREENS().field` (line ~11064) |
| Title | 고객·현장 | dc.html `field: { title: "고객·현장" … }` |
| Nav group | 현장 운영 (`fieldOps`), icon `mapPin` | `web/src/console/shell/nav.ts` lines 216–236 (already declared, gate `OPERATIONAL_ROLES × work_order_read_all`) |
| Route | `/console/field` | lane charter STORY-FIELD-001 |
| Benchmark | ServiceNow FSM · Salesforce Field Service | ROADMAP module matrix rows 86/98 |
| Honest state in prototype | 🟡 v1 module surface (계약·근태·CS- 연동); deep FSM slice = P3 | ROADMAP rows 86, 98, 116 |
| Object codes on screen | `ST-`(현장) `CL-`(거래처) `C-`(계약) `WO-`(정비) `CS-`(회신) `IN-`(접수) `JL-`(업무일지) `SUP-`(티켓) | dc.html field rows; AGENTS §4 code table |
| Personas with `field` in scope | v1 운영 관리자, v3 현장 반장, v10 CX·영업 | dc.html VIEWERS (lines 10346, 10352) |

## 2. Layout zones (generic module surface — MOD_SCREENS grammar, AGENTS 2026-07-08 (3))

1. **Header**: screen title + compact 1-row stat bar (§4-11 — NO big-number KPI
   cards) + multi-attribute search input + one primary action button.
2. **List pane**: shared-track table (all rows share one px+fr track formula; no
   per-row max-content). Columns for `field`: `현장 · 고객 · 계약 · SLA`.
3. **Detail pane** (pinned panel — §4.7 "상세 보기의 기본은 핀 패널"): code chip +
   title + status chip, `en` enum chip row, `kv` key-value rows, link chips
   (upstream/downstream traversal), action buttons. Opens on row click; J/K/Enter
   keyboard nav; Esc chain closes.
4. Narrow width (`modNarrow` / `effW`): compact columns (code·title·status only);
   NO horizontal table scroll (§4-19 layout preset).

## 3. Stat bar (design values — real build must derive live)

| Stat | Prototype value | Tone | Drill |
|---|---|---|---|
| SLA 위반 | 0 | default (0 hidden or `—` per §4-3 예외 우선) | filtered list |
| 진행 이슈 | 2 | warn | filtered list |
| 상주 현장 | 12 | default | list |

Every stat is a click-through filter (§4.7-9 분석=drill 불변식 — no non-clickable
numbers). AGENTS 156/167: hardcoded stats were a truthfulness violation and were
re-derived from rows — the real build derives all stats from the same query the
list uses.

## 4. Primary action

`「이슈 접수」` — customer/site issue intake. Prototype routes to intake
(`action: { label: "이슈 접수", scr: "ingest" }`; AGENTS (4): "현장=이슈 접수" domain
action). The operations map right-click menu also offers `이슈 접수 기안` per site
marker (AGENTS (20)). Real build: issue intake creates a support ticket bound to
the site (see design-contract §4).

## 5. List rows (object cards) — the four seed CustomerSite rows

Each row = a CustomerSite object; every noun is a clickable object or absent (§4-1).

| code | c1 현장 | c2 고객 | c3 계약/서비스 | st chip | tone | en enum |
|---|---|---|---|---|---|---|
| ST-01 | 대원강업 상주 | 대원강업 | C-207 경비 | 이슈 1 | warn | 서비스=경비 |
| ST-04 | 평택항 물류센터 | 대한제강 | 지게차 임대 | 회신 대기 | warn | 서비스=지게차 임대 |
| ST-07 | 안산공장 정비 | BESTEC | 설비 유지보수 | 정상 | ok | 서비스=설비 유지보수 |
| ST-09 | 순천세아제강 미화 | 세아제강 | 미화·소모품 | 정상 | ok | 서비스=미화 |

**kv rows** (per site): 상주 인원+담당 (`경비 44명 · 이종호`), 이슈 (`7/3 결원 → 대근
편성`), SLA (`100% · 무위반`), 요청 (`임대 연장 조건 회신 — CS-118`), 담당, 진행
(`WO-2638 예방정비 D-2`), 납품 (`IN-0620 · 6/16`), 회신 (`CS-114 단가 확인`).

**links** (traversable chips — the ≥2 upstream + ≥2 downstream requirement is
natively satisfied):
- Upstream: `거래처 CL-xx` (customer object card), `C-207` (contract), 담당 person chip.
- Downstream: `WO-` work orders, `CS-` replies, `IN-` intake/deliveries, `근태 현황`
  (att screen), `메일 스레드` (mail), `지도에서 보기` (map screen with
  `{ mapOv, mapSel }` preset — per-row overlay: `iss`/`ct`/`wo`/`cov`; AGENTS (47)).

**acts**: ST-04 only — `연장 계약 기안` (prefilled contract-approval draft,
`appr: "contract"`, guardrailed template: authority preflight, self-checklist,
four-eyes peer, SoD approval line — AGENTS 2026-07-08 (1)).

## 6. Status-chip vocabulary

Site-level: `정상`(ok) · `이슈 N`(warn) · `회신 대기`(warn) · `SLA 위반`(danger,
implied by stat). Ticket-level (backend FSM): OPEN → IN_PROGRESS ⇄ ON_HOLD →
RESOLVED → CLOSED (+ reopen RESOLVED→IN_PROGRESS). Status is ALWAYS a chip,
never a sentence (§4-8). 0-values hidden or `—` (§4-3).

## 7. Related design systems this module touches

- **거래처 CL- persistent objects** (AGENTS (24), TODO 181): CLIENTS CL-01~04 with
  tier (`핵심 고객`/`일반`), terms, transaction chain (`C-`, `CP-`, `SR-`, `WO-`,
  `JL-`, `IN-`, `CS-`, `VC-`, `FL-`), and home site. Field rows link to them.
- **업무일지 JL-** (DESIGN §2): JL- = 일자 × 현장 × 작성자 object, cross-linked to
  근태(AT-) and 정비(WO-). Seeds: `JL-0703 대원강업 야간조 업무일지` (이종호, closed
  7/3, keep 3년), `JL-0702 안산공장 예방정비 일지 WO-2638` (김성호). Registered in
  the DOCS registry with lifecycle card (lcOpen), retention, drag/audit (AGENTS 151).
  Field-worker persona (v7, ROADMAP §8) files JL- directly from 내 업무.
- **출근 체크인/아웃** (dc.html `attCheckVerify`/`attCheckIn` lines 10437–10465):
  deterministic gate = registered device × site geofence radius. Fail-closed: no
  site assignment/registered device ⇒ forbid + audit (`§3.10-①`); permit ⇒ audit
  with device+geo reason, "실적 트랙 실시간 기록 · 급여 연동"; check-out = "실근무
  확정 · 연장 여부 자동 판정". Real GPS/beacon/device attestation is explicitly a
  backend-layer contract (comment at line 10443, HANDOFF §13).
- **Ops map (운영 지도)**: site markers with pulse/selection ring, overlay segments
  (커버리지/이슈/계약/정비·배차), per-site summary card, right-click quick actions
  (요약/이동/근태/배차/이슈 기안). Field rows round-trip via `지도에서 보기`.
- **SLA ≠ SLO** (DESIGN §4-26, binding): 현장·계약 = **SLA** (contractual external
  commitment; violation = penalty, belongs to the contract object, egress-grade
  severity). Support tickets = SLO. Labels/chips must never mix the two. Both must
  be **configurable setting objects** (threshold·window·escalation — no-code edit,
  revision staging §3.9.0, HO-01 grammar) — never hardcoded constants.
- **Series** (§4-15): recurring per-site charges/visits roll up under SR- series
  (e.g. SR-206 대근비 linked from CL-02 chain).
- **PII**: compliance ledger P-04 「파견·현장 배정」 — 자격·근태 data, retention
  3 years post contract, disclosure to customer = contract basis (dc.html 11078).
  Requester contact on customer intake is PII: never logged, never echoed.

## 8. Lifecycle, guardrails, CRUD doctrine applied to this module

- **§3.9 lifecycle**: intake/ticket objects follow draft→…→archive semantics where
  applicable; closure ≠ final approval — the author/owner confirms then closes
  (finalization rule, DESIGN §2 종결). Hard delete forbidden; archive = hidden+kept.
- **§3.10 guardrails**: every action preflighted — authority gate (deny-by-omission),
  fail-closed transitions, all attempts audited (denies too). Contract-draft action
  passes self-checklist + four-eyes + SoD.
- **§4-27 input invariants**: intake form = normalized masked inputs, required =
  fail-closed submit gate, reasons/categories = curated enums (+N+1), person/object
  pickers = typeahead not enumeration; downstream-effect one-liner on create forms.
- **§4-29 checklist gates**: significant actions (dispatch visit, close with
  acceptance) get auto-checked preflight + recorded postflight.
- **HANDOFF §20 CRUD**: every visible element must answer create/read/update/remove
  via UI; U on active objects = override (reason + four-eyes) or revision staging.

## 9. Signature story mapped to design intent (STORY-FIELD-001)

"A customer site intake becomes a field visit with check-in, work log, and customer
acceptance reflected in SLA and billing."

1. **Intake**: 「이슈 접수」/public customer intake ⇒ ticket object (SUP-/IN- grammar)
   bound to site ST- + customer CL-; SLA clock starts (due_at from priority).
2. **Field visit**: triage links/dispatches a WO- (정비/방문) for the site — visible
   as 진행 kv + WO- link chip; dispatch queue/map round-trip.
3. **Check-in**: assigned field worker checks in at the site (device × geofence,
   fail-closed, audited; arrival/departure events feed 근태 실적 track).
4. **Work log**: worker files the visit report/JL- (work log linked to WO- and site;
   evidence attachments BEFORE/DURING/AFTER).
5. **Customer acceptance**: resolution is confirmed by the customer (수령확인/ack
   grammar — same family as 게시판 수령확인·InboxDoc confirm); acceptance event is
   the audited closure evidence.
6. **SLA & billing reflection**: resolved_at vs due_at ⇒ met/breached on the site
   SLA rollup; acceptance + visit evidence feed the contract/billing chain (C- →
   정산/전표 downstream — laborcost/finance surfaces drill back to these objects).

## 10. Empty/error/denied affordances

- Empty list: one line = reason + next action (§4-10), e.g. "등록된 현장이 없습니다 —
  거래처·현장 등록으로 이동".
- Denied: deny-by-omission at nav; direct route hit renders the denied state
  (role=status), never a fabricated view.
- Errors: alert + retry (production exemplar grammar); loading = `role="status"`;
  no explanatory captions/subtitles anywhere (§4-12 — hard gate).
