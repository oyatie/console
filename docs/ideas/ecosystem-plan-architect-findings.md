# Architect findings on the ecosystem plan

> `Status: ARCHITECT REVIEW — verdict SOUND_WITH_FIXES. The Critic pass did not run (quota
> exhaustion 2026-07-29) and is being re-run; the plan stays PENDING APPROVAL.`
>
> Three lenses reviewed `ecosystem-plan-DRAFT.md`: architectural soundness, fidelity to the
> external research, and completeness against an independent requirements checklist written
> before the plan existed. 69 deduplicated findings.

## BLOCKING (12)

### §8 Phase 0 / Phase 7 (prepwork before fanout)

The plan cites the external-research corpus zero times. `grep -c` over the plan returns 0 for "benchmark", "research-", "Foundry", "Workday", "SAP", "Odoo", "NetSuite", "ServiceNow", "Salesforce" — 20 corpus files (~570 KB) plus four sourced surveys of exactly this plan's subjects (org model, authority, approvals, work, economics, lineage) are never opened. Meanwhile §4.7 grants MMO games evidentiary standing with an explicit burden of proof ("the game's shape is prior art and the burden is on us to justify deviating", plan:825-826), and :847 calls games "the strongest available evidence" for the keystone. So the plan does not reject external evidence; it uses one unsourced body and skips the sourced one. Phase 0 then forecloses the reconciliation step by declaring both reference documents "transcription, not design" (plan:1568), and Phase 7's prepwork table (plan:1664-1672) has no research line item. By the plan's own sequencing, Phase 0 and Phase 1 gate Phase 2 (Slice 0), so the missing item blocks Slice 0. One reading pass produced the six other findings below; the plan's own §0 standard ("reviewing it against executable code finds nine more") applied to research would have produced them too.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:825-826, :846-848, :1568, :1664-1672`
- **Required change:** Add one Phase-0 prepwork line item: reconcile §4 and §5 against `docs/program/benchmark-matrix/INDEX.md` + `lenses/data-model.md` + `lenses/governance.md` + `appr.md` + `compliance.md` and the four `docs/ideas/research-*.md` / `foundry-reference.md` surveys, with the deltas below as its minimum contents, and record which corpus findings the plan adopts, rejects, or contradicts. Confidence labels must be carried through: no plan decision may rest on an `UNCERTAIN`/`UNKNOWN` row.

### §4.6 (Actions dispatch, they do not bypass) vs §5.11 (DN-0003 inherited invariants) vs §4.1 (`work` is Tier T, projected)

Slice 0 ships `work` as a Tier T table projected into the ontology (plan:536-540). That is exactly Foundry's third write path, which the research establishes as CONFIRMED and which both the corpus and the plan miss: object property values change through the datasource with "no Action submitted, no submission criteria evaluated, and no action-log entry", so "'all mutation flows through one audited verb' is not achievable by locking actions alone if a pipeline also writes the same object type" (foundry-reference.md:46-48, :184-186; corpus binary framing at object-platform.md:102). The plan asserts "Actions dispatch, they do not bypass" (plan:801) and lists DN-0003 invariant 1 — every consequential mutation is an Action, direct property edits are not the write path — as inherited and binding (plan:1381-1384). But a `work` row's assignee, due_at or realized_start is mutable by ordinary domain SQL, and the projected read is a live read-through with `version` always 1 and empty fixity hashes (plan:115-117 citing `instances.rs:1522`). ADR-0002's audit-coverage gate covers the *audit* half; nothing covers the Action-verb half or the history half. The conflict is unstated, and Slice 0 is where it lands.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:801, :536-540, :115-117, :1381-1384; docs/ideas/foundry-reference.md:46-48, :184-186`
- **Required change:** State how DN-0003 invariant 1 is satisfied for Tier T/Tier P entities before `work` is built: either every consequential `work` mutation is an `ont_action_types` dispatch (and say how, given projected types have no owned revision store), or record it as an explicit bounded exception naming the gate that holds it and the history it forfeits. Add the corresponding probe to §7 with its known-bad control.

### §5.1 A — the read circle; §4.5 traversal

The definer's grant read has no org predicate, so it returns another tenant's grants for any party legitimately visible in two orgs. §4.5's pseudocode is `grant instances WHERE subject = p_party AND scope ⊇ p_scope AND asof ∈ [...]` executed with `row_security = off`. Grant instances are `ont_instances`/`ont_instance_revisions` rows whose ONLY isolation is FORCE RLS (`org_id UUID NOT NULL` at 0155:18 and :39, RLS armed at :82ff) — the definer switches that off by design. Check (1) gates the *party* on `app.current_org`; nothing gates the *grant rows*. For a contractor visible in orgs A and B — the exact case §4.2 exists to serve — org A's call passes check (1) and then reads B's grant revisions. Only check (3) ("asserting every returned scope is inside the armed org's reachable scope set") stands in the way, and it is the vaguest of the four; for a group-scope grant the "reachable scope set" arguably includes it. This is pre-mortem 1 realised through a path pre-mortem 1 does not name, and the plan's own probe `definer_ignores_parameter_org` tests the party parameter, not this. Slice 0 ships this definer (Phase 3 crate #1; Slice 0 table row `effective_grants_for`).

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:730-737 against backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:18,39`
- **Required change:** Add an explicit fifth check and make it the first one: every grant/revision row read inside the definer is filtered `AND org_id = current_setting('app.current_org')::uuid`, literal not parameter. Add probe `definer_returns_no_foreign_org_grant`: one party with a visibility edge in BOTH orgs and a grant in each; org A's call must return exactly one. Its known-bad control is the definer as §4.5 currently specifies it.

### §5.1 A — payment term (2); §4.6; §7 `definer_revalidation_each_check`

Re-chaining `prev_hash`/`row_hash` on every read cannot be implemented today, for two independent reasons, and the plan makes it a required Slice 0 acceptance probe with a GREEN baseline. (a) `verify_chain` false-positives on untampered data: `revision_row_hash` hashes `serde_json::to_vec` of a `Value` whose `attributes` bag is read back from jsonb, and `serde_json::Map` is insertion-ordered because cedar-policy-core enables `preserve_order`, which reaches the ontology adapter through `console-platform-authz`. The code says so and says the fix was deliberately withheld pending a re-seal decision: "The suite is green because it does not recompute hashes — not because recomputation would succeed." A `grant` bag (subject, capability, scope, source) has 4+ keys, so it is squarely in the biting case. (b) The hash is Rust-side SHA-256 over serde_json bytes; a plpgsql `SECURITY DEFINER` cannot compute it at all, so the check cannot live where §5.1 puts it. This is verified against the dependency's own manifest, not against the code comment.

- **Evidence:** `backend/crates/ontology/adapter-postgres/src/instances.rs:1297-1322 and :1345-1356; /Users/jasonlee/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cedar-policy-core-4.11.2/Cargo.toml:148-150 (`[dependencies.serde_json] features = ["preserve_order"]`); backend/crates/ontology/adapter-postgres/Cargo.toml:11`
- **Required change:** Delete check (2) from the definer and from Slice 0. Either (i) make Slice 0's fixity assertion chain LINKAGE only (`prev_hash == predecessor.row_hash`), which is what company_conformance.rs already does and which SQL can do, and record the reason; or (ii) declare the canonicalization fix (sort keys explicitly + re-seal) an explicit Phase-0 prerequisite with an audit-chain owner. Do not list `definer_revalidation_each_check` as a four-way probe until the count of implementable checks is settled.

### §4.1 Tier O `party`; §5.4 D; §5.11 G1

No mechanism is specified by which two orgs arrive at the same `party` row — and the plan closes every available route itself. `party` is `(id, party_kind, status, created_at)` "and nothing else", with "Recommendation: never put personal attributes on party" as a permanent invariant. Option 4 (matching service) is rejected as "a guess". §4.7's account/character analogy resolves identity by the account *authenticating*, which ADR-0022:35-38 confines to local passkey accounts and org/account administration. A grep of the plan for any resolution, dedup or discovery mechanism returns nothing. Consequence: the duplication `party` exists to remove is not removed — org B onboarding the same human mints a second `party`, reproducing `users`/`employees` one tier up, and the confidentiality property becomes vacuous because there is nothing to be confidential about. This blocks Slice 0 by the plan's own rule: G1 is listed as blocking Slice 0, and G1 is an ADR whose entire claim is "one durable identity per natural or legal person, across every tenant and vertical".

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:510-512, :376-387, :1038-1040 against docs/decisions/ADR-0022-local-identity-only.md:35-38`
- **Required change:** State the resolution mechanism in §4.1 before G1 is drafted, and pick one explicitly: (a) party is minted per passkey credential and self-linked by the human at second-org onboarding (name the endpoint and the ADR-0022 delta this needs), or (b) party resolution is an owner-tier operator action with an audit record, never a tenant capability, or (c) narrow G1's claim to "one identity per org-cluster" and accept that cross-group dedup is out of scope — in which case say so in §3.2's Option-4 rejection, which currently rejects an option for a weakness the recommended option shares.

### §8 Phase 7 — prepwork; §8 deployment dependency

MISSING: the plan never mentions the capability registry or its hold rule, and asserts "slice 0's Tier T half lands and ships". All 27 capabilities carry truth {implementation: HOLD, verification: HOLD, exposure: HOLD}, and the hold rule fails closed on "empty Buck2 target sets" — a clause the 2026-07-25 amendment explicitly declines to relax while `prelude/` is absent. §8 flags build-system governance as unresolved and schedules X8 to explain how the CI buck jobs pass, but never connects either to the clause that pins every capability at HOLD on implementation. No prepwork item resolves it, so there is no admissible path from this plan to a first commit.

- **Evidence:** `docs/program/console-capability-registry.json:7 (hold_rule), :7173 (amendment limits retain the Buck2 clause), 27/27 capability truth blocks = HOLD; docs/ideas/ecosystem-plan-DRAFT.md:1688 ("lands and ships"), :1674-1682 (build-system governance, no hold-rule mention)`
- **Required change:** Add a Phase-7 item that resolves the hold_rule Buck2 clause explicitly — either an amendment through the same lane/amendment mechanism as `hold_rule_amendment`, or a recorded finding that the clause is satisfiable for these targets — and retract the "lands and ships" claim in favour of "CI-provable, exposure remains HOLD".

### §8 Phase 7 — prepwork

MISSING: no program-registration item. Zero of the eighteen desired concepts are registered as work, and the ledger states "Nothing in the idea document is approved work." The registry's dispatch rule admits a row to in_progress only with one writer, an isolated worktree, disjoint ownership roots, a signature story, evidence paths and executable leaf gates. Phase 7 supplies ownership rungs and a pre-reservation commit but no registry row, no signature story and no evidence path for slice 0, so slice 0 cannot enter in_progress even once the ADRs land.

- **Evidence:** `docs/program/console-program-ledger.md:823; docs/program/console-capability-registry.json dispatch_rule; docs/ideas/ecosystem-plan-DRAFT.md:1664-1672 (Phase 7 table — no registration row)`
- **Required change:** Add a Phase-7 rung item: register slice 0 (and each widening group) as capability rows with signature story, evidence_path, leaf commands and ownership roots, and state which of the eighteen concepts each row covers.

