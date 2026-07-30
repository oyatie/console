# Ecosystem plan — policy, organization, identity, approvals as one entity model

> Status: PENDING APPROVAL
> RALPLAN-DR planner pass, 2026-07-29. Deliberate mode (auth/security + migrations + PII).
> Every "what exists" claim cites **executable** code or DDL. **Two citation forms, deliberately:**
> `path:line` into **unmodified source** (migrations, Rust, specs this revision does not touch) — re-verified;
> and **quoted sentence plus heading name**, no line number, into any file this session also edits — the three
> amended ADRs, `docs/ideas/authority-and-approval-model.md`, `docs/program/LANE-PROTOCOL.md`, and this plan
> itself. The second form exists because adding a header to the input shifted every body line by ~30 and the
> first repair invalidated itself in the same edit. A mechanical citation audit (**X-CITE**, §8 Phase 0) is a
> plan deliverable, because the failure was systemic rather than clerical.
>
> **This is a DELTA, not a fresh design.** Finality is decided by ADR-0023, the Cedar strangler by
> ADR-0021, the workflow engine and its org-local spine by ADR-0018, branch scope by ADR-0003, local
> identity by ADR-0022. Where a matter is accepted, this plan states only the delta and inherits the rest
> (`docs/decisions/README.md` rules 1-6). **A plan cannot supersede an accepted ADR** (rule 4), so this
> document's real output is the governance records of **§5.11**, which carry their own allocated numbers there:
> five reciprocal amendment pairs (**ADR-0027** through **ADR-0031**) and four or five non-amending records
> (**ADR-0032** onward). **Two of them block Slice 0: D2 (ADR-0028) and D3 (ADR-0029).** G1 is **withdrawn**
> (premise false), G6 and G7 are **struck**, and G8 takes no record. Numbers are assigned centrally in the
> integrator's commit; no lane computes "next free".
>
> **It also corrects itself in three places**, because authoring and review are separate passes here:
> the five-classes/two-spines frame is **retracted** (§4.0 — it re-created the `Feature`-matrix disease);
> `work` moves Tier N → Tier T (§0.14); and the "make the object a dimension on the existing GL" premise
> is **dead** — there is no general ledger (§5.5).
>
> Input challenged: `docs/ideas/authority-and-approval-model.md`. Mechanisms owned elsewhere are
> referenced, not restated: `docs/ideas/fanout-plan-DRAFT.md` §5/§5.1/§6/§6.5/§7,
> `docs/program/LANE-PROTOCOL.md`, `docs/ideas/no-code-ontology.md`.

---

## 0. Corrections to the input document

The input was authored in one session by one hand and already retracts four claims. Reviewing it
against executable code finds **nine more** (§0.1-§0.6, §0.12, §0.13). Two of them change the plan's shape, so they
come first. §0.7-§0.11 are repo facts the input did not cause and this plan does not fix, recorded because
the entity model trips over them. §0.12-§0.14 are corrections to claims made TO this plan and BY it.

### 0.1 The input's headline cost estimate belongs to a design its own body rejected — BLOCKING

Anchored by **quoted sentence and heading**, never by line number: that file has grown a correction header
twice, and each time every line-number citation into it silently moved while remaining textually plausible.

In `docs/ideas/authority-and-approval-model.md`, under **`## Where employees belong`**, it retracts
group-scoped people — *"This revises the earlier 'group is the tenancy boundary for people' answer."* and
*"The group is not high enough … Group-scoping relocates the duplication rather than removing it"*. Then under
**`## Recommended Direction`** it recommends exactly that — *"**People are group-scoped.** Per the owner's
choice, the group is the tenancy boundary for people"* — and under **`## The two hard problems`** it sizes
`app.current_group` across 141 RLS tables as *"This is the largest single engineering cost in the chosen
model"*.

That cost is incurred **only** by the retracted option. The body's own conclusion — *"The person belongs to the
platform; the tenant owns the edge"* — needs **no second tenancy dimension at all**, which **X4 later confirmed
by execution** (`docs/ideas/experiment-x4.md`; see §4.2 for what X4 does and does not cover).
The largest line item in the input's cost model is an artifact of an internal contradiction.

### 0.2 `Feature` cannot be deleted — it is Cedar's action vocabulary

`cedar_pbac/engine.rs:430` builds the Cedar action UID from `request.action.feature().as_str()`.
Delete the enum and Cedar has no action names. The brief and the input both treat
"`Feature` × 6-role matrix" as one deletable unit (`backend/crates/platform/authz/src/lib.rs:109` and `:573`);
they are two things:

| Thing | Location | Verdict |
|---|---|---|
| `pub enum Feature` — capability *name*, ~500 call sites across 25+ crates | `authz/src/lib.rs:109` | **KEEP.** Cedar's action id; already mirrored as data in `feature_catalog` (`0065:11-14`) |
| `pub enum Role` — the 6 demo roles | `authz/src/lib.rs:35` | **DELETE** |
| `const fn matrix_row(self) -> [PermissionLevel; 6]` — the *decision* | `authz/src/lib.rs:573` | **DELETE** |

Only the decision is scaffolding. This shrinks problem C rather than deferring it (§5.3).

### 0.3 Cedar `parents` hierarchy is unimplemented, not merely unused

Both entities ship with an empty parent set: `backend/crates/platform/authz/src/cedar_pbac/engine.rs:392`
and `:425` pass `HashSet::new()`, and `backend/crates/platform/authz/src/cedar_pbac/engine.rs:449` hands `Entities::from_entities` exactly two entities,
validated against `bundle.schema`. The input's *"Cedar expresses this natively through entity `parents`, so the
corporate graph **becomes** the Cedar hierarchy"* (`docs/ideas/authority-and-approval-model.md`, quoted rather
than line-cited — that file's anchors have moved twice) is a property of **Cedar the library**, not of this
engine. It costs a change at `:392` / `:425` / `:449` plus a schema declaration. The `engine.rs` line numbers
stay as line numbers: that file is unmodified source.

### 0.4 `users` is not keyed `(id, org_id)` — and the keystone is cheaper as a result

`0002_create_users.sql:8` is `id UUID PRIMARY KEY`. `0034_enforce_org_id_rollout.sql:122` **adds**
`users_id_org_key UNIQUE (id, org_id)` so children can pin the tenant via a composite FK; it does not
replace the PK. `employees` is the same shape: `0063:3` PK on `id`, `0076:10` adds the composite
UNIQUE.

The input's *consequence* stands (a user row carries one `org_id`, so one human at two companies is
two rows). Its *mechanism* is wrong, and the correction is load-bearing: **`users.party_id` and
`employees.party_id` are single-column references, not composite ones.** No key surgery on either table.
(Whether that reference is an enforced FK is a *different* question, and the answer is no: D1's
non-foreclosure constraint 1 forbids a cross-tenant identifier as a FOREIGN KEY, so both are bare nullable
`UUID`s, app-validated — §4.1. An earlier draft of this section said "plain single-column FKs", which
contradicted that constraint; the single-column half is the part that corrects the input.)
Relatedly, `group_role_grants.user_id UUID NOT NULL REFERENCES users(id)` (`0060:43`) is ordinary
DDL permitted by the PK — not a cross-tenant carve-out earned by special design.

### 0.5 `org_unit` is not production schema

`parent_org_unit_id` appears only in a conformance **test fixture**
(`backend/crates/ontology/rest/tests/company_conformance/fixtures/org_unit.rs:146`). The input's
*"…so it is load-bearing while under-specified"* (`docs/ideas/authority-and-approval-model.md`, under
**`## Permanent 부서 and temporary 사업장 are different kinds`**) describes a fixture. Org structure is greenfield in production; `home_branch` in `0166` is a separate real thing.

### 0.6 `notices` carries the same cross-company blocker the input rejects elsewhere

The input concludes *"So 통지 → 인지 is built. What is missing is narrower than it first appears"*
(`authority-and-approval-model.md`, under **`## 전자결재 — the line is resolved by competence, not by rank`**)
— naming only the missing content and closure. It missed a third, structural gap:
`notice_receipts` has `FOREIGN KEY (recipient_user_id, org_id) REFERENCES users(id, org_id)`
(`0162:50`), so a recipient **must be a user of that org**. That is the same foreign-key blocker the
input correctly identifies in `gov_approvals` (`0153:79` — the *approver* FK; `:78` is the `requested_by`
twin, and earlier drafts of this plan cited `:78` for both) and rejects the mechanism for. A group-level
line member in another company cannot be notified. Resolved in §5.2 gap 3.

### 0.7 Two object registries exist, and one warned against the other

Two registries hold object kinds, and both are executable:

- `object_types` (`0102:19`) — global, non-RLS, `kind TEXT PRIMARY KEY`, seeding `person` (`:32`) and
  `org_unit` (`:33`). Tier G.
- `ont_object_types` (`0152:18`) — per-org, versioned, `UNIQUE (org_id, stable_key, schema_version)`
  (`:33`). Tier T.

`0115_seed_identity_object_kinds.sql:13` executes `INSERT INTO object_types` — the global one — while
that migration's own header warns against *"a second registry."* Not this plan's defect to fix, but
every new entity must state which registry names it, or it will be seeded into the wrong one. This
plan's answer: `work` gets an `object_types` row (it is an `object_links` endpoint kind, §4.3); all
Tier N types in §4.1 are `ont_object_types` rows only.

### 0.8 The engine does NOT already pay for metrics — the substrate has no read model

I was told `BackingKind::{Projected, Instance}` means "a projection concept exists rather than needing
invention." **Partly true, and the useful half is missing.**

- `list_projected_rows_tx` (`ontology/adapter-postgres/src/instances.rs:1522`) is a **live
  read-through** — it SELECTs the backing table per call and synthesises `InstanceState`s. Not
  materialized, no cache, and `version` is always 1 with empty fixity hashes.
- `ont_analytics` (`0152:107-119`) stores `key`, `title`, `formula JSONB`, `result_type JSONB`. It is a
  **formula registry with nowhere to put results.**
- **Zero `CREATE MATERIALIZED VIEW` in all 205 `.sql` migrations in the main checkout as of `8e76dffb4`.**
  Verified repo-wide by `ls backend/crates/platform/db/migrations/*.sql | wc -l` = **205**, highest
  `0205_ont_policy_api_attach_writer.sql`; the 206th directory entry is `BUCK`, which is why three places in
  this plan said 206. Reservations start at **0207** (0205 landed, 0206 is in flight in lane-1). Restating a
  derived fact in prose is what `docs/ideas/fanout-plan-DRAFT.md:243` warns against, using the migration count
  as its own worked example — *"simultaneously wrong in three planning docs"*. The claim itself is unchanged;
  only the count moves.

So the replayability-versus-aggregate tension (Addendum 5) is real and the substrate does not resolve
it. Resolved in §5.7 by putting the two questions in two stores rather than building a read model.

### 0.9 Realtime is shipped, and the transport forces the answer

`backend/crates/platform/realtime/src/lib.rs` already implements Postgres `LISTEN/NOTIFY` →
local `axum` WebSocket hub: `WebSocketUpgrade` (`:11`), `PostgresMessageNotifier` (`:120`),
`PostgresNotificationNotifier` (`:273`), `pg_notify` calls (`:145`, `:174`), three channel consts
(`:37-39`), and the `RealtimeEvent` enum (`:318-337`).

The decisive line is `pub const NOTIFY_PAYLOAD_LIMIT_BYTES: usize = 8000;` (`:40`). **A computed fold
cannot be pushed over this transport.** So "materialise or compute on demand" is not an open design
choice at the notification layer — the push carries ids and the client re-reads. Resolved in §5.6.

### 0.10 Scoped channels are cheaper than stated — the kinds already exist

`messenger_threads.kind CHECK (kind IN ('work_order','team','dm','group'))` (`0012:9`) is already the
four channel kinds the game lens asks for, and `0012:16-19` + the partial unique index `0012:22-24`
already give **exactly one channel per work order** — so "the conversation follows the work" is
structurally present, not new.

The claim that membership is hand-maintained is **TRUE**: `messenger_thread_members` (`0012:30-36`) is
`(thread_id, user_id, role, joined_at)` with `PRIMARY KEY (thread_id, user_id)` and nothing deriving
it. Cost and resolution in §4.8.

### 0.11 `policy_assignment_preview_receipts` is a ceremony, not a simulator — and `policy_versions` is HALF the cache key

Confirmed: `0065:159-172` stores `actor_id`, `user_id`, `current_branch_ids`, `current_role_ids`,
`role_ids` (proposed), `policy_version`, `expires_at`, `consumed_at`. It records the **inputs** of a
proposed change with expiry and single consumption — a real preview→receipt→consume ceremony, but it
never stores a computed outcome. So "simulate a role or 전결규정 change before committing" is **new
work with a cost** (§4.8), not a free reuse.

The more valuable find sits four lines below: **`policy_versions`** (`0065:177-181`), a per-org
monotonic version bumped on every role write, and already a required cache-key part in the coexistence map. It
is **half** of the cache-invalidation key the realtime question needs — it is `PRIMARY KEY (org_id)`, so on its
own it invalidates every client in the tenant on any change, and it is not bumped by assignment writes at all.
The other half is `authz_subject_version`; §5.6 keys on both.

### 0.12 CONFIRMED: on every *reachable* path, a link type alone produces no edge — every relationship must ride a property

Verified by reading `sync_property_links_tx` (`ontology/adapter-postgres/src/instances.rs:874`) in full.
The claim is **true**, and it constrains every relationship in §4.3.

- `:888-891` — the loop iterates **properties**, and
  `let Some(link) = prop.config.get("link") else { continue; };` skips any property without
  `config.link`.
- `:895-904` — `stable_key` and `to_type` come from **the property's `config.link`**.
- `:906-919` — the link type is resolved by `SELECT id FROM ont_link_types WHERE object_type_id = $1 AND
  stable_key = $2`. It selects `id` only. **`to_object_type_id` is never read.**
- `:954-981` — the target-type check compares the referent's actual `ont_object_types.stable_key`
  against `to_type` **from the property config** (`:973-979`), not against the link type's declared
  target.
- `:986-988` — `if link_type_ids.is_empty()`, the type "touches `ont_links` at all", and the code notes
  every built-in catalog type is in that branch.

**So a relationship declared only as a link type — even with `to_object_type_id` correctly set —
produces zero edges, silently, forever, on every path a user or an API caller can reach.** A canvas that draws
relationships as link types would ship an empty graph that looks configured. **Executed as X1**
(`docs/ideas/experiment-x1-x2.md`, re-runnable at `docs/ideas/experiments/x1/run.sh`): the link-type-only case
wrote **0** edges; the property case wrote edges.

**Two sharpenings the earlier draft lacked, and both change what an implementer does.**

- **The rule is about *reachable* paths, not about all code.** `PgInstanceStore::create_link`
  (`backend/crates/ontology/adapter-postgres/src/instances.rs:291`, audited INSERT at `:319`) *does* write an `ont_links` row directly from a bare
  `link_type_id` — but **every call site in the repo is under `tests/`**, and `ONTOLOGY_ROUTE_PATHS`
  (`backend/crates/ontology/rest/src/lib.rs:213-228`) is exactly **14** paths, **none of which creates a link**.
  So the absolute form of the claim is false and the reachability form is exact. Stated because someone will
  find `create_link` and conclude the trap is not real.
- **`to_object_type_id` is DECORATION.** It appears **zero** times in the whole write module
  (`instances.rs`). Setting it correctly buys nothing; setting it wrongly costs nothing. Say so, or an
  implementer will trust it as the declaration of intent it looks like.

**Resolution, chosen over fixing it:** every relationship in §4.3 is specified as travelling through a
property carrying `config.link = {stable_key, to_type}`. This is not a workaround — it is the only path
the writer implements, and the shipped `employment` fixture already does exactly this
(`backend/crates/ontology/rest/tests/company_conformance/fixtures/employment.rs:161`, `:172`). The plan adds one
cheap guard instead of a refactor: a check inside `validate_draft` that a link type with `to_object_type_id` set
has some property referencing its `stable_key`. One check, and it fails closed on the trap rather than leaving
it armed.

**The guard is absent today, and that is the record the plan's own discipline requires.** `validate_draft`
exists (`backend/crates/ontology/adapter-postgres/src/lib.rs:416`, `:458`), but its **entire** link-type
validation is `:1142-1151`, which checks **duplicate `stable_key` only** and nothing about property references.
So §7's `link_type_alone_is_rejected` is **observed RED today** — the known-bad control is present behaviour,
which is exactly the state principle 5 demands before a guard lands.

### 0.13 A published Tier N type lists EMPTY forever until a policy is attached — and `docs/ideas/no-code-ontology.md` is now stale on the fix

Two findings, and they pull in opposite directions.

**The trap is real and confirmed.** `residual::lower` carries
`// Deny-by-omission: no applicable permit ⇒ nothing is visible.` followed by
`if permits.is_empty() { return ResidualFilter::deny_all(); }`
(`platform/authz/src/cedar_pbac/residual.rs:200-203`). `list_instances` lowers object policies to a SQL
residual unconditionally, so **a freshly published no-code type lists `[]` forever** with nothing
attached. Combined with §0.12, the no-code path has **two independent silent-empty traps**: a
relationship that writes no edge, and a type that returns no rows. Both look like working
configuration.

**But the fix is now reachable, and the input document says otherwise.**
`docs/ideas/no-code-ontology.md` states *"Publish a type … **NOT reachable. This is the one missing
route.** `ONTOLOGY_ROUTE_PATHS` is exactly 12 paths and none touches the schema FSM."* That is **stale**:
`OBJECT_TYPE_LIFECYCLE_PATH` (`backend/crates/ontology/rest/src/lib.rs:201`) and `OBJECT_TYPE_POLICIES_PATH`
(`:202`) both exist and are both registered in `ONTOLOGY_ROUTE_PATHS`, which runs `:213-228` and holds **14**
paths — not the 12 that document counted, and not the `:213-217` an earlier draft of this plan cited. Landed by
#521 and #525/0205 after that document was written.

The attach path exercised over HTTP is **`POST /api/v1/ontology/object-types/{stable_key}/policies`**
(`OBJECT_TYPE_POLICIES_PATH`), backed by the audited definer in `0205_ont_policy_api_attach_writer.sql`.
Measured as X2: `200 OK []` with no policy, then `201 Created` and rows with one attached.

**And the consequence X2 measured is sharper than "visible but unfiltered": an unpoliced entity is ABSENT.**
A row the list hides is **`404` by id — deliberately**, so a 403 is not an existence oracle
(`backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs:133-141`, asserting *"a row the
list hides must not be fetchable by id"*). This matters for §4.2: deny-by-omission is a confidentiality
mechanism here, not a usability defect.

**Consequence for this plan:** every Tier N entity must ship with its object policy attached in the same
change that publishes it — **which is a `view` permit, and that is all an authored object policy can
express** — and `tier_n_type_lists_nonempty` is its probe (§7). What remains true from that
document is the deployment hold (§8), not the missing-route claim.

### 0.14 Correction to this plan's own §4.1: `work` must be Tier T, not Tier N

My earlier draft placed `work` in Tier N. Given §0.8, that was wrong: an ontology instance's state is a
fold over revisions, which answers as-of questions well and `AVG(cycle_time)` over 10,000 rows badly —
and there is no read model to bridge it. `work` becomes a Tier T table projected into the ontology
(§4.1), while `assignment` stays Tier N because *that* is what authority folds over. Authoring and
review being separate passes is the repo's rule; this is that rule catching a planner error.

### 0.15 The `Feature` freeze is on MINTING, not composing — so the canvas costs no amendment

`Feature::ALL` is `[Self; 96]` (`authz/src/lib.rs:372`), not the ~40 the specs assume.
`docs/specs/rbac-configurable.md:257-259`, under **"Hard invariants (NON-NEGOTIABLE)"**: *"Only the **assignment** of
the existing `Feature` set is editable. No SQL/console path creates a new `Feature`."*

**A canvas that composes the existing 96 breaks nothing.** Only a canvas that mints capabilities hits that
invariant. This plan's canvas composes — so the freeze needs no amendment, and the earlier "delete the
matrix" framing was arguing for a much more expensive change than the requirement needs. The same document
already calls the six roles *"bootstrap columns for migration parity, not the target operating model"*
(`:122-124`) and bans role-string authorization outright in R1 (`:30-39`), so the direction is settled;
only minting is frozen.

### 0.16 BLOCKING: **two** shipped derivations mint `BranchScope::All` from `Role`, and the `org_id` × `BranchScope` divergence they create is PRESENT-tense

`resolve_branch_scope_in_org` (`authz/src/lib.rs:1472-1483`):

```rust
if roles.iter().any(|role| matches!(role, Role::SuperAdmin | Role::Executive)) {
    return Ok(BranchScope::All);
}
```

That is **a** tenant-side derivation of `BranchScope::All`, and it keys on `Role`. Every KPI rollup and
cross-branch read depends on it. `ADR-0003`'s `## Decision` says *"`All` for SUPER_ADMIN/EXECUTIVE rollups, an
explicit branch set otherwise"*.

**It is not the sole one, and that changes what the problem is.** A **second shipped derivation** exists:
`backend/crates/platform/request-context/src/lib.rs:421-422` mints `BranchScope::All` for a `{Role::Admin}`-only
principal **after live group-membership proof**, through
`effective_branch_scope_for_tenant(BranchScope::All, access_scope, org_id)`. The realtime fan-out is in scope
too: `backend/crates/platform/realtime/src/lib.rs:843`, `:885` and `:899` filter on `user_id` and
`branch_scope.allows(branch_id)` and **never compare `org_id`**, so an all-branch scope reaches every branch the
hub can address.

**So the trigger is PRESENT tense.** The `org_id` × `BranchScope` divergence exists **today**, independent of any
`Role` deletion. C5 / `Role` deletion is therefore **not** the trigger and must not gate the plan on it — the
record is owed now, and §5.3's C4b and C5 rows say so.

**R11, carried from the group-scope analysis: group designation stays STORED, EXPLICIT and AUDITED, and control
edges may never be an input to any authorization resolver.** The reason is exactly the second derivation above:
`group_memberships` is the **sole** input to the cross-entity `{ADMIN}` + all-branch mint. Deriving group
designation from control edges (W7) and then feeding it back into that resolver would make an authorization
scope a function of an inferred graph, which is the one shape a share-percentage cycle can silently widen.

And the obvious replacement is *forbidden*: `rbac-configurable.md:366` — *"Custom role definitions do **not**
widen `BranchScope::All`, group scope, or platform scope. Those scopes remain resolved by the existing
membership/token systems."*

**Resolution (C5, §5.3):** replace **both** derivations with a **built-in `Feature`** check — authored in code,
not mintable from the console, and not a custom-role definition. That satisfies `:366` (not a custom role) and
`:257-259` (no console path mints it). The governance record is **D2 (ADR-0028)**, which amends `ADR-0003`'s
`## Decision` in place and **merges what earlier drafts split into G2 and G2b** — because CI cannot detect two
records editing the same Decision line incompatibly (§5.11). C5 is blocked until D2 is accepted; the **record
is owed now**, because the divergence it describes is present-tense.

One more write site the `Role`-deletion path must cover, named so it is not missed: the **onboarding seeder**
inside `create_org` (`backend/crates/platform/platform-rest/src/lib.rs:568`) seeds the first SUPER_ADMIN, so it
writes a principal whose scope the deleted match arm used to resolve.

### 0.17 Scope expressions are already decided against — delete them from this plan

`rbac-configurable.md:421-423` decides: *"model org-wide reach as the `OrgWideQueueTriage` capability (and
future `*OrgWide` capabilities), **not a free-form scope DSL** — keeps RLS the hard floor."*

Earlier drafts of this plan carried "scope expressions over the control graph". **Deleted.** Reach is a
named capability (`*OrgWide`, `*GroupWide`); the control graph supplies which scopes *exist*, never a query
language. Fewer moving parts, smaller attack surface, and it is what the spec already chose.

---

## 1. Principles

1. **The tenant-visible fact is the edge, never the person row.** A durable identity is platform-level
   and unreadable by the runtime role; what a tenant sees is an ordinary org-scoped row under the RLS
   floor that already exists. Confidentiality comes from the floor, not from an exception to it.
2. **Additive grants only.** `effective(person, scope) = fold(grants)`. Revocation closes a validity
   interval; it never writes a deny. Routing is a default, never a capability restriction. Standing is
   membership in the 결재권 graph, never scope descent. **Additive-only constrains the FOLD; it does not
   forbid an authoring-time exclusion.** Refusing to *create* a grant that would form a conflicting pair
   (§5.11, SoD) is a check at write time on a set that does not exist yet — it removes nothing from any
   fold, and conflating the two would make segregation of duties inexpressible.
3. **The four dimensions are vocabulary, not hierarchy.** 소속 / 직급·직책 / 직무 / 결재선 are
   predicates for writing grant rules. None confers authority. **Two of the four have no substrate today** —
   `policy_role_conditions.attribute` holds 17 literals and 직무/직급 are not among them, so they arrive in W6
   rather than in slices 0/1 (§4.4). Stated here because this is the principle that reads as though all four
   were already data.
4. **Reuse the classification, not just the code.** Every storage decision names **one of the four
   CI-enforced tiers, optionally projected** (§3.1 adds Tier P — projected — twenty-odd lines later, and Phase
   0's `ecosystem-PORTING.md` is meant to be looked up rather than re-derived). A new *tier* is a plan defect;
   a projection is not.
5. **A probe is untrusted until it has been RED.** Six probes were defective in one session here. No
   acceptance criterion counts without its known-bad control (§7).
6. **Concerns are components; entities compose them, and both sets are open** (§4.0). No closed,
   hand-enumerated list of classes or concerns — that is the `Feature`-matrix disease one level up. Any
   table in this plan is the *current* set plus a stated extension mechanism.
7. **Intuitive surface, uncompromised depth.** Game systems are prior art for structure and ergonomics
   (§4.7), never for consequence. The quest log is the UX; the ledger, the metrics and the capacity
   record are what must not be simplified away.

## 2. Decision drivers

1. **The 141-table org-isolation floor must not be weakened or duplicated.** It is the only tenancy
   GUC (`app.current_org`); it is enforced by `backend/ci/gates/tenant-isolation`. Any design needing
   a second dimension pays the largest cost in the program — so the design must not need one.
2. **Canvas-configurable and replayable must be free, not built.** The ontology already gives
   effective-dated, fixity-chained, fold-based state (`0155:37-64`). Entities that take that path cost
   an authored type; entities that don't cost a migration plus a history mechanism. **Qualified: what is free
   is replay along the VALID-time axis.** Knowledge-time correction is not free and is not built — the
   append-only trigger (`0155:112-160`) refuses in-place repair by design, so a correcting axis is a decision
   §5.9 records rather than a property this driver supplies.
3. **Payroll correctness is the first vertical's acceptance bar, and PII is where it attaches.** Every
   jurisdiction binding and Korea control reads HOLD
   (`docs/program/console-program-ledger.md:327`, `:420`), and the PII substrate itself
   (Jurisdiction/Consent/DSR objects) is an explicitly deferred epic (`:174`). The design must let
   slice 0 land **without** storing any new personal data anywhere new.

## 3. Viable options

### 3.1 Shared vocabulary — the four storage tiers that already exist

Named once here; every entity in §4 refers to them. All four are enforced by
`backend/ci/gates/tenant-isolation/src/lib.rs`.

| Tier | Definition | Gate hook | Runtime reach |
|---|---|---|---|
| **G — global read** | no `org_id`, no RLS, `console_rt` may SELECT | `global_table_allowlist()` `:44`, entries `:48-70` | direct |
| **O — owner-only** | no `org_id`, no RLS, **no runtime grant at all** | `owner_only_table_allowlist()` `:115`, entries `:117-129` | SECURITY DEFINER only |
| **T — tenant** | `org_id NOT NULL` + `ENABLE`/`FORCE` RLS on `app.current_org` | default classification; anything else is `UnclassifiedTable`, `:804-808` | direct, filtered |
| **N — ontology instance** | rows in `ont_instances`/`ont_instance_revisions`/`ont_links`, themselves Tier T | `0155:16`, `:37`, `:66` | via ontology API |

Two facts about the tiers decide most of §4:

- **Every Tier G rationale is literally "no tenant data"** (`backend/ci/gates/tenant-isolation/src/lib.rs:48-70`).
  PII therefore cannot go
  in Tier G. Tier O is where cross-tenant authorization data already lives — `group_memberships` and
  `group_role_grants`, with rationale *"cross-tenant … resolver only"* (`:117-129`).
- **Tier N cannot hold a cross-tenant edge.** `ont_instances.org_id` is `NOT NULL` (`0155:18`) and
  `ont_links` pins **both** endpoints to the same org via composite FK — `0155:76`
  `FOREIGN KEY (from_instance_id, org_id) REFERENCES ont_instances(id, org_id) ON DELETE CASCADE,` and its
  `to_instance_id` twin at `:77`. This is structural, not a missing feature. It is the single constraint that
  shapes the entity model.
  **Its consequence, applied where it bites:** an `ont_link` endpoint must be an `ont_instances` row in the
  same org, so **no scope descriptor can be an `ont_link`** — not a `Group`, not an `Org`, and by the
  uniformity argument not an `org_unit` instance either, even though that one *is* storable. Every scope edge
  in §4.3 is therefore a property `{level, node_id}` carrying `AccessScope`, whose `level` is one of the five
  shipped `AccessScopeLevel` variants and nothing else (§4.1). This section named the constraint and §4.3 contradicted it; the
  contradiction is the defect, and X4b measured which side was right.

A fifth path, **Tier P — projected**: `ont_object_types.backing_kind = 'projected'` (`0152:25`,
CHECK at `:34-38`) makes an existing Tier T table canvas-visible without moving its data, against a
compiled-in allowlist (`backend/crates/ontology/adapter-postgres/src/instances.rs:1479-1498`). Cost: projected
types have no owned revision store, so no fixity chain and no as-of replay (`instances.rs:1522`) — and,
because they own no `ont_instances` rows at all (`instances.rs:1443-1450`), **no `ont_link` may name a
projected type's row as an endpoint** (§4.3).

**Tier P is code-gated, not CI-gated**, and the difference decides who can add one: `allowlisted_projected_table`
(`instances.rs:1479-1498`) is a compiled-in `match` arm plus a CHECK, **not** an entry in
`backend/ci/gates/tenant-isolation/src/lib.rs`. So a new projection is a code change reviewed like code, and
no gate will tell you it is missing.

**The tier rule.** Cross-tenant → O. Within one tenant, needs authoring + replay → N. Needs a
constraint the ontology cannot express (uniqueness, money, FK to legacy) → T, projected into P if the
canvas must see it.

### 3.2 The options

#### Option 1 — Platform-homed opaque handle + tenant edge + Tier N authority  ← RECOMMENDED

`party` as an attribute-free row **homed at the platform sentinel org** under the ordinary `org_isolation`
policy, so it is invisible and un-mintable from every tenant without a carve-out (§4.1);
`party_org_visibility` in Tier T under the same ordinary `app.current_org` RLS; one re-validating definer for
the **authority fold**; the authority/approval entities of §4.1 as ontology instance types.

**Pros.** Zero new GUCs, zero changes to the 141 RLS policies, zero new gate classifications, **and no
owner-only table for the handle itself**. Reuses the existing tier classifications unchanged, the shipped
`SECURITY DEFINER` resolver pattern (`0060:99-126`),
the `object_links` edge store (`0102:54`), and the re-validating-read bargain (`backend/crates/platform/authz-rest/src/store.rs:576-593`).
Canvas-editability and replay arrive free for every Tier N entity — the large majority. PII does not
move. Slice 0 is
unblocked while every Korea control reads HOLD.

**Cons.** Cross-store pointers: `employment.party_id` and `grant.subject` are attribute-bag UUIDs, not
`ont_links`, so referential integrity to `party(id)` is app-enforced and the ontology's search-around
traversal will not cross that hop. Two hops in two stores for any party-rooted query. The definer is a
standing security-review surface that must be re-proven every time it changes.

#### Option 2 — Second tenancy dimension (`app.current_group`)

What the input's **`## Recommended Direction`** assumes — *"People are group-scoped"* and
*"the largest single engineering cost in the chosen model"* (§0.1).

**Pros.** A group-scoped person row is directly readable by `console_rt`, so ontology links to it work
natively and no definer is needed.

**Invalidated** — by the requirement it exists to serve. The input establishes it itself, under
**`## Where employees belong — reasoned from the objects, not the schema`**: *"The group is not high
enough. A person can work for companies in **different** groups — a contractor, a director on two unrelated
boards, anyone moving from group A to group B."* (Quoted, not line-cited: this was the **last** surviving
line-number anchor into the one file whose anchors moved twice — `grep -F ':89-92'` returned exactly one hit,
here.) So group-scoping relocates the duplication rather than removing it, and
cannot represent a person before they are grouped. It then also costs a second GUC bridged into 141
RLS policies and a gate classification that does not exist. **Maximum cost for a design that does not
meet the requirement.**

