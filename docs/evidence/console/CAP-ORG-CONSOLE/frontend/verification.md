# CAP-ORG-CONSOLE frontend — Stage-3 fresh-eyes adversarial verification

Verifier: independent sessions, 2026-07-24 (pass A) and 2026-07-25 (pass B). Scope:
`web/src/console/org/**`, `web/src/i18n/org.ts`, evidence manifests. Neither verifier wrote the
module; every claim below was re-derived from the code, the design mirror
(`docs/design/oyatie-console/"Oyatie Console.dc.html"`, change-log 190), and fresh gate runs.

## Verdict

PASS with fixes applied (pass A: 5 findings; pass B: 3 findings — all fixed and re-verified) and
honest open items listed at the end. No stubs, no TODO/FIXME, no test.skip/.only, no dead
controls, no raw colors, no inline Hangul in components, no fabricated data found.

## Findings fixed — pass A

| # | Finding | Fix |
|---|---------|-----|
| 1 | Org-change modal step separator rendered `>`; design (dc.html line 7224) uses `›` | one-char change, `OrgChangeModal.tsx` |
| 2 | `applyPendingOps` REASSIGN merge concatenated positions — duplicate 직급 titles produced duplicate React keys in `OrgTeamCard` (`key={position.title}`) and split rows for one title | merge-by-title (sum totals, concat employees) in `orgTree.ts`; asserted in `orgTree.test.ts` |
| 3 | Leaving edit mode kept a stale "삭제 차단" guard banner on screen | `closeEdit` clears the guard, `OrgChartScreen.tsx` |
| 4 | Non-conforming ARIA: `role="tree"`/`role="treeitem"` on non-focusable divs without arrow-key management | replaced with `role="group"` container + `aria-expanded` on the entity toggle button (standard disclosure pattern) |
| 5 | Test gap: settlement gating (폐지 보관 blocked until every checklist item settled) had no test | added evidence-topology test at the fetch boundary asserting the real completion route and CTA gating |

## Findings fixed — pass B (second independent verifier)

| # | Finding | Fix |
|---|---------|-----|
| 6 | 10 dead i18n keys (`offline`, `ocNew`, `ocDrafter`, `ocSupersedes`, `ocDetailError`, `collapseTeams`, `entitySlug`, `entityStatus`, `team`, `ocStage.unknown`) — `offline` in particular implied an offline UI state that is not wired (network failures surface through the repo-standard `Error.message` path, same as the production exemplar) | deleted from `web/src/i18n/org.ts` and the `i18n.json` manifest; no dead strings remain |
| 7 | Persisted `openChangeId` restored the org-change modal even when `canReadChanges` was revoked between sessions — rendered a denied affordance as an error dialog instead of absence | restore now gated on `capabilities.canReadChanges` (`OrgChartScreen.tsx`); new test proves no dialog and zero org-change fetches |
| 8 | Traversable-link contract point had no test: no assertion that team → messenger/people and events → audit drills fire the real navigation targets | drill assertions added to the team-card and read-only-modal tests (`onNavigate` called with `"messenger"`, `"people"`, `"audit"`) |

## 9-point module completion contract

1. **List/overview layer** — 조직도 tree canvas (three real sources merged: hr org-chart ×
   org-entities × identity regions/branches), root stat bar (재직·법인·사업장·팀, derived, compact
   stat bar not KPI cards), 조직 변경 결재 strip listing org-changes with kind/status chips.
2. **Object detail layer** — entity card (name, slug, status chip, headcount, org counts), site
   card (region, 운영/중지 chip), team card (책임자 derived from leader-titled position with
   조직도 조회 chip, headcount, 직급 구성 with per-person drills), org-change detail modal
   (stage chips, stat strip, proposal ops, preflight report, approval line, settlement checklist,
   event history).
