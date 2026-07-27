# CAP-PAYROLL-CONSOLE frontend — Stage 3 adversarial verification

Date: 2026-07-24 · Verifier: fresh-eyes lane (did not author the code)
Scope: `web/src/console/payroll/**`, `web/src/i18n/payroll.ts` on branch
`claude/console-payroll-frontend-20260724` (build commits `a4ec7fac..b1313e58`).

Verdict: **PASS with fixes applied in this pass** (commit below). All findings
were fixed and re-verified; residual gaps are listed honestly at the end.

## 1. Module completion contract (roadmap 9 points)

| # | Requirement | Verdict | Evidence |
|---|---|---|---|
| 1 | List/overview layer | PASS | 회차 run list card (selectable, aria-pressed) + 5-step pipeline stepper; empty/loading/error+retry states are real (`PayrollScreen.tsx`, tests "truthful empty state", "retries an initial error") |
| 2 | Object detail layer | PASS | Run workspace: roster (readiness columns, per-line expand), exceptions, totals, schedule cards; per-line detail dl with hours/leave |
| 3 | Action/workflow layer | PASS | Full CTA machine close→calculate→exceptions→submit→decide(SoD)→schedule→attest(SUBMITTED_TO_BANK/PAID/FAILED+reason)→issue-payslips; preflight is attested and fail-closed; `preflight_blocked` 409 refetches checks |
| 4 | History layer | PASS (design parity), thin vs. a full audit feed | Milestone timestamps (calculated_at, submitted/decided_at, scheduled_at), rejection reason, disbursement state chain incl. FAILED reason, payslip issued/acknowledged counts, carried-over (이월) exception state. Actor names (decided_by/attested_by/resolved_by) are in the DTOs but not rendered — the design mirror does not render them either; the console audit module owns the full trail |
| 5 | ≥2 upstream links | PASS | attendance (stepper drill, preflight blocking refs, exception refs) and people (line 인사 카드, exception name) — both traversable |
| 6 | ≥2 downstream links | PASS | appr (CTA 결재함에서 열기 + stepper), laborcost (인건비 분석), inbox (수신함 delivery drill) |
| 7 | Deny-by-omission authz, no leakage | PASS | `canRead=false` renders title+denied with **zero** fetches (test asserts `api.GET` not called); read-only capability exposes zero mutation affordances (test); JWT-floor fail-closed while `/me/authz` loads; `request_only` denied (route tests); SoD submitter≠decider is server-enforced, mirrored by `canDecide` |
| 8 | Keyboard/focus/contrast/Korean/responsive | PASS after fixes | J/K/Enter roster cursor (test), focus-visible outlines on all interactive kinds, dialog: focus-in + Escape + **focus trap + focus restore (fixed this pass)**; token colors only (no hex/rgb in module CSS except scrim rgba, same pattern as the shell backdrop token file); all strings in `web/src/i18n/payroll.ts` (check-ui-strings clean); breakpoints 1180/780 with wrap/ellipsis. Column resize is pointer-only (reset button is keyboard-reachable) |
| 9 | Selection/drafts survive refresh/retry/Back | PASS | selected run + mask + column widths persist per-actor in localStorage and survive full remount (test "keeps the selected run across a remount"); mask defaults ON. One-line reason inputs (hold/decision/fail) are ephemeral by design |

## 2. Design fidelity vs. `Oyatie Console.dc.html` pay module (lines ~2111–2400, state machine ~15195–19190)

Verified exact parity:

- Header: h1 + subline (source_label · 산정 period · 지급일 · 대상 N명), status
  chip, single-CTA slot. CTA machine matches the prototype's decision tree and
  labels verbatim: 근태 마감 → 급여 계산 실행(pulse while calculating) → chip
  `남은 예외 N건` → 결재 상신 → 회수 후 재상신 준비 / 결재함에서 열기 → chip
  `지급 준비 완료 · 이체 예약됨`, extended past the prototype's end state with
  이체 예약/명세서 발송 per the extended lifecycle contract.
- Stepper: 5 steps (근태 마감·계산·예외 검토·결재·이체) with done ✓/number
  dots, sub-labels (`N명 완료`, `N건 남음`, `재상신 필요`…), drills on close
  (→attendance) and approval (→appr) exactly where the prototype makes steps
  clickable.
- Zones = design `pay` layout: main roster (`reg`) + side `ex`/`cost`/`sched`.
- Roster: title+count chip, 이름·법인 검색 (only when calculated), the three
  gate texts match `payGateRegText` variants verbatim, sticky header with
  per-column drag resize, J/K hint honored via keyboard handler, audit footer
  `급여 열람은 감사 로그에 기록됩니다`.
- Exceptions: dot+meta (`남음 N / total`, `N건 완료`, `계산 대기` — verbatim
  `payExMeta`), 처리 후 상신 가능 hint, severity-toned kind chips, name drill,
  masked amounts, expand → detail lines + purple linked-ref chips (kind+code)
  → object routes, 확인 처리 / 이번 회차 보류 (+reason, an addition), resolved
  (확인됨/보류·다음 회차/이월) states.
- Totals: tag chip (계산 전/계산 미완료/확정 계산), account status line from
  live ACCOUNT_VERIFICATION exceptions (오류/보류/0건 tones = `payAcct`),
  laborcost drill.