#### Option 3 — `party` as a Tier G global-read table

**Pros.** Simplest possible: no definer, no resolver, direct reads.

**Invalidated** — Tier G means `console_rt` may SELECT with no filter, so any tenant enumerates every
party on the platform. That directly contradicts the confidentiality requirement (company A must not
learn its employee also works at company B), and every existing Tier G rationale is literally *"no
tenant data"* (`tenant-isolation/src/lib.rs:48-70`). Adding PII-adjacent identity there breaks the
allowlist's own stated meaning.

#### Option 4 — a party row **per tenant**, deduplicated by a matching service

**Not Option 1 with different words, and the distinction is the whole point.** Option 1 also puts the handle
under ordinary RLS (§4.1) — but at **one** home, the platform sentinel org, so there is exactly one row per
human and no matching. Option 4 is one row **per tenant** with a service guessing which rows are the same
person. The tier is not what separates them; the cardinality is.

**Pros.** No new tier usage at all; every row stays under the existing floor.

**Invalidated** — this is the status quo (`users` + `employees` + `person_name`), whose three failed
attempts are the reason this work exists. The executable evidence that matching cannot substitute for
identity is in `0076` itself: the link is a **nullable** column (`0076:13-14`) with a partial unique
index (`0076:22-24`), and its backfill promotes a row only where `HAVING count(*) = 1` holds for the
employee number (`0076:40-46`), leaving every duplicate unlinked. `employees` even carries
`identity_resolution_strategy` and `identity_resolution_confidence` (`0075_employee_identity_resolution.sql:6`, `:13`) — a confidence
model, which is what you build when matching is a guess. A mechanism that must decline the ambiguous
majority is not an identity.

---

## 4. The entity model

### 4.0 The frame: concerns are components; entities compose them

> 사람, 물건, 일, 계약, 회사 — 또는 그 모든 것들도 결국은 그 **기록**과 그 **economics**가 중요한 것.

The owner's list is **illustrative, not exhaustive** — the sentence ends 또는 그 모든 것들도. An earlier
draft of this plan read it as five classes over two spines. That was wrong, and wrong in a specific way
worth naming: **it re-created the `Feature` × 6-role disease one level up** — a closed, hand-enumerated
vocabulary where the requirement is authorability. Both sets are open.

**Concerns are components. Entities declare which they compose.** This is the ECS mapping taken
literally, and it is the honest answer to "how does this tie into the engine": the ontology's typed
properties and links **are** the component mechanism, and actions with dispatch are the systems. What a new
entity class does **not** get is automatic behaviour per concern — §4.0.2 states the boundary, and an earlier
draft of this section asserted the opposite two paragraphs above it.

#### 4.0.1 The component vocabulary — current set, not a closed one

Each concern is an attachable thing with a **contract**: what an entity must supply, and what it gets.
This is the current set. It is extended by adding a row, and nothing in the design depends on its size.

