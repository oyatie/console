# CAP-DISPATCH-CONSOLE frontend — Stage-3 adversarial verification

Date: 2026-07-24 · Verifier: fresh-eyes lane (did not author the code) ·
Scope: `web/src/console/dispatch/**`, `web/src/i18n/dispatch.ts`, module evidence.

## Verdict

Two real defects found, proven red-first, fixed, re-verified green
(`332523d2`, `fea5aa5f`). Module meets the completion contract within the
recorded cross-lane gaps.

## Defects found and fixed

1. **Busy fence stuck after every successful action** (`DispatchScreen.tsx`
   `mutate`). `mutate` awaited `load()`, which bumps the shared generation
   token, so `finally { if (isCurrent(token)) setBusy(false) }` never fired on
   the success path — 배차 요청/배차 확정/더 불러오기 and all candidate radios
   stayed disabled until remount. Proven red with an `aria-busy` probe test,
   fixed by clearing the fence after the reload while the mutation still owns
   it (mutations are unreachable while busy, so no newer owner can exist).
2. **배차 확정 offered in an illegal FSM state.** The pick panel opened for any
   dispatch with `status !== "AUTO_ASSIGNED"`, but
   `backend/crates/dispatch/domain/src/lib.rs::force_assign` only accepts
   `MANAGER_FORCE_PENDING` — during `BROADCASTING` the offered confirm could
   never succeed (server 409): a dead control. Proven red, panel now gated on
   `MANAGER_FORCE_PENDING` exactly.
3. **Dead code removed**: `canRespond` capability (work_order_start has no
   console affordance — respond is the technician mobile surface),
   `dispatchApi.dispatch()` and the `P1DispatchSummary`/`UserSummary` aliases
   (zero callers).

## Module completion contract

- **Layers**: queue list/overview + selected-order detail panel + actions
  (배차 요청 broadcast, 배차 확정 force-assign) + history (platform audit,
  `target_type=p1_dispatch`) — all render only from authorized backend reads;
  loading/error/empty/denied states are explicit and truthful.
- **Traversable links**: upstream 작업 지시 (work_order) + 장비 (equipment);
  downstream 담당 기사 (person, when assigned) + the dispatch's own
  호출 현황/응답 이력/처리 이력 layers. 고객 and PO-/IV-/JL-/VC- links remain
  the recorded cross-lane gaps in `manifests/mount.json` — omitted, never
  fabricated or dead.
- **Authz**: deriveDispatchCapabilities maps exactly the features the backend
  enforces (verified in `dispatch/rest`: start→`WorkOrderCreate`,
  force-assign→`AssigneeManage`); denied read renders without fetching
  (test-asserted); object peek renders 403/404 identically as absent (no
  leakage); JWT-floor fail-closed until `/api/v1/me/authz` loads.
- **Keyboard/focus/a11y/responsive**: j/k + arrows row navigation, native
  radiogroup arrows, Escape-closing `role="dialog"` peek with autofocused
  close, `:focus-visible` outlines, token colors only, flex-wrap +
  `overflow-wrap` for Hangul expansion, ≤1000px single-column (test-asserted).
- **Persistence**: selection and candidate pick survive retry/reload within
  the session; pick is keyed to its dispatch (fail-closed across selection
  changes, test-asserted). No cross-refresh persistence — matches the
  production exemplar convention.

## API contract fidelity (field-level, this worktree)

- Typed routes verified present in `clients/ts/src/schema.d.ts`:
  `POST .../p1-dispatch` (`include_region` optional, default false — sent
  explicitly), `POST .../force-assign` (`mechanic_id` required),
  `GET /api/v1/users`, `GET /api/objects/{kind}/{id}`.
- `/api/audit`: backend `AuditQuery` accepts `target_id` (openapi.yaml omits
  it — pre-existing spec drift, backend-owned; isolation is server-side and
  test-covered in `backend/app/tests/audit_api.rs`). `occurred_at` is RFC3339
  (`time` `serde-well-known`); malformed records are dropped, never invented.
- Object kinds `work_order`/`equipment`/`person` all registered in
  `backend/app/src/objects.rs` RESOLVABLE_KINDS (person = MembershipOnly).
- The three untyped routes (queue/candidates/responses-read) are the backend
  lane's G1–G3 additions, absent from this worktree by design; error envelope
  parsing and shape-mismatch rejection are test-covered.

## UI grammar and gates

No captions/subtitles/meta text; status = chips (`StatusChip` tones incl.
purple for 외주); stats are compact chip-buttons, not KPI cards; plain string
literal classNames; zero inline Hangul in components; zero
TODO/FIXME/skip/only. Flow chips 접수→계획·부품 예약→실행→정산→전표 match the
design's maintenance order-cycle stepper (mirror AGENTS.md 64/77/78).
Design deltas remain as recorded in `manifests/mount.json` (배차 요청 not
기안; 가용 기사 stat live-only; site/customer names absent from the queue DTO).

## Runs (after fixes)

- `vitest run src/console/dispatch` — 4 files, **25/25 green**
- `tsc -b` — clean
- `eslint src/console/dispatch src/i18n/dispatch.ts --max-warnings 0` — clean
- `check-console-purity.mjs` — OK (407 files)
- `check-ui-strings.mjs` — zero dispatch violations (pre-existing
  facilities-lane failure unchanged, outside this lane)
