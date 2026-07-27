# CAP-ORG-CONSOLE — Design Spec (dc.html extract + markdown intent)

> Source of authority: `docs/design/oyatie-console/` mirror (change-log 190). Extracted 2026-07-24
> from `Oyatie Console.dc.html` (screen key `org`), DESIGN.md §3.9/§3.9.1–3/§3.10/§4,
> HANDOFF.md §15/§16/§18/§20, AGENTS.md change-log entries 12, 13, 18, 128, 140, 143, 168.
> Design intent only — never implementation-status evidence.

## 1. Story and route

STORY-ORG-001: an organization change drafts a new entity/site/team structure, passes impact
preflight and SoD approval, takes effect on its effective date, and reflects across scope
segments and the graph. Route `/console/org`; web nav screen key `orgchart`
(nav group HR, gate = DIRECTORY_ROLES + `EMPLOYEE_DIRECTORY_READ`, ko label `조직도` already in
`i18n/ko.ts` at `console.shell.nav.orgchart`).

Reference implementation in the prototype: `orgChange` state machine (dc.html 12524–12570,
renderVals 15347–15384, modal template 7211–7284) — DESIGN §3.9.2's worked example, and the
template from which the generic lifecycle engine (`lcSeed`) was later generalized.

## 2. Screen layout (dc.html 1315–1427)

Zones, top to bottom:

1. **Header row**: `h1 조직도` + (conditional) **proposal banner**: warn-toned chip row
   `제안 변경 {N}건 — 개편 결재 확정 전 임시` with a single `개편 결재` CTA (opens the org-change
   modal for the dirty entity). Right-aligned `편집`/`완료` toggle button (pencil icon).
   No subtitles, no caption text (§4-12).
2. **Tree canvas** (scrollable, min-width 940px):
   - Root card: group name + `이사회 · 그룹 경영지원 · 재직 1,284` (headcount = live derivation).
   - **Entity columns** (one per 법인, connector lines drawn between): entity card block
     (name, meta = 업종 · 인원), click = expand/collapse whole column; `i` corner button =
     entity info card. Border highlights (`--signal`) when expanded.
   - **Site rows** under each entity: name + collapsed team-count badge `+N` + monospace
     headcount. Click = expand/collapse teams. Site row is a drop target for team drag.
   - **Team rows** under expanded sites: name + monospace headcount. Click (view mode) = team
     card. Draggable in edit mode (drag team → another site = move).
3. **Edit mode affordances** (only when `편집` active): inline rename inputs for entity/site/team;
   `+ 팀 추가` per site; `+ 사업장 추가` per entity; `+ 법인 추가` trailing column (opens the
   entity setup wizard); per-team `X` delete button.

## 3. Edit semantics — sandbox proposal, never live (§3.9.0-④)

- Every inline edit (rename, add site, add team, move team, delete team) calls `orgTouched`:
  increments `orgDirty` counter + audit event `조직 변경 제안` (cat submit, reason
  "제안 변경 — 조직 개편 결재로 확정 전 임시 (§3.9.0)").
- Team move via drag logs toast `팀 이동됨 — 실제 반영은 조직 개편 결재로 상신됩니다`.
- **Team delete guard (fail-closed §3.9.1)**: team with headcount > 0 → blocked, audit
  `팀 삭제 차단 — 인원 잔존` (decision forbid, reason 참조 무결성 — 재배치 완료 후 삭제 가능) +
  danger toast. Headcount 0 → delete counts as a proposal change.
- Team head click in edit mode → toast `책임자 변경은 인사 발령(전자결재)으로 처리됩니다`
  (head changes are an HR appointment approval, NOT an org edit).
- **Closing edit mode with dirty > 0 auto-opens the org-change modal** with the diff count
  toast `변경 N건 — 조직 개편 결재로 확정 (사전점검→SoD→발효)` (change-log 140 closed loop).
- Submit (상신) resets the dirty counter.

## 4. Org-change modal (STORY-ORG-001 core; dc.html 7211–7284 + 12524–12570 + 15347–15384)

