# L-F1 — Window host: mount, nested-provider repair, tray-restore

Lane `w4-f1-window` · branch `claude/w4-f1-window-20260725` · worktree
`maintenance-worktrees/w4-f1-window-20260725` · wave-4 charter §2 "L-F1", DECISIONS D-4.

## 1. The defect, restated from the code

`WindowManagerProvider` existed in exactly three places, none of them the console shell:

| Place | File | Consequence |
|---|---|---|
| Legacy shell | `web/src/components/shell/AppShell.tsx:172` | Only the legacy routes had a window model. |
| Per-test wrappers | 9 test files | Every module's own test supplied the host the shell never mounted. |
| Nested inside one screen body | `web/src/console/screens/_ontology/OntologyWorkspaceBody.tsx:299` | The ontology workspace forked its own arrangement. |

`grep -c WindowManagerProvider web/src/console/shell/ConsoleShell.tsx` -> **0**.

So for every console screen body `useOptionalWindowManager()` returned `null`, and the
pin / minimize / tray / restore grammar was **dead in production while green in jsdom**.
A fidelity audit read that as "window model missing" in 13 modules; it was one absent
provider, not 13 defects.

The single-pin data-loss path is a consequence of the same absence, not a separate bug:
`pin()` already demotes the previous `pinnedId` into `minimizedIds`
(`WindowManager.tsx:168-178`), but with no provider mounted **no tray was ever rendered**,
so the demoted card had no restore affordance. Mounting the host with its tray closes the
path; no new state machine was added (D-4 keeps the 4-state engine deferred).

## 2. What changed

| File | Change |
|---|---|
| `web/src/console/window/WindowManager.tsx` | `data-window-host` on the host wrapper (names the single layout partition, assertable in jsdom *and* in a real browser); `hostStyle` so a flex shell can make the wrapper participate in its layout; `trayStyle` forwarded to `TrayDock`'s existing `style` hook. |
| `web/src/console/shell/ConsoleShell.tsx` | Mounts **one** `WindowManagerProvider` wrapping the whole shell (sidebar + main + rail + palette), above every screen body. |
| `web/src/console/screens/_ontology/OntologyWorkspaceBody.tsx` | Nested provider deleted; its authority `key` moved to the sibling gate provider so an authority change still remounts the subtree. |
| `web/src/console/testing/renderWithWindowManager.tsx` (new) | The frozen shared harness. |

### Mount decisions, and why

- **Level.** The host wraps `[data-cshell-root]`, not the screen `<section>`. The pinned
  panel is `position: fixed` at the right edge, and the host's `padding-right` is what
  moves content out of its way — applied at the section it would leave the sidebar and
  comms rail underneath the panel. Verified in a real browser (section 4).
- **`hostStyle`.** The host wrapper is a plain block. Dropped into `ConsoleApp`'s flex
  column it breaks the chain and the console collapses to content height. The shell
  passes `display:flex / flexDirection:column / flex:1 1 auto / minHeight:0 / minWidth:0`;
  the split padding stays manager-owned. jsdom cannot catch this — the browser check does.
- **Partition.** `ontologyWorkspaceAuthorityKey(session, viewAs)` is reused verbatim, so
  the key is byte-identical to the one the nested provider used, and the cross-incarnation
  isolation the nested provider was security-reviewed for is preserved, now shell-wide.
  `key={windowAuthority ?? ...}` on the provider makes an authority change tear the
  arrangement down. **Correction (stage-2 verification):** this scopes the storage KEY, not
  a working saved layout — nothing writes that key and nothing restores from it. See
  `verification.md` §2.
- **`retentionEnabled={windowAuthority !== undefined}` + rendered tray.** A session
  without an owned org/user/incarnation gets the **complete in-memory** window model and
  **never touches storage**. This is deliberate: gating interaction on an owned partition
  (as `AppShell` does, via `renderTray={false}`) would leave the model dead again for any
  session that lacks an incarnation claim.
- **Tray placement.** `trayStyle={{ left: calc(<sidebar>px + var(--sp-5)) }}` keeps the
  floating tray pill off the sidebar's controls. The design's docked bottom task bar
  (`Oyatie Console.dc.html:6596`, `DESIGN.md:143-147`) is **not** built here — see section 6.

## 3. Before / after — jsdom

