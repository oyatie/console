> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Revision log — `docs/ideas/ecosystem-plan-DRAFT.md`

Two lines per wave: what the wave changed, and anything found that the brief did not anticipate.
Brief defects are recorded under their own heading and are NOT implemented.

---

## Wave 1 — the signature store, the capacity record, the definer's checks

**Changed.** §5.1's genesis circle is now a platform-principal capability (`PlatformFeature::TenantCreate`
at `platform-rest/src/lib.rs:574`, route `:235`, seeder `:568`) instead of "a migration fact", and the
definer's re-validation is four *named* checks (`org_predicate`, `visibility_predicate`, `chain_linkage`,
`scope_containment`) with recomputation struck on the `serde_json/preserve_order` evidence and the
`store.rs:576-593` precedent bounded to re-validation-as-discipline. §4.5's four traversals rewritten: an
`org_id` predicate on both grant reads, a `scope.level` branch sending `group` to Tier O, 인계 완료 demoted
from a query to one audited assertion that cannot hard-gate offboarding, 대리/전보 split into two
operations, and "why may this person" retargeted at `gov_approvals`. The `approval_signature` entity is
deleted (§4.1, §5.9, Slice 1) and the capacity columns retargeted to `gov_approvals` (§4.0.3, §4.1, §4.4,
§8 Phase 3, Slice 0) with the `audit_events` pair deferred and its 466-site `AuditEvent` reach priced. §4.4
records that an N-node 결재 line already ships (`orgchange/adapter-postgres/src/lib.rs:1477-1488`), so
`UNIQUE (org_id, request_ref)` is one signature per node. §5.2 gains a sixth delta (release-reset). §7
gained the PG-18 GUC instrument trap plus `definer_returns_no_foreign_org_grant` and
`daeri_records_both_parties`, and three probes were retargeted or renamed.

**Not anticipated by the brief.** (1) Deleting `approval_signature` from Tier N invalidated four entity
counts the brief does not mention — "fourteen Tier N types" (§0.7), "the fourteen authority/approval
entities" (§3.2), "14 of 16 entities" (§3.2) and "Fourteen of the sixteen new entities" (§4.6). All four
are now countless references to §4.1, on the same reasoning the brief applies to the CI-job count. §9's
"Sixteen new entities" is left for wave 2 item 2.5, which rewrites that sentence. (2) §8 Phase 3 crate 1
listed "`audit_events` capacity columns" in migrations 0207+; the brief retargets that pair everywhere
except here. Retargeted to `gov_approvals` for wave-1 consistency; wave 2 item 2.6 still owes the
`party`/`party_org_visibility` strike on the same row.

**Variance from the brief, recorded.** Item 1.3(a) says to *delete* the §4.3 rows `signature_grant` and
`signature_on_behalf_of`. They are **retargeted** instead — source endpoint `approval_signature` →
`gov_approvals`, stored as the two new columns. Reason: §4.3's "Stored as" column already carries non-`ont_link`
forms (`users.party_id` FK, `party_org_visibility` row), so a column-stored relationship is at home there,
and §4.3 is the only place in the plan that records cardinality. Deleting would have dropped
"OneOne, nullable" with nowhere else to state it. The brief's stated goal — one signature store, no
`approval_signature` entity — holds either way.

## Wave 2 — the storage substrate: what can and cannot be an `ont_link`

**Changed.** §4.3: `grant_scope`, `position_at_scope` and all four `work_*` edges became scope-descriptor
**properties** `{level, node_id}` on the shipped `AccessScope` vocabulary
(`kernel/core/src/access_scope.rs:28-34`, `:37-40`) instead of `ont_link`s, with the measured FK rejection
(X4b CASE 3a) and the projected-type reason (`instances.rs:1443-1450`) stated inline, plus the hard caveat
that Slice 0 must not publish a `grant_scope` link type declaring `group` or `organization` targets. The
`work_artifact` slug-regex reason is struck for the `0130`/`0132` registry (a new edge kind IS a migration);
`person_artifact` is declared as a row; `lot_derivation` → `lot_split`; the absolute no-`ont_link` rule
became a **reachability** rule naming `create_link`'s zero non-test callers. §4.1: Tier O is two tables,
group-scoped grants moved there with the caller-is-the-org-floor burden, the `party` family is DEFERRED with
R2's five constraints, `org_id`-leads-the-key is recorded as a security control the migration text must
carry, and resolution is decided as a platform-principal operation. §3.1 fixed `0155:78-79` → `:76-77`,
added the consequence sentence, and recorded Tier P as code-gated. §4.2 bounded the central claim to
visibility-within-the-armed-org and stated the group-scope falsifying case and Variant B. §9's cost line
now says two owner-only tables and one definer each; "untyped" deleted; `0076:49-50` → `0075:6,13`.

**Not anticipated by the brief.** Deferring the `party` family out of Slice 0 (item 2.2(b)) contradicts
three rows of §8's **Slice 0** table, which the brief's row-addressed list for wave 2 does not include:
`party` "1 row", `party_org_visibility` "1 row", `users.party_id` "populated". Left alone, the plan would
have asserted both DEFERRED and shipped-in-Slice-0. Resolved the only way that keeps "no lane waits on the
party" true: Slice 0's grant `subject` is the raiser's `users.id`, and §5.1's `visibility_predicate` binds
against `users.org_id = current_setting('app.current_org')` until the edge table lands — the same predicate
against a table that already exists, so **no check is removed from Slice 0**. §5.1's check 2 and §4.5's
definer trace were updated to say so, and the party family was added to "Explicitly out of slice 0".
Also: §3.1's `owner_only_table_allowlist` entry anchor `:118-124` was stale (three entries now span
`:117-129`); corrected while in that row.

**Done early, out of its wave.** Item 4.2's instruction to delete the undeliverable G1 claim *"one durable
identity per natural or legal person, across every tenant and vertical"* from the §4.1 `party` purpose cell
landed here, because item 2.2 rewrote that same cell. Wave 4 still owes the G1 row itself and the three
block-work claims.

## Wave 3 — exposure, gates, slots, the CI premise

**Changed.** §8 Phase 7's *"lands and ships"* became *"CI-provable; exposure remains HOLD for both halves"*
on the counted 27/27 `implementation`/`exposure` HOLDs plus the ledger's *"Nothing in the idea document is
approved work"*, with the capability-row registration rung added and `dispatch_rule`/`hold_rule` named as
fields **nothing enforces** (`grep` returns nothing) against the executable
`validate-console-truth-ledger.mjs:254-257`. The buck2 "graph is already broken" clause is deleted and
replaced with the measured chain (`.buckconfig:15-16` `prelude = bundled`, blake3-pinned DotSlash launcher,
the required job at `ci.yml:164` running a real `tools/buck2 test` at `:192`); X8 is **ANSWERED** and the
wrong "five buck steps" count is dropped for the job name. Phase 0 gained the per-lane 0207+ slot table
(instantiating `LANE-PROTOCOL.md:89`, with the nine already-claimed slots and gap-free contiguity as the
Phase-4 serialiser), the D3 write-path enumeration, prerequisites 5.7a and 5.7b, the benchmark/survey
reconciliation line, the `'KR'`/`'KRW'` non-foreclosure constraint, and X-CITE. §5.11 gained a GATE row
classifying safety pins vs decision pins. The experiment phase was renumbered **6 → 2** and moved before the
trial run (2→3, 3→4, 4→5, 5→6), gained an **ANSWERED** column with record paths and `run.sh` probes, gained
**X4b** as its own row, marked X3/X5/X6 as slice-0 work at ladder rung 4, restated X5 as a constructed query
with a concrete RED input, explained why X7 is blocked rather than pending, and de-circularised the gate.
CI wiring is now per **test** with the four-link chain cited by target name; "14 CI jobs" → every job (ten,
listed once, at a named commit); Bun's 60,624 qualified as Linux x64; "6 platforms" → all platforms; and a
Phase-7 correction rung records LANE-PROTOCOL's three stale spots including **why `.cargo/config.toml` must
stay absent**.

