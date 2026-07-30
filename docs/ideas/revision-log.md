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
