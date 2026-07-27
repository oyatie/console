# Scout: spine reality check (wave-4 pre-planning)

**Scope of inspection:** `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`
**Branch at read time:** `wave23-consolidation-20260724`
**HEAD:** `a5bccdc1ccb6484cd2bf672bd0a5107c08033167` — `docs(registry): truthful wave-2/3 consolidation states` (2026-07-25T00:03:35-04:00)
**Registry `source_revision`:** `origin/main@8e42b9a2ea42c4d79ed498044a9f50f623299f7f` (2026-07-23T01:02:29-04:00, `chore(main): release 0.2.1 (#487)`)
**Delta:** `git rev-list --count 8e42b9a2..HEAD` = **869 commits**
**Working tree:** clean except ` M backend/Cargo.lock` (integrator-held)

Prior art consulted, not re-derived:
- `docs/program/console-enterprise-roadmap.md` (implementation authority; §"Non-negotiable module completion contract", §"Executable user-story gate", §"Frontier 4: production exposure", §"Truthful implementation snapshot")
- `docs/program/console-capability-registry.json` (24 rows, `last_refreshed: 2026-07-25`)
- `docs/program/console-fanout-epoch-contract.md` (admission algebra for ownership roots)
- `docs/program/console-buck2-scale-playbook.md` (Buck2-only Rust completion evidence)
- `docs/evidence/console/wave23-consolidation-inventory.md` (per-lane merge/defer decisions, migration renumbering)
- `/Users/jasonlee/Developer/maintenance/.omc/research/` — contains only pre-console research (`backend-survey.md`, `benchmark-brief.md`, `be-ontology-engine-arch.md`, all dated ≤ 2026-07-10). **No wave-1..3 console research lives there**; the program docs above are the live authority. Several `.omc/research/` files were copied verbatim into `docs/program/` on 2026-07-23 (identical byte sizes for `backend-survey`, `be-ontology-engine-arch`, `benchmark-brief`).

---

## 1. Hot zones — do NOT edit directly in a wave-4 lane

Measured: `git log --since=48.hours --name-only --pretty=format: 8e42b9a2..HEAD`, path-hit counts, `BUCK`-only paths excluded.

### 1a. Shared collision roots (contractually owned, not merely hot)

`console-capability-registry.json → shared_collision_roots.owner = "console-consolidation"`:

| Path | 48h hits | Note |
|---|---|---|
| `web/src/i18n/ko.ts` | 28 | i18n keys must be emitted as a *manifest* for the integrator, never edited |
| `web/src/console/shell/nav.ts` | (in `console/shell` 57) | also holds `EXPOSED_SCREEN_KEYS` — see §2 |
| `web/src/console/screens/registry.ts` | (in `console/screens` 79) | mount registration |
| `backend/openapi/openapi.yaml` | **61** | single most contended non-vendored file |
| `backend/crates/platform/db/migrations/**` | (in `platform` 205) | fan-out contract: *"excluded from all leaf writable roots"* |
| `docs/program/console-enterprise-roadmap.md` | 8 | |
| `docs/program/console-capability-registry.json` | 13 | |

`openapi.yaml` also carries a live scar: `ee277e16 feat(openapi): document 6 merged lanes' routes + schemas` was **reverted whole** by `9bb877c6 revert(openapi): restore origin openapi.yaml — mechanical fragment splice corrupted it`. Mechanical fragment splicing of this file is a proven failure mode. Consequence: the 6 merged lanes' routes are currently **absent from openapi.yaml**, so the generated `clients/{ts,kotlin,swift}` faces do not cover them (this is the openapi-drift-gate hazard already in project memory).

### 1b. Hot backend crates (48h feature commits, BUCK-only excluded)

Tier 1 — heavy churn, high collision probability:
- `backend/crates/platform` (**205**) — includes migrations; effectively integrator-owned
- `backend/crates/attendance` (106) — `CAP-ATTENDANCE-CONSOLE` is `writer_assigned_gap_closure_in_progress`; active writer
- `backend/app/tests` (93) and `backend/app/src` (65) — every merged lane mounts its router in `backend/app/src/lib.rs`; guaranteed conflict
- `backend/crates/production` (49), `logistics` (35), `inventory` (35)

