# Wave-4 charter — Lens A (substrate: ontology projections + §18 action dispatch)

Spine: `/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`, branch
`wave23-consolidation-20260724` (HEAD `c290a1de` at charter time). Everything stacks onto PR #488.

Authoritative evidence for this lens:
- `scratchpad/wave4/scout-ontology-engine.md` (adopted; its L-A1…L-A5 are refined here, not replaced)
- `scratchpad/wave4/scout-spine-delta.md` §1 (hot zones), §4 (Buck2 broken at HEAD), §1e (migration numbers)
- `scratchpad/wave4/scout-shared-grammar.md` §5 + §3-step-7 (code-prefix priming is the cross-lens unblock)
- `scratchpad/intent/design-intent-register.md` C-1…C-7, C-26, C-27, C-32, ONT-1…ONT-4, B-2
- fidelity registers (`tasks/wlhg23xnz.output`) — the backend-blocked object-card / dynamics-layer /
  token-grammar findings on equipment, inventory, dispatch, field, directory, maintenance, evaluation, board
- `docs/decisions/notes/DN-0003-adr-0025-operational-object-runtime.md` (ORU contract, Slice 1/2)

All facts below were re-verified in the worktree; line numbers are from HEAD `c290a1de`.

---

## Verified deltas vs the scout brief

| # | Verified at | Consequence for chartering |
|---|---|---|
| V1 | `backend/app/src/objects.rs:2571-2630` — both sync tests only constrain **resolvable** kinds ("Non-resolvable registry kinds are intentionally absent because they count 0") | Inserting a row into legacy `object_types` **does not** require touching `RESOLVABLE_KIND_AUTH`. The code-prefix registration lane (L-A6) is therefore fully disjoint from the security-reviewed surface. This is the cheapest cross-lens unblock in the whole lens. |
| V2 | `backend/crates/ontology/rest/src/lib.rs:1562-1588` — `instance_acting` / `object_type_acting` are live routes over `acting_on_instance` / `acting_on_type` | The **dynamics layer read path already exists for any registered type**. The fidelity register's backend-blocked "no acting automations/policies chips" findings (equipment blocker, inventory/field/directory/maintenance major) are unblocked by *registration alone* — no new endpoint. |
| V3 | `backend/crates/ontology/adapter-postgres/src/seed.rs:302-308` — `work_order_draft` registers `request_no`, **not** `code`; only 4 instance-backed drafts register a `code` prop (`:421,459,505,535`) | R8 (`resolve_by_code`) is **not** a query widening. Projected types have no declared code column, so the fix needs a per-type code-property designation (parallel to the existing `title_property_key`) → schema change → catalog version bump → hard dependency on L-A1. |
| V4 | `backend/crates/ontology/*/BUCK` all exist; targets are **one per integration-test file** (`mnt-ontology-adapter-postgres-itest-<name>`) | Any lane adding a `tests/*.rs` file must regenerate `tools/buck/gen_first_party.py` output. The generator rewrites all 147 BUCK files — lanes commit **only their own crate's BUCK hunk** and leave `backend/app/BUCK` (proven stale, spine-delta §4) to the spine. |
| V5 | Migrations on disk end at `0202`; `0201` is the reserved docs-retention gap | Provisional slots start at **0203**; integrator renumbers at merge. |
| V6 | `install_builtin_catalog` guards at `0165:1128-1143` re-read and confirmed; `ont_builtin_catalog_allowlist` has **no** version-chain column | The upgrade path must add a migration-owned predecessor column, not accept a caller-supplied "from version" (that would be a privilege escalation). |

---

## THE SEQUENCING HAZARD (read before dispatching anything)

`BUILTIN_CATALOG_VERSION` (`seed.rs:68`) + its sha256 digest + its `ont_builtin_catalog_allowlist`
row form **one serialized chain**, exactly like `openapi.yaml`. Three lanes need to bump it
(L-A3, L-A7, L-A9). They **cannot run in parallel** — two concurrent bumps produce two manifests,
two digests, and an allowlist that admits neither.

**Resolution: one catalog train, one owner, strict order `L-A3 → L-A7 → L-A9`**, each taking the
next version string (`2026-07-25.1`, `.2`, `.3`) and each riding L-A1's upgrade path. No other lane
may edit `seed.rs`'s manifest, `BUILTIN_CATALOG_VERSION`, or `ont_builtin_catalog_allowlist`.

Second hazard: `seed.rs` is also touched by L-A1 and L-A8. Run **L-A8 → L-A1 → (train)**, or fold
L-A8's two code items into L-A1 if the same agent runs both.

---

## Lane set (ranked by value)

### Foundation / decision lanes — MUST complete before fan-out

**L-A1 · Built-in catalog additive upgrade path** — FOUNDATION, blocking, size L, migration 0203
**L-A5a · Three-registry unification DECISION** — DECISION, doc-only, size M
**L-A4a · Projected command receipt DESIGN GATE** — DECISION, doc-only, size S
**L-A8 · Cheap correctness + stale-doc fixes** — size S, run first (touches `seed.rs`)

### Fan-out lanes

**L-A6 · Legacy object-type rows + code prefixes** — size M, migration 0204, no dependencies
**L-A2 · Schema publish route** — size M, no migration
**L-A3 · First live projected action (equipment)** — size M, migration 0205, catalog train #1
**L-A7 · Wave-2/3 module projections** — size L, migration 0206, catalog train #2
**L-A9 · Projected code resolution (R8)** — size M, migration 0207, catalog train #3
**L-A4b · Projected command receipts** — size L, migration 0208, depends on L-A4a

### Deliberately NOT chartered