| Component | Contract: entity supplies | Contract: entity gets | Substrate today |
|---|---|---|---|
| **record** | an action for every consequential mutation | append-only history, actor, time, **capacity** | `audit_events` (`0003:10`) — **capacity missing**, §4.0.2 |
| **authority** | a scope, and a Cedar resource type | fold-decided access; `effective(party, scope)` | `cedar_pbac` + `grant` (§5.1) |
| **tenancy/scope** | `org_id` | FORCE-RLS isolation on `app.current_org` | 141 tables; gate at `tenant-isolation/src/lib.rs:44` |
| **time/effectivity** | `valid_from` / `valid_to` | as-of replay, fixity chain | `ont_instance_revisions` (`0155:37`) |
| **lifecycle** | a state set | FSM transitions, terminal soft states | `ont_instances.lifecycle_state` (`0155:27`); `lifecycle_transition_rules` |
| **identity/naming** | a display key | human-navigable UI, `!`-code deref | `title_property_key` (`0152:23`); `object_types` (`0102:19`) |
| **relationships** | a property carrying `config.link` | traversal | `ont_links`; `object_links` (`0102:54`) — **see §0.12** |
| **approval** | a document class | routed line, signatures | ADR-0023 (§5.2) |
| **communication** | a channel kind | scoped thread, derived membership | `messenger_threads.kind` (`0012:9`) |
| **custody/handover** | an assignee edge | transfer; 인계 완료 as **one audited assertion, not a query and not a gate** | §4.5 |
| **economics** | a dimension reference | **cost** as a query over voucher lines (revenue/profit need the peer plan's account master — §5.5) | **largely absent** — §5.5 |
| **structural lineage** | a quantity + a parent edge | split/merge DAG with conservation | **absent** — §5.8 |
| **measurement** | a metric formula | aggregates | `ont_analytics` (`0152:107`) stores formulas, **no result store** — §0.8 |
| **compliance/jurisdiction** | a jurisdiction binding | control traces | register; **all HOLD** — §5.4 |

**Composition, shown for four entities — including one the owner never named**, to demonstrate the
mechanism rather than five special cases:

| Entity | Components composed |
|---|---|
| `contract` | record, authority, tenancy, time, lifecycle, naming, relationships, approval, economics, lineage |
| `asset` | record, authority, tenancy, time, lifecycle, naming, custody, economics, measurement |
| `notice` | record, authority, tenancy, lifecycle, naming, communication |
| **`grant`** *(not on the owner's list)* | record, authority, tenancy, time, lifecycle, naming, relationships |

`grant` is the interesting one: it composes **authority** both as a subject *and* as a governed object —
which is exactly hard problem A (§5.1). No entity carries every component; that is the point of
composition and the reason a fixed list of spines was wrong.

#### 4.0.2 How a NEW class is added — and the honest boundary

| Step | Authored (canvas) | Requires code |
|---|---|---|
| declare the type, properties, links | yes — `POST`/`PUT` object-types are live (`ontology/rest/src/lib.rs:194`, `:370`) | — |
| publish it | yes — `OBJECT_TYPE_LIFECYCLE_PATH` (`:201`) | — |
| make it readable | yes — `OBJECT_TYPE_POLICIES_PATH` (`:202`), and **required** or it lists `[]` (§0.13) | — |
| **record** | — | **code**: an audit action code |
| **authority** | partly | **code**: a `Feature` variant — Cedar's action id (§0.2) |
| **tenancy** | — | **code**: a migration, if it needs a table |
| **economics / lineage** | — | **code**: today both substrates are absent |
| **projected backing** | — | **code**: one arm in `allowlisted_projected_table` (`instances.rs:1479`) |
| **an action on a projected type** | — | **code**: one `ProjectedDispatchRegistry` handler per action. `backend/crates/ontology/rest/src/lib.rs:160-195` — `pub struct ProjectedDispatchRegistry { handlers: HashMap<String, ProjectedHandler> }`, a chainable `register(target, handler)` builder, and a `dispatch` returning `ActionError::NotWiredYet`. Registered in the App composition root; **unwired = `NotWiredYet`**, which is fail-closed but is still code |
| **the authoring-action vocabulary** | — | **code, and it is CLOSED at five elements**: `backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252` `AUTHORING_ACTIONS` = `view`, `edit`, `read_field`, `console:configure`, `console:deploy`. A sixth authoring verb is a code change, not an authored row |

**So "manageable without developers" is true for the dimension side and false for the component side.**
Declaring a new *type* is authored; giving it a *new concern* is code. That boundary is the honest answer
to the requirement, and it is also the matter named in §5.11 G7: DN-0003 decides extensibility is
**bounded** — declarative tenant definitions over compile-time-allowlisted first-party tools. This plan
does not pretend otherwise.

**Size the wiring, do not label it.** `docs/specs/ecosystem-entity-components.tsv` (Phase 0) must carry a
**handler count** for `work`'s Slice-0, W4, W11 and W13 actions, so §8 Phase 4's `app` crate row reads as a
number rather than as the word "wiring". One `ProjectedDispatchRegistry` handler per action is a countable unit;
"wiring" is not.

**DN-0003 invariant 1 for Tier T and Tier P, answered before `work` is built.** The invariant is *"Every
consequential mutation is an Action. Direct object-property edits are not the normal operational write path"*
(`DN-0003:73-74`). `work` is Tier T projected, and a projected type's domain use-case remains the sole writer
(`instances.rs:1443-1450`) — so the two paths must be reconciled explicitly, not assumed. This plan takes the
**bounded exception**: `work`'s consequential mutations run through the domain use-case, which is the *normal*
write path for that table and is itself audited, while the ontology surface is read-only over it. The gate that
holds the exception is the projected-type read-only contract in `instances.rs:1443-1450` plus the audited
domain use-case; what is forfeited is the **revision history** — a projected type has no fixity chain and no
as-of replay (§3.1), which is exactly why authority does **not** live in Tier T. Probe:
`projected_mutation_goes_through_the_domain_usecase`; known-bad control: a write reaching the backing table
through an ontology property edit.


#### 4.0.3 The headline finding: one missing field, inherited by every entity composing `record`

`audit_events` current shape is `0003:11-28` plus `org_id` (`0032:84`) plus
`ip, user_agent, auth_method, device, classification_badges, anomaly, reason` (`0149:6-13`).

It carries: `actor` (`0003:13`), `action` (`:16`), `target_type`/`target_id` (`:17-18`),
`branch_id` (`:20`), `before_snap`/`after_snap` (`:22-23`), `trace_id`/`span_id` (`:25-26`),
`occurred_at` (`:27`), `reason` (`0149:13`).

It does **not** carry:

| Missing | Consequence |
|---|---|
| **the authorizing grant (capacity)** | a 결재 signature cannot record which of several grants authorised it; the feed cannot say "as 그룹 인사"; "why may this person do this?" is unanswerable from the record |
| **`on_behalf_of`** | 대리 / 대결 cannot record both parties |
| decision scope | `branch_id` is the *operational* scope (ADR-0003) — not the unit a decision was made for |
| quantity / amount / unit | a ledger line cannot be rendered |

**This is a gap in the `record` component's contract, so every entity composing `record` inherits it.**
That is what makes it a high-leverage finding rather than one feature's missing field:
`authorizing_grant_id` + `on_behalf_of_party_id` supplies the 결재 capacity record (§4.1), the activity
feed, the audit trail, and any ledger line.

**Where the two columns land in Slice 0: `gov_approvals`, not `audit_events`.** The finding stands; its
target moved, and the reason is reversibility.

- `built_in_audited_tables()` (`backend/ci/gates/migration-safety/src/lib.rs:164-172`) is exactly
  `audit_events, regions, branches, users, user_branches`, so **a column on `audit_events` is permanent from
  the day it lands** — `DROP COLUMN` on an audited table is a gate violation. A Slice-0 shape that has not
  yet been exercised should not be the shape that can never be withdrawn.
- `gov_approvals` is in **neither** that built-in list **nor** the `-- console-gate: audited-table` marker
  set that `discover_audited_tables` (`:174-187`) folds into it, so the same two columns there are
  reversible. Its additive-column precedent is in the same table
  (`0164_bind_consume_four_eyes.sql:34`, §4.1).
- The probe that would give the `audit_events` pair meaning —
  `capacity_recorded_on_every_authority_mutation` (§7) — needs the **D3 write-path enumeration** (§8 Phase 0)
  to exist first. Landing a permanent column ahead of the artifact that says where null is a defect is the
  wrong order.

**The `audit_events` pair is therefore DEFERRED, and it is priced rather than scheduled.** Its DDL is two
nullable columns and the `0149:6-13` precedent is exact — that migration added seven nullable columns to
this same append-only table. The real cost is not the DDL: it is **reaching the value**. `AuditEvent`
(`backend/crates/kernel/core/src/audit.rs:83`) carries `id, actor, action, target_type, target_id,
branch_id, org_id, before, after, request_context, classification, trace, occurred_at` and **no capacity
field**, so every site that wants to populate the column must have the authorising grant id in hand at the
call. There are **466** non-test `with_audit` references under `backend/crates`. An
`AuditEvent::authorized(…, grant_id)` constructor that the compiler enforces across all of them is a
**RECOMMENDATION, not a requirement** — a compiler-enforced constructor at 466 sites is a larger change than
this plan can price, and pretending otherwise is how a two-column estimate becomes a quarter. The subset
where a null capacity is a **defect** rather than merely absent is named by the D3 enumeration, not by this
paragraph.

**Quantity and amount do NOT go here.** They belong to the `economics` component (§5.5). Putting money
on `audit_events` builds a second ledger, which is the divergence failure by construction.

### 4.1 Entity types

Ordered by tier. "Slice 0" marks the minimum shape the proving slice needs; everything else is a widening.

**Vocabulary is adopted, not invented.** `docs/specs/org-editor-primitives-ux.md:468` names **fourteen** org
primitives: Group/HQ, Organization, OrgUnit, Worksite/Cell, **Person, Employee, User, Position**,
**PolicyRole hook**, ReportingLine, EmploymentAssignment, CrossOrgAssignment, SetupDraft, Audit — with the
separation this plan needs at `:256`: *"A Person is not automatically an Employee; an Employee is not
automatically a User; a Position is not automatically an access Role."* **None of them is built** (no
`positions`, `org_units`, `persons`, `reporting_lines` or `worksites` table exists; the spec admits it at `:25`).
So this is specified-and-unbuilt: use those names.

**All fourteen are listed above, including the one an earlier draft dropped.** The dropped primitive was the
**PolicyRole hook** — which is the one carrying this plan's own quoted separation (*"a Position is not
automatically an access Role"*), so dropping it silently read as an oversight exactly where principle 3 is
weakest. It is **not** an entity here: it is §5.3's `Feature` work. Thirteen of the fourteen are entities in
this section; the fourteenth is a capability hook.

`party` is this plan's only rename, and only because later verticals need customers and suppliers under one
identity — noted so the mapping to `Person` stays obvious.

**`ReportingLine` is EXCLUDED from slices 0/1, and the exclusion is defended rather than silent.** It would be
`reporting_line` (position → position, Tier N, with the spec's cycle and single-primary-path validation). It is
excluded because **nothing in this plan's authority model reads it**: routing resolves through
`delegation_rule` (category × band × scope → competent unit), never up a reporting chain — that is principle 3,
and a reporting line that confers nothing is an org-chart rendering concern. It arrives with the canvas (W10)
or with 인원편성, whichever lands first, and it must not become a grant source when it does.

**직책 needs a non-colliding `stable_key`, and the collision is with a shipped built-in.**
`backend/crates/ontology/adapter-postgres/src/seed.rs:74` is `pub const POSITION_KEY: &str = "position";`, so a
Tier N type keyed `position` collides with a seeded catalog type. Four names now point at overlapping concepts —
the org-editor spec's **"Position"**, the seeded built-in **`position`**, the shipped **`job_position`**, and
this plan's 직책 type. **Record the mapping across all four in `docs/specs/ecosystem-PORTING.md`** (Phase 0);
do not resolve it by picking a name in prose here.

#### Tier O — platform, definer-mediated (**1** new table, DEFERRED out of Slice 0)

| Entity | Purpose | Identity / key | Lifetime | Slice 0 shape |
|---|---|---|---|---|
| **group-scoped grant store** | grants whose `scope.level = Group`; unreachable from Tier N | `(id, org_id_of_grantor, …)`, keyed inside its definer | effective-dated | **DEFERRED** — W5/W8 |

**`party` is NOT in this tier — decided here, and it was the last contradiction left standing in this plan.**
Constraint 4 of the deferral (below) said the handle lands as *"an ordinary tenant-scoped row homed at the
existing sentinel org … **not** a Tier O carve-out, **no** new GUC, **no** definer-mediated read"*, while this
heading, §3.2, §4.2, §4.6, two §7 probes and §9 all still put it in Tier O. **Constraint 4 wins, on four
grounds, and it is the smaller claim:**

1. **The row holds nothing a tenant needs to read.** `party` is `(id, org_id, party_kind, status,
   created_at)` and no attributes (§5.4). The only useful field is the id, and a tenant already has it from
   its **own** `party_org_visibility` row. §4.2 already says the confidential fact is *"which parties does
   org A hold edges to"*, not *"who is this party"* — so no tenant read path is being taken away.
2. **The sentinel org already exists and is already the home for platform-owned rows that outlive tenants.**
   `0036_platform_onboarding.sql:224` INSERTs `organizations` id
   `00000000-0000-0000-0000-00000000face`, slug `platform`, status `ARCHIVED`, with the reason in its own
   text at `:217-221`: *"The platform sentinel needs an `organizations` row so the FK from `users.org_id` is
   satisfiable for the platform admin. It is NOT a tenant … Status ARCHIVED so it can never be mistaken for
   an onboardable/active tenant."* It is excluded from `platform_list_organizations()` (`:121`), and
   `0051_platform_remove_organization.sql:34` **re-homes a deleted tenant's `audit_events` rows to it** so
   *"the immutable record of the action survives verbatim under the platform tier"*. A durable handle that
   must outlive every tenant is exactly that shape, already shipped.
3. **Tier T + FORCE RLS closes the cardinality leak by OMISSION, which is DN-0003 invariant 5's own word.**
   X4 measured `SELECT count(*) FROM x4probe_party` returning **2** where org A held one edge, and invariant 5
   is *"Denied data is omitted, including counts and relationship existence"* (`DN-0003:85-86`). Under
   `org_isolation` on `app.current_org` a tenant-armed `SELECT count(*) FROM party` returns **0** — omitted,
   not denied-with-an-error.
4. **The same policy's `WITH CHECK` makes minting platform-only for free.** With
   `CHECK (org_id = '00000000-0000-0000-0000-00000000face'::uuid)` on the column and `org_isolation` armed, a
   tenant-armed INSERT cannot satisfy both predicates at once. So *"resolution is a platform-principal
   operation, never a tenant capability"* (decided below) is enforced by the two constraints rather than by a
   definer anyone has to review.

**What this removes from the plan:** one `owner_only_table_allowlist` entry, one gate classification, and one
audited `SECURITY DEFINER` surface. **What it does not remove:** Tier O itself — the group-scoped grant store
stays there, because X4b measured a sibling org reading **0** rows from Tier N and `group_role_grants` is the
exact shipped precedent (§4.2). Tier O is used once, for the case that genuinely needs a cross-org read.

`party_kind ∈ {NATURAL, LEGAL}`. **The row holds no personal data** — no name, no phone, no
주민등록번호. It is an opaque durable handle. That is what makes §5.4 (PII) and erasure tractable, and
it is why the row is safe to exist while every Korea control reads HOLD.

Why the sentinel-homed tenant row and not the alternatives: **Tier O** buys a definer and a gate entry for a
read nobody performs (above); **Tier T homed in a tenant org** reintroduces the duplication the entity exists
to remove, and lets that tenant read and mint handles; **Tier G** would let any tenant enumerate every party on
the platform, contradicting the confidentiality requirement and every Tier G rationale; **Tier N** is not
forbidden by `0155:18` once the row is homed at the sentinel — it is rejected because minting is a
platform-principal write and **every ontology write runs on the command pool, which is `None` wherever this
ships** (§8), so a Tier N handle would be green on every PR and dead in production.

Named `party`, not `employee`, so the sales, procurement and governance verticals reuse one identity.
Employment is one relationship kind among `CUSTOMER`, `SUPPLIER`, `DIRECTOR`, `CONTRACTOR`.

**`party` and its three companions are DEFERRED out of Slice 0, and Slice 0 does not wait on them.** Struck
from the 0207+ list: the `party` table and the Tier T rows `party_org_visibility`, `users.party_id`
and `employees.party_id`. **Two different reasons, named separately, because only one of them is
irreversibility:**

- **`users.party_id` is irreversible.** `users` is in `built_in_audited_tables()`
  (`backend/ci/gates/migration-safety/src/lib.rs:164-172`), so `DROP COLUMN` on it is a gate violation —
  **a column added today is permanent, while adding it later is purely additive**. (`employees` is **not** in
  that list and carries no `-- console-gate: audited-table` marker, so `employees.party_id` is reversible; it
  is deferred with `party` only because it has nothing to point at.)
- **`party` and `party_org_visibility` are new tables and therefore droppable.** They are deferred because
  **Slice 0 does not need them** — its grant `subject` is the raiser's `users.id` — and because the
  resolution mechanism below is a named pre-condition rather than a Slice-0 design. Stating this as
  "irreversibility" would have been the wrong reason attached to the right decision.

Slice 0's two grants are `Worksite`-scoped and intra-org (§8), and its capacity recording lands on
`gov_approvals`, so **no lane waits on the party**.

Five non-foreclosure constraints hold while it is deferred, so the deferral cannot become a foreclosure:

1. **No cross-tenant identifier as a FOREIGN KEY**, and none in any UNIQUE constraint or index **whose key
   does not lead with `org_id`** — see the security-control note below, measured in X4 CONTROL 3.
2. **The authorization path never reads `employees`.** It is an HR projection, not an identity.
3. **`0075_employee_identity_resolution.sql:16-17`'s
   `CHECK (identity_name_only_merge = FALSE)` is never dropped or relaxed.** Name-only merge is the failure
   mode this whole entity exists to remove.
4. **DECIDED, not merely constrained** (above): when the handle lands it is **an ordinary tenant-scoped row
   homed at the existing sentinel org** `00000000-0000-0000-0000-00000000face` (`0036:224`), carrying
   `org_id NOT NULL` + a CHECK pinning it to that id + `ENABLE`/`FORCE ROW LEVEL SECURITY` under the standard
   `org_isolation` policy — **not** a Tier O carve-out, **no** new GUC, **no** definer-mediated read, **no**
   `owner_only_table_allowlist` entry.
5. Any eventual edge FK is `RESTRICT` / `NO ACTION`, never `CASCADE` — **if** constraint 1 is ever amended to
   permit one at all. As it stands constraint 1 forbids the FK, so this constraint is the fallback shape and
   not a licence.

**`party_org_visibility`'s key order is a security control, not a style choice.**
`UNIQUE (org_id, party_id, relationship_kind, valid_from)` — **`org_id` leads the key because a unique index
is enforced physically BELOW RLS.** Drop it from the front and error `23505` alone discloses that another
org holds an edge to this party, past a correctly-armed FORCE policy. Measured: X4 CONTROL 3 returned
`ERROR: duplicate key value violates unique constraint "x4probe_edge_control_uniqleak_party_id_relationship_kind_va_key"`
where the real key returned `insert-accepted-no-collision`. **The 0207-series migration that creates this
table must carry that reason as a comment in its own text**, or the next person to "tidy" the key order
reopens a confidentiality hole with a green test suite. Its probe is
`visibility_unique_key_leads_with_org_id` (§7).

**The `party` resolution mechanism — one human, two orgs — is a named pre-condition for when `party` lands,
not a Slice-0 design.** Picked on the record, option (b): **resolution is a platform-principal operation with
an audit record, never a tenant capability.** A tenant-side "is this the same person" action is the matching
service §3.2 Option 4 already rejects, and it would let one tenant probe another's roster. Option (a) —
`party` minted per passkey credential and self-linked by the human at second-org onboarding — remains
admissible and ADR-0022 permits it, because it is **local** identity rather than federation; it is not
chosen because it needs an endpoint this plan has not designed. Constraint 4 above governs either way, and
under it the mechanism is **enforced by DDL rather than by an endpoint's authorization check**: the
`org_isolation` policy's `WITH CHECK` plus the sentinel-pinning column CHECK make a tenant-armed INSERT into
`party` impossible, so "platform-principal only" holds even if a later handler forgets to assert it.

**The scope vocabulary is the shipped enum and nothing else, and there are FIVE variants.**
`backend/crates/kernel/core/src/access_scope.rs:28-34` is
`enum AccessScopeLevel { Group, Org, Region, Branch, Worksite }`. **`org_unit` and `organization` are not
variants** — `Org` is, `Group` is, and no level names a department. An earlier revision of this section wrote
`{org_unit, organization, region, branch, worksite}` into the `grant` row and into §4.5's definer trace, which
is a mismatch against a shipped kernel enum in a **Slice-0 deliverable**. Corrected at both sites, and stated
here because pre-revision §4.5 carried no enumerated branch at all, so the mismatch was *introduced* by a
repair.

**Decision — a 부서-scoped grant has NO scope level in slices 0/1, and the plan is not to invent one.** Two
vocabularies were being conflated, and neither gives 부서 a scope level:

- `AccessScopeLevel` is matched **exhaustively, with no wildcard arm**, in two shipped places —
  `access_scope.rs:86-98` (`branch_scope_for_org`) and
  `backend/crates/platform/authz/src/lib.rs:1524-1538` (`effective_branch_scope_for_tenant`). A sixth variant
  is therefore a change to a **kernel** enum plus a compile error at both sites plus a decided
  `branch_scope_for_org` projection for the new level. That is code, not an authored row.
- `policy_role_conditions.attribute` **does** hold `department` (`0065:115`, one of the 17 literals), but the
  resolver evaluates **only** `branch` and `team` (`authz/src/lib.rs:1403-1429`, `_ => return None`), so a
  `department` condition is **writable and resolver-void today** — and §4.4 narrows the write path away from it
  precisely for that reason. It is not a Slice-0 mechanism either.

So: Slice 0's two scopes are 현장 = `Worksite` and 본사 = `Org`, both shipped variants, and a 부서 bounds a
decision the way the plan already routes — as a **competent `org_unit` instance** named by `delegation_rule`,
which is an instance reference and not an `AccessScope` level at all. Adding a department level to
`AccessScopeLevel` is **W5** work and must arrive with its `branch_scope_for_org` arm decided. Taken as the
smaller honest claim: the alternative is to write a level the kernel does not have into the one deliverable
Slice 0 must ship.

**`Group`-scoped grants cannot be Tier N at all** — they are the **one** new Tier O table this plan adds,
beside `group_role_grants`. (An earlier revision called this "the second Tier O table", counting `party` as the
first; `party` is not in Tier O, above.) `Org`-, `Region`-, `Branch`- and `Worksite`-scoped grants stay Tier N: X4b CASE 1 measured
that arm resolving end to end, with subject, capability and scope folded out of Tier N. The `Group` arm fails
structurally: `0155:18` makes `ont_instances.org_id` `NOT NULL`, leaving no third option for the row's
tenancy, so the grant is homed in exactly one org — and X4b CASE 2c/2d measured org B, a **sibling in the
same group**, reading **0** rows while RLS-bypassed ground truth showed the revision present. A
group-scoped grant that its own group cannot read is not a group-scoped grant.

**The burden that store inherits, measured in X4b §5:** the shipped group definer
(`group_role_grants_for_user`) references **no `app.*` GUC at all**, so **the caller is the org floor** for
every group-scoped authority read today. A new Tier O grant store must therefore be
**authorisation-complete inside the definer**, keyed on the **authenticated principal** — never on a
caller-supplied org id or group id. `group_role_grants` is already in `owner_only_table_allowlist`
(`backend/ci/gates/tenant-isolation/src/lib.rs:115-129`), so the new store takes the same classification and
the same review posture. **It is the ONE owner-only table this plan adds** — `party` is not one (above) — and
`no_new_gate_classification` (§7) names it alone.

#### Tier T — tenant, ordinary RLS (Slice 0: **1** new table and **2** new columns; the party rows are deferred)

| Entity | Purpose | Identity / key | Lifetime | Slice 0 |
|---|---|---|---|---|
| **`party`** | the durable identity handle, **homed at the platform sentinel org** and therefore invisible and un-mintable from every tenant (above) | `id UUID PRIMARY KEY` (plain, per §0.4); `org_id NOT NULL` CHECKed `= …00face` | 永久; never hard-deleted, terminal soft state only | **DEFERRED** — Slice 0 does not need it |
| **`party_org_visibility`** | **the keystone edge.** The tenant-owned fact that this org holds a relationship to this party | `(id)`; `UNIQUE (org_id, party_id, relationship_kind, valid_from)` — **`org_id` leads the key as a security control**, above | effective-dated interval | **DEFERRED** with `party` |
| **`work`** (업무) | first-class work; the join point for artifacts, actions, handover, ledger and metrics | `(id, org_id)` | open → closed | yes — one row |
| **`worksite_registration`** | 사업장 legal attributes: 4대보험 registration unit, optional 사업자등록번호 | `(id, org_id)`; `UNIQUE (org_id, business_registration_no)` | permanent per site | **no** — W3 |
| **`worksite_contract`** (사업장 계약) | the contract a temporary unit's existence is DERIVED from | `(id, org_id)` | a term | **no** — W15 |
| **`lot`** | quantity-bearing node: the splittable/mergeable unit (§5.8) | `(id, org_id)` | until consumed | **no** — W16 |
| `users.party_id` | link the per-org account to the durable identity | **bare nullable `UUID`, app-validated** — constraint 1 below forbids a cross-tenant identifier as an FK, so this is not an FK; §0.4's point is only that a *single-column* reference suffices | — | **DEFERRED** — `users` is audited, so the column is permanent once landed |
| `employees.party_id` | link the imported HR row to the durable identity | **bare nullable `UUID`**, same posture | — | **DEFERRED** with `party` |
| `gov_approvals.authorizing_grant_id`, `.on_behalf_of_party_id` | **capacity** — §4.0.1, §4.0.3 | two nullable additive columns | — | **yes** |

**The capacity columns land on `gov_approvals`, not on `audit_events`** (§4.0.3). Two nullable additive
columns, following the precedent in the same table:
`ALTER TABLE gov_approvals         ADD COLUMN target_ref UUID;` (`0164_bind_consume_four_eyes.sql:34`, with
its `gov_approval_requests` sibling at `:33`), whose own comment reads *"target only. No FK — like
`request_ref`, it is a logical ref across lanes."* — the same posture `on_behalf_of_party_id` takes.

**The invariant, stated once so no implementer reads capacity as a relaxation: capacity refines a
signature; it never satisfies a four-eyes gate.** `CHECK (approver_id <> requested_by)`
(`0153_create_governance.sql:74`) is retained **verbatim**, and it is one of **five DB-enforced four-eyes
CHECKs in this tree** — `0153:74`, `0122_create_leave_requests.sql:63`,
`0163_finance_gl_voucher_sod.sql:25-27`, `0186_payroll_run_lifecycle.sql:39`,
`0191_create_inventory_cycle_counts.sql:46`. **None of the five becomes conditional on capacity.** A 대리
signature is still a distinct `approver_id` from `requested_by`; recording *on whose behalf* it was taken
does not make it the same person's signature twice.

While `party` is deferred (below), `on_behalf_of_party_id` carries **no FOREIGN KEY** — a bare nullable
`UUID`. D1's non-foreclosure constraint forbids a cross-tenant identifier as a FK regardless, so this is not
a temporary shortcut waiting on `party`; the absent FK is the decision, and it is stated here so a later
lane does not "complete" it.

**`work` is Tier T, not Tier N** (§0.14). It needs indexed aggregate SQL for cycle-time and
cost-rollup metrics, and there is no materialized read model to bridge a revision fold to that (§0.8).
It is added to `allowlisted_projected_table` (`instances.rs:1479-1498`) — **one match arm** — so the
canvas sees it. `work_orders` (`:1481`) and `employees` (`:1482`) are already there; this is the proven
pattern, not a new one.

Fields on `work` that are genuinely new and not projections of any component: `estimated_effort`,
`actual_effort`, `due_at`, `sla_breach_rule`, `cost_rate`, `planned_start`/`planned_end`,
`realized_start`/`realized_end`. Dependencies between work items are an `object_links` row, not a
column.

`party_org_visibility` carries `org_id NOT NULL` + `ENABLE`/`FORCE ROW LEVEL SECURITY` on
`app.current_org` — the ordinary Tier T shape, no allowlist entry, no gate exception. It also carries
`relationship_kind`, `valid_from`/`valid_to`, `created_by`, and a mandatory `reason` (the shape
`clearance_assignments` already proves at `0147:14-32`).

`worksite_registration` is Tier T rather than Tier N precisely because 사업자등록번호 needs a UNIQUE
constraint and payroll needs an FK — neither expressible in an attribute bag. It is projected into
Tier P so the canvas can see it.

`users.employee_id` (`0076:13-20`) stays as-is. `employees` becomes the **per-tenant HR projection of
a party**, not a rival identity — which is also where its personal attributes keep living, already
under RLS (`0063:21-25`).

#### Tier N — ontology instance types (canvas-editable and replayable for free)

| Entity | Purpose | Identity | Lifetime | Slice 0 |
|---|---|---|---|---|
| **`grant`** *(`scope.level` ∈ {`Org`, `Region`, `Branch`, `Worksite`} — the shipped `AccessScopeLevel` minus `Group`)* | binds *(subject party, capability, scope, source)*. **`Group`-scoped grants are NOT here** — they are Tier O, above | instance id | effective-dated; revocation closes `valid_to` | yes — 2, both `Worksite` (현장) scope |
| **`org_unit`** | 조직 structure, with **kind** (부서/팀/TF/사업장) and **lifetime** | instance id | permanent, or derived from a contract (§5.10) | yes — 1 (kind = 사업장) |
| **`delegation_rule`** (전결규정) | (category × amount band × raising scope) → competent unit, terminal? | instance id | effective-dated | yes — 1 row, 1 band |
| **`approval_template`** | per document class: ordered/parallel steps, each with competent-unit-by-lookup + required capability + mode | instance id | versioned by the registry | yes — 1 step |
| **`approval_line`** | a raised document's line. Stores **line-as-raised AND line-as-executed** | instance id | raised → in_progress → closed → **confirmed** | yes |
| **`employment`** | revise the shipped fixture type | instance id | a period, with terms | no — W2 |
| `authority_rule` | predicate → grant generator over the four dimensions | instance id | effective-dated | no — W5 |
| **`position`** | 직책 **at a scope** — a post that exists unoccupied | instance id | permanent, or bounded with its unit | **slice 1** |
| `assignment` | party ⟶ position, or party ⟶ work, for a period. Carries an **assignment kind** ∈ {substantive, acting, seconded} and a **return-right marker**, both authored properties | instance id | a period | **slice 1** (position), W4 (work) |
| `contract` / `contract_line` | 계약 and its declared quantity (the parent of a lot tree) | instance id | a term | no — W16 |
| `employment_type` | authored type with per-type accrual/insurance/severance rules | instance id | effective-dated | no — W6 |
| `group_designation` | the **derived** effective-dated group fact, with its reason | instance id | recomputed on control-edge change | no — W7 |
| `correction` | the compensating revision a post-확정 반려 emits (§5.2) | instance id | immutable | no — W14 |

Every one of these is Tier N for the same reason: it is authored, lives inside one tenant, and needs
effective-dated replay — all three of which `0155:37-64` already provides. None of them earns a
migration.

**There is no `approval_signature` entity, and an earlier draft of this plan invented one.** The signature
store is `gov_approvals`, already shipped (§4.4). Storing a signature in `ont_instance_revisions.attributes`
would be a strict **regression**, not a reuse: a JSONB attribute bag under `ON DELETE CASCADE` on org
(`0155:37-56`) loses the `(approver_id, org_id) REFERENCES users(id, org_id)` FK (`0153:79`), the
self-approval CHECK (`0153:74`), the single-use `UNIQUE (org_id, request_ref)` index (`0153:76`) and the
`ON DELETE RESTRICT` durability posture that every shipped four-eyes gate binds against. The error was
copying `0153`'s own inline comment `-- one decision per request` instead of reading its caller — see §4.4.

Three notes where Tier N bites:

- **`employment` cannot link to `party` with an `ont_link`** (`0155:76-77`). `party_id` is an
  attribute-bag **property** holding a UUID, app-validated. The shipped precedent for an untyped
  cross-store pointer is `recruit_postings.position_ref TEXT` (`0187:29`), commented *"optional
  ontology position instance ref"*. Same seam, opposite direction.
- **`employment` must separate employer from worksite** (파견 puts them in different legal entities):
  `employer_party_id` (property → `party`) and `worksite_org_unit` (`ont_link` → `org_unit`). The
  split is already half-anticipated in `recruit_postings`, as two free-text columns `company` and
  `worksite` (`0187:20-21`).
- **Replace `person_name`.** `fixtures/employment.rs:142` declares `title_property_key:
  Some("person_name")` and `:148` declares it a string property. The occupant becomes `party_id`;
  `person_name` becomes a display projection, not identity.

#### Deferred with its constraint stated, not its schema

**`party_link`** — control edges `(holder_party, held_party, basis, share_bp, valid_from, valid_to,
reason)` between LEGAL parties. ManyMany; **cycles permitted** (순환출자); group designation is
derived, never stored. Its rows must be **read across orgs** by the group-designation derivation, which is the
one property `party` does not have, so it is **Tier O** and reuses the shipped `group_role_grants` definer
pattern (§4.2) — not a "`party` definer", which this plan does not build (§4.1). Slice 0 does not touch it and
neither does any widening before W7; designing its DDL now would be speculative.

### 4.2 Why there is no second tenancy dimension

**Independently confirmed by a shipped spec.** `docs/specs/org-hierarchy.md:3-7` self-declares P0-P3 **IMPLEMENTED**
(`backend/crates/platform/group/src/lib.rs` exists) and states the posture verbatim: *"the per-법인
`app.current_org` boundary is **UNCHANGED**. This spec adds a controlled cross-entity scope **above** that
boundary; it never punches a hole in it."*

That is the same move this section makes for `party`, already executed once. Requirement 4 (org plane vs
group plane) is therefore **reuse, not design**: `org-hierarchy.md:172-173` specifies
`AccessScope { level ∈ {Group, Org, Region, Branch, Worksite} }`, and `:241-246` protects the separation
with an invariant — *"A GroupFeature NEVER confers a tenant Feature; a tenant Role NEVER confers a
GroupFeature"*. Adopt both.

One line must give way: `org-hierarchy.md:96` `PRIMARY KEY (group_id, org_id), UNIQUE (org_id)` — *"an Org
is in at most ONE group"* — is exactly what forbids joint ventures, so W7's control edges supersede **that
line specifically** rather than the spec. And `:298` risk 4 puts **intercompany/elimination out of scope
(Track C)**, which is the citation that bounds §5.5's peer plan.



This is the plan's central claim, and it is the reason §0.1 matters.

**X4 confirms it, and confirms it in a bounded form that must be stated with it.** What was measured — 30
assertions, 3 controls RED, zero new GUCs, the 141 RLS policies untouched — is **visibility of a known party
within the armed org**. X4's own scope line says so: *"Schema-level only. No `organizations` FK, no
Cedar/PBAC layer, no application path."* It does **not** extend to **cross-org authority resolution**, which
X4b then measured **failing**.

**The falsifying case, stated honestly rather than left for an implementer to hit.** Armed to org A, resolve
the eligible approvers for a step whose competent unit sits at **group** scope, where the only qualifying
holder is a user of org B. On this design that question is **not answerable** without either iterating the
member orgs or reading a Tier O group-scoped grant store (§4.1) — because a group-scoped grant cannot live
in Tier N at all, and a sibling org reads zero rows from the one that does exist. So "no second tenancy
dimension" is true, and "everything resolves in one query under `app.current_org`" is not.

**Which of the two readings of the definer this plan means: Variant B.** The party/edge join runs **inside**
the definer, which re-derives the org floor from `current_setting('app.current_org')` and never accepts an
org from the caller. Under the tier decision of §4.1, Variant A (the join outside, the definer answering
narrow questions) is not merely undesirable — **it is not implementable**: the outside half of the join would
read `party` as `console_rt` armed to a tenant, and `org_isolation` returns **zero rows** there. Variant A was
already rejected on measurement, and the measurement is why the handle may never be granted to `console_rt`
unfiltered: X4 measured `SELECT count(*) FROM x4probe_party` returning **`2`** where org A held
an edge to only one — and DN-0003 invariant 5 makes that a violation, not an inelegance: *"Denied data is
omitted, including counts and relationship existence"* (`DN-0003:85-86`). The sentinel-homed row satisfies
invariant 5 in its own terms: the count is **omitted** (0), not refused with an error.

The confidential fact is not *"who is this party"* — it is *"which parties does org A hold edges to"*.
That fact lives in `party_org_visibility`, which names exactly one `org_id` per row. Ordinary
`app.current_org` RLS therefore gives the whole requirement: org A reads only its own edges and
**cannot observe that org B holds an edge to the same party**. The `party` row itself is homed at the platform
sentinel org under the same policy, so a tenant-armed read returns nothing and a tenant-armed INSERT is
refused by the policy's own `WITH CHECK` (§4.1) — **one mechanism covering both the handle and the edge**,
which is why no second one is specified.

Consequences: **zero new GUCs. Zero changes to the 141 RLS policies. Zero new gate classifications.**
The input's "largest single engineering cost" does not arise.

The one new mechanism is a resolver, and its pattern is already shipped:
`group_role_grants_for_user` (`0060:99-126`) — `SECURITY DEFINER`, `SET search_path = public,
pg_temp`, `SET LOCAL row_security = off`, a narrow parameterised query, `SET LOCAL row_security = on`,
an `EXCEPTION WHEN OTHERS` handler that restores it before re-raising (`0060:90-92`),
`REVOKE ALL … FROM PUBLIC`,
`GRANT EXECUTE … TO console_rt`, and an `-- rls-arming: ok` marker for the gate at
`backend/ci/gates/rls-arming/src/lib.rs:69`.

**One deliberate deviation from that precedent, and it is a security fix.**
`group_role_grants_for_user(p_user UUID)` (`0060:99`) resolves "own grants only" by trusting the
caller to pass its own user id — the function itself never reads `app.current_org`. **`effective_grants_for`
must not copy that** (there is no separate "party resolver" — the handle needs none, §4.1): it filters on
`current_setting('app.current_org')`, not on a parameter, or any org reads any org's grants. Stated here
because copying the precedent verbatim is the likely failure, and `definer_ignores_parameter_org` (§7) is the
probe.

### 4.3 Relationships

Cardinality uses the engine's own vocabulary, `LinkCardinality::{OneOne, OneMany, ManyMany}`
(`ontology/domain/src/lib.rs:184-188`); `ont_link_types.cardinality` CHECKs the same three at
`0152:77`.

| Relationship | From → To | Card. | Tenant-scoped | Stored as | Owner |
|---|---|---|---|---|---|
| `party_visible_to_org` | party → organization | ManyMany | **yes** (T) | `party_org_visibility` row | tenant |
| `account_of` | users → party | OneOne per org | yes | `users.party_id` — a bare `UUID`, **no FK** (§0.4, constraint 1) | tenant |
| `hr_record_of` | employees → party | OneOne per org | yes | `employees.party_id` — a bare `UUID`, **no FK** | tenant |
| `controls` | party(LEGAL) → party(LEGAL) | ManyMany, cyclic | no (O) | `party_link` row | platform |
| `parent_org_unit` | org_unit → org_unit | OneMany | yes | `ont_link` | tenant |
| `worksite_legal_reg` | org_unit → worksite_registration | OneOne | yes | FK (T) + projection | tenant |
| `grant_subject` | grant → party | OneMany | yes | property (UUID) | tenant |
| `grant_scope` | grant → a scope **descriptor** | OneMany | yes | **property `{level, node_id}`** — `AccessScope`, never an `ont_link` | tenant |
| `grant_source_assignment` | grant → assignment | OneOne | yes | `ont_link` | tenant |
| **`position_at_scope`** | position → a scope **descriptor** | OneMany | yes | **property `{level, node_id}`** — `AccessScope`, never an `ont_link` | tenant |
| **`holds_position`** | party → position | **ManyMany** | yes | `assignment` instance | tenant |
| `derived_from` | lot → lot, **quantity-bearing** | **ManyMany** | yes | `lot_split` row (T) | tenant |
| `declares` | contract_line → lot (the root) | OneMany | yes | FK | tenant |
| `work_scope` | work → a scope descriptor | OneOne | yes | **property `{level, node_id}`** on the `work` row — never an `ont_link` | tenant |
| `work_origin` (발생지) | work → a scope descriptor | OneOne | yes | **property `{level, node_id}`** on the `work` row | tenant |
| `work_performed_at` (수행지) | work → a scope descriptor | OneOne | yes | **property `{level, node_id}`** on the `work` row | tenant |
| `work_jurisdiction` (결재 관할) | work → a scope descriptor | OneOne | yes | **property `{level, node_id}`** on the `work` row | tenant |
| `work_assignee` | work → party | ManyMany | yes | property set + `assignment` | tenant |
| `work_artifact` | work → email_thread \| document \| … | ManyMany | yes | `object_links` row | tenant |
| **`person_artifact`** | person → email_thread \| document \| … | ManyMany | yes | `object_links` row on the **seeded `person` kind** (`0102:32`) | tenant |
| `competent_for` (전담) | org_unit → (category, band, scope) | ManyMany | yes | `delegation_rule` | tenant |
| `line_step` | approval_line → step | OneMany ×2 sets | yes | `ont_link` (raised / executed) | tenant |
| `step_edge_kind` | step → {결재, 협조, 보고} | — | yes | property on step | tenant |
| `signature_grant` | `gov_approvals` → grant | OneOne | yes | `gov_approvals.authorizing_grant_id` column (T) | tenant |
| `signature_on_behalf_of` | `gov_approvals` → party | OneOne, nullable | yes | `gov_approvals.on_behalf_of_party_id` column (T), **no FK** | tenant |
| `obligation_notice` | approval_line → notices | OneMany | yes | `notices` FK | tenant |

**No *reachable* path writes an `ont_links` row without a property carrying
`config.link = {stable_key, to_type}`** — §0.12. A link type alone writes no edge on any path a user or an
API caller can reach. This is a hard specification, not a style preference. **The one exception, named so
nobody discovers it and thinks the rule is false:** `PgInstanceStore::create_link`
(`backend/crates/ontology/adapter-postgres/src/instances.rs:291`, INSERT at `:319`) writes a row directly
from a bare `link_type_id` — but every call site in the repo is under `tests/`, so it has **zero non-test
callers**. `grep 'INSERT INTO ont_links'` finds **two** sites and only one of them is the property
mechanism.

**Why every scope edge above is a property and not an `ont_link` — measured, not argued.** An `ont_link`
endpoint must be an `ont_instances` row **in the same org**: `0155:76-77` FKs both
`(from_instance_id, org_id)` and `(to_instance_id, org_id)` to `ont_instances(id, org_id)`. Neither an
`organization` nor a `group` is an `ont_instances` row, so `grant → group` and `grant → organization` are
**not storable at all**. X4b CASE 3a executed the write and got
`ERROR: insert or update on table "ont_links" violates foreign key constraint "ont_links_to_instance_id_org_id_fkey"`.
The `org_unit` arm *is* storable, but it is made a property too, so there is **one** storage form and the
fold reads `AccessScope` uniformly instead of branching on how the scope happened to be recorded. The
vocabulary is the shipped one: `backend/crates/kernel/core/src/access_scope.rs:28-34`
`enum AccessScopeLevel { Group, Org, Region, Branch, Worksite }` and `:37-40`
`struct AccessScope { level, node_id }`, specified at `docs/specs/org-hierarchy.md:172-173`.

> **Caveat, and it is a hard one: Slice 0 must not publish a `grant_scope` link type whose declared target
> set includes the `groups` or `organizations` table** (the *tables*, not the `Group`/`Org` enum variants —
> the two vocabularies are distinct, §4.1). That arm fails at the first write, and by then the schema is
> already published — an authored type's published schema is the expensive thing to withdraw, not the row.

**Why the four `work_*` edges are properties: `work` is Tier T projected, and a projected type owns no
instances.** `backend/crates/ontology/adapter-postgres/src/instances.rs:1443-1450` states it: *"A
`projected` object type owns no store of its own … This is a READ-ONLY view … there is no create/stage path
here, only list."* So `work` has no `ont_instances` row to be an `ont_link` endpoint, and all four edges are
rejected by referential integrity **today**. They become scope-descriptor properties on the `work` row — or
an `object_links` row, the shape `work_artifact` already uses.

Six of these deserve their reason stated:

- **`holds_position` is ManyMany, and `position` belongs to a scope — not to a person.** 직책 is not a
  global attribute. A guild officer post and an alliance officer post are different posts held
  concurrently; corporately, 부장 at a subsidiary and 재무관 at the group are two positions with two
  grant sets. `position_at_scope` is what makes 겸직 and 파견 expressible, and ManyMany is what makes it
  *concurrent* rather than sequential. Checked against my own draft: no cardinality here assumed one
  position per person, so no correction was needed — but the scope link was implicit and is now
  explicit.

- **`work_artifact` uses `object_links` (`0102:54`), not `ont_links` — and a new edge kind IS a
  migration.** The storage choice stands: `object_links` addresses endpoints as `src_kind`/`src_id` and
  `dst_kind`/`dst_id` (`0102:57`, `:59`) with **no FK to either endpoint id**, which is exactly a
  work→artifact edge across heterogeneous stores, already
  `UNIQUE (org_id, src_kind, src_id, dst_kind, dst_id, link_type)` (`0102:68`). Both `person` and `org_unit`
  are already seeded kinds (`0102:32-33`); `work` needs one appended row.
  **The reason previously given for it is struck.** The plan said *"`link_type` is validated only by slug
  regex (`0102:63`) — so a new edge kind needs no migration."* That went stale at `0130`/`0132`:
  `0130_create_link_types.sql` created the registry, seeded **twelve** labels (`:37-49`, none of them
  `work_artifact`), granted `console_rt` **SELECT only** (`:52`), and added
  `object_links_link_type_fkey … ON DELETE RESTRICT NOT VALID` (`:75`), validated by `0132:8`. `console_rt`
  cannot INSERT a `link_type` — measured: *"permission denied for table link_types"* (X4b S1). So **a new
  edge kind is one appended `link_types` row per kind in a migration**, exactly like the `object_types` kind
  row this plan already budgets. Carry the per-kind cost D3 names: one `RESOLVABLE_KIND_AUTH` row, one
  `resolve_*` arm, and **a one-time audit of pre-existing links of that kind**, because registering a kind
  makes prior links of it retroactively resolvable with no backfill re-check.
- **`person_artifact` is declared, not left to a probe to discover.** The PII and handover boundary rests on
  the distinction between an artifact linked to a **work** and one linked to a **person**, and until now that
  distinction existed only inside §4.5's 인계 완료 clause 2 and the `handover_moves_work_artifacts_only`
  probe. It is an `object_links` row on the already-seeded `person` kind (`0102:32`), ManyMany, owned by the
  tenant, and 인계 완료 clause 2 now references it by name.
- **Three edge kinds, not one with a flag.** 결재 / 협조 / 보고 differ in direction and meaning; 보고
  is the return leg C and D owe B. A flag on a signing edge cannot carry a reverse-direction
  obligation.
- **합의 is a parallel branch, so a step index cannot express the line.** Steps carry a
  `branch_group` and a `mode ∈ {serial, parallel_합의, terminal_if_전결}`. This is precisely what
  `work_order_approval_steps.step_order SMALLINT CHECK (step_order BETWEEN 1 AND 3)` (`0008:62`)
  forecloses.

### 4.4 What each existing mechanism can and cannot absorb

**The heading used to read "why the existing mechanisms cannot be widened", and three of the four rows below
contradict it.** `gov_approvals` **is** the signature store and absorbs capacity with nothing relaxed;
`policy_role_conditions` is **narrowed at the write path** while its DB CHECK stays as the additive extension
point; `notices` is **extended** and no second ack mechanism is built. Only
`work_order_approval_steps` is genuinely un-widenable, and it is left alone. The blockers below are
structural where they are structural and additive where they are additive, per row — which is the whole
point of stating them per row.

| Mechanism | Executable blocker | Verdict |
|---|---|---|
| `work_order_approval_steps` | `step_order SMALLINT CHECK (step_order BETWEEN 1 AND 3)` `0008:62`; `role CHECK (role IN ('MECHANIC','ADMIN','EXECUTIVE'))` `:63`; `UNIQUE (work_order_id, role)` `:71` | 3 steps max, demo roles, each role once, serial only. 합의 inexpressible. **Leave alone.** |
| `gov_approvals` | `FOREIGN KEY (approver_id, org_id) REFERENCES users(id, org_id)` **`0153:79`** — the approver **must** be a user of that org, so a group-level approver is forbidden by the FK. (`:78` is the `requested_by` FK; three sites in earlier drafts cited `:78` for the approver, and §4.1 already cited `:79` correctly — one fact, two line numbers, now one.) `UNIQUE (org_id, request_ref)` `0153:76` is **not** a blocker — see below | **This IS the signature store.** The cross-org FK is the real limit and it stands. Capacity costs two nullable columns and **nothing has to be relaxed.** |
| `policy_role_conditions` | `attribute CHECK` over **17** literals `0065:110-127`; `operator CHECK (operator IN ('equals','not_equals','in'))` `0065:129`. **The resolver evaluates far less than the CHECK admits:** `backend/crates/platform/authz/src/lib.rs:1404-1430` returns `None` unless the operator is `equals`\|`in` **and** the attribute is `branch`\|`team` | **Narrow the WRITE PATH to `{branch, team}` × `{equals, in}`**, with a test asserting write-accepted ⊆ resolver-evaluated; leave the DB CHECK permissive as the additive extension point. `not_equals` stays writable-but-unwritten rather than removed. **Two of the four dimensions have no substrate at all** — see below |
| `notices` / `notice_receipts` | `notice_receipts` is `(id, org_id, notice_id, recipient_user_id, acknowledged_at, created_at)` `0162:41-51` — **no content column**; `notices.status CHECK (status IN ('draft','published'))` `0162:22` — **no closure state**; `FOREIGN KEY (recipient_user_id, org_id) REFERENCES users(id, org_id)` `0162:50` — **recipient must be a user of that org**; and **no per-recipient audience targeting at all** — see below | 통지 → 인지 is built. **Four** gaps, two of them structural. **Extend; never build a second ack mechanism.** |

**`policy_role_conditions`: the vocabulary is narrower than "already exists as data", in two ways.**

*(a) The resolver's fail-closed whole-role void is CORRECT and must never be relaxed.* When
`effective_scope_for_custom_role_conditions` returns `None`, the caller at
`backend/crates/platform/authz/src/lib.rs:1350-1360` does `else { continue; }` — **it drops the whole role**, not
the one condition it could not evaluate. That is the safe direction, and it must never be "improved" into
per-condition ignoring, which would silently grant a role whose narrowing condition the evaluator did not
understand. The contrary comment at `0065:101-103` — *"unsupported conditions remain review/audit metadata until
a richer evaluator lands"* — is **struck**: it describes the conditions as inert, when in fact they void the
role. Both readings are fail-closed; only one is true, and the false one invites the relaxation.

**Competence** enters as a **subject-side condition attribute** in exactly the shape the `"team"` arm already has
(`authz/src/lib.rs:1421-1425`) — not a third relation, and not a change to the scope type.

A read-only **inert-condition census (X-T2f)** must run **before** the narrowed write path ships: today's rows may
already carry operators or attributes the resolver voids, and narrowing the write path does not migrate them.

*(b) The plan's four dimensions do not all have a substrate.* `0065:110-127` holds **17** attribute literals and
**직무 and 직급 are not among them.** So 소속 (`organization`, `department`) and 결재선 (`team`, `position`,
`assignment`) are expressible **as condition attributes** — a different vocabulary from `AccessScopeLevel`, and
the confusion of the two is what put a non-existent `org_unit` scope level into an earlier revision (§4.1). Note
that `department` is in this list while **no scope level names a department**, and that the resolver voids a
`department` condition anyway (above). **직무 / 직급 are in neither vocabulary** — widening that CHECK is a migration, and it would be a
**third** closed vocabulary. §1 principle 3 names all four as though all four were data; they are not.
**Decision: 직무 and 직급 have no substrate in slices 0 or 1, and they arrive in W6** with `employment_type`,
where the accrual/insurance/severance rules that need them already land. They are not built here.

**`gov_approvals` already runs an N-node 결재 line, and the plan misread it.** The blocker cell used to
read *"`UNIQUE (org_id, request_ref)` — one decision per request"*, which would have made a multi-step line
inexpressible and is the reason an earlier draft invented an `approval_signature` entity. It is wrong.
`backend/crates/orgchange/adapter-postgres/src/lib.rs:1477-1488` — the newest approval domain in the repo —
INSERTs into `gov_approvals` binding `request_ref` to **`step_id`** and `requested_by` to
`request.drafted_by`, with its own comment *"Record the decision through `gov_approvals` so the DB-level
approver <> requester CHECK is the second SoD net."* So an eight-step `org_change_request` writes **eight**
immutable rows. The constraint is **one signature per node**, not one per request.

**Name the failure mode, because it is the same one `ADR-0002` produced.** The plan cited the migration's
own inline comment (`-- one decision per request`, `0153:76`) instead of reading the caller. A comment in a
migration is prose about code, exactly like an ADR Decision line, and it was wrong in the direction that
invents work.

**What capacity costs here: two nullable columns, and nothing relaxed.** 전결 by delegated authority is
already two rows (same `approver_id`, different `request_ref`, same `requested_by`), and 전결 where the
competent authority **is** the drafter needs **zero** approval nodes — the self-approval CHECK is never
reached because no signature is required. The cross-org approver FK (`0153:79`) remains the one real blocker,
and W1 is where it is addressed.

**`notices`' fourth gap is a confidentiality regression, not a missing feature.** W1's party-keyed recipient
fixes the cross-org FK; it does **not** fix the **org-wide fan-out**.
`backend/crates/notices/adapter-postgres/src/lib.rs:413-433` publishes through two SQL variants keyed on
`audience_scope == "branches"`, and **both end `WHERE … org_id = $1 AND is_active = true`** — the branch variant
joins `user_branches` and `notice_audience_branches` but is still org-wide *within the selected branches*, and
the else-branch is `SELECT $1, $2, id FROM users WHERE org_id = $1 AND is_active = true`, i.e. every active user
in the org. So until this is fixed, **a 반려 notice on a 결재 matter reaches every active user in the org.**

The fix is **per-recipient audience targeting** with its own DDL: an explicit recipient list keyed by party
(`notice_audience_parties (org_id, notice_id, party_id)`), with the snapshot INSERT selecting from it rather than
from `users`.

**And that DDL must be ADDITIVE, because both tables are gate-marked audited.** `0162:12` is
`-- console-gate: audited-table notices` and `0162:40` is `-- console-gate: audited-table notice_receipts`,
which `discover_audited_tables` (`backend/ci/gates/migration-safety/src/lib.rs:174-187`) folds into the same
set as the five built-ins — so **`DROP COLUMN` on either is a gate violation**, exactly as for `audit_events`
(§4.0.3) and the voucher (§5.5). `notice_receipts.recipient_user_id` is `NOT NULL` (`0162:45`). So W1's
"party-keyed recipient **replacing** the org-composite FK" is precisely: `ALTER COLUMN recipient_user_id DROP
NOT NULL`, `DROP CONSTRAINT` on the composite FK, `ADD COLUMN recipient_party_id UUID` — and **never** a
`DROP COLUMN`. The new `notice_audience_parties` table is greenfield and therefore free. Recorded here because
the word "replacing" in W1 reads as a column swap, and a column swap is refused by a gate. And `obligation_notifies_line_as_raised` (§7) must assert that **non-members receive nothing**,
with **the shipped org-wide snapshot as its known-bad control** — the probe as previously written asserts only
that truncated member D *is* notified, which the current org-wide fan-out satisfies trivially.

### 4.5 The traversals

Concrete paths for the operations that matter.

**Resolve effective authority** — `effective(party, scope, asof)`:
```
definer effective_grants_for(p_party, p_scope, p_asof)          -- row_security off, re-validating
  → visibility_predicate  WHERE org_id = current_setting('app.current_org')  -- the visibility gate
                            AND party_id = p_party AND asof ∈ [valid_from, valid_to)
     -- table: party_org_visibility once it lands; the subject's own `users` row in Slice 0 (§5.1)
  → BRANCH ON p_scope.level        -- the shipped AccessScopeLevel, NOT a DSL (§0.17)
                                   -- exactly five variants; there is no org_unit level (§4.1)
    ├ Org | Region | Branch | Worksite:
    │   → grant instances     WHERE org_id = current_setting('app.current_org')   -- org_predicate
    │                           AND subject = p_party
    │                           AND scope ⊇ p_scope
    │                           AND asof ∈ [valid_from, valid_to)
    │   → grant revisions     WHERE org_id = current_setting('app.current_org')   -- org_predicate
    └ Group:
        → the Tier O group-scoped grant store, through ITS definer (§4.1)
          — NOT Tier N: a group-scoped grant cannot live in ont_instances at all,
            measured in X4b, and a sibling org reads zero rows
  → fold: union of capabilities. No deny rows. No subtraction.
  → re-validate: the four named checks of §5.1
      (org_predicate, visibility_predicate, chain_linkage, scope_containment)
```
The `org_id = current_setting('app.current_org')` predicate on **both** grant reads is not decoration. The
visibility edge only proves the armed org holds an edge to this party; without the predicate on the grant
read itself the definer returns that party's grants **from every org**, because `row_security` is off. A
party visible in two orgs is the ordinary case this whole design exists to serve, so this is the likely
production failure, not an exotic one. Its probe is `definer_returns_no_foreign_org_grant` (§7).

**Resolve a 결재 line** — at raise time, never at request time:
```
document.category, document.amount, raising org_unit
  → delegation_rule lookup (category × band × raising scope)     -- may resolve above/lateral/BELOW
  → competent org_unit + terminal?
  → approval_template for the document class
  → per step: eligible approvers = effective(·, step.competent_unit.scope) ∩ {step.required_capability}
  → persist line-as-raised (immutable) AND line-as-executed (revised on every re-route)
```

**Hand over work — two operations, not one with a flag.** 대리 and 전보 differ in whether the authority
comes back, and a design with one path gets the revert wrong:

```
대리 (time-boxed, REVERTING — 휴직 / 연차 / 출장)
  open a grant with valid_to SET at creation      -- the revert is the interval, not a task
  work_assignee edges are NOT re-pointed          -- the work stays with the absentee
  every signature taken under it records on_behalf_of = the absent party (§4.0.3)
  at valid_to the grant closes with no operator action; nothing to remember, nothing to forget

전보 (permanent, NON-REVERTING — 전보 / 퇴사)
  close work_assignee(work, outgoing) ; open work_assignee(work, incoming)
  artifacts follow automatically — they were linked to `work`, never to the person
  scope-bounded: only works whose work_scope ⊆ the relinquished scope move
  position-sourced grants close at 발령일 (§5.9); 인계 완료 is asserted, below
  분배 is neither: it opens N assignee edges and closes none
```

**복직** is the 대리 interval simply ending, so it costs nothing — which is the reason to model 대리 as a
bounded interval rather than as a delegate flag someone must later clear. **퇴사** is 전보 with no incoming
party named, which is the case where 인계 완료 matters most and is exactly where it cannot be hard-gated
(below).

**Link an inbound email** — deterministic or manual, never inferred:
```
In-Reply-To / References → existing linked email_thread          → link, carries full prior history
per-work reply address (work-1234@, our mail server)             → link
sender → party (via party_org_visibility)                        → candidate only
otherwise → manual triage queue
  linking is an AUTHORIZATION event: it grants every assignee retroactive read.
  Gate it with the same capability class as granting authority, and audit it.
```

**인계 완료 — one audited assertion, NOT a query, and NOT a hard gate.** The three clauses below are
computed **server-side under a fixed authority** and then **asserted** as one audited record — outgoing
party, incoming party, relinquished scope, and the count **as asserted**:

```
∃ work where work_assignee = departing ∧ no successor edge ∧ work_scope ⊆ relinquished  → incomplete
      (storage: `work_scope` is a scope-descriptor PROPERTY on the `work` row, §4.3 — not an ont_link)
∃ artifact linked to departing via the person-endpoint object_links edge (§4.3) but to no work
                                                                                          → incomplete
∃ obligation loop (§5.2) with an unacknowledged node or no 조치보고                       → incomplete
```

**Offboarding cannot be hard-gated on 인계 완료 in Slice 0.** An incomplete handover is visible and
provable, but not blocking, because a completeness count over heterogeneous artifact edges is
**principal-relative**. `resolve_head` says so in its own comment — *"`Ok(None)` is byte-identical to 'not
found'/'not visible', so no existence oracle"* (`backend/app/src/objects.rs:690-697`) — and DN-0003
invariant 5 makes omission-including-counts **binding**: *"Denied data is omitted, including counts and
relationship existence"* (`DN-0003:85-86`). So two people run this and get two different answers, and the
delta may not be exposed to either of them. A gate that blocks 퇴사 on a number one principal cannot see is
a gate that blocks the wrong people; the assertion records who asserted what they could see, which is
auditable and honest. Hard-gating is a widening (W4) that needs a fixed-authority count first.

**No frequency is claimed for orphan edges.** `object_links.src_id` and `.dst_id` carry **no FK** to any
endpoint (`0102:57`, `:59`), so orphans are structurally possible — but no delete path was traced, so this
plan states the possibility and does **not** assert how often it happens.

**"Why may this person do this?"** — the audit answer, against the signature store that actually ships
(`gov_approvals`, §4.4 — there is no `approval_signature` entity):
```
gov_approvals row → authorizing_grant_id → grant → {source, scope, valid_from, reason}
                  → the grant's revision chain at the signature's decided_at
                  → on_behalf_of_party_id, when the signature was taken 대리
  replay: re-fold at asof = raise time and at asof = decision time; both must reconstruct exactly.
```

### 4.6 How this ties into the engine

The mapping is the input's, and it holds: archetype → object type; entity → instance; component →
typed property / link with cardinality; system → action with dispatch
(`ActionDispatch::{ProjectedUsecase, InstanceRevision}`, `ontology/domain/src/lib.rs:213-218`;
`ont_action_types.dispatch` CHECK at `0152:99`); deterministic replay → effective-dated
fixity-chained revisions where state is a fold.

What the plan adds:

- **Authored types get canvas + replay for free.** Most of the new entities are Tier N (§4.1 is the
  list; no count is restated here, because counts in this plan have rotted twice).
  Their history, as-of queries and tamper-evidence come from `ont_instance_revisions.prev_hash` /
  `row_hash` (`0155:52-53`), the one-open-revision index (`:57-58`) and the as-of index (`:59-60`).
  This is what makes
  *"what could this person approve on 2026-03-01?"* answerable rather than reconstructed.
- **Three entities must be ordinary tables rather than ontology instances, each for a stated reason — and
  all three are Tier T, none is Tier O.** `party` — minting it is a platform-principal write, and every
  ontology write runs on the command pool that is `None` wherever this ships (§8), so a Tier N handle would be
  green on every PR and dead in production; homed at the platform sentinel org it needs no carve-out at all
  (§4.1). `party_org_visibility` — it *is* the RLS-bearing row, and it must be an ordinary Tier T table to get
  the floor for free. `worksite_registration` — needs a UNIQUE constraint and an FK an attribute bag cannot
  express.
- **Actions dispatch, they do not bypass.** Granting, signing, closing, confirming and linking are all
  `ont_action_types` with `dispatch = 'instance_revision'`, except the four-eyes-gated authority
  changes, which are `'projected_usecase'` routing through `gov_approvals` /
  `gov_approval_consumptions`.
- **What Cedar reads, and how.** Cedar decides capabilities; Postgres decides row reach — the
  invariant the coexistence map already states as `rlsHardBoundary`. Today the subject entity carries
  `org`, `roles` (a set of **strings**), `subject_version`, `clearance_keys`
  (`engine.rs:370-391`); the resource carries `org`, `resource_type`, `resource_id`, `branch`
  (`:403-424`); the action id is `Feature::as_str()` (`:430`).
  - The fold's output enters as **two new subject attributes**: `capabilities: Set<String>` and
    `scopes: Set<String>`. `roles` is already a string set, so authored role keys need no schema
    change to that attribute.
  - **All three new attributes must be declared in the bundle schema BEFORE any code reads the fold.**
    `Schema::from_str(schema_src)`
    (`backend/crates/platform/authz/src/cedar_pbac/engine.rs:306`) parses the bundle schema, and
    `Entities::from_entities([subject, resource_entity], Some(&bundle.schema))` (`:449`) **validates against
    it** — so an undeclared attribute fails entity construction, and a failed entity construction
    **denies everything**. This applies to `capabilities: Set<String>`, to `scopes: Set<String>` and to the
    decision-scope resource attribute, not only to the hierarchy. §8 carries it as a **hard Phase-4 ordering
    constraint: `platform/authz`'s schema change lands first**, before any crate reads the fold.
  - **Decision scope is a new resource attribute**, beside `branch` — not a replacement.
    `branch` stays the operational scope (ADR-0003, and the non-null `branch_id` on every operational
    row). This is how decision scope splits from operational scope without touching the floor.
  - Hierarchy requires populating the parent sets at `engine.rs:392`/`:425` and adding the scope
    entities to `Entities::from_entities` (`:449`), plus declaring them in the bundle schema (§0.3).
    Not free.
  - Caching: `crossRequestAllowDecisionCache: false` in the coexistence map already forbids caching
    a fold across requests. Do not add one. Recorded as **N1 (ADR-0032)** so the absence is a decision.

**What already ships declaratively, stated so this plan is not later read as understating the substrate.** Two
**type-agnostic declarative systems are executable code today**, not roadmap:
`sync_property_links_tx` (`backend/crates/ontology/adapter-postgres/src/instances.rs:874`, called at `:723` and
`:836`) and `resolve_derived_attributes_tx` (`:1142`, called at `:681` and `:769`). And plpgsql itself INSERTs a
generic `create` action on publish — `0165:1024-1041` builds `params_schema` and `edits` from
`ont_property_defs` and inserts an `ont_action_types` row with `dispatch = 'instance_revision'`, for any type
that has no instance-revision action yet.

So extensibility is **open in the entity dimension and closed in the verb dimension** — which is DN-0003
invariant 10 **already implemented**, not a gap. The cheap axis is therefore widening the `derive` op set:
`instances.rs:1166` rejects every op but `sum` with *"property '…' declares derivation op '{op}', which this
engine does not implement"*. One arm per op, no new mechanism.

### 4.7 The game-system lens (requirement 8)

MMO guild/alliance systems are working multi-layer authorization systems that millions of untrained
users configure correctly. Where a game solved a problem this plan has, the game's shape is prior art
and the burden is on us to justify deviating. Treated as engineering evidence, three of the mappings
are load-bearing and the rest are confirmations.

| Game system | Corporate | Verdict |
|---|---|---|
| alliance / guild / party | 기업집단 / 법인 / 팀·TF | control edges, `party`(LEGAL), `org_unit.kind` |
| alliance officer / guild officer | group-scope / company-scope grant holder | `position_at_scope` + `grant` — **already ManyMany** (§4.3) |
| **account → many characters** | **`party` → many `users` + many positions** | **validates the keystone** — see below |
| **guild rank editor** | the no-code role canvas | direct prior art for requirement 6 |
| **guild bank limit per rank per tab per day** | **전결규정** | direct prior art; sets the ergonomics bar |
| **guild bank / guild log** | the `record` component, self-describing | **exposes the capacity gap** (§4.0.3) |
| guild / party / whisper channels | 전사 / 부서 / 프로젝트 / DM | `messenger_threads.kind` already has all four (`0012:9`) |
| quest, quest log, inventory | 업무 + linked artifacts | `work` + `object_links` |
| quest share / handoff | 분배 / 인계 | assignee edges (§4.5) |
| raid lead and assist | 대리 / 대결 | `on_behalf_of_party_id` (§4.0.1) |
| buffs / cooldowns | time-boxed grants, 대리 기간, 파견 | grant validity windows |
| reputation / achievements | 평가 (`evaluation` crate) | out of scope; adjacency noted |

**1. The account/character split validates the keystone — it does not contradict it.** One account, many
characters, each in a different guild at a different rank, is structurally identical to one `party`,
many per-org `users` rows (`users.party_id`, §0.4), and many concurrent positions at different scopes.
Games have shipped this for 25 years with untrained users, which is **the strongest evidence cited here**
that `party`-above-`users` is the natural model rather than an architectural indulgence — "strongest available"
would need a ranked corpus, and this plan cites none. Note the game
also confirms the confidentiality design: your guildmates see your character, never your account roster.
That is exactly `party_org_visibility` under RLS (§4.2).

**2. Guild-bank withdrawal limits are 전결규정, and they set the acceptance bar.** "Rank X may withdraw
N **per day** from tab Y" is (role × amount band × category × **period**) → permitted, authored in a grid by
non-technical users. **The per-day half is a real gap, not a rounding of the metaphor:** `delegation_rule`
carries **no periodic or cumulative quota** in §4.1 or §4.3, so as specified it authorises an unbounded number of
₩1,000,000 approvals per day. **Decision: `delegation_rule` gains a nullable `(period, cumulative_limit)` pair**,
null meaning per-transaction only — additive, and it keeps the guild-bank shape whole. It is **not** in slice 0
(one band, one approval), and its widening is W5. **If the 전결규정 authoring surface is harder to use than a guild bank UI, the
design is wrong.** That is a testable bar, not a sentiment (§4.8).

**3. Enforcement is synchronous, at the transaction.** A guild bank refuses the withdrawal when the
button is pressed. It does not permit it and flag it at month-end. So 전결규정 bands are checked **in
the transaction path**, not in reconciliation — a real departure from the common enterprise pattern of
approve → spend → discover the overspend at close. Applied to the proving slice: the ₩100,000 band is
checked **when the purchase is raised**, and `slice0_band_enforced_synchronously` is its probe (§7).

**4. The differentiator is regulation-centric RENDERABILITY, and every probe in this plan misses it.** SAP's
named failure is *"Approval authority as an intersection of process config and role assignment, with no readable
artefact … You cannot print 'what is Kim's authority as of today' … A 전결규정 has legal force; if the system
cannot render it, a spreadsheet becomes the source of truth"* (`docs/ideas/research-sap.md:937-939`). Every
`slice0_*` probe and E1-E6 is **person-centric**: they ask what one person may do. **None renders the
regulation.** So the probe: the complete 전결규정 — (category × band × scope) → competent unit, terminal? —
renders as **one artefact as of an arbitrary date**; known-bad control: routing expressed only inside approval
templates, so the regulation can only be reconstructed by reading every template. Framed as **current-state
renderability** ("as of today"), **not** historical replay — the "as of 2026-07-01" phrasing that appeared in an
earlier draft came from a mis-transcribed quote, and replay is §5.7's concern, not this one.

**Where the lens does NOT transfer, and it matters.** A game rank change is instant and
consequence-free. 강등 under 근로기준법 can constitute 징계 requiring procedural justification (§5.9).
And a game has no PIPA. The lens is prior art for *structure and ergonomics*, never for *consequence*.

**Retracted upstream and not planned:** alliance/guild "tax" as taxation, transfer pricing,
국제조세조정법 or 부당행위계산부인. The corrected reading — complete internal transaction
instrumentation — is the record spine (§4.0) and is in scope. Allocation with a recorded basis is
scoped in §5.5.

### 4.8 Ergonomics as acceptance criteria

Seven criteria — E1-E5 in the table, then E6 and E7 below it. Each states what it costs, because two of them
were assumed free and are not.

| # | Criterion | Substrate | Cost |
|---|---|---|---|
| E1 | **Explainability surfaced, not just logged.** A user sees "you may approve this because grant G at scope S", and symmetrically why not | `cedar_decision_log.determining_policies JSONB` (`0159:29`) already stores the deciding policy ids | **low** — the payload exists; surfacing is UI. **But** `0159:28-30` notes it is empty on deny-by-omission, so the case a user most needs explained has no explanation. Closing that is real work |
| E2 | **A character sheet as the unifying screen** — party, positions per scope, the fold per scope, active 업무, pending 결재, delegations in/out | every §4.1 entity has a home on it; `action-inbox` crate exists | medium. **Ships in W20 with an EXECUTABLE completeness test**: one row per §4.1 entity mapped to its character-sheet section, and `every_entity_has_a_home` fails on an unmapped entity. Previously the completeness test was prose, no widening shipped E2 at all (W17 ships E4, W18 ships E1, W11 ships E6), and §7's `every_entity_declares_its_components` asserts rows in a TSV — not homes on a screen |
| E3 | **Progressive disclosure** — controls shown are the fold | falls out of `effective(party, scope)` | **free** |
| E4 | **Reversible exploration** — simulate a role or 전결규정 change before committing | Two halves exist: the preview→receipt→consume **ceremony** (`policy_assignment_preview_receipts`, `0065:159-172` — stores inputs, never an outcome, §0.11) and Cedar policy **simulation** (`cedar_pbac/authoring.rs` `simulate_inner`), which `ADR-0023:153-154` says this program ships as *"read-only NL rows + simulation"* | **partly new, and the fold simulator inherits NOTHING from Cedar simulation.** `simulate_inner` simulates **policy** decisions over Cedar's own entities; a fold over a hypothetical **grant** set is a different evaluator over a different input, and treating one as a free extension of the other is how E4 gets under-budgeted. Reuse both halves as *surfaces*; build the fold evaluator as new work |
| E5 | **Named entities, not ids** | `ont_object_types.title_property_key` (`0152:23`) and `ont_instances.title` (`0155:21`) already exist | **free** for Tier N; Tier T entities need a display key declared |

**Derived channel membership (E6).** `messenger_thread_members` (`0012:30-36`) is a hand-maintained
roster (§0.10). It becomes a **projection maintained by the assignment action** — not a view, because
the inbox needs the `(user_id, thread_id)` index at `0012:38-39`, and not hand-maintained, because
nobody maintains guild chat rosters by hand. Cost: one write path per membership-changing action, plus
a rebuild-from-graph routine. Generalising `messenger_threads.work_order_id` (`0012:11`) to a `work`
reference is the same change that gives 업무-scoped channels, and the conversation then follows the work
on 인계 for free.

**E7 — the ergonomics criterion §4.7 promises, made a criterion.** §4.7 asserts of the guild-bank comparison
*"That is a testable bar, not a sentiment (§4.8)"*, and E1-E6 contained no such test. Here it is: **authoring one
complete 전결규정 band — (category × amount band × raising scope) → competent unit, terminal? — takes no more
steps and no more distinct screens than setting one guild-bank rank limit**, counted on the shipped surface, by a
participant who has not seen the schema. Substrate: the authored `delegation_rule` grid. Cost: **medium**, and it
is a measurement rather than an opinion — the step and screen counts are recorded in
`docs/specs/known-bad-controls.tsv` beside the probes, with the known-bad control being an authoring flow that
requires editing an approval template per band (which is SAP's failure, §4.7 point 4). Ships in W20 with E2.

**The governing constraint.** Intuitive surface, uncompromised depth. The quest log is the UX; the
ledger and the metrics are what must not be simplified away (§5.7). Any design delivering the metaphor
without them fails requirement 8.

**A realtime feed is an access-control surface**, so it is governed by the same authority model —
and the gate already exists: `audit_stream_event_labels` with
`sensitivity IN ('STANDARD','COVERT','CEO_COVERT')` (`0147:46-55`), deny-by-omission, read against
`clearance_assignments` (`0147:14-32`). The activity feed is `audit_events` projected through those
labels. No new visibility mechanism.

**No AI/LLM judgment.** Game systems are deterministic rule engines, so this lens pushes toward
explicit configured rules and away from inference. Aligned, not in tension.

---

## 5. Resolutions to the hard problems

### 5.1 A — Bootstrap circularity: DECIDED

Two distinct circles; the input names only the second.

**The genesis circle** — the first grant in an org cannot be granted by a grant. **Resolved outside the
tenant plane.** Genesis is a **platform-principal** capability, never a tenant capability: org creation is
gated on `PlatformFeature::TenantCreate`
(`backend/crates/platform/platform-rest/src/lib.rs:574`, on the live route registered at `:235`,
`PLATFORM_ORGS_PATH … .post(create_org)`), and the extension point that mints the genesis grant is the
seed-first-SUPER_ADMIN step already inside `create_org` (`:568`), whose own doc header reads *"POST
/api/platform/orgs — onboard a NEW tenant (the only place org rows are created by the app), seed its first
SUPER_ADMIN, and return a one-time OTP."* So no **tenant**-authenticated path can mint authority from
nothing; the path that can is authenticated as the platform and audited. (The earlier framing — "genesis is
a migration fact" — was wrong: it is a live handler, and calling it a migration would have sent an
implementer looking for DDL.) Every subsequent authority change goes through four-eyes (`gov_approvals`,
`0153:65-79`).

**The read circle** — Cedar gates instance reads, so reading a grant would need a grant. **Resolved by
one definer that re-validates, following the precedent exactly.** The precedent is
`PgCedarPolicyStore::load_enforced_object_policy_blocks`
(`platform/authz-rest/src/store.rs:576-593`): it re-runs the validator, compares canonicality, checks
effect agreement, and **errors the whole load** on any failure — and `0205:69-74` marks that
re-validation a LIVE CONSTRAINT whose deletion *"would kill this justification silently while every
test here stays green."*

`effective_grants_for(p_party, p_scope, p_asof)` takes the same bargain: it reads outside per-type
Cedar gating, and pays on **every** read with four **named** checks. They are named rather than counted so
the number cannot rot and so a probe can delete them one at a time:

1. **`org_predicate`** — every grant instance row and every grant revision row read is filtered
   `org_id = current_setting('app.current_org')`, **a literal, never a parameter**;
2. **`visibility_predicate`** — the subject is proven visible to the armed org by a row filtered on
   `current_setting('app.current_org')`, **never a parameter** (§4.2). While `party_org_visibility` is
   deferred (§4.1) the row is the subject's own `users` row and the predicate is
   `users.org_id = current_setting('app.current_org')`; when the edge table lands the predicate moves to it
   unchanged in form. The check is the predicate, not the table — which is why deferring the table does not
   remove a check from Slice 0;
3. **`chain_linkage`** — each returned revision's `prev_hash` equals its predecessor's `row_hash`
   (`0155:52-53`), erroring the whole load on the first mismatch;
4. **`scope_containment`** — every returned scope is inside the armed org's reachable scope set.

**No cross-request cache** is a stated property of this read (ADR-0021 decision 4), not a fifth check —
there is nothing to delete to make it RED, so it does not belong in a deletion probe.

**Chain linkage, not recomputation — and recomputation is unavailable to anyone.** Hash *recomputation* is
not available in any language until an explicit key sort plus a re-seal lands with a named audit-chain
owner, because canonicalization is insertion-order dependent. The measurement is already in the tree, in
`backend/crates/ontology/adapter-postgres/src/instances.rs`'s **KNOWN DEFECT** block (`:1297-1320`), which
records `cargo tree -e features -i serde_json` yielding *"`serde_json` feature `preserve_order` ←
cedar-policy-core v4.11.2"*, reaching that crate through `console-platform-authz`, so *"`serde_json::Map`
is therefore an insertion-ordered IndexMap, not a BTreeMap, so this canonicalization is order-DEPENDENT"*.
The lock corroborates the edge: `serde_json 1.0.150` lists `indexmap 2.14.0` among its dependencies
(`backend/Cargo.lock:6659-6671`), which is exactly what that feature does. And `revision_row_hash` is
Rust-side SHA-256 over `serde_json::to_vec`, so plpgsql cannot compute it at all. The same block states the
consequence verbatim: *"The suite is green because it does not recompute hashes — not because recomputation
would succeed."* Linkage is what SQL can do, and it is what `backend/crates/ontology/rest/tests/company_conformance.rs` already asserts. If
this plan later wants recomputation, it is a **Phase-0 prerequisite with a named owner**, never a Slice-0
check.

**What the precedent is, and what it is not.** `backend/crates/platform/authz-rest/src/store.rs:576-593`
is a precedent for **re-validation as a discipline**, and **not** a precedent for reading with RLS off: it
is validator re-execution plus canonicality plus effect agreement on an ordinary pooled read, containing no
`row_security` manipulation at all. The one shipped function that does turn `row_security` off —
`group_role_grants_for_user` (`0060:99-126`) — does it on an **owner-only table carrying no RLS policy**,
where the switch is nearly inert. `effective_grants_for` reads Tier T rows that **do** carry policies, so
it is the first place in this repo where the switch is load-bearing. That is why the checks are named and
individually deletable rather than described.

Payment terms, so this cannot rot the way `0205` warns: an `-- rls-arming: ok` marker naming the
bargain (gate at `rls-arming/src/lib.rs:69`), and a test that **deletes each named check in turn and must
go RED** (§7). A re-validation nobody has proven RED is not a re-validation.

### 5.2 B — Finality: **ALREADY DECIDED by ADR-0023.** Delta only.

**Stop deliberating this.** `ADR-0023:79-86` (accepted) decides the answer: a generalized definition
builder for **arbitrary approval-line DAGs** (dynamic 결재선, 검토/승인/합의/참조 roles, enum reasons,
object-link targets) and a **pre-terminal finalization model** — 최종승인 and 수령확인 are pre-terminal
`WAITING` nodes, *"never a reopened terminal run"*, with **사후 반려 modeled as a compensating
document/event**. Gated on an **Engine-Gen** milestone which *"opens with a 1-2 day spike validating this
FSM shape; if structurally infeasible, execution stops and returns to consensus"*.

An earlier draft of this plan deliberated 이의기간 vs execution-triggered 확정 vs an explicit step and
"decided" a 확정 state. That was re-litigating an accepted ADR. The pre-terminal-WAITING model reaches the
same place by a better route — it never creates a terminal state to reopen — so the deliberation is
withdrawn, not merely superseded. 합의 as parallel branches and 결재/협조/보고 as distinct edge kinds are
likewise covered by "arbitrary DAGs" plus the four declared role kinds.

**The delta this plan owns** — each verified absent from the entire ADR set:

| Delta | Why it is new |
|---|---|
| **전결규정 routing** — (category × amount band × scope) → competent unit, terminal | 전결 appears **zero times** in any ADR or note |
| **Capacity on the signature** — `authorizing_grant_id` (§4.0.3) | "capacity" / "authorizing grant" appear zero times |
| **Standing after closure** — line membership survives truncation and demotion (§5.9) | the DAG decision covers routing, not standing |
| **The obligation loop** — 통지 → 인지 → **조치보고** → 종료 over `notices` (§0.6) | the return leg, and its cross-org recipient blocker |
| **Line-as-raised vs line-as-executed** — both stored | required by the obligation loop's notification scope |
| **Release-reset on a band crossing** — a signature is a statement about a document **state**, so an amendment that crosses a `delegation_rule` band **invalidates the signatures taken under the prior band** and re-routes | the DAG decision covers routing and finality, not the invalidation of already-taken signatures. Slice 0's **one band and one step cannot surface this** — with one band there is no crossing — so it is recorded here as a delta the slice does not prove. Its probe's known-bad control: an implementation that keeps signatures valid after the amount is raised across a band |

Inherited and unchanged: `ADR-0018:115-116` requires **fresh passkey step-up** for approval/signature,
role/policy change, payroll/HR/legal, asset ownership, financial/payment and cross-org transfer — every
authority mutation in this plan is in that set. And `ADR-0018:231-233` decides the runtime spine is
**org-local**, with group-wide flows as a *parent orchestration envelope spawning auditable child runs
inside each participating org* rather than shared rows crossing RLS. That is the decided answer to the
group-scope half of 결재, and this plan reconciles **with** it: a cross-company line is an envelope, never
a row spanning tenants.


### 5.3 C — `Feature` sequencing: **ALREADY DECIDED by ADR-0021.** Delta only.

`ADR-0021` (accepted) decides the strangler: **"PBAC via Cedar, not role-string RBAC… Built-in roles and
tenant custom roles are subject inputs/policy bundle generators, not authoritative allow decisions by
themselves"** (decision 1, `:46-48`); the four-mode ladder (decision 6, `:64-66`); compiled-bundle caching
with **no cross-request allow/deny caching** (decision 4, `:55-56`); `authz_subject_version` freshness
bumped synchronously by role/assignment/employment/branch/credential changes (decision 5, `:58-60`); and
**"No live switch in this ADR/G001"** (decision 8, `:70-72`).

The ladder, the end state and the promotion discipline are decided. Not restated here.

**The delta this plan owns:**

| # | Delta | Note |
|---|---|---|
| C1 | A gate asserting `feature_catalog` rows ≡ `Feature::ALL` | makes the enum a vocabulary, not a decision. Known-bad: add a variant without the migration row |
| C2 | `policy_feature_catalog()` (`identity/rest/src/lib.rs:1433-1448`) sources from `feature_catalog` + grant rows, not `Feature::ALL × Role::ALL × permission_for` | the one endpoint still serving the compile-time matrix |
| C3 | **`Feature` the enum SURVIVES** — it is Cedar's action id (`engine.rs:430`), §0.2 | corrects the brief's premise; ADR-0021 never asks for its deletion |
| C4 | `Role` (`authz/src/lib.rs:35`) and `matrix_row` (`:573`) deleted once every map entry reads `cedar_only` | the ladder already gates this |
| C4b | **BLOCKING:** before `Role` dies, **both** derivations of `BranchScope::All` must come from a **built-in `Feature`** — `resolve_branch_scope_in_org` (`backend/crates/platform/authz/src/lib.rs:1472-1483`) **and** `request-context/src/lib.rs:421-422`'s group-proof mint | §0.16. Needs **D2 (ADR-0028)** on `ADR-0003`'s `## Decision`. **The record's trigger is the present-tense `org_id` × `BranchScope` divergence, not this deletion** — so D2 is owed now and C4b does not gate it. Existing SUPER_ADMIN/EXECUTIVE principals need a migration path, and the onboarding seeder (`platform-rest/src/lib.rs:568`) is a write site it must cover |
| C5 | Four new coexistence-map entries: `authority.grant`, `approval.line`, `party.identity`, `work.assignment` | `enrolledDomainMissingEntry: "deny"` already handles the runtime side. **This is not the trigger for D2** — see C4b |

ADR-0021 decision 1 also means configurable roles **partially ship today** — `policy_roles` +
`policy_role_permissions` + `user_role_assignments` (`0065`) are the "policy bundle generators" it names.
The canvas's role half extends shipped data rather than breaking ground.


### 5.4 D — PII: DECIDED — the durable handle holds no personal data

**`party` is `(id, org_id, party_kind, status, created_at)` and nothing else** — `org_id` present only because
the row is an ordinary tenant-classified table pinned to the platform sentinel org (§4.1), which is what earns
it the RLS floor and the tenant-isolation gate's default classification. **No identifying attribute.** All of
them stay in tenant-scoped rows under the RLS floor that already protects them — `employees` (`0063:21-25`)
keeps holding name, and everything more sensitive continues to live in the tenant tier.

Three consequences, and the third is the one that unblocks slice 0:

1. **No new PIPA surface.** Slice 0 stores no personal data anywhere new. It adds a durable opaque
   handle and a tenant-scoped edge. This matters because the HOLD is total and current: the program
   ledger states that *"every jurisdiction `release_disposition`, and every control trace remains
   `HOLD`"* (`docs/program/console-program-ledger.md:327`) and repeats *"Every capability, evidence
   contract, jurisdiction binding, Korea control, review disposition, legal state, release state, and
   exposure state remains `HOLD`"* through its most recent entry (`:420`). Decisively, the ledger lists
   *"규제 PII/multi-jurisdiction (Jurisdiction/Consent/DSR objects)"* under **"Epics (documented,
   later)"** (`:174`) — the PII substrate is deliberately not built. `ADR-0023:157` independently names
   the **multi-jurisdiction PII program** as out of scope. So a design that needed it would be blocked by
   an accepted ADR, not merely unbudgeted; this one does not need it.
2. **Erasure stays possible.** A fixity chain that referenced a person's attributes could not be
   erased without breaking tamper-evidence. Because the chain references only the opaque handle,
   erasure deletes the tenant-scoped attributes and keeps the handle — history stays verifiable and
   the person stays erased. The handle-only design is an **erasure requirement**, not merely PII
   hygiene.
3. **Cross-tenant inference is closed by RLS, not by policy.** The only cross-tenant fact is "the same
   handle appears in two orgs", and no tenant can read another's `party_org_visibility` rows — nor the
   `party` row itself, for the same reason and under the same policy (§4.1). One mechanism, not two.

**What must be true before a `party` row itself holds real personal data** (i.e. before any widening
puts an attribute on `party`):

- the deferred **규제 PII/multi-jurisdiction epic** (Jurisdiction/Consent/DSR objects, ledger `:174`)
  has landed, **and the bar below has been met by a qualified external authority.** The plan previously said
  *"the jurisdiction binding and Korea controls have moved off HOLD"*, which reads as a milestone somebody here
  can reach. It is not one:
- every `party_org_visibility` edge carries a **lawful basis** and a retention clock, not just a
  `reason`;
- 주민등록번호 gets its own design — encrypted at rest, a separate access capability, and its own
  audit stream (the `clearance_assignments` + covert-stream substrate at `0147` is the shape);
- an erasure procedure exists that provably does not break `prev_hash`/`row_hash` continuity.

**The bar, stated verbatim, because it is what a later lane will mistake for satisfiable.** Six controls in
`docs/program/console-jurisdiction-register.json` all carry `release_disposition: HOLD`:
**CTRL-KR-PRIVACY-001**, **CTRL-KR-WORKFORCE-001**, **CTRL-KR-SAFETY-001**, **CTRL-KR-FINANCE-001**,
**CTRL-KR-LOCATION-001**, **CTRL-KR-RECORDS-001**. Two of the six are the ones this plan touches:
**CTRL-KR-RECORDS-001** (approvals, notices, documents, retention) and **CTRL-KR-WORKFORCE-001** (payroll).

`unhold_authority` is verbatim *"Qualified Korea legal/compliance authority with attributable I2/I3
candidate-bound receipt."* And `uncertainty_rule` (`:1186`) is verbatim: *"Missing, stale, conflicting, or
unqualified authority is HOLD; **agents may not invent certainty**."* **A native agent produces only
`I1_NON_INDEPENDENT` evidence**, which is by construction below the bar. So this section asserts **no** compliance
conclusion and proposes **no** unholding; it records what the bar is, so nobody later reads a HOLD as a task.

**Recommendation: never put personal attributes on `party` — and here is what the alternative costs, priced the
way §5.7 prices its three.** The alternative is not "the same design plus four preconditions". It is:

| If attributes go on `party` | Cost |
|---|---|
| the six HOLD controls | must be unheld by a **qualified external authority** with an I2/I3 receipt — not a schedulable engineering task, and no agent here can produce the evidence |
| the fixity chain | a chain over person attributes **cannot be erased without breaking tamper-evidence**, so erasure and audit become mutually exclusive rather than both satisfied |
| 주민등록번호 | its own encrypted-at-rest design, a separate access capability and its own audit stream (`0147` is the shape) — a subsystem, not a column |
| every `party_org_visibility` edge | a lawful basis and a retention clock, retroactively, on rows already written |
| the erasure procedure | must be **proven** not to break `prev_hash`/`row_hash` continuity, which is the recomputation problem §5.1 already shows is unavailable |

**Against that: keeping `party` attribute-free costs one extra join** from the handle to the tenant-scoped row
that already holds the attribute under RLS — a hop this plan already pays for and already lists as a Con in §3.2.
That is the whole trade, and it is why attribute-free is the cheaper **end state** rather than a staging posture.

### 5.5 E — Economics: DECIDED — there is no GL; build the spine, seeded by the voucher

**Correction to an earlier premise in this plan: no general ledger exists.** `finance-gl` is two tables.
Verified absent from all **205** `.sql` migrations as of `8e76dffb4`: `gl_postings`, `journal_entries`,
`gl_accounts`, `chart_of_accounts`, `fiscal_periods`, `trial_balance`. So "make the object a dimension on existing GL
postings" was not available, and the earlier §5.5 built on it.

What actually exists, and its four disqualifying gaps:

| Fact | Citation |
|---|---|
| `finance_gl_vouchers` — header FSM `DRAFT/BALANCE_CHECKED/APPROVED/POSTED/REVERSED`, reversal pointers | `0160:22-50`, `:38-39` |
| `finance_gl_voucher_lines` — `account_code`, `side`, `amount_won` | `0160:57-68` |
| Real enforcement: balance gate + POSTED/REVERSED immutability trigger; lines append-only | `0160:78-118`, `:122-140`, `:161-164` |
| SoD: `approved_by` write-once, `CHECK (approved_by <> created_by)` | `0163:19`, `:25-27` |
| **No business date.** Only `posted_at TIMESTAMPTZ` (`:41`), `created_at` (`:42`) | `0160:41-43` |
| **No account master.** `account_code TEXT` ≤40, no FK | `0160:62` |
| **No currency.** Hard-coded in the column name `amount_won BIGINT` | `0160:64` |
| **No line-level dimension.** The object ref is header-only and untyped | `0160:34-35`, lines `:57-68` |

**Consequence the earlier draft got wrong: period locks cannot apply to a voucher.**
`assert_period_open(tx, domain, date)` (`platform/db/src/period_lock.rs:60-96`) is keyed on a **DATE**,
and the voucher has none. Worse, the lock **does not enforce itself**: nothing in the database applies it
to any other table — the only triggers protect the lock row (`0107:59-88`) — so enforcement is an opt-in
Rust call with **five** non-test sites: `backend/crates/financial/adapter-postgres/src/lib.rs:1254`,
`backend/crates/workflow/adapter-postgres/src/lib.rs:792`,
`backend/crates/orgchange/adapter-postgres/src/lib.rs:611` **and** `:744` (two separate guards, not one),
and `backend/app/src/hr.rs:1706`. **`finance-gl` is not among them.** Omit the call and the write succeeds.
The count is corrected because it is checkable: a lane that greps, finds five, and reads "four" stops trusting
the paragraph — and the paragraph's substantive point is the one thing here that must be trusted.

**Decision: EXTEND the voucher into a posting model; do not build alongside.** Argued, not assumed.

| Option | Verdict |
|---|---|
| **Extend the voucher** ← chosen | It already owns the two hardest parts: a DB-enforced double-entry balance gate and POSTED immutability (`0160:78-118`), plus reversal linkage (`:38-39`) and SoD (`0163:25-27`). Those are the parts that are expensive to get right and easy to get wrong. What is missing — a business date, an account master, a currency column, a line-level dimension — is additive DDL. **That "no production data" claim is an ASSERTION, not a verified fact, and the DDL is IRREVERSIBLE once landed:** `0160_create_finance_gl_vouchers.sql:21` is `-- console-gate: audited-table finance_gl_vouchers`, and `discover_audited_tables` (`backend/ci/gates/migration-safety/src/lib.rs:174-187`) folds that marker into the same audited set as the five built-ins — so `DROP COLUMN` on the voucher is a **gate violation**. Additive here means permanent. |
| Build a parallel spine | Two records of the same money diverge; that is a certainty, not a risk. And it would need its own balance and immutability enforcement, re-deriving `0160:78-118`. **Rejected.** |

**"Two records of the same money diverge" has already happened three times in this tree, and the reconciliation
backlog belongs to the peer plan.** Naming them, because a certainty stated in the future tense reads as
caution rather than as a debt:

| Parallel money store | Guard |
|---|---|
| `equipment_cost_ledger` (`0015:45-58`) | gate-marked audited (`0015:45`) |
| `equipment_3r_dispositions.cost_minor` / `.sale_amount_minor` (`0182:96-97`) | **no period-lock guard** |
| `equipment_3r_rental_cases.monthly_rate_minor` (`0182:33`) | **no period-lock guard** |

The additive delta, smallest first:

1. **`accounting_date DATE NOT NULL`** on the voucher — unlocks `assert_period_open`, and the guard must
   be called in the finance-gl store (the omission above is a live gap, not a new requirement).
2. **Push the object dimension down to the line** and **type it**: `source_object_type` FK →
   `object_types(kind)`, which already seeds `'voucher'` (`0102:40`). Typing it closes the "any string is
   accepted" hole; pushing it to the line is what makes per-object economics a real query rather than a
   header approximation. **`finance_gl_voucher_lines` carries its own audited marker (`0160:56`)**, so these
   columns are as permanent as the header's — the same "additive means permanent" warning, on the table this
   item actually alters.
3. Account master and currency — **the peer plan** (below).

**Because the spine is greenfield, COST-as-a-query is a design freedom taken deliberately**, not a constraint
inherited from an existing GL. Taken: no entity carries a cost or profit column; both are aggregates over lines
dimensioned by `(object_kind, object_id)`. **The claim is scoped to COST.** Revenue and profit need an account
master and a sign convention this plan does not build, so calling them "queries" today asserts a capability the
substrate does not have — three tables already hold money in parallel (above), which is what the unscoped claim
looks like in practice.

**N5's three prerequisites, and they DO block Slice 0** even though the record does not:

1. **`accounting_date DATE NOT NULL`** distinct from `posted_at` — irreversible once landed (above);
2. **a line-level `branch_id`** — without it a posting cannot be attributed to the scope that authorised it;
3. **an `assert_period_open(tx, PeriodLockDomain::Accounting, accounting_date)` caller in the finance-gl store**
   — while `backend/crates/finance-gl/rest/src/lib.rs:28` is
   `const VOUCHER_FEATURE: Feature = Feature::PeriodLockManage;`, i.e. the crate already names period locks as
   its capability while enforcing none.

**확정 requires an open period — decided here, in one place.** A compensating voucher posts with an
`accounting_date` in the **current OPEN period** while referencing the original's date and id; it never posts
into the closed period it corrects. Without this decided in one place **W14 is self-contradictory for its own
case**: the `assert_period_open` guard W14 adds would refuse the compensating posting W14 exists to prove. Add
the locked-period 반려 probe: a 반려 arriving after the period closed must produce a current-period compensating
voucher, and the known-bad control is an implementation that back-dates it.

**Whether one voucher line may be reported against more than one object: NOT decided here, and that is the
point.** Real-versus-statistical assignment and percentage distribution are decisions the **peer finance plan**
owns — §5.5's own must-not-foreclose list already demands *"allocation with a recorded basis"*, which is the
same question. This plan takes the single-valued case (one line, one object dimension) and forecloses nothing:
a distribution table keyed to a line is additive.

**In scope for this plan:** items 1 and 2, plus one posted voucher in slice 0. The ₩100,000 purchase has
a cost and an authorization, so the minimal `economics` component is *in*. **The single posted voucher is not
evidence the dimension shape is settled** — one line against one object cannot distinguish the single-valued
design from the distributed one.

**`economics_is_a_view` has an unmeasured dependency.** It groups by `account_code`, and **X-T9b predicts that
is not reproducible**: `0160:62` rejects blank but stores untrimmed, so `'100'` and `' 100'` are two groups
holding the same account. Free-text-versus-account-master is an open owner question, and the probe inherits it.

**Explicitly a peer plan, argued rather than dropped:** account master / chart of accounts, multi-currency
and FX (the degenerate single-value CHECKs at `0179:68` and `0182:35` on `currency_code`, and `0172:10` on a
column simply named `currency`, show the convention is single-currency throughout), depreciation and accrual posting — today depreciation is only a
synchronous pricing formula on quotes (`0015:16-18`, `:88-90`, computed at
`financial/domain/src/lib.rs:203`, emitted as a `"DEPRECIATION"` quote line at `:214`) and is **never
posted**; nothing schedule-driven writes any ledger despite `workflow_schedules.cron_expr` existing
(`0106:18-22`). Also peer: overhead allocation, shared-service charge-out, inter-company charges.

**What the entity model must not foreclose:** allocated cost is an explicit, audited, reproducible posting
with its basis recorded — never an implicit derivation in a report; asset economics are generated over
time, so the spine must accept scheduled postings rather than assuming one per object; inter-company
charges are allocation with a recorded basis, not taxation.


### 5.6 F — Realtime authority propagation: DECIDED — invalidate, never push the fold

The transport decides this (§0.9). `NOTIFY_PAYLOAD_LIMIT_BYTES = 8000`
(`platform/realtime/src/lib.rs:40`) means a computed capability set cannot travel over
`LISTEN/NOTIFY`. So:

| Question | Decision |
|---|---|
| Change propagation path | grant revision → bump **both** counters (below) → `pg_notify` on a new `authority_changed` channel (a 4th const beside `0012`-era `:37-39`) → WebSocket hub → client re-reads |
| Invalidation key | **per `(org, user)`, not per org.** `policy_versions` is `PRIMARY KEY (org_id)` (`0065:177-181`), so keying invalidation there alone makes **one grant edit invalidate every connected client in a 10k-employee tenant** |
| Which counters a grant revision bumps | **both.** `authz_subject_version` for the subject party's users **and** `policy_versions` for the org. Neither alone is sufficient — see below |
| What the push carries | **ids and the new version only.** Never capabilities |
| What is authoritative on disagreement | **the server, always.** Every protected endpoint re-authorizes — the coexistence map's `serverAuthoritative` invariant. A stale client shows a stale button; pressing it is refused |
| Cache scope | per request, never across requests — `crossRequestAllowDecisionCache: false` |

**The materialise option is deleted, not deferred.** An earlier draft's first row read *"Materialise per (party,
scope), keyed on `policy_versions.version`"*. It contradicted **row 5 of its own table** (per request, never
across requests), §4.6's own text, and `ADR-0021` decision 4's compiled-bundle caching with **no cross-request
allow/deny caching** — which a plan cannot supersede (`README` rule 4). It was also **mis-keyed**, as the
invalidation row now records. This is a **one-row deletion, not a governance question**: no ADR is engaged, and
**N1 (ADR-0032)** records the mechanism so the deletion reads as a decision rather than an omission.

