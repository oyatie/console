# Ecosystem plan — policy, organization, identity, approvals as one entity model

> Status: PENDING APPROVAL
> RALPLAN-DR planner pass, 2026-07-29. Deliberate mode (auth/security + migrations + PII).
> Every "what exists" claim cites **executable** code or DDL. Line numbers re-verified this session.
>
> **This is a DELTA, not a fresh design.** Finality is decided by ADR-0023, the Cedar strangler by
> ADR-0021, the workflow engine and its org-local spine by ADR-0018, branch scope by ADR-0003, local
> identity by ADR-0022. Where a matter is accepted, this plan states only the delta and inherits the rest
> (`docs/decisions/README.md` rules 1-6). **A plan cannot supersede an accepted ADR** (rule 4), so this
> document's real output is the reciprocal ADR pairs **G1-G9 in §5.11** (G7 needs none) — **G1, G2 and G2b
> block work**, G2b specifically blocking `Role` deletion (§0.16).
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

`authority-and-approval-model.md:89-92` retracts group-scoped people: *"The group is not high enough…
Group-scoping relocates the duplication rather than removing it."* Then `:545-546` recommends exactly
that — *"People are group-scoped… the group is the tenancy boundary for people"* — and `:575-579`
sizes `app.current_group` across 141 RLS tables as *"the largest single engineering cost in the chosen
model."*

That cost is incurred **only** by the retracted option. The body's own conclusion (person at the
platform, tenant owns the edge, `:83-87`) needs **no second tenancy dimension at all** — see §4.2.
The largest line item in the input's cost model is an artifact of an internal contradiction.

### 0.2 `Feature` cannot be deleted — it is Cedar's action vocabulary

`cedar_pbac/engine.rs:430` builds the Cedar action UID from `request.action.feature().as_str()`.
Delete the enum and Cedar has no action names. The brief and the input both treat
"`Feature` × 6-role matrix" as one deletable unit (`lib.rs` `:109` and `:573`); they are two things:

| Thing | Location | Verdict |
|---|---|---|
| `pub enum Feature` — capability *name*, ~500 call sites across 25+ crates | `authz/src/lib.rs:109` | **KEEP.** Cedar's action id; already mirrored as data in `feature_catalog` (`0065:11-14`) |
| `pub enum Role` — the 6 demo roles | `authz/src/lib.rs:35` | **DELETE** |
| `const fn matrix_row(self) -> [PermissionLevel; 6]` — the *decision* | `authz/src/lib.rs:573` | **DELETE** |

Only the decision is scaffolding. This shrinks problem C rather than deferring it (§5.3).

### 0.3 Cedar `parents` hierarchy is unimplemented, not merely unused

Both entities ship with an empty parent set: `engine.rs:392` and `:425` pass `HashSet::new()`, and
`engine.rs:449` hands `Entities::from_entities` exactly two entities, validated against
`bundle.schema`. "The corporate graph *becomes* the Cedar hierarchy"
(`authority-and-approval-model.md:125`) is a property of Cedar the library, not of this engine. It
costs a change at `:392`/`:425`/`:449` plus a schema declaration.

### 0.4 `users` is not keyed `(id, org_id)` — and the keystone is cheaper as a result

`0002_create_users.sql:8` is `id UUID PRIMARY KEY`. `0034_enforce_org_id_rollout.sql:122` **adds**
`users_id_org_key UNIQUE (id, org_id)` so children can pin the tenant via a composite FK; it does not
replace the PK. `employees` is the same shape: `0063:3` PK on `id`, `0076:10` adds the composite
UNIQUE.

The input's *consequence* stands (a user row carries one `org_id`, so one human at two companies is
two rows). Its *mechanism* is wrong, and the correction is load-bearing: **`users.party_id` and
`employees.party_id` are plain single-column FKs to `party(id)`.** No key surgery on either table.
Relatedly, `group_role_grants.user_id UUID NOT NULL REFERENCES users(id)` (`0060:43`) is ordinary
DDL permitted by the PK — not a cross-tenant carve-out earned by special design.

### 0.5 `org_unit` is not production schema

`parent_org_unit_id` appears only in a conformance **test fixture**
(`ontology/rest/tests/company_conformance/fixtures/org_unit.rs:146`). The input's *"`org_unit` today
has only `parent_org_unit_id`… so it is load-bearing while under-specified"* (`:214-218`) describes a
fixture. Org structure is greenfield in production; `home_branch` in `0166` is a separate real thing.

### 0.6 `notices` carries the same cross-company blocker the input rejects elsewhere

The input concludes *"So 통지 → 인지 is built. What is missing is narrower than it first appears"*
(`:378-381`) — naming only the missing content and closure. It missed a third, structural gap:
`notice_receipts` has `FOREIGN KEY (recipient_user_id, org_id) REFERENCES users(id, org_id)`
(`0162:50`), so a recipient **must be a user of that org**. That is the same foreign-key blocker the
input correctly identifies in `gov_approvals` (`0153:78`) and rejects the mechanism for. A group-level
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
- **Zero `CREATE MATERIALIZED VIEW` in all 206 migrations.** Verified repo-wide.

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

### 0.11 `policy_assignment_preview_receipts` is a ceremony, not a simulator — and `policy_versions` is the key

Confirmed: `0065:159-172` stores `actor_id`, `user_id`, `current_branch_ids`, `current_role_ids`,
`role_ids` (proposed), `policy_version`, `expires_at`, `consumed_at`. It records the **inputs** of a
proposed change with expiry and single consumption — a real preview→receipt→consume ceremony, but it
never stores a computed outcome. So "simulate a role or 전결규정 change before committing" is **new
work with a cost** (§4.8), not a free reuse.

The more valuable find sits four lines below: **`policy_versions`** (`0065:177-181`), a per-org
monotonic version bumped on every role write. That is the cache-invalidation key the realtime question
needs, and `policy_version` is already a required cache-key part in the coexistence map.

### 0.12 CONFIRMED: a link type alone produces no edge, ever — every relationship must ride a property

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
produces zero edges, silently, forever.** A canvas that draws relationships as link types would ship an
empty graph that looks configured.

**Resolution, chosen over fixing it:** every relationship in §4.3 is specified as travelling through a
property carrying `config.link = {stable_key, to_type}`. This is not a workaround — it is the only path
the writer implements, and the shipped `employment` fixture already does exactly this
(`fixtures/employment.rs:161`, `:172`). The plan adds one cheap guard instead of a refactor: a
`validate_draft` check that a link type with `to_object_type_id` set has some property referencing its
`stable_key`. One check, and it fails closed on the trap rather than leaving it armed.

### 0.13 A published Tier N type lists EMPTY forever until a policy is attached — and `no-code-ontology.md` is now stale on the fix

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
`OBJECT_TYPE_LIFECYCLE_PATH` (`ontology/rest/src/lib.rs:201`) and `OBJECT_TYPE_POLICIES_PATH` (`:202`)
both exist and are both registered in `ONTOLOGY_ROUTE_PATHS` (`:213-217`) — landed by #521 and #525/0205
after that document was written.

So publish and policy-attach are both HTTP-reachable, and the empty-list trap is closeable over HTTP.
**Consequence for this plan:** every Tier N entity must ship with its object policy attached in the same
change that publishes it, and `tier_n_type_lists_nonempty` is its probe (§7). What remains true from that
document is the deployment hold (§8), not the missing-route claim.

### 0.14 Correction to this plan's own §4.1: `work` must be Tier T, not Tier N

My earlier draft placed `work` in Tier N. Given §0.8, that was wrong: an ontology instance's state is a
fold over revisions, which answers as-of questions well and `AVG(cycle_time)` over 10,000 rows badly —
and there is no read model to bridge it. `work` becomes a Tier T table projected into the ontology
(§4.1), while `assignment` stays Tier N because *that* is what authority folds over. Authoring and
review being separate passes is the repo's rule; this is that rule catching a planner error.

### 0.15 The `Feature` freeze is on MINTING, not composing — so the canvas costs no amendment

`Feature::ALL` is `[Self; 96]` (`authz/src/lib.rs:372`), not the ~40 the specs assume.
`rbac-configurable.md:257-259`, under **"Hard invariants (NON-NEGOTIABLE)"**: *"Only the **assignment** of
the existing `Feature` set is editable. No SQL/console path creates a new `Feature`."*

**A canvas that composes the existing 96 breaks nothing.** Only a canvas that mints capabilities hits that
invariant. This plan's canvas composes — so the freeze needs no amendment, and the earlier "delete the
matrix" framing was arguing for a much more expensive change than the requirement needs. The same document
already calls the six roles *"bootstrap columns for migration parity, not the target operating model"*
(`:122-124`) and bans role-string authorization outright in R1 (`:30-39`), so the direction is settled;
only minting is frozen.

### 0.16 BLOCKING for C5: deleting `Role` deletes the only path to `BranchScope::All`

`resolve_branch_scope_in_org` (`authz/src/lib.rs:1472-1483`):

