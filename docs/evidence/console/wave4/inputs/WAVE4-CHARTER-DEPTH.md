# WAVE 4 — DEPTH-FIRST CHARTER

**Supersedes** `WAVE4-CHARTER.md`'s §2 phases, §5 lane table, §8 deferrals and §9 open
list. Everything else in that document — §4 collision map, §6 DoD template, §7
truthfulness doctrine — remains normative and is **cited, not restated**, so it cannot
drift into two versions.

**Binding input:** `DECISIONS.md` (2026-07-25). D-0 (depth-first wins), D-1 (one module
to EXPOSED with evidence), D-2 (plain-merge train), D-3 (8 worktrees approved), D-4
(narrow window fix), D-5 (dead code = delete), D-6 (statutory-registry location deferred).

**Doctrine:** `docs/intent/console-north-star.md` + the beyond-prototype amendment
(lenses A/B/C/D, "prototype fidelity is a floor, not a ceiling", the truthfulness line) +
`design-intent-register.md` — of which **CRM-1…CRM-6 and WFL-9 are the operative
`[>190]` set** for this wave, and **C-64 `[>190]` is now obeyed rather than waived.**

**Spine:** PR #488 · integration worktree
`/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync` · branch
`wave23-consolidation-20260724`.

**Lane count: 19** — 5 Phase-0 foundations + 14 Phase-1 CRM lanes.

---

## 0. §0 corrections — three charter facts are now stale, verified 2026-07-25

