# Wave-4 charter — lens C: beyond-prototype enterprise UX depth

Worktree: `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`
Branch: `wave23-consolidation-20260724`. All lanes stack onto PR #488.
Author pass: 2026-07-25. Every repo claim below carries `file:line` or a command I ran.

---

## 0. The thesis, and the one rule that generates every lane

The fidelity registers show the same seven violations repeating across 15 modules
(§4.7-1 list grammar, §4-27-4 scale framing, §4-10 empty states, §4.7-2 window
model, §4-20 drag sources, §4.7-9 dead numbers, §4-22 add-anything). They repeat
because **there is no shared mechanism** — there are 15 hand-rolled lists.
Measured (`grep` over `web/src/console/<module>`): 5 modules render `<table>`,
1 renders `role="grid"`, 9 render bespoke divs, and `components/ui/data-table.tsx`
(the one shared table that exists) has exactly **one** consumer,
`pages/EquipmentBrowsePage.tsx`.

So the lens-C rule is: **each lane builds ONE shared primitive under a new,
uncontended root `web/src/console/grammar/**`, proves it on 1–2 pilots, and hands
adoption to the lens-B module fan-out as a mechanical port.** No lens-C lane owns
a module body. Thirteen bespoke implementations is the failure this lens exists to
prevent, and chartering 13 adoption lanes here would recreate it.

Ponytail: `web/src/console/grammar/` does not exist today (`ls web/src/console/`
verified) — free root, zero collision, no integrator negotiation.

## 1. Stack facts that override the brief's assumptions

| Assumption | Reality (verified) |
|---|---|
| React 18 | **19.2.7** (`web/package.json:28`) — and `useOptimistic`/`useActionState`/`useTransition` appear **0 times** in 522 `.tsx` files |
| ko line-breaking needs fixing | **already done** — `word-break: keep-all; overflow-wrap: anywhere` at `web/src/styles.css:159-160`, `--lh-base: 1.5` at `console/tokens.css:68`. L-C10 shrinks accordingly |
| Need a virtualization dep | server pagination caps DOM rows; virtualization is only needed for genuinely unbounded scroll (evidence register, messenger history). Deferred to L-C11 behind a measurement gate |
| No a11y test infra | `playwright.config.ts` + `e2e/specs/{chrome-02-axe,chrome-03-workspace,admin-29-console-window,chrome-01-mobile-drawer}.spec.ts` + `e2e/fixtures/ux.ts` (AxeBuilder, wcag2a/2aa/21a/21aa, critical+serious block) already exist under the `dev-auth` project |
| Need a new cursor scheme | **no** — `backend/crates/docs` already landed an opaque base64url keyset cursor with snapshot stability (`EvidenceObjectCursor`, adapter `:798-807`, REST `:193-205`), test-covered, and the consolidation inventory records it as the winner. Generalize it; do not re-derive |

**The backend pagination truth** (my parse of `backend/openapi/openapi.yaml`, 434 paths,
205 GET ops, 34 list-envelope responses):

| Capability | Count |
|---|---|
| `limit` param | 20 |
| `offset` param | 10 |
| `cursor` param | 3 |
| `total` in envelope | 13 |
| `has_more` / `next_cursor` | 1 / 3 |
| **server `sort`/`order` param** | **0** |

- **14 list endpoints have no pagination parameter at all** — incl. `/api/v1/workflow-tasks`,
  `/api/v1/workflow-studio/definitions`, `/api/v1/leave/balances`, `/api/v1/equipment-by-location`,
  `/api/v1/me/dispatch-offers`.
- **7 have `limit` but no `offset` or `cursor`** — literally no page 2:
  `/api/v1/equipment`, `/api/messenger/{threads,channels,search}`, `/api/v1/leave/requests`,
  `/api/v1/me/todos`, `/api/v1/period-locks`.
- The named 1,284-row roster is `/api/v1/employees` (`openapi.yaml:1675`): offset paging,
  `limit` **default 500 / max 1000**, `EmployeePage`, **no sort param**.
  `/api/v1/directory/people` (`:6423`) is `limit` max 200, fixed `display_name, id` order.

Consequence, and it is a truthfulness bug not a perf bug: **column-header sort cannot be
implemented honestly on any list today.** A client sort of a truncated page is a lie, and
the field register already records the same class of lie —
*"스탯이 limit:100으로 잘린 클라이언트 페이지에서 파생 … 100현장 초과 조직에서 스탯 바가 허위 수치"*.
This is why **L-C1 is the foundation lane** and why L-C2's sort ships dark until L-C1 lands.

**The transport that exists today** is one multiplexed WebSocket, `GET /api/v1/ws`
(`openapi.yaml:60`), driven by `web/src/features/comms/realtimeHub.ts` — ref-counted,
resume cursor `last_message_id`, backoff capped at 30 s. Its event union is exactly two
variants: `message_posted`, `notification_created` (`realtimeHub.ts:7-9`). Presence has
**no realtime path** — only REST `/api/messenger/threads/{threadId}/presence`. Any lane
promising presence/live status outside messenger is promising a backend that does not exist.
Reconnect is driven purely by the `close` event (`:79`), so a half-open socket leaves the
console silently stale while looking healthy.

## 2. Dependency justification

**One new dependency is contemplated, in one lane, behind a gate:** `@tanstack/react-virtual`
(headless, no markup/styles) in L-C11, and only after L-C11's measurement step proves a
surface that server pagination cannot bound. Everything else is React 19 built-ins, `Intl`,
CSS container queries, `sessionStorage`, and generalizing hooks already in the repo. No
TanStack Query, no SWR (React 19 `useOptimistic`/`useActionState` covers §2 of the brief),
no table library, no i18n runtime (`ko.ts` is 8,715 lines but lint-gated and zero-dep;
a second locale is not chartered).

## 3. Collision discipline for every lane in this lens