Tier 2 — moderate churn, still live lanes:
`workorder` (30), `support` (30), `ontology` (30), `docs` (30), `dispatch` (30), `payroll` (27), `consulting` (25), `compliance` (24), `financial` (23), `identity` (22), `comms` (20), `backend/ci/gates` (20), `facilities` (19), `evaluation` (19).

Tier 3 — quiet crates (safest wave-4 landing zones):
`kernel` (3), `analytics-quant` (3), `action-inbox` (3), `erp` (2), `policy` (6), `governance` (6), `finance-gl` (6), `registry` (7), `todos` (7), `sales` (7), `orgchange` (7).

### 1c. Hot frontend dirs

`web/src/console` totals **800** path-hits. Per-module:
`features/attendance` 145 · `console/screens` 79 · `console/shell` 57 · `compliance` 51 · `dispatch` 50 · `production` 46 · `evidence` 42 · `mail` 37 · `payroll` 31 · `inventory` 31 · `finance` 25 · `workflows` 23 · `org` 22 · `equipment` 22 · `appr` 22 · `recruiting` 21 · `logistics` 18 · `people` 16 · `modules` 16 · `maintenance` 16 · `board` 16 · `notif` 15 · `field` 15 · `evaluation` 15 · `comms-rail` 15 · `inspection` 14 · `sales` 13 · `ontology` 11 · `directory` 10 · `consulting` 10 · `messenger` 7 · `asset` 7.

**The parent's constraint holds repo-wide:** a consolidation integrator currently owns `web/`. Nine wave-2/3 frontend bodies were mounted dark in a single integrator commit `d165dab2 feat(console-wiring): mount 9 wave-2/3 bodies dark (payroll/recruit/orgchart/evaluation/maintenance/field/notif/board/directory)`. A wave-4 lane must emit *mount manifests* (the established pattern: `docs/evidence/console/CAP-*/frontend/manifests/mount.json` + i18n key inventory) and let the integrator apply them.

### 1d. Tooling / CI, also hot
`tools/buck/gen_first_party.py` (20) + its test (16), `.github/workflows/ci.yml` (19), `scripts/check-ci-preflight.mjs` (+test, 30), `scripts/console/plan-fanout.mjs` (+test, 25), `scripts/dev-up*.mjs` (20), `package.json` (13), `backend/Cargo.lock` (26), `third-party/rust/vendor` (656 — vendored, ignore).

### 1e. Migration numbering
High-water = `0202_notification_policies_and_object_agg.sql` (202 files in `backend/crates/platform/db/migrations/`).
**`0201` is a reserved gap** — `wave23-consolidation-inventory.md:85` allocates it: `docs 0195_evidence_retention → 0201_evidence_retention`. A wave-4 lane must take **0203+** and re-check immediately before push (collision is the documented failure mode).

### 1f. Concurrency scale
`git worktree list` = **510** worktrees (103 under `~/Developer/maintenance-worktrees`). ~40 branches have commits within the last 36h. Active codex lane clusters right now: `codex/pr488-ci-*` (5 lanes: devup, app-config, dispatch-dep, support-rls, pr473-harness), `codex/pr488-ios-*` (2), `codex/pr488-office-*` (2), `codex/console-{global-scope-foundation,search-object-fabric,shell-scheduled-nav}`, `codex/attendance-*` (3), `codex/customer-site-registry-foundation`, `codex/pr488-rolling-train-20260725`, `wave23-consolidation-20260724` (this worktree).

Notably, `codex/console-search-object-fabric-20260724` and `codex/console-global-scope-foundation-20260724` are **cross-cutting console-shell lanes** — any wave-4 lane touching global search or scope selection will collide.

---

## 2. The sales EXPOSED precedent — and its warning

