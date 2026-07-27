# L-F1 stage-2 — fresh-eyes adversarial verification

Verifier did not write the stage-1 code. Everything below was re-derived from the tree at
`8c917bba`, not from the build report. Verdict: **the fix is real and correctly rooted;
one DoD line was left open and was described inaccurately. Corrected here.**

---

## 1. The defect and the fix — CONFIRMED at the root

Re-derived independently:

| Claim | Verified how | Result |
|---|---|---|
| `WindowManagerProvider` was absent from every console screen | `grep -rn WindowManagerProvider web/src` at base `49e53bb7` | only `components/shell/AppShell.tsx:172`, 9 test wrappers, and the nested `OntologyWorkspaceBody.tsx:299`. Zero in `console/shell/**`. |
| So screen bodies got `null` | `useOptionalWindowManager()` returns `useContext(...)`, no default | confirmed |
| The fix is at the root, not per-module | every console body reaches the DOM through `SCREEN_REGISTRY` → `ConsoleShell`'s `<ScreenBody />` (`ConsoleShell.tsx:472`), and `screens/registry.ts` has exactly one consumer | one provider above `<ScreenBody />` fixes all 15 at once |
| The mount is not cosmetic — real screens do call it | `windowManager.open(...)` in `ontology/OntologyManagerScreen.tsx:1006`, `explore/ObjectExplorerScreen.tsx:868`, `modules/GenericModuleScreen.tsx:752,1015`, `leave/LeaveConsole.tsx:644`, `configconsole/DashboardEditor.tsx:537` | these were all dead before the mount and are live after |
| No second host | `grep WindowManagerProvider` at HEAD → `ConsoleShell.tsx` only, within `console/**`. `AppShell`'s is a disjoint route tree (`/console/*` renders `ConsoleApp` with no `AppShell` parent, `AppRouter.tsx:441`) | confirmed |

## 1.1 Red-green proof, re-run by the verifier

Reverted **only** `ConsoleShell.tsx` + `OntologyWorkspaceBody.tsx` to `49e53bb7` (`git show
<sha>:path >`, with a `cp` backup — never `git checkout` over a working tree), ran, restored
from the backup, confirmed `git status` clean.

```
BEFORE  3 files: 8 failed | 31 passed
        consoleShellWindowHost.test.tsx  5 failed / 5
          × mounts exactly one window host, and it wraps every screen body
          × keeps the previously pinned object recoverable when a second one is opened
          × keeps the pinned panel and the tray across screen navigation
          × gives a session with no owned incarnation the full in-memory model and no storage
          × reads the layout key under a partition scoped to the exact session incarnation
        ExploreBody.test.tsx / OntologyManagerBody.test.tsx  3 failed (host count 2, not 1)
AFTER   39 passed
```

The tests are genuine evidence, not pass-either-way assertions.

## 1.2 Real-browser proof, re-run and **extended**

`docs/evidence/console/wave4/L-F1/browser-window-host.mjs` against headless Chromium on a
`vite dev` origin. Two verifier changes:

1. **Its documented invocation was wrong** (`cd web && node docs/...` — there is no
   `web/docs`, the script cannot run from `web/`). Header corrected to run from the repo
   root.
2. **Added the check the design actually turns on.** DESIGN §4.7-2 requires the pin to be
   `"본문이 옆으로 재배치 … 오버레이 아닌 진짜 split"`. Nothing verified that the padding the
   manager puts on the host survives the shell's flex chain — jsdom cannot measure it, and a
   swallowed padding would make the whole mount cosmetic. The probe applies the manager's
   own `paddingRight` to `[data-window-host]` and requires `[data-cshell-root]` to give the
   space back.

```
ok exactly one window host — count=1
ok host wraps the shell root
ok no host nested inside a screen section
ok shell still fills the viewport — shell=900 viewport=900
ok screen body keeps real height — screen=844
ok no document-level horizontal scroll — scrollWidth=1440 clientWidth=1440
ok no uncaught page error
ok host padding reflows the shell (real split, not overlay) — 1440 -> 1080   [new]
ok the split does not push the document into horizontal scroll               [new]
ok removing the padding restores the full width — restored=1440              [new]
```