Opened from: entity card `조직 개편 결재` button, header dirty-banner CTA, edit-close autoflow,
ops-map draft-site card `개편 결재로 확정`.

- **Header**: title `조직 변경 · {entity}` + stage chip. Stage labels:
  `초안 / 사전점검 / 결재 진행 / 활성(발효) / 정산 진행 / 보관됨`.
- **Stepper**: pipeline chips (done = ok-solid, active = ink, pending = muted):
  - 개편(REORG)/신설(NEW): `초안 → 사전점검 → 결재 → 발효`
  - 폐지(DISSOLVE): `초안 → 사전점검 → 결재 → 발효 → 정산 → 보관`
- **Draft-stage fields** (editable only in draft): kind segmented enum `신설 | 개편 | 폐지`;
  effective date (`발효일`, monospace date input, seed 2026-08-01).
- **Target stat strip** (always): 대상 (법인) · 인원 · 사업장 · 팀 — derived from the target
  entity's tree (hc = Σ site hc, teams = Σ site teams).
- **Precheck section** (visible from precheck stage on): `영향 분석 · 사전점검`
  - blockers (danger chips): dissolve-only — `소속 직원 {hc}명·의존 개체 정산 필요 ({done}/{total})`
  - warnings (warn chips): `진행 중 공고·결재 종결 필요`, `급여 마감·회계 결산 동결창 확인`
  - Blocker vs warning distinction is contract (DESIGN §3.9.1: blocker blocks, warn advises).
- **Approval section** (from approval stage): `결재선` — ordered 4-role SoD chain
  `HR → 재무 → 법무 → 임원`, each row = role chip + named approver + `승인` button (only while
  in approval stage and undecided) or ✓승인 done marker. CTA disabled until all approved,
  label `결재 대기 (k/4)`. Maker-checker: drafter ≠ approver (SoD, §3.9.1).
- **Settlement section** (dissolve only, from settling stage): `폐지 정산` checklist, 6 items:
  1. 소속 직원 전보·전적 (동의·통지 기간)
  2. 포지션 이관·폐지
  3. 코스트센터·예산 재배정
  4. 진행 중 공고·결재 종결
  5. 자산 이관·반납
  6. 급여·4대보험·퇴직 정산
  Each row = label + `정산 완료` button / ✓정산 marker.
- **Single CTA footer** (one context CTA per stage, §4.7-6):
  draft → `사전점검 실행`; precheck → `상신`; approval → `발효` (or disabled 결재 대기 k/n);
  published(dissolve) → settling; settling → `보관` (disabled `정산 진행 k/n` until all done);
  published(non-dissolve) → `활성 — 완료`; archived → `닫기`.
- **Archive gate**: attempting archive with unsettled items → toast
  `모든 의존·법정 정산 완료 후 보관할 수 있습니다 (참조 무결성 게이트)`. Archive sets the entity
  column `archived: true` (hidden from active view, history preserved — NO hard delete).
- **Audit**: every transition emits an event — 조직 변경 초안 / 상신 (사전점검 통과 ·
  결재선 상신 maker-checker) / 승인 ({role} {who} 승인 SoD) / 발효 (발효일 {date} · 조직 버전 N+1)
  / 폐지 발효 (의존 개체 정산 착수) / 보관 (정산 완료 · 숨김·이력 보존 · 하드삭제 아님).
  Target code pattern in prototype: `ORG-{n}`.

## 5. Entity card (법인 정보, dc.html 7150–7209)

- Header: initial avatar + name + meta. Drag header = split-pin; pin + close buttons.
- Property grid: 대표이사 · 설립 · 사업자번호 · 소재지 · 조직 ({n}개 사업장·본사).
- **Setup-stage direct edit** (§3.9.0-③): newly created entities (draft/setup) allow inline
  field edit (대표자/사업자번호[mask `000-00-00000`]/소재지) with audit; established entities
  are read-only — tooltip `확정 법인 — 변경은 콘솔 변경 기안 경유`. Honest empty labels:
  `발급 대기` (사업자번호), `미등록 — 편집`.
