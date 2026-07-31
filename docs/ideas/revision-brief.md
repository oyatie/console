# Revision brief — `docs/ideas/ecosystem-plan-DRAFT.md`

> Status: REVISION BRIEF — derived from a verified triage; killed items listed so they are not re-proposed
>
> Target artifact: `docs/ideas/ecosystem-plan-DRAFT.md` (1,854 lines, `Status: PENDING APPROVAL`).
> Verdicts being answered: Architect SOUND_WITH_FIXES; Critic ITERATE, `implementation_ready: false`.
> **This is one revision pass, not a redesign.** 93 candidate items survived triage; 36 were killed and
> are carried verbatim in §KILLED. Nothing in §KILLED may be re-proposed without new executable evidence.
>
> Branch: `docs/ecosystem-plan-session`. Doc edits only. No build, no cargo, no buck2, no npm.
> Do not edit anything under `backend/`, `deploy/`, `scripts/`, `.github/`.

---

## How to execute this brief

1. **Waves are commits.** Do one wave, verify the plan reads consistently at that commit, then continue.
   A wave never leaves the plan asserting two incompatible things.
2. **Each plan section is edited in exactly one wave**, with three declared exceptions that are
   *row-addressed* tables: §7's probe tables, §8's Slice-0 / widenings tables, and §5.11's G-table. When a
   wave touches one of those, it names the exact row and **must not re-flow the table**.
3. **Anchor discipline.** Citations into files this session also edits (`authority-and-approval-model.md`,
   the three amended ADRs, `LANE-PROTOCOL.md`, this plan) are **quoted sentences plus heading names, never
   line numbers**. Citations into unmodified source keep `path:line`. Every code citation is
   path-qualified on first use.
4. **An ADR Decision line is prose about code, not code.** Where an ADR and the code disagree, cite the
   code and record the divergence.
5. ADR numbers come from §ADR ALLOCATION below. **No lane computes "next free".**

### Section → wave index

| Plan section | Wave |
|---|---|
| preamble / header block | 4 (G-claims), 6 (anchor note) |
| §0.1, §0.3 | 6 |
| §0.8 count | 5 |
| §0.12, §0.13 | 6 |
| §0.16 | 4 |
| §1 principles | 4 (p2), 7 (p3, p4) |
| §2 driver 2 | 5 |
| §3.1 | 2 |
| §3.2 Option 4 anchor | 6 |
| §4.0, §4.0.2 | 5 |
| §4.0.3 | 1 |
| §4.1 | 1 (signature + capacity rows only), 2 (tiering), 7 (vocabulary paragraph) |
| §4.2 | 2 |
| §4.3 | 1 (two signature edges only), 2 (everything else) |
| §4.4 | 1 (`gov_approvals` row only), 7 (the other three rows) |
| §4.5 | 1 |
| §4.6 | 5 |
| §4.7, §4.8 | 7 |
| §5.1 | 1 |
| §5.2 | 1 |
| §5.3 | 4 |
| §5.4 | 7 |
| §5.5, §5.6, §5.7 | 5 |
| §5.8, §5.9, §5.10 | 5 |
| §5.11 | 3 (gate row only), 4 (everything else) |
| §6 pre-mortem | **not edited** — recorded sound |
| §7 | row-addressed: 1, 2, 5, 7 |
| §8 Phase 0-7 | 3 |
| §8 Slice 0 / Slice 1 / widenings | row-addressed: 1, 2, 3, 4, 5, 7 |
| §9 ADR block | 2 (cost line), 4 (standing), 5 (economics sentence) |

### What must NOT change — recorded sound by the Critic; do not re-litigate

§0 as applied to what exists, including both silent-empty traps · the four storage tiers · §3's
evidence-backed rejections · §5.7 and §0.14 as models of deciding by pricing · the five pre-mortem
scenarios (they predicted three of the defects the review then confirmed) · §7's known-bad-control and
recorded-RED discipline · and that no gate is weakened, no exposure widened, and no Korea claim is made.

---

## ADR ALLOCATION — assigned here, once

Read `docs/decisions/`: highest issued is **ADR-0026**; `ADR-0013` was never issued and must never be
reused (`docs/decisions/README.md:13`). The true next free number is **0027**.

| Slot | Record | Kind | Amends / relates | Blocking Slice 0? |
|---|---|---|---|---|
| **ADR-0027** | **D1** — Identity linkage is human-asserted; no platform identity row in Slice 0 | reciprocal amendment pair, **narrowing** | amends `ADR-0022`; `ADR-0022` gains `amended_by: [ADR-0027]`, one Decision bullet after its `## Decision` block, index row → `accepted, amended` | no — it *defers* `party` |
| **ADR-0028** | **D2** — `org_id` × `BranchScope` composition and capability-derived all-branch scope | reciprocal amendment pair; **merges G2 + G2b + T5 + T11** | amends `ADR-0003` (Decision edited in place, key created); `ADR-0021`, `ADR-0018` gain `ADR-0028` in `related` | **yes** |
| **ADR-0029** | **D3** — Audit-coverage exclusions are two, bound to a (file, function) pair | reciprocal amendment pair, **retroactive**; Decision text edited in place | amends `ADR-0002` (key created); `ADR-0014` gains `ADR-0029` in `related` | **yes** |
| **ADR-0030** | **D4-A1** — Console rebuild charter (Leptos; carbon-copy visual authority, path/shell structure and enumerated spine boundary withdrawn; §7 nine-item bar retained) | reciprocal amendment pair | amends `ADR-0025` (key created), index row → `accepted, amended` | no |
| **ADR-0031** | **D4-A2** — Generated-client and dual-native reconciliation | reciprocal amendment pair; Decision text edited in place | amends `ADR-0009` (key created); `ADR-0012` gains `related` only | no |
| **ADR-0032** | **N1** — The fold is computed per request; no cross-request materialisation | new, non-amending | `related: [ADR-0021]` | no (unblocks §5.6) |
| **ADR-0033** | **N3** — 전결규정 routing, capacity, obligation loop as a delta on ADR-0023's Engine-Gen | new, non-amending | `related: [ADR-0018, ADR-0023]` | no |
| **ADR-0034** | **N4** — Quantity-bearing lineage, deferred with constraints | new, non-amending | `related: [ADR-0001]` | no |
| **ADR-0035** | **N5** — Economics spine: extend the voucher; COST as a query | new, non-amending | `related: [ADR-0023]` | prerequisites yes, record no |
| **ADR-0036** | **N2** — *OPTIONAL.* Object-policy revocation as a catalog status transition | new, non-amending | `related: [ADR-0023]` | no |

Rules that bind every one of these:

- Pre-acceptance each draft carries `status: proposed`, `doc_status: review`,
  `proposes_amendments_to: [...]` and **may not declare active `amends`** (`docs/decisions/README.md:26`).
- Reciprocity is required **in both records** (`README.md:9`, `:26`) and the `docs/decisions/README.md`
  index rows are updated **atomically in the same commit** (`README.md:3`).
- If **ADR-0036 / N2** is never written, the number stays an **unused gap**. It is never reassigned.
- G-slot disposition: G1 **withdrawn**; G2 + G2b **merged into D2**; G3 → N3; G4 → N4; G5 → N5;
  G6 **struck**; G7 **struck**; G8 **no record** (argue-only); G9 **reclassified to D3, blocking**.

---

## CONFLICTS RESOLVED HERE — a writer would otherwise guess

**C-1 — Which table receives the capacity columns.** §4.0.3 / §4.1 / §8 Phase 3 put
`authorizing_grant_id` + `on_behalf_of_party_id` on `audit_events` (precedent `0149:6-13`); adjudication
N3 / §8.3 put the same two columns on `gov_approvals` (precedent
`backend/crates/platform/db/migrations/0164_bind_consume_four_eyes.sql:34`).
**RESOLVED: Slice 0 lands them on `gov_approvals` only. `audit_events` capacity columns are DEFERRED**
to the widening that lands the D3 write-path enumeration. Reasons, both verified in this tree:
`built_in_audited_tables()` (`backend/ci/gates/migration-safety/src/lib.rs:164-172`) is exactly
`audit_events, regions, branches, users, user_branches`, so an `audit_events` column is **permanent from
the day it lands** (DROP COLUMN is a gate violation); `gov_approvals` is in neither that list nor the
`-- console-gate: audited-table` set, so the same columns there are reversible; and the probe that gives
the `audit_events` pair meaning (`capacity_recorded_on_every_authority_mutation`) needs the enumeration
to exist first. The cost `audit_events` really carries is reaching the value through `AuditEvent`
(`backend/crates/kernel/core/src/audit.rs:83`) at the relevant subset of 466 non-test `with_audit`
references — which is why it is priced, not scheduled.

**C-2 — The §4.0 "systems light up" sentence.** A21 says delete it; B6 says qualify it.
**RESOLVED: delete it.** §4.0.2 already states the correct opposite in the same document; a qualified
version leaves a hedged form of a claim the next section contradicts. B6's substantive additions (the
projected-actions requires-code row, the handler count, the DN-0003 invariant-1 paragraph) all land.

**C-3 — How many re-validation checks the definer has.** B1 says "three implementable"; B2 adds a fourth
(the org predicate on the grant read). **RESOLVED: FOUR named checks.** The §7 probe and the Slice-0 row
**name each check** rather than carrying a count, so the number cannot rot and cannot be mistaken for the
un-fixed text. The **recomputation** check is not one of them.

**C-4 — D4's status.** A17 records D4 as "BLOCKED ON AN OWNER DECISION" with two options.
**RESOLVED: overtaken by newer evidence.** `docs/ideas/d4-frontend-charter.md` (untracked, dated
2026-07-30) captures four owner decisions and splits D4 into **two** records (A1 amends ADR-0025, A2
amends ADR-0009). D4 is therefore blocked on **acceptance**, not on an owner decision, and A17's option
(b) is the one taken. §5.11's row records that; the plan still depends on nothing in it.

**C-5 — G6 (A15 + 20b), G9 (A3 + C4 + 18b), the migration count (3a + C1 + X4b-7 + A22), the CI job
count (16f + A22), and the citation form (2a + C10)** are each **one edit demanded by two or more
streams**. Merged below with all evidence citations retained.

**C-6 — §0.1's anchors.** B10 and the review propose `:116` / `:571` / `:606`. **RESOLVED: use quoted
sentences and heading names only.** The review's replacements are already stale — the upstream file grew
a correction header again.

### ADDED BY VERIFICATION during consolidation — not in any stream

**V-1.** `finance_gl_vouchers` is gate-marked audited: `0160_create_finance_gl_vouchers.sql:21` is
`-- console-gate: audited-table finance_gl_vouchers`, and `discover_audited_tables`
(`backend/ci/gates/migration-safety/src/lib.rs:174-187`) folds that marker into the same audited set as
the five built-ins. So §5.5's `accounting_date DATE NOT NULL` is **irreversible once it lands**. This
strengthens A14's demotion of *"additive DDL on a table with no production data claim"* from a cheapness
argument to an assertion with a named permanence cost. Lands in wave 5 item 5.1.

**V-2.** The `gov_approvals` additive-column precedent is `0164_bind_consume_four_eyes.sql:34`
(`ALTER TABLE gov_approvals         ADD COLUMN target_ref UUID;`), with the sibling at `:33`. The
adjudication's `0164:32-33` is off by one; `:32` is a comment line. Cite the quoted statement.

---

# WAVE 1 — BLOCKING: the signature store, the capacity record, and the definer's checks

Six items. Independently committable. After this wave the plan has exactly one signature store, one
capacity target, and a definer whose checks are all implementable.

### 1.1 §5.1 A — Bootstrap circularity

**(a) Genesis.** Replace the sentence

> Genesis is a migration fact, never a runtime capability, so there is no runtime code path that can mint authority from nothing.

with: genesis is a **platform-principal** capability gated on `PlatformFeature::TenantCreate`
(`backend/crates/platform/platform-rest/src/lib.rs:574`, on the live route registered at `:235`), never a
tenant capability; the existing seed-first-SUPER_ADMIN step inside `create_org` (`:568`, whose own doc
header reads *"POST /api/platform/orgs — onboard a NEW tenant (the only place org rows are created by the
app), seed its first SUPER_ADMIN, and return a one-time OTP"*) is the extension point that mints the
genesis grant.

**(b) The four checks.** Replace the numbered list and its "Payment terms" sentence with four **named**
checks, in this order:

1. `org_predicate` — every grant instance and grant revision row read is filtered
   `org_id = current_setting('app.current_org')`, **a literal, never a parameter**;
2. `visibility_predicate` — `party_org_visibility` filtered on `current_setting('app.current_org')`,
   never a parameter (§4.2);
3. `chain_linkage` — each returned revision's `prev_hash` equals its predecessor's `row_hash`, erroring
   the whole load on the first mismatch;
4. `scope_containment` — every returned scope is inside the armed org's reachable scope set.

"No cross-request cache" stays as a stated property of the read, **not** as a deletable check. Delete

> 2. re-chaining every returned grant revision's `prev_hash`/`row_hash` (`0155:52-53`) and erroring the whole load on the first mismatch;

and delete *"a test that **deletes each of the four checks in turn** and must go RED"*, replacing it with a
test that deletes **each named check** in turn.

**(c) Why recomputation is not available.** Add one sentence: hash **recomputation** is unavailable to
anyone in any language until an explicit key sort plus re-seal lands with a named audit-chain owner,
because canonicalization is insertion-order dependent — `cedar-policy-core 4.11.2`
(`backend/Cargo.lock:569`) turns on `serde_json/preserve_order`, so `serde_json 1.0.150` carries
`indexmap 2.14.0` (`backend/Cargo.lock:6660-6671`) and `serde_json::Map` is an insertion-ordered
IndexMap; `revision_row_hash` is Rust-side SHA-256 over `serde_json::to_vec`, so plpgsql cannot compute
it. `backend/crates/ontology/adapter-postgres/src/instances.rs`'s KNOWN DEFECT block states the
consequence: *"The suite is green because it does not recompute hashes — not because recomputation would
succeed."* Chain **linkage** is what SQL can do and what `company_conformance.rs` already asserts. If the
plan later wants recomputation, it is a **Phase-0 prerequisite with a named owner**, never a Slice-0 check.

**(d) Precedent scope.** Add one sentence: `backend/crates/platform/authz-rest/src/store.rs:576-593` is a
precedent for **re-validation as a discipline** and **not** for reading with RLS off — it is validator
re-execution plus canonicality plus effect agreement on an ordinary pooled read, containing no
`row_security` manipulation. And `group_role_grants_for_user` turns `row_security` off on an owner-only
table carrying no RLS policy, where the switch is nearly inert.

