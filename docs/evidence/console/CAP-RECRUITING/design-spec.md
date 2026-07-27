# CAP-RECRUITING — Design Spec (dc.html extract + markdown intent)

> Source of authority: `docs/design/oyatie-console/Oyatie Console.dc.html` (change-log 190 mirror),
> DESIGN.md §3/§3.9/§3.10/§4, HANDOFF.md §15/§16/§18/§20, AGENTS.md change-log entries
> 2026-07-08 (5)(6), 2026-07-09 (36)(37)(38), TODO #26/#40, ROADMAP row `채용 | recruit`,
> BENCHMARK row `채용 | Greenhouse`. Extracted 2026-07-24 for STORY-RECRUITING-001.
> Mirror content is design intent, never implementation-status evidence.

## 1. Object chain position (DESIGN §3)

```
계약(C-) → 인원편성(포지션) → [모집 공고 JP-] → [지원자 APL-] → 입사 확정 → 직원 + 타임테이블
                                    ↘ 탈락(사유 enum) → 인재풀 → 인력풀(비상근) 전환 제안
```

Backend ontology seed already authors this spine: `contract → position → posting` with the
`posting → employee` link deliberately unresolved (`POSTING_EMPLOYEE_LINK_KEY`,
`backend/crates/ontology/adapter-postgres/src/seed.rs:72-83,1051-1085`). Posting ontology props:
scope(choice internal/external), fill_count(integer), deadline(date).

## 2. Screen: recruit (채용) — dc.html template lines 1429–1622

### Zone A — header row
- `h1` 채용 + one operational stat line (`rcHead`): `공고 {N}건 · 진행 지원 {N}명 · 면접 예정 {N}건`
  (live counts — judged 적합 under §4-12 because it is operational data, not prose).
- Primary CTA (signal/amber): **공고 등록** → opens posting composer modal.

### Zone B — posting list (single card "모집 공고")
Grid track: `minmax(140px,1.3fr) 96px minmax(150px,1fr) 52px 24px`; sticky header row;
J/K/Enter keyboard nav with selection ring `inset 2.5px 0 0 var(--signal)`; row click toggles
accordion (`rcOpen`).

Columns:
1. **공고 · 사업장** — role (800 ellipsis) + entity chip (muted) + conditional chips:
   `내부 공모` (purple, when `scope === "internal"`), `초안 · 게시` **button** (warn tone, when
   `draft`) → publish preflight. Second line: site (faint, ellipsis).
2. **충원** — fill bar (teal, ok-solid when full) + mono `hired / need`.
3. **진행 단계** — 4 stage chips 접수·서류·면접·오퍼, each with live count; zero-count chips
   dimmed (op 0.55).
4. **마감** — mono; `상시` = faint, date = warn-tx.
5. chevron (expand state).

### Zone C — expanded applicant sub-rows (accordion, canvas bg)
Per applicant: stage chip (44px min: 접수 muted / 서류 info / 면접 purple / 오퍼 ok / 탈락 danger)
· name **button** (opens candidate card, title "지원자 카드 — 지원서·평가·오퍼") · `보류` chip ·
`보충 대기` chip (warn) · status text `d` (ellipsis) · next-action button (`{next-stage}로 이동`,
`입사 확정`, `인력풀 등재`, or `재검토`) · overflow ⋯ menu: 보충 서류 요청 / 보류(다음 라운드 대기)↔보류
해제 / 탈락 처리(danger). Empty state: `아직 지원자가 없습니다`.

Next-button behavior (renderVals 19578-19587): rejected → reinstate; st==2 without assessment →
open card with fail-closed error; st==2 with assessment → open card (offer is card-only);
otherwise advance.

### Zone D — candidate card modal (right-anchored, `min(470px,94vw)`, dc.html 1516–1622)
- Header: initial avatar (ink circle) · name · `role · site` · close (Esc chain registered).
- **Stage stepper**: 접수 → 서류 → 면접 → 오퍼 → 확정 (dot+label; done=ok-solid, current=signal).
- **Rejected banner** (danger, when rejected): `탈락 · {reason} — 인재풀 보관` +
  **인력풀 등재 제안** (purple; talent-pool→workforce-pool conversion, `본인 동의 확인 후 가용`) +
  **탈락 철회 · 재검토**.
- **프로필** section: structured bullet lines `rs[]` = primary surface (§4-13 file-as-boundary);
  `원본 · {file}.pdf` = subordinate provenance chip button — click logs an audited view
  (`provenance 원본 (경계 포맷) — 구조화 프로필이 1급 개체 · 해시 검증 · 열람 감사`).
