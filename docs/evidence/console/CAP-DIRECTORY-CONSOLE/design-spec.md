# CAP-DIRECTORY-CONSOLE — design spec extract (stage 1 scout)

Source of authority: design mirror `docs/design/oyatie-console/` (byte-exact sync, change-log 190).
Extracted 2026-07-24 from `Oyatie Console.dc.html` (20,129 lines) with grep anchors `directory` / `주소록`,
plus DESIGN.md, HANDOFF.md, AGENTS.md, ROADMAP.md, TODO.md, BENCHMARK.md, CLAUDE.md, README.md.
All content below is **design intent**, not implementation-status evidence.

## 1. Module identity

- Screen key: `directory` · nav label `주소록` · nav group **커뮤니케이션** (msgr · mail · notif · board · directory).
- Route (real console): `/console/directory`.
- Benchmark row (ROADMAP §matrix): `주소록 | directory | Workday·People | Person | 🟡 v1 (PEOPLE 동적·메시지/메일/카드)`; P4 target `Person·조직`.
- Audience (DESIGN §4.8): **전 직원** — the directory is one of the modules every employee uses
  (전자결재·급여 셀프서비스·메신저·메일·알림·주소록·게시판). Prototype nav shows it to every internal
  persona (v1–v10); the only restriction is the external-applicant persona (v6), see §5.
- Mobile (TODO §mobile): directory is one of the **7 mobile employee-app modules**; it sits in the
  bottom-tab "더보기" sheet (`{ label: "주소록", scr: "directory" }`).

## 2. List/overview layer — shared module surface (`MOD_SCREENS`)

The directory is one of the 10 module surfaces rendered by the single shared template
(dc.html ~4555–4900, "모듈 서피스 (ERP·현장운영·컴플라이언스·분석·게시판·주소록 공통)"). Its config
(dc.html 11219):

```js
directory: {
  title: "주소록",
  action: { label: "새 대화", scr: "overview", palette: true },   // opens the command palette
  cols: ["연락처", "이름", "직책", "소속"],
  stats: [{ label: "임직원", v: "1,284" }, { label: "법인", v: "4" }],
  rows: dirRows
}
```

Rows are derived live from the person registry (dc.html 11016):

```js
dirRows = Object.keys(PEOPLE).map((nm) => ({
  id: "p_" + nm,
  code: pe.ext && pe.ext !== "-" ? "내선 " + pe.ext : "현장",   // contact-channel column (mono)
  c1: nm,                       // 이름
  c2: pe.title || "",           // 직책
  c3: pe.team || "",            // 소속(팀)
  st: pe.entity || "",          // status-chip slot reused as 법인 chip, tone "neutral"
  kv: [["직책", …], ["소속", team + " · " + entity], ["이메일", …], ["입사", …]],
  links: [{ label: "인사 카드", person: nm }],
  acts: [{ label: "메시지", thread: nm }, { label: "메일", mailTo: pe.email || "" }]
}))
```

Person seed fields (`PEOPLE`, dc.html 8547): `title(직책), team, entity(법인), ext(내선 | "-"), email?,
joined(입사 YYYY.MM), phone?, emp?(고용 형태), me?, thread?`. People with `ext === "-"` render code
`현장` (field staff without a desk extension).

### Shared surface zones (all inherited, not directory-specific)

1. **Header stat bar** (single compact row — never big-number KPI cards, §4-11):
   title `주소록` + stats as clickable label+value pairs (임직원 / 법인) + free spacer + **search input**
   (`검색`, multi-attribute over all visible columns) + sheet-open button (현재 목록을 협업 시트로) +
   화면 구성 (cfg) toggle + primary action button `새 대화` (opens command palette; palette people
   entries run `openPerson`, and `startThread` handles new-DM creation).
2. **화면 구성 row** (cfg, personal view): column chips add/remove/drag-reorder from ontology type
   attributes, stat add/remove, saved filter presets, widget select (분포 바/칸반), 행 선택 거동 segment,
   기본값 복원, 팀 배포 — 결재 (shared layout deploy = approval, §3.9.0-④).
3. **List table**: sticky header, grid track shared by all rows (`mod.grid`), sortable column headers
   (클릭 = 정렬 오름/내림/해제), row = `code | c1 | c2 | c3 | custom cells | status chip | chevron`.
   Rows are buttons: click = select detail; `draggable` — drag = reference-token attach (`[코드 제목]`
   payload). J/K/Enter keyboard nav; empty state = `검색 결과 없음` + `필터 해제 — 전체 목록` (+ action).
4. **Detail pane** (right, `mod.det`, default-on): code chip + status chip + title, kv rows
   (직책/소속/이메일/입사), link chips (`인사 카드` → person card), action buttons (`메시지`, `메일`).
   All link routing through single `modLinkGo` (code/node/person/thread/mail/appr/panel/screen).

### Directory-specific behaviors

- `links: [{ person: nm }]` → `openPerson(nm)` (person card modal — §3 below).
- `acts: 메시지` → `startThread(nm)`: reuse existing DM if `threads` has one for that person, else
  create DM `{kind:"dm", members:[me, name]}` + audit event `대화 개설` (cat submit, permit).