**Why both counters, measured.** Assignment writes bump the **subject** counter
(`backend/crates/identity/adapter-postgres/src/lib.rs:304`, `:672`, `:1606` — `bump_subject_version_tx`), while
role-definition and role-status edits bump only the **org** counter (`:1284`, `:1369` —
`bump_policy_version_tx`). So keyed on either alone, **a whole class of authority change pings nobody.**
`ADR-0021` decision 5 requires both directions: *"Role, assignment, **responsibility**, employment state,
branch/team, or credential changes synchronously bump subject/policy versions so stale subject material cannot
keep granting access"* — and a grant is a responsibility change. Note honestly that **per-org invalidation alone
is strictly coarser**: any grant change would invalidate every party's fold in that org, a cost **X6 does not
measure**. Probe: `grant_write_bumps_subject_version`; known-bad control: a grant write that bumps only
`policy_versions`.

Cost: one `RealtimeEvent` variant (`realtime/src/lib.rs:318-337`), one channel const, one notifier
following `PostgresNotificationNotifier` (`:273`). The `pg_notify` call sites carry
`// rls-arming: ok pg_notify is not a tenant-table read` (`:148`, `:177`) — copy that marker.

`DisconnectReason` / `DisconnectNotice` (`:407`, `:414`) already exist, so a demotion that must force
re-auth has a mechanism.