**Current value (verified in source):**
```
web/src/console/shell/nav.ts:134
export const EXPOSED_SCREEN_KEYS: readonly MountedScreenKey[] = ["sales"];
```
Enforced at three call sites: `web/src/AppRouter.tsx:443-444` (`ConsoleRolloutBoundary approvedScreenKeys` + `ConsoleApp screenKeys`), `web/src/console/ConsoleApp.tsx:29`, `web/src/console/shell/ConsoleShell.tsx:61`, `web/src/console/rollout/ConsoleRolloutBoundary.tsx:35`. Locked by test: `web/src/console/shell/nav.test.ts:39,58` — `expect(EXPOSED_SCREEN_KEYS).toEqual(["sales"])`, plus a negative assertion at `:158`.

**The evidence chain that got sales in — four commits, all frontend-only:**

| # | Commit | What it added |
|---|---|---|
| 1 | `7f296014 feat(console): add sales inquiry workbench` | `SalesCrmScreen.tsx` (345 L) + **443 L of component tests**, `SalesCrmScreenBody.tsx`, `salesAccess.ts`, `web/src/i18n/salesCrm.ts` (57 L) |
| 2 | `b9bb2df5 feat(console): mount authenticated sales operations` | added to `MOUNTED_SCREEN_KEYS` (`nav.ts` +14), `screens/registry.ts` +2 & its test +5, `nav.test.ts` +20, `i18n/ko.ts` +2 → state = **MOUNTED/DARK** |
| 3 | `45c9eba4 fix(console): expose and fence sales workbench` | **the flip**: `EXPOSED_SCREEN_KEYS: [] → ["sales"]` (9-line nav.ts diff incl. the ADR-0025 comment rewrite), `AppRouter.test.tsx` +51/-? , `SalesCrmScreen.test.tsx` +34, `nav.test.ts` ±10 |
| 4 | `b8a8eb4c fix: share live sales authorization projection` | post-exposure hardening: `ProtectedRoute.tsx`, `console/policy/authz.ts` (+22) & test, `console/shell/authz.ts` (99 changed), `AppRouter.test.tsx` +86 |

**The rationale text sales wrote into `nav.ts` (this is the doctrine to imitate):**
> *"Bodies in `MOUNTED_SCREEN_KEYS` remain development inventory unless named here. Sales is the sole reviewed vertical slice: its authenticated route is still gated by the server-owned rollout decision and its management grant. Every other body remains DARK until separately approved."*

**Three defects in this precedent that wave-4 must not copy:**

1. **No evidence directory.** `docs/evidence/console/` has 14 `CAP-*` dirs — `ATTENDANCE, BOARD, DIRECTORY, DISPATCH, EQUIPMENT-3R-PILOT, EVALUATION-001, EVALUATION-CONSOLE, FIELD, LOGISTICS-PILOT, MAINTENANCE, NOTIF, ORG, PAYROLL, RECRUITING` — and **no `CAP-SALES-*`**.
2. **No registry row.** None of the 24 rows in `console-capability-registry.json` is a sales row. Sales is EXPOSED with zero registry provenance, zero signature story, zero `buck2_targets`, zero backend evidence.
3. **The design mirror still says the manifest is empty.** `docs/design/oyatie-console/ROADMAP.md:13` (ADR-0025 overlay) and `docs/design/oyatie-console/SYNC-MANIFEST.md:43` both assert *"현재 `EXPOSED_SCREEN_KEYS`는 비어 있다"* / *"`EXPOSED_SCREEN_KEYS` empty"*. The mirror is **stale relative to code**. Downstream lane docs propagate the correct-but-inconsistent value: `CAP-EVALUATION-CONSOLE/.../mount.json:22` and `gap-analysis.md:64` say `["sales"]`; `wave23-consolidation-inventory.md:54` says `EXPOSED_SCREEN_KEYS = ["sales"]` — **DO NOT TOUCH**.