Command (identical in both states; only `ConsoleShell.tsx` and `OntologyWorkspaceBody.tsx`
were reverted to `49e53bb7` for the "before" run):

```
cd web && npx vitest run \
  src/console/window/consoleShellWindowHost.test.tsx \
  src/console/screens/explore/ExploreBody.test.tsx \
  src/console/screens/ontology-manager/OntologyManagerBody.test.tsx
```

| | Result |
|---|---|
| **before** (`vitest-before.txt`) | **8 failed** / 31 passed |
| **after** (`vitest-after.txt`) | **39 passed** |

The eight failures, and what each proves:

| Failure | What it proves was broken |
|---|---|
| `mounts exactly one window host, and it wraps every screen body` -> `expected [] to have a length of 1` | The shell mounted **zero** hosts. |
| `keeps the previously pinned object recoverable when a second one is opened` -> `Unable to find [data-testid="probe-open-a"]` | The probe rendered its `probe-without-manager` branch: a screen body saw **no manager at all**. |
| `keeps the pinned panel and the tray across screen navigation` -> same | idem, across navigation. |
| `gives a session with no owned incarnation the full in-memory model and no storage` -> same | idem. |
| `partitions the saved layout by the exact session incarnation` -> `expected +0 to be 2` | No host means no partitioned layout read at all. |
| `has no render-time API identity inference and mounts no window host of its own` -> source still contains `WindowManagerProvider` | The nested provider. |
| `adds no second window host inside the shell's...` -> `expected ...(2) to have a length of 1 but got 2` | Two hosts per root: the shell's and the nested one. |
| `mounts no window host of its own across API recreation or authority change` -> same | idem. |

`consoleShellWindowHost.test.tsx` renders the real `ConsoleApp -> ConsoleAuthzProvider ->
ConsoleShell` — never a hand-wrapped body. Only the screen registry (a probe per mounted
key) and four transports (authz projection, nav badges, self profile, comms rail) are
mocked, so the composition under test is production's.

## 4. Before / after — real browser (6.3-22)

jsdom is the harness that hid this bug, and it cannot lay anything out. `browser-window-host.mjs`
drives headless Chromium against a `vite dev` origin with the boot silent-refresh and every
`/api/**` read fulfilled by route handlers — no backend, no shared dev stack touched.

```
cd web && VITE_CONSOLE_DEV_PREVIEW=1 npx vite --host 127.0.0.1 --port 5199
node docs/evidence/console/wave4/L-F1/browser-window-host.mjs
```

| Check | before (`browser-before.txt`) | after (`browser-after.txt`) |
|---|---|---|
| exactly one window host | **FAIL — count=0** | ok — count=1 |
| host wraps the shell root | **FAIL** | ok |
| no host nested inside a screen section | ok | ok |
| shell still fills the viewport | ok (900/900) | ok (900/900) |
| screen body keeps real height | ok (844) | ok (844) |
| no document-level horizontal scroll | ok (1440/1440) | ok (1440/1440) |
| no uncaught page error | ok | ok |

The four layout rows are the browser-only value: they prove the inserted host wrapper did
not break `ConsoleApp`'s flex chain (C-42 body-scroll defect class included).
Screenshot: `console-window-host.png`.

## 5. Gates

Run on the merged tree (`f07f591c`; spine `8c3da1cd` plain-merged in per the D-2 train):

| Gate | Result |
|---|---|
| `npx vitest run src/console src/components/shell src/pages` | **225 files / 2036 tests passed** |
| `npx tsc -b` | **clean** |
| `npx eslint src/console/window src/console/testing src/console/shell/ConsoleShell.tsx src/console/screens/{_ontology,explore,ontology-manager} --max-warnings 0` | **clean** |
| `node scripts/check-ui-strings.mjs` | clean |
| `node scripts/check-console-purity.mjs` | clean — 572 files |
| `node docs/evidence/console/wave4/L-F1/browser-window-host.mjs` | all checks passed |

## 6. Named gaps and open items — nothing here is silently deferred

