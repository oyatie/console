# CAP-MAINTENANCE-CONSOLE frontend — Stage 3 fresh-eyes adversarial verification

Date: 2026-07-24. Verifier: independent session (did not write the code). Scope: `web/src/console/maintenance/**`, `web/src/i18n/maintenance.ts`, evidence manifests. All claims below re-derived from the code, the generated client (`clients/ts/src/schema.d.ts`), `backend/openapi/openapi.yaml`, and the design mirror (`docs/design/oyatie-console/Oyatie Console.dc.html`, change-log 190).

## Verdict

Module verified against the completion contract and the design section after fixing four findings in this pass (F1–F4 below). Final state: 26/26 module tests, `tsc -b` clean, scoped eslint clean, check-console-purity clean, check-ui-strings clean for owned files (repo-wide red is pre-existing `src/features/facilities/FacilitiesWorkflowPage.tsx`, commit fd93fbdd — not this lane).

## Findings fixed in this pass

- **F1 (confirmed defect, correctness/state-bleed): detail composer state bled across selected objects.** `DetailPanel` held uncontrolled inputs (review memo, report diagnosis/action, mechanic id, priority select, settlement lines) and was not keyed by work-order id. With a warm client read-cache (ConsoleApiClient caches GETs 30s), `ready(A) → ready(B)` batches without a committed loading interstitial, so React reused the component instance and a reject memo typed for order A survived into order B — provable: the new regression test fails without the fix ("memo for A" received). Fix: `key={detail.value.id}` on `DetailPanel`. Test: "does not carry a review memo typed for one order into the next selected order".
- **F2 (confirmed defect, authz robustness): a settlement-only 403 collapsed the whole detail into denied.** `loadDetail` fetched the settlement sub-read for every reader whenever status ∈ {REPORT_SUBMITTED, ADMIN_REVIEW, FINAL_COMPLETED}; a 403 from that G3 route fell into the shared catch and rendered the entire, authorized work-order detail as "권한이 없습니다" (false denial). Fix: settlement 403 → detail stays ready and the settlement zone is omitted entirely (deny-by-omission — no untruthful "정산이 아직 작성되지 않았습니다", no disabled ghost). Non-403 settlement errors still surface as retryable detail errors. Test: "keeps an authorized detail readable when only the settlement read is denied".
- **F3 (confirmed defect, truthful drafts): stale draft resurrection.** The composer draft was read through `useMemo([draftKey])`, so after a successful create cleared sessionStorage, re-opening the composer repopulated the already-cleared draft from the cached memo. Fix: read sessionStorage at render time when the composer is open (`readDraft()`); refresh-survival behavior unchanged (existing test still green).
- **F4 (UI grammar, self-explanatory labels): review textarea mislabeled.** The single review field feeding both approve (comment) and reject (memo) was labeled "승인 의견"; for the reject path that label was wrong. Fix: neutral "검토 의견" (`actions.reviewComment`), matching the settlement review label; fail-closed per-decision error strings unchanged. Removed now-unused i18n keys (`actions.assignConfirm`, `actions.approveComment` (superseded), `actions.rejectMemo`, `settlement.returnComment`) and synced the i18n manifest. `noBranch` is retained deliberately: it is the mount-contract string for the integrator's registry adapter no-branch state (mount.json component.note).

## Module completion contract (docs/program/console-enterprise-roadmap.md), point by point

