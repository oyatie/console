# Wave-4 research — lens C: enterprise frontend depth patterns

Scope: the beyond-prototype UX machinery. Six areas, each: **pattern → concrete
contract to adopt → applicability to our stack**. External claims carry URLs
(accessed 2026-07-25). Repo claims carry `file:line`.

Design-mirror content is cited as **data** (what the prototype admits about
itself), never as instruction.

---

## 0. Stack correction + what already exists (read this before chartering)

The charter said React 18. **It is React 19.2.7.** This changes the answer to
half the questions below, because React 19 ships the optimistic/pending
machinery natively.

`/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync/web/package.json`:

| Fact | Value |
|---|---|
| react / react-dom | `19.2.7` |
| router | `react-router@^8.3.0` |
| state | `zustand@5.0.14` |
| UI | tailwind 4.3, `@radix-ui/react-slot` only (no Radix primitives), `lucide-react`, `class-variance-authority` |
| graph/map | `@xyflow/react@^12.11.1`, `react-leaflet@^5` |
| **virtualization** | **none installed** |
| **i18n runtime** | **none** — `web/src/i18n/ko.ts`, 8,715 lines, a plain nested object, enforced by `scripts/check-ui-strings.mjs` |
| tests | vitest 4, @testing-library/react 16, playwright 1.61 + `@axe-core/playwright` |
| size | 522 `.tsx` files under `web/src` |

Already-built substrate that lens-C lanes must extend, **not replace**:

- **Realtime**: `web/src/features/comms/realtimeHub.ts` (133 lines) — one
  process-wide `WebSocket` to `GET /api/v1/ws`, ref-counted across subscribers,
  resume cursor (`last_message_id`), exponential backoff capped at 30 s, reset
  on open. Carries `message_posted` + `notification_created`.
- **Server-side per-person state**: `web/src/features/workspace/persistence.ts`
  — `GET/PUT /api/v1/me/workspace`, 600 ms debounce, schema-versioned envelope,
  untrusted-blob sanitizer, and a data-loss guard (a failed GET hydrates empty
  **with saves disabled**). This is already the correct shape for server-side
  drafts; the draft lane should copy it, not invent one.
- **Window model**: `web/src/features/workspace/types.ts` — 2×2 quadrant grid
  (`tl/tr/bl/br` + halves), `PanelMode = "pinned" | "float" | "minimized"`,
  `FloatRect` popouts, minimized tray, dedupe identity `(kind, code)`.
- **KST time**: `web/src/lib/datetime.ts` — `Intl.DateTimeFormat` +
  `Intl.RelativeTimeFormat("ko-KR", {numeric:"auto"})`, all pinned to
  `Asia/Seoul`, em-dash for null/invalid. Correct already.
- **Grid attempt**: `web/src/console/module/ModuleScreen.tsx:331` and `:392`
  — two `role="grid"` regions. Both are **wrong** (§1.3).

Backend facts (`backend/openapi/openapi.yaml`, 30,531 lines):

- **Pagination is offset/limit**, not cursor: 69 `name: limit` params, ~dozens
  of `name: offset`, exactly **one** `name: cursor` (line 1172). List envelopes
  carry `{items, limit, offset, total}` (e.g. line 15443); `has_more` appears
  once (line 18989).
- **Optimistic concurrency already exists but only in Ontology**: `If-Match`
  required header (line 12070) with a strong validator pattern
  `^"ont-object-type-key:[0-9a-f]{32}:r[1-9][0-9]*"$`, `ETag` response headers,
  and shared responses `Conflict` (15234), `PreconditionFailed` (15240,
  re-emits current `ETag` + `Cache-Control: no-store`), `PreconditionRequired`
  (15252).
- `ErrorBody` already has the merge hook: `error.reasons[]` (machine-readable)
  and `error.current_key_write_revision`. **409 appears 168 times** across the
  spec — i.e. conflict is pervasive in the backend and essentially unsurfaced
  as an affordance in the frontend.

What the prototype admits, as data:
`docs/design/oyatie-console/BENCHMARK.md:23` — "**스케일**: 1,284명·수백 행은
시드 — 가상 스크롤·서버 페이지네이션·인덱스 검색 없음." The prototype names
the exact three gaps §1 covers.

---

## 1. Data-grid enterprise grammar

### 1.1 Pattern — the ARIA grid pattern, and when *not* to use it

