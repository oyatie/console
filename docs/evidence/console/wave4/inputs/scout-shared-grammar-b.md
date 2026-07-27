# Wave-4 lens-B scout (second pass) — shared console grammar: contract deltas, hardening, port-path fork

Worktree: `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`. Paths relative to `web/src/` unless noted. Read-only, 2026-07-25.

**Relationship to the peer brief.** A parallel scout wrote
`scratchpad/wave4/scout-shared-grammar.md` before me. I independently traced the same code and
**agree with its structural findings** — two window models, only the 3-state one is production,
`useWindowEngine` is harness-only (`AppRouter.tsx:63-69,460`), 0/15 adoption across the new module
dirs, `codeGrammar.ts` is a genuinely dynamic prefix source. I have not duplicated those here.

This file records **four places where my trace contradicts or materially extends that brief**, plus
the evidence. Read both; where they disagree, the disagreements are all in §1–§4 below and each one
carries a file:line.

Also consulted and not re-derived: `docs/program/audit-4-22-23.md` §4-23 + Finding 2;
`docs/program/console-program-ledger.md` (collision-file/manifest discipline);
`/Users/jasonlee/Developer/maintenance/.omc/research/*` — pre-console-foundation, nothing on this
FE grammar, no overlap.

---

## 1. DISAGREEMENT — "hardening needed before 13 lanes: none blocking" is wrong on a11y

The peer brief rates `objDrag` a11y as fine because "the keyboard-only alternative is the
click→pin gesture". **That is true only where the drag host is a real button.** It is not true in
the two files it then names as the best-practice exemplars.

Verified non-focusable, no `onClick`, no `tabIndex`, drag-only — zero keyboard or AT path:

| Site | Element |
|---|---|
| `console/objectcard/ObjectCard.tsx:779` | `<span {...objDrag(descriptor.code, descriptor.title)}>` — **inside the shared card itself**, so every module that drills inherits it |
| `console/explore/ObjectExplorerScreen.tsx:358` | `NodePill` → `<span {...objDrag(...)}>` |
| `console/explore/ObjectExplorerScreen.tsx:464` | relation row → `<li {...objDrag(...)}>` |
| `console/explore/ObjectExplorerScreen.tsx:526` | `RegistryCard` → `<article {...objDrag(...)}>` |

The correct pattern exists and is documented in-repo:
- `console/modules/GenericModuleScreen.tsx:718-733` — `<button type="button" {...objDrag(...)}>`
  with `aria-label`, wrapped in `PolicyGated`.
- `console/configconsole/DashboardEditor.tsx:136` — comment states the rule verbatim:
  *"drill result row: real button (keyboard/AT operable, ≥44px) with objDrag on top"*.

**Why this matters for chartering:** if 13 lanes are told to copy `ObjectExplorerScreen`, the AA
a11y bar breaks 13 times and the `no-explanatory-ui` / enterprise-production-standard gate should
reject all 13 at merge. Fix `ObjectCard.tsx:779` in the shared code *first*, and write the button
rule into the charter text, not the exemplar pointer.

Second a11y item, same class: **`ObjectCardModal` has no focus trap and no initial focus**
(`ObjectCard.tsx:895-913`). It is `role="dialog" aria-modal="true"` with `onKeyDown={Escape}` bound
to a non-focusable `<div>` — Escape only fires if focus already happens to be inside. This modal is
the mandated no-provider fallback in every port recipe (`GenericModuleScreen.tsx:752-767`), so all
13 lanes inherit it and every unit test exercises it.

Neither is expensive. Both are cheaper once, in shared code, than 13 times in review.

## 2. EXTENSION — there is a **fourth** code recognizer, and it is hardcoded and divergent

The peer brief's §4 concludes the hardcoded-regex problem is solved at the grammar layer, with
only three per-kind maps left (`KIND_META`, `SLUG_META`, `COMPOSER_KIND_TO_SLUG`). Correct as far
as it goes, but it misses:

**`console/composer/grammar.ts:36` — `BARE_CODE_RE`:**
```js
const BARE_CODE_RE = /(^|[\s([{])([A-Z]{1,8}-[0-9]{1,10}(?:-[0-9]{1,6})?)/gu;
```

