# CAP-PAYROLL-CONSOLE — Design Spec (dc.html extract + markdown intent)

> Source of authority: `docs/design/oyatie-console/` mirror (change-log 190). Extract of the
> `pay` screen from `Oyatie Console.dc.html` (template lines ~2112–2399; logic: state ~8865,
> data ~8580–8606, methods ~9640–9717, renderVals ~15190–15244 and ~19060–19188) plus the
> payroll-relevant entries of DESIGN.md / AGENTS.md / TODO.md / ROADMAP.md / BENCHMARK.md /
> DEMO.md / HANDOFF.md. This is design intent, not implementation-status evidence.

## 1. Identity

- Prototype screen key: `pay`. Console route: `/console/payroll`, nav screen key `payroll`
  (already declared in `web/src/console/shell/nav.ts` — payroll group, `labelKey
  "console.shell.nav.payroll"`, icon `calc`, gate `DIRECTORY_ROLES +
  FEATURES.EMPLOYEE_DIRECTORY_READ`; **not yet** in `MOUNTED_SCREEN_KEYS`/`SCREEN_REGISTRY`).
- Object codes: run = 회차 (series SR-205 정기급여, monthly instances), payslip = `PS-`,
  approval = `AP-` (prototype instance AP-3124), attendance exceptions = `AT-`.
- Signature story STORY-PAYROLL-001: attendance-complete close → calculate → exceptions
  resolved → approval routing → disbursement scheduled → payslips delivered with ack.
- Benchmark row (BENCHMARK.md): Workday. Proven strengths: plan-vs-actual timeline, close
  gate (exceptions fail-closed), calculate→exception→submit→transfer chain, run series.
  **Honest gaps (L3, by design)**: real statutory calc engine (tax law / retro rulesets),
  multi-country compliance, audited retro calculation, large-roster scale.

## 2. Layout zones (pay screen)

1. **Header row**: `h1 급여` + one operational subline (period · pay date · headcount:
   "7월 정기 지급 · 산정 6/1–6/30 · 지급일 7/10 (금) · 대상 1,284명" — judged operational
   data, not caption, per TODO.md §4-12 sweep) + **회차 시리즈 chip** (purple, opens SR-205
   series card: instance history/trend/next run) + layout-preset menu (배치) + 기본 배치
   reset (only when custom) + **deadline chip** + **single CTA** (see §5).
2. **Pipeline stepper** (5 steps, one card row): 근태 마감 → 계산 → 예외 검토 → 결재 → 이체.
   Per-step: number/check circle, label, live sub-label, connector line. States:
   `done | active | wait | reject | locked` (locked = faint). Clickable only: step 1 while
   active (→ attendance screen) and step 4 once submitted (→ approval doc `pa1`).
   Sub-labels (state-derived):
   - 근태 마감: "4/4 법인 · 7/3 확정" / "3/4 법인 · 코스 대기"
   - 계산: "1,284명 완료" / "계산 중…" / "실행 대기" / "마감 후 가능"
   - 예외 검토: "N건 남음" / "5건 처리 완료" / "계산 후 검토"
   - 결재: "AP-3124 결재 중 | 승인 | 반려" / "예외 처리 후 상신"
   - 이체: "7/10 04:00 예약됨" / "재상신 필요" / "결재 승인 시 예약" / "7/10 04:00 예정"
3. **Card zone** (window model — every card supports pin/split/float/minimize via the shared
   `payLay*` card grammar, per-user persisted layout, drop indicator, split bar):
   a. **급여 명부 (roster)** b. **예외 검토 (exceptions)** c. **지급 총액 (totals)**
   d. **지급 일정 (schedule)**.

## 3. Cards in detail

### 3a. 급여 명부 (roster)
- Header: title + mono count chip `{shown} / 1,284` (cap+search, §4-27-4 scale rule) +
  **sheet button** (계산 후에만 노출 — opens 6-col sheet 이름·법인·기본급·수당·공제·실지급 with
  live Σ totals; **viewing the sheet is itself a sensitive-class audited event, DLP chip**)
  + search input (이름·법인, multi-attribute).