**Brief defect recorded — item 3.4's path does not resolve.** The brief cites
`scripts/console/check-ci-preflight.mjs:430-453`. That file does not exist; the real path is
`scripts/check-ci-preflight.mjs`, where `requireOntologyRestItestReachability` does span `:430-453` and does
exactly what the brief claims. Implemented with the corrected path, plus its own header at `:428` (*"a
per-crate decision with the same shape as this one, not a cleverer regex"*), which turned out to be better
evidence than the line range — it makes the point in the repo's own voice. Recorded because the same wrong
path appears in item 3.1's sibling citation style and a future pass should not re-derive it.

**Not anticipated by the brief.** (1) The renumber invalidated three internal cross-references to "Phase 3"
(the version-space serialisation line, the GATE row's CI-wiring clause, and Phase 7's CI-wiring rung) plus
X9's own "the by-crate queue (Phase 3)"; all four now read Phase 4. Wave 5 item 5.3(a) speaks of "a hard
Phase-3 ordering constraint" — that is now **Phase 4**, and wave 5 must write it that way. (2) `tokenize_sql`
keeps `_` as well as alphanumerics, so the brief's "every non-alphanumeric character" is one character too
broad; stated precisely instead. The conclusion (`only`/`public` resolve as table names) is unaffected.

## Wave 4 — the governance surface

**Changed.** §5.11's preamble now records that **reciprocity is machine-enforced but clause compatibility is
not** (`check-adrs.mjs:23-27` reciprocates only `amends`/`supersedes`; the loop at `:399-406` checks only that
the target declares the key; `related` is validated as an array at `:248-249`) — which is the stated reason
G2 and G2b merge — and that **numbers are assigned centrally**, naming the observed failure of four judges
each computing "next free after ADR-0026". The G-table was rewritten row by row into the D/N records with the
allocation table (ADR-0027 to ADR-0036) and the reciprocation mechanics carried in full, including that
`ADR-0003`, `ADR-0002`, `ADR-0025`, `ADR-0009` and `ADR-0022` **carry no `amended_by` key today**, so
reciprocation must create it. G1 → WITHDRAWN (the string "org-scoped" appears **zero** times in ADR-0022;
`## Context` is at `:23`, `## Decision` at `:31-39`), and its three downstream block-work claims — the header,
§8 Phase 7's first rung and §9's standing — now name D2 and D3. G6 struck (no charter clause; `ADR-0023:148`
is a header, the canvas bullet at `:153-154` carries none, *"enters as its own charter"* is at `:156` on a
different bullet) with W10 marked deferred-by-follow-up rather than charter-gated. G7 struck structurally
(DN-0003 is a design note and cannot take an ADR pair). G8 reduced to two SQL invariants, with `ADR-0001:23`
noted as a Consequences bullet. G9 → D3, BLOCKING and retroactive, with the two exclusions bound to
(file, function) pairs and cited to `audit-coverage/src/lib.rs:90-111` plus the test name — never to the ADR.
§0.16 lost "sole" and gained the second shipped derivation (`request-context/src/lib.rs:421-422`), the realtime
fan-out, R11, and a **present-tense** trigger so D2 no longer waits on `Role` deletion; §5.3's C4b and C5 say
the same, and the onboarding seeder is named as a write site. Segregation of duties is decided **IN**, as a
grant-authoring-time constraint inside N3 with widening W19 and a probe, and §1 principle 2 gained the note
that additive-only constrains the fold, not authoring.

**Brief defects recorded — three wrong paths and one unresolvable instruction.**
(1) Item 4.6(b) cites `docs/ideas/no-code-operational-logic.md:211` and `docs/ideas/operations-intelligence.md:170`.
Neither exists; both live under **`docs/specs/`**, where the quoted lines resolve exactly. Implemented with the
corrected paths. (2) Item 4.5's G3 acceptance condition requires *"add the delegation-transitivity arm to
`backend/crates/governance/adapter-postgres/src/lib.rs:585-604`"*. **Not implemented.** That range is
`four_eyes_consume_conn` — a bind-match-and-consume `INSERT … ON CONFLICT` for `gov_approval_consumptions` —
and `grep -n delegat` over that entire file returns **zero hits**. There is no delegation arm there to extend,
and no substitute was invented. The other half of the acceptance condition, `CHECK (delegator_id <> delegate_id)`
in the same migration, is self-contained and **was** implemented.

**Not anticipated by the brief.** `ADR-0023:158` is cited in §5.4 for *"the multi-jurisdiction PII program"*; the
follow-up list actually places that bullet at **`:157`** (`:158` is "Object graph explorer"). Item 6.4 lists
`ADR-0023:158` among the anchors to *leave as line numbers* — leaving it would have left it wrong. Corrected to
`:157` here, alongside the `:154-155` → `:153-154` fix in the same family.

## Wave 5 — the deferred-decision paragraphs

**Changed.** §5.5 downgraded the claim to **COST**-as-a-query at all three sites (revenue and profit need the
peer plan's account master), named the three shipped parallel money stores as the reconciliation backlog,
recorded N5's three Slice-0 prerequisites, demoted "no production data" to an assertion and added **V-1**
(the voucher is gate-marked audited at `0160:21`, so `accounting_date` is irreversible once landed), corrected
four period-lock sites to **five** (orgchange has two guards, `:611` and `:744`), decided 확정-requires-an-open-period
in one place so W14 stops contradicting itself, stated the single-valued dimension as this plan's take with
the distributed case owned by the peer plan, flagged `economics_is_a_view`'s dependency on X-T9b, and moved
**206 → 205** at all three sites with the commit named. §5.6 deleted the materialise row (contradicted its own
row 5, §4.6, and ADR-0021 decision 4, and was mis-keyed on a `PRIMARY KEY (org_id)` table), re-keyed to
per `(org, user)`, and recorded that **both** counters must bump with the five measured call sites. §4.6 added
the bundle-schema ordering constraint (an undeclared attribute fails `Entities::from_entities` and denies
everything) and the two shipped declarative systems plus the plpgsql `create`-action insert at `0165:1024-1041`.
§5.8 kept the row CHECK but added the `FOR UPDATE` mechanism, withdrew the two over-claiming sentences, stated
the aggregate honestly, fixed `production_plans` → **`production_operations`** at all three sites, and carried
N4's three non-foreclosure constraints. §5.9 decided the correction axis as a **stated deferral** with its
consequence, and Slice 1 gained the assignment kind and return-right marker. §4.0's "systems light up" sentence
is deleted; §4.0.2 gained the two requires-code rows, the handler count, and the DN-0003 invariant-1 answer.

**Not anticipated by the brief.** (1) §0.11's *"That is the cache-invalidation key the realtime question
needs"* becomes false once §5.6 keys per `(org, user)`. §0.11 is in no wave's section index. Softened to "half
of the key", with the reason (`PRIMARY KEY (org_id)`, and assignment writes do not bump it). (2) W12's
acceptance row said *"`policy_versions` invalidation"* — now both counters. (3) The brief's item 5.4(a) cites a
domain `state.consume(quantity)` at `inventory/adapter-postgres/src/lib.rs:411`; `:411` is the event `INSERT`
and the domain call is at **`:406`**. Cited both, at their real lines. (4) §2 driver 2 needed the qualification
the brief asks for in 5.5(a) — added there, since driver 2 is the sentence that promises replay is free.

## Wave 6 — anchors and counted facts

**Changed.** §0.1 is re-anchored on **quoted sentences plus heading names** (`## Where employees belong`,
`## Recommended Direction`, `## The two hard problems`) with `:89-92`, `:545-546`, `:575-579` and `:83-87`
dropped and the review's `:116`/`:571`/`:606` **not** substituted; the same treatment was swept across §0.3
(`:125` was a blank line), §0.5 (`:214-218`), §0.6 (`:378-381`) and §3.2 Option 2 (`:545-546`, `:575-579`).
The preamble now names **which** citation form applies where and why, and points at X-CITE. §0.12's heading
and body carry the **reachability** wording, plus the two findings it lacked: `to_object_type_id` appears
**zero** times in the write module so it is decoration, and the proposed guard is **absent** — `validate_draft`
exists (`adapter-postgres/src/lib.rs:416`, `:458`) but its entire link-type validation is `:1142-1151` and
checks duplicate `stable_key` only, so `link_type_alone_is_rejected` is observed RED today. §0.13 corrects the
route list to `:213-228` and **14** paths, names the attach route
(`POST /api/v1/ontology/object-types/{stable_key}/policies` backed by `0205`), adds X2's sharper consequence
(an unpoliced row is `404` by id, deliberately, so a 403 is not an existence oracle), and restates the
consequence as a `view` permit. Nineteen bare basenames are path-qualified on first use; `README.md:12` →
`docs/decisions/README.md:12`; `lib.rs` disambiguated at both of its ambiguous sites.
`fanout-plan-DRAFT.md:243` is cited only for the derived-facts rule, never for anchor discipline.

**Not anticipated by the brief.** Two tables were left structurally broken by earlier waves: the §5.11 SoD row
(wave 4) and GATE row (wave 3) each carried **three** cells in a four-column table, so both would have rendered
with a column shifted. Found by a pipe-count check across every table block, now run as a habit at the end of
each wave. Nothing else in the document mismatches.

**Brief item deviated from on evidence.** Item 6.3 describes `validate_draft` as *"confirmed absent"*. The
**function** is present at two call sites; what is absent is the **check** the plan proposes to add to it. The
brief's own evidence (`:1142-1151` is the entire link-type validation) says exactly that, so the conclusion —
`link_type_alone_is_rejected` is RED today — is unchanged; only the wording is accurate now. Writing "absent"
would have sent an implementer to create a function that exists.

## Wave 7 — vocabulary, ergonomics, recorded costs

**Changed.** §4.4's `policy_role_conditions` row narrows the **write path** to `{branch, team}` × `{equals, in}`
(the resolver returns `None` on anything else, `authz/src/lib.rs:1404-1430`) while the DB CHECK stays permissive;
the fail-closed **whole-role void** is recorded as CORRECT and never to be relaxed, `0065:101-103`'s contrary
"inert metadata" comment is struck, competence is placed as a subject-side condition attribute, X-T2f is required
first, and **직무/직급 are decided as having no substrate** (17 attribute literals, neither among them) arriving
in W6. `notices` gains a **fourth** gap — the org-wide fan-out at `notices/adapter-postgres/src/lib.rs:413-433`,
where **both** SQL variants end `org_id = $1 AND is_active = true`, so a 반려 notice reaches every active user in
the org — with per-recipient DDL and the shipped snapshot as `obligation_notifies_line_as_raised`'s known-bad
control. §4.1's vocabulary paragraph lists all **fourteen** primitives including the dropped PolicyRole hook,
defends `ReportingLine`'s exclusion, and records the four-way `position` `stable_key` collision for PORTING.md.
§4.7 gained the regulation-renderability differentiator and its probe, the per-day cumulative quota decision, and
the superlative is qualified to "cited here". §4.8 gained E7 (the bar §4.7 promised), an executable E2
completeness test, W20 to ship both, and E4 qualified so the fold simulator inherits nothing from Cedar
simulation. §1 principles 3 and 4 corrected. §5.4 now prices the alternative in a table, names all six control
ids, and quotes `unhold_authority` and `uncertainty_rule` verbatim — asserting no conclusion and proposing no
unholding.

**Not anticipated by the brief.** Item 7.4(a) says to give E2 "a widening with acceptance" but no widening
existed and the widening list ends at W18; added **W20** rather than overloading an existing row, and put E7
there too since both are measurements on the same surface. Item 7.1(b) offers "either add 직무/직급 to §4.1 or
state which widening brings them" — chose the second, and named **W6** specifically, because that is where
`employment_type`'s accrual/insurance/severance rules already need them.

**Table-integrity check is now part of each wave.** Two four-column §5.11 rows written in waves 3 and 4 had only
three cells; caught in wave 6, and the check passes over every table block at wave 7.

## Post-wave-7 correction — the D4 count changed underneath the revision

`5a4cdd0ba` (`docs(ideas): D4 consensus …`) landed on this branch **between wave 6 and wave 7**, by another
agent, touching only `docs/ideas/d4-frontend-charter.md`. It did not touch the plan or this log, so no wave was
merged over. But it changed the fact wave 4 had just recorded: the charter now names a **third** amendment
target — adding a `ui` layer to `ADR-0001`'s **enumerated** crate family amends ADR-0001 — where the brief's
allocation table and my §5.11 D4 row both say "two records".

**Not resolved by inventing ADR-0037.** Numbers are assigned centrally in this workflow, and the plan now
carries the record of four judges each computing "next free" and all four claiming 0027. Instead: the D4 row and
the allocation table now say **at least two**, carry the third target as an explicitly *(unallocated)* row, and
state that the count is the charter's to give at acceptance rather than this plan's to restate. The integrator
allocates it with the rest in the same atomic commit.

This is the migration-count lesson arriving live: a derived count restated in a second document went stale
within one session of being written.

---

## Global-consistency pass — the question the section-clustered waves never asked

Waves 1-7 were section-clustered, and that is why verification returned REGRESSIONS_INTRODUCED twice: a
wave fixed §4.5 correctly while §5.10 kept asserting the mechanism §4.5 had withdrawn, because no step asked
*"what else in this document refers to what I just changed?"* This pass asks it exhaustively. The checklist
below is the deliverable; the seven named repairs are a subset of it.

**Commits:** `4435e94e9` (인계 완료 + four headings + D3), `07bcfde12` (R3 + the 부서 decision),
`7a54a5d09` (D1), `a94b2586a` (D2 + the citation and audited-table sweep), `36ba15c84` (§9, last),
`eab23ffa9` (two remaining headings).

### The sweep, by class — what each turned up

**1. Headings advertising a claim the body withdrew.** R4 was one of **seven**.
- §5.8 *"conservation as a row CHECK"* — the body withdrew sufficiency (two concurrent splits both write
  (100, 60, 40)). → *"conservation by a parent-row lock with a row CHECK beneath it"*.
- §0.11 *"`policy_versions` is the key"* — the body says **half** the key. → *"is HALF the cache key"*.
- §0.16 *"deleting `Role` deletes the **only** path to `BranchScope::All`"* — the body says *"It is not the
  sole one"* and that `Role` deletion is **not** the trigger. → two derivations, divergence PRESENT-tense.
- §5.10 *"**Party** lifetime derived from a contract"* — in this plan `party` is the handle that is 永久 and
  never hard-deleted. The section is about a bounded `org_unit`. → *"A temporary **UNIT's** lifetime"*, and
  the in-body *"when a party dissolves"* → *"when a unit dissolves"*. **The most dangerous of the seven:** a
  lane grepping "party lifetime" would have concluded the identity handle expires with a contract.
- §4.4 *"Why the existing mechanisms cannot be widened"* — three of its four rows extend or narrow.
  → *"What each existing mechanism can and cannot absorb"*, and its lead sentence with it.
- §3.1 *"the four storage tiers that already exist"* — the body adds Tier P and wave 6 already corrected
  principle 4 to *"optionally projected"*. → *"four CI-enforced … plus one CODE-gated projection"*.
- §4.0.3 *"one missing field"* — two columns, four missing things in its own table, target moved.
  → the claim without the count.

**2. §9, the ADR block, read against every other section LAST.** Four disagreements, not one.
- **R1**: still `on audit_events`. Retargeted, with the `built_in_audited_tables()` reversibility argument
  and the 466-site price of the deferred pair.
- **Authority "as ontology instance types" stated with no exception** — X4b *measured* that a `Group`-scoped
  grant cannot be one. §9 would have recorded a claim an experiment refuted.
- **Drivers (2) still carried "replay must be free" UNQUALIFIED** — the form §2 driver 2 corrected in wave 5.
  §9 is downstream of §2 and nothing had propagated the valid-time qualification into it.
- **The cost line, the alternatives list, the standing paragraph and the consequences** all still described
  the Tier O `party`. All four retargeted after D1.
- The preamble's *"corrects itself in three places"* → four.

**3. §7 probes binding a mechanism the plan no longer specifies.** D1's two, and no others survived the read.
- `party_not_readable_as_console_rt` asserted a **denial** — only true of Tier O. Renamed
  `party_is_invisible_and_unmintable_from_a_tenant`: zero rows, `count(*)` = 0, and a refused INSERT.
- `no_new_gate_classification` required **both** tables in `owner_only_table_allowlist`. Now exactly one, and
  an owner-only `party` is one of its known-bad controls.
- Checked and sound: `fold_is_scope_parameterised` and `requirement_3` (the Tier O store survives),
  `capacity_recorded_on_every_authority_mutation` (already scoped to `gov_approvals`), `basis_survives_the_chain`
  (uses the shipped `audit_events.reason`, not the deferred pair), `lot_conservation` (asserts the per-row
  property only; the concurrency property is a separate probe), `visibility_*`, `definer_*`, `link_type_alone_is_rejected`.
- **No probe asserted 인계 완료 as a gate** — the withdrawal had reached §7 already. The gate survived only in
  §5.10's table and W4's acceptance.

**4. Phase 3/4 crate tables, Phase 7 prepwork, W1-W20, Slice 0/1 rows.**
- **No widening landed the party family at all.** `party`, `party_org_visibility` and the two `party_id`
  columns were DEFERRED with nothing carrying them. W2 is the first widening that cannot proceed without
  them; W2 now lands them with acceptance.
- **W1 said "party-keyed recipient REPLACING the org-composite FK".** `notices` and `notice_receipts` are
  **both gate-marked audited** (`0162:12`, `:40`), and `recipient_user_id` is `NOT NULL` (`:45`) — so a column
  swap is a gate violation. W1 and §4.4 now spell the additive form and W1 carries the fourth gap and the
  non-members-receive-nothing half of its probe, which §4.4 required and W1 omitted.
- **W4's acceptance still said "인계 완료 queries GREEN"** (R2) — now the assertion, plus the fixed-authority
  count without which hard-gating is unavailable.
- **W5** did not carry §4.7's `(period, cumulative_limit)` decision, which named W5. Added, with the
  `AccessScopeLevel` department-level widening beside it.
- **§5.11 D3 "two 0207+ rows" vs Phase 7 rung ② "four"** — the same list at two horizons (Slice 0 vs W16).
  Said so rather than reconciled to one number.
- Phase 4 crate 1, the Slice-0 `party` row and "Explicitly out of slice 0" all attributed the deferral to
  **irreversibility**, which is exact only for `users.party_id`. Two reasons now stated separately.

**5. Cost and economics sentences.**
- §9's cost line: two owner-only tables → **one**, each item priced, and what Slice 0 actually pays.
- §4.1's Tier O heading: 2 new tables → **1**. The in-body "the second Tier O table" with it.
- §3.2 Option 1's Pros gained "and no owner-only table for the handle itself".
- §5.5 item 2 and the Slice-0 voucher row put two columns on `finance_gl_voucher_lines` while the
  irreversibility warning cited only the header (`0160:21`). The lines carry their own marker at `0160:56`.
- Checked and sound: the migration count (205 as of `8e76dffb4`), the CI job count ("every job"), the 466
  `with_audit` sites, the five period-lock call sites, the 27/27 HOLDs, the ADR allocation count.

**6. Enumerations against shipped vocabularies — every one re-read, not sampled.**
- **R3, the worst of the four regressions.** `AccessScopeLevel` is `{Group, Org, Region, Branch, Worksite}`
  (`access_scope.rs:28-34`). §4.1's grant row and §4.5's definer trace said
  `{org_unit, organization, region, branch, worksite}`. Corrected at both, plus §3.1's uniformity sentence
  and §4.3's published-schema caveat, which named the two vocabularies interchangeably.
- Verified correct as written: `LinkCardinality` / `ont_link_types.cardinality` (`0152:77`), `ActionDispatch`
  / `ont_action_types.dispatch` (`0152:99`), `AUTHORING_ACTIONS` = 5 (`authoring.rs:246-252`),
  `messenger_threads.kind` = 4 (`0012:9`), `notices.status` = 2 (`0162:22`),
  `work_order_approval_steps.role` = 3 (`0008:63`) and `step_order BETWEEN 1 AND 3` (`:62`),
  `policy_role_conditions.attribute` = **17** (`0065:110-127`) and `operator` = 3 (`:129`),
  `clearance_assignments.status` = 3 (`0147:20`), `audit_stream_event_labels.sensitivity` = 3,
  the voucher FSM = 5 (`0160:28`), `inventory_consumption_events.source_kind` = 2 (`0156:87`),
  `group_role_grants.group_role` = 3 (`0060:45`), `ont_instances.lifecycle_state` = 5 (`0155:27`),
  `Feature::ALL` = `[Self; 96]` (`authz/src/lib.rs:372`), `LANE_TYPES: [&str; 5]`,
  `0130`'s **twelve** seeded `link_types` labels (`:38-49`), the **fourteen** org primitives
  (`org-editor-primitives-ux.md:468`) and their separation sentence (`:256`).
- One near-miss recorded: `0172:10` CHECKs a column named `currency`, not `currency_code`.

### The two decisions taken under step 3

**D1 — `party` is NOT in Tier O.** Constraint 4 of the deferral wins and every Tier O reference to `party`
goes. Four grounds, and it is the smaller claim: (1) the row holds nothing a tenant reads — a tenant already
has the id from its **own** edge row, and §4.2 already located the confidential fact in
`party_org_visibility`; (2) **the sentinel org is shipped and already holds platform rows that outlive
tenants** — `0036:224` seeds `organizations` `…00face`, slug `platform`, status ARCHIVED, its reason in its own
text at `:217-221`, excluded from `platform_list_organizations()` at `:121`, and
`0051_platform_remove_organization.sql:34` **re-homes a removed tenant's `audit_events` there** so *"the
immutable record of the action survives verbatim under the platform tier"*; (3) Tier T + FORCE RLS closes X4's
measured cardinality leak (`count(*)` = 2 where org A held one edge) **by omission**, which is DN-0003
invariant 5's own wording, rather than by denial; (4) the same policy's `WITH CHECK` plus a column CHECK
pinning `org_id` to the sentinel make a tenant-armed INSERT impossible, so *"resolution is a
platform-principal operation"* is enforced by DDL rather than by a handler nobody has written. Removes one
allowlist entry, one gate classification and one definer. Tier O is **not** emptied — the `Group`-scoped grant
store stays, because X4b measured a sibling org reading 0 rows from Tier N.

**R3's 부서 question — decided, not left open.** A 부서-scoped grant has **no scope level** in slices 0/1, and
the plan does not invent one. `AccessScopeLevel` is matched **exhaustively with no wildcard** at
`access_scope.rs:86-98` and `authz/src/lib.rs:1524-1538`, so a sixth variant is a kernel change, a compile
error at both sites and a decided `branch_scope_for_org` projection — code, not an authored row.
`policy_role_conditions.attribute` **does** hold `department` (`0065:115`) but the resolver evaluates only
`branch`|`team` (`authz/src/lib.rs:1403-1429`, `_ => return None`), so it is writable and resolver-void, which
is why §4.4 narrows the write path away from it. Slice 0's scopes are 현장 = `Worksite` and 본사 = `Org`, both
shipped; a 부서 bounds a decision as a **competent `org_unit` instance** named by `delegation_rule`, an instance
reference rather than a level. A department level is W5 work and must arrive with its projection arm.

### Not anticipated by the findings

1. **§0.4 contradicted D1's own constraint 1.** It read *"`users.party_id` and `employees.party_id` are plain
   single-column **FKs** to `party(id)`"* while constraint 1 forbids a cross-tenant identifier as a FOREIGN
   KEY — the constraint §4.1 cites two paragraphs later to explain why `on_behalf_of_party_id` carries no FK.
   §0.4's correction of the input is about **single-column versus composite**; that half stands, and all three
   columns are bare nullable `UUID`s. §4.1's and §4.3's cells said "FK" too; all corrected.
2. **`gov_approvals`' approver FK is `0153:79`, not `:78`.** `:78` is the `requested_by` twin. Three sites
   cited `:78` for the approver while §4.1 cited `:79` correctly — one fact, two line numbers, in a claim
   ("the cross-org FK is the one real blocker") that W1 exists to address.
3. **`finance_gl_voucher_lines` carries its own audited marker** (`0160:56`). The plan's irreversibility
   warning named only the header while two of its own items add columns to the lines.
4. **`derived_from` is already a seeded `link_types` label** (`0130:43`, *"Source was produced from the
   destination (lineage)"*), which makes N4 constraint 1 — lineage edges may never live in `object_links` —
   a warning against something that looks pre-built. Named, because that is the concrete shape of the mistake.
5. **Constraint 5 of the party deferral was dead letter.** *"Any eventual edge FK is `RESTRICT`/`NO ACTION`"*
   presumes an FK constraint 1 forbids. Bounded as the fallback shape *if* constraint 1 is ever amended,
   rather than deleted, so the posture survives if the decision is revisited.
6. **§4.6's reason for `party` not being Tier N was wrong once D1 landed.** `ont_instances.org_id NOT NULL`
   does not forbid a row homed at the sentinel org. The real reason is that minting is a platform-principal
   write and **every ontology write runs on the command pool that is `None` wherever this ships** (§8) — so a
   Tier N handle would be green on every PR and dead in production. A better reason, and it was already in
   the plan, twenty sections away.
7. **The Phase 0 lead said "both of which"** after D3 restored the third row to its table.

### Discipline notes

- `git stash` and `git reset` were not used. Every commit staged by path.
- `Status: PENDING APPROVAL` unchanged. No approval added, no HOLD softened, no gate weakened, no Korea
  conclusion asserted. `party_is_invisible_and_unmintable_from_a_tenant` and `no_new_gate_classification` are
  both **stricter** after this pass, not looser.
- Table-integrity check (pipe count per block, discounting escaped `\|`) run at the end of every commit:
  clean.
- No code was executed. Every claim above is a read of the file cited.

---

## Adversarial re-read of the global-consistency pass — what the repair itself broke

Read fresh against code, not against the findings list. R1-R4 and D1-D3 all verified **fixed** at every site
(§9 now says `gov_approvals`; 인계 완료 is an assertion in §5.10/§4.0.1/W4; §4.5's branch and §4.1's grant row
carry `{Group, Org, Region, Branch, Worksite}`, re-verified at `access_scope.rs:28-34`; §5.8's heading matches
its body; the Phase-0 table has three rows). Ten new contradictions, three of them **written by this pass**.

**Written by this pass (regressions).**
1. **"It is the ONE owner-only table this plan adds"** (§4.1, `7a54a5d09`) and **"ONE new owner-only table"**
   (§9, `36ba15c84`) are false by the plan's own text: `party_link` is *"**Tier O** and reuses the shipped
   `group_role_grants` definer pattern"* (§4.1), `no (O)` in §4.3's `controls` row, and *"`party_link` control
   edges (Tier O)"* at W7. Both Tier O tables are deferred, so deferral cannot be the distinction. §9's
   Consequences counts two definer surfaces where three are scheduled, and §7's probe *"names it alone"*.
   The gate reads only the compiled list (`tenant-isolation/src/lib.rs:115-130`, 3 entries today; the
   `-- console-gate: owner-only-table` markers at `0060:30`,`:39` have no reader) — so each is its own entry.
2. **§9's "settled figure" undercounts Slice 0** (`36ba15c84`): *"Slice 0 pays for exactly one of those tables —
   `work` — plus the two `gov_approvals` columns and the definer"* vs Phase 4 crate 1's own list (*"voucher
   `accounting_date`"*), the Slice-0 addition row (`accounting_date` + line `branch_id` + line dimension, all
   **irreversible**, `0160:21`/`:56` verified) and §5.5's *"N5's three prerequisites, and they DO block Slice
   0"*. §9's "three new tenant tables … plus" list also omits W1's `notice_audience_parties`.
3. **D1's load-bearing ground cites a comment** (`7a54a5d09`): `0051…:34` is header prose; the executable
   re-homing is `UPDATE audit_events SET org_id = sentinel_org` at **`0051:195-196`**. Cited that way twice
   (§4.1 ground 2, §9 "Why chosen"), against the preamble's own "cites **executable** code or DDL". Same
   shape: §5.11 cites `audit-coverage/src/lib.rs:90-111` for a set that ends at **`:107`** (`:109-111` is
   `check_workspace`).

**Left standing by it (gaps of the exact shape it swept for).**
4. §3.2 **Option 1 — the RECOMMENDED option** — still reads *"Tier N authority"* in its heading and *"the
   authority/approval entities of §4.1 as ontology instance types"* in its body, with **no X4b exception**.
   That is the §9 defect this pass fixed, one section earlier, unfixed.
5. *"zero new gate classifications"* survives in §3.2 Option 1's Pros (the sentence this pass edited) and
   §4.2's Consequences, while §4.1 removes *"one gate classification"* of two and §7's probe asserts
   **exactly one**. The probe is still named `no_new_gate_classification`.
6. **X6 resurrects the deleted materialise option:** *"materialization keyed on `policy_versions` if not"* —
   §5.6 deleted that row as ADR-0021-violating and **mis-keyed**, and N1 records *"no cross-request
   materialisation"*. X6 is one of the three experiments gating Slice-0 green.
7. **Slice 0 cannot satisfy two of its own `slice0_*` probes:** its grants are *"2 instances … 현장 …
   + one at a **different** 현장"* and *"both `Worksite`-scoped"*, but `slice0_본사_may_still_approve` needs
   *"본사 … at company scope"* and `slice0_second_band` routes *">band → 본사"*. Three grants, not two.
   Separately, Acceptance says *"Every `slice0_*` probe"*, which excludes `daeri_records_both_parties` — the
   probe the Slice-0 addition row makes non-optional.
8. **One fact, two ranges, both outside their function:** §4.2 cites `0060:90-92` and §7 `0060:88-91` for
   `group_role_grants_for_user`'s `EXCEPTION` restore. That function is `0060:99-126` and its handler is
   **`:120-122`**; `:88-92` belongs to `group_member_org_ids`. This is the `0153:78`/`:79` defect again.
9. §4.6 *"Three entities must be ordinary tables … all three are Tier T"* (this pass's clause) undercounts —
   `work` is the fourth (§0.14), plus `worksite_contract`, `lot` — in the section whose previous bullet says
   *"no count is restated here, because counts in this plan have rotted twice"*.
10. §7 Observability still watches for *"a scope-expression bug"*, a mechanism §0.17 **deleted**. And
    §4.0.1's `record` row sends the capacity gap to **§4.0.2**; it lives in §4.0.3.

No code executed. `Status: PENDING APPROVAL` untouched; nothing above softens a HOLD or a gate.

## Verification pass over the global-consistency commits — two citations went the wrong way

R1-R4 and D1-D3 all confirmed fixed against the current text, and 7/7 headings, both rewritten §7 probes and
12 of the 16 "verified sound" enumerations re-read exact. Two numbers introduced by `a94b2586a` are wrong, and
one entry in the enumeration list was marked verified while it is off:

1. **`gov_approvals`' approver FK is `0153:78`, NOT `:79`.** Read
   `backend/crates/platform/db/migrations/0153_create_governance.sql`: `:77` is
   `FOREIGN KEY (requested_by, org_id)`, `:78` is `FOREIGN KEY (approver_id, org_id)`, `:79` is `);`. The pass
   moved three **correct** `:78` cites to `:79` (§0.4-area line 117, §4.4's `gov_approvals` row, §5.9's
   blocker sentence) to agree with §4.1, which was the wrong one, and added prose at two of them asserting
   *"`:78` is the `requested_by` FK"* — it is `:77`. Four sites now carry the off-by-one, inside the claim W1
   exists to fix. The plan's other cites into this file (`:71`, `:74`, `:76`) all match exactly, so the
   convention is not in doubt and there is only one copy of the file outside `console-lanes/`.
2. **`derived_from` is `0130:44`, not `0130:43`** (`:43` is `belongs_to`). The quoted description is right; only
   the number is wrong. Twelve labels at `0130:38-49`, as claimed.
3. **`ont_action_types.dispatch` CHECK is `0152:97`, not `0152:99`** (`:99` is `control_points`). §4.6's cite,
   pre-existing since `fc704f29f` — so this is the enumeration checklist over-claiming "every one re-read",
   not a regression.

Also open, same shape as the seven headings: **`party_link` is still Tier O** (§4.1's deferred-constraint
block, W7) while §4.1's Tier O heading says **1** new table, §4.1 says *"the ONE owner-only table this plan
adds"*, and §9's cost bullet says *"ONE new owner-only table"*. The tenant bullet beside it names its
widening-horizon tables; the owner-only bullet names none, so a lane greps that sentence and concludes W7's
`party_link` needs no allowlist entry. Pre-existing (the pre-revision text said "two", also excluding
`party_link`), but it is the same reader-facing failure the heading sweep was for.

Nits, one fact with two numbers each: the custom-role resolver is `authz/src/lib.rs:1403-1429` in §4.1 and
`:1404-1430` in §4.4 (the fn is 1399-1430); the POSTED trigger is `0160:78-118` in W14 and `0160:79` in §7
(the fn opens at `:78`); `allowed_audit_exclusions()` is cited `:90-111` and ends at `:107`. And W1 calls the
recipient change *"All of it ADDITIVE"* while its acceptance asks for `recipient_user_id` *"present,
nullable"* — `0162:45` is `NOT NULL`, so that leg is a relaxation the gate permits, not an addition.

## Citation-class sweep, wave 0 — the baseline and the check that holds it

Two consecutive consistency passes each shipped new defects and every one was a line number. So this wave
fixes no citation. It measures the population and lands the gate: `scripts/console/verify-doc-citations.mjs`,
wired as `npm run check:doc-citations`.

**The argument the script encodes.** A line number cannot be checked. You can confirm the file has that many
lines; you cannot confirm the line says what the citation claims — so an off-by-one survives every review that
does not open both files side by side, which is how `0153:79`, `0130:43` and `0152:99` each shipped. A symbol
or a quoted fragment is checkable by `grep`: it resolves or it does not. The script's verdicts follow that
split, and the `--max-unverifiable=N` threshold is what lets the sweep ratchet in waves instead of demanding
one perfect pass.

**Baseline on `docs/ideas/ecosystem-plan-DRAFT.md`, nothing fixed:**

| verdict | count | meaning |
|---|---|---|
| total citations | **719** | parsed from the doc's own grammar, fenced blocks excluded |
| RESOLVES | **2** | a symbol found in the cited file — `authoring.rs` `simulate_inner`, `seed.rs` `BUILTIN_CATALOG_VERSION` |
| UNVERIFIABLE | **566** | line numbers. **79% of all 719, and 566 of the 568 that make any claim about content** |
| BROKEN | **0** | nothing provably wrong survives, which is why the control below matters |
| FILE-ONLY | 129 | a file named with no checkable claim attached |
| MISSING | 22 | a file named that is not in the repo — planned artifacts and deliberate absences, so reported, not fatal |

566 unverifiable against 2 resolving is the finding. The document's reasoning was never the problem; almost
none of its evidence is machine-checkable, so consistency passes had nothing to check against but each other,
and agreeing with a wrong neighbour was indistinguishable from being right.

Of the 566, **2 are SUSPECT** — a bare `:line` past the end of the file the script infers for it:

- §"deferred 규제 PII epic" (line 1696) — `ledger :174`, but the nearest preceding citation is `ADR-0023:157`
  and ADR-0023 has 166 lines. The ledger has 1045, so `:174` is fine *for the ledger*: the antecedent lives in
  the English word "ledger", not in the citation.
- §LANE-PROTOCOL staleness row (line 2684) — `:268-269` for a file the row named three spans earlier, with a
  bare `` `0204` `` in between that captured the antecedent. LANE-PROTOCOL has 270 lines, so again the number
  is right and the binding is not.

Neither is reported BROKEN, deliberately: the inheritance is the script's guess, and asserting a guess is wrong
is the exact move that propagated `:79`. But both prove the bare `:N` form is unsound even when its digits are
correct — the reader binds it by prose, and prose drifts.

**Known-bad control, observed RED.** A probe with no demonstrated failure mode is not evidence. Fixture at
`scratchpad/known-bad-citations.md` carries eight citations with pre-declared verdicts:

```
$ node scripts/console/verify-doc-citations.mjs .../known-bad-citations.md
BROKEN — provably wrong, fix these first
  known-bad-citations.md:9   `.../authz/src/lib.rs` `fn no_such_function_exists_here()`  → not found in file
  known-bad-citations.md:12  `.../authz/src/totally_made_up.rs:42`                       → file not found
  known-bad-citations.md:15  `9998:12`                                                   → no migration with that number
  known-bad-citations.md:18  `0153_create_governance.sql:99999`                          → line 99999 past end of file (141 lines)
RESOLVES — verifiable by grep
  known-bad-citations.md:23  authz/src/lib.rs                 found "pub enum Feature"
  known-bad-citations.md:26  .../instances.rs                 found "sync_property_links_tx"
UNVERIFIABLE
  known-bad-citations.md:31  0153_create_governance.sql:78
  known-bad-citations.md:33  0153_create_governance.sql:79  (inherited file)
  total 8 · RESOLVES 2 · UNVERIFIABLE 2 · BROKEN 4        EXIT=1
```

All eight match. `total citations : 8` also confirms the seven decoys in that fixture — `Feature::ALL`,
`app.current_org`, `LISTEN/NOTIFY`, `prelude/`, `ontology/*`, `.sql`, `(id, org_id)` — were not counted as
citations. Threshold checked both directions: `--max-unverifiable=2` exits 0 on a doc with 2, and
`--max-unverifiable=1` fails it.

**Two defects found in the verifier itself while taking this baseline**, both worth recording because both are
the document's own failure mode reappearing in the tool:

1. The first version inherited the antecedent file across a *failed* resolution, so `engine.rs:370-391`
   (ambiguous — `platform/authz/src/cedar_pbac/engine.rs` and `workflow/runtime/src/engine.rs` both match)
   left a stale binding and the following `:403-424` and `:430` were reported BROKEN against a file nobody
   cited. The tool was propagating a false fact to stay consistent. Fixed: an unresolved anchor clears the
   antecedent, and ambiguity is UNVERIFIABLE, never BROKEN.
2. The first run reported 59 BROKEN; 57 of them were the checker's fault, from three causes. It read
   directories and globs as files (`prelude/`, `platform/db`, `ontology/*`) and prose containing a slash as a
   path (`LISTEN/NOTIFY`, `DRAFT/BALANCE_CHECKED/…`). It looked for design notes only in `docs/decisions/`,
   so every `DN-0003` cite failed — the file is at `docs/decisions/notes/`. And it made no distinction
   between a citation that is wrong and a file the plan has not written yet, so `docs/specs/*.tsv`
   deliverables, `ecosystem-PORTING.md`, and the never-issued `ADR-0013` the document *correctly* describes as
   absent all came back as defects. Those are now MISSING, reported and non-fatal, because the tool cannot
   tell a typo from a plan and must not pretend it can. A citation checker that cries wolf gets switched off,
   which is worse than not having one.

`npm run check:doc-citations` is pinned at `--max-unverifiable=566` and passes today. It fails on any BROKEN
citation and on any *growth* in the unverifiable count, so the number can only go down. Wave 1 is the rewrite:
each `:N` becomes a symbol or a quoted fragment at least as specific as the line number it replaces, and the
pin drops to match.

---

## Wave 1 — §0 corrections (`DRAFT:63`-`DRAFT:207`)

`UNVERIFIABLE 566 → 513` · `RESOLVES 2 → 54` · `BROKEN 0` · total 719 → 717

Citation form used throughout the sweep: `` `path` `fragment` `` — a path span immediately followed by a
symbol or a quoted fragment of the line, which is what `verify-doc-citations.mjs` can confirm by grep. Where
the prose already quoted the target text and a `(`path:N`)` followed it, the two are merged into one anchored
citation rather than left duplicated.

### Factually wrong, corrected to what the file says

- **`0153:79` was not the approver FK** (`DRAFT:117`). Line 79 of `0153_create_governance.sql` is `);`.
  `FOREIGN KEY (approver_id, org_id)` is line 78 and `FOREIGN KEY (requested_by, org_id)` is line 77 — so the
  parenthetical was wrong twice over: it named `:79` for the approver FK and `:78` for the `requested_by`
  twin, when `:78` *is* the approver FK. The previous pass had changed three correct `:78` cites to `:79` to
  agree with this one wrong one. Both now cite their own FOREIGN KEY text and the file decides.
- **`0102:19` was a comment, not the table** (`DRAFT:125`). Line 19 is
  `-- Canonical snake_case kind slug; mirrors the existing object_type CHECK`. `CREATE TABLE object_types` is
  line 18 and `kind TEXT PRIMARY KEY` — the text the sentence claims is at `:19` — is line 21.
- **`0152:33` was a comment, not the UNIQUE constraint** (`DRAFT:128`). Line 33 is
  `-- projected types must name their backing table + PK; instance types must not.`;
  `UNIQUE (org_id, stable_key, schema_version)` is line 32. Off by one.
- **`0152:18` named a column, not the table** (`DRAFT:127`). Line 18 is `id UUID PRIMARY KEY DEFAULT
  gen_random_uuid(),`; `CREATE TABLE ont_object_types` is line 17.
- **`:392` / `:425` / `:449` at `DRAFT:83` were bound to the wrong file.** They intend
  `cedar_pbac/engine.rs`, but the nearest preceding resolvable citation is
  `docs/ideas/authority-and-approval-model.md`, so a reader following the document's own convention lands on
  approval prose (`:392` there is *"D must be notified even though D never saw the matter"*). The bare `:N`
  form binds to whatever precedes it, not to what the sentence means.

### Prose about citations that the sweep made false, and had to move with it

- `DRAFT:84` said *"The `engine.rs` line numbers stay as line numbers: that file is unmodified source."* That
  was the justification for the two-form policy; with the line numbers gone it now reads: anchored by symbol
  rather than by line. **The document header still describes the old two-form policy and is corrected in the
  last wave, not this one** — it is one edit and belongs with the final count.
- `DRAFT:118` carried *"earlier drafts of this plan cited `:78` for both"*, a note that only means anything
  while the citations are line numbers. Replaced by the two distinct FOREIGN KEY fragments plus the surviving
  fact that two earlier passes conflated them.

No decision, section number, or claim about the system changed in this wave.

## Wave 2 — §0.12-§0.17 and §3.1 (`DRAFT:219`-`DRAFT:420`)

`UNVERIFIABLE 513 → 471` · `RESOLVES 54 → 93` · `BROKEN 0` · total 717 → 714

### Factually wrong, corrected to what the file says

- **`ontology/rest/src/lib.rs:201` and `:202` are `dispatch` internals, not the route consts** (`DRAFT:258`,
  `DRAFT:259`). `:201` is `target: Some(input.target),` and `:202` is `}),`.
  `pub const OBJECT_TYPE_LIFECYCLE_PATH` is line **210** and `pub const OBJECT_TYPE_POLICIES_PATH` is line
  **211** — nine lines below what was cited. The claim (both exist, both registered) is true; the anchors
  pointed at an unrelated match arm.
- **`ONTOLOGY_ROUTE_PATHS` is not at `:213-228`** (`DRAFT:221`, `DRAFT:259`). That span is the tail of the
  path consts; `pub const ONTOLOGY_ROUTE_PATHS: &[&str] = &[` is line **222** and the array closes at **237**.
  The **14** it claims is correct — lines 223-236 are exactly fourteen entries — so only the anchor was wrong,
  in a sentence whose whole point is that a stale document counted 12.
- **`ontology/adapter-postgres/src/lib.rs:416` and `:458` are not `validate_draft`** (`DRAFT:237`). `:416` is
  `})` and `:458` is `&self,`. `fn validate_draft` is line **1124**; its two call sites are **429** and
  **471**. Now cited as the definition plus `validate_draft(&draft)?`, which occurs exactly twice and so
  carries the "both sites" claim itself.
- **`:1142-1151` is the duplicate *property* key check, not link-type validation** (`DRAFT:238`). The sentence
  claims the entire link-type validation is duplicate-`stable_key`-only; that check is
  `duplicate link key {key:?} in object type draft` at **1155-1164**. The cited range is a different loop over
  a different field — the claim is true, the evidence pointed elsewhere.

### Judgement calls recorded rather than hidden

- `DRAFT:399` cited `console-program-ledger.md:327` **and** `:420` for "every jurisdiction binding and Korea
  control reads HOLD". `:327` states it specifically (`every control trace remains HOLD`). `:420` states it
  only through the closing sentence that **every rebind entry in that ledger repeats verbatim** — there is no
  fragment unique to `:420` that supports this claim, and the one thing unique to that line is iOS toolchain
  evidence. Collapsed to the `:327` anchor plus "restated in every rebind entry since", which is what the
  ledger actually shows. A per-entry line number here was precision the source does not have.
- `DRAFT:260` carried *"and not the `:213-217` an earlier draft of this plan cited"* — citation history that
  means nothing once no citation is a line number. Removed; the surviving fact (that document counted 12, the
  code has 14) is untouched.

## Wave 3 — §3.1 tiers through §4.0 component table (`DRAFT:423`-`DRAFT:581`)

`UNVERIFIABLE 471 → 439` · `RESOLVES 93 → 125` · `BROKEN 0` · total 714

### Factually wrong, corrected to what the file says

- **`0152:25` is `primary_key_property`, not `backing_kind`** (`DRAFT:436`). The sentence claims
  `ont_object_types.backing_kind = 'projected'`; `backing_kind TEXT NOT NULL CHECK (backing_kind IN
  ('projected','instance'))` is line **23**, and `:25` is `primary_key_property TEXT NULL, -- projected: PK
  column`. Two lines off, onto a different column of the same table.
- **`0152:23` is `backing_kind`, not `title_property_key`** (`DRAFT:551`) — the same off-by-one from the other
  side. `title_property_key TEXT NULL` is line **22**. One fact, two sections, two different wrong numbers.
- **`ontology/rest/src/lib.rs:194` and `:370` are not the object-type write routes** (`DRAFT:579`). `:194` is a
  bare `}` and `:370` is `#[derive(Debug, Deserialize)]`. The live routes are
  `get(list_object_types).post(create_object_type)` (**245**) and
  `get(get_object_type).put(stage_object_type_revision)` (**249**), with handlers at **350** and **392**. The
  claim that POST/PUT are live is true; nothing at the cited lines showed it.
- **`:201` / `:202` for the two route consts recur here too** (`DRAFT:580`, `DRAFT:581`) — same defect as
  `DRAFT:258`, fixed the same way.
- **`0102:54` is `object_links.id`, not the table** (`DRAFT:464`, `DRAFT:552`). `CREATE TABLE object_links` is
  line **53**.
- **`0076:22-24` spans the index tail and a blank line** (`DRAFT:514`). The partial unique index begins at
  **21** with `CREATE UNIQUE INDEX users_org_employee_unique_idx`; `:24` is empty. Same shape at
  `DRAFT:513`, where `:13-14` covers `ADD COLUMN employee_id UUID;` plus a blank line.

### Anchored deliberately at the table, not the column

`DRAFT:424` claims `ont_instances.org_id` is `NOT NULL`, cited `0155:18`, which is correct. It is now cited as
`0155` `CREATE TABLE ont_instances`, because the column line
`org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,` is **not unique** in that migration —
all three tables declare it, differing only in alignment padding. An anchor that resolves by whitespace is
worse than one that names the table containing the column. The claim is unchanged and one grep away.

## Wave 4 — §4.0.2 record contract through §4.1 constraints (`DRAFT:586`-`DRAFT:830`)

`UNVERIFIABLE 439 → 401` · `RESOLVES 125 → 166` · `BROKEN 0` · total 717

### Factually wrong, corrected to what the file says

- **`0051_platform_remove_organization.sql:34` is a comment, not the re-home** (`DRAFT:726`). Line 34 reads
  `--      rows to the platform sentinel org (...00face, an existing organizations`. The statement that
  actually re-homes a removed tenant's audit trail is `UPDATE audit_events` / `SET org_id = sentinel_org` at
  **195-199**, guarded by `PERFORM set_config('app.audit_rehome', 'on', true)` at **194**. The plan cited the
  prose *about* the mechanism where it meant the mechanism — the same class of error as citing an ADR Decision
  line for what a gate returns.

### Per-column citations kept per-column

`DRAFT:619`-`DRAFT:621` cited nine individual `audit_events` columns by line (`:13`, `:16`, `:17-18`, `:20`,
`:22-23`, `:25-26`, `:27`). All nine were **correct**. Each is now its own DDL fragment
(`actor UUID REFERENCES users(id)`, `trace_id CHAR(32)`, …) rather than being collapsed into one
`CREATE TABLE audit_events` anchor: collapsing would have made the reader scan eighteen lines to check a
one-column claim, which is the vagueness this sweep is supposed to remove, not introduce.

## Wave 5 — §4.1 party/employment through §4.2 group plane (`DRAFT:833`-`DRAFT:1032`)

`UNVERIFIABLE 401 → 367` · `RESOLVES 166 → 201` · `BROKEN 0` · total 718

### Factually wrong, corrected to what the file says

- **`0153:79` for the approver FK, a third time** (`DRAFT:946`). Same defect as `DRAFT:117`: line 79 is `);`,
  and the FK is `FOREIGN KEY (approver_id, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT` at **78**.
  Three sections had inherited one wrong number.
- **`0187:29` is `status`, not `position_ref`** (`DRAFT:955`). `position_ref TEXT, -- optional ontology
  position instance ref` is line **28**; `:29` is
  `status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT','PUBLISHED','CLOSED'))`. Off by one, onto a
  column that has nothing to do with the cross-store-pointer claim.

The five DB-enforced four-eyes CHECKs (`DRAFT:886`-`DRAFT:888`) were all cited correctly and now each carries
its own CHECK expression, so the count claim "five" is checkable one grep at a time instead of resting on five
line numbers in four migrations.

## Wave 6 — §4.3 relationships and §4.4 substrate table (`DRAFT:1034`-`DRAFT:1208`)

`UNVERIFIABLE 367 → 322` · `RESOLVES 201 → 245` · `BROKEN 0` · total 717

### The origin of the `:79` propagation, found and corrected

`DRAFT:1167` is where it started. The cell read:

> `FOREIGN KEY (approver_id, org_id) REFERENCES users(id, org_id)` **`0153:79`** — … (`:78` is the
> `requested_by` FK; three sites in earlier drafts cited `:78` for the approver, and §4.1 already cited `:79`
> correctly — one fact, two line numbers, now one.)

Every part of that parenthetical is false. In `0153_create_governance.sql`, **`:77` is
`FOREIGN KEY (requested_by, org_id)`, `:78` is `FOREIGN KEY (approver_id, org_id)`, and `:79` is `);`.** The
three sites that said `:78` for the approver were **right**, and this cell is the reason a later pass
"corrected" them to `:79`. A note claiming to have reconciled one fact to one number had reconciled it to the
wrong one — which is exactly why the sweep replaces the number with the FK text and lets the file decide.

The cell now carries both FOREIGN KEY expressions verbatim, and the citation-history note is reduced to the one
thing that is true: two earlier passes read the approver FK one line low and then rewrote the sites that had
it right.

### A tooling near-miss worth recording, because the gate caught it

The batch rewriter used `String.prototype.replace(find, repl)`. In the replacement string, `$'` means
"everything after the match", so the DDL fragment
`CHECK (link_type ~ '^[a-z][a-z0-9_]{1,63}$')` silently rewrote itself into the rest of the line at
`DRAFT:1132`. `verify-doc-citations.mjs` reported it as **BROKEN** on the next run — the only BROKEN this sweep
has produced — and it was fixed by passing a function replacer. A citation checker that only counted line
numbers would have shipped that corruption.

## Wave 7 — §4.4 notices through §5.1 authority mechanics (`DRAFT:1215`-`DRAFT:1550`)

`UNVERIFIABLE 322 → 273` · `RESOLVES 245 → 292` · `BROKEN 0` · total 715

### Factually wrong, corrected to what the file says

- **`0152:99` is `control_points`, not `dispatch`** (`DRAFT:1357`). The sentence cites the
  `ont_action_types.dispatch` CHECK; that is
  `dispatch TEXT NOT NULL CHECK (dispatch IN ('projected_usecase','instance_revision'))` at **97**. Two lines
  low, onto the JSONB column below `dispatch_target`.
- **`0152:23` for `title_property_key` recurs** (`DRAFT:1494`) — third appearance of the same off-by-one, same
  fix as `DRAFT:551`.
- **`0153:79` for the approver FK, a fourth time** (`DRAFT:1215`).
- **`engine.rs:370-391`, `:403-424`, `:392`/`:425`, `:449` were bound to nothing checkable**
  (`DRAFT:1382`, `DRAFT:1383`, `DRAFT:1398`, `DRAFT:1399`). `engine.rs` alone is ambiguous —
  `platform/authz/src/cedar_pbac/engine.rs` and `workflow/runtime/src/engine.rs` both match — so the verifier
  could not bind the bare continuations at all, and a reader following them had two candidate files. Now
  `cedar_pbac/engine.rs` plus the actual expressions (`let subject_attrs = HashMap::from([`,
  `let mut resource_attrs = HashMap::from([`, the two `Entity::new(… HashSet::new())` calls).
- **`instances.rs:1166` is the `return Err(` line, not the guard** (`DRAFT:1414`). The rejection is
  `if op != "sum"` at **1165**; the message the sentence quotes begins at **1167**.

## Wave 8 — §5.1 re-validation through §5.5 voucher inventory (`DRAFT:1551`-`DRAFT:1746`)

`UNVERIFIABLE 273 → 238` · `RESOLVES 292 → 327` · `BROKEN 0` · total 715 · **SUSPECT 2 → 1**

One of the two baseline SUSPECT citations is resolved. `DRAFT:1696` cited `ledger :174` where the antecedent
lived only in the English word "ledger", so the verifier bound it to `ADR-0023` (166 lines) and flagged the
mismatch. The citation was **correct** — `console-program-ledger.md:174` is the deferred-epics line — and the
binding was the defect. It now names the file and the `Epics (documented, later):` heading, so nothing has to
be inferred.

Nothing else in this range was factually wrong. Five accepted-ADR decision citations (`ADR-0021` decisions 1,
4, 5, 6, 8) now quote the decision sentence instead of numbering its lines, which also removes the failure mode
this document warned about in its own discipline note: an ADR Decision line is prose about code, and a citation
that can only be checked by counting lines invites re-numbering to make two prose passages agree.

## Wave 9 — §5.5 GL inventory through §5.6 propagation (`DRAFT:1747`-`DRAFT:1853`)

`UNVERIFIABLE 238 → 198` · `RESOLVES 327 → 363` · `BROKEN 0` · total 710

### Bare `:N` bound to the wrong file by the table it sits in

- `DRAFT:1748` — `Only posted_at TIMESTAMPTZ (:41), created_at (:42) | 0160:41-43 |`. The bare `:41`/`:42`
  inherit the row above, which cited `0163`, so following them lands on
  `IF OLD.approved_by IS NOT NULL AND NEW.approved_by IS DISTINCT FROM OLD.approved_by` — the approver-
  immutability trigger, nothing to do with business dates. The trailing `0160:41-43` in the same cell was the
  correct target. One cell, two files, three numbers; now one anchored citation to
  `posted_at TIMESTAMPTZ,`.
- `DRAFT:1853` — *"a 4th const beside `0012`-era `:37-39`"*. The three channel consts are in
  `realtime/src/lib.rs`, not in migration `0012`; the bare `:37-39` bound to `0012`, where it lands on
  `CREATE INDEX idx_messenger_thread_members_user`. "0012-era" was a note about *when*, which the citation
  grammar read as *where*.
- `DRAFT:1777` cited `0015:45` for "gate-marked audited". Line 45 is `CREATE TABLE equipment_cost_ledger (`;
  the marker `-- console-gate: audited-table equipment_cost_ledger` is **44**. Both cells of that row pointed
  at the same line while claiming two different things.

### No stable anchor, cited by shared text with the ambiguity stated

`DRAFT:1834` cited `0015:16-18` **and** `:88-90` for the depreciation-method CHECK. Both spans are
**byte-identical** (`depreciation_method TEXT NOT NULL CHECK (depreciation_method IN ('STRAIGHT_LINE',
'DECLINING_BALANCE'))`) — one on the equipment config table, one on the purchase-request table — so no
fragment can distinguish them. Cited once with the two tables named in prose instead of pretending a fragment
picks one out.

## Wave 10 — §5.6 through §5.11 governance (`DRAFT:1854`-`DRAFT:2233`)

`UNVERIFIABLE 198 → 124` · `RESOLVES 363 → 436` · `BROKEN 0` · total 709

### Factually wrong, corrected to what the file says

- **`0130:43` is `('belongs_to', …)`, not `derived_from`** (`DRAFT:2010`). The seed row
  `('derived_from',  'Source was produced from the destination (lineage)'),` is line **44**. One line off, in
  the sentence whose whole point is which of the twelve seeded labels exist.
- **`0156:107` is the `inventory_stock_locations` FK, not the `work_orders` FK** (`DRAFT:1930`,
  `DRAFT:2019`). `FOREIGN KEY (work_order_id, org_id) REFERENCES work_orders(id, org_id)` is line **108**.
  Both sections claim "a hard FK to `work_orders`" and both pointed one line above it.
- **The `ADR-0022` "Verified:" note was itself off by one, three times** (`DRAFT:2172`). It said `:25` is
  Context prose, `## Context` is at `:23`, and the `## Decision` block is `:31-39`. In the file, `## Context`
  is **24**, `:25` is **blank**, the Context prose starts at **26**, `## Decision` is **32** and its clauses run
  **34-40**. The substantive finding — that the lines G1 was grounded on are Context, not Decision, and that
  "org-scoped" appears nowhere in ADR-0022 — is **unchanged and still true**; the audit of the citation was
  performed with the same instrument that produced the error.
- **`:87` at `DRAFT:2024` and `:23`/`:24` at `DRAFT:2041` were bound to `0072`**, the migration cited two
  sentences earlier, while the sentences are about `0156` and `0147` respectively.

### Four citations the checker could not see at all

`DRAFT:2039`-`DRAFT:2042` wrapped two quoted DDL fragments across line breaks, leaving an **unbalanced code
span**. Markdown code spans do not cross newlines, so the scanner paired the stray backticks with the wrong
partners and `0147:20`, `:21`, `:22` and `:25` were never counted as citations in the first place — they were
invisible to the audit while looking perfectly cited to a human. (All four were correct.) That is also why
`:23`/`:24` on the next line inherited `0072`: the `0147` that should have been their antecedent had been
swallowed by the broken span. A repo-wide scan of the document found **16 lines** with unbalanced spans, eight
pairs; only this one hid citations, and it is now one fragment per line.

## Wave 11 — §6 probes through §8 lane protocol (`DRAFT:2240`-`DRAFT:2709`)

`UNVERIFIABLE 124 → 49` · `RESOLVES 436 → 498` · `BROKEN 0` · total 696

### Factually wrong, corrected to what the file says

- **`0160:79` is `RETURNS TRIGGER AS $$`, not the trigger body** (`DRAFT:2443`). The known-bad control says
  "the trigger must fire"; what fires is
  `RAISE EXCEPTION 'finance_gl voucher % is posted and immutable'` at **86**, inside the function that starts
  at **78**. Now anchored on the exception text, which is what a failing test would actually observe.
- **`0060:88-91` is the `RETURN QUERY SELECT`, not the `EXCEPTION` restore** (`DRAFT:2405`).
  `EXCEPTION WHEN OTHERS THEN` / `SET LOCAL row_security = on;` is **90-92**. The cited span starts two lines
  early and stops one line short of the thing named.

### Opaque targets that were never citations at all

`preflight:75`, `support-domain-unit:163`, `postgres-domain-reachability:194`, `company-conformance:244`,
`generated-face-authority:291`, `backend:340`, `dev-up-smoke:684`, `repo-gates:741`, `api-contract:827`,
`kubernetes-manifests:906` (`DRAFT:2552`) and `realtime:40` (`DRAFT:2453`) name a **job or a crate**, not a
file, so no tool could bind them and only a human who already knew the repo could follow them. Each is now
`ci.yml` plus the job key itself — which is what the line number was standing in for. The document's own
instruction two sentences later, *"Cite the job by name"*, is now what the citations do.

## Wave 12 — §8 tail through §10, plus the header and the pin (`DRAFT:2684`-`DRAFT:2952`)

`UNVERIFIABLE 49 → 0` · `RESOLVES 498 → 548` · `BROKEN 0` · total 695 · **SUSPECT 0**

```
$ npm run check:doc-citations
  total citations : 695
  RESOLVES        : 548
  UNVERIFIABLE    : 0     (max allowed 0)
  BROKEN          : 0
  FILE-ONLY       : 126
  MISSING         : 21    (non-fatal)
EXIT=0
```

### Factually wrong, corrected to what the file says

- **`ontology/rest/src/lib.rs:201-202` for the two route consts, a fifth time** (`DRAFT:2944`) — the same
  defect as `DRAFT:258`, `:259`, `:580`, `:581`. They are **210** and **211**.
- **`0156:107` for the `work_orders` FK, a third time** (`DRAFT:2950`) — it is **108**.
- **`ontology/rest/src/lib.rs:1786-1790` is `fn digest_hex`** (`DRAFT:2720`), cited as the evidence that
  `command_pool()` is `None` unless `ONTOLOGY_COMMAND_DATABASE_URL` is set. It is a hex-formatting helper in the
  wrong crate. The real evidence is `ontology/adapter-postgres/src/lib.rs` `fn command_pool` (**364**, with
  `command_pool: None` at **339**); the companion cite `backend/app/src/lib.rs:2925-2930` was correct.
- **The second baseline SUSPECT is resolved** (`DRAFT:2684`). `:89` and `:268-269` both intended
  `docs/program/LANE-PROTOCOL.md` — the file the sentence is *about* — but the nearest preceding resolvable
  citation was `console-program-ledger.md`, so `:89` landed on a `moduleScreens.ts` row and `:268-269` past the
  end of migration `0204`. Both numbers were right about LANE-PROTOCOL and bound to the wrong file, which is
  precisely the failure the bare `:N` form guarantees.

### One line-number citation the checker could not see, inside a fenced block

`DRAFT:2624` carried `` `.github/workflows/ci.yml:239` `` **inside a fenced code block**, which the verifier
skips by design (fences hold sample code, not citations). It was correct, and it sat one line above the
document's own sentence *"Target names rather than line numbers, because the line numbers in that chain have
already drifted once."* It now names the workflow step, per that rule.

### The header and the pin

- The header described **two citation forms**, one of them `path:line` "into unmodified source — re-verified".
  That justification is what the sweep disproves: 566 of 568 content-bearing citations were line numbers, and
  every defect two consecutive passes introduced was an off-by-one in one of them. It now describes one form
  and points at the gate.
- `npm run check:doc-citations` drops `--max-unverifiable=566`. The default is **0**, so any line-number
  citation added to this document from now on fails CI, and the control fixture still exits 1 on its four
  planted BROKEN citations.

### No stable anchor found

One case, recorded rather than papered over: `DRAFT:1834` cites the depreciation-method CHECK, which exists
**twice, byte-identically**, in `0015_create_financial.sql` (equipment config and purchase request). No fragment
can distinguish the two occurrences, so the citation names the text once and the two tables in prose. Every
other citation in the document resolves to something a reader can grep and land on.