Re-verified at HEAD `4cabe239` ("test(notif): assert the ADR-0025 invariant, not an
exposure snapshot"), branch `wave23-consolidation-20260724`. **Universal DoD §6.1-0 still
applies: re-run your lane's rows before writing code.**

| Charter §0 claim | Verified NOW | Command |
|---|---|---|
| `EXPOSED_SCREEN_KEYS` `nav.ts:134` → `["sales"]` | **WRONG — it is `[]` at `nav.ts:135`**, on this branch *and* on `origin/codex/operational-object-runtime-progress`. `nav.test.ts:39,57` assert `toEqual([])`. It was emptied by **`b9e7fd74 fix(console): fail closed without exposure evidence`**. | `grep -n EXPOSED_SCREEN_KEYS web/src/console/shell/nav.ts`; `git log -6 -- web/src/console/shell/nav.ts` |
| `check:console-truth-ledger` is on the spine, not this branch | **WRONG — it is on BOTH** (`package.json:89`). The gate is CI-blocking against every wave-4 lane **today**, not after the next merge. | `grep -n truth-ledger package.json` |
| Tree DIRTY with the in-flight openapi agent's files | **Clean.** `git status --short` is empty. The openapi work is committed. | `git status --short` |
| Migration high-water `0202`, `0201` absent | **Confirmed.** `0201` is a reserved gap (the evidence-retention subject). **No depth-first lane may take 0201.** | `ls backend/crates/platform/db/migrations \| tail -3` |
| `WindowManagerProvider` absent from `ConsoleShell.tsx` | **Confirmed — zero hits.** Nested provider confirmed at `OntologyWorkspaceBody.tsx:12,299,391`. | `grep -c WindowManager web/src/console/shell/ConsoleShell.tsx` |
| `ObjectCard.tsx:779` is a `<span {...objDrag(…)}>` | **Confirmed, byte-exact.** `objectCardWindowEntry` confirmed at `ObjectCard.tsx:863`. | `sed -n '779p;863p' web/src/console/objectcard/ObjectCard.tsx` |
| `nav.ts:236` dead `dispatch` affordance | **Confirmed** (now ~`:237`): a gated nav item for a screen in **neither** `MOUNTED_SCREEN_KEYS` **nor** `SCREEN_REGISTRY`. A live §6.1-4 violation. | `grep -n dispatch web/src/console/screens/registry.ts` → no hits |

**Why the first row reframes D-1.** The exposure entry does not exist and was *withdrawn
for exactly the reason D-1 addresses* — no evidence. So L-X14 is not "re-flip a flag": it
is the first evidenced exposure this program has ever produced. `sales` is already in
`MOUNTED_SCREEN_KEYS` (`nav.ts:93`), has a nav item (`:197`) and a registry body
(`screens/registry.ts:59`) — the DARK half is complete. The wave's user-visible
deliverable is the evidence chain, and the flag flip is its receipt.

---

## 1. Why depth-first changes the shape, not just the count

Five structural consequences, each of which removes a bottleneck the 60-lane charter had
to engineer around:

1. **The catalog train collapses from five links to two.** `L-A1 → L-X7` replaces
   `L-A1 → L-A3 → L-A4b → L-A7 → L-A9`. One version bump (`.1`), one digest, one
   allowlist row. The wave-long width-1 train owner disappears.
2. **The i18n collision root mostly evaporates.** Sales owns `web/src/i18n/salesCrm.ts`
   (57 lines, module-private) rather than living in `web/src/i18n/ko.ts` (28 hits/48h).
   Only strings that must live under `ko.console.*` go to the integrator manifest.
3. **`backend/app/BUCK` stops being a 20-lane bottleneck.** Five backend lanes, five
   `manifests/buck-app-test.json` rows, one drain. **L-P0-BUCK is dropped** — the
   `--only <crate>` and glob-target deliverables existed to serve a fan-out that no
   longer exists, and the lane was gated on three unreleased buck branches.
4. **The FE fan-out gate disappears.** L-B21F existed to validate a port recipe for 13
   followers. With one module, the recipe *is* the module; recipe validation happens in
   L-X10's review, not in a preceding gate lane.
5. **C-64 is satisfied literally, not by reading.** No waiver is written (D-0). The
   §4-25 closed loop and the §4-21 benchmark pass (§6.3-18) become a real gate on one
   module rather than a promise repeated fourteen times.

---

## 2. Phase 0 — foundations (5 lanes)

Kept only where **blocking** or where they **repair a verified defect**. Every Phase-0
lane below also benefits the 13 undeepened modules; nothing is kept solely for them.

### L-P0-EPOCH · program/integrator, t0, serialized before every code lane

Rescoped to what depth-first still needs. **Dropped from its 60-lane scope:** normalizing
60 lane ids, seeding six of the seven CAP rows, the `CAP-DOCS-EVIDENCE-CONSOLE`
`frontend_roots` correction (its consumer lanes are gone).

- **Registry correction** (`docs/program/console-capability-registry.json`): seed
  **`CAP-SALES-CRM`** with roots verified on disk (`backend/crates/sales/**`,
  `web/src/console/sales/**`, `web/src/i18n/salesCrm.ts`); seed
  **`CAP-ONTOLOGY-ENGINE`** (L-A1, L-X7); refresh `source_revision` (pinned at
  `origin/main@8e42b9a2`, ~900 commits stale, while `last_refreshed` claims 2026-07-25);
  carry the §6.1-3 `hold_rule` amendment (a gap-manifest entry with a register anchor +
  a named missing backend contract satisfies it).
  **Records the sales inconsistency the scout found:** sales was EXPOSED once with zero
  registry provenance. The `CAP-SALES-CRM` row exists *before* L-X14 can request a flip.
- **Migration-slot ledger** (`docs/program/migration-slots.json`, new, append-only,
  integrator-owned). Seeded with §5's assignments. **`0201` is recorded as RESERVED and
  unavailable.** Slots are *requested*; no lane runs `ls`.
- **Evidence base committed** to `docs/evidence/console/wave4/`:
  `research-statutory-params.md`, `fidelity-registers.json`, `depth-registers.json`,
  and this charter. Every DoD anchor becomes a repo-relative JSON pointer. Live-fetched
  parameters (L-X8 only) append to the committed brief, producing a reviewable diff.
- **`backend-blocked-index.json`** — generated, one row per backend-blocked fidelity
  finding → owning lane | deferral id. **Kept, and more important than in the 60-lane
  charter:** dropping 13 modules makes the ledger of what is *not* being done the wave's
  main honesty artifact.
- **Epoch-contract amendment (D-2), written into
  `docs/program/console-fanout-epoch-contract.md:115-118`:** the admission train is
  **plain `git merge origin/<spine>` before push**. Rebase is classifier-blocked on this
  spine; the rebase/cherry-pick clause is struck, not carried forward and not escalated.
  Migrations, BUCK, clients and openapi remain integrator manifests (contract line
  126-127 satisfied). **No lane may rebase.** This is the line that makes all 19 lanes
  admissible.

roots: `docs/program/**`, `docs/evidence/console/wave4/**` · must_not_touch: all code.

### L-F1 · Window host — mount, tray-restore, nested-provider repair *(D-4)*

The window model is **dead in production**: `WindowManagerProvider` exists only in the
legacy `AppShell`, in tests, and as a nested mount inside `OntologyWorkspaceBody`. All 15
module bodies run with no provider.

- Mount `WindowManagerProvider` in `ConsoleShell.tsx`.
- **Remove the nested provider** at `OntologyWorkspaceBody.tsx:299` (or explicitly
  reconcile it), with one test proving a **single tray and a single layout partition**
  across the ontology workspace and one module screen. A shell-level mount over a nested
  one produces a second partition; both trays would be real and neither complete.
- **Tray-restore contract** — closes the single-pin data-loss path: `pin()` pushes the
  previous `pinnedId` into `minimizedIds` (`WindowManager.tsx:168-178`), so a second
  `open()` visually loses the previous card. Restore is tested, with a Korean accessible
  name on the tray chip. **Not multi-pin** — the 4-state provider stays deferred (D-4);
  the five registers' pin/popout/tray/preset blockers are gap-manifested with anchors.
- Create **`renderWithWindowManager`** (zero hits repo-wide today) and **freeze its name
  and signature** for the wave — L-X10 through L-X13 assert through it.
- `saveLayout()` / `restoreDefault()`: wire both controls into the console window layer
  **or delete the API and its `ko.ts:1808-1809` keys** (D-5 default is delete). Either
  way no documented control lacks an implementation.
- Named gap-manifest line for the `localStorage` layout ceiling
  (`oyatie.console.window.layout.v2.*` violates do-not-ship ban #9) with its six §4.7-2
  anchors, plus a `ponytail:` comment naming the server-persisted upgrade path.
- **§6.3-22 real-browser proof is mandatory**: assert the provider in the **live React
  tree** at the dark `/console` route, not in the jsdom harness. jsdom is the harness
  that hid this bug.

roots: `web/src/console/shell/ConsoleShell.tsx`, `web/src/console/ConsoleApp.tsx`,
`web/src/console/window/**`, `web/src/console/screens/_ontology/OntologyWorkspaceBody.tsx`
(provider removal only) · must_not_touch: `web/src/console/objectcard/**`,
`web/src/console/sales/**`, `nav.ts`, `screens/registry.ts`, `web/src/features/workspace/**`.
**Ownership must be confirmed against three live local heads before start** (§7).

### L-F2 · Shared object-card a11y + signature freeze

Two defects in shared code that every drilling module inherits:

- **`ObjectCard.tsx:779` — `<span {...objDrag(descriptor.code, descriptor.title)}>`.**
  A drag host with no role, no tab stop, no keyboard activation, **inside the shared
  card**. → focusable `<button type="button">` with a Korean accessible name, wrapped in
  `PolicyGated`. Fixing it here is the root-cause fix: 13 module inheritances collapse
  into one diff.
- **`ObjectCardModal` (`ObjectCard.tsx:895-913`) has no focus trap and no initial
  focus.** It is `role="dialog" aria-modal="true"` with only an Escape handler. → initial
  focus on open, trap while open, focus **returns to the invoking control** on close.
- **Freeze `objectCardWindowEntry`'s public signature and descriptor type for the wave**
  (`ObjectCard.tsx:863`), proven by a type-level test. L-X10 imports it from
  `console/objectcard`, never from `console/window`.

roots: `web/src/console/objectcard/{ObjectCard.tsx,ObjectCard.test.tsx,index.ts}` ·
must_not_touch: `objectcard/kinds.ts` (L-F3's), `console/window/**`, `ConsoleShell.tsx`.

### L-F3 · Code-grammar unification — one dynamic source, four call sites

`console/ontology/codeGrammar.ts` is already the single dynamic prefix source
(`primeCodePrefixes` :75-88; union, never replace). Three per-kind maps and one regex
still bypass it, so every new object kind costs four frontend edits:

| Call site | Divergence |
|---|---|
| `composer/grammar.ts:36` `BARE_CODE_RE` = `/(^\|[\s([{])([A-Z]{1,8}-[0-9]{1,10}(?:-[0-9]{1,6})?)/gu` | Never imports `codeGrammar`. **Digit-only body** — `OT-FINANCE`, `PAY-CHO`, `EV-2026-00012` drag but do not linkify. **Uppercase-only prefix** — `Bid-…` can never match, though `Bid` is in `FALLBACK_CODE_PREFIXES`. |
| `composer/objectKinds.ts:52` `KIND_META` | 11 hardcoded kinds (tone/label) |
| `objectcard/kinds.ts:18` `SLUG_META` | hardcoded label/tone |
| `objectcard/kinds.ts:50-56` `COMPOSER_KIND_TO_SLUG` | 5 linkable slugs; a missing row makes `linkTargetFromCode` return `undefined` |

**The hook already exists and is unused:** `objectCodeBodySource()` is exported at
`codeGrammar.ts:92` — *"for consumers that build their own combined pattern"* — with
**zero consumers repo-wide**. Pointing the four call sites at it is the whole fix.

DoD adds: a test that a prefix primed at runtime (not in `FALLBACK_CODE_PREFIXES`)
linkifies from bare text in the composer **and** resolves a card slug — i.e. that L-X7's
`DL-` prefix costs the frontend **zero** map edits. That assertion is L-F3's proof of
value and L-X10's precondition.

roots: `web/src/console/composer/{grammar.ts,objectKinds.ts}`,
`web/src/console/objectcard/kinds.ts`, `web/src/console/ontology/codeGrammar.ts` ·
must_not_touch: `ObjectCard.tsx`, `console/window/**`, `console/sales/**`.

### L-A1 · Catalog additive-upgrade path — **KEPT, and it is blocking**

**The conditional in the task resolves to KEEP.** Verified: the 27 seeded types are
`customer` (CU-), `site` (SI-), `equipment` (FL-), `work_order` (WO-), `contract`, … —
**no `deal`, no `sales_listing`, no `customer_inquiry`.** CRM-1 requires the `DL-` deal
type to join the ontology, so L-X7 must register at least one new type, and:

- `install_builtin_catalog` requires an exact digest match against the migration-owned
  allowlist (`0165_ontology_object_type_key_revisions.sql:121-131`, single row
  `e2b5fdff…3247`, enforced at `0165:1113-1124`);
- **two fail-closed guards with no upgrade path** (`0165:1128-1143`): a tenant on a
  *different* catalog version raises `different_catalog_already_installed`; a tenant with
  **any** existing `ont_object_types` row raises `empty_org_required`;
- **there is no catalog-upgrade function in the repo.** Every already-seeded tenant is
  frozen at 27 types.

So L-A1 ships the idempotent additive install: new keys only, existing keys untouched,
digest chain allowlisted, `BUILTIN_CATALOG_VERSION` `2026-07-19.1 → .2`. Folded in from
the 60-lane L-A8: the `validate_draft` allowlist fix, validating `backing_table` against
`allowlisted_projected_table` at draft time (`adapter-postgres/src/lib.rs:1080`), and the
stale `create_action` doc comment at `seed.rs:100-104`.

The ontology crate is **not codex-hot** — two merge commits in 48h, no content commits
since 2026-07-23. A lane can own `backend/crates/ontology/**` cleanly.

roots: `backend/crates/ontology/adapter-postgres/src/{seed.rs,lib.rs,instances.rs}`,
`manifests/migration/0203_ontology_catalog_additive_upgrade.sql` ·
must_not_touch: `backend/app/**`, `backend/crates/sales/**`, `openapi.yaml`,
`seed.rs`'s type drafts (L-X7 adds those, on top, same owner, strict order).

### Phase-0 lanes DROPPED, with reason

`L-P0-PARAM` (fan-out statutory sweep — only one CRM parameter remains; L-X8 fetches it
directly) · `L-P0-ID-a` / `L-P0-ID-c` (a 14-module triage and a registry-authority
decision blocking only the deleted L-A9; the CRM classification is three lines inside
L-X7) · `L-P0-BUCK` (§1-3) · `L-A4a` (projected-receipt design gate — its consumer L-A4b
is out) · `L-B23` (dispatch dual-impl decision — no dispatch lane) · `L-A6` (legacy
`object_types` rows for 13 kinds — L-X7 adds the CRM rows only) · `L-C8` (lint-gate
extension — the gate is already live and already runs against L-X13) · `L-D0` / `L-D0b`
/ `L-D3` / `L-D12` (see §8: **L-D3 and L-D12 are live defects and are named there, not
buried**) · `L-C1` **is not dropped — it moves into Phase 1 as L-X9**, because its only
wave-4 consumer is the CRM list.

---

## 3. Phase 1 — CRM (sales) to genuine production depth (14 lanes)

### 3.0 What exists today, and what "depth" therefore means

`backend/crates/sales` is a complete four-crate quartet (2,494 lines) — but it is a
**forklift sales *catalog***, not a CRM: `sales_listings` (0043), `sales_listing_media`,
`customer_inquiries` (NEW → CONTACTED → CLOSED). Four admin routes, four storefront
routes. RLS FORCE + `org_isolation` on all three tables; audit snapshots deliberately
PII-light.

`web/src/console/sales/SalesCrmScreen.tsx` (377 lines, 548 lines of tests) is a listings
panel + inquiry inbox + detail aside. Its own docstring is the honest floor:

> *"It does not invent a customer master, opportunity pipeline, quote, owner assignment,
> or branch scope that the backend does not expose."*

**That honesty is exactly the gap CRM-1…6 name.** Depth for this module is therefore not
polish — it is building the deal pipeline the authority specifies, to production
standard, and then taking the resulting surface through lenses B, C and D.

The screen's verified lens-B/C defects, each of which a lane below owns:

| Defect | Location | Lane |
|---|---|---|
| Zero grammar adoption — no `objDrag`, no `useOptionalWindowManager`, no `objectCardWindowEntry`, no `data-obj-code` | whole file (scout: 0 hits in all 15 module dirs) | L-X10 |
| Listings render as an inert `<ul>` — not selectable, no drill, no card | `:325-333` | L-X10 |
| Offset pagination `{limit, offset}` on both streams; unbounded growth | `:172`, `:209` | L-X9 |
| Inquiry list is a `role="listbox"`, not an ARIA grid; no PageUp/PageDown, no column semantics | `:348-361` | L-X11 |
| No 409/412 handling — `result.error \|\| !result.response.ok` → one generic `actionFailed` | `:286-289` | L-X12 |
| No idempotency key on the mutation; a retry after a timeout may double-write | `:279-282` | L-X12 |
| The status FSM is **client-side** (`NEXT_STATUS` map at `:42-46`) | `:42-46` | L-X2 |
| Explanatory UI: `sales-kicker` + `sales-muted` description paragraphs — a **binding merge-gate breach** already in the tree | `:317-319`, `:338` | L-X13 |
| Locale hardcoded `Intl.DateTimeFormat("ko-KR")` inline | `:51` | L-X13 |
| Public storefront collects `name` + `phone` with **no consent record and no retention clock** | `0043:98-111` | L-X8 |

### 3.1 Lens A — ontology projection + §18 dispatch (2 lanes)

**L-X7 · Register the CRM object types as ontology projections + prove §18 dispatch
through a domain use-case.**

The scout's finding the charter told us to exploit: **`instance_acting` and
`object_type_acting` are already live routes** (`ontology/rest/src/lib.rs:1562-1588`), so
the dynamics-layer findings are unblocked **by registration alone** — no new endpoint, no
openapi change (the 12 ontology routes are already documented at
`openapi/openapi.yaml:11951-12996`).

- `projected_draft` builders + `*_KEY` consts for **`deal`** (semantic: name, amount,
  currency, expected close, stage, owner, inflow provenance; kinetic: the stage lifecycle;
  dynamic: activity recency, weighted amount), and for **`sales_listing`** and
  **`customer_inquiry`** as projections of their existing tables.
- `PROJECTED_DOMAIN_KEYS` + the `drafts` vec in `builtin_catalog_manifest`
  (`seed.rs:1178-1207`); **`allowlisted_projected_table`** (`instances.rs:955-974`) —
  without this the type registers and then 400s on every list.
- **Links** via `fk_link` with `to_stable_key` strings only — the installer **forbids
  physical link ids** (`0165:1153-1157`). deal→`customer` (account), deal→`contract`
  (converted), deal→`sales_listing`, `customer_inquiry`→deal (provenance).
- **The governed Action, and the §18 proof:** one `ActionTypeInput` with
  `dispatch: ProjectedUsecase`, `dispatch_target = "sales.advance_deal_stage"`,
  `submission_criteria: []` (R4 hard-rejects anything else for projected types),
  `control_points` = `authority` + `self_checklist`. Plus a `ProjectedHandler` + one
  `register(...)` line + a `sales_error_to_action_error` shim in `backend/app/src/lib.rs`
  (~35 lines, the pattern of `update_equipment_projected_handler` at `:2478-2523`).
  **This makes the first seeded projected action in the repo's history** — today
  `seed.rs:121` hardcodes `dispatch_target: None` and `projected_draft` ships
  `actions: Vec::new()`, so the one wired handler is unreachable dead code (R1/R3).
- **Legacy-registry row + code prefix**: an `object_types` row `kind='deal'`,
  `code_prefix='DL-'` and a code counter, so `GET /api/v1/object-types` primes
  `codeGrammar` at runtime and L-F3's zero-FE-edit assertion holds.
- **`RESOLVABLE_KIND_AUTH`** (`app/src/objects.rs:121-139`) needs a `deal` row for card
  resolution. **This file is integrator-owned and security-reviewed** — emit a manifest,
  never an edit. Its own header records two prior Login-gated-only regressions, and
  adding a kind **retroactively makes pre-existing `object_links` of that kind resolvable
  with no backfill re-check** (`objects.rs:117-120`). The manifest must state the
  backfill answer explicitly.
- **Named, anchored deferrals — do not let these resolve into silence:** R4 (submission
  criteria hard-rejected for projected actions — a criterion would fail *open*), R5
  (projected dispatch gets `receipt: None`, `rest/src/lib.rs:840`), R6 (four-eyes for
  projected is consumed in a separate committed step; a failed dispatch spends the
  approval), R7 (TOCTOU disclaimed, left to the domain use-case), R8 (`resolve_by_code`
  only queries `ont_instances`, so a `DL-` code on a projected type cannot resolve).
  **R6's consequence is binding: no CRM action may require a four-eyes-gated projected
  action this wave.** R8's consequence: the card resolves via the legacy path (L-X10),
  and the ontology-side resolution is gap-manifested.
- **The largest integration gap, named not closed:** `useObjectCard.ts` reads the
  **legacy** stack (`/api/v1/object-links`, `/api/objects/{kind}/{id}`,
  `/api/v1/lifecycles/…`, a raw `fetch` to `/api/audit`) while `ObjectExplorerModel.ts`
  reads the engine. DN-0003 Slice 1's "one governed object card" is split across two
  backends. L-X10 consumes what exists; the split is a gap-manifest line with anchors.

deps: L-A1 (strict, same owner, L-A1 merges first), L-X2 (the action must target a real
use-case) · slot 0209 · PVL n · size L

**L-X6 · Derived pipeline analytics — no stored aggregates.**

CRM-1: *"all stats derived (weighted pipeline = Σ size × win-rate)"*. Server-computed
over real deal records: weighted pipeline, per-stage win rate, conversion rate, cycle
time (created→won, and per-stage dwell from L-X2's transition history), inactivity
distribution from L-X3.

DoD lines that make this truthful, from §7.4: **`total: Option<i64>`, never a fabricated
count**; a stat with no underlying rows renders the honest-empty pattern
(`web/src/console/leave/model.ts:199-203`), never a zero presented as a measurement; no
stat is stored and re-served — recomputation from the same inputs must produce the same
number, asserted by test. Analytics reads are already policy-filtered (`bae882bf`) and
`AnalyticSummary` needs no per-type registration.

deps: L-X2, L-X3 · slot — · PVL n · size M

### 3.2 Lens D — CRM business-logic depth (4 lanes, plus the trunk)

**L-X1 · Deal aggregate — the trunk.**

New domain aggregate in the existing `backend/crates/sales` quartet (**no new crate** —
§4-W forbids it, and a new crate forces a `gen_first_party.py` regen). `deals`,
`deal_activities`, `deal_stage_transitions`. Stages are **lifecycle flow lanes**, and per
CRM-1 **Lost is explicitly NOT a stage** — it is a closed outcome with a reason (L-X3),
which is why the stage column and the closure columns are separate from the start.
Inflow provenance is a link to the source event (a `customer_inquiry`, a renewal, a
manual entry), never a free-text "source" string.

Full DoD: RLS `FORCE` with `org_isolation` matching `0043:49-51`, **every assertion
executed as `console_rt`** (bootstrap with `console_buck_admin` + the `mnt.sqlx_test_bootstrap`
GUC), deny-by-default PBAC, audit row **in the same transaction** as every mutation,
canonical envelope `{error:{code,message}}` with 422/409, first-outcome idempotent
replay, one story-level app integration test as `console_rt`, `manifests/buck-app-test.json`,
`buck2 test //backend/crates/sales/...` green, and the four unconditional CI gates
(`rls-arming`, `tenant-isolation`, `audit-coverage`, `dev-auth-absence`).

**Edge-case matrix (§6.4-27), CRM-specific rows** plus the five universal categories:
a deal whose account is soft-deleted mid-pipeline; an owner who leaves the org while
holding open deals; amount currency change after a stage advance; a deal created from an
inquiry that is subsequently CLOSED; concurrent stage advance from two sessions;
month-end/회계연도 boundary on cycle-time and forecast attribution (KST, DST-free).

deps: — · slot 0204 · PVL n · size L

**L-X2 · Stage transitions as audited actions with a per-stage evidence enum,
fail-closed** *(CRM-3)*.

The transition FSM moves **server-side** — today it is a `NEXT_STATUS` map in the
browser. Each target stage declares its required evidence kind(s) as an enum; a
transition without conforming evidence is **refused (422), not warned**. Every transition
writes a `deal_stage_transitions` row and an audit row in the same tx. `If-Match` /
`expected_revision` → **412** on a stale write; same idempotency key + matching
fingerprint → the **first outcome**, no second write and no second audit row; same key +
different fingerprint → **409**. Both branches tested as `console_rt`.

This lane also fixes the inquiry FSM it supersedes: `NEXT_STATUS` is deleted from the
frontend and the NEW→CONTACTED→CLOSED transition becomes a server-owned action with the
same envelope, so the exposed screen stops carrying a private copy of the domain rule.

deps: L-X1 · slot 0205 · PVL n · size L

**L-X3 · Activity discipline + Closed-Lost reason enum** *(CRM-2 — promoted from the
60-lane charter's wave-5 list, per the founder's lens-D brief)*.

- *"No next activity"* is a **danger state**, computed as a **pure query** — deals whose
  latest activity timestamp is older than the policy threshold. **§6.4-29 grep gate
  applies: no `tokio::spawn`, no `LISTEN`, no `NOTIFY` for staleness detection.** No
  listener, no daemon, no missed-message failure mode.
- Inactivity statistics feed L-X6.
- **Closed-Lost requires a reason from an enum**, and the enum is queryable so monthly
  pattern analysis is real aggregation over real rows, not a free-text word cloud.
- The **deterministic auto-Lost policy is a settings object** (Palantir flavor, C-64):
  an effective-dated, audited configuration row that a privileged principal edits through
  the console — not a constant, not an env var. Flipping the row changes the outcome with
  no code change, asserted by test.
- **Reversal path (§6.4-27):** a Closed-Lost deal can be reopened; the reversal is a
  transition with its own audit row and the original linked. Auto-Lost must be
  reversible or it is a data-loss mechanism.

deps: L-X1, L-X2 · slot 0206 · PVL n · size M

**L-X4 · Deterministic round-robin assignment** *(CRM-5 / WFL-9 — promoted from wave 5)*.

Trigger = deal created without owner. Condition = inflow ≠ renewal (renewals belong
deterministically to wf9 owner-succession — **no rule overlap**, and a test asserts a
renewal never enters the wf10 rotation). Action = **fixed rotation against a real
rotation roster, executed server-side**: same input produces the same output, asserted by
a replay test. A first-activity deadline is set on assignment and feeds L-X3's danger
state. The run log is real.

Also owns **bulk owner reassignment** (the rep-departure workflow), because assignment
has exactly one owner: per-item audit rows, a partial-failure summary in the canonical
envelope — never silent all-or-nothing, never a success toast over a partial write.

*Bulk stage advance is deliberately NOT built*: L-X2 requires per-transition evidence, so
a bulk advance would either fabricate evidence or fail per item. Named as a deferral with
its anchor rather than shipped as a disabled control (§6.1-4: deny-by-omission).

deps: L-X1 · slot 0207 · PVL n · size M

**L-X5 · Won deal → contract conversion through the guarded composer** *(CRM-6)*.

A WON transition (L-X2) auto-converts to a `C-` contract through the **guarded composer**,
joining the `Bid-` chain. `contract` is already a seeded ontology type, so this is a link
plus a governed action, not a new type. **Dedupe is structural** — a uniqueness
constraint on (deal, contract), so a replayed conversion returns the first outcome rather
than creating a second contract. The large-deal manager alert threshold is a **settings
object**, effective-dated and audited, not a literal.

**Truthfulness line (§7.2):** the conversion creates a *draft* contract object. Contract
execution, counterparty signature and any 전자세금계산서 relay remain **gated
attestations** — a lane that computes one has failed. VAT arithmetic is explicitly out of
scope this wave and anchored to the deferred finance lanes.

deps: L-X2 · slot 0208 · PVL n · size M

### 3.3 The CRM regulatory-parameter lane (1 lane, `param_verify_live: true`)

**L-X8 · Lead PII: consent record, retention clock, masking, audited sensitive view.**

`customer_inquiries` stores `name` + `phone` collected from a **public, unauthenticated
storefront form** (`0043:98-111`), with a migration comment acknowledging
개인정보보호법 — and **no consent record, no purpose statement, no retention period, no
destruction path** anywhere in the schema or the crate. This is the one genuinely
regulatory surface in CRM, and it is the one the exposure lane will be asked about.

- **`param_verify_live: true`.** Live-source, with `source_url` + `retrieved_on` NOT
  NULL, appended to the committed brief at
  `docs/evidence/console/wave4/research-statutory-params.md`: 개인정보 보호법 §15
  (수집·이용 근거), §22 (동의를 받는 방법), §21 (파기), and 정보통신망법 marketing-consent
  rules where a lead is later contacted for marketing. **Every statutory constant carries
  its citation in the test name; no `unwrap_or` on a parameter resolve; an unverified
  parameter resolves to `Err(ParamUnverified)`, never a default.**
- **D-6 interaction, stated so it cannot become a second registry.** The statutory
  registry (`kernel/core::statutory`) is not in this wave. L-X8 therefore stores its
  retention periods as **effective-dated rows in its own table** with `source_url` +
  `retrieved_on` NOT NULL, plus a `ponytail:` comment naming the upgrade path. **Forward
  obligation, written into the ledger: when L-D0 lands, it MIGRATES these rows — it does
  not re-fetch them.** Two independent live fetches of the same gazetted parameter is the
  exact divergence L-D0 exists to abolish.
- **Masking defaults + audited sensitive view** (north-star amendment §3, §4-27): the
  console masks `phone` by default; unmasking is a **server-side view event on the read
  path**, audited with actor and reason. No frontend lane may implement this — it is a
  backend obligation, which is why it lives here and not in L-X13.
- Destruction on purpose-fulfilment / retention expiry is a real job with a real test,
  not a documented intention.
- The existing PII discipline is preserved and asserted, not re-derived: audit snapshots
  stay PII-light (`adapter-postgres/src/lib.rs:363`, `sales_store.rs:162`), handlers never
  log the values, and the `pii-no-logs` CI gate is literal-only so the test carries the
  real assertion.

deps: L-X1 (deal↔lead link) · slot 0210 · **PVL y** · size M

### 3.4 Lens C — beyond-prototype depth (4 lanes)

**L-X9 · `kernel/core::paging` extraction + sales keyset adoption.**

The CRM list can grow unbounded (a real pipeline plus every public inquiry ever
submitted); the exposed screen uses `{limit, offset}` on both streams. A second cursor
encoding is **forbidden** — the spine landed an opaque base64url keyset cursor with
snapshot stability and round-trip tests over 13 commits
(`EvidenceObjectCursor`, `backend/crates/docs/application/src/lib.rs:48`).

So this is an **extraction, not a new primitive**, and its first DoD line is *generalize
the landed cursor — no new encoding, no new error strings* — proven by the docs REST token
round-tripping through the shared type with the **existing docs cursor tests passing
unchanged**. `kernel/core` stays **pure**: cursor codec, `ListPage<T>`, sort-key enum
contract, validation errors, one `base64` dependency line via the §4-D manifest. **No
SQL, no sqlx** — `backend/ci/gates/layer-boundary` enforces it and the crate's own
Cargo.toml says "Pure — no async, no sqlx, no axum". The predicate and snapshot query
stay inside each adapter.

Sales adopts it for listings, inquiries and deals. `total: Option<i64>` — absent rather
than fabricated. **`backend/crates/docs/**` is the §4-T docs owner's surface**: this lane
touches it only to delegate, and coordinates before it does.

**Virtualization is measurement-gated, not assumed** (the 60-lane L-C11 rule): L-X11
records real row counts; if the pipeline or inbox exceeds the threshold, virtualization
is a wave-5 lane **with the measurement attached**, not a dependency added on suspicion.

deps: L-P0-EPOCH only — no CRM dependency, so it runs at t1 **alongside** the Phase-0
F-lanes and is ready before L-X1's list surface needs it · slot — · PVL n · size M

**L-X10 · CRM console grammar port — codes, drag, window, 3-layer card, drill-with-
identity** *(lens B)*.

- Every deal row and every listing row renders a **`data-obj-code`** (`DL-…`) and a
  **`[draggable="true"]`** drag host that is a focusable `<button>` — never a `span`,
  `li` or `article` with `objDrag` spread on it (L-F2 fixed the shared instance; this
  lane must not reintroduce it locally).
- `useOptionalWindowManager()` + `open(objectCardWindowEntry(...))`, imported from
  **`console/objectcard`**. The lane touches nothing in `console/window/**`, does not
  promote `useWindowEngine`, and does not use `features/workspace`. **The §6.3-19 window
  contract paragraph is pasted verbatim into the PR body.**
- **Three unit assertions per ported surface** under `<WindowManagerProvider>` via
  `renderWithWindowManager`: (a) the row renders `[draggable="true"]` **and** the expected
  `data-obj-code`; (b) activation pins the target **and the previously pinned entry
  remains recoverable in the tray with a Korean accessible name**; (c)
  `getByRole("button")` resolves the drag host.
  **Bucket classification (replacing the deleted L-P0-ID-a, three lines):** `deal` has an
  issued code (`DL-`, L-X7) → full (a)/(b)/(c). `sales_listing` and `customer_inquiry`
  have **no code column** → **bucket-3 variant: assert (b) and (c) only**, and the missing
  code is a named gap-manifest line with its anchor. **No lane fabricates a code from a
  UUID.**
- **Drill-with-identity**, the C-3 chain 1-click traversable in the real console:
  deal → account (`CU-`) → contract (`C-`) → listing/equipment (`FL-`), with object
  identity carried end-to-end. The 3-layer object card opens for every noun.
- Listings stop being an inert `<ul>`: they become selectable, drillable rows.
- **§6.3-22 real-browser proof mandatory** — live React tree, one keyboard journey
  (open → pin → Escape → focus returns to the invoking control), one axe pass on the
  mounted window layer, trace committed.
- Object typeahead / search fabric is **gap-manifested, not built** — the contract is
  owned by `codex/console-search-object-fabric-20260724` (§7).

deps: L-F1, L-F2, L-F3, L-X1, L-X7 · slot — · PVL n · size L

**L-X11 · Pipeline as a real ARIA grid — full keyboard model + scale measurement**
*(lens C)*.

The inbox is a `role="listbox"` with roving tabindex and Arrow/Home/End — a reasonable
floor and the wrong structure for a multi-column pipeline. This lane builds the complete
keyboard model the amendment names: ARIA grid semantics with `aria-rowcount` (**`-1` when
`total` is absent**, never a fabricated count), roving tabindex across cells, Arrow /
Home / End / PageUp / PageDown, Escape semantics, explicit focus management on data
refresh (focus must not be lost when a page loads or a row is removed), and undo
semantics where a transition is reversible (L-X3).

Records the row-count measurement that gates virtualization (L-X9). Column sort headers
ship **disabled with a stated reason** on any endpoint that has not adopted L-X9's keyset
— **a client sort of page 3 of 40 presented as a sort is a defect** (§7.4).

deps: L-X10 · slot — · PVL n · size M

**L-X12 · Optimistic updates + conflict UX + draft autosave** *(lens C; absorbs the
60-lane L-C3, bound to CRM as its named first consumer)*.

- **409 / 412 surfaced as merge affordances, not toasts.** L-X2 emits a real 412 with the
  current revision; the UI shows what changed, preserves the user's draft, and offers
  reapply-or-discard. The user never retypes.
- **Optimistic update with a truthful rollback**: the row reverts and says why. Today a
  failure produces one generic `actionFailed` string for every cause.
- **Idempotency key generated client-side per mutation attempt**, so a retry after a
  timeout is safe against L-X2's first-outcome contract — the mobile-client failure mode
  the §6.2-13 rewrite exists to prevent.
- **Draft autosave + restore** for the deal composer. Extract the highest-value line from
  the legacy `features/workspace/persistence.ts` — the **save-disabled-until-load-succeeds
  guard** — without importing `features/workspace` (the universal FE DoD forbids it).
  Storage ceiling is a named gap-manifest line with a `ponytail:` upgrade path, same
  treatment as L-F1's.
- Skeleton loading and error-recovery paths that **preserve state**.

deps: L-X2, L-X10 · slot — · PVL n · size M

**L-X13 · Truthful states, AA a11y matrix, ko expansion, responsive, no-explanatory-UI**
*(lens C)*.

- **Empty / denied / error states name the reason and the next action** (§4-10), using
  the honest-empty pattern at `web/src/console/leave/model.ts:199-203`. The current
  `denied` state is a bare string. **No dead affordance**: a control that cannot act is
  omitted, not disabled (§6.1-4).
- **The no-explanatory-UI merge gate, applied to code already in the tree**: delete
  `sales-kicker` and `sales-muted` description paragraphs (`:317-319`, `:338`). Status is
  a **chip**, not colour, and informational chips are not focusable. Only action-driving
  copy survives. Benchmark against Palantir / Teams / Slack.
- **AA is a check, not a claim**:
  `CONSOLE_DEV_AUTH_E2E=1 npx playwright test --project=dev-auth e2e/specs/chrome-02-axe.spec.ts`
  over the surface, committed. Explicitly: 1.4.10 reflow · 2.4.7 focus visible · 1.4.11
  non-text contrast · 2.5.8 target size ≥44px · Korean accessible name on every icon-only
  control.
- **ko expansion tolerance** and locale honesty: the inline `Intl.DateTimeFormat("ko-KR")`
  at `:51` routes through the shared locale layer; no layout breaks at expanded string
  widths.
- **Responsive down to mobile employee-app widths**; body-level horizontal scroll is a
  defect class (C-42) — wide content scrolls inside its own container.
- **§6.3-18: the §4-25 eight-question closed loop and the §4-21 benchmark pass**
  (Palantir / Workday / Slack / Greenhouse / SAP), grounded in BENCHMARK.md's honest-gap
  column, run against the finished surface, with the ranked output committed as CRM's
  next-slice register. **This is C-64's actual requirement and the only lane in the wave
  that discharges it.**

deps: L-X11, L-X12 · slot — · PVL n · size M

### 3.5 The exposure lane (1 lane)

**L-X14 · ADR-0025 evidence chain for `sales`, and the flag flip** *(D-1)*.

**The bar is `console-enterprise-roadmap.md` §"Frontier 4: production exposure", not the
withdrawn 2026-07-23 precedent.** That precedent — four frontend-only commits, mount →
fence → flip → harden — is the template for the *mechanism* (the flip is one ~9-line
`nav.ts` diff plus tests) and **explicitly not for the evidence bar**: it shipped with no
`CAP-SALES-*` evidence directory, no registry row, and a stale design mirror, and
`b9e7fd74` withdrew it for precisely that.

Required, all committed under `docs/evidence/console/CAP-SALES-CRM/`:

1. **No unmounted nav, no hidden required workflow** on the exposed screen.
2. **Executable user-story replay in a real browser at the real route**, committed —
   the roadmap's eight scenarios: happy path · least-privileged view · **denial without
   leakage** · failure / retry / recovery · **lifecycle + audit readback** ·
   **cross-tenant isolation** · responsive + keyboard · non-regressing action count.
3. **Visual + a11y matrix green** (L-X13's axe run, over the final candidate).
4. **Buck2-only Rust build/test completion evidence** — `buck2 test` output, not cargo.
   Cargo is dependency metadata.
5. **Independent review satisfied** (§6.1-8) — a different agent, correctness +
   RLS-as-`console_rt` + codex cross-model + browser/a11y. Never self-approved.
6. **CI + immutable image authorization + deployment + live readback + rollback
   verified** — ops observation, not a plan.
7. **Truth-ledger admission** (§6.1-7): signed commits, the canonical receipt at
   `docs/evidence/console/reviews/CAP-SALES-CRM/<candidate_sha>.json`, registration
   through `scripts/console/plan-fanout.mjs`.

Only then: `EXPOSED_SCREEN_KEYS` `[] → ["sales"]` at `nav.ts:135`, `nav.test.ts:39,57`
updated from `toEqual([])`, and the ADR-0025 rationale comment rewritten. **Requested via
manifest; the integrator applies it. `nav.ts` is not a lane root.**

Also in this lane's manifests, because they are the same truthfulness problem:

- the **design-mirror + roadmap delta** — `docs/design/oyatie-console/ROADMAP.md:13` and
  `SYNC-MANIFEST.md:43` assert *"`EXPOSED_SCREEN_KEYS` empty"*, which is true today and
  becomes false on the flip. Integrator-owned (§4-L).
- the **dead `dispatch` nav affordance** at `nav.ts:~237` — a gated nav item for a screen
  in neither `MOUNTED_SCREEN_KEYS` nor `SCREEN_REGISTRY`. A shipped §6.1-4 violation, and
  it renders next to the screen we are exposing. Its removal needs no dispatch-lane
  verdict: the entry points at nothing.

> **The bar is not lowered to reach the milestone.** If any of items 1-7 does not hold,
> **the entry does not land**, the wave says so in this evidence doc, and the CRM work
> merges DARK. A flip without its evidence is the exact failure `b9e7fd74` already
> corrected once.

deps: L-X6, L-X8, L-X13 (and transitively all of Phase 1) · slot — · PVL n · size M

---

## 4. Canonical lane table

`PVL` = `param_verify_live`. `Slot` = provisional migration slot **from the append-only
ledger**, never from `ls`; the lane emits `manifests/migration/<slot>_<name>.sql` and the
integrator applies and renumbers. Every migration is **self-contained with no cross-lane
FK**, so renumbering is a filename change.

| Lane | Phase | Title | Deps | Slot | PVL | Size |
|---|---|---|---|---|---|---|
| L-P0-EPOCH | 0-t0 | Registry correction · slot ledger · evidence base · epoch amendment (D-2) | — | — | n | M |
| L-A1 | 0-t1 | Catalog additive-upgrade path + `validate_draft` fix **[train 1 of 2]** | EPOCH | 0203 | n | L |
| L-F1 | 0-t1 | Window host: mount, nested-provider repair, tray-restore, `renderWithWindowManager` | EPOCH | — | n | M |
| L-F2 | 0-t1 | Shared card a11y (`:779` span→button, modal focus trap) + freeze `objectCardWindowEntry` | EPOCH | — | n | S |
| L-F3 | 0-t1 | Code-grammar unification — four call sites → `objectCodeBodySource()` | EPOCH | — | n | S |
| L-X9 | 1-t1 | `kernel/core::paging` extraction (landed cursor generalized) + sales adoption | EPOCH | — | n | M |
| L-X1 | 1-t1 | **Deal aggregate — the CRM trunk** | EPOCH | 0204 | n | L |
| L-X2 | 1-t2 | Stage transitions: audited action, per-stage evidence enum, fail-closed *(CRM-3)* | L-X1 | 0205 | n | L |
| L-X3 | 1-t3 | Activity discipline · Closed-Lost reason enum · auto-Lost settings object *(CRM-2)* | L-X1, L-X2 | 0206 | n | M |
| L-X4 | 1-t3 | Deterministic round-robin wf10 + bulk owner reassignment *(CRM-5 / WFL-9)* | L-X1 | 0207 | n | M |
| L-X5 | 1-t3 | Won → contract `C-` via the guarded composer + large-deal threshold object *(CRM-6)* | L-X2 | 0208 | n | M |
| L-X8 | 1-t2 | Lead PII: consent, retention, masking, audited sensitive view | L-X1 | 0210 | **y** | M |
| L-X7 | 1-t3 | Ontology projections (deal/listing/inquiry) + §18 projected dispatch **[train 2 of 2]** | L-A1, L-X2 | 0209 | n | L |
| L-X6 | 1-t4 | Derived pipeline / conversion / cycle analytics — nothing stored *(CRM-1)* | L-X2, L-X3 | — | n | M |
| L-X10 | 1-t4 | CRM grammar port: codes, drag, window pin, 3-layer card, drill-with-identity | L-F1, L-F2, L-F3, L-X1, L-X7 | — | n | L |
| L-X11 | 1-t5 | ARIA grid + full keyboard model + virtualization measurement | L-X10 | — | n | M |
| L-X12 | 1-t5 | Optimistic + conflict UX (409/412 merge affordances) + draft autosave | L-X2, L-X10 | — | n | M |
| L-X13 | 1-t6 | Truthful states · AA matrix · ko expansion · responsive · no-explanatory-UI · §4-25 loop | L-X11, L-X12 | — | n | M |
| L-X14 | 1-t7 | **EXPOSURE — ADR-0025 evidence chain + the `nav.ts` flip** | L-X6, L-X8, L-X13 | — | n | M |

**Migration-slot ledger** (`docs/program/migration-slots.json`, append-only,
integrator-owned): **0201 RESERVED — unavailable** · 0203 L-A1 · 0204 L-X1 · 0205 L-X2 ·
0206 L-X3 · 0207 L-X4 · 0208 L-X5 · 0209 L-X7 · 0210 L-X8.

### Roots and `must_not_touch` — disjointness contract

Phase-0 roots are in §2. Phase-1:

| Lane | roots | must_not_touch |
|---|---|---|
| L-X1 | `backend/crates/sales/{domain,application,adapter-postgres,rest}/src/**` + their `tests/**`; `manifests/migration/0204_*` | `backend/app/**`, `backend/crates/ontology/**`, `web/**`, `openapi.yaml` |
| L-X2 | same quartet (**serial after L-X1, same owner**) | as above |
| L-X3 / L-X4 / L-X5 | same quartet (serial, same owner) | as above |
| L-X6 | `backend/crates/sales/{domain,application}/src/**` (read-only over adapter) | migrations, `backend/app/**` |
| L-X7 | `backend/crates/ontology/adapter-postgres/src/seed.rs` (drafts only, **after L-A1**), `instances.rs` allowlist; `manifests/{kernel-core.json,migration/0209_*}`; `manifests/app-register.json` | `backend/app/src/lib.rs` (**manifest only**), `backend/app/src/objects.rs` (**manifest + security review**), `openapi.yaml` |
| L-X8 | sales quartet + `manifests/migration/0210_*`; `docs/evidence/console/wave4/research-statutory-params.md` (append) | `kernel/core/src/statutory*` (does not exist this wave — do not create) |
| L-X9 | `backend/crates/kernel/core/src/paging.rs` (new, pure); `backend/crates/docs/**` **delegation only, coordinated with the §4-T owner**; sales adapter call sites | any SQL in `kernel/core`; a second cursor encoding |
| L-X10 | `web/src/console/sales/**`, `web/src/i18n/salesCrm.ts` | `console/window/**`, `console/objectcard/**`, `console/module/**`, `console/modules/**`, `nav.ts`, `screens/registry.ts`, `features/workspace/**` |
| L-X11 / L-X12 / L-X13 | `web/src/console/sales/**`, `web/src/i18n/salesCrm.ts` (**serial, same FE owner**) | as L-X10 |
| L-X14 | `docs/evidence/console/CAP-SALES-CRM/**`, `e2e/specs/*sales*`; `manifests/mount.json` | `nav.ts`, `nav.test.ts`, `screens/registry.ts`, the design mirror — **all integrator-applied** |

### Shared collision roots — serialized to the integrator via manifests

Per `WAVE4-CHARTER.md` §4, which stays binding. Only the rows a depth-first wave can
reach:

| Root | Owner | Manifest a lane emits |
|---|---|---|
| `backend/openapi/openapi.yaml` | **openapi-integrator** (a live agent in this session) | `manifests/api/openapi-fragment.yaml`, hand-written, **`tags:` per domain** or the Kotlin client regresses to one `DefaultApi.kt` and OOMs kotlinc. **Never splice** — `ee277e16` → whole-file revert `9bb877c6`. |
| `clients/{ts,kotlin,swift}/**` | openapi-integrator | Regenerated only. Three gates: the `openapi_drift` Rust test + `check:api-drift:portable` + `:swift`. |
| `backend/crates/platform/db/migrations/**` | **Integrator, never a leaf write** | `manifests/migration/<slot>_<name>.sql`, slot from the ledger |
| `backend/app/src/lib.rs` | Integrator | `manifests/app-register.json` — appended router/handler `register(...)` lines only |
| `backend/app/src/objects.rs` | Integrator + **security review** | `RESOLVABLE_KIND_AUTH` row for `deal` (L-X7), **with the retroactive-`object_links`-resolvability answer stated** |
| `kernel/core/{src/lib.rs,Cargo.toml}`, `backend/Cargo.lock` | Integrator | `manifests/kernel-core.json`, one `pub mod` line + one dependency line. `Cargo.lock` is **regenerated** at merge, never merged as a diff |
| `**/BUCK`, `backend/app/BUCK`, `tools/buck/generated_face_registry.json` | Integrator | `manifests/buck-app-test.json` (target, srcs, deps) — **confirm the buck branches are released before applying** |
| `web/src/i18n/ko.ts` | Integrator | `frontend/manifests/i18n-keys.json` — **rarely needed**: CRM strings live in the module-private `web/src/i18n/salesCrm.ts` |
| `web/src/console/shell/nav.ts`, `screens/registry.ts` | Integrator | `frontend/manifests/mount.json`. **Only L-X14 may request the exposure change.** |
| `docs/program/console-enterprise-roadmap.md`, `console-capability-registry.json`, `console-program-ledger.md`, `scripts/console/**` | **Integrator (candidate-rebind owner)** | Roadmap-delta line in the evidence manifest. Every code merge invalidates the truth ledger and forces a serialized rebind. |
| Catalog train: `seed.rs` + `BUILTIN_CATALOG_VERSION` + its sha256 + the `ont_builtin_catalog_allowlist` row | **ONE agent**, strict order `L-A1 → L-X7`, version `.1 → .2` | Two concurrent bumps produce two digests and an allowlist admitting neither |
| **New crates, anywhere** | **Forbidden this wave** | A new crate forces a `gen_first_party.py` regen across 161 first-party BUCK files |

---

## 5. Definition of done

**`WAVE4-CHARTER.md` §6.1 / §6.2 / §6.3 / §6.4 remain the normative template, verbatim.**
Restating 30 items here would create a second version that drifts. The normalized
per-lane DoD is:

**Universal (§6.1), every lane** — (0) re-run your lane's §0 rows before writing code ·
(1) **zero stubs**: no `TODO`, no `test.skip`/`.only`, no unimplemented branch, no filler
copy; a `ponytail:` comment only where it names a ceiling *and* an upgrade path · (2)
collision roots are manifests, never edits · (3) truthfulness: no fabricated rows,
totals, codes, relations or statuses; every deferral is a named gap-manifest line with a
repo-relative register anchor · (4) **no dead affordance** — deny-by-omission, not
disabled · (5) push discipline: **`git fetch && git merge origin/<spine>` — plain merge,
never rebase (D-2)** · (6) **evidence doc at `docs/evidence/console/CAP-SALES-CRM/`**
(Phase-0 lanes: `CAP-CONSOLE-GRAMMAR` / `CAP-ONTOLOGY-ENGINE`) · (7) truth-ledger
admission: signed commits, canonical review receipt, `plan-fanout.mjs` registration —
**the gate is live on this branch today** (§0) · (8) **independent review by a different
agent**; the integrator queues no manifest without a GO verdict.

**Backend lanes additionally (§6.2)** — `cargo fmt` + `cargo clippy -D warnings` clean
(compile-verify in a **spawned subagent**; local cargo is hook-disabled in the main
session only) · **RLS `FORCE`, every assertion executed as `console_rt`** (bootstrap
`console_buck_admin` + `mnt.sqlx_test_bootstrap`; a superuser `BYPASSRLS` pass proves
nothing) · **deny-by-default PBAC + an audit row in the SAME transaction** as every
mutation · canonical envelope `{error:{code,message}}`, **422** validation / **409**
conflict, with **one negative test driving a DB CHECK violation to 422, never 500** ·
**idempotency: same key + matching fingerprint returns the FIRST outcome** — identical
status and body, no second write, no second audit row; same key + different fingerprint
→ 409; **both branches tested as `console_rt`** · one **story-level** app integration test as
`console_rt` (emit `manifests/buck-app-test.json`; do not edit `backend/app/BUCK`) ·
**`buck2 test //backend/crates/<owned>/...` green — Buck2 is the completion evidence** ·
**unconditional CI gates** for any lane adding a migration, table or route: `rls-arming`,
`tenant-isolation`, `audit-coverage`, `dev-auth-absence` (as applicable:
`layer-boundary`, `pii-no-logs`, `migration-safety`) · **openapi fragment manifest +
regenerated clients for any route change** — three independent drift gates; a Rust-only
pre-check misses two of them.

**Frontend lanes additionally (§6.3)** — the window-contract paragraph pasted verbatim
into the PR body and honoured · three unit assertions per ported surface via
`renderWithWindowManager`, with the **bucket-3 variant** for code-less kinds · **real
browser proof** (live React tree + keyboard journey + axe on the mounted window layer) —
**jsdom-only is not completion evidence for any window claim** · **AA is a check**: the
named playwright command, committed, plus 1.4.10 / 2.4.7 / 1.4.11 / 2.5.8 and a Korean
accessible name on every icon-only control · `vitest` green + `tsc --noEmit` clean +
`npm --prefix web run lint` (already runs `check-ui-strings.mjs` — **live against you
today**) · **no explanatory UI** (binding merge gate) · **§4-25 closed loop + §4-21
benchmark pass** — discharged once, by L-X13, for the module.

**Keyed on what the lane does (§6.4)** — any `param_verify_live` lane (**L-X8 only**):
citation in the test name, no local literal, `Err(ParamUnverified)` and **no `unwrap_or`
on a statutory resolve** · any lane writing domain state transitions or money/time
arithmetic (**L-X1…L-X6**): the domain's own `edge_case_gaps` rows by anchor **plus** the
five universal categories (mid-period join/leave · backdated correction + effective-dated
recalculation · concurrent transition race · reversal/compensation path · boundary dates:
KST month-end, 회계연도) · any lane owning one side of a cross-domain invariant
(**L-X5**: deal→contract; **L-X8**: lead→deal PII lineage): close it or name-defer it
with its anchor · **grep gate (L-X3): no `tokio::spawn`, `LISTEN` or `NOTIFY` introduced
for staleness detection.**

---

## 6. Dependency DAG and concurrency

### DAG, text form

```
L-P0-EPOCH ─┬─> L-A1 ──────────────────────────────────────┐
            │                                              │
            ├─> L-F1 ─┐                                    │
            ├─> L-F2 ─┤                                    │
            ├─> L-F3 ─┤                                    │
            │         │                                    │
            ├─> L-X9  │   (independent; may run at t1)     │
            │         │                                    │
            └─> L-X1 ─┴──> L-X2 ──┬──> L-X3 ──┐            │
                          │       ├──> L-X5   │            │
                          │       └──> L-X7 <─┴────────────┘   [catalog train link 2]
                          │
                          ├──> L-X4        (needs L-X1 only)
                          └──> L-X8        (needs L-X1 only)

L-X2, L-X3            ──> L-X6
L-F1, L-F2, L-F3,
L-X1, L-X7            ──> L-X10 ──> L-X11 ──┐
L-X2, L-X10           ──> L-X12 ────────────┴──> L-X13
L-X6, L-X8, L-X13     ──────────────────────────> L-X14   [EXPOSURE — the only sink]
```

Edges, exhaustively: `EPOCH → {A1, F1, F2, F3, X9, X1}` · `A1 → X7` · `X1 → {X2, X4, X8}`
· `X2 → {X3, X5, X7, X6, X12}` · `X3 → X6` · `{F1,F2,F3,X1,X7} → X10` · `X10 → {X11,X12}`
· `{X11,X12} → X13` · `{X6,X8,X13} → X14`.

**Serial chains (the critical path):**
`EPOCH → X1 → X2 → X7 → X10 → X11 → X13 → X14` — **8 links**, the longest chain in the
wave and the one that determines its length. `L-A1 → L-X7` must complete before `L-X10`
can assert a `DL-` code, which is why L-A1 dispatches at t1 alongside the F-lanes rather
than waiting.

**Rule applied throughout: an FE lane's phase = max(phase of its backend deps).** L-X10
cannot precede L-X7; L-X12 cannot precede L-X2's 412.

### Concurrency cap: **4**, justified

D-3 approved 8 worktrees (~150 GB against ~3.0 TiB free), so disk is not the constraint.
**The constraint is file-level exclusivity in a single-module wave**, and honouring it
gives four real slots:

1. **Sales-crate owner — width 1.** `backend/crates/sales/domain/src/lib.rs` is 320 lines
   and `application` 332. L-X1 → L-X2 → {L-X3, L-X4, L-X5} → L-X6 all rewrite the same
   files. Two concurrent lanes there is a merge conflict per commit, and the plain-merge
   train (D-2) has no rebase to clean it up with. One agent, strict order.
2. **Sales-frontend owner — width 1.** `SalesCrmScreen.tsx` is one 377-line file with 548
   lines of tests. L-X10 → L-X11 → L-X12 → L-X13 is the same file four times. One agent,
   strict order.
3. **Platform/shared owner — width 1.** L-A1, L-X7, L-X9 and L-X8's migration all touch
   integrator-serialized surfaces (`seed.rs` + the catalog digest, `kernel/core`,
   `app/src/lib.rs` register lines, `objects.rs`). The catalog train is width-1 **by
   construction** — two concurrent version bumps produce two digests and an allowlist
   admitting neither.
4. **Integrator / candidate-rebind — width 1, and it is real work, not overhead.** One
   serialized apply queue: migrations, openapi fragments, client regens, `nav.ts`,
   `registry.ts`, BUCK, `kernel/core` appends, roadmap deltas — **plus the truth-ledger
   rebind, which every code merge invalidates**. The spine's own recent history is
   literally rebinds fighting this. **In-flight migrations are capped at the integrator's
   measured drain rate, not at the lane cap.**

Phase 0's F-lanes (L-F1, L-F2, L-F3) are file-disjoint and can occupy slots 1-3
concurrently *before* the CRM trunk starts, which is why Phase 0 finishes in roughly one
slot-width of wall time. **A fifth concurrent lane buys nothing**: it would have to be a
second writer on one of the four surfaces above, and review throughput — one independent
review per lane by a different agent (§6.1-8) — is the second binding limit.

Resource envelope: 4 worktrees ≈ **75 GB** (Rust `target/` 12-18 GB + `node_modules`
1.5 GB each). Comfortably inside D-3.

---

## 7. Live-branch collision warnings — read first and absorb

Carried forward **unchanged in force**, filtered to branches touching CRM or a Phase-0
surface. Re-verified 2026-07-25; **three of these are LOCAL heads, not remote refs — a
`git for-each-ref refs/remotes` sweep misses them entirely.**

| Branch | Surface | Lane | Protocol |
|---|---|---|---|
| `codex/console-shell-scheduled-nav-20260724` **(local head)** | `ConsoleShell.tsx`, `nav.ts` | **L-F1**, L-X14 | **Confirm ownership release BEFORE L-F1 starts.** §4-N makes `ConsoleShell.tsx` + `ConsoleApp.tsx` L-F1-owned for the wave; that ownership is not self-granted. |
| `codex/console-global-scope-foundation-20260724` **(local head)** | `ConsoleShell.tsx`, console scope | **L-F1** | Same confirmation. A scope-foundation change to the shell and a provider mount in the shell are the same 40 lines. |
| `codex/console-search-object-fabric-20260724` **(local head)** | object typeahead/search contract, `web/src/api/ontology.ts` | **L-X10** | **Owns the contract.** L-X10 does not build typeahead — it **gap-manifests** it with the register anchor. `typeRegistrySource.ts:20-24` also notes `api/ontology.ts` is under concurrent edit by the serial-wire lane; verify before claiming it. |
| `codex/console-sales-crm-20260723` **(local head)** | the original sales workbench | **L-X1, L-X10** | **Read and confirm absorbed; NEVER merge.** Tip `e45cc6e7` sits on `8e42b9a2` (release 0.2.1), ~900 commits stale — `git diff HEAD` shows **2,366 files / 479,631 deletions**. Its content already exists at HEAD. A merge would revert the wave-2/3 consolidation. |
| `codex/console-fanout-planner-hardening-20260724` (remote) | `scripts/console/**`, `plan-fanout.mjs`, the truth ledger | **L-P0-EPOCH** and **every lane's §6.1-7 registration** | **NEW — absent from the 60-lane charter's §4-V.** The admission gate every lane must pass is being changed on this branch. EPOCH reads it before writing the epoch amendment. |
| `codex/default-branch-authority-bootstrap-20260725` (remote, today) | branch/candidate authority | **Integrator (rebind owner)** | **NEW.** Authority bootstrap and candidate rebind are the same surface; the integrator absorbs before the first rebind. |
| `codex/buck-h0-classification-20260724`, `codex/buck-generated-gate-parser-fix`, `fix/pr488-generated-face-authority` | `**/BUCK`, `generated_face_registry.json` | **Integrator only** | No lane writes BUCK (L-P0-BUCK is dropped), but the integrator **confirms release before applying `manifests/buck-app-test.json`**. |
| `codex/operational-object-runtime-progress` (remote) | the spine | all | Plain-merge before every push (D-2). |

**Dropped from the eight, because no CRM or Phase-0 surface is touched:**
`equipment-evidence-custody` · `directory-backend-private-leaf` · `inventory-private-leaf`
· `dispatch-private-leaf` · `attendance-frontend-private-leaf` ·
`people-team-condition-regressions`. **`equipment-evidence-custody` is the exception that
still needs a program action — see §8.**

---

## 8. Deferred to later waves

Depth-first buys one module at full maturity by not touching thirteen. What that costs,
named so it cannot resolve into silence. Each item keeps its register anchor via
`backend-blocked-index.json` (L-P0-EPOCH).

**Dropped as a class:**

- **All 13 non-CRM module FE lanes** — L-B8F/B9/B10F/B11F/B13F/B14F/B15F/B16F/B17F/B18F/
  B19F/B23F/B26F, plus the L-B21F recipe gate. They receive **only** their share of the
  Phase-0 shared fixes (mounted window provider, unified code grammar, shared card a11y).
  Their fidelity registers stay open, gap-manifested as a block with anchors.
- **The entire lens-D domain chain** — L-D0, L-D0b, L-D1…L-D18. Includes the wage chain
  `L-D0 → L-D0b → L-D4 → L-D5`, which the 60-lane charter marked *"never cut."* It is not
  cut; it is **sequenced next**, and **R-1 below is the binding consequence.**
- **The rest of the catalog train** — L-A3, L-A4a, L-A4b, L-A7, L-A9. The train shrinks to
  two links. L-A7's fourteen module projections become one module's projections (L-X7);
  R5 (projected receipts) and R8 (`resolve_by_code` for projected types) stay open with
  anchors.
- **L-F4** (console list grammar, module engine, personal-view config strip, C-41's N+1)
  — its consumer list was "all module lanes". CRM builds its list surface locally in
  L-X10/L-X11. **Do not generalize a primitive from one consumer** (the L-C6/L-C7
  reasoning, applied to L-F4 in a one-module wave). Extract in wave 5, from two real
  consumers, when WMS lands.
- **L-C9** (responsive/density/locale generalization) — CRM gets the concrete version in
  L-X13. Inventory's `findings[0]` blocker stays open with its anchor.
- **L-C4** (realtime), **L-C6** (bulk primitive), **L-C7** (draft-autosave primitive),
  **L-C11** (virtualization), **L-B5** (search fabric), **L-A2** (schema publish route),
  **L-B0b** (4-state window provider) — all already deferred by the 60-lane charter for
  reasons depth-first only strengthens. L-C6/L-C7's *concrete* forms ship inside L-X4 and
  L-X12 for the one consumer that exists.
- **CRM-4 / B-26** (D-90 renewal automation with dedupe) — stays wave 5. It needs real
  contract-expiry events, and L-X5 only creates the contract. Recorded, anchored.
  **POL-7** and **WMS-5** stay wave 5; WMS is next under C-64's CRM→WMS→MES order.

**Four items a reviewer will look for, and will not find. Named, not buried:**

1. **L-D12 — equipment 3R handover custody, P0, RED at HEAD.** A dropped column with a
   currently-failing test. **A RED test must not sit across a wave.** This is not a
   wave-4 lane; it is a **program action measured in minutes**: read
   `codex/equipment-evidence-custody-20260724` — if it already repairs the drop, absorb
   and close. If not, **re-add the column** (the smaller diff; turns the test green
   immediately; leaves the L-D13/L-B8F contracts unchanged) as a standalone hotfix
   outside the wave.
2. **L-D14 — benefits four-eyes: `transition_lifecycle` records the actor but never
   compares it against the prior transition's actor.** One principal can self-approve.
   **This is a live SoD bypass on the spine**, `benefits.rules[4]`, and it is being
   deferred. Paired with `benefits.rules[6]`: `cedar_policy_ref` is a ≤200-char text check
   only, so dangling and fabricated refs are accepted. Both are security defects, not
   depth gaps. Recommend the same treatment as item 1 — a standalone hotfix, not a wave
   slot.
3. **L-D3 — leave §61 촉진, the only *wrongly-fabricated* finding in the whole audit**
   (`validate_round` checks only `round ∈ {1,2}` while the statutory two-round windows go
   unenforced). It is the one truthfulness *defect* as opposed to a truthfulness *gap*.
   Deferred with R-1's wave.
4. **L-D15 — org 변경 동결창: `effectuate` has no freeze check at all**, so an org change
   can apply inside a payroll period. Deferred; anchored at `org.rules[9]`.

**Also dropped, and deliberately so:** the C-64 **waiver** (§9-0 of the 60-lane charter).
Depth-first obeys C-64 rather than waiving it, so there is nothing to waive. The 60-lane
charter's §9-1 (epoch contract) is **resolved by D-2**, not escalated. §9-4 (4-state
provider) is **resolved by D-4** as a deferral. §9-6 (statutory registry location) is
**resolved by D-6** as out-of-path. §9-2 (exposure) is **resolved by D-1** and executed
by L-X14. §9-5 (`projected_hours`, `SupportCase`) is **resolved by D-5** — delete is the
default — but **neither lane exists this wave, so both remain recorded anchors**, not
completed deletions.

---

## 9. Standing risks

Reproduced verbatim from `DECISIONS.md` §"Standing risk recorded (not scheduled this
wave)".

> **R-1 · Payroll wage-law shallowness is a ship-blocker, not a live liability.**
> 연장 ×1.5 is gate-only, 야간/휴일 hours are dead columns, overlap is
> structurally unrecoverable, 주휴수당 is absent, and the golden-case gate never
> executes. This is criminal-exposure territory (임금체불) IF payroll ever runs
> real money — but every payroll surface is DARK and nothing is exposed, so there
> is no live exposure today. Binding consequence: **payroll must not be exposed,
> and must not process real runs, until the L-D0/L-D4 wage-law lanes land.** That
> pair opens the next wave.

> **R-2 · Payroll timestamps serialize as tuples, not RFC 3339** (bare `time`
> types + no `serde-human-readable`) — spec and wire disagree today on already
> shipped payroll endpoints. Workspace-wide sweep needed; queued with R-1.

**Enforcement in this wave.** R-1 is binding on **L-X14**: the exposure lane requests
`EXPOSED_SCREEN_KEYS = ["sales"]` and **nothing else**. Any manifest that would add a
second key — payroll above all — is refused by the integrator on R-1's authority. R-2 is
binding on **L-X1's** wire contract: the deal aggregate must not repeat the defect. Any
`time` type crossing the REST boundary carries `serde-human-readable`, and one test
asserts an RFC 3339 string on the wire — the smallest possible check that stops R-2 from
propagating into a second domain while the sweep is queued.

### L-X7 — MANDATORY ADDITION (L-A1-D4, 2026-07-25)

L-A1 makes the catalog upgrade *possible*; **nothing makes it happen.**
`TenantConfigSeeder` has exactly one call site — the tenant-onboarding handler
at `backend/crates/platform/platform-rest/src/lib.rs:602`, for a brand-new org.
There is no re-seed route, no startup reconcile, and no migration that calls the
installer. So once L-X7 bumps `BUILTIN_CATALOG_VERSION`, every tenant onboarded
before that deploy stays at 27 types forever: no `deal`, no `DL-` resolution, an
absent CRM ontology surface. Every test in the suite drives the installer
directly, so CI cannot see this.

L-X7 therefore delivers the **existing-tenant upgrade trigger** as part of its
scope, not as a follow-up: `console_ontology_cmd` credentials, `scope_org` per
tenant, deny-by-default authz, an audit row, and a test against a tenant seeded
at the OLDER version. The installer is already idempotent, so the trigger needs
no guard rails of its own.

Without this line, L-X7 ships a CRM ontology that exists only for orgs created
after the deploy — which is indistinguishable, from any existing tenant's seat,
from not shipping it at all.
