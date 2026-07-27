# CAP-EVALUATION-CONSOLE — design spec (design-authority extract)

Stage-1 scout artifact. Source of authority: Claude Design project "B2B SaaS Console
Design" mirrored byte-exact at `docs/design/oyatie-console/` (change-log 190,
synced 2026-07-24). Everything below is **design intent**, not implementation
status. Route: `/console/evaluation`, screen key `evaluation` (prototype key:
`review`). Story STORY-EVALUATION-001: "A review cycle opens with goals, gathers
self and manager evaluations with evidence links, calibrates, and finalizes into
the person ledger with audit."

## 1. Where the module lives in the design authority

| Artifact | Location | What it says |
|---|---|---|
| ROADMAP §4 module matrix | `ROADMAP.md:65` | `인사평가 \| review \| Lattice·15Five \| Review·KPI·근태연동 \| 🟡` (partial) |
| Change log #149 (2026-07-22) | `AGENTS.md:338`, `ROADMAP.md:174`, `TODO.md:314` (#18, checked) | 평가 스코어카드 + 인사 카드 평가 이력: scorecard modal with 3 auto-attached contexts (근태·최근 업무·KPI, click = drill), S–D grade segment, submit = RV- code issuance + audit (OT-24), person-card history section gated to 민감정보/비밀 clearance, row click = audited view |
| Change log #174 (2026-07-23) | `AGENTS.md:392` | 인사 카드 = 직원 원장 (person ledger): 평가 rows (RV-, sensitive gate) merge into the time-ordered all-module ledger; row click = drill to the review screen |
| Ontology registry | dc.html:8716, 13802 | `OT-24 평가` — active type, steward 인사팀, note "KPI·근태 연동 스코어카드"; props `[직원 person, 주기 enum, KPI text, 근태 연동 text, 상태 lifecycle]`; linkTypes `[대상 직원 → 직원 N:1, 근태 근거 → 근태 1:N]`; actions `[review, card]` |
| Field classification #157 | dc.html:13477–13487 | 평가/평가 스코어카드/오퍼 fields classified **민감** (sensitive) — masking, egress, context gates propagate |
| Lifecycle charter | `DESIGN.md` §3.9 | 평가 explicitly listed among objects that follow Draft→Archive lifecycle; every transition = audit event, version, PBAC gate, status chips |
| Guardrails | `DESIGN.md` §3.10, `HANDOFF.md` §16 | fail-closed preflight: authority gate, self checklist, four-eyes (submitter ≠ reviewer), SoD, detective audit |
| Backend contract | `HANDOFF.md` §0, §2, §7, §15, §18 | append-only audit events with before/after; Cedar/PBAC deny-by-omission; scope-relative aggregates; lifecycle engine states draft→submitted→approved→active→archived |

## 2. dc.html extract — screen `review` (평가)

### 2.1 Screen layout (template lines 1604–1712)
- Header row: `h1 평가` + subtitle "2026 상반기 정기평가 · 7/18 마감" (**note**: our
  build must render the active-cycle name and due date as data — per §4-12 the
  free-text subtitle becomes a cycle object chip row, not caption prose).
- 배치 (layout preset) menu + 기본 배치 reset — standard card-window grammar
  (§4.7-2); cards support grab/popout/pin/resize.
- Two cards in the card zone:
  1. **팀별 진행률** (team progress): rows `팀명 | progress bar | NN%`.
     Seed: 경영지원팀 80 · 경비팀 45 · 정비사업팀 62 · 관제센터 100.
     Bar color: 100% = `--ok-solid`, <50% = `--warn-solid`, else `--teal`
     (logic line 19639–19641).
  2. **내 평가 할 일** (my evaluation tasks): rows `due chip (D-3 / 7/15, warn
     tone, mono) | task title (click = open person card when subject known) |
     작성 button (solid ink)`. Seed tasks: 수습 근무평가 — 조이슨 (D-3), 수습
     근무평가 — 최민석 (D-3), 자기평가 제출 (7/15, subject = 본인). Submitted
     tasks disappear from the list (line 19652 filter on `rvDone`).

### 2.2 Scorecard modal (#18, template 7642–7675, logic 19653–19680)
- `role=dialog aria-modal` centered modal, 440px: title = task name, chip
  `OT-24 평가`.
- **자동 첨부 컨텍스트 — 클릭=원 화면** (auto-attached context, click = drill):
  exactly 3 evidence rows per subject — 근태 (e.g. "출근 21/21 · 지각 0 · 연장
  4h" → att screen), 최근 업무 (object codes e.g. "CS-118 고객사 회신 · JL-0630
  일지 3건" → mywork/dispatch), KPI (e.g. "고객 응답 SLA 97% (목표 95%)" →
  dashboard). Each row is a button that navigates to the source screen.
- **등급** segment: S / A / B / C / D single-select chips (selected = ink bg).
- **평가 의견 (선택)** free-text textarea (optional supplement — §4-19 typed
  fields: grade is the enum, note is the appended prose).
