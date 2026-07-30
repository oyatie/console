# X1 and X2 — the two silent-empty traps, run

Both experiments from `ecosystem-plan-DRAFT.md` §8 Phase 6. Both were CONFIRMED by code reading
only. Both are now **CONFIRMED by execution**.

Run in `console-lanes/lane-5` at `5330914c2` (`origin/main`, which includes #525 and migration
`0205_ont_policy_api_attach_writer.sql`). Migrations in this tree end at **0205** — lane-1's 0206 is
not here, so nothing below depends on it.

Every read runs as the genuine `console_rt` role, observed NOSUPERUSER at runtime in both
experiments. `console_rt` is created `LOGIN NOSUPERUSER NOBYPASSRLS NOINHERIT`
(`ops/postgres-reconcile-topology.sh:303`, re-asserted at `:307`).

---

## X1 — an authored relationship produces no edge

**Verdict: CONFIRMED.** A relationship declared only as a link type writes zero `ont_links` rows.
The instance write returns success, the referring attribute keeps the referent, and the edge is
never created.

### Evidence verified, with my own citations

Every citation I was given holds. Paths below are relative to
`backend/crates/ontology/adapter-postgres/src/`; line numbers are mine, read in this tree.

| Claim | Where | What it says |
|---|---|---|
| the loop iterates **properties**, and skips any property without `config.link` | `instances.rs:888-891` | `for prop in props { let Some(link) = prop.config.get("link") else { continue; };` |
| `stable_key` and `to_type` come from the **property's** config, not the link type's | `instances.rs:895-898` | `link.get("stable_key")`, `link.get("to_type")` |
| the link type is resolved by `(object_type_id, stable_key)`, and `to_object_type_id` is in **neither** the SELECT nor the WHERE | `instances.rs:906-911` | `SELECT id FROM ont_link_types WHERE object_type_id = $1 AND stable_key = $2` |
| the target check compares against `to_type` **from the property config**, never the link type's declared target | `instances.rs:954-981`, decisive at `:973` | `Some(actual) if actual != to_type` |
| a type with no `config.link` property returns having touched `ont_links` zero times | `instances.rs:988-990` | `if link_type_ids.is_empty() { return Ok(()); }` |
| `link_type_ids` is pushed **only** inside the `config.link` loop | `instances.rs:920` | one push site, unreachable without `config.link` |

Two additions of my own, both stronger than the reading the plan records:

1. **`to_object_type_id` appears zero times in the entire 2,004-line write module.**
   `grep -c to_object_type_id instances.rs` → `0`. The field is not merely unread by this function;
   nothing in the instance write path can see it.

2. **The `validate_draft` guard §0.12 proposes does not exist, so the trap is armed today.**
   `adapter-postgres/src/lib.rs:1142-1151` is the whole link-type validation, and it checks
   duplicate `stable_key` and nothing else. This is the plan's `link_type_alone_is_rejected` probe
   (§7) observed RED, as the plan requires before the guard lands.

### One correction to the claim as stated

"Edges are written only when a property carries `config.link`" is true of every **reachable** path,
but not of the crate's API. `PgInstanceStore::create_link` (`instances.rs:291-339`) writes an
`ont_links` row directly from a bare `link_type_id`, with no property involved — an audited write
at `:319`.

It changes nothing, because it is unreachable: `create_link` has **zero non-test callers**
(`ONTOLOGY_ROUTE_PATHS`, `ontology/rest/src/lib.rs:213-228`, is exactly 14 paths and none creates a
link; the four call sites are all in `tests/`, and `rest/tests/company_conformance.rs:304` already
records this). Worth stating precisely anyway: the reference documents must say *no reachable path*,
because a reader who greps for `INSERT INTO ont_links` finds two sites and only one of them is the
property mechanism.

### Control observation — RED-proving, run first

The probe asserts the working path returns 1 **before** the bare path is allowed to return 0. A
count query that always returned 0 would "confirm" X1 while measuring nothing.

```
$ bash docs/ideas/experiments/x1/run.sh

X1 CONTROL 0  current_user=console_rt rolsuper=false
X1 CONTROL 1  edges via property config.link = 1
```

The probe is `docs/ideas/experiments/x1/probe.rs`, and `run.sh` borrows it into the crate's `tests/`
directory for the length of one run and deletes it on every exit path. It is deliberately **not** a
committed CI target: it asserts today's defect, so the guard §0.12 proposes would turn it red, and a
red probe is how a guard gets deleted.

