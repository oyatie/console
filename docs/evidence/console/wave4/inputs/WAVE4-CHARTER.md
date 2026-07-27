# WAVE 4 — FINAL CHARTER (post-adversarial revision)

**Spine:** PR #488 · integration worktree `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync` · branch `wave23-consolidation-20260724`.

**Revision:** this document supersedes the merged charter reviewed by four adversarial judges (execution-feasibility, truthfulness/enterprise-bar, coverage-against-evidence, sequencing-and-value — all four `needs-revision`). Every blocker and major is answered in §10 with CHANGED or REJECTED + reason.

**Doctrine:** `docs/intent/console-north-star.md` + the beyond-prototype amendment + `design-intent-register.md` (612 intents, incl. the 14 `[>190]` the build lanes never saw). Prototype fidelity is a floor, not a ceiling. Statutory-deterministic rules are IMPLEMENTED with citation + test; externally-certified artifacts stay gated attestations.

> **THE FOUR LENS CHARTERS (`charter-lens-{A,B,C,D}.md`) ARE SUPERSEDED FOR ROOTS, MIGRATION SLOTS, PHASES AND DEPENDENCIES.** They remain valid as *rationale and evidence*. No lane may be dispatched with a lens-charter root list. The normative per-lane table is §5 of this document plus the machine-readable `docs/program/wave4/lanes.json` that L-P0-EPOCH commits.

**Lane count: 60** (§5 canonical table is the single source; the phase sections reference it, never restate it). 22 lanes cut or deferred with named register anchors (§8).

---

## 0. Facts re-verified NOW, with the command that produced each

The previous §0 was anchored to a moved tree (it pinned HEAD `1ea0c185`; the branch has moved twice since). Numbers on a spine taking ~900 commits in two days are a snapshot presented as state. **Therefore each row carries its command, and universal DoD step 0 (§6.1-0) requires each lane to re-run the rows it depends on before writing code.**

| Claim | Command | Verified 2026-07-25 |
|---|---|---|
| HEAD | `git log --oneline -1` | `8b26c16b`; tree DIRTY with `backend/openapi/openapi.yaml`, `clients/ts/src/schema.d.ts`, `web/src/console/payroll/payrollApi.ts` — **this is the in-flight openapi-integrator agent. Do not stash, do not charter.** |
| Spine delta | `git rev-list --left-right --count HEAD...origin/codex/operational-object-runtime-progress` | **188 ahead / 45 behind.** Spine tip `34d1f6df` = "chore(console): rebind corrected product candidate". |
| Migration high-water | `ls backend/crates/platform/db/migrations \| tail` | `0202_notification_policies_and_object_agg.sql`. `0201` **absent** (reserved gap). The 45 spine-only commits add **zero** migrations, so 0203 is a safe floor *today only*. |
| Truth-ledger admission gate | `git show origin/codex/...:package.json \| grep truth-ledger` | **`check:console-truth-ledger` exists on the SPINE, not yet on this branch.** It arrives with the next plain-merge and then blocks every code merge (§4-U). |
| `WindowManagerProvider` in the console shell | `grep -n "WindowManager" web/src/console/shell/ConsoleShell.tsx` | ZERO hits. All 15 module bodies run with no provider in production. |
| Nested provider | `grep -n WindowManagerProvider web/src/console/screens/_ontology/OntologyWorkspaceBody.tsx` | `:12` import, `:299` mount — a shell-level mount **nests** it; the inner wins and produces a second layout partition. |
| Single pin = demotion | `sed -n '168,178p' web/src/console/window/WindowManager.tsx` | `pin()` pushes the previous `pinnedId` into `minimizedIds`. A second `open()` visually loses the previous card. |
| `objectCardWindowEntry` owner | `grep -rn "export function objectCardWindowEntry" web/src` | `web/src/console/objectcard/ObjectCard.tsx:863` — **L-F2's root, not L-F1's.** |
| `renderWithWindowManager` | `grep -rn renderWithWindowManager web/src` | ZERO hits. It does not exist; 14 lanes' assertions depend on L-F1 creating it. |
| Exposure | `grep -n EXPOSED_SCREEN_KEYS web/src/console/shell/nav.ts` | `:134` → `["sales"]`. `nav.ts:236` renders a live `dispatch` nav entry for a screen in neither `MOUNTED_SCREEN_KEYS` nor `SCREEN_REGISTRY` — a shipped dead affordance. |
| `check-ui-strings.mjs` | `ls web/scripts` | **Already exists and is already wired** into `web/package.json` `lint`. L-C8 extends it; it gates nothing today. |
| First-party BUCK files | `find backend web tools -name BUCK \| wc -l` | **161** (not 147). |
| `kernel/core` purity | `cat backend/crates/kernel/core/Cargo.toml` | deps are exactly serde, serde_json, thiserror, uuid, time; description says "Pure — no async, no sqlx, no axum". `backend/ci/gates/layer-boundary` enforces it. |
| Landed keyset cursor | `grep -rn EvidenceObjectCursor backend/crates/docs` | `docs/application/src/lib.rs:48` — opaque base64url keyset cursor with snapshot stability, round-trip tested, 13 commits. **A second cursor is forbidden.** |
| `/api/v1/employees` | `grep -rn EMPLOYEES_PATH backend/app/src` | `backend/app/src/hr.rs:31` — an 11.7k-line app-tier file with two live codex writers. **Not a viable L-C1 pilot.** |
| `web/src/console/docs` | `ls web/src/console` | **Does not exist.** `web/src/console/evidence/` is the real directory. `CAP-DOCS-EVIDENCE-CONSOLE.frontend_roots` declares the wrong path → every docs lane is held by `dispatch_rule`. |
| Shared collision roots (machine-readable) | `console-capability-registry.json .shared_collision_roots.paths` | ko.ts · nav.ts · registry.ts · openapi.yaml · **migrations/\*\*** · **console-enterprise-roadmap.md** · **console-capability-registry.json**. The last three were missing from the previous §3. |
| CI gates on disk | `ls backend/ci/gates` | audit-coverage · **dev-auth-absence** · iac-tier · layer-boundary · migration-safety · pii-no-logs · rls-arming · tenant-isolation · vendor-lockin |
| Statutory brief coverage | `grep -cE '부가가치세\|VAT' research-statutory-params.md` → **0**; `grep -c 공휴일` → **1** | The brief carries **no VAT content** and **no holiday calendar**. Three lanes need VAT and five need the calendar. |
| Live codex leaf branches | `git for-each-ref --sort=-committerdate refs/remotes/origin/codex` | equipment-evidence-custody · directory-backend-private-leaf · inventory-private-leaf · dispatch-private-leaf · attendance-frontend-private-leaf · buck-h0-classification · buck-generated-gate-parser-fix · people-team-condition-regressions — **all unmerged, all colliding with chartered lanes** (§4-Q). |

---

## 1. The five things this revision changes structurally

1. **Migrations and BUCK stop being leaf writes.** `docs/program/console-fanout-epoch-contract.md:126` says so and the registry agrees; the previous charter handed 41 slots to leaves with a "re-check before push" rule that detects collisions *after* the fact. Proof it fails: `0197_customer_site_registry_foundation.sql` and `0197_notice_audience_and_category.sql` both exist across refs, as do duplicate 0180/0181/0182/0185/0186/0193-0196. → **Lanes emit `manifests/migration/<slot>_<name>.sql`; the integrator applies and renumbers.** Renumbering becomes a filename change, which is only true if every migration is self-contained with no cross-lane FK — that is now a DoD line.
2. **`kernel/core/src/lib.rs` ordered append slots are deleted.** They created a false width-1 bottleneck and an internally contradictory DAG. → one manifest line per lane, integrator appends. Same for `kernel/core/Cargo.toml`, `backend/Cargo.lock` (regenerate at merge), and `backend/app/BUCK`.
3. **The wave has a headline user-visible deliverable.** The previous charter conceded "nothing in wave 4 becomes user-visible" while `sales` is the *only* exposed screen and the authority's newest intents (CRM-1…6, `[>190]`) target exactly it. → **L-X1 CAP-SALES-CRM** is chartered.
4. **Evidence stops evaporating.** The statutory brief and both registers lived in a session-scoped `/private/tmp` path while the DoD required citing them. → L-P0-EPOCH commits them to `docs/evidence/console/wave4/` and every anchor is repo-relative.
5. **No lane self-certifies.** A review-gate pass by a different agent is a completion precondition, and the truth-ledger receipt is the artifact that proves it.

---

## 2. Phases and the real critical path

### The critical path is the catalog train, and it is the longest chain in the wave

```
L-A1 → L-A3 → L-A4b → L-A7 → L-A9        strictly serial, ONE owner agent, Phase 0 → Phase 2
```

Six of fourteen module FE lanes depend on **L-A7 — link 4 of five**. The fan-out is therefore ordered by dependency, never by fidelity score. **The train owner gets nothing else for the whole wave.**

### Phase 0 — program actions, decisions, foundation (20 lanes)

```
t0  program/serialized, before ANY code lane:
      L-P0-EPOCH   (integrator/program: registry, epoch, evidence base, slot ledger, blocked index)
t0  decisions + research (doc-only, reviewed by a different agent than the author):
      L-P0-ID-a  code triage + port-path assignment      → unblocks the FE fan-out
      L-P0-ID-c  object-type registry authority          → blocks only L-A9
      L-P0-PARAM live statutory fetch sweep              → blocks L-D0, L-D9/D13/D14/D15/D17
      L-A4a      projected receipt design gate           → blocks only L-A4b
      L-B23      docs+dispatch dual-impl decision        → blocks L-B23F, L-B26F
      L-P0-BUCK  generated-face regen (+ --only, glob app-test)
t0  foundation code (max 8 concurrent, see the cap):
      L-A1 (train link 1) · L-A6 · L-C1 · L-D0 · L-D0b · L-D3 · L-D12 · L-F1
t1:
      L-D1 (← L-D0) · L-F2 (← L-F1) · L-F3 (← L-A6, L-P0-ID-a) · L-F4 (← L-C1, L-F2) · L-C8
```

### Phase 1 — shared contracts (6)

`L-A3` (train 2 ← L-A1) · `L-B4` (← L-P0-ID-a, L-A6) · `L-B7` (← L-C1) · `L-D2` (← L-D0, L-D1) · `L-C3` (← none; pilot = ontology instance-command 412 path) · `L-B22a` (0201 retention merge ← L-C1, same owner)

### Phase 2 — train, first FE fan-out, domain depth (18)

```
TRAIN (one owner, strict order):     L-A4b → L-A7 → L-A9
FE (dependency-clean, dispatch in this order — L-B21F is a GATE, not an ordering):
      L-B21F (gate) → then L-B15F · L-B9 · L-B23F
DOMAIN DEPTH:
      L-D4 · L-D6 · L-D9 · L-D13 · L-D15 · L-D16 · L-D17 · L-D18
UX:   L-C9
```

### Phase 3 — dependent FE, deep domain, the headline vertical (16)

```
      L-X1 (CAP-SALES-CRM — the wave's only user-visible deliverable)
      L-B8F · L-B10F · L-B11F · L-B13F · L-B14F · L-B16F · L-B17F · L-B18F · L-B19F · L-B26F
      L-D5 · L-D7 · L-D8 · L-D10 · L-D11 · L-D14 · L-B22
```

**Never cut** `L-D0 → L-D0b → L-D4 → L-D5`. 임금체불 is criminal exposure; the wage chain outranks lower-scored lanes.