*Evidence: B1, B2, B4.*

### 1.2 §4.5 The traversals

**(a)** Replace the `effective(party, scope, asof)` block. The grant read gains
`AND org_id = current_setting('app.current_org')::uuid` (and so does the revision read); the block
**branches on `scope.level`** — `org_unit` / `organization` levels read Tier N grant instances under the
armed org, the `group` level reads the Tier O group-grant store through its definer (§4.1); and the line

> → re-validate every row's prev_hash/row_hash chain before returning (§5.1)

is replaced by `→ re-validate: the four named checks of §5.1`.

**(b)** Rewrite **인계 완료**. Delete the framing *"a query, not a checkbox"*. It is **one audited
assertion** — outgoing party, incoming party, relinquished scope, and the count **as asserted**, computed
server-side under a fixed authority. State that offboarding **cannot be hard-gated** on 인계 완료 in
Slice 0: an incomplete handover is visible and provable but not blocking, because a completeness count
over heterogeneous artifact edges is principal-relative — `resolve_head` states in its own comment that
`Ok(None)` is byte-identical for "absent" and "not visible"
(`backend/app/src/objects.rs:691-697`) and `DN-0003:85-86` makes omission-including-counts binding, so two
people run it and get different answers and the delta may not be exposed. Do **not** assert a frequency
for orphan edges: `object_links.src_id`/`dst_id` carry no FK (`0102:57`, `:59`) and no delete path was
traced. Clause 1 of the traversal must name the storage the four scope edges actually get (wave 2, item
2.1) — today it has none either, not only clause 2. Clause 2 must reference the person-linked artifact
edge declared in §4.3 (wave 2).

**(c)** Split **handover** into two operations: time-boxed reverting **대리** (with automatic revert) and
permanent **전보** (grant revocation plus the 인계 완료 gate). Cover **연차** and **퇴사** (all four of
연차/퇴사/휴직/복직 occur zero times in the plan today).

**(d)** Restate the *"Why may this person do this?"* traversal against the signature store that actually
ships: `gov_approvals` row → `authorizing_grant_id` → `grant` → `{source, scope, valid_from, reason}`,
then the revision chain at the signature's timestamp. (`approval_signature` is deleted in 1.3.)

*Evidence: B1, B2, X4b-5, A8, 33a, A20, 26.*

### 1.3 §4.1 and §4.3 — the signature rows only (do not re-flow either table)

**(a)** Delete the §4.1 Tier N row

> | **`approval_signature`** | *(signer party, authorising grant, scope, capacity, on_behalf_of)* | instance id | immutable | yes — 1 |

and delete the two §4.3 rows `signature_grant` and `signature_on_behalf_of`. Reason, stated inline: the
signature store is `gov_approvals`, already shipped; storing a signature in
`ont_instance_revisions.attributes` is a strict regression — a JSONB bag with `ON DELETE CASCADE` on org
(`0155:37-56`) loses the `(approver_id, org_id) REFERENCES users` FK, the SoD CHECK, the single-use
`UNIQUE (org_id, approval_id)` index and the RESTRICT durability posture every shipped gate binds against.

**(b)** Replace the §4.1 Tier T row

> | `audit_events.authorizing_grant_id`, `.on_behalf_of_party_id` | **capacity** — §4.0.1 | nullable columns | — | **yes** |

with `gov_approvals.authorizing_grant_id`, `.on_behalf_of_party_id` — two nullable additive columns
following the precedent `ALTER TABLE gov_approvals ADD COLUMN target_ref UUID;`
(`0164_bind_consume_four_eyes.sql:34`). Retain `CHECK (approver_id <> requested_by)`
(`0153_create_governance.sql:74`) **verbatim** and add the invariant sentence: **capacity refines a
signature; it never satisfies a four-eyes gate.** Record that five DB-enforced four-eyes CHECKs exist
(`0153:74`, `0122:63`, `0163:27`, `0186:39`, `0191:46`) and **none becomes conditional on capacity**.
While `party` is deferred, `on_behalf_of_party_id` carries **no FK** (D1 clause 4(a) forbids a
cross-tenant FK regardless) — declare it a bare nullable UUID and state the absent FK.

*Evidence: A6, A7 (per conflict C-1), A5 (R9), 23.*

### 1.4 §4.0.3 and §4.4's `gov_approvals` row

**(a) §4.0.3.** Keep the finding; retarget it. The two columns land on `gov_approvals` in Slice 0 (C-1).
State that the `audit_events` pair is **deferred**, that its DDL is two nullable columns but its real cost
is reaching the value through `AuditEvent` (`backend/crates/kernel/core/src/audit.rs:83` — fields id,
before, after, request_context, classification, trace, occurred_at; no capacity field) at the relevant
subset of **466** non-test `with_audit` references under `backend/crates`, and that an
`AuditEvent::authorized(…, grant_id)` constructor is a **RECOMMENDATION, not a requirement** — a
compiler-enforced constructor across 466 sites is a larger change than this plan can price. Name the set
where null is a defect by pointing at the D3 enumeration (wave 3, Phase 0).

**(b) §4.4.** Replace the `gov_approvals` row's blocker cell

> `UNIQUE (org_id, request_ref)` `0153:75` — one decision per request

with: anchor `0153:76`; an **N-node 결재 line already ships** and runs the newest approval domain in the
repo — `backend/crates/orgchange/adapter-postgres/src/lib.rs:1479-1483` binds `request_ref` to `step_id`
and `requested_by` to `request.drafted_by`, so an eight-step `org_change_request` writes eight immutable
rows; the constraint is **one signature per node**, not one per request. Name the failure mode: the plan
cited the migration's own inline comment (`-- one decision per request`, `0153:76`) instead of reading the
caller — the same defect class as `ADR-0002:20`'s false cardinality. State that capacity costs two
nullable columns and **nothing has to be relaxed**: 전결 by delegated authority is already two rows (same
approver, different `request_ref`, same `requested_by`), and 전결 where the competent authority IS the
drafter needs zero approval nodes. Keep the cross-org FK blocker (`0153:78`) as written.

*Evidence: A7, 18a, A5, K2-new-signature-store (killed — do not reintroduce a new signature store).*

### 1.5 §7 — row-addressed probe changes

| Action | Row | New content |
|---|---|---|
| replace | `definer_revalidation_each_check` | asserts: baseline GREEN with all four named checks; **each of the four deletions individually RED** — `org_predicate`, `visibility_predicate`, `chain_linkage`, `scope_containment`. Known-bad control: any one check deleted. **No count in the prose — name the checks.** |
| add (Integration) | `definer_returns_no_foreign_org_grant` | one party with a visibility edge in **both** orgs and one grant in each; armed as org A the call returns exactly one row. Known-bad control: the definer exactly as §4.5 specified it before this revision (no org predicate on the grant read). |
| rename | `genesis_grant_not_runtime_mintable` → `genesis_grant_mintable_only_by_platform_principal` | known-bad control: a **tenant**-authenticated endpoint creating a grant with no authorising grant. |
| add (E2E) | `daeri_records_both_parties` | a 대리 signature records outgoing and on-behalf-of parties. Known-bad control: a 대리 signature whose `on_behalf_of` is null. |
| edit | `slice0_capacity_recorded` | retarget to `gov_approvals.authorizing_grant_id`; keep the `gov_approvals.approver_id`-only shape as the known-bad control. |
| edit | `capacity_recorded_on_every_authority_mutation` | state that it reads the D3 write-path enumeration (wave 3) and covers the `gov_approvals` columns; the `audit_events` pair is out of scope until deferred columns land. |
| add to §7 preamble | — | the measured instrument trap: on PG 18 a placeholder GUC set with `SET app.current_org` is readable through `current_setting()` but **never appears in `pg_settings`**, so a GUC inventory must be extracted from stored policy expressions and function bodies, not from session state. X4's first attempt returned 0 rows and looked like a pass. |

*Evidence: B1, B2, B4, 26, X4-4.*

### 1.6 §5.2's delta table and §8's Slice-0 rows

**(a) §5.2.** Add a **sixth** delta row: *release-reset* — a signature is a statement about a document
**state**, so a change crossing a `delegation_rule` band invalidates signatures taken under the prior
band and re-routes. Note in the same row that Slice 0's one band and one step cannot surface this. Its
probe's known-bad control: an implementation that keeps signatures valid after the amount is raised.

**(b) §8 Slice 0.** Row-addressed:

- replace `| `effective_grants_for` | the definer, with all four re-validation checks |` with
  `the definer, with the four named re-validation checks of §5.1 (org_predicate, visibility_predicate, chain_linkage, scope_containment)`;
- replace the `approval_signature` row with a `gov_approvals` row: 1 row carrying
  *(signer, authorising grant, scope)*;
- replace the addition row `| `audit_events.authorizing_grant_id` | populated on the one signature |`
  with `gov_approvals.authorizing_grant_id` + `on_behalf_of_party_id`, and state that `on_behalf_of` is
  **exercised** by the 대리 probe rather than shipped unused — pre-mortem 4's named failure is a column
  nothing writes.

*Evidence: 23, B1, A7, 26.*

---

# WAVE 2 — BLOCKING: the storage substrate — what can and cannot be an `ont_link`

Six items. This wave carries the §9 cost line it implies; do not split it.

### 2.1 §4.3 Relationships — every remaining row and bullet, in one pass

**(a) Scope descriptors.** `grant_scope` and `position_at_scope` become **`Stored as: property {level, node_id}`** using the shipped `AccessScopeLevel` vocabulary
(`backend/crates/kernel/core/src/access_scope.rs:28-34` `enum AccessScopeLevel { Group, Org, Region, Branch, Worksite }`, `:37-40` `struct AccessScope { level, node_id }`; spec `docs/specs/org-hierarchy.md:172-173`) — **not** `ont_link`. Delete `organization` and `group` from every `ont_link` target set. Make the `org_unit` arm a property too, so there is one storage form and the fold reads `AccessScope` uniformly. Reason inline: an `ont_link` endpoint must be an `ont_instances` row **in the same org** —
`0155_create_ontology_instances.sql:76-77` FKs both endpoints to `ont_instances(id, org_id)` — so
`grant → group` is not storable (X4b CASE 3a, executed:
`ERROR: insert or update on table "ont_links" violates foreign key constraint "ont_links_to_instance_id_org_id_fkey"`) and `grant → organization` is not either. **Caveat, verbatim: Slice 0 must not publish a `grant_scope` link type whose declared target set includes `group` or `organization`** — that arm fails at the first write and the schema is already published by then.

**(b) The four `work_*` edges.** Replace `work_scope`, `work_origin`, `work_performed_at`,
`work_jurisdiction` — today all `ont_link` — with the same mechanism: a scope-descriptor **property** on
the `work` row (or an `object_links` row, the shape §4.3 already uses for `work_artifact`). Reason: `work`
is Tier T **projected**, and a projected type owns no `ont_instances` rows —
`backend/crates/ontology/adapter-postgres/src/instances.rs:1443-1450`: *"A `projected` object type owns no
store of its own … This is a READ-ONLY view … there is no create/stage path here, only list."* So all four
edges are rejected by referential integrity today.

**(c) `work_artifact`.** Keep `object_links` as the storage choice; **strike the reason.** Delete

> `link_type` is validated only by slug regex (`0102:63`) — so a new edge kind needs no migration

It is stale as of `0130`/`0132`: `0130_create_link_types.sql` created the registry, seeded twelve labels
(`:37-49`, none of them `work_artifact`), granted `console_rt` **SELECT only** (`:52`), and added
`object_links_link_type_fkey … ON DELETE RESTRICT NOT VALID` (`:75`), validated by `0132:7-8`. `console_rt`
cannot INSERT a `link_type`, measured: *"permission denied for table link_types"* (X4b S1). So **a new
edge kind IS a migration** — one appended `link_types` row per kind, exactly like the `object_types` kind
row the plan already budgets. Carry D3's per-kind cost: one `RESOLVABLE_KIND_AUTH` row, one `resolve_*`
arm, and a one-time audit of pre-existing links of that kind, because registering a kind makes prior links
retroactively resolvable with no backfill re-check.

**(d) Person-linked artifacts.** Declare the person-endpoint artifact edge with its cardinality and owner
(an `object_links` row on the seeded `person` kind — `0102:32-33` already seeds it), and reference it from
§4.5's 인계 완료 clause 2. Today the distinction the PII and handover boundary rests on is inferable only
from a probe.

**(e) Naming.** Use **one** spelling of the split table everywhere: `lot_split` at the §5.8 table and
`lot_derivation` at §4.3 and inside §5.8's own traversal are the same thing. Pick `lot_split`.

**(f) The bold rule under the table.** Change the absolute to a reachability statement: **no *reachable*
path writes an `ont_links` row without a property carrying `config.link`.** Name the exception in one
line: `PgInstanceStore::create_link`
(`backend/crates/ontology/adapter-postgres/src/instances.rs:291`, INSERT at `:319`) writes a row directly
from a bare `link_type_id` and has **zero non-test callers** (every call site is under `tests/`), so
`grep 'INSERT INTO ont_links'` finds two sites and only one is the property mechanism.

*Evidence: X4b-1, B9, 33b, A8, A9, X4b-6, 33c, 33a, B8, X1-1.*

### 2.2 §4.1 — tiering

**(a)** Change the Tier O heading **"(1 new table)" → "(2 new tables)"**. Split the Tier N `grant` row by
scope level: org_unit- and organization-scoped grants stay Tier N (X4b CASE 1 measured working end to
end); **group-scoped grants cannot be Tier N at all** — they move to **Tier O beside `group_role_grants`**,
reachable only through an audited definer, deferred to W5/W8. State the inherited burden measured in X4b
§5: the shipped group definer references no `app.*` GUC, so the **caller** is the org floor for every
group-scoped authority read; a new Tier O grant store must be authorisation-complete **inside** the
definer, keyed on the authenticated principal, never on a caller-supplied org or group id. Evidence:
X4b CASE 2c/2d — org B, a sibling in the same group, gets **0** rows while RLS-bypassed ground truth shows
the revision present; `0155:18` `org_id … NOT NULL` leaves no third option for the row's tenancy;
`group_role_grants` is in `owner_only_table_allowlist` (`backend/ci/gates/tenant-isolation/src/lib.rs:115-129`).