`CONTROL 0` kills the RLS-bypass explanation; `CONTROL 1` proves
`SELECT COUNT(*) FROM ont_links WHERE from_instance_id = $1` can see an edge in this fixture.

### Both halves

The two halves differ in **exactly one field**. Both declare the same link type with
`to_object_type_id` resolved to the real target type; both carry a `reference` property holding the
referent's instance id. Only the property's `config` differs — `{"link": {...}}` versus `{}`.

```
$ bash docs/ideas/experiments/x1/run.sh

X1 both drafts accepted: linked=ObjectTypeId(14f9fb35-750a-4d5e-8055-d2e1cf92fb70)
                         bare=ObjectTypeId(4c158c83-8742-48d4-9bbe-0b3c8c81d6ef)
X1 ont_link_types.to_object_type_id  linked=Some(9aec4d2c-60b7-403d-a34a-f6de18f1f4c7)
                                     bare=Some(9aec4d2c-60b7-403d-a34a-f6de18f1f4c7)
                                     target_type=9aec4d2c-60b7-403d-a34a-f6de18f1f4c7
X1 CONTROL 1  edges via property config.link = 1
X1 MEASURED   edges via link type alone   = 0
X1 the row exists and reports success: title="via-link-type-only"
                                       target_ref="06630c86-40ae-4ec1-9ed7-55e9cc739537"
test x1_a_link_type_alone_writes_no_edge ... ok
```

Read the third line carefully: **the bare half's link type persisted with the target resolved to the
same uuid as the working half's.** The declaration is present, correct, and readable by the tenant.
The registry believes the relationship exists. `ont_links` has nothing in it.

The last line is the trap's shape. The write succeeded, the instance is there, and `target_ref`
still holds the referent — so a form that shows the raw attribute looks correct. Only the graph is
empty, and `traverse` reads `ont_links` (`instances.rs:579`, BFS over `FROM ont_links`), so every
traversal of that relationship returns nothing, forever, with no error anywhere.

### Consequence for the plan

**Every relationship in the entity model must ride a property's `config.link`. This is a mechanical
rule, and `ecosystem-PORTING.md` must state it as one, not as advice.** Concretely:

- A relationship is authored as a `reference` property whose `config` is
  `{"link": {"stable_key": "<link type key>", "to_type": "<target stable_key>"}}`, plus a link type
  of that same `stable_key` on the same object type. Both, always.
- `to_object_type_id` on the link type is **decoration**: it is never read on write, and the target
  is enforced from the property's `to_type` against the referent's actual `stable_key`
  (`instances.rs:973`). Setting it correctly buys nothing; setting it wrongly costs nothing. Say so,
  or an implementer will trust it.
- §0.12's proposed `validate_draft` guard is confirmed **absent** and confirmed **necessary**. A
  canvas that draws an arrow between two boxes — the obvious no-code gesture — produces a
  permanently empty relationship with no error at any layer. One check in
  `validate_draft` (`lib.rs:1142-1151`) fails it closed at authoring time.

---

## X2 — a published type lists empty until a policy is attached

**Verdict: CONFIRMED.** A published type with instances lists `200 OK []` with no object policy
attached, and lists its rows once an enforced permit is attached over HTTP.

### Evidence verified, with my own citations

The residual mechanism is where I was told it is. `platform/authz/src/cedar_pbac/residual.rs:200-203`:

```rust
// Deny-by-omission: no applicable permit ⇒ nothing is visible.
if permits.is_empty() {
    return ResidualFilter::deny_all();
}
```

and `deny_all()` is `where_sql: "FALSE"` (`residual.rs:145-150`).

The chain from an unpoliced published type to `[]` is unconditional at every step:

1. `ontology/rest/src/lib.rs:559` — the list route calls `object_view_policies` before anything else.
2. `rest/src/lib.rs:942-951` — `applicable_object_policies` filters the loaded blocks. **No
   attachment rows ⇒ no blocks ⇒ empty `Vec<ObjectPolicy>`.** There is no default and no fallback.
3. `rest/src/lib.rs:565` — the route then calls `list_instances_filtered` unconditionally. There is
   no "unpoliced" branch that skips filtering.
4. `adapter-postgres/src/instances.rs:491` — `visible_instances` calls `lower(...)` unconditionally,
   whatever `policies` holds.
5. `residual.rs:201-203` — empty permits ⇒ `deny_all()` ⇒ `"FALSE"`.
6. `instances.rs:515` — that string is interpolated as `AND ({residual})`, so the list SQL becomes
   `... WHERE i.object_type_id = $1 AND (...) AND (FALSE)`.