**The real bar, per the roadmap (this, not the sales chain, is the template to write against):**
`console-enterprise-roadmap.md` §"Frontier 4: production exposure" requires: no unmounted nav or hidden required workflow · all module stories replayed on the exact candidate · visual + a11y matrix green · **Buck2-only Rust build/test completion evidence** · independent review satisfied or explicitly held · CI + immutable image authorization + deployment + live readback + rollback verified. Plus §"Executable user-story gate": happy path, least-privileged view, denial without leakage, failure/retry/recovery, lifecycle+audit readback, cross-tenant isolation, responsive+keyboard, and non-regressing action count.

Sales satisfies roughly item 1 of that list. **Treat sales as a precedent for the *mechanism* (a 4-commit mount → fence → flip → harden sequence, with the flip isolated to one 9-line nav.ts diff plus tests) and explicitly NOT as a precedent for the *evidence bar*.**

---

## 3. Docs-lane conflict area — evidence-register cursor pagination

**Registry verdict** (`CAP-DOCS-EVIDENCE-CONSOLE`):
```
backend:  deferred_spine_evidence_incompatible
frontend: deferred_spine_evidence_body_kept
e2e: missing | runtime: blocked | independent_review: missing | production_exposure: dark
tests.files: []  leaf_commands: []  buck2_targets: []
worktree: null   branch: null   commits: []
ownership: frontend_roots ["web/src/console/docs/**"], backend_roots ["backend/crates/docs/**"]
```
Note the frontend root `web/src/console/docs/**` **does not exist** on disk; the real dir is `web/src/console/evidence/` (12 files: `evidenceApi.ts`, `EvidenceRecords.tsx`, `EvidenceCard.tsx`, `EvidencePolicyGate.tsx`, `evidenceModel.ts`, `evidenceFixtures.ts`, `types.ts` + tests). The registry's declared root is wrong — under the fan-out contract's "every lane root must be wholly covered by exactly one capability-owned private root", a wave-4 docs lane would be **held** until this root is corrected.

**What the spine actually landed** (13 commits on `backend/crates/docs` in 48h) — a complete opaque keyset-cursor evidence register:

- `0b81a75e fix(docs): return opaque register scan cursors`
- `a3824338 test(docs): cover opaque register cursor encoding`
- `c7787cca fix(docs): snapshot register scans by immutable sequence` (118-line adapter rewrite + application + rest + RLS test)
- `858c3613 fix(docs): restore typed evidence adapter compilation`
- `1a574d67 fix(docs): retain cursor paging mode after query move`
- `48a89167 test(docs): prove snapshots reject backdated inserts`
- `26f15694 feat(docs): distinguish verified originals from derivatives`, `0b87726d fix(docs): derive evidence verification from storage attestation`, `a623d36c fix(docs): stage evidence register migration safely`

Shape now in tree (`backend/crates/docs/{domain,application,adapter-postgres,rest}`):
- `EvidenceObjectCursor { snapshot_sequence, register_sequence, id }`, keyset predicate `AND (register_sequence, id) < ($rs, $id)` with `ORDER BY register_sequence DESC, id DESC` (`adapter-postgres/src/lib.rs:798-807`, `:142`)
- snapshot stability via `register_sequence <= snapshot_sequence`, seeded from `SELECT COALESCE(MAX(register_sequence),0) FROM docs_evidence_objects` (`:120`)
- hard validations: `"EV cursor paging cannot be combined with offset"` (`:100-102`), `"EV cursor snapshot does not match as_of"` (`:105-108`)
- REST boundary: base64url-JSON opaque token, `decode_register_cursor`/`encode_register_cursor`, errors `"cursor is not valid base64url"` / `"cursor payload is invalid"` (`rest/src/lib.rs:193-205`), round-trip test at `:879-888`
- `EVIDENCE_ROUTE_PATHS` already mounted in the app (`wave23-consolidation-inventory.md:44`)

**Merge decision already recorded** (`wave23-consolidation-inventory.md:75`):
> `**docs** | console_docs_rest mounted (EVIDENCE_ROUTE_PATHS); retention absent | lane 0195_evidence_retention collides w/ spine 0195_docs_gaps | **MERGE** lane's retention additions + renumber.`
and `:85` → `docs 0195_evidence_retention → **0201_evidence_retention**`.