**(b)** Mark **DEFERRED** and strike from the 0207+ list: the Tier O `party` table, and the Tier T rows
`party_org_visibility`, `users.party_id`, `employees.party_id`. Carry R2's five non-foreclosure
constraints verbatim: (i) no cross-tenant identifier as a FOREIGN KEY and none in any UNIQUE constraint or
index whose key does not lead with `org_id` (X4 CONTROL 3, measured); (ii) the authorization path never
reads `employees`; (iii) `0075:16-17`'s CHECK is never dropped or relaxed; (iv) when the handle lands it is
an ordinary tenant-scoped row homed at the existing sentinel org
`00000000-0000-0000-0000-00000000face` under the standard `org_isolation` policy — **not** a Tier O
carve-out, no new GUC, no definer-mediated read; (v) any eventual edge FK is `RESTRICT`/`NO ACTION`.
State explicitly that **Slice 0's grants and capacity recording do not need the party, so no lane waits on
it.** Reason for the deferral: `users` is in `built_in_audited_tables()`
(`backend/ci/gates/migration-safety/src/lib.rs:164-172`), so DROP COLUMN is a gate violation — a column
added today is permanent while adding it later is purely additive.

**(c)** Annotate `party_org_visibility`'s key as a **security control**:
`UNIQUE (org_id, party_id, relationship_kind, valid_from)` — **`org_id` leads the key because a unique
index is enforced physically below RLS; dropping it from the front discloses, through error `23505`
alone, that another org holds an edge to this party.** Require the comment in the 0207 migration text
itself. Measured: X4 CONTROL 3 returned
`ERROR: duplicate key value violates unique constraint "x4probe_edge_control_uniqleak_party_id_relationship_kind_va_key"`
past a correctly-armed FORCE policy, while the real key returned `insert-accepted-no-collision`.

**(d)** State the `party` resolution mechanism as a **named pre-condition for when `party` lands** (not
for Slice 0), picking one on the record: (a) `party` minted per passkey credential and self-linked by the
human at second-org onboarding — name the endpoint, and note this is **local** identity, not federation,
so ADR-0022 permits it; or (b) resolution is a platform-principal operator action with an audit record,
never a tenant capability. Record R2 constraint (iv) beside it and **reconcile the Tier O heading**
"platform, definer-mediated" with it.

*Evidence: X4b-2, B9, A2, X4-1, B3.*

### 2.3 §3.1 Shared vocabulary

Fix the FK anchor **`0155:78-79` → `:76-77`** in the "Tier N cannot hold a cross-tenant edge" bullet (and
at the §4.1 `employment` bullet, same wave). Verified by direct read: `:76` is
`FOREIGN KEY (from_instance_id, org_id) REFERENCES ont_instances(id, org_id) ON DELETE CASCADE,`, `:77` its
`to_instance_id` twin, `:78-79` the closing paren and `CREATE INDEX idx_ont_links_from`. Add one sentence
pointing the constraint at its consequence — the scope-descriptor property of §4.3 — so the constraint the
section already calls *"the single constraint that shapes the entity model"* is applied where it bites.
This is a **consistency edit, not a discovery**: §3.1 states the constraint and §4.3 contradicted it.
Also state, in the Tier P paragraph, that projection is **code-gated (a match arm plus a CHECK), not
CI-gated** — `allowlisted_projected_table` (`instances.rs:1479-1498`) is a compiled-in `match`, not
`tenant-isolation/src/lib.rs`.

*Evidence: X4b-4, 2a, 29.*

### 2.4 §4.2 Why there is no second tenancy dimension

**(a)** Bound the central claim. After

> This is the plan's central claim, and it is the reason §0.1 matters.