- **면접 평가** scorecard: 3 buttons 적합(ok)/보통(warn)/부적합(danger); once picked shows
  `{by} · {at}` (assessor signature + time). Recording audits `면접 평가 … 스코어카드 기록 · 평가자
  서명·시각`.
- **오퍼** box (visible when st≥3 and not rejected): mono amount · status chip
  (`회신 대기 · 발송 {date}`) · adjust input (`조정 금액 — 월 ₩3,500,000` placeholder) ·
  **조정 발송** · **회수** (danger).
- **Error banner** (danger, fail-closed messages, e.g. `면접 평가 기록 후 오퍼 제안 가능 (fail-closed)`).
- Footer: **불합격** (opens reason menu — mandatory enum: 경력 미달 / 직무 불일치 / 처우 불일치 /
  타사 수락 / 기타) · **보충 서류** · spacer · primary CTA (signal):
  st0=`서류 심사 통과` → st1=`면접 확정` → st2=`오퍼 제안` → st3=`입사 확정`
  (pool posting: `인력풀 등재 — 비상근`).

### Zone E — posting composer modal (공고 등록 · dc.html 650–720, logic 10492–10511)
Header chip: `JP- 초안`. Typed fields (§4-19, §4-27):
- 포지션명 — text, required (fail-closed with 현장·팀).
- 법인 — enum chips: 코스 / BESTEC / KNL / 그룹 / 스태핑.
- 현장 · 팀 — text, required.
- 고용 형태 — enum chips: 정규직 / 상주 교대 / 파트타임 / 일용직 (인력풀); picking 일용직 shows chip
  `확정 = 인력풀 등재 · 비상근` (pool posting: hire ≠ employee creation).
- 공개 범위 — enum: 외부 공개 / 내부 공모; internal shows chip `외부 비노출 — p12`
  (Cedar policy p12: external applicants must not see internal postings — existence itself hidden).
- 충원 — stepper (min 1). 마감 — text/date (`상시` default).
- 자격 요건 `req[]` — chip list with inline add (Enter) / remove (N+1 enum, §4-27-3).
- Footer: **초안 저장** (§3.9.0-③ direct-save whitelist, 모집 비노출) and **게시** (publish).

### Zone F — publish preflight gate (§4-29, logic `rcPublishDraft` 10512–10527)
Reuses `wfPre` checklist grammar (`_preGo` callback). Auto-judged checks:
1. 포지션·현장 정의 (role+site present)
2. 모집 인원·마감 (need+due present)
3. 동일 포지션 모집 중 중복 (no other non-draft posting with same role; note on failure:
   `중복 존재 — 기존 공고 마감 후 게시`)
plus one manual attest: 노출 범위 (exposure scope) — signature recorded. fail-closed: unmet =
cannot publish. Postflight note: `지원 유입 자동 집계`.

## 3. Simulated behaviors = behavioral contract (logic-class methods)

| Method (line) | Behavior the backend must support |
|---|---|
| `rcPostSave(publish)` 10496 | Create posting; JP- code; draft (audit `공고 초안 저장`, submit) or published (audit `공고 게시`, approve, reason states scope + headcount). fail-closed on missing role/site. |
| `rcPublishDraft` 10512 | Draft→published only through §4-29 preflight (3 auto checks + exposure attest); audit `공고 게시 — 프리플라이트 통과`. |
| `rcCandOpen` 12391 | Opening an applicant card is an **audited PII view** (`지원자 개인정보 열람 기록`). |
| `rcAdvance` 12650 | Stage advance 접수→서류→면접→오퍼. At st≥3 CTA: **non-pool** → 입사 확정 = employee object creation (사원, 입사 예정, `온보딩 — 포지션 프리셋 상속`) + 타임테이블 자동 연결 + 근로계약 개인 수신함(passkey) 발송 + hired++ (`충원 완료, 공고 마감 검토` at need); applicant leaves pipeline. **pool** → workforce-pool registration (비상근·호출, 재직 명부 비합산, 배정 시 건별 근로계약), NOT an employee. Both audited. |
| `rcAssess` 12405 | Scorecard write: {score∈적합/보통/부적합, by, at}; audited. |
| `rcOffer("send"/"adjust"/"retract")` 12415 | send: st→3, offer {amt, sent, status:회신 대기}, audit reason `처우 전결(DoA) 확인 · 서면 오퍼 · 회신 기한 7일`; adjust: same + `기존 조건 이력 보존` (version history); retract: offer cleared, st→2, audit `오퍼 철회 — 면접 단계 복귀 · 사유 기록 · 후보 통보`. |
| `rcPrimary` 12456 | Fail-closed: offer proposal blocked without recorded assessment. |
| `rcRejectWithReason` 12431 | Mandatory enum reason; applicant marked rejected; archived to talent pool; audit `사유: {reason} · 후보 통보 · 인재풀 보관 (재접촉 가능)`. |
| `rcReconsider` 12440 | Reinstate: rejected flag cleared, talent-pool row removed, history preserved; audit `결정 철회 — 이력 보존 · 파이프라인 복귀`. |
| `rcCandAct(doc/hold/reject)` 12688 | 보충 서류 요청 (docReq flag + notify), 보류 toggle, row-level 탈락. |
| `rcPoolPropose` 10485 | Talent-pool → workforce-pool conversion proposal: joins pool as 일용직, `본인 동의 대기` (not available until consent); dedupe by name; audit. |
| `candApply` 14461 | Candidate self-service apply: dedupe (`이미 지원한 공고입니다`), joins pipeline st=0, code `APL-…`, audit `지원자 셀프서비스 — … 「지원 접수」 합류 (OT-14)`. |

