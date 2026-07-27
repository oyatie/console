# Wave-4 scout — ontology engine vs lens A (projection registration + §18 action dispatch)

Scope: read-only survey of `backend/crates/ontology/**` on the PR-488 spine worktree
(`/Users/jasonlee/Developer/maintenance-worktrees/pr488-design-mirror-sync`, branch
`wave23-consolidation-20260724`), plus DN-0003, the two prior program docs, and the
frontend consumers. All line numbers are from that worktree at survey time.

## Prior art — do not re-derive

| Doc | What it already settles |
|---|---|
| `.omc/research/be-ontology-engine-arch.md` (= `docs/program/be-ontology-engine-arch.md`, identical, 321 lines) | The **design spec**, written pre-code. §1a/§1b projected-vs-instance split, §18 table shapes, §16 gate chain, §5d Cedar residual, the 7 build sub-lanes, and the 8 flagged risks. Header still says "Status: design, no code" — that is now **stale**; most of §1/§2/§4 is built. Read it as intent, not as state. |
| `docs/program/ontology-coverage-matrix.md` (180 lines) | The **fixed-revision source audit** at `86a97771`. Names the exact 27 seeded tenant types (9 governed-config + 3 C-chain instance, 15 projected), the per-row test ceiling, the Evidence custody story, and the `finance_voucher` "domain exists, ontology registration absent" case. Its 5 "current planning implications" are still binding. |
| `docs/decisions/notes/DN-0003-adr-0025-operational-object-runtime.md` (219 lines) | The **ORU contract**: 10 core invariants, 4 implementation slices, the domain-module rule (8 deliverables per new module), and the 7 activation gates. Slice 1 = inspect/preflight/act/verify; Slice 2 = deterministic instance command (command id + digest + expected revision + receipt); Slices 3/4 explicitly DARK. |

Everything below is what those three do **not** say.

---

## 1. How an ObjectType is registered today

There are **three** registration paths, and only one of them is complete.

### 1a. Path A — the digest-allowlisted built-in catalog (the only path that produced the 27 types)

`seed_governed_config_object_types` → `install_builtin_catalog` → the DB function
`ontology_api.install_builtin_catalog`.

- Manifest built in Rust: `backend/crates/ontology/adapter-postgres/src/seed.rs:1159-1250`
  (`builtin_catalog_manifest`), pinned by `BUILTIN_CATALOG_VERSION = "2026-07-19.1"`
  (`seed.rs:68`).
- The DB canonicalizes the JSONB, sha256s it, and requires an exact match against a
  **migration-owned** allowlist row:
  `backend/crates/platform/db/migrations/0165_ontology_object_type_key_revisions.sql:121-131`
  (table `ont_builtin_catalog_allowlist`, sole row digest
  `e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247`), enforced at
  `0165:1113-1124` (`ontology_builtin.manifest_not_allowlisted`, ERRCODE 42501).
- **Two hard fail-closed guards with no upgrade path** (`0165:1128-1143`):
  - a tenant that already installed a *different* catalog version →
    `ontology_builtin.different_catalog_already_installed` (23505);
  - a tenant with **any** pre-existing `ont_object_types` row →
    `ontology_builtin.empty_org_required` (23514).
  - Exact re-install of the same version+digest is a no-op.
- Only `console_ontology_writer` may read the allowlist (`0165:230`); `console_rt` is revoked
  (`0165:132`).

**Consequence (the single biggest lens-A finding):** adding a 28th seeded type is not
"add a draft builder + bump the version + add a migration row". For any tenant that has
already installed `2026-07-19.1`, the installer *raises*. There is **no catalog-upgrade
function** in the repo. A wave-4 lane that adds seeded types must build one
(idempotent additive install: new keys only, existing keys untouched, digest chain
allowlisted), or every existing environment is stuck at 27.

### 1b. Path B — the no-code REST registry (Ontology Manager) — **incomplete: cannot publish**

Router: `backend/crates/ontology/rest/src/lib.rs:219-243`. Exactly 12 routes, all
present in `backend/openapi/openapi.yaml:11951-12996`.

```
POST /api/v1/ontology/object-types                create_object_type   (rest/src/lib.rs:225)
PUT  /api/v1/ontology/object-types/{key}          stage_object_type_revision (rest/src/lib.rs:367-389)
```

`stage_revision` takes a full `CreateObjectTypeDraft` — properties, links, **actions**
(incl. `dispatch` + `dispatch_target`), analytics — so a v+1 with a projected action
*can* be staged over REST.

