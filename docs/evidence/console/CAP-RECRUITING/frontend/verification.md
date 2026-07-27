# CAP-RECRUITING frontend — Stage-3 fresh-eyes adversarial verification

Verifier: independent stage-3 lane (did not write the code). Verified against the actual
code in `web/src/console/recruiting/**` + `web/src/i18n/recruiting.ts`, the design authority
mirror (`docs/design/oyatie-console/Oyatie Console.dc.html`, RECRUIT section, lines 1429–1603
+ recruit state/JS at 8900–8901, 10486–10517, 12391–12700, 14968–14979, 19548–19637), and the
module completion contract in `docs/program/console-enterprise-roadmap.md`.

Date: 2026-07-24. Branch `claude/console-recruiting-frontend-20260724` (not pushed).

## Verdict

PASS with 3 findings — all three fixed in this lane, suite re-run green.
No stubs, no TODO/FIXME, no `test.skip`/`.only`, no dead controls, no fabricated data found.

## Findings (fixed)

| # | Severity | Finding | Fix |
|---|----------|---------|-----|
| 1 | convention/purity | `CandidateCard.tsx` built a className with a template literal (`recruiting__score--${tone}`) — the only occurrence in the entire `src/console` tree; the purity gate only inspects `className` attribute literals, so a variable-built template class evades it. | Replaced with an explicit ternary over full string literals. |
| 2 | a11y/keyboard (contract §keyboard/focus) | All three modals (`CandidateCard`, `PostingComposer`, `PreflightModal`) and the card error shell rendered `role="dialog" aria-modal="true"` without moving focus into the dialog. The overlay `onKeyDown` Escape handler was dead until the user clicked inside (focus stayed behind the modal). Exemplar `ObjectCardModal` autoFocuses its close control. | `autoFocus` on the close/cancel/retry control of each dialog; new test asserts focus lands inside the card and Escape closes it without a prior click. |
| 3 | evidence-topology gap | The hire handshake — the module's hardest business outcome (OFFER + ACCEPTED offer → `POST /applicants/{id}/hire` → employee object link) — had zero test coverage; no test exercised `/api/v1/branches`, the field-level hire body, the fill toast, or the `hired_employee_id` → people link. | Added a full-path test: open card → 입사 확정 CTA → branches-backed form → field-level POST body assert (incl. prefilled `position`/`site`/`base_pay` from posting+offer) → toast `충원 1 / 2` → 직원 카드 열기 → `onNavigate("/console/people")`. |

## Module completion contract — point by point

1. **List/overview** — PASS. Posting accordion with live stat header (`headStatLine`, exact
   `rcHead` formula from dc.html:19637: 공고 N건 · 진행 지원 N명 · 면접 예정 N건 where 면접 = interview
   stage count), fill bar, 4 stage-count chips (zero → dim), 상시/warn mono deadline.
   Tested: "renders the live stat line and posting grammar from the server list".
2. **Object detail** — PASS. Posting detail = expanded subrow (server `GET /postings/{id}`);
   applicant detail = right-side candidate card (stepper 접수→확정, profile + 원본 provenance
   chip, scorecard with by·at, offer chain, enum-reason rejection banner, hire form).
3. **Action/workflow** — PASS. Composer (typed enum chips, headcount stepper, requirements
   chips, fail-closed validation), §4-29 preflight publish gate (server checks + exposure
   attest; 422 check vector re-rendered via `RecruitingApiError.checks`), advance/hold/doc/
   reject(enum)/reinstate, offer extend/adjust/withdraw/record-reply (one live EXTENDED),
   hire handshake, close posting, DRAFT edit (PUT with `expected_updated_at`).
4. **History layer** — PASS. `events[]` timeline in the card (`eventLabel` enum map, unknown
   actions render raw — truthful). Tested via ADVANCED event assertion.
5. **≥2 upstream + ≥2 downstream links** — PASS with caveat. Up: applicant→posting (card meta
   link focuses the row), posting→`position_ref`→objectExplorer (renders only when non-null).
   Down: hire→employee (`/console/people`, only for HIRED + `hired_employee_id`; now tested),
   reject→talent-pool section (`GET /talent-pool`, tested). Caveat: the talent-pool downstream
   is a co-screen section rather than a cross-screen object link; both cross-screen targets
   resolve only after their screens are exposure-approved (recruit itself is DARK).
6. **Deny-by-omission authorization** — PASS. Grants-not-roles (`recruiting_read`/`recruiting_manage`/
   hire = manage + `employee_directory_manage`) over the parsed `/me/authz` projection with JWT
   floor fail-closed (`request_only` → denied, tested); denied-before-fetch (no GET fired,
   tested); server 403/401 → denied state, never an error alert (tested); read-only grant
   omits every manage affordance (tested); talent-pool denial hides the whole section.
