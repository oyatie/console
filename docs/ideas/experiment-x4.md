> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Experiment X4 — no second tenancy dimension

> Runs `docs/ideas/ecosystem-plan-DRAFT.md` §8 Phase 6 experiment **X4** (`:1647`), the plan's own
> priority-one probe because it tests §4.2, the central claim. Executed in `console-lanes/lane-3`,
> 2026-07-29, on `postgres:18.4`.
>
> **Verdict: CONFIRMED** on the claim as X4 states it — with one wording defect in the plan that the
> team lead must resolve, because under one of §4.2's two readings a `SECURITY DEFINER` is
> unavoidable. §141-table cost does **not** return under either reading.
>
> Probe: `docs/ideas/experiments/x4/probe.sql` + `run.sh`. **30 assertions PASS, 0 FAIL, 3 controls
> observed RED.** Re-runnable: `bash docs/ideas/experiments/x4/run.sh`.

## 1. The claim under test

§4.2 (`:625-634`) asserts that tenant visibility is mediated by an **edge**, not by scoping the party
row, so a platform-level `party` needs no second tenancy dimension:

> The confidential fact is not *"who is this party"* — it is *"which parties does org A hold edges
> to"*. That fact lives in `party_org_visibility`, which names exactly one `org_id` per row. Ordinary
> `app.current_org` RLS therefore gives the whole requirement […] Consequences: **zero new GUCs. Zero
> changes to the 141 RLS policies. Zero new gate classifications.**

Load-bearing context, verified rather than assumed:

| Claim | Evidence |
|---|---|
| `app.current_org` is the only tenancy GUC | policy text at `0147:72-73` |
| **141** tables enable RLS | `grep -c 'ENABLE ROW LEVEL SECURITY' backend/crates/platform/db/migrations/*.sql` → **141** |
| 157 `org_isolation` policy references | same directory, `grep -c org_isolation` → **157** |
| `console_rt` is a genuine non-superuser | `ops/postgres-reconcile-topology.sh:303` — `LOGIN NOSUPERUSER NOBYPASSRLS NOINHERIT` |
| denied data must omit counts | `DN-0003:84-86` verbatim: *"Denied data is omitted, including counts and relationship existence."* |

The confidentiality requirement: **company A must not learn that its employee also works at company
B.**

## 2. What was built

A scratch probe, deliberately **not** under `backend/crates/platform/db/migrations/` — it must never
take a migration slot. Every object is prefixed `x4probe_`.

- **`x4probe_party`** — Tier O. `(id, party_kind, status, created_at)` and nothing else, checked
  against §4.1:510 before writing; no PII per §4.1:512; **no `org_id`, no RLS org filter**.
- **`x4probe_party_org_visibility`** — the edge. Columns and `UNIQUE (org_id, party_id,
  relationship_kind, valid_from)` from §4.1:527 and §4.1:547-550. RLS armed by copying
  `clearance_assignments` at **`0147:68-75`** verbatim — same `ENABLE`, same `FORCE`, same `USING`,
  same `WITH CHECK`. The real pattern, not an invented one.
- Seed: one `party` with edges to **org A and org B** (the human at two companies), plus a second
  party visible only to B.

Everything below ran as **`console_rt`**, proved in-band before any assertion:

```
SELECT session_user, current_setting('is_superuser'), rolbypassrls FROM pg_roles WHERE rolname = session_user;
  -> console_rt|off|f
```

## 3. The known-bad controls — run FIRST, observed RED

A probe with no demonstrated failure mode is not evidence. The runner **exits before any test** if
these three do not leak.

### CONTROL 1 — identical edge table, RLS never enabled

```sql
SET app.current_org = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
SELECT org_id || ' ' || party_id || ' ' || relationship_kind
FROM x4probe_edge_control_norls WHERE org_id = 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb' ORDER BY valid_from;
```

```
bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb 11111111-1111-4111-8111-111111111111 EMPLOYMENT
bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb 22222222-2222-4222-8222-222222222222 EMPLOYMENT
PASS  CONTROL 1 leaks org B's edges to org A (RED expected)      2
```