```rust
if roles.iter().any(|role| matches!(role, Role::SuperAdmin | Role::Executive)) {
    return Ok(BranchScope::All);
}
```

That is the sole tenant-side derivation of `BranchScope::All`, and it keys on `Role`. Every KPI rollup and
cross-branch read depends on it. `ADR-0003:20` names those two roles in its Decision text.

And the obvious replacement is *forbidden*: `rbac-configurable.md:366` — *"Custom role definitions do **not**
widen `BranchScope::All`, group scope, or platform scope. Those scopes remain resolved by the existing
membership/token systems."*

**Resolution (C5, §5.3):** replace the role match with a **built-in `Feature`** check — authored in code,
not mintable from the console, and not a custom-role definition. That satisfies `:366` (not a custom role)
and `:257-259` (no console path mints it). Only `ADR-0003:20`'s Decision text changes, so **one reciprocal
ADR pair (G2b), and C5 is blocked until it is accepted.**

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
   membership in the 결재권 graph, never scope descent.
3. **The four dimensions are vocabulary, not hierarchy.** 소속 / 직급·직책 / 직무 / 결재선 are
   predicates for writing grant rules. None confers authority.
4. **Reuse the classification, not just the code.** Every storage decision names one of the four tiers
   the CI gates already enforce (§3.1). A new tier is a plan defect.
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
   an authored type; entities that don't cost a migration plus a history mechanism.
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
| **O — owner-only** | no `org_id`, no RLS, **no runtime grant at all** | `owner_only_table_allowlist()` `:115`, entries `:118-124` | SECURITY DEFINER only |
| **T — tenant** | `org_id NOT NULL` + `ENABLE`/`FORCE` RLS on `app.current_org` | default classification; anything else is `UnclassifiedTable`, `:804-808` | direct, filtered |
| **N — ontology instance** | rows in `ont_instances`/`ont_instance_revisions`/`ont_links`, themselves Tier T | `0155:16`, `:37`, `:66` | via ontology API |

Two facts about the tiers decide most of §4:

- **Every Tier G rationale is literally "no tenant data"** (`lib.rs:48-70`). PII therefore cannot go
  in Tier G. Tier O is where cross-tenant authorization data already lives — `group_memberships` and
  `group_role_grants`, with rationale *"cross-tenant … resolver only"* (`:118-124`).
- **Tier N cannot hold a cross-tenant edge.** `ont_instances.org_id` is `NOT NULL` (`0155:18`) and
  `ont_links` pins **both** endpoints to the same org via composite FK (`0155:78-79`). This is
  structural, not a missing feature. It is the single constraint that shapes the entity model.

A fifth path, **Tier P — projected**: `ont_object_types.backing_kind = 'projected'` (`0152:25`,
CHECK at `:34-38`) makes an existing Tier T table canvas-visible without moving its data, against a
compiled-in allowlist (`ontology/adapter-postgres/src/instances.rs:1479-1498`). Cost: projected types
have no owned revision store, so no fixity chain and no as-of replay (`instances.rs:1522`).

**The tier rule.** Cross-tenant → O. Within one tenant, needs authoring + replay → N. Needs a
constraint the ontology cannot express (uniqueness, money, FK to legacy) → T, projected into P if the
canvas must see it.

### 3.2 The options

#### Option 1 — Global opaque handle + tenant edge + Tier N authority  ← RECOMMENDED

`party` in Tier O holding no PII; `party_org_visibility` in Tier T under ordinary
`app.current_org` RLS; one re-validating definer; the authority/approval entities of §4.1 as
ontology instance types.

**Pros.** Zero new GUCs, zero changes to the 141 RLS policies, zero new gate classifications. Reuses
three existing tier classifications, the shipped `SECURITY DEFINER` resolver pattern (`0060:99-126`),
the `object_links` edge store (`0102:54`), and the re-validating-read bargain (`store.rs:576-593`).
Canvas-editability and replay arrive free for every Tier N entity — the large majority. PII does not
move. Slice 0 is
unblocked while every Korea control reads HOLD.

**Cons.** Cross-store pointers: `employment.party_id` and `grant.subject` are attribute-bag UUIDs, not
`ont_links`, so referential integrity to `party(id)` is app-enforced and the ontology's search-around
traversal will not cross that hop. Two hops in two stores for any party-rooted query. The definer is a
standing security-review surface that must be re-proven every time it changes.

#### Option 2 — Second tenancy dimension (`app.current_group`)

What the input's Recommended Direction assumes (`:545-546`, `:575-579`).

**Pros.** A group-scoped person row is directly readable by `console_rt`, so ontology links to it work
natively and no definer is needed.

**Invalidated** — by the requirement it exists to serve. The input's own `:89-92` establishes that a
person can work for companies in *different* groups (a contractor, a director on two unrelated boards,
anyone moving between groups), so group-scoping relocates the duplication rather than removing it, and
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

#### Option 4 — `party` in Tier T, deduplicated by a matching service

**Pros.** No new tier usage at all; every row stays under the existing floor.

**Invalidated** — this is the status quo (`users` + `employees` + `person_name`), whose three failed
attempts are the reason this work exists. The executable evidence that matching cannot substitute for
identity is in `0076` itself: the link is a **nullable** column (`0076:13-14`) with a partial unique
index (`0076:22-24`), and its backfill promotes a row only where `HAVING count(*) = 1` holds for the
employee number (`0076:40-46`), leaving every duplicate unlinked. `employees` even carries
`identity_resolution_strategy` and `identity_resolution_confidence` (`0076:49-50`) — a confidence
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
properties and links **are** the component mechanism, and actions with dispatch are the systems. A new
entity class declares its components; the systems light up for it without anyone hand-writing an
integration per concern.

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
| **custody/handover** | an assignee edge | transfer, 인계 완료 query | §4.5 |
| **economics** | a dimension reference | cost/revenue/profit as queries | **largely absent** — §5.5 |
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

**So "manageable without developers" is true for the dimension side and false for the component side.**
Declaring a new *type* is authored; giving it a *new concern* is code. That boundary is the honest answer
to the requirement, and it is also the collision named in §5.11: DN-0003 decides extensibility is
**bounded** — declarative tenant definitions over compile-time-allowlisted first-party tools. This plan
does not pretend otherwise.


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

**Vocabulary is adopted, not invented.** `org-editor-primitives-ux.md` already specifies 14 org primitives
— Group, Org, OrgUnit, Worksite, **Person, Employee, User, Position**, ReportingLine,
EmploymentAssignment, CrossOrgAssignment, SetupDraft, Audit — with the separation this plan needs at `:256`:
*"A Person is not automatically an Employee; an Employee is not automatically a User; a Position is not
automatically an access Role."* **None of them is built** (no `positions`, `org_units`, `persons`,
`reporting_lines` or `worksites` table exists; the spec admits it at `:25`). So this is
specified-and-unbuilt: use those names. `party` is this plan's only rename, and only because later verticals
need customers and suppliers under one identity — noted so the mapping to `Person` stays obvious.

#### Tier O — platform, definer-mediated (1 new table)

| Entity | Purpose | Identity / key | Lifetime | Slice 0 shape |
|---|---|---|---|---|
| **`party`** | one durable identity per natural or legal person, across every tenant and vertical | `id UUID PRIMARY KEY` (plain, per §0.4) |永久; never hard-deleted, terminal soft state only | `(id, party_kind, status, created_at)` — **and nothing else** |

`party_kind ∈ {NATURAL, LEGAL}`. **The row holds no personal data** — no name, no phone, no
주민등록번호. It is an opaque durable handle. That is what makes §5.4 (PII) and erasure tractable, and
it is why the row is safe to exist while every Korea control reads HOLD.

Why Tier O and not the alternatives: Tier T reintroduces the duplication the entity exists to remove;
Tier G would let any tenant enumerate every party on the platform, contradicting the confidentiality
requirement and every Tier G rationale; Tier N is forbidden by `0155:18`.

Named `party`, not `employee`, so the sales, procurement and governance verticals reuse one identity.
Employment is one relationship kind among `CUSTOMER`, `SUPPLIER`, `DIRECTOR`, `CONTRACTOR`.

#### Tier T — tenant, ordinary RLS (2 new tables, 2 new columns)