7. **Keyboard/focus/contrast/Korean/responsive** — PASS after fix #2. Roving J/K/Arrow/Home/End
   + Enter accordion (tested), `:focus-visible` outlines on all 13 interactive classes, token
   colors only (no hex except the design's own `#141a21` on-signal ink, identical to dc.html),
   860px breakpoint collapses the stages column, flexible wrap for Korean expansion.
8. **Selection/drafts survive refresh/retry/Back** — PARTIAL (recorded honestly). Selection
   (`openId`, open card) and composer drafts survive retry, server reconcile, and conflict
   re-reads (`reload({openId, card})`); the session fence intentionally resets them on
   actor/tenant/api change. A full browser refresh resets to the list — the route contract is
   parameterless (`Record<string, never>`), so no URL/sessionStorage persistence exists.
9. **Truthful states** — PASS. Distinct loading/empty/denied/error(+retry)/conflict copy per
   surface; every mutation is server-reconciled (re-reads list + open posting + card) with
   `expected_updated_at`; 409 shows the conflict copy AND re-reads server truth (tested).

## Design fidelity (re-extracted RECRUIT section vs build)

Matches: header (h1 채용 + stat line + signal 공고 등록 CTA with identical plus icon), panel
title 모집 공고, sticky column header 공고·사업장/충원/진행 단계/마감, exact grid
`minmax(140px,1.3fr) 96px minmax(150px,1fr) 52px 24px`, ent chip / 내부 공모 purple chip /
초안·게시 warn action chip with the §3.9 title, fill bar teal→ok-solid at 100%, 4 stage chips
(dim at zero), 상시 faint vs date warn mono, chevron flip, subrow stage chip min-width 44px +
name link + 보류/보충 대기 chips + next-action ghost + ⋯ menu (170px, 보충 서류 요청 / 보류
(다음 라운드 대기)↔보류 해제 / rule / 탈락 처리 in danger), 아직 지원자가 없습니다 empty, card
min(470px,94vw) 92vh right-anchored with 38px avatar, 5-step stepper, rejected danger banner
`탈락 · {reason} — 인재풀 보관` + 탈락 철회 · 재검토, 프로필 bullets + 원본 provenance chip,
면접 평가 적합/보통/부적합 with mono by·at note, offer box (mono amount, 회신 대기 warn chip,
조정 발송/회수), footer 불합격 (upward enum menu — the exact 5 design reasons 경력 미달/직무
불일치/처우 불일치/타사 수락/기타) + 보충 서류 + signal CTA ladder (서류 심사 통과→면접
확정→오퍼 제안→입사 확정), fail-closed copy 면접 평가 기록 후 오퍼 제안 가능 (fail-closed).

Deviations (each justified, none silent):

| Deviation | Rationale |
|---|---|
| Preflight reveals 게시 only when publishable+attested (design dims an always-present button) | deny-by-omission over dead-looking controls; flagged for design review (open item) |
| Subrow ⋯ 탈락 처리 opens the card instead of instant reject | real contract requires an enum reason; design's subrow reject is reasonless simulation |
| Rejected banner omits 인력풀 등재 제안 | no workforce-pool backend — deliberate omission (api-contract.json), never a fake control |
| 원본 chip is non-interactive (design has a file button) | no document endpoint; a dead download button would violate the truthfulness bar |
| Card CTA at OFFER = hire handshake form gated on recorded ACCEPTED reply (design's CTA advances directly) | §20 CRUD governance: employee creation needs the full handshake; record-reply substitutes the absent candidate persona |
| J/K roving is focus-based on row toggles (design uses a global screen listener + rcSel ring) | roving-tabindex is the a11y-correct equivalent; open row keeps the inset signal ring |

## API/module contract fidelity

- Error envelope: canonical `ErrorBody { error: { code, message } }` — `envelopeMessage`
  matches openapi.yaml:21809; status carried; tested (422 message, checks vector).
- Denied (401/403) and conflict (409) classified without leaking into generic errors (tested).
- `GET /api/v1/branches` returns a bare `BranchSummary[]` per openapi.yaml:8047 — the
  `listBranches` array handling is correct (now covered by the hire test).
- No N+1: one list call embeds `stage_counts`; reload batches ≤4 parallel reads via
  `Promise.allSettled`; mutations reconcile with a single batched reload.
- Query serialization delegates to openapi-fetch (`undefined` params dropped, tested).
- One documented boundary cast (`ponytail:` comment) until clients/ts regenerates; weakest
  field-level assumptions remain flagged `assumption:true` in api-contract.json
  (`RecruitStageEventView` shape, `assessment` shape, preflight check keys).

## Gates re-run after fixes

- `npx vitest run src/console/recruiting` — 4 files, **27/27 passed** (25 prior + 2 new).
- `npx eslint src/console/recruiting src/i18n/recruiting.ts --max-warnings 0` — clean.
- `npx tsc -b` — clean.
- `node scripts/check-console-purity.mjs` — 411 files clean.
- `node scripts/check-ui-strings.mjs` — recruiting files clean; repo-wide run still fails only
  on pre-existing `src/features/facilities/FacilitiesWorkflowPage.tsx` (other lane, commit
  fd93fbdd) — must be fixed by the facilities lane/integrator before a repo-wide green.

## Residual open items (honest)

1. Refresh persistence (contract point 8): in-memory selection only; parameterless route.
   If refresh-survival is required for recruiting, it needs an integrator decision
   (URL param vs sessionStorage) — neither exists in the closest exemplar (production).
2. Reload-reconcile edge: a 403 arriving on the *open-posting* or *card* read inside a batch
   reload renders the section error+retry (retry then resolves to denied); only the list read
   flips the screen to denied directly. Behavior converges; cosmetic asymmetry.
3. Assessment buttons remain pressable on a HIRED applicant (backend is the authority and the
   design keeps scores clickable post-stage); server rejection surfaces in the card banner.
4. All prior open items from the build report stand: integrator mount actions (mount.json),
   contract sync on clients/ts regeneration, deliberate omissions list, DARK cross-screen
   link resolution, preflight dimmed-button design-review flag.