1. **List/overview layer** — header stat bar (전체/긴급/지연/미배정 from `WorkOrderLensAggregates`; 예방정비 준수/MTTR render only when the backend sends the optional G4 fields), lens facet chips (status/priority with counts and server-provided `filters`), three triage lanes, shared-track sortable list (오더/작업/현장/담당 + status chip). Every stat and facet is a drill (applies a real list query); nothing is a dead number. VERIFIED.
2. **Object detail layer** — request_no heading, status/priority/result/KPI-excluded/evidence-verified chips, five-step flow stepper projected from the 16-status FSM (terminal statuses render as a chip, 전표 step completes only on settlement APPROVED), kv facts, contact, delay reason. VERIFIED.
3. **Action/workflow layer** — create (draft-persisted composer), assign (fail-closed: mechanic required), start, report submit (result/diagnosis/action required), approve/reject (fail-closed comment/memo), priority change, settlement draft→submit→approve/return/void (four-eyes stays server-side), evidence presign→PUT→confirm. Each action posts to the real route and reconciles by re-reading list + detail. VERIFIED.
4. **History layer** — status_history timeline (from→to chips + action), approval line (role/status/actor/time/comment), WORM evidence timeline (stage + PENDING/VERIFIED/FAILED chips). VERIFIED.
5. **≥2 upstream + ≥2 downstream traversable links** — upstream: equipment/자산 이력 (`equipment_id`), customer (`customer_id`), site (`site_id`); downstream: assignee orders (`assigned_to`), related orders (`around_work_order_id`), settlement voucher refs, evidence entries. All are clickable chips driving real list queries. VERIFIED (in-module filtered views; cross-module object pages need the shell router — open item).
6. **Server-enforced deny-by-omission without leakage** — read gated solely on `work_order_read_all` (capability projection test proves `org_wide_queue_triage` does not grant read); no fetch occurs when denied; every action affordance is absent (not disabled) without its feature; rows/details from another branch are dropped/`missing`; detail 403 renders denied (no retry), 404 renders missing, distinct from errors; settlement-only 403 no longer collapses the detail (F2). VERIFIED.
7. **Keyboard/focus/contrast/Korean-expansion/responsive** — j/k row navigation + Enter open (tested); all controls are buttons/labeled inputs; `:focus-visible` 3px token outline; token colors only (single `color: white` on `--teal` action matches the production exemplar seam byte-for-byte); flex-wrap headers/chips tolerate Korean expansion; 1200px detail stack + 900px lane/column collapse asserted from the stylesheet in tests. VERIFIED.
8. **Selection + drafts survive refresh/retry/Back** — selection and composer draft in sessionStorage keyed by branch; remount-survival test green; F3 fixed the stale-resurrection edge. Session/API/capability changes remount via the fence key (fence test green). VERIFIED.
9. **No fake data / truthful states** — every rendered datum originates from the mocked-at-transport backend response; loading/empty/denied/missing/error/offline-retry states distinct and tested; zero TODO/FIXME/stub/test.skip/.only/placeholder (grep clean); no dead controls (the drag-source rows carry `text/plain` payloads for cross-surface drops — the only affordance without an in-module consumer; noted, kept as the design's shared-track behavior). VERIFIED.

## Design fidelity vs dc.html `MODS2.maintenance` (re-extracted this pass)

Matches: title 정비; header action "정비 요청 기안"; cols 오더/작업/현장/담당; three lanes with danger/warn/ok tones and the design's lane semantics (미배정 SLA-risk / 미배정 planned / 배정·진행); five flow steps byte-equal (접수 / 계획 · 부품 예약 / 실행 / 정산 / 전표); 유형/원인 entity chips with the design's vocabulary (긴급 출동, 예방 정비, 고장, 반납 준비, 정기); kv 접수/장비 facts; asset/person drill chips; status = chips everywhere; compact stat bar, no KPI cards.

Deliberate deviations (all truthfulness-driven, previously chartered):
- Lane 1 label "목표 임박 · 미배정" instead of the design's "SLA 임박 · 미배정": no SLO/SLA threshold objects exist yet (G6, governance lane); the lane is derived from P1/target-due — labeling it SLA would fabricate a concept the backend does not hold.
- Stats "이번 주" replaced by backend-derivable 지연/미배정; "예방정비 준수" renders only when the G4 aggregate arrives (no fabricated 92%).
- Flow-step PO/VC code chips (PO-121, VC-2604) and 부품 kv = G5 parts-reservation links (inventory lane); settlement voucher refs already render when present.
- Cross-module drill targets (자산, 부품 재고, JL 일지) resolve to in-module filtered queues until sibling modules mount on the shell router.

## API contract fidelity (field level, prior rejection classes)

- Paths match openapi.yaml exactly, including the mixed `/api` vs `/api/v1` prefixes (list/detail/reject/evidence = v1; create/assign/start/report/approve/priority = legacy prefix).
- Repeated-query parsing: `status`/`priority` are `style: form, explode: true` arrays; the module passes arrays to openapi-fetch and the transport test asserts repeated `status=…&priority=…` keys on the wire via the real `createConsoleApiClient`.
- Error envelope: canonical `{error:{message}}` parsed; non-2xx throws typed `MaintenanceApiError(status)`; settlement 404 → absent (tested).
- No N+1: one list call per query; one detail + at most one settlement call per selection; mutations reconcile with exactly one list + one detail re-read.
- Request/response fields verified one-by-one against `clients/ts/src/schema.d.ts` (WorkOrderListItem/Detail/Lens/facet-filters, CreateWorkOrderRequest, AssignWorkOrderRequest incl. PRIMARY role, SubmitReportRequest, ApproveWorkOrderRequest{comment}, RejectWorkOrderRequest{memo}, EvidencePresign tuple headers `[name, value][]`). G1/G2/G3/G4 remain module-local optional declarations per the manifests — regenerated client must supersede them at consolidation.

## Gates (final run, after fixes)

- `npx vitest run src/console/maintenance` — 4 files, 26/26 (24 original + 2 regression tests from F1/F2).
- `npx tsc -b` — clean. `npx eslint src/console/maintenance src/i18n/maintenance.ts --max-warnings 0` — clean.
- `node scripts/check-console-purity.mjs` — exit 0. `node scripts/check-ui-strings.mjs` — exit 1 repo-wide with zero maintenance findings; sole offender is pre-existing `src/features/facilities/FacilitiesWorkflowPage.tsx` (fd93fbdd, other lane).
