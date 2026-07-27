# Wave-4 charter — LENS B: fidelity floor (shared-grammar ports + backend-blocked gaps)

Integration worktree: `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`
Branch: `wave23-consolidation-20260724`. Migration high-water **0202** (`0201` reserved for the
docs evidence-retention renumber) → lens-B provisional slots start at **0210** (gap left for lens A);
the integrator renumbers at merge.

Evidence base (every lane cites these, none re-derives them):
- `scratchpad/wave4/scout-shared-grammar.md` (peer brief: contracts, port recipe, codeGrammar)
- `scratchpad/wave4/scout-shared-grammar-b.md` (delta brief: a11y defects, BARE_CODE_RE, port fork, code triage)
- `scratchpad/wave4/scout-spine-delta.md` (hot zones, collision roots, Buck2 breakage, epoch hold)
- `scratchpad/wave4/research-depth-patterns-frontend.md` §5.2 (window a11y contract)
- `tasks/wlhg23xnz.output` → `result.registers[]` (15 fidelity registers, 180 findings, 69 BE-blocked)
- `scratchpad/intent/north-star-amendment-beyond-prototype.md` (fidelity = floor, truthfulness line)

---

## 0. THE FINDING THAT RESETS THIS LENS

Both grammar briefs assume `WindowManagerProvider` is ambient because it is mounted at
`web/src/components/shell/AppShell.tsx:172`. **It is not ambient for the modules.** Verified:

| Route | Shell | Window system |
|---|---|---|
| `/overview`, `/attendance` | `components/shell/ConsoleShell.tsx` | `features/workspace` — quadrant panels + float + tray, **server-persisted** (`useWorkspacePersistence`) |
| legacy pages (`/users`, `/ontology`, …) | `components/shell/AppShell.tsx` | `console/window` **3-state** `WindowManagerProvider` |
| **`/console/*` — all 15 module bodies** | `console/shell/ConsoleShell.tsx` (via `ConsoleApp`, `AppRouter.tsx:437-449`) | **NONE.** `grep -n "WindowManager\|window/" console/shell/ConsoleShell.tsx` → zero hits |
| `/console-dev/window` (DEV only) | `console/window/harness.tsx` | `useWindowEngine` 4-state carbon copy |

Consequences, all verified in source:
- `console/screens/leave/LeaveBody.tsx:463` renders `LeaveConsole` under `/console`. Its
  `useOptionalWindowManager()` (`LeaveConsole.tsx:581`) therefore returns **null in the running app**.
  The "exemplar adoption" is real code that is dead in production and green only in jsdom tests.
- Same for `GenericModuleScreen` (`ModuleFinanceScreenBody`, `ComplianceModuleScreenBody` are both
  `/console` bodies): every `open(objectCardWindowEntry(...))` falls through to `ObjectCardModal`.
- There are **THREE** window systems, not two. `console/window/types.ts:1-14` states the 4-state engine
  is a verbatim carbon copy of the prototype's *card* engine and explicitly "NOT the legacy
  `features/workspace` quadrant-panel system … see prototype-anatomy/01 'do not conflate'".

So the design grammar's pin/popout/tray/split/preset model is implemented **twice** and mounted
**zero times** on the surface the 15 modules live on. Chartering 13 module ports before fixing this
would ship 13 modals labelled "pin".

### Decision this charter takes (L-B0 executes and records it)

**Mount `WindowManagerProvider` (the 3-state model) into `console/shell/ConsoleShell.tsx` now.
Do NOT promote `useWindowEngine` in wave 4.**

Rationale: the 3-state provider is test-covered (`WindowManager.test.tsx` 373L), a11y-labelled, and
requires **zero per-module configuration** — a module supplies a descriptor and calls `open()`.
`useWindowEngine` demands a per-screen `CardRegistry`/`CardMeta` per module (13× new config), a host
component, persistence, and the whole §5.2 window-a11y contract (which axe cannot verify — it needs
explicit Playwright keyboard journeys). Promoting it is a genuine lens-C/foundation project, not a
prerequisite for closing the fidelity floor.

**The binding contract every module lane copies verbatim into its DoD:**

> This lane consumes the window model ONLY through
> `useOptionalWindowManager()` + `windowManager?.open(objectCardWindowEntry(descriptor, handlers))`,
> with `ObjectCardModal` as the null-provider fallback. It does not read or write
> `pinnedId`, panel width, tray state, split ratio, or layout presets directly, and it does not
> import from `console/window/{useWindowEngine,WindowEngine,types,geometry,sanitize}` or from
> `features/workspace/**`. The mounted model has a **single** `pinnedId` — a second `open()` demotes
> the previous pin to the tray (`WindowManager.tsx:168-177`). Popout, split, and named layout presets
> **do not exist**; any fidelity finding demanding them is recorded in this lane's gap manifest as
> deferred-to-L-B0b with the finding's register anchor, never faked and never described as shipped.

That one paragraph is what makes a later provider upgrade (L-B0b) a swap behind one context instead
of 13 rewrites.

---

## 1. Lane index (ranked; foundation lanes first, then value order)

| # | Lane | Kind | Size | Gate |
|---|---|---|---|---|
| 1 | L-B0 window host decision + mount + test helper | FOUNDATION/DECISION | M | **blocks all fan-out** |
| 2 | L-B1 code-grammar unification | FOUNDATION | S | blocks all fan-out |
| 3 | L-B2 shared drag/modal a11y | FOUNDATION | S | blocks all fan-out |
| 4 | L-B3 object-code triage + port-path assignment | FOUNDATION/ANALYSIS | S | blocks all fan-out |
| 5 | L-B4 object-code issuance kernel + registry seed | X-CUT BE | M | parallel w/ foundation |
| 6 | L-B5 object typeahead/search fabric | X-CUT BE | M | parallel; **codex collision** |
| 7 | L-B6 operational object runtime read surface | X-CUT BE | L | parallel; **codex collision** |
| 8 | L-B7 approval-compose prefill contract | X-CUT BE | M | parallel |
| 9 | L-B8 equipment (40) | FULL-STACK | L | after L-B0..3 |
| 10 | L-B9 logistics (42) | FULL-STACK | L | after L-B0..3 |
| 11 | L-B10 evaluation (50) | FULL-STACK | L | after L-B0..3 |
| 12 | L-B11 payroll FE (52) | FE | L | after L-B0..3 |
| 13 | L-B12 payroll pay-table BE | BE | M | parallel w/ L-B11 |
| 14 | L-B13 field (55) | FE | M | after L-B0..3 |
| 15 | L-B14 directory (55) | FULL-STACK | M | after L-B0..3 |
| 16 | L-B15 board (56) | FE | M | after L-B0..3 |
| 17 | L-B16 inventory (58) | FE | L | after L-B0..3 |
| 18 | L-B17 org (58) | FULL-STACK | L | after L-B0..3 |
| 19 | L-B18 maintenance (62) | FE | M | after L-B0..3 |
| 20 | L-B19 recruiting (78) | FULL-STACK | M | after L-B0..3 |
| 21 | L-B20 notif (76) | FE | S | after L-B0..3 |
| 22 | L-B21 identity/UsersPage grammar port | FE | S | after L-B0..3 |
| 23 | L-B22 docs records-registry BE | BE | L | parallel |
| 24 | L-B23 docs + dispatch dual-impl reconciliation | DECISION+FE | M | after L-B0..3 |
| 25 | L-B24 attendance FE (50) | FE | M | **HELD** on attendance writer |
| 26 | L-B25 attendance schedule/cover BE | BE | L | **HELD** on attendance writer |

---

## 2. Universal DoD clauses (every lane; not repeated per lane below)