**Rebase-on-spine item, concretely:** the wave docs lane's *pagination* work is superseded — the spine's cursor contract wins outright and is test-covered. The only surviving lane delta is **evidence retention**, which must land as migration **`0201_evidence_retention`** (the reserved gap) plus additive adapter/application/REST code that consumes the existing `EvidenceObjectCursor`/`EvidenceObjectPage` types rather than reintroducing offset paging. Do not re-derive the cursor.

---

## 4. Buck2 state — **the graph is broken at HEAD**

Policy (`console-buck2-scale-playbook.md`): *"Rust completion evidence is Buck2-only. Cargo remains manifest/dependency metadata and is never a substitute for product verification."* And the registry's `hold_rule` names *"empty Buck2 target sets"* as a hold condition.

**Coverage:** 156 crate `Cargo.toml` under `backend/crates`, **147 `BUCK` files** → **9 missing**:

```
backend/crates/evaluation/adapter-postgres     (evaluation/{domain,application} DO have BUCK — 128a5ce9)
backend/crates/evaluation/rest
backend/crates/orgchange/adapter-postgres
backend/crates/orgchange/domain
backend/crates/orgchange/rest
backend/crates/recruiting/adapter-postgres     (recruiting has ZERO BUCK files)
backend/crates/recruiting/application
backend/crates/recruiting/domain
backend/crates/recruiting/rest
```

All nine are declared Cargo workspace members (`backend/Cargo.toml:17,27,43` globs + path deps at `:203-206, :235-238, :323-325`).

**Worse — `backend/app` is Cargo/Buck divergent, i.e. the Buck app target cannot compile:**

- `backend/app/Cargo.toml` declares `console-recruiting-{adapter-postgres,application,domain,rest}` (`:90-93`), `console-orgchange-{adapter-postgres,rest}` (`:117-118`), `console-evaluation-{adapter-postgres,rest}` (`:167-168`)
- `backend/app/src/lib.rs` **uses** them: `use console_orgchange_adapter_postgres::PgOrgChangeStore;` (`:79`), `use console_orgchange_rest::OrgChangeRestState;` (`:80`), `use console_recruiting_adapter_postgres::PgRecruitingStore;` (`:118`), `use console_recruiting_rest::RecruitingRestState;` (`:119`), `.merge(console_recruiting_rest::router(...))` (`:3000`), `.merge(console_orgchange_rest::router(...))` (`:3207`); plus `backend/app/src/recruiting_hire.rs:18,21`
- `backend/app/BUCK` has **6056 `console-` dep lines and ZERO matches** for `recruiting|orgchange|evaluation`

**Proven stale, not merely unregenerated:** `backend/app/BUCK` was last touched by `bfa0a635 test(dispatch): harden replay and console contracts`; `git merge-base --is-ancestor bfa0a635 8a99f4c9` → true, i.e. the BUCK file predates all three lane merges (`8a99f4c9 merge(recruiting)`, `a4ae5ab5 merge(org)`, `d5a2ba73 merge(evaluation)`) that added the Cargo deps. Those merges touched `backend/app/Cargo.toml` and never regenerated BUCK.

**Fix path is a generated face, not a hand-edit.** `backend/app/BUCK` line 1: `# @generated by tools/buck/gen_first_party.py from Cargo.toml — do not edit by hand.` The remedy is re-running `tools/buck/gen_first_party.py` and committing all 10 changed BUCK files. `tools/buck/generated_face_registry.json` registers the face as `first-party-buck` (alongside `reindeer-third-party-rust`, `openapi-{typescript,kotlin,swift}`), and the playbook makes the registry the writable-face authority with a `writer-snapshot` drift gate. Per the fan-out contract, generated faces are serialized shared roots — **not a leaf-lane write**.

**Registry corroborates the gap:** across all 24 capability rows, only `CAP-EQUIPMENT-3R-PILOT` has a non-empty `tests.buck2_targets` (3 targets). Every other row, including all 8 `integrated_dark_on_pr488` backends, has `buck2_targets: []`. `CAP-BENEFITS-CATALOG.state.backend` literally reads `implemented_but_buck2_unregistered`, and `CAP-COMPLIANCE-CATALOG` reads `..._pending_central_buck`.