Writing that probe caught a measurement trap worth recording: `hostStyle` carries
`transition: padding 0.18s`, so a synchronous `getBoundingClientRect()` after setting the
padding reads the *pre-transition* width and reports a false "overlay" verdict. The probe
settles for 400 ms.

---

## 2. Finding — layout retention is a phantom, and stage 1 described it as working

The charter's L-F1 DoD says: *"`saveLayout()` / `restoreDefault()`: wire both controls into
the console window layer **or delete the API and its `ko.ts:1808-1809` keys** (D-5 default is
delete). Either way no documented control lacks an implementation."* Stage 1 did neither and
asked the integrator to decide. The verifier's issue is not the deferral — it is that the
gap is **larger than stage 1 stated**, and its own comments and report asserted the opposite.

Verified by exhaustive grep over `web/src`:

- `writeSavedLayout` — the only writer of `oyatie.console.window.layout.v2.*` — is called
  from exactly one place, `saveLayout()` (`WindowManager.tsx:277`).
- `saveLayout()` has **zero non-test callers repo-wide**. (`PurchaseRequestPanel.tsx:977`
  and `ko.ts:7531` are unrelated namespaces.) → **the key is never written.**
- `register()` — the only consumer of `savedStatesRef`, i.e. the only path that re-applies a
  saved state — has **zero callers repo-wide** outside `AppShell.test.tsx`'s harness.
- It could not work if it were called: `WindowEntry.render` is a closure, so only the owning
  screen can re-supply an entry. Restoring a pinned card across a reload is not a wiring
  gap, it is missing per-screen re-registration.

So both ends are unwired, and `readSavedLayout` reads a key nothing writes. Stage 1's
statements that "a user's saved layout keeps its partition" and that "the saved arrangement
lives in `oyatie.console.window.layout.v2.<partition>`" are not true today.

