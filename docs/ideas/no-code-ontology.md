# No-code ontology

> Status: **idea one-pager, pending approval.** Written 2026-07-28 against the tree at
> `docs/no-code-ontology`; revised 2026-07-29 after two adversarial reviews, one of which returned
> FAIL. Every claim below was verified by reading the cited code in this checkout, not from a header
> comment or a prior document. Where something was not verified, the sentence says so. The revision
> made the first slice bigger, not smaller — under-scoping was what the review caught.

## Problem Statement

**How might we** let an admin define an object type — its fields, its relationships, its actions and
its policy — from a canvas in the console, and have instances of it exist, be governed and be read
back, without anyone writing Rust, SQL or a migration?

## Where we actually are

The engine is much further along than "we need to build a no-code ontology" suggests. Nearly the
whole loop is already reachable over HTTP; exactly one write verb and one read path are not. This
table exists so nobody re-derives it.

| Layer | Verified state | Evidence |
|---|---|---|
| **Any of this, in a deployed environment** | **503.** Every ontology WRITE runs on the command pool; `command_pool()` is `None` unless `ONTOLOGY_COMMAND_DATABASE_URL` is set, and that variable is supplied by a kustomize component that **`prod`, `on-prem` and `oci-guest` do not reference at all** — only the two experimental `pr-473-expand-*` overlays do. CI *does* set it, so the whole authoring surface is green on every PR and dead where it ships. This is not about the new route: `create_object_type` and `stage_revision` are 503 in production today. | overlays `deploy/apps/console/overlays/{prod,on-prem,oci-guest}/kustomization.yaml` (no `components:` block); component `deploy/apps/console/components/governed-command-database/kustomization.yaml:120`; fallback `backend/app/src/lib.rs:2925-2930`; 503 at `ontology/rest/src/lib.rs:1786-1790`; CI `.github/workflows/ci.yml:573` |
| **Create a type** (props, links, actions, analytics in one payload) | **Reachable over HTTP.** `POST` takes the full `CreateObjectTypeDraft` verbatim off the wire. | `backend/crates/ontology/rest/src/lib.rs:194`, route `:226-229`, handler `:328`; DTO `backend/crates/ontology/adapter-postgres/src/lib.rs:175-193` |
| **Revise a type** | **Reachable over HTTP, and append-only for children.** `PUT` with a mandatory strong `If-Match`. If a draft exists it updates *that row's own scalars* in place (title, backing kind, primary key) and re-submits the full child snapshot; if none exists it opens `max(schema_version)+1`; a `review_pending` version rejects the write. "In place" stops at the parent row: `insert_children(…, p_allow_existing => TRUE)` rejects the write unless the submitted snapshot still contains every existing child byte-for-byte after canonicalization — one error per child kind, `property_`/`link_`/`action_`/`analytic_snapshot_conflict`. | `rest/src/lib.rs:370`, CAS parser `:279-326`; SQL branches `backend/crates/platform/db/migrations/0165_ontology_object_type_key_revisions.sql:866-903` (`review_pending_immutable` `:875`, scalar `UPDATE` `:877-884`, snapshot equality `:508-530`, raise `:529`) |
| **Publish a type** (`draft → review_pending → published`) | **NOT reachable. This is the one missing route.** `ONTOLOGY_ROUTE_PATHS` is exactly 12 paths and none touches the schema FSM; `PgOntologyStore::transition_lifecycle` exists and works but every caller in the tree is a test. | `rest/src/lib.rs:207-220`; store `adapter-postgres/src/lib.rs:511`; caller grep hits only `*/tests/*` |
| **Four-eyes approval for publish** | **Reachable over HTTP.** Both halves are live `POST` routes, but they are not symmetric: `/approvals` takes a client-supplied `kind`, `target_ref` and `payload_summary`; `/approvals/decide` takes only `request_ref`, `kind`, `requested_by` and `decision` — its `target_ref` is `None` on the wire and is sourced from the pending request row, so an approver cannot redirect what they approve. | `backend/crates/governance/rest/src/lib.rs:46-47`, `:64-65`; create `:182-199`, decide `:204-227` (`target_ref: None` at `:219`) |
| **Instantiate** | **Reachable over HTTP, but only through an action, and only for `backing_kind = 'instance'`.** There is no `POST /instances` — `INSTANCES_PATH` is GET-only. Publish auto-attaches a generic `create` action derived from the property set, which is what makes instantiation possible with zero Rust — but the guard is `p_to_state = 'published' AND v_backing_kind = 'instance' AND NOT EXISTS (… dispatch = 'instance_revision')`. A `projected` type gets no auto-`create` and therefore no zero-Rust instantiation path at all. | `rest/src/lib.rs:235`, `:243`; auto-attach `0165_…sql:1024-1042`, guard `:1024-1029` |
| **Read one instance** | **Reachable, unfiltered.** `GET /instances/{id}` authorizes org-wide and then reads; no residual, no object policy. | `rest/src/lib.rs:526-540` |
| **List instances** | **Reachable and permanently empty.** `list_instances` unconditionally lowers object policies to a SQL residual; `applicable_object_policies` returns empty with nothing attached, and `residual::lower` returns `deny_all()` on an empty permit set. A freshly published no-code type lists `[]` forever. | `rest/src/lib.rs:404-460`, filter `:466-477`; `backend/crates/platform/authz/src/cedar_pbac/residual.rs:201-203` |
| **Attach a policy so the list works** | **NOT reachable, and it is a DB privilege wall, not a missing handler.** `cedar_policy_catalog_entries` has exactly two privilege statements across every migration: `GRANT SELECT … TO console_rt` and `REVOKE INSERT, UPDATE, DELETE … FROM console_rt`. So the true claim is narrower than "no grant": **no INSERT/UPDATE/DELETE grant exists for any application role**, and every writer in the tree is a test or `test-support`. Attachment is downstream of it: a trigger rejects any attachment without a same-org catalog row. | `0150_create_cedar_policy_staging.sql:117-118` (a grep for the table name across `migrations/` returns no other privilege statement); `0170_harden_object_policy_attachment_and_blockers.sql:5-27`; writers `backend/crates/platform/test-support/src/lib.rs:135`, `:150` |
| **Relationships** | **Real edges, but only from a property's `config.link`.** `LinkTypeInput.to_object_type_id` *is* read — selected, mapped onto the DTO, returned by `GET /object-types/{key}`, and resolved and written by `install_builtin_catalog`. What never consults it is the **edge writer**: `sync_property_links_tx` pulls `stable_key` and `to_type` out of the property's `config.link` and validates the referent's type against that stable key, never against `object_type_id`. | writer `adapter-postgres/src/instances.rs:849-1010` (config read `:870-875`, type check `:948-951`), called from `:698` and `:811`; the field's real readers `adapter-postgres/src/lib.rs:657`, `:1028`, `:1034`, `seed.rs:174`, `:963` |
| **Analytics** | **Authorable and inert.** `formula` is stored, shape-checked and returned; a grep across `backend/crates` non-test Rust finds no evaluator — only DTO, read and validation sites. | `adapter-postgres/src/lib.rs:169`, `:691`, `:1161` |
| **Change a field in place** (rename / retype / drop / un-require) | **Structurally impossible through any shipped path — but not for the reason the grants suggest.** Every ontology write runs inside a `SECURITY DEFINER` function in `ontology_api`, so `REVOKE … FROM console_rt` does no work *inside* one. The binding guard is `insert_children(p_allow_existing => TRUE)`: staging a revision must resubmit the whole child snapshot, and any existing child that does not survive canonicalization byte-for-byte is rejected, with a DISTINCT error per child kind — `property_snapshot_conflict` (`:508-530`, raise `:529`), `link_snapshot_conflict` (`:532-553`), `action_snapshot_conflict` (`:556-582`), `analytic_snapshot_conflict` (`:585-602`), all four mapped separately at `adapter-postgres/src/lib.rs:90-97`. The definer role is itself granted only `SELECT, INSERT` on the four child tables, which is a second and weaker line — a one-line `GRANT UPDATE` in a later migration removes it, while the snapshot check lives in the function body. | definer `0165_…sql:488`, `:944`; snapshot equality `:508-530`, raise `:529`; grants `:214-215`, `:221-229` |
| **Capability tiering** (admin vs business user) | **Absent, and it is a compile-time enum, not configuration.** One org-wide `authorize_org_wide(&principal, Action::new(Feature::RoleManage))` gates all 12 ontology routes — read, author and execute alike. `Feature` is a closed Rust enum and every variant is authorized by `const fn matrix_row(self) -> [PermissionLevel; 6]` over six fixed roles. Adding a tier is a code change and a recompile. | gate `rest/src/lib.rs:1649-1656`, call `:1654`; enum `backend/crates/platform/authz/src/lib.rs:109`, matrix `:573-576`, `RoleManage` row `:605` |
| **Any UI** | **No frontend application in this repo.** `git ls-files '*.jsx' '*.tsx' '*.vue' '*.svelte'` returns exactly one path — `docs/design/oyatie-console/ios-frame.jsx`, a design reference, not an app — and no root directory builds one; the `decommission-web` and `strip-frontend-ci` lanes are active. | `git ls-files` at repo root |