3. **Action/workflow layer** — edit-mode sandbox (rename/deactivate/create branch, create region,
   org-unit reassign with merge projection, blocked team delete with guard), dirty banner → staged
   governance modal: 초안 저장 → 사전점검 (blockers gate submission) → 결재 상신 → ordered SoD
   decisions (next pending step only; approve or 반려+사유) → 발효 → (폐지) 정산 checklist → 보관;
   기안 취소 with reason at DRAFT/PRECHECKED. Every action goes through the real route on the
   authenticated client and adopts the server readback (no optimistic fabrication).
4. **History layer** — 활동 이력 — 감사 체인 section on every loaded change + 감사 로그 drill to
   the audit screen.
5. **Traversable links** — downstream (wired here): team → people (팀원 명부), team → messenger
   (팀 채널), entity → people (소속 명부), person name → people, events → audit. Upstream (hr
   person card → org, ontology 조직 node → org) live in those modules' ownership — open item.
6. **Deny-by-omission** — capability projection from the canonical authz projection with JWT floor
   fail-closed (`useOrgConsoleAuthz`); `canReadTree=false` renders the denial state with **zero**
   fetches (tested); 401/403/404 on side reads render absence, not error (tested); org-changes 404
   renders the truthful "API not yet deployed" state (tested); approval/draft/apply affordances
   absent without the matching capability (tested). Backend remains the enforcing authority.
7. **Keyboard/focus/contrast/Korean/responsive** — dialogs receive focus on mount, Escape closes
   (tested via keyboard-only open/close), all actions are real `<button>`s, entity toggle uses
   `aria-expanded`, token colors only (182 `var(--…)` refs, zero raw colors; scrim is the
   `color-mix` token pattern), flexible column widths absorb Korean expansion, tree scrolls inside
   `.org-canvas` (`overflow:auto`, `min-width:940px` inner per design) so narrow viewports never
   overflow the page.
8. **Survival** — sandbox ops + open change id persist in `sessionStorage` keyed per actor;
   remount with a new session fence restores the dirty banner and proposal (tested); strip retry
   and tree retry re-fetch without losing the sandbox.
9. **Truthful states** — loading (`aria-busy` + status line), empty (표시할 조직이 없습니다),
   denied, error+retry, changes-unavailable; every visible datum is a backend response or a
   derived count over backend responses (root stats, unit heads, merge projections labelled 임시
   via the dirty banner and pending chips).

## Design fidelity (dc.html §ORG CHART line 1315–1427, org-change modal 7211–7284, team card 7115–7148, entity card 7150–7209)

Matches: header layout (h1 + warn dirty banner with 개편 결재 CTA + pencil 편집/완료 toggle, same
SVG path), root card with `--signal` border above a spine, horizontally scrolling columns
(min-width 940 canvas), entity card with corner `i` info button (법인 정보 카드), site rows with
mono meta, team rows opening 팀 정보 카드, edit-mode inline rename inputs, team delete ✕ affordance
(built: guard-blocked — no team-delete op exists; teams are `employees.org_unit`), + 사업장 추가
dashed button, org-change modal zone-for-zone (glyph header, stage chip, step chips with ›
separator, 변경 유형 picker, 발효일, 대상/인원/사업장/팀 stat strip, danger/warn preflight banners
with identical triangle SVG, 결재선 rows with role chip + ✓ 승인, 폐지 정산 rows with 정산 완료,
CTA-left/닫기-right footer), team card (team SVG avatar, 책임자/인원 grid, 팀원 명부 + 팀 채널
buttons), entity card (initial avatar, 조직 개편 결재 + 소속 명부 buttons).

Deliberate, truthful deviations (design simulates data the backend does not have):

- Root card shows a derived stat bar instead of the simulated group-holding name/meta line (no
  group-name endpoint; meta subtext would also violate the no-caption grammar).
- Teams render under the entity column, not nested under sites — hr org-chart has no team→site
  relation (`employees.org_unit` TEXT).
- Native `type="date"` input for 발효일 instead of the prototype's free-text mono input.
- Reject (반려+사유) and cancel (기안 취소+사유) exist beyond the prototype's approve-only line —
  required by the backend contract's REJECTED/CANCELLED statuses.
