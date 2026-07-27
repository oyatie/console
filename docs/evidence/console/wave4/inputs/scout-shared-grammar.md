# Wave-4 lens-B scout — shared console grammar machinery (exact APIs + port cost)

Worktree: `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync` (all paths below relative to `web/src/` unless absolute). Read-only survey, 2026-07-24.

Prior art consulted (do not re-derive): `docs/program/console-program-ledger.md` (lane-ownership + collision-file model, "add-a-type 6 manual steps" L209), `docs/program/audit-4-22-23.md` (runtime pin/drag adoption audit, §"Surface | Pin | Split | Tray | objDrag"), `docs/program/bestinclass-register.md` L11 (deployed module list = `nav.ts NAV_GROUPS` + `moduleScreens.ts MOD_SCREENS`).

## 0. Headline findings

1. **There are TWO window models in `console/window/`; only one is production.**
   - `WindowManagerProvider` + `windowModel.ts` + `WindowFrame.tsx` + `windowManagerContext.ts` — the **3-state** model (`default | pinned | minimized`, `windowModel.ts:10`). Popout is *deliberately absent* ("no-dead-affordance gate", `windowModel.ts:6-9`). Mounted once around the whole shell (`components/shell/AppShell.tsx:172-176`, `authorityPartition=windowAuthorityKey`, `renderTray=false` — ShellDock hosts `TrayDock`). This is what every adopted screen consumes.
   - `useWindowEngine` + `WindowEngine.tsx` + `types.ts` + `geometry.ts` + `sanitize.ts` — the **4-state carbon-copy card engine** (grid / popout-float / pin-split / tray-minimize, per-screen `CardRegistry`, split ratio, budget heights; `window/types.ts:1-15`). Its ONLY consumer is the demo harness lazy-mounted in `AppRouter.tsx:67/460` (`WindowEngineHarness`). **No production screen uses it.** Lens-B lanes must NOT charter onto it.
2. **Zero grammar adoption in all 15 wave-2/3 module dirs.** `grep objDrag|useOptionalWindowManager|objectCardWindowEntry|usePinnedPanel` over `console/{people,sales,consulting,logistics,equipment,inventory,payroll,recruiting,org,evaluation,maintenance,field,notif,board,directory}` → **0 files each**. The port surface for lens B is exactly these bodies. (Consistent with `docs/program/audit-4-22-23.md` runtime FAIL on the Users table.)
3. **The "hardcoded code-prefix regex" is already fixed at the grammar layer** — `console/ontology/codeGrammar.ts` is the single dynamic source (see §5). What is STILL hardcoded per kind: composer `KIND_META` (tone/label, 11 kinds), objectcard `COMPOSER_KIND_TO_SLUG` (5 linkable slugs), and rich `ONT_TYPES` defs (6 types). Ledger L209's "regex triplicated" item is stale — cite `codeGrammar.ts:1-13` when chartering.

## 1. Exact integration contracts

### 1a. Window model (pin / tray / saved layout)

Provider is already mounted shell-wide (`AppShell.tsx:172`); a module body adds **no provider** — it only consumes.

```ts
import { objDrag, useOptionalWindowManager } from "../window";
import { objectCardWindowEntry } from "../objectcard";

const windowManager = useOptionalWindowManager();       // null-safe: unit tests / no shell
windowManager?.open(objectCardWindowEntry(descriptor, handlers));  // §4.7-3 default open gesture
```

- `WindowEntry` = `{ id, title, code?, render: () => ReactNode }` (`windowModel.ts:12-21`). `render()` **persists across screen changes** — must be self-contained (no closure over screen state that dies on unmount).
- Context API (`windowManagerContext.ts:5-28`): `open / close / minimize / restore / togglePin / register / stateOf / saveLayout / restoreDefault / setPanelWidth / pinnedId / minimizedIds / narrow / panelWidth`. `register` applies user-saved state without forcing a pin; `open` pins (demoting the previous pin to the tray, `WindowManager.tsx:168-178`).
- Geometry: desktop right pin 360–620px (default 420, `windowModel.ts:24-26`), <1024px → 42vh bottom sheet (`windowModel.ts:27-28`, `WindowManager.tsx:332-348`). Host body padding is reserved automatically (`WindowManager.tsx:324-330`).
- Persistence: localStorage key `oyatie.console.window.layout.v2.{authorityPartition}` (`WindowManager.tsx:36-45`); partition = session authority key from AppShell — modules never touch storage.
- Fallback when `windowManager === null`: render `ObjectCardModal` (Escape/backdrop close, `ObjectCard.tsx:895-929`) — exactly what `GenericModuleScreen.tsx:760` does for the type chip.

