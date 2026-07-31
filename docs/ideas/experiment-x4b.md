# Experiment X4b — a group-scoped grant cannot live in Tier N

> Runs the half of **X4** that `docs/ideas/ecosystem-plan-review.md` finding **B9** (`:271-301`)
> identified as untested. X4 confirmed tenant *visibility* can be edge-mediated with zero new GUCs;
> B9's charge is that X4 tested only the intra-org half and that the **grant-scope** half is
> structurally impossible on the substrate `docs/ideas/ecosystem-plan-DRAFT.md` §4.1/§4.3 specifies.
> Executed in `console-lanes/lane-4`, 2026-07-29, on `postgres:18.4`.
>
> **Verdict: CONFIRMED.** Both halves of B9 reproduce by execution. A group-scoped grant stored as a
> Tier N `ont_instances` row is unreadable from every sibling org that needs it, and the
> `organization → group` edge §4.3:666 specifies is inexpressible — the FK rejects it.
>
> Probe: `docs/ideas/experiments/x4b/probe.sql` + `run.sh`. **27 assertions PASS, 0 FAIL**, two
> controls observed as required. Re-runnable: `bash docs/ideas/experiments/x4b/run.sh`.
>
> **Slice 0 is not blocked.** Both of Slice 0's grants are at 현장 scope (`:1702`) — intra-org, the
> case CASE 1 shows working. B9 bites the `group` arm, which Slice 0 does not use.

## 1. The claim under test

The plan puts `grant` in **Tier N** — an `ont_instances` row with `ont_instance_revisions` history and
`ont_links` edges (`:563`) — and specifies its scope as an edge:

> `grant_scope` | grant → org_unit \| organization \| group | OneMany | yes | **`ont_link`** +
> `AccessScope.level` — §4.3:666

A group-scoped grant is what lets an HR officer whose 소속 is subsidiary A hold authority over the
whole group: the owner's requirement 3, and the case the party design exists to serve.

B9 asserts this cannot work, on two citations. **Both were verified before anything was built, and
then re-verified against the running database rather than the source file** — because a `.sql` file
can be superseded by a later migration, which is exactly what happened to `object_links` (§6).

### 1.1 B9's citations, verified

| B9's claim | Citation | Verified | How |
|---|---|---|---|
| `ont_links` FKs **both** endpoints to `ont_instances(id, org_id)` | `0155_create_ontology_instances.sql:76-77` | **correct** | source read + `pg_get_constraintdef` on the live table |
| `groups` is a different tier — `global_table_allowlist`, *"group identity metadata only, no tenant data"* | `backend/ci/gates/tenant-isolation/src/lib.rs:48` | **correct** | source read; `groups` has **no `org_id` column** in the live schema |
| the shipped cross-org authority answer is Tier O + definer | `tenant-isolation/src/lib.rs:121-124` (`group_role_grants`, *"cross-tenant group role authorization; own-grants resolver only"*) | **correct** | source read + executed in §5 |
| `ont_instances.org_id` is `NOT NULL` | `0155:18` | **correct** | source read |

Live-schema confirmation, printed by the probe:

```
  ont_links FKs to ont_instances:
    FOREIGN KEY (from_instance_id, org_id) REFERENCES ont_instances(id, org_id) ON DELETE CASCADE ;
    FOREIGN KEY (to_instance_id, org_id) REFERENCES ont_instances(id, org_id) ON DELETE CASCADE
PASS  B9(a) BOTH ont_links endpoints FK to ont_instances(id, org_id) 2
  ont_instances (relrowsecurity|relforcerowsecurity): true|true
PASS  ont_instances has RLS ENABLEd and FORCEd                   true|true
  groups.org_id columns: 0  (0 = groups is not tenant-scoped, i.e. a different tier)
PASS  B9(a) groups is NOT an org-scoped table                    0
  ont_instances rows whose id is a groups id: 0
PASS  B9(a) no groups row has an ont_instances row               0
```

One drift, already logged by the review itself (`ecosystem-plan-review.md:847-848`): the **plan**
cites these FKs as `0155:78-79` at `:321`. They are at `:76-77`. B9's numbers are the right ones.
Substance is unaffected either way.

## 2. What was built