- Schedule: 4 milestones (계산·예외 검토/결재/이체/명세서 발송) with dot +
  time + 완료/진행/대기 chips, retry footer.

Deliberate deviations (all truthfulness- or contract-driven, none silent):

1. Run list card added (completion-contract list layer); the prototype's
   회차 시리즈 chip is omitted — no series object/route exists in the backend.
2. Roster shows readiness columns (근무일/연장/원천/계산 상태), not
   기본급/수당/공제/실지급 — `PayrollLineSummary` carries no amounts; rendering
   them would fabricate. Row click expands readiness detail with the 인사 카드
   drill instead of opening the person card directly.
3. Totals omits per-entity bars, 사업자 부담, 전월 delta (no backend data);
   adds default-on masking (금액 표시 toggle) and the payable release-gate chip.
4. Attendance close runs in-module via the attested preflight dialog (backend
   contract) instead of routing to the attendance screen; the attendance drill
   remains on the stepper and on blocking refs.
5. Layout presets/card drag/sheet(DLP) view omitted; column resize + 기본 배치
   reset built. Head-chip deadline texts (승인 기한 …) omitted — no deadline
   field exists. Exceptions-card gate button variants are folded into the
   roster gate + header CTA (single CTA source).

## 3. Findings (this pass) — all FIXED and re-verified

| # | Severity | Finding | Fix |
|---|---|---|---|
| F1 | **High (truthfulness)** | Collection reads relied on server default paging (`DEFAULT_LIMIT=100`, `MAX_LIMIT=500` in `backend/crates/payroll/adapter-postgres`): a 1,284-line roster silently showed 100 rows while search filtered only loaded rows; exceptions page likewise | `payrollApi.getRun`/`listExceptions` now send `limit=500` and walk offsets to the server-reported total (empty-page guard); `listRuns` requests limit=500. Regression test "walks run-line pages to the server total" added |
| F2 | Medium (correctness) | `timeText` sliced raw ISO (`Z`) strings — rendered UTC clock times to KST operators for 이체 예약/milestones | Local `Intl.DateTimeFormat("ko-KR", …)` per sibling-module convention; non-date passthrough |
| F3 | Medium (a11y) | Preflight dialog had no focus trap and no focus restore; attestation checkbox survived a preflight re-fetch (stale attestation over refreshed checks) | Tab/Shift-Tab trap within the dialog, focus restored to the opener on every close path, attested reset on each (re)fetch |
| F4 | Low (a11y) | `role="list"` container included non-listitem children (grid header, empty status) | rows-only `role="list"` wrapper |
| F5 | Low (robustness) | `nts_tax_row_status` had no unknown-value fallback (undefined chip class/label on a new server enum) | `ntsLabel` + tone fallback, `ntsStatus.unknown` string added |
| F6 | Low (docs) | `payrollApi.ts` and `mount.json` referenced `docs/evidence/console/CAP-PAYROLL-CONSOLE/design-contract.md`, which does not exist in this worktree or on main | References corrected to the mount manifest + design mirror; paging contract note added for the integrator |

Checked and clean: error envelope `{error:{code,message}}` parsing (code
surfaced, no synthesized success — test), no N+1 (detail+exceptions fetched as
one parallel pair; zero per-row fetches), no repeated-query params in use, no
TODO/FIXME/stub/`test.skip`/`.only`/dead controls (grep), purity gate (no
cn/clsx, plain string classNames), no inline Hangul in components, token
colors only, `aria-busy`/`role=status`/`role=alert` states truthful,
CALCULATING poll (4s) cleans up, abort/generation fencing on session remount,
authz hook is byte-parity with the production exemplar.

## 4. Test/gate evidence (after fixes)

- `npx vitest run src/console/payroll` → 4 files, **25/25 passed**
- `npx tsc -b` → exit 0
- `npx eslint src/console/payroll src/i18n/payroll.ts --max-warnings 0` → exit 0
- `node scripts/check-console-purity.mjs` → OK (407 files)
- `node scripts/check-ui-strings.mjs` → zero payroll findings; fails only on
  pre-existing untouched `web/src/features/facilities/FacilitiesWorkflowPage.tsx`

## 5. Residual open items (honest)

1. Lifecycle routes remain locally typed until the integrator lands
   `backend/openapi` + regenerated `clients/ts`; divergence after regeneration
   is a defect in this module. The exceptions route must accept
   `limit`/`offset` (recorded in mount.json).
2. Responsive behavior is CSS-breakpoint verified only (jsdom cannot evaluate
   media queries).
3. Full `npm --workspace web run lint` stays red from the pre-existing
   facilities check-ui-strings violation (outside this lane's ownership).
4. Integrator todos in mount.json (MOUNTED_SCREEN_KEYS, SCREEN_REGISTRY, nav
   badge slot, `Feature::PayrollRunManage` backend manifest).
5. Design omissions listed in §2 (series chip, pay-amount columns, entity
   bars, layout presets, sheet/DLP view, deadline chips) are deliberate,
   pending backend data/objects that do not exist yet.
6. Self-service payslip rendering belongs to the inbox surface; this module
   exposes `myPayslips()` and drills to /inbox only.
7. Column resize is pointer-only (keyboard users have the 기본 배치 reset);
   run-history actor names are not rendered (design parity, audit module owns
   the full trail).