**Exemplar consumers (best-practice order):** `console/explore/ObjectExplorerScreen.tsx:778,868` (graph node → `open(objectCardWindowEntry(descriptor))`, every pill/card an `objDrag` source at :358,:464,:526); `console/modules/GenericModuleScreen.tsx:1009-1018` (row select → pin, split detail stands without a shell); `console/leave/LeaveConsole.tsx:581,644` (domain screen with local descriptor builder `ledgerDescriptor`).

### 1b. Drag-source / drop-target tokens (`window/objDrag.ts`)

- Source: spread `{...objDrag(code, title)}` on any chip/row/code label (`objDrag.ts:40-49`). Emits typed mime `application/x-mnt-objref` + text/plain token `[CODE title]`; sets `data-obj-code` for DOM-level PBAC reads.
- Target: `useObjectDrop({ onRef, canAccept? })` (`objDrag.ts:86-109`) — `canAccept(code)` is the PBAC deny hook; parse order = typed mime → bracketed token → bare code (`parseObjectRef`, :62-78).
- Recognition ≠ authorization: rendering/drop stays PBAC-gated downstream (`codeGrammar.ts:11-13`).
- Port cost per module: 1 line per draggable surface + 1 `useObjectDrop` per drop zone. New codes need **no objDrag/grammar edit** once their prefix is in the registry (§5).

### 1c. Drill-to-object-card with identity

Two tiers:
- **Descriptor tier (cheap, what modules use):** build an `ObjectCardDescriptor` (`objectcard/types.ts:91-108` — id, code, title, objectType, lifecycleState, properties, relations, lifecycle, history, actions) via the ready-made builders `rowCardDescriptor(type, row)` / `typeCardDescriptor(type)` (`modules/typeRegistry.ts:480/425`), then `open(objectCardWindowEntry(descriptor))`. `GenericModuleScreen` does this for free when a module rides it.
- **Governed tier (real REST):** `GovernedObjectCard` (`objectcard/wired.tsx:229-312`) — props `{ api, descriptor, handlers?, buildActionRequest?, onInstanceChange?, refreshEpoch? }`; wires action execute / lifecycle / links / revisions with an authority-fingerprint remount fence. PBAC actions are the fixed `OBJECT_CARD_ACTIONS` map (`objectcard/types.ts:143-152`) evaluated through the ambient `PolicyGateProvider`.
- Identity: `WindowEntry.id` = descriptor.id (row id), `code` = drag token code — so pin, tray chip, and drag payload all share one identity spine.

### 1d. Module-surface grammar (`console/modules/`)

`ModuleScreenConfig` (`modules/types.ts:287-323`) is the whole surface contract: `id/screen/route/navLabelKey/titleKey/objectKind/typeKey/codePrefix/emptyMode/policy/data/dataAdapter?/statbar/search?/list{columns,sharedTrack,keyboard,display?,laneGroupBy?}/detail{fields,linkChips,actions}/primaryAction?/rows`.

- Registration point: `MOD_SCREENS` in `modules/moduleScreens.ts:610-614` — today **3 hand-authored entries** (finance, asset, compliance). `getModuleScreen()` (:674-682) falls back to a **generic registry-derived surface** for any kind registered in the ontology (`genericModuleScreen`, :628-667 — renders frame/stats/empty, `blocked-until-backend`, no list endpoint yet).
- Live data: `ModuleDataAdapter` (`types.ts:274-283`) — `loadRows(ctx{api,signal,query,hasPolicy})`, `loadDetail(ctx{api,signal,row,hasPolicy})`, `renderCompose`, `executeAction`. Asset adapter (`moduleScreens.ts:214-338`) is the reference implementation (policy-gated parallel fetches, link chips, catalog-gated actions).
- Registry binding: `typeKey` binds `ONT_TYPES` (`modules/typeRegistry.ts:137` — finance_voucher, equipment, employee, approval, support_ticket, compliance_catalog_item) so column labels/variants, status-chip choices (`choiceStatus`), and kanban lanes (`laneGroupBy` → `propChoices`) derive from the type def (`GenericModuleScreen.tsx:978-1007`).
- Screen exposure is a **second, separate spine**: ConsoleShell screen key → body in `console/screens/registry.ts:45-78` (SCREEN_BODIES) + nav entry in `console/shell/nav.ts:151+` (NAV_GROUPS, role/feature gate) + ko.ts label keys. `ModuleFinanceScreenBody` (`screens/module-finance/ModuleFinanceScreenBody.tsx`) is the canonical wrapper: binds authenticated `api` + a role/feature `PolicyGateProvider` around `GenericModuleScreen` (the R4 "blank plane" lesson — no ambient gate ⇒ DENY_ALL ⇒ empty surface; Cedar is shadow-only, so gates are role/feature maps per body).
- Collision-file discipline (ledger §Parallelism): `ko.ts`, `AppRouter.tsx`/`nav.ts`, `MOD_SCREENS`, ONT_TYPES are never concurrently edited — lanes emit manifests; a serial wire-up applies them.