**Armed as A, org B's edges came back.** This is the observation that makes every GREEN below
meaningful: the harness demonstrably exercises RLS, and `console_rt` genuinely reaches these tables.

### CONTROL 2 — the `0060:99` shape: a definer that trusts a parameter

`group_role_grants_for_user(p_user UUID)` at **`0060:99-126`** filters on its parameter and never
reads `app.current_org`. §4.2:644-649 names copying it verbatim as *"the likely failure"*. Measured:

```sql
SET app.current_org = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
SELECT party_id || ' ' || relationship_kind FROM x4probe_resolve_leaky('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb');
```

```
11111111-1111-4111-8111-111111111111 EMPLOYMENT
22222222-2222-4222-8222-222222222222 EMPLOYMENT
PASS  CONTROL 2 leaks via parameter-trusting definer (RED expected) 2
```

Armed as A, passing B's org id returned **both** of B's edges. §4.2's warning is real and now
measured, not predicted.

### CONTROL 3 — correct RLS, but the UNIQUE key omits `org_id`

A unique index is enforced physically, **below** RLS. Same table, same policy, key changed to
`UNIQUE (party_id, relationship_kind, valid_from)`:

```sql
SET app.current_org = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
INSERT INTO x4probe_edge_control_uniqleak (org_id, party_id, relationship_kind, valid_from, reason)
VALUES ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', '11111111-1111-4111-8111-111111111111', 'EMPLOYMENT', '2026-02-01T00:00:00Z', 'probing for B row');
```

```
ERROR:  duplicate key value violates unique constraint "x4probe_edge_control_uniqleak_party_id_relationship_kind_va_key"
PASS  CONTROL 3 unique index leaks B's edge existence (RED expected) 1
```

Org A learned, from an error code alone, that **someone else holds an EMPLOYMENT edge to its
employee** — the exact fact §4.2 must hide, leaked past a correctly-armed policy. This control was
added specifically because the first draft of this experiment *asserted* the `org_id`-leading key was
load-bearing without measuring it. **§4.1:527's column order is a confidentiality control, not a
convention.** It should be commented as such wherever it lands.

## 4. The three assertions

### 1. Armed as org A — the party resolves, only A's edge is visible

```sql
SET app.current_org = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
SELECT p.id || ' ' || p.party_kind || ' ' || v.org_id || ' ' || v.relationship_kind
FROM x4probe_party p JOIN x4probe_party_org_visibility v ON v.party_id = p.id ORDER BY v.valid_from;
```

```
11111111-1111-4111-8111-111111111111 NATURAL aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa EMPLOYMENT
PASS  A resolves the party / A sees exactly one edge / A sees no edge of org B
```

### 2. Armed as org B — the same party resolves, only B's edge is visible

```
11111111-1111-4111-8111-111111111111 NATURAL bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb EMPLOYMENT
PASS  B resolves the same party id / B sees exactly one edge / B sees no edge of org A
```

Same `party.id` from both orgs — one durable identity, two disjoint views. That is the mechanism
§4.2 claims, working.

### 3. Confidentiality — no query as A reveals that B's edge exists

Per `DN-0003:84-86` a leaking `COUNT` is a failure, so counts are tested first-class.

| # | Query (armed as A) | Result | |
|---|---|---|---|
| 3a | `count(*) FROM edge` | `1` | PASS |
| 3b | `count(*) … WHERE party_id = P1` | `1` | PASS |
| 3c | `count(*) … WHERE org_id = B` | `0` | PASS |
| 3d | `count(DISTINCT org_id) FROM edge` | `1` | PASS |
| 3e | `EXISTS (… WHERE org_id = B)` | `f` | PASS |
| 3f | correlated `EXISTS` via the party row | `0` | PASS |
| 3g | `max(valid_from)` — aggregate side channel | `2026-01-01 00:00:00+00` (A's own row, not B's `2026-02-01`) | PASS |
| 3h | **"does anyone ELSE employ my employee?"** `count(*) … WHERE party_id = P1 AND org_id <> A` | `0` | PASS |
| 3i | insert colliding with B's invisible row, real key | `insert-accepted-no-collision` — no `23505` | PASS |
| 3j | forge an edge for org B | `ERROR: new row violates row-level security policy` — no data | PASS |
| 3k | `UPDATE … WHERE org_id = B` | `0` rows | PASS |