The one nuance worth recording: an *unresolvable* type id is a 404, not an empty policy set — step 1
resolves the version through `object_type_version`, which raises `not_found`
(`adapter-postgres/src/lib.rs:597`) rather than returning no policies. So "type not found" and "type
not policed" are distinguishable at the route. What is not distinguishable is "no rows" from
"no policy" — both are `200 OK []`.

### Control observation

```
X2 CONTROL 0  current_user=console_rt rolsuper=false
```

Read on the same pool the router's stores are built on (`console_platform_test_support::runtime_role_pool`,
`test-support/src/lib.rs:17-30`, which is `SET ROLE console_rt` in `after_connect`). Not a superuser,
so the empty list is not an RLS artifact — and HALF 2 below is the proof the query can return a row
at all: same route, same principal, same connection, one attachment's difference.

### Both halves

X2 needed no new probe: the experiment it asks for is already a shipped test over HTTP —
`ontology/rest/tests/object_policy_attach_as_runtime_role.rs:56`,
`an_attached_permit_is_the_only_thing_that_makes_instances_visible`, landed by #525. What it does not
do is print. `docs/ideas/experiments/x2/instrumentation.patch` adds four `println!`s and changes no
assertion; `run.sh` applies it, runs, and reverses it on every exit path.

```
$ bash docs/ideas/experiments/x2/run.sh

X2 CONTROL 0  current_user=console_rt rolsuper=false
X2 HALF 1     no policy attached -> 200 OK []
X2 HALF 2     policy attached (201 Created) -> 200 OK titles=Some([String("visible-to-owner")])
X2 HALF 1 bis  second unpoliced type in the SAME request -> 200 OK []
test an_attached_permit_is_the_only_thing_that_makes_instances_visible ... ok
```

`HALF 1 bis` matters: a second published type, same org, same request, no attachment, `[]` — so
HALF 1's emptiness cannot be a mis-seeded fixture, because the fixture demonstrably serves rows for
the type that has a policy.

**Attach path used:** `POST /api/v1/ontology/object-types/{stable_key}/policies` —
`OBJECT_TYPE_POLICIES_PATH` (`ontology/rest/src/lib.rs:202`), registered in `ONTOLOGY_ROUTE_PATHS`
(`:213-228`), backed by the audited definer in `0205_ont_policy_api_attach_writer.sql`. This is
#525's writer, not lane-1's 0206, which is not in this tree.

The mechanism was independently confirmed at the adapter level by the shipped
`adapter-postgres/tests/instances_residual_filter_as_runtime_role.rs` (2 tests, both pass here),
whose `:228` asserts an unfiltered list sees all three rows and whose `:244-248` asserts
`list_instances_filtered(.., &[])` is empty — the same control/measurement pair, one layer down.

### Consequence for the plan

**No authored entity is usable until policy attachment works.** A published Tier N type with rows in
it is indistinguishable from an empty one at every read surface — and it is worse than the list:
`rest/tests/object_policy_attach_as_runtime_role.rs:133-141` shows a row the list hides is `404` by
id, deliberately (a 403 would be an existence oracle). So an unpoliced entity is not "visible but
unfiltered", it is **absent**.

This is exactly what lane-1's 0206 slice is for. Two things follow for the plan text:

- Every Tier N entity must ship its object policy attached **in the same change that publishes it**.
  A publish without an attach is not a partial success, it is a no-op with a green status code.
- `no-code-ontology.md` remains stale as §0.13 records: publish and policy-attach are both
  HTTP-reachable today (`OBJECT_TYPE_LIFECYCLE_PATH` at `rest/src/lib.rs:201`,
  `OBJECT_TYPE_POLICIES_PATH` at `:202`, both in `ONTOLOGY_ROUTE_PATHS`). I attached one over HTTP,
  above.

---

## What the two traps share

Both fail with a success status and an empty result, and in both cases the authoring surface reports
that the configuration landed — X1's link type is readable with its target resolved, X2's type is
published and its rows are really in `ont_instances`. Neither produces a log line, an error, or any
observable difference from correctly-configured-but-empty.

§8's `link_type_alone_is_rejected` and `tier_n_type_lists_nonempty` are both worth having, and the
plan's note at §9 that they "deserve one gate between them" is right for a reason the runs make
concrete: a publish that produces a queryable, traversable entity requires **three** things
together — a property carrying `config.link` for each relationship, a link type of that
`stable_key`, and an attached enforced `view` policy. Any one missing yields the same silent empty.