1. **Window contract paragraph from §0 pasted verbatim into the lane's PR body and honoured.**
2. **Three unit assertions** per ported surface, in jsdom under `<WindowManagerProvider>` (helper
   `renderWithWindowManager` shipped by L-B0; shape derived from `modules/moduleEngine.test.tsx:51`):
   - (a) the row/chip renders with `[draggable="true"]` **and** the expected `data-obj-code`;
   - (b) activating the row sets `pinnedId` to the descriptor id;
   - (c) `getByRole("button")` resolves the drag host — i.e. it is a focusable `<button>`, never a
     `span`/`li`/`article` with `objDrag` spread on it.
3. **Collision roots are manifests, never edits.** `web/src/i18n/ko.ts`, `console/shell/nav.ts`,
   `console/screens/registry.ts`, `backend/openapi/openapi.yaml`, `clients/**`,
   `backend/app/src/lib.rs` (beyond appended register lines), `backend/app/src/objects.rs`,
   `backend/crates/platform/db/migrations/**` → emit
   `docs/evidence/console/CAP-<CAP>/frontend|backend/manifests/*.json` for the integrator.
4. **Truthfulness.** No fabricated rows, totals, codes, relations, or statuses. Honest-empty pattern
   is `console/leave/model.ts:199-203` (`relations: []` + a `wire-pending` comment). Any deferral is a
   named line in the lane's gap manifest with its fidelity-register anchor
   (`registers[module].findings[n]`), never silence and never a disabled control.
5. **No dead affordance.** A control that cannot act is omitted (deny-by-omission), not disabled.
6. **A11y AA**: every icon-only control has a Korean accessible name; status = text chip, not colour;
   informational chips are not focusable. `npx vitest run <lane files>` + `pnpm -C web tsc --noEmit`.
7. **Backend lanes additionally**: RLS `FORCE` and tested as `mnt_rt` (never superuser), deny-by-default
   PBAC, audit row on every mutation, canonical error envelope `{error:{code,message}}`, idempotency
   key on every POST, one story-level integration test, `BUCK` target present and building.
8. **No stubs.** No `TODO`, no `test.skip`/`.only`, no unimplemented branch. `ponytail:` comments are
   allowed only where they name a ceiling and an upgrade path.
9. **Push discipline**: `git fetch && git merge origin/<spine>` (plain merge — rebase is
   classifier-blocked); re-check migration high-water immediately before push.

---

## 3. Lanes

### L-B0 — Window host: decision, mount into the `/console` shell, shared test helper
**FOUNDATION / DECISION — nothing fans out before this lands.**

Why: §0. `/console` — the shell hosting all 15 module bodies — has no window provider, so every
"pin" in every ported module would silently be a modal. Closes the root cause of the window-model
invariant violation cited by **9 of 15 registers** (payroll blocker 1, attendance blocker 1,
evaluation blocker 1, org blocker 1, docs major, logistics major, plus equipment/inventory/board
detail-pane findings).

Scope:
1. Mount `WindowManagerProvider` in `console/shell/ConsoleShell.tsx`, wrapping the screen-body slot
   and the shell dock, with `authorityPartition` = the console session authority key (mirror
   `components/shell/AppShell.tsx:167-176`; `renderTray` decision follows whether the console shell
   already owns a dock — if it does, host `TrayDock` there, else `renderTray` on the provider).
2. Verify pin geometry against the console grid: the provider reserves body padding
   (`WindowManager.tsx:324-330`) and clamps 360–620px desktop / 42vh below 1024px
   (`windowModel.ts:24-28`). The console shell's own grid must not double-reserve.
3. Ship `web/src/console/window/testing.tsx` exporting `renderWithWindowManager(ui, opts)` returning
   `{ ...renderResult, windowManager }` so every downstream lane can assert `pinnedId` without
   re-deriving the harness.
4. Write `docs/program/console-window-target-decision.md`: the three systems table from §0, the
   decision, the binding contract paragraph, and the explicit deferred list (popout, split/second
   pin, named presets, `saveLayout`/`restoreDefault` — the last two are a **dead context API**, only
   callers are `AppShell.test.tsx:138,199`, so the recipe must not tell lanes to document them).
5. Register **L-B0b** (not built in wave 4) in that doc: promote/replace with a 4-state provider
   behind the same context, plus the §5.2 window-a11y contract and Playwright keyboard journeys.

Roots: `web/src/console/shell/ConsoleShell.tsx`, `web/src/console/shell/ConsoleShell.test.tsx`,
`web/src/console/window/testing.tsx`, `docs/program/console-window-target-decision.md`.
Must not touch: `components/shell/**`, `features/workspace/**`, `console/window/{useWindowEngine,
WindowEngine,types,geometry,sanitize}.ts*`, any module dir.
Note `console/shell` is a hot dir (57 path-hits/48h) — run this lane **alone**, plain-merge, push fast.

DoD:
- `console/shell/ConsoleShell.test.tsx`: rendering a screen body that calls
  `useOptionalWindowManager()` gets a **non-null** manager; `open()` then `pinnedId === entry.id`;
  the pinned panel is `role="region"` with `aria-labelledby`; below 1024px it is the 42vh sheet.
- Regression: `console/screens/leave/LeaveBody.test.tsx` (or a new case) proves `LeaveConsole` row
  activation now pins in the `/console` tree instead of falling back to the modal.
- `renderWithWindowManager` is consumed by at least one existing test, proving the export shape.
- `npx vitest run web/src/console/shell web/src/console/window` green; `tsc --noEmit` green.
- The decision doc names every deferred capability with its fidelity-register anchor.

---

### L-B1 — Code-grammar unification: one dynamic source, four call sites
**FOUNDATION.**

Why: `scout-shared-grammar-b.md` §2. `composer/grammar.ts:36`
`BARE_CODE_RE = /(^|[\s([{])([A-Z]{1,8}-[0-9]{1,10}(?:-[0-9]{1,6})?)/gu` never imports `codeGrammar`
and diverges in two directions: digit-only bodies (so `OT-FINANCE`, `PAY-CHO`, `EV-2026-00012` —
all three asserted round-tripping through objDrag at `window/objDrag.test.ts:78-90` — do **not**
tokenize as `codeLink` in composer text) and uppercase-only prefixes (`Bid` is in
`FALLBACK_CODE_PREFIXES` and can never match). `objectCodeBodySource()` is exported at
`codeGrammar.ts:92` "for consumers that build their own combined pattern" and has **zero callers**.
Fixing this removes 3 of the 4 per-module code edits and the worst 4-file collision before 13 lanes
hit it.

Scope:
1. Rebuild `BARE_CODE_RE` from `objectCodeBodySource()` (recompile on `primeCodePrefixes`, do not
   cache a stale RegExp across a prime).
2. `composer/objectKinds.ts` `KIND_META` and `objectcard/kinds.ts` `SLUG_META` /
   `COMPOSER_KIND_TO_SLUG`: keep tone/label as data, derive the **prefix set** from `codeGrammar`
   so a newly-registered prefix is parseable without an edit. `COMPOSER_KIND_TO_SLUG` currently has
   5 entries (`kinds.ts:50-56`) — a kind absent there is **not linkable** (`linkTargetFromCode`
   returns `undefined`), which is the deny-by-omission behaviour to preserve, not to widen blindly.
3. Extend `FALLBACK_CODE_PREFIXES` (`codeGrammar.ts:16-19`) with the offline floor for the prefixes
   L-B3 confirms exist today. Adding a prefix here is a one-literal change; it is **not**
   authorization (`codeGrammar.ts:11-13`).