## 4. Adjacent surfaces consuming recruiting objects

- **postings** module surface (11230): internal career board — JP- rows, cols 공고/포지션/현장/상태,
  `지원하기` action (→ candApply), applied rows show `지원함`; recruiters get `공고 등록` action.
- **candidate** module surface (11220, v6 persona 서준호): "내 지원" — candidate-facing stage
  vocabulary `지원 접수 / 서류 검토 / 면접 / 오퍼 / 입사 확정`; stats 오퍼·모집 공고; own applications
  only (internal assessments invisible); offer row OFR- `회신 대기` (danger) → inbox.
- **inbox** (9185): offer = legal InboxDoc `OFR-2607`, basis `채용절차법 §4`, passkey read =
  receipt evidence, reply deadline shown; links back to the posting.
- **hr 직원 등록** (6201): direct-hire modal offers `채용 파이프라인 경유 →` as the standard path —
  recruiting is the canonical route into employee creation.
- **workforce** (11241): stats `채용 연동 신규` / `인력풀 공고` link back to recruit; pool rows carry
  `src: {jp, role, at}` provenance to the posting.
- **explore graph** (13916-13919): `편성 → post1(공고: 충원/채용 확정/지원자/상태) → appl1(지원자)` edges.
- **dashboard/forecast** (11146, 13293): insight `충원 공고 검토` action targets recruit screen.
- **messenger** (8999): recruiter↔candidate scoped DM thread (candidate persona sees only the
  recruiting contact — `personVisible` = 채용 담당 접점만, deny audited).

## 5. Ontology + PBAC categories (dc.html 8704, 8721, 13477–13487, 13778–13796, 12386)

- **OT-31 채용공고 (JP-)**: props 포지션(text)·법인(enum)·고용 형태(enum)·공개 범위(enum)·충원(number)·
  상태(lifecycle); links 지원자(1:N)·편성(N:1); analytics `충원율 = 채용 ÷ 충원`.
- **OT-14 지원자 (APL-)**: props 공고(text)·단계(enum)·평가(text)·오퍼(enum); card categories
  `공고·단계·평가 스코어카드·오퍼·제출 서류·인재풀`; data classes: 평가=민감, 오퍼=민감,
  제출 서류=개인정보, 평가 스코어카드=민감.
- **Policy p12** (active): `외부 인원(지원자)은 내부 공모 공고를 열람할 수 없다 — 존재 자체 비노출`
  (deny-by-omission).
- Viewer matrix: HR 담당(v2) has recruit/postings/candidate; 지원자(v6) has candidate/postings/
  inbox/msgr/support only.

## 6. Charter constraints binding this module

- §3.9 lifecycle: posting = 초안→게시→마감 (게시 = 권한 행위 §3.9.0-④; 초안 저장 = whitelist ③);
  no hard delete — closed postings retained.
- §3.10 guardrails: fail-closed everywhere (publish preflight, offer-without-assessment,
  reject-without-reason impossible, mandatory fields).
- §4-11/§4-12: compact one-line stat header, status = chips, zero explanatory captions.
- §4-19/§4-27: reasons and employment/scope are curated enums; req[] = N+1 chip list; forms are
  fail-closed gates, not validation prose.
- §5: every transition = audit event + back-references; one-click up/down chain moves
  (posting↔position, applicant↔posting, hire→employee, reject→talent pool→workforce pool).
- BENCHMARK honest gap (channel row): 스코어카드 협업 · 이메일 시퀀스 · 소싱 통합 · 구조화 면접 키트 —
  explicitly out of scope for this slice.