**The real shipped schema, not a replica.** X4 replicated `clearance_assignments`' policy shape
because `party` does not exist yet. X4b could not do that: B9's claim *is* a claim about the real FK,
so a hand-copied `ont_links` could be wrong about the one thing under test. `run.sh` therefore
applies **all 205 migrations** under `backend/crates/platform/db/migrations` in filename order, and
`ont_instances` / `ont_instance_revisions` / `ont_links` / `groups` / `group_memberships` /
`object_links` are the production definitions.

Two obstacles, both recorded because the next probe will hit them:

- **`0196_platform_force_command_and_fk_closure.sql:34-42`** refuses to apply unless it is either
  `console_app` on a `console_app`-owned database, or the Buck SQLx superuser bootstrap:
  `CURRENT_USER = console_buck_admin`, startup marker `console.sqlx_test_bootstrap =
  buck-sqlx-superuser-v1`, a database matching `^_sqlx_test_[A-Za-z0-9_]{52}$`, and `datdba` = the
  applier. The probe takes the second path — the same one `tools/lanes/pgtest.sh` builds into its
  `DATABASE_URL`.
- **`0165:429-445`** puts deferred constraint triggers on the ontology **type registry** demanding one
  matching protected audit row per mutation, and `0165:354-357` lets only `console_rt` (with a proven
  parent mutation) or `console_ontology_cmd` write that row — so no seed script can satisfy it. Two
  named registry triggers are disabled for the seed and re-enabled before any assertion runs.

That second one is a deliberate simplification, scoped so it **cannot rescue the claim under test**:
it touches only `ont_object_types` / `ont_link_types`, nothing on `ont_instances` /
`ont_instance_revisions` / `ont_links`; and `DISABLE TRIGGER` disables neither RLS nor foreign keys.
The probe proves both by execution rather than by assertion — CONTROL 1 shows `ont_instances` RLS
still hiding a row, CASE 3a shows the `ont_links` FK still rejecting a bad endpoint.

**The scenario.** Group `G` with two member 법인 `A` and `B` (`groups` + `group_memberships`, both
rows present and asserted). One HR officer, `org_id = A`, holding `GROUP_ADMIN` in
`group_role_grants`. Object types `grant`, `organization_scope`, `group_scope`; a `grant_scope`
link type; a `group_scope` instance minted in **each** org — the most generous reading of §4.3:666
available, since an `ont_link` target must be an `ont_instances` row somewhere. Two grants, both
`purchase.approve`: one company-scoped in A, one group-scoped over G.

Everything below ran as **`console_rt`**, proved in-band before any assertion:

```
SELECT session_user, current_setting('is_superuser'), rolbypassrls FROM pg_roles WHERE rolname = session_user;
  -> console_rt|off|f
```

## 3. The known-bad controls — run FIRST

The brief specifies one control: mint in org A, arm as org B, confirm it is invisible. Run alone that
control is **unfalsifiable** — an empty result set is equally consistent with a typo, a wrong id, or
a query that can never return anything. Six probes were defective in one session here for related
reasons. So it is run as a **pair**, and `run.sh` exits before any case if either half misbehaves.

### CONTROL 1 — org isolation is live on the real `ont_instances` (the RED observation)

```sql
SET app.current_org = 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
SELECT id::text || ' ' || title FROM ont_instances WHERE id = 'e1000000-0000-4000-8000-0000000000ff';
```

```
  OUTPUT:
      <empty>
PASS  CONTROL 1 org B cannot see org A's ont_instances row       0

    ...and the same row IS there. Ground truth, RLS bypassed (superuser):
      aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa
PASS  CONTROL 1 the hidden row really exists, owned by org A     aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa
```

**Armed as B, org A's row is gone — and ground truth confirms the row exists, owned by A.** Had
`console_rt` seen it, the harness would not be exercising RLS and every result below would be void.

### CONTROL 2 — proof CONTROL 1's emptiness is caused by RLS, not by a broken query

A `LIKE ont_instances INCLUDING ALL` copy holding the same rows, RLS never enabled. Same query shape,
same armed org.

```sql
SET app.current_org = 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
SELECT id::text || ' ' || title FROM x4b_instances_control_norls WHERE id = 'e1000000-0000-4000-8000-0000000000ff';
```

```
  OUTPUT:
      e1000000-0000-4000-8000-0000000000ff x4b control row minted in org A
PASS  CONTROL 2 leaks org A's row to org B (RED expected)        1
```