| Entity | Purpose | Identity / key | Lifetime | Slice 0 |
|---|---|---|---|---|
| **`party_org_visibility`** | **the keystone edge.** The tenant-owned fact that this org holds a relationship to this party | `(id)`; `UNIQUE (org_id, party_id, relationship_kind, valid_from)` | effective-dated interval | yes — one row |
| **`work`** (업무) | first-class work; the join point for artifacts, actions, handover, ledger and metrics | `(id, org_id)` | open → closed | yes — one row |
| **`worksite_registration`** | 사업장 legal attributes: 4대보험 registration unit, optional 사업자등록번호 | `(id, org_id)`; `UNIQUE (org_id, business_registration_no)` | permanent per site | **no** — W3 |
| **`worksite_contract`** (사업장 계약) | the contract a temporary unit's existence is DERIVED from | `(id, org_id)` | a term | **no** — W15 |
| **`lot`** | quantity-bearing node: the splittable/mergeable unit (§5.8) | `(id, org_id)` | until consumed | **no** — W16 |
| `users.party_id` | link the per-org account to the durable identity | nullable FK `→ party(id)` | — | yes |
| `employees.party_id` | link the imported HR row to the durable identity | nullable FK `→ party(id)` | — | yes |
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
| **`grant`** | binds *(subject party, capability, scope, source)* | instance id | effective-dated; revocation closes `valid_to` | yes — 1 |
| **`org_unit`** | 조직 structure, with **kind** (부서/팀/TF/사업장) and **lifetime** | instance id | permanent, or derived from a contract (§5.10) | yes — 1 (kind = 사업장) |
| **`delegation_rule`** (전결규정) | (category × amount band × raising scope) → competent unit, terminal? | instance id | effective-dated | yes — 1 row, 1 band |
| **`approval_template`** | per document class: ordered/parallel steps, each with competent-unit-by-lookup + required capability + mode | instance id | versioned by the registry | yes — 1 step |
| **`approval_line`** | a raised document's line. Stores **line-as-raised AND line-as-executed** | instance id | raised → in_progress → closed → **confirmed** | yes |
| **`employment`** | revise the shipped fixture type | instance id | a period, with terms | no — W2 |
| `authority_rule` | predicate → grant generator over the four dimensions | instance id | effective-dated | no — W5 |
| **`position`** | 직책 **at a scope** — a post that exists unoccupied | instance id | permanent, or bounded with its unit | **slice 1** |
| `assignment` | party ⟶ position, or party ⟶ work, for a period | instance id | a period | **slice 1** (position), W4 (work) |
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

- **`employment` cannot link to `party` with an `ont_link`** (`0155:78-79`). `party_id` is an
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
derived, never stored. It is cross-tenant by construction, so it is **Tier O** and reuses the `party`
definer pattern. Slice 0 does not touch it and neither does any widening before W7; designing its DDL
now would be speculative.

### 4.2 Why there is no second tenancy dimension

**Independently confirmed by a shipped spec.** `org-hierarchy.md:3-7` self-declares P0-P3 **IMPLEMENTED**
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

The confidential fact is not *"who is this party"* — it is *"which parties does org A hold edges to"*.
That fact lives in `party_org_visibility`, which names exactly one `org_id` per row. Ordinary
`app.current_org` RLS therefore gives the whole requirement: org A reads only its own edges and
**cannot observe that org B holds an edge to the same party**. The `party` row itself is Tier O with
no `console_rt` grant, so it is never directly readable.

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
caller to pass its own user id — the function itself never reads `app.current_org`. The party
resolver must **not** copy that: it filters on `current_setting('app.current_org')`, not on a
parameter, or any org can enumerate any party. Stated here because copying the precedent verbatim is
the likely failure.

### 4.3 Relationships

Cardinality uses the engine's own vocabulary, `LinkCardinality::{OneOne, OneMany, ManyMany}`
(`ontology/domain/src/lib.rs:184-188`); `ont_link_types.cardinality` CHECKs the same three at
`0152:77`.

| Relationship | From → To | Card. | Tenant-scoped | Stored as | Owner |
|---|---|---|---|---|---|
| `party_visible_to_org` | party → organization | ManyMany | **yes** (T) | `party_org_visibility` row | tenant |
| `account_of` | users → party | OneOne per org | yes | `users.party_id` FK | tenant |
| `hr_record_of` | employees → party | OneOne per org | yes | `employees.party_id` FK | tenant |
| `controls` | party(LEGAL) → party(LEGAL) | ManyMany, cyclic | no (O) | `party_link` row | platform |
| `parent_org_unit` | org_unit → org_unit | OneMany | yes | `ont_link` | tenant |
| `worksite_legal_reg` | org_unit → worksite_registration | OneOne | yes | FK (T) + projection | tenant |
| `grant_subject` | grant → party | OneMany | yes | property (UUID) | tenant |
| `grant_scope` | grant → org_unit \| organization \| group | OneMany | yes | `ont_link` + `AccessScope.level` (`org-hierarchy.md:172-173`) | tenant |
| `grant_source_assignment` | grant → assignment | OneOne | yes | `ont_link` | tenant |
| **`position_at_scope`** | position → org_unit \| organization \| group | OneMany | yes | `ont_link` | tenant |
| **`holds_position`** | party → position | **ManyMany** | yes | `assignment` instance | tenant |
| `derived_from` | lot → lot, **quantity-bearing** | **ManyMany** | yes | `lot_derivation` row (T) | tenant |
| `declares` | contract_line → lot (the root) | OneMany | yes | FK | tenant |
| `work_scope` | work → org_unit | OneOne | yes | `ont_link` | tenant |
| `work_origin` (발생지) | work → org_unit | OneOne | yes | `ont_link` | tenant |
| `work_performed_at` (수행지) | work → org_unit | OneOne | yes | `ont_link` | tenant |
| `work_jurisdiction` (결재 관할) | work → org_unit | OneOne | yes | `ont_link` | tenant |
| `work_assignee` | work → party | ManyMany | yes | property set + `assignment` | tenant |
| `work_artifact` | work → email_thread \| document \| … | ManyMany | yes | `object_links` row | tenant |
| `competent_for` (전담) | org_unit → (category, band, scope) | ManyMany | yes | `delegation_rule` | tenant |
| `line_step` | approval_line → step | OneMany ×2 sets | yes | `ont_link` (raised / executed) | tenant |
| `step_edge_kind` | step → {결재, 협조, 보고} | — | yes | property on step | tenant |
| `signature_grant` | `gov_approvals` → grant | OneOne | yes | `gov_approvals.authorizing_grant_id` column (T) | tenant |
| `signature_on_behalf_of` | `gov_approvals` → party | OneOne, nullable | yes | `gov_approvals.on_behalf_of_party_id` column (T), **no FK** | tenant |
| `obligation_notice` | approval_line → notices | OneMany | yes | `notices` FK | tenant |

**Every `ont_link` row above is authored as a property carrying `config.link = {stable_key, to_type}`,
never as a bare link type** — §0.12. A link type alone writes no edge. This is a hard specification, not
a style preference.

Five of these deserve their reason stated:

- **`holds_position` is ManyMany, and `position` belongs to a scope — not to a person.** 직책 is not a
  global attribute. A guild officer post and an alliance officer post are different posts held
  concurrently; corporately, 부장 at a subsidiary and 재무관 at the group are two positions with two
  grant sets. `position_at_scope` is what makes 겸직 and 파견 expressible, and ManyMany is what makes it
  *concurrent* rather than sequential. Checked against my own draft: no cardinality here assumed one
  position per person, so no correction was needed — but the scope link was implicit and is now
  explicit.

- **`work_artifact` uses `object_links` (`0102:54`), not `ont_links`.** `object_links` addresses
  endpoints as `src_kind`/`src_id` and `dst_kind`/`dst_id` (`0102:57-60`) with **no FK to either
  endpoint id**, and `link_type` is validated only by slug regex (`0102:63`) — so a new edge kind needs
  no migration. That is exactly a work→artifact edge across heterogeneous stores, already
  `UNIQUE (org_id, src_kind, src_id, dst_kind, dst_id, link_type)` (`0102:68`). Both `person` and
  `org_unit` are already seeded kinds (`0102:32-33`); `work` needs one appended row.
- **Three edge kinds, not one with a flag.** 결재 / 협조 / 보고 differ in direction and meaning; 보고
  is the return leg C and D owe B. A flag on a signing edge cannot carry a reverse-direction
  obligation.
- **합의 is a parallel branch, so a step index cannot express the line.** Steps carry a
  `branch_group` and a `mode ∈ {serial, parallel_합의, terminal_if_전결}`. This is precisely what
  `work_order_approval_steps.step_order SMALLINT CHECK (step_order BETWEEN 1 AND 3)` (`0008:62`)
  forecloses.

### 4.4 Why the existing mechanisms cannot be widened

Structural, not missing features — so the plan builds beside them rather than extending them.