- **관할 (jurisdiction) chips**: per-entity chain (국가→광역→기초), each chip → compliance
  screen filtered by regulation code (click = drill + audit event 관할 규제 탐색).
- **재무 요약**: PBAC gate — section only exists for clearance ∈ {민감정보, 비밀}
  (deny-by-omission); collapsed state = `재무 요약 — 민감 · 펼치면 열람 기록` lock affordance;
  expand = audited view event; shows 월 매출 / 인건비율 / 마진 + `대시보드 — 법인 스코프 drill`.
- Footer actions: `조직 개편 결재` (opens org-change modal) · `소속 명부` (→ HR filtered by entity).

## 6. Team card (dc.html 7115–7148)

책임자 (clickable → person card; head derived via `deptHeadOf` org-tree lookup — no hardcoded
names, manual override map only) · 인원 · actions `팀원 명부` (→ HR filtered) · `팀 채널`
(→ messenger thread). Path line `법인 › 사업장`.

## 7. Entity setup wizard (신설 법인, change-log 128/168; dc.html orgAddEntity/entNewCreate)

3-step progressive disclosure:
1. 기본: 법인명(필수) · 약칭 · 업종 · 대표자 · 사업자번호(input-normalized mask, blank allowed
   "발급 전이면 비워두세요") · 소재지.
2. 조직: 최초 사업장 **필수 fail-closed** (`조직 없는 법인 금지`) · 팀 칩(+직접 입력) ·
   결재선 프리셋.
3. 통합: 셋업 채널 생성 · 알림 · 주52h 감시 자동화 시드(초안) 토글.

Duplicate name/short → fail-closed error `동명 법인·약칭 존재 — 개편 경로를 쓰세요`.
Create = joins ALL surfaces at once: scope segments, org-chart column, ontology graph object
(`OB-` code, type 조직·현장, status `셋업 중 — 개편 결재로 확정`), messenger `# {약칭} 셋업`
channel, notification, draft monitoring workflow + audit. **Confirmation = the org-change
approval (orgChange)** — the wizard only starts setup (§3.9 draft stage).

## 8. Object links (traversability contract — ≥2 upstream, ≥2 downstream)

Upstream (into org): HR person card 팀/법인 labels → team/entity cards (change-log 143);
ops-map site card → org screen; ontology graph 조직 node actions (`org` → 조직도, `roster` →
명부); scope selector segments; entity setup wizard.
Downstream (out of org): team → person card (head), team → HR roster filter, team → messenger
channel; entity → HR roster, entity → dashboard entity-scope drill, entity → compliance
jurisdiction drill; org-change → audit events; dissolve settlement → positions/postings/
assets/payroll (checklist items name the dependent object families).

## 9. Status/chip vocabulary

- Org-change stage: `초안 | 사전점검 | 결재 진행 | 활성(발효) | 정산 진행 | 보관됨`.
- Change kind enum: `신설 | 개편 | 폐지`.
- Proposal banner: warn tone. Blockers: danger tone. Warnings: warn tone. Approvals: ok tone.
- Entity setup: `셋업 중` meta suffix; honest field placeholders `발급 대기`/`미등록 — 편집`.

## 10. §4 invariants binding this module

- No explanatory captions/subtitles; status = chips; single CTA per gate step (§4-12, §4.7-6).
- Every noun clickable or absent: entities/sites/teams/heads/jurisdictions all drill (§4-1).
- Fail-closed guards surface reason + resolution path in the toast (§3.10, §4-27-2).
- Effective-dating + sandbox draft + impact preflight + SoD + settlement gate + archive-not-
  delete are the §3.9.1 governance mechanisms — all six appear in this one screen.
- Numbers monospace; headcounts derived, never hardcoded (change-log 156 truthfulness audit).
- Keyboard: Esc closes org-change modal/cards (14915); responsive: columns h-scroll ≥940px.
