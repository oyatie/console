# L-F2 — shared ObjectCard a11y + `objectCardWindowEntry` signature freeze

**Lane** `w4-f2-a11y` · **branch** `claude/w4-f2-a11y-20260725` ·
**worktree** `/Users/jasonlee/Developer/maintenance-worktrees/w4-f2-a11y-20260725` ·
**date** 2026-07-25

Charter: `WAVE4-CHARTER-DEPTH.md` §2 "L-F2 · Shared object-card a11y + signature freeze".
Two defects in shared code that every drilling module inherits, plus a contract freeze
that L-X10…L-X13 depend on. Fixing them here is the root-cause fix: the audit found 13
module lanes poised to copy the broken shape.

## 1. Defect: drag hosts were inert `<span>`s

`ObjectCard.tsx:779` spread `objDrag(...)` onto a bare `<span>` — `draggable=true` and a
`data-obj-code`, but **no role, no tab stop, and no way to reach the reference the host
carries without a mouse.**

The same defect existed **twice** in the same file: the header object code (`:779`, the
one the charter names) and **every relation row's far-end reference** (`:396`). The
charter's own reasoning — fix it in the shared card so 13 inheritances collapse into one
diff — applies identically to the second site, so both were fixed by one extracted
control rather than patching only the line the audit cited.

### The fix

One `ObjectRefDragHost` component (`ObjectCard.tsx`), used at both sites:

- renders a **`<button type="button">`** carrying the unchanged `objDrag(code, title)`
  spread — the drag payload, `draggable="true"` and `data-obj-code` are byte-identical
  to before, so pointer drag behaviour is untouched;
- **keyboard activation copies the exact same reference token** the drag payload carries
  (`objectRefToken(code, title)`, i.e. `[WO-2643 4호기 유압 점검]`). That token is what
  `parseObjectRefText` already reads back in any relation-draw input, so the drag grammar
  now has a real keyboard path instead of a mouse-only one. **This is not a decorative
  focus stop** — activating it does the keyboard equivalent of the drag, which is why it
  is not a dead control under the truthfulness bar;
- a **clipboard rejection is reported**, not swallowed: `StatusChip tone="danger"
  role="alert"`. Success renders `StatusChip tone="ok" role="status"`. Status is a chip,
  no captions, no explanatory copy (DESIGN §4-12);
- the chip clears on blur, so no permanent status sticks to the card header. No timer,
  so nothing to leak or flake on;
- token colours only (`dragHostButtonStyle` reuses the existing `actingChipButtonStyle`:
  transparent background, no border, `minHeight/minWidth: 44` for AA 2.5.8). The caller's
  own style (`monoStyle` / `chipRowStyle`) is spread last and wins, so the rendered
  typography and layout are unchanged.

Precedent followed, not invented: `console/modules/GenericModuleScreen.tsx:745` already
renders a drag host as `<button {...objDrag(...)} aria-label=…>`. This lane makes the
shared card match the pattern the repo already had.

## 2. Defect: `ObjectCardModal` had no focus trap and no focus return

`ObjectCard.tsx:895-913` was `role="dialog" aria-modal="true"` with a single Escape
handler on the overlay `div`. Tab walked straight out of the dialog into the page behind
it, and closing dropped focus on `<body>`.

### The fix

- **Initial focus** on the close button via a ref (was `autoFocus`, which already worked —
  the pin is new, the behaviour is not; see §4).
- **Trap**: a document-level capture `keydown` listener wraps Tab / Shift+Tab at the
  panel's focusable boundary, and pulls focus back to the panel if it has escaped
  entirely. Same `FOCUSABLE_SELECTOR` and same trap shape as the existing
  `console/dispatch/StartP1DispatchDialog.tsx`.
- **Focus return** to the invoking control on close, for both exits (Escape and the close
  button), implemented in the effect cleanup so it fires however the modal unmounts.
- **`onClose` is read through a ref.** Every caller passes an inline arrow, so a
  `[onClose]` dependency would re-run the effect on each render — resetting focus to the
  close button mid-interaction and firing the focus-return on every render. This is
  documented in the code because it is the non-obvious part of the fix.