This never imports `codeGrammar`. It diverges in both directions:

- **Digit-only body.** `codeGrammar`'s body is `(?:PREFIX)-[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*`
  (`codeGrammar.ts:55`) — alphanumeric. So `OT-FINANCE`, `PAY-CHO`, `EV-2026-00012` drag and drop
  fine but **do not tokenize as `codeLink` in composer text**. These are not hypotheticals:
  `console/window/objDrag.test.ts:78-90` explicitly asserts all three round-trip through objDrag.
- **Uppercase-only prefix.** `Bid` is in `FALLBACK_CODE_PREFIXES` (`codeGrammar.ts:17`) and can
  never match `BARE_CODE_RE`.
- **Prefix-blind.** It accepts *any* `[A-Z]{1,8}` prefix, so unregistered codes parse and then
  render inert via `kindFromCode` — the file even says so (`grammar.ts:38-40`).

`messengerModel.ts:5` and `appr/composeModel.ts:1` do consume `codeGrammar`, so the divergence is
isolated to the composer's own parser — but that parser backs `TokenText`/`TokenComposer`, i.e.
every 기안/할 일/코멘트 surface.

**The lazy fix already has an unused hook.** `objectCodeBodySource()` is exported from
`codeGrammar.ts:92` with the comment *"for consumers that build their own combined pattern"* and
**nothing calls it** (grep: zero consumers). Pointing `BARE_CODE_RE`, `KIND_META`, and
`COMPOSER_KIND_TO_SLUG` at the dynamic source removes three of the four per-module edits
permanently, in one small pre-wave PR.

**Revised cost to add 13 module codes end-to-end, today:** up to 4 edits per code —
backend `code_prefix` row (or one literal in `FALLBACK_CODE_PREFIXES`), `KIND_META`
(`composer/objectKinds.ts:52`), `SLUG_META` (`objectcard/kinds.ts:18`), `COMPOSER_KIND_TO_SLUG`
(`objectcard/kinds.ts:57`, currently only 5 entries — a kind absent here is **not linkable**,
`linkTargetFromCode` returns `undefined`), plus a `BARE_CODE_RE` widening if any code body is not
pure digits. Four files that all 13 lanes would want. One pre-wave lane, not 13 concurrent edits.

## 3. EXTENSION — the port path is a fork, and one branch is 3–5× cheaper

The peer recipe assumes one path: author a `ModuleScreenConfig` + `ModuleDataAdapter` + a
`ScreenBody` wrapper riding `GenericModuleScreen` (its estimate: ~2–3 lane-days/module, adapter
being 80%). That is the right path for a module being *built*. But all 15 targets already exist as
hand-rolled bodies with their own list+detail selection state — e.g. `directory/DirectoryScreen.tsx`
(722 L, `Selection{kind,id}` at `:123`), `equipment/EquipmentScreen.tsx` (407 L, `select({kind,id})`
at `:390`), `payroll/PayrollScreen.tsx` (1321 L). Rewriting them onto the template is a *rewrite*,
not a *port*.

The cheap branch already has a working precedent: **`console/leave/LeaveConsole.tsx`** — a
hand-rolled body that adopted the grammar without becoming a `ModuleScreenConfig`:
- local descriptor mapper `ledgerDescriptor(row): ObjectCardDescriptor` in
  `console/leave/model.ts:164` — **≈55 LOC**;
- `useOptionalWindowManager()` at `LeaveConsole.tsx:581`;
- `windowManager?.open(objectCardWindowEntry(ledgerDescriptor(row)))` at `:644`;
- `objDrag` on the roster row at `:1352`.

**≈120–200 LOC + tests, ~1 lane-day per module.** Note the honest-empty discipline in that mapper
(`leave/model.ts:199-203`: `relations: []` with a `wire-pending` comment rather than a fabricated
edge) — that is the pattern the charter should mandate.

The fork should be an explicit per-module chartering decision:
- module has a real list endpoint **and** its body is thin → template path (peer recipe);
- module body is large/bespoke and already correct → LeaveConsole path;
- either way the four grammar touchpoints are identical.