### §5.1 A payment term (2); §4.6; §7 `definer_revalidation_each_check`; Slice 0 `effective_grants_for`

Re-chaining `prev_hash`/`row_hash` on every definer read cannot be implemented today, and the code documents both reasons and its own deliberate non-fix. (a) `revision_row_hash` hashes `serde_json::to_vec` of a `Value` whose `attributes` bag is read back from jsonb; `serde_json::Map` is insertion-ordered because cedar-policy-core 4.11.2 enables `preserve_order`, reaching this crate via `console-platform-authz`, so `verify_chain` false-positives on untampered data for any bag with 2+ keys. A `grant` bag (subject, capability, scope, source) has four. The comment states the fix is withheld pending a re-seal decision and an audit-chain owner, and that "The suite is green because it does not recompute hashes — not because recomputation would succeed." (b) The hash is Rust-side SHA-256 over serde_json bytes; a plpgsql `SECURITY DEFINER` cannot compute it at all, so the check cannot live where §5.1 puts it. The plan nevertheless lists it as one of four required Slice 0 checks with a GREEN baseline and a four-way deletion test. Verified independently against the dependency manifest, not the comment. Highest-confidence finding in this review.

- **Evidence:** `backend/crates/ontology/adapter-postgres/src/instances.rs:1297-1322 and :1346-1357; docs/ideas/ecosystem-plan-DRAFT.md:931-932, :1486, :1707`
- **Required change:** Delete check (2) from the definer and from Slice 0's `effective_grants_for` row. Then pick one on the record: (i) Slice 0's fixity assertion is chain LINKAGE only (`prev_hash == predecessor.row_hash`) — what `company_conformance.rs` already does and what SQL can do — with the reason recorded; or (ii) the canonicalization fix (explicit key sort + re-seal) becomes a named Phase-0 prerequisite with an audit-chain owner. Stop describing `definer_revalidation_each_check` as four-way until the count of implementable checks is settled.

### §5.1 A — the read circle; §4.5 traversal; §7 `definer_ignores_parameter_org`

The definer's grant read has no org predicate, so it returns another tenant's grant revisions for any party legitimately visible in two orgs — the exact case §4.2 exists to serve. §4.5's pseudocode is `grant instances WHERE subject = p_party AND scope ⊇ p_scope AND asof ∈ [...]` executed with `row_security = off`. Grant instances are `ont_instances`/`ont_instance_revisions` rows whose ONLY isolation is FORCE RLS on `app.current_org`. Check (1) gates the *party*; nothing gates the *grant rows*. Only check (3) ("every returned scope is inside the armed org's reachable scope set") stands in the way, and it is the vaguest of the four — for a group-scope grant the reachable set arguably includes it. This is pre-mortem 1 realised through a path pre-mortem 1 does not name, and the plan's own probe tests the party parameter, not this. Slice 0 ships this definer.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:730-737, :927-934 against backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:18,39 (org_id NOT NULL) and :93-96 (ENABLE/FORCE RLS + org_isolation policy)`
- **Required change:** Add a fifth check and make it the FIRST: every grant/revision row read inside the definer is filtered `AND org_id = current_setting('app.current_org')::uuid`, literal not parameter. Add probe `definer_returns_no_foreign_org_grant` — one party with a visibility edge in BOTH orgs and a grant in each; org A's call must return exactly one — with the known-bad control being the definer as §4.5 currently specifies it.

### §4.1 Tier O `party`; §3.2 Option 4; §5.4 D; §5.11 G1

No mechanism is specified by which two orgs arrive at the same `party` row, and the plan closes every available route itself. `party` is `(id, party_kind, status, created_at)` "and nothing else", with "never put personal attributes on party" as a permanent invariant. Option 4 (matching) is rejected because "a mechanism that must decline the ambiguous majority is not an identity". §4.7's account/character analogy resolves identity by the account AUTHENTICATING. I grepped the plan for any resolution, dedup, self-link or discovery mechanism: the only hits are Option 4's rejection. Consequence: org B onboarding the same human mints a second `party`, reproducing `users`/`employees` one tier up, and the confidentiality property becomes vacuous. This blocks Slice 0 by the plan's own rule — G1 is listed as blocking Slice 0 and G1's entire claim is "one durable identity per natural or legal person, across every tenant and vertical". ADJUDICATION: the soundness lens treats ADR-0022 as confining self-linking to local org/account administration. I read ADR-0022 and its Decision text is narrower than either the lens or the plan assumes — it decides against a speculative EXTERNAL IdP seam and confines `console-identity-application` to local commands; it never decides identity is org-scoped. A platform-level `party` is still LOCAL identity. So ADR-0022 obstructs option (a) less than §5.11 G1 claims, which makes the fix cheaper — but the missing mechanism is the defect and it stands on its own.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:510-512, :376-387, :1038-1040 (grep for resolve/dedup/self-link/same party returns only :376-387 and :630); docs/decisions/ADR-0022-local-identity-no-external-idp.md Decision block (external-IdP seam, local org/account administration)`
- **Required change:** State the resolution mechanism in §4.1 BEFORE G1 is drafted, and pick one explicitly: (a) `party` is minted per passkey credential and self-linked by the human at second-org onboarding — name the endpoint, and note ADR-0022 permits it since this is local identity, not federation; (b) party resolution is an owner-tier operator action with an audit record, never a tenant capability; or (c) narrow G1 to "one identity per org-cluster" and accept cross-group dedup as out of scope — in which case say so in §3.2, which currently rejects Option 4 for a weakness the recommended option shares.

### §8 Phase 7 prepwork; §8 deployment dependency (:1688)

There is no admissible path from this plan to a first commit, and the plan never mentions the two artifacts that make it inadmissible. (a) All 27 capabilities carry truth {implementation: HOLD, verification: HOLD, exposure: HOLD}, and `hold_rule` fails closed on "empty Buck2 target sets" — the single recorded `hold_rule_amendment` (2026-07-25, lane L-P0-EPOCH) amends ONLY the "missing backend contracts" clause, so the Buck2 clause survives while `prelude/` is absent. §8 schedules X8 to explain how the buck jobs pass and flags build-system governance as open, but never connects either to the clause pinning every capability. (b) Zero of the eighteen desired concepts are registered as work; the ledger says "Nothing in the idea document is approved work"; the registry's dispatch_rule admits a row only with one writer, isolated worktree, disjoint ownership roots, a signature story, evidence paths and executable leaf gates. Phase 7 supplies ownership rungs and a pre-reservation commit and no registry row. Merged from two completeness findings because there is one prepwork rung to add, not two.

- **Evidence:** `docs/program/console-capability-registry.json hold_rule ("empty Buck2 target sets … fail closed"), hold_rule_amendment (amends the missing-backend-contracts clause only), 27/27 truth blocks HOLD; docs/program/console-program-ledger.md:823; docs/ideas/ecosystem-plan-DRAFT.md:1664-1672, :1688`
- **Required change:** Add one Phase-7 rung with two items: (1) resolve the hold_rule Buck2 clause explicitly — an amendment through the same lane/amendment mechanism as `hold_rule_amendment`, or a recorded finding that the clause is satisfiable for these targets; (2) register Slice 0 and each widening group as capability rows with signature story, evidence_path, leaf commands, ownership roots, and which of the eighteen concepts each covers. Retract ":1688 lands and ships" in favour of "CI-provable; exposure remains HOLD".

### §4.0 / §4.0.2 / §0.14 / §4.6 ("Actions dispatch, they do not bypass") vs §5.11 DN-0003 invariant 1

Two lenses found this from opposite sides and together it invalidates the plan's own frame for the entity it matters most for. (a) §4.0 claims "the systems light up for it without anyone hand-writing an integration per concern". `ActionDispatch` has two arms; the `projected_usecase` arm routes through `ProjectedDispatchRegistry`, a `HashMap<String, ProjectedHandler>` of Rust closures, "Empty by default ⇒ every projected dispatch fails closed (NotWiredYet)", populated one closure at a time by the App composition root. So EVERY action on a projected type is a hand-written integration. §0.14 just moved `work` — Tier T + projected, the declared join point for artifacts, actions, handover, ledger and metrics, the entity composing the most concerns — into exactly that tier. §4.0.2 lists projected backing as "one arm in allowlisted_projected_table", understating it by the whole handler registry. (b) A `work` row's assignee, due_at or realized_start is also mutable by ordinary domain SQL, and the projected read is a live read-through with `version` always 1 and empty fixity hashes — so DN-0003 invariant 1 (every consequential mutation is an Action; direct property edits are not the write path), which §5.11 inherits as binding, has no mechanism holding it for Tier T/P. ADR-0002's gate covers the audit half; nothing covers the Action-verb half or the history half. Blocking because Slice 0 ships `work`.

- **Evidence:** `backend/crates/ontology/rest/src/lib.rs:160-195 (ProjectedHandler type, HashMap, NotWiredYet fail-closed); backend/crates/ontology/domain/src/lib.rs:213-218; docs/ideas/ecosystem-plan-DRAFT.md:402-406, :455, :536-540, :801, :1381-1384, :115-117`
- **Required change:** Add a row to §4.0.2's requires-code column: "actions on a projected type — one ProjectedDispatchRegistry handler per action, registered in the App composition root; unwired = NotWiredYet". Enumerate `work`'s Slice-0/W4/W11/W13 actions with a handler count in `ecosystem-entity-components.tsv` so the Phase-3 `app` crate row is sized rather than labelled "wiring". Then state how DN-0003 invariant 1 is satisfied for Tier T/P before `work` is built — either every consequential `work` mutation is an `ont_action_types` dispatch, or it is a bounded exception naming the gate that holds it and the history it forfeits — with a probe and known-bad control.

## MAJOR (42)

### §1 Principle 2 (additive grants only) · §5.1 · §5.11

The plan's fold is positive-only by principle ("Additive grants only… Revocation closes a validity interval; it never writes a deny", plan:271-273), and pre-mortem 2 defends that property hard. The research says this is precisely SAP's model and names the consequence with CONFIRMED sourcing: because PFCG is positive-only and additive and roles accumulate, users drift into combinations "individually legitimate and jointly dangerous", nothing in the core detects it, and SAP therefore sells a separate product — "SoD is an emergent property of an authorization system, not a feature of it" (research-sap.md:246-263), rejected explicitly at :921-925 ("mutual exclusion belongs in the authorization model. Detecting violations after the fact means they existed"). The corpus independently ranks a SoD *ruleset* as finding #1 of its top-10 governance findings, "L, highest governance ROI", touching appr/finance/people/compliance, and draws the exact line the plan sits on: "We block one person approving their own request; we don't detect 'this person holds two roles that together enable fraud'" (lenses/governance.md:155). The plan's only SoD content is the shipped four-eyes `CHECK (approved_by <> created_by)` (plan:1056). Mutual exclusion does not require a deny in the fold — it is a constraint at grant-authoring time — so Principle 2 is not in tension with it.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:271-273, :1056; docs/program/benchmark-matrix/lenses/governance.md:155; docs/ideas/research-sap.md:246-263, :921-925`
- **Required change:** Decide toxic-combination checking in or out on the record. If in: name it as a grant-authoring-time constraint (conflict pairs over `Feature`, evaluated where `gov_approvals` four-eyes already runs) with a widening and a probe. If out: state it in §5.11 with the cited cost, so the decision is a choice rather than an omission inherited from SAP's shape.