**3h-truth — the assertion that stops this being vacuous.** 3b/3h could pass simply because no second
edge exists. So the hidden row's existence is confirmed independently:

```
ground truth (superuser, RLS bypassed): 2 edges for the shared party
visible to console_rt armed as org A  : 1
```

The second edge **is in the table**, and org A cannot see it, count it, or infer it. Org A cannot
distinguish "P1 works only here" from "P1 also works at company B". **The confidentiality requirement
is satisfied.**

## 5. Zero new GUCs — measured from the schema, not from session state

First attempt used `SELECT … FROM pg_settings WHERE name LIKE 'app.%'` and returned **0 rows**, which
looked like a failure. It was a defective instrument: in PG 18 a placeholder GUC set with `SET
app.current_org` is readable via `current_setting()` but **never appears in `pg_settings`** (isolated
and confirmed on a clean `postgres:18.4`: `current_setting` → the value, `pg_settings` → 0 rows).
Recorded because it is a trap for the next probe.

The schema itself is the right instrument — GUC names extracted from stored policy expressions and
from the resolver body:

```
GUC names referenced by every policy this probe created: app.current_org
GUC names referenced by the Variant B resolver:          app.current_org
```

**Zero new GUCs. No `app.current_group`.** And the only policies this probe created are on its own
tables — no existing policy was created, altered, or dropped, so **zero changes to the 141**.

## 6. The one real finding: §4.2 has two readings, and they differ on the definer

The brief's refutation conditions include *"a `SECURITY DEFINER` that bypasses the org floor to do
the join"*. Whether X4 trips that depends on which of §4.2's two mutually inconsistent readings is
meant. Both were built and measured.

**Variant A — `party` granted to `console_rt`, no RLS, reached only through the edge join.** This is
what §4.2:627-629 describes and what §4.1:510's "no `org_id`" implies. Result: **works with no
definer at all** — §4 assertions above. Cost: because `party` carries no RLS, org A can count rows it
holds no edge to:

```
q: SELECT count(*) FROM x4probe_party;  -> 2      (org A holds an edge to only ONE)
q: SELECT count(*) FROM x4probe_party WHERE id = PARTY2 -> 1
```

A learns *"2 opaque UUIDs exist platform-wide"* — **not** who they are (§4.1:512: no PII) and **not**
that B holds an edge to either (3c-3h). Platform cardinality, not the confidential fact. But it is a
cross-tenant aggregate, and by the letter of `DN-0003:84-86` the `party` row is denied data whose
count is not omitted.

**Variant B — the plan as literally written.** §4.1:506 heads the tier *"platform,
**definer-mediated**"* and §4.2:630-631 says *"The `party` row itself is Tier O with **no `console_rt`
grant**, so it is never directly readable."* Revoking that one grant and re-running:

```
q: SELECT count(*) FROM x4probe_party;                     -> ERROR: permission denied for table x4probe_party
q: the plain edge JOIN, with no grant on party              -> ERROR: permission denied for table x4probe_party
PASS  B2 the no-definer join STOPS WORKING under §4.1:506
```

**Under the plan's own words the edge join cannot execute, and a `SECURITY DEFINER` becomes
mandatory.** The correctly-written resolver then behaves:

```
x4probe_resolve_correct() armed as A -> 1 edge   (no B edge)
x4probe_resolve_correct() armed as B -> 2 edges
```