Roots: `web/src/console/composer/{grammar.ts,objectKinds.ts}` + their tests,
`web/src/console/objectcard/kinds.ts` + test, `web/src/console/ontology/codeGrammar.ts` + tests.
Must not touch: any module dir, `objDrag.ts`, `messengerModel.ts`, `appr/composeModel.ts` (they
already consume `codeGrammar` correctly).

DoD:
- New tests: `OT-FINANCE`, `PAY-CHO`, `EV-2026-00012`, `Bid-77` each (a) round-trip through
  `objDrag`/`parseObjectRef` **and** (b) tokenize as `codeLink` through `parseTokenGrammar` — one
  table-driven test asserting parity between the two parsers on the same corpus.
- `primeCodePrefixes(["ZZ"])` then `parseTokenGrammar("ZZ-1 참조")` yields a `codeLink` with no
  source edit; `resetCodePrefixes()` restores the fallback floor (fail-closed union semantics hold).
- An unregistered prefix (`COVID-19`) still parses inert / renders unlinked — no widening of
  authorization.
- `npx vitest run web/src/console/{composer,objectcard,ontology}` green.

---

### L-B2 — Shared drag/modal a11y, before 13 lanes copy the pattern
**FOUNDATION.**

Why: `scout-shared-grammar-b.md` §1. The drag hosts in the two files the recipe names as exemplars
are non-focusable, have no `onClick`, no `tabIndex` — zero keyboard or AT path:
`objectcard/ObjectCard.tsx:779` (`<span {...objDrag(...)}>` **inside the shared card**, so every
drilling module inherits it), `explore/ObjectExplorerScreen.tsx:358` (span), `:464` (`li`), `:526`
(`article`). The correct pattern is in-repo at `modules/GenericModuleScreen.tsx:718-733`
(`<button type="button" {...objDrag(...)} aria-label=… >` inside `PolicyGated`) and stated verbatim
at `configconsole/DashboardEditor.tsx:136`. Fixing it once is cheaper than rejecting it 13× at merge.

Scope:
1. `ObjectCard.tsx:779` span → `<button type="button">` with a Korean `aria-label`, ≥44px target,
   click = copy/drill per the existing card handler contract (no new behaviour invented — if the
   card has no click semantic for the code, the button's action is the descriptor's own open/drill).
2. `ObjectCardModal` (`ObjectCard.tsx:895-929`): add a focus trap. **Verify current state first** —
   `autoFocus` is already on the close button in this worktree, so "no initial focus" is stale; the
   missing piece is tab containment and focus return to the invoker on close (APG dialog rules).
3. Same span→button conversion at `ObjectExplorerScreen.tsx:358,464,526`.
4. Add the rule to the port recipe text in `docs/program/console-window-target-decision.md`
   (created by L-B0) so lanes copy the rule, not the exemplar pointer.

Roots: `web/src/console/objectcard/ObjectCard.tsx` + `ObjectCard.test.tsx`,
`web/src/console/explore/ObjectExplorerScreen.tsx` + test.
Must not touch: `objDrag.ts` (the token contract is correct), module dirs.

DoD:
- `ObjectCard.test.tsx`: the code chip is `getByRole("button")`, has a Korean accessible name, and
  still carries `draggable="true"` + `data-obj-code`.
- Modal test: Tab from the last focusable wraps to the first (containment); Escape closes from
  anywhere inside; focus returns to the element that opened it.
- `ObjectExplorerScreen.test.tsx`: all three converted hosts resolve via `getByRole("button")`.
- `npx vitest run web/src/console/{objectcard,explore}` green; axe clean on the card + modal.

---

### L-B3 — Object-code triage and per-module port-path assignment
**FOUNDATION / ANALYSIS — no product code.**

Why: `scout-shared-grammar-b.md` §3 closing paragraph. `objDrag(code, …)` needs an issued code and
`parseObjectRef` re-validates it against `objectCodeRegex()` before accepting a typed payload
(`objDrag.ts:66`) — **a UUID-keyed row cannot participate at all**. Sampling on the branch shows the
split is real: `evaluation` has `rv_code` (`evaluationApi.ts:66,151`), `maintenance`/`dispatch`/
`inventory` have `request_no`, `equipment` keys on `serialNo`, `logistics` on `sku` + server UUIDs,
`directory` on member/employee UUIDs with no code at all. Chartering a code-less module as an FE
lane guarantees a mid-wave stall.