### §4.1 (Tier N effective-dating) · §5.9 · §7

The plan has no correct-versus-new-effective-change axis, and the shipped store structurally forbids the first. `ont_instance_revisions` carries `valid_from`/`valid_to` with `CHECK (valid_to IS NULL OR valid_to > valid_from)` (0155:53) and a unique index permitting exactly one open revision (0155:57-58), and the append-only trigger forbids modifying a closed revision (0155:~118-129). So correcting an erroneous revision *at the same effective date* is inexpressible: you cannot rewrite it, you cannot close it at a zero-length interval, and appending at the same `valid_from` overlaps. Every path leaves the fold returning the wrong value for the period between the error and its discovery. The corpus names this as Workday's core distinction — "correct(overwrite) vs new-dated-change" — and ranks both the UX split and the dual entry-date/effective-date (bi-temporal) axis as steals at cost M (lenses/data-model.md:107, :126-127). The plan's only "correction" is the compensating document for post-확정 반려 (plan:577), a different concern. 소급 정정 of a mis-entered 발령일 or pay grade is routine payroll work, and Slice 1 is 인사발령.

- **Evidence:** `backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:53, :57-58; docs/ideas/ecosystem-plan-DRAFT.md:577, :1233-1243; docs/program/benchmark-matrix/lenses/data-model.md:107, :126-127`
- **Required change:** Decide the correction path explicitly before Slice 1: either a correcting revision carrying `corrects_revision_id` plus a knowledge-time argument on as-of reads (the entry-date axis the corpus prices at M), or a stated deferral with the consequence named. §2 driver 2 ("replayable must be free, not built") must be qualified accordingly, and §7 needs a probe whose known-bad control is a correction that silently rewrites history.

### §4.7 point 3 (enforcement is synchronous)

The plan asserts that checking bands in the transaction path is "a real departure from the common enterprise pattern of approve → spend → discover the overspend at close" (plan:857-861). The sourced research contradicts this. SAP's classic release procedure checks item-wise release "as you enter the data" and the release indicator gates what the document may be used for at each state (research-sap.md:299-301); Odoo attaches approval rules to buttons, so the gate is the action itself (research-omni-platforms.md:73); FI tolerance groups are per-person monetary ceilings enforced at posting (research-sap.md:216-221). Worse than the factual error is what it hides: the mechanism the research does name as a steal is **approval is a statement about a document *state*, not a document ID** — changing a released purchasing document past a threshold *resets* the release and forces re-approval (research-sap.md:386-390, adopt #7 at :867-870), which "closes the 'approved for ₩10M, then edited to ₩100M' hole structurally rather than by policy". The plan stores line-as-raised and line-as-executed but nothing invalidates a signature when the document's amount later crosses into another band; DN-0003's expected-revision/412 covers concurrent writes, not a legitimate post-signature amendment. Slice 0's single step cannot surface it.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:857-861; docs/ideas/research-sap.md:299-301, :386-390, :867-870; docs/ideas/research-omni-platforms.md:73`
- **Required change:** Delete the false-departure claim. Add the release-reset semantics to §5.2's delta list: a signature is bound to a document state, and a change crossing a `delegation_rule` band invalidates signatures taken under the prior band and re-routes. Add the probe with its known-bad control (an implementation that keeps signatures valid after the amount is raised).

### §5.5 (economics: line-level typed object dimension) · §9 ADR block

§5.5 item 2 pushes a single typed `(source_object_type, source_object_id)` pair down to the voucher line (plan:1081-1084) and Slice 0 posts the first dimensioned line (plan:1719). The research names two shipped answers this shape forecloses. First, SAP's real-versus-statistical account assignment: "the same cost may be *reported* against several objects while being *owned* by exactly one… it resolves the perennial double-counting argument declaratively instead of by convention" (research-sap.md:851-854, mechanism at :439-440, :490). Second, Odoo analytic plans: "a journal line carries several independent analytic dimensions simultaneously… and one amount splits by percentage across analytic accounts" (research-omni-platforms.md:80), with NetSuite showing the dimension itself packaged into a vertical (:114-116). A single-valued line dimension means per-object economics for a cost touching `work` and `lot` and a contract must either post N lines (double-counting) or pick one and lose the others — and §5.5's own "must not foreclose" list promises allocation with a recorded basis (plan:1102-1105).

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1081-1084, :1102-1105, :1719; docs/ideas/research-sap.md:851-854; docs/ideas/research-omni-platforms.md:80, :114-116`
- **Required change:** State whether one line may be reported against more than one object. If not, record real-versus-statistical assignment and percentage distribution as decisions the peer finance plan owns, and note that Slice 0's single posted voucher must not be cited as evidence the dimension shape is settled.

### §5.11 G1 (platform-level `party`) · §4.7 point 1

G1 blocks Slice 0 and must argue platform-level `party` against ADR-0022 in a new accepted ADR, yet its case rests on two internal documents (plan:1334) while §4.7 claims games are "the strongest available evidence" for the design (plan:846-848). The corpus holds four stronger, externally checkable precedents the plan never uses: Microsoft had to retrofit the concepts into Dataverse by installed package — "Dataverse includes new concepts such as company and party", with date-effectivity and monetary precision added the same way, read by the researcher as "a specification of what a substrate needs *before* modules are built on it" and "the strongest external evidence that the effective-dating-first posture is the right order of construction" (research-omni-platforms.md:132-134, CONFIRMED); Salesforce's retrofit is irreversible once enabled — Person Accounts "cannot be disabled" (:169, CONFIRMED); Odoo's `res.partner` is one native party for companies, individuals, customers, suppliers, contacts and users (:68); NetSuite's records-as-multiple-types shares one internal ID across roles with an explicit two-field divergence list (:102-106). The last is also the open question the research flags as *unsolved* — "whether anyone has published a principled account of which attributes must be role-scoped" (:365) — which §5.4's "never put personal attributes on `party`" answers by taking the shared set to empty, an answer worth stating as such.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1334, :846-848, :1038-1040; docs/ideas/research-omni-platforms.md:68, :102-106, :132-134, :169, :365`
- **Required change:** Cite the four precedents in G1's argument, and drop or qualify the "strongest available evidence" claim in §4.7. State §5.4's empty-shared-attribute-set as the plan's answer to the role-scoped-attribute question the research records as unsolved.

### §4.1 (delegation_rule) · §4.8 (ergonomics) · §7

The plan's single clearest differentiator against the benchmark is unnamed, unclaimed and unprobed. The research records, CONFIRMED by absence across every source read, that SAP has no artefact rendering the approval authority: "There is no artefact you can print and hand to an auditor saying 'this is our approval authority as of 2026-07-01'. It is distributed across customising tables, workflow scenarios, team definitions and role assignments" (research-sap.md:360-365, :412), and it is reject #1: "a 전결규정 is a document with legal force; if the system cannot render it, the system is not the source of truth and a spreadsheet becomes one" (:915-919). The plan's `delegation_rule` as an effective-dated Tier N type (plan:566) with as-of replay supplies exactly that artefact — and §4.7's own guild-bank ergonomics bar implies the grid rendering. Nothing in §4.8 or §7 asserts it: E1-E6 and every `slice0_*` probe are person-centric ("what could this person approve"), never regulation-centric ("render the whole matrix as of D"). Unclaimed means unprotected: a lane may implement routing as per-template step conditions — SAP's exact failure mode — and every probe still passes.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:566, :876-882, :1494-1531; docs/ideas/research-sap.md:360-365, :412, :915-919`
- **Required change:** Name it as a differentiator in §4.7/§4.8 and add one probe: the complete 전결규정 (category × band × scope → competent unit, terminal?) renders as one artefact as of an arbitrary date, with the known-bad control being routing expressed only inside approval templates.

### §4.1 ("Vocabulary is adopted, not invented") · §4.3

The plan lists `ReportingLine` among the 14 org primitives it declares adopted from `org-editor-primitives-ux.md` (plan:497-501), then omits it from the Tier N entity table and from every row of §4.3. The same spec makes position-to-position the *preferred* form: "Reporting lines should prefer Position-to-Position definitions for stable org charts, with Employee/Assignment overrides for temporary or exceptional cases" (org-editor-primitives-ux.md:257), a Position "may report to another Position" (:143), with cycle and single-primary-path validation (:147). The external research makes the same structure the leading one: in SuccessFactors "the position hierarchy connects positions to positions and by default, Position Hierarchy is the leading hierarchy", as distinct from the employee→manager tree, with position-based permissions deriving from the seat and the hierarchy (research-sap.md:683, :685, CONFIRMED). The plan's only structural edges are `parent_org_unit` and `position_at_scope`, so no position hierarchy exists.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:497-504, :657-683; docs/specs/org-editor-primitives-ux.md:143, :147, :257; docs/ideas/research-sap.md:683, :685`
- **Required change:** Either add `reporting_line` (position→position, with the cycle and primary-path validation the spec already specifies) or state its exclusion explicitly and defend it with the research that supports the plan's routing choice — SAP's own concession that approval authority should not derive from the HR org chart (research-sap.md:337-343, :861-865). Do not leave a named adopted primitive silently dropped.

### §0.1 (BLOCKING)

The BLOCKING correction's three evidence citations do not resolve, and the break is self-inflicted. `authority-and-approval-model.md:89-92` is now a table row about `group_role_grants`; the retraction text "The group is not high enough… Group-scoping relocates the duplication rather than removing it" is at :116-118. ":545-546" is now about `employees` as a spreadsheet import; "People are group-scoped… the group is the tenancy boundary for people" is at :571. ":575-579" is now about roles-as-grant-bundles; "the largest single engineering cost in the chosen model" is at :606. The ~+27..30 shift is exactly the SUPERSEDED header the plan author added to that document at :3-20, which restates the same stale citations. The plan's preamble claims "Line numbers re-verified this session". The substantive claim is TRUE and I confirmed it at the new lines — but a lane sent to verify the plan's one BLOCKING item will read a table about group_role_grants and conclude §0.1 is fabricated.

- **Evidence:** `docs/ideas/authority-and-approval-model.md:10-11 (stale citations in the header), :116-118, :571, :605-606; docs/ideas/ecosystem-plan-DRAFT.md:34-38`
- **Required change:** Re-anchor §0.1 to :116-118 / :571 / :605-606 and fix the same three citations in the input's own header at :10-11. Then apply the rule fanout-plan-DRAFT.md:243 already implies: citations into a document you also edit must be quoted-text anchors, not line numbers. Sweep the plan's other cross-document citations into `authority-and-approval-model.md` for the same +27..30 drift.

### §4.3 (`grant_scope`, `position_at_scope`); §4.1 Tier N `grant`; §4.2