**What is visible in realtime, and to whom:** presence, who holds which 업무, live 결재 status, the
activity feed — all governed by the same fold, and the feed specifically through the existing
`audit_stream_event_labels` sensitivity gate (§4.8).

### 5.7 G — Replayability versus aggregate metrics: DECIDED — two stores, no read model

Both are required and the substrate resolves neither (§0.8). Rejected: building a materialized read
model — there are zero materialized views in the **205** migrations as of `8e76dffb4`, so it would be the
first, and it would need its own invalidation, rebuild and staleness semantics.

**Split by question, not by entity:**

| Question | Store | Why |
|---|---|---|
| *"What could this person approve on 2026-03-01?"* | Tier N revisions — `grant`, `assignment`, `approval_line` | fold + fixity chain; this is what replay is for |
| *"Average cycle time across 10,000 tasks"* | Tier T tables — `work`, postings | ordinary indexed SQL |
| *"Show me this work in the canvas"* | Tier P projection of `work` | one match arm in `instances.rs:1479-1498` |

Authoritative on disagreement: **the Tier N revision chain for anything authority- or
approval-related; the Tier T row for its own current state.** They cannot disagree about the same fact
because they do not store the same fact — `work` holds task state, the revisions hold who held it and
under what authority. That non-overlap is the design, and the probe that protects it is
`no_duplicated_fact` (§7).

