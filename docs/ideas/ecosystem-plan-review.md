> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Ecosystem plan — review record

Status: REVIEW — architect + critic, plan remains PENDING APPROVAL

Reviews `docs/ideas/ecosystem-plan-DRAFT.md` (1,853 lines, `Status: PENDING APPROVAL`) against the
exit criteria the owner set: implementation-ready in the Bun shape, experiments on the parts that
need experimentation, all prepwork before fanout, one PR. Two independent lenses (architect,
critic) plus a synthesis pass that re-verified every blocking claim against executable code. Line
numbers in this record were re-checked in this session; the checks are listed in §7.

---

## 1. Verdict summary

**Architect verdict: SOUND_WITH_FIXES.** The entity model and the tier discipline are correct and
structurally derived from executable gates; the defects concentrate wherever the plan crosses a
store boundary or asserts a mechanism instead of citing one.

**Critic verdict: ITERATE — `implementation_ready: false`.** All five architect blocking findings
CONFIRM, two confirm stronger than stated, and four further defects change what an implementer
writes in Slice 0 itself. No automatic-REJECT trigger fires: no gate is weakened, no production
exposure is widened, no Korea conclusion is asserted, no I2/I3 evidence is claimed.

**Implementation-ready: NO.** Twelve items must land before the first Slice 0 implementation
commit (§2). The plan is one revision pass from ready — not a redesign. The §4 model, the §6
pre-mortem and the §7 known-bad-control discipline survive review intact (§5).

---

## 2. Blocking items — these gate Slice 0

Twelve items. Each names the plan §, the executable evidence, and what the change must achieve.
Ordered by how early the fix has to land.

### B1 — §5.1 payment term (2); §4.6; §7 `definer_revalidation_each_check`; Slice 0 `effective_grants_for`

Re-chaining `prev_hash`/`row_hash` on every definer read cannot be implemented today, for two
independent reasons, and the code states both itself.

(a) `backend/crates/ontology/adapter-postgres/src/instances.rs:1297-1322` records, in the
imperative, that a previous comment claiming sorted-key canonicalization was FALSE; that
`cargo tree -e features -i serde_json` measured `serde_json` feature `preserve_order` ← `cedar-policy-core
v4.11.2`, reaching this crate through `console-platform-authz`; that `serde_json::Map` is therefore
insertion-ordered, so `verify_chain` "reports a break on untampered data … any object with two or
more attribute keys whose insertion order differs from read-back order"; that the fix is
deliberately withheld pending a re-seal decision and an audit-chain owner; and the sentence that
settles it — "The suite is green because it does not recompute hashes — not because recomputation
would succeed." A `grant` bag (subject, capability, scope, source) has four keys.

(b) `revision_row_hash` at `instances.rs:1346-1357` is Rust-side SHA-256 over
`serde_json::to_vec` bytes. A plpgsql `SECURITY DEFINER` cannot compute it at all, so the check
cannot live where §5.1 puts it.

The plan nevertheless lists it as one of four required Slice 0 checks with a GREEN baseline and a
four-way deletion probe (`:931-932`, `:1486`, `:1707`). Verified independently against the
dependency manifest, not only the comment. Highest-confidence finding in this review.

**Required.** Delete check (2) from §5.1 and from Slice 0's `effective_grants_for` row. Then pick
one on the record: (i) the definer's fixity assertion is chain LINKAGE only
(`prev_hash == predecessor.row_hash`) — what `company_conformance.rs` already does and what SQL
can do — with the reason recorded; or (ii) the canonicalization fix (explicit key sort + re-seal)
becomes a named Phase-0 prerequisite with an audit-chain owner. Stop describing
`definer_revalidation_each_check` as four-way until the count of implementable checks is settled.

### B2 — §5.1 the read circle; §4.5 traversal; §7 `definer_ignores_parameter_org`

The definer's grant read has no org predicate, so it returns another tenant's grant revisions for
any party legitimately visible in two orgs — the exact case §4.2 exists to serve.

§4.5's pseudocode (`:730-737`) is `grant instances WHERE subject = p_party AND scope ⊇ p_scope AND
asof ∈ [...]`, executed with `row_security = off`. Those rows are `ont_instances` /
`ont_instance_revisions`, whose ONLY isolation is FORCE RLS on `app.current_org`
(`backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:18` org_id NOT NULL;
`:90-98` ENABLE + FORCE ROW LEVEL SECURITY + `org_isolation` policy on both `USING` and `WITH
CHECK`). Check (1) gates the *party*; nothing gates the grant *rows*. Only check (3) — the vaguest
of the four — stands in the way, and for a group-scope grant the reachable set arguably includes
it. This is pre-mortem 1 realised through a path pre-mortem 1 does not name; the plan's own probe
tests the party parameter, not this path. Slice 0 ships this definer.

Two cited precedents do not support the construction. `store.rs:576-593` is Rust-side validator
re-execution on an ordinary pooled read — a precedent for re-validation as a discipline, not for
reading with RLS off. `group_role_grants_for_user` turns `row_security` off on an owner-only table
carrying no RLS policy at all (`backend/ci/gates/tenant-isolation/src/lib.rs:121-124`, rationale
"cross-tenant group role authorization; own-grants resolver only"), where the switch is nearly
inert. The plan copies the syntax of a safe pattern into a place where it is the whole security
property.

**Required.** Add a fifth check and make it the FIRST: every grant and revision row read inside
the definer is filtered `AND org_id = current_setting('app.current_org')::uuid`, literal not
parameter. Add probe `definer_returns_no_foreign_org_grant` — one party with a visibility edge in
BOTH orgs and a grant in each; org A's call returns exactly one — with known-bad control the
definer exactly as §4.5 specifies it today. State in §5.1 what `store.rs:576-593` is and is not a
precedent for.

### B3 — §4.1 Tier O `party`; §3.2 Option 4; §5.4 D; §5.11 G1; §8 Phase 6 X4

No mechanism is specified by which two orgs arrive at the same `party` row, and the plan closes
every available route itself.

`party` is `(id, party_kind, status, created_at)` "and nothing else" with "never put personal
attributes on party" as a permanent invariant (`:510-512`). Option 4 (matching) is rejected
because "a mechanism that must decline the ambiguous majority is not an identity" (`:376-387`).
§4.7's account/character analogy resolves identity by the account AUTHENTICATING. A grep across
the plan for resolve / dedup / self-link / same-party returns only Option 4's rejection and
`:630`. Consequences: org B onboarding the same human mints a second `party`, reproducing
`users`/`employees` one tier up; §4.2's confidentiality property becomes vacuous; and X4 —
"the plan's central claim, so test it first" (`:1647`) — requires "a person in two orgs" and
therefore has no specified input. G1's own claim is "one durable identity per natural or legal
person, across every tenant and vertical", and G1 is listed as blocking Slice 0.

**Adjudication, and a citation correction inside the governance table.** §5.11 G1 reads ADR-0022
as deciding identity is local/org-scoped. `docs/decisions/ADR-0022-local-identity-no-external-idp.md`
Decision block is narrower: it decides against a speculative EXTERNAL IdP seam and confines
`console-identity-application` to local org/account administration. It never decides identity is
org-scoped. A platform-level `party` is still LOCAL identity, so the obstacle is smaller than G1
claims — which makes the fix cheaper. The missing mechanism is the defect and stands on its own.

**Required.** State the resolution mechanism in §4.1 BEFORE G1 is drafted, and pick one
explicitly: (a) `party` is minted per passkey credential and self-linked by the human at
second-org onboarding — name the endpoint, and note ADR-0022 permits it because this is local
identity, not federation; (b) party resolution is a platform-principal operator action with an
audit record, never a tenant capability; or (c) narrow G1 to "one identity per org-cluster" and
say so in §3.2, which currently rejects Option 4 for a weakness the recommended option would then
share. Then rewrite §5.4's recommendation to price the loss the way §5.7 prices its three (see
the tension, §3.2 of this record).

### B4 — §5.1 genesis circle; §7 `genesis_grant_not_runtime_mintable`

The genesis resolution is factually wrong about the substrate and its Slice-0 probe is RED against
shipped code on day one. §5.1 states genesis "is a migration fact, never a runtime capability, so
there is no runtime code path that can mint authority from nothing." But `create_org`
(`backend/crates/platform/platform-rest/src/lib.rs:568`, live POST route registered on
`PLATFORM_ORGS_PATH` at `:235`) is documented in its own header as "onboard a NEW tenant (the only
place org rows are created by the app), seed its first SUPER_ADMIN, and return a one-time OTP",
authorizing `PlatformFeature::TenantCreate` at `:574`. Org creation is a runtime capability and a
runtime path already mints authority from nothing.

This is the same failure mode §0 was built to catch — reading the artifact that documents an
invariant instead of the code that enforces it — turned on the plan's own §5. The fix makes the
plan stronger, not weaker.

**Required.** Restate the genesis resolution against the shipped path: genesis is a
PLATFORM-PRINCIPAL capability gated on `PlatformFeature::TenantCreate`, never a tenant capability,
and the existing seed-first-SUPER_ADMIN step is the extension point for minting the genesis grant.
Re-scope the probe to `genesis_grant_mintable_only_by_platform_principal`, known-bad control a
TENANT-authenticated endpoint creating a grant with no authorising grant. Note in C4b/G2b that the
onboarding seeder is a SUPER_ADMIN write site the `Role`-deletion migration path must cover.

### B5 — §8 Phase 7 prepwork; §8 deployment dependency (`:1688`)

There is no admissible path from this plan to a first commit, and the plan never mentions the two
artifacts that make it inadmissible.

(a) All 27 capabilities in `docs/program/console-capability-registry.json` carry
`truth {declared: DECLARED, implementation: HOLD, verification: HOLD, exposure: HOLD}` — counted,
27/27, no exceptions. `hold_rule` reads "Unassigned worktrees, overlapping ownership, missing
backend contracts, **empty Buck2 target sets**, stale reference digests, or missing jurisdiction and
independent-review receipts fail closed." The single recorded `hold_rule_amendment` (2026-07-25,
lane L-P0-EPOCH) amends only "the 'missing backend contracts' clause", and its own `limits[0]`
says verbatim: "Does not relax any other hold-rule clause: unassigned worktrees, overlapping
ownership, **empty Buck2 target sets**, stale reference digests and missing jurisdiction or
independent-review receipts still fail closed." So the one amendment on the record explicitly
reaffirms the clause pinning every capability — while `prelude/` is absent and a required job runs
buck2 (`.github/workflows/ci.yml:163-164` "Support domain — Buck2 unit reachability"; `:192`
`tools/buck2 test //backend/crates/support/domain:console-support-domain-unit`). §8 schedules X8 to
*explain* how the buck jobs pass and flags build-system governance as open, but never connects
either to the clause.