Serialized single-owner surfaces — **emit a manifest, never edit**:
`web/src/i18n/ko.ts` (28 hits/48 h), `web/src/console/shell/nav.ts`,
`web/src/console/screens/registry.ts`, `backend/openapi/openapi.yaml` (61 hits; a whole-file
revert scar at `9bb877c6` proves mechanical splicing fails), `clients/**`,
`backend/app/src/lib.rs`, `backend/app/src/objects.rs`, `backend/crates/platform/db/migrations/**`.
Manifest path convention already in use: `docs/evidence/console/CAP-*/frontend/manifests/mount.json`
+ an i18n key inventory.

Migration numbering: high-water `0202`, **`0201` is a reserved gap** (docs retention).
Only L-C1 needs a migration → provisional **0203**, emitted as a manifest; the integrator
renumbers at merge and the lane re-checks immediately before push.

Codex fleet: hot-check (`git log --since=48.hours --name-only`) before touching any backend
crate; plain-merge before push (rebase is classifier-blocked).

## 4. Lanes, ranked by value

Foundation lanes **F** must complete before the lens-B module fan-out starts, because each
one fixes a defect that would otherwise be reproduced 13 times in review.

| # | Lane | F | Size | Shared primitive built | Consumed by |
|---|---|---|---|---|---|
| 1 | L-C1 Page contract (BE) | ✅ | L | `backend/crates/listing` keyset+sort+total | every list endpoint; L-C2/6/9 |
| 2 | L-C2 Console list grammar | ✅ | L | `grammar/list` `ConsoleList`/`ConsoleGrid` | all 15 module bodies, both module engines |
| 3 | L-C5 Window + object-card a11y | ✅ | M | hardened `console/window` + `objectcard` | every drill/pin surface |
| 4 | L-C3 Conflict + mutation contract | ✅ | L | `grammar/mutation` `useConsoleMutation`, `ConflictPanel` | every write surface |
| 5 | L-C8 Truthful state primitive | ✅ | S | `grammar/state` `ConsoleState` + lint gate | every screen |
| 6 | L-C4 Realtime hardening | | M | hardened `realtimeHub` + `ConnectionStatus` | comms rail, notif, any live number |
| 7 | L-C6 Bulk operations | | M | `grammar/bulk` | list surfaces with actions |
| 8 | L-C7 Draft contract | | M | `grammar/draft` `useDebouncedServerState` | every form |
| 9 | L-C9 Responsive + density truth | | M | `grammar/responsive` + viewport sweep spec | all screens |
| 10 | L-C10 Locale + number/time honesty | | S | `lib/currency`, `lib/datetime` tick, expansion probe | all screens |
| 11 | L-C11 Virtualization (gated) | | S | `grammar/virtual` | evidence register, messenger history |

Run order: L-C1 ∥ L-C2 ∥ L-C5 ∥ L-C8 first (L-C2 builds against L-C1's TS contract type and
wires its pilot after). Then L-C3 ∥ L-C4 ∥ L-C7 ∥ L-C10. Then L-C6 ∥ L-C9 ∥ L-C11.

---

### L-C1 — Server page contract: keyset cursor + server sort + honest totals