Two traps worth naming, because both have already cost a plan:

`INSTANCE_LIFECYCLE_PATH` (`rest/src/lib.rs:201`, handler `:1472`) reads like the publish route and is
not. It transitions an **instance** through Draft/Active/Locked/Archived/Disposed. The **schema** FSM
is a different enum — `Draft/ReviewPending/Published/Superseded/Retired`,
`backend/crates/ontology/domain/src/lib.rs:83-89` — and has no HTTP surface at all.

"Add one route and the UI can publish" under-scopes the client. Publishing is four ordered calls:
transition `draft → review_pending`; `POST /governance/approvals` carrying
`payload_summary.key_revision` equal to the revision **after** that bump, with `requested_by` set to
the actor who will publish; a different person's `/approvals/decide`; then transition
`review_pending → published`. A `key_revision` read before the review transition fails with
`ontology_write.publish_approval_required`: the lookup joins on
`(gr.payload_summary->>'key_revision')::BIGINT = v_revision` and raises when it finds nothing
(`0165_…sql:1003`; `:1002` is the `decision = 'approved'` clause, raise `:1009`).

## Recommended Direction

**Ship the missing verb, and prove the loop at the HTTP layer — not at the store layer, where it is
already green.** The engine's ability to go from nothing to a living instance without per-type Rust
is not in question: `backend/crates/ontology/rest/tests/publish_auto_create_action_as_runtime_role.rs:192-298`
already drives create → review → approve → publish → execute create against the real `console_rt`
and `console_ontology_cmd` roles and asserts exactly one instance. What is unproven is that a
*browser* can drive it. That is the whole of the gap, and it is one handler.