- **Gate state (pre-calc)**: lock icon + state text ("근태 마감 → 계산 실행 후 명부가
  생성됩니다" / "계산 중 — 잠시만요" / "계산을 실행하면 1,284명 명부가 생성됩니다") + gate
  button (근태로 이동 | 급여 계산 실행). No roster rows are rendered before calculation.
- **Post-calc table**: sticky header, shared grid track
  `minmax(100px,1.2fr) ent40 base96 allow88 ded88 net104 mom92` (px widths are the
  `payColW` state, **column-drag resize clamp 36–220px**, personal view — no audit,
  §3.9.0-①). Columns: 이름(avatar initial + name + title), 법인 chip, 기본급, 수당, 공제
  (rendered −), 실지급 (bold), trailing cell = exception flag chip (type label, tone bg;
  click opens that exception) OR 보류 chip OR 전월 대비 delta (mono, warn tone when
  flagged). Rows with unresolved exceptions sort to the top (`payListSorted`, single query
  point §4-19). **J/K/Enter** keyboard nav with selection ring
  (`inset 2.5px 0 0 var(--signal)`), Enter → person card. **Row click = person card
  (인사 카드); payroll view of another person is an audited read** (row title says so;
  `onPayView` toast "급여 열람 — 감사 로그에 기록되었습니다").
- Footer strip: "단위 ₩ · 공제 = 4대보험 + 소득세" + "급여 열람은 감사 로그에 기록됩니다".

### 3b. 예외 검토 (exceptions)
- Header: status dot (faint pre-calc / danger while open / ok when done) + mono meta
  ("남음 N / 5" | "5건 완료" | "계산 대기") + right hint "처리 후 상신 가능".
- Pre-calc gate state mirrors roster ("6월 근태 마감(4/4 법인) 후 계산을 실행할 수
  있습니다" etc.).
- **Exception rows** (5 seeded types = the exception taxonomy):
  | id | type | tone | amount | one-line | linked chips |
  |----|------|------|--------|----------|--------------|
  | px1 | 연장수당 (overtime allowance) | warn | +₩412,000 (auto-updates to +₩438,000 with "wf3 자동 반영" label when AT-0702-07 approved — automation writeback) | 6월 연장 14.5h · 사전승인 없는 2h 포함 | AT-0702-07 근태, WO-2638 정비 |
  | px2 | 소급 인상 (retro raise) | info | +₩180,000 | 7/1 인상 6월분 소급 · 명세 검증 대기 | messenger thread |
  | px3 | 결근 공제 (absence deduction) | danger | −₩186,000 | 7/3 미출근 1일 무급 · 소명 시 재계산 | AT-0703-02 근태 |
  | px4 | 일할 계산 (proration) | info | ₩3,410,000 | 6/22 입사 2명 · 9일 일할 · 4대보험 취득신고 완료 | 인사 입사 처리 (noPerson) |
  | px5 | 계좌 확인 (account verification) | danger | 지급 보류 위험 | 계좌 변경 요청 · 본인 인증 미완료 · 7/8까지 미인증 시 기존 계좌 지급 | 직원 인사 카드 |
- Row anatomy: type chip (tone bg/bd/tx) + person-name button (→ person card; plain span
  when `noPerson`) + mono amount + one-line + inline 확인 button + chevron. Click expands
  detail: explanation lines + **linked-object chips** (AT-/WO-/thread/person — 1-click
  drill) + two actions: **확인 처리** (solid) and **이번 회차 보류** (hold → carried to
  next run). Resolved rows: dim 0.55, state chip 확인됨 (ok) / 보류 · 다음 회차 (warn),
  actions removed. Toast on action names who·type·remaining ("… · 남은 예외 N건" /
  "예외 검토 완료, 상신 가능").

### 3c. 지급 총액 (totals)
- Tag chip: "확정 계산" post-calc / "예상 · 전월 기준" pre-calc (truthful basis labeling).
- Big mono total ₩41.8억 + "전월 +1.8%" (units/basis always stated).
- Per-entity bars: HR 스태핑 ₩15.1억 · BESTEC ₩13.4억 · ㈜코스 ₩11.2억 · KNL ₩2.1억
  (scope = authorized-entity union, PBAC-relative "전체").
- Divider, then: "사업자 부담 · 4대보험+퇴직 — ₩5.6억" and **지급 계좌 상태** ("오류 1건 ·
  조이슨" danger → "오류 0건" ok / "보류 1건" warn — live-derived from px5 resolution).

### 3d. 지급 일정 (schedule)
- 4 milestone rows (dot state done/now/wait + mono date + label + state chip):
  7/3 계산·예외 검토 → 7/8 결재 기한 16:00 → 7/9 이체 파일 은행 제출 → 7/10 04:00 이체 ·
  명세서 발송. States derive from the run FSM (approval done ⇒ 7/9 becomes "진행").
- Footer (action-driving copy, allowed): "이체 실패 건은 당일 08:00 재시도 후 알림으로
  보고됩니다".

## 4. Run FSM as simulated (behavioral contract)

State variables: `attCossClosed` (attendance close, per-entity 4/4), `payCalcing`,
`payCalced`, `payExDone{id: "ok"|"hold"}`, `paySubmitted`, approval item `pa1`
(`done+doneTone` → approved/rejected).

- `payStageN = !attClosed ? 1 : !calced ? 2 : exceptionsLeft>0 ? 3 : !submitted ? 4 : 5`.
- **Close gate**: attendance close is confirmed on the attendance screen via a **§4-29
  preflight modal** — auto-checks (근태 예외 처리 0건 — fail-closed with a **fix-link to the
  first unresolved exception**; 타법인 마감 3/3) + soft warn (미결 연차 N건 — "승인 시 소급
  반영") + human attest; passing logs an audit event and unlocks calculation. Notification
  emitted: "6월 근태 마감 완료 (4/4 법인) — 급여 계산이 가능합니다" → links to pay screen.
- **Calculate** (`payRunCalc`): guard `attClosed && !calced && !calcing`; async progress
  state; completion toast "계산 완료 — 1,284명 · 검토할 예외 5건".
- **Exception resolution** (`payExAct`): per-exception `ok|hold`; hold = defer to next run
  (이월). Submit is fail-closed until every exception is resolved.
- **Submit** (`paySubmit`): guard `calced && all-exceptions-resolved && !submitted`; creates
  approval object **AP-3124 "7월 정기급여 지급 승인 — 1,284명"** (amount ₩41.8억, due 7/8
  16:00, detail includes hold count "보류 N건 — 다음 회차 이월", attachments 대장.xlsx +
  법인별 집계표.pdf as export artifacts, links 급여 회차 + 근태 마감 기록, 6-month total
  sparkline) + notification. Toast: "AP-3124 상신 완료 — 결재 승인 시 7/10 이체가
  예약됩니다".
- **Approval decision** (in the approval module): approve ⇒ transfer reserved 7/10 04:00 +
  notification "7월 정기급여 이체 예약 완료 — 7/10 04:00 · 1,284명"; reject ⇒ header
  "반려 — 재상신 필요", CTA 회수 후 재상신 준비 (`payResubmit` withdraws the AP item and
  clears submitted).
- **Header chip** state: 승인 기한 7/8 (수) 16:00 (warn) → 결재 대기 · 기한 7/8 16:00
  (info) → 이체 예약 7/10 04:00 (ok) | 반려 — 재상신 필요 (danger).

## 5. Single-CTA state machine (§4.7-6: gates = pipeline steps, one CTA)

| state | CTA / chip |
|-------|-----------|
| attendance open | ghost CTA "근태 마감으로 이동" |
| closed, not calced | primary CTA "급여 계산 실행" (pulse anim while calcing) |
| calced, exceptions left | warn chip "남은 예외 N건" (no CTA) |
| exceptions done, not submitted | primary CTA "결재 상신" |
| rejected | ghost CTA "회수 후 재상신 준비" |
| submitted, pending | ghost CTA "결재함에서 열기" |
| approved | ok chip "지급 준비 완료 · 이체 예약됨" |

Nav badge on 급여 item: unresolved-exception count while `calced && !submitted`.

## 6. Self-service & delivery (§4.8 / HANDOFF §3)

- Payslips are **personal-inbox objects** (`PS-`): kind `pay`, fields ref/title/from(급여 ·
  자동 발행)/date/net/base/allow/ded/payDate/delta(+dTone)/links(급여 회차, 근태 마감,
  related WO-)/`confirmed{by,at}`. Delivered on pay date ("첫 명세서는 지급일에 자동 발송").
- Self-view of one's own payslip is **frictionless and NOT audited** (self-view right);
  `legal:false` docs need no passkey; ack (`confirmed{by,at}`) is recorded on view/confirm
  and is the delivery receipt readback. Viewing **another** person's pay is always audited.
- Mail/notification surfaces reference PS- codes as live object links.

## 7. Personas (ROADMAP §8)

- **급여 전담 (v8 한미정)**: scope pay·att·leave·benefit·appr·docs·finance·laborcost·
  dashboard(+comms). Full pipeline operator.
- **운영 관리자 (v1)/HR (v2)**: pay visible.
- **사무직 (v4)** and **CX (v10)**: pay roster/exceptions **deny-by-omission** (nav item
  absent); self payslip via personal inbox only.
- **정비 기사 (v7 김성호)**: own payslip PS-2618 (owner-scoped), no roster.
- Data-scope rule: every aggregate (totals, entity bars, counts) is computed over the
  caller's authorized-entity union only (§4.5).

## 8. Cross-module edges (≥2 up / ≥2 down)

Upstream: 근태 마감 (attendance close, per-entity) · 근태 예외 AT- (drill from exception
chips) · 연장근로 자동화 wf3 (AT- approval → exception amount writeback) · 대근비 SR-206
(substitute-pay settlement joins the run) · 인사 (new-hire proration, employee card).
Downstream: 전자결재 AP- (submission) · 재무 VC-2606 (급여 이체 voucher, flow 기표→차대
검증→승인 AP-3124→전기, links back to 급여 명부/회차) · 개인 수신함 PS- (delivery+ack) ·
인건비 분석/대시보드 (인건비율 drill lands on pay) · 계약 수익성 환류.

## 9. Invariant compliance notes for the build

- No explanatory captions; status = chips; only action-driving copy (경고·에러) as free text.
- Compact stat rows, no big-number KPI cards (totals card's single big figure is the
  object's own value with basis chip, not a KPI card grid).
- Every noun on screen is a drillable object (person, entity, AT-, WO-, AP-, PS-, SR-205).
- Token colors only; monospace for numbers/codes; Korean-first noun-phrase labels.
- Fail-closed gates at every stage transition; toasts state result + path.
- Roster/exception data appears **only** after a real calculation; before that the truthful
  gate/empty states above are the whole surface.