---

## 5. Registry / roadmap changes since `8e42b9a2`

**`source_revision` is UNCHANGED** — still `origin/main@8e42b9a2ea42c4d79ed498044a9f50f623299f7f`, even though `last_refreshed` is `2026-07-25` and HEAD is 869 commits ahead. The fan-out contract requires `source_revision` be *"a resolvable `<ref>@<40-sha>` immutable provenance commit that is an ancestor of the anchor"* — 8e42b9a2 is still an ancestor, so receipts still bind, but every registry claim is described against a 2-day-stale tree.

**`fanout_epoch`** — `current_epoch: 1`, `normalized_lane_ids: []`, status:
> *"No currently declared lane has a fresh, clean, exact worktree, root, resource, and leaf-gate normalization; legacy lanes remain explicitly held."*

**Every lane is formally HELD right now.** Wave-4 lanes cannot be dispatched under the contract until they are normalized into an epoch.

**Registry commits in the window (13):** `8c694b89` (seed) → `2de4d24e` / `63f8df97` / `0502b2e6` / `56bcdc61` (wave-2/3 row seeding + claims) → `2b8f54b5` (wave-1 truth) → `4050604a`/`dba63fb1`/`2ecc397c`/`3a69dc88`/`70c0c06d` (fan-out epoch machinery) → `7e150d9d` → **`a5bccdc1` (HEAD): 2538 insertions / 2467 deletions — near-total rewrite**, message: *"8 backends integrated_dark_on_pr488 (each test-verified, shas recorded); 9 frontends integrated_dark_on_pr488 (mounted dark, tsc green); docs + dispatch-FE deferred with rationale; inventory/attendance spine-kept."*

**Roadmap commits in the window (8):** `8c694b89` (establish) · `0fbc3fb8` / `3373b398` / `23d808ee` (rolling design sync/acceptance/baseline) · `d9dbfc48 docs(build): plan current-train Buck2 reconstruction` · `ca7dc048 docs(console): define operational object runtime` · `b66136f8 build(buck2): add fail-closed scale preflight` · `5bc28a4b docs(console): refresh implementation source freeze`.

**Roadmap's declared implementation source freeze:** `78cb5197927a031ead30c6dc0426c23455d3cb16` (`fix(buck): classify reporting application tests`). Note this is **behind** HEAD by ~50 commits, including all nine `merge(*-fe)` commits and `d165dab2`. The roadmap's "Truthful implementation snapshot" therefore does **not** describe the 9 mounted-dark wave-2/3 bodies.

**Two new program docs since the epoch:** `console-buck2-scale-playbook.md` and `console-fanout-epoch-contract.md` (both `2026-07-24 20:00`). Both are enforceable policy a wave-4 charter must satisfy.

**All 24 capability rows are `production_exposure: dark`.** The registry knows of no exposed capability — which is exactly the sales inconsistency from §2 restated from the other side.

**Registry state census (24 rows):**
- `integrated_dark_on_pr488` (be+fe): PAYROLL, RECRUITING, ORG, EVALUATION, MAINTENANCE, FIELD, NOTIF, BOARD (8); DIRECTORY has fe integrated / be = `existing_rest_reused_identity_crate_hot_manifest_only`
- writer-active: ATTENDANCE, INVENTORY (`writer_assigned_gap_closure_in_progress`)
- deferred: DOCS-EVIDENCE (`deferred_spine_evidence_incompatible`), DISPATCH (`spine_landed_spec_consumed` / `spine_body_kept_lane_deferred`)
- Frontier-2 pilots mostly `gap_analysis_required` + `runtime: blocked`: PRODUCTION, IFM, CONSULTING, OWNED-PLANTS; LOGISTICS `integrated_dark_on_pr488` fe; EQUIPMENT-3R `request_changes_2026_07_23`
- `runtime: blocked` on 19 of 24 rows; `e2e: missing` on 14; `independent_review: missing` on 15