Scope: for each of the 15 modules, read its `*Api.ts` row/summary types and the screen's row key and
record: (a) does every listed row carry a server-issued, `objectCodeRegex()`-valid code? (b) if not,
is a client-derivable code legitimate (e.g. `field`'s `ticketCode` derivation) or is it BE issuance
(`L-B4`)? (c) port path — **LeaveConsole descriptor-mapper** (`leave/model.ts:164` `ledgerDescriptor`,
≈55 LOC; total ≈120-200 LOC + tests, ~1 lane-day) for large bespoke bodies that are already correct,
vs **ModuleScreenConfig + ModuleDataAdapter rewrite** (`moduleScreens.ts:479-608` asset exemplar,
2-3 lane-days) for thin bodies with a live list endpoint. (d) which `FALLBACK_CODE_PREFIXES` literals
L-B1 must add.

Roots: `docs/program/console-object-code-triage.md` (new) + a machine-readable
`docs/program/console-object-code-triage.json` consumed by the module lanes.
Must not touch: any source file.

DoD:
- All 15 modules classified with file:line evidence for the row-key claim.
- The FE-portable / BE-code-blocked split is explicit, and every BE-blocked module names the
  L-B4 deliverable it waits on.
- Port path assigned per module with the LOC/day estimate and the exemplar file:line to copy.
- The `FALLBACK_CODE_PREFIXES` delta list is handed to L-B1 (L-B1 may land first with the prefixes
  already verifiable; the rest arrive as a one-literal follow-up in the owning module lane's manifest).

---

### L-B4 — Object-code issuance kernel + object-type registry `code_prefix` seed
**CROSS-CUTTING BACKEND.**

Why: the single highest-leverage BE-blocked item — it unblocks `objDrag` for every code-less module.
Registers: `payroll` finding "Issue a display code (PS-YYMM style) on the run contract";
`logistics` "human-readable code series (ASN-/SH- numbering) needs backend allocation";
`equipment` "true code issuance (EQ-/RC- display codes) needs a backend contract field";
`field` site codes; `directory` (no code at all). FE side is already dynamic: `primeCodePrefixes()`
(`codeGrammar.ts:73-82`) is fed by the bootstrap `GET /api/v1/object-types`, so a registry row with
`code_prefix` makes a new code drag/parse-able with **zero FE edits**.

Scope: a reusable, gap-free, per-tenant code-issuance primitive in the **quiet** `registry` crate
(7 path-hits/48h): sequence allocation (advisory-lock or sequence-per-(org, prefix), never
`max()+1`), format contract (`PREFIX-YYMM-NNNN` vs `PREFIX-NNNNN` decided once and documented),
idempotent re-issuance, and the object-type registry rows carrying `code_prefix` for every kind
L-B3 flags. Domain adoption is each domain lane's job — this lane ships the kernel plus the seed.

Roots: `backend/crates/registry/**`, migration slot **0210** (`0210_object_code_issuance.sql`),
`docs/program/console-object-code-contract.md`.
Must not touch: any other crate, `backend/app/src/lib.rs` beyond appended register lines,
`openapi.yaml` (manifest), `console/**`.
Migration slot: 0210 (provisional; integrator renumbers).

DoD:
- Concurrency test: N parallel issuances for the same (org, prefix) produce N distinct, gap-free codes
  (run as `mnt_rt`, not superuser).
- RLS `FORCE` on the sequence table; a cross-org read as `mnt_rt` with `app.current_org` armed to org B
  returns zero rows for org A.
- Idempotency: re-issuing for the same (kind, entity id, idempotency key) returns the same code.
- Every issued prefix appears as an object-type registry row with `code_prefix`, and a REST test of
  `GET /api/v1/object-types` shows it, so `primeCodePrefixes` picks it up with no FE change.
- Audit row on issuance. `BUCK` target present and building.

---

### L-B5 — Object typeahead / search fabric (§4-27-4)
**CROSS-CUTTING BACKEND. Codex-collision: coordinate with `codex/console-search-object-fabric-20260724`
before starting — if that lane owns this contract, this lane becomes an adoption-only FE follow-up.**

Why: `§4-27-4` ("개체·인물 선택은 전량 열거 칩이 아니라 타입어헤드") is cited as BE-blocked by **five**
registers: `inventory` blocker (WO/dispatch typeahead, free-text today), `evaluation` major
(cross-object code search for `!CODE` refs), `maintenance` major (mechanic + equipment pickers —
"the raw-ID input cannot ship as the assignment UX"), `equipment` major (customer/site selectors —
"the contract only accepts strings"), `org` (entity picker). One PBAC-filtered search surface closes
all five.

Scope: `GET /api/v1/objects/search?kind=&q=&limit=` returning `{code, title, kind, id}` ranked, hard
PBAC-filtered (a code the caller cannot read is **absent**, not marked forbidden), Korean
initial-consonant / substring matching, stable pagination, and a `kinds=` multi-filter. Plus the
shared FE `<ObjectTypeahead>` primitive that consumes it and emits a validated typed ref.

Roots: `backend/crates/ontology/search/**` (or the crate the coordination decides),
`web/src/console/components/ObjectTypeahead.tsx` + test, migration slot **0211** if an index is needed.
Must not touch: module dirs, `objects.rs`, `openapi.yaml` (manifest).

DoD:
- Query as `mnt_rt` for org A never returns an org-B object; a PBAC-denied kind returns zero rows
  (deny-by-omission), verified in an integration test with two orgs.
- Korean substring + 초성 query returns the expected ranked set; empty query returns the
  recent/suggested set, not everything.
- `<ObjectTypeahead>` is combobox-role, keyboard-operable (↑↓/Enter/Escape), announces result count
  via `role="status"`, and emits only validated `{kind, id, code}` — free text cannot escape it.
- Story test: pick a work order in a maintenance-style form → the stored value is the typed ref.

---

### L-B6 — Operational object runtime read surface (full card + links + dynamics + series)
**CROSS-CUTTING BACKEND. Codex-collision: the spine branch is literally
`codex/operational-object-runtime-progress` — check what exists before building.**

Why: the most-cited BE gap across the registers — the 3-layer object card (semantic / kinetic /
dynamic, DESIGN §4-14) has no backing read surface. `dispatch` major: "ObjectHead only carries
code/title/status"; `equipment` blocker: "object-card affordance and dynamics strip … needs backend
surfaces"; `docs` major: EV- linked-object refs; `directory` major: dynamics footer + object card;
`board` major: linked-object references; `field` majors ×2 (link chips + automation rules);
`inventory` majors ×2 (dynamics row + object-link resolver); `maintenance` major (series + automation
chips); `notif` major (rowTarget → object surface).

Scope: per resolvable kind — (a) full card read: properties, relations, lifecycle history, audit;
(b) object-links read (typed edges both directions, PBAC-filtered); (c) acting-automations read
(which rules touch this type/instance); (d) series read (`SR-` parent series with member timeline).
All four are reads; no mutation in this lane.

Roots: `backend/crates/ontology/**` (card/links/dynamics/series read modules) + REST,
migration slot **0212** if projections are needed. `backend/app/src/objects.rs` is a **serialized
surface** — emit a manifest for the kind-registration lines, do not edit.
Must not touch: `objects.rs`, `lib.rs` beyond appended register lines, `openapi.yaml` (manifest).

DoD:
- Each of the four reads returns real rows for at least two kinds, tested as `mnt_rt` with
  `app.current_org` armed; a cross-org id 404s (not 403 leaking existence).
- Relations respect PBAC per-edge: an edge to an unreadable object is omitted, and the test asserts
  the omission rather than a masked placeholder.
- An object with no automations returns `[]` and the FE contract test proves the card renders the
  honest-empty state, not a fabricated chip.
- `GovernedObjectCard` (`objectcard/wired.tsx:229-312`) consumes the new reads in one wired test.
- `BUCK` target present and building.

---

### L-B7 — Approval-compose prefill contract (the modules' primary CTAs)
**CROSS-CUTTING BACKEND + thin FE.**

Why: four registers name the same missing surface as the module's **primary action**:
`dispatch` major (「배차 요청 기안」 → approval compose prefill), `field` major (「연장 계약 기안」),
`inventory` major (「긴급 구매 기안」 / 「PO 자동 제안」), `equipment` major (「취득 기안」 —
"§3.9.0 declares un-whitelisted direct saves a design violation"). Today equipment registers assets
by direct save, which is the violation.

Scope: a typed prefill contract — `{ approvalKind, title, prefilledFields, sourceObject }` — that a
module CTA hands to the approval composer, plus the composer entry point that consumes it and the
server-side validation that the prefill's source object is readable by the actor. Draft→approve
lifecycle for the four kinds above.

Roots: `backend/crates/workflow/compose/**` (or `approval` per the spine's naming),
`web/src/console/appr/composePrefill.ts` + test, migration slot **0213**.
Must not touch: module dirs (each module lane wires its own CTA against this contract),
`openapi.yaml` (manifest).

DoD:
- Prefill round-trip: module CTA → composer opens with the exact fields → submit creates the draft
  with `source_object` recorded → audit row.
- Prefill referencing an object the actor cannot read is rejected server-side (deny-by-default),
  tested as `mnt_rt`.
- Idempotency key honoured: double-submit yields one draft.
- Contract test consumed by at least one module lane's CTA test (equipment 취득 기안).

---

### L-B8 — equipment (fidelity 40, lowest) — full-stack
Why: lowest score of 15; 2 blockers, 7 majors; 5 BE-blocked. Register `registers[equipment]`.

Scope (FE): drag sources on both row lists and detail headers via `objDrag` with
`[<serialNo> <modelName>]` / `[<caseId-label> <customer>]`; search over all visible attributes;
J/K keydown over the filtered list; shared-track grid (serial · model/capacity · availability) with
tri-state sortable headers; "N 표시 / 전체 M" count framing; the shared module-config strip (custom
columns from detail kv, stat add, filter presets — client-persisted); KRW input mask (digits-only +
live thousand separators, ₩ affordance) on all four money fields; object-card entry + dynamics strip
via L-B6 (honest-empty until it lands); customer/site typeahead via L-B5 replacing string inputs.
Scope (BE): `EQ-`/`RC-` display-code issuance adopting L-B4; draft→approve acquisition via L-B7
(removing the direct-save violation); disposition REPAIR → WO- link + finance posting readback,
or a named gap-manifest entry if the workorder edge is out of reach this wave.

Roots: `web/src/console/equipment/**`, `backend/crates/equipment/**`, migration slot **0214**,
`docs/evidence/console/CAP-EQUIPMENT-3R-PILOT/**`.
Must not touch: shared collision roots; other module dirs. Note this CAP is the **only** registry row
with non-empty `buck2_targets` — keep them green.

DoD: universal clauses + the three window assertions on both row lists; KRW mask test (typing
`1234567` renders `1,234,567`, submits `1234567`); sort cycle test asc→desc→none; acquisition draft
test proves no direct save path remains; `mnt_rt` RLS test on the new code column.

---

### L-B9 — logistics (42) — full-stack, ModuleScreenConfig path
Why: score 42; 1 blocker, 6 majors; the FE is "honestly built around the gap" — the single BE finding
is the whole read surface. Register `registers[logistics]`.

Scope (BE): `GET` list endpoints for ASNs / fulfillments / shipments (+ stock projection) scoped by
branch, replacing the session working set with server truth; short-form server ids exposed as mono
codes now, `ASN-`/`SH-` series via L-B4.
Scope (FE): express the three queues as `ModuleConfig` specializations of the shared `ModuleScreen`
with a load adapter (this is the **ModuleScreenConfig path** — copy `assetModuleScreen`
`moduleScreens.ts:479-608` and `ModuleFinanceScreenBody` for the `PolicyGateProvider` wrapper; the
R4 trap: no ambient gate ⇒ DENY_ALL ⇒ blank plane); swap `logistics__chip` for `StatusChip`; J/K +
header sort + count framing come free with the adoption; `objDrag` on rows and detail header codes;
each stat chip sets its queue's filter; flow stepper per detail from the known enum chain with the
existing conditional forms as the single current-step CTA.

Roots: `web/src/console/logistics/**`, `backend/crates/logistics/**`, migration slot **0215**,
`docs/evidence/console/CAP-LOGISTICS-PILOT/**`. `MOD_SCREENS` entry → manifest.
Must not touch: `modules/moduleScreens.ts` (manifest), other module dirs.

DoD: universal clauses + the three window assertions; `mnt_rt` RLS test proving branch scoping on all
three list endpoints; adapter mapper test (server row → `ModuleRow`); a config-shape test mirroring
`FinanceModuleScreen.test.tsx`; stepper renders `aria-current="step"` with state in **text**.

---

### L-B10 — evaluation (50) — full-stack
Why: 2 blockers, 7 majors. `rv_code` already exists (`evaluationApi.ts:66,151`) so this module is
immediately FE-portable. Register `registers[evaluation]`.

Scope (FE): descriptor mapper for subjects/ledger entries (LeaveConsole path) + row activation → pin;
`objDrag` on `RV-` chips and object rows; make each stat a button filtering subjects by state and
each team-progress row a button filtering to that org_unit (client-side over loaded data — no new
fetch); `ATTENDANCE: "att"` added to `EVIDENCE_SCREEN`; manager typeahead via L-B5; result toast on
mutation naming the `RV-` code / new stage / calibrated grade with the drill path; multi-attribute
search + J/K/Enter + "N 표시 / 전체" chips from the returned totals.
Scope (BE): per-subject cross-module context endpoint (attendance summary, recent completed tasks,
KPI) returning drillable typed refs — the auto-attached context chips; cycle-kind N+1 extensibility
or a documented closed-enum exemption.

Roots: `web/src/console/evaluation/**`, `backend/crates/evaluation/**`, migration slot **0216**,
`docs/evidence/console/CAP-EVALUATION-CONSOLE/**`.
Must not touch: shared collision roots. **This crate is missing `BUCK` files**
(`evaluation/adapter-postgres`, `evaluation/rest`) — the lane must not hand-write them; it flags the
generated-face regeneration (`tools/buck/gen_first_party.py`) to the integrator in its manifest.

DoD: universal clauses + three window assertions; stat-button filter test (click 진행중 → list shows
only that state, count matches); context endpoint returns real cross-module rows as `mnt_rt` and
fabricates nothing when a source module has no data (`[]` + honest-empty chip); toast test asserts the
`RV-` code appears in the message.

---

### L-B11 — payroll FE (52)
Why: 1 blocker, 6 majors, and an SoD defect. Register `registers[payroll]`.

Scope: mount the four pay cards' pin/drill through the L-B0 provider (**popout / split / 4-preset
menu do not exist — gap-manifest them against `registers[payroll].findings[0]`**, do not build a
bespoke card engine); `objDrag` on run rows, roster rows, and ref chips emitting the standard
`[코드 제목]` payload; **hide (not disable)** the 승인/반려 decision block when
`run.submitted_by === actorId` (the field exists at `payrollApi.ts:37`) — deny-by-omission, server
check stays as backstop; pass object identity through cross-module navigation (route param /
`?code=AT-1042` / open the shared object card) instead of landing on bare module roots; make
`calc.kernel_rate_table` (`PayrollScreen.tsx:1217`) either a real drill or remove the dead mono chip;
empty state names reason + next action.

Roots: `web/src/console/payroll/**` (FE only), `docs/evidence/console/CAP-PAYROLL-CONSOLE/frontend/**`.
Must not touch: `backend/crates/payroll/**` (that is L-B12), other module dirs.

DoD: universal clauses + three window assertions on run rows and roster rows; SoD test — render as
the submitter, assert the decision block is **absent from the DOM** (`queryBy… === null`), not
disabled; navigation test asserts the target receives the code; the dead chip is gone or drills.

---

### L-B12 — payroll pay-table BE depth
**`param_verify_live: true`** — the 공제 breakdown is 4대보험 + 소득세. Any rate this lane needs must be
cited from `scratchpad/wave4/research-statutory-params.md` §2 / §8 (live-sourced), never from model
memory. If the payroll engine does not already compute a component, this lane **exposes nothing for
it** and files it to lens D — it must not invent a rate to fill a column.

Why: 5 BE-blocked findings in `registers[payroll]` — the roster is not a pay table without them.

Scope: extend the line DTO with `base / allowance / deduction / net` (+ prior-run delta) so the roster
renders as masked-by-default mono amount columns with delta tone; per-entity totals + prior-run delta
+ employer burden on the run calc summary (the 지급 총액 bars, each drilling to the filtered roster);
a decision-due timestamp on the run contract (header deadline chip); `PS-YYMM` display code via L-B4;
`SR-` series read for 회차 시리즈 (or a named gap-manifest entry deferring to L-B6's series read).

Roots: `backend/crates/payroll/**`, migration slot **0217**,
`docs/evidence/console/CAP-PAYROLL-CONSOLE/backend/**`. `openapi.yaml` → manifest.
Must not touch: `web/src/console/payroll/**` (that is L-B11).

DoD: universal BE clauses; every exposed amount traces to an engine-computed value with a test naming
its source (no derived-in-the-DTO arithmetic); masked-by-default is enforced server-side for callers
without the unmask grant, tested as `mnt_rt`; totals are `null` unless every line calculated (preserve
the existing "never a partial sum shown as a total" rule at `payrollApi.ts` `RunCalcSummary`); any
statutory rate used is cited inline with its source URL and effective date.

---

### L-B13 — field (55)
Why: 1 blocker, 7 majors; 7 BE-blocked but nearly all resolve to **gap-manifest entries**, so the FE
lane is unblocked today. Register `registers[field]`.

Scope: restore the site code as the mono first column (client-derived `ticketCode`-style until L-B4
issues canonical codes — L-B3 rules on which); `objDrag` on rows, issue items, and WO chips via the
single helper; client tri-state sort (ko `localeCompare` / numeric extraction) with header spans
converted to buttons; extend the existing `storeKey` store with the personal-view config (custom
columns from detail-field mapping, SLA/customer count stats, search presets, dist widget);
"N 표시 / 전체 M" framing from `page.total` with server totals for the total stat; link chips to
`dispatch` **now**, and map/graph/거래처/mail/contract/JL- as named gap-manifest entries with their
register anchors.

Roots: `web/src/console/field/**`, `docs/evidence/console/CAP-FIELD-CONSOLE/**`.
Must not touch: `backend/crates/facilities|workorder/**`, other module dirs.

DoD: universal clauses + three window assertions; sort test covers Korean collation and numeric
columns; the gap manifest lists all 6 deferred link targets with register anchors; count framing test
distinguishes filtered vs total.

---

### L-B14 — directory (55) — full-stack
Why: 1 blocker, 4 majors; 5 BE-blocked; and it is **UUID-keyed with no code**, so it is the canonical
L-B3 "BE-code-blocked" case. Register `registers[directory]`.

Scope (BE): add `ext` / `phone` / `email` to `MessengerMemberSummary` and `Employee` (or a
directory-profile endpoint) with PBAC-safe field-level gating; enrich the non-privileged roster with
title/company/ext so both tiers render one column set; **server-side** `sort` param on
`GET /api/v1/employees` (client-side sort of a partial page would be untruthful).
Scope (FE): 연락처 as the lead mono column + 이메일 kv on the card; header cycling on all five columns
against the server sort; the personal-view cfg strip (columns from `Employee` fields, stCount stats,
saved query presets — session-persisted); drag/pin only once a code exists (L-B4) — until then the
lane ships the columns and **files the drag work as blocked**, it does not fake a code from a UUID.

Roots: `web/src/console/directory/**`, `backend/crates/identity/directory/**`, migration slot **0218**,
`docs/evidence/console/CAP-DIRECTORY-CONSOLE/**`.
Must not touch: `backend/crates/identity` auth/session paths; other module dirs. `identity` is tier-2
hot (22 hits/48h) — plain-merge before push.

DoD: universal clauses; field-level PBAC test as `mnt_rt` (a caller without the contact grant gets the
field **absent**, not masked-empty); server sort test proves ordering is stable across pages; the
drag/pin deferral is a named gap-manifest line, and no test asserts a UUID as a code.

---

### L-B15 — board (56)
Why: 1 blocker, 6 majors; only 2 BE-blocked → almost fully FE-closable. Register `registers[board]`.
Note the blocker is in the **shared `ListTable`** (single definition per §4-18) — this lane therefore
owns a shared file and must run without a concurrent sibling in the same file.

Scope: `draggable` + `onDragStart` `text/plain "[<code> <rowTitle>]"` in the shared `ListTable` row
(`config.rowId`/`rowTitle` already exist) — implemented as `objDrag`, not a bespoke handler; tri-state
sort on the shared header (numeric-aware `keyOf`, arrow suffix on the active column); `openRow`
follows a single `selectedId` defaulting to the first filtered row, × remains an explicit dismiss;
detail header gains config-provided mono draggable code + status `ModuleCell`; per-row `prog` slot
(done/total → existing `ProgBar`) for manager callers with the kv line kept for non-managers; the
config strip ported to the shared `ModuleScreen` as personal-view state.

Roots: `web/src/console/board/**`, plus the shared `ListTable`/`ModuleScreen` files it must edit —
**declare them explicitly in the lane's root list and hold every other module lane off those files
for its duration** (the sort/selection/config-strip work that other registers request lands here once).
Must not touch: `modules/GenericModuleScreen.tsx` (different surface), other module dirs.

DoD: universal clauses + three window assertions **through the shared ListTable** (so every consumer
inherits them); sort cycle test; selection-default test; a consumer-regression test proving an existing
`ListTable` user (another module) still renders unchanged.

---

### L-B16 — inventory (58)
Why: 3 blockers, 5 majors. Register `registers[inventory]`. Two blockers are pure FE.

Scope: remove `overflow-x` from the list wrapper; minmax column behaviour with ellipsis and
drop 보관/수량 below a width threshold (container query or `ResizeObserver`) per the modNarrow grammar,
keeping `overflow-x:auto` only on `.inventory__mrp-table`; implement the personal-view config strip
(column add from `InventoryItemView` fields, stat add from status/location counts, saved search
presets, dist-bar widget, det-mode segment — all client-persistable like the existing sessionStorage
state; 팀 배포 gated behind future appr integration, i.e. **omitted**, not disabled);
"N 표시 / 전체 M" chip when `page.total > items.length` (the API already accepts `offset`);
WO/dispatch typeahead via L-B5 replacing the free-text ref; remove the inert `OT-17` chip now
(dead affordance, §4-16) and reintroduce it as a real type-card button when L-B6 lands;
「긴급 구매 기안」 CTA via L-B7.

Roots: `web/src/console/inventory/**`, `docs/evidence/console/CAP-INVENTORY-CONSOLE/**`.
Must not touch: `backend/crates/inventory/**` (tier-1 hot, 35 hits — file BE items to the gap
manifest), other module dirs.

DoD: universal clauses + three window assertions; a jsdom width test proves the responsive column drop
without horizontal page scroll; personal-view state survives a remount (sessionStorage round-trip);
the inert chip is gone (assert absence).

---

### L-B17 — org (58) — full-stack
Why: 2 blockers, 5 majors; **7 BE-blocked** — the heaviest BE ratio of any module. `OC-` codes already
exist on the changes strip, so the drag blocker is closable today. Register `registers[org]`.

Scope (FE): replace the bespoke `CardShell` modal with the shared `ObjectCard` +
`objectCardWindowEntry` entry (delete the bespoke modal — deletion over addition); `objDrag` on the
`OC-` code chips first, then entity/team surfaces as codes arrive; carry entity/team/person identity
through `consoleScreenPath` navigation and open the shared person card directly for head/employee
clicks; restore per-site disclosure with the `+N` team-count chip and spine styling.
Scope (BE, `orgchange` — a **quiet** tier-3 crate, 7 hits, good landing zone): branch↔org_unit linkage
+ per-branch headcount; entity profile (설립/사업자번호/소재지/관할) + PBAC-gated finance summary with
a view-audit event on expand (deny-by-omission section per §4.5); org-change op vocabulary extensions —
site-scoped team reassignment, dissolve-org-unit for `total === 0`, entity rename, `reason_kind` enum
(curated presets + 직접 입력 N+1) with detail text.

Roots: `web/src/console/org/**`, `backend/crates/orgchange/**`, migration slot **0219**,
`docs/evidence/console/CAP-ORG-CONSOLE/**`.
Must not touch: shared collision roots. **`orgchange` is missing 3 `BUCK` files**
(`adapter-postgres`, `domain`, `rest`) — flag the generated-face regeneration to the integrator; do
not hand-write them.

DoD: universal clauses + three window assertions on the `OC-` chips; the bespoke `CardShell` file is
**deleted**, not left orphaned; view-audit test proves expanding the finance section writes an audit
row as `mnt_rt`; the `hc > 0` team-removal guard still fail-closes with a forbid audit while `hc === 0`
enqueues a counted proposal; reason enum rejects an unlisted value unless it arrives via the
직접 입력 path, which is recorded as such.

---

### L-B18 — maintenance (62)
Why: 1 blocker, 4 majors; `request_no` already exists so the blocker is a one-line fix.
Register `registers[maintenance]`.

Scope: replace the inline drag handler with `{...objDrag(row.request_no, equipmentLabel(row))}` from
`../window/objDrag` on the row button **and** the lane cards (currently not drag sources at all);
adopt the shared module-config strip (client-state custom columns from detail fields, count-stat add,
saved search presets); wire 자산 이력 to the asset route with the equipment preselected; make the 전표
flow step and settlement `voucher_ref` chips drill to the finance voucher; mechanic/equipment pickers
via L-B5 (the raw-ID input cannot ship); part-reservation `PO-` drills and the 시리즈/automation chips
are named gap-manifest entries (L-B6).

Roots: `web/src/console/maintenance/**`, `docs/evidence/console/CAP-MAINTENANCE-CONSOLE/**`.
Must not touch: `backend/crates/workorder/**`, other module dirs.

DoD: universal clauses + three window assertions on both row buttons and lane cards; asset-history
navigation test asserts the equipment id reaches the target; voucher chip drill test.

---

### L-B19 — recruiting (78) — full-stack
**`param_verify_live: true`** — pool registration and rejected-candidate handling sit on
채용절차법 결과 통지 / 서류 반환·파기 duties and PIPA consent + retention. Cite
`scratchpad/wave4/research-statutory-params.md` §6 and §7 for every deadline and retention period;
never a remembered number.

Why: highest score (78) but carries a **blocker**: the pool-registration terminal action.
Register `registers[recruiting]`.

Scope (BE): `POST …/applicants/{id}/register-pool` — OFFER-stage applicants on `POOL_DAILY` postings
terminate into a workforce-pool entry (`src=JP-`, `avail=본인 동의/즉시`) instead of an employee;
a pool-proposal endpoint for the rejected-candidate banner (`avail=본인 동의 대기`, `src=posting code`);
`applicant_id` + `posting_id` on the talent-pool payload; an original-document provenance read that
logs an audited view.
Scope (FE): the pool CTA + subrow next-action wired to the real endpoint (**until it exists, the gap
manifest — never a client-side fake**); 인력풀 등재 제안 on the rejected banner; talent-pool row click
opens the candidate card and the `APL-` chip becomes a drag source; screen-scoped keydown maintaining
`selectedId` with rowBg+ring independent of open, Enter toggles; derive `org_unit` from
`posting.worksite`, default `home_branch` from the posting's site mapping, backend issues the employee
number, phone is the only manual field (the form becomes a confirm sheet, not data entry); phone
digit+auto-hyphen mask and 사번 pattern mask; corporation list from the server (org-entity read, same
pattern as `listBranches`) with the dotted 「+ 직접 입력」 affordance on enum chip rows.

Roots: `web/src/console/recruiting/**`, `backend/crates/recruiting/**`, migration slot **0220**,
`docs/evidence/console/CAP-RECRUITING/**`.
Must not touch: shared collision roots. **`recruiting` has ZERO `BUCK` files** (all four crates) and
`backend/app/BUCK` has no recruiting deps at all — the lane must flag the
`tools/buck/gen_first_party.py` regeneration to the integrator as a blocking manifest item.

DoD: universal clauses; pool registration creates a pool entry and **no** employee, verified as
`mnt_rt` with the audit row; consent state is explicit on every pool entry (no implied consent);
retention/notification periods cite their statutory source inline; mask tests for phone and 사번;
the confirm-sheet test asserts only phone is editable.

---

### L-B20 — notif (76)
Why: highest-scoring module, 3 findings, no blockers — a genuinely small lane. Register
`registers[notif]`. Its one FE action (`MOUNTED_SCREEN_KEYS` + `SCREEN_REGISTRY` registration) has
**already landed** on this branch (verified: `screens/registry.ts:39,81`, `nav.ts` list) — so this
lane is the remaining three.

Scope: route `rowTarget {type:'object'}` and `openToken` to the object-card/pin surface through the
single chokepoint `notifModel.rowTarget` (now possible after L-B0), keeping the object-filter as the
fallback for unresolvable heads; give `TokenText` mentions an optional `onOpenPerson` fed from the
directory-scoped person provider; record in the gap manifest the count of notification screen-link
targets that are dark under `EXPOSED_SCREEN_KEYS = ["sales"]`, so exposure reviews see the dead-end
number (this is authority-sanctioned fail-closed behaviour per ADR-0025 — **no code change**, a
counted, named gap).

Roots: `web/src/console/notif/**`, `docs/evidence/console/CAP-NOTIF-CONSOLE/**`.
Must not touch: `composer/TokenText` internals beyond the optional prop; other module dirs.

DoD: universal clauses + three window assertions; a notification whose head resolves opens the pinned
object card, one whose head does not resolve falls back to the object filter (both asserted); the dark
link-target count in the gap manifest matches a test-computed number, not a guess.

---

### L-B21 — identity / UsersPage grammar port (audit-4-22-23 Finding 2)
Why: `scout-shared-grammar-b.md` §4 — `pages/UsersPage.tsx` / `console/identity/*` have **zero**
draggables on the one live-data surface the runtime audit could actually exercise. Same recipe, same
wave; and this surface sits under `AppShell`, where the provider already exists, so it is the
lowest-risk proof that the recipe works end to end.

Scope: `objDrag` + descriptor mapper + row activation → pin on the users list, following the
LeaveConsole path; drag host is a real button.

Roots: `web/src/pages/UsersPage.tsx`, `web/src/console/identity/**`, their tests.
Must not touch: auth/session logic, other module dirs.

DoD: universal clauses + the three window assertions; because this runs under a real provider on real
data, additionally capture a browser/E2E screenshot of a pinned user card as the recipe's proof
artifact for the other lanes to reference.

---

### L-B22 — docs records-registry backend (48)
**`param_verify_live: true`** — 보존기한 (retention periods) are statutory; cite the source, do not
assume. Coordinate with the `0201` reserved evidence-retention renumber
(`wave23-consolidation-inventory.md:85`) — `scout-spine-delta.md` §3 flags this exact conflict area.

Why: 7 of 12 findings BE-blocked — the heaviest BE ratio alongside org. Register `registers[docs]`.

Scope: records-registry list/read REST for non-EV finalized objects (`AP-`/`JL-`/`NT-`/`IN-`/`C-`)
so the archive is the archive of **all** finalized records; a pending/approve stage on registration
(§3.9 생성=등재 → 검토) with the 등재 대기 state; the egress-gated export endpoint (the contract defines
the `evidence_export.create` audit action but no endpoint); `GET copies/{id}/derived-preview` + the
ZIP entry-index REST; linked-object refs on the EV wire contract (or a generic object-links read via
L-B6); an audited-forbid path so an attempt to open the WORM-sealed original records the forbid
server-side.

Roots: `backend/crates/docs/**`, migration slot **0221**,
`docs/evidence/console/CAP-DOCS-EVIDENCE-CONSOLE/backend/**`.
Must not touch: `web/src/console/evidence/**` (that is L-B23's decision), `openapi.yaml` (manifest).
`docs` is tier-2 hot (30 hits) — plain-merge before push.

DoD: universal BE clauses; the forbid path writes an audit row **before** the refusal, tested as
`mnt_rt`; export passes the §3.10-⑤ egress gate and logs it; retention periods cite their source;
pending registration is not visible as archived until approved (state test, not UI copy).

---

### L-B23 — docs + dispatch dual-impl reconciliation
**DECISION + FE.**

Why: consolidation deferred `docs` FE and `dispatch` FE as incompatible dual-implementations ("spine
wins"), yet both carry live fidelity registers (`docs` 48 with 1 blocker + 7 majors; `dispatch` 55
with 1 blocker + 7 majors). Verified on the branch: `console/dispatch/**` exists (863 LOC) but is in
neither `MOUNTED_SCREEN_KEYS` nor `SCREEN_REGISTRY`; there is no `console/docs` dir (`console/evidence`
is the nearest). Chartering these as ordinary ports would be planning against code that is not the
spine's.

Scope: **first** identify, per module, which implementation the spine kept and where it lives; then
re-anchor each fidelity register finding onto that file set (a finding written against the deferred
impl may already be closed, or may be a different defect); **then** apply the L-B0..L-B3 recipe to
the surviving body. Publish the re-anchored registers so the numbers stay honest.

Highest-value re-anchored items to expect: dispatch — `objDrag` on `.dispatch__row` and `WO-` chips,
tri-state column sort, the module-config strip, customer-context second line (BE), ops-map link (BE
gap manifest); docs — shared-track table replacing the card-row list (sticky header, 6 columns,
whole-row button, per-type toned chip, `overscroll-behavior: contain`), archive card in the window
grammar.

Roots: `web/src/console/dispatch/**`, `web/src/console/evidence/**` (pending the identification),
`docs/program/console-dual-impl-reconciliation.md`.
Must not touch: anything until the identification step is written down and the roots are fixed.

DoD: the reconciliation doc names the surviving impl per module with file:line evidence and marks each
original register finding `still-open` / `already-closed` / `re-anchored-to <path>`; the surviving
bodies then satisfy the universal clauses and the three window assertions; if a body is genuinely not
mountable this wave, that is stated with its blocker, and no port work is claimed.

---

### L-B24 — attendance FE (50) — **HELD**
**Gate: `CAP-ATTENDANCE-CONSOLE` is `writer_assigned_gap_closure_in_progress` with an active writer;
`features/attendance` is the hottest FE dir on the spine (145 path-hits/48h) and
`backend/crates/attendance` is tier-1 (106). Do not dispatch until the current writer's lane lands or
explicitly hands off.**

Why: 3 blockers, 4 majors — the most blockers of any module. Register `registers[attendance]`.
Note the body lives at `web/src/features/attendance`, **not** `console/attendance` (which does not
exist), and it is reachable from **two** shells (`/attendance` under the `features/workspace` quadrant
shell, and `SCREEN_REGISTRY.attendance` under the `/console` shell) — the lane must state which shell
it is fixing and verify the window contract holds in both, or the pin lands in one and not the other.

Scope: mount the four cards through the L-B0 provider (popout/split/presets → gap manifest against
`registers[attendance].findings[0]`); the shared drag helper on exception rows, `AT-` codes, person
chips, and close rows; seed rows from `listAttendanceSummary` (already fetched for the sub pool) so
every employee appears, group/drill by team (`exception.team` exists) with a breadcrumb chain and
per-level summary — 출근율 stays deferred behind the schedule registry, named in the gap manifest;
an employee-day detail pin panel composing that day's segments, exceptions and resolutions.

Roots: `web/src/features/attendance/**`, `docs/evidence/console/CAP-ATTENDANCE-CONSOLE/frontend/**`.
Must not touch: `backend/crates/attendance/**` (that is L-B25), other module dirs.

DoD: universal clauses + three window assertions **in both shells**; the grouped roster test proves
every employee from the summary appears (not just those with exceptions); the deferred 출근율 and plan
lane are named gap-manifest lines with register anchors.

---

### L-B25 — attendance schedule / cover-planner backend — **HELD** (same gate as L-B24)
Why: attendance's blocker 1 is BE — there is no schedule/timetable read surface, so the dashed 계획
lane and the 7-entry legend cannot be truthful. Register `registers[attendance]`, 6 BE-blocked.

Scope: schedule/timetable read surface (planned segments per employee-day, including 연장 예정 and
승인 휴가) so the plan track and the full legend become real; the cover planner as a **D+7 forward
queue** of approved+pending absences × cover-required roles with the 4-state chip machine
(편성됨/미편성/승인 대기/장기) and per-state routing targets; site attribution on attendance records
(or a roster join) so day rows group by site with an entity segment filter; a deadline field on the
close board; a follow-up work-item link on 주52 근무 조정 acknowledgement.

Roots: `backend/crates/attendance/**`, migration slot **0222**,
`docs/evidence/console/CAP-ATTENDANCE-CONSOLE/backend/**`.
Must not touch: `web/src/features/attendance/**` (that is L-B24), `openapi.yaml` (manifest).

DoD: universal BE clauses; the plan lane is **absent** (not zero-filled) for employees with no
schedule row — honest-empty, asserted; cover queue returns only D+7 and only cover-required roles,
tested as `mnt_rt`; site grouping is server-attributed, not client-guessed.

---

## 4. Sequencing hazards and open decisions

1. **L-B0 is a true gate.** Any module lane dispatched before it ships pins-that-are-modals and its
   assertion (b) can only pass in an artificial harness. Do not parallelize around it.
2. **Every lane is formally HELD by the fan-out epoch contract right now**
   (`fanout_epoch.current_epoch: 1`, `normalized_lane_ids: []`, "legacy lanes remain explicitly held").
   Lens B cannot dispatch until these lanes are normalized into an epoch — that normalization is a
   program-level action, not something a lane can self-authorize.
3. **Two codex collisions are live**: `codex/console-search-object-fabric-20260724` overlaps L-B5, and
   the spine branch `codex/operational-object-runtime-progress` overlaps L-B6. Check both before
   chartering; if either already owns the contract, the lane degrades to FE adoption only.
4. **`backend/app/BUCK` is stale and the Buck graph is broken at HEAD** — 9 crates have no `BUCK`
   file (all 4 recruiting, 3 orgchange, 2 evaluation) and `backend/app/BUCK` has zero
   recruiting/orgchange/evaluation deps while `lib.rs` uses them. Since "Rust completion evidence is
   Buck2-only", L-B10 / L-B17 / L-B19 **cannot produce valid completion evidence** until the
   generated face is regenerated (`tools/buck/gen_first_party.py`) by the integrator. This is a
   serialized generated face, not a leaf write. Sequence the regeneration before those three lanes'
   DoD is assessable, or accept cargo-only evidence explicitly and record the exception.
5. **`openapi.yaml` has a live scar**: a mechanical fragment splice corrupted it and was reverted
   whole (`ee277e16` → `9bb877c6`), so 6 merged lanes' routes are currently absent and the generated
   clients do not cover them. Every BE lane here emits an openapi manifest; **an agent is integrating
   openapi right now** — do not let a lane touch that file.
6. **Migration slots 0210-0222 are provisional.** `0201` is reserved for the docs evidence-retention
   renumber; high-water is 0202. Re-check immediately before push — collision is the documented
   failure mode.
7. **Nothing in this lens becomes visible.** `EXPOSED_SCREEN_KEYS = ["sales"]`. Closing the fidelity
   floor on 15 dark modules produces zero user-visible change; the exposure/evidence charter is a
   separate decision and should be planned alongside, or wave 4 is invisible by construction.
8. **Open decision the charter cannot make alone:** whether L-B0b (the 4-state provider with
   popout/split/presets) is in wave 4 at all. Five registers carry blockers that only L-B0b can close
   (payroll, attendance, evaluation, org, docs). This charter defers it and requires each of those
   lanes to gap-manifest the deferral truthfully. If the program wants those blockers closed in wave 4,
   L-B0b must be inserted between L-B0 and the fan-out — and it brings the entire §5.2 window-a11y
   contract plus Playwright keyboard journeys with it (axe cannot verify any of it).
9. **L-B15 owns shared `ListTable`/`ModuleScreen` files.** Other module lanes request the same sort /
   selection / config-strip work. Landing it once in L-B15 is correct, but it means L-B15 must be
   sequenced before (or explicitly locked against) L-B13/L-B16/L-B18, whose registers ask for the same
   primitives.
10. **Do not conflate the three window systems in any lane text.** `features/workspace` is the
    prototype's `panels`/`snapTo` mechanism; `console/window/WindowManager` is the mounted 3-state
    model; `console/window/useWindowEngine` is the prototype's *card* engine. The frontend-depth brief
    §5.2 describes the a11y contract for the **first**; L-B0's target is the **second**.