Replayability is not traded away. It is retained exactly where it is load-bearing — authority, 결재
audit, and the quantity lineage — and not paid for where it is not (a task's due date).

### 5.8 H — Quantity-bearing lineage: DECIDED — one table, one edge, conservation by a parent-row lock with a row CHECK beneath it

This is **structural** lineage, not temporal: a DAG of quantity-bearing nodes where one splits into
several and several merge into one. **Do not model it as revisions** — the revision chain gives the
state history of one object; this needs a tree. That is the one place the engine's existing machinery
is the wrong tool.

Name: **`lot`**. Chosen over `allocation` (which reads as the act, not the thing) and `stack` (a game
word that will not survive contact with an accountant).

**Prior art exists and it is closer than I first wrote.** `inventory_consumption_events`
(`0156:81-111`) is already a quantity-bearing, cost-carrying movement event:

- `quantity_before_milli`, `quantity_consumed_milli`, `quantity_after_milli` (`0156:90-92`)
- **`CHECK (quantity_before_milli - quantity_consumed_milli = quantity_after_milli)`** (`0156:103`) — a
  conservation invariant as a plain row-level CHECK, which works because the whole triple is on one row
- `unit_cost_won` + `cost_won` (`0156:93-94`) — the economics already ride the movement
- `source_kind CHECK (source_kind IN ('WORK_ORDER','P1_DISPATCH'))` (`:87`) with
  `work_order_id NOT NULL` (`:88`) and a hard FK (`:107`) — consumption **already links to work**, but
  only to `work_orders`, not to a generic dimension
- `milli` fixed-point integers throughout — the repo's quantity convention

`production_demand_contracts` (`0173:6`) is already a contract entity for the production case, and
**`production_operations`** — not `production_plans` — already carries `output_quantity` / `scrap_quantity`
(`CREATE TABLE` at `0173:75`, columns at `:81-82`), so scrap is a modelled concept rather than a new one.
`production_plans` carries only `quantity` (`0173:50`); the distinction matters because a lane looking for scrap
on the wrong table concludes it does not exist.

**What is genuinely missing is only the derivation edge.** Linear stock decrement exists; the
split/merge tree does not. So:

| Thing | Shape |
|---|---|
| `lot` (Tier T) | `(id, org_id, uom, quantity_milli, state)`, rooted at a `contract_line` or a production output — `milli` per `0156` |
| `lot_split` (Tier T) | **one row per split**: `(parent_lot_id, parent_qty_before_milli, split_qty_milli, parent_qty_after_milli, child_lot_id, uom, conversion_factor, reason)` |
| UoM conversion | **stored explicitly** on the row. 100 units consumed as 2 pallets is a conversion; an unrecorded one makes the tree unauditable |

**The row-level CHECK stays, and it is NOT sufficient on its own.**
`CHECK (parent_qty_before_milli - split_qty_milli = parent_qty_after_milli)` on each `lot_split` row is exactly
the `0156:103` pattern and it is worth having. But **two concurrent splits of a 100-unit lot can both write
(100, 60, 40)**: each row satisfies the CHECK in isolation, and the lot is over-allocated by 20. A per-row
constraint cannot see a sibling row, so an earlier draft's claim that it *"makes conservation unviolatable
without any procedural code"* is **withdrawn**, along with the claim that *"a definer is needed when an
invariant spans sibling rows, and putting before/split/after on one row removes the span"* — putting the triple
on one row removes the span **within** a row, not **across** concurrent writers.

**The mechanism, copied from the precedent that actually handles this.** The split write **locks the parent lot
row `FOR UPDATE` inside the action's transaction**, derives `parent_qty_before_milli` **from the locked row and
never from the request**, and updates `lot.quantity_milli` in the same transaction. The shipped shape is
`backend/crates/inventory/adapter-postgres/src/lib.rs:394` `fetch_item_for_update_tx`, layered on
`lock_consumption_idempotency_key_tx` at `:376` and a domain `state.consume(quantity)` at `:406`, with the event
INSERT at `:411`. Three guards — idempotency lock, row lock, domain check — with the row CHECK underneath all of
them. Probe: `lot_concurrent_split_cannot_overallocate`, whose **known-bad control is the row-CHECK-only
implementation**, i.e. this plan's previous design.

**`lot.quantity_milli` is AUTHORITATIVE**, updated under the same lock — because a derived reading requires
summing the split tree on every read, which is the aggregate the down-traversal already describes and cannot be
a row constraint.

**And the invariant, stated honestly:** what ships is a **per-row CHECK plus a whole-tree aggregate**. The
aggregate is not a row CHECK and is therefore **not unviolatable** — §5.8's own down-traversal states it as an
aggregate (*"sum(leaf lots) + sum(scrap lots) must equal it"*) a few lines below where an earlier draft called it
unviolatable.

A merge is the same row read in the other direction. Yield loss, scrap and shrinkage are **explicit child
lots**, never silent slack — and `production_operations.scrap_quantity` (`0173:82`) shows the repo already
treats scrap as a first-class quantity.

**The structural echo, reused rather than reinvented:** a contract line's **declared** quantity and its
**realized** set of splits are both stored and neither is derivable from the other — which is exactly
`line-as-raised` versus `line-as-executed` (§4.1). One shape serves both, and that is worth more than
two correct shapes.

**Traversals:**

```
up   (traceable input for production):
  finished_good lot → lot_split(child_lot_id=·) recursively → roots → contract_line → contract
down (does the accounting close?):
  contract_line → declared qty; sum(leaf lots) + sum(scrap lots) must equal it
```

**Authority reuses the routing model unchanged.** Who may split, merge or reallocate is a capability,
and the quantity bands are the same `delegation_rule` shape as the ₩100,000 purchase — a reallocation
above a band routes for approval. That it reuses the model without modification is evidence both are
shaped right.

**Both components:** every split, merge and consumption is a `record` event (actor, quantities, from/to
nodes, authorizing grant) and an `economics` posting dimensioned `'lot'`. A lot in flight
belongs to 업무, so it follows the work on 인계 like any other artifact.

**Non-foreclosure constraints while lineage is deferred (N4 / ADR-0034, §5.11).** W16 has no 0207+ slot, so the
constraints are recorded instead of the schema:

1. **Quantity-bearing or lineage edges may never live in `object_links`.** `0102:68` permits exactly one edge per
   `(org, src, dst, link_type)`, and `:86` grants `console_rt` `SELECT, INSERT, DELETE` — **no UPDATE**. A
   quantity that cannot be updated and cannot repeat is not a quantity. **The temptation is concrete, not
   hypothetical:** `derived_from` — *"Source was produced from the destination (lineage)"* — is **already one of
   the twelve seeded `link_types` labels** (`0130:43`), so the edge kind §4.3 names `derived_from` looks
   pre-built. It is not: it is the wrong store for it.
2. **A `TRANSFER` movement carries its from/to pair on ONE row**, for the same reason the conservation triple
   does: a pair split across two rows is a span no row constraint can close.
3. **No lineage ADR is accepted until the N-into-1 merge names its serialization point and lock order** — every
   shipped precedent in this tree locks exactly **one** row, so a merge is the first case with no precedent to
   copy, and lock order is where that becomes a deadlock rather than a bug.

**Adjacent precedents, and what each lacks.** `inventory_consumption_events` (`0156:81`) — quantity,
cost and conservation, but linear and bound to `work_orders` by FK (`:107`) rather than a generic
dimension. `production_demand_contracts` (`0173:6`) and `production_operations` scrap (`0173:81-82`) — the
contract and scrap concepts. `equipment_ownership_transfer_requests` / `_events` (`0072:8`, `:35`) — a
request/event custody-transfer pair, the right shape for *movement* but carrying no quantity and no
derivation. **Extend these rather than adding a parallel model:** the honest new surface is `lot` +
`lot_split`, and generalising `inventory_consumption_events.source_kind` (`:87`) from its two-value CHECK
to the `work` dimension.

### 5.9 I — Promotion and demotion: promotion is slice 1, the write path

The ₩100,000 purchase exercises **read/decide**. Promotion (승진 / 인사발령) exercises **write**, and it
is where structure, authority and 결재 meet in one operation: it changes 직책 (structure), changes
effective permissions via position-sourced grants (authority), requires approval (결재), is
effective-dated on 발령일 (a revision, not an update), and must replay.

Minimum entity shape for slice 1: `position` ×2 (from, to), `assignment` (closed + opened),
`approval_template` for 인사발령, `approval_line`, a `gov_approvals` signature carrying capacity, and the
`gov_approvals.authorizing_grant_id` column. No new entity classes.

**Demotion is grant EXPIRY, not deletion.** Otherwise replay dies. The shape to generalise is already
shipped and verified: `clearance_assignments` carries `status TEXT CHECK (status IN
('ACTIVE','REVOKED','EXPIRED'))` (`0147:20`), `starts_at` (`:21`), `expires_at` (`:22`), `granted_by`
(`:23`), `revoked_by` (`:24`) and `grant_reason TEXT NOT NULL CHECK (char_length(grant_reason) BETWEEN
1 AND 512)` (`:25`). That generalises to every grant source unchanged — a mandatory reason on every
authority change is exactly what 강등-as-징계 needs. **Adopt it verbatim rather than inventing a grant
lifecycle.**

**Demotion must NOT strip standing on lines already joined — and the design yields this without a
special case.** Standing is line membership (§5.2), and membership is recorded on the
`approval_line` instance, not derived from the holder's current grants. So a demoted member keeps the
power to 반려 a matter they were already placed on, and loses position-sourced grants only for *future*
routing. Confirmed by construction; the probe is `demoted_member_retains_standing` (§7).

**The correction axis is decided before Slice 1, and this plan takes the DEFERRAL with its consequence named.**
An erroneous revision **cannot be repaired in place**, and the substrate is explicit about it:
`0155_create_ontology_instances.sql:112-160` — `ont_instance_revisions_append_only()` raises on any DELETE,
raises if `OLD.valid_to IS NOT NULL`, and raises unless the **only** changed column is `valid_to`, so
`attributes`, `valid_from`, `version`, `prev_hash` and `row_hash` are all pinned. With
`CHECK (valid_to IS NULL OR valid_to > valid_from)` and `idx_ont_instance_revisions_one_open`, an erroneous
revision cannot be rewritten, cannot be closed at a zero-length interval, and a new revision at the same
`valid_from` **overlaps**. The migration's own header says the intent: *"a correction is a NEW revision, never a
rewrite of a stored one."*

The alternative on the record is the **bi-temporal entry-date axis**: a correcting revision carrying
`corrects_revision_id` plus a knowledge-time argument on as-of reads. It is **not taken in slices 0/1**, and the
consequence of deferring it must be stated rather than discovered: **between the error and its discovery, the
fold returns the wrong value for that period, and the record of that period cannot be made right — only
annotated forward.** For authority that is a real cost, not a cosmetic one.

Note this is a **different concern** from the post-확정 반려 compensating `correction` revision (§5.2), which is
the plan's only correction concept today: that one corrects a *decision*, this one would correct a *record of
what was true*. Probe: an as-of read across a corrected interval; known-bad control: **a correction that
silently rewrites history**, which the append-only trigger already refuses — so the probe is really asserting
that no application path tries.

**The basis carries through the whole chain**, and this is the one place the current design would drop
it: 발령 (the 결재 document's own record) → grant expiry (`grant_reason`, above) → audit event
(`audit_events.reason`, `0149:13`). All three links exist. The gap is that nothing *enforces* the basis
is the same across them, so the probe is `basis_survives_the_chain` and the enforcement is that the
grant revision and the audit event both reference the 결재 line id.

**What happens to in-flight work and lines when the holder moves** — the interaction neither slice
tests alone:

| Thing | On promotion / transfer |
|---|---|
| in-flight `approval_line` where they are a member | **unchanged.** Membership is not re-resolved; standing survives (above) |
| pending steps not yet reached | re-resolved at the step, from the fold at that time — so the new position applies going forward |
| assigned `work` | **stays assigned.** Promotion is not handover; moving work is a separate, scope-bounded act (§4.5) |
| position-sourced grants | old closes at 발령일, new opens — two revisions, never an update |
| open obligation loops | follow the 업무 (§5.2), so they are not lost |

### 5.10 J — A temporary UNIT's lifetime derived from a contract: disband vs transfer

**The heading used to say "Party lifetime", and in this plan `party` is the durable identity handle that is
永久 and never hard-deleted (§4.1) — the exact opposite of what this section decides.** The subject here is an
`org_unit` of a bounded kind (팀/TF/사업장), the "party" of the game lens, never the `party` entity. Renamed so a
lane grepping for party lifetime does not land here and conclude the identity handle expires with a contract.

A temporary unit's existence is **not configured, it is derived** — a 사업장 계약 or a body of work.
"When does this team stop existing" is answered by data, not by an administrator remembering.

`worksite_contract` (§4.1, Tier T) carries the parties, the term, and what it scopes. `org_unit`
instances of a bounded kind link to it, and their validity interval is the contract's.

**Which lifetime governs, legal or organisational?** They may not coincide — a 사업장 is a 4대보험
registration unit whose registration can outlive or predate the contract. **The legal lifetime governs
the legal facts (`worksite_registration`, payroll, 4대보험) and the organisational lifetime governs
authority scope.** They are two intervals on two entities precisely because collapsing them would make
one of the two wrong. This is the same split as operational versus decision scope (§4.6).

**Two terminal transitions, not one with a flag:**

| | Disband | Transfer to a new post |
|---|---|---|
| the unit | ceases — validity interval closes | **persists**, rebinds to a new contract |
| members | disperse | stay together (ordinary for construction/project crews) |
| assignment-sourced grants | expire **because the assignment ended** | continue |
| assigned 업무 + artifacts | must be reassigned or closed; the 인계 완료 **assertion records** whether that happened — it does **not** gate the disband (§4.5) | follow the unit |
| derived chat channel | archived, history retained | persists |
| open obligation loops | follow the 업무, not the unit | unaffected |
| in-flight 결재 lines | see below | unaffected — scope persists |

**This is the strongest argument for assignment as a grant source.** When a **unit** dissolves,
assignment-derived authority expires *because the assignment ended* — no administrator revokes
anything and nothing dangles. Contrast a design where unit membership grants a role directly:
dissolution then needs manual cleanup, which is forgotten, which is precisely how orphaned permissions
accumulate. Least privilege and correct teardown are the same property here, and they are free.

**The open question, resolved: scope survives its unit.** A dissolved `org_unit` is retained as a
closed-interval instance, never deleted. Reasons: (a) `approval_line` instances reference the scope, and
migrating standing on disband would rewrite history and break the fixity chain; (b) the ontology already
has terminal soft states (`ont_instances.lifecycle_state` includes `archived` and `disposed`,
`0155:27`) and no hard delete anywhere; (c) "what could this person approve on 2026-03-01" must still
resolve a scope that no longer exists operationally.

So: **standing does not migrate; the scope persists in a terminal state.** Transfer is the easy case
because the scope simply continues. Disband is handled by retention, not migration — which is the
cheaper answer *and* the only one that preserves replay.

---

### 5.11 The governance surface: what this plan may decide, and what needs an ADR

`docs/decisions/README.md` rules 1-6 bind this: an accepted ADR is authoritative in its scope (rule 1);
only another **accepted** ADR may amend it (rule 2); a later number does not win — amendment must be
explicit **in both records** (rule 3); *"`proposed`, `draft`, `design-note`, plan, prototype, and DARK
material cannot supersede an accepted ADR"* (rule 4); and code divergence is *"a governance gap, not
silent supersession"* (rule 6).

**This document is a plan. It therefore decides nothing on its own.** Every item below is prepwork that
gates the fanout, not a paragraph that authorizes it.

**Reciprocity is machine-enforced; clause compatibility is not.** `scripts/check-adrs.mjs:23-27` defines
`RECIPROCAL_RELATIONSHIPS` as **exactly** `amends`/`amended_by` and `supersedes`/`superseded_by`, and the loop
at `:399-406` fails only when the *target* does not declare the reciprocal key; `related` is validated only as
an inline array (`:248-249`). **There is no clause-level collision check anywhere.** So two accepted ADRs that
both declare `amends: [ADR-0003]` and edit the same Decision line **incompatibly** pass CI and leave the
authoritative record self-contradictory, with nothing to detect it. **This is the stated reason G2 and G2b
merge into one record** rather than shipping as two pairs against the same clause.

**ADR numbers are assigned centrally, and the failure this prevents was observed.** Every draft carries
`ADR-XXXX-DRAFT`; the integrator assigns the number in **one atomic commit** together with the
`docs/decisions/README.md` index rows (`README:3` — the index *"must be updated atomically with every ADR
status, identity, amendment, or supersession change"*). **No lane computes "next free".** In the prior review
run, four independent judges each computed "next free after ADR-0026" and **all four claimed `ADR-0027`** —
which is what "next free" produces whenever more than one writer is awake. Numbers are also never recycled:
`ADR-0013` was never issued and `README:13` forbids reuse or backfill, and any allocated-but-unwritten number
in this plan's set stays an **unused gap** on the same rule. The assignment of record for this plan's nine (or
ten) records is the allocation table below.

| # | Matter | Status | Required artifact |
|---|---|---|---|
| G1 | **Platform-level `party`** | **WITHDRAWN — premise false; there is no clause to amend.** G1 grounded on *"`ADR-0022:25,33-39` decides identity is local/org-scoped"*. Verified: `:25` is **Context prose** (`## Context` is at `:23`), the `## Decision` block is `:31-39`, and the string **"org-scoped" appears nowhere in ADR-0022**. Its `## Decision` opens *"Do not ship a speculative external IdP seam."* and confines `console-identity-application` to *"only local org/account administration commands, read models, and audit builders"*. Nothing there forbids a durable handle | **D1 → ADR-0027**, a reciprocal amendment pair that **narrows** ADR-0022: identity linkage across orgs is **human-asserted**, and no durable identity row lands in Slice 0. **Narrower still after §4.1's tier decision:** the handle is an ordinary tenant-classified row homed at the platform sentinel org, not a Tier O carve-out and not a new gate classification — so what D1 records is a *linkage* rule, not a new tier or a new privileged read. **Does not block Slice 0** — it *defers* `party` to W2 (§4.1) |
| G2 + G2b | **`org_id` × `BranchScope` composition, and `BranchScope::All` after `Role` deletion** (§0.16) | **documented gap, and it is ONE gap.** `ADR-0003`'s Decision says *"`All` for SUPER_ADMIN/EXECUTIVE rollups, an explicit branch set otherwise"* and has **no org concept**; `ADR-0021` decision 2 makes `org_id` the RLS boundary Cedar may not widen. **No ADR states how they compose** | **D2 → ADR-0028**, one reciprocal pair on `ADR-0003` with its Decision **edited in place** (merging G2 + G2b). Filed as **one** record because CI cannot see a clause collision (above): two pairs against the same Decision line would both pass. `ADR-0021` and `ADR-0018` gain `ADR-0028` in `related`. **BLOCKS Slice 0** |
| G3 | **전결규정 routing, capacity, obligation loop** | **not greenfield — "zero ADR hits" is struck.** `ADR-0023:81-82` already decides arbitrary approval-line DAGs and the 검토/승인/합의/참조 vocabulary, so this is a **delta on ADR-0023's Engine-Gen**, not new ground | **N3 → ADR-0033**, non-amending, `related: [ADR-0018, ADR-0023]`. **NOT blocking Slice 0** — this is the theme where corrected evidence *removes* work from the critical path. Acceptance condition: the first migration introducing `delegation_rule` carries `CHECK (delegator_id <> delegate_id)` in the same file |
| G4 | **Quantity lineage** (§5.8) | new; zero ADR hits | **N4 → ADR-0034**, non-amending, `related: [ADR-0001]`. Deferred **with constraints instead of schema** (below); no 0207+ slot |
| G5 | **Economics spine** (§5.5) | new; zero hits for GL/posting/dimension | **N5 → ADR-0035**, non-amending, `related: [ADR-0023]`. The **record** does not block Slice 0; its three **prerequisites** do (§5.5) |
| G6 | **The no-code canvas** | **STRUCK.** Out-of-scope is **silence, not prohibition** (`docs/decisions/README.md:7` — an accepted ADR is authoritative *within its stated scope*), so no accepted ADR defers the canvas and **there is no charter clause** to amend or satisfy. Verified: `ADR-0023:148` is the header *"Follow-ups (named out of scope for this program)"*; the canvas bullet at `:153-154` carries **no** charter clause; *"enters as its own charter"* is at `:156`, on the Contract→Position(인원편성)→PolicyPreset bullet | **none.** The canvas's exclusion from slices 0/1 is **this plan's own scope choice**, standing on its own merits. *"Do not smuggle it in"* stays, as a scope statement rather than an inherited prohibition. **W10 is deferred-by-follow-up and off the slice-0/1 critical path — it is NOT gated on a charter.** Optionally, **N2 → ADR-0036** records object-policy revocation as a catalog status transition; if it is never written the number stays an unused gap |
| G7 | **DN-0003 bounded extensibility vs open sets** | **STRUCK on structural grounds, not "aligned as written".** DN-0003 is `kind: design-note`, `authority: subordinate`; `README:26` governs **ADR** relationship keys while design notes declare `parent_adr`. So DN-0003 **cannot take a reciprocal ADR pair at all**, whatever the substance | **none, and none is possible.** The header's *"(G7 needs none)"* stays true, for a better reason than the one given |
| G8 | **DB-enforced invariants vs ADR-0001 domain purity** | tension, see below. **Two examples, not three** — the lot CHECK is withdrawn (§5.8: a row CHECK does not hold conservation across concurrent splits) | **argue only, no record.** `ADR-0001:23` is a **Consequences** bullet, not the Decision, so no ADR question is engaged unless the argument is rejected |
| G9 | **Audit coverage for the new write paths** | **reclassified: BLOCKING, and a retroactive amendment.** `ADR-0002`'s Decision states its *"exclusion set contains exactly one entry"* and that *"a test asserts that is the only exclusion"*. The gate returns **TWO** | **D3 → ADR-0029**, a **retroactive** reciprocal amendment pair on `ADR-0002` whose Decision text is **edited in place** — a reciprocal key alone leaves a false sentence standing. `ADR-0014` gains `ADR-0029` in `related`. **BLOCKS Slice 0** |
| D4 | **The console rebuild charter and the generated-client reconciliation** | **owner decisions CAPTURED, not accepted.** `docs/ideas/d4-frontend-charter.md` (2026-07-30) records four owner decisions and splits D4 into **at least two** amendment records. So D4 is blocked on **acceptance**, not on an owner decision | **at least two** pairs: **ADR-0030** (D4-A1, amends `ADR-0025`) and **ADR-0031** (D4-A2, amends `ADR-0009`, Decision edited in place; `ADR-0012` gains `related` only). **The count is the charter's to state at acceptance, not this plan's to restate** — the charter is a live document and already names a **third** target: adding a `ui` layer to `ADR-0001`'s **enumerated** crate family amends it. That number is allocated by the integrator with the rest, in the same atomic commit; no lane computes it. **NOT on Slice 0's path**, and the record is owed **independently of whether this plan is approved.** `ADR-0025`'s clause 1 — a reachable mounted body for every exposed navigation state — survives **unamended**, and the CI gate asserting the console frontend does not exist **stays**: under the charter it is the enforcement mechanism for "planning only" |
| N1 | **Where the fold is computed** (§5.6) | not a collision; a mechanism worth recording | **N1 → ADR-0032**, non-amending, `related: [ADR-0021]`. Records that the fold is computed **per request with no cross-request materialisation**. Unblocks §5.6 by making the deleted materialise row a recorded decision rather than an omission |
| **SoD** | **Segregation of duties** | **IN, as a grant-authoring-time constraint.** Three shipped specs already decide it is not a UI concern — `docs/specs/cedar-pbac-authorization.md:122` *"Segregation of duties and self-approval checks are PBAC conditions, not UI-only rules."*, `docs/specs/no-code-operational-logic.md:211` *"segregation of duties and self-approval prevention"*, and `docs/specs/operations-intelligence.md:170` *"required evidence, segregation of duties, self-approval restrictions, and conflict-of-interest flags"*. Earlier drafts of this plan mentioned none of them, which read as a silent contradiction rather than a choice | **no new record — it lands inside N3.** Mechanism: **conflict pairs over `Feature`**, evaluated at **grant-authoring time**, in the place the `gov_approvals` four-eyes check already runs. Not a fold-time subtraction — §1 principle 2 is unaffected (see the note there). Widening: **W19**, with probe `conflicting_grant_pair_refused_at_authoring`; known-bad control: **a fold that accumulates a conflicting pair silently**, which is today's behaviour |
| **GATE** | **What each CI gate pins** | **classification, so amendability is decidable.** A gate pinning a **safety property** (`tenant-isolation`'s classification of every table, `migration-safety`'s audited-table `DROP COLUMN` refusal, `rls-arming`, `audit-coverage`) is **never weakened**, and no item in this plan asks for one to be. A gate pinning a **decision** by asserting literal sameness (`ADR-0025`'s console-absence assertion; `route-inventory`'s and `check-ci-preflight`'s generated-artifact equality; `command-database`) is amendable **with its ADR**, and the replacement is derivation per crate rather than a widened literal. **Prerequisites 5.7a and 5.7b (Phase 0) are gate hardening, not gate weakening.** And the Phase-4 CI-wiring cost is a **defect to delete, not a toll to pay**: `scripts/check-ci-preflight.mjs:430-453` (`requireOntologyRestItestReachability`) **already** derives the requirement from the generated BUCK file and walks the whole itest → `sh_test` → `ci.yml` chain, failing with *"`ci.yml` must execute … or `{itest}` runs nowhere"*. Its own header says the fix is *"a per-crate decision with the same shape as this one, not a cleverer regex"* (`:428`). So the per-test wiring step exists only where that pattern was not adopted | none — this row is a classification, not an ADR question. It exists because §5.11 named no gate at all, while five of them bind this plan |

**The allocation, assigned once, here.** Highest issued is **ADR-0026**; `ADR-0013` was never issued and must
never be reused (`docs/decisions/README.md:13`). True next free: **0027**.

| Slot | Record | Kind | Counterpart obligations | Blocks Slice 0 |
|---|---|---|---|---|
| **ADR-0027** | D1 — identity linkage is human-asserted | amendment pair, **narrowing** | `ADR-0022` gains `amended_by: [ADR-0027]`; its index row → `accepted, amended` | no |
| **ADR-0028** | D2 — `org_id` × `BranchScope` (merges G2 + G2b) | amendment pair; Decision **edited in place** | `ADR-0003` gains `amended_by` — **the key does not exist today and must be created**; `ADR-0021`, `ADR-0018` gain `related` | **yes** |
| **ADR-0029** | D3 — audit exclusions are **two**, each bound to a (file, function) pair | amendment pair, **retroactive**; Decision edited in place | `ADR-0002` gains `amended_by` — **key created**; `ADR-0014` gains `related` | **yes** |
| **ADR-0030** | D4-A1 — console rebuild charter (Leptos) | amendment pair | `ADR-0025` gains `amended_by` — **key created**; index row → `accepted, amended` | no |
| *(unallocated)* | D4-A3 — a `ui` layer in `ADR-0001`'s enumerated crate family, **if** the charter carries it at acceptance | amendment pair | `ADR-0001` gains `amended_by` — key created | no |
| **ADR-0031** | D4-A2 — generated-client / dual-native reconciliation | amendment pair; Decision edited in place | `ADR-0009` gains `amended_by` — **key created**; `ADR-0012` gains `related` only | no |
| **ADR-0032** | N1 — fold computed per request, no cross-request cache | non-amending | `related: [ADR-0021]` | no |
| **ADR-0033** | N3 — 전결규정 / capacity / obligation loop as an ADR-0023 delta | non-amending | `related: [ADR-0018, ADR-0023]` | no |
| **ADR-0034** | N4 — quantity lineage deferred with constraints | non-amending | `related: [ADR-0001]` | no |
| **ADR-0035** | N5 — economics spine; COST as a query | non-amending | `related: [ADR-0023]` | prerequisites yes, record no |
| **ADR-0036** | N2 — *OPTIONAL.* object-policy revocation as a catalog status transition | non-amending | `related: [ADR-0023]` | no |

**Reciprocation mechanics, which no ordered list in earlier drafts covered.** For **each surviving pair**, the
work is three things and the third is the one that gets forgotten: name the **counterpart record**, name the
**exact clause amended**, and **add the relationship key on BOTH sides**. `README:9`: *"A later number does not
win automatically. Amendment or supersession must be explicit in both records."* `README:26`: relationship keys
*"must be reciprocal where applicable."* **`ADR-0003` carries no `amended_by` key today** — reciprocation must
**create** it, and the same is true of `ADR-0002`, `ADR-0025`, `ADR-0009` and `ADR-0022`. Pre-acceptance each
draft carries `status: proposed`, `doc_status: review` and `proposes_amendments_to: [...]`, and **may not declare
active `amends`** (`README:26`). The index rows land in the **same atomic commit** (`README:3`).

**The pair list is shorter than earlier drafts implied:** G6 and G7 are struck, G1 is withdrawn, G8 takes no
record. Five amendment pairs are allocated here (ADR-0027 through ADR-0031) plus four or five non-amending records. D4
may add a sixth pair — the charter is live and gained an `ADR-0001` target after this table was written, which is
precisely why the count is not restated as a fact.

**G8 — DB-enforced invariants vs ADR-0001 domain purity. Decided: argue, not amend.**
`ADR-0001:23` states *"Domain logic stays pure and exhaustively unit-testable"*, gated by a CI
layer-boundary check. Note first that `ADR-0001:23` is a **Consequences** bullet, not the Decision, so no ADR
*question* is engaged unless the argument below is rejected.

This plan puts **two** invariants in SQL: the voucher balance gate (`0160:78-118`, already shipped) and the
authority fold in a definer (§5.1). **The third — lot conservation as a row CHECK — is withdrawn**: a row CHECK
does not hold conservation across concurrent splits (§5.8), so it was never an example of an invariant SQL alone
enforces.

The argument, on the record: **these are integrity constraints, not domain logic.** Each answers "is this
row internally coherent", never "what should the business do" — and a constraint that *cannot* be violated
beats a pure function that callers may forget. `0205` set that precedent deliberately, and the balance gate
predates this plan. The layer-boundary gate still holds because no domain crate gains a SQL dependency: the
constraints live in migrations, and the domain crates keep their pure validators. **If the Critic rejects
that distinction, G8 becomes a reciprocal amendment to ADR-0001, not a silent divergence** — which is what
`docs/decisions/README.md:12` requires either way.

**D3 (ADR-0029) — audit coverage. BLOCKING, and a retroactive amendment, because the ADR is wrong.**
`ADR-0002`'s `## Decision` states that the CI `audit-coverage` gate's *"exclusion set contains exactly one
entry — the LocationPing ingestion path (ADR-0014) — and a test asserts that is the only exclusion."*
**That sentence is false, and the code is authoritative.** The gate returns **TWO**, each bound to a
**(file, function)** pair, not to a table name:
`backend/ci/gates/audit-coverage/src/lib.rs:90-111` returns
`location_ping_ingestion` / `record_location_ping` and
`location_data_retention_purge` / `purge_expired_location_data`, both in
`crates/compliance/adapter-postgres/src/lib.rs`. And the test is
`backend/ci/gates/audit-coverage/tests/gate_detects_violation.rs:26`
`fn allowed_exclusion_set_is_the_two_location_carveouts()`, asserting `exclusions.len() == 2`.

**Cite the gate and the test name, never `ADR-0002`'s Decision line for this fact.** An ADR Decision line is
**prose about code**. This one was propagated as a state fact through an entire review pass before anyone opened
the gate — which is the same failure class as `0153`'s `-- one decision per request` comment (§4.4). Two
independent instances in one plan is why the plan now cites code even when an authoritative document says
otherwise.

Because a reciprocal key alone would leave the false sentence standing, D3 **edits `ADR-0002`'s Decision text in
place** and is therefore **retroactive**: it corrects a statement about behaviour that was already wrong when the
ADR was accepted. `ADR-0014` gains `ADR-0029` in `related`.

What D3 must also record, so the enumeration is sized rather than labelled:

- This plan routes granting, signing, closing, confirming and linking through `ont_action_types` with
  `dispatch = 'instance_revision'`, and **nothing states whether that path goes through `with_audit`.** The
  **write-path enumeration** (Phase 0) answers it, one row per path.
- **T7 adds zero exclusions.** The enumeration's purpose is coverage, not carve-outs.
- Two 0207+ migration rows this plan had priced at **zero**: an `object_types` row for `work` and a `link_types`
  row for `work_artifact` (§4.3).
- Extending `is_handler_surface` to path component `app` has **UNMEASURED** blast radius. **X-T7a must run
  first**; the enumeration must not assume it.

**G7 is STRUCK, and on a structural ground that holds regardless of the substance.**
`DN-0003` is `kind: design-note`, `authority: subordinate`. `docs/decisions/README.md:26` governs **ADR**
relationship keys, while design notes declare `parent_adr`; and `scripts/check-adrs.mjs:23-27` reciprocates only
over the ADR map. **So DN-0003 cannot take a reciprocal ADR pair at all** — G7 was never an available action.
The substance is aligned anyway, and it is worth stating because it is the answer to "how does this tie into
the engine": `DN-0003` (design-note, `activation: in_progress` at `:6`) invariant 10 reads, at `:97-99`:

> **Extensibility is bounded.** Tenant definitions are declarative; trusted first-party tools are
> compile-time allowlisted; external connectors are server-side, typed, scoped, audited, idempotent, and
> outbox-backed.

The bound falls on **tools and connectors**. The same sentence **affirms that tenant definitions are
declarative** — the open side. So §4.0.2's boundary (declaring a type is authored; giving a type a new
concern is code) is not a reconciliation this plan invented; it is DN-0003's own line, restated on the
axis this plan cares about. **No reciprocal pair is needed, and none is possible.**

What would be a real collision is a claim that adding a *component* requires no code. This plan does not
make that claim (§4.0.2 says the opposite, explicitly).

Four DN-0003 invariants bind the design directly and are **inherited, not re-decided**: every
consequential mutation is an **Action** (direct property edits are not the write path); commands are
deterministic (command id + payload digest + expected revision — replay returns the original receipt,
digest mismatch `409`, stale revision `412`); the **write transaction is complete** (mutation + history +
authz recheck + approval consumption + audit + receipt commit together); and **denied data is omitted
including counts**. The third is why §4.0.3's capacity column is cheap: the receipt-writing transaction
already exists and already carries the authz recheck.

## 6. Pre-mortem

Five scenarios. The first three are the authority model's failure modes; the last two arrived with the
widened scope and are, on current evidence, the more likely ones.

### Scenario 1 — The definer becomes the hole it was built to avoid

`effective_grants_for` is copied from `group_role_grants_for_user` (`0060:99`) *including* its
trust-the-parameter weakness (§4.2). A caller passes another org's party id; the definer has
`row_security = off`; it returns grants from an org the caller cannot see. The org floor evaporates
while every test stays green — precisely the failure `0205:69-74` warns about in the analogous case.

**Leading indicator.** Any definer body that reads a party or grant without a literal
`current_setting('app.current_org')` predicate. Gate it as a grep, not a review habit — the review
habit is what failed in the precedent.

### Scenario 2 — Grants quietly become subtractive

`policy_role_conditions` already models two of the four dimensions as data (`attribute` CHECK, `0065:110-127`
— §4.4(b): 직무 and 직급 are not literals there)
— and its operator set is `CHECK (operator IN ('equals','not_equals','in'))` (`0065:129`). **`not_equals`
is already in the shipped vocabulary.** The path of least resistance is to reuse that table and its
resolver, and the moment one condition narrows rather than adds, `effective(P, B) > effective(Q, B)`
(requirement 3) stops holding, delegation starts removing authority, and routing becomes a capability
restriction. All three of the input's retractions come back at once, silently, as an implementation
detail — and the DDL permits it today.

**Leading indicator.** Any deny row, any `NOT IN`, any `EXCEPT`, or any narrowing predicate inside the
fold. A test that adds a second grant to a party and asserts the capability set **strictly grows** is
the cheap detector; it must be proven RED against a narrowing implementation.

### Scenario 3 — Slice 0 passes, and proves nothing

The ₩100,000 slice is built with one grant, one rule row, one band, one step. Every assertion passes
because there is only ever one candidate for everything: one rule to look up, one approver to resolve,
one signature to record. The fold is never exercised as a fold; routing is never exercised as a
lookup. The slice goes green and the model is still unproven — the exact failure mode that made six
probes defective in one session here.

**Leading indicator.** Slice 0 must include, in the same test run, a **negative twin** for each
mechanism: a second grant that must NOT authorise (wrong scope), a second band that must route
elsewhere, a signature whose `authorising_grant` is closed at the decision timestamp and must be
refused. If a probe's RED case does not exist, the probe does not count.

---

### Scenario 4 — the metaphor ships and the depth does not

Requirement 8 is the most seductive scope in the plan: the character sheet, the quest log and the guild
log all demo beautifully with shallow data. The ledger, the metrics and the capacity record are the parts
nobody sees in a demo. So the likely failure is not that ergonomics is skipped — it is that ergonomics
ships *first and alone*, `authorizing_grant_id` stays null because nothing visible depends on it, and the
system becomes a pleasant surface over an unauditable core. "Intuitive surface, uncompromised depth"
fails on the second clause, quietly, while looking like progress.

**Leading indicator.** Any null `authorizing_grant_id` on a signature or authority mutation, and any
`work` cost rendered from something other than a voucher. Both are single-query checks — run them as
gates, not as reviews. A demo built on data where every capacity field is null is the concrete tell.

### Scenario 5 — two silent-empty traps make a working canvas look configured

§0.12 (a link type alone writes no edge) and §0.13 (a policy-less type lists `[]`) fail the same way:
**no error, empty result, configuration that looks correct.** An author draws the entity model on the
canvas, saves it, sees no relationships and no rows, and concludes the model is wrong — or worse, does
not check and ships it. Two independent traps on the one surface the whole no-code requirement depends
on.

**Leading indicator.** A published type with zero attached policies, or a link type with no property
referencing its `stable_key`. Both are queryable and both belong in the same gate. The known-bad control
matters unusually here: `link_type_alone_is_rejected` and `tier_n_type_lists_nonempty` must both be
observed RED against today's behaviour before their GREEN means anything, because today's behaviour *is*
the bug.

## 7. Test plan

Every critical probe is listed with the **known-bad input that must make it RED**. A probe whose RED
has not been observed is not evidence.

**One measured instrument trap, recorded here because it made a probe in this very plan look like a pass.**
On PG 18 a placeholder GUC set with `SET app.current_org = …` is readable through `current_setting()` but
**never appears in `pg_settings`**. So a GUC inventory built by querying session state returns **zero rows**
and looks like proof that no second tenancy dimension exists — which is the answer X4 was designed to test,
arrived at by measuring nothing. X4's first attempt did exactly this. A GUC inventory must be extracted from
**stored policy expressions and function bodies** (`pg_policy.polqual`, `pg_proc.prosrc`), never from
session state. Any probe in this section that asserts an absence must state where it looked.

### Unit

| Probe | Asserts | Known-bad control (must be RED) |
|---|---|---|
| `fold_is_additive` | adding a grant strictly grows the capability set | a fold implementation with any narrowing predicate |
| `fold_is_scope_parameterised` | `effective(P, group G) ≠ effective(P, company A)`, with the **group** arm sourced from the **Tier O** group-scoped grant store (§4.1) and the company arm from Tier N | a fold that ignores the scope argument |
| `requirement_3` | associate-at-A with a group-G grant out-ranks executive-at-B — the group-G grant coming from the **Tier O** store, because Tier N cannot hold it (X4b) | any rank-derived ordering; and a Tier N-only substrate, which cannot produce the precondition at all |
| `delegation_adds_never_removes` | delegating retains the delegator's grant | an implementation that closes the source grant |
| `asof_replay` | fold at raise time ≠ fold at decision time when a grant closed between | a fold that ignores `p_asof` |
| `feature_catalog_matches_enum` | `feature_catalog` ≡ `Feature::ALL`, both directions | a `Feature` variant with no catalog row; a catalog row with no variant |
| `line_as_raised_immutable` | re-routing revises the executed line, never the raised line | code that mutates the raised line |
| `parallel_hapui_not_ordered` | two 합의 branches have no relative order | a step-index implementation |

### Integration (Postgres, `console_rt`)

| Probe | Asserts | Known-bad control |
|---|---|---|
| `party_is_invisible_and_unmintable_from_a_tenant` | armed to any tenant org as `console_rt`: `SELECT * FROM party` returns **zero rows** and `SELECT count(*)` returns **0** (omitted, not refused — DN-0003 invariant 5), **and** an INSERT is refused by the `org_isolation` `WITH CHECK` + the sentinel-pinning column CHECK (§4.1). Renamed from `party_not_readable_as_console_rt`, which asserted a **denial** — the Tier O reading the plan no longer takes | the table without `FORCE` RLS, or without the sentinel CHECK: the SELECT then returns platform-wide cardinality, which is exactly what X4 measured (`count(*)` = **2** where org A held one edge) |
| `visibility_edge_rls` | org A cannot see org B's `party_org_visibility` rows for the same party | RLS not FORCEd, or policy omitted |
| `definer_ignores_parameter_org` | passing another org's party id returns **zero rows** | a definer that filters on a parameter instead of `app.current_org` — i.e. `0060:99` copied verbatim |
| `definer_revalidation_each_check` | baseline GREEN with all named checks of §5.1 present; then deleting **`org_predicate`**, **`visibility_predicate`**, **`chain_linkage`** and **`scope_containment`** each in turn fails the suite | each named deletion individually RED. The probe **names each check** rather than carrying a count, so the number cannot rot |
| `definer_returns_no_foreign_org_grant` | one party with a visibility edge in **both** orgs and one grant in each; armed as org A, the call returns exactly **one** row | the definer as §4.5 specified it before this revision — no `org_id` predicate on the grant read |
| `row_security_restored_on_error` | an exception inside the definer leaves `row_security = on` | a definer without the `EXCEPTION WHEN OTHERS` restore of `0060:88-91` |
| `genesis_grant_mintable_only_by_platform_principal` | a grant with no authorising grant can be minted **only** on the platform-principal path gated by `PlatformFeature::TenantCreate` (§5.1) | a **tenant**-authenticated endpoint creating a grant with no authorising grant |
| `no_new_gate_classification` | the tenant-isolation gate passes with **exactly one** new `owner_only_table_allowlist` entry — the group-scoped grant store — and with `party`, `party_org_visibility` and `work` **unlisted**, i.e. taking the default Tier T classification. `party` is deliberately **not** an owner-only table (§4.1) | the grant store added to `global_table_allowlist` (must be RED per §3.2 Option 3); **and** `party` added to `owner_only_table_allowlist`, which must also be RED — an owner-only `party` is the Tier O reading this plan withdrew, and it would silently remove the RLS floor that carries the confidentiality property |
| `visibility_unique_key_leads_with_org_id` | an insert into `party_org_visibility` that collides **only** with an invisible other-org row is **accepted** | the same table keyed `UNIQUE (party_id, relationship_kind, valid_from)` — must be RED with `23505` (X4 CONTROL 3) |
| `ont_link_to_projected_row_is_rejected` | an `ont_links` INSERT naming a **projected** type's row as an endpoint fails **on the FK** (`0155:76-77`) | the four `work_*` edges as §4.3 specified them before this revision, i.e. as `ont_link`s |
| `worksite_reg_no_unique` | duplicate 사업자등록번호 rejected by the DB | the constraint expressed only in app code |

### End-to-end (the proving slice)

| Probe | Asserts | Known-bad control |
|---|---|---|
| `slice0_terminal_at_현장` | ₩100,000 비품 raised at 현장 R resolves to a terminal 현장 signature | routing that escalates to 본사 |
| `slice0_negative_scope` | a grant at a *different* 현장 does not authorise | a fold ignoring scope |
| `slice0_second_band` | ₩100,000,000 re-routes by the same lookup, no special case | a hard-coded escalation path |
| `slice0_capacity_recorded` | the `gov_approvals` row carries `authorizing_grant_id` beside *(signer, scope)* | a signature recording only the signer — today's `gov_approvals.approver_id`-only shape (`0153:71`, no capacity column) |
| `daeri_records_both_parties` | a 대리 signature records the outgoing signer **and** `on_behalf_of_party_id` | a 대리 signature whose `on_behalf_of_party_id` is null |
| `slice0_closed_grant_refused` | a grant closed at the decision timestamp cannot authorise | a fold using `now()` instead of the decision timestamp |
| `slice0_본사_may_still_approve` | 본사 retains `purchase.approve` at company scope; routing did not restrict it | routing implemented as a capability restriction |
| `retroactive_반려_after_확정` | emits a `correction`; the original stays `confirmed` in history | an implementation that transitions or rewrites the original |
| `obligation_notifies_line_as_raised` | truncated member D is notified though D never saw the matter, **and every non-member of the line receives nothing** | notification over the executed line; **and the shipped org-wide snapshot** (`notices/adapter-postgres/src/lib.rs:413-433`), which notifies every active user in the org and would pass the first half alone |
| `handover_is_scope_bounded` | relinquishing a group duty leaves the subsidiary post's work in place | handover moving everything the person holds |
| `handover_moves_work_artifacts_only` | person-linked material does not transfer | handover moving all artifacts |
| `link_email_is_authorized` | an unauthorised actor cannot link an email into a work | an open triage queue |

### Added by the addenda — components, game lens, promotion, lineage

| Probe | Asserts | Known-bad control |
|---|---|---|
| `every_entity_declares_its_components` | each §4.1 entity has a row per composed component in `docs/specs/ecosystem-entity-components.tsv` | an entity with no rows — the §4.0.1 completeness test, as a test |
| `every_entity_has_a_home` | each §4.1 entity maps to a character-sheet section (E2, §4.8) — a **screen** completeness test, distinct from the TSV one above | an entity with no section mapping |
| `capacity_recorded_on_every_authority_mutation` | reads the **D3 write-path enumeration** (§8 Phase 0) and asserts every enumerated authority-mutating path writes `gov_approvals.authorizing_grant_id`. The `audit_events` pair is **out of scope** until those deferred columns land (§4.0.3) | a mutation writing a null capacity where the enumeration says it is required |
| `no_duplicated_fact` | `work` (Tier T) and the revision chain never store the same field | a `work.assignee` column duplicating the assignment edge |
| `projected_mutation_goes_through_the_domain_usecase` | every consequential `work` mutation runs through the audited domain use-case, satisfying DN-0003 invariant 1 for a projected type (§4.0.2) | a write reaching the backing table through an ontology property edit |
| `tier_n_type_lists_nonempty` | a published Tier N type returns rows | a type published with no object policy attached — `deny_all()` at `backend/crates/platform/authz/src/cedar_pbac/residual.rs:200-203` (§0.13) |
| `link_type_alone_is_rejected` | a link type with `to_object_type_id` and no property referencing its `stable_key` fails `validate_draft` | today's behaviour — **must be RED before the guard lands** (§0.12) |
| `slice0_band_enforced_synchronously` | the ₩100,000 band is refused **at raise**, not flagged at close | a check that only reports at period close |
| `regulation_renders_as_one_artefact` | the complete 전결규정 — (category × band × scope) → competent unit, terminal? — renders as **one artefact as of an arbitrary date** (§4.7 point 4). Current-state renderability, **not** historical replay | routing expressed only inside approval templates, so the regulation can be reconstructed only by reading every template — SAP's named failure (`docs/ideas/research-sap.md:937-939`) |
| `economics_is_a_view` | `work` cost equals `SUM` over voucher LINES dimensioned to that work; no cost column on `work` | a stored cost column |
| `posted_voucher_cannot_be_rewritten` | a post-확정 반려 produces a contra voucher; the original stays POSTED | code attempting to UPDATE a POSTED voucher — the `0160:79` trigger must fire |
| `demoted_member_retains_standing` | a demoted member may still 반려 a line already joined | standing re-resolved from current grants |
| `basis_survives_the_chain` | 발령 → grant `grant_reason` → `audit_events.reason` all reference the same 결재 line | a grant expiry with no basis |
| `disband_retains_scope` | after disband, "what could P approve on <date>" still resolves the dissolved scope | hard-deleting the `org_unit` |
| `disband_expires_assignment_grants` | assignment-sourced grants end with no explicit revocation | membership granting a role directly, leaving an orphan |
| `transfer_keeps_crew_and_scope` | rebinding to a new contract preserves unit, members and in-flight lines | transfer implemented as disband + recreate |
| `lot_conservation` | `parent_before − split = parent_after` per row; scrap is an explicit lot | a split leaving unaccounted slack |
| `lot_concurrent_split_cannot_overallocate` | two concurrent splits of the same lot cannot both commit an over-allocating pair; the parent is locked `FOR UPDATE` and `parent_qty_before_milli` comes from the locked row (§5.8) | the **row-CHECK-only** implementation — two writes of (100, 60, 40) both satisfy the CHECK and over-allocate by 20 |
| `lot_uom_conversion_recorded` | a cross-UoM split stores its factor | an implicit conversion |
| `lot_traversal_up` | a finished good enumerates every contributing contract line | a broken derivation chain |
| `realtime_push_carries_no_capability` | the NOTIFY payload contains ids and a version only | a payload containing a capability set — must also fail the 8000-byte cap (`realtime:40`) |
| `grant_write_bumps_subject_version` | a grant revision bumps `authz_subject_version` for the subject party's users **and** `policy_versions` for the org (§5.6) | a grant write that bumps only `policy_versions` — a whole class of authority change then pings nobody |
| `stale_client_button_is_refused` | pressing a stale action after demotion is denied server-side | trusting the client projection |
| `channel_membership_is_derived` | changing an assignment updates the thread roster with no manual step | a hand-maintained roster (today's `0012:30-36`) |
| `deny_by_omission_is_explained` | a refusal a user sees carries a reason | `determining_policies` empty with no fallback (today, `0159:29`) |

### Observability

- Every authority decision emits engine mode + policy version + schema version + bundle digest — the
  coexistence map's `audit_metrics_with_engine_mode_and_versions` promotion requirement, reused, not
  redefined.
- Definer invocations counted and alerted on rate anomaly; a definer is the one place the floor is off.
- Obligation loops open past threshold — an unclosed 반려 loop is a correctness defect, not a backlog.
- 인계 완료 incompleteness count per departure, as a gauge rather than a one-shot check.
- Fold cardinality per decision. A fold returning an unexpectedly large capability set is the leading
  signal of a scope-expression bug, and it is cheap to watch.

---

## 8. Sequencing — Bun shape

Modelled on the Bun Zig→Rust port: reference documents before any conversion, an immutable target, a
three-file trial with adversarial review, a by-crate queue, a progressive verification ladder, and
**one PR** off a single long-lived branch. 6,502 commits, +1,009,272 lines, 11 days, no incremental
merges. That last property is what makes it compatible with LANE-PROTOCOL's *"parallelise the work,
serialise the landing"*.

Landing discipline, the reservation scheme, the three CI gates replacing a human coordinator, structural
guards and the Bun mechanisms are owned by `docs/ideas/fanout-plan-DRAFT.md` §5, §5.1, §6, §6.5, §7
(APPROVED) and `docs/program/LANE-PROTOCOL.md`. Referenced, not restated.

### Phase 0 — the reference documents (before any code)

Bun spent ~3 hours producing `PORTING.md` + `LIFETIMES.tsv` before converting one file. Ours — **three files,
all of which must exist and be reviewed before fanout opens**:

| File | Content | Bun analogue |
|---|---|---|
| `docs/specs/ecosystem-entity-components.tsv` | one row per (entity, component): substrate, tier, status, owning crate. The §4.0.1 and §4.1 tables, machine-readable — so a lane looks up rather than re-derives | `LIFETIMES.tsv` |
| `docs/specs/ecosystem-PORTING.md` | the **mechanical rule set**, no prose: which tier a new entity takes and why; relationships MUST ride a property `config.link` (§0.12); a published type MUST have a policy attached (§0.13); every consequential mutation is an Action carrying `authorizing_grant_id`; `milli` fixed-point for quantities; `object_types` vs `ont_object_types` (§0.7); migrations start at 0207 | `PORTING.md` |
| `docs/specs/ecosystem-LANES.tsv` | **one row per lane**: crates, owned paths, **migration slot block from 0207**, and the widenings it may take — with W11-W13 in it, since those three are independent (below) | the reservation half of `PORTING.md` |

**The lane table instantiates an existing mechanism; it is not a new one.** `docs/program/LANE-PROTOCOL.md:89`
already says migrations are *"single global sequence, highest `0204`. Blocks assigned per lane in the Phase-0
commit; take the number immediately before push"*. **That quoted high-water is stale — 0205 landed and 0206
is in flight in lane-1, so blocks are assigned from 0207** (Phase 7 carries the correction rung). This table
is that Phase-0 commit's artifact. Two facts
make it the binding constraint rather than a convenience:

- **Nine 0207+ slots are already claimed** against an unallocated serial resource — D2/T5 ≤2, T2 1, D3 2,
  N3 1, N5 1, N1 1, T10 1 — before any lane opens.
- `check_migration_versions` (`backend/ci/gates/migration-safety/src/lib.rs:131-141`) enforces **gap-free
  contiguity**: a missing version between two present ones is a `NonContiguousMigrationVersion` violation. So
  a lane cannot hold a reserved number open while another lane lands past it. **The version space serialises
  Phase 4** — this, not any CI collision, is what orders the work.
- **Non-foreclosure: no migration 0207+ may hard-code `'KR'` or `'KRW'`.** Korea is HOLD (§5.4); a literal in
  DDL is the one form of that assumption that cannot be withdrawn later.

Two further Phase-0 artifacts, and they are prerequisites rather than deliverables:

| Artifact | Content |
|---|---|
| **the D3 write-path enumeration** | one row per authority-mutating write path, in `docs/specs/ecosystem-entity-components.tsv`: the path, its `with_audit` status, and whether a null capacity there is a defect. Moved here from Phase-7 prepwork because it is the artifact `capacity_recorded_on_every_authority_mutation` (§7) **reads** — a probe cannot precede the list it asserts over |
| **X-CITE** | a mechanical citation audit of this plan, as a plan deliverable. The citation failures found in review were **systemic, not clerical** — an ADR Decision line was propagated as a state fact for hours, and adding a header to an upstream file invalidated every line-number anchor into it at once. A one-time human pass does not close a systemic defect |

**Prerequisite 5.7a — harden the audited-table `DROP COLUMN` resolver — gates ANY migration 0207+, and Slice
0 lands migrations at 0207+.** Verified by direct read: `table_name_after_alter_table`
(`backend/ci/gates/migration-safety/src/lib.rs:314-322`) advances past only `if exists`; `tokenize_sql`
(`:443-460`) emits a token boundary at **every** character that is neither ASCII-alphanumeric nor `_` —
including `.`; and
`built_in_audited_tables()` (`:164-172`) holds neither `only` nor `public`. So
`ALTER TABLE ONLY users DROP COLUMN x` resolves the table name to **`only`** and
`ALTER TABLE public.users DROP COLUMN x` resolves it to **`public`**, and neither raises
`DropAuditedColumn`. The `#[test]` count in that file is **0**. The fix is one negative unit case per
spelling.

**Prerequisite 5.7b — a Leptos-shape extractor in `route-inventory.mjs` plus the reciprocal assertion in
`validate-console-truth-ledger.mjs` — gates any console surface.** Not on Slice 0's path, recorded so it is
not discovered by the first lane that needs it.

**One reconciliation line, owed and not yet done:** reconcile §4 and §5 against the benchmark matrix and the
four research surveys, recording adopt / reject / contradict per row with the source's own confidence label
carried through, and **no plan decision resting on an UNCERTAIN or UNKNOWN row**. Today a grep over all of
this plan returns **zero** hits for benchmark, research-, Foundry, Workday, SAP, Odoo, NetSuite, ServiceNow
and Salesforce — so the surveys exist and this plan cites none of them, which is either an omission or an
implicit rejection, and only one of those is admissible.

**The first two rows** are derived from work already done in this plan, so writing them is transcription, not
design. The lane table, the D3 enumeration and X-CITE are not.

### Phase 1 — the immutable target

Bun's was the existing test suite: **60,624 tests on Linux x64** (macOS arm64 58,850, Windows x64 57,337),
**0 skipped, 0 deleted.** The platform qualifier matters because the figure is a per-platform count, not a
suite total; the *0 skipped, 0 deleted* half is what actually transfers. A target you cannot renegotiate
is what makes a large diff safe. Ours, and it must exist before fanout:

| Target | Rule |
|---|---|
| **Every job in `.github/workflows/ci.yml`** | pass unchanged. No gate weakened, no allowlist widened without its own justified commit. **No count is restated** — there are ten as of `8e76dffb4` (`preflight:75`, `support-domain-unit:163`, `postgres-domain-reachability:194`, `company-conformance:244`, `generated-face-authority:291`, `backend:340`, `dev-up-smoke:684`, `repo-gates:741`, `api-contract:827`, `kubernetes-manifests:906`), the plan previously said 14, and a number in prose rots while "every job" does not |
| `tools/lanes/fanout.py run` | **0 out-of-slice writes.** Already tooling; do not build another |
| `docs/specs/known-bad-controls.tsv` | **the real immutable artifact.** One row per probe: probe name, known-bad input, and the commit where it was **observed RED**. **No probe may enter the suite without a RED record.** |

That last row is the direct analogue of Bun's "0 skipped". Six probes were defective in one session here;
a GREEN with no recorded RED is not evidence, and the ledger makes that structural rather than cultural.

### Phase 2 — the experiments (before the design is trusted, and before the trial run)

**Renumbered from Phase 6.** This phase's own heading always claimed the order the numbering contradicted —
*"before the design is trusted"*, and X4's row says *"so test it first"*. Principle 5 (§1) is the authority: a
probe is untrusted until it has been RED, so the experiments cannot follow the slice they exist to justify.
This is a renumber, not a redesign.

Each has a falsifiable prediction and a known-bad control. A probe proven RED on a known-bad input before
its GREEN is the only kind that counts.

| # | Experiment | Prediction | Known-bad control | ANSWERED | If refuted |
|---|---|---|---|---|---|
| X1 | **Edges from an authored type.** Publish a type whose relationship is declared *only* as a link type, then again with a property carrying `config.link` | first writes 0 edges, second writes edges | the link-type-only case must be **RED** (it is today's behaviour, §0.12) | **YES — CONFIRMED.** `docs/ideas/experiment-x1-x2.md`; re-runnable at `docs/ideas/experiments/x1/run.sh`. Sharpened to *no **reachable** path* (§4.3) | the canvas cannot express relationships; §0.12's guard becomes a blocking fix |
| X2 | **A published type lists rows.** Publish with, and without, an attached object policy | without → `[]`; with → rows | the no-policy case must be RED (`residual.rs:200-203`) | **YES — CONFIRMED.** `experiment-x1-x2.md`; `experiments/x2/run.sh`. Measured `200 OK []` then `201 Created` → rows | every Tier N entity is unusable; Tier T for all of them |
| X3 | **The definer survives attack.** Build `effective_grants_for`, then run the #525 exploit shape and a foreign-org party id against it | both refused **by execution**, not by argument | a definer filtering on a parameter instead of `app.current_org` must leak — the `0060:99` shape copied verbatim | **no — and it CANNOT be prepwork.** It needs `effective_grants_for` to exist, so it is **slice-0 work**, run at ladder rung 4 | the bootstrap resolution fails; grants cannot be read outside the gate |
| X4 | **No second RLS dimension is needed.** Answer `effective(party, scope)` for a person in two orgs using only `app.current_org` + the visibility edge. **Bounded to visibility of a known party within the armed org**; the falsifying case is a **group**-scope step whose only qualifying holder is a user of a sibling org, which X4b measured returning 0 rows (§4.2) | answerable with zero new GUCs, for the bounded claim | an attempt that requires `app.current_group`. **Instrument control:** a GUC inventory read from `pg_settings` rather than from stored policy expressions returns 0 rows and looks like a pass (§7 preamble) | **YES — CONFIRMED.** `docs/ideas/experiment-x4.md`; `experiments/x4/run.sh`. 30 assertions, 3 controls RED, zero new GUCs, the 141 RLS policies untouched | §4.2 collapses and the 141-table cost returns — this is the plan's central claim, so test it first |
| **X4b** | **Can a group-scoped grant live in Tier N?** Attempt the `grant → group` edge, then read a group-scoped grant revision from a sibling org in the same group | the edge is storable and the sibling reads it | the FK rejection, and the sibling's row count, both measured rather than argued | **YES — CONFIRMED, and it REFUTED the plan.** `docs/ideas/experiment-x4b.md`; `experiments/x4b/run.sh`. The `organization → group` edge is FK-rejected and a sibling org reads **0** rows. §4.1 and §4.3 are corrected accordingly; **Slice 0 is not blocked**, because its grants are 현장-scoped and intra-org | (already refuted — the correction is the group-scoped grant store in Tier O) |
| X5 | **Cedar decides alone.** Encode the four grant sources over one person in two companies as a **constructed query with an expected-fail baseline**, in X4's shipped form: the four sources written out, the specific decision Cedar must reach named, and the **concrete input** on which a Rust-fallback implementation is RED | Cedar alone decides, on that named input | **a specific input, not a scenario:** the encoded four-source case run against an implementation that consults a companion Rust evaluator, which must be RED. (Its previous control — *"a case needing a companion evaluator"* — is a refutation *scenario*, not an observable input, which principle 5 forbids) | **no — and it CANNOT be prepwork.** Needs `effective_grants_for`; **slice-0 work**, ladder rung 4 | two evaluators will diverge; the fold moves entirely into Cedar or entirely out |
| X6 | **Fold cost per request.** Measure `effective(party, scope)` at realistic grant counts, materialized vs on-demand | on-demand is acceptable at slice-0 scale; materialization keyed on `policy_versions` if not | a fold whose cost grows with total org grants rather than the person's | **no — and it CANNOT be prepwork.** Needs `effective_grants_for` **and** realistic grant counts; **slice-0 work**, ladder rung 4 | §5.6's invalidation design becomes load-bearing earlier |
| X7 | **Draft-PR CI coverage.** Push a backend-touching commit and a docs-only commit to a draft PR; compare required contexts | backend runs every job in `.github/workflows/ci.yml`; docs-only runs none | trusting the UI's absence of red as green | **no — unrun for a different reason.** It requires **pushing a branch**, so it is outward-facing and needs **explicit authorization**. It is not a pending prediction; it is a blocked action | the one-PR model needs an explicit verification checkpoint per rung |
| X8 | **How do the CI buck2 jobs currently pass?** | they pass by an identifiable mechanism that must be named before any test is wired to them | this is an **investigation, not a probe** — there is no GREEN to distrust, so the proven-RED discipline does not apply to it in this form | **YES — ANSWERED.** `docs/ideas/experiment-results.md`. `.buckconfig:15-16` `[external_cells]` / `prelude = bundled` supplies the prelude from inside the binary; `tools/buck2:1` is a blake3-pinned DotSlash launcher; the required job **"Support domain — Buck2 unit reachability"** (`ci.yml:164`) runs a real `tools/buck2 test` at `:192`. **buck2 is fully functional**; the earlier "the graph is broken" claim was WRONG | (already answered — no forced migration; the governance question stands) |
| X9 | **Trace one new test end to end:** test file → `rust_test` target → Postgres `sh_test` wrapper → workflow step | each link nameable **by target name** | a test that passes locally and never executes in CI | **YES — ANSWERED.** `experiment-results.md`, traced through a real test (§8 Phase 4) | the work queue (Phase 4) needs a **per-test** CI wiring step |

**The gate, stated so it is not circular.** **X1, X2, X4, X4b, X8 and X9 must have recorded outcomes before
the first implementation commit** — all six do. **X3, X5 and X6 must have recorded outcomes before Slice 0
may be declared green.** The stricter form ("no implementation commit until X1-X5 and X8-X9 are recorded")
forbids the commit those experiments require, because three of them cannot run until the definer exists.

`ADR-0023:79-86` already specifies its own **1-2 day Engine-Gen spike** validating the pre-terminal FSM
shape, with *"if structurally infeasible, execution stops and returns to consensus"*. That is X0 and it is
already decided — reference it, do not duplicate it.

### Phase 3 — the trial run (before any scale)

Bun converted **3 files** with 1 implementer + 2 adversarial reviewers, and only then did all 1,448.

Ours: **slice 0, the ₩100,000 비품 purchase.** One implementer, two adversarial reviewers who see the
**diff only** — the repo's `slice` skill already implements exactly that shape. Fanout does not open until
slice 0 is green *and* every slice-0 probe has a RED record. If slice 0 refutes the model, the cost is one
slice.

### Phase 4 — the work queue, by crate

Next crate activates only when the current one is clean. Derived from §4.1, in dependency order:

| # | Crate | Ships |
|---|---|---|
| 1 | `platform/db` | migrations 0207+: `work`, the **`gov_approvals`** capacity columns (§4.0.3 — the `audit_events` pair is deferred), voucher `accounting_date`, the definer. **`party` and `party_org_visibility` are struck** — deferred out of Slice 0 because Slice 0 does not need them; `users.party_id` is struck on irreversibility, which is a different reason (§4.1) |
| 2 | `platform/authz` | `feature_catalog` ≡ `Feature` gate (C1); grant fold; Cedar subject/resource attrs. **HARD ORDERING: the bundle-schema declaration of `capabilities`, `scopes` and the decision-scope resource attribute lands FIRST** — `Entities::from_entities` validates against the schema (`engine.rs:449`), so an undeclared attribute denies everything (§4.6) |
| 3 | `platform/authz-rest` | the re-validating definer read path (§5.1) |
| 4 | `ontology/*` | the Tier N types + their attached policies; `allowlisted_projected_table` arm for `work` |
| 5 | `identity/rest` | C2 — `policy_feature_catalog()` off data |
| 6 | `finance-gl` | line-level typed dimension; **and the missing `assert_period_open` call** (§5.5) |
| 7 | `messenger` + `platform/realtime` | derived membership; the `authority_changed` event |
| 8 | `app` | `/overview` surface, and **N `ProjectedDispatchRegistry` handlers** — one per `work` action across Slice 0 / W4 / W11 / W13, counted in `ecosystem-entity-components.tsv` (§4.0.2). Sized, not labelled "wiring" |

**CI wiring is per TEST, not per crate — and the template is a worked one, cited by target name.** X9 traced
all four links through a real test:

```
test file    backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs
rust_test    //backend/crates/ontology/rest:console-ontology-rest-itest-object_policy_attach_as_runtime_role
sh_test      //tools/buck:ontology-object-policy-attach-postgres        (the Postgres wrapper)
workflow     the step listing that sh_test target (`.github/workflows/ci.yml:239`)
```

Target names rather than line numbers, because the line numbers in that chain have already drifted once.

**Why per test and not per crate:** `mapped_srcs` hand-lists every file a test crate reads, and **buck2 does
not glob**. So a file added later to a shared harness is invisible to the target until that list is edited —
which is exactly how link 2 or link 3 goes missing while the test passes locally and **nothing fails**. A
per-crate step cannot see that; only a per-test assertion can. `scripts/check-ci-preflight.mjs:430-453`
already enforces this shape for `ontology/rest`, and its own header says the fix elsewhere is *"a per-crate
decision with the same shape as this one, not a cleverer regex"* (`:428`) — so this is adoption, not
invention.

### Phase 5 — the progressive verification ladder

Bun's was `cargo check` → `bun --version` → one test file → 100 sharded random files → **full CI on all
platforms** (the primary source supports "all platforms"; no primary reading supports six). Ours, cheapest
first, each rung gating the next:

1. `cargo check` on the touched crate
2. the migration applies, and **re-applies onto a populated DB**
3. the tenant-isolation gate classifies every new table (`tenant-isolation/src/lib.rs:804-808`)
4. the definer probes — `definer_ignores_parameter_org`, `definer_returns_no_foreign_org_grant`, and the
   named re-validation deletions (`org_predicate`, `visibility_predicate`, `chain_linkage`,
   `scope_containment`). **`X3`, `X5` and `X6` run here**, not in Phase 2: all three need
   `effective_grants_for` to exist, so they are slice-0 work rather than prepwork (Phase 2)
5. slice 0 end-to-end, with its RED records
6. slice 1 (promotion) end-to-end
7. every job in `.github/workflows/ci.yml`
8. `tools/lanes/fanout.py run` — 0 out-of-slice writes

### Phase 6 — one PR

Single long-lived consolidation branch, **one PR, no incremental merges**. A permanently-open **draft PR**
gives CI on every push, because `pull_request:` (`.github/workflows/ci.yml:36`) has **no `branches:`
filter** while `push:` covers only `main` and tags (`:3-8`).

**Two caveats I verified, and both bite this plan specifically:**

- `pull_request:` carries a **`paths:` filter** (`:37-61`) that does **not** include `docs/ideas/**`. So a
  docs-only commit gets **no checks at all** — and *no checks* looks identical to *passing checks* in the
  UI. Silence is not success.
- `concurrency` sets `cancel-in-progress: true` for PRs (`:72`). On a branch taking many commits, most
  intermediate runs are **cancelled, not verified**. Verify at rung boundaries deliberately rather than
  assuming per-commit CI.


### Phase 7 — prepwork before fanout, enumerated

LANE-PROTOCOL §4:72-78 ranks ownership mechanisms: **① NOT SHARED → ② PRE-RESERVED → ③ SERIALISED**, with
*"Prefer the earlier one — later ones rely on discipline, and discipline is what fails."*

| Rung | Prepwork item |
|---|---|
| — | **The governance records of §5.11**, with the numbers allocated there. Under `README` rules 2-4 these *gate* the work. **D2 (ADR-0028) and D3 (ADR-0029) block slice 0**; D2 subsumes the old G2b, so it is also what gates C5. G1 is withdrawn, G6 and G7 struck |
| — | Phase 0 reference documents; Phase 1 immutable target incl. the empty `docs/specs/known-bad-controls.tsv` |
| **②** | ONE pre-reservation commit: `LANE_TYPES: [&str; 5]` widened (`ontology/rest/tests/company_conformance.rs:184`); `allowlisted_projected_table` arms (`instances.rs:1479-1498`); `object_types` kind rows for `work`/`lot`; **`link_types` rows for each new edge kind** (`work_artifact`, `person_artifact` — §4.3: `console_rt` has SELECT only, so each is a migration); `RealtimeEvent` variants + channel consts; migration slots 0207+ |
| — | *(moved to Phase 0)* The **D3 write-path enumeration** is a Phase-0 artifact, not Phase-7 prepwork — `capacity_recorded_on_every_authority_mutation` reads it, and a probe cannot precede its own input. **The exclusion set has TWO entries**, each bound to a (file, function) pair; see D3 in §5.11 |
| — | **CI wiring per TEST** (not per crate — see Phase 4), targeting the CI that **exists** (buck2 live, X8 ANSWERED) not the one `docs/PIVOT-2026-07-28.md` §6 describes |
| **①** | Everything else — the new tables, the definer, the capacity columns, each in files no other lane owns |
| **③** | `backend/crates/ontology/adapter-postgres/src/seed.rs` `BUILTIN_CATALOG_VERSION` — *"the one true bottleneck"* (`docs/program/LANE-PROTOCOL.md:90`), **inherited, not introduced**. 0204 made installs additive and version-keyed, so lanes can ship disjoint catalog versions; until that fully lands, serialise it |
| — | **Correction rung: `docs/program/CATALOG.md:62-68` lists a type set that never shipped.** It names OrgUnit / Position / Person / Employment / PayRun; the shipped set is company / org_unit / job_position / employment / pay_run — **`Person` never landed**. Correct it to the shipped names, or the next plan budgets against a catalog that does not exist |
| — | **Correction rung: `docs/program/LANE-PROTOCOL.md` is stale in three places, and one of them would be "fixed" wrongly.** (a) Its status header reads *"Status: **prep artifact, not yet exercised.** Fan-out is not authorized until §4 passes."* — stale against `docs/program/console-program-ledger.md:769` (*"the fan-out is green"*) and `:751`. §8 must cite the **corrected** header where it opens fanout, not the stale one. (b) Its migration high-water at `:89` still reads **`0204`**: **0205 landed, 0206 is in flight in lane-1, so reserve from 0207.** (c) `:268-269` says *"this repo has **no `.cargo/config.toml` and no `[profile]` section**"*. Correct **only** the second half — `[profile]` landed (`backend/Cargo.toml:359` `[profile.dev]`, `:362` `[profile.test]`) and sccache is wired via the subprocess environment with a measured **0% → 35.4%** (`docs/program/console-program-ledger.md:675`). **Keep "no `.cargo/config.toml`", and record WHY it must stay absent:** the ledger states the file *"would apply in CI where no runner has sccache and **every Rust job would fail**"*. Without that reason recorded, a later lane reads the line as a TODO and breaks every Rust job |

**Build-system governance is unresolved and this plan must not assume either side — but the status quo is
healthy, so there is no forced migration.** `docs/PIVOT-2026-07-28.md` §6 decides *"Build system: cargo, not
buck2"* — **unexecuted**. And `PIVOT-2026-07-28.md` **is not in `docs/decisions/`**, so under
`docs/decisions/README.md` rules 1-2 it binds nothing; neither cargo nor buck2 is an accepted decision. Three
documents hold three positions (`docs/ideas/governed-object-engine-PLAN.md:75` "buck2 RETAINED" vs its own
`:301-302` "dropped"; `docs/ideas/no-code-ontology.md:133-141` builds on Buck wiring). **Flagged as an open
governance question, not a premise.**

**What is NOT open, and what this plan said wrongly: buck2 is fully functional.** An earlier draft here said
*"`prelude/` is missing so the buck2 graph is already broken"*, and inferred that CI's buck2 jobs must pass
by some accident. Both halves are false, and X8 measured the chain:

- `prelude/`'s **absence is correct**. `.buckconfig:15-16` declares `[external_cells]` / `prelude = bundled`,
  which is buck2's own mechanism for supplying the prelude **from inside the binary**. There is nothing to
  vendor.
- `tools/buck2:1` is `#!/usr/bin/env dotslash`, a launcher with **per-platform blake3-pinned digests** — the
  runtime is hash-pinned, not floating.
- The required job **"Support domain — Buck2 unit reachability"** (`.github/workflows/ci.yml:164`) passes
  because `:192` runs a real
  `tools/buck2 test //backend/crates/support/domain:console-support-domain-unit`. Not a no-op, not a path
  filter, not a cached graph.

The **five buck steps** count is also dropped: it was wrong (install steps appear at `:103`, `:176`, `:215`,
`:271`, `:307`, `:398`, `:703`, `:860`, with `tools/buck2 test` at `:192`, `:465`, `:660`, `:664`, `:675`) and
it was load-bearing for nothing. Cite **the job by name** instead. Phase 7's *"targeting the CI that exists
(buck2 live)"* is therefore **positively grounded**, not a bet.

Also: `rust-toolchain.toml` pins **1.97.1**; `docs/specs/foundation-gates.md:60`'s 1.96.0 is stale
(`backend/Cargo.toml:53` `rust-version = "1.96"` is the MSRV floor, not the toolchain).

**Deployment dependency this plan does not own and must not plan to flip:** every ontology WRITE runs on
the command pool, `command_pool()` is `None` unless `ONTOLOGY_COMMAND_DATABASE_URL` is set, no production
overlay references the component — *"green on every PR and dead where it ships"*
(`docs/ideas/no-code-ontology.md`, evidence at `backend/app/src/lib.rs:2925-2930` and
`backend/crates/ontology/rest/src/lib.rs:1786-1790`). **So slice 0's Tier T half is CI-provable; exposure
remains HOLD for both halves.** The earlier wording — *"lands and ships"* — claimed a release this plan
cannot grant. `docs/program/console-capability-registry.json` carries `"implementation": "HOLD"` on **27 of
27** capabilities and `"exposure": "HOLD"` on **27 of 27** (counted), and
`scripts/console/validate-console-truth-ledger.mjs` fails any jurisdiction control whose
`release_disposition` is not `HOLD`. `docs/program/console-program-ledger.md` states it outright: *"Nothing
in the idea document is approved work."* CI-provable and deployable are different claims, and only the first
is available. This is still a second, independent reason `work` is Tier T.

**One Phase-7 rung follows from that, and its enforceability must be stated with it.** Slice 0 and each
widening group are registered as capability rows carrying a signature story, an `evidence_path`, leaf
commands and ownership roots. This is a **governance step required by `dispatch_rule` prose** — and
`dispatch_rule` and `hold_rule` have **no executable reader**: `grep -rn dispatch_rule scripts/ backend/
tools/ .github/` returns **nothing**, so both are fields nothing enforces. The constraints that *are*
executable, and that a registry row must therefore satisfy, are in
`scripts/console/validate-console-truth-ledger.mjs:254-257`: buck2 targets are keyed on
`delivery.rust_status`, a `REQUIRED` unit with empty targets fails, a `REQUIRED_UNRESOLVED` unit must stay
`HOLD`, and **every declared target must resolve**. Plus the jurisdiction-control HOLD loop in the same file.
No Buck2-clause amendment is proposed or needed — see the build-system paragraph below.

### Slice 0 — the ₩100,000 비품 purchase, terminal at a 현장

Minimum shape of each entity, and nothing more:

| Entity | Slice-0 minimum |
|---|---|
| `party` | **not in Slice 0** — deferred because Slice 0 does not need it, **not** on irreversibility: it is a new table and therefore droppable (§4.1). The grant's `subject` is the raiser's `users.id`, an ordinary tenant-scoped UUID |
| `party_org_visibility` | **not in Slice 0** — deferred with `party`, same reason. `visibility_predicate` (§5.1) binds against `users.org_id = current_setting('app.current_org')` until the edge table lands, which is the same predicate against a table that already exists |
| `users.party_id` | **not in Slice 0** — `users` is audited, so the column is permanent once landed (§4.1) |
| `org_unit` | 1 instance, `kind = 사업장`. No legal attributes. |
| `work` | **1 `work` ROW written by the domain use-case, listed through the projection** — a projected type has no instance-create path (`instances.rs:1443-1450`) — with `work_scope` as a scope-descriptor property naming that 현장 |
| `grant` | 2 instances: `purchase.approve` at 현장 scope (authorises) + one at a **different** 현장 (must not) |
| `delegation_rule` | 2 rows: the ≤₩1,000,000 band → 현장 terminal; the >band → 본사 |
| `approval_template` | 1 template, 1 step, `mode = terminal_if_전결` |
| `approval_line` | 1 instance, both raised and executed lines persisted |
| `gov_approvals` | 1 row carrying *(signer, authorising grant, scope)* — the shipped signature store, not a new entity (§4.4) |
| `effective_grants_for` | the definer, with the four **named** re-validation checks of §5.1: `org_predicate`, `visibility_predicate`, `chain_linkage`, `scope_containment` |

**Acceptance.** Every `slice0_*` probe in §7 GREEN, **and** each of its known-bad controls observed
RED. The two-grant and two-band rows exist specifically so the fold is a fold and the lookup is a
lookup (pre-mortem 3).

Two additions the addenda make non-optional even at minimum depth, because omitting them would prove
the wrong thing:

| Entity | Slice-0 minimum | Why it cannot wait |
|---|---|---|
| `gov_approvals.authorizing_grant_id` + `.on_behalf_of_party_id` | both columns land; `authorizing_grant_id` is populated on the one signature, and `on_behalf_of_party_id` is **exercised** by `daeri_records_both_parties` (§7) rather than shipped unused | the capacity field is what makes the signature a signature (§4.0.1) — and pre-mortem 4's named failure **is** a capacity column nothing writes, so a column landing unexercised is the failure, not the mitigation |
| `finance_gl_vouchers` | 1 posted voucher with `accounting_date` (**irreversible once landed** — the table is gate-marked audited, `0160:21`), a line-level `branch_id` and a line dimensioned to the work (**also irreversible** — `finance_gl_voucher_lines` carries its own marker at `0160:56`, which earlier drafts did not say while putting two new columns on it), and the `assert_period_open` call the crate does not make today | the purchase has a cost; the header dimension pair already exists, the date and line-level push are §5.5 items 1-2. **One voucher is not evidence the dimension shape is settled** (§5.5) |

**Explicitly out of slice 0:** `party` and `party_org_visibility` and the two `party_id` columns (§4.1 — the
tables because Slice 0 does not need them, `users.party_id` on irreversibility; they land together in **W2**),
group-scoped grants and the one Tier O store that would hold them, control edges,
group designation, `Feature` work, the canvas, 합의, 협조/보고 edges, employment, employment type, PII
attributes, retroactive linking, lots, metrics, allocation.

### Slice 1 — promotion (승진 / 인사발령): the write path

Slice 0 proves read/decide. Slice 1 proves **write**, and it is the smallest operation touching
structure, authority and 결재 at once.

| Entity | Slice-1 minimum |
|---|---|
| `position` | 2 instances, both carrying a `position_at_scope` scope-descriptor property — one at the 현장, one at the company |
| `assignment` | the old closes at 발령일, the new opens — two revisions, never an update |
| `approval_template` | 1 인사발령 template |
| `approval_line` + a `gov_approvals` signature | with capacity (`authorizing_grant_id`) |
| `grant` | position-sourced, opening and closing on the same 발령일 |
| `assignment` **kind** + **return right** | the two new authored properties (§4.1), exercised rather than declared: 육아휴직 복직 is statutory and HR+payroll is the first vertical, so this entity carries them either way — as a property now, or as a reshape of a shipped type later. `holds_position` is already ManyMany, so a substitute's concurrent assignment is already expressible |

**Acceptance.** `asof_replay` GREEN across the 발령일 boundary (the fold differs either side);
`demoted_member_retains_standing` GREEN; `basis_survives_the_chain` GREEN; assigned `work` and in-flight
lines demonstrably unchanged (§5.9). Demotion is the same slice run in reverse and must produce **grant
expiry, never deletion**.

### Widenings, each an acceptance criterion

| # | Widening | Acceptance |
|---|---|---|
| W1 | Obligation loop: extend `notices` with a content-bearing 조치보고 leg, an originator closure state, a **party-keyed recipient** superseding the org-composite FK (`0162:50`), and **per-recipient audience targeting** (§4.4's fourth gap). **All of it ADDITIVE — `notices` and `notice_receipts` are both gate-marked audited (`0162:12`, `:40`), so no `DROP COLUMN` is available** (§4.4) | `obligation_notifies_line_as_raised` GREEN **including a recipient in another company and with every non-member receiving nothing**, against the shipped org-wide snapshot as its known-bad control; post-확정 correction GREEN; no second ack mechanism exists; and `recipient_user_id` still present, nullable |
| W2 | **The party family lands here** — no other widening carried it, and W2 is the first that cannot proceed without it: the `party` table (sentinel-homed, §4.1), `party_org_visibility`, `users.party_id`, `employees.party_id`. Then `employment` revised: `party_id` replaces `person_name`; employer split from worksite | `party_is_invisible_and_unmintable_from_a_tenant`, `visibility_edge_rls` and `visibility_unique_key_leads_with_org_id` GREEN with their controls RED; `visibility_predicate` (§5.1) moves from `users` to `party_org_visibility` **unchanged in form**; a 파견 employment with employer ≠ worksite round-trips |
| W3 | `org_unit` kinds/lifetime + `worksite_registration` (Tier T, projected) | duplicate 사업자등록번호 rejected by the DB; a bounded TF expires |
| W4 | `work` handover + `assignment` as a grant source; **and the fixed-authority 인계 완료 count** without which hard-gating is not available (§4.5) | `handover_is_scope_bounded` and `handover_moves_work_artifacts_only` GREEN; the 인계 완료 **assertion** recorded with its asserted count. Hard-gating offboarding lands **only** with the fixed-authority count — until then the assertion is evidence, not a gate |
| W5 | Remaining grant sources + `position` + `authority_rule` + named `*OrgWide`/`*GroupWide` reach capabilities (§0.17 — no DSL) + `delegation_rule`'s `(period, cumulative_limit)` pair (§4.7) + **a department level on `AccessScopeLevel` if 부서-scoped grants are wanted** (§4.1 — a kernel enum change with two exhaustive `match` sites and its own `branch_scope_for_org` arm) | **requirement 3 provable**; `fold_is_additive` still GREEN with all five sources; and if the department level lands, both `match` sites compile with a decided projection rather than a wildcard |
| W6 | `employment_type` as authored data; both CHECK vocabularies (`0172:7`, `0187:22`) retired | 파견/도급/일용/프리랜서 expressible; neither CHECK remains |
| W7 | `party_link` control edges (Tier O) + derived `group_designation` | a joint venture under two groups, a nested group, and a 순환출자 cycle all resolve; `group_memberships UNIQUE (org_id)` (`0060:36`) and `organizations.group_id` (`0060:27`) collapse to one representation |
| W8 | Cedar scope hierarchy: populate parents at `engine.rs:392`/`:425`, extend `:449`, declare in schema | a group-scoped approver signs a company-raised document, decided by Cedar alone with no Rust fallback |
| W9 | `Feature` sequencing C1→C6 (§5.3) | every coexistence-map entry `cedar_only`; `matrix_row` and `Role` deleted; `Feature` retained |
| W10 | Canvas over the authored types, four-eyes on every authority change. **Deferred by follow-up and off the slice-0/1 critical path — NOT gated on a charter** (§5.11 G6: no charter clause exists) | no authority change lands without a `gov_approval_consumptions` row |
| W11 | Derived channel membership; `messenger_threads.work_order_id` (`0012:11`) generalised to `work` | `channel_membership_is_derived` GREEN; the conversation follows the work on 인계 |
| W12 | Realtime authority propagation (§5.6): one `RealtimeEvent` variant, one channel, invalidation keyed per `(org, user)` bumping **both** counters | `realtime_push_carries_no_capability` and `stale_client_button_is_refused` GREEN |
| W13 | `work` metrics: the new fields (§4.1) + cycle-time aggregates over Tier T | `no_duplicated_fact` GREEN; an aggregate over 10k rows does not fold revisions |
| W14 | The pre-terminal finalization path (ADR-0023) end to end, incl. the compensating document and its contra voucher | `posted_voucher_cannot_be_rewritten` GREEN via the `0160:78-118` trigger; **and** `assert_period_open` called from finance-gl |
| W15 | `worksite_contract` + disband/transfer (§5.10) | all four disband/transfer probes GREEN |
| W16 | `lot` + `lot_split` + contract lines (§5.8); `inventory_consumption_events.source_kind` (`0156:87`) generalised | `lot_conservation`, `lot_uom_conversion_recorded`, `lot_traversal_up` GREEN |
| W17 | E4 fold simulator — the fold against a hypothetical grant set, over the shipped receipt ceremony and Cedar simulation | a role change is inspectable before commit; neither existing half is replaced |
| W18 | E1 explainability surfaced, incl. a reason for deny-by-omission | `deny_by_omission_is_explained` GREEN |
| W19 | **Segregation of duties** (§5.11 SoD row): conflict pairs over `Feature`, refused at **grant-authoring time**, where the `gov_approvals` four-eyes check already runs | `conflicting_grant_pair_refused_at_authoring` GREEN, with a fold that accumulates a conflicting pair silently observed RED first |
| W20 | **E2 the character sheet + E7 the authoring-effort bar** (§4.8) | `every_entity_has_a_home` GREEN over one row per §4.1 entity, failing on an unmapped one; and the 전결규정-band authoring step/screen count recorded against the guild-bank baseline |

Ordering notes. W7 collapsing the duplicate org→group representation is a defect fix independent of this
design and may land earlier if a lane is free. **W11-W13 depend on `work` (slice 0) but not on each
other** — three lanes, no shared files, rung ① . **W16 is the largest single widening** and is the only
one introducing a new entity class; it is last among the substantive ones deliberately, because
everything before it widens something proven.

**Not in this plan, and argued rather than dropped (§5.5):** the ERP finance subsystem — account master,
multi-currency, depreciation and accrual generation, overhead allocation, shared-service charge-out,
inter-company charges. It is a peer plan, not a widening. Folding it in would double this plan and couple
the authority model's delivery to accounting design. The entity model is shaped so that plan is additive:
economics is already a dimension on an existing posting table, and allocation is a voucher with a
recorded basis.

---

## 9. ADR block

**Decision (1 of 2 — identity and authority).** Introduce one platform-level, attribute-free `party` identity in the owner-only tier,
made visible to tenants through an ordinary org-scoped `party_org_visibility` edge under the existing
`app.current_org` RLS floor, and model authority and 전자결재 as ontology instance types whose effective
state is an additive fold over effective-dated, fixity-chained grants. `work` is a Tier T table projected
into the ontology, because aggregate metrics cannot come from a revision fold and no read model exists.
Retain the `Feature` enum as the capability and Cedar action vocabulary; delete `Role` and `matrix_row`.

**Decision (2 of 2 — components).** Model concerns as **components** with contracts, and entities as
compositions of them (§4.0). Both sets are open; any table in this plan is the current set plus a stated
extension mechanism. Extend the **`record`** component's contract with `authorizing_grant_id` +
`on_behalf_of_party_id` on `audit_events` — a gap every entity composing `record` inherits, which is what
makes two nullable columns the highest-leverage change here. Build the **`economics`** component by
**extending `finance_gl_vouchers`** with a business date and a typed, line-level object dimension: there is
no general ledger to reconcile to, and the voucher already owns the DB-enforced balance gate and POSTED
immutability that are the expensive parts. **Cost** is a query over voucher lines, never a stored field — a
design freedom taken deliberately rather than a constraint inherited. Revenue and profit are **not** claimed as
queries: they need the account master and sign convention the peer plan owns (§5.5).

**Standing of this document.** It is a plan, so under `docs/decisions/README.md` rule 4 it decides
nothing. Its output is the governance records of **§5.11** — five reciprocal amendment pairs (ADR-0027 to
ADR-0031) and four or five non-amending records (ADR-0032 onward) — of which **D2 (ADR-0028) and D3
(ADR-0029) block slice 0**. D2 subsumes the old G2b and so is also what gates `Role` deletion. The old G1 is
**withdrawn**: its premise (that ADR-0022 decides identity is "org-scoped") is false, and its deliverable
claim was undeliverable, so the governance action is **D1**, a *narrowing* amendment to ADR-0022, not an
authorisation for a platform identity. Where a matter is already accepted — finality
(ADR-0023), the Cedar strangler (ADR-0021), the workflow engine and its org-local spine (ADR-0018),
branch scope (ADR-0003), local identity (ADR-0022) — this plan states **only the delta** and inherits the
rest.

**Drivers.** (1) The 141-table org-isolation floor must not be weakened or duplicated. (2)
Canvas-editability and replay must be free, which means Tier N wherever possible. (3) Payroll is the
first vertical and PII is where its obligations attach, while every Korea control reads HOLD.

**Alternatives considered.** A second tenancy dimension `app.current_group` — invalidated by the
requirement it serves, since a person can work across groups. `party` in the global-read tier —
invalidated by confidentiality and by the tier's own "no tenant data" meaning. `party` in the tenant
tier with a matching service — invalidated as the status quo, whose own backfill declines every
ambiguous row (`0076:40-46`) and carries a confidence model
(`0075_employee_identity_resolution.sql:6`, `:13`).

**Why chosen.** It is the only option that meets the confidentiality requirement without a second
tenancy dimension, and it reuses four shipped mechanisms rather than adding any: the tier
classifications the CI gate already enforces, the `SECURITY DEFINER` resolver pattern (`0060:99-126`),
the `object_links` edge store (`0102:54`), and the re-validating-read bargain
(`backend/crates/platform/authz-rest/src/store.rs:576-593`, `0205:69-74`). The cost, corrected: **two
owner-only tables** — `party` **and** the group-scoped grant store — each carrying its own
`owner_only_table_allowlist` classification and its own audited definer, so **one definer per owner-only
store**, not one in total; two tenant tables; and two nullable columns on `gov_approvals`. Both owner-only
tables are deferred out of Slice 0 (§4.1), so Slice 0 pays for **one** tenant table and the two columns.

**Consequences.** A standing `SECURITY DEFINER` surface that must be re-proven on every change, with
the parameter-trust weakness of its own precedent as the named failure mode. Cross-store pointers mean
party-rooted queries are two hops in two stores and referential integrity to `party(id)` is
app-enforced. `Feature` survives, so the "delete the matrix" work is smaller than the input estimated
and the ~500 call sites are never touched. Personal data never moves, so the HOLD controls do not gate
slice 0 — and never putting attributes on `party` becomes a permanent invariant, not a staging posture.

**Follow-ups.**
- `docs/specs/cedar-pbac-coexistence-map.json` gains a terminal end state and four new domain entries
  (§5.3 C4).
- `group_role_grants.group_role` is a CHECK over three literals (`0060:45`); configurable group roles
  require it to become a reference — a migration on an owner-only table.
- Two object registries coexist (§0.7); every new entity must declare which one names it.
- Open: **who may author authority, and in which scope?** Four-eyes answers *how many*, not *which
  scope*. A group-scoped authority editor can grant into every company in the group. This is not
  resolved by this plan and should not be treated as resolved.
- **`docs/ideas/no-code-ontology.md` is stale** on the publish route (§0.13): `OBJECT_TYPE_LIFECYCLE_PATH`
  and `OBJECT_TYPE_POLICIES_PATH` now exist (`ontology/rest/src/lib.rs:201-202`). Correct it or the next
  plan will re-derive a blocker that has been removed.
- **The two silent-empty traps (§0.12, §0.13) deserve one gate between them**, not two guards in two
  plans: a published type with no attached policy, and a link type with no property referencing its
  `stable_key`, are the same class of "configuration that looks correct".
- `inventory_consumption_events.source_kind` is a two-value CHECK bound to `work_orders` by FK
  (`0156:87`, `:107`); generalising it to the `work` dimension is W16's prerequisite.
- The GL has no account master (`account_code` is free `TEXT`, `0160:62`) and is single-currency
  (`amount_won BIGINT`, `:64`). Both belong to the peer finance plan, named so it is not discovered late.
- `docs/ideas/authority-and-approval-model.md` should be marked SUPERSEDED by this document, or have
  §0.1's contradiction corrected in place — its Recommended Direction currently contradicts its own
  body.