### Concurrency cap: 8, and the five bottlenecks

Reduced from 10. The previous cap was contradicted twice internally (an 11-lane t0 against a stated 10) and had no resource budget.

1. **The catalog train** — width 1 for the whole wave, one agent, five links.
2. **The FE foundation** L-F1 → L-F2 → L-F4 — width 1 at Phase-0 t1; module fan-out capped at **6 concurrent** by review throughput, not disjointness.
3. **The integrator's apply queue** — openapi, i18n, nav, registry, migrations, BUCK, `kernel/core` appends, roadmap deltas. One serialized drain. **In-flight migrations are capped at the integrator's measured drain rate, not at the lane cap.**
4. **The docs owner** — L-C1 → L-B22a → L-B22 all touch `backend/crates/docs`.
5. **Candidate rebind** (NEW) — the truth-ledger gate invalidates the ledger on every code merge and forces a serialized authority rebind. **Named owner: the integrator.** The spine's last six commits are literally rebinds fighting this.

**Resource budget, stated because 8 concurrent lanes means 8 worktrees:** each lane needs an exact isolated worktree per the registry's `dispatch_rule` (Rust `target/` ≈ 12-18 GB, `node_modules` ≈ 1.5 GB). 8 lanes ≈ **150 GB and 8 checkouts**. If the budget is not available, the cap is whatever the disk allows — say so before dispatch rather than discovering it at lane 5.

---

## 3. Doctrine reconciliations the previous charter resolved by silence

### 3a. C-64 vs the fan-out — an explicit conflict, not a nuance

`design-intent-register.md:108`, **C-64 `[>190]`**, verbatim: *"One-module-at-a-time full maturity: each new suite module passes the §4-25 loop before the next starts; **thin fan-out banned**; order CRM→WMS→MES."* This charter fans out 14 modules, 6 concurrent. That is a head-on conflict with a post-190 authority intent, and the previous charter did not mention 공휴일, C-64, or the design-intent register once.

**Resolution taken (recorded, not assumed):** wave 4 is a **substrate + depth-per-module** wave, and the defensible reading of C-64 is satisfied *only if* each module lane closes its entire fidelity register **and** its domain's depth register before merge — which is now the DoD (§6.3-18). C-64's *ordering* clause is honoured by chartering **CRM first among the visible modules** (L-X1). **This requires a program waiver, written into §9-0.** Do not dispatch 60 lanes against a standing ban without it.

### 3b. The epoch contract cannot be satisfied as written

`console-fanout-epoch-contract.md:115-118` requires "a serialized rebase/cherry-pick admission train. No merge … is admitted", while rebase is classifier-blocked on this spine (project memory) and §6.1-5 mandates plain merge. As chartered, all 60 lanes are formally inadmissible. **§9-1 is the program decision.** This charter takes the half it can: migrations, BUCK, clients and openapi become integrator manifests (contract line 126-127 satisfied), and the merge-vs-rebase clause is escalated rather than ignored.

### 3c. The §4-25 closed loop is the amendment's definition of done and was absent

Amendment §"Consequence for scoring": *"wave-4 DoD = fidelity floor + lens-C register worked off + §4-25 closed loop run."* → §6.3-18.

---

## 4. Collision map — every shared root, its single owner, the protocol

Reconciled against the registry's machine-readable `shared_collision_roots.paths`, not a hand-derived list.