**The query CAN return the row. RLS is what stops it.** Together the pair pins the cause: CASE 2's
and CASE 3d's empty results below are org isolation, not instrument failure.

## 4. The three cases

### CASE 1 — baseline: company-scoped grant, minted in A, read from A. Resolves.

```sql
SET app.current_org = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
SELECT g.title, r.attributes->>'capability', r.attributes->>'subject_party_id', s.title
FROM ont_instances g
  JOIN ont_instance_revisions r ON r.instance_id = g.id AND r.valid_to IS NULL
  JOIN ont_links l ON l.from_instance_id = g.id AND l.valid_to IS NULL
  JOIN ont_instances s ON s.id = l.to_instance_id
WHERE g.title = 'grant: company-scoped, org A';
```

```
grant: company-scoped, org A|purchase.approve|11111111-1111-4111-8111-111111111111|organization scope: A
PASS  CASE 1 the company-scoped grant resolves with its scope    1
```

The full authority input a 결재 raise needs — subject, capability, scope — folded out of Tier N
through an `ont_link`, exactly as §4.3 specifies. **The intra-org arm of §4.3:666 works.** This is
also what Slice 0 needs (`:1702`), and it is unaffected by everything below.

### CASE 2 — the falsifying case: group-scoped grant, read from a sibling org. Zero rows.

The grant is minted in A because `ont_instances.org_id` is `NOT NULL` (`0155:18`) — there is no third
option. Its scope is `{scope_level: 'group', scope_node_id: G}`, the descriptor form B9's *Required*
section proposes (`AccessScope{level, node_id}`, `org-hierarchy.md:172`, shipped as
`kernel/core/src/access_scope.rs:28-40`). Group G verifiably contains both orgs
(`group_memberships` → 2 rows, asserted).

**2b — org A, the minting org, reads it fine:**

```
grant: group-scoped over G|purchase.approve|group|99999999-9999-4999-8999-999999999999
PASS  CASE 2b org A (the minting org) can read it                1
```

**2c — org B, a sibling in the SAME group, raising a 결재 whose competent unit is at group scope:**

```sql
SET app.current_org = 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
SELECT g.title, r.attributes->>'capability', r.attributes->>'scope_node_id'
FROM ont_instances g
  JOIN ont_instance_revisions r ON r.instance_id = g.id AND r.valid_to IS NULL
WHERE r.attributes->>'scope_level' = 'group'
  AND r.attributes->>'scope_node_id' = '99999999-9999-4999-8999-999999999999';
```

```
  OUTPUT:
      <empty>
PASS  CASE 2c org B gets ZERO rows for the group-scoped grant    0
```

**2d — and the grant is genuinely in the table.** Ground truth, RLS bypassed: `1` group-scoped grant
revision exists. Org B cannot reach it. This is the assertion that stops 2c being vacuous.

**2e / 2f — the two obvious workarounds, both measured:**

| Attempt | Result |
|---|---|
| org B names org A explicitly: `WHERE org_id = A` | `0` — the `USING` predicate is `org_id = app.current_org`, so a wider `WHERE` cannot widen it |
| org B arms `app.current_org = G` (the **group** id) | `0` instances. `org-hierarchy.md:181` already forbids this as security-critical invariant **C1** (*"`app.current_org` is ALWAYS a real Org id, NEVER a Group id"*); measured here, it also simply does not work — no org's rows match a group id |

**§4.5's requirement — *"per step: eligible approvers = effective(·, step.competent_unit.scope) ∩
{step.required_capability}"* (`:746`) — is not answerable from org B when the scope is a group and the
holder is a user of org A.**

B9's *Required* section words the falsifying case as the mirror image: armed to org A, resolve a
group-scope step whose only qualifying holder is a user of org B. The direction run here is the same
structural fact and is the one the owner's requirement 3 actually needs — the HR officer's 소속 is A,
and the document is raised in B. Both directions fail for one reason: `ont_instances` rows are visible
only to the org that owns them, and there is no direction in which that helps.

### CASE 3 — the edge case: the FK does not permit an `organization → group` edge

All three attempts as `console_rt`, armed as org A, against the real `ont_links`.