- 사유 field added (contract requires a reason; prototype omits it).
- Pin/split-panel (핀 — 분할 패널 고정) and drag-snap on cards: console-shell-wide prototype
  feature, not owned by this module — open item below.
- Entity rename input in edit mode: no RENAME_ENTITY op in the org-change contract — omitted, open
  item below.
- + 팀 추가, + 법인 추가 wizard, corporate fields (대표이사/설립/사업자번호/소재지), 관할 chips,
  clearance-gated 재무 요약, drag-drop team moves, per-site headcount/subsN: no backend source or
  route — named follow-ups (see open items).
- Region+branch composite creation: with zero regions the add-site form proposes CREATE_REGION
  only; CREATE_BRANCH requires an existing `region_id`, so a branch inside a still-pending region
  is not expressible in the op contract — contract follow-up.

## API-module contract fidelity

- Generated-client reads (`regions`, `branches`, `orgChart`, `me`) use typed `components["schemas"]`
  DTOs; org-change routes go through one documented `rawClient` cast on the authenticated
  `ConsoleApiClient` (auth/refresh preserved), deleted when the OpenAPI schema lands.
- Error envelope identical to the production exemplar (`{error:{message}}` with status-coded
  fallback), tested for 403-with-message and 409-without-envelope.
- Query params are scalars only (no repeated-query pitfall); list+detail topology, no N+1; every
  mutation adopts the server readback; `AbortSignal` threaded through every call; stale responses
  fenced by generation tokens and whole-screen session fences.

## Gates (fresh runs, pass B)

- `npx vitest run src/console/org src/i18n` → **28/28 passed** (5 files)
- `npx eslint src/console/org src/i18n/org.ts --max-warnings 0` → clean
- `npx tsc --noEmit -p tsconfig.json` → clean
- `node scripts/check-console-purity.mjs` → 410 files clean
- `node scripts/check-ui-strings.mjs` → fails **only** on pre-existing out-of-lane
  `src/features/facilities/FacilitiesWorkflowPage.tsx` (spine commit fd93fbdd); no org file flagged
- grep sweeps: no TODO/FIXME/HACK, no `test.skip`/`.only`, no raw colors (`#hex`/`rgb`/`hsl`) in
  `org.css`, no `cn(`/`clsx`, no inline Hangul outside tests/i18n, no unused i18n keys
- mount.json integrator claims re-verified against the spine: `orgchart` nav item + gate
  `g(DIRECTORY_ROLES, [EMPLOYEE_DIRECTORY_READ])` at `nav.ts:164-169`, ko label at `ko.ts:971`,
  and `orgchart` absent from both `MOUNTED_SCREEN_KEYS` and `SCREEN_REGISTRY` (dark, as declared)

## Open items (honest)

1. Integrator: add `"orgchart"` to `MOUNTED_SCREEN_KEYS` and `SCREEN_REGISTRY` per
   `manifests/mount.json`; exposure via `EXPOSED_SCREEN_KEYS` stays a roadmap-owner call.
2. Integrator: replace hand-typed org-change DTOs with generated `components["schemas"]` types and
   delete the `rawClient` cast once `backend/openapi` + clients land.
3. Backend lane: `org_change_*` Feature variants + org-change routes; until then the screen is a
   read-only tree with a truthful unavailable state for the changes strip.
4. Design follow-ups blocked on backend contract: entity rename op, + 팀 추가, + 법인 추가 setup
   wizard, entity corporate/finance/jurisdiction fields, drag-drop team moves (keyboard path
   shipped), per-site headcount, region+branch composite creation, group-holding root card name.
5. Console-shell follow-up: pin/split-panel (핀) affordance on object cards is a shell-wide design
   feature no module currently implements — belongs to the shell lane, not per-module.
6. Upstream links into this module (hr person card → org, ontology 조직 node → org) are owned by
   those modules.
7. Pre-existing repo gate failure outside this lane: `check-ui-strings` red on
   `src/features/facilities/FacilitiesWorkflowPage.tsx` — needs its owning lane.