Group- and company-scoped authority cannot be stored where §4.1 puts it. Two distinct failures. (a) §4.3 specifies `grant → org_unit | organization | group` and `position → org_unit | organization | group` as "Stored as: `ont_link`", but `ont_links` FKs BOTH endpoints to `ont_instances(id, org_id)`, so an edge to `organizations` or `groups` is structurally impossible — and `groups` is a Tier G table by the gate's own allowlist, not an ontology instance. (b) The deeper one: `grant` is Tier N, so a group-scope grant physically lives in exactly one org's `ont_instances` rows under FORCE RLS, hence is unreadable by every sibling org that needs it at raise time. §4.5 requires precisely that read ("eligible approvers = effective(·, step.competent_unit.scope)"). The shipped answer for cross-org authority is Tier O + a definer (`group_role_grants`, rationale "cross-tenant group role authorization; own-grants resolver only"), and org-hierarchy.md:175,299 confirms the shipped posture is one armed org per request iterated per member. §4.2 adopts org-hierarchy's `AccessScope` vocabulary but not its storage tier.

- **Evidence:** `backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:78-79; backend/ci/gates/tenant-isolation/src/lib.rs:48 (`groups` = global tier); backend/crates/platform/db/migrations/0060_create_groups.sql:40-50; docs/specs/org-hierarchy.md:175,299`
- **Required change:** In §4.3, replace `ont_link` with the real substrate for the `organization` and `group` arms — a scope descriptor property (`{level, node_id}` per org-hierarchy.md:172-173), not an edge. In §4.1, split `grant` by scope level: org_unit/organization-scoped grants stay Tier N; group-scoped grants are **Tier O** beside `group_role_grants` and reached only through the definer. State it in §4.2, because it means the plan does add one owner-only table beyond `party`, and the ADR-block's "one owner-only table, two tenant tables" cost line is wrong.

### §5.8 H — conservation as a row CHECK

The row CHECK does not conserve, and the reason the plan gives for rejecting the definer is refuted by the precedent it cites. `CHECK (parent_qty_before - split = parent_qty_after)` is per-row arithmetic; the invariant that actually matters spans *successive splits of the same parent*. Two concurrent splits of a 100-unit lot, each written as (before 100, split 60, after 40), both satisfy the CHECK and over-allocate by 20. The plan's claim — "a definer is needed when an invariant spans sibling rows, and putting before/split/after on one row removes the span" — is wrong: the shipped `0156` pattern conserves via `fetch_item_for_update_tx` (a `SELECT … FOR UPDATE` on the item's current quantity) plus an advisory lock on the idempotency key plus `state.consume(quantity)` in the domain. `0156:103` is the arithmetic backstop on top of that, not the mechanism. Secondary: §5.8's `lot` shape carries `quantity_milli` while the split rows also carry the deltas — the same fact in two places, which `no_duplicated_fact` (§7) forbids.

- **Evidence:** `backend/crates/inventory/adapter-postgres/src/lib.rs:376,394-396,411 and backend/crates/platform/db/migrations/0156_create_inventory.sql:103`
- **Required change:** Keep the CHECK (it is cheap and correct as far as it goes) and add the mechanism the precedent actually uses: the split write locks the parent lot row `FOR UPDATE` inside the action's transaction and derives `parent_qty_before` from the locked row, never from the request. Add probe `lot_concurrent_split_cannot_overallocate` with the known-bad control being the row-CHECK-only implementation §5.8 currently specifies. Decide whether `lot.quantity_milli` is authoritative or derived, and say which.

### §4.0 / §4.0.2 — "concerns are components; systems light up"

For the projected tier the frame is an aesthetic, not a mechanism — and §0.14 just moved the most component-dense entity into that tier. §4.0 claims "the systems light up for it without anyone hand-writing an integration per concern". `ActionDispatch` has exactly two arms; the `projected_usecase` arm routes through `ProjectedDispatchRegistry`, a `HashMap<String, ProjectedHandler>` that is "Empty by default ⇒ every projected dispatch fails closed (`NotWiredYet`)" and is populated by the App composition root registering one Rust closure per target. So every action on a projected type is a hand-written integration. `work` is Tier T + projected (§0.14, §4.1) and is declared the join point for artifacts, actions, handover, ledger and metrics — i.e. the entity composing the most concerns is exactly the one that gets none of them for free. §4.0.2's honest boundary table lists projected backing as "one arm in `allowlisted_projected_table`", which understates it by the whole handler registry.

- **Evidence:** `backend/crates/ontology/rest/src/lib.rs:91-95, :167-195, :1278-1290; backend/crates/ontology/domain/src/lib.rs:213-218`
- **Required change:** Add a row to §4.0.2: "actions on a projected type — code: one `ProjectedDispatchRegistry` handler per action, registered in the App composition root; unwired = `NotWiredYet`." Then enumerate `work`'s Slice-0/W4/W11/W13 actions with a handler count, in `ecosystem-entity-components.tsv`, so the Phase-3 `app` crate row is sized rather than labelled "wiring".

### §8 Phase 6 X4; §4.2

The experiment designated to test the plan's central claim cannot refute it. X4 is "Answer `effective(party, scope)` for a person in two orgs using only `app.current_org` + the visibility edge", with known-bad "an attempt that requires `app.current_group`". That is the easy half — resolving a *known* party's authority *within* the armed org. The hard half is what §4.5 actually requires at raise time and what §4.2's sufficiency claim covers: (a) enumerate the parties competent at group scope from an org-A session, and (b) read a group-scope grant that lives in another org's Tier N rows. Both are deferred to W5/W8 while §4.2 asserts sufficiency globally and the plan says of X4 "this is the plan's central claim, so test it first". A first-run experiment that passes on the easy half will be read as validation of the whole claim.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1647 against :745-746 and :633-634`
- **Required change:** Extend X4 with the falsifying case: from a session armed to org A, resolve the eligible-approver set for a step whose competent unit is at group scope, where the only qualifying holder is a user of org B. Prediction must be stated honestly — on the current design this is NOT answerable without either iterating member orgs under group reach or a Tier O grant store. If it is not answerable, §4.2's claim narrows to "no second tenancy dimension for intra-org authority" and the ADR block's wording changes with it.

### §4.0.3 — "it is two nullable columns"

The plan's self-declared "highest-leverage change" is sized as DDL and its real cost is plumbing. Two nullable columns on `audit_events` is correct as DDL, and the `0149` precedent is exact. But the value has to reach the row: `AuditEvent` is a kernel struct with no capacity field, and there are 451 non-test references to `with_audit`/`with_audits` across the crates. Every one becomes a populate-or-leave-null decision, and "authority mutation" — the set where null is a defect per pre-mortem 4's leading indicator and per probe `capacity_recorded_on_every_authority_mutation` — is never enumerated anywhere in the plan; §5.11 G9 explicitly defers the enumeration to Phase-7 prepwork. So the gate the pre-mortem asks for has no definition to run against.

- **Evidence:** `backend/crates/kernel/core/src/audit.rs:83-108 (no capacity field, `AuditEvent::new` has 6 params) and 451 non-test `with_audit` references under backend/crates`
- **Required change:** Move the G9 enumeration from Phase 7 to Phase 0 and make it the artifact the probe reads: one row per authority-mutating write path in `ecosystem-entity-components.tsv`. Then make capacity non-optional at those sites by construction — a distinct constructor (`AuditEvent::authorized(…, grant_id)`) rather than a nullable field plus a gate — so the compiler enforces what the gate would otherwise have to police across 451 sites.

### §8 Phase 6 / Phase 2 ordering

§8 is Bun-shaped in content but not in order. Phase 6 (experiments) is numbered after Phase 5 (one PR), while its own contents say the opposite: X4 is "the plan's central claim, so test it first", and Phase 7 says "X8 runs first". Bun's trial and its 3-hour mapping preceded all conversion. A lane executing §8 in phase order runs the experiments after the single PR has been built.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1637 (Phase 6 heading), :1647 ("test it first"), :1670 ("X8 runs first"), :1622 (Phase 5 one PR)`
- **Required change:** Renumber so experiments precede the Phase-2 trial run, and state the gate explicitly: no slice-0 implementation commit until X1-X5 and X8-X9 have recorded outcomes in `known-bad-controls.tsv`.

### §8 Phase 6 — X8, X9, X4, X5

X8's prediction is not falsifiable as written — "they pass by some mechanism that must be identified" cannot be refuted by any observation — and its known-bad control ("wiring a new test and assuming it runs") is a fallacy, not a runnable input. X9, X4 and X5 have the same defect: their control column names a refutation scenario ("a case needing a companion evaluator", "an attempt that requires app.current_group") rather than an input a probe can be observed RED on. By the plan's own principle 5, four of nine experiments do not yet count.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1652 (X8), :1653 (X9), :1647 (X4), :1648 (X5); §1 principle 5 at :279-280`
- **Required change:** Give X8/X9 one shared runnable control: add a new test file containing a deliberately failing assertion, confirm CI goes RED, then fix it — and name the enumerated candidate mechanisms X8 must discriminate between (path filter, continue-on-error, no-op required job, cached graph). Restate X4 and X5 as constructed queries/cases with an expected-fail baseline.

### §8 Phase 3 / Phase 7 rung ①

MISSING: the lane→path reservation instance. §8 references fanout-plan-DRAFT §5's reservation scheme and LANE-PROTOCOL's five-lane pool but never instantiates either for the eight crates of Phase 3 or the eighteen widenings. Rung ① is asserted ("each in files no other lane owns") without the per-lane owned-path table that would prove it — and LANE-PROTOCOL:72-78, which the plan quotes, is precisely the warning that unproven ownership degrades to discipline, "and discipline is what fails".

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1554-1556, :1661-1672, :1593-1606 (crate queue with no owner column)`
- **Required change:** Add a Phase-0 artifact: one row per lane with its crates, owned paths, migration slots and the widenings it may take, so rung ① is demonstrated rather than claimed. W11-W13's "three lanes, no shared files" claim must appear in that table.

### §8 Phase 7 — prepwork; §4.1 vocabulary

MISSING: no correction of the program catalog, and an unreconciled type-name collision. CATALOG.md lists the five types as OrgUnit/Position/Person/Employment/PayRun and calls types 2-5 "expansion work"; the executable const is company/org_unit/job_position/employment/pay_run — Person never landed and company is absent from the catalog. §4.1 adopts `org-editor-primitives-ux.md`'s "Position" and introduces a Tier N `position`, while `position` is already a seeded built-in stable_key and the shipped conformance type is `job_position`. A lane transliterating either document builds the wrong set or collides with the built-in.