- `acts: 메일` → mail compose prefilled `mailTo` (person's email).
- Stats `임직원`/`법인` are seed numbers in the prototype; in the real console every stat must be a
  live PBAC-scoped count and a drill (§4.7-9: 집계 화면의 모든 수치는 원본 개체로 이동하는 버튼).

## 3. Object-detail layer — person card ("인사 카드", PERSONNEL CARD)

Template dc.html 7919–8115; logic (renderVals) 19884–20010. Opened via `openPerson(name)` from:
directory rows, @mention chips, messenger member lists, audit actor names, org chart, pay/leave rows,
map markers, notifications — the card is the console-wide person object surface.

Zones, top to bottom:

1. **Header**: photo slot (80×100) · 이름 + 직책 + 사번 chip (`OY-20…` derived) · **팀 (click = 팀 카드
   drill)** · **법인 chip (click = 법인 카드 drill)** · `재직` status chip · `출입 카드` compact toggle ·
   pin (분할 패널 고정, drag = quadrant snap) · close. Card is a **window-model citizen**: pinnable,
   draggable, Esc chain.
2. **출입 카드 compact mode** (#20): photo + basic info + 출입 권한 구역 chips + 최근 출입 record.
3. **기본 정보** (`전체 공개` access chip): 내선, 입사, 이메일, **휴대폰 (click = inline edit — direct-save
   whitelist §3.9.0-① 개인 연락처, input mask `010-0000-0000`, audit event on apply)**, 고용 형태.
4. **직무 정보** (`팀 내 공개` access chip + `Cedar 정책` button): 직무, 배정 포지션, **평가 이력 RV- rows**
   (visible only when viewer clearance ∈ {민감정보, 비밀}; each open = audited view).
5. **최근 업무 · KPI** (`관리자 · 열람 기록됨` chip): 이번 주 근무 h / 진행 업무 / 완료 — each from live
   state; linked work items (ref chip → object detail); empty = `연결된 진행 업무 없음`.
6. **상세 정보** (`민감 · 인사 책임자` chip): collapsed by default behind **`열람 — 기록 남음` gate**;
   after gated open shows masked 비상 연락처/급여 계좌/주소 + `열람 기록됨 · 감사 로그` badge.
7. **인사 관리 모드** (collapsible, admin): 인사 원장 (cross-module ledger — 입사/평가/근태/연차/급여/결재
   entries, each a link into the source module, clearance-gated), masked 급여 정보 + gated 열람,
   HR actions (정보 수정 · 발령·이동 · 연차 부여 · 징계·포상 · 휴직 처리 · 퇴사 처리 — all route into
   결재 compose prefill, never direct mutation).
8. **Footer**: self-card (`me`) → 퇴근/로그아웃 + 잔여 연차; other-card → **메시지** CTA (startThread).

History layer for the module contract = card ledger (§7) + person.view audit events + 입사/lifecycle
entries; upstream/downstream links = 팀 카드, 법인 카드, work objects, 평가 RV-, thread/mail.

## 4. Team / entity drill (linked objects)

- `onPTeamOpen` → `teamCard { name, head(팀장|반장 match), hc(인원수), path("법인 › 팀") }` — small
  linked-object card, itself linking back to member list.
- `onPEntOpen` → 법인 카드 (`entCard` index into orgData) or toast `법인 카드 미등록 (그룹 공통)`.
- These satisfy the ≥2 upstream (team, entity/branch) and ≥2 downstream (thread, mail, work objects,
  RV-) traversal requirement for STORY-DIRECTORY-001.

## 5. PBAC / audit semantics (design contract)

- **Single choke point** (§4-19): person visibility only via `personVisible`/`peopleAllowed`;
  @-autocomplete only via `mentionPeople`/`mentionScopes`. No surface may query the people registry
  directly. Deny-by-omission: the external/applicant persona sees only the recruiting contact
  (`peopleAllowed() → ["김성아"]`), and team/entity mention scopes are hidden entirely for it.
- `person:view` action (dc.html 10367): `if (action === "person:view") { al = peopleAllowed(); return !al || al.indexOf(resource) >= 0; }`.
- **Denied open is audited**: `openPerson` on an invisible person logs
  `인사 카드 열람 시도 / decision: deny / reason: fail-closed` + policy-deny toast.
- **Permitted non-self open is audited**: `열람 (cat view, target 직원 "X 인사 카드", permit)` — self-view
  (전성진 = me) records nothing (self-view right, HANDOFF §2).
- Sensitive sections are **category-gated** (기본 정보 = 전체 공개, 직무 = 팀 내 공개, 상세 = 민감·인사
  책임자, 평가/급여 = clearance): chips always show the access category; sensitive default-collapsed
  behind an explicit audited open (§4.5).

## 6. UI grammar constraints binding this module (DESIGN §4)

- No explanatory captions/subtitles/meta text; status = chips; compact 1-row stat bar (§4-11/§4-12).
- Every noun clickable/pinnable or absent (§4-1): name → person card, team → team card, entity →
  entity card, ext/email → action, codes mono.
- Rows/chips = drag sources (`objDrag`, §4-23); detail default = pinned panel (§4.7).
- Typeahead over enumeration for people pickers at production scale — 3,000명 기준, list caps +
  search prompt, server pagination is a HANDOFF contract (§4-27-4).
- Table: shared track formula, no per-row max-content, no horizontal scroll (minmax + ellipsis,
  `modNarrow` compact fallback), J/K/Enter, ends with spacer + fade (§4.7-1, §4-19).
- Korean-first labels via i18n mechanism (check-ui-strings gate in the real console), token colors
  only, className plain string literals.

## 7. Honest gaps the prototype leaves to the real backend (from BENCHMARK/HANDOFF)

- Stats (1,284 임직원 / 4 법인) are seed constants — real console needs PBAC-scoped live counts.
- PEOPLE is a client dictionary keyed by name — real console needs id-keyed, tenant-scoped, paginated
  directory reads (BENCHMARK "구조적 격차": 서버 페이지네이션·인덱스 검색 없음).
- Read-audit, deny-by-omission, and category gates must be **server-enforced** (HANDOFF §2, §7).
- TODO 181 (future, out of this lane): 주소록 거래처 탭 (client/vendor tab).