add that X4 confirms it **for visibility of a known party within the armed org** and does **not** extend
to cross-org authority resolution, which X4b then measured failing (X4 §8: *"Schema-level only. No
`organizations` FK, no Cedar/PBAC layer, no application path."*). Add the falsifying case: armed to org A,
resolve eligible approvers for a step whose competent unit is at **group** scope where the only qualifying
holder is a user of org B — on the current design this is not answerable without iterating member orgs or
a Tier O grant store. Say so honestly.

**(b)** Resolve the two readings in one sentence: the plan means **Variant B** — the party/edge join runs
**inside** the definer, which re-derives the org floor from `current_setting('app.current_org')` and never
accepts an org from the caller. §7 already decides it (`party_not_readable_as_console_rt` asserts the
direct SELECT is denied). Reason it is the better of the two, measured: Variant A leaves platform-wide
party cardinality readable by any tenant (`SELECT count(*) FROM x4probe_party` returned `2` where org A
holds an edge to only one), which collides with `DN-0003:84-86`.

*Evidence: B9, X4-3, X4-2.*

### 2.5 §9 ADR block — the cost line

Replace

> Sixteen new entities cost one owner-only table, two tenant tables, two nullable FK columns and one definer.

with **two** owner-only tables (`party` **and** the group-scoped grant store), each carrying its own
`owner_only_table_allowlist` gate classification and its own audited definer; two tenant tables; two
nullable columns on `gov_approvals`; one definer per owner-only store. Delete the word **"untyped"** from
*"the untyped `object_links` edge store"* (the registry is typed as of `0130`). Re-anchor
`0076:49-50` → `0075:6,13` in the Alternatives paragraph.

*Evidence: X4b-3, B9, X4b-6, A22.*

### 2.6 §7 and §8 — row-addressed consequences of this wave

- §7 `no_new_gate_classification`: name **both** owner-only tables.
- §7 add `visibility_unique_key_leads_with_org_id`: asserts an insert colliding only with an invisible
  other-org row is **accepted**; known-bad control: the same table keyed
  `UNIQUE (party_id, relationship_kind, valid_from)` must be RED with `23505`.
- §7 add the falsifying probe: an `ont_link` INSERT naming a projected type's row must be **RED on the
  FK**.
- §7 `fold_is_scope_parameterised` and `requirement_3`: name the **Tier O** group-grant source, or both
  probes assert a result the Tier N substrate cannot produce.
- §8 Phase 6 X4 row: add the falsifying case from 2.4(a).
- §8 Phase 7 rung ②: add the `link_types` rows for the new edge kinds beside the existing
  `object_types` kind rows.
- §8 Phase 3 crate 1: strike `party`, `party_org_visibility` from *"migrations 0207+"*.
- §8 Slice 0: restate the `work` row as **"1 `work` ROW written by the domain use-case, listed through the
  projection"** — a projected type has no instance-create path.

*Evidence: X4b-3, X4-1, 33b, X4b-5, B9, A9, A2.*

---

# WAVE 3 — BLOCKING: exposure, gates, slots, and the CI premise

Six items, all in §8 plus one §5.11 row.

### 3.1 §8 Phase 7 — the deployment-dependency paragraph

Replace

> So slice 0's Tier T half lands and ships; its Tier N half is CI-provable but not deployable.

with: *"So slice 0's Tier T half is **CI-provable**; **exposure remains HOLD for both halves**"* — per
27/27 `"implementation": "HOLD"` and `"exposure": "HOLD"` in
`docs/program/console-capability-registry.json` (counted: 27 each) and the jurisdiction-control HOLD check
in `scripts/console/validate-console-truth-ledger.mjs`. `docs/program/console-program-ledger.md` states
*"Nothing in the idea document is approved work."* Add one Phase-7 rung: register Slice 0 and each widening
group as capability rows carrying signature story, `evidence_path`, leaf commands and ownership roots —
stated as a **governance step required by `dispatch_rule` prose**, explicitly noting that `dispatch_rule`
and `hold_rule` have **no executable reader** (`grep -rn dispatch_rule scripts/ backend/ tools/ .github/`
returns nothing) and that the executable constraints are
`validate-console-truth-ledger.mjs:254-257` (buck targets keyed on `delivery.rust_status`; each declared
target must resolve) and its jurisdiction-control HOLD loop. **Do not** add a Buck2-clause amendment — see
KILLED B5a.

*Evidence: B5b.*

### 3.2 §8 Phase 7 build-system paragraph + Phase 6 X8 row

Delete the clause

> while `prelude/` is missing so the buck2 graph is already broken

and delete the X8 row's *"`prelude/` is **missing** (verified: no `prelude` dir)"* setup and its
inference. Replace with the measured chain: `prelude/`'s absence is **correct** because `.buckconfig:15-16`
declares `[external_cells]` / `prelude = bundled`, supplying the prelude from inside the buck2 binary;
`tools/buck2:1` is `#!/usr/bin/env dotslash` with per-platform blake3-pinned digests; and the required job
**"Support domain — Buck2 unit reachability"** (`.github/workflows/ci.yml:164`) passes because `:192` runs
a real `tools/buck2 test //backend/crates/support/domain:console-support-domain-unit`. Rewrite the X8 row
as **ANSWERED**, pointing at `docs/ideas/experiment-results.md`. **Keep the governance question exactly as
framed** — `docs/PIVOT-2026-07-28.md` is not in `docs/decisions/`, so neither cargo nor buck2 is an
accepted decision — but state that the status quo is healthy, so there is **no forced migration**. Phase
7's *"targeting the CI that **exists** (buck2 live)"* becomes positively grounded rather than a bet. Drop
the **"five buck steps"** count and cite the job **by name** instead; the count is wrong (install steps at
`:103, :176, :215, :271, :307, :398, :703, :860` plus `tools/buck2 test` at `:192, :465, :660, :664, :675`)
and load-bearing for nothing.

*Evidence: 16e, X8-1, X8-2.*

### 3.3 §8 Phase 0 — the reference documents

Add to the two-file table:

- a **third artifact**: one row per lane — crates, owned paths, **migration slot block from 0207**, and the
  widenings it may take, with W11-W13 in it. Present it as instantiating
  `docs/program/LANE-PROTOCOL.md:89` (*"single global sequence, highest `0204`. Blocks assigned per lane in
  the Phase-0 commit; take the number immediately before push"*), **not** as a new mechanism. Carry the
  adjudication's count: **nine** slots are already claimed (D2/T5 ≤2, T2 1, D3 2, N3 1, N5 1, N1 1, T10 1)
  against an unallocated serial resource, and `check_migration_versions`
  (`backend/ci/gates/migration-safety/src/lib.rs:131-141`) enforces gap-free contiguity, so the version
  space **serialises Phase 3** — this, not any CI collision, is what orders the work. Add the
  non-foreclosure constraint that **no migration 0207+ may hard-code `'KR'` or `'KRW'`**.
- the **D3 write-path enumeration** moved here from Phase-7 prepwork: one row per authority-mutating write
  path in `docs/specs/ecosystem-entity-components.tsv`. It is the artifact
  `capacity_recorded_on_every_authority_mutation` reads.
- **two prerequisites**, named as prerequisites: **5.7a** — harden the audited-table DROP COLUMN resolver
  against the `ONLY` and schema-qualified spellings with one negative unit case per spelling — is a
  prerequisite to **any migration 0207+**, and Slice 0 lands migrations at 0207+. Verified by direct read:
  `table_name_after_alter_table` (`migration-safety/src/lib.rs:314-322`) advances past only `if exists`,
  `tokenize_sql` (`:443-460`) emits a boundary at every non-alphanumeric character including `.`, and
  `built_in_audited_tables()` (`:164-172`) holds neither `only` nor `public` — so
  `ALTER TABLE ONLY users DROP COLUMN x` resolves the table to `only` and
  `ALTER TABLE public.users DROP COLUMN x` to `public`, raising no `DropAuditedColumn`; the `#[test]` count
  in that file is 0. **5.7b** — a Leptos-shape extractor in `route-inventory.mjs` plus the reciprocal
  assertion in `validate-console-truth-ledger.mjs` — is a prerequisite to any console surface.
- one line: reconcile §4 and §5 against the benchmark matrix and the four research surveys, recording
  adopt/reject/contradict with confidence labels carried through, and **no plan decision resting on an
  UNCERTAIN/UNKNOWN row**. (Today a grep over all 1,853 lines returns zero hits for benchmark, research-,
  Foundry, Workday, SAP, Odoo, NetSuite, ServiceNow and Salesforce.)
- one line: **X-CITE**, a mechanical citation audit, as a plan deliverable — the citation failure is
  systemic rather than clerical.

*Evidence: 15, A11, 18a, A10, 28, A22.*

### 3.4 §5.11 — add a gate row (row-addressed; the rest of §5.11 is wave 4)

Add one row classifying gates by **what they pin**: safety pins are never weakened;
literal-sameness pins on *decisions* are replaced by derivation per crate. Record 5.7a and 5.7b as
prerequisites (3.3). Note the Phase-3 CI-wiring cost is a **defect to delete, not a toll**:
`scripts/console/check-ci-preflight.mjs:430-453` already derives its requirement from a generated BUCK
file. §5.11 today has no gate row at all — `grep -ci` over the plan returns 0 for `ADR-0025`,
`route-inventory`, `check-ci-preflight`, `migration-safety` and `command-database`.

*Evidence: A10.*

### 3.5 §8 Phases 5 and 6 — renumber, and record what has been executed

**(a)** Renumber so the **experiment phase precedes the Phase-2 trial run**. The Phase 6 heading already
claims the order the numbering contradicts (*"experiments (their own phase, before the design is
trusted)"*, and its X4 row says *"so test it first"*). This is a renumber, not a redesign; principle 5
(§1) is the authority.

**(b)** Give the table an **ANSWERED** column. Mark **X1, X2, X4, X8, X9 ANSWERED** with their record
paths (`docs/ideas/experiment-x1-x2.md`, `experiment-x4.md`, `experiment-results.md`) and their re-runnable
probes under `docs/ideas/experiments/{x1,x2,x4,x4b}/run.sh`. Add **X4b as a row of its own** — it exists as
a record (`docs/ideas/experiment-x4b.md`) and appears nowhere in the plan's table.

**(c)** State explicitly: **X3, X5 and X6 cannot be prepwork** — all three need `effective_grants_for` to
exist, so they are slice-0 work; move them beside Phase 4 rung 4 ("the definer probes"). X6 additionally
needs realistic grant counts. **X7 is unrun for a different reason** — it requires pushing a branch, so it
is outward-facing and needs explicit authorization; say so rather than listing it as a pending prediction.
Restate the gate as: **X1, X2, X4, X4b, X8, X9 recorded before the first implementation commit; X3, X5, X6
recorded before Slice 0 may be declared green.** (The stricter form is circular — see KILLED 16d.)

**(d)** Restate **X5** as a constructed query with an expected-fail baseline, in X4's shipped form: the
four grant sources encoded, the specific decision Cedar must reach, and the concrete input on which a
Rust-fallback implementation is RED. Its current known-bad control (*"a case needing a companion
evaluator"*) is a refutation scenario, not an observable input, which principle 5 forbids.

**(e)** Delete Phase 7's *"X8 runs first"* — now a no-op.

*Evidence: 16b, XS-1, 16g, 16d, X8-1.*

### 3.6 §8 Phases 1, 3, 4, 7 — counts, CI wiring, and the LANE-PROTOCOL corrections

**(a)** Make the CI-wiring step **per test, not per crate**, and cite the worked template by **target
name** (line numbers already drifted): test file
`backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs` → `rust_test` target
`//backend/crates/ontology/rest:console-ontology-rest-itest-object_policy_attach_as_runtime_role` →
`sh_test` Postgres wrapper `//tools/buck:ontology-object-policy-attach-postgres` → the workflow step
listing that target (`.github/workflows/ci.yml:239`). State the reason it is per test: `mapped_srcs`
hand-lists every file the test crate reads and **buck2 does not glob**, so a file added later to a shared
harness is invisible until that list is edited — which is why link 2 or link 3 can be missing while the
test passes locally and nothing fails.

**(b)** Replace **"The 14 CI jobs"** (Phase 1), **"the 14 CI jobs"** (Phase 4 rung 7) and X7's
*"backend runs all 14"* with **"every job in `.github/workflows/ci.yml`"**, or "the ten jobs as of
<commit>". There are **ten**: `preflight:75`, `support-domain-unit:163`,
`postgres-domain-reachability:194`, `company-conformance:244`, `generated-face-authority:291`,
`backend:340`, `dev-up-smoke:684`, `repo-gates:741`, `api-contract:827`, `kubernetes-manifests:906`.
Drop the number where possible so it cannot rot again.

**(c)** Phase 1's immutable target: qualify the Bun count — *"**60,624 tests on Linux x64** (macOS arm64
58,850, Windows x64 57,337), **0 skipped, 0 deleted**"*. The "0 skipped, 0 deleted" half is CONFIRMED and
stays.

**(d)** Phase 4: change *"full suite on 6 platforms"* to **"full CI on all platforms"** — the wording the
primary-source table supports; no primary reading supports six.

**(e)** Add a Phase-7 correction rung for `docs/program/LANE-PROTOCOL.md`: its status header
(*"Status: **prep artifact, not yet exercised.** Fan-out is not authorized until §4 passes."*) is stale
against `docs/program/console-program-ledger.md:769` (*"the fan-out is green"*) and `:751`; and its
migration high-water at `:89` still reads `0204` — **0205 landed, 0206 is in flight in lane-1, reserve
from 0207**. Cite the corrected header where §8 opens fanout. For `:268-269` (*"this repo has **no
`.cargo/config.toml` and no `[profile]` section**"*) correct **only what changed**: the `[profile]` section
landed (`backend/Cargo.toml:359` `[profile.dev]`, `:362` `[profile.test]`) and sccache is wired via the
subprocess environment with a measured 0% → 35.4% (`console-program-ledger.md:675`). **Keep "no
`.cargo/config.toml`" and record WHY it must stay absent** — the file would apply in CI where no runner has
sccache and every Rust job would fail — or a later lane will "fix" it and break every Rust job.

*Evidence: X9-1, 16f, A22, C2, C3, 17a, 17c.*

---

# WAVE 4 — BLOCKING: the governance surface

Six items. §5.11 is edited **once**, in this wave (except the gate row added in 3.4). Every downstream
claim about G1-G9 moves with it — header, §0.16, §5.3, §8 Phase 7 first rung, §9 standing.

### 4.1 §5.11 preamble

Add two sentences and the allocation reference:

- **Reciprocity is machine-enforced; clause compatibility is not.** Two accepted ADRs both declaring
  `amends: [ADR-0003]` and editing the same Decision line incompatibly **pass CI** and leave the
  authoritative record self-contradictory. Verified: `scripts/check-adrs.mjs:23-27` defines
  `RECIPROCAL_RELATIONSHIPS` as exactly `amends/amended_by` and `supersedes/superseded_by`; the loop at
  ~`:399-406` fails only when the target does not declare the reciprocal key; `related` is validated only
  as an inline array (`:248-249`). No clause-level collision check exists. **This is the stated reason G2
  and G2b merge into one record.**
- **ADR numbers are assigned centrally.** Every draft carries `ADR-XXXX-DRAFT`; the integrator assigns
  numbers in **one atomic commit** together with the `docs/decisions/README.md` index rows (`README:3`).
  No lane computes "next free". Name the observed failure: four independent judges each computed "next free
  after ADR-0026" and all four claimed `ADR-0027`. Point at this brief's allocation table as the
  assignment of record.

*Evidence: A12, A18.*

### 4.2 §5.11 G1 → WITHDRAWN, and its three downstream claims

Replace the G1 row with **"WITHDRAWN — premise false; no clause to amend"**, and record that the
governance action is **D1 (ADR-0027)**, a **narrowing** amendment pair on ADR-0022. Delete *"unauthorized
by ADR"* and the `ADR-0022:25,33-39` citation: `:25` starts `## Context`, the `## Decision` block is
`:31-39`, and the string **"org-scoped" appears nowhere in ADR-0022** — it decides against a speculative
external IdP seam and confines `console-identity-application` to local org/account administration. Delete
the plan's undeliverable G1 claim *"one durable identity per natural or legal person, across every tenant
and vertical"* from the §4.1 `party` row's purpose cell — `README:7` makes an accepted ADR authoritative in
scope, so writing an unachievable guarantee into one is a governance liability.

Then delete G1 from all three block-work claims:

- header: *"**G1, G2 and G2b block work**"* → G2 (now D2/ADR-0028) only;
- §8 Phase 7 first rung: *"G1 (platform `party`) and G2 (`org_id` × `BranchScope`) block slice 0"* → D2
  only;
- §9 standing: *"of which **G1 and G2 block slice 0**"* → D2 and D3 block slice 0.

*Evidence: A1, B3.*

### 4.3 §5.11 G2 + G2b → one record (D2 / ADR-0028), and §0.16 + §5.3

Merge G2 and G2b into **one** record and say **why the merge is mandatory**: CI cannot see a clause
collision (4.1). In §0.16, delete **"sole"** from *"That is the sole tenant-side derivation of
`BranchScope::All`"* and delete the conclusion *"Only `ADR-0003:20`'s Decision text changes, so **one
reciprocal ADR pair (G2b)**"*. Name the **second shipped derivation**:
`backend/crates/platform/request-context/src/lib.rs:421-422` mints `BranchScope::All` for a
`{Role::Admin}`-only principal after live group-membership proof. Bring the realtime fan-out into scope
(`backend/crates/platform/realtime/src/lib.rs:843`, `:885`, `:899` never compare `org_id`). **Re-trigger to
PRESENT tense:** the divergence exists today, independent of any `Role` deletion, so C5 / `Role` deletion
is **not** the trigger and must not gate the plan — amend §5.3's C4b and C5 rows accordingly. Add **R11**:
group designation stays **STORED / EXPLICIT / AUDITED**, and control edges may never be an input to any
authorization resolver, because `group_memberships` is the sole input to the cross-entity `{ADMIN}` + `All`
mint. Add one line to C4b/G2b naming the **onboarding seeder** (`platform-rest/src/lib.rs:568`) as a
SUPER_ADMIN write site the `Role`-deletion path must cover.

*Evidence: A4, B4.*

### 4.4 §5.11 G9 → D3 (ADR-0029), BLOCKING

Reclassify G9 from Phase-7 prepwork to **BLOCKING as D3**, a **retroactive** reciprocal amendment pair on
ADR-0002 in which ADR-0002's Decision text is **edited in place** — a reciprocal key alone leaves a false
sentence standing. Correct both plan sites to **two** exclusions, each bound to a **(file, function)**
pair. Delete the prose

> `ADR-0002:20` requires the `with_audit` path and states the CI `audit-coverage` gate's *"exclusion set contains exactly one entry"* (ADR-0014 establishes it is `location_pings`, with a test asserting it is the only one).

and the §8 Phase-7 row's *"The exclusion set has exactly one entry (`ADR-0002:20`) and a test asserts
it"*. The gate returns TWO: `backend/ci/gates/audit-coverage/src/lib.rs:90-111` —
`location_ping_ingestion` / `record_location_ping` and `location_data_retention_purge` /
`purge_expired_location_data`, both in `crates/compliance/adapter-postgres/src/lib.rs` — and the test is
`backend/ci/gates/audit-coverage/tests/gate_detects_violation.rs:26`
`fn allowed_exclusion_set_is_the_two_location_carveouts()` asserting `exclusions.len() == 2`. **Cite the
gate and the test name, not `ADR-0002:20`** — the ADR is prose about code and is wrong here. Record D3's
two 0207+ migration rows the plan prices at zero (`object_types` for `work`, `link_types` for
`work_artifact`), that T7 adds **zero** exclusions, and that extending `is_handler_surface` to path
component `app` has **UNMEASURED** blast radius (X-T7a first). Do **not** list `attach_object_policy` as an
unaudited path — see KILLED K13.

*Evidence: A3, C4, 18b, K12.*

### 4.5 §5.11 rows G3-G8, and W10

| Row | Action |
|---|---|
| **G3** | Re-scope as **N3 (ADR-0033)**, non-amending, `related: [ADR-0018, ADR-0023]`. **Strike "zero ADR hits"** — `ADR-0023:81-82` decides arbitrary approval-line DAGs and the 검토/승인/합의/참조 vocabulary, so this is a delta on ADR-0023's Engine-Gen, not greenfield. Record N3 as **NOT blocking Slice 0** — this is the theme where corrected evidence *removes* work from the critical path. Acceptance condition: any migration first introducing `delegation_rule` must, in the same file, add the delegation-transitivity arm to `backend/crates/governance/adapter-postgres/src/lib.rs:585-604` and a `CHECK (delegator_id <> delegate_id)`. `related` needs no reciprocal key (`check-adrs.mjs:23-27`), though each named record's `related` list gains the new id in the same atomic commit. |
| **G4** | → **N4 (ADR-0034)**. Carry R12's non-foreclosure constraints for the deferred lineage: (a) quantity-bearing or lineage edges may never live in `object_links` (`0102:68` permits one edge per `(org, src, dst, link_type)`; `:86` grants no UPDATE); (b) a `TRANSFER` movement carries its from/to pair on **one** row; (c) no lineage ADR is accepted until the N-into-1 merge names its serialization point and lock order, since every shipped precedent locks exactly one row. |
| **G5** | → **N5 (ADR-0035)**, with the Slice-0 prerequisites recorded in §5.5 (wave 5). |
| **G6** | **STRUCK.** Reason: out-of-scope is **silence, not prohibition** (`docs/decisions/README.md:7`), so no accepted ADR defers the canvas and there is **no charter clause** to amend or satisfy. `ADR-0023:148` is the header *"Follow-ups (named out of scope for this program)"*; the canvas bullet at `:153-154` carries no charter clause; *"enters as its own charter"* is at `:156` on the Contract→Position(인원편성)→PolicyPreset bullet. So the canvas's exclusion from slices 0/1 is **this plan's own scope choice**, standing on its own merits. Keep *"do not smuggle it in"* as a scope statement. Fix the §4.7/§4.8 anchor `ADR-0023:154-155` → `:153-154`. Mark **W10 deferred-by-follow-up and off the slice-0/1 critical path** — **not** gated on a charter (KILLED 20a). Optionally reference **N2 (ADR-0036)** as a non-amending record, not a charter. |
| **G7** | **STRUCK**, not "aligned as written", and on the **structural** ground: DN-0003 is `kind: design-note`, `authority: subordinate`, and cannot take a reciprocal ADR pair at all — `README:26` governs ADR relationship keys while design notes declare `parent_adr`. Fix the invariant-10 anchor `:98-100` → `:97-99`. The header's *"(G7 needs none)"* stays true. |
| **G8** | Stays **argue-only, no record**. Drop the lot CHECK from its three examples, leaving two (the shipped voucher balance gate, and the authority fold in a definer) — see wave 5 item 5.4. Note `ADR-0001:23` is a **Consequences** bullet, not the Decision, so no ADR question is engaged. |
| **D4** | Add a row: **D4 = two records, ADR-0030 (amends ADR-0025) and ADR-0031 (amends ADR-0009)**, owner decisions **captured** 2026-07-30 in `docs/ideas/d4-frontend-charter.md`, **blocked on acceptance**. Pre-acceptance each carries `status: proposed`, `doc_status: review`, `proposes_amendments_to: [...]` and **may not declare active `amends`** (`README:26`). State that the record is owed independently of whether this plan is approved, that D4 is **NOT on Slice 0's path**, and that `ADR-0025:133` clause 1 (a reachable mounted body for every exposed navigation state) survives unamended. |

*Evidence: A16, A15, 20b, A24, A23, A17, C-4.*

### 4.6 §5.11 — reciprocation mechanics, SoD, and §1 principle 2

**(a)** Add the item no ordered list covered: for **each surviving pair**, name the counterpart record, the
**exact line amended**, and the **relationship key to be added on BOTH sides** — including that **ADR-0003
carries no `amended_by` key today**, so reciprocation must **create** it (same for ADR-0002, ADR-0025,
ADR-0009, ADR-0022). `README:9`: *"A later number does not win automatically. Amendment or supersession
must be explicit in both records."* `README:26`: relationship keys *"must be reciprocal where
applicable."* State that the pair list is **shorter than §5.11's table implies** — G6 and G7 struck, G1
withdrawn.

**(b)** Add one row deciding **segregation of duties** in or out, citing `docs/specs/cedar-pbac-authorization.md:122` either way (*"Segregation of duties and self-approval checks are PBAC conditions, not UI-only rules."* — verified verbatim; a grep for segregation/toxic/mutual/self-approval/conflict-of-interest over all 1,853 plan lines returns **zero** hits). If **in**: name it a **grant-authoring-time** constraint — conflict pairs over `Feature`, evaluated where the `gov_approvals` four-eyes check already runs — with a widening and a probe whose known-bad control is a fold that accumulates a conflicting pair silently. If **out**: state it as a recorded cost with all three citations (`cedar-pbac-authorization.md:122`, `docs/ideas/no-code-operational-logic.md:211`, `docs/ideas/operations-intelligence.md:170`), so it reads as a choice rather than a silent contradiction.

**(c)** §1 principle 2: note that **additive-only does not forbid an authoring-time exclusion**.

*Evidence: 34-m4, B12.*

---

# WAVE 5 — MAJOR: the deferred-decision paragraphs

Six items. Each names a decision the plan currently leaves implied.

### 5.1 §5.5 E — Economics (plus §0.8, §5.7, §4.0.1, §9 closing, Slice-0 voucher row)

**(a)** Downgrade the claim to **COST-as-a-query** at all three sites — §4.0.1's economics row
(*"cost/revenue/profit as queries"*), §5.5's *"cost/revenue/profit are queries, not stored fields"*, and
§9's closing *"Cost, revenue and profit are queries, never stored fields"* — and name the **three shipped
parallel money stores** as the peer plan's reconciliation backlog: `equipment_cost_ledger` (`0015:45-58`),
`equipment_3r_dispositions.cost_minor` / `sale_amount_minor` (`0182:96-97`), and
`equipment_3r_rental_cases.monthly_rate_minor` (`0182:33`) — the last two with no period-lock guard. The
plan's *"Two records of the same money diverge; that is a certainty, not a risk"* is true and has already
happened three times; say so.

**(b)** Record **N5's blocking prerequisites for Slice 0**: `accounting_date DATE NOT NULL` distinct from
`posted_at`; a **line-level `branch_id`**; and an
`assert_period_open(tx, PeriodLockDomain::Accounting, accounting_date)` caller — while
`backend/crates/finance-gl/rest/src/lib.rs:28` is
`const VOUCHER_FEATURE: Feature = Feature::PeriodLockManage;`.

**(c)** Demote *"additive DDL on a table with no production data claim"* to an **assertion**, and add
**V-1**: the DDL is **irreversible** once landed — `0160_create_finance_gl_vouchers.sql:21` is
`-- console-gate: audited-table finance_gl_vouchers`, and `discover_audited_tables`
(`backend/ci/gates/migration-safety/src/lib.rs:174-187`) folds that marker into the same audited set as the
five built-ins, so DROP COLUMN on the voucher is a gate violation.

**(d)** Correct **"four non-test" period-lock sites → FIVE**, listing
`orgchange/adapter-postgres/src/lib.rs:611` **and** `:744` separately (the others:
`financial/adapter-postgres/src/lib.rs:1254`, `workflow/adapter-postgres/src/lib.rs:792`,
`backend/app/src/hr.rs:1706`). The substantive point — `finance-gl` is not among them — is unchanged and is
**why the count must be right**: a lane that greps and finds five stops trusting the paragraph.

**(e)** Decide in **one** place whether **확정 requires an open period**, and state that a compensating
voucher posts with an `accounting_date` in the current **OPEN** period while referencing the original's
date and id. Add the locked-period 반려 probe. Without this, **W14 is self-contradictory for its own
case** — the `assert_period_open` guard W14 adds would refuse the compensating posting W14 exists to
prove.

**(f)** State whether **one voucher line may be reported against more than one object**. If not, record
real-versus-statistical assignment and percentage distribution as decisions the **peer finance plan** owns
(§5.5's own must-not-foreclose list demands *"allocation with a recorded basis"*). Add one sentence to the
Slice-0 voucher row: **the single posted voucher is not evidence the dimension shape is settled.**

**(g)** Note that the `economics_is_a_view` probe depends on `GROUP BY account_code` being reproducible,
which **X-T9b predicts it is not** (`'100'` vs `' 100'`; `0160:62` rejects blank but stores untrimmed), and
that free-text-vs-account-master is an open owner question.

**(h)** **Migration count, all three sites**: §0.8's *"Zero `CREATE MATERIALIZED VIEW` in all 206
migrations"*, §5.5's *"Verified absent from all 206 migrations"* and §5.7's *"zero materialized views in
206 migrations"* → **"all 205 `.sql` migrations in the main checkout as of `<commit sha>`"**, naming the
commit. Verified: `ls backend/crates/platform/db/migrations/*.sql | wc -l` = **205**, highest
`0205_ont_policy_api_attach_writer.sql`; the 206th directory entry is `BUCK`. State that reservations start
at **0207** (0205 landed, 0206 in flight in lane-1). Cite `docs/ideas/fanout-plan-DRAFT.md:243` as the
standing rule being obeyed — its own worked example is *"Migration count … simultaneously wrong in three
planning docs"*. The claims themselves survive unchanged; only the count moves.

*Evidence: A14, 24a, 24b, 25, 3a, C1, X4b-7, A22, V-1.*

### 5.2 §5.6 F — Realtime authority propagation

**(a)** **Delete row 1** of the propagation table

> | Compute the fold on demand, or materialise? | **Materialise per (party, scope), keyed on `policy_versions.version`** (`0065:177-181`) … |

Keep the on-demand fold. It contradicts row 5 of the same table (*"per request, never across requests —
`crossRequestAllowDecisionCache: false`"*), §4.6's own text, and `ADR-0021:55-56`, which a plan cannot
supersede (`README:4`). It is also mis-keyed: `policy_versions` is `PRIMARY KEY (org_id)`
(`0065:177-181`), so keying there makes one grant edit invalidate every connected client in a
10k-employee tenant. This is a **one-row deletion, not a G-pair** — no ADR is engaged; N1 (ADR-0032)
records the mechanism.

**(b)** Re-key invalidation to **per `(org, user)`** and carry **both** counters: assignment writes bump
the subject counter (`identity/adapter-postgres/src/lib.rs:304`, `:672`, `:1606`) while role-definition and
role-status edits bump only the org counter (`:1284`, `:1369`) — keyed on either alone, a whole class of
authority change pings nobody.

**(c)** Add a row: **a grant revision bumps `authz_subject_version` for the subject party's users AND
`policy_versions` for the org**, citing ADR-0021 decision 5's *"Role, assignment, **responsibility**,
employment state, branch/team, or credential changes synchronously bump subject/policy versions so stale
subject material cannot keep granting access"* (verified verbatim). Note that per-org invalidation alone is
strictly coarser — any grant change invalidates every party's fold in that org, a cost X6 does not measure.
Add §7 probe `grant_write_bumps_subject_version`; known-bad control: a grant write that bumps only
`policy_versions`.

*Evidence: A13, B7.*

### 5.3 §4.6 How this ties into the engine

**(a)** After the two-new-subject-attributes bullet add: **each of the three new attributes must be
declared in the bundle schema** (`Schema::from_str(schema_src)`,
`backend/crates/platform/authz/src/cedar_pbac/engine.rs:306`) **before any code reads the fold** —
`Entities::from_entities([subject, resource_entity], Some(&bundle.schema))` (`:449`) validates, so an
undeclared attribute fails entity construction and **denies everything**. §4.6 today states the
bundle-schema requirement only in the hierarchy bullet, not for `capabilities: Set<String>`,
`scopes: Set<String>` or the decision-scope resource attribute. Record it in §8 as a **hard Phase-3
ordering constraint**: platform/authz's schema change lands first.

**(b)** Add the corrected substrate finding so the plan is not later read as understating what ships: two
**type-agnostic declarative systems are executable code** — `sync_property_links_tx`
(`backend/crates/ontology/adapter-postgres/src/instances.rs:874`, called `:723`, `:836`) and
`resolve_derived_attributes_tx` (`:1142`, called `:681`, `:769`) — and `0165:1024-1041` has plpgsql itself
INSERT a generic `create` action on publish. Extensibility is **open in the entity dimension and closed in
the verb dimension**, which is DN-0003 invariant 10 already implemented; the cheap axis is widening the
`derive` op set at `instances.rs:1166`.

*Evidence: B7, A21.*

### 5.4 §5.8 H — Quantity-bearing lineage

**(a)** Keep the CHECK; **add the mechanism the precedent actually uses.** Delete both sentences

> it makes conservation unviolatable without any procedural code

and

> a definer is needed when an invariant spans sibling rows, and putting before/split/after on one row removes the span. The shipped `0156:103` CHECK is the cheaper and stronger answer.

Two concurrent splits of a 100-unit lot both written (100, 60, 40) satisfy the row CHECK and over-allocate
by 20. The split write **locks the parent lot row FOR UPDATE inside the action's transaction** (precedent:
`backend/crates/inventory/adapter-postgres/src/lib.rs:394` `fetch_item_for_update_tx`, on top of
`lock_consumption_idempotency_key_tx` at `:376` and a domain `state.consume(quantity)` at `:411`), derives
`parent_qty_before_milli` from the **locked row** and never from the request, and updates
`lot.quantity_milli` in the same transaction. Add probe `lot_concurrent_split_cannot_overallocate` with the
row-CHECK-only implementation as its known-bad control. State whether `lot.quantity_milli` is
**authoritative or derived**.

**(b)** State the invariant honestly: a **per-row CHECK plus a whole-tree aggregate** that is not a row
CHECK and is therefore not unviolatable — §5.8's own down-traversal states it as an aggregate
(*"sum(leaf lots) + sum(scrap lots) must equal it"*) nineteen lines after calling it unviolatable.

**(c)** Fix `scrap_quantity` at **all three sites**: `output_quantity` / `scrap_quantity` are columns of
**`production_operations`** (`CREATE TABLE` at `0173:75`), not `production_plans`, which has only
`quantity` (`0173:50`). The third site is §5.8's dimension paragraph citing *"`production_plans` scrap
(`0173:81-82`)"*.

**(d)** Use one spelling of the split table (wave 2 item 2.1(e) chose `lot_split`).

*Evidence: B8, A23, A22.*

### 5.5 §5.9 I + §4.1 effective-dating + §2 driver 2 + Slice 1

**(a)** Decide the **correction axis** explicitly, before Slice 1, in both §4.1's effective-dating text and
§5.9: either a correcting revision carrying `corrects_revision_id` plus a knowledge-time argument on as-of
reads (the bi-temporal entry-date axis), **or** a stated deferral naming the consequence — that between
error and discovery the fold returns the wrong value for that period and **cannot be repaired in place**.
Verified: `0155_create_ontology_instances.sql:112-160` `ont_instance_revisions_append_only()` raises on any
DELETE, raises if `OLD.valid_to IS NOT NULL`, and raises unless the **only** changed column is `valid_to` —
`attributes`, `valid_from`, `version`, `prev_hash`, `row_hash` all pinned; with
`CHECK (valid_to IS NULL OR valid_to > valid_from)` and `idx_ont_instance_revisions_one_open`, an erroneous
revision cannot be rewritten, cannot be closed at a zero-length interval, and a new revision at the same
`valid_from` overlaps. The plan's only correction concept today is the post-확정 반려 compensating
`correction` revision — a different concern. Add a probe whose known-bad control is a correction that
silently rewrites history. Qualify **§2 driver 2** (*"replayable is free, not built"*) accordingly.

**(b)** When Slice 1 defines `assignment`, add an **assignment kind** (substantive / acting / seconded) and
a **return-right marker** as authored properties. 육아휴직 복직 is statutory and HR+payroll is the first
vertical, so it lands on this entity either way — as a property now or as a reshape of a shipped type
later. (휴직 and 복직 occur zero times in the plan; `holds_position` is already ManyMany so a substitute's
concurrent assignment is expressible.)

*Evidence: B11, 31.*

### 5.6 §4.0 and §4.0.2 — the component boundary

**(a)** **Delete** the §4.0 sentence

> A new entity class declares its components; the systems light up for it without anyone hand-writing an integration per concern.

Keep §4.0.2's *"**So 'manageable without developers' is true for the dimension side and false for the
component side.** Declaring a new *type* is authored; giving it a *new concern* is code."* (Conflict C-2.)

**(b)** Add two rows to §4.0.2's requires-code column:

- **actions on a projected type** — code: one `ProjectedDispatchRegistry` handler per action
  (`backend/crates/ontology/rest/src/lib.rs:160-195`: `pub struct ProjectedDispatchRegistry { handlers: HashMap<String, ProjectedHandler> }`,
  `register(target, handler)` chainable builder, `dispatch` returning `ActionError::NotWiredYet`),
  registered in the App composition root; unwired = `NotWiredYet`.
- **the authoring-action vocabulary** — five elements, closed:
  `backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252` `AUTHORING_ACTIONS` = view, edit,
  read_field, `console:configure`, `console:deploy`.

**(c)** Give `docs/specs/ecosystem-entity-components.tsv` a **handler count** for `work`'s Slice-0 / W4 /
W11 / W13 actions, so the Phase-3 `app` crate row is **sized** rather than labelled "wiring".

**(d)** Add one paragraph stating how **DN-0003 invariant 1** (*"not the NORMAL operational write path"*)
is satisfied for Tier T/P before `work` is built — either every consequential `work` mutation is an
`ont_action_types` dispatch, or it is a bounded exception naming the gate that holds it and the history it
forfeits, with a probe and a known-bad control.

*Evidence: A21, B6, 19.*

---

# WAVE 6 — MAJOR: anchors and counted facts (mechanical sweep)

Five items. Nothing in this wave changes a claim; every claim survives with a checkable anchor.

### 6.1 §0.1 and the preamble

Re-anchor §0.1 with **quoted sentences plus section headings, never line numbers**. Drop `:89-92`,
`:545-546`, `:575-579`, `:83-87`. Do **not** substitute the review's `:116` / `:571` / `:606` — already
stale. Use:

- under **`## Where employees belong`**: *"This revises the earlier 'group is the tenancy boundary for
  people' answer."* … *"The group is not high enough … Group-scoping relocates the duplication rather than
  removing it"*;
- under **`## Recommended Direction`**: *"**People are group-scoped.** Per the owner's choice, the group is
  the tenancy boundary for people"*;
- under **`## The two hard problems`**: *"This is the largest single engineering cost in the chosen
  model"*;
- the body's own conclusion: *"The person belongs to the platform; the tenant owns the edge"*.

Sweep **every other cross-document citation into `docs/ideas/authority-and-approval-model.md`** to the same
quoted form. Soften the preamble's *"Line numbers re-verified this session"* to name which citations are
quoted-text anchors and which are line numbers. Update §9's closing follow-up — *"`docs/ideas/authority-and-approval-model.md` should be marked SUPERSEDED … or have §0.1's contradiction corrected in place"* — to acknowledge that the **upstream correction block already exists** at the top of that file (it quotes the sentences instead of citing lines, and adds its own citation warning: *"Adding this header shifted every body line by ~30 … the claims were true, the anchors were not"*).

*Evidence: B10, B10-A1, B10-A2; KILLED B10-b.*

### 6.2 §0.3

Replace `authority-and-approval-model.md:125` (a **blank line**) with the quoted sentence *"Cedar expresses
this natively through entity `parents`, so the corporate graph **becomes** the Cedar hierarchy"*,
attributed to the document with no line number. **Leave** `engine.rs:392`, `:425`, `:449` as line numbers —
unmodified source.

*Evidence: B10-A3.*

### 6.3 §0.12 and §0.13

**§0.12** — apply the reachability wording (wave 2 item 2.1(f) states the rule; §0.12's heading and body
must match). Add the two findings §0.12 lacks: (a) `to_object_type_id` appears **zero** times in the whole
write module, so it is **decoration** — setting it correctly buys nothing and setting it wrongly costs
nothing; say so or an implementer will trust it; (b) `validate_draft` is **confirmed absent** —
`backend/crates/ontology/adapter-postgres/src/lib.rs:1142-1151` is the entire link-type validation and
checks duplicate `stable_key` only — so §7's `link_type_alone_is_rejected` is **observed RED today**, which
is the record the plan's own discipline requires before the guard lands.

**§0.13** — three repairs: the registered path list runs to `backend/crates/ontology/rest/src/lib.rs:228`
and holds **14** paths, not the `:213-217` cited; name the attach path exercised over HTTP —
`POST /api/v1/ontology/object-types/{stable_key}/policies` (`OBJECT_TYPE_POLICIES_PATH`), backed by the
audited definer in `0205_ont_policy_api_attach_writer.sql`; and add the sharper consequence X2 measured —
an unpoliced entity is not "visible but unfiltered", it is **absent**: a row the list hides is `404` by id,
**deliberately**, so a 403 is not an existence oracle
(`ontology/rest/tests/object_policy_attach_as_runtime_role.rs:133-141`). Restate §0.13's consequence
sentence as *"ships with a `view` permit attached — which is all an authored object policy can express"*.

*Evidence: X1-1, X1-2, 19.*

### 6.4 The three ADRs that gain `amended_by` — cite by heading and clause

At all **nine** occurrences (§0.16 ×2, §5.11 G1/G2/G2b/C4b, §8 Phase 7, §9), replace `ADR-000N:NN` with
`## Decision` heading plus the quoted clause, because each of these three files gains an `amended_by`
frontmatter key in the same commit that lands D1/D2/D3 — which shifts line 20 and lines 25/33-39:

- `ADR-0003` Decision — *"`All` for SUPER_ADMIN/EXECUTIVE rollups"*;
- `ADR-0002` Decision — *"its exclusion set contains exactly one entry"* (quoted as the **false** sentence
  D3 corrects, per wave 4 item 4.4);
- `ADR-0022` Context and Decision — *"Do not ship a speculative external IdP seam."*

**Leave as line numbers** (no amendment pair targets them): `ADR-0001:23`, `ADR-0018:115-116` / `:231-233`,
`ADR-0023:79-86` / `:148-155` / `:158`.

*Evidence: C9.*

### 6.5 Citation form — path-qualify, and fix the remaining anchors

- **Path-qualify all 19 bare basenames on first use** (`lib.rs:` is the worst — dozens of files match):
  `instances.rs`×5, `engine.rs`×6, `residual.rs`×2, `store.rs`×2, `company_conformance.rs`×1,
  `fixtures/employment.rs`×2, `lib.rs`×1. Bare `:NNN` continuations within the same paragraph are fine.
  Resolutions needed: `docs/specs/` holds `rbac-configurable.md`, `korean-legal-boundaries.md`,
  `org-editor-primitives-ux.md`, `org-hierarchy.md`, `foundation-gates.md`; `docs/ideas/` holds
  `authority-and-approval-model.md`, `no-code-ontology.md`, `governed-object-engine-PLAN.md`;
  `README.md:12` is `docs/decisions/README.md:12`; `Cargo.toml:53` is `backend/Cargo.toml:53`.
- Remaining anchor fixes: `0153:75` → `:76` (done in wave 1 item 1.4 — verify only);
  `ADR-0023:154-155` → `:153-154`; DN-0003 invariant 10 `:98-100` → `:97-99`; `0076:49-50` → `0075:6,13`
  at the §3.2 Option 4 site (§9's was fixed in wave 2).
- **Do NOT** cite `docs/ideas/fanout-plan-DRAFT.md:243` as the authority for the *anchor-discipline* rule —
  `:243` is about restating derived facts, which is the migration-count rule, not this one. State the
  anchor rule on its own merits (KILLED 2b).
- `docs/decisions/README.md:12` needs no protective quoting: the `## Current index` table starts at line
  30, **below** it, so adding index rows cannot shift it (KILLED KR-4).

*Evidence: C10, 2a, A22, A15-anchor, A24-anchor.*

---

# WAVE 7 — MAJOR / MINOR: vocabulary, ergonomics, and the remaining recorded costs

Six items.

### 7.1 §4.4 — the three remaining rows

**(a) `policy_role_conditions`.** Narrow the claim: the authored vocabulary must be narrowed **at the write
path** to what the resolver evaluates — **`{branch, team}` × `{equals, in}`** — with a test asserting
write-accepted ⊆ resolver-evaluated, while the DB CHECK stays permissive as the additive extension point.
Verified: `backend/crates/platform/authz/src/lib.rs:1404-1430` returns `None` unless the operator is
`equals`|`in` **and** the attribute is `branch`|`team`, and the caller at `:1350-1360` does
`else { continue; }` — dropping the **whole role**. Record that the fail-closed whole-role void is
**CORRECT and must never be relaxed** into per-condition ignoring, that the contrary comment at
`0065:101-103` is struck, and that **competence** is a subject-side condition attribute in the shape the
`"team"` arm already has (`authz/src/lib.rs:1421-1425`) — not a third relation and not a scope-type change.
Note the read-only **inert-condition census (X-T2f)** must precede shipping the narrowed write path. Do
**not** propose per-condition ignoring — the plan never made that proposal (KILLED K8).

**(b) Attribute vocabulary coverage.** The vocabulary covers **two of the four dimensions**: `0065:110-127`
holds 17 attribute literals and **직무 and 직급 are not among them**, so they have **no substrate** and
widening the CHECK is a migration — a third closed vocabulary. Correct the row's *"The predicate
vocabulary the plan needs **already exists as data**"*. Then either add 직무/직급 to §4.1 with their
attribute keys, or state which dimensions have no substrate in slices 0/1 and in which widening they
arrive (they appear exactly once in the whole plan, at §1 principle 3).

**(c) `notices` — a fourth gap.** Add **per-recipient audience targeting** with its DDL (an explicit
recipient list keyed by party). W1's party-keyed recipient fixes the cross-org FK but **not** the org-wide
fan-out: `backend/crates/notices/adapter-postgres/src/lib.rs:413-433` publishes via two SQL variants keyed
on `audience_scope == "branches"`, and **both** end
`WHERE u.org_id = $1 AND u.is_active = true` (the branch variant joins `user_branches` /
`notice_audience_branches`, still org-wide within the branch). Until fixed, a 반려 notice on a 결재 matter
reaches **every active user in the org** — a confidentiality regression, not a missing feature. Make
`obligation_notifies_line_as_raised` assert that **non-members receive nothing**, with the shipped org-wide
snapshot as its known-bad control.

*Evidence: A19, 19, 22.*

### 7.2 §4.1 — the vocabulary paragraph

**(a)** Either list **all fourteen** primitives including **"PolicyRole hook"**, or say *"thirteen of the
fourteen; the PolicyRole hook is §5.3's `Feature` work, not an entity here"*.
`docs/specs/org-editor-primitives-ux.md:468` names fourteen: Group/HQ, Organization, OrgUnit,
Worksite/Cell, Person, Employee, User, Position, **PolicyRole hook**, ReportingLine, EmploymentAssignment,
CrossOrgAssignment, SetupDraft, Audit. The dropped primitive is the one carrying the plan's own quoted
separation (*"a Position is not automatically an access Role"*), so dropping it silently reads as an
oversight exactly where principle 3 is weakest. (The plan's `:25` and `:256` citations into that spec are
both CORRECT — leave them.)

**(b)** Add **`reporting_line`** (position→position, with the spec's cycle and single-primary-path
validation) as a Tier N row and a §4.3 row, **or** state its exclusion and defend it. Today `ReportingLine`
occurs only in the vocabulary sentence and appears in no Tier N row and no §4.3 row.

**(c)** Give the plan's **직책 type a non-colliding `stable_key`**:
`backend/crates/ontology/adapter-postgres/src/seed.rs:74` is
`pub const POSITION_KEY: &str = "position";`, so the plan's Tier N type would collide with a seeded
built-in. Record the mapping across {org-editor "Position", built-in `position`, shipped `job_position`,
this plan's type} in `docs/specs/ecosystem-PORTING.md`. Add a Phase-7 item correcting
`docs/program/CATALOG.md:62-68` (OrgUnit / Position / Person / Employment / PayRun) to the shipped set:
company / org_unit / job_position / employment / pay_run — **Person never landed**.

*Evidence: 21b, 21.*

### 7.3 §4.7 The game-system lens

**(a)** Name the **regulation-centric renderability** differentiator, and add one probe: the complete
전결규정 (category × band × scope → competent unit, terminal?) renders as **one artefact as of an arbitrary
date**; known-bad control: routing expressed only inside approval templates. Frame it as **current-state
renderability** (`docs/ideas/research-sap.md:937-939`, *"as of today"*), **not** historical replay — the
"as of 2026-07-01" phrasing is an artefact of a mis-transcribed quote. Every `slice0_*` probe and E1-E6 is
person-centric; none renders the regulation.

**(b)** Point 2 restates the guild bank as *"(role × amount band × category) → permitted"* while its own
table row reads *"guild bank limit per rank per tab **per day**"* — the per-day limit is dropped, and
`delegation_rule` carries no periodic or cumulative quota in §4.1 or §4.3. Either **add a period /
cumulative quota** to `delegation_rule` or record dropping it with a reason.

**(c)** Qualify the superlative at *"Games have shipped this for 25 years with untrained users, which is
the strongest available evidence…"* → **"the strongest evidence cited here"**. The corpus that would rank
it is cited zero times.

*Evidence: 27, 28.*

### 7.4 §4.8 Ergonomics as acceptance criteria

**(a)** Give **E2** a widening with acceptance, and make its completeness test **executable**: one row per
§4.1 entity mapped to its character-sheet section, with a probe that fails on an unmapped entity. Today E2
is *"the completeness test: an entity with no home on this screen is a modelling smell"* and **no widening
ships it** (W17 ships E4, W18 ships E1, W11 ships E6), while §7's `every_entity_declares_its_components`
asserts rows in a TSV, not homes on a screen.

**(b)** Qualify **E4** so the fold simulator is **not** assumed to inherit Cedar simulation for domain
capabilities.

**(c)** Give §4.8 the **ergonomics criterion §4.7 promises** — §4.7 asserts *"That is a testable bar, not a
sentiment (§4.8)"* and E1-E6 contain no such test.

*Evidence: 30, 19, 27.*

### 7.5 §1 principles 3 and 4

**(a)** Restate principle 4 as **"one of the four CI-enforced tiers, optionally projected"**, since §3.1
adds *"A fifth path, **Tier P — projected**"* twenty-odd lines later and Phase 0's PORTING.md is meant to
be looked up, not re-derived.

**(b)** Principle 3 names 소속 / 직급·직책 / 직무 / 결재선 as vocabulary; cross-reference 7.1(b) so the
two dimensions with no substrate are visible where the principle is stated.

*Evidence: 29, 19.*

### 7.6 §5.4 D — PII

**(a)** Rewrite the recommendation to **price the loss** the way §5.7 prices its three, rather than
asserting *"Keeping `party` attribute-free permanently is the cheaper end state"* without the cost of the
alternative.

**(b)** Name the **six control ids** in the precondition list and state the bar **verbatim**: a qualified
Korea legal/compliance authority with an attributable **I2/I3 candidate-bound receipt**, and that a native
agent produces only **I1_NON_INDEPENDENT** evidence. The six, all `release_disposition: HOLD` in
`docs/program/console-jurisdiction-register.json`: CTRL-KR-PRIVACY-001, CTRL-KR-WORKFORCE-001,
CTRL-KR-SAFETY-001, CTRL-KR-FINANCE-001, CTRL-KR-LOCATION-001, CTRL-KR-RECORDS-001. Note
**CTRL-KR-RECORDS-001** (approvals, notices, documents, retention) and **CTRL-KR-WORKFORCE-001** (payroll)
are the two this plan touches. Quote `uncertainty_rule` (`:1186`): *"Missing, stale, conflicting, or
unqualified authority is HOLD; agents may not invent certainty."* This asserts **no** compliance conclusion
and proposes **no** unholding — it records what a later lane must not mistake for satisfiable. Replace the
plan's *"the jurisdiction binding and Korea controls have moved off HOLD"* with the named bar.

*Evidence: B3, 32.*

---

# KILLED — do NOT resurrect any of these

Carried verbatim from the triage. Several read perfectly plausible; each is premise-false, already-fixed,
out-of-scope, or superseded by an executed experiment. Stream-duplicated ids were renumbered (`KX-*` =
experiments stream, `KR-*` = research/citation stream); no content was changed.

- **B5a** — premise-false: B5(a) argues the first commit is inadmissible because the registry's `hold_rule` reaffirms an "empty Buck2 target sets" clause while `prelude/` is absent. Both halves fail. (1) `grep -rn "hold_rule" scripts/ backend/ tools/` returns NOTHING; the token exists only as data in `docs/program/console-capability-registry.json` (3 occurrences, `:7` and the `hold_rule_amendment` at `:7165-7168`). The executable HOLD constraints are elsewhere: `scripts/console/validate-console-truth-ledger.mjs:254` fails only when `delivery.rust_status === 'REQUIRED'` and buck targets are empty, `:255` requires `REQUIRED_UNRESOLVED` to stay HOLD, `:257` requires each declared target to resolve. `adr-adjudication.md` §5.5 names exactly this as the architect finding that fails: "The inadmissibility finding argues from a field nothing enforces." (2) The "prelude/ is absent so the buck graph is broken" premise is false — `.buckconfig` declares `[external_cells] prelude = bundled`, which supplies the prelude from inside the buck2 binary, and `tools/buck2` is a blake3-pinned DotSlash launcher (X8, `experiment-results.md`). Buck2 target sets are producible today. So there is no Buck2-clause amendment to write and no deadlock to escape. The surviving half of B5 is carried separately as B5b.
- **B10-b** — already-fixed: B10's second required edit — "fix the same three citations at `authority-and-approval-model.md:11`" — has no target. That file's SUPERSEDED header now quotes the sentences instead of citing line numbers: `:10` "**\"The group is not high enough.\"**", `:11-12` "**\"People are group-scoped. Per the owner's choice, the group is the tenancy boundary for people\"**", `:13` "**\"the largest single engineering cost in the chosen model.\"**", and `:16-20` adds its own citation warning ("Adding this header shifted every body line by ~30 … the claims were true, the anchors were not"). No line-number citations remain in the header to repair. The plan-side half of B10 is unfixed and survives.
- **3b** — already-fixed: Item 3 requires "reserve migration slots from 0207". The plan already does, in three places: Phase 0's `ecosystem-PORTING.md` row ("migrations start at 0207"), Phase 3 crate 1 ("migrations 0207+: `party`, `party_org_visibility`, `work`, …"), and Phase 7 rung ② ("migration slots 0207+"). Nothing to change. The per-LANE allocation is a different requirement and survives as item 15.
- **2b** — citation-unresolvable: Items 2 and B10 both cite `fanout-plan-DRAFT.md:243` for "citations into a document you also edit are quoted-text anchors, not line numbers". `:243` actually reads: "**Restating derived facts in prose across multiple documents** | Migration count, `include_str!` line numbers, worktree count and crate count were **simultaneously wrong in three planning docs** — 11 corrections in §11. Docs drift faster than anyone re-reads them. **Recommend: generate these into the docs from the tree, or reference one source; never restate.**" That is item 3's authority, not item 2's. The anchor-discipline rule must be stated on its own merits or sourced elsewhere; do not propagate the misattribution into the plan.
- **16a** — superseded-by-experiment: Item 16 requires "give X8/X9 one shared runnable control (add a test file with a deliberately failing assertion, confirm CI goes RED, then fix it) and name the candidate mechanisms X8 must discriminate between (path filter, continue-on-error, no-op required job, cached graph)". X8 ran and answered: the mechanism is `[external_cells] prelude = bundled` (`.buckconfig:15-16`) with a blake3-pinned DotSlash launcher, and `ci.yml:192` runs a real `tools/buck2 test`. None of the four candidate mechanisms applies. `experiment-results.md` also records, correctly, that X8 is "an **investigation, not a probe**, so the 'proven RED on a known-bad input' discipline does not apply to it in the same form — there is no GREEN to distrust", and that the control listed for X8 is really X9's. X9's four-link trace is already worked end to end on `object_policy_attach_as_runtime_role`.
- **16c** — superseded-by-experiment: Item 16's "restate X4 … as constructed queries with expected-fail baselines" is done: `experiment-x4.md` records X4 CONFIRMED with 30 assertions and three controls RED, zero new GUCs, the 141 RLS policies untouched. X1 and X2 are likewise CONFIRMED with recorded outcomes (`experiment-x1-x2.md`). Only X5 still names a scenario instead of an input — carried as survivor 16g.
- **16d** — premise-false: Item 16 requires "no Slice-0 implementation commit until X1-X5 and X8-X9 have recorded outcomes". Circular for X3 and X5: `experiment-results.md`'s own table says they "need `effective_grants_for` to exist — these are slice-0 experiments, not prepwork" (X6 likewise follows X4). The gate as written forbids the commit the experiment requires. Restate as: X1, X2, X4, X4b, X8, X9 recorded before the first implementation commit; X3, X5, X6 recorded before Slice 0 may be declared green.
- **17b** — premise-false: Item 17 says `LANE-PROTOCOL.md:268-270` "claims no `[profile]` section and no sccache; both landed". Half false: `ls .cargo/` → "No such file or directory", and `console-program-ledger.md:675` records that sccache is set in the subprocess environment and workflow prompt, "deliberately not in `.cargo/config.toml`, which would apply in CI where no runner has sccache and every Rust job would fail". Only `[profile]` landed (`backend/Cargo.toml:359`, `:362`). Applying item 17 verbatim would write a false correction into the plan and invite a lane to create the file that must not exist. Corrected form carried as survivor 17c.
- **20a** — premise-false: Item 20 requires marking W10 "gated on the G6 charter". There is no charter clause: ADR-0023's canvas bullet at `:153-154` carries none, and "enters as its own charter" is at `:156` on the Contract→Position(인원편성)→PolicyPreset bullet. The adjudication STRUCK G6 on exactly this ground ("the charter clause does not exist"). Under `README.md:7` an accepted ADR is authoritative only within its stated scope, so a follow-up list is silence, not prohibition, and no amendment is owed. The §5.11-vs-W10 contradiction is real and survives as 20b; the gate mechanism named in the item does not exist.
- **K1-draft-G1-as-scoped** — premise-false: Any revision item that says "correct G1's ADR-0022 reading, then draft it as scoped" dies: there is no incumbent clause to amend. adr-adjudication §5.4.1 and R1 — plan §5.11's G1 grounds on "`ADR-0022:25,33-39` decides identity is local/org-scoped", but `:25` is Context, the Decision is `:31-39`, and the string "org-scoped" appears nowhere in ADR-0022. Its deliverable claim ("one durable identity per natural or legal person, across every tenant and vertical") is undeliverable without the PII matching the plan itself rejects, and README:7 makes an accepted ADR authoritative in scope. Replacement: D1, a narrowing amendment to ADR-0022. Reopens only if a DIFFERENT ADR clause is identified as the incumbent.
- **K2-new-signature-store** — premise-false: Every item arguing that `gov_approvals` cannot hold a 결재 line (and therefore that a new signature store is needed) dies. Verified by direct read: `backend/crates/orgchange/adapter-postgres/src/lib.rs:1479-1483` INSERTs `gov_approvals` binding `request_ref` to `step_id` and `requested_by` to `request.drafted_by`, so an N-step request writes N immutable rows; `0153` line 76 is `UNIQUE (org_id, request_ref)` — one signature per NODE, not per request. An N-node line ships and runs the newest approval domain in the repo. The plan copied the migration's own inline comment "-- one decision per request" instead of reading the caller, which is how `approval_signature` (R7) got invented.
- **K3-two-employer-passkey** — premise-false: "One physical passkey cannot belong to a person who works at two companies" — already shipped, inverted twice. `0004_create_auth.sql:7` is UNIQUE per CREDENTIAL; `auth/src/webauthn.rs:349-353` passes the per-org `users.id` as the WebAuthn user handle and `:339-342` builds `exclude_credentials` from that user's own passkeys only, so one device against two handles yields two credential ids and nothing is rejected. Login is usernameless/discoverable with an EMPTY allowCredentials list (`:786-796`) resolved by a deterministic `LIMIT 1` over a UNIQUE column (`0038:79-80`) — not arbitrary. The passkey choice IS the org choice and the account chooser the product thesis asks for is already shipped (adjudication §5.1.1). No plan edit, no mechanism to design.
- **K4-inbound-email-fk-columns** — premise-false: "Inbound email links to work through two hardcoded nullable FK columns" — the columns are DEAD. Verified by grep over `backend/`, `scripts/` and `docs/specs/`: `linked_work_order_id` appears only in `0053_create_comms_webmail.sql:94`, its partial index at `:196`, one `scripts/dev-seed.sql` row, and a copy under `backend/target/`. Zero `.rs`, zero `openapi.yaml`, zero frontend references (adjudication §5.1.3). Nothing links through them, so nothing needs DROP COLUMN and the migration-safety gate is never engaged. Any revision item pricing a column removal here dies.
- **K5-one-of-seven-types-wired** — premise-false: "Only 1 of 7 ontology types is wired" is wrong in both numerator and denominator, and there is no plan text to correct. Verified: `backend/crates/ontology/adapter-postgres/src/seed.rs` contains 15 `projected_draft(` call sites and 25 `stable_key` occurrences (27 types = 15 projected + 12 instance), and `dispatch_target` occurs exactly ONCE in the whole seed. The honest figure is the no-code-reachable DOMAIN-WRITE count: 0 of 15. Separately verified: `grep "1 of 7"` over both `ecosystem-plan-DRAFT.md` and `ecosystem-plan-review.md` returns zero hits, so the number lives only in the unadjudicated raw collisions and must not be imported into the plan.
- **K6-G6-charter-premise** — premise-false: "ADR-0023 defers the no-code canvas and requires a named charter" — adjudication §5.1.12: `:148` is the header "Follow-ups (named out of scope for this program)", `:152` is an unrelated Audit-SSE bullet, the canvas bullet at `:153-154` contains no charter clause at all, and "enters as its own charter" is at `:156` on a different bullet. Under README:7 out-of-scope is silence, not prohibition. This voids G6's amendment premise entirely — there is no clause to amend and no charter to satisfy, so any item proposing "propose the charter" or "accept the ADR-imposed deferral" dies. Verified no downstream plan section depends on the struck slot.
- **K7-DN0003-reciprocal-pair** — out-of-scope: G7 as a reciprocal ADR pair on DN-0003 is structurally impossible, not merely unnecessary. DN-0003 is `kind: design-note`, `authority: subordinate`; README:26 governs ADR relationship keys and design notes declare `parent_adr` (adjudication §8.1). Verified independently that reciprocity in CI covers only ADR-to-ADR pairs: `scripts/check-adrs.mjs:23-27` lists exactly `amends/amended_by` and `supersedes/superseded_by` over the ADR map. Any item drafting or reviewing a DN-0003 amendment pair dies; only the anchor fix (`:98-100` → `:97-99`) and the restated reason survive.
- **K8-strike-per-condition-ignoring** — premise-false: R5 attributes the per-condition-ignoring proposal to "the plan", but the plan never makes it — so there is nothing to strike. Verified in the plan: §4.4's `policy_role_conditions` row concludes "**Reuse the attribute vocabulary; never its operator set.**", and pre-mortem Scenario 2 argues at length AGAINST narrowing conditions ("the moment one condition narrows rather than adds … delegation starts removing authority"). The proposal lives in `0065:101-103`'s comment and in the collision lenses. What survives is the opposite-direction edit, captured as survivor A19: narrow the WRITE path to the `{branch, team}` × `{equals, in}` the resolver actually evaluates (verified `authz/src/lib.rs:1404-1430` returns `None` on anything else, and the caller at `:1350-1360` drops the whole role).
- **K9-hold-rule-buck-rung** — premise-false: The review's revision item 14 ("add the `hold_rule` Buck2-clause rung, quoting the existing amendment's `limits[0]`") argues from a field nothing enforces. Verified: `grep -rn "hold_rule" scripts/ backend/ tools/` returns ZERO; the token appears only as data in `docs/program/console-capability-registry.json` and under `docs/evidence/`. The executable HOLD constraints are `scripts/console/validate-console-truth-ledger.mjs:255` (binding only capabilities at `rust_status === 'REQUIRED_UNRESOLVED'`) and the jurisdiction-loop fail on `control.release_disposition !== 'HOLD'`, both of which I read directly. The hold_rung half of item 14 dies; the capability-registry-rows half is untouched by this kill, and Korea stays HOLD either way.
- **K10-bun-fragmentation-rationale** — premise-false: The instruction to "fix anywhere the plan cites Bun for the task-fragmentation rationale" has no target. Verified: `grep -n "fragmentation"` over `ecosystem-plan-DRAFT.md` returns zero hits, and the only per-crate text is the §8 heading "Phase 3 — the work queue, by crate" and the prepwork row "CI wiring per crate" — neither offers a fragmentation rationale. The correct rule (crate = unit of agent assignment and review; file = unit of error bookkeeping within it) is not contradicted anywhere in the plan. The remaining research-stream corrections (the 60,624 figure is the Linux x64 count, and the cost figure omits 72B cached input reads) still apply and belong to that stream, not this one.
- **K11-delete-ADR-0021-slot** — premise-false: "Delete the reserved ADR-0021 G-slot for the realtime theme" — no such slot exists. Verified by reading §5.11's table row by row: it contains exactly G1, G2b, G8, G9, G2, G3, G4, G5, G6, G7, and ADR-0021 appears only INSIDE G2's row text ("`ADR-0021` decision 2 makes `org_id` the RLS boundary Cedar may not widen"). Adjudication §5.8 reaches the same conclusion. The correct action is to widen G2's stated scope (survivor A4), not to delete anything.
- **K12-one-exclusion-carried-forward** — premise-false: Any follow-up text that still treats the audit-coverage exclusion set as having exactly one entry dies here — adjudication §0.3 names T6's and T8's follow-up text as still saying "exactly one entry" and warns it must not be copied forward. The gate returns TWO: `backend/ci/gates/audit-coverage/src/lib.rs:90-107` returns `location_ping_ingestion` and `location_data_retention_purge`, and the test is `allowed_exclusion_set_is_the_two_location_carveouts` asserting `exclusions.len() == 2`. `ADR-0002:20`, the gate's own module doc at `:9-11`, `kernel/core/src/audit.rs:3` and the plan in two places all say "one" — the ADR Decision line is prose about code, and the code is authoritative.
- **K13-attach-object-policy-audit-worry** — already-fixed: `attach_object_policy` must be removed from the G9/D3 audit-coverage worry list: it IS audited. adjudication §5.9 — `with_audit::<_, Uuid, PgCedarError>(&self.pool, event, …)` at `backend/crates/platform/authz-rest/src/store.rs:231` wraps the definer call at `:234`, with the event built at `:217`. Any revision item asking the plan to enumerate it as an unaudited new write path dies.
- **K14-bridge-object-type-registries** — out-of-scope: Bridging the `object_types` and `ont_object_types` registries is DEFERRED and must not consume slot 0207 (adjudication R19). `backend/ci/gates/tenant-isolation/src/lib.rs:43-46`, `:53-59` allowlists `object_types` as a global no-RLS table precisely because "the kinds themselves are not tenant data", so a tenant-authored kind there is a cross-tenant NAME LEAK — the bridge is a confidentiality question, not a plumbing one. Non-foreclosure recorded instead: nullable `src_ont_object_type_id`/`dst_ont_object_type_id` on `object_links` plus a CHECK that exactly one endpoint form is set is additive, needing no FK change and no data migration. Likewise R20 defers converging the four shipped ways to point at a domain object; `work` must simply not gain a column that duplicates an edge, guarded by the plan's own `no_duplicated_fact` probe.
- **KX-1** — premise-false: "`prelude/` is missing so the buck2 graph is already broken" (§8 Phase 7) and every revision item premised on it — including any plan to execute `PIVOT-2026-07-28.md` §6's "cargo, not buck2" because the buck2 graph is unusable. `.buckconfig:15-16` declares `[external_cells]` / `prelude = bundled`, which is buck2's mechanism for supplying the prelude from inside the binary; `tools/buck2:1` is `#!/usr/bin/env dotslash` with blake3-pinned per-platform digests; `ci.yml:176` installs the pinned runtime and `:192` runs `tools/buck2 test //backend/crates/support/domain:console-support-domain-unit` in the required job at `:164`. buck2 is fully functional and hash-pinned. The governance question (neither cargo nor buck2 is an accepted decision, since `PIVOT-2026-07-28.md` is not in `docs/decisions/`) survives untouched — what dies is the forced-migration premise. Retained as survivor X8-1, which is the deletion, not a new claim.
- **KX-2** — premise-false: "The capability registry's `hold_rule` fails closed on empty Buck2 target sets, which is keyed to a dead build system, so HOLD is permanent and the plan must escape it." Two independent reasons this generates no plan edit: buck2 target sets are producible today (KX-1's evidence), so the clause is a requirement to satisfy rather than a deadlock; and `grep -n 'hold_rule\|capability-registry' docs/ideas/ecosystem-plan-DRAFT.md` returns **zero hits** — the plan never makes the claim, so there is nothing to correct. Reported by X8 as a consequence, but it lands on `docs/program/console-capability-registry.json`, not on this document. No edit.
- **KX-3** — premise-false: "B9 blocks Slice 0 / Slice 0 cannot start until the grant substrate is re-tiered." Slice 0's two grants are both at 현장 scope (§8's Slice 0 table: "2 instances: `purchase.approve` at 현장 scope (authorises) + one at a **different** 현장 (must not)") — intra-org, which X4b CASE 1 measured resolving end to end with subject, capability and scope folded out of Tier N through an `ont_link`. B9 bites the `group` arm only, which Slice 0 does not use. What survives is narrower and is captured in X4b-1: Slice 0 must not *publish* a `grant_scope` link type whose declared target set includes `group` or `organization`.
- **KX-4** — superseded-by-experiment: "§4.2 collapses and the 141-table cost returns" (§8 Phase 6's X4 "If refuted" branch) and any revision item that re-prices the entity model against the 141 RLS policies or introduces `app.current_group`. X4 executed 30 assertions PASS / 0 FAIL with 3 controls observed RED; the GUC inventory extracted from every stored policy expression the probe created returned `app.current_org` and nothing else in **both** variants; only `x4probe_*` policies were created, so none of the 141 was created, altered or dropped. The confidentiality requirement held across 11 probes including `COUNT`, `EXISTS`, `DISTINCT`, aggregate max, `UPDATE` and a `23505` collision, with the hidden row proven present by RLS-bypassed ground truth. §3.2 Option 2 stays rejected on evidence, not on argument.
- **KX-5** — premise-false: "Every Tier N entity is unusable, so move them all to Tier T" (§8 Phase 6's X2 "If refuted" branch). X2's prediction was confirmed, not refuted: the empty list is closeable over HTTP today. Executed: `HALF 1 no policy attached -> 200 OK []`, then `HALF 2 policy attached (201 Created) -> 200 OK titles=Some([String("visible-to-owner")])` via `POST /api/v1/ontology/object-types/{stable_key}/policies`, backed by the audited definer in `0205_ont_policy_api_attach_writer.sql`. The requirement is "attach the object policy in the same change that publishes the type", which §0.13 already states. No tier migration.
- **KX-6** — premise-false: The strong form of the X1 correction — "§0.12's claim is wrong, because `create_link` writes an `ont_links` row with no property involved." `PgInstanceStore::create_link` (`adapter-postgres/src/instances.rs:291`, audited write at `:319`) exists, but every call site in the repo is under `tests/` (`instances_rls_surfaces_as_runtime_role.rs:382,386`; `c_chain_as_runtime_role.rs:308,320`; `object_policy_attach_as_runtime_role.rs:2833`) and `ONTOLOGY_ROUTE_PATHS` (`rest/src/lib.rs:213-228`) is exactly 14 paths, none of which creates a link. The claim is intact for every reachable path; only its wording needs the word "reachable", which is survivor X1-1. Nothing about the guard, the trap or §4.3's specification changes.
- **KX-7** — out-of-scope: "Fix §5.11 G9's and Phase 7's claim that the `audit-coverage` exclusion set has exactly one entry." The claim is indeed false — but it rests on `backend/ci/gates/audit-coverage/src/lib.rs` and `ADR-0002:20`, not on any experiment record, and no probe in X1/X2/X4/X4b/X8/X9 touched it. It belongs to the citation/adjudication stream, which the brief already names. Flagging so it is not dropped in the seam, not claiming it here.
- **KR-1** — premise-false: The plan does not cite Bun for the fragmentation rationale anywhere. `grep -n "fragment" docs/ideas/ecosystem-plan-DRAFT.md` returns **zero hits**, as does `grep -n "never by file\|by file"`. §8's only crate-rule statements are neutral and correct: the opening paragraph lists "a by-crate queue" among the Bun mechanisms, and Phase 3 says "the work queue, by crate / Next crate activates only when the current one is clean" — which is exactly Correction 1's true rule (the crate is the unit of agent assignment and review). Nothing to fix in this artifact. The error is real but lives in two other documents, flagged for their owners: `docs/ideas/lane-assembly-line.md:19` "Overlapping the stages is the task fragmentation Bun grouped by crate to avoid" and `docs/ideas/governed-object-engine-PLAN.md:232` "Errors grouped **by crate**, not file (anti-fragmentation)".
- **KR-2** — premise-false: The plan contains no Bun cost figure to correct. `grep -n "165,000\|5\.9B\|690M\|72B\|cached input" docs/ideas/ecosystem-plan-DRAFT.md` returns **zero hits**; §8's Bun numbers are only "6,502 commits, +1,009,272 lines, 11 days, no incremental merges", all CONFIRMED in `research-agentic-practice.md:25-27`. Correction 4 applies to `docs/ideas/lane-assembly-line.md:62` — "| Cost | 690 M output tokens, 5.9 B uncached input, **~$165,000**, 11 days | — |" — which does omit the 72B cached input reads. Not this artifact.
- **KR-3** — premise-false: No code or spec `file:line` anchor in the plan needs converting — twenty were verified and every one resolves exactly, which is the required form. `authz/src/lib.rs:109` = `pub enum Feature {`; `:372` = `pub const ALL: [Self; 96] = [`; `:573` = `const fn matrix_row(self) -> [PermissionLevel; 6] {`; `:1472` = `pub async fn resolve_branch_scope_in_org(`; `instances.rs:291` = `pub async fn create_link(`; `:1479` = `fn allowlisted_projected_table(name: &str)`; `tenant-isolation/src/lib.rs:804-808` = the four-condition classification guard; `realtime/src/lib.rs:318` = `pub enum RealtimeEvent {`; `residual.rs:200-203` = the `// Deny-by-omission` comment + `if permits.is_empty() { return ResidualFilter::deny_all(); }`; `rbac-configurable.md:257-259`, `:366`, `:421-423`; `korean-legal-boundaries.md:40-43`; `org-editor-primitives-ux.md:256`; `org-hierarchy.md:3-7`, `:96`, `:172-173`; `foundation-gates.md:60`; `LANE-PROTOCOL.md:90`; `governed-object-engine-PLAN.md:75` and `:301-302`; `no-code-ontology.md:133-141`; `console-program-ledger.md:327`. Leave all of these as line numbers.
- **KR-4** — premise-false: `README.md:12` (plan §5.11 G8) is not a latent B10. `docs/decisions/README.md` will indeed be edited — its own line 3 says the index "must be updated atomically with every ADR status, identity, amendment, or supersession change", and nine records are owed — but the `## Current index` heading and its table begin at line 30, **below** line 12, so adding index rows cannot shift it. Line 12 is Authority rule 6, "Current implementation/live evidence may show that code diverged from an ADR; that is a governance gap, not silent supersession. Reconcile it through a new decision" — precisely what G8 cites it for. The anchor holds unless the Authority rules list itself gains a rule. Only the bare basename needs fixing (folded into C10).
- **KR-5** — premise-false: `Cargo.toml:53` in §8 Phase 7 is not unresolvable. There is no root Cargo.toml, but `backend/Cargo.toml:53` is `rust-version = "1.96"` under `[workspace.package]`, exactly the MSRV floor the plan says it is — and the adjacent claim checks out too, since `foundation-gates.md:60` does read "Rust is pinned to 1.96.0 for CI/local parity". Under-qualified path only; folded into C10, not a separate finding.
- **KR-6** — out-of-scope for the citation stream, owned by the experiments stream: §8 Phase 7's "buck2 is live in CI now (five steps + a required reachability job), while `prelude/` is missing so the buck2 graph is already broken" and the X8 row's "`prelude/` is **missing** (verified: no `prelude` dir)". `experiment-results.md` X8 found buck2 FULLY FUNCTIONAL, so "the graph is already broken" is premise-false. The plan already contradicts itself: the Phase 7 row two lines above says "targeting the CI that **exists** (buck2 live)". This is a premise defect, not a citation defect — carried as survivors 16e / X8-1 / X8-2 in wave 3.

---

# WHAT THIS REVISION DOES NOT FIX

An honest not-doing list is what stops the next pass re-deriving these. Each entry names **why** it is not
fixed and **what** would unblock it.

### Needs a qualified external authority

1. **Korea stays HOLD.** Six controls in `docs/program/console-jurisdiction-register.json`, all
   `release_disposition: HOLD`. `unhold_authority` is verbatim *"Qualified Korea legal/compliance authority
   with attributable I2/I3 candidate-bound receipt"*, and `:1186` is *"Missing, stale, conflicting, or
   unqualified authority is HOLD; **agents may not invent certainty**."* A native agent produces only
   `I1_NON_INDEPENDENT`. **No item in this brief asserts a Korea compliance conclusion or proposes
   unholding.** Wave 7 item 7.6 records the bar; it does not satisfy it.
2. **The payroll 노무사 / 세무사 validation artifact does not exist.** The plan may not claim payroll
   correctness is validated. It is the first vertical's acceptance bar (§2 driver 3) and the bar is
   unmet by construction until a qualified professional signs an attributable artifact.
3. **Exposure stays HOLD for both halves of Slice 0.** 27/27 capabilities carry
   `"implementation": "HOLD"` and `"exposure": "HOLD"`. Wave 3 item 3.1 deletes the plan's "lands and
   ships" claim; nothing in this revision can move exposure.

### Needs an owner decision that has been captured but not accepted

4. **D4 (ADR-0030 + ADR-0031).** `docs/ideas/d4-frontend-charter.md` captures four owner decisions
   (2026-07-30) and splits D4 into two records. Both are **pre-acceptance**: `status: proposed`,
   `doc_status: review`, `proposes_amendments_to`, and **no active `amends`**. Nothing in the plan depends
   on either. The CI gate asserting the console frontend does not exist **stays** — under the charter it is
   the enforcement mechanism for "planning only". This revision records the state; it does not accept the
   records, write them into `docs/decisions/`, or resolve the charter's four open questions (contracts-crate
   DTO shape, SSR-vs-CSR, Leptos 0.9 beta fallback, the absent string gate).
5. **§9's open question — "who may author authority, and in which scope?"** Four-eyes answers *how many*,
   not *which scope*. The plan already says this is unresolved. It stays unresolved.

### Needs an experiment that has not run

6. **X3, X5, X6** — all three need `effective_grants_for` to exist, so they are **slice-0 work, not
   prepwork** (wave 3 item 3.5). Recorded before Slice 0 may be declared green; not before the first
   commit.
7. **X7** — requires pushing a branch, so it is outward-facing and needs **explicit authorization**.
8. **X-T7a** — extending `is_handler_surface` to path component `app` has **unmeasured** blast radius;
   D3's enumeration must not assume it.
9. **X-T9b** — whether `GROUP BY account_code` is reproducible (`'100'` vs `' 100'`; `0160:62` rejects blank
   but stores untrimmed). `economics_is_a_view` depends on it.
10. **X-T2f** — the read-only inert-condition census must precede shipping the narrowed
    `policy_role_conditions` write path (wave 7 item 7.1(a)).
11. **X-CITE** — the mechanical citation audit is added as a plan deliverable (wave 3 item 3.3); this
    revision fixes the citations it found, not the ones it has not run against.

### Deferred by decision, with constraints recorded instead of schema

12. **`party`, `party_org_visibility`, `users.party_id`, `employees.party_id`** — deferred out of Slice 0 on
    irreversibility (D1 clause 3, R2), with five non-foreclosure constraints carried verbatim. No lane
    waits on them.
13. **The `party` resolution mechanism** (one human, two orgs) — stated as a named pre-condition for when
    `party` lands, not designed here.
14. **`audit_events` capacity columns** — deferred by conflict resolution C-1; the 466-site `AuditEvent`
    reach is priced, not scheduled, and the `AuditEvent::authorized(…)` constructor is a recommendation.
15. **Quantity-bearing lineage (`lot` / `lot_split`, W16)** — R12 defers it with no 0207+ slot; the
    conservation mechanism and non-foreclosure constraints are recorded (wave 5 item 5.4, wave 4 G4/N4).
16. **The `object_types` / `ont_object_types` bridge** — deferred (KILLED K14); it is a confidentiality
    question, not plumbing, and must not consume slot 0207.
17. **직무 and 직급** — no substrate exists; the plan states which widening brings them and does not build
    them.
18. **The no-code canvas (W10)** — off the slice-0/1 critical path, deferred by follow-up. No charter is
    owed (G6 struck), so the deferral stands on this plan's own merits.
19. **The peer finance plan** — account master / chart of accounts, multi-currency and FX, depreciation and
    accrual posting, overhead allocation, shared-service charge-out, inter-company charges. Named, argued,
    not folded in. The three shipped parallel money stores are its reconciliation backlog.

### Other documents this revision does not edit

20. `docs/ideas/lane-assembly-line.md:19` and `docs/ideas/governed-object-engine-PLAN.md:232` carry the Bun
    **fragmentation** misattribution; `lane-assembly-line.md:62` omits the **72B cached input reads** from
    the cost figure. Real errors, wrong artifact (KILLED KR-1, KR-2). Flagged for their owners.
21. `docs/program/LANE-PROTOCOL.md` (stale status header, `0204` high-water) and
    `docs/ideas/no-code-ontology.md` (stale publish-route claim) and `docs/program/CATALOG.md:62-68`
    (unshipped type list) are **recorded as Phase-7 correction items inside the plan**; this revision does
    not edit those files.
22. `docs/program/console-capability-registry.json` — the registry rows Slice 0 owes are named as a
    Phase-7 governance step (wave 3 item 3.1); they are not written here, and `dispatch_rule` / `hold_rule`
    remain fields nothing enforces.
23. **`docs/ideas/ecosystem-plan-review.md` and `docs/ideas/adr-collisions-raw.md` are not corrected.** The
    review's stale `:116`/`:571`/`:606` proposals and the raw 75's premise-false collisions survive in
    those files by design — this brief is the record of what was adjudicated, and §KILLED is why nothing
    in them may be re-imported without new executable evidence.