| Mechanism | Executable blocker | Verdict |
|---|---|---|
| `work_order_approval_steps` | `step_order SMALLINT CHECK (step_order BETWEEN 1 AND 3)` `0008:62`; `role CHECK (role IN ('MECHANIC','ADMIN','EXECUTIVE'))` `:63`; `UNIQUE (work_order_id, role)` `:71` | 3 steps max, demo roles, each role once, serial only. 합의 inexpressible. **Leave alone.** |
| `gov_approvals` | `FOREIGN KEY (approver_id, org_id) REFERENCES users(id, org_id)` `0153:78` — the approver **must** be a user of that org, so a group-level approver is forbidden by the FK. `UNIQUE (org_id, request_ref)` `0153:76` is **not** a blocker — see below | **This IS the signature store.** The cross-org FK is the real limit and it stands. Capacity costs two nullable columns and **nothing has to be relaxed.** |
| `policy_role_conditions` | `attribute CHECK (… 'group','organization','department','team','position','assignment','site','branch' …)` `0065:110-128`; `operator CHECK (operator IN ('equals','not_equals','in'))` `0065:129` | The predicate vocabulary the plan needs **already exists as data** — but `not_equals` makes it subtractive-capable, and the fold must be additive. **Reuse the attribute vocabulary; never its operator set.** |
| `notices` / `notice_receipts` | `notice_receipts` is `(id, org_id, notice_id, recipient_user_id, acknowledged_at, created_at)` `0162:41-51` — **no content column**; `notices.status CHECK (status IN ('draft','published'))` `0162:22` — **no closure state**; `FOREIGN KEY (recipient_user_id, org_id) REFERENCES users(id, org_id)` `0162:50` — **recipient must be a user of that org** | 통지 → 인지 is built. Three gaps, one of them structural (see below). **Extend; never build a second ack mechanism.** |

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
reached because no signature is required. The cross-org FK (`0153:78`) remains the one real blocker, and W1
is where it is addressed.

### 4.5 The traversals

Concrete paths for the operations that matter.

**Resolve effective authority** — `effective(party, scope, asof)`:
```
definer effective_grants_for(p_party, p_scope, p_asof)          -- row_security off, re-validating
  → party_org_visibility  WHERE org_id = current_setting('app.current_org')  -- the visibility gate
                            AND party_id = p_party AND asof ∈ [valid_from, valid_to)
  → BRANCH ON p_scope.level                                     -- AccessScope, NOT a DSL (§0.17)
    ├ org_unit | organization | region | branch | worksite:
    │   → grant instances     WHERE org_id = current_setting('app.current_org')   -- org_predicate
    │                           AND subject = p_party
    │                           AND scope ⊇ p_scope
    │                           AND asof ∈ [valid_from, valid_to)
    │   → grant revisions     WHERE org_id = current_setting('app.current_org')   -- org_predicate
    └ group:
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
- **Three entities must be platform tables, each for a stated reason.** `party` — cross-tenant, and
  `ont_instances.org_id` is `NOT NULL` (`0155:18`). `party_org_visibility` — it *is* the RLS-bearing
  row, and it must be an ordinary Tier T table to get the floor for free. `worksite_registration` —
  needs a UNIQUE constraint and an FK an attribute bag cannot express.
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
  - **Decision scope is a new resource attribute**, beside `branch` — not a replacement.
    `branch` stays the operational scope (ADR-0003, and the non-null `branch_id` on every operational
    row). This is how decision scope splits from operational scope without touching the floor.
  - Hierarchy requires populating the parent sets at `engine.rs:392`/`:425` and adding the scope
    entities to `Entities::from_entities` (`:449`), plus declaring them in the bundle schema (§0.3).
    Not free.
  - Caching: `crossRequestAllowDecisionCache: false` in the coexistence map already forbids caching
    a fold across requests. Do not add one.

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
Games have shipped this for 25 years with untrained users, which is the strongest available evidence
that `party`-above-`users` is the natural model rather than an architectural indulgence. Note the game
also confirms the confidentiality design: your guildmates see your character, never your account roster.
That is exactly `party_org_visibility` under RLS (§4.2).

**2. Guild-bank withdrawal limits are 전결규정, and they set the acceptance bar.** "Rank X may withdraw
N per day from tab Y" is (role × amount band × category) → permitted, authored in a grid by
non-technical users. **If the 전결규정 authoring surface is harder to use than a guild bank UI, the
design is wrong.** That is a testable bar, not a sentiment (§4.8).

**3. Enforcement is synchronous, at the transaction.** A guild bank refuses the withdrawal when the
button is pressed. It does not permit it and flag it at month-end. So 전결규정 bands are checked **in
the transaction path**, not in reconciliation — a real departure from the common enterprise pattern of
approve → spend → discover the overspend at close. Applied to the proving slice: the ₩100,000 band is
checked **when the purchase is raised**, and `slice0_band_enforced_synchronously` is its probe (§7).

**Where the lens does NOT transfer, and it matters.** A game rank change is instant and
consequence-free. 강등 under 근로기준법 can constitute 징계 requiring procedural justification (§5.9).
And a game has no PIPA. The lens is prior art for *structure and ergonomics*, never for *consequence*.

**Retracted upstream and not planned:** alliance/guild "tax" as taxation, transfer pricing,
국제조세조정법 or 부당행위계산부인. The corrected reading — complete internal transaction
instrumentation — is the record spine (§4.0) and is in scope. Allocation with a recorded basis is
scoped in §5.5.

### 4.8 Ergonomics as acceptance criteria

Each criterion states what it costs, because two of the five were assumed free and are not.

| # | Criterion | Substrate | Cost |
|---|---|---|---|
| E1 | **Explainability surfaced, not just logged.** A user sees "you may approve this because grant G at scope S", and symmetrically why not | `cedar_decision_log.determining_policies JSONB` (`0159:29`) already stores the deciding policy ids | **low** — the payload exists; surfacing is UI. **But** `0159:28-30` notes it is empty on deny-by-omission, so the case a user most needs explained has no explanation. Closing that is real work |
| E2 | **A character sheet as the unifying screen** — party, positions per scope, the fold per scope, active 업무, pending 결재, delegations in/out | every §4.1 entity has a home on it; `action-inbox` crate exists | medium — and it doubles as the completeness test: an entity with no home on this screen is a modelling smell |
| E3 | **Progressive disclosure** — controls shown are the fold | falls out of `effective(party, scope)` | **free** |
| E4 | **Reversible exploration** — simulate a role or 전결규정 change before committing | Two halves exist: the preview→receipt→consume **ceremony** (`policy_assignment_preview_receipts`, `0065:159-172` — stores inputs, never an outcome, §0.11) and Cedar policy **simulation** (`cedar_pbac/authoring.rs` `simulate_inner`), which `ADR-0023:154-155` says this program ships as *"read-only NL rows + simulation"* | **partly new.** Policy simulation ships; **fold** simulation over a hypothetical grant set does not. Reuse both halves, build only the fold evaluator |
| E5 | **Named entities, not ids** | `ont_object_types.title_property_key` (`0152:23`) and `ont_instances.title` (`0155:21`) already exist | **free** for Tier N; Tier T entities need a display key declared |

**Derived channel membership (E6).** `messenger_thread_members` (`0012:30-36`) is a hand-maintained
roster (§0.10). It becomes a **projection maintained by the assignment action** — not a view, because
the inbox needs the `(user_id, thread_id)` index at `0012:38-39`, and not hand-maintained, because
nobody maintains guild chat rosters by hand. Cost: one write path per membership-changing action, plus
a rebuild-from-graph routine. Generalising `messenger_threads.work_order_id` (`0012:11`) to a `work`
reference is the same change that gives 업무-scoped channels, and the conversation then follows the work
on 인계 for free.

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
2. **`visibility_predicate`** — `party_org_visibility` filtered on `current_setting('app.current_org')`,
   never a parameter (§4.2);
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
would succeed."* Linkage is what SQL can do, and it is what `company_conformance.rs` already asserts. If
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
| C4b | **BLOCKING:** before `Role` dies, `resolve_branch_scope_in_org` (`authz/src/lib.rs:1472-1483`) must derive `BranchScope::All` from a **built-in `Feature`**, not `Role::SuperAdmin \| Role::Executive` | §0.16. Needs reciprocal pair **G2b** on `ADR-0003:20`; existing SUPER_ADMIN/EXECUTIVE principals need a migration path |
| C5 | Four new coexistence-map entries: `authority.grant`, `approval.line`, `party.identity`, `work.assignment` | `enrolledDomainMissingEntry: "deny"` already handles the runtime side |

ADR-0021 decision 1 also means configurable roles **partially ship today** — `policy_roles` +
`policy_role_permissions` + `user_role_assignments` (`0065`) are the "policy bundle generators" it names.
The canvas's role half extends shipped data rather than breaking ground.


### 5.4 D — PII: DECIDED — the durable handle holds no personal data

**`party` is `(id, party_kind, status, created_at)` and nothing else.** All identifying attributes stay
in tenant-scoped rows under the RLS floor that already protects them — `employees` (`0063:21-25`)
keeps holding name, and everything more sensitive continues to live in the tenant tier.

Three consequences, and the third is the one that unblocks slice 0:

1. **No new PIPA surface.** Slice 0 stores no personal data anywhere new. It adds a durable opaque
   handle and a tenant-scoped edge. This matters because the HOLD is total and current: the program
   ledger states that *"every jurisdiction `release_disposition`, and every control trace remains
   `HOLD`"* (`docs/program/console-program-ledger.md:327`) and repeats *"Every capability, evidence
   contract, jurisdiction binding, Korea control, review disposition, legal state, release state, and
   exposure state remains `HOLD`"* through its most recent entry (`:420`). Decisively, the ledger lists
   *"규제 PII/multi-jurisdiction (Jurisdiction/Consent/DSR objects)"* under **"Epics (documented,
   later)"** (`:174`) — the PII substrate is deliberately not built. `ADR-0023:158` independently names
   the **multi-jurisdiction PII program** as out of scope. So a design that needed it would be blocked by
   an accepted ADR, not merely unbudgeted; this one does not need it.
2. **Erasure stays possible.** A fixity chain that referenced a person's attributes could not be
   erased without breaking tamper-evidence. Because the chain references only the opaque handle,
   erasure deletes the tenant-scoped attributes and keeps the handle — history stays verifiable and
   the person stays erased. The handle-only design is an **erasure requirement**, not merely PII
   hygiene.
3. **Cross-tenant inference is closed by RLS, not by policy.** The only cross-tenant fact is "the same
   handle appears in two orgs", and no tenant can read another's `party_org_visibility` rows.

**What must be true before a `party` row itself holds real personal data** (i.e. before any widening
puts an attribute on `party`):

- the deferred **규제 PII/multi-jurisdiction epic** (Jurisdiction/Consent/DSR objects, ledger `:174`)
  has landed, and the jurisdiction binding and Korea controls have moved off HOLD;
- every `party_org_visibility` edge carries a **lawful basis** and a retention clock, not just a
  `reason`;
- 주민등록번호 gets its own design — encrypted at rest, a separate access capability, and its own
  audit stream (the `clearance_assignments` + covert-stream substrate at `0147` is the shape);
- an erasure procedure exists that provably does not break `prev_hash`/`row_hash` continuity.

**Recommendation: never put personal attributes on `party`.** The four preconditions above are the
cost of doing so, and the tenant tier already holds that data correctly. Keeping `party` attribute-free
permanently is the cheaper end state, not a staging posture.

### 5.5 E — Economics: DECIDED — there is no GL; build the spine, seeded by the voucher

**Correction to an earlier premise in this plan: no general ledger exists.** `finance-gl` is two tables.
Verified absent from all 206 migrations: `gl_postings`, `journal_entries`, `gl_accounts`,
`chart_of_accounts`, `fiscal_periods`, `trial_balance`. So "make the object a dimension on existing GL
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
Rust call with four non-test sites (`financial/adapter-postgres/src/lib.rs:1254`,
`workflow/adapter-postgres/src/lib.rs:792`, `orgchange/adapter-postgres/src/lib.rs:611`,
`app/src/hr.rs:1706`). **`finance-gl` is not among them.** Omit the call and the write succeeds.

**Decision: EXTEND the voucher into a posting model; do not build alongside.** Argued, not assumed.

| Option | Verdict |
|---|---|
| **Extend the voucher** ← chosen | It already owns the two hardest parts: a DB-enforced double-entry balance gate and POSTED immutability (`0160:78-118`), plus reversal linkage (`:38-39`) and SoD (`0163:25-27`). Those are the parts that are expensive to get right and easy to get wrong. What is missing — a business date, an account master, a currency column, a line-level dimension — is **additive DDL on a table with no production data claim**. |
| Build a parallel spine | Two records of the same money diverge; that is a certainty, not a risk. And it would need its own balance and immutability enforcement, re-deriving `0160:78-118`. **Rejected.** |

The additive delta, smallest first:

1. **`accounting_date DATE NOT NULL`** on the voucher — unlocks `assert_period_open`, and the guard must
   be called in the finance-gl store (the omission above is a live gap, not a new requirement).
2. **Push the object dimension down to the line** and **type it**: `source_object_type` FK →
   `object_types(kind)`, which already seeds `'voucher'` (`0102:40`). Typing it closes the "any string is
   accepted" hole; pushing it to the line is what makes per-object economics a real query rather than a
   header approximation.
3. Account master and currency — **the peer plan** (below).

**Because the spine is greenfield, "cost/revenue/profit are queries, not stored fields" is now a design
freedom taken deliberately**, not a constraint inherited from an existing GL. Taken: no entity carries a
cost or profit column; both are aggregates over lines dimensioned by `(object_kind, object_id)`.

**In scope for this plan:** items 1 and 2, plus one posted voucher in slice 0. The ₩100,000 purchase has
a cost and an authorization, so the minimal `economics` component is *in*.

**Explicitly a peer plan, argued rather than dropped:** account master / chart of accounts, multi-currency
and FX (the degenerate `currency_code = 'KRW'` CHECKs at `0179:68`, `0182:35`, `0172:10` show the
convention is single-currency throughout), depreciation and accrual posting — today depreciation is only a
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
| Compute the fold on demand, or materialise? | **Materialise per (party, scope), keyed on `policy_versions.version`** (`0065:177-181`) — the per-org counter already bumped on every role write, and already a required cache-key part in the coexistence map |
| Change propagation path | grant revision → bump `policy_versions` → `pg_notify` on a new `authority_changed` channel (a 4th const beside `0012`-era `:37-39`) → WebSocket hub → client re-reads |
| What the push carries | **ids and the new version only.** Never capabilities |
| What is authoritative on disagreement | **the server, always.** Every protected endpoint re-authorizes — the coexistence map's `serverAuthoritative` invariant. A stale client shows a stale button; pressing it is refused |
| Cache scope | per request, never across requests — `crossRequestAllowDecisionCache: false` |

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
model — there are zero materialized views in 206 migrations, so it would be the first, and it would
need its own invalidation, rebuild and staleness semantics.

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

### 5.8 H — Quantity-bearing lineage: DECIDED — one table, one edge, conservation as a row CHECK

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
`production_plans` already carries `output_quantity` / `scrap_quantity` (`0173:81-82`), so scrap is a
modelled concept rather than a new one.

**What is genuinely missing is only the derivation edge.** Linear stock decrement exists; the
split/merge tree does not. So:

| Thing | Shape |
|---|---|
| `lot` (Tier T) | `(id, org_id, uom, quantity_milli, state)`, rooted at a `contract_line` or a production output — `milli` per `0156` |
| `lot_split` (Tier T) | **one row per split**: `(parent_lot_id, parent_qty_before_milli, split_qty_milli, parent_qty_after_milli, child_lot_id, uom, conversion_factor, reason)` |
| UoM conversion | **stored explicitly** on the row. 100 units consumed as 2 pallets is a conversion; an unrecorded one makes the tree unauditable |

**The invariant is a row-level CHECK, not a definer — and that is a change from my first draft.**
`CHECK (parent_qty_before_milli - split_qty_milli = parent_qty_after_milli)` on each `lot_split` row is
exactly the `0156:103` pattern, and it makes conservation unviolatable without any procedural code. A
merge is the same row read in the other direction. Yield loss, scrap and shrinkage are **explicit child
lots**, never silent slack — and `production_plans.scrap_quantity` (`0173:82`) shows the repo already
treats scrap as a first-class quantity.

I had reached for a `SECURITY DEFINER` here on the `0205` precedent. That was over-built: a definer is
needed when an invariant spans sibling rows, and putting before/split/after on one row removes the span.
The shipped `0156:103` CHECK is the cheaper and stronger answer.

**The structural echo, reused rather than reinvented:** a contract line's **declared** quantity and its
**realized** set of splits are both stored and neither is derivable from the other — which is exactly
`line-as-raised` versus `line-as-executed` (§4.1). One shape serves both, and that is worth more than
two correct shapes.

**Traversals:**

```
up   (traceable input for production):
  finished_good lot → lot_derivation(child=·) recursively → roots → contract_line → contract
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