| # | Shared root | Owner | Protocol for every other lane |
|---|---|---|---|
| A | `backend/openapi/openapi.yaml` | **openapi-integrator (in flight — do NOT charter)** | Emit `…/api/openapi-fragment.yaml`, hand-written, `tags:` per domain. **Never splice** (`ee277e16` → whole-file revert `9bb877c6`). 68 hits/48h. |
| B | `clients/{ts,kotlin,swift}/**` | openapi-integrator | Regenerated only. Three gates: `openapi_drift` Rust test + `check:api-drift:portable` + `:swift`. |
| C | **Catalog train**: `ontology/adapter-postgres/src/seed.rs` + `BUILTIN_CATALOG_VERSION` + its sha256 + the `ont_builtin_catalog_allowlist` row | **ONE agent**, order `L-A1 → L-A3 → L-A4b → L-A7 → L-A9`, versions `.1/.2/.3` | Two concurrent bumps produce two digests and an allowlist admitting neither. (The previous "stale seed.rs is a painful merge" rationale is dropped — `seed.rs` has **zero** commits in 48h; the digest/version/allowlist argument is sufficient on its own.) |
| D | `backend/crates/kernel/core/src/lib.rs`, `…/Cargo.toml`, `backend/Cargo.lock` | **Integrator** | Emit the `pub mod` line / the single dependency line in `manifests/kernel-core.json`. `Cargo.lock` is **regenerated** at merge, never merged as a diff (45 hits/48h). |
| E | `backend/app/src/lib.rs` | Integrator | Emit the router-register line in the manifest. Hottest backend file on the spine. |
| F | `backend/app/src/objects.rs` | Integrator (security-reviewed `RESOLVABLE_KIND_AUTH`) | No lane edits it. L-A6 verified not to need it (`objects.rs:2606-2609`). |
| G | `backend/crates/platform/db/migrations/**` | **Integrator (never a leaf write)** | Emit `…/backend/manifests/migration/<slot>_<name>.sql`. Slots come from the **append-only ledger** `docs/program/migration-slots.json` — a lane takes a slot by *requesting* it, never by running `ls`. One self-contained file per lane, **no cross-lane FK**, so renumbering is a filename change. |
| H | `**/BUCK` + `tools/buck/generated_face_registry.json` | **L-P0-BUCK** for the regen; integrator thereafter | `gen_first_party.py` rewrites all **161** first-party files. L-P0-BUCK ships a `--only <crate>` path as part of its own deliverable. Leaf lanes emit `manifests/buck-hunk.txt`. |
| I | `backend/app/BUCK` | **Integrator** | ~20 backend lanes need a `rust_test` block in one machine-generated file. Emit `manifests/buck-app-test.json` (target, srcs, deps). **Better: L-P0-BUCK ships a glob-driven app-test target and the bottleneck disappears permanently.** |
| J | `web/src/i18n/ko.ts` (28 hits/48h) | Integrator | Emit `frontend/manifests/i18n-keys.json`. |
| K | `web/src/console/shell/nav.ts` · `web/src/console/screens/registry.ts` | Integrator. `EXPOSED_SCREEN_KEYS` locked by `nav.test.ts:39,58` | Emit `frontend/manifests/mount.json`. Only **L-X1** may request an exposure change, and only via §9-2. The dead `dispatch` nav entry (`nav.ts:236`) is removed by the integrator on L-B23's verdict. |
| L | `docs/program/console-enterprise-roadmap.md` | **Integrator** (was entirely absent from the previous map, despite being a declared collision root AND the implementation authority) | Emit a roadmap-delta line in the evidence manifest. |
| M | `docs/program/console-capability-registry.json` | **L-P0-EPOCH / program only** | No lane self-authorizes epoch normalization. |
| N | `web/src/console/shell/ConsoleShell.tsx` · `web/src/console/ConsoleApp.tsx` | **L-F1 for the wave** (14 path-hits/48h; three live codex lanes are on this surface) | Ownership must be confirmed with `codex/console-shell-scheduled-nav-20260724`, `codex/console-global-scope-foundation-20260724`, `codex/console-search-object-fabric-20260724` **before L-F1 starts**. |
| O | `web/src/console/window/**` | **L-F1** | Module lanes call `useOptionalWindowManager()` + `open(objectCardWindowEntry(...))` and nothing else. **`renderWithWindowManager`'s name and signature are FROZEN by L-F1's DoD** — 14 lanes hard-depend on it and it does not exist today. |
| P | `web/src/console/objectcard/**` — split at file level | **L-F2** owns `ObjectCard.tsx(.test)`, `wired.tsx(.test)`, `useObjectCard.ts(.test)`, `types.ts`, `strings.ts`, `stub.ts`, `index.ts`. **L-F3** owns `kinds.ts(.test)` only | `objectCardWindowEntry` is exported from **here** (`ObjectCard.tsx:863`), not from `console/window`. **L-F2's DoD freezes its public signature and descriptor type for the wave, proven by a type-level test.** |
| Q | `web/src/console/module/**` + `console/modules/**` | **L-F4** | Module lanes consume, never edit. **Primitive-change-request protocol:** a module lane that needs a primitive L-F4 did not build emits `frontend/manifests/primitive-request.json` (primitive · need · register anchor) and gap-manifests the deferral. It does not edit the shared file. |
| R | `web/src/console/grammar/**` (new dir) | Disjoint sub-paths: `list/` L-F4 · `state/` L-C8 · `mutation/` L-C3 · `responsive/`+`locale/` L-C9. `grammar/index.ts` = **L-F4 sole owner**; others emit an export line | |
| S | `web/scripts/check-ui-strings.mjs` · `check-console-purity.mjs` | **Integrator** (every lane's `lint` runs through them) | L-C8 is the only lane permitted to extend `check-ui-strings.mjs`, as a named, additive rule. |
| T | `backend/crates/docs/**` | **The docs owner** (L-C1 → L-B22a → L-B22, one agent, serial) | Read-reference for everyone else. The landed `EvidenceObjectCursor` is the normative cursor; a second encoding is a reject. |
| U | **`docs/program/console-{capability-registry,jurisdiction-register}.json`, `console-program-ledger.md`, `scripts/console/**`** | **Integrator (candidate-rebind owner)** | The truth-ledger gate's `assertAuthorityOnlyDiff` permits **only** these paths to differ between the frozen candidate SHA and the integration tip. Every code merge invalidates the ledger and forces a rebind. See §6.1-7. |
| V | **Live codex leaf branches — confirm ownership release BEFORE starting** | The branch owner | `codex/equipment-evidence-custody-20260724` → **L-D12** (read it FIRST; the P0 repair may already exist) · `codex/directory-backend-private-leaf-20260724` → L-B14F's backend · `codex/inventory-private-leaf-20260724` → L-D18 · `codex/dispatch-private-leaf-20260724` → L-B23F · `codex/attendance-frontend-private-leaf-20260724` + 3 `codex/attendance-*` → L-D8 · `codex/buck-{h0-classification,generated-gate-parser-fix}` + `fix/pr488-generated-face-authority` → **L-P0-BUCK is gated on their release** · `backend/crates/logistics` 55 hits/48h → L-B9 · `backend/app/tests/hr_people_create_api.rs` 26 hits (`codex/people-team-condition-regressions`) → any app-tier HR test. |
| W | New crates, anywhere | **Forbidden this wave** | A new crate forces a `gen_first_party.py` regen — a serialized generated face. This is why L-C1 lives in `kernel/core` and L-B5 is deferred. `backend/crates/listing` is **not** resurrected. |

---

## 5. CANONICAL LANE TABLE — the normative dispatch record

**60 lanes.** `PVL` = `param_verify_live`. `Slot` = provisional migration slot from the ledger (integrator renumbers; the lane emits a manifest, never a file under `migrations/`). Full `scope`, `roots`, `must_not_touch` and per-lane `dod` are in the structured lane set / `docs/program/wave4/lanes.json`; this table is the ownership and sequencing contract.

| Lane | Phase | Title | Deps | Slot | PVL | Size |
|---|---|---|---|---|---|---|
| L-P0-EPOCH | 0-t0 | Capability registry, epoch normalization, evidence base, slot ledger | — | — | n | M |
| L-P0-PARAM | 0-t0 | Live statutory parameter sweep (VAT, 내용연수, 검사주기, 비과세, 신고기한, 보존기한, 공휴일) | — | — | **y** | M |
| L-P0-ID-a | 0-t0 | Object-code triage, port-path assignment, code **format** decision | — | — | n | S |
| L-P0-ID-c | 0-t0 | Object-type registry authority decision + stale-doc corrections | — | — | n | M |
| L-A4a | 0-t0 | Projected command receipt design gate | — | — | n | S |
| L-B23 | 0-t0 | docs + dispatch dual-impl reconciliation DECISION | — | — | n | S |
| L-P0-BUCK | 0-t0 | Generated-face regen + `--only <crate>` + glob app-test target | (buck branches released) | — | n | M |
| L-A1 | 0-t0 | Catalog additive upgrade path **[train 1]** + `validate_draft` allowlist fix | — | 0205 | n | L |
| L-A6 | 0-t0 | Legacy `object_types` rows + code prefixes for 13 kinds | — | 0206 | n | M |
| L-C1 | 0-t0 | `kernel/core::paging` — extract the landed docs cursor, add sort-key enum | — | — | n | M |
| L-D0 | 0-t0 | Statutory parameter registry + **공휴일 calendar** + **retention periods** | L-P0-PARAM | 0203 | **y** | L |
| L-D0b | 0-t0 | Regulated-dataset ingestion: NTS 간이세액표 + 산재 업종별 요율 | L-P0-PARAM | 0204 | **y** | M |
| L-D3 | 0-t0 | Leave 촉진 truthfulness repair (**the only wrongly-fabricated finding**) | — | 0208 | **y** | S |
| L-D12 | 0-t0 | Equipment 3R handover custody repair (**P0, RED at HEAD**) | (custody branch read) | 0207 | n | M |
| L-F1 | 0-t0 | Window host: mount, tray-restore contract, a11y, `renderWithWindowManager` | (shell branches) | — | n | M |
| L-D1 | 0-t1 | Calc-artifact spine | L-D0 | 0209 | n | L |
| L-F2 | 0-t1 | Object card + explorer a11y; **freeze `objectCardWindowEntry`** | L-F1 | — | n | M |
| L-F3 | 0-t1 | Code-grammar unification (one dynamic source, four call sites) | L-A6, L-P0-ID-a | — | n | S |
| L-F4 | 0-t1 | Console list grammar + module engine + **personal-view config strip** | L-C1, L-F2 | — | n | L |
| L-C8 | 0-t1 | Truthful empty/denied/error states + **extension** to the existing lint gate | — | — | n | S |
| L-A3 | 1 | First live projected action (equipment) **[train 2]** | L-A1 | 0210 | n | M |
| L-B4 | 1 | Object-code issuance kernel + registry seed | L-P0-ID-a, L-A6 | 0211 | n | M |
| L-B7 | 1 | Approval-compose prefill contract | L-C1 | 0212 | n | M |
| L-B22a | 1 | Evidence retention merge (the reserved `0201` subject) | L-C1 | 0213 | n | S |
| L-D2 | 1 | Worktime interval engine (additive premium flags, **휴일 via L-D0 calendar**) | L-D0, L-D1 | 0214 | **y** | L |
| L-C3 | 1 | Conflict + mutation contract; pilot = ontology instance-command 412 path | — | — | n | M |
| L-A4b | 2 | Projected command receipts **[train 3]** | L-A4a, L-A3 | 0215 | n | L |
| L-A7 | 2 | Wave-2/3 module projections **[train 4]** | L-A1, L-A3, L-A4b | 0216 | n | L |
| L-A9 | 2 | Projected code resolution (R8) **[train 5]** | L-A7, L-P0-ID-c | 0217 | n | M |
| L-B21F | 2 | **RECIPE GATE** — identity/UsersPage + notif grammar port | L-F1..F4, L-C8, L-P0-ID-a | — | n | S |
| L-B15F | 2 | board (56) | L-B21F | — | n | M |
| L-B9 | 2 | logistics (42) full-stack | L-B21F, L-B4 | 0231 | n | L |
| L-B23F | 2 | dispatch FE port or deletion, per L-B23's verdict | L-B23, L-B21F | — | n | M |
| L-D4 | 2 | Payroll gross-to-net + **대근 pay derivation** + golden-case execution | L-D0, L-D0b, L-D1, L-D2 | 0218,0219 | **y** | L |
| L-D6 | 2 | 연차 accrual + §61 + 미사용수당 payout | L-D0, L-D2, L-D3 | 0220,0221 | **y** | L |
| L-D9 | 2 | Finance GL depth + VAT unification | L-D0, L-D1, L-P0-PARAM | 0222,0223 | **y** | L |
| L-D13 | 2 | Equipment rental economics + 내용연수 + 검사주기 | L-D12, L-D0, L-D1, L-P0-PARAM | 0224,0225 | **y** | L |
| L-D15 | 2 | Org effective-dating + **변경 동결창** + entity-profile read | L-D0, L-D1, L-P0-PARAM | 0226 | **y** | L |
| L-D16 | 2 | Evaluation scoring + PROBATION behaviour + subject-context read | L-D1 | 0227 | n | M |
| L-D17 | 2 | Recruiting statutory duties + pool-registration endpoint | L-D0, L-D1, L-P0-PARAM | 0228,0229 | **y** | L |
| L-D18 | 2 | Inventory valuation + **master CRUD REST route** | L-D1 (inventory leaf) | 0230 | n | M |
| L-C9 | 2 | Responsive + density + locale honesty (absorbs old L-C10) | L-F4 | — | n | M |
| L-X1 | 3 | **CAP-SALES-CRM vertical — the wave's user-visible deliverable** | L-A6, L-F1..F4, L-B4 | 0240 | n | L |
| L-B8F | 3 | equipment (40) | L-A7, L-D12, L-D13, L-B4, L-B7 | — | n | L |
| L-B10F | 3 | evaluation (50) | L-A7, L-D16 | — | n | M |
| L-B11F | 3 | payroll FE (52) | L-D4 | — | n | L |
| L-B13F | 3 | field (55) | L-A7 | — | n | M |
| L-B14F | 3 | directory (55) **FE-only**; backend half held on its leaf branch | L-A7, L-B4 | — | n | M |
| L-B16F | 3 | inventory (58) | L-A7, L-B7, L-F4 | — | n | L |
| L-B17F | 3 | org (58) | L-A7, L-D15 | — | n | L |
| L-B18F | 3 | maintenance (62) | L-A7 | — | n | M |
| L-B19F | 3 | recruiting (78) | L-D17 | — | n | M |
| L-B26F | 3 | docs/evidence FE port (`console/evidence/**`) | L-B23, L-B22a | — | n | M |
| L-B22 | 3 | docs records registry (non-EV) + egress-gated export | L-B22a | 0241 | n | L |
| L-D5 | 3 | Payroll run lifecycle + retro-runs-forward | L-D1, L-D4 | 0232,0233 | **y** | L |
| L-D7 | 3 | Leave integrity edges | L-D6 | 0234 | **y** | M |
| L-D8 | 3 | Attendance close integrity (**HELD** on the attendance writer) | L-D1, L-D2 | 0235 | **y** | L |
| L-D10 | 3 | GL derivation chains incl. **payroll → 급여 전표 posting** | L-D9, L-D4, L-D5 | 0236 | **y** | L |
| L-D11 | 3 | Cross-domain writeback: attendance exception → payroll, 대근 → contract → pay | L-D2, L-D4, L-D5, L-D6 | 0237 | **y** | L |
| L-D14 | 3 | Benefits engine + **four-eyes** + **cedar_policy_ref resolution** | L-D0, L-D1, L-D4, L-P0-PARAM | 0238,0239 | **y** | L |

**Migration slot ledger** (`docs/program/migration-slots.json`, append-only, integrator-owned): 0203 L-D0 · 0204 L-D0b · 0205 L-A1 · 0206 L-A6 · 0207 L-D12 · 0208 L-D3 · 0209 L-D1 · 0210 L-A3 · 0211 L-B4 · 0212 L-B7 · 0213 L-B22a · 0214 L-D2 · 0215 L-A4b · 0216 L-A7 · 0217 L-A9 · 0218-0219 L-D4 · 0220-0221 L-D6 · 0222-0223 L-D9 · 0224-0225 L-D13 · 0226 L-D15 · 0227 L-D16 · 0228-0229 L-D17 · 0230 L-D18 · 0231 L-B9 · 0232-0233 L-D5 · 0234 L-D7 · 0235 L-D8 · 0236 L-D10 · 0237 L-D11 · 0238-0239 L-D14 · 0240 L-X1 · 0241 L-B22.

**L-D12's direction is decided here, not by the lane:** read `codex/equipment-evidence-custody-20260724` first. If it already repairs the drop, **absorb it and cancel L-D12's custody scope**. If not, **re-add the column** (slot 0207) — not rewrite the adapter. Reason: re-adding is the smaller diff, turns the RED test green immediately, and leaves L-D13 and L-B8F's contracts unchanged. Rewriting the adapter changes both downstreams and is a different lane.

---

## 6. Definition of done — one normalized template

### 6.1 Universal (every lane)

0. **Re-run the §0 verification commands your lane depends on before writing code.** If a claim changed, escalate before proceeding. The §0 table is a snapshot; the commands are the state.
1. **No stubs.** No `TODO`, no `test.skip`/`.only`, no unimplemented branch, no filler copy. A `ponytail:` comment is allowed **only** where it names a ceiling and an upgrade path.
2. **Collision roots are manifests, never edits.** §4 is binding. Emit to `docs/evidence/console/CAP-<CAP>/{api,frontend,backend}/manifests/`.
3. **Truthfulness.** No fabricated rows, totals, codes, relations, or statuses. Honest-empty pattern: `web/src/console/leave/model.ts:199-203`. Every deferral is a **named line** in the gap manifest with its repo-relative register anchor — `docs/evidence/console/wave4/fidelity-registers.json#/registers/<module>/findings/<n>` or `…/depth-registers.json#/registers/<domain>/rules/<n>`. Never silence, never a disabled control. **A gap-manifest entry carrying a register anchor plus a named missing backend contract satisfies the registry's `hold_rule`** (§9-3 carries this amendment).
4. **No dead affordance.** A control that cannot act is omitted (deny-by-omission), not disabled.
5. **Push discipline.** `git fetch && git merge origin/<spine>` — plain merge; rebase is classifier-blocked. Never write `migrations/**` or `**/BUCK` directly.
6. **Evidence doc** at `docs/evidence/console/CAP-<CAP>/` with the lane id, findings closed, findings truthfully deferred.
7. **Truth-ledger admission (NEW — this gate is CI-blocking on the spine).** Commits are **signed** (`git config commit.gpgsign true`; `git verify-commit` must pass). A canonical review receipt exists at `docs/evidence/console/reviews/<CAP-ID>/<candidate_sha>.json`. The lane is registered through `scripts/console/plan-fanout.mjs` — 60 hand-written lanes bypassing the spine's machine-checked fanout planner is not admissible. Only §4-U paths may differ between the candidate SHA and the integration tip; the integrator owns the rebind.
8. **Independent review (NEW).** A lane is complete only after a **review-gate pass by an agent other than the author** — correctness + RLS-as-`console_rt` security + codex cross-model, plus the browser/a11y axis for any `web/` lane. The GO verdict and every must-fix closure are recorded in the evidence doc. **The integrator does not queue a manifest from a lane without one.** Standing project rule: never self-approve in the same active context.

### 6.2 Backend lanes additionally

9. `cargo fmt` clean; `cargo clippy -p <owned crates> -- -D warnings` clean. (Local cargo runs are hook-disabled in the main session only — compile-verify in a spawned subagent before pushing.)
10. **RLS `FORCE`, every assertion executed as `console_rt`.** Bootstrap with `console_buck_admin` + the `mnt.sqlx_test_bootstrap` GUC; assert as `console_rt`. A superuser `BYPASSRLS` pass proves nothing — project memory records a totally broken read path masked exactly this way.
11. Deny-by-default PBAC; **audit row in the SAME transaction** as every mutation.
12. **Canonical error envelope** `{error:{code,message}}`. **422** for validation, **409** for conflict. **One negative test per lane drives a DB CHECK violation and asserts 422 (or 409 for a uniqueness conflict) — never 500.** (Previously a parenthetical; a parenthetical cannot fail a build.)
13. **Idempotency, correctly stated (REWRITTEN).** A replay carrying the same idempotency key **and a matching canonical fingerprint returns the FIRST outcome** — identical status and body, **no second write, no second audit row**. A same-key / different-fingerprint replay returns 409. **One test per lane asserts both branches, run as `console_rt`.** (The old wording — "idempotency key + canonical fingerprint on every create/decide" — is satisfied by storing the key and 409-ing every replay, which is duplicate-rejection, not idempotency, and reads to a retrying mobile client as a failure that actually succeeded.)
14. One **story-level** app integration test as `console_rt`. Emit `manifests/buck-app-test.json`; do not edit `backend/app/BUCK`.
15. `buck2 test //backend/crates/<owned>/...` green — Buck2 is the completion evidence, cargo is dependency metadata.
16. **CI gates, split by obligation.** **UNCONDITIONAL** for any lane adding a migration, a table, or a route: `rls-arming`, `tenant-isolation`, `audit-coverage`, `dev-auth-absence`. A lane that believes one does not apply records the reason in the evidence doc and gets it accepted at review. **As applicable:** `layer-boundary`, `pii-no-logs`, `migration-safety`.
17. openapi fragment manifest for any route change. Migration manifest, self-contained, no cross-lane FK.

### 6.3 Frontend lanes additionally

18. **§4-25 closed loop + §4-21 benchmark pass (NEW — the amendment's own definition of done).** Run the eight-question loop and the three-question Palantir/Workday/Slack/Greenhouse/SAP pass against the ported surface, grounded in BENCHMARK.md's gap column; commit the ranked output as the module's next-slice register. **A module lane is not done at prototype parity.** This is also what makes §3a's C-64 reconciliation honest.
19. **The window contract paragraph pasted verbatim into the PR body and honoured:** the lane uses `useOptionalWindowManager()` and `open(objectCardWindowEntry(...))` — imported from **`console/objectcard`** — and touches nothing else in `console/window/**`. It does not promote `useWindowEngine` and does not use `features/workspace`.
20. **Three unit assertions per ported surface** under `<WindowManagerProvider>` via `renderWithWindowManager`:
    - (a) the row/chip renders `[draggable="true"]` **and** the expected `data-obj-code` — **except for bucket-3 modules** (see 21);
    - (b) **activation pins the target AND the previously pinned entry remains recoverable** in the tray with a Korean accessible name. *(The old wording — "activating the row sets `pinnedId`" — passes green while the user's previous card silently disappears; with 13 modules opening cards that reads as data loss.)*
    - (c) `getByRole("button")` resolves the drag host — a focusable `<button>`, never a `span`/`li`/`article` with `objDrag` spread on it.
21. **Bucket-3 (intentionally code-less) DoD variant.** `0113_create_object_code_counters.sql:49` states `person` and `org_unit` deliberately keep `code_prefix = NULL`, and `equipment_3r_rental_cases` has no code column at all. For modules L-P0-ID-a classifies as code-less: assert (b) and (c) only; the missing code is a **named gap-manifest line with its register anchor**. **No lane fabricates a code from a UUID.**
22. **Real-browser proof (NEW — mandatory for L-F1, L-F2, L-F4, L-B21F, L-X1).** jsdom is the harness that hid the bug this wave exists to fix: the exemplar everyone was told to copy is green in jsdom and dead in production. Navigate the dark `/console` route in a real browser; assert `WindowManagerProvider` is present in the **live React tree**, not the test harness; run one keyboard journey (open → pin → Escape → focus returns to the invoking control) and one axe pass on the mounted window layer; commit the trace. Remaining module lanes inherit via a **shared journey fixture parameterized by screen key**. Infrastructure already exists: `playwright.config.ts`, `e2e/fixtures/ux.ts` (AxeBuilder, wcag2a/2aa/21a/21aa), the `dev-auth` project. **jsdom-only is not completion evidence for any lane asserting window behaviour.**
23. **A11y AA is a check, not a claim.** `CONSOLE_DEV_AUTH_E2E=1 npx playwright test --project=dev-auth e2e/specs/chrome-02-axe.spec.ts` over each ported surface, committed to the evidence doc. Plus, explicitly: 1.4.10 reflow · 2.4.7 focus visible · 1.4.11 non-text contrast · 2.5.8 target size (≥44px) · Korean accessible name on every icon-only control · status as a **text chip, not colour** · informational chips not focusable.
24. `npx vitest run <lane files>` green + `pnpm -C web tsc --noEmit` clean + `npm --prefix web run lint` (which already runs `check-ui-strings.mjs`, **extended** by L-C8 — the gate is live against you today, it is not shipped by L-C8).
25. **No explanatory UI** (binding merge gate): no subtexts, subtitles, captions or filler. Status = chips. Only action-driving copy. Benchmark against Palantir/Teams/Slack.

### 6.4 Keyed on what a lane DOES, not on which lens it came from

*(The previous §5.4 was keyed on lens membership, which left a `param_verify_live` lane and two full-stack lens-B lanes outside the statutory DoD.)*

26. **Any lane with `param_verify_live: true`:** every statutory constant carries **its citation in the test name**; every parameter is resolved through `kernel/core::statutory`, never a local literal; every `needs-verification` item resolves to `Err(ParamUnverified)` — **no `unwrap_or` on a statutory resolve** (L-D0 ships the grep gate).
27. **Any lane writing domain state transitions or money/time arithmetic:** the **edge-case matrix**, one test per row of **your domain's own `edge_case_gaps` array** cited by anchor, **plus** the five universal categories (mid-period join/leave · backdated correction + effective-dated recalculation · concurrent transition race · reversal/compensation path · boundary dates: KST month-end, 회계연도 boundary, the 2026-07-01 NPS cap boundary, week-start Monday). **A gap with neither a test nor a named deferral line is a blocker.** *(The generic five-category matrix let 131 enumerated domain-specific gaps fall through — e.g. payroll's "a run with `calculation_status=BLOCKED_LEGAL_GATE` lines can reach APPROVED→PAID→ISSUED with those employees silently unpaid", which matches none of the five and is criminal exposure.)*
28. **Any lane owning one side of a cross-domain invariant:** close or **name-defer with its anchor** every `cross_domain_invariants[n]` where your domain is the owning side. 110 invariants (29 explicitly missing) cannot be funnelled into L-D11.
29. **Grep gate** (L-D5, L-D8, L-D10, L-D11): no `tokio::spawn` / `LISTEN` / `NOTIFY` introduced for recalc detection. "Needs recalculation" is the pure query *"committed input versions newer than the version my committed output consumed"* — no listener, no daemon, no missed-message failure mode.
30. **Retro runs forward.** Payroll is cumulative: reprocessing runs forward from the reprocess date across **all subsequent runs**, resolving parameters **as-of the reprocessed period**. L-D5 carries a test that a single-period implementation fails.

---

## 7. Truthfulness: parameters, the gated/computable line, and the fabrication index

### 7.1 `param_verify_live` — regenerated mechanically from the register

Derivation command, recorded so it can be re-run after any lane reassignment:

```
python3 -c "import json;d=json.load(open('docs/evidence/console/wave4/depth-registers.json'))['result'];
[print(r['domain'],i,x['rule'][:80]) for r in d['registers'] for i,x in enumerate(r['rules']) if x.get('param_verify_live')]"
```

Every rule the register marks `param_verify_live: true` appears below under its owning lane. Parameters not in the brief are fetched by **L-P0-PARAM once, before the wave**, and appended to the committed brief at `docs/evidence/console/wave4/research-statutory-params.md` §8/§9 with `source_url` + `retrieved_on`. Five lanes each pausing mid-build to source law is five independent chances to fall back on model memory — the single behaviour the doctrine forbids. **A lane whose parameters L-P0-PARAM failed to fetch is not chartered; the gap is registered.**

| Lane | Parameters | Source (corrected) |
|---|---|---|
| **L-P0-PARAM** | 부가가치세법 §30 매입세액공제 · §14 rental/resale 10% split · 법인세법 시행규칙 별표 기계장치 내용연수 · 산안법 §93 / 건설기계관리법 §13 검사주기 · 소득세법 §12 비과세 한도 won-amounts · 근기법 §24③ + 4대보험 자격상실신고 기한 · 근기법 §42 서류 보존기한 · **2026 관공서 공휴일 + 대체공휴일 고시** | **All fetched live; none is in the brief.** |
| **L-D0** | The registry itself: 4대보험 rates + caps, 최저임금, 209h divisor, 소득세 basis, **공휴일 calendar (effective-dated, per year)**, **문서 보존기한** | brief §2, §8 + L-P0-PARAM; seeds V1–V14 as `NeedsVerification` |
| **L-D0b** | NTS 간이세액표 rows (Excel ingest, **never transcribe**) · 산재 업종별 + 출퇴근재해 요율 (data.go.kr 15068737, **never hand-type**) | The datasets themselves; checksum-pinned |
| **L-D2** | 주52 (근기법 §53) · 가산 §56 · 휴게 §54 · **휴일 classification from L-D0's calendar** | brief **§3.1/§3.2/§3.4/§3.5** *(was mis-cited to §1, which is 최저임금)* |
| **L-D3** | 근기법 §61 촉진 procedure timing | brief §4.2 |
| **L-D4** | 4대보험 rates/caps/bases · 장기요양 = 건보료 × 0.9448/7.19 **on the rounded 건보료** · 주휴수당 · 최저임금 하한 · **대근 pay derivation (SR-206)** | brief §2, §8 |
| **L-D5** | Retro must resolve parameters **as-of the reprocessed period**, not current-year | brief §8 (effective-dated reads) |
| **L-D6** | §60 accrual tiers · §61 windows · §60⑥ (V5) · **V7 미사용수당 기준임금 (통상 vs 평균)** · **공휴일 charge-zero** | brief §4.1/§4.2/§4.3, §9 |
| **L-D7** | §60/§61 boundary edges | brief §4 |
| **L-D8** | 주52 enforcement · **business-hours + 공휴일 accrual** | brief §3.1 + L-D0 calendar |
| **L-D9** | **부가가치세법 §30 매입세액공제** — the effective-dated table already in `erp/domain` is the correct-but-unwired impl | **L-P0-PARAM** *(was mis-cited to brief §5, which is 소득세 간이세액표; the brief contains zero VAT content)* |
| **L-D10** | VAT derivation on the source→voucher chain | L-P0-PARAM |
| **L-D11** | 대근 → C-D contract issuance → pay reflection | brief §3.2 + L-D0 |
| **L-D13** | 내용연수 · 산안법 §93 · 건설기계관리법 §13 · VAT 10% rental/resale split | L-P0-PARAM |
| **L-D14** | 소득세법 §12 비과세 한도 · 국민연금법 §88 · 국민건강보험법 §73 + 장기요양 · 고용보험·징수법 · **산재 업종별 요율 (consume L-D0b, never fetch)** | L-P0-PARAM + L-D0b |
| **L-D15** | 근기법 §24③ 협의 기간 · 4대보험 자격상실신고 기한 · 퇴직금 14일 (근퇴법 §9) | L-P0-PARAM |
| **L-D17** | 채용절차법 §10/§11 · PIPA consent + retention; V9/V10/V12 unresolved | brief §6, §7, §9 |

**`param_verify_live: false` on every other lane, derived from the command above — not asserted.** Two rules follow from that:
- **Retention periods and the holiday calendar are L-D0 registry rows.** L-B22, L-D14 and every consumer resolve through `kernel/core::statutory` and are **forbidden from fetching**. Two independent live fetches of the same gazetted parameter is exactly the divergence L-D0 exists to abolish. **L-D0 ships a grep gate asserting no lane carries a local holiday list, a local retention table, or a weekday-only 휴일 heuristic.**
- If any lane discovers a regulated parameter mid-build it **stops and escalates** rather than sourcing from model memory.

### 7.2 The truthfulness line

**Stays a gated attestation, no matter what** (a lane that computes one has failed): final NTS 간이세액표 rows · 산재 업종별 요율 table · 노무사/세무사 sign-off and the payroll release gate · 공단 고지액 · bank-transfer attestation · 전자세금계산서 relay refusal · the six org dissolve settlement attestations · 전적 per-employee consent · equipment handover custody evidence · 산안법 §81 대여자 조치 서면 · the legal-notice passkey receipt chain · candidate offer acceptance.

**Currently gated but computable — ALL 18, enumerated from the register, with an owner** (a lane that gates a computable rule has also failed). The previous charter named 2 of 18 and presented them as the set.

| # | Anchor | Rule | Owner |
|---|---|---|---|
| 1 | `payroll.rules[0]` | 골든케이스 재계산: stored `expected_total` validated for version match but **never executed against `build_line_calculation`** — a wrong expected value passes the gate that 노무사/세무사 sign-off leans on | **L-D4** |
| 2 | `payroll.rules[11]` | 연장근로 가산 ×1.5 — hours captured, math never run, human confirms blindly | **L-D4** |
| 3 | `payroll.rules[14]` | 일할계산 proration — `PRORATION` exception admitted by CHECK, no producer, no formula | **L-D4** |
| 4 | `payroll.rules[15]` | 소급(retro) + effective-dated recalculation — `version > 1` unreachable via REST | **L-D5** |
| 5 | `payroll.rules[16]` | 최저임금 하한 — tables exist with **zero callers** | **L-D4** |
| 6 | `attendance.rules[12]` | **대근비 산정 (SR-206)** — `worker_rate` correctly refuses to fabricate, but the deterministic derivation is absent | **L-D2** (classification) + **L-D4** (pay) |
| 7 | `attendance.rules[18]` | `projected_hours` = `current_hours` — a named field whose computation does not exist | **L-D8** — §9-5 decides compute-or-delete |
| 8 | `leave.rules[8]` | §61 촉진 two-round statutory windows — `validate_round` only checks `round∈{1,2}` | **L-D3** + **L-D6** |
| 9 | `evaluation.rules[14]` | PROBATION cycle kind: stored enum, zero behavioural difference from REGULAR | **L-D16** |
| 10 | `org.rules[9]` | **변경 동결창: `effectuate` has no freeze check at all — a payroll-period apply is possible** | **L-D15** |
| 11 | `inventory.rules[6]` | Master CRUD fully audited in the adapter, **no REST route** — collides with the standing rule that all mutations go through the audited console API | **L-D18** |
| 12 | `maintenance-workorder.rules[5]` | 정산 → 전표: `voucher_ref` read but **never written**; settlement posts nothing | **L-D10** |
| 13 | `maintenance-workorder.rules[8]` | SLA escalation: `Delayed` exists in the FSM, no path reaches it | *deferred §8, anchor recorded* |
| 14 | `maintenance-workorder.rules[12]` | Outsource lifecycle: 5 of 6 statuses have no transition | *deferred §8, anchor recorded* |
| 15 | `field-support.rules[18]` | `SupportCase` aggregate: ~900 lines of tested, unreachable scaffolding | *deferred §8 — §9-5 decides wire-or-delete; **deletion is the default*** |
| 16 | `finance.rules[4]` | 승인→전표 파생: `create_draft_from_source` exists with **zero callers** repo-wide | **L-D10** |
| 17 | `benefits.rules[4]` | **four-eyes: `transition_lifecycle` records the actor but never compares it against the prior transition's actor — one principal can self-approve.** A live SoD bypass on the spine | **L-D14** |
| 18 | `benefits.rules[6]` | **`cedar_policy_ref`: ≤200-char text check only; dangling/fabricated refs accepted** | **L-D14** |

Three are promoted to named DoD lines: **L-D4** — "the stored golden-case expected total is executed against the kernel; a deliberately wrong expected value must make the test RED." **L-D14** — "`transition_lifecycle` rejects an actor equal to the prior transition's actor, tested as `console_rt`." **L-D14** — "`cedar_policy_ref` resolves against the policy store; a dangling ref is 422, not accepted."

### 7.3 Statutory traps copied forward

1. **연장 + 야간 stacks to 2.0×.** Premiums are **additive independent flags on a time segment**, never an enum. The current model *cannot represent it* — hour buckets are pre-flattened, so overlap is structurally unrecoverable at calc time. This is why L-D2 is foundation, not a payroll sub-task.
2. **휴일 and 연장 do NOT stack** for the same hour; 휴일 has its own 50/100 ladder. 야간 is the only orthogonal add-on. **The ladder is unimplementable without L-D0's 공휴일 calendar** — today work on a holiday is indistinguishable from a weekday.
3. **장기요양 is computed on the rounded 건강보험료**, not on income. The current code does it the income way; the divergence shows up won-level against every NHIS EDI 고지서.
4. **NPS caps run July→June; the rate runs calendar.** Effective-dating is per parameter, not per year.
5. **제53조제3항 (<30인 +8h) expired 2022-12-31.** A 60h/week cap in 2026 is wrong and punishable; L-D2 asserts 60h is unreachable.
6. **휴게 must be interior to the shift**; a trailing break does not satisfy §54.
7. **연차 미사용수당 defaults to PAY.** §61 gives only the negative. Fail-open-to-paying is safe; the reverse is 임금체불.
8. **Rounding default is none.** Rounding down against the worker is 임금체불. The knob is data, effective-dated, symmetric by construction (`CHECK (increment > grace)`), default off.
9. **제11조 <5인 applicability is a data table** (`article, min_headcount, effective_from`), never an `if headcount < 5`. Flipping a row flips the outcome with no code change — asserted by a test in L-D6.
10. **Reversal into a closed period is forbidden** — it posts into the current open period with the original linked. SAP negative posting is skipped (`ponytail:` ceiling: compute net-of-reversals at read time).

### 7.4 Named fabrication temptations → lane + DoD line

| Temptation | Lane | DoD line that forbids it |
|---|---|---|
| A zero-filled attendance plan | *(L-D22 cut — §8)* | Anchor recorded; the honest-empty rule (§6.1-3) binds the surviving attendance work |
| A UUID dressed as an object code | **L-B14F** | §6.3-21 bucket-3 variant: no test asserts a UUID as a code |
| A client-side fake pool registration | **L-B19F** | The CTA is wired to L-D17's real endpoint or gap-manifested; never a client fake |
| A DTO-computed 공제 column | **L-B11F / L-D4** | Every exposed amount traces to an engine-computed value with a test naming its source |
| A polled number labelled 실시간 | *(L-C4 cut — §8)* | No surviving lane may render a 실시간 label over a polled number |
| A client sort of page 3 of 40 presented as a sort | **L-F4** | Header buttons ship **disabled with a stated reason** on endpoints that have not adopted L-C1 |
| A fabricated `total` | **L-C1 / L-F4** | `total: Option<i64>`; `aria-rowcount="-1"` when absent. A fabricated total is a defect |
| A gate that reports validation it never performed | **L-D4** | The golden-case DoD line: a wrong expected value must make the test RED |

---

## 8. Deferred / cut — every item with its register anchor

### 8a. Deduplicated into another lane

lens A **L-A8** → **L-A1** (`validate_draft` + `seed.rs` comment) and **L-P0-ID-c** (the two `.md` header corrections) and **L-A6** (`typeRegistrySource.ts` header comment) · lens A **L-A5a** + lens B **L-B3** + open-decision C → **L-P0-ID-a / L-P0-ID-c** · lens B **L-B0** + window half of **L-C5** → **L-F1** · lens B **L-B2** + card half of **L-C5** → **L-F2** · lens B **L-B15**'s ListTable half + lens C **L-C2** → **L-F4** · lens C **L-C10** → **L-C9** · lens B **L-B12** → **L-D4/L-D5** · lens B **L-B25** → *(cut with L-D22)* · lens B **L-B20** + **L-B21** → **L-B21F** · BE halves of **L-B8/L-B10/L-B17/L-B19** → **L-D12/L-D13, L-D16, L-D15, L-D17**.

### 8b. CUT to wave 5 — each with the reason and the anchor

| Item | Reason | Anchors left open |
|---|---|---|
| **L-A2 — schema publish route** | **No consumer in the wave.** Neither `web/src/api/ontology.ts` nor `console/ontology/wire.ts` has a publish call, so nothing can call it — while it costs `openapi.yaml` (68 hits/48h, whole-file-revert scar) plus three regenerated client faces. Charter it in wave 5 **bundled with its frontend caller** so the cost buys a working loop. L-A1 already unblocks every wave-4 seeded-type need. | ONT-1, C-6 |
| **L-C4 — realtime hardening** | Builds affordances with no server-side source. `/api/v1/ws` carries exactly two event variants; `realtimeHub.ts` has **zero** console consumers; presence has no realtime transport at all. Extending the notifier seam to arbitrary module objects is a platform lane, not "hardening" — and shipping UI for a signal the server never sends violates §6.1-4. | `attendance.findings` 실시간 lie; BENCHMARK structural gaps |
| **L-C6 — bulk operations** | Depends on L-F4's `selectionSlot` seam and on L-C1 query-scoped identity, and **no module lane this wave has a bulk action to wire**. Building the primitive before the first consumer is the speculative-abstraction failure. | C-41, §4-22 in dispatch/inventory/maintenance/field/board/directory/equipment |
| **L-C7 — draft autosave/restore** | Same: its pilot is "one existing 기안 draft", and no wave-4 lane owns a draft-bearing form. **This also dissolves the L-C7 root contradiction** — it declared `features/workspace/persistence.ts` while the universal FE DoD forbids `features/workspace`. When a consumer exists, the thing to extract is `persistence.ts`'s save-disabled-until-load-succeeds guard, which is the highest-value line in the file. | §4-27 draft affordances |
| **L-C11 — virtualization** | Measurement-gated, may legitimately produce zero code, and the wave's only contemplated new dependency. Downgraded to a **measurement obligation inside L-F4's DoD**: record real row counts for the evidence register and messenger history; if either exceeds the threshold, file a wave-5 lane with the measurement attached. | — |
| **L-B5 — object typeahead / search fabric** | `codex/console-search-object-fabric-20260724` owns the contract; the proposed root was a new crate (§4-W). Consumers **L-B8F, L-B10F, L-B16F, L-B17F, L-B18F** gap-manifest it. | `inventory.findings[?]` blocker, `evaluation`, `maintenance`, `equipment`, `org` majors |
| **L-B6 — operational object runtime read surface** | **This is the spine branch's own work.** Verified `ontology/rest/src/lib.rs:1562-1588` — `instance_acting`/`object_type_acting` are live routes, so the dynamics-layer findings are unblocked **by registration alone** (L-A7), with no new endpoint. That is why L-A7 is high-value despite shipping no handlers. | — |
| **L-B0b — 4-state window provider** (popout/split/presets) | The 3-state provider is test-covered and needs zero per-module config; `useWindowEngine` needs a per-screen `CardRegistry` per module (13×), a host, persistence, and the entire §5.2 window-a11y contract axe cannot verify. **§9-4 is the program decision.** L-F1 closes the *data-loss reading* via the tray-restore contract; it does not build multi-pin. | payroll[0], attendance[0], evaluation, org, docs §4.7-2 blockers |
| **Server-persisted per-person layout** | Only the *legacy* `features/workspace` has it, on the model nobody uses. **L-F1 carries a named gap-manifest line** for the localStorage ceiling (`oyatie.console.window.layout.v2.*` violates do-not-ship ban #9) plus a `ponytail:` comment naming the upgrade path. Six §4.7-2 violations stay open, **attributable to a lane, not to a footnote**. | 6 × §4.7-2 |
| **L-D19 field-support SLA · L-D20 maintenance WO+PM · L-D21 notifications/notices · L-D22 attendance timetable** | Below the cut line; cut to fund L-X1 and to keep the wave inside the concurrency cap. **`field-support.rules[18]` (SupportCase, ~900 dead lines), `maintenance-workorder.rules[8]` and `[12]` (§7.2 rows 13-15) are named §7.2 items with no wave-4 owner — that is the point of naming them.** | `field-support`, `maintenance-workorder`, `notif-board-routing` depth registers in full |
| **L-B24F attendance FE · attendance schedule BE** | `CAP-ATTENDANCE-CONSOLE` is `writer_assigned_gap_closure_in_progress`; `features/attendance` is the hottest FE dir (145 hits/48h); `backend/crates/attendance` is tier-1 (106 hits) with 4 live codex lanes. **Fallback, stated now so it cannot resolve into silence: if ownership is not released by dispatch time, attendance is formally out of wave 4 and its two registers are gap-manifested as a block with anchors.** L-D8 carries the same gate. Note the body is at `web/src/features/attendance`, under **two** shells. | attendance 3 blockers + 4 majors; ~30 depth rules |
| **The registry BRIDGE implementation** | Shape undecided; L-P0-ID-c produces the decision, the bridge is a wave-5 lane. | — |
| **Bulk §18 action handlers** | Prove the pattern once (L-A3), charter per-target. `app/src/lib.rs` is the hottest backend file. | — |
| **Four-eyes on projected actions** | Deferred, not solved. L-A3 ships `authority` + `self_checklist` only (R6: a failed dispatch spends the approval). **No wave-4 lens-D lane may require a four-eyes-gated projected action before L-A4b lands.** | — |
| **Generalizing `idempotency_keys`** | §6.2-13 now requires real first-outcome replay **per lane**; generalizing into a shared platform mechanism is a wave-5 platform lane. | — |
| **SAP-style negative posting** | `ponytail:` ceiling in L-D9 — net-of-reversals at read time; upgrade path named. | — |

### 8c. `[>190]` intents — the authority's newest, previously invisible

`design-intent-register.md` was never cited by the previous charter. Its 14 post-190 intents are the freshest authority and target the **one exposed screen**.

| Intent | Wave 4 |
|---|---|
| **CRM-1** DL- deal type joins the ontology; stages as lifecycle flow lanes; all stats derived | **L-X1** |
| **CRM-3** Stage transitions are audited actions with a per-stage evidence enum, fail-closed | **L-X1** |
| **CRM-6** Won deal auto-converts to contract `C-` through the guarded composer | **L-X1** |
| **CRM-2** Activity discipline + deterministic auto-Lost as a settings object | wave 5 — recorded |
| **CRM-4 / B-26** D-90 renewal automation with dedupe | wave 5 — recorded |
| **CRM-5 / WFL-9** wf10 round-robin on a real rotation roster | wave 5 — recorded |
| **POL-7** Rule-builder policy-link row, DoA clause authoring draft→four-eyes→active, TK- settings-object ledger | wave 5 — recorded |
| **WMS-5** Vendor-suite depth in existing grammar; WMS after CRM | wave 5 — the C-64 order |
| **C-64** One-module-at-a-time full maturity, thin fan-out banned | **§3a + §9-0 waiver** |
| **C-41** Add-anything N+1 in place | **L-F4** config strip DoD |

### 8d. The 69 backend-blocked fidelity findings

The previous charter reasoned by class and left several classes with neither a lane nor a deferral row. **L-P0-EPOCH generates `docs/program/wave4/backend-blocked-index.json` — one row per finding: `module.findings[n] → owning lane | §8b deferral id` — and it is the integrator's completion checklist.** Generated, not hand-copied, because a 69-row table in prose goes stale on the first merge.

**Seven classes have no wave-4 owner today. That is the point of naming them:** `SR-` series objects (C-33: payroll[4], equipment[1], maintenance[3]) · egress-gated export + audited-forbid original access (C-17/B-4: docs[3][5][7], maintenance[6], recruiting[11] — **partially closed by L-B22**) · map/geo (dispatch[2], B-20) · mail surface (directory[0] 메일 act, field[4], B-10) · data-ingest surface (inventory[13]) · `JL-` 업무일지 API (field[14]) · workforce-pool registry beyond L-D17's endpoint (attendance[9], WF-1…4).

---

## 9. Open decisions the program must resolve BEFORE dispatch

0. **C-64 waiver.** C-64 `[>190]` bans thin fan-out and orders CRM→WMS→MES. This charter fans out. Either grant an explicit written waiver — *"wave 4 is a substrate + depth-per-module wave; C-64's one-at-a-time clause applies from wave 5, CRM first"* — or restructure to 2-3 modules at full §4-25 maturity instead of 14 at floor+depth. **Do not dispatch against a standing ban without the waiver written down.** (§3a)
1. **The epoch contract vs plain merge.** `console-fanout-epoch-contract.md:115-118` admits only a serialized rebase/cherry-pick train; rebase is classifier-blocked on this spine, so the contract is currently unsatisfiable. Amend the contract to admit merge-trains, or the 60 lanes are formally inadmissible. (§3b)
2. **Exposure.** `EXPOSED_SCREEN_KEYS = ["sales"]`, locked by `nav.test.ts:39,58`. With L-X1 chartered, wave 4 *can* produce visible value — but only if the sales exposure survives and the CRM work is allowed to reach it. Decide whether any additional screen is exposed this wave; if none, say so, and the wave's visible surface is exactly L-X1.
3. **Epoch normalization + registry corrections are WORK, not a decision — owned by L-P0-EPOCH.** Normalize 60 lane ids; seed `CAP-ONTOLOGY-ENGINE`, `CAP-STATUTORY-KERNEL`, `CAP-CONSOLE-GRAMMAR`, `CAP-CONSOLE-PAGE`, `CAP-LEAVE`, `CAP-FINANCE-GL`, `CAP-SALES-CRM` with roots verified to exist on disk; **correct `CAP-DOCS-EVIDENCE-CONSOLE.frontend_roots` from `web/src/console/docs/**` (absent) to `web/src/console/evidence/**`**; reconcile or retire `CAP-EVALUATION-001`; refresh `source_revision` (pinned at `origin/main@8e42b9a2`, ~900 commits stale, while `last_refreshed` claims 2026-07-25); and carry the §6.1-3 `hold_rule` amendment. **No lane dispatches before it completes.**
4. **Is L-B0b (4-state provider) in wave 4?** Deferring is the recommendation; it costs five registers' pin/popout/tray/preset blockers, gap-manifested truthfully. L-F1's tray-restore contract closes the data-loss reading but not the blockers.
5. **Two decide-and-record obligations lanes must not silently choose.** `attendance.rules[18]` `projected_hours` — compute it or delete it (a named field that lies is worse than an absent one). `field-support.rules[18]` `SupportCase` — wire ~900 lines of unreachable-but-well-tested scaffolding or delete it; **deletion is the default**, and the register's phrase *"reads as depth but cannot execute"* is the citation. Both lanes are cut (§8b), so **these are program calls, recorded as anchors.**
6. **Statutory registry in Rust (`kernel/core`), not a DB table** — recommendation taken. The DB carries only genuinely per-org attested facts (상시근로자수 / 우선지원대상기업 / 업종코드) plus org conventions. If the program wants DB-governed `RG-` rows, decide now — L-D0 is the wrong lane to discover it in.
7. **Attendance ownership release** (L-D8, and the cut L-B24F/L-D22 if reinstated) confirmed with the CAP-ATTENDANCE-CONSOLE writer **before** dispatch.
8. **Worktree/disk budget** for 8 concurrent lanes (≈150 GB). If unavailable, the cap is whatever the disk allows.

---

## 10. Judgment responses

### Execution-feasibility judge

| Finding | Response |
|---|---|
| **[blocker]** No per-lane root table; lens charters carry superseded slots | **CHANGED.** §5 is the normative table; lens charters declared SUPERSEDED for roots/slots/phases/deps in the header; full per-lane roots + `must_not_touch` in the structured lane set / `lanes.json`. |
| **[blocker]** Reversed L-A2 edge + unserialized `seed.rs` write | **CHANGED, by deletion.** L-A2 (publish route) is **cut to wave 5** (see the sequencing judge — no consumer, costs the repo's most contended file). Its `validate_draft`/`seed.rs` items move into **L-A1** as part of link 1; the doc-header items go to L-P0-ID-c and L-A6. The reversed edge no longer exists. |
| **[blocker]** L-A2 and L-A9 both own `validate_draft` | **CHANGED.** `validate_draft` belongs to **L-A1**. L-A9 keeps `resolve_by_code` + `ObjectTypeSummary` only and lists L-A1 in `must_not_touch`. |
| **[blocker]** `kernel/core::paging` is layer-boundary-illegal | **CHANGED.** L-C1 is now a **pure** extraction: cursor codec, `ListPage<T>`, sort-key enum contract, validation errors (one `base64` dep line via the §4-D manifest). No SQL, no sqlx. The predicate/snapshot query stays inside the `docs` adapter it is extracted from. Cross-ref corrected to §4 rules **D and W**. |
| **[blocker]** L-C1's pilot has no crate quartet | **CHANGED, by rescope.** The pilot is the crate L-C1 extracts *from*: `backend/crates/docs` delegates to the shared type and its existing cursor tests pass unchanged; one sort-key enum lights up there. No new quartet, no `/api/v1/employees` (11.7k-line app file, two live writers), no resurrection of `backend/crates/listing`. |
| **[blocker]** Four lanes collide with unlisted live codex leaves | **CHANGED.** §4-V names all eight branches with their lanes. **L-D12 reads `codex/equipment-evidence-custody-20260724` first and absorbs-or-cancels** — the P0 may already be fixed. |
| **[blocker]** Truth-ledger admission gate no lane can pass | **CHANGED.** §6.1-7: signed commits, the `docs/evidence/console/reviews/<CAP>/<sha>.json` receipt, registration through `plan-fanout.mjs`, §4-U authority-only-diff paths. **Candidate rebind is bottleneck #5 with the integrator as named owner.** Verified: the gate is on the spine, not yet on this branch — it arrives with the next merge. |
| **[blocker]** Concurrency cap contradicted twice, unachievable | **CHANGED.** Cap is **8**. The `kernel/core/src/lib.rs` ordered slots are **deleted** (manifest line → integrator), which removes bottleneck 2 entirely. L-C8 and L-D3 moved off the critical t0 group. Worktree/disk budget stated (§2, §9-8). |
| **[major]** L-C8's Phase-0 premise false | **CHANGED.** Demoted to t1, rescoped to "the **existing** `check-ui-strings.mjs`, extended by L-C8"; §6.3-24 tells all 14 FE lanes the gate is live against them today. `check-ui-strings.mjs` + `check-console-purity.mjs` added as §4-S. |
| **[major]** Wrong owner for `objectCardWindowEntry`; `renderWithWindowManager` doesn't exist | **CHANGED.** §4-P cites `console/objectcard/ObjectCard.tsx:863` and **freezes** the public signature + descriptor type for the wave (type-level test in L-F2's DoD). §4-O freezes `renderWithWindowManager`'s name and signature in L-F1's DoD. |
| **[major]** L-C7 root contradicts the universal FE DoD | **CHANGED, by cut.** L-C7 is deferred (§8b) — it has no consumer this wave, which dissolves the contradiction rather than carving an exemption. |
| **[major]** L-B15F must edit files it is forbidden to edit; `ListTable` doesn't exist | **CHANGED.** `ListTable` struck. §4-Q adds the **primitive-change-request protocol**: emit `frontend/manifests/primitive-request.json` + gap-manifest; L-F4 stays sole writer. |
| **[major]** L-A4b omitted from the train order | **CHANGED.** §4-C: `L-A1 → L-A3 → L-A4b → L-A7 → L-A9`, one owner. L-A4b flips L-A3's `receipt: None` assertion in the same commit. |
| **[major]** The catalog train is the critical path and is never labelled | **CHANGED.** §2 opens with it. Fan-out re-ordered by **dependency, not score**: L-B21F (gate) → L-B15F/L-B9/L-B23F in Phase 2; the six L-A7-dependent lanes in Phase 3. Train owner gets nothing else. |
| **[major]** `kernel/core/Cargo.toml` and `Cargo.lock` ignored | **CHANGED.** §4-D covers all three; `Cargo.lock` is **regenerated** at merge, never merged as a diff. |
| **[major]** Migration "reservation" is a re-check, not a reservation | **CHANGED.** Append-only ledger `docs/program/migration-slots.json`, integrator-owned; slots are **requested**, never derived from `ls`; migrations become manifests (§4-G) and must be self-contained with no cross-lane FK so renumbering is a filename change. Duplicate-0197 evidence recorded. |
| **[major]** L-P0-BUCK: wrong file count, no partial-commit mechanism, three live contenders | **CHANGED.** 147→**161**; gated on the three buck branches releasing; ships `--only <crate>` as its own deliverable; `generated_face_registry.json` integrator-owned; §4-I makes `backend/app/BUCK` integrator-owned and asks L-P0-BUCK for a glob-driven app-test target that removes the bottleneck permanently. |
| **[major]** `console-enterprise-roadmap.md` absent from the collision map | **CHANGED.** §4-L. Whole map reconciled against the registry's machine-readable `shared_collision_roots` (which also surfaced `console-capability-registry.json` → §4-M). |
| **[major]** L-B14's backend root does not exist | **CHANGED.** Renamed **L-B14F**, FE-only; the identity quartet is held on `codex/directory-backend-private-leaf-20260724`; the backend half is a named gap-manifest block. |
| **[major]** L-B23's roots are unknowable by construction | **CHANGED.** Split: **L-B23** = Phase-0 doc-only decision; **L-B23F** (`web/src/console/dispatch/**`) and **L-B26F** (`web/src/console/evidence/**`) execute against roots verified to exist. No lane dispatches with undeclared roots. |
| **[minor]** Broken `(§3, rule N)` pointer | **CHANGED** → §4 rules D and W. |
| **[minor]** §3-C's fleet-churn rationale is wrong | **CHANGED.** Dropped; `seed.rs` has zero 48h commits. The digest/version/allowlist argument stands alone. |
| **[minor]** §0 anchored to a stale, dirty tree | **CHANGED.** §0 fully re-verified at `8b26c16b` with the command per row, plus universal DoD step 0. The dirty files are the in-flight openapi agent's, not recruiting's (that work is now committed). |
| **[minor]** "no lens-D collision" reads as an all-clear | **CHANGED.** §4-V names logistics' 55 hits/48h and directory's leaf branch explicitly. |

### Truthfulness + enterprise-bar judge

| Finding | Response |
|---|---|
| **[blocker]** No per-lane DoD anywhere; ~15 lanes have no source spec | **CHANGED.** §5 canonical table + `docs/program/wave4/lanes.json` committed by L-P0-EPOCH, carrying roots, scope, anchors, `param_verify_live`, slot, evidence path, DoD delta per lane. Every §7.4 fabrication temptation resolves to a lane id + DoD line. |
| **[blocker]** The wage chain has no data-ingestion owner | **CHANGED.** **L-D0b chartered** (Phase 0): NTS 간이세액표 Excel → rows keyed by `tax_table_version`; data.go.kr 15068737 → 업종별 + 출퇴근재해 요율. Every row `source_url` + `retrieved_on` + source-file checksum NOT NULL, with an ingest test that fails on checksum drift. **Hard dependency of L-D4.** Ingest, never transcribe — the gate stays a gate. |
| **[blocker]** FE completion evidence is jsdom — the failure mode the wave exists to fix | **CHANGED.** §6.3-22: real-browser proof mandatory for L-F1/L-F2/L-F4/L-B21F/L-X1 (live React tree assertion + keyboard journey + axe on the mounted window layer); the rest inherit a shared journey fixture parameterized by screen key. jsdom-only is not completion evidence for any window claim. |
| **[blocker]** 공휴일/대체공휴일 calendar unowned, no §6 row, no live source | **CHANGED.** Added to **L-D0** as effective-dated per-year rows sourced live (관공서의 공휴일에 관한 규정 + the year's 대체공휴일 고시) via L-P0-PARAM, `source_url`+`retrieved_on` NOT NULL. §7.1 row added; L-D2/D6/D8/D11 declared consumers; **grep gate against any local holiday list or weekday-only 휴일 heuristic.** Verified: the brief contains the string 공휴일 exactly once and carries no calendar. |
| **[blocker]** 2 of 18 gate-only computable rules named | **CHANGED.** §7.2 enumerates **all 18** with anchor + owner, generated from the register. Three promoted to named DoD lines (golden-case execution, benefits four-eyes, `cedar_policy_ref` resolution). Three of the previously-unnamed are active defects, including a **live SoD self-approval bypass**. |
| **[major]** §5.2-9 does not require first-outcome replay | **CHANGED.** §6.2-13 rewritten verbatim to the judge's text, with both branches tested as `console_rt`. |
| **[major]** No independent review pass — 57 self-certified lanes | **CHANGED.** §6.1-8: review-gate pass by a different agent (correctness + RLS-as-`console_rt` + codex cross-model + browser/a11y for `web/`), GO verdict in the evidence doc, **integrator queues nothing without one.** |
| **[major]** `backend/app/BUCK` unlisted, blocks the story-test DoD | **CHANGED.** §4-I integrator-owned + `manifests/buck-app-test.json`; L-P0-BUCK asked for a glob-driven app-test target to remove it permanently. |
| **[major]** "CI gates as applicable" makes security gates optional | **CHANGED.** §6.2-16 splits them: `rls-arming`, `tenant-isolation`, `audit-coverage`, `dev-auth-absence` **unconditional**; opting out requires a written reason accepted at review. `dev-auth-absence` was missing and is verified present on disk. |
| **[major]** §6 param index incomplete; "false on every other lane" is a false negative | **CHANGED.** §7.1 regenerated mechanically with the derivation command recorded. L-D5, L-D7, L-D9, L-D10, L-D11 added; L-D6 gains V7; L-D14 gains the full four-insurance span; L-D8 gains the holiday/business-hours rule. |
| **[major]** Silent drops: 대근 (SR-206), audited views/masking, §4-25 loop, design-intent register never cited | **CHANGED.** 대근 → **L-D2** (classification) + **L-D4** (pay derivation) + **L-D11** (contract→pay chain), `param_verify_live`, §7.2 row 6. Audited sensitive-view/masking → the **owning backend lane's** DoD (server-side view event on the read path; **L-D4** for onPaySheet), because no FE lane may implement it. §4-25 loop → §6.3-18. The design-intent register is now cited throughout, with §8c for the `[>190]` set and §3a/§9-0 for C-64. |
| **[major]** The evidence base has no durable home | **CHANGED.** L-P0-EPOCH commits the brief and both registers to `docs/evidence/console/wave4/`; §6.1-3 anchors are repo-relative JSON pointers; live-fetched parameters append to the **committed** brief, producing a reviewable diff. |
| **[major]** kernel/core DAG self-contradiction + t0 count vs cap | **CHANGED.** Ordered slots deleted (manifest); cap 8; t0 recounted. |
| **[major]** §5.4 keyed on lens, not on what the lane does | **CHANGED.** §6.4 re-keyed on properties: items 26 (any `param_verify_live` lane — pulls in the previously-exempt lanes) and 27 (any lane writing domain state or money/time arithmetic — pulls in L-B9, L-B14F, L-X1). |
| **[major]** L-D12 declared migration-free while its defect is a dropped column | **CHANGED.** Direction decided in §5: read the custody branch, then **re-add the column** (slot **0207** reserved) rather than rewrite the adapter — smaller diff, immediate green, no change to L-D13/L-B8F contracts. |
| **[minor]** "AA a11y" is a claim, not a check | **CHANGED.** §6.3-23 names the command and adds 1.4.10 / 2.4.7 / 1.4.11 / 2.5.8, paired with the §6.3-22 browser journey rather than a second harness. |
| **[minor]** Error envelope stated inconsistently, no test obligation | **CHANGED.** 422 in both places; §6.2-12 converts the parenthetical into a required negative test. |

### Coverage-against-evidence judge

| Finding | Response |
|---|---|
| **[blocker]** DAG inversion: Phase-2 FE lanes consume Phase-3 backend | **CHANGED, by the cheaper alternative the judge offered.** The rule is now **FE lane phase = max(phase of its backend deps)**: L-B19F←L-D17, L-B17F←L-D15, L-B10F←L-D16, L-B8F←L-D13 all move to Phase 3. **REJECTED: splitting four thin contract slices into their own lanes** — the FE fan-out is capped at 6 by review throughput, so early dispatch buys no capacity, and four extra lanes buy four extra collision surfaces. Reordering is free; slicing is not. |
| **[blocker]** dispatch and docs FE have no port lane | **CHANGED.** **L-B23F** and **L-B26F** chartered with roots verified on disk. The dead `dispatch` nav entry (`nav.ts:236`) is removed by the integrator on L-B23's verdict (§4-K). |
| **[blocker]** The DoD codifies the single-pin data-loss defect | **CHANGED.** §6.3-20(b) restated: activation pins the target **and the previously pinned entry remains recoverable**. L-F1's named deliverable is the tested tray-restore contract with a Korean accessible name. **REJECTED: building multi-pin/split in the production provider** — that is L-B0b's entire scope (13× per-screen config, host, persistence, the §5.2 contract axe cannot verify). The tray-restore closes the data-loss reading at ~20 lines; the blockers stay gap-manifested and §9-4 stays open. |
| **[major]** L-C1 reinvents the landed docs cursor | **CHANGED.** L-C1's first DoD line is *generalize the landed cursor — no new encoding, no new error strings* — proven by the docs REST token round-tripping through the shared type and the existing docs cursor tests passing unchanged. |
| **[major]** `0201_evidence_retention` decided, slotted, unowned | **CHANGED.** **L-B22a** chartered (slot 0213 → the reserved 0201 subject), additive, consuming the existing `EvidenceObjectCursor`/`EvidenceObjectPage`, with an explicit "do not reintroduce offset paging" DoD line. L-B22 keeps the non-EV records registry only. Both share the §4-T docs owner. |
| **[major]** Capability-registry preconditions under-scoped | **CHANGED.** §9-3 folds all of it into L-P0-EPOCH, including the `web/src/console/docs/**` → `web/src/console/evidence/**` correction (verified absent/present), the seven seeded CAP rows, `CAP-EVALUATION-001` reconciliation, and the stale `source_revision`. |
| **[major]** Three concrete §6 citation errors | **CHANGED.** VAT (three lanes) → L-P0-PARAM, since the brief has **zero** VAT content; 주52 → §3.1 not §1; L-D19 was `false` with two param-bearing rules — the lane is cut and both rules are anchored in §8b. |
| **[major]** 문서 보존기한 assigned to two independent live fetches | **CHANGED.** Retention periods become **L-D0 registry rows**; L-B22 (and any consumer) resolve through the registry and are **forbidden from fetching**. §7.1 swept for other duplicates; the holiday calendar got the same treatment pre-emptively. |
| **[major]** Generic five-category matrix drops 131 enumerated gaps | **CHANGED.** §6.4-27: one test per row of **your domain's own `edge_case_gaps`**, cited by anchor, **plus** the five universal categories. The payroll `BLOCKED_LEGAL_GATE` example is quoted in the DoD text. |
| **[major]** 110 cross-domain invariants funnelled into L-D11 | **CHANGED.** `L-D10 ← L-D4, L-D5` added with the **payroll → 급여 전표 posting** edge named; §6.4-28 makes every lens-D lane close or name-defer the invariants it owns a side of; L-D11 narrowed to the attendance-exception→payroll and 대근→contract→pay chains. |
| **[major]** C-64 never cited, rebutted or reconciled | **CHANGED.** §3a states it as a conflict, takes the depth-per-module reading, and §9-0 requires a written waiver. §6.3-18 makes the reading honest by requiring the §4-25 loop. |
| **[major]** Every `[>190]` intent unrepresented AND unregistered | **CHANGED.** §8c registers all of them; **L-X1 CAP-SALES-CRM chartered** for CRM-1/3/6 against the only exposed screen; exposure promoted to §9-2. |
| **[major]** 69 backend-blocked findings never individually accounted | **CHANGED in substance, REJECTED in form.** The obligation is accepted: `backend-blocked-index.json` is generated by L-P0-EPOCH and is the integrator's completion checklist, and §8d names the seven ownerless classes in prose. **REJECTED: hand-writing a 69-row table into the charter** — a copied table goes stale on the first merge; generated from the committed registers it stays true. |
| **[major]** L-F4 omits the personal-view config strip | **CHANGED.** Named as an explicit L-F4 deliverable (custom columns from a config-declared attribute map, count stats that drill via the search filter, saved presets per module key, dist widget, density segment) with C-41's N+1 「+ 직접 입력」 affordance in its DoD; consumer list corrected to **all** module lanes (inventory[?] is a blocker, plus directory[2], equipment[6], dispatch[1], recruiting). |
| **[minor]** `saveLayout()`/`restoreDefault()` are dead APIs shipping ko strings | **CHANGED.** L-F1 DoD: wire both controls into the console window layer **or** delete the API and its `ko.ts:1808-1809` keys. Either way the port recipe documents no control that does not exist. |
| **[minor]** Lane arithmetic does not reconcile | **CHANGED.** One canonical table (§5), 60 lanes, phases reference it and never restate it. |
| **[minor]** Row T missing | **CHANGED** → §4-L. |
| **[minor]** `hold_rule` vs the gap-manifest protocol | **CHANGED.** §6.1-3 states the amendment; §9-3 carries it into the same program action. |
| **[minor]** Attendance has no fallback | **CHANGED.** §8b states it: if ownership is not released by dispatch time, attendance is **formally out of wave 4** and both registers are gap-manifested as a block with anchors. |
| **[minor]** Inventory's blocker owned by Phase-3 L-C9 while L-B16F is Phase 2 | **CHANGED.** L-B16F fixes its own container locally (its blocker, its lane); **L-C9 generalizes afterwards** and moved to Phase 2 so it is not behind its own consumer. |

### Sequencing-and-value judge

| Finding | Response |
|---|---|
| **[blocker]** Zero user-visible value; the one lit module gets zero lanes | **CHANGED, partially as proposed.** **L-X1 CAP-SALES-CRM chartered as the headline lane** (CRM-1 DL- deal type into the ontology, CRM-3 stage transition as an audited action with a per-stage evidence enum, CRM-6 won→contract through the guarded composer), backend `crates/sales/**` (cold, tier-3, full quartet on disk) + `web/src/console/sales/**`, scoped to an ADR-0025 evidence-approved state. Funded by cutting L-D19/D20/D21/D22 and L-C6. **REJECTED: also cutting L-C7 and L-C9 for capacity** — L-C9 closes inventory's `findings[0]` blocker and C-42 makes body-level horizontal scroll a defect class, so it is register work, not polish. L-C7 is cut anyway, for a different and better reason: **no consumer this wave** (§8b). CRM-2/4/5 and B-26/WFL-9 are registered as wave-5 (§8c) rather than crammed in. |
| **[blocker]** C-64 head-on conflict, unmentioned | **CHANGED.** §3a + §9-0 as a written waiver requirement. |
| **[blocker]** Push protocol and leaf migrations/BUCK violate the epoch contract | **CHANGED, option (b) taken.** Migrations, BUCK, clients and openapi are integrator manifests (§4-G/H/I/A/B) — contract line 126-127 satisfied. The merge-vs-rebase clause is unsatisfiable on this spine and is escalated as §9-1 rather than silently violated. |
| **[blocker]** `data-obj-code` MUST on modules with no code | **CHANGED.** **L-P0-ID-a** is a standalone Phase-0 triage publishing the per-module table {has issued code · needs L-B4 issuance · intentionally code-less}; **every FE DAG edge is rewritten from it**; §6.3-21 gives bucket-3 modules a DoD variant (assert (b) and (c) only, name the missing code in the gap manifest). Verified: `equipment_3r_rental_cases` has no code column; `0113:49` deliberately keeps `person`/`org_unit` prefix NULL. |
| **[major]** L-P0-ID bundles three decisions at three clock speeds | **CHANGED, two-way not three.** **L-P0-ID-a** = triage **+ code format** (they are one decision in practice — the triage's bucket-2 modules need the format to size L-B4, and `codeGrammar.ts:55` already accepts both shapes so no FE lane is blocked either way). **L-P0-ID-c** = registry authority, blocking only L-A9, free to run the whole wave. The FE fan-out depends on **L-P0-ID-a only**. |
| **[major]** L-C8's blocking rationale factually wrong | **CHANGED** (see the feasibility judge's matching finding). |
| **[major]** L-C1 builds a second paging primitive | **CHANGED** (extraction; §4-T; docs tests unchanged). |
| **[major]** L-F1 has no declared owner for `ConsoleShell.tsx`; nested provider unnoticed | **CHANGED.** §4-N makes `ConsoleShell.tsx` + `ConsoleApp.tsx` L-F1-owned for the wave with ownership confirmation required against the three live shell branches. L-F1 DoD: the nested `OntologyWorkspaceBody` provider (`:299`, verified) is removed or explicitly reconciled, with one test proving a single tray and a single layout partition across the ontology workspace and one module screen. |
| **[major]** FE fan-out over-serialized on six foundations | **CHANGED.** Gate split by **port path**, assigned per module by L-P0-ID-a: `port_path: adopt` (LeaveConsole descriptor-mapper, ≈120-200 LOC) depends on **L-F1 + L-F2 only**; `port_path: template` depends on all six. |
| **[major]** L-B21F is called "first" but is not a gate | **CHANGED.** It is an explicit **GATE**: no second FE lane dispatches until L-B21F merges **and** the §6.3 port recipe is amended with what it learned (or explicitly confirmed unchanged). Its value is recipe validation, not finding closure — stated so reviewers do not score it on findings. |
| **[major]** L-A2 is a road to nowhere through the worst traffic | **CHANGED.** Cut to wave 5, bundled with its frontend caller (§8b). |
| **[major]** L-C4 and L-C3 build affordances with no server source | **CHANGED, split verdict.** **L-C4 cut** (§8b) — no console consumer, two event variants, no presence transport; that is a platform lane, not hardening. **L-C3 kept but bound to a named first consumer**: the ontology instance-command write path, the one place that actually emits 412 today (`ontology/rest/src/lib.rs` 412 mapping + `web/src/api/ontology.ts:151,230,243` as the sole existing consumer), with the Ontology Manager revision-staging surface as the pilot. No conflict UI ships for a signal the server never sends. |
| **[major]** Five lanes each carrying an undone live statutory fetch | **CHANGED.** **L-P0-PARAM** chartered as a Phase-0 research lane fetching all of them (plus VAT and the holiday calendar) once, appended to the committed brief with `source_url` + `retrieved_on`. **A lane whose fetch failed is not chartered; the gap is registered.** |
| **[major]** The §4-25 loop is mandated by doctrine and absent from the DoD | **CHANGED** → §6.3-18. |
| **[major]** 41 hand-renumbered leaf migrations under merge-only push | **CHANGED.** Ledger + manifests + self-contained-file rule + in-flight cap at the integrator's drain rate (§2 bottleneck 3, §4-G). |
| **[minor]** L-A6's headline overstates tokenization | **CHANGED.** Reworded: *drag/drop grammar with zero frontend edits; tokenization and bare-code linkability arrive with L-F3.* L-F3's scope notes that `objectCodeBodySource()` is already exported at `codeGrammar.ts:92` with zero consumers — pointing `BARE_CODE_RE` at it is the whole fix. |
| **[minor]** L-F1 hardens a localStorage store that ban #9 forbids | **CHANGED.** Deferral kept, silence removed: L-F1 DoD requires the localStorage ceiling as a **named gap-manifest entry with its six §4.7-2 anchors** plus a `ponytail:` comment naming the server-persisted upgrade path. |
| **[minor]** Epoch normalization is unowned work, not a decision | **CHANGED.** **L-P0-EPOCH** is the true t0 lane, program/integrator-owned, including `CAP-SALES-CRM` and the on-disk root verification. |
| **[minor]** §0 is a snapshot presented as state | **CHANGED.** Commands published per row + universal DoD step 0. |
