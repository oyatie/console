# CAP-FIELD-CONSOLE frontend — stage-3 adversarial verification

Fresh-eyes verification of `web/src/console/field/**` (고객·현장) against the
module completion contract, the design mirror (`docs/design/oyatie-console/`,
change-log 190, `MOD_SCREENS.field`), and the api-contract/UI-grammar gates.
Verifier did not author the stage-2 code. All findings below were fixed in
commit `0a5aaf76` and re-verified green.

## Findings (all fixed)

| # | Severity | Finding | Fix |
|---|----------|---------|-----|
| F1 | critical | All four async ops shared one `generation` counter; `reconcile()`'s three parallel reloads invalidated each other, so **every successful mutation stranded the site list (and detail pane) in a permanent loading state**; a restored selection also raced the debounced list load on mount. Proven red by test "reconciles the list from the server after a mutation…" before the fix. | Per-pane op fencing: a load settles state only while it is its own pane's latest un-aborted `AbortController` (`settles(slot, controller)`); session/tenant invalidation stays on the remount fence key. |
| F2 | high | Intake ran create+link as one atomic mutation: a `linkTicket` failure after a successful `createTicket` kept the form open with the draft — **resubmitting duplicated the ticket**. Proven red by test "keeps a created ticket when site-linking fails…". | The created ticket is kept and selected; a link failure surfaces as the action error while the intake completes; the ticket pane's 이 현장에 연결 action remains for manual retry. |
| F3 | medium | Work-order drill href was `/dispatch?source=field&wo=…` — `DispatchPage` parses neither param, so the drill landed on an unfiltered board (dead deep-link). The canonical producer (`WorkOrderList`) uses `around_work_order_id`. | Href is now `/dispatch?around_work_order_id=<id>`; test asserts the exact href. |
| F4 | medium (a11y/design) | `.field__primary` paired `color: var(--accent-tx)` with `background: var(--signal)`; in dark theme that is #fcd34d on #f6b521 ≈ **1.3:1 contrast** (unreadable). The design authority pairs signal-background buttons with fixed `#141a21` in both themes (dc.html lines 645/723/1180…). | `color: #141a21` (design-fixed ink; light-theme `--ink` is the same value). Dead `#fff` fallback removed. |
| F5 | low (evidence) | Acceptance screen test asserted only POST method, not the business payload. | Test now asserts the wire body `{kind, channel, accepted_by}` matches the form's answers verbatim. |

## Module completion contract — point by point

1. **List/overview**: site table (현장·고객·이슈·작업·SLA) from `GET /api/v1/field/sites`, drillable 1-row stat bar (SLA 위반 → BREACHED filter, 진행 이슈 → open-ticket drill, 현장 → clear) derived from the same rows query. PASS.
2. **Object detail**: pinned site pane — SLA chips, kv (고객/주소/담당/지오펜스/다음 기한/SLA 90일), four history sections. PASS.
3. **Action/workflow**: 이슈 접수 intake → site link → assign-self/FSM transitions (`allowedTransitions`) → comments → RESOLVED-gated customer acceptance (`POST …/acceptance`). PASS.
4. **History layers**: tickets, work-order refs (with dispatch deep-links), attendance ARRIVAL/DEPARTURE, acceptance records, ticket comments. PASS.
5. **≥2 upstream + ≥2 downstream traversals**: upstream — customer drill-filter chip, contact-person search chip; downstream — ticket open, WO dispatch deep-link (plus attendance/acceptance records). PASS.
6. **Deny-by-omission authz**: denied capability renders the denied status without fetching (test: fetch never called); 404/403 site detail renders absence, not error; viewer capabilities hide (not disable) intake/triage/acceptance/comment affordances; `request_only` denies. PASS.
7. **Keyboard/focus/contrast/Korean/responsive**: J/K/Arrow/Enter grid nav (ModuleScreen-exemplar pattern), `:focus-visible` outlines on all interactives, signal-button contrast fixed (F4), all strings via `i18n/field.ts` + `ko.support` reuse, narrow widths collapse the two wide columns (no horizontal scroll). PASS.
8. **Selection + drafts survive remount**: session+branch-keyed localStorage (`mnt.field.<session>.<branch>`); test remounts and re-hydrates selection + intake draft. PASS.
9. **Truthful states**: loading, empty vs filtered-empty (+필터 해제), denied, absent, error+retry on list/detail/ticket; mutations reconcile every open pane from the server (F1). PASS.