APG grid (https://www.w3.org/WAI/ARIA/apg/patterns/grid/):

- Roles: `grid` → (`rowgroup`) → `row` → `columnheader` / `rowheader` /
  `gridcell`. **All content must be inside a cell.**
- Composite widget: "Only one of the focusable elements contained by the grid
  is included in the page tab sequence" → **roving tabindex**, not
  `aria-activedescendant`, and the author writes the focus movement.
- Keys: arrows = one cell; `Home`/`End` = row ends; `Ctrl+Home`/`Ctrl+End` =
  grid ends; `PageUp`/`PageDown` = author-defined row jump;
  `Ctrl+Space`/`Shift+Space` = column/row select; `Shift+Arrow` extends.
- Properties: `aria-rowcount`/`aria-colcount` on the grid,
  `aria-rowindex`/`aria-colindex` per row/cell, `aria-selected`,
  `aria-readonly`, `aria-sort`, `aria-multiselectable`.
- The APG's own lazy-load warning: "key events that move focus to the
  beginning or end of the grid, such as Control + End, may move focus to the
  last row in the DOM rather than the last available row in the back-end data."

The counter-authority matters as much as the pattern:

- Adrian Roselli, *ARIA Grid As an Anti-Pattern*
  (http://adrianroselli.com/2020/07/aria-grid-as-an-anti-pattern.html):
  reserve `role="grid"` for genuinely Excel-like 2-D editing. For read-only
  tabular data, "an HTML `<table>` is all you need" — sorting, sticky headers
  and embedded controls all work in a plain table. With `role="grid"` he
  documents: Tab-vs-arrow confusion with no discoverable affordance, meaningless
  arrow semantics after responsive reflow, JAWS announcing the role while
  navigation commands fail, and broken column tracking under infinite scroll.
- Sarah Higley, *Grids Part 2: Semantics*
  (https://sarahmhigley.com/writing/grids-part2/): `aria-rowcount`/`-rowindex`
  affect **announcements only** — they never change DOM traversal order.
  "Screen readers do not follow the visual order" and ignore `aria-rowindex`
  reordering, so a virtualizer that recycles/reorders DOM nodes breaks AT
  entirely. Non-semantic wrapper divs need `role="presentation"`. She flags
  `aria-selected` on gridcells as use-at-your-own-risk and demonstrates
  **checkbox-in-a-cell** selection instead.
- `aria-rowcount="-1"` is the standard signal for "total unknown"
  (https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-rowcount).
- `aria-sort` takes exactly `ascending` / `descending`, lives on the sorted
  header only, and is **removed** from the previous header on change; the header
  itself should be a `<button>`
  (https://www.w3.org/WAI/ARIA/apg/patterns/table/examples/sortable-table/).

Virtualization: TanStack Virtual
(https://tanstack.com/virtual/latest/docs/introduction) is headless — a
`Virtualizer` computing offsets for vertical/horizontal/grid axes with dynamic
measurement and overscan, no markup or styles, framework adapters incl. React.
It is the virtualization logic alone, not a table library.

### 1.2 Contract to adopt — "Console Grid Contract v1"

1. **Two components, one decision rule.** `DataTable` (semantic `<table>`,
   Tab-and-links focus, row-level actions) is the **default**. `DataGrid`
   (`role="grid"`, roving tabindex, 2-D arrows) is allowed **only** for
   cell-editable surfaces (payroll cells, attendance corrections, ontology
   property matrices). A `role="grid"` on a read-only list is a review reject.
2. **Server pagination is the source of truth for counts.** Because our list
   envelopes carry `total`, render `aria-rowcount={total}` and
   `aria-rowindex={offset + i + 1}` on every row (1-based, header row = 1 where
   present). Where `total` is genuinely unknown, `aria-rowcount="-1"` — never
   the loaded-page length.
3. **Virtualize only above a measured threshold.** Below ~200 rows render all
   rows: the a11y risk of recycled DOM exceeds the perf win. Above it,
   virtualize with **append-only DOM order** (never reorder/recycle nodes out of
   logical order), keep `aria-rowindex` truthful, and pair the scroller with a
   visible "N of TOTAL" status.
4. **`Ctrl+End` must page to the true last row**, not the last DOM row — fetch
   the tail page, then focus. Same for `Ctrl+Home` after deep scroll. This is
   the APG's named lazy-load failure and is the one keyboard behaviour most
   grids get wrong.
5. **Column ops**: sort = header `<button>` + `aria-sort` on the sorted header
   only, and the sort must be **server-side** (`?sort=`), never a client sort of
   the loaded page — a client sort of page 3 of 40 is a lie. Resize keeps the
   existing pointer handle but adds keyboard: focus the handle,
   `ArrowLeft/Right` = ±8 px, `Home` = reset, and the handle needs an accessible
   name (today it is `aria-hidden` with a `title` —
   `console/module/ModuleScreen.tsx:317-323`). Column show/hide + order persist
   per person via the existing `/api/v1/me/workspace` envelope (a new top-level
   key; `persistence.ts` already carries unknown keys through).
6. **Bulk selection**: a real `<input type="checkbox">` inside the first cell
   per Higley, plus a header select-all that is **page-scoped by default** and
   offers an explicit "select all N matching this filter" escalation as a second
   affordance (page-scoped vs query-scoped must never be ambiguous — this is
   where bulk-op incidents come from). Selection count lives in a
   `role="status"` region. `Shift+Click` extends by range; `Ctrl/Cmd+Click`
   toggles. Selection survives pagination only if query-scoped; page-scoped
   selection is cleared on page change **with an announcement**, never silently.
7. **Bulk mutation is server-batched with per-item outcome**: one request, a
   response of `{id, ok|error}` per item, and a results panel that lists
   failures with a retry-failed-only action. A bulk op that reports only
   "23 succeeded" is not shippable.

### 1.3 Applicability — and a live defect

`web/src/console/module/ModuleScreen.tsx` currently violates the pattern in
four ways, and it is the shared module shell, so the defect is replicated
across every module screen:

- The `role="columnheader"` spans (`:315`) sit in a **sibling div outside** the
  `role="grid"` container (`:331`) and are not wrapped in a `role="row"` — the
  headers are therefore not in the grid's accessibility tree at all.
- Rows are `role="row"` **direct children of the grid** with no `rowgroup`, and
  carry `aria-selected` while never being focusable (the grid itself holds
  `tabIndex={0}`; no roving tabindex, no `aria-activedescendant`). Keyboard
  users get one tab stop and a J/K handler AT cannot discover.
- No `aria-rowcount` / `aria-rowindex` / `aria-colcount` anywhere in `web/src`
  (verified by grep: zero hits).
- The Kanban (`:392`) applies `role="grid"` with lanes as `role="row"` and
  **cards as `role="gridcell"`**. A kanban is not a grid; this should be nested
  lists with a `listbox`-style selection or plain buttons.

Dependency call: **do not add a table library.** For the virtualized cases add
`@tanstack/react-virtual` (headless, ~small, no markup) *only* when a lane has a
measured list that exceeds the threshold — justified because a correct
dynamic-measurement virtualizer with overscan is ~400 lines of subtle code we
would otherwise write and get wrong. Everything else (sort, resize, column
persist, selection) is our own code over a plain `<table>`.

---

## 2. Optimistic concurrency UX (409 / 412 as merge affordances)

### 2.1 Pattern

- `If-Match: <etag>` on the mutating request; mismatch → **412 Precondition
  Failed** (not 428 — 428 is *missing* precondition). Comparison is the **strong**
  algorithm, so a `W/`-prefixed weak ETag never matches. Canonical client flow:
  re-fetch → merge or retry → resend with the new validator
  (https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/If-Match).
- 409 is the domain-level counterpart: state conflict / illegal transition,
  where the resource version may be fine but the transition is not.
- The UX literature converges on three moves, in order of preference:
  (a) **prevent** — presence indicators and soft locks so two people rarely
  collide; (b) **auto-merge at field granularity** — disjoint field edits merge
  silently, only same-field edits surface; (c) **surface a choice** — show mine
  vs theirs vs merged, never a toast.
  (https://www.fernandoux.com/en/wiki/concepts/conflict-resolution/,
  https://medium.com/@saurabh.singh.shakya/smart-save-without-overwrite-building-conflict-aware-collaborative-systems-9b20db91478d
  — secondary sources; treat as corroboration of the shape, not as spec.)
- React 19 `useOptimistic(value, reducer?)`
  (https://react.dev/reference/react/useOptimistic): optimistic state applies
  immediately inside an Action/Transition and **converges with the real value in
  a single render** when the Action settles; on throw it **reverts to `value`**.
  Documented caveat, and the one that matters here: *it does not preserve the
  user's input on failure* — the draft must be held in separate state and
  restored by the caller.

### 2.2 Contract to adopt — "Conflict Contract v1"

1. **Every mutating console request carries a validator.** Generalize the
   Ontology precedent (`openapi.yaml:12070`, `:15240`) beyond Ontology: strong
   `ETag` on the GET/detail response, required `If-Match` on PUT/PATCH/POST-
   transition. Adopt the shape already in the spec, including the 412 response
   **re-emitting the current `ETag` and `Cache-Control: no-store`**.
2. **Conflict responses must be actionable, not just typed.** `ErrorBody`
   already has `error.reasons[]` and `error.current_key_write_revision`. Extend
   the console convention: a 409/412 body carries (i) `reasons[]` as stable
   codes the UI maps to Korean copy, and (ii) enough of the **current server
   state** to render a comparison without a second round trip. A conflict the
   client can only respond to by discarding is a bug in the contract.
3. **Three-way surface, not a toast.** On 409/412 the editing surface enters a
   `conflict` state in place: the affected fields render **your value / current
   value** side by side with per-field "keep mine" / "take theirs", disjoint
   fields auto-merge with a "merged automatically" marker, and the primary
   button becomes "다시 저장" armed with the fresh validator. The pane stays
   open; nothing the user typed is discarded. The conflict region gets
   `role="alert"` for the summary and the first conflicting field takes focus.
4. **Optimistic updates are permitted only where reversal is cheap and
   visible** — chips, toggles, row status, ordering. Never for money, approvals,
   or anything that emits an audit event: those show a pending state and wait.
   This is the same line lens D draws for "computable vs attested".
5. **Retry preserves the draft, always.** Standard hook shape:
   `useOptimistic` for the display value + a separate `draft` state that is
   written before the Action and cleared only on success. On failure: draft
   restored, field-level errors from `reasons[]`, focus to the first invalid
   field, and the retry button re-enabled. React's automatic revert alone loses
   the user's typing — this is documented, not speculative.
6. **Idempotency for the retry.** A retried transition must carry a client
   request id so a 409 caused by "my own first attempt actually succeeded" is
   resolved as success, not shown as a conflict.

### 2.3 Applicability

Low-dependency, high-yield: `useOptimistic` / `useActionState` /
`useTransition` are in React 19.2.7 already — no TanStack Query, no SWR needed
for this. The work is (a) a shared `useConflictAwareMutation` wrapper over
`api/client`, (b) a `ConflictPanel` presentation component, (c) threading
`ETag`/`If-Match` through the generated TS client, and (d) spreading the
existing Ontology OpenAPI shape to other write paths. Note the OpenAPI drift
gate: every route change needs `openapi.yaml` + regenerated
`clients/{ts,kotlin,swift}` (three independent CI gates).

---

## 3. Draft autosave / restore

### 3.1 Pattern

The distinction that matters is **who owns the draft**:

- *Local draft* (browser): survives an accidental navigation, a crash, a 500.
  Does not follow the person to another device, and is a PII/DLP liability if it
  holds 주민등록번호-class data.
- *Server draft object* (a real row with a lifecycle): follows the person,
  participates in RLS/authz/audit, can be resumed by a delegate, and can be
  reported on ("3 unsubmitted 결재 drafts"). Cost: a write path, a retention
  rule, and a conflict story of its own.

Our repo already contains the correct server-draft engineering, in
`features/workspace/persistence.ts`: schema-versioned envelope, sanitizer over
the untrusted blob, debounced save, unknown-key carry-through, and — the part
most implementations miss — **saves disabled until a load succeeds**, so a
transient GET failure can never overwrite good server state with an empty
local one.

### 3.2 Contract to adopt — "Draft Contract v1"

1. **Classify every form once**, at charter time:
   - *ephemeral* (filters, search) → URL search params, no draft machinery;
   - *local-draft* (comments, notes, non-sensitive free text) → `sessionStorage`
     keyed by `person + object + form`, TTL 24 h, cleared on successful submit;
   - *server-draft* (결재 documents, payroll adjustments, work orders,
     inspection reports, anything approvable or audited) → a real draft object.
   Sensitive-field forms are **never** local-draft.
2. **Save state is a visible, three-valued affordance**, not a toast:
   `저장 안 됨` → `저장 중` → `HH:mm 저장됨` in a `role="status"` region
   (`aria-live="polite"`). Debounce 600 ms (match the existing constant) with a
   forced flush on blur, on `visibilitychange → hidden`, and on route change.
3. **Never overwrite good state with a failed load.** Copy the
   `saveEnabled=false-until-load-succeeds` guard verbatim. This is the single
   highest-value line in `persistence.ts`.
4. **Restore is explicit and comparative.** On reopening an object with a newer
   draft than the server record, do not silently apply it: show
   "작성 중이던 내용이 있습니다 — HH:mm" with 이어쓰기 / 버리기, and 버리기 is
   undoable for the session.
5. **Drafts obey the conflict contract.** A server draft against a record that
   moved on is the §2 three-way surface, not a silent clobber.
6. **Retention + DLP.** Server drafts expire on a stated schedule, appear in the
   person's own data export, and are purged on offboarding. Local drafts are
   cleared on logout and on 세션 fencing.

### 3.3 Applicability

Generalize `persistence.ts` into a `useDebouncedServerState(load, save, opts)`
hook and keep the workspace layout as its first caller — one abstraction with
two proven callers, not a speculative framework. `sessionStorage` covers the
local tier with no new dependency (note: `localStorage` is already used in 15
files, incl. `AppShell.tsx`, `LoginPage.tsx`, `api/device.ts` — the draft lane
should audit those for sensitive content while it is in there).

---

## 4. Realtime presence / unread at enterprise scale

### 4.1 Pattern

- **Subscribe to what is on screen, not to the org.** Slack's Flannel
  (https://slack.engineering/flannel-an-application-level-edge-cache-to-make-slack-scale/):
  the boot payload that shipped every user object collapsed by **7×** on a 1.5K
  team and **44×** on a 32K team once user objects were lazily fetched, and
  moving presence to per-view pub/sub — "subscribe to the series of events that
  are relevant in the current view" — cut presence events reaching clients by
  **5×**. For a conglomerate with tens of thousands of employees this is the
  whole design.
- Unread counters are the expensive half: per-user read positions across large
  membership sets create write pressure, and >10k-member channel fan-out is the
  known scaling cliff; the standard mitigations are a pub/sub backplane plus
  **batched** delivery grouped per socket
  (https://websocket.org/guides/websockets-at-scale/ — secondary; corroborates
  the shape).
- **SSE vs WebSocket**: `EventSource` gives auto-reconnect and `Last-Event-ID`
  resume for free, but is server→client only and is capped at **6 concurrent
  connections per browser per origin over HTTP/1.1** (Chrome and Firefox both
  "won't fix"); HTTP/2 lifts this to the stream limit (~100)
  (https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events).
  With a console people keep open in several tabs, HTTP/1.1 SSE is a trap.
- **Announcement semantics** (https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Guides/Live_regions):
  `role="log"` (implicit polite) for chat/sequential events where order matters;
  `role="status"` (implicit polite) for counters and connection state;
  `role="alert"` (implicit assertive) **only** for interrupting errors. Add the
  redundant explicit `aria-live` for compatibility. `aria-relevant="additions
  removals"` is the documented pattern for a **presence roster**.
  APG adds: an alert must not move focus, must not auto-dismiss (auto-dismiss
  risks a WCAG failure), and must not fire often
  (https://www.w3.org/WAI/ARIA/apg/patterns/alert/).

### 4.2 Contract to adopt — "Realtime Contract v1"

1. **Keep the single multiplexed socket.** `realtimeHub` is already the right
   architecture (one connection, ref-counted, resume cursor, capped backoff).
   Extend its event union; do not open a second socket or introduce SSE
   alongside it.
2. **Add view-scoped subscriptions.** `subscribe(params, listener, {topics})`
   where topics are the objects/threads/rosters currently rendered; unsubscribe
   on unmount. Presence is requested for **visible avatars only**. Without this,
   presence is O(employees) per client and will not survive the conglomerate
   scale the platform is chartered for.
3. **Add liveness detection.** Today reconnect is driven purely by the `close`
   event (`realtimeHub.ts:79`); a half-open socket (idle NAT, VPN drop) never
   fires `close`, so the console silently stops updating while looking healthy.
   Contract: server heartbeat every 30 s, client watchdog at 45 s → force
   `close()` → existing backoff path.
4. **Connection state is first-class UI, in three states**, surfaced in the
   shell: `실시간` (silent) / `재연결 중` / `오프라인 — 마지막 갱신 HH:mm`. The
   degraded states go in a `role="status"` region, and every realtime-derived
   number (unread counts, presence dots, live totals) must visibly go stale
   rather than lie — dim the value and show the last-updated time. Never a
   spinner that implies live.
5. **Graceful degradation is a poll, not a blank.** Offline/failed socket →
   fall back to on-focus + interval refetch of the same endpoints at a low rate,
   and re-sync from the resume cursor when the socket returns. Actions taken
   while offline queue as drafts (§3) and replay under the conflict contract
   (§2) — they do not fail silently.
6. **Unread is server-authoritative and read-position based.** The client never
   derives unread by counting locally-received events; it renders a
   server-computed count against a read cursor it advances explicitly. Otherwise
   a missed reconnect window makes the badge permanently wrong.
7. **Announcements are rate-limited.** Incoming messages go into a
   `role="log"`; the unread badge is `role="status"` and updates at most once
   per ~2 s (batched). No realtime event ever produces `role="alert"` except a
   genuine interrupt (session fenced, 권한 revoked).

### 4.3 Applicability

All of this is edits to `realtimeHub.ts` plus a shell status component and a
backend topic-subscription frame. No new dependency. The one non-trivial
backend ask is per-view presence topics; that is the item to charter early
because the frontend contract depends on it.

---

## 5. Accessibility for dense Korean enterprise UIs

### 5.1 Grids, steppers, chips

- **Grids**: §1.2. The single highest-value change is *stopping* the
  read-only-`role="grid"` misuse; a plain `<table>` gives Korean screen-reader
  users working table navigation for free, which the current div-grid does not.
- **Steppers / process bars**: not a native pattern. Contract: `<ol>` of steps,
  the current step carrying `aria-current="step"`, completed/pending state in
  **text** inside the step (not colour or an icon alone), and the whole stepper
  labelled. If a step is a link/button it stays a real link/button.
- **Chips (상태 chips)**: the console standard is "status = chips, no
  explanatory subtext" (project rule). A11y contract: a status chip is
  **text**, not a coloured dot with a `title`. A removable chip is
  `<button aria-label="{label} 제거">` inside the chip; a chip group is a list;
  removing a chip moves focus to the next chip, or to the group container when
  the last is removed, and announces the removal in a `role="status"`. A chip
  that is purely informational must not be focusable.
- **Every icon-only control needs a Korean accessible name.** The resize handle
  at `ModuleScreen.tsx:317` (`aria-hidden` + `title`) is the pattern to grep for
  across all 522 components.

### 5.2 The window model — the unusual part

Our model (`features/workspace/types.ts`) is a 2×2 quadrant grid of **pinned
panels that reflow the page body** (not overlays), plus **`float` popouts**
(`position: fixed`, `DEFAULT_FLOAT_RECT`), plus a **minimized tray**, persisted
server-side per person. The APG has no pattern for this. The closest anchors:
APG's modal dialog rules (initial focus inside, contained tab sequence, Escape
closes, focus returns to the invoker unless it no longer exists, `aria-modal`,
`aria-labelledby` on a visible title —
https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/) and its note that
non-modal dialogs also contain their tab sequence.

**Window a11y contract the model must satisfy:**

1. **Pinned panels are landmarks, not dialogs.** A pinned panel reflows content
   and is non-modal → `role="region"` (or `complementary`) with
   `aria-label={object title}`. It must **not** be `aria-modal`, must **not**
   trap focus, and must **not** steal focus on pin. Nothing outside becomes
   inert.
2. **Float popouts are non-modal dialogs.** `role="dialog"`, `aria-modal="false"`,
   `aria-labelledby` the panel title, Escape closes and returns focus, and
   they are **not** focus-trapped (the user must be able to Tab back to the
   underlying screen — that is the point of a popout).
3. **Tab order follows the visual quadrant order, always.** `tl → tr → bl → br
   → floats → tray`, regardless of the order panels were created. Floats are
   `position: fixed` and therefore visually detached from DOM order — they need
   an explicit ordering rule or the reading order is arbitrary.
4. **Every window operation is keyboard-reachable and announced.** Pin,
   move-to-quadrant, popout, minimize, restore, close must exist as menu items
   with shortcuts, not drag-only. Drag-to-snap (the `SnapZone` model) is an
   accelerator, never the only path.
5. **Focus never disappears.** On minimize → focus the tray chip. On close →
   focus the invoking row/chip, or the panel's container if the invoker is gone
   (APG's rule). On restore-from-tray → focus the panel's title.
6. **State changes are announced once**, in a single shell-level
   `role="status"`: "{title} 우측 상단에 고정됨" / "최소화됨" / "닫힘". One
   region, not one per panel.
7. **A skip mechanism.** With four panels plus a tray, a keyboard user needs
   "본문으로 이동" and a panel-cycling shortcut (e.g. `F6`, the OS convention
   for pane cycling) that walks section → panels → tray.
8. **Persisted layout must be sane on load.** The sanitizer already drops
   unknown pin kinds; extend it to guarantee the restored layout is
   keyboard-reachable (no zero-size float, nothing positioned off-viewport —
   clamp `FloatRect` to the viewport on hydrate).

Verification: `@axe-core/playwright` is already installed. Axe cannot see any of
7 above; the window a11y contract needs **explicit Playwright keyboard journeys**
(pin → popout → minimize → restore → close, asserting `document.activeElement`
at every step) as merge evidence. Axe-clean is necessary, not sufficient.

### 5.3 Korean-specific

- CJK glyphs need more vertical space — the W3C notes East Asian and other
  complex scripts wanting on the order of ~150 % of the Latin line box
  (https://www.w3.org/International/articles/article-text-size/). Dense Korean
  tables set at Latin line-heights clip 받침 and diacritics.
- Korean screen readers read the accessible name; a chip whose meaning lives in
  colour reads as nothing.

---

## 6. i18n expansion tolerance + ko formatting

### 6.1 Pattern

- **Expansion by source length** (W3C, citing IBM):
  ≤10 chars → 200–300 %; 11–20 → 180–200 %; 21–30 → 160–180 %; 31–50 →
  140–160 %; >70 → 130 %. So the **short strings — buttons, chips, column
  headers, tab labels — are the ones that break**, which is exactly our
  chip-dense, subtext-free UI. Guidance: no tight fixed-width containers, allow
  reflow, keep presentation separate so font/line-height can be tuned per
  locale, never bake text into images, and account for the fact that
  abbreviations often have no target-language equivalent
  (https://www.w3.org/International/articles/article-text-size/).
- **Korean line breaking**: `word-break: keep-all` prevents breaks inside CJK
  runs, i.e. keeps Korean 어절 intact instead of wrapping mid-word (the browser
  default for Korean breaks anywhere). `word-break: auto-phrase` (experimental)
  goes further, using language analysis to avoid breaking natural phrases.
  `word-break: break-word` ≡ `overflow-wrap: anywhere` + `word-break: normal`
  (https://developer.mozilla.org/en-US/docs/Web/CSS/word-break).
- **Numbers**: `Intl.NumberFormat` `notation: "compact"` yields locale-native
  large-number units — the MDN example shows `zh-CN` producing `9.9亿`; ko-KR
  behaves analogously with 만/억. For **KRW**, `style: "currency"` defaults
  `maximumFractionDigits` to the ISO-4217 minor-unit count, which is **0** for
  KRW — so won amounts round to whole units with no decimals by default
  (https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat).
- **Relative time**: `Intl.RelativeTimeFormat("ko-KR", {numeric:"auto"})` gives
  "어제" / "3분 전" from CLDR data with no literals — already how
  `lib/datetime.ts` does it.

### 6.2 Contract to adopt — "Locale Contract v1"

1. **Design to 2× the Korean string for anything ≤10 chars.** Buttons, chips,
   tabs, column headers: no fixed widths, `min-width` + wrap, and a visual
   regression check with a pseudo-locale that doubles string length. Even
   ko-only today, this is what makes the layout robust to a real Korean string
   that turned out longer than the mock's.
2. **Truncation is never silent.** A truncated cell keeps the full value in
   `title` **and** as the accessible name, and the column is resizable. Prefer
   wrapping over ellipsis in dense tables.
3. **Global CSS: `word-break: keep-all` + `overflow-wrap: anywhere`** on text
   containers. `keep-all` stops mid-어절 wrapping; `overflow-wrap: anywhere`
   is the escape hatch so an unbreakable token (a long code, a URL) still cannot
   blow out the layout. This is one CSS rule and fixes a whole class of ugly.
4. **Korean line-height floor.** Body/table text ≥ 1.5 (not the 1.2–1.35 Latin
   default) so 받침 and 이중모음 are not clipped in dense rows.
5. **All numbers go through helpers, never `toLocaleString()` inline.** Extend
   `lib/currency.ts`: `formatKrw` (`ko-KR`, KRW, 0 fraction digits — matching
   the ISO default rather than fighting it), `formatCompactKo` (만/억 for KPI
   tiles), and `formatCount`. Tabular numerals + right alignment for every
   numeric column.
6. **Relative time ticks and stays honest.** `formatRelativeKo` is correct but
   computed at render, so "3분 전" freezes on a screen left open. Contract: one
   shell-level clock tick (30 s under 1 h old, then 5 min) driving all relative
   labels, and every relative label carries the absolute KST timestamp in its
   `title`/accessible name. Absolute time is mandatory (not relative) on
   anything audited, legal, or 근태-related.
7. **Keep the ko.ts discipline.** 8,715 lines of nested object with a lint gate
   is not elegant, but it is enforced and zero-dependency. Do **not** introduce
   an i18n runtime for a single-locale product; the cost is only justified when
   a second locale is actually chartered. What *should* change: split ko.ts by
   module (it is a named collision root at
   `docs/program/console-enterprise-roadmap.md:253`, so every parallel lane
   fights over it) and add pluralization/interpolation helpers where lanes are
   currently concatenating.

---

## 7. Cross-cutting: what to charter, in order

1. **Grid contract** — biggest surface (shared module shell), contains a live
   a11y defect, and unblocks scale. Split: (1a) `<table>` conversion +
   `role="grid"` removal from read-only lists; (1b) server pagination/sort
   plumbing; (1c) virtualization above threshold; (1d) bulk selection + batched
   mutation with per-item results.
2. **Conflict contract** — the backend already models it (168× 409, full
   412/428 machinery in Ontology); the frontend simply does not surface it.
   Cheap, and it is the difference between a demo and a multi-user system.
3. **Realtime hardening** — heartbeat/watchdog and connection-state UI are
   small and fix a silent-failure class today; view-scoped presence needs a
   backend counterpart, so charter it early even if it lands later.
4. **Draft contract** — generalize the hook that already exists.
5. **Window a11y contract** — no library can verify it; needs written keyboard
   journeys as merge evidence.
6. **Locale contract** — mostly one CSS rule, one line-height token, three
   number helpers, and a ticking clock; cheapest of the six.

Dependency verdict overall: **one new dependency justified**
(`@tanstack/react-virtual`, headless, only for measured-large lists). Everything
else is React 19 built-ins, `Intl`, CSS, and generalizing hooks already in the
repo.

---

## Sources

External (accessed 2026-07-25):

- https://www.w3.org/WAI/ARIA/apg/patterns/grid/
- https://www.w3.org/WAI/ARIA/apg/patterns/table/examples/sortable-table/
- https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/
- https://www.w3.org/WAI/ARIA/apg/patterns/alert/
- http://adrianroselli.com/2020/07/aria-grid-as-an-anti-pattern.html
- https://sarahmhigley.com/writing/grids-part2/
- https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-rowcount
- https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Guides/Live_regions
- https://tanstack.com/virtual/latest/docs/introduction
- https://react.dev/reference/react/useOptimistic
- https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/If-Match
- https://slack.engineering/flannel-an-application-level-edge-cache-to-make-slack-scale/
- https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
- https://websocket.org/guides/websockets-at-scale/ (secondary)
- https://www.fernandoux.com/en/wiki/concepts/conflict-resolution/ (secondary)
- https://www.w3.org/International/articles/article-text-size/
- https://developer.mozilla.org/en-US/docs/Web/CSS/word-break
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat

Repo (worktree `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`):

- `web/package.json`
- `web/src/console/module/ModuleScreen.tsx:315,317,331,392,414`
- `web/src/features/comms/realtimeHub.ts:1-133` (esp. `:79` close-driven reconnect)
- `web/src/features/workspace/types.ts`, `persistence.ts`, `sanitize.ts`
- `web/src/lib/datetime.ts`
- `web/src/i18n/ko.ts` (8,715 lines)
- `backend/openapi/openapi.yaml:1172,12070,15234,15240,15252,15443,18989`
- `docs/design/oyatie-console/BENCHMARK.md:23` (design-mirror data)
- `docs/program/console-enterprise-roadmap.md:253` (ko.ts as collision root)
- scratchpad `intent/north-star-amendment-beyond-prototype.md` (lens-C directive)