**Adjacent precedents, and what each lacks.** `inventory_consumption_events` (`0156:81`) — quantity,
cost and conservation, but linear and bound to `work_orders` by FK (`:107`) rather than a generic
dimension. `production_demand_contracts` (`0173:6`) and `production_plans` scrap (`0173:81-82`) — the
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

### 5.10 J — Party lifetime derived from a contract: disband vs transfer

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
| assigned 업무 + artifacts | must be reassigned or closed; 인계 완료 query gates it | follow the unit |
| derived chat channel | archived, history retained | persists |
| open obligation loops | follow the 업무, not the unit | unaffected |
| in-flight 결재 lines | see below | unaffected — scope persists |

**This is the strongest argument for assignment as a grant source.** When a party dissolves,
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

| # | Matter | Status | Required artifact |
|---|---|---|---|
| G1 | **Platform-level `party`** | **unauthorized by ADR, but already SPECIFIED.** `ADR-0022:25,33-39` decides identity is local/org-scoped. But `korean-legal-boundaries.md:40-43` decides *"Keep **one** `Person`/identity record for audit and login continuity, but create separate `EmploymentEpisode` records per legal employer"*, and `org-editor-primitives-ux.md:256` *"A Person is not automatically an Employee"* | **new accepted ADR**, arguing **implementation of a specified design**, not a new one |
| G2b | **`BranchScope::All` after `Role` deletion** (§0.16) | ADR-0003:20 names the roles | reciprocal pair on ADR-0003. **Blocks C5** |
| G8 | **DB-enforced invariants vs ADR-0001 domain purity** | tension, see below | argue or amend — on the record either way |
| G9 | **Audit coverage for the new write paths** | unstated | prepwork enumeration, not an ADR |
| G2 | **`org_id` × `BranchScope` composition** | **documented gap.** `ADR-0003:20` decides `BranchScope` with default-deny and has no org concept; `ADR-0021` decision 2 makes `org_id` the RLS boundary Cedar may not widen. **No ADR states how they compose** | **new accepted ADR** — this is where the competence / decision-scope split belongs. Frame it as *closing* the gap, never as fighting ADR-0003 |
| G3 | **전결규정, capacity, obligation loop** | new; zero ADR hits | new ADR, `related: [ADR-0018, ADR-0023]` |
| G4 | **Quantity lineage** (§5.8) | new; zero ADR hits | new ADR |
| G5 | **Economics spine** (§5.5) | new; zero hits for GL/posting/dimension | new ADR (peer plan owns the rest) |
| G6 | **The no-code canvas** | **deferred by an accepted ADR.** `ADR-0023:148-155` lists the Contract→Position(인원편성)→PolicyPreset chain editor, covert clearance as a resource, and the no-code policy/workflow visual canvas as out of scope, *needing their own charter* | either accept the deferral (recommended — it is not on the critical path for slices 0/1) or propose the charter. **Do not smuggle it in** |
| G7 | **DN-0003 bounded extensibility vs open sets** | **NOT a collision** — verified below | none. Aligned as written |

