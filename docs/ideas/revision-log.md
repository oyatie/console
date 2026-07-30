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