- **The registry-bridge implementation (scout's "L-A5b").** Its shape is undecided; a lane whose
  scope is "implement whatever DN-XXXX picks" is not independently buildable. L-A5a produces it.
- **Bulk §18 handlers.** Per the scout: prove the pattern once (L-A3), then charter per-target.
  `app/src/lib.rs` is the hottest backend file on the spine; ~35 lines per handler is cheap later
  and expensive now.
- **`openapi.yaml` edits.** Another agent owns openapi integration right now. L-A2 emits a fragment
  manifest only.

---

## Universal lane contract (applies to every lane below)

**Must not touch, all lanes:** `backend/openapi/openapi.yaml`, `clients/**`,
`web/src/i18n/ko.ts`, `web/src/console/shell/nav.ts`, `web/src/console/screens/registry.ts`,
`backend/app/src/objects.rs`, `backend/crates/platform/db/migrations/0165_*.sql` (additive
migrations only — never edit a landed one), `docs/program/console-capability-registry.json`,
`docs/program/console-enterprise-roadmap.md`.

**Enterprise bar (DoD floor, every lane):** RLS FORCE and every assertion executed as `mnt_rt`
(superuser `BYPASSRLS` masks a broken read path — project memory); deny-by-default authz; audit row
on every mutation; canonical error envelope; idempotent retry; a story-level integration test; the
lane's own crate BUCK hunk regenerated; AA a11y where UI is touched; **no stubs, no TODOs, no
`test.skip`**. Plain-merge before push (rebase is classifier-blocked on this spine). Take the next
free migration integer immediately before push, not at author time.

**Statutory params:** no lane in this lens implements a rule with yearly/regulatory parameters.
`param_verify_live = false` throughout. If a lane discovers one, it stops and escalates rather than
sourcing from model memory.

---

## L-A8 · Cheap adjacent correctness + stale-doc fixes

**Why.** Scout §5.6. Four items, all small, all evidence-backed; two of them are in files L-A1 and
L-A7 will rewrite, so they must land first.

**Scope.**
1. `validate_draft` (`adapter-postgres/src/lib.rs:1080-1180`) rejects a `projected` draft whose
   `backing_table` is absent from `allowlisted_projected_table` (`instances.rs:955-974`). Today the
   type registers cleanly and then 400s on every list with
   `"projected object type has no allowlisted backing table"` (`instances.rs:1016`) — fail-closed but
   a hostile authoring error, and exactly the trap L-A7 will hit 10 times.
2. Fix the stale doc comment at `seed.rs:100-104`: it claims `transition_lifecycle` "reuses
   `create_action`" in Rust; the auto-attach actually lives in SQL at `0165:1039`
   (`('create','저장', …)`), proven by `rest/tests/publish_auto_create_action_as_runtime_role.rs`.
3. Delete the "Status: design, no code" header on `docs/program/be-ontology-engine-arch.md` and its
   byte-identical twin `.omc/research/be-ontology-engine-arch.md` — most of §1/§2/§4 is built.
   Replace with a one-line "built as of <sha>; §5d/§18 residual open" pointer.
4. Correct the header claim at `web/src/console/ontology/typeRegistrySource.ts:1-6` ("a type
   registered via the Ontology Manager wires its codes with NO frontend edit"). It is true only of
   the **legacy** registry it actually fetches (`GET /api/v1/object-types`); the ontology engine's
   `ObjectTypeSummary` (`adapter-postgres/src/lib.rs:222-234`) has no `code_prefix` field at all.
   Comment-only — it must state the fact and defer the remedy to L-A5a, not pre-empt the decision.

**Roots (owned):**
`backend/crates/ontology/adapter-postgres/src/lib.rs` (`validate_draft` + its unit tests only),
`backend/crates/ontology/adapter-postgres/src/seed.rs` (comment lines 100-104 only),
`docs/program/be-ontology-engine-arch.md`, `.omc/research/be-ontology-engine-arch.md`,
`web/src/console/ontology/typeRegistrySource.ts` (header comment only).

**Must not touch:** everything in the universal list, plus `instances.rs`, `web/src/api/ontology.ts`
(`typeRegistrySource.ts:20-24` records it as under concurrent edit by the serial-wire lane — verify
`git log --since=24.hours -- web/src/api/ontology.ts` before claiming any web root).

**DoD.**
- Unit test: a projected draft with `backing_table: "not_a_table"` → `validate_draft` returns a
  validation error naming the field; a draft with `"work_orders"` passes.
- Unit test: a projected draft with a `backing_table` that IS allowlisted but a `backing_column`
  failing `is_safe_ident` still fails (no regression of the existing guard).
- `cargo test -p mnt-ontology-adapter-postgres --lib`
- `buck2 test //backend/crates/ontology/adapter-postgres:mnt-ontology-adapter-postgres-unit`
- `npx tsc --noEmit` green for the web comment edit (no behavior change; no test needed).
- Diff contains zero changes to `seed.rs` outside lines 100-104.

**Size** S · **Risk** low · **Migration** none

---

## L-A1 · Built-in catalog additive upgrade path  ⟵ FOUNDATION, BLOCKING

**Why.** The single biggest lens-A finding (scout §1a). `ontology_api.install_builtin_catalog`
fails closed with no upgrade path: a tenant already on `2026-07-19.1` raises
`ontology_builtin.different_catalog_already_installed` (23505, `0165:1138`), and any tenant with a
pre-existing `ont_object_types` row raises `ontology_builtin.empty_org_required` (23514,
`0165:1141`). **Every already-seeded environment is permanently frozen at 27 types.** Without this
lane, L-A3, L-A7 and L-A9 are undeliverable to any live tenant. Design-intent C-6 ("user-proposed
types are first-class immediately with zero hardcoding") and B-2 are dead until it lands.

**Scope.**

*SQL (new migration, provisional `0203_ontology_catalog_upgrade.sql`):*
- `ALTER TABLE public.ont_builtin_catalog_allowlist ADD COLUMN supersedes_version TEXT REFERENCES ... `
  (self-referencing on `catalog_version`, NULL for the genesis row). The predecessor chain is
  **migration-owned** — never a caller argument. Backfill the existing `2026-07-19.1` row with NULL.
- New function `ontology_api.upgrade_builtin_catalog(p_org_id, p_actor, p_trace_id, p_span_id,
  p_catalog_version, p_manifest)` returning `(upgraded BOOLEAN, added_count BIGINT)`:
  1. `assert_write_context`, manifest shape check, sha256 digest, allowlist lookup — byte-identical
     to `install_builtin_catalog` (`0165:1106-1120`). Digest mismatch → 42501
     `ontology_builtin.manifest_not_allowlisted`.
  2. Same `pg_advisory_xact_lock('ontology-bootstrap:' || org)` (`0165:1125-1127`).
  3. Read `ont_builtin_catalog_installs`. No row → error `ontology_builtin.upgrade_requires_install`
     (23514) — the caller must use the install path. Row at the target version with the target
     digest → no-op `(FALSE, 0)`. Row at any version that is not the target's `supersedes_version` →
     23505 `ontology_builtin.catalog_version_not_predecessor`.
  4. **Additive-only proof.** For every `stable_key` already in `ont_object_types` for the org that
     also appears in the manifest, compare the manifest snapshot against the stored type +
     children; any difference → 23514 `ontology_builtin.non_additive_catalog_upgrade`, tx rolls
     back. Keys present in the DB but absent from the manifest are left untouched (a runtime-authored
     type must survive an upgrade). Only manifest keys with no existing row are installed.
  5. Install the delta keys through the same pass-1/pass-2 logic: `insert_children`, the
     `to_stable_key` link rewrite, and the `ontology_builtin.physical_link_id_forbidden` guard
     (`0165:1153-1157`) all preserved verbatim. Cross-version links (a new type linking to an
     already-installed one) must resolve against existing rows, not only the manifest.
  6. `UPDATE ont_builtin_catalog_installs SET catalog_version, manifest_digest`.
  7. `ontology_api.write_audit(..., 'ontology.object_type.builtin_upgrade', ...)` per added key,
     mirroring the install audit at `0165:1174+`.
- Grants identical to `install_builtin_catalog`; `mnt_rt` stays revoked from the allowlist
  (`0165:132`).

*Rust:*
- `seed.rs`: `upgrade_builtin_catalog(store, actor, occurred_at, requested_keys)` mirroring
  `install_builtin_catalog` (`seed.rs:1256-1280`).
- `seed_governed_config_object_types` (`seed.rs:1307`): attempt install; on
  `different_catalog_already_installed`, route to upgrade; return the requested published heads in
  caller order in both paths. Exact re-run of either path is a no-op.

**Roots (owned):** `backend/crates/ontology/adapter-postgres/src/seed.rs`,
`backend/crates/ontology/adapter-postgres/tests/catalog_upgrade_as_runtime_role.rs` (new),
`backend/crates/ontology/adapter-postgres/BUCK` (own hunk only),
`backend/crates/platform/db/migrations/0203_ontology_catalog_upgrade.sql` (new).

**Must not touch:** universal list, plus `backend/app/src/**`, `web/**`,
`backend/crates/ontology/rest/**`, `instances.rs`, and — critically — **`BUILTIN_CATALOG_VERSION`
must stay at `2026-07-19.1`**. This lane builds the machine; the catalog train drives it.

**DoD.** New test file `catalog_upgrade_as_runtime_role.rs`, all assertions as `mnt_rt`, using a
throwaway second manifest version fixture (do not bump the real constant):
1. fresh org → install path unchanged, 27 types, `ont_builtin_catalog_installs` at `2026-07-19.1`.
2. org at 27 → upgrade to the fixture version adds exactly the delta keys; every pre-existing type's
   `id`, `schema_version`, `key_write_revision`, and children are byte-identical before/after.
3. a fixture manifest that mutates an existing key's `title` → `non_additive_catalog_upgrade`, and
   the org still has exactly 27 types after rollback.
4. exact re-run of the upgrade → `(FALSE, 0)`, no new audit rows.
5. allowlist row absent for the target version → 42501; **the raise happens before any write**.
6. install row at a version that is not the target's `supersedes_version` → 23505.
7. a runtime-authored (REST-created) type present in the org survives the upgrade untouched.
8. `mnt_rt` cannot `SELECT` from `ont_builtin_catalog_allowlist` (assert the 0165:132 revoke holds).
9. a new type whose `fk_link` targets an already-installed type resolves correctly (cross-version link).
10. cross-tenant: org B's install is unaffected by org A's upgrade.

Commands: `cargo test -p mnt-ontology-adapter-postgres --test catalog_upgrade_as_runtime_role` ·
`python3 tools/buck/gen_first_party.py` then commit **only** the
`backend/crates/ontology/adapter-postgres/BUCK` hunk ·
`buck2 test //backend/crates/ontology/adapter-postgres:mnt-ontology-adapter-postgres-itest-catalog_upgrade_as_runtime_role`.
Evidence: test output + the migration's `EXPLAIN`-free plain SQL review + a note in
`docs/evidence/console/CAP-ONTOLOGY-ENGINE/` recording that the digest chain stays migration-owned.

**Size** L · **Risk** HIGH — a caller-supplied predecessor version, or an additive check that
compares only titles, is a tenant-schema-corruption vector. Review the additive-proof comparison as
security-relevant code.

---

## L-A5a · Three-registry unification DECISION  ⟵ DECISION, doc-only, serialized

**Why.** Scout §1c + FE gaps 9/11/12. Three registries exist and none of them is authoritative:

| Registry | Where | Feeds | Keys |
|---|---|---|---|
| ontology `ont_object_types` | `crates/ontology`, 27 seeded keys | explore graph, instances, actions, acting | `approval`, `mail`, `equipment`, … |
| legacy `object_types` | `app/src/objects.rs:39,1578,1625`, seeded by migrations `0113`/`0131`/`0188` | `GET /api/v1/object-types` → `primeCodePrefixes` → **the entire console code grammar** | `approval_run`, `mail_thread`, `voucher`, … |
| static `ONT_TYPES` | `web/src/console/modules/typeRegistry.ts:137+`, 6 types | rich per-type columns/choices in module surfaces | prefixes `VC- FL- HR- AP- TK- CP-`, matching **neither** other registry |

Non-matching pairs: `approval_run`↔`approval`, `mail_thread`↔`mail`. Legacy-only:
`voucher`, `purchase_request`, `listing`, `document`, `asset_transfer`, `notification`.
Ontology-only: `contract`, `position`, `posting`, `console_view`, `sla_setting`, and 10 more.

Fidelity register corroboration (all backend-blocked): board minor — *"No type chip; the built
console has no ontology type registry or explore surface to target"* (design wants `OT-19 공지`);
inventory major — *"renders a static `IV` chip, not `OT-17`"*; dispatch major — *"links resolve via
`GET /api/objects/{kind}/{id}` into a centered peek that renders only code/title/status — a head,
not an object card"*; directory/equipment/maintenance major — no object card because
`console/objectcard/kinds.ts` has no matching kind. This split is what makes DN-0003 Slice 1's
"one governed object card" untrue: `web/src/console/objectcard/useObjectCard.ts` calls the legacy
stack (`/api/v1/object-links:121`, `/api/objects/{kind}/{id}:145`,
`/api/v1/lifecycles/{objectType}/{objectId}:161`, raw `/api/audit:79`) while
`web/src/console/explore/ObjectExplorerModel.ts:6` imports the engine.

**Scope — produce one decision note, no code.** `docs/decisions/notes/DN-XXXX-object-type-registry-unification.md`
must answer, each with a rejected-alternatives paragraph:
1. **Which registry is the source of truth**, and what the other two become (generated view /
   deleted / frozen legacy shim with a sunset date).
2. **The key-mapping table** for every non-matching pair and every one-sided key, with the migration
   or alias mechanism for each. Legacy kinds with no ontology counterpart must each get a verdict:
   promote to a projection, keep legacy-only, or retire.
3. **Where `code_prefix` lives.** Either `ObjectTypeSummary` (`adapter-postgres/src/lib.rs:222-234`)
   grows `code_prefix` + an `OT-` type code, or legacy `object_types` stays the prefix authority and
   the engine syncs into it. Note the design authority demands a **user-visible `OT-nn` code** for
   the module header type chip (board + inventory findings) — whichever registry wins must issue it.
4. **`RESOLVABLE_KIND_AUTH` policy.** `objects.rs:117-120` documents that adding a kind
   *retroactively* makes pre-existing `object_links` of that kind resolvable with **no backfill
   re-check** (the #220 / #239 bug class). The note must mandate a per-kind entry condition: the
   exact audit query over existing `object_links`, who signs it off, and the required-auth verdict
   per kind. This is the reason the lane is serialized.
5. **`ObjectCard` convergence sequencing** — which of `useObjectCard.ts`'s four legacy calls moves to
   the engine first, and what `GET /api/audit`'s missing-from-openapi status
   (`useObjectCard.ts:17-18` ponytail marker) implies for that move.

**Roots (owned):** `docs/decisions/notes/DN-XXXX-object-type-registry-unification.md` only.

**Must not touch:** all code. Any file outside `docs/decisions/notes/`.

**DoD.** The note answers all 5 questions with a chosen option and a rejected-alternatives section;
contains the full key-mapping table (no "TBD" rows); contains the literal `object_links` audit SQL;
names the follow-on implementation lanes with their roots and their order; is reviewed and signed off
by the program owner before any bridging lane is chartered. Zero code diff — a lane diff touching a
`.rs`/`.ts`/`.sql` file fails this lane.

**Size** M · **Risk** medium (a wrong winner costs a second migration of every console surface)

---

## L-A6 · Legacy object-type rows + code prefixes for the wave-2/3 module kinds

**Why.** The highest value-per-line lane in this lens, and it unblocks lenses B and C. The shared-
grammar scout §5 establishes that `web/src/console/ontology/codeGrammar.ts` compiles its regex from a
runtime-primed prefix set (`primeCodePrefixes:73-82`, **union-only, never replace**), fed by
`typeRegistrySource.ts:63-79`'s bootstrap fetch of `GET /api/v1/object-types`. Its verdict:
*"Adding 13 modules' codes therefore costs 13 backend registry rows with `code_prefix` set. Drag/parse:
zero further FE work."* The fidelity register logs drag-source findings — several at **blocker** —
against payroll, attendance, org (×2), dispatch, maintenance, field, equipment, logistics and
evaluation, every one of which needs a real issued prefix before `objDrag`'s `[코드 제목]` payload can
be truthful (design-intent C-36: *"an undraggable object representation is a violation"*; C-26/C-27
token grammar; DESIGN §4-20/§4-23).

**Verified safe (V1):** inserting a row **without** a `RESOLVABLE_KIND_AUTH` entry keeps the two
sync tests green — `objects.rs:2606-2609` states non-resolvable registry kinds are intentionally
absent because they count 0. This lane therefore never touches the security-reviewed file.

**Scope.** One migration (provisional `0204_seed_wave23_object_type_kinds.sql`) following the exact
landed pattern of `0113`/`0131:16`/`0188:6`:
`INSERT INTO object_types (kind, description, code_prefix) VALUES (…) ON CONFLICT (kind) DO NOTHING;`
- Kind list: derive from the wave-2/3 bodies recorded in
  `docs/evidence/console/wave23-consolidation-inventory.md` and `console-capability-registry.json`
  (payroll, recruiting, org, evaluation, maintenance, field, notif, board, directory, logistics,
  inventory, dispatch, sales). Only kinds with a real backing domain table get a row.
- Prefix values are **taken from the design authority, never invented**: the fidelity register
  already cites `AT-` (attendance), `RV-` (evaluation), `OC-` (org change), `ST-` (field site),
  `FL-`/`WO-`/`EQ-` (equipment/work order), `PO-` (purchase order), `OT-nn` (type codes). Where the
  authority names none, the lane records the choice and its authority search in the migration
  comment block — an undocumented prefix fails review.
- Every prefix must satisfy the `0113:26` CHECK `^[A-Z][A-Z0-9]*-$` and must be **unique** across
  the table: `codeGrammar.compile()` builds longest-prefix-first, so a duplicate silently
  mis-attributes codes to the wrong kind.
- Emit `docs/evidence/console/CAP-ONTOLOGY-ENGINE/manifests/code-prefixes.json` — the offline-floor
  manifest (`kind`, `codePrefix`, `description`) for whoever owns `FALLBACK_CODE_PREFIXES`
  (`codeGrammar.ts:16-19`) and the three kind maps (`composer/objectKinds.ts` `KIND_META`,
  `objectcard/kinds.ts` `SLUG_META` + `COMPOSER_KIND_TO_SLUG`). **This lane does not edit any of
  them** — the shared-grammar brief assigns all thirteen to one grammar lane.

**Roots (owned):** `backend/crates/platform/db/migrations/0204_seed_wave23_object_type_kinds.sql`
(new), `docs/evidence/console/CAP-ONTOLOGY-ENGINE/manifests/code-prefixes.json` (new),
`backend/app/tests/object_types_registry_as_runtime_role.rs` (new test file only).

**Must not touch:** `backend/app/src/objects.rs` (V1 makes this unnecessary — an edit here means the
lane went out of scope), `backend/app/src/lib.rs`, `web/**`, universal list.

**DoD.**
- Integration test as `mnt_rt` (app-tier harness: `mnt_buck_admin` bootstrap +
  `mnt.sqlx_test_bootstrap` GUC, per project memory): `GET /api/v1/object-types` returns every new
  kind with its `code_prefix`, `status`, and `active_count = 0`.
- Assert prefix uniqueness across the whole `object_types` table (`SELECT code_prefix, count(*) …
  HAVING count(*) > 1` returns zero rows) and CHECK-conformance of every new value.
- Re-running the migration is a no-op (`ON CONFLICT` proof).
- `cargo test -p mnt-app --test object_types_registry_as_runtime_role`
- `cargo test -p mnt-app --lib objects::` — the two sync tests
  (`resolvable_kinds_and_declared_auth_stay_in_sync`, `count_kind_and_resolvable_kinds_stay_in_sync`)
  pass **unchanged**, which is the structural proof the resolvable surface was not widened.
- Manifest JSON validates and lists exactly the migration's rows.
- Evidence: test output + the authority citation table for every prefix chosen.

**Size** M · **Risk** low-medium (prefix collision is the one real failure mode; the DoD test catches it)

---

## L-A2 · Schema publish route

**Why.** Scout §1b. `PgOntologyStore::transition_lifecycle` (`adapter-postgres/src/lib.rs:511-568`)
is the only schema-lifecycle mutator and its **only callers in the entire repo** are
`adapter-postgres/tests/registry_rls_surfaces_as_runtime_role.rs:169,205`. Neither
`web/src/api/ontology.ts` nor `web/src/console/ontology/wire.ts` has a publish call. So the
Ontology Manager can create (`rest/src/lib.rs:225`) and stage revisions (`:367-389`, which accepts a
full draft including `actions` with `dispatch`/`dispatch_target`) but the result is **stranded in
`draft` forever and can never serve instances**. This is the no-code loop's missing last step —
design-intent ONT-1 (proposal sandbox → review → active schema v+1) and C-6.

**Scope.** One route on the ontology REST router (`rest/src/lib.rs:219-243`):
`POST /api/v1/ontology/object-types/{key}/publish`.
- Body carries the four-eyes approval reference; header `If-Match` carries `key_write_etag`
  (`ObjectTypeSummary.key_write_etag`). Mismatch → 412 via the existing CAS mapping (see
  `object_type_cas_as_runtime_role.rs`).
- Runs on `command_pool()`, **not** `mnt_rt`: `adapter-postgres/src/lib.rs:507-509` states "draft
  publication is never available to mnt_rt". Follow the instance-write precedent for pool selection.
- Publication **consumes target-bound four-eyes evidence atomically** (same doc comment) — the route
  must pass the approval ref through, never synthesize one.
- Publishing auto-attaches the generic `create` action in SQL (`0165:1039`, `('create','저장', …)`).
  The route asserts this rather than duplicating it in Rust.
- **Publish only.** Deprecate/archive have no consumer; do not build them.
- `openapi.yaml` is owned by another agent right now. Emit
  `docs/evidence/console/CAP-ONTOLOGY-ENGINE/openapi/publish-route.fragment.yaml` — a hand-written
  (never spliced; `9bb877c6` reverted a mechanical splice that corrupted the file) fragment carrying
  `tags: [ontology]` on the operation (a missing per-domain tag regresses the Kotlin client to a
  monolithic `DefaultApi.kt` that OOMs kotlinc — project memory / PR #261). Record in the manifest
  that the integrator owes the three drift gates plus regenerated `clients/{ts,kotlin,swift}`.

**Roots (owned):** `backend/crates/ontology/rest/src/lib.rs`,
`backend/crates/ontology/rest/tests/publish_route_as_runtime_role.rs` (new),
`backend/crates/ontology/rest/BUCK` (own hunk),
`docs/evidence/console/CAP-ONTOLOGY-ENGINE/openapi/publish-route.fragment.yaml` (new).

**Must not touch:** `backend/openapi/openapi.yaml`, `clients/**`, `seed.rs`, `backend/app/src/**`,
`web/**`, universal list.

**DoD.** New test file, every assertion as `mnt_rt` where the caller is a runtime principal:
1. a `mnt_rt`-pooled caller **cannot** publish (the command-pool boundary holds) — assert the error,
   assert the type is still `draft`.
2. the command-pool path publishes a staged draft; the type's `lifecycle_state` becomes `published`
   and it immediately serves `GET /ontology/instances?type=`.
3. the generic `create` action is auto-attached (assert its presence and its `저장` title, matching
   `publish_auto_create_action_as_runtime_role.rs`).
4. missing / stale `If-Match` → 412 with the current etag echoed.
5. missing or already-consumed four-eyes ref → the gate error, and the type stays `draft`.
6. republish of an already-published key → 409, no second `create` action.
7. cross-tenant: org B cannot publish org A's key; the response is a 404, never a 403 (deny-by-
   omission, matching `resolve_code`'s contract at `rest/src/lib.rs:1601`).
8. canonical error envelope on every failure path.

Commands: `cargo test -p mnt-ontology-rest --test publish_route_as_runtime_role` ·
`python3 tools/buck/gen_first_party.py` + commit only the rest-crate BUCK hunk ·
`buck2 test //backend/crates/ontology/rest:mnt-ontology-rest-itest-publish_route_as_runtime_role`.

**Size** M · **Risk** medium (four-eyes consumption + pool selection are the security-relevant parts)

---

## L-A3 · First live projected action — equipment status via `registry.update_equipment`
### catalog train #1 · depends on L-A1

**Why.** The smallest possible real proof that §18 works end to end, and it needs **zero new
handler code**. Scout §2 R1/R3: `update_equipment_projected_handler` (`app/src/lib.rs:2478-2513`) is
written, registered as `"registry.update_equipment"` (`:2518-2523`), and installed (`:3199`) — and it
is **unreachable dead code in production**, because no seeded action type has `dispatch_target` set:
`create_action` hardcodes `dispatch_target: None` (`seed.rs:121`) and `projected_draft` ships
`actions: Vec::new()` (`seed.rs:181-204`). Closes the equipment fidelity blocker (backend-blocked:
*"Unit detail and case detail show kv + history/stepper only — no object card, no acting automations,
no series, no derived analytics"*) because V2 shows `object_type_acting`/`instance_acting` already
serve the dynamics layer once an action exists. Design-intent ONT-3 (Actions are writeback functions,
policy-evaluated + audited + guardrailed), C-2 (kinetic layer), C-32.

**Scope.** Exactly one `ActionTypeInput` added to `equipment_draft` (`seed.rs`):
- key `update_status`, Korean title from the design authority; `dispatch: ProjectedUsecase`;
  `dispatch_target: Some("registry.update_equipment")`.
- `submission_criteria: []` — **mandatory**. `rest/src/lib.rs:784-796` hard-rejects criteria on
  projected actions (the engine cannot read the domain row, so a criterion would fail *open*).
- `control_points`: `authority` + `self_checklist`. **Not `four_eyes`** — R6: for projected actions
  four-eyes is consumed in a separate committed step *before* dispatch (`rest/src/lib.rs:801-823`),
  so a failed dispatch spends the approval. The lane's decision note records this as a deliberate,
  named deferral to L-A4b, not an omission.
- Bump `BUILTIN_CATALOG_VERSION` to `2026-07-25.1`; recompute the manifest digest
  (`builtin_catalog_manifest()` → canonical JSONB → sha256, matching `0165:1113`); add the allowlist
  row with `supersedes_version = '2026-07-19.1'` in migration `0205_ontology_catalog_2026_07_25_1.sql`.
- Ride L-A1's `upgrade_builtin_catalog` for already-seeded tenants. Fresh tenants take the install path.
- **No `app/src/lib.rs` edit.** If the lane finds itself editing the hottest backend file on the
  spine, it has gone out of scope.

**Roots (owned):** `backend/crates/ontology/adapter-postgres/src/seed.rs` (the `equipment_draft`
action + the version constant + the digest fixture),
`backend/crates/platform/db/migrations/0205_ontology_catalog_2026_07_25_1.sql` (new),
`backend/app/tests/ontology_projected_action_equipment.rs` (new test file),
`backend/app/BUCK` — **do not regenerate**; the app BUCK is proven stale (spine-delta §4) and is the
spine's to fix. Record the new test target as an owed item in the evidence dir instead.

**Must not touch:** `backend/app/src/**`, `backend/crates/ontology/rest/src/**`, `web/**`,
universal list.

**DoD.** Story-level integration test (`backend/app/tests/…`, `mnt_buck_admin` bootstrap +
`mnt.sqlx_test_bootstrap` GUC, **every assertion executed as `mnt_rt`**):
1. seed/upgrade → `GET /api/v1/ontology/object-types/equipment` surfaces the `update_status` action
   with its `dispatch_target`.
2. `POST /api/v1/ontology/actions/preflight` returns the gate chain (authority + self_checklist)
   with real verdicts — not a stub.
3. `POST /api/v1/ontology/actions/execute` flips `registry_equipment.status` to the requested value.
4. the registry crate's own `audit_events` row exists with before/after — proof the domain use-case
   ran as sole writer (§9.3: no second source of truth).
5. a principal without the registry feature → 403 with **no leakage** of the equipment's existence.
6. an unknown `dispatch_target` still returns `not_wired_yet` (`rest/src/lib.rs:184`, error code at
   `:1695-1712`) — the fail-closed default is intact.
7. an invalid `status` param → the domain validation error survives the `ActionError::domain` shim
   with its `KernelError.kind` (404/409/412/403 preserved, `rest/src/lib.rs:701-704`).
8. `ExecuteOutcome.projected` is populated **and `receipt` is `None`** — assert the known R5 gap
   explicitly so it is named in the test, never hidden.
9. cross-tenant: org B cannot execute against org A's equipment id.
10. `GET /api/v1/ontology/object-types/equipment/acting` returns 200 (dynamics layer live, V2).

Commands: `cargo test -p mnt-app --test ontology_projected_action_equipment` ·
`cargo test -p mnt-ontology-adapter-postgres` (catalog digest tests still green) ·
`buck2 test //backend/crates/ontology/adapter-postgres:...`.
Evidence: full test output + the computed digest + the allowlist row SQL + the four-eyes deferral note.

**Size** M · **Risk** medium (the digest must be computed from the real manifest, never hand-typed)

---

## L-A7 · Register the wave-2/3 module object types as ontology projections
### catalog train #2 · depends on L-A1, L-A3, L-A8

**Why.** Design-intent C-5 (*"one type registry … ALL surfaces reference it; defining the same
ontology twice is a violation"*), C-6 (*"literally everything is ontology"*), B-2. Nine wave-2/3
backends and ten frontends landed dark on this spine with **no ontology registration at all** — the
coverage matrix's `finance_voucher` case ("domain exists, ontology registration absent") is now the
rule, not the exception. V2 is the payoff: `object_type_acting` / `instance_acting` already serve the
dynamics layer, and `list_projected_rows_tx` (`instances.rs:417-455`) already serves rows, so
**registration alone** closes the backend-blocked fidelity findings on equipment (blocker),
inventory, field, directory and maintenance (*"no object card, no acting automations/policies chips,
no series"*).

**Scope.** Per domain, four coordinated edits — the second is the one whose omission makes the type
register cleanly and then 400 on every list (`instances.rs:1016`):
1. `seed.rs`: a `projected_draft(...)` with `projected_prop` / `choice_prop` / `fk_link` builders, a
   `*_KEY` const, and entries in `PROJECTED_DOMAIN_KEYS` (`seed.rs:1132-1148`) and the `drafts` vec
   (`seed.rs:1178-1207`).
2. `instances.rs:955-974`: add the backing table to `allowlisted_projected_table`.
3. Links use `to_stable_key` strings only — the installer raises
   `ontology_builtin.physical_link_id_forbidden` on a physical id (`0165:1153-1157`); the rewrite
   pass lives at `seed.rs:1225-1246`.
4. Catalog: bump to `2026-07-25.2`, recompute the digest, migration
   `0206_ontology_catalog_2026_07_25_2.sql` with `supersedes_version = '2026-07-25.1'`.

**Already projected — do not duplicate:** `work_order`, `support_ticket`, `equipment`, `employee`,
`customer`, `site`, `approval`, `evidence`, `leave_request`, `workflow_definition`,
`messenger_thread`, `mail`, and the three compliance types (`PROJECTED_DOMAIN_KEYS`, 15 entries).

**Backing tables confirmed present** (verified by `grep -rn "CREATE TABLE" migrations/`):
`applicants` (0187), `evaluation_cycles` (0190), `notices` (0162), `org_change_requests` (0198),
`logistics_asns` (0179), `inventory_items` (0156). The remaining candidates named in the wave-2/3
inventory (payroll run, posting, rental agreement, dispatch assignment, production order) have **no
table under those names** — the lane must locate the real table per domain crate before drafting, and
**skip any domain with no backing table**, recording the skip and its reason. Never invent a table
name: `allowlisted_projected_table` returns the compiled-in literal, so a wrong name is a silent
dead type.

**Actions stay `Vec::new()`.** No `dispatch_target`, no handler, no `app/src/lib.rs` edit. Per the
scout, §18 wiring is chartered per-target after L-A3 proves the pattern.

**Roots (owned):** `backend/crates/ontology/adapter-postgres/src/seed.rs`,
`backend/crates/ontology/adapter-postgres/src/instances.rs` (the allowlist fn only),
`backend/crates/ontology/adapter-postgres/tests/wave23_projections_as_runtime_role.rs` (new),
`backend/crates/ontology/adapter-postgres/BUCK` (own hunk),
`backend/crates/platform/db/migrations/0206_ontology_catalog_2026_07_25_2.sql` (new).

**Must not touch:** `backend/app/src/**`, `backend/crates/ontology/rest/src/**`, `web/**`, any
domain crate, universal list.

**DoD.** Per registered type, as `mnt_rt`:
1. `GET /ontology/object-types` lists it with `backing_kind = projected` and `lifecycle_state = published`.
2. `GET /ontology/instances?type=<id>` returns **real domain rows** as `InstanceState`, with
   `version = 1` and empty fixity hashes (the documented projected contract, `instances.rs:993-999`).
3. `GET /ontology/object-types/{key}/acting` returns 200.
4. every declared `backing_column` exists on the backing table (a schema-drift test that fails loudly
   the day a domain lane renames a column).
5. cross-tenant: org B's list returns zero rows for org A's data (RLS as `mnt_rt`, not superuser).
6. a type whose backing table is *not* in the allowlist is rejected at draft-validate time (proving
   L-A8 item 1 landed) — not at read time.
7. the catalog upgrade from `2026-07-25.1` to `.2` is exercised end-to-end on a tenant that already
   has the `.1` catalog, and the `update_status` action added by L-A3 survives untouched.
8. skipped domains are enumerated in the lane's evidence note with the reason (no backing table),
   not silently absent.

Commands: `cargo test -p mnt-ontology-adapter-postgres --test wave23_projections_as_runtime_role` ·
`cargo test -p mnt-ontology-adapter-postgres` (full crate, catalog tests included) ·
`python3 tools/buck/gen_first_party.py` + own hunk · `buck2 test //backend/crates/ontology/adapter-postgres:...`.

**Size** L · **Risk** medium — the scope is wide but each type is mechanical; the real risk is a
wrong backing-table name, which DoD item 4 catches.

---

## L-A9 · Projected code resolution (R8)
### catalog train #3 · depends on L-A1, L-A7

**Why.** Scout R8: `resolve_by_code` (`adapter-postgres/src/lib.rs:829-863`) queries **only**
`ont_instances` joined to `ont_instance_revisions`, so a `WO-`/`EQ-`/`AT-`/`RV-` code on a projected
type cannot resolve. This is the backend blocker behind the token-grammar fidelity findings:
evaluation major — *"evidence 개체 코드 is an unvalidated free-text input: any string becomes an
object_ref rendered as a code chip"* against DESIGN §4.7-7 (`!CODE` links resolve **only** to
PBAC-authorized objects) and §4.7-10 (완전 추적성: a code that isn't a real link is banned); inventory
major — *"MovementSourceToken renders WO-/PO- ids as inert mono spans"*; maintenance blocker; notif
minor. Design-intent C-26 (one token grammar app-wide, PBAC-gated at resolution time, *"unauthorized
`!CODE` does not link"*) and C-27.

**V3 makes this a schema change, not a query widening:** `work_order_draft` registers `request_no`,
not `code` (`seed.rs:302-308`); only four instance-backed drafts register a `code` property
(`:421,459,505,535`). Projected types have **no declared code column**.

**Scope.**
1. **Declare the code column per type.** Add `code_property_key` to `ont_object_types` (mirroring the
   existing `title_property_key`) in migration `0207_ontology_code_property_key.sql`, nullable,
   plumbed through `CreateObjectTypeDraft`, `validate_draft` (must reference a registered property
   whose `backing_column` passes `is_safe_ident`), the installer's pass 1 (`0165:1160+`), and
   `ObjectTypeSummary`.
2. **Set it on the projected drafts that have a human code**: `work_orders.request_no`,
   `registry_equipment.<asset code>`, `support_tickets.<ticket no>`, and each type L-A7 registered
   that carries one. Types with no human code leave it NULL and are excluded from resolution.
3. **Extend `resolve_by_code`**: keep the existing `ont_instances` query, then — only if it misses —
   resolve across projected types via a single UNION over the allowlisted backing tables that declare
   a `code_property_key`, `LIMIT 1`. Table and column names come from `allowlisted_projected_table`
   (which returns the compiled-in literal, never the caller string) and from the registered
   `backing_column` guarded by `is_safe_ident` (`instances.rs:979-988`) — **never** from caller input.
   `with_org_conn` keeps it RLS-scoped.
4. **Preserve the contract verbatim**: unknown or cross-tenant → `None` → the caller renders 404,
   never 403 (`adapter-postgres/src/lib.rs:823-826`, `rest/src/lib.rs:1601`) — the endpoint must leak
   neither existence nor cross-tenant membership.
5. Catalog: bump to `2026-07-25.3`, recompute the digest, `supersedes_version = '2026-07-25.2'`.
6. Delete the now-obsolete `ponytail:` comment at `adapter-postgres/src/lib.rs:827-828` ("lift to a
   per-type configurable code property if a type ever names its code column differently") — this lane
   is that lift.

**Roots (owned):** `backend/crates/ontology/adapter-postgres/src/lib.rs` (`resolve_by_code`,
`ObjectTypeSummary`, `validate_draft`), `backend/crates/ontology/adapter-postgres/src/seed.rs`
(code-property designations + version), `backend/crates/ontology/adapter-postgres/src/instances.rs`
(read-only use of the allowlist),
`backend/crates/ontology/adapter-postgres/tests/projected_code_resolution_as_runtime_role.rs` (new),
`backend/crates/platform/db/migrations/0207_ontology_code_property_key.sql` (new).

**Must not touch:** `backend/app/src/**`, `web/**`, `backend/crates/ontology/rest/src/lib.rs`
routing (the `resolve_code` handler at `:1595-1601` is unchanged), universal list.

**DoD.** As `mnt_rt`:
1. an existing `WO-…` code on a real `work_orders` row resolves to `{id, type_key: "work_order", title}`.
2. an instance-backed code still resolves via the original path (no regression) — assert the
   `ont_instances` path is tried first.
3. an unknown code → `None` → 404; a **cross-tenant** code → 404 and **not** 403; assert the response
   body is byte-identical for the two cases (no existence leak).
4. SQL-injection attempt: a type registered (via REST) with a hostile `backing_column` is rejected by
   `is_safe_ident` at draft time; a hostile *code argument* is bound as a parameter and resolves to
   `None`. Include a literal `'; DROP TABLE` case.
5. a projected type with `code_property_key = NULL` is excluded from the UNION.
6. two types sharing a code value → deterministic single result (document the ordering rule; do not
   leave it to plan chance).
7. query-plan sanity: the UNION issues one round trip, not one per type (assert via a query-count
   probe or a bounded `EXPLAIN` review recorded in evidence).
8. catalog upgrade `.2` → `.3` exercised end-to-end; L-A3's action and L-A7's types survive.

Commands: `cargo test -p mnt-ontology-adapter-postgres --test projected_code_resolution_as_runtime_role` ·
`cargo test -p mnt-ontology-adapter-postgres` · `buck2 test //backend/crates/ontology/adapter-postgres:...`.

**Size** M · **Risk** medium-high — dynamic SQL over table/column names is the one place in this lens
where a mistake is a security bug, not a bug. Treat the identifier plumbing as security-reviewed code.

---

## L-A4a · Projected command receipt DESIGN GATE  ⟵ DECISION, doc-only

**Why.** Scout §2 R5/R6, and the parent's explicit instruction: this is a real design decision, not a
mechanical port. The ORU Slice-2 receipt machinery landed 2026-07-23 (`e117d048` → `4e3df210`) and is
**instance-only**: migration `0177_ontology_action_command_receipts.sql` (PK `(org_id, command_id)`,
`payload_digest BYTEA(32)`, FORCE RLS `org_isolation`, **immutability trigger on UPDATE/DELETE**,
`GRANT SELECT, INSERT` to `mnt_rt` only), consumed by the execute path at `rest/src/lib.rs:1058-1202`
(command id required `:1062-1064`, `pg_advisory_xact_lock` `:1074-1078`, replay returns the stored
receipt `:1083-1094`, different actor → forbidden `:1085-1087`, same id + different digest → 409
`:1088-1091`, CAS before four-eyes consumption `:1096-1105`, receipt written in the same tx `:1191-1202`).

Projected dispatch has **none** of it: `receipt: None` at `rest/src/lib.rs:840`. And it cannot simply
be ported — the domain use-case commits in its own transaction (TOCTOU explicitly disclaimed at
`rest/src/lib.rs:770-775`), so a receipt cannot be atomic with the mutation. Compounding it, `0177`'s
immutability trigger **forbids UPDATE**, so "claim now, record the outcome later" needs either a
second row or a trigger amendment. That is the decision.

**Scope — one decision note, no code.** `docs/decisions/notes/DN-XXXX-projected-action-receipts.md`
must answer, each with a rejected-alternatives paragraph:
1. **Which design.** (a) lazy claim → dispatch → record (the scout's defensible default);
   (b) a domain-side hook passing `&mut Transaction` into every use-case (atomic, but changes every
   domain crate's signature); (c) an outbox. Name the cost of each in files touched.
2. **What an interrupted dispatch looks like to a replaying caller.** A claimed-but-unresolved
   receipt must produce a *verdict* — `claimed_unresolved` returned to the caller, silent re-execute,
   or 409 — never a silent second domain write. Pick one and justify it against the instance path's
   contract so the two are not gratuitously different.
3. **Does four-eyes consumption move after the claim (R6)?** Today it is consumed in a separate
   committed step *before* dispatch (`rest/src/lib.rs:801-823`) and a failed dispatch spends the
   approval. Decide whether the claim becomes the consumption point.
4. **The receipt row's outcome state machine and its storage**, given `0177`'s UPDATE ban: a second
   `outcome` row keyed `(org_id, command_id, seq)`, a nullable `outcome` column with a narrowed
   trigger (`UPDATE` permitted only NULL → non-NULL, once), or a separate table. Name the exact
   migration and columns L-A4b will write.
5. **Handler idempotency.** A non-idempotent handler makes any replay policy unsound. Mandate a
   per-target idempotency declaration on registration, and say what happens to a target that does not
   declare one (refuse to register / refuse receipts / warn).

**Roots (owned):** `docs/decisions/notes/DN-XXXX-projected-action-receipts.md` only.

**Must not touch:** all code.

**DoD.** All 5 questions answered with a chosen option and a rejected-alternatives section; the note
names the exact migration + columns + trigger change for L-A4b, and the exact test list L-A4b must
satisfy; reviewed and signed off before L-A4b is chartered. Zero code diff.

**Size** S · **Risk** low (the risk is skipping it)

---

## L-A4b · Projected command receipts  ⟵ depends on L-A4a sign-off

**Why.** R5/R6. Extending deterministic receipts to projected dispatch is the highest-value §18
increment after L-A3 — it is what makes a projected action safely retryable, and therefore what makes
the §18 surface usable by real clients rather than a demo.

**Scope.** Implement exactly the design signed off in DN-XXXX (L-A4a) — no re-litigation. Concretely,
whatever the note picks must produce:
- `command_id` required on projected actions (parity with the instance requirement at
  `rest/src/lib.rs:1062-1064`), rejected with the canonical envelope when absent.
- The claim/record sequence around the dispatch at `rest/src/lib.rs:758-842`, reusing
  `pg_advisory_xact_lock` on the command id (`:1074-1078`) so same-id attempts serialize.
- Replay: same actor + same digest → the stored receipt, no second dispatch. Different actor →
  `forbidden`. Same id + different digest → 409. Byte-identical semantics to the instance path except
  where the note deliberately diverges (and where it does, the divergence is commented in code with a
  pointer to the note).
- A claimed-but-unresolved receipt returns the note's chosen verdict, never a silent re-dispatch.
- The migration the note specifies (provisional `0208_projected_action_receipts.sql`).
- `ExecuteOutcome.receipt` populated for projected actions — the assertion L-A3 pinned as `None`
  flips here, and L-A3's test is updated in the same commit.

**Roots (owned):** `backend/crates/ontology/rest/src/lib.rs` (the `ProjectedUsecase` execute arm),
`backend/crates/ontology/rest/tests/projected_receipts_as_runtime_role.rs` (new),
`backend/crates/ontology/rest/BUCK` (own hunk),
`backend/crates/platform/db/migrations/0208_projected_action_receipts.sql` (new),
`backend/app/tests/ontology_projected_action_equipment.rs` (the one assertion flip only).

**Must not touch:** `backend/app/src/lib.rs`, `seed.rs`, `web/**`, universal list.

**DoD.** As `mnt_rt`:
1. replay with the same actor + digest returns the stored receipt **and the domain row is provably
   unwritten a second time** — assert `registry_equipment.updated_at` / version is unchanged.
2. a handler that returns an error leaves a resolved-failed receipt; a subsequent identical replay
   returns that receipt rather than re-dispatching.
3. an interrupted dispatch (simulated by aborting between claim and record) produces the note's
   chosen verdict on replay — assert the exact response, and assert no second domain write.
4. concurrent same-`command_id` executes serialize; exactly one domain write occurs.
5. different actor, same id → `forbidden`; same id, different digest → 409.
6. cross-tenant: org B cannot read or collide with org A's receipt (FORCE RLS `org_isolation`).
7. the `0177` immutability guarantee still holds for whatever the note changed — an arbitrary UPDATE
   or DELETE on a receipt row still raises.
8. L-A3's story test passes with the flipped `receipt` assertion.

Commands: `cargo test -p mnt-ontology-rest --test projected_receipts_as_runtime_role` ·
`cargo test -p mnt-app --test ontology_projected_action_equipment` · `buck2 test //backend/crates/ontology/rest:...`.

**Size** L · **Risk** HIGH — the highest-risk lane in the lens. Do not start before L-A4a is signed off.

---

## Open decisions this lens needs resolved

1. **Who owns the catalog train?** L-A3/L-A7/L-A9 must be one serialized chain under one owner.
   If they are dispatched to three agents in parallel, all three fail at the digest.
2. **L-A5a's winner constrains L-A9 and L-A6.** If the decision makes the ontology engine the prefix
   authority, L-A6's legacy migration becomes transitional and needs a sunset note; if legacy wins,
   `ObjectTypeSummary` never grows `code_prefix` and L-A9's `code_property_key` is the only bridge.
   L-A6 is safe to run either way (it is additive and reversible); L-A9 is not — hold L-A9 until
   L-A5a answers question 3.
3. **`backend/app/BUCK` is stale and divergent at HEAD** (spine-delta §4: 6056 `mnt-` dep lines, zero
   matches for `recruiting|orgchange|evaluation`; the Buck app target cannot compile). L-A3's and
   L-A6's app-tier test targets cannot be registered until the spine regenerates it. Both lanes ship
   cargo evidence + an owed-target note; the program must decide whether that satisfies the
   Buck2-only completion rule or whether the regen is a prerequisite.
4. **Fan-out epoch.** `console-capability-registry.json` reports `fanout_epoch.current_epoch: 1`,
   `normalized_lane_ids: []`, and "legacy lanes remain explicitly held". Every lane here needs epoch
   normalization before dispatch, and there is **no `CAP-ONTOLOGY-ENGINE` row** in the 24-row registry
   — one must be seeded (with correct `frontend_roots` / `backend_roots`, avoiding the
   `CAP-DOCS-EVIDENCE-CONSOLE` mistake of declaring a root that does not exist on disk).
5. **Four-eyes on projected actions is deferred, not solved** (L-A3 ships `authority` +
   `self_checklist` only). If any wave-4 module needs a four-eyes-gated projected action before
   L-A4b lands, it is blocked — surface that to lens D now rather than discovering it mid-build.