**But there is no publish route.** `PgOntologyStore::transition_lifecycle`
(`adapter-postgres/src/lib.rs:511-568`) is the only schema-lifecycle mutator, and its
only callers in the whole repo are
`adapter-postgres/tests/registry_rls_surfaces_as_runtime_role.rs:169,205`. Neither
`web/src/api/ontology.ts` nor `web/src/console/ontology/wire.ts` has a publish call.
So a user-authored type is stranded in `draft` and can never serve instances.

Two further constraints on that publish, once it is exposed:
- `transition_object_type` runs as `console_ontology_cmd`, **not** `console_rt`
  ("draft publication is never available to console_rt", `adapter-postgres/src/lib.rs:507-509`),
  so the REST tier needs the command pool (`command_pool()`), same as instance writes.
- publication **consumes target-bound four-eyes evidence atomically** (same doc comment)
  — a publish route must carry an approval ref, not just an If-Match.
- publish auto-attaches the generic `create` action in SQL
  (`0165:1039`, `('create','저장', …)`) — closing "no-code gap ①". Note `seed.rs:100-104`
  claims `transition_lifecycle` "reuses `create_action`" in Rust; that is a **stale
  doc comment**, the logic lives in the migration. Proof test:
  `rest/tests/publish_auto_create_action_as_runtime_role.rs`.

### 1c. Path C — the LEGACY `object_types` table (a parallel, unrelated registry)

This is the one that actually feeds the console's code grammar, and it is **not the
ontology engine**.

- Table `object_types` with `kind` / `code_prefix` / `description` / `status`, seeded by
  migrations (`0113_create_object_code_counters.sql:25-49` sets the prefixes;
  `0131_create_series.sql:16`, `0188_create_attendance_console.sql:6` add more; ~21
  distinct kinds).