**Why.** Zero list endpoints accept a sort param; 14 have no pagination at all; 7 have
`limit` with no way to reach page 2; the roster defaults to `limit=500`. Every §4.7-1
sort requirement, every §4-27-4 「N 표시 / 전체」 requirement, and the field register's
*"허위 수치"* stat bar are all downstream of this. Closes design-intent **C-47** (scale
honesty: *"server-side indexed search + pagination endpoints per selectable type are a named
backend contract"*) and unblocks **C-39** (*"keyboard nav over server-paginated lists"*).

**Scope.**
1. New crate `backend/crates/listing` extracting the docs-lane precedent verbatim in shape:
   opaque base64url-JSON keyset cursor `{snapshot_sequence, keyset_tuple, id}`, `ORDER BY
   (sort_key, id) DESC/ASC` predicate builder, snapshot stability via
   `sequence <= snapshot_sequence`, and the two hard validations the docs lane already
   proved (`cursor cannot be combined with offset`, `cursor snapshot does not match as_of`).
   Do not re-derive the cursor — read `backend/crates/docs/adapter-postgres/src/lib.rs:100-142,798-807`
   and `backend/crates/docs/rest/src/lib.rs:193-205` first.
2. A whitelisted-sort-key contract: sort keys are a per-endpoint enum, never a free string
   (SQL-injection surface and an unindexable surface). Unknown key → canonical 400.
3. `ListPage<T> { items, total: Option<i64>, next_cursor: Option<String>, has_more }`.
   `total` is `None` where it cannot be computed cheaply — the frontend renders
   `aria-rowcount="-1"` for that case. A fabricated total is a defect.
4. Pilot retrofit of **exactly one** endpoint end-to-end: `/api/v1/employees` (the named
   1,284-row roster). Hot-check its owning crate first; if it is under active codex churn,
   fall back to `/api/v1/directory/people` and record the swap.
5. Covering index for the pilot's keyset tuple → migration manifest `0203_list_keyset_indexes.sql`.
6. `docs/specs/console-page-contract.md`: the contract + a per-endpoint adoption checklist,
   so lens-A/B lanes retrofit their own endpoints without re-deciding anything.

**Roots (owned).** `backend/crates/listing/**`, `docs/specs/console-page-contract.md`,
`docs/evidence/console/CAP-CONSOLE-PAGE/**`, plus the single pilot endpoint's
`{domain,application,adapter-postgres,rest}` files named in the lane's opening hot-check.

**Must not touch.** `backend/crates/docs/**` (the precedent is read-only to this lane),
`openapi.yaml`, `clients/**`, `migrations/**`, `backend/app/src/lib.rs` — all manifest-only.

**DoD.**
- `ListPage`/cursor unit tests incl. round-trip, tampered-cursor rejection, offset+cursor rejection,
  unknown-sort-key → canonical 400 envelope.
- Pilot integration test proving: page 2 via cursor returns no duplicate and no skipped row
  **under a concurrent insert** (this is what snapshot stability buys, and the docs lane's
  `48a89167 test(docs): prove snapshots reject backdated inserts` is the model).
- Sort applied server-side, asserted by row order across two pages.
- RLS: pilot list executed **as `console_rt`** with `app.current_org` armed, plus a cross-tenant
  negative asserting zero rows (superuser `#[sqlx::test]` BYPASSRLS masks this — project memory).
- Buck2 targets for the new crate build+test green (`buck2 test //backend/crates/listing/...`);
  cargo run via a spawned subagent as a pre-push check.
- `npm run check:openapi-app && npm run check:api-drift:portable && npm run check:api-drift:swift`
  green **after** the integrator applies the openapi manifest (lane records the manifest + the
  regenerated-client diff it expects).
- Audit: list reads on the pilot are audited per the sensitive-view rule where applicable; no
  new mutation, so no new audit surface otherwise.
- Evidence in `docs/evidence/console/CAP-CONSOLE-PAGE/`: the 34-endpoint coverage table with
  each row marked adopted / to-adopt / no-pagination-needed.

**Migration slot.** provisional `0203` (manifest; integrator renumbers).
**Risk.** Retrofitting more than the one pilot; the lane must resist. The 33 other endpoints
are their owning lanes' work, guided by the spec.

---

### L-C2 — Console list grammar: one list primitive, ARIA-correct, server-paginated

**Why.** 15 bespoke lists; §4.7-1 violations in equipment, logistics, evaluation, board,
directory, dispatch, field, notif; `aria-rowcount|aria-rowindex|aria-colindex` appear
**0 times** in `web/src`. And there is a live shared defect: `console/module/ModuleScreen.tsx`
puts `role="columnheader"` spans in a sibling div **outside** the `role="grid"` (`:315` vs `:331`)
with no `role="row"`, makes rows `aria-selected` while never focusable (grid holds the only
`tabIndex`, J/K is undiscoverable to AT), and applies `role="grid"` + `role="gridcell"` to a
**kanban** (`:392`). Closes **C-39**, **C-40**, **C-47**; floor for every lens-B port.

**Scope.**
1. `grammar/list/ConsoleList` — semantic `<table>` by default (Roselli: read-only tabular data
   needs no `role="grid"`; a plain table gives Korean screen-reader users working table
   navigation for free). Sticky `<thead>`, shared-track column widths (per-row `max-content`
   is banned by §4.7-1), end padding + `overscroll-behavior: contain` + bottom fade preserved
   from the current grammar.
2. `grammar/list/ConsoleGrid` — `role="grid"` + roving tabindex + 2-D arrows, permitted **only**
   for cell-editable surfaces (payroll cells, attendance corrections, ontology property matrices).
   `role="grid"` on a read-only list is a documented review reject. Delete the kanban's grid roles
   (nested lists + real buttons instead).
3. Keyboard model beyond J/K/Enter: roving tabindex, `ArrowUp/Down`, `Home`/`End`,
   `Ctrl+Home`/`Ctrl+End`, `PageUp`/`PageDown`. `Ctrl+End` **fetches the tail page and then
   focuses** — the APG's named lazy-load failure, and the single behaviour most grids get wrong.
   `Escape` clears selection and returns focus to the list.
4. Truthful counts: `aria-rowcount={total}` and `aria-rowindex={offset + i + 1}` from L-C1's
   envelope; `aria-rowcount="-1"` when `total` is `None`. A visible 「N 표시 / 전체」 chip beside
   the list, and a `role="status"` announcement on page change.
5. Server sort only: header `<button>`, `aria-sort` on the sorted header **only** (removed from
   the previous one), `?sort=` to L-C1. Ship dark (header buttons disabled with the reason) on
   endpoints that have not yet adopted L-C1 — never a client sort of a truncated page.
6. Column resize keyboard path: the handle is focusable with an accessible Korean name,
   `ArrowLeft/Right` = ±8 px, `Home` = reset. Today it is `aria-hidden="true"` with a `title`
   (`ModuleScreen.tsx:317-323`) — unreachable.
7. Per-person column width/order/visibility persisted through the existing
   `GET/PUT /api/v1/me/workspace` envelope as a new top-level key (`features/workspace/persistence.ts`
   already carries unknown keys through). No new endpoint.
8. `columns[].priority` prop exposed for L-C9's density demotion; `selectionSlot` prop exposed
   for L-C6. Documented as the extension seams so those lanes never edit this lane's internals.
9. Multi-attribute search over visible attributes (§4.7-1), debounced, pushed to the server
   `search` param where one exists, and **disabled with a stated reason** where it does not.
10. Port the two shared engines onto it: `console/module/ModuleScreen.tsx` and
    `console/modules/GenericModuleScreen.tsx`. That alone fixes board, finance, asset, compliance.

**Roots (owned).** `web/src/console/grammar/list/**`, `web/src/console/grammar/index.ts`,
`web/src/console/module/**`, `web/src/console/modules/**`,
`docs/evidence/console/CAP-CONSOLE-LIST/**`.

**Must not touch.** The 15 module dirs under `web/src/console/<domain>/` and
`web/src/features/attendance/**` (lens-B ports), `ko.ts` / `nav.ts` / `registry.ts` (manifests),
`grammar/{bulk,responsive,virtual}` (other lanes).

**DoD.**
- Vitest: roving-tabindex focus walk; `Ctrl+End` triggers a tail-page fetch then focuses the true
  last row (mock asserts the fetch happened before focus); `aria-rowindex` equals `offset+i+1`
  across two pages; `aria-rowcount` is `-1` when `total` is absent; `aria-sort` present on exactly
  one header; sort issues a server request and never reorders client-side.
- Vitest: `ConsoleList` renders **no** `role="grid"`; kanban renders no `role="row"`/`gridcell`.
- `npm --prefix web run lint && npm --prefix web run test && npm --prefix web run build`.
- e2e: extend `e2e/specs/chrome-02-axe.spec.ts` with the ported board screen — zero
  critical/serious axe violations; add a keyboard journey spec asserting `document.activeElement`
  after each of Arrow/Home/End/Ctrl+End/Escape.
  `CONSOLE_DEV_AUTH_E2E=1 node scripts/dev-up.mjs bootstrap && CONSOLE_DEV_AUTH_E2E=1 npx playwright test --project=dev-auth e2e/specs/chrome-02-axe.spec.ts`
- Screen-reader flow written up (VoiceOver + NVDA table navigation over a 2-page list) in the
  evidence dir — axe cannot see any of this.
- No stubs: every prop either works or is absent. A disabled sort header states its reason.

**Depends on.** L-C1 (type contract; pilot wiring after L-C1 lands).
**Risk.** It owns two shared engines that four screens consume — a regression here is wide.
Mitigation: the existing `ModuleScreen.test.tsx`, `moduleEngine.test.tsx`,
`FinanceModuleScreen.test.tsx`, `AssetModuleScreen.test.tsx` must stay green unmodified except
for role assertions that were asserting the defect.

---

### L-C5 — Window system + object-card: keyboard operability and focus truth

**Why.** Two independent scouts flagged the same thing: the shared drag host
`console/objectcard/ObjectCard.tsx:779` is a `<span {...objDrag(...)}>` with no `tabIndex`, no
`onClick`, no AT path — and **every module that drills inherits it**, so the AA bar breaks 13
times in lens-B review instead of once here. `ObjectCardModal` (`ObjectCard.tsx:895-913`) is
`role="dialog" aria-modal="true"` with `Escape` bound to a non-focusable `<div>` — no focus
trap, no initial focus, so Escape only works by luck; it is the mandated no-provider fallback
in every port recipe. Six fidelity registers (payroll, attendance, evaluation, docs, field, org)
record §4.7-2/§4-23 window-model violations. Closes **C-38**, part of **C-37**.

**Scope.**
1. `ObjectCard.tsx:779` span → real `<button>` with a Korean `aria-label` and ≥44 px target,
   `objDrag` spread on top — the rule `configconsole/DashboardEditor.tsx:136` already states
   verbatim. Same sweep over `console/explore/ObjectExplorerScreen.tsx:358,464,526`.
2. `ObjectCardModal`: initial focus inside, contained tab sequence, `Escape` on the dialog,
   focus returned to the invoker (or its container if the invoker is gone, per APG).
3. Window semantics: pinned panels are `role="region"`/`complementary` with the object title as
   `aria-label` — **not** `aria-modal`, no focus trap, no focus steal on pin. Popouts (if/when
   the provider gains them) are `role="dialog" aria-modal="false"`, Escape-closable, **not** trapped.
4. Every window operation reachable by keyboard as menu items with shortcuts: pin,
   move-to-quadrant, minimize, restore, close. Drag-to-snap stays an accelerator, never the only path.
5. Focus never disappears: minimize → tray chip; close → invoking row/chip; restore → panel title.
6. **One** shell-level `role="status"` announcing state changes
   (`{title} 우측 상단에 고정됨` / `최소화됨` / `닫힘`) — one region, not one per panel.
7. `F6` pane cycling (section → panels → tray) and a 본문으로 이동 skip link.
8. Hydration sanity: clamp restored geometry to the viewport so a persisted layout can never
   restore a zero-size or off-screen panel that is unreachable by keyboard.
9. Decide and record: `saveLayout()`/`restoreDefault()` are a dead context API (only callers are
   `AppShell.test.tsx:138,199`) while `ko.console.window.saveLayout/restoreDefault` strings exist
   (`i18n/ko.ts:1801-1802`). Either wire a control or delete both — a documented control that does
   not exist is a §4-12 dead affordance. Ponytail: wire it, the strings and the API both already exist.

**Roots (owned).** `web/src/console/window/**`, `web/src/console/objectcard/**`,
`web/src/console/explore/ObjectExplorerScreen.tsx`, `e2e/specs/admin-29-console-window.spec.ts`,
`docs/evidence/console/CAP-CONSOLE-WINDOW-A11Y/**`.

**Must not touch.** `console/window/{WindowEngine,useWindowEngine,geometry,sanitize}` — the
4-state engine is harness-only (`AppRouter.tsx:63-69,460`); this lane hardens the **production**
3-state `WindowManagerProvider` only. Promoting the 4-state engine is a separate decision, not
this lane's.

**DoD.**
- Vitest under `<WindowManagerProvider>`: every drag host is a focusable `button` with
  `data-obj-code`; modal takes initial focus and traps tab; Escape returns focus to the invoker.
- Playwright keyboard journey extending `admin-29-console-window.spec.ts`:
  pin → popout/minimize → restore → close, asserting `document.activeElement` at **every** step,
  plus `F6` cycling. Axe-clean is necessary and explicitly **not** sufficient — the journey is the
  merge evidence.
  `CONSOLE_DEV_AUTH_E2E=1 npx playwright test --project=dev-auth e2e/specs/admin-29-console-window.spec.ts`
- `npm --prefix web run lint && npm --prefix web run test`.
- Evidence: a before/after AT transcript for the drill gesture.

**Risk.** Shared code every module drills into. Low churn (`console/window`, `console/objectcard`
are absent from the 48 h hot list), so the window is now.

---

### L-C3 — Conflict + mutation contract: 409/412 as merge affordances

**Why.** `409` appears **168×** in `openapi.yaml`; the full `If-Match`/`ETag`/412 machinery exists
but **only in Ontology** (`web/src/api/ontology.ts:151,230,243` is the sole frontend consumer;
`openapi.yaml:12070,15240,15252`). Every other write path can silently clobber. And React 19's
optimistic machinery is installed and **entirely unused** (0 hits for `useOptimistic`,
`useActionState`, `useTransition` across 522 components). The evaluation register records the
matching symptom: *"§4-2 기록 가시성: submit/calibrate/transition give no recorded-action feedback"*.

**Scope.**
1. `grammar/mutation/useConsoleMutation` over `api/client`: `useOptimistic` for the display value
   **plus a separate `draft` state** written before the Action and cleared only on success —
   React's documented revert does **not** preserve the user's input, so the caller must. On
   failure: draft restored, field errors mapped from `error.reasons[]`, focus to the first invalid
   field, retry re-armed.
2. Optimistic updates permitted only where reversal is cheap and visible — chips, toggles, row
   status, ordering. **Never** for money, approvals, or anything emitting an audit event: those
   render a pending state and wait. This is the same line lens D draws between computable and attested.
3. `grammar/mutation/ConflictPanel`: on 409/412 the editing surface enters a `conflict` state
   **in place** — affected fields show 내 값 / 현재 값 side by side with per-field 내 값 유지 /
   서버 값 적용, disjoint fields auto-merge with a merged marker, primary CTA becomes 다시 저장
   armed with the fresh validator. Pane stays open; nothing typed is discarded. Summary is
   `role="alert"`, focus moves to the first conflicting field. **A conflict is never a toast.**
4. `web/src/api/conflict.ts` + `web/src/api/idempotency.ts` (new files) threading `If-Match`,
   `ETag`, and a client-generated `Idempotency-Key` through the request helper — a retried
   transition whose first attempt actually succeeded resolves as **success**, not a conflict.
   `api/refresh.test.ts:122` already proves `If-Match` survives the 401 retry clone; keep that green.
5. Contract requirement recorded for the backend: a 409/412 body must carry stable `reasons[]`
   codes **and enough current server state to render the comparison without a second round trip**.
   A conflict the client can only answer by discarding is a bug in the contract. Emitted as an
   openapi manifest; the pilot is one non-ontology write path chosen after hot-check.

**Roots (owned).** `web/src/console/grammar/mutation/**`, `web/src/api/conflict.ts`,
`web/src/api/idempotency.ts`, `docs/specs/console-conflict-contract.md`,
`docs/evidence/console/CAP-CONSOLE-CONFLICT/**`. One surgical, named edit to the request helper
in `web/src/api/` for header threading.

**Must not touch.** `web/src/api/ontology.ts` (the precedent, read-only), module bodies,
`openapi.yaml`/`clients/**` (manifest), the `idempotency_keys` table — that is lens A's
(backend brief §5, Stripe-shape first-outcome replay); this lane **depends on** it and ships the
client half with a documented fallback.

**DoD.**
- Vitest: 412 → ConflictPanel renders both values, draft preserved verbatim, focus on the first
  conflicting field, retry sends the **new** ETag; disjoint-field edit auto-merges without prompting.
- Vitest: a retry carrying the same `Idempotency-Key` after a first-attempt success renders success,
  not conflict.
- Vitest: an audited/money mutation refuses optimistic application (asserted at the hook level).
- `npm --prefix web run test && npm --prefix web run lint`.
- e2e: two-session conflict on the pilot write path, asserting the merge surface and that no data
  is lost. Canonical error envelope respected throughout.
- Evidence: the conflict-response contract table (which codes, which current-state fields, per route).

**Depends on.** lens A's idempotency-keys table for full semantics; ships and tests without it.
**Risk.** Touching the shared request helper. Keep the edit to header threading; `refresh.test.ts`
is the regression fence.

---

### L-C8 — Truthful empty / denied / error states, with a lint gate

**Why.** §4-10 violations recorded in payroll (*"'표시할 급여 회차가 없습니다.' names no reason +
next action"*), docs, board, logistics; org additionally ships a banned tech-stack caption
(*"조직 변경 API가 아직 배포되지 않았습니다"*). Today there are two ad-hoc local `EmptyState`
functions (`pages/DispatchMapPage.tsx:274`, `console/inventory/InventoryScreenBody.tsx:158`) and
bare strings everywhere else. Closes **C-30**, guards **C-29**. Cheapest lane per module covered.

**Scope.**
1. `grammar/state/ConsoleState` with exactly five truthful variants, each requiring a
   **reason + next action in one line**: `empty` (no records yet → the creation CTA),
   `filtered-empty` (→ 「필터 해제 — 전체 목록」, the prototype's own affordance at dc.html 4770-4778),
   `denied` (deny-by-omission: states the missing grant, never the data), `blocked-until-backend`
   (names the missing capability + who owns it), `load-failed` (→ 다시 시도, preserving filters).
2. Every variant is a `role="status"` region except `load-failed`, which is `role="alert"`.
3. Lint gate: extend `web/scripts/check-ui-strings.mjs` to reject a rendered empty-state string
   that is not routed through `ConsoleState` — so the 13 lens-B ports cannot regress it. This is
   the mechanism that makes the primitive stick; without it the lane is advice.
4. Migrate the two existing local `EmptyState` functions and delete them.

**Roots (owned).** `web/src/console/grammar/state/**`, `web/scripts/check-ui-strings.mjs`,
`web/src/pages/DispatchMapPage.tsx` (delete local EmptyState only),
`web/src/console/inventory/InventoryScreenBody.tsx` (delete local EmptyState only — coordinate
with the lens-B inventory lane; this is a two-function deletion, not a body edit),
`docs/evidence/console/CAP-CONSOLE-STATE/**`.

**Must not touch.** Module bodies beyond the two named deletions; `ko.ts` (key manifest).

**DoD.**
- Vitest: each variant renders reason + action; `denied` leaks no record data; `load-failed`
  retry preserves the active filter set.
- The lint gate fails on a planted bare empty string and passes on the migrated call sites:
  `npm --prefix web run lint`.
- `npm --prefix web run test`.
- Evidence: the ko copy table for all five variants, reviewed against §4-12 (no captions, no
  subtexts, action-driving only).

---

### L-C4 — Realtime hardening: liveness, connection truth, view-scoped subscriptions

**Why.** Reconnect is `close`-driven only (`realtimeHub.ts:79`) — a half-open socket (idle NAT,
VPN drop) never fires `close`, so the console **silently stops updating while looking healthy**.
The attendance register records the matching lie: *"header claims 실시간 현황 but data is fetch-once
with no refresh path"*. Presence has no realtime transport at all (REST-only
`/api/messenger/threads/{threadId}/presence`), and BENCHMARK.md's own structural-gaps section
admits 실시간·멀티유저 is simulation-only.

**Scope.**
1. Liveness: server heartbeat every 30 s, client watchdog at 45 s → force `close()` → the existing
   backoff path. Keep the single multiplexed socket; do **not** add SSE (6-connection HTTP/1.1
   per-origin cap is a trap for a multi-tab console) and do not open a second socket.
2. `grammar/realtime/ConnectionStatus` in the shell, three states: `실시간` (silent) /
   `재연결 중` / `오프라인 — 마지막 갱신 HH:mm`, in a `role="status"` region. Mounted via manifest
   (shell is a collision root).
3. **Stale-marking**: every realtime-derived number (unread counts, presence dots, live totals)
   dims and shows its last-updated time when the socket is degraded. It must visibly go stale
   rather than lie. Never a spinner that implies live.
4. Degradation is a poll, not a blank: on-focus + low-rate interval refetch of the same endpoints,
   re-sync from the resume cursor on reconnect.
5. View-scoped subscription frame: `subscribe(params, listener, {topics})`, topics = objects/threads
   currently rendered, unsubscribed on unmount; presence requested for **visible avatars only**.
   Without this, presence is O(employees) per client and will not survive conglomerate scale
   (Slack's Flannel measured a 5× cut in presence events from exactly this move). The **server**
   topic-subscription frame and a presence event on `/api/v1/ws` are a **named backend dependency** —
   this lane ships the client frame + the REST-poll fallback and files the contract.
6. Unread stays server-authoritative against a read cursor the client advances explicitly; the
   client never derives unread by counting received events (one missed reconnect window and the
   badge is permanently wrong).
7. Announcement rate limit: messages into `role="log"`, unread badge `role="status"` batched to at
   most once per ~2 s. No realtime event ever produces `role="alert"` except a genuine interrupt
   (session fenced, 권한 revoked).

**Roots (owned).** `web/src/features/comms/realtimeHub.ts` + its tests,
`web/src/console/grammar/realtime/**`, `docs/specs/console-realtime-contract.md`,
`docs/evidence/console/CAP-CONSOLE-REALTIME/**`.

**Must not touch.** `console/shell/**` (mount via manifest), `console/messenger/**`,
`console/comms-rail/**` (consumers, ported by their own lanes), backend comms crate.

**DoD.**
- Vitest with a fake socket: no heartbeat for 45 s → forced close → reconnect with the resume
  cursor; heartbeat arriving keeps the connection alive; backoff still capped at 30 s and reset on open.
- Vitest: degraded socket → status region announces once, live numbers render dimmed with a
  last-updated timestamp, poll fallback fires.
- Vitest: `topics` unsubscribe on unmount (no listener leak across 100 mount/unmount cycles).
- `npm --prefix web run test && npm --prefix web run lint`.
- Evidence: the backend contract ask (heartbeat frame, topic subscribe/unsubscribe frames,
  presence event shape, read-cursor endpoint) as a filed manifest, plus an explicit statement of
  what is poll-backed until it lands. No UI claims 실시간 for a polled number.

---

### L-C6 — Bulk operations with per-item outcomes

**Why.** No module has bulk anything; §4-22 add-anything is recorded absent in dispatch, inventory,
maintenance, field, board, directory, equipment. Bulk-op incidents come from one specific ambiguity
— page-scoped vs query-scoped selection — so the primitive must make it impossible to confuse.

**Scope.**
1. Selection lives in `grammar/bulk`, injected into `ConsoleList` through the `selectionSlot` seam
   L-C2 exposes. A real `<input type="checkbox">` in the first cell (Higley: `aria-selected` on
   gridcells is use-at-your-own-risk; a checkbox is not).
2. Header select-all is **page-scoped by default**, with an explicit second affordance
   「필터에 해당하는 N건 전체 선택」 for query-scoped. Never ambiguous. Query-scoped selection carries
   the filter identity, not a row-id list, so it survives pagination; page-scoped selection is
   cleared on page change **with an announcement**, never silently.
3. `Shift+Click` extends by range, `Ctrl/Cmd+Click` toggles, `Ctrl+Space`/`Shift+Space` for the
   keyboard path. Selection count in a `role="status"` region.
4. Server-batched mutation returning **per-item outcome** `{id, ok | error{code, message}}`, a
   results panel listing failures, and a 실패 건만 재시도 action. A bulk op that reports only
   "23건 성공" is not shippable. Each item carries its own idempotency key so retry-failed-only is safe.
5. Bulk actions are PBAC-gated per item, deny-by-omission: an item the principal cannot act on is
   not selectable, not selected-then-rejected.

**Roots (owned).** `web/src/console/grammar/bulk/**`, `docs/specs/console-bulk-contract.md`,
`docs/evidence/console/CAP-CONSOLE-BULK/**`.

**Must not touch.** `grammar/list/**` internals (consume the documented seam only), module bodies.

**DoD.**
- Vitest: page-scoped select-all then page change → selection cleared **and** announced;
  query-scoped select-all survives the page change and reports the query-scoped count.
- Vitest: partial-failure batch renders each failure with its code and retries only those.
- Vitest: an unauthorized row is not selectable.
- `npm --prefix web run test && npm --prefix web run lint`.
- e2e: keyboard-only bulk selection + execute + retry-failed on the pilot list, axe-clean.
- Backend: batch endpoint audited per item; RLS verified as `console_rt`; idempotent retry proven.

**Depends on.** L-C2 (seam), L-C1 (query-scoped identity needs a stable server-side filter+sort).

---

### L-C7 — Draft autosave / restore

**Why.** No draft machinery exists outside the workspace layout. The correct engineering already
sits in `web/src/features/workspace/persistence.ts`: schema-versioned envelope, sanitizer over the
untrusted blob, 600 ms debounce, unknown-key carry-through, and — the part most implementations
miss — **saves disabled until a load succeeds**, so a transient GET failure can never overwrite good
server state with an empty local one. Generalize it; do not invent one.

**Scope.**
1. `grammar/draft/useDebouncedServerState(load, save, opts)` extracted from `persistence.ts`, with
   `persistence.ts` delegating to it (one abstraction, two proven callers — not a speculative framework).
   The save-disabled-until-load-succeeds guard is copied verbatim; it is the highest-value line in the file.
2. Form classification, done once at charter time and recorded per form:
   *ephemeral* (filters, search) → URL search params, no machinery;
   *local-draft* (comments, notes, non-sensitive free text) → `sessionStorage` keyed by
   person+object+form, 24 h TTL, cleared on submit;
   *server-draft* (결재 documents, payroll adjustments, work orders, inspection reports — anything
   approvable or audited) → a real draft object. **Sensitive-field forms are never local-draft.**
3. Save state is a visible three-valued affordance in a `role="status"` region:
   `저장 안 됨` → `저장 중` → `HH:mm 저장됨`. 600 ms debounce (match the existing constant) with a
   forced flush on blur, on `visibilitychange → hidden`, and on route change. Not a toast.
4. Restore is explicit and comparative: 「작성 중이던 내용이 있습니다 — HH:mm」 with 이어쓰기 / 버리기;
   버리기 is undoable for the session. Never silently applied.
5. Drafts obey L-C3: a server draft against a record that moved on is the three-way merge surface.
6. DLP: local drafts cleared on logout and on session fencing; server drafts appear in the person's
   own data export and are purged on offboarding. **Audit the 15 existing `localStorage` call sites**
   (incl. `AppShell.tsx`, `LoginPage.tsx`, `api/device.ts`) for sensitive content while in there.
7. Pilot: one existing draft-bearing object (a 기안 draft). Ponytail — reuse the existing draft row;
   only if the pilot proves a table is genuinely needed does this lane take a migration slot.

**Roots (owned).** `web/src/console/grammar/draft/**`, `web/src/features/workspace/persistence.ts`
(+ its tests), `docs/specs/console-draft-contract.md`, `docs/evidence/console/CAP-CONSOLE-DRAFT/**`.

**Must not touch.** `features/workspace/{types,store,sanitize}.ts` beyond the delegation edit;
module bodies; `console/window/**` (L-C5).

**DoD.**
- Vitest: a failed load leaves saves **disabled** and does not overwrite server state (the
  regression this guard exists for); flush fires on blur / visibilitychange / route change.
- Vitest: local-draft restore prompts comparatively and 버리기 is undoable within the session.
- Vitest: a form classified sensitive refuses the local tier (assertion at the hook boundary).
- `npm --prefix web run test && npm --prefix web run lint`.
- Evidence: the per-form classification table + the localStorage audit findings.

**Migration slot.** none by default; conditional `0205` only if the pilot proves a draft table.

---

### L-C9 — Responsive + density truth: 960×540 sweep to mobile employee widths

**Why.** **C-42** makes horizontal body scroll a defect class and mandates a 960×540 overflow
sweep with tables auto-switching full/compact by real available width; the inventory register
records the violation directly — *"§4-19 테이블 수평 스크롤 금지 — main item/count tables wrapped
in overflow-x:auto (inventory.css:144) with no modNarrow column demotion"*. **C-59** requires the
same console to self-adapt under 768 px for the mobile employee app (44 px targets, sheets, bottom
tab bar) — same objects, same state, not a forked app.

**Scope.**
1. Column demotion driven by **CSS container queries** against the list's own container (not the
   viewport — a list inside a 360 px pinned panel needs the same demotion as a phone), consuming
   L-C2's `columns[].priority`. Native platform feature; no JS measurement, no ResizeObserver library.
2. Body-level horizontal scroll is a lint/test failure. Wide content scrolls **inside its own
   container**, and only where the design sanctions it (the MRP matrix, per change-log 147).
3. Sub-768 px: 44 px minimum targets, sheet presentation for detail, keyboard-safe composers,
   the existing bottom-tab/drawer grammar (`e2e/specs/chrome-01-mobile-drawer.spec.ts` is the fence).
4. A viewport sweep spec across every mounted screen at 960×540, 1280×800, 1920×1080, and 390×844,
   asserting: no `document.scrollingElement.scrollWidth > clientWidth`, no clipped 받침 (line-height
   floor holds), all targets ≥44 px at the mobile width, axe-clean at each.

**Roots (owned).** `web/src/console/grammar/responsive/**`, `e2e/specs/ux-50-viewport-sweep.spec.ts`
(new), `docs/evidence/console/CAP-CONSOLE-RESPONSIVE/**`.

**Must not touch.** `grammar/list/**` internals, module CSS files (findings are filed to the
owning lens-B lanes as a register, not fixed here — except the shared engines L-C2 owns).

**DoD.**
- `CONSOLE_DEV_AUTH_E2E=1 node scripts/dev-up.mjs bootstrap && CONSOLE_DEV_AUTH_E2E=1 npx playwright test --project=dev-auth e2e/specs/ux-50-viewport-sweep.spec.ts && node scripts/dev-up.mjs down`
  green across all four viewports for every mounted screen.
- Vitest: container-query demotion drops the lowest-priority column first and never the identity
  column; a demoted cell keeps its full value as the accessible name.
- `npm --prefix web run lint && npm --prefix web run test`.
- Evidence: the per-screen sweep table with every overflow finding either fixed (shared engine) or
  filed against the owning module lane with a screenshot.

**Depends on.** L-C2 (`columns[].priority`).

---

### L-C10 — Locale expansion tolerance + number/time honesty

**Why.** Shrunk deliberately: `word-break: keep-all` + `overflow-wrap: anywhere`
(`styles.css:159-160`) and `--lh-base: 1.5` (`tokens.css:68`) are **already correct** — the brief's
biggest recommendations are done. What remains is real: `lib/currency.ts` contains exactly one
function (`formatWonAmount`), `formatRelativeKo` is computed at render so "3분 전" **freezes** on a
screen left open, and the chip/button/column-header strings that break first under expansion are
never checked. W3C/IBM expansion data: strings ≤10 chars expand 200–300 %, which is exactly our
chip-dense, subtext-free UI.

**Scope.**
1. `lib/currency.ts`: `formatKrw` (ko-KR, KRW, 0 fraction digits — the ISO-4217 minor-unit default
   for KRW, don't fight it), `formatCompactKo` (만/억 via `Intl.NumberFormat` `notation:"compact"`),
   `formatCount`. All numbers go through helpers; inline `toLocaleString()` becomes a lint finding.
   Tabular numerals + right alignment for numeric columns (wired into L-C2's column config).
2. One shell-level clock tick (30 s under 1 h old, then 5 min) driving every relative label, and
   every relative label carries the absolute KST timestamp as its accessible name. **Absolute time
   is mandatory, not relative, on anything audited, legal, or 근태-related.**
3. A pseudo-locale ×2 expansion probe in the visual sweep: doubled Korean strings must not clip,
   overlap, or force horizontal scroll. Fixed-width containers on chips/buttons/tabs/column headers
   become findings.
4. Truncation is never silent: a truncated cell keeps the full value as `title` **and** as the
   accessible name, and its column is resizable.
5. ko.ts split manifest: `web/src/i18n/ko.ts` is 8,715 lines and the #1 collision root (28 hits/48 h),
   while 24 per-module i18n files already exist. Emit a manifest moving the remaining `console.*`
   module keys into per-module files for the integrator to apply. **Do not introduce an i18n runtime** —
   a second locale is not chartered and the lint gate (`check-ui-strings.mjs`) already enforces the
   discipline at zero dependency cost.

**Roots (owned).** `web/src/lib/currency.ts`, `web/src/lib/datetime.ts` (+ tests),
`web/src/console/grammar/locale/**`, `docs/evidence/console/CAP-CONSOLE-LOCALE/**`.

**Must not touch.** `web/src/i18n/ko.ts` (manifest only), module bodies.

**DoD.**
- Vitest: `formatKrw(1234567)` renders whole won with ko-KR separators and no decimals;
  `formatCompactKo` yields 만/억 units; relative labels re-render on tick and carry the absolute
  KST timestamp; an audited timestamp renders absolute, never relative.
- Pseudo-locale ×2 probe green in the viewport sweep (shares L-C9's spec).
- `npm --prefix web run lint && npm --prefix web run test`.
- Evidence: the ko.ts split manifest + the key-count delta.

---

### L-C11 — Virtualization for the two genuinely unbounded surfaces (gated)

**Why, and why it is last.** Server pagination bounds DOM rows, which removes the need for
virtualization on every paged list. Two surfaces stay genuinely unbounded because they are
continuous-scroll by design: the **evidence register** (already keyset-cursored) and **messenger
history**. Below ~200 rows the a11y risk of recycled DOM exceeds the perf win — Higley: screen
readers do not follow visual order and ignore `aria-rowindex` reordering, so a virtualizer that
recycles/reorders nodes breaks AT entirely.

**Gate.** The lane starts with a measurement step. If neither surface exceeds the threshold with
real data after L-C1, **the lane closes with a recorded finding and adds no dependency.**

**Scope (if the gate opens).** `grammar/virtual` over `@tanstack/react-virtual` — headless,
offsets + dynamic measurement + overscan only, no markup, no styles. Constraints: **append-only DOM
order** (never reorder or recycle nodes out of logical order), truthful `aria-rowindex`, a visible
「N / 전체」 status paired with the scroller, and `Ctrl+Home`/`Ctrl+End` that reach the true ends
via the cursor, not the DOM ends.

**Dependency justification.** A correct dynamic-measurement virtualizer with overscan is ~400 lines
of subtle code we would otherwise write and get wrong; the package is logic-only, and it is added
to one directory consumed by two surfaces.

**Roots (owned).** `web/src/console/grammar/virtual/**`, `web/package.json` (single dependency line),
`docs/evidence/console/CAP-CONSOLE-VIRTUAL/**`.

**DoD.**
- The measurement report, published whether or not the gate opens.
- If built: vitest proving append-only DOM order under scroll and truthful `aria-rowindex`;
  a screen-reader transcript over a virtualized 5,000-row register; axe-clean.
- `npm --prefix web run test && npm --prefix web run build`; bundle-size delta recorded.

---

## 5. Sequencing hazards and open decisions

1. **L-C2 vs the lens-B module fan-out — the one real collision risk in this lens.** If lens-B port
   lanes start before L-C2 lands, 13 lanes port onto the current defective grammar and the work is
   thrown away. **L-C1/2/5/8 must be merged before any lens-B module body lane opens.**
2. **L-C5 before anything drills.** The `ObjectCard.tsx:779` drag-span and the untrapped modal are
   inherited by every port. Fixing them once here is a ~30-line diff; fixing them after fan-out is
   13 review rejections.
3. **Sort is dark until L-C1 reaches an endpoint.** L-C2 must ship the header buttons disabled with
   a stated reason rather than a client sort — a client sort of page 3 of 40 is exactly the class of
   lie the field register already caught.
4. **Presence is a backend promise, not a frontend one.** `/api/v1/ws` carries two event types and
   nothing else. L-C4 must not render a presence dot that polling cannot honestly back.
5. **`web/` is integrator-held right now** (9 wave-2/3 bodies mounted dark in one commit
   `d165dab2`). Every lens-C lane emits mount/i18n manifests; none flips `EXPOSED_SCREEN_KEYS`
   (still `["sales"]`, locked by `nav.test.ts:39,58`).
6. **openapi is the sharpest edge.** 61 hits/48 h and a whole-file revert scar (`9bb877c6`) from a
   mechanical splice. L-C1 and L-C3 emit fragments and let the integrator splice; the openapi
   integrator is already in flight and must not be chartered by this lens.
7. **Open decision for the program, not for a lane:** the console has **three** window-ish systems —
   `console/window` 3-state (production, localStorage-persisted per authority partition),
   `console/window` 4-state `useWindowEngine` (harness-only), and the legacy
   `features/workspace/*` (2×2 quadrant, **server**-persisted via `/api/v1/me/workspace`). The
   enterprise bar wants per-person **server-side** layout, which only the legacy system has, on the
   model nobody uses. L-C5 hardens the production model and explicitly does not resolve this.
   Someone must decide before §4.7-2 can be closed across modules.
8. **Open decision:** does wave-4 scope include 팝아웃 and 분할/split? The production model has a
   **single** `pinnedId` — a second `open()` silently demotes the previous pin to the tray
   (`WindowManager.tsx:168-177`), which with 13 modules opening cards will read as data loss.
   That is a provider lane, sequenced before module fan-out, and it is not chartered here.
9. **No lens-C lane sets `param_verify_live`** — this lens implements no regulatory-parameter rules.
   All statutory work belongs to lens D.