**G8 — DB-enforced invariants vs ADR-0001 domain purity. Decided: argue, not amend.**
`ADR-0001:23` states *"Domain logic stays pure and exhaustively unit-testable"*, gated by a CI
layer-boundary check. This plan puts three invariants in SQL: lot conservation as a row CHECK (§5.8),
the voucher balance gate (`0160:78-118`, already shipped), and the authority fold in a definer (§5.1).

The argument, on the record: **these are integrity constraints, not domain logic.** Each answers "is this
row internally coherent", never "what should the business do" — and a constraint that *cannot* be violated
beats a pure function that callers may forget. `0205` set that precedent deliberately, and the balance gate
predates this plan. The layer-boundary gate still holds because no domain crate gains a SQL dependency: the
constraints live in migrations, and the domain crates keep their pure validators. **If the Critic rejects
that distinction, G8 becomes a reciprocal amendment to ADR-0001, not a silent divergence** — which is what
`README.md:12` requires either way.

**G9 — audit coverage. Prepwork, not an ADR.**
`ADR-0002:20` requires the `with_audit` path and states the CI `audit-coverage` gate's *"exclusion set
contains exactly one entry"* (ADR-0014 establishes it is `location_pings`, with a test asserting it is the
only one). This plan routes granting, signing, closing, confirming and linking through `ont_action_types`
with `dispatch='instance_revision'`. **Nothing states whether that path goes through `with_audit`.**
Enumerating every new write path — with its `with_audit` status and any gate entry required — is a Phase-7
prepwork line item (§8). Cheap now; this is the defect shape that has bitten five times.

**G7 was reported to me as a direct collision. It is not — I verified the invariant.**
`DN-0003` (design-note, `activation: in_progress` at `:6`) invariant 10 reads, at `:98-100`:

> **Extensibility is bounded.** Tenant definitions are declarative; trusted first-party tools are
> compile-time allowlisted; external connectors are server-side, typed, scoped, audited, idempotent, and
> outbox-backed.

The bound falls on **tools and connectors**. The same sentence **affirms that tenant definitions are
declarative** — the open side. So §4.0.2's boundary (declaring a type is authored; giving a type a new
concern is code) is not a reconciliation this plan invented; it is DN-0003's own line, restated on the
axis this plan cares about. **No reciprocal pair is needed and G7 costs nothing.**

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

`policy_role_conditions` already models the four dimensions as data (`attribute` CHECK, `0065:110-128`)
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
| `fold_is_scope_parameterised` | `effective(P, group G) ≠ effective(P, company A)` | a fold that ignores the scope argument |
| `requirement_3` | associate-at-A with a group-G grant out-ranks executive-at-B | any rank-derived ordering |
| `delegation_adds_never_removes` | delegating retains the delegator's grant | an implementation that closes the source grant |
| `asof_replay` | fold at raise time ≠ fold at decision time when a grant closed between | a fold that ignores `p_asof` |
| `feature_catalog_matches_enum` | `feature_catalog` ≡ `Feature::ALL`, both directions | a `Feature` variant with no catalog row; a catalog row with no variant |
| `line_as_raised_immutable` | re-routing revises the executed line, never the raised line | code that mutates the raised line |
| `parallel_hapui_not_ordered` | two 합의 branches have no relative order | a step-index implementation |

### Integration (Postgres, `console_rt`)

| Probe | Asserts | Known-bad control |
|---|---|---|
| `party_not_readable_as_console_rt` | direct `SELECT * FROM party` as `console_rt` is **denied** | a `GRANT SELECT … TO console_rt` on `party` |
| `visibility_edge_rls` | org A cannot see org B's `party_org_visibility` rows for the same party | RLS not FORCEd, or policy omitted |
| `definer_ignores_parameter_org` | passing another org's party id returns **zero rows** | a definer that filters on a parameter instead of `app.current_org` — i.e. `0060:99` copied verbatim |
| `definer_revalidation_each_check` | baseline GREEN with all named checks of §5.1 present; then deleting **`org_predicate`**, **`visibility_predicate`**, **`chain_linkage`** and **`scope_containment`** each in turn fails the suite | each named deletion individually RED. The probe **names each check** rather than carrying a count, so the number cannot rot |
| `definer_returns_no_foreign_org_grant` | one party with a visibility edge in **both** orgs and one grant in each; armed as org A, the call returns exactly **one** row | the definer as §4.5 specified it before this revision — no `org_id` predicate on the grant read |
| `row_security_restored_on_error` | an exception inside the definer leaves `row_security = on` | a definer without the `EXCEPTION WHEN OTHERS` restore of `0060:88-91` |
| `genesis_grant_mintable_only_by_platform_principal` | a grant with no authorising grant can be minted **only** on the platform-principal path gated by `PlatformFeature::TenantCreate` (§5.1) | a **tenant**-authenticated endpoint creating a grant with no authorising grant |
| `no_new_gate_classification` | the tenant-isolation gate passes with `party` in `owner_only_table_allowlist` and `party_org_visibility` unlisted | `party` added to `global_table_allowlist` (must be RED per §3.2 Option 3) |
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
| `obligation_notifies_line_as_raised` | truncated member D is notified though D never saw the matter | notification over the executed line |
| `handover_is_scope_bounded` | relinquishing a group duty leaves the subsidiary post's work in place | handover moving everything the person holds |
| `handover_moves_work_artifacts_only` | person-linked material does not transfer | handover moving all artifacts |
| `link_email_is_authorized` | an unauthorised actor cannot link an email into a work | an open triage queue |

### Added by the addenda — components, game lens, promotion, lineage

| Probe | Asserts | Known-bad control |
|---|---|---|
| `every_entity_declares_its_components` | each §4.1 entity has a row per composed component in `ecosystem-entity-components.tsv` | an entity with no rows — the §4.0.1 completeness test, as a test |
| `capacity_recorded_on_every_authority_mutation` | reads the **D3 write-path enumeration** (§8 Phase 0) and asserts every enumerated authority-mutating path writes `gov_approvals.authorizing_grant_id`. The `audit_events` pair is **out of scope** until those deferred columns land (§4.0.3) | a mutation writing a null capacity where the enumeration says it is required |
| `no_duplicated_fact` | `work` (Tier T) and the revision chain never store the same field | a `work.assignee` column duplicating the assignment edge |
| `tier_n_type_lists_nonempty` | a published Tier N type returns rows | a type published with no object policy attached — `deny_all()` at `residual.rs:200-203` (§0.13) |
| `link_type_alone_is_rejected` | a link type with `to_object_type_id` and no property referencing its `stable_key` fails `validate_draft` | today's behaviour — **must be RED before the guard lands** (§0.12) |
| `slice0_band_enforced_synchronously` | the ₩100,000 band is refused **at raise**, not flagged at close | a check that only reports at period close |
| `economics_is_a_view` | `work` cost equals `SUM` over voucher LINES dimensioned to that work; no cost column on `work` | a stored cost column |
| `posted_voucher_cannot_be_rewritten` | a post-확정 반려 produces a contra voucher; the original stays POSTED | code attempting to UPDATE a POSTED voucher — the `0160:79` trigger must fire |
| `demoted_member_retains_standing` | a demoted member may still 반려 a line already joined | standing re-resolved from current grants |
| `basis_survives_the_chain` | 발령 → grant `grant_reason` → `audit_events.reason` all reference the same 결재 line | a grant expiry with no basis |
| `disband_retains_scope` | after disband, "what could P approve on <date>" still resolves the dissolved scope | hard-deleting the `org_unit` |
| `disband_expires_assignment_grants` | assignment-sourced grants end with no explicit revocation | membership granting a role directly, leaving an orphan |
| `transfer_keeps_crew_and_scope` | rebinding to a new contract preserves unit, members and in-flight lines | transfer implemented as disband + recreate |
| `lot_conservation` | `parent_before − split = parent_after` per row; scrap is an explicit lot | a split leaving unaccounted slack |
| `lot_uom_conversion_recorded` | a cross-UoM split stores its factor | an implicit conversion |
| `lot_traversal_up` | a finished good enumerates every contributing contract line | a broken derivation chain |
| `realtime_push_carries_no_capability` | the NOTIFY payload contains ids and a version only | a payload containing a capability set — must also fail the 8000-byte cap (`realtime:40`) |
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

