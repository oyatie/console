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
