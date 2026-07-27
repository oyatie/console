# CAP-EVALUATION-CONSOLE frontend — Stage 3 adversarial verification

Fresh-eyes verification of the Stage 2 build (commits `f32f7aa5..e105927f`) against the
module completion contract (docs/program/console-enterprise-roadmap.md), the design
mirror (docs/design/oyatie-console/"Oyatie Console.dc.html", REVIEW section L1604-1730,
scorecard dialog L7642-7674, person-view RV ledger L7990-8005, JS state L19639-19680),
and the console UI grammar. Verifier did not author the code under review.

## Verdict

PASS after 4 fixes (commit below). All gates green post-fix:
`vitest run src/console/evaluation` 25/25 (was 23, +2 regression tests), `eslint
src/console/evaluation src/i18n/evaluation.ts --max-warnings 0` clean, `tsc -b` clean,
`check-console-purity.mjs` clean (407 files), `check-ui-strings.mjs` clean for this
module (sole remaining violation is pre-existing `src/features/facilities/
FacilitiesWorkflowPage.tsx`, committed at `fd93fbdd`, an ancestor of this lane's base).

## Findings (all fixed)

1. **Ledger 404 rendered as retryable error, not deny-by-omission status**
   (`EvaluationScreen.tsx` person view). Cycle and subject zones map 403→권한 없음 and
   404→표시할 수 없는 개체 statuses; the RV- ledger mapped only 403, so a 404 from
   `GET /evaluation/employees/{id}/reviews` surfaced the raw backend message in a
   danger alert with a retry button — contradicting the lane's own claim and the
   authz-without-leakage bar. Fixed to the same 403/404 status pattern; regression
   test added ("denied-by-omission ledger 404 as a status").
2. **Dead control for a submit-only capability**. The my-tasks person-name button set
   `view: person`, but the detail panel renders only under `canRead` — for
   `evaluation_submit`-without-`evaluation_read` the click did nothing. The design
   mirror itself has the `whoOn`/`whoOff` grammar (button vs plain span, L1694-1699).
   Fixed: plain `span` when `!canRead`; regression test added; 작성 flow unaffected.
3. **Invalid HTML: `<h2>` inside `<button>`** (SubjectZone header drill). `button`
   permits phrasing content only. Inverted to `<h2><button class="evaluation__link">`
   (+ `font-weight: inherit` so the heading weight is preserved).
4. **CSS cascade defect: generic card/dialog button rule overrode every classed
   variant**. `.evaluation__card button` (specificity 0,1,1) beat all single-class
   buttons (0,1,0): `.evaluation__filter-chip--on` and `.evaluation__segment-btn--on`
   lost their ink background — the selected stage filter and the selected S–D grade
   were visually indistinguishable from unselected — and `.evaluation__row`,
   `.evaluation__task-title`, `.evaluation__link` gained borders/steel text. Rewrote
   the generic defaults as zero-specificity `:where()` rules so authored variants win;
   hover/disabled kept at zero specificity (jsdom cannot assert computed cascade — this
   fix is by-construction, verified by selector-specificity analysis).

## Module completion contract — point by point

1. **List/overview layer** — cycle rail with stage filter chips (전체/준비/진행/조정/확정/보관,
   `aria-pressed`), rows with stage + D-day chips; compact stat bar (대상/자기/관리자/조정/확정,
   `.evaluation__stats`, not KPI cards); per-unit progress bars. PASS.
2. **Object detail layer** — cycle detail zone (chips, stats, per-unit bars, subjects);
   subject card (goals, SELF/MANAGER review cards, calibration zone, RV- code, grade);
   scorecard dialog fetches live subject detail. PASS.
3. **Action/workflow layer** — create cycle; preflight-gated stage transitions
   (blockers = danger chips disable fail-closed, advisories = warn chips); subject
   enrollment (employee typeahead + manager picker); goals replace-set editor
   (weight-sum readout, ≤20 rows); scorecard draft/submit (manager submit requires ≥1
   evidence, grade required); four-eyes calibration (zone omitted for the manager
   evaluator; reason required when the grade diverges). PASS.
4. **History layer** — audited person ledger (RV- entries, purple 평가 chip, 열람 기록됨
   info chip), drill both ways (ledger row → subject; subject/task → person). PASS.
5. **≥2 upstream + ≥2 downstream object links** — outbound: evidence chips drill to
   mywork/appr/dashboard (`consoleScreenPath`; all in `MOUNTED_SCREEN_KEYS`), 인사 명부
   → people. Internal: cycle↔subject↔person↔RV-. Inbound mounting is the
   integrator's (mount.json). PASS.
6. **Server-enforced deny-by-omission without leakage** — no fetch and no controls when
   neither read nor submit; capability projection via `deriveEvaluationCapabilities`
   over the parsed MeAuthzResponse with fail-closed JWT floor; 403/404 as statuses
   everywhere (post-fix); `request_only` denied (route test). PASS.
7. **Keyboard/focus/contrast/Korean/responsive** — native buttons throughout (Enter
   activation tested); dialog `aria-modal`, initial focus + focus return, Esc and
   backdrop dismissal tested; `:focus-visible` outlines; token colors only (0 raw color
   literals; teal chip = surface-on-#0f766e ≈5.9:1 light, #161c24-on-#2dd4bf dark);
   all strings via `web/src/i18n/evaluation.ts` (zero inline Hangul in components);
   960px single-column collapse, `flex-wrap` chip rows. PASS.
8. **Selection/drafts survive refresh/retry/Back** — cycle + view restored from
   per-actor sessionStorage (typo-guarded parse); scorecard drafts server-persisted
   (임시 저장 → PUT review, reconciled from the returned object); create/enroll forms keep
   values on failure; 뒤로 buttons on subject/person zones. PASS (scorecard dialog
   *open* state intentionally not persisted; the draft itself is).
9. **UI grammar** — no captions/subtitles/meta text (the mirror's header subtitle and
   scorecard footer caption are correctly re-expressed as chips/omitted); status =
   chips; plain string-literal classNames (purity gate); every on-screen noun
   clickable or absent (post-fix #2). PASS.

## Design fidelity (mirror REVIEW section vs built module)

Matches: two-zone layout (팀별 진행률 main + 내 평가 할 일 side per `LAYOUTS.review`);
progress-bar thresholds byte-equal to the prototype (`100→ok-solid, <50→warn-solid,
else teal`); task row = D-chip + person drill + solid 작성; scorecard = 440px dialog
(`min(27.5rem,94vw)`), title + OT-24 평가 chip, S–D segment with ink-on selection,
"평가 의견 (선택)" textarea, 취소/제출, submit disabled without grade, backdrop dismissal;
person view RV- history rows (mono code + cycle + grade tile + date) with 열람-audited
drill. Deliberate deviations (all justified): 배치 layout-preset/drag-resize omitted
(shell-level window grammar, out of module scope); auto-attached 근태/업무/KPI context
rows realized as real typed goals + evaluator-attached evidence links instead of
fabricated cross-module summaries (truthfulness bar; no aggregation API exists);
임시 저장 added beyond the prototype's 취소/제출 (server-persisted drafts); header
subtitle → chips (no-explanatory-UI gate). Additions beyond the mirror (cycle CRUD,
preflight, enrollment, goals, calibration, ledger endpoint wiring) realize the
HANDOFF §15/§16/§20 behavioral contract and the completion contract.

## API-module contract fidelity

Locally-typed transport matches api-sync.json field-for-field (page shapes, task
summary fields, transition/create → CycleDetail, save/submit → Review, ledger
`{items}`, lowercase review-kind path segments — asserted in `evaluationApi.test.ts`
including a real openapi-fetch client run proving URL/query/bearer serialization).
Canonical `{error:{message}}` envelope with status-carrying `EvaluationApiError`;
non-envelope bodies fall back to a generic message. No repeated-array query params in
this module. No N+1 (subjects arrive embedded in cycle detail; one fetch per view).
Cross-module calls use the *generated* typed routes: `/api/v1/employees?search`
(param verified in `clients/ts/src/schema.d.ts` listEmployees) and `/api/v1/users`
(UserSummary.display_name/is_active verified). All writes reconcile from returned
backend objects; every fetch is generation+AbortController fenced with a WeakMap
API-identity fence and session-key remount.

## Anti-fabrication sweep

Zero TODO/FIXME/XXX/test.skip/.only/stubs in the module; both `ponytail:` ceilings are
real and named (manager picker first-100, index keys on unsaved goal rows); no dead
controls post-fix; empty/denied/error/loading states are truthful statuses with
action-driving copy only.

## Open items (honest)

- DTOs stay locally typed until backend/openapi + client regen; divergence with the
  backend lane reconciles at consolidation per api-sync.json.
- Integrator: add `evaluation` to `MOUNTED_SCREEN_KEYS` + `SCREEN_REGISTRY`
  (mount.json); nav entry + ko.ts key already present; module lands DARK.
- Manager picker fetches first 100 active users; needs a server-side search param when
  the directory outgrows a page.
- SELF-review authoring reachable via my-tasks only (client cannot resolve
  caller→employee linkage; server authorizes per-subject).
- ATTENDANCE/OTHER evidence kinds render as non-drill data chips (no attendance screen
  mounted).
- CSS cascade fix (#4) is verified by specificity analysis, not runtime assertion —
  jsdom cannot compute the cascade; visual QA belongs to the design-review lane.
- Pre-existing repo-wide check-ui-strings failure in
  `web/src/features/facilities/FacilitiesWorkflowPage.tsx` (not this lane).