- Footer: caption "제출=RV- 개체 · 평가자 서명·시각 보존" + 취소 + **제출**
  (disabled until a grade is picked — fail-closed).
- Submit behavior (line 19662–19669): issues `RV-` code (seed continues
  RV-2600+), records `{code, cycle:"2026 상반기 · 수습", grade, by:viewer,
  at, note}` into per-person history, removes the task, writes audit event
  `인사평가 제출` (cat submit, decision permit, reason "스코어카드 — 근태·업무·KPI
  컨텍스트 자동 첨부 · 평가자 서명·시각 보존 · OT-24"), toast with code + grade.
- Esc closes the modal (line 14887).

### 2.3 Person card 평가 이력 (template 7990–8006, logic 19888–19894)
- Section "평가 이력 — RV- 개체" inside the person card 직무 정보 block.
- **PBAC gate**: rendered only when viewer clearance ∈ {민감정보, 비밀}
  (`pRevOn`, line 19888) — deny-by-omission for everyone else.
- Row = `RV-code (mono, info tone) | cycle label | grade tile | by · at meta`;
  row click = **audited view** (`title="평가 개체 — 열람 감사"`) opening the RV-
  object.
- Seeds: 조이슨 RV-2501 "2025 하반기 · 정기" A (정하늘, 26-01) · 최민석 RV-2502
  B · 김성아 RV-2503 S. Submitted scorecards prepend live.

### 2.4 Person ledger merge (인사 원장, logic 19968–19971)
- The person card's all-module ledger merges 평가 entries: `push(at, "평가",
  purple, cycle + " — " + grade, code, → review screen)`. Finalized evaluations
  are therefore first-class person-ledger rows with drill both ways.

### 2.5 Adjacent surfaces referencing the module
- Overview calendar: "7/8 수 — 수습 근무평가 마감 · 2명, hint 평가로" → review
  screen (line 19791).
- Persona matrix: HR 담당 (clearance 민감정보) has `review` in her screen list
  (line 10345); nav item 평가 with circleCheck icon (line 16302); command
  palette + token grammar map 평가 → review (14187, 14845, 16938).
- Recruiting has its own interview scorecard (면접 평가, OT-14) — **separate
  module**; only the pattern (grade + evidence + audit) is shared.

## 3. Markdown-derived intent beyond the prototype's current pixels

The prototype implements only the "gather manager/self evaluations" middle of
the story. The charter dictates the full shape our module must support:

1. **Cycle = lifecycle object** (§3.9, §15): a review cycle (정기/수습) is a
   governed object — draft → open → (gather) → calibration → finalized →
   archived, every transition audited, stage = chips/stepper, single next-CTA.
   The screen header's "2026 상반기 정기평가 · 7/18 마감" is the active cycle
   object surfaced.
2. **Goals**: the story opens a cycle "with goals". §4-19 typed fields → goals
   are structured (metric kind enum + target + weight), not prose. OT-24 props
   (KPI, 근태 연동) are the goal/evidence axes.
3. **Evidence links** (§4.7-10 완전 추적성, §4-14): the scorecard's 3 auto
   contexts are object links (근태 AT-, 업무 WO-/CS-/JL-, KPI) — in the real
   backend they persist as typed evidence links on the review, drillable.
4. **Calibration**: §3.10-③ four-eyes + §3.9.1 maker-checker — the calibration
   decision is made by someone other than the submitting evaluator, with a
   required reason when the grade changes, all audited.
5. **Finalization into the person ledger**: #174 person-ledger and #149 history
   section — finalized reviews carry an RV- code, appear in the per-person
   ledger, reads of another person's history are themselves audited (§4.5
   "권한이 있어도 열람은 반드시 기록"), and the section is deny-by-omission
   below the sensitivity clearance (#157: evaluation fields = 민감).
6. **Benchmarks** (§4-21): Lattice · 15Five — cycles, goal check-ins,
   calibration, review packets. BENCHMARK.md's honest-gap column for adjacent
   modules (scorecard collaboration) applies.
7. **No-explanatory-UI** (§4-12) binding on our build: cycle status = chips;
   no subtitles; stat bar (not KPI cards) for progress; every noun (cycle,
   subject, RV-, evidence code) clickable or absent.

## 4. UI grammar constraints for the frontend build (from DESIGN §4)

- Status chips only; compact one-row stat bar for cycle progress (§4-11).
- Submit gates fail closed: no grade → disabled submit; missing evidence →
  blocked with inline reason (§4-27, §4-29).
- All object codes (RV-, AT-, WO-, AP-) render as drillable references.
- Keyboard: Esc closes the scorecard; list keyboard navigation per §4.7-1.
- Team/subject pickers are typeahead, not enumerations (§4-27-4).
- Empty states carry reason + next action one-liner (§4-10).