- Served at `GET /api/v1/object-types` — `backend/app/src/objects.rs:39,1578,1625`.
- Resolution/graph are gated by a **hand-maintained Rust table**
  `RESOLVABLE_KIND_AUTH` (`app/src/objects.rs:121-139`, 10 kinds) with hand-written
  `resolve_head` / `count_kind` match arms; consistency is enforced by unit tests at
  `app/src/objects.rs:2564-2630`. The header comment records that this table exists
  *because* two prior bugs (#220 work_order+equipment, #239 account) shipped
  Login-gated-only.

**The two registries do not share keys**: legacy `approval_run` vs ontology `approval`;
legacy `mail_thread` vs ontology `mail`; legacy `voucher`/`purchase_request`/`listing`/
`document`/`asset_transfer`/`notification` have no ontology counterpart at all;
ontology `contract`/`position`/`posting`/`console_view`/`sla_setting`/… have no legacy
counterpart.

### Projected vs instance-backed — the D1/D2 distinction as built

| | Projected (15 seeded) | Instance (12 seeded) |
|---|---|---|
| Store | the real domain table | `ont_instances` + `ont_instance_revisions` |
| Writer | the domain crate's own use-case (sole writer, `seed.rs:1091-1097`) | ontology revision append |
| Actions at seed time | **`actions: Vec::new()`** — `seed.rs:181-204` (`projected_draft`) | generic `create` action, `seed.rs:105-130` (`create_action`, 12 call sites) |
| Read | `list_projected_rows_tx` over the backing table, `instances.rs:417-455` | revision store, `instances.rs:429-450` |
| Property shape | `projected_prop` — `required:false`, `backing_column = key` (`seed.rs:133-151`) | `prop` — `required:true` |
| as-of / history | **none** — `get_as_of`/`history` only query `ont_instances*` (coverage matrix §"Dynamic and lifecycle boundary") | full, hash-chained |
| Fixity | `version` always 1, hashes empty (`instances.rs:993-999`) | per-(org,instance) chain |

Backing tables are constrained by a **compiled-in allowlist** returning the literal, not
the caller string: `allowlisted_projected_table`, `instances.rs:955-974` — 15 entries
matching the 15 projected types exactly. `is_safe_ident` (`instances.rs:979-988`) guards
`backing_column`. Note `validate_draft` (`adapter-postgres/src/lib.rs:1080-1180`) does
**not** constrain `backing_kind`/`backing_table`, so a REST-created projected type with
an unlisted table registers fine and then fails at read time with
`"projected object type has no allowlisted backing table"` (`instances.rs:1016`). That is
fail-closed but a poor authoring error — surfacing it at draft-validate time is a cheap
wave-4 win.

---

## 2. The §18 residual — where dispatch hits NotWiredYet, and what wiring one costs

### The machinery is built and fail-closed

`ProjectedDispatchRegistry` — `rest/src/lib.rs:159-189`.

```rust
// rest/src/lib.rs:181-188
async fn dispatch(&self, input: ProjectedDispatch) -> Result<Value, ActionError> {
    match self.handlers.get(&input.target) {
        Some(handler) => handler(input).await,
        None => Err(ActionError::NotWiredYet { target: Some(input.target) }),
    }
}
```

**The two NotWiredYet sites:**
- `rest/src/lib.rs:184` — unknown `dispatch_target` key.
- `rest/src/lib.rs:797-800` — a `projected_usecase` action with `dispatch_target = NULL`
  (`ActionError::NotWiredYet { target: None }`).

Both are reached from `execute_action` → `ActionDispatch::ProjectedUsecase`
(`rest/src/lib.rs:758-842`). Default state is empty (`rest/src/lib.rs:87-90,108`), so
absent App-tier installation *every* projected dispatch fails closed. Error mapping:
`rest/src/lib.rs:1695-1712` → code `"not_wired_yet"`; `:1785`.

### What one handler looks like (there is exactly ONE)

`backend/app/src/lib.rs:2478-2513` — `update_equipment_projected_handler`, registered as
`"registry.update_equipment"` at `app/src/lib.rs:2518-2523`, installed at
`app/src/lib.rs:3199`. Pattern per handler:

1. `ProjectedHandler = Arc<dyn Fn(ProjectedDispatch) -> Pin<Box<dyn Future<Output = Result<Value, ActionError>> + Send>> + Send + Sync>`
   (`rest/src/lib.rs:153-157`).
2. `ProjectedDispatch { principal, target, target_id: Option<Uuid>, params, reason, occurred_at }`
   (`rest/src/lib.rs:133-148`). Org is **ambient** via `app.current_org`, deliberately not
   in the struct.
3. Pull typed params out of `input.params`, build the domain command, call the domain
   store's use-case (which owns its own RLS + audit + tx).
4. Map the domain error via a per-crate shim — `registry_error_to_action_error`,
   `app/src/lib.rs:2466-2472`, preserving `KernelError.kind` so 403/409/404 survive
   (`ActionError::domain`, `rest/src/lib.rs:701-704`).
5. Return an opaque `Value` summary → `ExecuteOutcome.projected`.

**Cost per new target: ~35 lines in `app/src/lib.rs`, no new crate, no migration, no
trait.** The dependency inversion is already done (ontology REST has no domain-adapter
edge; App supplies the map, same as `TenantConfigSeeder`).

### The residual is NOT the handler — it is that no action can reach it

`registry.update_equipment` appears in exactly three places: the App handler, the two
test files, and a doc comment. **No seeded action type has `dispatch_target` set** —
`seed.rs:121` hardcodes `dispatch_target: None` on `create_action`, and `projected_draft`
(`seed.rs:181-204`) ships `actions: Vec::new()`. `list_object_types`+`get_action_type`
therefore cannot surface any projected action in production. The one wired handler is
**unreachable at runtime.**

So the real §18 residual, in order:

| # | Gap | Where |
|---|---|---|
| R1 | No seeded projected action exists → the wired handler is dead code in prod | `seed.rs:181-204`, `seed.rs:121` |
| R2 | Adding one requires either a catalog-version bump (blocked, §1a) or a REST publish route (missing, §1b) | `0165:1128-1143`; `rest/src/lib.rs:219-243` |
| R3 | 14 of 15 projected types have no handler even if an action existed | `app/src/lib.rs:2518-2523` |
| R4 | Submission criteria are **hard-rejected** for projected actions (the engine can't read the domain row, so a criterion would fail *open*) | `rest/src/lib.rs:784-796` |
| R5 | Projected dispatch gets **no receipt** — `receipt: None` | `rest/src/lib.rs:840`, `:661-664` |
| R6 | Four-eyes for projected is consumed in a **separate committed step before** dispatch (the domain tx can't be joined); a failed dispatch spends the approval | `rest/src/lib.rs:801-823` |
| R7 | TOCTOU-safety of the projected mutation is explicitly disclaimed and left to the domain use-case | `rest/src/lib.rs:770-775` |
| R8 | `resolve_by_code` only queries `ont_instances` → a `WO-…`/`EQ-…` code on a **projected** type cannot resolve | `adapter-postgres/src/lib.rs:829-863` |

### What the ORU command-receipt work already provides (Slice 2, landed 2026-07-23)

Commits `e117d048` → `4e3df210` (feat: deterministic ontology action receipts; replay
before gate consumption; bind receipts to executing actor; canonicalize immutable
payloads; isolation test).

- Migration `0177_ontology_action_command_receipts.sql`: `ont_action_command_receipts`,
  PK `(org_id, command_id)`, `payload_digest BYTEA(32)`, `receipt JSONB`, FORCE RLS
  `org_isolation`, immutability trigger on UPDATE/DELETE, `GRANT SELECT, INSERT` to
  `console_rt` only.
- Execute path `rest/src/lib.rs:1058-1202`:
  - `command_id` **required** for `instance_revision` (`:1062-1064`);
  - `expected_revision` **required** whenever `instance_id` is present (`:1065-1068`);
  - `pg_advisory_xact_lock` on the command id serializes same-id attempts (`:1074-1078`);
  - replay: same actor + same digest → the **stored receipt**, no second write (`:1083-1094`);
  - different actor → `forbidden` (`:1085-1087`); same id + different digest →
    `conflict`/409 (`:1088-1091`);
  - CAS: `SELECT r.version … FOR UPDATE`, mismatch → `ActionPreconditionFailed{current}`
    → 412, **before** four-eyes consumption (`:1096-1105`);
  - four-eyes re-consumed and the whole §16 chain re-run **inside** the tx (`:1106-1130`);
  - receipt written in the same tx (`:1191-1202`).
- `CommandReceipt { command_id, payload_digest, instance, gates }` — `rest/src/lib.rs:669-674`.

**All of this is instance-only.** It is exactly the shape lens A needs for projected
dispatch and exactly what projected dispatch does not have (R5/R6). Extending receipts
to projected actions is the highest-value §18 increment, and it is genuinely hard: the
domain use-case commits in its own tx, so a receipt cannot be atomic with the mutation
without a domain-side hook. A defensible lazy design: write the receipt row *first* in
its own committed tx (claiming the command id), dispatch, then record the outcome — an
interrupted dispatch leaves a claimed-but-unresolved receipt that replay reports rather
than silently re-executing. That is a real design decision for the lane charter, not a
mechanical port.

### Gate evidence (already there, reusable)

Gate chain is the governance crate's, not ontology's: `GateChainConfig`/`GateChainOutcome`/
`GateEvidence`/`evaluate_gate_chain` from `console_governance_domain` (`rest/src/lib.rs:33-36`).
Config parsed from `ont_action_types.control_points` by
`parse_control_points` (`application/src/lib.rs:42-70`; recognizes `authority`,
`self_checklist`, `four_eyes`, `egress_dlp`). Authority today = the **legacy**
authorization contract via `authority_effect_from_cedar` (`rest/src/lib.rs:31`,
`:926-929`) — Cedar is still the shadow seam, not the enforcer.

---

## 3. Registering ONE new module type end-to-end — the real touch list

Assume a new projected module type (the common wave-4 case: a domain crate landed in the
wave-2/3 consolidation and now wants an ontology projection + one write action).

**Backend**

1. `seed.rs`: a `projected_draft(...)` + `projected_prop`/`choice_prop`/`fk_link` builder,
   a `*_KEY` const, and an entry in `PROJECTED_DOMAIN_KEYS` + the `drafts` vec in
   `builtin_catalog_manifest` (`seed.rs:1178-1207`).
2. `instances.rs:955-974`: add the backing table to `allowlisted_projected_table`.
   Without this the type registers and then 400s on every list.
3. **Catalog upgrade** — bump `BUILTIN_CATALOG_VERSION`, compute the new digest, add a
   migration inserting into `ont_builtin_catalog_allowlist`, **and build the additive
   upgrade path that does not exist** (§1a). Alternatively expose the publish route
   (§1b) and register at runtime. One of these two is unavoidable and is the lane's
   critical path.
4. Action: an `ActionTypeInput` with `dispatch: ProjectedUsecase` and a `dispatch_target`,
   `submission_criteria: []` (R4 forbids anything else), `control_points`.
5. `app/src/lib.rs`: a `ProjectedHandler` + a `register(...)` line + a
   `*_error_to_action_error` shim for the new domain crate (~35 lines).
6. Links: `fk_link` entries; note the catalog installer **forbids physical link ids**
   (`0165:1153-1157`, `ontology_builtin.physical_link_id_forbidden`) — link targets must
   be `to_stable_key` strings, resolved in the installer's pass 2
   (`seed.rs:1225-1246` does that rewrite).
7. OpenAPI: **no change** if only registry data is added — the 12 ontology routes are
   already documented (`openapi/openapi.yaml:11951-12996`). A *publish route* would
   require openapi.yaml + regenerated ts/kotlin/swift clients (3 CI drift gates; every
   op needs a per-domain `tags:` or the Kotlin client OOMs).
8. Analytics: `AnalyticSummary` (`adapter-postgres/src/lib.rs:302-308`) is optional per
   draft; the workbench aggregates client-side over
   `GET /ontology/instances?type=` (`web/src/console/ontology/analytics/OntologyAnalyticsWorkbench.tsx:180`).
   Reads are already policy-filtered (commit `bae882bf`, "filter analytics reads by policy").
   **No per-type registration needed.**

**Frontend — smaller than expected; two of the four feared gaps are already closed**

9. **Code-prefix regex: already dynamic, NOT a gap.** `codeGrammar.ts` compiles its regex
   from a runtime-primed prefix set (`primeCodePrefixes`,
   `web/src/console/ontology/codeGrammar.ts:75-88`); `FALLBACK_CODE_PREFIXES` (`:18-21`,
   21 entries) is only the offline/pre-fetch floor, and priming is **union**, never
   replace. Every parser (objDrag, messengerModel, composeModel) builds from here.
   *But* the feed is `typeRegistrySource.loadObjectTypeRegistry`
   (`web/src/console/ontology/typeRegistrySource.ts:63-79`), which fetches
   **`GET /api/v1/object-types`** — the **legacy** registry (§1c), whose payload is
   `{kind, code_prefix, description, status, active_count}`. The ontology engine's
   `ObjectTypeSummary` (`adapter-postgres/src/lib.rs:223-234`) has **no `code_prefix`
   field at all**. So a type registered in the ontology engine does *not* get a code
   prefix, and the file's own promise ("a type registered via the Ontology Manager
   wires its codes with NO frontend edit", `typeRegistrySource.ts:1-6`) is **false as
   written** — it is true only for the legacy registry.
   → Real gap: **the two registries must be bridged, or `ObjectTypeSummary` must carry
   `code_prefix`.**
10. **Graph legend: already dynamic, NOT a gap.** Derived from the traversal payload with
    a hash-keyed palette — `web/src/console/screens/_ontology/GraphExplorer.tsx:472-482`,
    `typeColor` at `:45-51`. New types colour themselves. (`ponytail:` comment at `:31-32`
    flags the fixed palette as a design-system follow-up.)
11. **Static `ONT_TYPES`: a third registry, and a real gap.**
    `web/src/console/modules/typeRegistry.ts:137+` hardcodes 6 types with their own
    `codePrefix` values (`VC-`, `FL-`, `HR-`, `AP-`, `TK-`, `CP-`) that match **neither**
    `FALLBACK_CODE_PREFIXES` nor the DB `object_types.code_prefix` set. `getObjectType`
    (`:369-380`) falls back to the fetched registry with `codePrefix: registered.codePrefix ?? ""`
    (`:355`). Rich per-type detail still comes from this static file.
12. **ObjectCard reads the legacy stack, not the engine.**
    `web/src/console/objectcard/useObjectCard.ts` calls `/api/v1/object-links` (`:121`),
    `/api/objects/{kind}/{id}` (`:145`), `/api/v1/lifecycles/{objectType}/{objectId}`
    (`:161`), and a raw `fetch` to `/api/audit` (`:79`, with a `ponytail:` marker at
    `:17-18` noting `GET /api/audit` is missing from openapi.yaml). Meanwhile
    `web/src/console/explore/ObjectExplorerModel.ts:6` imports `../../api/ontology`.
    So DN-0003 Slice 1's "governed object card" is split across two backends: the
    explorer is engine-backed, the card is legacy-backed. **This is the largest
    integration gap for lens A.**
13. i18n: type/action titles are server-supplied Korean strings in the seed drafts
    (`create_action` title is `"저장"`, `seed.rs:118`); chrome strings live in
    `ko.console.explore` / `screens/_ontology/koManifest.ts` (13 lines). Adding a type
    needs **no** i18n edit; adding a *screen* does.

**Net:** the frontend cost of a new type is far lower than feared (2 of 4 suspected
hardcodings are already dynamic). The cost is concentrated in the backend catalog-upgrade
hole and in the three-registry split.

---

## 4. Collision zones

- **The ontology crate is NOT codex-hot.** `git log --since=48.hours -- backend/crates/ontology`
  returns only two *merge* commits (`a4ae5ab5` merge(org): CAP-ORG-CONSOLE, `8a99f4c9`
  merge(recruiting)) — both touch `Cargo.lock`/BUCK/`app/src/lib.rs`, not ontology sources.
  Last real content commits are **2026-07-23**: the ORU receipt series
  (`e117d048`, `ffa8cd88`, `6cd96f7b`, `874f7c66`, `4e3df210`) and the policy-filter
  series (`bae882bf`, `f45b6aa1`, `85b7f397`, `d27fd09e`, `a5374834`, `05515d46`).
  **A wave-4 lane can own `crates/ontology/**` cleanly.**
- **`backend/app/src/lib.rs` IS hot** — every wave-2/3 module merge lands router mounts
  there (`d165dab2` mounted 9 bodies; `55b81203`, `331d695c`, `4331c5f1` add routers).
  The projected-dispatch registry lives at `:2458-2523` and its install at `:3199`.
  Expect conflicts; keep the diff to appended `register(...)` lines + a new handler fn,
  and plain-merge before push (rebase gets classifier-blocked on this spine).
- **`backend/app/src/objects.rs`** (2632 lines, legacy registry + `RESOLVABLE_KIND_AUTH`)
  is the shared surface for any registry-bridging work. Its own header documents two
  prior security regressions from editing it carelessly, and adding a kind to
  `RESOLVABLE_KIND_AUTH` **retroactively makes pre-existing `object_links` of that kind
  resolvable with no backfill re-check** (`objects.rs:117-120`). Treat as
  security-reviewed, single-owner.
- **Migration numbers**: highest ontology-adjacent is `0198` (renumbered in `a4ae5ab5`
  from 0189). Reserve the next free integer immediately before push, never at author time.
- **`backend/openapi/openapi.yaml`** was corrupted and reverted 48h ago (`9bb877c6`
  revert: "mechanical fragment splice corrupted it") after `ee277e16`. Any lane touching
  it must hand-edit and run the three drift gates, not splice.
- `web/src/api/ontology.ts` — `typeRegistrySource.ts:20-24` notes `api/ontology.ts` is
  "under concurrent edit by the serial-wire lane". Verify before claiming it.

---

## 5. Ranked recommendations for wave-4 lens-A lane chartering

1. **L-A1 · Catalog upgrade path** (blocking, backend-only, `crates/ontology` + one
   migration). Additive install for an already-seeded tenant. Without it, no new seeded
   type ships to any live environment. Fully disjoint, no `app/src/lib.rs` edit.
2. **L-A2 · Schema publish route** (`POST /ontology/object-types/{key}/publish`) —
   carries four-eyes ref + If-Match, runs on `command_pool`, exercises the existing SQL
   auto-attach. Closes the no-code loop. Costs openapi + 3 client regens. Alternative to
   L-A1 for runtime-authored types; **do both, they serve different tenants.**
3. **L-A3 · First real projected action** — one seeded `projected_usecase` action for
   `equipment` targeting the *already-written* `registry.update_equipment` handler.
   Smallest possible end-to-end proof of §18. Depends on L-A1 or L-A2.
4. **L-A4 · Projected command receipts** (R5/R6) — the genuine design work; needs an
   explicit decision on receipt-vs-domain-tx atomicity. Charter with a design gate, not
   as a mechanical port.
5. **L-A5 · Registry unification** — the three-registry split (§1c, gaps 9/11/12) is the
   biggest structural debt and the thing that makes DN-0003 Slice 1's "one governed
   object card" untrue. Scope it as a *decision* lane first (which registry wins?), not
   an implementation lane. Touches `app/src/objects.rs` — security-reviewed, serialize it.
6. **Cheap wins, foldable into any lane:** validate `backing_table` against
   `allowlisted_projected_table` at draft time (`adapter-postgres/src/lib.rs:1080`);
   delete the stale "Status: design, no code" header on
   `be-ontology-engine-arch.md`; fix the stale `create_action` doc comment at
   `seed.rs:100-104`; extend `resolve_by_code` to projected types (R8).