**Policy is not "before" or "after" the loop — one narrow piece of it is inside, and the rest is
genuinely after.** The list endpoint fail-closes to `[]` without an enforced Cedar permit, so any
slice that claims "the type now holds instances you can see" must also ship policy attachment. That
is a much bigger slice than one route: it requires a promotion path (`cedar_policy_drafts`
terminates at `approved_for_promotion` and nothing turns that into a catalog row), the revoked
`INSERT` grant re-granted with an owner for who may promote, the `0169` normalization blocker queue
drained (`0169_add_normalized_catalog_policy_blocks.sql:44-48` is still `NOT VALID`, and no later
migration validates it), and only then the attachment handler that `0170` gates. Slice 1 sidesteps
all of it by asserting through `GET /instances/{id}`, which authorizes and reads with no residual
(`rest/src/lib.rs:526-540`) — exactly what the existing conformance `RestDriver` already does, and
for exactly this reason (`backend/crates/ontology/rest/tests/company_conformance/rest.rs:194-206`).
The list work then becomes an honestly-scoped slice 2 instead of a surprise discovered mid-slice-1.

**The tiering the owner asked for — admins author freely, business users edit fields on types they
own — is already half-built by the storage model, and should be delivered as a capability split, not
as a new substrate.** Because a staged draft is append-only for children — no submitted snapshot may
drop or alter an existing one (`0165_…sql:508-530`) — *every* field change is structurally a proposal
that an admin publishes under four-eyes. Stage-vs-publish is therefore the natural tier boundary. It
is also a bigger change than "split one capability" sounds: `Feature` is a closed compile-time enum
with a `const fn` matrix over six fixed roles (`authz/src/lib.rs:109`, `:573-576`), so a new tier is a
recompile and a new matrix row, not a config change. It is still smaller and more honest than
building the three things that do not exist (an ObjectType Cedar resource entity, schema-mutation
actions in the fixed `AUTHORING_ACTIONS` vocabulary, and a per-type ownership column). Per-type
ownership can come later if stage-vs-publish proves too coarse; it should not be assumed necessary up
front.

