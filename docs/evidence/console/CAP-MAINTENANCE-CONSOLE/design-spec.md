# CAP-MAINTENANCE-CONSOLE — Design Spec (Stage 1 scout)

Source of authority: design mirror `docs/design/oyatie-console/` (byte-exact, change-log 190).
Extract anchors: `Oyatie Console.dc.html` lines ~11058–11063 (`MOD_SCREENS_RAW().maintenance`), ~11018–11022 (`dpRows`
dispatch single definition), ~8952/8956 (items `d1`/`w1` processing-panel payloads), ~9464–9480 (`panelDecide`),
~13307 (`OBJ_SERIES["SR-203"]`), ~13791 (`ONT_TYPES` 정비/OT-13), ~16047–16290 (generic module surface renderer),
~10835–10884 (`modLinkGo`/`modApprPrefill`/module config). Signature story STORY-MAINTENANCE-001:
"A maintenance request becomes a scheduled work order, executes with evidence, and closes into asset history and cost."
Route: `/console/maintenance` (screen key `maintenance`, nav group 현장 운영, icon `wrench`).

Mirror content is design intent, not implementation-status evidence (roadmap `docs/program/console-enterprise-roadmap.md`
governs engineering discipline; EXPOSED_SCREEN_KEYS stays `["sales"]` — this module ships DARK).

## 1. Screen layout (generic module surface grammar — MOD_SCREENS consumer)

Single surface, no floating cards. Zones top→bottom:

1. **Header row**: title 「정비」 · compact one-line stat bar (never KPI cards, §4-11) · search input (`modQ`,
   multi-attribute: code+c1+c2+c3+st+enum values) · primary action button 「정비 요청 기안」 (opens approval
   composer prefilled — request-draft path, `modApprPrefill("general", "정비 요청 — ")`) · 「시트로 열기」 ·
   config gear (personal view config §3.9.0-①: add/remove ontology-property columns, add stat chips from
   real-coverage fields only `k · n/N`, detail behavior segment 분할 패널|개체 카드, filter presets, reset —
   every config mutation audited).
2. **Stat bar** (all clickable → filter drill; exceptions only, 0 hidden): 긴급 (danger tone) · 이번 주 ·
   예방정비 준수 % (preventive-maintenance compliance).
3. **SLA kanban lanes** (`lanes` extension field — maintenance is the reference implementation of the module
   kanban): 「SLA 임박 · 미배정」(danger) / 「예정 · 미배정」(warn) / 「배정 · 진행」(ok). Cards = row refs,
   click = row select (list/detail sync).
4. **List** (shared-track grid, cols 「오더·작업·현장·담당」 + status chip): J/K/Enter keyboard nav, column
   sort (numeric/currency aware), row select → detail. Narrow layout auto-demotes to code+title+status
   (`modNarrow` — no horizontal scroll ever). Row is a drag source (`objDrag` — payload `[WO-#### title]`).
5. **Detail** (right split panel by default; 「개체 카드」 mode opens the lifecycle object card `lcOpen`):
   - **flow stepper** (order cycle): 접수 → 계획 · 부품 예약 → 실행 → 정산 → 전표. Steps carry done/cur/next
     state and optional linked object codes (PO-121, PO-118, VC-2604) which drill.
   - **en chips** (typed enums §4-19, click = list filter): 유형 (긴급 출동 | 교체 정비 | 예방 정비),
     원인 (고장 | 반납 준비 | 정기).
   - **kv rows**: 접수 time + SLA window, 장비 + failure frequency ("FL-2643 · 고장 3회/90일"), 부품 (PO ref),
     체크리스트 progress, 일지 (JL- ref).
   - **links** (one-click up/downstream, each a chip): 자산 FL- (asset), 부품 재고/PO- (inventory/purchase),
     JL- (work log), 담당 person, 지도에서 보기 (map overlay preset). All route through `modLinkGo`
     (code→objectLinkGo, node→explore, person→people card, scr→screen nav, item→processing panel).
   - **acts**: 「배차·처리 패널」 (unassigned orders) / 「승인·처리 패널」 (assigned/report path).

## 2. Seed rows (behavioral fixtures — the three canonical states)

| code | title | site | assignee | status chip | flow position | enums | links |
|---|---|---|---|---|---|---|---|
| WO-2643 | 지게차 유압 누유 — 긴급 출동 | 인천 제2센터 | 미배정 | SLA 임박 (danger, "SLA 38분") | 계획·부품 예약 cur (PO-121) | 긴급 출동/고장 | FL-2643, PO-121, 재고, map |
| WO-2641 | 타이어 교체 — 반납 전 정비 | 인천 제1센터 | 미배정 | 내일 10:00 (warn) | 계획·부품 예약 cur | 교체 정비/반납 준비 | FL-2641, IV-018, map |
| WO-2638 | 정기 예방정비 — 조립라인 3호기 | 안산공장 | 김성호 | D-2 (ok/warn) | 실행 cur (계획 done PO-118, 전표 next VC-2604) | 예방 정비/정기 | JL-0702, 김성호, PO-118, map |