(b) Zero of the eighteen desired concepts are registered as work;
`docs/program/console-program-ledger.md:823` — "Nothing in the idea document is approved work."
The registry's `dispatch_rule` admits a row only with one writer, an isolated worktree, disjoint
ownership roots, a signature story, evidence paths and executable leaf gates. Phase 7 supplies
ownership rungs and a pre-reservation commit and no registry row.

"Slice 0's Tier T half lands and ships" (`:1688`) is not a claim this plan can make.

**Required.** Add one Phase-7 rung with two items: (1) resolve the `hold_rule` Buck2 clause
explicitly — an amendment through the same lane/amendment mechanism as `hold_rule_amendment`, or a
recorded finding that the clause is satisfiable for these targets — quoting the existing
amendment's own `limits[0]` so the next reader cannot mistake it for coverage; (2) register Slice 0
and each widening group as capability rows carrying signature story, `evidence_path`, leaf
commands, ownership roots, and which of the eighteen concepts each covers. Retract `:1688` in
favour of "CI-provable; exposure remains HOLD".

### B6 — §4.0 / §4.0.2 / §0.14 / §4.6 "Actions dispatch, they do not bypass"; §5.11 DN-0003 invariant 1

§4.0 claims "the systems light up for it without anyone hand-writing an integration per concern".
For a projected type that is false. `ActionDispatch`'s `projected_usecase` arm routes through
`ProjectedDispatchRegistry` — `backend/crates/ontology/rest/src/lib.rs:160-195`: a
`HashMap<String, ProjectedHandler>` of `Arc`'d Rust closures, `#[derive(Clone, Default)]` so empty
by default, with the documented fail-closed contract "an **unknown target is a typed `NotWiredYet`
error**", populated one `register(target, handler)` call at a time by the App composition root. So
EVERY action on a projected type is a hand-written integration. §0.14 just moved `work` — Tier T +
projected, the declared join point for artifacts, actions, handover, ledger and metrics, the entity
composing the most concerns — into exactly that tier, and Slice 0 ships it. §4.0.2 lists projected
backing as "one arm in `allowlisted_projected_table`", understating it by the whole handler
registry.

Secondary half, downgraded from blocking to major during synthesis (see §7): DN-0003 invariant 1
reads "not the NORMAL operational write path", which is weaker than the plan's paraphrase — but it
still has no mechanism holding it for Tier T/P. A `work` row's assignee or `due_at` is mutable by
ordinary domain SQL, and `list_projected_rows_tx` is a live read-through with `version` always 1
and empty fixity hashes. ADR-0002's gate covers the audit half; nothing covers the Action-verb half
or the history half.

**Required.** Add a row to §4.0.2's requires-code column: "actions on a projected type — one
`ProjectedDispatchRegistry` handler per action, registered in the App composition root; unwired =
`NotWiredYet`". Enumerate `work`'s Slice-0/W4/W11/W13 actions with a handler count in
`ecosystem-entity-components.tsv` so the Phase-3 `app` crate row is sized rather than labelled
"wiring". Then state how DN-0003 invariant 1 is satisfied for Tier T/P before `work` is built —
either every consequential `work` mutation is an `ont_action_types` dispatch, or it is a bounded
exception naming the gate that holds it and the history it forfeits — with a probe and known-bad
control.

### B7 — §5.6 F; §5.3 C-deltas; §4.6 "What Cedar reads, and how"

Two undischarged obligations on the path the fold's output takes into Cedar.

(a) ADR-0021 decision 5 (`:57-60`) requires that "Role, assignment, **responsibility**, employment
state, branch/team, or credential changes synchronously bump subject/policy versions so stale
subject material cannot keep granting access", and decision 6 makes stale subjects a DENY. A grant
is exactly a responsibility change. §5.6 chooses `policy_versions` (per-ORG) as the invalidation
key and never assigns any write path to `authz_subject_version` (per-SUBJECT, carried into the
Cedar subject entity at `backend/crates/platform/authz/src/cedar_pbac/engine.rs:368-379` as
`subject_version`). The plan cites `authz_subject_version` once, at `:979`, as inherited, then never
discharges it. Per-org invalidation is also strictly coarser: any grant change invalidates every
party's fold in that org, a cost X6 does not measure.

(b) `Entities::from_entities([subject, resource_entity], Some(&bundle.schema))` at `engine.rs:449`
VALIDATES entities against the bundle schema. §4.6 introduces `capabilities: Set<String>`,
`scopes: Set<String>` and a decision-scope resource attribute, and prices only `roles` (correctly:
`roles` is already `RestrictedExpression::new_set` of strings at `engine.rs:365-374`). Until the
bundle schema declares the three new attributes, entity construction fails — which fails closed
and denies everything. A hard Phase-3 ordering constraint stated nowhere.