## First slice

A complete, shippable increment: **the no-code loop closes over HTTP, end to end, with a conformance
test that a browser could replay verbatim.** Nothing in it is scaffolding to be redone.

It is seven files plus the deployment enablement, not one: `ontology/rest/src/lib.rs` (handler + route const),
`backend/openapi/openapi.yaml`, `scripts/check-ontology-write-precondition.test.mjs`, the new test,
`ontology/rest/BUCK`, `tools/buck/BUCK`, `.github/workflows/ci.yml`. That is the honest size, and
part 3 explains why the last three are not optional.

**1. `POST /api/v1/ontology/object-types/{key}/lifecycle`.** Body `{"to": "<state>"}`. `?version=N`
is **required**, not optional: the CAS token is keyed per `(org_id, stable_key)`
(`0165_…sql:92-102`) and cannot disambiguate which version to transition, while
`PgOntologyStore::transition_lifecycle` takes an `ObjectTypeId` — a version UUID
(`adapter-postgres/src/lib.rs:511-518`). Resolve `key` + `version` through the existing
`get_object_type` path. Requires `If-Match`; reuse `required_object_type_write_precondition`
(`rest/src/lib.rs:279`) verbatim rather than inventing a second parser. Call the existing store
method unchanged. Gate on `authorize_ontology` — no second gate invented in this slice.

The route passes `to` through and lets the FSM adjudicate, because the FSM is the only thing that
knows the current state. **That is six legal edges, not two** (`0165_…sql:987-1020`):
`draft→review_pending`, `review_pending→draft`, `review_pending→published` (the one that consumes an
approval, `:989-1013`), `published→superseded`, `published→retired`, `superseded→retired`. Exactly
one edge is specially rejected — `draft→published` raises `ontology_write.review_required` (`:988`) —
and every other combination raises `illegal_lifecycle_transition` (`:1021`). So the route must
authorize and test all six: three are the publish path, `review_pending→draft` is the withdraw a UI
needs, and `published→retired` is an **irreversible** off-switch that this one handler makes
reachable over HTTP for the first time, under nothing but the org-wide `RoleManage` gate and the CAS
token. That last edge is a real cost of the slice and is named in Not Doing rather than hidden.

**2. Route inventory and openapi.** Add the path to `ONTOLOGY_ROUTE_PATHS` and to
`backend/openapi/openapi.yaml`, or `openapi_yaml_covers_configured_route_inventory`
(`backend/app/tests/openapi_drift.rs:351`) fails the build. In the same pass, give
`CreateObjectTypeDraft` real child schemas: `properties`, `links`, `actions` and `analytics` are all
declared `type: array, items: {type: object, additionalProperties: true}`
(`backend/openapi/openapi.yaml:31238-31257`, under `CreateObjectTypeDraft` at `:31215`), so every
generated client hands a UI
`Record<string, unknown>[]` — no field-level contract for the very editor this whole idea is about.
This is where the vocabulary belongs; the drift gate already keeps it honest, so it needs no second
serving surface. Also extend `scripts/check-ontology-write-precondition.test.mjs`: its
`stageOperation` helper is hardcoded to the `{key}` path's `put:` block (`:14-21`), so the new
route's `If-Match`/412/428 contract is covered by nothing until that file learns the second path.