| # | Edge attempted | Outcome |
|---|---|---|
| 3a | `to_instance_id` = the real `groups.id` (Tier G row) | `ERROR: insert or update on table "ont_links" violates foreign key constraint "ont_links_to_instance_id_org_id_fkey" / DETAIL: Key is not present in table "ont_instances".` |
| 3b | `to_instance_id` = a `group_scope` **instance minted in org B** — the sibling that needs to see it | same FK violation; the composite key requires `(to_instance_id, org_id)` to match, and B's instance has B's `org_id` |
| 3c | `to_instance_id` = a `group_scope` instance minted in **org A** | `edge-accepted` |
| 3d | org B then reads that accepted edge | `0` rows |

**3a is the direct refutation of §4.3:666.** The `group` arm as written — an `ont_link` to a group —
is not storable. 3c shows the only shape the FK allows: mint a *per-org shadow instance* of the group
inside each tenant. 3d shows why that does not help. The shadow buys expressibility and buys nothing
else: org B's shadow of G is a different row with a different id in a different tenant, so a grant
edge in A tells B nothing, and the group ceases to be one object.

## 5. What substrate WOULD carry it, and what each costs

### S1 — `object_links` (`0102:53-69`): storable, still unreadable

`object_links` addresses endpoints as `src_kind`/`src_id` and `dst_kind`/`dst_id` — opaque `TEXT`,
**no FK to either endpoint**, so a group id goes in without complaint:

```
object_links-insert-accepted
PASS  S1b object_links accepts a group id as an opaque dst_id    1
  read back armed as org B -> 0
PASS  S1b but org B still cannot read it (same org floor)        0
```

**It solves the FK problem and none of the tenancy problem.** `object_links.org_id` is `NOT NULL`
with `ENABLE`/`FORCE` RLS on `app.current_org` (`0102:76-79`) — the identical floor. Moving the edge
here converts an FK error into a silent empty result, which is strictly worse.

**And its cost line is stale in the plan and the review alike.** `0102:61-62` comments `link_type`
*"Free-form-but-validated so new link types need no migration."* That is no longer true:
`0130:24-31` created a `link_types` table with `link_type` as its **PRIMARY KEY**, `0130:75` added
`object_links_link_type_fkey`, and `0132:8` validated it. Measured:

```
INSERT ... link_type = 'grant_scope'
  -> ERROR: insert or update on table "object_links" violates foreign key constraint
            "object_links_link_type_fkey" / DETAIL: Key is not present in table "link_types".
  seeded link_types vocabulary size (0130:37-49): 12
INSERT INTO link_types ... (as console_rt)
  -> ERROR: permission denied for table link_types
```

`grant_scope` is not in the 12-value vocabulary, and `console_rt` holds `SELECT` only (`0130:52`).
**Routing grant scope through `object_links` costs a migration to extend the vocabulary** — the exact
cost the header comment says it avoids.

### S2 — Tier O + `SECURITY DEFINER`: the shipped answer, and it works

```
q: SELECT count(*) FROM group_role_grants;                        (armed as B)
   -> ERROR: permission denied for table group_role_grants
q: SELECT ... FROM group_role_grants_for_user(<org A user>);      (armed as B)
   -> 99999999-9999-4999-8999-999999999999 GROUP_ADMIN
```

**Armed as org B, the definer returns an org A user's group authority.** This is the only mechanism
in this experiment that answers the question the plan needs answered — and it works by being **Tier
O behind a definer, not by being a Tier N `ont_instance`**. It is the shape
`tenant-isolation/src/lib.rs:121-124` already classifies and rationalises.

**Its cost, measured, not assumed.** `group_role_grants_for_user` filters on `p_user` alone
(`0060:113-114`) and references **no** `app.*` GUC at all:

```
  GUC names referenced by group_role_grants_for_user: <none>
PASS  S2-cost the shipped definer has NO org predicate           <none>
```