Dispatch screen (`dispatch`) shares the SAME row definition (`dpRows`, §4-18 single definition) with candidate
mechanics + SLA + processing-panel act; the mechanic persona's 내 업무 (mywork) lists their assigned WO- rows
from the same source.

## 3. Simulated behaviors = behavioral contract

- **Processing panel (배차)**: item `d1` — payload: ref WO-2643, urgency, site, 미배정, "접수 25분 경과"
  (danger), detail text, files (현장 사진 zip), links (장비 FL-2643, 고객), stats ("최근 90일 고장 3회 — 빈발
  장비" + drill to explorer). Actions: **confirm** = driver pick required → status "배차 완료 · <name>",
  audit event 배차 확정, notification dismissed, panel closed. **return/reject** = comment REQUIRED
  (fail-closed inline error), audit with reason, originator notified (toast copy).
- **Processing panel (승인)**: item `w1` — approve/return/reject with the same comment-required fail-closed
  rule; approve = "승인 완료" + audit.
- **Request draft**: primary action opens the approval composer (기안) prefilled "정비 요청 — " → the WO- is
  born of an AP- (요청 기안 N:1 link type). Duplicate-claim auto-check applies (composer grammar).
- **Typed creation wizard**: OT-13 「+ 새 개체」 wizard fields = schema props (유형 enum, 원인 enum, 대상 자산,
  SLA date, 상태 lifecycle) → draft OB-/WO- with fail-closed name, joins graph + module list (AGENTS 171).
- **Lifecycle loop (proven e2e AGENTS 117)**: WO- object card → 상신 (v2 revision draft, v1 stays active) →
  four-eyes approvals (SoD: drafter ≠ approver) → 발효·게시 → v2 active, v1 preserved; every transition an
  audit event; archive/dispose gated (no hard delete).
- **Order cycle side effects**: 계획 step reserves parts (inventory shows 출고 예약 −N with WO code; MRP
  proposes PO when shortage); 정산/전표 steps close costs into a finance voucher (VC-2604 flow shows the WO's
  settlement blocked by 차대 불균형 fail-closed on the finance side); asset timeline (FL-2643) accrues the WO
  as a lifecycle event; series SR-203 (지게차 FL-2643 정비 이력: rule "비정기(고장)+분기 예방정비", trend
  "90일 3회 — 빈도 상승 · 교체 손익 검토 AN-204") renders a mini-timeline on the instance card.
- **Analytics**: OT-13 analytics `MTTR = Σ 처리시간 ÷ 건수`; stat 예방정비 준수 % derives from preventive
  orders meeting target; forecast FC-05 links 정비 일정 as an overtime-trend driver.
- **SLA vs SLO (§4-26)**: 정비 처리 목표 = **SLO** (internal target, breach = alert), 현장/계약 서비스 수준 =
  SLA (contractual). Chips must label which one; thresholds are configurable setting objects (no-code,
  revision staging) — never hardcoded copy.

## 4. Empty/error/authz affordances

- Deny-by-omission: nav item hidden without grant (gate = operational roles + `work_order_read_all`); a
  direct route hit renders the module's denied state (single `role="status"` line, no leakage of counts).
- Empty list: one line "reason + next action" (§4-10) — e.g. no orders → point at 정비 요청 기안.
- Every mutation: fail-closed comment/required-field gates, toast states result + where the object went,
  audit event with reason. Loading/error: `role="status"` / `role="alert"` + retry (production exemplar).

## 5. UI grammar invariants binding this module (audit checklist)

§4-11 compact stat bar (no big-number cards) · §4-12 no explanatory captions/subtitles (status=chips, only
action-driving copy) · §4-19 유형/원인/SLA are typed fields, never free text · §4-1 every noun (WO-, FL-, PO-,
IV-, JL-, VC-, person, site) is clickable or absent · §4-18/§4-14 module = MOD_SCREENS-grammar object surface,
no bespoke template · §4-23 rows/chips are drag sources · §4-25-6 mock independence: every datum is a real
authorized backend response or a truthful loading/empty/denied/error state · token colors only ·
className = plain string literals (purity gate) · no inline Hangul (module i18n strings file).

## 6. Persona coverage (ROADMAP §8)

- 배차/운영 담당 (v1/v3): queue triage, dispatch confirm, SLA lanes, map roundtrip.
- 정비 기사 김성호 (v7): mywork = assigned WO rows, start/execute, 일지 등재 direct, evidence upload, own
  payslip untouched — sees only own branch scope.
- 관리자/임원: completion review (approve/return/reject), settlement approval, analytics drill.