**3. Wire the test into the build, or it never runs.** A `.rs` file dropped under `tests/` is
invisible to this repo's CI — Buck2 does not glob. The slice needs, in the same pass:
a `rust_test` target in `backend/crates/ontology/rest/BUCK` whose `mapped_srcs` names **every** file
the test crate reads (the neighbouring `publish_auto_create_action_as_runtime_role` target,
`:297-338`, with the ten hand-listed `mapped_srcs` at `:299-301` and re-exports the migrations tree as `external`); an `sh_test`
wrapper in `tools/buck/BUCK` around `run_test_with_postgres_env.sh`, because the test needs a
database (`tools/buck/BUCK:148-154` is the shape); and a step in `.github/workflows/ci.yml` running
`tools/buck/test_needs_postgres.sh --num-threads=1 //tools/buck:<target>` (the `company-conformance`
job, `:229`, step `:271-274`, is the template). **Four files, not one.** Skip any of them and the
test compiles locally and is never executed on a PR.

**4. Map the 500 that this route makes reachable.** The auto-attach guard keys on
`dispatch = 'instance_revision'`, not on the stable key (`0165_…sql:1024-1029`). A client-authored
action named `create` with `dispatch = 'projected_usecase'` therefore does not suppress the
auto-`INSERT` at `:1035-1041`, which violates `UNIQUE (object_type_id, stable_key)`. A bare 23505 on
that constraint is not matched by `ontology_database_kernel_error`
(`adapter-postgres/src/lib.rs:62-107` maps only the named `ontology_write.*` messages, plus one
constraint-name special case), so it falls through to an unmapped error. **Traced, not executed** — no database in this lane. The stable key is
client-supplied, so this is a trust boundary and not lazy-away-able; it becomes reachable the moment
the route ships and belongs in the same slice.

**5. Pin the list's fail-closed behavior with a test, and do not "fix" it here.** The tempting move
is to re-key `list_instances` from version id to `stable_key`, since publishing v2 currently hides
every v1 instance (`list_object_types` is `DISTINCT ON (o.stable_key) … published DESC`,
`adapter-postgres/src/lib.rs:580-589`; the instance query then filters on that one id). Do not.
`ont_object_policies.object_type_id` is bound the same way
(`backend/crates/platform/authz-rest/src/store.rs:440`), and `declared` is built from the head
version (`rest/src/lib.rs:426-443`). Re-keying instances alone would list v1 and v2 rows against a
policy attached to one version and validated against one version's attribute set — a **widening of a
security filter**. Both bugs are on the same path and their fix is one coherent change in slice 2.
Today the version-orphaning turns `[]` into `[]`, so it is not urgent.

**The test.** A new file, `backend/crates/ontology/rest/tests/nocode_loop_over_http_as_runtime_role.rs`.
It must be a new file. `company_conformance.rs` and `company_conformance/{harness,rest,store,fixtures}.rs`
are owned outside the lanes; a lane owns exactly one file under `company_conformance/fixtures/`, and
those five slots are the five company types (`company_conformance.rs:14-17`). The `declare` seam is
not in `rest.rs` — it is one `pub async fn declare(h: &Harness)` per type in
`company_conformance/fixtures/{company,org_unit,job_position,employment,pay_run}.rs`, aggregated by
the outside-owned `fixtures.rs:65-70`. A no-code loop test is not a sixth company type, so there is
no slot for it and negotiating an ownership exception costs more than a fresh file.

The test drives only HTTP, against the real `console_rt` and `console_ontology_cmd` roles, with two
super-admin principals (both halves gate on org-wide `Feature::RoleManage` — `rest/src/lib.rs:1649-1656`
and the governance equivalent). The command pool is not optional dressing: `ontology_api.*` `EXECUTE`
is revoked from `console_rt` and granted only to `console_ontology_cmd` (`0165_…sql:1232-1236`), and
`transition_lifecycle` opens its transaction on `self.command_pool()?`
(`adapter-postgres/src/lib.rs:524`), which returns `CommandUnavailable` when that pool is unset
(`:364-367`). A harness wired with only `console_rt` fails the new route on its first call.

Happy path: `POST /object-types` → `PUT` a revision → `POST …/lifecycle {"to":"review_pending"}` →
`POST /governance/approvals` with the post-bump `key_revision` → a second principal's
`/approvals/decide` → `POST …/lifecycle {"to":"published"}` → `POST /actions/create/execute` →
assert through **`GET /instances/{id}`**, never the list.