## Design fidelity vs `MOD_SCREENS.field`

Matches: title 고객·현장; action 이슈 접수; stat bar shape (3 compact drill
stats, no KPI cards); 4-col list with SLA-tone chips; per-site detail with
enum/kv/link-chips; customer (거래처) traversal; WO-2638-class work-order
links; status=chips grammar throughout.

Deviations, all contract-driven and rendered truthfully (no fabrication):

- 계약 column → 이슈·작업 counts (no contract linkage field in `FieldSiteRow`).
- 상주 현장 stat → total-sites stat (no resident-headcount field).
- 서비스 enum chips (경비/지게차 임대/설비 유지보수/미화) → SLA enum filter chips (no service field).
- Site code chips (ST-01…) absent — no code field in the DTO.
- Design link set per row includes 지도에서 보기 (map screen not mounted), 그래프 현장 체인 (object-graph screen not mounted), 근태 현황 → att screen (not mounted; attendance history is rendered inline instead), C-207/CS-118/IN-0620 object codes and 메일 스레드 (no such linkage fields in the contract DTOs).
- fs2's 연장 계약 기안 prefill act omitted — no approval-prefill API (absent, not a dead control).
- Intake is an inline form rather than a hop to the `ingest` screen (ingest not mounted; the inline form posts the same `CreateInternalTicketRequest`).

## API/contract fidelity

- Field routes go through the authenticated `ConsoleApiClient` (path templating,
  auth, refresh identical to typed calls); local DTOs are the written backend
  contract, to collapse onto generated `components["schemas"]` aliases on regen.
- Canonical error envelope (`error.message`) surfaced on every route;
  `FieldApiError.status` drives 404/403-as-absence.
- No N+1: one list call, one aggregate detail call, three parallel reloads on
  reconcile; query params only present when set (asserted).

## Gates re-run after fixes

- `vitest run src/console/field` — 4 files, **28/28 passed** (was 26; +2
  adversarial evidence tests, both red before the fixes).
- `eslint src/console/field src/i18n/field.ts --max-warnings 0` — clean.
- `tsc -b` (whole web project) — clean.
- `node scripts/check-console-purity.mjs` — clean (407 files).
- `node scripts/check-ui-strings.mjs` — module files clean; repo-wide red only
  on pre-existing `src/features/facilities/FacilitiesWorkflowPage.tsx`
  (another lane's file, outside this lane's ownership).
- Zero TODO/FIXME/`.skip`/`.only` in module files; no dead controls found
  (every rendered affordance posts to a real route or is capability-hidden).

## Open items (honest)

- Backend contract routes (`GET /api/v1/field/sites[/{id}]`, ticket
  `link`/`acceptance`) are the parallel backend lane's; `fieldApi.ts` DTOs are
  the sync point — regen collapse is an integrator step.
- Keyset pagination: first page only (limit=100, contract max); `next_cursor`
  plumbed but no load-more UI.
- Check-in write path (device-attested) and JL- work-log rows remain
  workorder-lane gaps (W3/W4); this module renders attendance/WO refs read-only.
- Map/graph/att/ingest drill targets stay absent until those screens mount.
- Integrator: apply `manifests/mount.json` (add `field` to
  `MOUNTED_SCREEN_KEYS` + `SCREEN_REGISTRY`); nav entry already exists;
  `EXPOSED_SCREEN_KEYS` unchanged (ADR-0025).
- `scripts/check-ui-strings.mjs` stays repo-red until the facilities lane or
  the integrator fixes `FacilitiesWorkflowPage.tsx`.