**What the verifier did NOT do, and why.** D-5's delete would cascade through
`authorityPartition` / `retentionEnabled` / the whole partitioning — reversing a
security-reviewed design, rewriting `WindowManager.test.tsx`, `consoleShellWindowHost`,
`ExploreBody`, `OntologyManagerBody`, and editing `components/shell/AppShell{.tsx,.test.tsx}`,
which is outside this lane's roots. Auto-persisting instead would add localStorage writes
(do-not-ship ban #9) that nothing can read back. Both are platform calls of the same shape as
D-4, not verification-pass edits. Reported, not silently absorbed.

**What the verifier did do** (all inside lane roots):

- `WindowManager.tsx` — a `ponytail:` note on `PARTITIONED_STORAGE_PREFIX` stating the layer
  is inert, why, and that the `/api/v1/me/workspace` contract should **delete** it rather
  than port it.
- `windowManagerContext.ts` — `saveLayout`, `restoreDefault`, `register` doc comments now say
  they are uncalled, instead of describing behaviour the app does not have.
- `ConsoleShell.tsx` — the comment block and the `ponytail:` line rewritten: the mount
  delivers an in-memory arrangement across navigation; the partition scopes a storage key for
  a future writer and is not a persistence claim.
- `consoleShellWindowHost.test.tsx` — the test named *"partitions the saved layout by the
  exact session incarnation"* proved only a partitioned **read** of an always-empty key. It
  is the exact false-green shape this lane exists to kill, so it is renamed to what it
  proves, and a **tripwire** is added: under an owned incarnation, pinning two objects must
  write nothing. Verified red-green — temporarily wiring `useEffect(() => saveLayout())`
  turns it red (`1 failed | 5 passed`), reverted.

The tripwire is the actionable artifact: the day anyone wires a writer it goes red, forcing
the partitioning, ban #9 and the two orphan `ko.console.window.*` strings to be re-decided
together.

**No `ko.ts` manifest was emitted, deliberately.** Deleting the two strings while the dead
API survives would let the DoD line look closed when it is not. The integrator's decision
should be on the whole layer (§5 open item 1).

---

## 3. Enterprise / frontend bar — checked line by line

| Bar | Result |
|---|---|
| plain-literal `className` (purity gate bans `cn`/`clsx`) | `check-console-purity.mjs` clean, 572 files; no `cn(`/`clsx` in the diff |
| no inline Hangul | `check-ui-strings.mjs` clean. The Hangul in `consoleShellWindowHost.test.tsx` is in a `.test.tsx`, explicitly allowed (`check-ui-strings.mjs:8`) |
| token colors only | diff adds no color; `trayStyle`/`hostStyle` are layout only; `TrayDock` keeps `var(--surface)` / `var(--border)` |
| status = chips, no explanatory captions | no copy added anywhere in the diff |
| AA a11y | tray = `role="group"` + `aria-label="작업 트레이"`; chips are real `<button>`s, `aria-label="{title} 복원"`, `minHeight: 44`; pinned panel is `role="region"` + `aria-labelledby`. Asserted by role/name, not by test id, in the shell test |
| no stubs / TODO / `test.skip` / `.only` / dead controls | swept the lane diff — zero hits |
| production bundle | `npx vite build` exit 0; `console/testing/**` (which imports `@testing-library/react`) is imported only by `.test.tsx` files and does not reach the bundle |
| backend bar (RLS, PBAC, audit, envelope, idempotency, statutory) | **N/A** — frontend-only lane. No migration, no `openapi.yaml`, no clients, no Rust |

## 4. Roots discipline

Lane commits touch: `console/window/**`, `console/shell/ConsoleShell.tsx`,
`console/testing/**`, `console/screens/_ontology/OntologyWorkspaceBody.tsx`,
`docs/evidence/console/wave4/L-F1/**` — all declared.

**Zero shared collision roots touched** by the lane's own commits: no `i18n/ko.ts`, no
`shell/nav.ts`, no `screens/registry.ts`, no `openapi.yaml`, no `clients/**`, no migrations,
no `backend/app/src/{lib,objects}.rs`. Verified against `git show --stat` for each of
`49e53bb7 b358b9bb ebbc49d0 8c917bba`. (`f07f591c` is the D-2 plain-merge of spine tip
`8c3da1cd`; the spine's own changes to `nav.ts` etc. arrive through it and are not the
lane's.)

Two files outside the declared roots, both disclosed by stage 1 and both unavoidable — they
directly asserted the removed provider: `screens/explore/ExploreBody.test.tsx`,
`screens/ontology-manager/OntologyManagerBody.test.tsx`. Reviewed: no coverage was dropped.
The two assertions that changed meaning were partitioning claims those bodies no longer own;
each is replaced by a body-level "mounts no host of its own" assertion plus the shell-level
partition assertion, and the ontology-manager suite gained a `hosted()` wrapper so the
docked-card surfaces are still exercised through a real provider.

## 5. Open items after verification

1. **Layout retention is inert end to end** (§2). Recommended resolution, integrator's call:
   delete the client storage layer (`readSavedLayout`/`writeSavedLayout`/`clearSavedLayout`,
   `saveLayout`, `restoreDefault`, `authorityPartition`, `retentionEnabled`) in the change
   that lands `GET/PUT /api/v1/me/workspace`, and delete `ko.console.window.saveLayout` +
   `ko.console.window.restoreDefault` with it. The tripwire test enforces the coupling.
2. **Modality: the panel and tray render above the shell's modal surfaces.** The pinned panel
   (`z-index 1100`) and `TrayDock` (`1200`) are siblings of the shell root, so they sit above
   the mobile drawer (`82`) and the palette backdrop (`90`), and the drawer's `inert` /
   `aria-hidden` (applied to `<main>` and the rail) does not reach them. Keyboard is still
   contained (the drawer traps Tab) and `aria-modal="true"` hides them from AT, so this is a
   pointer-level modality leak, newly reachable because the console now renders a tray at
   all. Fix belongs with the docked task-bar slice (open item 3), which relocates both.
3. Tray presentation is the floating pill, not the design's docked bottom task bar
   (`DESIGN.md:143-147`) — unchanged from stage 1, correctly scoped out.
4. Multi-pin remains deferred per D-4 — unchanged from stage 1.

## 6. Gates re-run by the verifier, on the verified tree

| Command (from `web/`) | Result |
|---|---|
| `npx vitest run` (full web suite) | **335 files / 2809 tests passed**, 0 failed |
| `npx tsc -b` | clean |
| `npx eslint . --max-warnings 0` | clean |
| `node scripts/check-ui-strings.mjs` | clean |
| `node scripts/check-console-purity.mjs` | clean — 572 files |
| `npx vite build` | exit 0 |
| `node docs/evidence/console/wave4/L-F1/browser-window-host.mjs` (from repo root) | **10/10** checks passed |