Whether that is a refutation turns on the word *bypasses*. The resolver does `SET LOCAL row_security
= off` — it mechanically bypasses RLS — but it **re-derives the org floor from
`current_setting('app.current_org')`** and never accepts an org from the caller, and it is measurably
org-sensitive (1 edge vs 2). It does not bypass *tenancy*; it re-implements tenancy inside a definer.
That is a narrower thing than the condition names, and it is a judgment call I am flagging rather
than resolving, because it changes the verdict's wording.

What it does **not** change: neither variant needs a second GUC, and neither touches the 141
policies.

## 7. Verdict

**CONFIRMED**, on the criterion X4 sets for itself at `:1647` — *"answerable with zero new GUCs"*,
refuted only by *"an attempt that requires `app.current_group`"*. No attempt required it. Measured
against the brief's four refutation conditions:

| Refutation condition | Result |
|---|---|
| a second GUC | **No.** Only `app.current_org`, in both variants (§5) |
| a change to any existing RLS policy | **No.** Only `x4probe_*` policies created; the 141 untouched |
| a `SECURITY DEFINER` that bypasses the org floor | **Partly — flagged.** Variant A needs none. Variant B (the plan's own §4.1:506 wording) makes one mandatory; it turns RLS off but re-derives the floor from the same GUC and is org-sensitive. See §6 |
| the confidentiality assertion fails | **No.** 11 probes incl. `COUNT`, `EXISTS`, `DISTINCT`, aggregate, `UPDATE`, and `23505` collision — all denied, with the hidden row proven present (§4) |

**§4.2 holds and the 141-table cost does not return.** The plan's "largest single engineering cost"
genuinely does not arise. The entity model does not change shape.

### What §4.2 should change

1. **Resolve the contradiction.** §4.1:506 "definer-mediated" and §4.2:630-631 "no `console_rt`
   grant" are inconsistent with §4.2:627-629's edge-join story. Measured: they are not two
   descriptions of one design, they are two designs, and only one needs a definer. Pick one and say
   which.
2. **Recommendation: Variant B, the plan's own reading.** It costs the definer §4.2:636-649 already
   specifies and budgets for, and it closes the platform-cardinality disclosure that Variant A leaves
   open (§6). §6's pre-mortem *"Scenario 1 — The definer becomes the hole it was built to avoid"* is
   the right worry, and CONTROL 2 shows exactly how it happens: filter on `current_setting`, never on
   a parameter. If Variant A is chosen instead, §4.2 must state that platform-wide party cardinality
   is deliberately readable and reconcile that with `DN-0003:84-86`.
3. **Comment `UNIQUE (org_id, …)` as a security control.** CONTROL 3 shows that dropping `org_id`
   from the front of that key leaks relationship existence through `23505` past a correct policy.
   Currently §4.1:527 reads as an ordinary uniqueness constraint.

## 8. Limits of this evidence — what it does NOT establish

- Schema-level only. No `organizations` FK, no Cedar/PBAC layer, no application path. It answers
  *"can Postgres RLS alone carry this?"* — not whether the REST surface does.
- The authority model was not implemented (out of scope by instruction).
- No timing or planner side channels tested; `EXPLAIN` was not used as an oracle. Row estimates on an
  RLS-filtered table are a known covert channel and remain **untested**.
- `party_link` (§4.1:599-603, cross-tenant control edges) is untouched. It is Tier O and cyclic, and
  it is the next thing that could reintroduce the cost X4 just ruled out.
- Single-statement sessions. Nothing about connection pooling, `SET LOCAL` vs `SET` discipline, or
  whether `app.current_org` can be left armed across a pool checkout was probed.

## 9. Reproducing

```bash
cd ~/Developer/console-lanes/lane-3
bash docs/ideas/experiments/x4/run.sh    # exits non-zero if any control fails to leak
```

Self-grading: prints `PASS`/`FAIL` per assertion and exits with the failure count. Container is
per-run (`x4probe-$$`), removed with `docker rm -fv` on every exit path, and leak-asserted on its
**own** container name only, per `LANE-PROTOCOL.md:152-154`. Observed on every run: `clean:
x4probe-<pid> removed with its volume`.