1. **`localStorage` layout ceiling (do-not-ship ban #9).** The arrangement *would* live in
   `oyatie.console.window.layout.v2.<partition>` — stage-2 verification found nothing ever
   writes or restores it, so the ceiling is a phantom rather than a live localStorage
   dependency (`verification.md` §2). Partitioning by
   the exact incarnation is the mitigation, not the fix. The `ponytail:` comment naming the
   upgrade path is on the `windowAuthority` memo in `ConsoleShell.tsx`; the
   server-persisted per-user workspace endpoint (`GET/PUT /api/v1/me/workspace`, already
   exercised by `e2e/specs/admin-29-console-window.spec.ts` for the `/console-dev/window`
   engine) is the target. **Not built here** — D-4 keeps the richer engine deferred.
2. **`saveLayout()` / `restoreDefault()` remain unwired in the console, deliberately.**
   `ko.console.window.saveLayout` (`ko.ts:1809`) and `...restoreDefault` (`ko.ts:1808`) have
   **zero render sites repo-wide**. They are API + strings, not dead *controls* (6.1-4 is
   about affordances that cannot act, and none is rendered). The console mount does not
   persist for a session without an owned incarnation, so a "배치 저장" button would be a
   no-op there. Wiring them belongs with item 1; deleting them is a `ko.ts` change and
   `ko.ts` is an integrator root. **Decision requested from the integrator**, with the
   anchors above. This lane did not fabricate a control to close the line.
3. **Tray presentation is the floating pill, not the design's docked bottom task bar.**
   `DESIGN.md:143-147` and `Oyatie Console.dc.html:6596` specify a `border-top` dock band
   with hover peek, drag-out and 모두 닫기. The console shell has no dock band; building one
   is a screen-chrome slice, not a provider mount. Mitigated to "does not obscure the
   sidebar" via `trayStyle`. Anchor for the follow-up: `WindowFrame.tsx:191` `TrayDock`.
4. **Multi-pin stays out (D-4).** One `pinnedId`; the second open demotes the first to the
   tray. Popout / split presets / the 4-state engine at `/console-dev/window`
   (`useWindowEngine.ts`) remain the deferred platform decision.
5. **Body-level tests were rewritten, not deleted, and two changed meaning.**
   `ExploreBody.test.tsx` "keeps StrictMode roots ... isolated by explicit authority
   partitions" and `OntologyManagerBody.test.tsx` "uses the same explicit nested
   persistence partition across API recreation" asserted a property those bodies no longer
   own. Each is replaced by (a) a body-level assertion that it mounts **no host of its
   own**, and (b) the shell-level partition assertion in
   `consoleShellWindowHost.test.tsx`. No coverage was dropped; it moved to where the
   behaviour now lives.
6. **Files touched outside the declared roots**, unavoidable collateral of the mandated
   provider removal, both of them tests that asserted the removed provider:
   `web/src/console/screens/explore/ExploreBody.test.tsx` and
   `web/src/console/screens/ontology-manager/OntologyManagerBody.test.tsx`.
   No other wave-4 lane declares `screens/explore/**` or `screens/ontology-manager/**`.
7. **Spine defect found while running the gates (not this lane's, already fixed upstream).**
   At this lane's base `4cabe239`, 12 files imported `react-router-dom`, which is in
   neither `package.json` nor `package-lock.json`: `npx vite build` failed outright and 9
   test suites — including `ConsoleShell.test.tsx`, `nav.test.ts`, `registry.test.ts` —
   could not even be collected. The spine tip `8c3da1cd` ("merge(web): import router
   symbols from react-router, not react-router-dom") fixes it, which is why this lane
   plain-merged the spine before reporting green. **Any lane still based on `4cabe239` is
   measuring a tree whose web app does not build.**

## 7. Frozen contract for L-X10 ... L-X13

```ts
import { renderWithWindowManager } from "web/src/console/testing";

renderWithWindowManager(
  ui: ReactNode,
  options?: { authorityPartition?: string; retentionEnabled?: boolean },
): RenderResult
```

Name and signature are **frozen for the wave**. It uses RTL's `wrapper` option, so
`rerender` cannot silently drop the host. It deliberately does not render shell chrome:
a test that must prove the shell mounts a host renders `ConsoleApp`/`ConsoleShell` — see
`web/src/console/window/consoleShellWindowHost.test.tsx`.

Also available to those lanes, and the cheapest possible regression guard:
`document.querySelectorAll("[data-window-host]")` must have length **1** — in jsdom and
in the browser.