Because the route exposes six edges it takes six positives, not two. Beyond the publish path:
`review_pending→draft` (withdraw, and the revision it returns to is still stageable);
`published→superseded` and `published→retired` on a second published version, asserting that
`retired` is terminal — no edge leaves it — and that neither consumed an approval.

Negatives, each pinning a real guard: `draft→published` returns the `review_required` failure; any
other pair (say `draft→superseded`) returns `illegal_lifecycle_transition`; a missing `If-Match`
returns 428 and a stale one 412; an approval whose `key_revision` predates the review bump returns
the `publish_approval_required` failure; and a projected-dispatch action named `create` returns a
mapped error rather than a 500. One further assertion documents the known dead end: after publish,
`GET /instances?type=<id>` returns `[]` — cause `residual.rs:201-203`, fix in slice 2.

## Key assumptions to validate

- [ ] **The 23505 collision on a projected `create` is real.** Traced from `0165_…sql:1024-1041` and
      the error map at `adapter-postgres/src/lib.rs:88-105`, never executed. Test: the negative case
      in the slice-1 file; if it does not reproduce, delete the mapping rather than keep dead code.
- [ ] **No tenant has ever had an enforced catalog row.** Established only from code and migrations —
      production code has zero writers. Test: `SELECT count(*) FROM cedar_policy_catalog_entries
      WHERE status IN ('enforced','shadow')` on the live cluster, read-only. If non-zero, someone
      wrote rows out of band and the promotion design has a precedent to reconcile.
- [ ] **`0169`'s blocker queue is empty in production.** The constraint is still `NOT VALID`
      (`0169_…sql:44-48`) and nothing validates it. Test: `SELECT count(*) FROM
      cedar_policy_catalog_normalization_blockers` before scheduling any slice-2 work.
- [ ] **Stage-vs-publish is a sufficient tier boundary.** Asserted from the INSERT-only grants, not
      from a user. Test: take one real business-user field change end to end — can they express it as
      a staged revision without needing to restructure a relationship? If they cannot, per-type
      ownership becomes necessary and slice 2 grows.
- [ ] **`?version=N` is what a UI actually has in hand at publish time.** Assumed. Test: write the
      four-call publish sequence as a client script against the slice-1 route; if the client must
      re-`GET` to learn the version between every call, the route should take the key's head instead.
      Note `key_write_etag` **is** a serialized field on `ObjectTypeSummary`
      (`adapter-postgres/src/lib.rs:231`) — a client can learn the CAS token from list JSON — while
      `key_write_validator_id` is `#[serde(skip)]` (`:232-233`) and can only come from the ETag.

## Not Doing (and why)

**Not re-keying `list_instances` to `stable_key` in slice 1.** It is a two-line change that widens a
security filter, for the reason in slice-1 part 5. It lands with policy-attachment re-keying or not
at all.

**Not building policy promotion or attachment in slice 1.** It is four coupled pieces — promotion
handler, re-granted `INSERT`, a decision about who may promote, and the `0169` drain — behind a DB
privilege wall. Folding it into "add the publish route" would turn a one-handler slice into a
multi-week one and hide the fact that the loop already closes without it.

**Not building ObjectType-as-a-Cedar-resource, schema-mutation verbs, or a per-type ownership
column.** All three are absent (`AUTHORING_ACTIONS` is a fixed five-element const —
`backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252`, enforced at `:273` — containing
no schema-mutation verb; the authoring `Resource` entity models a data row, not a type). Stage-vs-publish delivers the owner's tiering
decision with none of them. Build them only if that boundary measurably fails.

**Not making analytics work.** A canvas that offers "computed metric" today authors dead data — the
formula is stored and never evaluated. Better to omit the widget than to ship a lie. Either write an
evaluator as its own slice or drop `analytics` from the authoring surface; do not leave it half-shown.

**Not surfacing link types as the relationship widget.** `to_object_type_id` round-trips through the
read path and `install_builtin_catalog` resolves it, but the edge writer never consults it
(`instances.rs:870-875`, `:948-951`). The binding a UI must emit is a *property's*
`config.link = {stable_key, to_type}`, and
nothing cross-checks the two declarations (`validate_draft`,
`adapter-postgres/src/lib.rs:1080-1206`, does not). A canvas that draws a link type and fills only
`to_object_type_id` produces a relationship that is silently empty forever. Either collapse the two
declarations or add a coherence check — but not in the slice that ships the publish route.