The same review found the plan's *other* definer has no org predicate either. Here it is not a defect
per se — cross-group authority is deliberately not org-scoped, and a group-scope read that filtered
on `app.current_org` could not do its job. But it means **the caller, not the database, is the org
floor for every group-scoped authority read**, which is the same class of failure X4 measured as its
CONTROL 2 (`x4probe_resolve_leaky`, armed as A, leaking B's rows for a parameter it trusted). Any new
Tier O grant store inherits that burden: its authorisation must be inside the definer, keyed on the
authenticated principal, never on a caller-supplied org or group id.

### The three options, priced

| Substrate | Group arm expressible? | Sibling-org readable? | Cost |
|---|---|---|---|
| **Tier N** `ont_links` (as §4.3:666 specifies) | **No** — 3a FK violation | **No** — 2c/3d, zero rows | none; it does not work |
| **Tier N + per-org shadow group instances** (3c) | Yes | **No** — 3d | N instances per group, a group that is no longer one object, and still no answer |
| **`object_links`** | Yes (opaque `dst_id`) | **No** — same `org_id` floor | +1 migration for the `link_types` vocabulary; converts a loud FK error into a silent empty read |
| **Tier O + definer** (shipped shape) | Yes | **Yes** — executed in S2 | 1 owner-only table, 1 gate classification, 1 audited definer; the org floor moves from RLS into the definer's own predicate, so it must be authorisation-complete by construction |

No option needed a second GUC. The GUC inventory over every policy this experiment read returns
`app.current_org` and nothing else — **X4's headline finding survives X4b intact**. What fails is not
the tenancy dimension; it is the *storage tier* the plan chose for one arm of one relationship.

## 6. What this means for the plan

1. **§4.3:666 and §4.3:668 are wrong for two of their three arms as written.** `ont_link` cannot
   express `grant → group` (3a) and `position → group` fails identically — same table, same FK. The
   `organization` arm is in the same position: an `organization` is an `organizations` row, not an
   `ont_instances` row, so it needs the same treatment. B9's fix is the right shape: replace the edge
   with a **scope descriptor property** `{level, node_id}`, which the probe used throughout CASE 2 and
   which reads back correctly within its own org. `AccessScopeLevel` already ships with a `Group`
   variant (`access_scope.rs:28-34`), so the vocabulary exists.
2. **§4.1 must split `grant` by scope level.** org_unit- and organization-scoped grants stay Tier N —
   CASE 1 proves that arm works end to end. **Group-scoped grants cannot be Tier N at all** and
   belong in Tier O beside `group_role_grants`, reached only through a definer.
3. **§9's cost line at `:1822` — *"Sixteen new entities cost one owner-only table"* — is short by
   one.** That single owner-only table is `party`. A Tier O group-grant store is a second, plus its
   gate classification and its definer.
4. **The plan already knows this and contradicts itself.** §3.1:320-321 states *"Tier N cannot hold a
   cross-tenant edge… This is structural, not a missing feature. It is the single constraint that
   shapes the entity model."* §4.3:666 then puts a cross-tenant scope arm in an `ont_link` anyway. The
   fix is a consistency edit against a constraint the plan has already accepted, not a new discovery.
5. **§8's framing of X4 needs correcting.** The X4 row at `:1647` calls it *"the plan's central claim,
   so test it first"* and §4.2 asserts sufficiency at `:625` (*"This is the plan's central claim"*) —
   but X4 as specified could not reach this case. §4.2's sufficiency claim holds **for visibility of a
   known party within the armed org** and does not extend to cross-org authority resolution. Say so
   where the claim is made, at `:625`.

**For Slice 0: no impact.** Slice 0's two grants are both at 현장 scope (`:1702`) — intra-org, the
CASE 1 shape, measured working. Slice 0 can ship on Tier N grants unchanged. What must not happen is
Slice 0 shipping a `grant_scope` **link type** whose declared target set includes `group`, because
that arm will fail at the first write and the schema will already be published.

## 7. Honest limits

- The subject of both grants is a UUID property standing in for `party_id`; `party` does not exist
  yet. This is what §4.3:665 specifies (`grant_subject` → property, not link) and it does not touch
  the scope question.
- `ont_instance_revisions.prev_hash` / `row_hash` are placeholder constants. The fixity chain is not
  under test here; B11 covers effective-dating defects separately.
- Two named audit triggers on the **type registry** were disabled for seeding (§2). RLS and every FK
  stayed armed, and CONTROL 1 and CASE 3a prove it by execution.
- Only `console_rt` was exercised. A group-scope read might also be attempted by
  `console_ontology_cmd` or another command role; whether any of those could reach across orgs was
  not measured. The Tier O path (S2) is the one the codebase already uses, and it was measured.
- No attempt was made to make CASE 2c succeed by adding a definer over `ont_instances` that switches
  `row_security` off. That would not be a fix — it would delete the tenancy guarantee for the entire
  ontology instance store to serve one relationship arm, and it is reported here as a finding about
  the design rather than applied as a repair.