`ponytail:` comment in the file records that the selector/trap is duplicated from
`StartP1DispatchDialog` because neither file owns a shared a11y module, and names the
upgrade path (one `console/components/useFocusTrap` when a third dialog needs it). It was
not extracted now: `console/dispatch/**` is not this lane's root.

## 3. `objectCardWindowEntry` frozen

The charter flags this as a contract other lanes depend on. Frozen by:

- a doc comment on the function naming the freeze and pointing at the pinning test;
- `ObjectCard.test.tsx > objectCardWindowEntry frozen contract`, three assertions:
  - a **type-level** pin — the export is assigned to an explicit
    `(descriptor: ObjectCardDescriptor, handlers?: ObjectCardHandlers) => WindowEntry`,
    so a parameter-list or return-type change fails `tsc`, not just the runtime;
  - the descriptor → entry field mapping is verbatim (`id`/`title`/`code`) and the entry
    has **exactly** the keys `code, id, render, title` — an added or renamed field fails;
  - `entry.render()` renders the card **and threads handlers through**, proven by firing
    the action button and asserting the handler call.

L-X10's precondition (`import { objectCardWindowEntry } from "console/objectcard"`) is
unchanged — `index.ts` already re-exports it and was not touched.

## 4. Red → green

Tests were written first and run against the unmodified component.

**RED** (`vitest-red.txt`) — `8 failed | 18 passed (26)`:

| Failing test | Proves |
|---|---|
| renders the header object code as a focusable button, not an inert span | `:779` span defect |
| renders every relation row's far-end reference as a focusable button | `:396` span defect |
| copies the exact drag payload token on keyboard activation | no keyboard path existed |
| reports a failed copy instead of dying silently | error path |
| keeps Tab inside the dialog (forward wrap) | no focus trap |
| keeps Shift+Tab inside the dialog (backward wrap) | no focus trap |
| returns focus to the invoking control when Escape closes the dialog | no focus return |
| returns focus to the invoking control when the close button closes the dialog | no focus return |

**GREEN** (`vitest-green.txt`) — `3 files, 54 passed (54)` over the objectcard scope
(`ObjectCard.test.tsx`, `useObjectCard.test.ts`, `wired.test.tsx`).

**Honest note on the two tests that were already green before the fix**, so the red list
is not overstated:

- *"moves initial focus into the dialog on open"* passed pre-fix — React's `autoFocus`
  on the close button already did this. It is kept as a regression pin for the explicit
  ref that replaced it, not claimed as a fix.
- The three `objectCardWindowEntry frozen contract` tests passed pre-fix **by design** —
  a freeze pins existing behaviour; a red freeze test would mean the contract was already
  broken.

## 5. Gates

| Gate | Command | Result |
|---|---|---|
| unit (lane scope) | `npx vitest run src/console/objectcard` | **54/54 pass**, 3 files |
| unit (downstream consumers) | `npx vitest run src/console/{modules,ontology,explore,leave,configconsole}` | **179/179 pass**, 17 files |
| unit (whole console) | `npx vitest run src/console` | **1534 passed, 0 test failures**; 9 suites fail to *load* — see below |
| eslint | `npx eslint src/console/objectcard --max-warnings 0` | **exit 0**, clean |
| ui strings | `node scripts/check-ui-strings.mjs` | **OK** — no inline Hangul |
| console purity | `node scripts/check-console-purity.mjs` | **OK** — 569 files clean |
| typecheck | `npx tsc -b` | exit 1 on **7 pre-existing errors in files this lane does not own** — see below |

### `tsc` and the 9 unloadable suites are one pre-existing defect, not this lane's

Six console source files import **`react-router-dom`**, which is **not a declared
dependency and is not in `package-lock.json`** — `web/package.json` declares
`react-router@^8.3.0` only:

```
src/console/directory/DirectoryScreenBody.tsx      src/console/notif/NotifScreen.tsx
src/console/evaluation/EvaluationScreen.tsx        src/console/org/OrgConsoleRoute.tsx
src/console/payroll/PayrollScreen.tsx              src/console/recruiting/RecruitingScreenBody.tsx
```

Proven pre-existing, two independent ways:

1. **Baseline `tsc`.** The three lane files were backed up with `cp`, reverted with
   `git checkout --`, and `tsc -b` re-run: **the identical 7 errors, byte for byte.**
   Files then restored from the backup and the suite re-verified green. (Backup, not
   `git checkout`, per the red-green-on-uncommitted-fix rule.)
2. **The read-only spine** (`pr488-design-mirror-sync`) has the same imports against the
   same `react-router@^8.3.0`.

All 9 unloadable suites resolve to that one missing package — 6 direct, and
`screens/registry.test.ts`, `shell/ConsoleShell.test.tsx`, `shell/nav.test.ts`
transitively through `PayrollScreen.tsx`. **Zero assertion failures anywhere.**

> **Cross-lane finding for the integrator.** This is wave-blocking and nobody owns it in
> the depth-first lane table: `shell/nav.test.ts` and `screens/registry.test.ts` are
> exactly the suites L-X14's exposure flip must run green. Either add `react-router-dom`
> as a dependency or migrate the six imports to `react-router` (v7+ merged the DOM
> exports into the base package). Not fixed here — those files are outside this lane's
> roots.

## 6. Shared collision root touched → manifest, not an edit

The drag host needs a Korean accessible name and Korean status copy, which belong in
`web/src/i18n/ko.ts` — a serialized collision root. **No edit was made.** Instead:

- the lane ships `objectCardA11yStrings()` in the module-private
  `web/src/console/objectcard/strings.ts`, the same accessor-with-English-fallback
  mechanism the file already uses for `objectcardGov` / `objectcardDyn`, with the
  proposed Korean in a `koManifest` comment;
- the wire-up is requested in
  `docs/evidence/console/wave4/L-F2/manifests/i18n-keys.json`.

**Why a new `ko.console.objectcardA11y` namespace rather than a field on
`objectcardDyn`:** `ko.ts:3099` carries `satisfies ObjectCardDynStrings`, so widening
that interface would itself force a `ko.ts` edit. The separate namespace is what keeps
the collision root serialized. Recorded in the manifest so the integrator does not
"simplify" it back into `objectcardDyn`.

**Open gap, stated plainly:** until the integrator applies the manifest, the drag host's
accessible name and its two status chips render in **English** in an otherwise Korean
console. The accessor already prefers the ko values, so applying the manifest needs no
lane-side change. This is the charter's "Korean accessible name" requirement — **partially
met**: the mechanism and the Korean text are delivered, the wire-up is the integrator's
serialized step.

## 7. Not done (named, not silently dropped)

- **No real-browser / axe proof.** §6.3-22 makes that mandatory for L-F1 and L-X10; L-F2's
  charter entry does not require it and the shared card has no route of its own to drive.
  The keyboard journey through this card (open → pin → Escape → focus returns to the
  invoking control) is L-X10's committed browser proof, and it now has a component that
  can satisfy it.
- **`useFocusTrap` not extracted.** The trap is duplicated with
  `console/dispatch/StartP1DispatchDialog.tsx`; that file is outside this lane's roots.
  Marked with a `ponytail:` comment naming the upgrade path.
- **`StatusChip`'s `ariaLabel` on a role-less `<span>`** is inert per ARIA. Pre-existing
  in `console/components/StatusChip.tsx` (not a lane root) and not newly introduced here —
  inside the new drag-host button the button's own `aria-label` is the accessible name, so
  this lane's surface is unaffected. Noted for whoever owns `console/components/**`.

## Files

| Path | Change |
|---|---|
| `web/src/console/objectcard/ObjectCard.tsx` | `ObjectRefDragHost` + both drag hosts converted; modal focus trap / initial focus / focus return; freeze doc comment |
| `web/src/console/objectcard/strings.ts` | `ObjectCardA11yStrings` + `objectCardA11yStrings()` + koManifest |
| `web/src/console/objectcard/ObjectCard.test.tsx` | 11 new tests across 3 describes |
| `docs/evidence/console/wave4/L-F2/manifests/i18n-keys.json` | integrator request for `ko.console.objectcardA11y` |
| `docs/evidence/console/wave4/L-F2/{vitest-red,vitest-green,eslint,gates,tsc}.txt` | raw gate output |

`web/src/console/objectcard/index.ts` was **not** modified — `objectCardWindowEntry`,
`ObjectCard` and `ObjectCardModal` were already exported.