**Not adding in-place field editing, draft discard, or a schema-level dry run.** All three are real
gaps and none is a missing handler. In-place editing is barred by the child-snapshot equality check
inside `insert_children` (`0165_…sql:508-530`), not merely by a revocable grant. A draft is
append-only and there is no `draft→retired` or `draft→superseded` edge (`0165_…sql:1015-1020`), so a
typo can only be escaped by
publishing it under four-eyes and then staging v+1 — genuinely bad, and genuinely a design decision
rather than a route. Dry run does not exist at the schema level, and `preflight` is not a substitute:
it runs `prepare` + `evaluate_gates` only (`rest/src/lib.rs:753-754`) and never calls `apply_edits`,
so it returns `would_execute: true` for an action whose edits are invalid.

**Not splitting the authorization gate in slice 1.** The tiering above is the recommended direction,
not slice-1 work. `Feature` is a closed compile-time enum authorized by a `const fn` matrix over six
fixed roles (`authz/src/lib.rs:109`, `:573-576`), so a new tier is a recompile plus a matrix row plus
every consumer of that row — a security-matrix change welded onto a routing change, in the one slice
whose value is that it is small enough to be obviously correct. The cost of deferring is real and
accepted: slice 1 ships behind the same org-wide `Feature::RoleManage` gate as the other twelve
routes, so **no business user can touch the loop until the tier lands**, and the slice proves the
engine rather than the tiering.

**Not adding a four-eyes gate on `published → superseded` and `published → retired`.** Both edges are
legal in the FSM and neither consumes an approval — only `review_pending → published` does
(`0165_…sql:1018` vs `:989-1013`). So shipping one lifecycle route makes irreversible retirement
reachable over HTTP for the first time, gated by the CAS token and the org-wide check and nothing
else. The tempting fix — allow-list two edges in the handler — puts a second, divergent copy of the
FSM in Rust, which is exactly the confusion that already exists between the two lifecycle enums.
Slice 1 ships the edges the database permits, names this here, and pins `published → retired` with a
test so the behavior is recorded rather than discovered. Whether retirement deserves its own approval
kind is a governance decision, like the promotion question below.

## Open questions

**Who may promote a policy to `enforced`, and does that role exist?** Slice 2 has to re-grant an
`INSERT` that was deliberately revoked in `0150`. The revoke is the current security posture;
re-granting it to `console_rt` without a named approver would undo it silently. This is the single
question that gates slice 2, and it is a governance decision, not an engineering one.

**Is a draft meant to be instantiable?** Nothing gates instantiation on `published` —
`get_action_type` has no lifecycle filter (`adapter-postgres/src/lib.rs:725`) and
`require_instance_backed_object_type` checks only `backing_kind` (`instances.rs:1398-1415`). With a
custom action authored in a draft, instances can be created against an unpublished version, and
because drafts are append-only, adding a required property afterwards breaks edits on those
instances. Either drafts are deliberate sandboxes or execute needs a published gate. Decide before
the publish route makes the distinction visible to users.

**How does the editor manage the `create` action?** Authoring *any* `instance_revision` action —
including an edit-only one — unconditionally suppresses the auto-attached `create`
(`0165_…sql:1024-1029`, pinned by a passing test at
`publish_auto_create_action_as_runtime_role.rs:302-342`), removing the user's ability to create a
first instance with no diagnostic. And the auto-created action is a frozen snapshot of the v1
property set that is never re-derived, so whether a v2 property is settable depends on whether the
client carried the old `create` forward. The backend reconciles neither outcome. Either `create`
becomes an explicit concept the editor owns, or publish re-derives it.

**What is the migration story when a type gains a required field?** There is none, and none is
currently needed, because instances never follow their type: `ont_instances.object_type_id` FKs to a
specific version (`0155…:31`) and nothing in the tree ever repoints it. Once slice 2 re-keys reads to
`stable_key`, that changes — v1 instances become visible under a v2 schema that may require a field
they do not have. Answer this before slice 2, not during it.