**Required.** In §5.6 add a row: a grant revision bumps `authz_subject_version` for the subject
party's users AND `policy_versions` for the org, citing ADR-0021 decision 5's "responsibility"
clause; add probe `grant_write_bumps_subject_version` with known-bad control a grant write that
bumps only `policy_versions`. In §4.6 state that the three new Cedar attributes each require a
bundle-schema declaration, and make it an explicit Phase-3 ordering constraint: platform/authz's
schema change lands before any code reads the fold.

### B8 — §5.8 H conservation; §4.3 `derived_from`; §7 `lot_conservation`

The row CHECK does not conserve, and the reason §5.8 gives for retracting the definer is refuted by
the precedent it cites. `CHECK (parent_qty_before − split = parent_qty_after)` is per-row
arithmetic; the invariant that matters spans successive splits of the same parent. Two concurrent
splits of a 100-unit lot each written as (100, 60, 40) both satisfy the CHECK and over-allocate by
20.

The plan's claim — "a definer is needed when an invariant spans sibling rows, and putting
before/split/after on one row removes the span" — is wrong about the shipped pattern. `0156`
conserves via `lock_consumption_idempotency_key_tx`
(`backend/crates/inventory/adapter-postgres/src/lib.rs:376`), `fetch_item_for_update_tx` (`:394`, a
SELECT … FOR UPDATE on the item's current quantity) and a domain `state.consume(quantity)` (`:411`);
`0156_create_inventory.sql:103` — `CHECK (quantity_before_milli - quantity_consumed_milli =
quantity_after_milli)` — is the arithmetic backstop on top of that, not the mechanism.
`lot_conservation`'s known-bad control is single-row so it cannot detect this.

Secondary: `lot.quantity_milli` and the split deltas are the same fact in two places, which
`no_duplicated_fact` forbids; and the table is `lot_split` at `:1188` but `lot_derivation` at `:670`
and inside §5.8's own traversal at `:1211`.

**Required.** Keep the CHECK and add the mechanism the precedent actually uses: the split write
locks the parent lot row FOR UPDATE inside the action's transaction, derives `parent_qty_before`
from the locked row (never from the request), and updates `lot.quantity_milli` in the same
transaction. Add probe `lot_concurrent_split_cannot_overallocate` with the row-CHECK-only
implementation as its known-bad control. Decide whether `lot.quantity_milli` is authoritative or
derived and say which. Fix the table name to one spelling.

### B9 — §4.3 (`grant_scope`, `position_at_scope`); §4.1 Tier N `grant`; §4.2; §8 Phase 6 X4

Group- and organization-scoped authority cannot be stored where §4.1 puts it, and the experiment
designated to test the plan's central claim cannot refute it.

(a) §4.3 specifies `grant → org_unit | organization | group` and `position →` the same, "Stored as:
`ont_link`" — but `ont_links` FKs BOTH endpoints to `ont_instances(id, org_id)`
(`0155_create_ontology_instances.sql:76-77`), so an edge to an `organizations` or `groups` row is
structurally impossible. And `groups` sits in the tenant-isolation gate's `global_table_allowlist`
with rationale "group identity metadata only, no tenant data"
(`backend/ci/gates/tenant-isolation/src/lib.rs:48`), i.e. Tier G, not an ontology instance.

(b) The deeper half: `grant` is Tier N, so a group-scope grant physically lives in one org's
`ont_instances` rows under FORCE RLS and is unreadable by every sibling org that needs it at raise
time — precisely what §4.5 requires ("eligible approvers = effective(·,
step.competent_unit.scope)"). The shipped answer for cross-org authority is Tier O + a definer:
`group_role_grants`, owner-only, rationale "cross-tenant group role authorization; own-grants
resolver only" (`tenant-isolation/src/lib.rs:121-124`).

(c) X4 is scoped to the easy half — resolving a KNOWN party's authority WITHIN the armed org —
while its hard half is deferred to W5/W8, yet §4.2 asserts sufficiency globally and §8 calls X4
"the plan's central claim, so test it first".

**Required.** In §4.3 replace `ont_link` for the `organization` and `group` arms with the real
substrate — a scope descriptor property `{level, node_id}` per `org-hierarchy.md:172-173`, not an
edge. In §4.1 split `grant` by scope level: org_unit/organization-scoped stay Tier N;
group-scoped grants are Tier O beside `group_role_grants`, reached only through the definer — and
correct §9's cost line, which says "one owner-only table". Extend X4 with the falsifying case: from
a session armed to org A, resolve the eligible-approver set for a step whose competent unit is at
group scope where the only qualifying holder is a user of org B, and state honestly that on the
current design this is not answerable without iterating member orgs or a Tier O grant store.

### B10 — §0.1 (BLOCKING); preamble (`:5`); §0.8; §5.5

The plan's one BLOCKING correction has three citations and none resolves, and the break is
self-inflicted.

`docs/ideas/authority-and-approval-model.md:89-92` is now other material; the retraction text
"**This revises the earlier 'group is the tenancy boundary for people' answer.** The group is not
high / enough. … Group-scoping relocates the duplication rather than removing it" is at
`:116-121`. `:545-546` is now about `company` as free text; "**People are group-scoped.** Per the
owner's choice, the group is the tenancy boundary for people" is at `:571`. `:575-579` is now about
roles-as-grant-bundles; "This is the largest single engineering cost in the chosen model" is at
`:606`. The ~+27..30 shift is the SUPERSEDED header the plan's own author added at `:3-20` — and
that header restates the stale citations at `:11`.

The substantive claim is TRUE at the new lines; both lenses confirmed it there. But a lane sent to
verify the plan's one BLOCKING item reads the wrong passage and concludes §0.1 is fabricated —
under a preamble asserting "Line numbers re-verified this session", which asks the reader to extend
an unearned reliability to the 1,800 lines nobody will check.

Same class: §0.8 and §5.5 assert repo-wide negatives over "206 migrations" three times (`:120`,
`:1046`, `:1136`). The main checkout has **205** `.sql` migrations, highest `0205_ont_policy_api_attach_writer.sql`
(the 206th directory entry is `BUCK`), and `0206` is in flight in another lane's worktree — so any
reservation starts at 0207+.

**Required.** Re-anchor §0.1 to `:116` / `:571` / `:606` and fix the same three citations at
`authority-and-approval-model.md:11`. Then apply the rule `fanout-plan-DRAFT.md:243` already
implies: citations into a document you also edit are quoted-text anchors, not line numbers — and
sweep every other cross-document citation into that file for the same drift. Restate the migration
negatives as "all 205 migrations in the main checkout as of `<commit>`" and name the commit.

### B11 — §4.1 Tier N effective-dating; §5.9; Slice 1; §2 driver 2

The plan has no correct-versus-new-effective-change axis, and the shipped store structurally forbids
the first. `ont_instance_revisions` carries `valid_from`/`valid_to` with
`CHECK (valid_to IS NULL OR valid_to > valid_from)` (`0155:53`) and a unique index permitting
exactly one open revision (`:57-58`), and the append-only trigger forbids modifying a closed
revision. So correcting an erroneous revision AT THE SAME effective date is inexpressible: you
cannot rewrite it, you cannot close it at a zero-length interval, and appending at the same
`valid_from` overlaps. Every path leaves the fold returning the wrong value for the period between
error and discovery.

The plan's only "correction" is the compensating document for post-확정 반려 — a different concern.
Slice 1 is 인사발령, and 소급 정정 of a mis-entered 발령일 or pay grade is routine payroll work, so
this lands on a shipped Slice-1 entity, while §2 driver 2 says replayable is "free, not built".

**Required.** Decide the correction path explicitly before Slice 1: either a correcting revision
carrying `corrects_revision_id` plus a knowledge-time argument on as-of reads (the bi-temporal
entry-date axis), or a stated deferral with the consequence named. Qualify §2 driver 2 accordingly,
and add a probe whose known-bad control is a correction that silently rewrites history.

### B12 — §1 principle 2; §5.1; §5.11 — segregation of duties

SoD is absent from the plan and it collides with a shipped spec rather than being an inherited
omission. A grep for segregation / toxic / mutual across all 1,853 lines returns ZERO hits, while
`docs/specs/cedar-pbac-authorization.md:122` decides "Segregation of duties and self-approval
checks are PBAC conditions, not UI-only rules", `no-code-operational-logic.md:211` requires
"segregation of duties and self-approval prevention", and `operations-intelligence.md:170` requires
conflict-of-interest flags. Principle 2 states "Additive grants only… it never writes a deny", and
positive-only plus accumulation is exactly the shape that produces grant combinations individually
legitimate and jointly dangerous. The plan's only SoD content is the shipped four-eyes
`CHECK (approved_by <> created_by)`.

Crucially, mutual exclusion does NOT require a deny in the fold — it is a constraint at
grant-AUTHORING time — so principle 2 is not in tension with it and the omission is not forced.

**Required.** Decide it in or out on the record, citing `cedar-pbac-authorization.md:122` either
way. If in: name it as a grant-authoring-time constraint (conflict pairs over `Feature`, evaluated
where `gov_approvals` four-eyes already runs), with a widening and a probe whose known-bad control
is a fold that accumulates a conflicting pair silently. If out: state it in §5.11 with the three
spec citations as the recorded cost, so it is a choice rather than a silent contradiction.

---

## 3. The architect's antithesis and the tradeoff tension

Preserved verbatim. A future reader needs the strongest case against the plan, unsoftened.

### 3.1 Antithesis

The plan's §0 is the best thing in it and also the source of its central defect. §0 set a standard
— cite executable code, never a header comment — and applied it to *facts about what exists*, where
it works: I re-verified §0.12, §0.4, §0.8, §0.10, §0.13 and they hold, several against non-obvious
code. It did not apply that standard to the *mechanisms it invents*, and there the exact failure
mode §0 congratulates itself for catching recurs three times — reading the artifact that DOCUMENTS
an invariant instead of the code that ENFORCES it.

**LEG 1.** §5.1's re-validating definer cites `store.rs:576-593` as precedent. That is Rust-side
validator re-execution on an ordinary pooled read, not a `SECURITY DEFINER` with
`row_security = off`. The difference is the whole security property: check (1) gates the *party* on
`app.current_org`; nothing in §4.5's pseudocode gates the *grant rows*, whose only isolation is the
FORCE RLS the definer switches off by design (`0155:39`, armed at `:93-96`).

**LEG 2.** The definer's check (2) — re-chaining `prev_hash`/`row_hash` on every read — cannot be
implemented today, and the code says so in the imperative. `instances.rs:1297-1322` records that
`serde_json::Map` is insertion-ordered because cedar-policy-core enables `preserve_order`, that
`verify_chain` therefore reports breaks on untampered data for any bag with two or more keys, that
the fix is deliberately withheld pending a re-seal decision and an audit-chain owner, and — the
sentence that matters — "The suite is green because it does not recompute hashes — not because
recomputation would succeed." A `grant` bag (subject, capability, scope, source) has four keys. The
plan makes this a required Slice 0 acceptance probe with a GREEN baseline and a four-way deletion
test.

**LEG 3.** §5.8 retracts a definer in favour of `CHECK (before − split = after)`, citing
`0156:103`. The shipped conservation is `fetch_item_for_update_tx` + an advisory lock on the
idempotency key + a domain `state.consume()`; `0156:103` is the arithmetic backstop on top of that,
not the mechanism. Two concurrent splits of a 100-unit lot each writing (100, 60, 40) both satisfy
the CHECK and over-allocate by 20.

**LEG 4, the meta-leg.** §0.1 is the plan's one BLOCKING correction and its three citations do not
resolve: `:89-92` is a passage about `clearance_assignments`, `:545-546` is about `company` being
free text, `:575-579` is about roles-as-grant-bundles. The quoted text sits at `:116`, `:571`,
`:606`. The claim is TRUE at the new lines — I confirmed it — but the evidence path is broken, in
the very document the plan's author added a SUPERSEDED header to, and the same stale citations are
restated inside that header at `:11`. A preamble asserting "Line numbers re-verified this session"
over a BLOCKING item that does not resolve is asking the reader to extend an unearned reliability
to the 1,800 lines nobody will check.

**LEG 5.** And none of it can begin. 27/27 capabilities read HOLD on implementation, verification
and exposure; `hold_rule` fails closed on "empty Buck2 target sets"; the one recorded amendment
touches only the "missing backend contracts" clause. §8 schedules X8 to *explain* how the buck jobs
pass and never connects it to the clause pinning every capability. "Slice 0's Tier T half lands and
ships" (`:1688`) is not a claim this plan can make.

**What this does NOT establish:** that the entity model is wrong. It is not. The tier discipline is
real and structural — `0155:18` and `:78-79` correctly decide most of §4. The two silent-empty traps
are the plan's highest-value discovery and both verify. §5.7's split-by-question is the right call
for the right reason. The defects concentrate where the plan crosses a store boundary and where it
asserts a mechanism instead of citing one. That is fixable, not fatal.

*(Critic's addendum to LEG 5: it confirms harder than stated. The single `hold_rule_amendment`'s own
`limits` array says verbatim "Does not relax any other hold-rule clause: unassigned worktrees,
overlapping ownership, EMPTY BUCK2 TARGET SETS, stale reference digests and missing jurisdiction or
independent-review receipts still fail closed." The one amendment on the record explicitly
reaffirms the clause pinning all 27/27 capabilities, while `prelude/` is absent and `tools/buck2
test` runs in a required job.)*

### 3.2 The tension: replay versus aggregate, resolved by choosing — and denied twenty sections later

§5.7 wants every entity canvas-authorable and as-of replayable, and it wants `AVG(cycle_time)` over
10,000 rows. `list_projected_rows_tx` (`instances.rs:1522`) is a live read-through with `version`
always 1 and empty fixity hashes, and there are zero materialized views in the 205 migrations that
exist. The plan does not deny this and does not build the bridge. §0.14 retracts its own earlier
placement and moves `work` — the entity composing the most components, the declared join point for
artifacts, actions, handover, ledger and metrics — out of Tier N, accepting three real losses: no
fixity chain, no as-of replay, and (per `ProjectedDispatchRegistry`, one `HashMap` of Rust closures
failing closed on `NotWiredYet`) one hand-registered handler per action instead of free authored
behaviour. In exchange it avoids being the first materialized view in the repo and keeps replay
where it is load-bearing — authority, 결재, lineage. It pays for the choice with a probe
(`no_duplicated_fact`) rather than a paragraph. That is choosing, not denying.

The contrast is the tension's real edge. §5.4 faces the identical shape: an attribute-free `party`
buys erasure, zero new PIPA surface, and a Slice 0 that lands while every Korea control reads HOLD
— at the cost of any means of resolving one human to one `party` across orgs, which is the
duplication `party` exists to remove. The plan presents that as pure gain: "the cheaper end state,
not a staging posture" (`:1040`). Same author, same session; one tension named and priced, one
denied. The first fix this verdict asks for is that §5.4 pay the way §5.7 does.

---

## 4. Major and minor findings

Ordered. These do not gate Slice 0 but should be worked in the same revision pass; several become
blocking the moment the slice they touch is scheduled.

### Major

**M1 — §4.0.3; §5.11 G9; §7 `capacity_recorded_on_every_authority_mutation`.** The plan's
self-declared "highest-leverage change" is sized as DDL and its real cost is plumbing. Two nullable
columns on `audit_events` is correct as DDL and the `0149` precedent is exact. But the value must
reach the row: `AuditEvent` is a kernel struct with no capacity field
(`backend/crates/kernel/core/src/audit.rs:83-108`), and there are **466** non-test-directory
`with_audit` references under `backend/crates` (counted this session). Every one becomes a
populate-or-leave-null decision. "Authority mutation" — the set where null is a defect per
pre-mortem 4's leading indicator and per the probe — is never enumerated, and §5.11 G9 defers the
enumeration to Phase-7 prepwork. Pre-mortem 4 names exactly this failure and the plan's own
structure guarantees it. **Required:** move the G9 enumeration to Phase 0 and make it the artifact
the probe reads (one row per authority-mutating write path in `ecosystem-entity-components.tsv`);
then make capacity non-optional at those sites by construction — a distinct constructor
(`AuditEvent::authorized(…, grant_id)`) rather than a nullable field plus a gate — so the compiler
enforces what the gate would otherwise police across 466 sites.

**M2 — §4.0.2 the no-code boundary; §0.13; §4.8 E4; §1 principle 3; §4.4.** Three closed
vocabularies stand between §4.0.2's honest boundary and the owner's "manageable without
developers", and the plan surveys only one of them (`Feature` minting, §0.15, correctly).
(a) `AUTHORING_ACTIONS` is a five-element const — `view`, `edit`, `read_field`, `console:configure`,
`console:deploy` (`backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252`) — rejected
against at `:294-297` and again at `:714-720` inside `simulate_inner`. So an authored object policy
can never express a domain capability like `purchase.approve`, which bounds §0.13's resolution to
"a view permit" and bounds E4. The plan never mentions it. (b) `policy_role_conditions.attribute`
is a closed CHECK of **17** values (`0065_create_policy_roles.sql:110-127`: group, tenant,
organization, org, department, team, position, employment_status, assignment, location, site,
branch, device_posture, purpose, action, resource, sensitive_action) containing no 직무 and no
직급 — so two of the four dimensions §1 principle 3 declares "vocabulary" have no substrate to be
vocabulary in, and §4.1 adds no entity for either. Widening that CHECK is a migration, a third
closed vocabulary the plan does not name. *(One lens said 22 values; the count is 17. The substance
holds.)* **Required:** add a §4.0.2 requires-code row for the authoring-action vocabulary; restate
§0.13's resolution as "a `view` permit, which is all an authored policy can express"; qualify E4 so
the fold simulator is not assumed to inherit Cedar simulation for domain capabilities; then either
add 직무/직급 to §4.1 as authored types with their attribute keys and the migration widening the
CHECK, or state which of the four dimensions have no substrate in slices 0/1 and in which widening
they arrive.

**M3 — §5.11 G6 vs §8 W10.** Two sections contradict each other on a core owner requirement. G6
records the no-code canvas as deferred by an accepted ADR — verified at ADR-0023:153-154, "No-code
policy/workflow visual canvas (DESIGN §4.6 makes it the baseline; this program ships read-only NL
rows + simulation and defers the canvas)" — recommends accepting the deferral, and warns "Do not
smuggle it in". §8 then lists W10 "Canvas over the authored types" as an ordinary widening with no
gate on that decision. **Required:** mark W10 as gated on the G6 charter and state which of G6's
two options the plan recommends.

**M4 — §8 Phase 6 / Phase 5 ordering; X8, X9, X4, X5.** §8 is Bun-shaped in content but not in
order, and four of nine experiments do not count by the plan's own principle 5. Phase 6 is headed
"experiments (their own phase, before the design is trusted)" at `:1637` yet numbered after Phase 5
"one PR" at `:1622`, while its own contents say the opposite — X4 is "test it first" (`:1647`) and
Phase 7 says "X8 runs first" (`:1670`). Bun's 3-hour mapping and 3-file trial preceded all
conversion. Separately X8's prediction ("they pass by some mechanism that must be identified") is
unfalsifiable and its control is a fallacy rather than a runnable input; X9, X4 and X5 name a
refutation scenario rather than an input a probe can be observed RED on, which principle 5
(`:278-279`) forbids. **Required:** renumber so experiments precede the Phase-2 trial run, and
state the gate — no Slice-0 implementation commit until X1-X5 and X8-X9 have recorded outcomes in
`known-bad-controls.tsv`. Give X8/X9 one shared runnable control (add a test file with a
deliberately failing assertion, confirm CI goes RED, then fix it) and name the candidate mechanisms
X8 must discriminate between (path filter, continue-on-error, no-op required job, cached graph).
Restate X4 and X5 as constructed queries with expected-fail baselines.

**M5 — §8 Phase 3 / Phase 7 rung ①.** The lane→path reservation is asserted, not demonstrated. §8
references `fanout-plan-DRAFT` §5 and LANE-PROTOCOL's five-lane pool but never instantiates either
for the eight crates of Phase 3 or the eighteen widenings; the crate queue has no owner column, and
rung ① reads "each in files no other lane owns" — while `LANE-PROTOCOL.md:72-78`, which the plan
itself quotes, is precisely the warning that unproven ownership degrades to discipline, "and
discipline is what fails". W11-W13's "three lanes, no shared files" has the same shape.
**Required:** a Phase-0 artifact with one row per lane — crates, owned paths, migration slots
(0207+, since 0206 is in flight in lane-1), and the widenings it may take — with W11-W13 in that
table.

**M6 — §4.1 vocabulary "adopted, not invented"; §4.3; §8 Phase 0.** Phase 0's transcription will
build the wrong set from stale vocabulary documents, in three distinct ways, while Phase 0 declares
transcription risk-free. (a) `ReportingLine` is listed among the 14 adopted org primitives at
`:498` and then omitted from the Tier N table and from every row of §4.3, while the spec makes
position-to-position the preferred form with cycle and single-primary-path validation — so no
position hierarchy exists. (b) `docs/program/CATALOG.md:62-69` lists OrgUnit / Position / Person /
Employment / PayRun; what shipped is company / org_unit / job_position / employment / pay_run —
Person never landed, company is absent from the catalog. (c) §4.1 introduces a Tier N `position`
while `position` is already a seeded built-in stable_key
(`backend/crates/ontology/adapter-postgres/src/seed.rs:74`) and the shipped conformance type is
`job_position`, so a lane collides with the built-in. **Required:** add `reporting_line`
(position→position with the spec's cycle and primary-path validation) or state its exclusion and
defend it; add a Phase-7 item correcting `CATALOG.md:62-69` to the shipped set; state in
`ecosystem-PORTING.md` the stable_key mapping across {org-editor "Position", built-in `position`,
shipped `job_position`, this plan's 직책 type}, giving the plan's type a non-colliding key.

**M7 — §4.4 notices; §8 W1; §7 `obligation_notifies_line_as_raised`.** The obligation loop has no
audience targeting, so its headline probe cannot pass and its unfixed form is a confidentiality
regression. §4.4 names three `notices` gaps and W1 fixes all three — but the executable publish
path snapshots recipients as either every active user in the org, or every active user in the
notice's audience branches: two SQL variants keyed on `audience_scope == "branches"` at
`backend/crates/notices/adapter-postgres/src/lib.rs:413-433`, both `SELECT … FROM users WHERE
org_id = $1 AND is_active = true`. There is no per-recipient audience. So
`obligation_notifies_line_as_raised` (notify truncated member D specifically, though D never saw
the matter) cannot pass, and a 반려 notice fans out to every active org user on a 결재 matter.
**Required:** add per-recipient audience targeting as a fourth W1 gap with its DDL (an explicit
recipient list keyed by party) and make the probe assert that non-members receive nothing,
known-bad control the shipped org-wide snapshot.

**M8 — §5.2 B the delta list; §4.1 `approval_signature`.** A signature is bound to a document ID,
not a document state, so a legitimate post-signature amendment across a band leaves the signature
valid and no probe catches it: approved for ₩10M, edited to ₩100M, still approved. §4.1 stores
line-as-raised and line-as-executed, but nothing invalidates a signature when the document's amount
later crosses into another `delegation_rule` band. DN-0003's expected-revision/412 covers
concurrent writes, not a legitimate amendment. Slice 0's single step and single band cannot surface
it, and §5.2's delta table — the plan's complete claim of what it owns beyond ADR-0023 — does not
list it. *(Dropped during synthesis: the lens's secondary claim that §4.7 point 3's "real departure
from the common enterprise pattern" is factually false. Whether SAP checks synchronously changes no
code; the mechanism gap does.)* **Required:** add release-reset semantics to §5.2's delta list — a
signature is a statement about a document STATE, and a change crossing a `delegation_rule` band
invalidates signatures taken under the prior band and re-routes — with a probe whose known-bad
control keeps signatures valid after the amount is raised.

**M9 — §5.5 period locks; §5.2 finality; §8 W14.** W14 as written would refuse the compensating
posting and leave the obligation loop unclosable. §5.5 resolves the lock mechanics well (keyed on
DATE, the voucher has none, the lock does not enforce itself, finance-gl is not among the four
callers) and prescribes `accounting_date` plus the missing guard call. Neither §5.5 nor §5.2 decides
what happens when a post-확정 반려 arrives for a period already locked, and W14 pairs the
compensating voucher with "assert_period_open called from finance-gl" — two requirements that
contradict each other for exactly the case W14 exists to prove. **Required:** state in one place
whether 확정 requires an open period, and that a compensating voucher posts with an
`accounting_date` in the current open period while referencing the original. Add a probe for the
locked-period 반려 path.

**M10 — §5.5 economics, line-level typed object dimension; §9 ADR block; Slice 0.** A single-valued
`(source_object_type, source_object_id)` pair on the voucher line forecloses two answers the plan's
own "must not foreclose" list needs: real-versus-statistical account assignment (the same cost
REPORTED against several objects while OWNED by exactly one, resolving double-counting
declaratively) and analytic plans (one journal line carrying several independent dimensions with
percentage distribution). With a single-valued line dimension, a cost touching `work` and `lot` and
a contract must either post N lines (double-counting) or pick one and lose the others — while §5.5
promises allocation with a recorded basis. Slice 0 posts the first dimensioned line, which will
then be cited as evidence the shape is settled. **Required:** state whether one line may be
reported against more than one object; if not, record real-versus-statistical assignment and
percentage distribution as decisions the peer finance plan owns, and note that Slice 0's single
posted voucher is not evidence the dimension shape is settled.

**M11 — §4.5 handover; §5.9; Slice 0 `on_behalf_of_party_id`; W4.**
`on_behalf_of_party_id` lands in Slice 0 as a column and is exercised nowhere — no probe asserts
that 대리/대결 records both parties, and no slice or widening writes it. That is pre-mortem 4's own
named failure realised in the plan's own Slice-0 table. Relatedly, 연차 and 퇴사 appear zero times
in the plan: §4.5 gives the 인계 완료 query and the 대리/분배 mechanism but never decides that a
leave-based handover reverts automatically, and there is no revocation step at departure — while
§4.7 maps 대리/대결 to raid lead-and-assist and treats it as load-bearing. **Required:** add the
two handover modes to §4.5 as distinct operations (time-boxed reverting 대리 vs permanent transfer
+ grant revocation + 인계 완료 gate), and add probe `daeri_records_both_parties` with known-bad
control a 대리 signature with null `on_behalf_of`.

**M12 — §4.7 point 2; §4.8; §7 — the unclaimed differentiator.** `delegation_rule` as an
effective-dated Tier N type with as-of replay supplies something the benchmark has no equivalent
for: a single renderable artefact answering "this is our approval authority as of 2026-07-01",
where the comparison distributes it across customising tables, workflow scenarios, team definitions
and role assignments. A 전결규정 has legal force; if the system cannot render it, a spreadsheet
becomes the source of truth. But E1-E6 and every `slice0_*` probe are person-centric ("what could
this person approve"), never regulation-centric ("render the whole matrix as of D"). Separately
§4.7 restates the guild-bank shape as (role × amount band × category) → permitted, dropping the
PER-DAY limit its own first sentence carries, and `delegation_rule` has no periodic or cumulative
quota anywhere in §4.1 or §4.3 — while §4.7 asserts the guild-bank comparison "is a testable bar,
not a sentiment (§4.8)" and §4.8 contains no such test. **Required:** name it as a differentiator
in §4.7/§4.8 and add one probe — the complete 전결규정 (category × band × scope → competent unit,
terminal?) renders as one artefact as of an arbitrary date, known-bad control being routing
expressed only inside approval templates. Then either add a period/cumulative quota dimension to
`delegation_rule` or record dropping it as a decision with a reason, and give §4.8 the ergonomics
criterion §4.7 promises it.

### Minor

**m1 — §8 Phase 0 / Phase 7, external evidence.** DOWNGRADED from one lens's blocking (see §7).
The plan cites the external-research corpus zero times — grep for benchmark / research- / Foundry /
Workday / SAP / Odoo / NetSuite / ServiceNow / Salesforce returns 0 hits across 1,853 lines — while
§4.7 grants MMO games explicit evidentiary standing with a burden of proof and calls them "the
strongest available evidence" for the keystone. That asymmetry is real and cannot be defended as
evidentiary discipline. But its substantive payload is already promoted above (SoD → B12, the
correction axis → B11, release-reset → M8, multi-dimension lines → M10, `reporting_line` → M6), and
those are derivable without the corpus, so it does not by itself change what an implementer writes.
**Required:** one Phase-0 line — reconcile §4 and §5 against the benchmark matrix and the four
research surveys, recording which findings the plan adopts, rejects or contradicts, confidence
labels carried through, no plan decision resting on an UNCERTAIN/UNKNOWN row. Drop or qualify the
"strongest available evidence" claim at `:847`.

**m2 — §8 Phase 7, LANE-PROTOCOL as process authority.** §8 opens fanout under a protocol whose own
status header forbids it. `LANE-PROTOCOL.md:7` still reads "prep artifact, not yet exercised.
Fan-out is not authorized until §4 passes" after fan-out ran green and was promoted to a required
check; `:268-270` states the repo has no `[profile]` section and no sccache while `backend/Cargo.toml:359`
carries `[profile.dev]`/`[profile.test]` and sccache landed. **Required:** a Phase-7 item
correcting both, and cite the corrected header where §8 opens fanout.

**m3 — §1 principle 4; §3.1 — Tier P.** The plan breaks its own tier invariant within twenty lines.
Principle 4: "Every storage decision names one of the four tiers the CI gates already enforce
(§3.1). A new tier is a plan defect." §3.1 then introduces "A fifth path, Tier P — projected". Tier
P is not gate-enforced — it is a compiled-in match arm plus a `backing_kind` CHECK
(`instances.rs:1479-1498`) — so "All four are enforced by `tenant-isolation/src/lib.rs`" is true of
four and not of the fifth, which is where `work` and `worksite_registration` both land. It matters
because Phase 0's PORTING.md is meant to be looked up, not re-derived. **Required:** restate
principle 4 as "one of the four tiers, optionally projected" and say explicitly that projection is
code-gated not CI-gated, or drop the "Tier P" name.

**m4 — §5.11 reciprocal ADR pairs.** The pairs name only the new-ADR side, while
`docs/decisions/README.md:9` requires amendment to be explicit in BOTH records and `:26` requires
reciprocal relationship keys. **Required:** for G1, G2b and (if G8's integrity-vs-domain-logic
distinction is rejected) G8, name the counterpart record, the line amended, and the relationship key
to be added on both sides — including that ADR-0003 carries no `amended_by` key today, so the
reciprocation must create it, and that per B3's adjudication G1 may have no ADR-0022 counterpart to
amend at all.

**m5 — §4.8 E2, the character sheet.** The plan's self-declared completeness test has no delivery
vehicle and no executable form. E2 is named the completeness test ("an entity with no home on this
screen is a modelling smell") but appears in no slice and no widening — W17 ships E4, W18 ships E1,
W11 ships E6, E2 has nothing; Phase 3's only mention is "`app` | wiring, /overview surface".
`every_entity_declares_its_components` tests rows in a TSV, not homes on a screen. **Required:**
give E2 a widening with acceptance, and make the completeness test executable — one row per §4.1
entity mapped to its character-sheet section, with a probe that fails on an unmapped entity.

**m6 — §4.1 `assignment`; Slice 1.** No occupancy semantics for absence. `holds_position` is
ManyMany so a substitute's concurrent assignment is expressible, but nothing distinguishes the
absent holder's claim from the substitute's, and there is no return-right marker. 휴직/복직 is
absent from the plan while 육아휴직 복직 is a statutory right and HR+payroll is the first vertical,
so it will land on `assignment` regardless — as a reshape of a shipped Slice-1 entity rather than a
property addition now. **Required:** add an assignment kind (substantive / acting / seconded) and a
return-right marker as authored properties when Slice 1 defines the type.

**m7 — §2 driver 3; §5.4 preconditions — Korea release terms.** The HOLD is stated as a dependency
(correctly, and the plan does not plan to lift it) but its terms of release are not, so a later lane
cannot tell whether it has met the bar. §5.4 says "the Korea controls have moved off HOLD" without
naming the six controls, the qualified-authority requirement, or that native agents produce only
`I1_NON_INDEPENDENT` evidence while I2/I3 independent custody is required
(`docs/program/console-jurisdiction-register.json:1186` — "Missing, stale, conflicting, or
unqualified authority is HOLD; agents may not invent certainty"). **Required:** name the six
controls and the independent-evidence requirement in §5.4's precondition list.

**m8 — §4.3 relationships; §4.5 인계 완료 query.** Work-linked versus person-linked artifacts is
load-bearing and unmodelled. The 인계 완료 query relies on "∃ artifact linked to departing but to no
work" and `handover_moves_work_artifacts_only` asserts the boundary, yet §4.3 carries only
`work_artifact`; the person-endpoint edge is never declared, so the distinction the PII and handover
boundary rests on is inferable only from the probe. **Required:** add the person-linked artifact
edge to §4.3 with its cardinality and owner, and reference it from the query.

---

## 5. What the review confirmed as sound

Recorded so it is not re-litigated. Each of these was checked against executable code in this pass
or the two lens passes, not accepted on the plan's word.

1. **§0 as a method, applied to what exists.** §0.4, §0.8, §0.10, §0.11, §0.12, §0.13, §0.15,
   §0.16, §0.17 hold against non-obvious code — `sync_property_links_tx`, `residual.rs:200-203`,
   `list_projected_rows_tx`, `seed.rs`, `Cargo.toml`, and an independent count of materialized views
   (zero across all 205 migrations). §0.12's "a link type alone produces no edge, ever" and §0.13's
   "a published Tier N type lists EMPTY forever until a policy is attached" are the two
   silent-empty traps, and they are the plan's highest-value discovery.
2. **The four-tier discipline.** Structural and correctly derived from executable gates
   (`tenant-isolation/src/lib.rs` global/owner-only allowlists, `0155:18` org_id NOT NULL, `:90-98`
   FORCE RLS). It correctly decides most of §4. The one defect is the unlabelled fifth path (m3),
   not the discipline.
3. **§3's rejections.** Argued with executable evidence, not strawmanned. Option 4 dies on `0076`'s
   nullable column, the `HAVING count(*) = 1` backfill, and `identity_resolution_confidence`. Option
   2's `app.current_group` cost across 141 RLS tables is the retracted option's cost, correctly
   attributed.
4. **§5.7's split-by-question.** The right call for the right reason, and the model of how this plan
   should decide: it retracts its own §4.1 placement, names three concrete losses, and pays with a
   probe rather than a paragraph. §0.14 is the plan behaving exactly as its own §0 demands.
5. **§6 pre-mortem.** The opposite of ceremonial. All five scenarios carry leading indicators
   observable BEFORE failure and machine-checkable: a grep for definers lacking a literal
   `current_setting` predicate; any null `authorizing_grant_id`; a published type with zero attached
   policies. Scenarios 1, 4 and 5 each predicted a defect this review then confirmed independently
   — which is the strongest available evidence the pre-mortem is real.
6. **§7's known-bad-control discipline.** Essentially every probe is paired with a known-bad
   control, and `known-bad-controls.tsv` requires a recorded RED commit per probe. That is the
   correct analogue of Bun's "60,624 tests, 0 skipped, manually verified as actually running", and
   it is the mechanism that would have caught B1 and B4 had it been applied to §5's own claims.
7. **No automatic-REJECT trigger fires.** No gate is weakened — Phase 1 forbids it,
   `no_new_gate_classification` probes it, and
   `scripts/check-command-database-wiring.test.mjs:106` (`assert.doesNotMatch(prod,
   /components:|pr-473|governed-command-database/)`) is correctly read as the gate that flips if the
   ontology deployment gap is closed. No production exposure is widened. No Korea conclusion is
   asserted. No I2/I3 evidence is claimed.
8. **§4.0.3's DDL judgement.** Two nullable columns on `audit_events` with the `0149` precedent is
   the right shape; only the sizing and the enumeration are wrong (M1).
9. **§5.5's period-lock diagnosis.** The mechanics are resolved correctly — keyed on DATE, the
   voucher has none, the lock does not enforce itself, finance-gl is not among the four callers.
   Only the 반려 interaction is undecided (M9).
10. **The entity model is not wrong.** Both lenses converge here independently. The defects
    concentrate in one place — wherever the plan crosses a store boundary or asserts a mechanism
    instead of citing one — which is why this is one revision pass, not a redesign.

---

## 6. Revision list for the planner

Ordered. Each item names the § to change and what the change must achieve. Wording is the planner's.

**Before anything else — the evidence path (B10, M4).**
1. Re-anchor §0.1's three citations to `authority-and-approval-model.md:116` / `:571` / `:606`, and
   fix the same three inside that document's own SUPERSEDED header at `:11`.
2. Convert every cross-document citation into a document this plan also edits from a line number to
   a quoted-text anchor, per `fanout-plan-DRAFT.md:243`. Sweep, do not spot-fix.
3. Restate every repo-wide negative as "all 205 migrations in the main checkout as of `<commit>`",
   naming the commit; reserve migration slots from 0207.

**Then the four §5 mechanisms that cannot be built as written (B1, B2, B4, B8).**
4. §5.1: delete the re-chaining check; choose linkage-only or a named Phase-0 canonicalization
   prerequisite with an audit-chain owner; correct the four-way count in §7 and Slice 0.
5. §5.1/§4.5: make a literal `org_id = current_setting('app.current_org')::uuid` predicate the
   definer's FIRST check; add `definer_returns_no_foreign_org_grant`; state what
   `store.rs:576-593` and `group_role_grants_for_user` are and are not precedents for.
6. §5.1: restate genesis as a platform-principal capability gated on
   `PlatformFeature::TenantCreate`, with the onboarding seeder as the extension point; re-scope the
   probe; note the SUPER_ADMIN write site in C4b/G2b.
7. §5.8: add the FOR UPDATE parent-lot lock and same-transaction quantity update; keep the CHECK as
   backstop; add the concurrent-split probe; decide whether `lot.quantity_milli` is authoritative;
   one table name.

**Then the model gaps that decide Slice 0's shape (B3, B9, B6, B7, B11, B12).**
8. §4.1/§3.2/§5.4/§5.11 G1: state the party-resolution mechanism and pick one of the three options;
   correct G1's ADR-0022 reading; rewrite §5.4's recommendation to price its loss the way §5.7
   prices its three.
9. §4.3/§4.1: replace the `ont_link` arms for `organization` and `group` with a scope descriptor
   property; split `grant` by scope level with group-scope in Tier O; correct §9's cost line; extend
   X4 with the cross-org falsifying case.
10. §4.0.2/§4.6: add the `ProjectedDispatchRegistry` requires-code row; enumerate `work`'s handlers
    with a count; decide DN-0003 invariant 1 for Tier T/P with a probe.
11. §5.6/§4.6: add the `authz_subject_version` bump with its ADR-0021 decision-5 citation and probe;
    declare the three new Cedar attributes as a bundle-schema change that must land before any code
    reads the fold.
12. §4.1/§5.9/§2: decide the correction-versus-new-effective-change axis before Slice 1, or defer it
    with the consequence named; qualify driver 2.
13. §1 principle 2 / §5.11: decide SoD in or out, citing `cedar-pbac-authorization.md:122` either
    way.

**Then admissibility (B5, M5, M4-ordering, m2).**
14. §8 Phase 7: add the `hold_rule` Buck2-clause rung, quoting the existing amendment's `limits[0]`;
    add capability-registry rows for Slice 0 and each widening group; retract `:1688`.
15. §8 Phase 0: produce the lane→path/crate/migration-slot/widening table so rung ① is demonstrated;
    include W11-W13.
16. §8: renumber so experiments precede the Phase-2 trial run and gate the first implementation
    commit on recorded X1-X5 and X8-X9 outcomes; give X8/X9 a runnable shared control; restate X4
    and X5 as constructed queries with expected-fail baselines.
17. §8 Phase 7: correct `LANE-PROTOCOL.md:7` and `:268-270`, and cite the corrected header.

**Then the sized-wrong and unnamed items (M1, M2, M3, M6, M7, M8, M9, M10, M11, M12).**
18. §4.0.3/§5.11 G9: move the authority-mutation enumeration to Phase 0 as the artifact the probe
    reads, and make capacity non-optional by constructor rather than by gate across 466 sites.
19. §4.0.2/§0.13/§4.8: add the authoring-action vocabulary row; restate §0.13 as a `view` permit;
    qualify E4; resolve 직무/직급 substrate or state the deferral and its widening.
20. §8 W10: gate on the G6 charter and state the recommended option.
21. §4.1/§4.3/§8 Phase 0: resolve `reporting_line`, the `CATALOG.md:62-69` correction, and the
    `position` / `job_position` stable_key collision in `ecosystem-PORTING.md`.
22. §4.4/§8 W1: add per-recipient audience targeting as a fourth gap with DDL, and make the
    obligation probe assert non-members receive nothing.
23. §5.2: add release-reset on band crossing to the delta list, with its probe.
24. §5.5/§8 W14: decide the locked-period 반려 rule in one place and probe it.
25. §5.5: decide whether one voucher line may be reported against more than one object; record
    real-versus-statistical assignment and percentage distribution as the finance plan's; note Slice
    0 settles nothing here.
26. §4.5: split 대리 (time-boxed, reverting) from 전보 (permanent, with revocation and 인계 완료),
    cover 연차 and 퇴사, and probe `daeri_records_both_parties`.
27. §4.7/§4.8: name the 전결규정-as-rendered-artefact differentiator and probe it; resolve the
    dropped per-day/cumulative quota; give §4.8 the criterion §4.7 promises.

**Then the remaining minors (m1, m3, m5, m6, m7, m8).**
28. §8 Phase 0: one line reconciling §4/§5 against the benchmark matrix and the four surveys, with
    confidence labels carried through; qualify `:847`.
29. §1 principle 4 / §3.1: label projection as code-gated, not a fifth CI-enforced tier.
30. §4.8 E2: give it a widening and an executable completeness mapping.
31. §4.1 `assignment`: add assignment kind and return-right marker for Slice 1.
32. §5.4: name the six Korea controls and the I2/I3 independent-custody requirement.
33. §4.3: declare the person-linked artifact edge and reference it from the 인계 완료 query.

---

## 7. Synthesis notes — what was corrected, downgraded or dropped

Recorded because a finding's provenance matters to whoever acts on it.

**Corrected during synthesis.**
- `policy_role_conditions.attribute` CHECK: one lens said 22 values; the count is **17**
  (`0065:110-127`). Substance unchanged (M2).
- The inventory migration is `0156_create_inventory.sql`, not `0156_create_inventory_consumption.sql`
  (B8).
- `ont_links`' endpoint FKs are at `0155:76-77` (one lens said `:78-79`); both endpoints reference
  `ont_instances(id, org_id)` either way (B9).
- The plan's "206 migrations" is 205 in the main checkout; the 206th directory entry is `BUCK`, and
  `0206` is in flight in lane-1 (B10).
- `AuditEvent` is declared at `kernel/core/src/audit.rs:83`, not `:80`/`:81`; it has no capacity
  field either way (M1).
- The required Buck2 job is at `.github/workflows/ci.yml:163-164` with `tools/buck2 test` at `:192`
  (B5).
- §5.11 G1's ADR-0022 citation over-reads the ADR. Its Decision block rejects a speculative external
  IdP seam and confines `console-identity-application` to local commands; it never decides identity
  is org-scoped. This makes B3's fix cheaper, not the finding weaker.

**Downgraded.**
- The external-research-corpus finding: blocking → **minor** (m1). Its substantive payload is
  already promoted as B12, B11, M8, M10 and M6, and the lens conceded those are derivable without
  the corpus. It changes a Phase-0 line item, not what an implementer writes.
- DN-0003 invariant 1 for Tier T/P: blocking → **major**, folded into B6 as its secondary half. The
  invariant reads "not the NORMAL operational write path", which is weaker than the plan's
  paraphrase. The `ProjectedDispatchRegistry` half of B6 is unaffected and remains blocking.

**Dropped.**
- The claim that §4.7 point 3's "real departure from the common enterprise pattern" is factually
  false. Whether the benchmark checks authority synchronously changes no code; the release-reset
  mechanism gap does, and it survives as M8.

**Re-verified in this pass** (read-only; no build or test was executed, per the environment rule):
`instances.rs:1297-1322` and `:1346-1357`; `ontology/rest/src/lib.rs:160-195`;
`platform-rest/src/lib.rs:235`, `:568`, `:574`; `authoring.rs:246-252`;
`0065_create_policy_roles.sql:110-129`; `0155_create_ontology_instances.sql:18`, `:53`, `:57-58`,
`:76-77`, `:90-98`; `0156_create_inventory.sql:103`;
`notices/adapter-postgres/src/lib.rs:413-433`; `kernel/core/src/audit.rs:83`; the 466 non-test
`with_audit` references; 205 `.sql` migrations and zero materialized views;
`console-capability-registry.json` `hold_rule`, `hold_rule_amendment.limits[0]`, and 27/27 truth
blocks; `console-program-ledger.md:823`; `ADR-0023:153-154`;
`cedar_pbac/engine.rs:365-379` and `:449`; `cedar-pbac-authorization.md:122`;
`tenant-isolation/src/lib.rs:48` and `:121-124`; `ci.yml:163-164`, `:192` and the absence of
`prelude/`; `check-command-database-wiring.test.mjs:106`;
`authority-and-approval-model.md:3`, `:11`, `:116`, `:571`, `:606`.

**Not established, stated as such.** No claim in this record rests on the ontology deployment gap
being closable, on the Korea HOLD being liftable, or on the buck2 graph being repairable within this
plan's scope. B5 asserts only that the plan does not address the clause that pins every capability —
not that the clause can be satisfied.