- **Evidence:** `docs/program/CATALOG.md:62-69; backend/crates/ontology/rest/tests/company_conformance.rs:184-190; backend/crates/ontology/adapter-postgres/src/seed.rs:74 (POSITION_KEY = "position"); company_conformance/harness.rs:198; docs/ideas/ecosystem-plan-DRAFT.md:497-504, :572`
- **Required change:** Add a Phase-7 item correcting CATALOG.md:62-69 to the shipped set, and state in `ecosystem-PORTING.md` the stable_key mapping across {org-editor "Position", built-in `position`, shipped `job_position`, this plan's 직책 type} — with the plan's type given a non-colliding key.

### §8 Phase 7 — prepwork

MISSING: no correction of LANE-PROTOCOL's stale claims, though §8 leans on it as process authority. Its status header still reads "prep artifact, not yet exercised. Fan-out is not authorized until §4 passes" after fan-out ran green and was promoted to a required check, and :268-270 states the repo has "no `.cargo/config.toml` and no `[profile]` section" while `[profile.dev]` and `[profile.test]` both exist. A plan that opens fanout under a protocol whose own header forbids it has an unresolved governance contradiction.

- **Evidence:** `docs/program/LANE-PROTOCOL.md:7, :268-270; backend/Cargo.toml:359-363; docs/ideas/ecosystem-plan-DRAFT.md:1554-1556, :1661`
- **Required change:** Add a Phase-7 item correcting LANE-PROTOCOL:7 (fan-out status) and :268-270 (profile/sccache), and cite the corrected header where §8 opens fanout.

### §4.0.2, §0.13, §4.8 E4 — the no-code boundary

MISSING: `AUTHORING_ACTIONS` — the checklist's "second closed vocabulary" — appears nowhere in the plan. It is a five-element const rejecting every other action at two sites, so an authored object policy can carry only view/edit/read_field/console:configure/console:deploy. This bounds §0.13's resolution (attaching a policy buys read visibility, never a domain capability like `purchase.approve`) and it bounds E4: `simulate_inner` denies any action outside the same list, so "policy simulation ships" is true only for those five.

- **Evidence:** `backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252, :294-297, :714-720; docs/ideas/ecosystem-plan-DRAFT.md:444-461 (§4.0.2 boundary table), :450, :881 (E4)`
- **Required change:** Add a row to §4.0.2's "requires code" column for the authoring action vocabulary, restate §0.13's resolution as "a `view` permit, which is all an authored policy can express", and qualify E4 so the fold simulator is not assumed to inherit Cedar simulation for domain capabilities.

### §1 principle 3, §4.4 — the four dimensions

MISSING substrate for two of the four dimensions. §1 principle 3 declares 소속 / 직급·직책 / 직무 / 결재선 "predicates for writing grant rules", and §4.4 resolves this by reusing `policy_role_conditions`' attribute vocabulary. That vocabulary is a closed 22-value CHECK containing no 직무 (job/duty) and no 직급 (grade/rank) — only `position`. §4.1 adds no entity for either. So two of four dimensions can be neither authored nor predicated, and widening the CHECK is a migration, making it a third closed vocabulary the plan does not name.

- **Evidence:** `backend/crates/platform/db/migrations/0065_create_policy_roles.sql:110-127 (attribute CHECK), :129 (operator CHECK); docs/ideas/ecosystem-plan-DRAFT.md:274-275, :721`
- **Required change:** Either add 직무/직급 to §4.1 as authored types with their attribute keys and the migration that widens the CHECK, or state explicitly which of the four dimensions have no substrate in slices 0/1 and in which widening they arrive.

### §4.7 point 2, §4.8 — the ergonomics acceptance bar

MISSING: the owner's named acceptance bar has no test and its third axis is silently dropped. §4.7 asserts the guild-bank comparison "is a testable bar, not a sentiment (§4.8)" — and §4.8 contains E1-E6, none of which is that test. Worse, §4.7 restates the guild-bank shape as "(role × amount band × category) → permitted", dropping the per-day limit that the requirement and its own first sentence carry, and `delegation_rule` is (category × amount band × raising scope) with no periodic or cumulative quota anywhere in §4.1 or §4.3.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:852-855 ("testable bar … §4.8"), :872-894 (§4.8, no such criterion), :566 (delegation_rule shape)`
- **Required change:** Add the bar to §4.8 as a criterion with a substrate and cost (e.g. authoring-time and error-rate against a stated reference task), and either add a period/cumulative quota dimension to `delegation_rule` or record dropping it as a decision with a reason.

### §4.8 E2 — the character sheet

MISSING delivery vehicle and executable form for the unifying screen. E2 is named the completeness test ("an entity with no home on this screen is a modelling smell") but appears in no slice and no widening — W17 ships E4, W18 ships E1, W11 ships E6, and E2 has nothing. Phase 3's only mention is "`app` | wiring, `/overview` surface", which is not named as the character sheet and carries no acceptance. `every_entity_declares_its_components` tests rows in a TSV, not homes on a screen, so the character-sheet test does not exist in §7 either.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:879 (E2), :1606 (Phase 3 crate 8), :1745-1765 (widenings — no E2), :1512 (component probe)`
- **Required change:** Give E2 a widening with acceptance, and make the completeness test executable: one row per §4.1 entity mapped to its character-sheet section in `ecosystem-entity-components.tsv`, with a probe that fails on an unmapped entity.

### §4.4, §8 W1 — the obligation loop

MISSING: audience targeting. §4.4 names three `notices` gaps (no content column, no closure state, org-composite recipient FK) and W1 fixes the key, the content leg and closure — but not targeting. The executable publish path snapshots recipients as either every active user in the org or every user in the notice's branches; there is no per-recipient audience. So `obligation_notifies_line_as_raised` (notify truncated member D specifically) cannot pass, and a 반려 notice would fan out to every active org user — a confidentiality regression on a 결재 matter.

- **Evidence:** `backend/crates/notices/adapter-postgres/src/lib.rs:413-433 (branch-scoped or org-wide snapshot); docs/ideas/ecosystem-plan-DRAFT.md:722 (three gaps), :1747 (W1), :1503 (probe)`
- **Required change:** Add per-recipient audience targeting as a fourth W1 gap with its DDL (an explicit recipient list keyed by party) and make `obligation_notifies_line_as_raised` assert that non-members receive nothing.

### §5.5, §5.2 — period locks and finality

MISSING: the one shared answer the requirement asks for. §5.5 resolves the lock *mechanics* well (keyed on DATE, voucher has none, the lock does not enforce itself, finance-gl is not among the four callers) and prescribes `accounting_date` plus the missing guard call. Neither §5.5 nor §5.2 decides the *interaction*: what happens when a post-확정 반려 arrives for a period already locked. W14 pairs the compensating voucher with "assert_period_open called from finance-gl", which as written would refuse the compensating posting and leave the obligation loop unclosable.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1062-1068, :1079-1080, :1760 (W14); §5.2 at :940-971 (finality, no period mention)`
- **Required change:** State the rule in one place: whether 확정 requires an open period, and that a compensating voucher posts with an `accounting_date` in the current open period while referencing the original — with a probe for the locked-period 반려 path.

### §4.5, §5.9 — handover modes

MISSING: 연차 and 퇴사 appear zero times in the plan. The requirement distinguishes a time-boxed 대리 that reverts from a permanent transfer with revocation and a completeness check; §4.5 gives the 인계 완료 query and the 대리/분배 mechanism, but no decision that a leave-based handover reverts automatically, and no revocation step at departure. Relatedly, `on_behalf_of_party_id` lands in slice 0 as a column and is exercised nowhere: no probe asserts that 대리/대결 records both parties, and no slice or widening writes it — the exact null-capacity failure the plan's own pre-mortem 4 names.

- **Evidence:** `grep 연차/퇴사 = 0 hits in docs/ideas/ecosystem-plan-DRAFT.md; :750-756 (handover), :1750 (W4), :1434-1445 (pre-mortem 4), :534 (column in slice 0)`
- **Required change:** Add the two handover modes to §4.5 as distinct operations (time-boxed reverting 대리 vs permanent transfer + grant revocation + 인계 완료 gate) and add a probe `daeri_records_both_parties` with a known-bad control (a 대리 signature with null `on_behalf_of`).

### §5.8, §4.3 — quantity lineage

The conservation invariant does not hold across sibling splits, and the table is named twice. §5.8 argues a definer is unnecessary because "putting before/split/after on one row removes the span" — but two splits of the same parent can each record before=100/split=30/after=70 and both satisfy the row CHECK, leaving 60 unaccounted; nothing stated links `lot.quantity_milli` to the split rows or serialises them, and the 0156 precedent it copies has no such linkage either (only a UNIQUE idempotency key). `lot_conservation`'s known-bad control is single-row, so it cannot detect this. §4.3 also stores the edge as `lot_derivation` while §5.8 names it `lot_split`, and the traversal uses `lot_derivation`.

- **Evidence:** `backend/crates/platform/db/migrations/0156_create_inventory_consumption.sql:90-103 (row CHECK, no sibling linkage); docs/ideas/ecosystem-plan-DRAFT.md:1191-1200, :1188, :670, :1211, :1525`
- **Required change:** State the serialisation: the parent lot row is SELECT … FOR UPDATE'd and its `quantity_milli` updated in the same transaction, with `parent_qty_before_milli` required to equal it — or accept the definer. Add a concurrent-double-split known-bad control. Fix the table name to one spelling.

### §5.11 G6 vs §8 W10 — the no-code canvas

Two sections contradict each other on a core owner requirement. §5.11 G6 records the canvas as deferred by an accepted ADR, recommends accepting the deferral, and warns "Do not smuggle it in"; §8 lists W10 "Canvas over the authored types" as an ordinary widening with no gate on that decision. The ADR citation is accurate — the follow-up reads "this program ships read-only NL rows + simulation and defers the canvas" — so W10 as written is the smuggling G6 forbids.

- **Evidence:** `docs/decisions/ADR-0023 follow-ups (no-code policy/workflow visual canvas deferred); docs/ideas/ecosystem-plan-DRAFT.md:1342 (G6), :1756 (W10)`
- **Required change:** Mark W10 as gated on the G6 charter and state which of the two G6 options the plan recommends, so "roles configurable from a no-code canvas" is either an explicitly deferred requirement with the ADR quoted, or a charter proposed.

### §4.3 (`grant_scope`, `position_at_scope`); §4.1 Tier N `grant`; §4.2; §8 Phase 6 X4

Group- and organization-scoped authority cannot be stored where §4.1 puts it, and the experiment designated to test the plan's central claim cannot refute it. (a) §4.3 specifies `grant → org_unit | organization | group` and `position → …` as "Stored as: ont_link", but `ont_links` FKs BOTH endpoints to `ont_instances(id, org_id)`, so an edge to an `organizations` or `groups` row is structurally impossible — and `groups` is in the tenant-isolation gate's `global_table_allowlist` ("group identity metadata only, no tenant data"), i.e. Tier G, not an ontology instance. (b) The deeper one: `grant` is Tier N, so a group-scope grant physically lives in one org's `ont_instances` rows under FORCE RLS and is unreadable by every sibling org that needs it at raise time — which is precisely what §4.5 requires ("eligible approvers = effective(·, step.competent_unit.scope)"). The shipped answer for cross-org authority is Tier O + a definer (`group_role_grants`, rationale "cross-tenant group role authorization; own-grants resolver only"). (c) X4 is scoped to the easy half — resolving a KNOWN party's authority WITHIN the armed org — while its hard half (enumerate parties competent at group scope from an org-A session; read a group-scope grant living in another org's Tier N rows) is deferred to W5/W8 while §4.2 asserts sufficiency globally and the plan says of X4 "this is the plan's central claim, so test it first".

- **Evidence:** `backend/crates/platform/db/migrations/0155_create_ontology_instances.sql:78-79; backend/ci/gates/tenant-isolation/src/lib.rs:44-48 (`groups` = global tier) and :115-124 (`group_role_grants` = owner-only); docs/ideas/ecosystem-plan-DRAFT.md:666-668, :745-746, :1647`
- **Required change:** In §4.3 replace `ont_link` for the `organization` and `group` arms with the real substrate — a scope descriptor property `{level, node_id}` per org-hierarchy.md:172-173, not an edge. In §4.1 split `grant` by scope level: org_unit/organization-scoped stay Tier N; group-scoped grants are Tier O beside `group_role_grants`, reached only through the definer — and correct the §9 cost line, which says "one owner-only table". Extend X4 with the falsifying case: from a session armed to org A, resolve the eligible-approver set for a step whose competent unit is at group scope where the only qualifying holder is a user of org B, and state honestly that on the current design this is not answerable without iterating member orgs or a Tier O grant store.

### §5.8 H — conservation; §4.3 `derived_from`; §7 `lot_conservation`

Found independently by two lenses; I verified both halves. The row CHECK does not conserve, and the reason §5.8 gives for retracting the definer is refuted by the precedent it cites. `CHECK (parent_qty_before − split = parent_qty_after)` is per-row arithmetic; the invariant that matters spans successive splits of the same parent. Two concurrent splits of a 100-unit lot each written as (100, 60, 40) both satisfy the CHECK and over-allocate by 20. The plan's claim — "a definer is needed when an invariant spans sibling rows, and putting before/split/after on one row removes the span" — is wrong: the shipped `0156` pattern conserves via `fetch_item_for_update_tx` (a SELECT … FOR UPDATE on the item's current quantity) plus `lock_consumption_idempotency_key_tx` plus a domain `state.consume(quantity)`; `0156:103` is the arithmetic backstop on top. `lot_conservation`'s known-bad control is single-row so it cannot detect this. Secondary: `lot.quantity_milli` and the split deltas are the same fact in two places, which `no_duplicated_fact` forbids; and the table is named `lot_split` in §5.8 but `lot_derivation` in §4.3 and in §5.8's own traversal. (Correcting one lens: the file is `0156_create_inventory.sql`, not `0156_create_inventory_consumption.sql`.)

- **Evidence:** `backend/crates/inventory/adapter-postgres/src/lib.rs:376 (lock_consumption_idempotency_key_tx), :394 (fetch_item_for_update_tx), :411 (state.consume); backend/crates/platform/db/migrations/0156_create_inventory.sql:103; docs/ideas/ecosystem-plan-DRAFT.md:1188, :1191-1200, :670, :1211, :1525`
- **Required change:** Keep the CHECK — it is cheap and correct as far as it goes — and add the mechanism the precedent actually uses: the split write locks the parent lot row FOR UPDATE inside the action's transaction, derives `parent_qty_before` from the locked row (never from the request), and updates `lot.quantity_milli` in the same transaction. Add probe `lot_concurrent_split_cannot_overallocate` with the row-CHECK-only implementation as its known-bad control. Decide whether `lot.quantity_milli` is authoritative or derived and say which. Fix the table name to one spelling.

### §0.1 (BLOCKING); preamble; §0.8

The plan's one BLOCKING correction has three citations and none resolves; the break is self-inflicted and the same stale citations are restated in the input's own header. `authority-and-approval-model.md:89-92` is now a passage about `clearance_assignments`; the retraction text "The group is not high enough… Group-scoping relocates the duplication rather than removing it" is at :116-121. ":545-546" is now about `company` being free text and `person_name`; "People are group-scoped… the group is the tenancy boundary for people" is at :571. ":575-579" is now about roles-as-grant-bundles; "the largest single engineering cost in the chosen model" is at :606. The ~+27..30 shift is the SUPERSEDED header the plan's author added at :3-20, which repeats the stale citations at :11. The substantive claim is TRUE and I confirmed it at the new lines — but a lane sent to verify the plan's one BLOCKING item reads a passage about `clearance_assignments` and concludes §0.1 is fabricated. Same class: §0.8 and §5.5 assert repo-wide negatives over "206 migrations" three times; the tree has 205 (highest 0205) and 0206 is in another lane's worktree.

- **Evidence:** `docs/ideas/authority-and-approval-model.md:11 (stale citation inside the header), :116, :571, :606; docs/ideas/ecosystem-plan-DRAFT.md:34-38, :120, :1045`
- **Required change:** Re-anchor §0.1 to :116 / :571 / :606 and fix the same three citations at authority-and-approval-model.md:11. Then apply the rule fanout-plan-DRAFT.md:243 already implies: citations into a document you also edit are quoted-text anchors, not line numbers. Sweep every other cross-document citation into that file for the same drift. Restate the migration negatives as "all 205 migrations in the main checkout as of <commit>" and name the commit.

### §5.2 B — the delta list; §4.1 `approval_signature`; §7

A signature is bound to a document ID, not a document state, so a legitimate post-signature amendment across a band leaves the signature valid and no probe catches it: approved for ₩10M, edited to ₩100M, still approved. §4.1 stores line-as-raised and line-as-executed, but nothing invalidates a signature when the document's amount later crosses into another `delegation_rule` band. DN-0003's expected-revision/412 covers concurrent writes, not a legitimate amendment. Slice 0's single step and single band cannot surface it, and §5.2's delta table — which is the plan's complete claim of what it owns beyond ADR-0023 — does not list it. (I drop the lens's secondary claim that §4.7 point 3's "real departure from the common enterprise pattern" is factually false: whether SAP checks synchronously changes no code. The mechanism gap does.)

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:566, :568, :955-963, :1494-1531 (no probe), :1703-1706 (Slice 0: one band, one step)`
- **Required change:** Add release-reset semantics to §5.2's delta list: a signature is a statement about a document STATE, and a change crossing a `delegation_rule` band invalidates signatures taken under the prior band and re-routes. Add the probe with its known-bad control (an implementation that keeps signatures valid after the amount is raised).

### §4.0.2 the no-code boundary; §0.13; §4.8 E4; §1 principle 3; §4.4

Three closed vocabularies stand between §4.0.2's honest boundary and the owner's "manageable without developers", and the plan surveys only one of them (`Feature` minting, §0.15, correctly). (a) `AUTHORING_ACTIONS` is a five-element const — view / edit / read_field / console:configure / console:deploy — rejecting any other action at two sites. So an authored object policy can never express a domain capability like `purchase.approve`, which bounds §0.13's resolution to "a view permit" and bounds E4, since `simulate_inner` denies any action outside the same list. The plan never mentions it. (b) `policy_role_conditions.attribute` is a closed CHECK of 17 values (group, tenant, organization, org, department, team, position, employment_status, assignment, location, site, branch, device_posture, purpose, action, resource, sensitive_action) containing no 직무 and no 직급 — so two of the four dimensions §1 principle 3 declares "vocabulary" have no substrate to be vocabulary in, and §4.1 adds no entity for either. Widening that CHECK is a migration, i.e. a third closed vocabulary the plan does not name. (One lens said 22 values; the actual count is 17 — the substance holds.)

- **Evidence:** `backend/crates/platform/authz/src/cedar_pbac/authoring.rs:246-252, :294-297, :714-720; backend/crates/platform/db/migrations/0065_create_policy_roles.sql:110-127 (attribute CHECK), :129 (operator CHECK); docs/ideas/ecosystem-plan-DRAFT.md:444-461, :274-275, :721, :881`
- **Required change:** Add a row to §4.0.2's requires-code column for the authoring-action vocabulary; restate §0.13's resolution as "a `view` permit, which is all an authored policy can express"; qualify E4 so the fold simulator is not assumed to inherit Cedar simulation for domain capabilities. Then either add 직무/직급 to §4.1 as authored types with their attribute keys and the migration widening the CHECK, or state which of the four dimensions have no substrate in slices 0/1 and in which widening they arrive.

### §8 Phase 6 / Phase 2 ordering; X8, X9, X4, X5

§8 is Bun-shaped in content but not in order, and four of nine experiments do not count by the plan's own principle 5. Phase 6 (experiments) is numbered after Phase 5 (one PR), while its own contents say the opposite — X4 is "the plan's central claim, so test it first" and Phase 7 says "X8 runs first". Bun's 3-hour mapping and 3-file trial preceded all conversion; a lane executing §8 in phase order runs the experiments after the single PR is built. Separately, X8's prediction ("they pass by some mechanism that must be identified") is unfalsifiable and its control ("wiring a new test and assuming it runs") is a fallacy, not a runnable input; X9, X4 and X5 name a refutation scenario rather than an input a probe can be observed RED on.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1622 (Phase 5 one PR), :1637 (Phase 6 heading), :1647, :1648, :1652, :1653, :1670; §1 principle 5 at :278-279`
- **Required change:** Renumber so experiments precede the Phase-2 trial run, and state the gate: no slice-0 implementation commit until X1-X5 and X8-X9 have recorded outcomes in `known-bad-controls.tsv`. Give X8/X9 one shared runnable control — add a test file with a deliberately failing assertion, confirm CI goes RED, then fix it — and name the candidate mechanisms X8 must discriminate between (path filter, continue-on-error, no-op required job, cached graph). Restate X4 and X5 as constructed queries with expected-fail baselines.

### §4.4 notices; §8 W1; §7 `obligation_notifies_line_as_raised`

The obligation loop has no audience targeting, so its headline probe cannot pass and its unfixed form is a confidentiality regression. §4.4 names three `notices` gaps (no content column, no closure state, org-composite recipient FK) and W1 fixes all three — but the executable publish path snapshots recipients as either every active user in the org, or every active user in the notice's audience branches. There is no per-recipient audience. So `obligation_notifies_line_as_raised` (notify truncated member D SPECIFICALLY, though D never saw the matter) cannot pass, and a 반려 notice fans out to every active org user on a 결재 matter.

- **Evidence:** `backend/crates/notices/adapter-postgres/src/lib.rs:413-433 (branch-scoped or org-wide recipient snapshot); docs/ideas/ecosystem-plan-DRAFT.md:722, :1503, :1747`
- **Required change:** Add per-recipient audience targeting as a fourth W1 gap with its DDL (an explicit recipient list keyed by party) and make `obligation_notifies_line_as_raised` assert that non-members receive nothing.

### §5.5 period locks; §5.2 finality; §8 W14

W14 as written would refuse the compensating posting and leave the obligation loop unclosable. §5.5 resolves the lock mechanics well (keyed on DATE, the voucher has none, the lock does not enforce itself, finance-gl is not among the four callers) and prescribes `accounting_date` plus the missing guard call. Neither §5.5 nor §5.2 decides the interaction: what happens when a post-확정 반려 arrives for a period already locked. W14 pairs the compensating voucher with "assert_period_open called from finance-gl", and those two requirements contradict each other for exactly the case W14 exists to prove.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1062-1068, :1079-1080, :940-971, :1760`
- **Required change:** State the rule in one place: whether 확정 requires an open period, and that a compensating voucher posts with an `accounting_date` in the current open period while referencing the original. Add a probe for the locked-period 반려 path.

### §4.1 vocabulary ("adopted, not invented"); §4.3; §8 Phase 0 / Phase 7

Phase 0's transcription will build the wrong set from stale vocabulary documents, in three distinct ways, and Phase 0 declares transcription risk-free ("Writing them is transcription, not design"). (a) `ReportingLine` is listed among the 14 adopted org primitives and then omitted from the Tier N table and from every row of §4.3 — while the same spec makes position-to-position the PREFERRED form with cycle and single-primary-path validation. The plan's only structural edges are `parent_org_unit` and `position_at_scope`, so no position hierarchy exists. (b) `CATALOG.md:62-69` lists OrgUnit/Position/Person/Employment/PayRun; what shipped is company/org_unit/job_position/employment/pay_run — Person never landed, company is absent from the catalog. (c) §4.1 introduces a Tier N `position` while `position` is already a seeded built-in stable_key and the shipped conformance type is `job_position`, so a lane collides with the built-in.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:497-504, :572, :657-683, :1568; docs/specs/org-editor-primitives-ux.md:143, :147, :257; docs/program/CATALOG.md:62-69; backend/crates/ontology/adapter-postgres/src/seed.rs:74`
- **Required change:** Either add `reporting_line` (position→position with the cycle and primary-path validation the spec specifies) or state its exclusion and defend it. Add a Phase-7 item correcting CATALOG.md:62-69 to the shipped set. State in `ecosystem-PORTING.md` the stable_key mapping across {org-editor "Position", built-in `position`, shipped `job_position`, this plan's 직책 type}, giving the plan's type a non-colliding key.

### §8 Phase 3 / Phase 7 rung ①

The lane→path reservation is asserted, not demonstrated. §8 references fanout-plan-DRAFT §5's reservation scheme and LANE-PROTOCOL's five-lane pool but never instantiates either for the eight crates of Phase 3 or the eighteen widenings. Rung ① reads "each in files no other lane owns" with no per-lane owned-path table, and the Phase-3 crate queue has no owner column — while LANE-PROTOCOL:72-78, which the plan itself quotes, is precisely the warning that unproven ownership degrades to discipline, "and discipline is what fails". W11-W13's "three lanes, no shared files" claim has the same shape.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1554-1556, :1593-1606 (crate queue, no owner column), :1671, :1767-1768`
- **Required change:** Add a Phase-0 artifact: one row per lane with its crates, owned paths, migration slots (0207+) and the widenings it may take, so rung ① is demonstrated rather than claimed. W11-W13 must appear in that table.

### §5.11 G6 vs §8 W10

Two sections of the plan contradict each other on a core owner requirement. §5.11 G6 records the no-code canvas as deferred by an accepted ADR, recommends accepting the deferral, and warns "Do not smuggle it in". §8 then lists W10 "Canvas over the authored types" as an ordinary widening with no gate on that decision. The ADR citation is accurate — ADR-0023:153-154 defers the no-code policy/workflow visual canvas, shipping read-only NL rows plus simulation — so W10 as written is the smuggling G6 forbids, and "roles configurable from a no-code canvas" is left neither deferred nor chartered.

- **Evidence:** `docs/decisions/ADR-0023-*.md:153-154; docs/ideas/ecosystem-plan-DRAFT.md:1342, :1756`
- **Required change:** Mark W10 as gated on the G6 charter and state which of the two G6 options the plan recommends, so the requirement is either explicitly deferred with the ADR quoted, or a charter is proposed.

### §1 principle 2; §5.1; §5.11

Toxic-combination (SoD) checking is absent and the omission is inherited rather than chosen. The plan's fold is positive-only by principle ("Additive grants only… Revocation closes a validity interval; it never writes a deny") and pre-mortem 2 defends that property hard — correctly. But positive-only plus accumulation is precisely the shape that produces grant combinations individually legitimate and jointly dangerous, with nothing in the core detecting it; the plan's only SoD content is the shipped four-eyes `CHECK (approved_by <> created_by)`. Crucially, mutual exclusion does NOT require a deny in the fold — it is a constraint at grant-AUTHORING time — so principle 2 is not in tension with it and the omission is not forced.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:271-273, :1056, :1405-1417; docs/program/benchmark-matrix/lenses/governance.md:155; docs/ideas/research-sap.md:246-263, :921-925`
- **Required change:** Decide it in or out on the record. If in: name it as a grant-authoring-time constraint (conflict pairs over `Feature`, evaluated where `gov_approvals` four-eyes already runs) with a widening and a probe. If out: state it in §5.11 with the cited cost, so it is a choice rather than an inherited omission.

### §5.5 economics — line-level typed object dimension; §9 ADR block; Slice 0

A single-valued `(source_object_type, source_object_id)` pair on the voucher line forecloses two shipped answers the plan's own "must not foreclose" list needs. Real-versus-statistical account assignment lets the same cost be REPORTED against several objects while OWNED by exactly one, resolving double-counting declaratively; analytic plans let one journal line carry several independent dimensions with percentage distribution. With a single-valued line dimension, a cost touching `work` and `lot` and a contract must either post N lines (double-counting) or pick one and lose the others — and §5.5 promises allocation with a recorded basis. Slice 0 posts the first dimensioned line, which will then be cited as evidence the shape is settled.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:1081-1084, :1102-1105, :1719; docs/ideas/research-sap.md:851-854, :439-440; docs/ideas/research-omni-platforms.md:80, :114-116`
- **Required change:** State whether one line may be reported against more than one object. If not, record real-versus-statistical assignment and percentage distribution as decisions the peer finance plan owns, and note that Slice 0's single posted voucher is not evidence the dimension shape is settled.

### §4.5 handover; §5.9; Slice 0 `on_behalf_of_party_id`; W4

`on_behalf_of_party_id` lands in Slice 0 as a column and is exercised nowhere — no probe asserts that 대리/대결 records both parties, and no slice or widening writes it. That is pre-mortem 4's own named failure ("the capacity field stays null because nothing visible depends on it") realised in the plan's own Slice-0 table. Relatedly, 연차 and 퇴사 appear zero times in the plan: §4.5 gives the 인계 완료 query and the 대리/분배 mechanism but never decides that a leave-based handover reverts automatically, and there is no revocation step at departure — while the plan's own §4.7 maps 대리/대결 to raid lead-and-assist and treats it as load-bearing.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:534, :750-756, :840, :1434-1445, :1750 (W4 — no 대리 probe); grep 연차/퇴사 = 0 hits`
- **Required change:** Add the two handover modes to §4.5 as distinct operations (time-boxed reverting 대리 vs permanent transfer + grant revocation + 인계 완료 gate), and add probe `daeri_records_both_parties` with known-bad control a 대리 signature with null `on_behalf_of`.

### §4.7 point 2; §4.8; §7

The plan's clearest differentiator is unnamed, unclaimed and unprobed, so a lane can implement the failure mode and every probe still passes. `delegation_rule` as an effective-dated Tier N type with as-of replay supplies something the benchmark has no equivalent for: a single renderable artefact answering "this is our approval authority as of 2026-07-01" — where the comparison distributes it across customising tables, workflow scenarios, team definitions and role assignments. A 전결규정 has legal force; if the system cannot render it, a spreadsheet becomes the source of truth. But E1-E6 and every `slice0_*` probe are person-centric ("what could this person approve"), never regulation-centric ("render the whole matrix as of D"). Separately, §4.7 restates the guild-bank shape as (role × amount band × category) → permitted, dropping the PER-DAY limit its own first sentence carries, and `delegation_rule` has no periodic or cumulative quota anywhere in §4.1 or §4.3 — and §4.7 asserts the guild-bank comparison "is a testable bar, not a sentiment (§4.8)" while §4.8 contains no such test.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:566, :852-855, :876-894, :1494-1531; docs/ideas/research-sap.md:360-365, :412, :915-919`
- **Required change:** Name it as a differentiator in §4.7/§4.8 and add one probe: the complete 전결규정 (category × band × scope → competent unit, terminal?) renders as one artefact as of an arbitrary date, known-bad control being routing expressed only inside approval templates. Then either add a period/cumulative quota dimension to `delegation_rule` or record dropping it as a decision with a reason, and give §4.8 the ergonomics criterion §4.7 promises it.

## MINOR (15)

### §5.2 (the obligation loop) · W1

The corpus records that a 그룹 obligation cascading to 계열사 is modelled by none of the seven benchmarked vendors (compliance.md:248-251). The research narrows but does not eliminate it — NetSuite OneWorld and Dynamics F&O do model a group of legal entities properly (research-omni-platforms.md:111) — and explains why the cascade specifically is absent: it "requires the group to be a *node in the same model* as the entities, and most products made the entity a tenant boundary, at which point the group can only ever be a reporting overlay" (research-saas-bar.md:227), left open at :396. The plan's obligation loop (plan:962) is per-org, and W1's acceptance reaches only as far as a recipient in another company (plan:1747). A group-originated obligation with per-org closure and a group-level rollup — which the plan's Tier O `party_link` plus ADR-0018's parent-envelope pattern could express — is neither claimed nor tested.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:962, :1747; docs/program/benchmark-matrix/compliance.md:248-251; docs/ideas/research-omni-platforms.md:111; docs/ideas/research-saas-bar.md:227, :396`
- **Required change:** Say in W1 whether a group-originated obligation cascading to 계열사 with per-org closure is in scope. If in, its acceptance criterion belongs there; if out, record it as a named unclaimed differentiator rather than leaving W1's cross-company recipient fix to read as the whole capability.

### §4.0 (concerns are components) · §8 Phase 0

§4.0 claims "the ontology's typed properties and links **are** the component mechanism" and that "a new entity class declares its components; the systems light up for it without anyone hand-writing an integration per concern" (plan:402-406) — then §4.0.2 states the opposite honestly 50 lines later (giving a type a new concern is code, plan:456-461). The research supplies the missing construct and its ceiling: Foundry's answer to one application over many object types is **Interfaces** plus **shared properties** — "an Ontology type that describes the shape of an object type and its capabilities", composed of interface properties, link-type constraints and action-type constraints — and Foundry's own docs warn it is "fully supported in Ontology Manager, Marketplace and TypeScript v2 Functions, with **partial support in Actions and Object Set Service**", i.e. half-plumbed polymorphism precisely at the write layer (foundry-reference.md:137-139, CONFIRMED). The plan has no such construct; its only composition artefact is a TSV plus one probe (plan:1565, :1512). The cheap correct framing is already in the research: ServiceNow's CSDM as "a published opinion about how to use a schema that permits anything" (research-omni-platforms.md:193-195, steal #8).

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:402-406, :456-461, :1512, :1565; docs/ideas/foundry-reference.md:137-139; docs/ideas/research-omni-platforms.md:193-195`
- **Required change:** Downgrade "the systems light up" to what the substrate supports, cite Foundry Interfaces/shared properties as the prior art for the construct the plan does not build (with its documented Action-layer weakness), and describe `ecosystem-entity-components.tsv` as a CSDM-style prescribed model — a convention a lane must follow — not a runtime mechanism.

### §4.1 (`position`, `assignment`) · Slice 1

The plan's position/assignment model has no occupancy semantics for absence. `holds_position` is ManyMany (plan:673, :691-696) so a substitute's concurrent assignment is expressible, but nothing distinguishes the absent holder's claim from the substitute's. The research records SuccessFactors' incumbent tracking as CONFIRMED: the system tracks "whether an employee on a global assignment or a leave of absence have a right to return into their position" (research-sap.md:684, narrative at :651). The internal spec has incumbency but not the return right (org-editor-primitives-ux.md:143, :147). HR + payroll is the first vertical and 육아휴직 복직 is a statutory right, so 휴직/복직 — absent from the plan entirely — will land on `assignment` regardless.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:672-673, :691-696, :1730-1736; docs/ideas/research-sap.md:651, :684; docs/specs/org-editor-primitives-ux.md:143`
- **Required change:** Add an assignment kind (substantive / acting / seconded) and a return-right marker as authored properties on the Tier N `assignment` type when Slice 1 defines it — a property addition, not a migration — so 휴직 does not later require reshaping a shipped slice-1 entity.

### §1 principle 4; §3.1

The plan breaks its own tier invariant within twenty lines, and the guarantee attached to the invariant does not extend to the path it actually uses. Principle 4: "Every storage decision names one of the four tiers the CI gates already enforce (§3.1). A new tier is a plan defect." §3.1 then introduces "A fifth path, Tier P — projected", and §3.1's own tier rule ends "→ T, projected into P". Tier P is not gate-enforced — it is a compiled-in `match` in `allowlisted_projected_table` plus a `backing_kind` CHECK — so "All four are enforced by `backend/ci/gates/tenant-isolation/src/lib.rs`" is true of four and not of the fifth, which is where `work` and `worksite_registration` both land.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:279-280 against :324-331; backend/crates/ontology/adapter-postgres/src/instances.rs:1479-1498`
- **Required change:** Either restate principle 4 as "one of the four tiers, optionally projected" and say explicitly that projection is code-gated not CI-gated, or drop the "Tier P" name and call it what it is — a Tier T table with a projected view. The naming matters because Phase 0's PORTING.md is meant to be a mechanical rule a lane looks up rather than re-derives.

### §0.8; §5.5; §5.7

"206 migrations" is not the tree a lane will build in. The main checkout has 205 migration files, highest `0205`; the brief states 0206 is in flight in lane-1. The substantive claims — zero `CREATE MATERIALIZED VIEW`, and the absence of `gl_postings`/`journal_entries`/`gl_accounts`/`chart_of_accounts`/`fiscal_periods`/`trial_balance` — verify against the 205 that exist. But a repo-wide absence claim asserted over a count that only holds inside another lane's worktree is the same class of defect §0 exists to catch, and the plan repeats "206" three times as the scope of a negative.

- **Evidence:** `backend/crates/platform/db/migrations/ contains 205 .sql files (highest 0205); docs/ideas/ecosystem-plan-DRAFT.md:120,1046`
- **Required change:** Restate as "all 205 migrations in the main checkout as of <commit>" and name the commit, so the negative is re-checkable. Same for the `finance-gl` absence list.

### §5.11 — reciprocal ADR pairs

The pairs name the new-ADR side only. README requires amendment to be explicit in both records with reciprocal relationship keys; §5.11 quotes the rule but the table's "Required artifact" column says "new accepted ADR" or "reciprocal pair on ADR-0003" without naming the counterpart edit — which line of ADR-0022:36,38 the platform `party` amends, or that ADR-0003 currently carries no `amended_by` key to reciprocate with.

- **Evidence:** `docs/decisions/README.md:9, :26; docs/ideas/ecosystem-plan-DRAFT.md:1332-1343, :1666`
- **Required change:** For each of G1, G2b and (if the Critic rejects the distinction) G8, name the counterpart record, the line amended and the relationship key to be added on both sides.

### §4.3, §4.5 — artifact ownership

Work-linked versus person-linked artifacts is used but never modelled. §4.5's 인계 완료 query relies on "∃ artifact linked to departing but to no work" and `handover_moves_work_artifacts_only` asserts the boundary, yet §4.3's relationship table carries only `work_artifact`; the person-endpoint edge (an `object_links` row with `src_kind = 'person'`) is never declared, so the distinction the PII and handover boundary rests on is inferable only from the probe.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:677 (work_artifact only), :768-772 (query), :1505 (probe)`
- **Required change:** Add the person-linked artifact edge to §4.3 with its cardinality and owner, and reference it from the 인계 완료 query.

### §2 driver 3, §5.4 — Korea dependency

The HOLD is stated as a dependency (correctly, and the plan does not plan to lift it), but the terms of release are not: the six controls, the requirement for qualified authority, and that native agents produce only non-independent evidence while I2/I3 custody is required. §5.4's preconditions for putting attributes on `party` say "the Korea controls have moved off HOLD" without naming what would move them, so a later lane cannot tell whether it has met the bar.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:296-299, :1030-1031`
- **Required change:** Name the six controls and the independent-evidence requirement in §5.4's precondition list, citing the jurisdiction register, so the dependency is checkable rather than merely stated.

### §8 Phase 0 / Phase 7 prepwork — external evidence

DOWNGRADED from one lens's blocking. The plan cites the external-research corpus zero times — I verified: grep for benchmark / research- / Foundry / Workday / SAP / Odoo / NetSuite / ServiceNow / Salesforce returns 0 hits across 1,853 lines — while §4.7 grants MMO games explicit evidentiary standing with a burden of proof ("the game's shape is prior art and the burden is on us to justify deviating") and calls them "the strongest available evidence" for the keystone. That asymmetry is real and cannot be defended as evidentiary discipline. But it is a process finding whose payload is the substantive deltas already promoted above (SoD, the correction axis, release-reset, multi-dimension lines, reporting_line), and the lens itself concedes those are derivable without the corpus. It does not by itself change what an implementer writes, so it is one Phase-0 line item, not a gate on Slice 0.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:825-826, :846-848, :1568, :1664-1672; grep count = 0`
- **Required change:** Add one Phase-0 prepwork line: reconcile §4 and §5 against the benchmark matrix and the four research surveys, recording which findings the plan adopts, rejects or contradicts, with confidence labels carried through — no plan decision resting on an UNCERTAIN/UNKNOWN row. Drop or qualify the "strongest available evidence" claim at :847, and cite the four externally-checkable party/company retrofit precedents in G1's argument.

### §8 Phase 7 prepwork — LANE-PROTOCOL as process authority

§8 opens fanout under a protocol whose own status header forbids it. LANE-PROTOCOL:7 still reads "prep artifact, not yet exercised. Fan-out is not authorized until §4 passes" after fan-out ran green and was promoted to a required check; :268-270 states the repo has "no `.cargo/config.toml` and no `[profile]` section" while `[profile.dev]` and `[profile.test]` both exist. An unresolved governance contradiction in the document §8 leans on.

- **Evidence:** `docs/program/LANE-PROTOCOL.md:7, :268-270; backend/Cargo.toml:359-363; docs/ideas/ecosystem-plan-DRAFT.md:1554-1556, :1661`
- **Required change:** Add a Phase-7 item correcting LANE-PROTOCOL:7 (fan-out status) and :268-270 (profile/sccache), and cite the corrected header where §8 opens fanout.

### §5.11 — reciprocal ADR pairs

The pairs name only the new-ADR side, while the governance README requires amendment to be explicit in BOTH records with reciprocal relationship keys. The table's "Required artifact" column says "new accepted ADR" or "reciprocal pair on ADR-0003" without naming the counterpart edit — which line of ADR-0022 the platform `party` amends (and per the adjudication above, possibly none), or that ADR-0003 carries no `amended_by` key today so the reciprocation must create it.

- **Evidence:** `docs/decisions/README.md:9, :26; docs/ideas/ecosystem-plan-DRAFT.md:1332-1343, :1666`
- **Required change:** For G1, G2b and (if G8's integrity-vs-domain-logic distinction is rejected) G8, name the counterpart record, the line amended, and the relationship key to be added on both sides.

### §4.8 E2 — the character sheet

The plan's self-declared completeness test has no delivery vehicle and no executable form. E2 is named the completeness test ("an entity with no home on this screen is a modelling smell") but appears in no slice and no widening — W17 ships E4, W18 ships E1, W11 ships E6, E2 has nothing. Phase 3's only mention is "`app` | wiring, /overview surface", not named as the character sheet and carrying no acceptance. `every_entity_declares_its_components` tests rows in a TSV, not homes on a screen.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:879, :1512, :1606, :1745-1765`
- **Required change:** Give E2 a widening with acceptance, and make the completeness test executable: one row per §4.1 entity mapped to its character-sheet section in `ecosystem-entity-components.tsv`, with a probe that fails on an unmapped entity.

### §4.1 `assignment`; Slice 1

The position/assignment model has no occupancy semantics for absence. `holds_position` is ManyMany so a substitute's concurrent assignment is expressible, but nothing distinguishes the absent holder's claim from the substitute's, and there is no return-right marker. 휴직/복직 is absent from the plan entirely while 육아휴직 복직 is a statutory right and HR+payroll is the first vertical, so it will land on `assignment` regardless — as a reshape of a shipped slice-1 entity rather than a property addition now.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:672-673, :691-696, :1730-1736; docs/ideas/research-sap.md:651, :684; docs/specs/org-editor-primitives-ux.md:143`
- **Required change:** Add an assignment kind (substantive / acting / seconded) and a return-right marker as authored properties on the Tier N `assignment` type when Slice 1 defines it.

### §2 driver 3; §5.4 preconditions

The Korea HOLD is stated as a dependency (correctly, and the plan does not plan to lift it) but its terms of release are not, so a later lane cannot tell whether it has met the bar. §5.4's preconditions for putting attributes on `party` say "the Korea controls have moved off HOLD" without naming the six controls, the requirement for qualified authority, or that native agents produce only non-independent evidence while independent custody is required.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:296-299, :1030-1031; docs/program/console-jurisdiction-register.json:1186`
- **Required change:** Name the six controls and the independent-evidence requirement in §5.4's precondition list, citing the jurisdiction register, so the dependency is checkable rather than merely stated.

### §4.3 relationships; §4.5 인계 완료 query

Work-linked versus person-linked artifacts is load-bearing and unmodelled. §4.5's 인계 완료 query relies on "∃ artifact linked to departing but to no work" and `handover_moves_work_artifacts_only` asserts the boundary, yet §4.3 carries only `work_artifact`; the person-endpoint edge is never declared, so the distinction the PII and handover boundary rests on is inferable only from the probe.

- **Evidence:** `docs/ideas/ecosystem-plan-DRAFT.md:677, :768-772, :1505`
- **Required change:** Add the person-linked artifact edge to §4.3 with its cardinality and owner, and reference it from the 인계 완료 query.