### 1e. Composer token grammar (`console/composer/`)

Public API (`composer/index.ts`): `TokenComposer`, `TokenText`, `parseTokenGrammar/serializeTokenSpans/detectActiveTrigger/computeDropdownPosition`, `filterCandidates` + `create{Person,Channel,WorkOrder}CandidateProvider`, `KIND_META/TONE/kindFromCode`. Token kinds: `mention | channel | codeLink` (`grammar.ts:24`). Code links resolve via the same dynamic codeGrammar; chip tone/label come from the hardcoded `KIND_META` (`objectKinds.ts:54-67`). Modules only touch this if they add a composer surface or want kind-specific chip tones for new codes.

## 2. Production-readiness per mechanism

| Mechanism | Tests | A11y | Responsive | Verdict |
|---|---|---|---|---|
| WindowManager (3-state) | `WindowManager.test.tsx` (373L), `AppShell.test.tsx` | panel `role=region` + `aria-labelledby` (`WindowManager.tsx:363`), tray `role=group` + per-chip `aria-label` (`WindowFrame.tsx:202-208`), labeled control buttons | 1024px breakpoint → 42vh sheet; width clamp | **Ready.** Popout intentionally missing — if wave-4 promises 팝아웃, that is a *new F-lane on the provider*, not per-module work |
| WindowEngine (4-state) | geometry/sanitize/useWindowEngine/persist tests (~500L) | harness-only | harness-only | **Not adopted anywhere real.** Treat as future replacement/reference; porting 13 modules onto it now = wrong target |
| objDrag | `objDrag.test.ts` (92L) | `draggable` + title tooltips at call sites; keyboard-only alternative is the click→pin gesture | n/a | **Ready.** HTML5 DnD has no touch path — known platform ceiling, same as messenger today |
| ObjectCard (3-layer) | `ObjectCard.test.tsx` (162L), `useObjectCard.test.ts` (224L), `wired.test.tsx` (729L) | `article` + `aria-label`, modal `role=dialog aria-modal`, sectioned headings | card is single-column; fine in 360-620px pin | **Ready** (governed tier incl. authority-remount fence) |
| Module surface | `moduleEngine.test.tsx`, `typeRegistry.test.ts`, `FinanceModuleScreen.test.tsx` (224L), `AssetModuleScreen.test.tsx` (193L) | table/lanes markup with keyboard nav (J/K/Enter config) | detail-pin-aware column wrap (`types.ts:73-81` r14 lesson) | **Ready**; per-module risk is only in each lane's adapter |
| codeGrammar | `dynamicGrammar.noCode.test.ts` + composer/messenger tests | n/a | n/a | **Ready**; fail-closed fallback set |

Hardening genuinely needed BEFORE 13 lanes depend on it: **none blocking** for the 3-state model + objDrag + ObjectCard + MOD_SCREENS. Two watch items: (a) `genericModuleScreen` rows stay `blocked-until-backend` until `GET /api/v1/ontology/instances?type=` lands (`moduleScreens.ts:626`, `typeRegistrySource.ts` header) — lanes that ride the generic surface get frame-without-rows; (b) each body must supply its own `PolicyGateProvider` (R4 trap) — charter text should mandate copying the `ModuleFinanceScreenBody` gate pattern.

## 3. Per-module port recipe (mechanical checklist for lens-B lanes)

For a module `X` with object kind `x_thing`, code prefix `XT-`:

1. **ONT_TYPES def** (if rich columns/choices wanted): add `x_thing` entry via manifest to L-Ontology-owned `modules/typeRegistry.ts` (props with nameKey/type/choices, codePrefix `XT-`). Skip → generic surface derives bare columns.
2. **ModuleScreenConfig**: author `xModuleScreen` (copy `assetModuleScreen` shape, `moduleScreens.ts:479-608`) — policy map, `data` endpoints, statbar (`requiresBackend:true`, never fabricated), search fields, `list.columns` (≤3–4 next to the pin; r14), `detail.fields/linkChips/actions`, `laneGroupBy` if kanban. Emit as manifest entry for `MOD_SCREENS` (collision file).
3. **DataAdapter**: `loadRows` (list endpoint → `ModuleRow` mapper), `loadDetail` (policy-gated parallel enrichment), optional `renderCompose`/`executeAction`. This is ~80% of per-module effort; everything else is config.
4. **Screen body**: `screens/…/XScreenBody.tsx` = `PolicyGateProvider(role/feature gate)` + `<GenericModuleScreen api config={xModuleScreen}/>` (copy `ModuleFinanceScreenBody`). Register in `screens/registry.ts` + `nav.ts` NAV_GROUPS + ko.ts keys (all via wire-up manifests).
5. **Drag sources**: rows/type chips come free from GenericModuleScreen (`:724,749,819`). Custom (non-generic) sub-surfaces: spread `objDrag(code,title)` per chip; add `useObjectDrop` on any relation/compose input.
6. **Drill-to-card**: free via GenericModuleScreen (`:1015 open(objectCardWindowEntry(rowCardDescriptor(type,row)))`). Custom bodies: build descriptor (local builder like LeaveConsole `ledgerDescriptor`) + `useOptionalWindowManager()` + modal fallback.
7. **Code prefix**: ensure backend object-type registry row carries `codePrefix:"XT-"` → auto drag/parse via `primeCodePrefixes` bootstrap (no FE edit). For offline/test floor optionally append to `FALLBACK_CODE_PREFIXES` (`codeGrammar.ts:16-19`, one literal). If the kind must be *linkable* from bare codes in the object card: add composer kind to `KIND_META` (`objectKinds.ts:54`) + slug row to `COMPOSER_KIND_TO_SLUG` (`objectcard/kinds.ts:50-56`) + `SLUG_META` label/tone (`kinds.ts:18-28`) — 3 small map edits, one lane should own all 13 at once.
8. **Tests**: config-shape test (like `FinanceModuleScreen.test.tsx`), adapter mapper test, one pin/drag interaction test.

Estimated per-module cost with a live list endpoint: config+body ≈ ½ day, adapter ≈ 1–2 days, tests ≈ ½ day. Without a backend list endpoint: steps 1/2/4 only, `emptyMode:"blocked-until-backend"`.

## 4. Code-prefix recognition — current state (question 4)

- **Where**: `console/ontology/codeGrammar.ts` — the ONE compiled source (`FALLBACK_CODE_PREFIXES` :16-19; `compile()` :44-63 builds code/refToken/partial regexes, longest-prefix-first). Consumers: objDrag, messengerModel, composeModel, canvas PredicateEditor (per file header + grep).
- **How it grows**: `primeCodePrefixes()` (:73-82, union-only, fail-closed) fed by the bootstrap fetch of `GET /api/v1/object-types` in `console/ontology/typeRegistrySource.ts` (`ingest()` primes grammar; also feeds `registeredObjectType` → generic MOD_SCREENS derivation).
- **Adding 13 modules' codes therefore costs**: 13 backend registry rows with `code_prefix` set (BE seed/migration) + optionally 13 literals in `FALLBACK_CODE_PREFIXES` for the offline floor. Drag/parse: zero further FE work. Kind-tinted chips + bare-code linkability: the 3 hardcoded maps in §3-step-7 (single small PR).

## 5. Chartering implications (lens B)

- Charter lanes onto **WindowManagerProvider + objectCardWindowEntry + MOD_SCREENS/GenericModuleScreen**, explicitly NOT `useWindowEngine` (harness-only).
- Keep the ledger's manifest/wire-up discipline: lanes own `console/<domain>/**` + their `screens/<domain>Body`; MOD_SCREENS/ko/nav/ONT_TYPES edits land via one serial wire-up.
- Two shared prerequisite mini-lanes to run FIRST: (a) prefix/kind maps batch edit (13 kinds × 3 maps + fallback literals + BE registry rows); (b) decide per module: hand-authored config+adapter (has endpoints) vs generic surface (blocked-until-backend) — the generic path needs the ontology instances list endpoint to show rows.
- If wave-4 scope includes 팝아웃/split-grid presets (the 4-state model), that is one foundation lane upgrading the provider (or promoting WindowEngine), sequenced before, not inside, module lanes.