**Blocking precondition the peer recipe does not gate on: does every row have an issued object
code?** `objDrag(code,…)` needs one and `parseObjectRef` re-validates it against
`objectCodeRegex()` before accepting a typed payload (`objDrag.ts:66`), so a UUID-keyed row cannot
participate at all. `directory` (member/employee UUIDs) and `equipment` (case/unit ids) visibly key
rows by bare id. **Triage codes per module before chartering** — a lane without a code is blocked
on backend code issuance, and will stall mid-wave otherwise.

## 4. EXTENSION — three smaller facts the charter should absorb

- **"Split" is missing too, not just popout.** The peer brief flags popout as deliberately absent
  (`windowModel.ts:6-9`, correct). It does not flag that the production model has a **single**
  `pinnedId` — a second `open()` silently demotes the previous pin to the tray
  (`WindowManager.tsx:168-177`). With 13 modules opening cards, that will read as data loss. If the
  design grammar's 분할/split is in wave-4 scope, that is a provider lane, same bucket as popout.
- **`saveLayout()` / `restoreDefault()` are a dead context API.** Grep across `web/src` finds the
  only callers are `components/shell/AppShell.test.tsx:138,199`. No production control invokes
  them, though `ko.console.window.saveLayout / restoreDefault` strings exist
  (`i18n/ko.ts:1801-1802`). (`features/workspace/store.ts:98`'s `restoreDefault` is the *legacy*
  workspace store — a different system, not this one.) Either wire a control or drop them from the
  recipe; do not have 13 lanes document a control that does not exist.
- **`audit-4-22-23.md` Finding 2 is still open and belongs in this fan-out.**
  `pages/UsersPage.tsx` / `console/identity/*` have zero draggables on the one live-data surface
  the audit could actually exercise. Same recipe, same wave.
- **Make the audit's unverifiable assertions unit-testable.** The audit could only mark N/T because
  the scratch org had no rows. Every ported module should assert, in jsdom under
  `<WindowManagerProvider>` (helper shape at `modules/moduleEngine.test.tsx:51`): (a)
  `[draggable="true"]` present with the right `data-obj-code`, (b) row activation sets `pinnedId`,
  (c) the drag host is a focusable `button`. That closes §4-23 without seed data.

---

## 5. Net implications for wave-4 lane chartering

1. Charter onto `WindowManagerProvider` + `objDrag` + `objectCardWindowEntry`, explicitly not
   `useWindowEngine` (agreeing with the peer brief).
2. Run **one pre-wave grammar lane**: point `composer/grammar.ts` `BARE_CODE_RE`,
   `composer/objectKinds.ts` `KIND_META`, and `objectcard/kinds.ts`
   `SLUG_META`/`COMPOSER_KIND_TO_SLUG` at `codeGrammar` (`objectCodeBodySource()` is already
   exported and unused). Removes 3 of 4 per-module edits and the worst 4-file collision.
3. Run **one pre-wave a11y lane** on shared code: `ObjectCard.tsx:779` span→button, modal focus
   trap. Then the exemplar pointer is safe to hand to 13 lanes.
4. **Triage object codes per module first.** No issued code ⇒ backend dependency, not a FE lane.
5. Charter the **port-path fork per module** (template rewrite vs LeaveConsole-style adoption);
   budget ~1 lane-day for the latter, 2–3 for the former.
6. If 팝아웃 **or 분할/split** or named presets are in scope, that is a provider/foundation lane
   sequenced before the module fan-out.
7. Serialize `i18n/ko.ts` (and `MOD_SCREENS`/`nav.ts`/`ONT_TYPES`) via the ledger's manifest +
   serial-wire-up discipline; 13 lanes touching ko.ts concurrently is a guaranteed conflict.
8. Fold `pages/UsersPage.tsx` (audit Finding 2) into the same fan-out.
9. Nothing ships visible: `EXPOSED_SCREEN_KEYS = ["sales"]` (`console/shell/nav.ts:134`). Plan the
   exposure/evidence charter separately or the wave is invisible.