Bun spent ~3 hours producing `PORTING.md` + `LIFETIMES.tsv` before converting one file. Ours, both of
which **must exist and be reviewed before fanout opens**:

| File | Content | Bun analogue |
|---|---|---|
| `docs/specs/ecosystem-entity-components.tsv` | one row per (entity, component): substrate, tier, status, owning crate. The §4.0.1 and §4.1 tables, machine-readable — so a lane looks up rather than re-derives | `LIFETIMES.tsv` |
| `docs/specs/ecosystem-PORTING.md` | the **mechanical rule set**, no prose: which tier a new entity takes and why; relationships MUST ride a property `config.link` (§0.12); a published type MUST have a policy attached (§0.13); every consequential mutation is an Action carrying `authorizing_grant_id`; `milli` fixed-point for quantities; `object_types` vs `ont_object_types` (§0.7); migrations start at 0207 | `PORTING.md` |

Both are derived from work already done in this plan. Writing them is transcription, not design.

### Phase 1 — the immutable target

Bun's was the existing test suite: **60,624 tests, 0 skipped, 0 deleted.** A target you cannot renegotiate
is what makes a large diff safe. Ours, and it must exist before fanout:

| Target | Rule |
|---|---|
| The 14 CI jobs in `.github/workflows/ci.yml` | pass unchanged. No gate weakened, no allowlist widened without its own justified commit |
| `tools/lanes/fanout.py run` | **0 out-of-slice writes.** Already tooling; do not build another |
| `docs/specs/known-bad-controls.tsv` | **the real immutable artifact.** One row per probe: probe name, known-bad input, and the commit where it was **observed RED**. **No probe may enter the suite without a RED record.** |

That last row is the direct analogue of Bun's "0 skipped". Six probes were defective in one session here;
a GREEN with no recorded RED is not evidence, and the ledger makes that structural rather than cultural.

### Phase 2 — the trial run (before any scale)

Bun converted **3 files** with 1 implementer + 2 adversarial reviewers, and only then did all 1,448.

Ours: **slice 0, the ₩100,000 비품 purchase.** One implementer, two adversarial reviewers who see the
**diff only** — the repo's `slice` skill already implements exactly that shape. Fanout does not open until
slice 0 is green *and* every slice-0 probe has a RED record. If slice 0 refutes the model, the cost is one
slice.

### Phase 3 — the work queue, by crate

Next crate activates only when the current one is clean. Derived from §4.1, in dependency order:

| # | Crate | Ships |
|---|---|---|
| 1 | `platform/db` | migrations 0207+: `party`, `party_org_visibility`, `work`, the **`gov_approvals`** capacity columns (§4.0.3 — the `audit_events` pair is deferred), voucher `accounting_date`, the definer |
| 2 | `platform/authz` | `feature_catalog` ≡ `Feature` gate (C1); grant fold; Cedar subject/resource attrs |
| 3 | `platform/authz-rest` | the re-validating definer read path (§5.1) |
| 4 | `ontology/*` | the Tier N types + their attached policies; `allowlisted_projected_table` arm for `work` |
| 5 | `identity/rest` | C2 — `policy_feature_catalog()` off data |
| 6 | `finance-gl` | line-level typed dimension; **and the missing `assert_period_open` call** (§5.5) |
| 7 | `messenger` + `platform/realtime` | derived membership; the `authority_changed` event |
| 8 | `app` | wiring, `/overview` surface |

### Phase 4 — the progressive verification ladder

Bun's was `cargo check` → `bun --version` → one test file → 100 sharded random files → full suite on 6
platforms. Ours, cheapest first, each rung gating the next:

1. `cargo check` on the touched crate
2. the migration applies, and **re-applies onto a populated DB**
3. the tenant-isolation gate classifies every new table (`tenant-isolation/src/lib.rs:804-808`)
4. the definer probes — `definer_ignores_parameter_org`, `definer_returns_no_foreign_org_grant`, and the
   named re-validation deletions (`org_predicate`, `visibility_predicate`, `chain_linkage`,
   `scope_containment`)
5. slice 0 end-to-end, with its RED records
6. slice 1 (promotion) end-to-end
7. the 14 CI jobs
8. `tools/lanes/fanout.py run` — 0 out-of-slice writes

### Phase 5 — one PR

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

### Phase 6 — experiments (their own phase, before the design is trusted)

Each has a falsifiable prediction and a known-bad control. A probe proven RED on a known-bad input before
its GREEN is the only kind that counts.

| # | Experiment | Prediction | Known-bad control | If refuted |
|---|---|---|---|---|
| X1 | **Edges from an authored type.** Publish a type whose relationship is declared *only* as a link type, then again with a property carrying `config.link` | first writes 0 edges, second writes edges | the link-type-only case must be **RED** (it is today's behaviour, §0.12) | the canvas cannot express relationships; §0.12's guard becomes a blocking fix |
| X2 | **A published type lists rows.** Publish with, and without, an attached object policy | without → `[]`; with → rows | the no-policy case must be RED (`residual.rs:200-203`) | every Tier N entity is unusable; Tier T for all of them |
| X3 | **The definer survives attack.** Build `effective_grants_for`, then run the #525 exploit shape and a foreign-org party id against it | both refused **by execution**, not by argument | a definer filtering on a parameter instead of `app.current_org` must leak — the `0060:99` shape copied verbatim | the bootstrap resolution fails; grants cannot be read outside the gate |
| X4 | **No second RLS dimension is needed.** Answer `effective(party, scope)` for a person in two orgs using only `app.current_org` + the visibility edge | answerable with zero new GUCs | an attempt that requires `app.current_group` | §4.2 collapses and the 141-table cost returns — this is the plan's central claim, so test it first |
| X5 | **Cedar decides alone.** Encode four grant sources over one person in two companies; confirm requirement 3 with no Rust fallback | Cedar alone decides | a case needing a companion evaluator | two evaluators will diverge; the fold moves entirely into Cedar or entirely out |
| X6 | **Fold cost per request.** Measure `effective(party, scope)` at realistic grant counts, materialized vs on-demand | on-demand is acceptable at slice-0 scale; materialization keyed on `policy_versions` if not | a fold whose cost grows with total org grants rather than the person's | §5.6's invalidation design becomes load-bearing earlier |
| X7 | **Draft-PR CI coverage.** Push a backend-touching commit and a docs-only commit to a draft PR; compare required contexts | backend runs all 14; docs-only runs none | trusting the UI's absence of red as green | the one-PR model needs an explicit verification checkpoint per rung |

| X8 | **How do the CI buck2 jobs currently pass?** `prelude/` is **missing** (verified: no `prelude` dir), yet 169 `BUCK` files, `.buckconfig` and five buck steps live in `ci.yml` (`:103`, `:119-120`, `:148-149`, `:151-152`) plus a required job *"Support domain — Buck2 unit reachability"* (`:164`) that passed on #525 | they pass by some mechanism that must be identified before any test is wired to them | wiring a new test and assuming it runs | every new test file is invisible to CI — the exact defect shape that has produced five instances this week, most recently a build failure with zero tests run |
| X9 | **Trace one new test end to end:** test file → BUCK target → wrapper → workflow step | each link nameable | a test that passes locally and never executes in CI | the by-crate queue (Phase 3) needs a per-crate CI wiring step |

`ADR-0023:79-86` already specifies its own **1-2 day Engine-Gen spike** validating the pre-terminal FSM
shape, with *"if structurally infeasible, execution stops and returns to consensus"*. That is X0 and it is
already decided — reference it, do not duplicate it.

### Phase 7 — prepwork before fanout, enumerated

LANE-PROTOCOL §4:72-78 ranks ownership mechanisms: **① NOT SHARED → ② PRE-RESERVED → ③ SERIALISED**, with
*"Prefer the earlier one — later ones rely on discipline, and discipline is what fails."*

| Rung | Prepwork item |
|---|---|
| — | **The reciprocal ADR pairs G1-G9 (§5.11).** Under README rules 2-4 these *gate* the work; G1 (platform `party`) and G2 (`org_id` × `BranchScope`) block slice 0; **G2b blocks C5** |
| — | Phase 0 reference documents; Phase 1 immutable target incl. the empty `known-bad-controls.tsv` |
| **②** | ONE pre-reservation commit: `LANE_TYPES: [&str; 5]` widened (`company_conformance.rs:184`); `allowlisted_projected_table` arms (`instances.rs:1479-1498`); `object_types` kind rows for `work`/`lot`; `RealtimeEvent` variants + channel consts; migration slots 0207+ |
| — | **G9 audit-coverage enumeration (§5.11):** every new write path, its `with_audit` status, and any `audit-coverage` gate entry required. The exclusion set has exactly one entry (`ADR-0002:20`) and a test asserts it |
| — | **CI wiring per crate**, targeting the CI that **exists** (buck2 live) not the one `PIVOT-2026-07-28.md` §6 describes. X8 runs first |
| **①** | Everything else — the new tables, the definer, the capacity columns, each in files no other lane owns |
| **③** | `seed.rs` `BUILTIN_CATALOG_VERSION` — *"the one true bottleneck"* (`LANE-PROTOCOL.md:90`), **inherited, not introduced**. 0204 made installs additive and version-keyed, so lanes can ship disjoint catalog versions; until that fully lands, serialise it |

**Build-system governance is unresolved and this plan must not assume either side.**
`docs/PIVOT-2026-07-28.md` §6 decides *"Build system: cargo, not buck2"* — **unexecuted**: buck2 is live in
CI now (five steps + a required reachability job), while `prelude/` is missing so the buck2 graph is
already broken. Three documents hold three positions (`governed-object-engine-PLAN.md:75` "buck2
RETAINED" vs its own `:301-302` "dropped"; `no-code-ontology.md:133-141` builds on Buck wiring). And
`PIVOT-2026-07-28.md` **is not in `docs/decisions/`**, so under `README.md` rules 1-2 it binds nothing.
**Flagged as an open governance question, not a premise.** X8 establishes the empirical answer before any
test is wired. Also: `rust-toolchain.toml` pins **1.97.1**; `foundation-gates.md:60`'s 1.96.0 is stale
(`Cargo.toml:53` `rust-version = "1.96"` is the MSRV floor, not the toolchain).

**Deployment dependency this plan does not own and must not plan to flip:** every ontology WRITE runs on
the command pool, `command_pool()` is `None` unless `ONTOLOGY_COMMAND_DATABASE_URL` is set, no production
overlay references the component — *"green on every PR and dead where it ships"*
(`docs/ideas/no-code-ontology.md`, evidence at `backend/app/src/lib.rs:2925-2930` and
`ontology/rest/src/lib.rs:1786-1790`). **So slice 0's Tier T half lands and ships; its Tier N half is
CI-provable but not deployable.** A second, independent reason `work` is Tier T.

### Slice 0 — the ₩100,000 비품 purchase, terminal at a 현장

Minimum shape of each entity, and nothing more:

| Entity | Slice-0 minimum |
|---|---|
| `party` | 1 row: `(id, NATURAL, ACTIVE)`. No attributes. |
| `party_org_visibility` | 1 row: the raiser, `EMPLOYMENT`, open interval |
| `users.party_id` | populated for the raiser and the approver |
| `org_unit` | 1 instance, `kind = 사업장`. No legal attributes. |
| `work` | 1 instance, with `work_scope` → that 현장 |
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
| `finance_gl_vouchers` | 1 posted voucher with `accounting_date` and a line dimensioned to the work | the purchase has a cost; the header dimension pair already exists, the date and line-level push are §5.5 items 1-2 |

**Explicitly out of slice 0:** control edges, group designation, `Feature` work, the canvas, 합의,
협조/보고 edges, employment, employment type, PII attributes, retroactive linking, lots, metrics,
allocation.

### Slice 1 — promotion (승진 / 인사발령): the write path

Slice 0 proves read/decide. Slice 1 proves **write**, and it is the smallest operation touching
structure, authority and 결재 at once.

| Entity | Slice-1 minimum |
|---|---|
| `position` | 2 instances, both via `position_at_scope` — one at the 현장, one at the company |
| `assignment` | the old closes at 발령일, the new opens — two revisions, never an update |
| `approval_template` | 1 인사발령 template |
| `approval_line` + a `gov_approvals` signature | with capacity (`authorizing_grant_id`) |
| `grant` | position-sourced, opening and closing on the same 발령일 |

**Acceptance.** `asof_replay` GREEN across the 발령일 boundary (the fold differs either side);
`demoted_member_retains_standing` GREEN; `basis_survives_the_chain` GREEN; assigned `work` and in-flight
lines demonstrably unchanged (§5.9). Demotion is the same slice run in reverse and must produce **grant
expiry, never deletion**.

### Widenings, each an acceptance criterion

| # | Widening | Acceptance |
|---|---|---|
| W1 | Obligation loop: extend `notices` with a content-bearing 조치보고 leg, an originator closure state, and a **party-keyed recipient** replacing the org-composite FK (`0162:50`) | `obligation_notifies_line_as_raised` GREEN **including a recipient in another company**; post-확정 correction GREEN; no second ack mechanism exists |
| W2 | `employment` revised: `party_id` replaces `person_name`; employer split from worksite | a 파견 employment with employer ≠ worksite round-trips |
| W3 | `org_unit` kinds/lifetime + `worksite_registration` (Tier T, projected) | duplicate 사업자등록번호 rejected by the DB; a bounded TF expires |
| W4 | `work` handover + `assignment` as a grant source | scope-bounded handover and 인계 완료 queries GREEN |
| W5 | Remaining grant sources + `position` + `authority_rule` + named `*OrgWide`/`*GroupWide` reach capabilities (§0.17 — no DSL) | **requirement 3 provable**; `fold_is_additive` still GREEN with all five sources |
| W6 | `employment_type` as authored data; both CHECK vocabularies (`0172:7`, `0187:22`) retired | 파견/도급/일용/프리랜서 expressible; neither CHECK remains |
| W7 | `party_link` control edges (Tier O) + derived `group_designation` | a joint venture under two groups, a nested group, and a 순환출자 cycle all resolve; `group_memberships UNIQUE (org_id)` (`0060:36`) and `organizations.group_id` (`0060:27`) collapse to one representation |
| W8 | Cedar scope hierarchy: populate parents at `engine.rs:392`/`:425`, extend `:449`, declare in schema | a group-scoped approver signs a company-raised document, decided by Cedar alone with no Rust fallback |
| W9 | `Feature` sequencing C1→C6 (§5.3) | every coexistence-map entry `cedar_only`; `matrix_row` and `Role` deleted; `Feature` retained |
| W10 | Canvas over the authored types, four-eyes on every authority change | no authority change lands without a `gov_approval_consumptions` row |
| W11 | Derived channel membership; `messenger_threads.work_order_id` (`0012:11`) generalised to `work` | `channel_membership_is_derived` GREEN; the conversation follows the work on 인계 |
| W12 | Realtime authority propagation (§5.6): one `RealtimeEvent` variant, one channel, `policy_versions` invalidation | `realtime_push_carries_no_capability` and `stale_client_button_is_refused` GREEN |
| W13 | `work` metrics: the new fields (§4.1) + cycle-time aggregates over Tier T | `no_duplicated_fact` GREEN; an aggregate over 10k rows does not fold revisions |
| W14 | The pre-terminal finalization path (ADR-0023) end to end, incl. the compensating document and its contra voucher | `posted_voucher_cannot_be_rewritten` GREEN via the `0160:78-118` trigger; **and** `assert_period_open` called from finance-gl |
| W15 | `worksite_contract` + disband/transfer (§5.10) | all four disband/transfer probes GREEN |
| W16 | `lot` + `lot_split` + contract lines (§5.8); `inventory_consumption_events.source_kind` (`0156:87`) generalised | `lot_conservation`, `lot_uom_conversion_recorded`, `lot_traversal_up` GREEN |
| W17 | E4 fold simulator — the fold against a hypothetical grant set, over the shipped receipt ceremony and Cedar simulation | a role change is inspectable before commit; neither existing half is replaced |
| W18 | E1 explainability surfaced, incl. a reason for deny-by-omission | `deny_by_omission_is_explained` GREEN |

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
immutability that are the expensive parts. Cost, revenue and profit are queries, never stored fields — now
a design freedom taken deliberately rather than a constraint inherited.

**Standing of this document.** It is a plan, so under `docs/decisions/README.md` rule 4 it decides
nothing. Its output is the reciprocal ADR pairs **G1-G9 (§5.11)**, of which **G1 and G2 block slice 0** and **G2b
blocks `Role` deletion**. G1 argues the *implementation of a design the specs already carry*
(`korean-legal-boundaries.md:40-43`), not a new one. Where a matter is already accepted — finality
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
ambiguous row (`0076:40-46`) and carries a confidence model (`0076:49-50`).

**Why chosen.** It is the only option that meets the confidentiality requirement without a second
tenancy dimension, and it reuses four shipped mechanisms rather than adding any: the tier
classifications the CI gate already enforces, the `SECURITY DEFINER` resolver pattern (`0060:99-126`),
the untyped `object_links` edge store (`0102:54`), and the re-validating-read bargain
(`store.rs:576-593`, `0205:69-74`). Sixteen new entities cost one owner-only table, two tenant tables,
two nullable FK columns and one definer.

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
