> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# ADR adjudication — 12 themes over 75 collisions

Status: PENDING APPROVAL — adversarially adjudicated by theme, not yet accepted

Date: 2026-07-29
Adjudicates: `docs/ideas/adr-collisions-raw.md` (75 collisions), grouped into 12 themes
Plan under test: `docs/ideas/ecosystem-plan-DRAFT.md` (PENDING APPROVAL)
Prior verdict: `docs/ideas/ecosystem-plan-architect-findings.md` (SOUND_WITH_FIXES, 12 blocking findings)
Governance floor: `docs/decisions/README.md`

This record decides the SHAPE of governance action per theme. It does not approve work.
All 27 capabilities remain HOLD; all six Korea controls remain `release_disposition: HOLD`.
No verdict here asserts a Korea compliance conclusion or proposes unholding anything.

---

## 0. Three findings that outrank every individual verdict

**0.1 — Four themes each claimed `ADR-0027`, and only one can have it.** T1 (identity linkage),
T2 (capability-derived branch scope), T7 (audit-coverage cardinality) and T12 (console
reconciliation) each independently computed "next free number after ADR-0026" and each named
ADR-0027. `docs/decisions/` confirms ADR-0026 is the highest issued and ADR-0013 must never be
reused (README:13). Every draft below therefore carries `ADR-XXXX-DRAFT`; **the integrator assigns
numbers in one atomic commit** with the `docs/decisions/README.md` index rows (README:3).

**0.2 — T2, T5 and T11 amend the SAME clause, and shipping them as separate records is a
governance collision the CI checker cannot see.** All three land on `ADR-0003:20` — verified
verbatim: "Principals carry a `BranchScope` (kernel type): `All` for SUPER_ADMIN/EXECUTIVE
rollups, an explicit branch set otherwise." T2 amends it to a capability-or-membership derivation;
T5 amends it to enumerate both shipped derivations; T11 requires the realtime fan-out path named in
its scope. `scripts/check-adrs.mjs:399-406` validates only that `amends`/`amended_by` is
reciprocal — two accepted ADRs both declaring `amends: [ADR-0003]` and editing line 20
incompatibly would pass CI and leave the authoritative record self-contradictory.
**They are merged into one draft (D2).** This is the single most consequential structural finding
in the adjudication.

**0.3 — The task brief's own STATE FACT is false, and two verdicts repeat the false version.**
The brief asserts "the `audit-coverage` gate's exclusion set has exactly ONE entry and a test
asserts it." Verified by direct read: `backend/ci/gates/audit-coverage/src/lib.rs:90-107` returns
**two** — `location_ping_ingestion` and `location_data_retention_purge` — and the test is named
`allowed_exclusion_set_is_the_two_location_carveouts` with `assert_eq!(exclusions.len(), 2)`
(`backend/ci/gates/audit-coverage/tests/gate_detects_violation.rs:26-28`). T3 and T7 corrected it;
**T6's and T8's follow-up text still says "exactly one entry"** and must not be copied forward.
`ADR-0002:20` says it too, verbatim — which is why D3 exists.

---

## 1. Summary table

| Theme | Patterns | Decision | ADR action | Confidence |
|---|---|---|---|---|
| T1 identity | P1 | DEFER_WITH_CONSTRAINTS | **D1** amends ADR-0022 (narrowing). G1 **withdrawn** | medium |
| T2 feature/role | P7 | THIRD_WAY | **D2** amends ADR-0003 (merged). ADR-0021 no action | high |
| T3 the fold | P6, P16 | THIRD_WAY | **N1** new, `related` only. ADR-0021 **not** amended | high |
| T4 approvals | P8, P9 | THIRD_WAY | **N3** = G3 re-scoped as ADR-0023 delta, `related` only | high |
| T5 organization | P3, P4, P5, P2 | THIRD_WAY | folded into **D2** (was a second ADR-0003 pair) | high |
| T6 open sets | P14, P7 | THIRD_WAY | **NONE**. G7 struck; ADR-0001 pair deleted | high |
| T7 work artifacts | P10, P11 | AMEND_INCUMBENT | **D3** amends ADR-0002 (retroactive) | high |
| T8 quantity | P12 | DEFER_WITH_CONSTRAINTS | **N4** = G4, `related` only | high |
| T9 economics | P13 | THIRD_WAY | **N5** = G5, `related` only | medium |
| T10 dry-run | P15, P6 | THIRD_WAY | **NONE**. G6 **struck**. **N2** optional | high |
| T11 realtime | P16 | THIRD_WAY | **NONE** — scope folded into **D2** | high |
| T12 gates | all | THIRD_WAY | **D4** amends ADR-0025. Owner decision required | high |

Nine records: four amendment pairs (D1-D4), five new non-amending records (N1-N5).
Two G-slots struck (G6, G7). One G-slot withdrawn (G1). One reclassified from prepwork to
blocking (G9 → D3). One G-slot needs no record at all (G8 — argue, not amend).

---

## 2. Draft ADRs for every AMEND

House structure matched against `ADR-0026-retire-coss-rn-public-site-surface.md` (newest) and
`ADR-0022-local-identity-no-external-idp.md`. Frontmatter keys, section order and tone are copied,
not invented. Required keys per README:15-26.

---

### D1 — `ADR-XXXX-DRAFT`: Identity linkage is human-asserted; no platform identity row in Slice 0

**Reciprocal record owed:** `ADR-0022-local-identity-no-external-idp.md` frontmatter gains
`amended_by: [ADR-XXXX]` (it currently has `amends: [ADR-0010]`, `supersedes: [ADR-0017]`,
`related: [ADR-0004, ADR-0010, ADR-0017]` — verified) and gains ADR-XXXX in `related`. Its
Decision list gains one bullet after `:38`. Index row for ADR-0022 changes from `accepted` to
`accepted, amended`. Without the `amended_by` key `scripts/check-adrs.mjs:399-406` fails the build.

```yaml
---
id: ADR-XXXX
status: accepted
doc_status: published
date: 2026-07-29
owner: jasonlee
decision: identity-linkage-is-human-asserted
amends: [ADR-0022]
related: [ADR-0004, ADR-0018, ADR-0022, ADR-0025]
---
```

# ADR-XXXX: Identity linkage is human-asserted; no platform identity row in Slice 0

## Status

Accepted. Narrowly amends ADR-0022's local-account-administration scope (`ADR-0022:36`) and adds a
prohibition its integration bullet (`:38`) does not cover. This is a **narrowing** amendment: it
grants no new capability, which is why it is safe to accept while the durable identity row stays
deferred.

## Context

Three things ADR-0022 does **not** decide, stated here so the next reader does not repeat the
error the ecosystem plan made:

1. **ADR-0022 never decides that identity is org-scoped.** `ecosystem-plan-DRAFT.md:1334` grounds
   its G1 on "`ADR-0022:25,33-39` decides identity is local/org-scoped." `:25` is Context, not
   Decision, and reads "creates and manages its own identities through local passkey-backed
   accounts". The words "org-scoped" appear nowhere in ADR-0022.
2. **ADR-0004 contains no tenancy clause.** Its Decision (`ADR-0004:20`) is webauthn-rs ceremonies,
   ES256/EdDSA access JWTs, opaque hashed refresh tokens, OTP bootstrap, AASA/assetlinks. The
   `(user_id, org_id)` credential pinning is a migration-level mechanism
   (`0034_enforce_org_id_rollout.sql:143-144`), not an accepted decision.
3. `docs/specs/korean-legal-boundaries.md:40-43` and `docs/specs/org-editor-primitives-ux.md:256`
   are adopted **by** this ADR as design content and carry no amending authority of their own
   (README:8).

One physical passkey already serves two employers: `0004_create_auth.sql:7` is UNIQUE per
*credential*, `auth/src/webauthn.rs:349-353` passes the per-org `users.id` as the WebAuthn user
handle, `:339-342` builds `exclude_credentials` from that user's own passkeys only, and login is
usernameless/discoverable with an empty allowCredentials list (`:786-796`) resolved by a
deterministic `LIMIT 1` over a UNIQUE column (`0038:79-80`). The account chooser the product thesis
asks for is shipped. What a platform-tier identity adds beyond that is group-level "same person"
resolution — read through a group-scope grant definer that has no `org_id` predicate
(`ecosystem-plan-architect-findings.md:28-31`, `:70`).

A DB-enforced ban on name-inferred identity merge already ships and nothing holds it:
`0075_employee_identity_resolution.sql:16-17` is
`identity_name_only_merge BOOLEAN NOT NULL DEFAULT FALSE CHECK (identity_name_only_merge = FALSE)`
(verified verbatim), yet `employees` is absent from `built_in_audited_tables()`
(`backend/ci/gates/migration-safety/src/lib.rs:164-172`) and no `-- console-gate: audited-table
employees` marker exists anywhere in the migration set (verified: zero matches), so a future
migration can drop that column with no gate violation.

## Decision

1. **Identity linkage is permitted local account administration within `ADR-0022:36`** and may be
   established ONLY by a user-verified WebAuthn assertion (`auth/src/webauthn.rs:480-484`,
   `result.user_verified() == true`) of a credential the person already holds, whose owner is
   re-checked against the asserted user handle (`:455-466`), resolved through a narrow SECURITY
   DEFINER in the exact shape of `platform_resolve_credential_org` (`0038:64-86` — `SET
   search_path`, `row_security` off→on, `REVOKE ALL … FROM PUBLIC`, EXECUTE to `console_rt` only)
   that returns the linkage handle and nothing else.
2. **No roster, import, attendance, payroll, matching or confidence-scored path may create, match
   or merge an identity link.** `0075:16-17` is the database expression of this rule and may not be
   dropped or relaxed.
3. **No platform-tier identity row and no `party` table is created in Slice 0.** The durable handle
   is deferred until it has a consumer that is not itself HOLD. `users.party_id` /
   `employees.party_id` are not added yet: `users` is a built-in audited table
   (`migration-safety/src/lib.rs:164-172`) and `ALTER TABLE users DROP COLUMN` is a gate violation
   (`:300-309`), so a column added today is permanent, while adding it later is purely additive and
   blocked by nothing (`0060:43` already proves a cross-tenant reference to `users(id)` needs no key
   surgery).
4. Until then: (a) no cross-tenant identifier may be declared a FOREIGN KEY, nor appear in any
   UNIQUE constraint or index whose key does not lead with `org_id`, because both are enforced
   physically below RLS (experiment X4 CONTROL 3, measured); (b) the authorization path may not read
   `employees` — true today (`backend/crates/platform/authz/src/lib.rs:239` is a doc comment, not a
   query) and to be frozen by CI assertion.
5. **When the handle lands it is an ordinary tenant-scoped row homed at the existing platform
   sentinel organization** `00000000-0000-0000-0000-00000000face` (`0036_platform_onboarding.sql:222-227`,
   verified: the row is inserted, rationale at `:217-221`; `0196:171-173` refuses it as a removal
   target), carrying `org_id NOT NULL` + ENABLE/FORCE ROW LEVEL SECURITY + the standard
   `org_isolation` policy on `app.current_org`. **Not** a Tier O carve-out: no
   `global_table_allowlist` or `owner_only_table_allowlist` entry, no new GUC, no definer-mediated
   read. Every party-level mutation is written with `with_audits(OrgId::platform())`
   (`backend/crates/platform/db/src/audit_tx.rs:111,121`) so it is stamped with a real `org_id` and
   readable back under the unchanged audit policy (`0035:107-112`), never with the `org_id = NULL`
   form that policy's `USING` clause cannot return.
6. **Cross-group deduplication is not guaranteed.** A person imported into `employees` at a second
   employer who never enrolls a credential is never auto-linked. This is a deliberate scope decision
   for a vendor-operated group-company platform, not an oversight.

## Consequences

- Cross-legal-entity identity continuity stays unrepresentable. `korean-legal-boundaries.md:40-43`
  remains specified-and-unbuilt, so 전적/전출 between 계열사 and concurrent 겸직 cannot be recorded
  as one person, and group-level headcount or authority rollups spanning subsidiaries are not
  expressible. `0031_runtime_role_and_immutable_org.sql:94,108ff` means the sequential-transfer case
  additionally has no UPDATE path; the only sanctioned re-home is the DELETE+re-INSERT the platform
  bootstrap performs at `0036:204-210`.
- Matching-based deduplication is foreclosed permanently. It stays an operator action.
- The party is foreclosed as an authentication subject: credentials stay pinned to
  `(user_id, org_id)` (`0034:143-144`), so switching companies remains a credential choice at the OS
  prompt rather than an in-app org switcher after one sign-in. Revisit only when the across-group
  담당자 case is a counted requirement; note it would require login to gain an org-selection step
  *before* the GUC is armed — surgery on the one path that must never break.
- Zero migration slots consumed, so 0207+ stays free for other lanes. Zero new GUCs, zero changes to
  the 141 policies, zero new gate classifications, no SECURITY DEFINER added. ADR-0003, ADR-0004,
  ADR-0001, ADR-0002, ADR-0018, ADR-0021 and DN-0003 take zero delta.
- Residual risk accepted: the sentinel-home shape is **unmeasured**. Experiment X4 built Variant A
  (party granted to `console_rt`, no RLS) and Variant B (no grant + definer) and never this. That is
  why confidence is medium (experiment X4b, §7).
- Whether an orphan pseudonymous identifier of a natural person is itself regulated data is a
  question this ADR does not answer and cannot answer: six Korea controls read
  `release_disposition: HOLD` and `console-jurisdiction-register.json:1186` forbids inventing
  certainty. Deferring the durable row is the disposition compatible with every one of them staying
  HOLD.

## Alternatives considered

### Create the `party` table and `users.party_id` in Slice 0
Rejected on irreversibility, not on merit. `users` is audited, so the column is permanent from the
day it lands, while adding it later is additive and blocked by nothing. Its only distinctive Slice-0
consumer is a group-scope grant read the architect CONFIRMED has no `org_id` predicate; building the
handle first ships the column that makes the leak reachable.

### Widen ADR-0022 to authorize a platform identity tier
Rejected. ADR-0022 needs narrowing, not widening — `:38`'s four forbidden verbs (authenticate,
assert sessions, grant roles, decide account status) do not include *linking two accounts to one
identity*, which is exactly how imported roster data would later confer group authority.

### Draft §5.11 G1 as scoped
Rejected. Its premise is false (see Context), so it has no clause to amend. Its claim — "one durable
identity per natural or legal person, across every tenant and vertical" (`plan:510`) — is
undeliverable without the PII matching the plan itself rejects, and README:7 makes an accepted ADR
authoritative in scope, so writing an unachievable guarantee into one is a governance liability.

---

### D2 — `ADR-XXXX-DRAFT`: `org_id` × `BranchScope` composition and capability-derived all-branch scope

**Merges T2's Record 1, T5's Records 1+2, and T11's fan-out scope. Supersedes plan slots G2 and
G2b.** See §0.2 for why these cannot be separate records.

**Reciprocal record owed:** `ADR-0003-branchscoped-authorization-model-nonnull-branch-scope.md`
frontmatter gains `amended_by: [ADR-XXXX]` — verified it currently carries `related: []` and no
`amended_by`, so this **creates** the key — and gains ADR-XXXX in `related`. Its Decision at line 20
is edited in place. `ADR-0021` and `ADR-0018` each gain ADR-XXXX in their own `related` list.
Index row for ADR-0003 changes to `accepted, amended`.

```yaml
---
id: ADR-XXXX
status: accepted
doc_status: published
date: 2026-07-29
owner: jasonlee
decision: org-id-branchscope-composition
amends: [ADR-0003]
related: [ADR-0003, ADR-0018, ADR-0021]
---
```

# ADR-XXXX: `org_id` × `BranchScope` composition and capability-derived all-branch scope

## Status

Accepted. Amends `ADR-0003:20` and closes a documented composition gap under `docs/decisions/README.md:12`.
This ADR **ratifies live behaviour**; it does not authorize new behaviour.

## Context

No accepted ADR composes tenancy with branch scope. `grep -rn BranchScope docs/decisions/` returns
exactly one line (ADR-0003:20) and `grep -rn org_id docs/decisions/` exactly two (ADR-0018:204,
ADR-0021:50). Verified independently.

`ADR-0003:20` reads, verbatim: "Principals carry a `BranchScope` (kernel type): `All` for
SUPER_ADMIN/EXECUTIVE rollups, an explicit branch set otherwise." **That sentence is already false
in shipped code, in two independent ways, and the divergence predates the ecosystem plan.**

1. `ecosystem-plan-DRAFT.md:233-243` (§0.16) asserts the `Role::SuperAdmin | Role::Executive` match
   at `backend/crates/platform/authz/src/lib.rs:1478-1482` "is the sole tenant-side derivation of
   `BranchScope::All`". **False.** `backend/crates/platform/request-context/src/lib.rs:352-364`
   early-returns when `claims.tenant_context == Some(TenantAccessContext::GroupAdmin)`, never
   calling `resolve_branch_scope_in_org`, and `resolve_group_admin_tenant_context_principal` passes
   `BranchScope::All` directly (verified: `effective_branch_scope_for_tenant(BranchScope::All,
   access_scope, org_id)` at `:421-422`) for a principal whose roles are asserted to equal exactly
   `{Role::Admin}` (`:392-393`) after live ACTIVE group membership is re-proved (`:405-411`). With
   `AccessScopeLevel::Org` matching the armed org, `backend/crates/kernel/core/src/access_scope.rs:88`
   returns `All`, so the intersect is `All`.
2. The realtime fan-out path holds no tenant predicate at all.
   `backend/crates/platform/realtime/src/lib.rs:843`, `:885` and `:899` never compare `org_id`, and
   `backend/crates/kernel/core/src/branch.rs:26-31` makes `BranchScope::All` return `true` for every
   `BranchId`. The tenant line there is held only by a messenger-specific RLS-armed membership read
   (`:890`, `:1096`) that an authority event has no thread to join against. `PgRealtimeHub::for_tests`
   is `pub` (`:486-488`) and with `pool: None` that check is skipped entirely (`:878`), so the
   current boundary is compiled out in exactly the configuration the unit tests run in.

Consequently `plan:1335`'s G2b, which scopes the ADR-0003 amendment to a FUTURE condition ("before
`Role` dies, replace the role match with a built-in `Feature`"), both understates the trigger — the
divergence exists today, independent of any plan step — and under-scopes the remediation, since
patching only the Role match leaves `request-context:421` minting `All` unchanged.

## Decision

1. **`BranchScope::All` is derived at exactly one point in tenant request context**, from either
   (i) a built-in `Feature` capability authored in code and not mintable from the console, or
   (ii) a live membership proof resolved against the database at request time. **Never from a
   role-name literal.**
2. This ADR **ratifies the already-shipped group-admin path** as instance (ii): a principal whose
   roles are exactly `{ADMIN}` receives `BranchScope::All` for one subsidiary after
   `group_admin_member_orgs` proves ACTIVE membership
   (`backend/crates/platform/request-context/src/lib.rs:383-434`), recorded as pre-existing
   divergence from ADR-0003:20 now brought under governance per README:12.
3. **`effective_branch_scope_for_tenant` (`backend/crates/platform/authz/src/lib.rs:1519-1541`) is
   the sole legal composition.** `AccessScopeLevel::Group` is refused on every ordinary tenant route
   and must use a group fan-out resolver (`:1525-1528`). `Region`/`Worksite` project to
   `BranchScope::none()` until a DB-backed hierarchy resolver supplies a matching `BranchProjection`
   (`:1536-1538`, `kernel/core/src/access_scope.rs:92-98`). The claim scope may only NARROW the live
   DB membership scope, never widen it (`:1541`). `AccessScopeLevel` is extensible by migration with
   the existing fail-closed default arm.
4. **A `BranchScope` predicate is never a tenant predicate.** Every fan-out, filter or projection
   path asserts `org_id` explicitly and may not rely on a co-located RLS-armed read for tenant
   isolation. This binds `realtime/src/lib.rs:843`, `:885`, `:899` by name.
5. **Custom role definitions cannot widen `All`** (`docs/specs/rbac-configurable.md:366` unchanged).
   Cross-tenant facts live ONLY in owner-only tables reached through SECURITY DEFINER resolvers,
   never in a tenant-writable row. This ADR does not widen `ADR-0021:50` — `org_id` remains the RLS
   boundary Cedar may not widen — and introduces no second GUC and no second isolation axis.
6. **Competence is a condition attribute on a custom role**, not a third relation and not a scope-type
   change. It takes the shape the `"team"` arm already has (`authz/src/lib.rs:1421-1425`): a
   subject-side predicate gating whether the role applies, leaving `BranchScope` untouched.
7. **The authored condition vocabulary is narrowed at the write path to what the runtime resolver
   evaluates** — attributes `{branch, team}`, operators `{equals, in}`
   (`authz/src/lib.rs:1406-1408`, `:1426`) — with a test asserting write-accepted ⊆
   resolver-evaluated. The fail-closed whole-role void at `authz/src/lib.rs:1352-1361` is CORRECT
   and must not be relaxed into per-condition ignoring; the contrary comment at
   `0065_create_policy_roles.sql:101-103` is struck. The DB CHECK (`0065:110-128`, verified: exactly
   17 attribute literals, 3 operators) stays permissive as the additive extension point.
8. **`feature_catalog` and `Feature::ALL` are one vocabulary in BOTH directions**, enforced by a new
   gate. `orgchange`'s `role_floor` (`backend/crates/orgchange/rest/src/lib.rs:396-436`) is replaced
   by `authorize(...)`.
9. **`Role` and `matrix_row` are a label and a default, not a decision.** They are deletable only
   after the six system roles are seeded as `is_system` data rows with a golden parity proof against
   `matrix_row()`, and only when every ADR-0021 coexistence-map entry reads `cedar_only`. Their
   deletion is NOT a prerequisite for any canvas capability and must not gate any plan.
10. Relaxing `authz/src/lib.rs:1388` to let a tenant-authored role carry
    `RoleManage`/`ElevatedRoleGrant` remains **forbidden** until the no-lockout holder floor
    (`rbac-configurable.md:260-262`) is implemented as a hard rejection under a per-org advisory
    lock. No application route can repair a locked-out org: `platform-rest` contains no writer of
    tenant `policy_roles` or `users.roles`; the sole writer is the tenant-scoped path at
    `backend/crates/identity/adapter-postgres/src/lib.rs:291`.

`ADR-0003:20` is amended to replace "`All` for SUPER_ADMIN/EXECUTIVE rollups" with the
capability-or-membership-proof derivation above, cross-referencing this ADR, and must name the
migration path for existing SUPER_ADMIN/EXECUTIVE principals.

## Consequences

- ADR-0003 must be amended reciprocally and this is not deferrable: leaving it unamended keeps a
  governance gap open under README:12 on every authenticated group-admin request.
- Strict two-way vocabulary equality means every future capability costs a migration row plus an
  enum variant plus (until `matrix_row` dies) one matrix cell — deliberately, foreclosing
  runtime-minted capability names, which is already the incumbent decision
  (`rbac-configurable.md:257-259`; `0065:64-65` grants `console_rt` SELECT only on
  `feature_catalog`).
- Narrowing the write path makes any already-written role carrying one of the other 15 attributes or
  `not_equals` un-resaveable without editing the condition. Those roles already grant nothing, so
  this is strictly better feedback, but it needs a one-time read-only count first (§7, X-T2f).
- The realtime `org_id` filter is a **narrowing of delivery**: any caller relying on a connection
  being reachable across orgs breaks. None was found; nothing was compiled.
- `AccessScopeLevel::Region`/`Worksite` stay fail-closed to `none()`. Shipping the enum arm without
  the resolver returns empty result sets, not errors — so **anything named "worksite-scoped read" is
  unavailable**, which bears directly on Slice 0's 현장 terminal (§8).
- It forecloses "delete the matrix to unlock the canvas" as a rationale: anyone proposing
  `matrix_row` deletion must justify it as cleanup, not capability.
- Deleting `team_policy_values` (`authz/src/lib.rs:1447-1455`, a hardcoded match on four Korean
  vehicle-workshop team names sourced from free-text `users.team`, `0002_create_users.sql:16`)
  breaks any deployment relying on those four names resolving through the current path; they must be
  re-expressed as principal attribute data first.
- The `group_memberships UNIQUE (org_id)` drop and `groups.parent_group_id` addition are sequenced
  AFTER this ADR and routed through **security** review, not schema review: those rows are the sole
  input to the cross-entity `{ADMIN}` + `All` mint at `request-context:405-421`.

## Alternatives considered

### Amend ADR-0021 as well
Rejected. Its decision 1 — roles "are subject inputs … not authoritative allow decisions by
themselves" — is a negative decision that stays true if the inputs are removed; it does not require
built-in roles to exist. Its decision 8 holds live routes until an explicit promotion, which is a
hold on switching, not a prohibition on removal.

### Two separate ADRs, one per theme
Rejected — see §0.2. `scripts/check-adrs.mjs` would pass two records amending the same clause
incompatibly.

### Draft G2b as scoped, conditional on `Role` deletion
Rejected. Its premise ("sole tenant-side derivation") is false, so it both understates the trigger
and under-scopes the remediation.

---

### D3 — `ADR-XXXX-DRAFT`: Audit-coverage exclusions are two, bound to a (file, function) pair

**Reciprocal record owed:** `ADR-0002-auditfirst-transactional-discipline-audit-event-in.md`
frontmatter gains `amended_by: [ADR-XXXX]` — verified it currently carries only
`related: [ADR-0014]`, so this creates the key — and gains ADR-XXXX in `related`. **Its Decision
text at line 20 must be edited in place**, not merely cross-referenced: a reciprocal key alone
leaves a false sentence standing in an authoritative record. Index row changes to
`accepted, amended`. ADR-0014 requires **no** amendment and is `related` only.

```yaml
---
id: ADR-XXXX
status: accepted
doc_status: published
date: 2026-07-29
owner: jasonlee
decision: audit-coverage-exclusion-cardinality-and-handler-surface
amends: [ADR-0002]
related: [ADR-0002, ADR-0014, ADR-0025]
---
```

# ADR-XXXX: Audit-coverage exclusions are two, bound to a (file, function) pair

## Status

Accepted. Amends `ADR-0002:20`. **Retroactive**: it records a pre-existing governance gap under
README:12 rather than authorising a new carve-out.

## Context

`ADR-0002:20` states, verbatim: "its exclusion set contains exactly one entry — the LocationPing
ingestion path (ADR-0014) — and a test asserts that is the only exclusion." That sentence is false
in `main`. Verified: `backend/ci/gates/audit-coverage/src/lib.rs:90-107` returns **two** exclusions,
`location_ping_ingestion` (`:92-96`) and `location_data_retention_purge` (`:101-105`), and
`backend/ci/gates/audit-coverage/tests/gate_detects_violation.rs:26-28` is named
`allowed_exclusion_set_is_the_two_location_carveouts` and asserts `exclusions.len() == 2`, pinning
the second by reason, file and function at `:40-45`.

The second carve-out landed with no governing record. ADR-0002 has no `amend*` key; ADR-0014 has
none either. Three prose sites still say "one": `ADR-0002:20`, the gate's own module doc
(`audit-coverage/src/lib.rs:9-11`), and the ecosystem plan (`:1359-1360`, `:1669`). The plan cited
the ADR prose rather than reading the gate — which is how the false premise propagated.

Two holes make the correction load-bearing rather than clerical:

- `is_handler_surface` (`audit-coverage/src/lib.rs:450-455`) matches path components exactly equal
  to `application`, `rest` or `worker`. `backend/app/src/objects.rs` has component `app`. It **is**
  scanned (`should_skip_dir` skips only `target`, `.git`, `tests`, `ci/gates` — `:648-663`) but is
  never classified as a handler surface, and zero `console-gate: state-changing-handler` markers
  exist anywhere under `backend/app/src/`. Its writers do call `with_audit` (`objects.rs:307`) and
  `with_audits` (`:388`) — audited by discipline, not by gate.
- `has_audit_emission` (`:434-444`) is a textual match, and `with_audits`
  (`backend/crates/platform/db/src/audit_tx.rs:112-139`) commits an empty `Vec<AuditEvent>` without
  objection. An empty vector satisfies both ADR-0002:20 and the gate. This is a legitimate path on
  idempotent replay (`backend/crates/ontology/rest/src/lib.rs:1621` returns `vec![]` on receipt
  replay), which is why a blanket runtime assert is the wrong fix.

## Decision

1. **The allowed-exclusion set is closed at exactly two entries**, each bound to a
   `(file, function)` pair, with a test asserting full set membership
   (`audit-coverage/src/lib.rs:90-107`; `tests/gate_detects_violation.rs:26-45`). A
   `(file, function)`-bound set of two is a strictly stronger invariant than an unbound set of one.
2. The second entry landed in `main` with no governing record. This amendment is retroactive and
   authorises nothing new.
3. **T7/P10/P11 add zero exclusions.** Every new write path — `work` create/close, `work_assignee`
   open/close, `work_artifact` link/unlink, email→work link, and the 인계 완료 assertion — routes
   through `with_audit`/`with_audits` with no carve-out. The email→work link is gated at the same
   capability class as granting authority, because linking confers retroactive read
   (`plan:761-762`).
4. **`is_handler_surface` is extended to treat `app` as a handler surface**, and each new verb
   carries a probe asserting ≥1 `audit_events` row **read from the database**, not inferred from
   source text.
5. **인계 완료 is one audited assertion, not a completeness query.** Outgoing party, incoming party,
   relinquished scope, and the count as asserted, computed server-side under a fixed authority. This
   is a consequence of `DN-0003:85-86` (denied data omitted, including counts and relationship
   existence), which this ADR cites as inherited and does not amend, and of `resolve_head`'s own
   stated no-existence-oracle rule (`backend/app/src/objects.rs:691-697`): `Ok(None)` is
   byte-identical for "absent" and "not visible". A completeness count over heterogeneous artifact
   edges is principal-relative — two people run it, get different answers, and the delta may not be
   exposed. A gate whose answer depends on who asks is not a gate.

Two stale prose sites are corrected in the same change, as documentation rather than governance:
`audit-coverage/src/lib.rs:9-11` ("The only allowed carve-out is LocationPing ingestion") and
`backend/crates/kernel/core/src/audit.rs:3`.

## Consequences

- Two migrations at 0207+ that the plan priced at zero: one `object_types` row for `work` and one
  `link_types` row for `work_artifact`. `0130_create_link_types.sql` closed the edge vocabulary
  (registry `:24-31`, twelve labels `:37-49`, FK `:75` validated by `0132:7-8`) and grants
  `console_rt` SELECT only (`0130:52`). `work_artifact` is not among the twelve.
- Per artifact kind P10 wants trustworthy: one `RESOLVABLE_KIND_AUTH` row (`objects.rs:121-138`),
  one `resolve_*` arm (`:702-724`), and a one-time audit of pre-existing links of that kind — the
  code requires this at `:120-124`, because registering a kind makes prior links retroactively
  resolvable with no backfill re-check. Of the sixteen kinds seeded at `0102:30-45`, the ten
  unregistered ones are exactly the artifact kinds while the ten registered are the actors, so P10's
  edges land precisely on the `Ok(())` skip at `:2306-2308`.
- **Offboarding cannot be hard-gated on 인계 완료 in Slice 0.** It ships as an audited assertion plus
  the gauge the plan already specifies at `:1540`. An incomplete handover is visible and provable but
  not blocking. Claiming otherwise would require an existence oracle.
- The `audit_events.action` FK is foreclosed permanently. Action-code drift stays a test-time
  concern: a projection that silently loses rows after a verb rename is caught by CI going red, not
  by the write path failing.
- One gate predicate change whose blast radius is **unmeasured** (§7, X-T7a). Treat it as unknown,
  not zero.
- Endpoint validation is at CREATE only; `object_links.src_id`/`dst_id` have no FK (`0102:57`,
  `:59`), so nothing removes an edge when its target dies. The gauge framing tolerates a dangling
  edge; a hard gate would not. This ADR must **not** assert a frequency for orphan edges — no delete
  path was traced.

## Alternatives considered

### Add an `audit_events.action` code catalog with an FK
Rejected on two independent grounds. An FK would let an unseeded verb abort the business transaction
it exists to record, inverting ADR-0002's same-transaction guarantee; and `feature_catalog` — the
cited precedent — is `GRANT SELECT` only (`0065:64-65`), so every new audited verb becomes a
migration and a deploy in a product whose governing constraint is that operations should not need
developers. A Rust const pinned by one unit test costs ten lines. The plan already refuses to build
the ledger on `audit_events` (`:490-491`, `:1133-1154`), so the target does not exist.

### Move the endpoint-resolvability invariant into a SQL trigger
Rejected. It would move a twice-battle-tested Rust check out of ADR-0001's unit-testable layer. The
design note names the two shipped security bugs it exists to stop (`objects.rs:106-120`).

### Leave ADR-0002:20 as-is and record the second carve-out only in the gate
Rejected. README:12 makes code diverging from an ADR a governance gap requiring a new decision, and
an authoritative record asserting a false cardinality is how this premise reached the plan.

---

### D4 — `ADR-XXXX-DRAFT`: Console frontend reconciliation and route-presentation instruments

**Owner decision required before this can be drafted to completion — see §6.1.**

**Reciprocal record owed:** `ADR-0025-carbon-copy-console-shared-platform-spine.md` frontmatter
gains `amended_by: [ADR-XXXX]` — verified it currently carries `amends: [ADR-0023]` and
`related: [ADR-0009, ADR-0018, ADR-0021, ADR-0022, ADR-0023]` with no `amended_by`. Pre-acceptance
this draft uses `proposes_amendments_to: [ADR-0025]` and **cannot** declare active `amends`
(README:26). Index row for ADR-0025 changes to `accepted, amended`.

```yaml
---
id: ADR-XXXX
status: proposed
doc_status: review
date: 2026-07-29
owner: jasonlee
proposes_amendments_to: [ADR-0025]
related: [ADR-0023, ADR-0025, ADR-0026]
---
```

# ADR-XXXX: Console frontend reconciliation and route-presentation instruments

## Status

Proposed. Becomes `accepted` with `amends: [ADR-0025]` only once the owner selects direction (a) or
(b) in the Decision below. This record is owed **independently of whether the ecosystem plan is
approved**: the tree already diverges from an accepted ADR with no amending record.

## Context

`git ls-tree HEAD web/` is empty. ADR-0025 is `status: accepted`, and its §7 legacy-deletion
conditions (rollback rehearsal, signed restorable packet, fourteen-day recurrence-aware
observation, 99.9% reconciled traffic classification) were not met; `:233` sets "one carbon-copy
visual system" as the target end state. `docs/PIVOT-2026-07-28.md` is not under `docs/decisions/`
and cannot supersede an accepted ADR (README:4, :10). Under README:12 that is a governance gap.

The route-presentation instrument fails in one direction only.
`scripts/console/validate-console-truth-ledger.mjs:270` refutes positive claims when
`route_source_present !== true`, but nothing refutes the inverse, and
`scripts/console/route-inventory.mjs:4-5` binds the extractor to two TypeScript paths
(`web/src/console/shell/nav.ts`, `web/src/console/screens/registry.ts`).
`scripts/console/route-inventory.test.mjs:14-21` concedes in its own comment that the deleted
positive/negative tests are permanently dead because the extractor parses TypeScript shapes "that
the Leptos rebuild will never emit at those paths." All 27 capabilities currently carry
`"route_keys": []`. **A Leptos console with real routes is therefore CI-green only if the authority
register records an empty, all-false entry: the anti-lying instrument compels a false record.**

## Decision

1. Record that `web/**` is absent and that ADR-0025 §7's deletion conditions were not met, so the
   deletion was a governance gap and not a supersession. `docs/PIVOT-2026-07-28.md` is named
   non-binding under README:1-2 and :10.
2. Resolve ADR-0025 §7 and its `:233` end state to exactly ONE of:
   **(a)** authorize the absence and restate §7's conditions as waived, naming the waiver authority;
   or **(b)** charter the rebuild against ADR-0025:133-142's nine-item shipped-screen evidence list,
   restating `:198-200`'s browser-destination-authority clause in stack-neutral terms so that
   `web/src/lib/objectRegistry.ts` is no longer named as the resolving artifact.
   **This selection is an owner decision (§6.1).**
3. `ADR-0025:133` clause 1 — a reachable, mounted body for every exposed navigation state —
   survives either direction unamended. `route-inventory.test.mjs` plus
   `validate-console-truth-ledger.mjs:270` are its enforcement, so the instrument fix below is an
   **implementation** of `:133`, not an amendment of it.
4. Before any console surface ships, `route-inventory.mjs` gains a Leptos-shape extractor and
   `validate-console-truth-ledger.mjs` gains the reciprocal assertion that a present source with
   routes forbids an empty/all-false register entry. `route-inventory.test.mjs:44-55` then asserts
   the derived state rather than the literal empty state.

## Consequences

- The Leptos extractor plus the reciprocal assertion are a **prerequisite** to any console surface,
  not a delta.
- Until it lands, no capability may claim a route, and the register's empty entries are the only
  honest record.
- Direction (a) and direction (b) have materially different downstream cost and neither can be
  inferred from the repository.

## Alternatives considered

### Treat `docs/PIVOT-2026-07-28.md` as the resolving record
Rejected by README:1-2 and :10. It is not in `docs/decisions/` and binds nothing.

### Swap `route-inventory.test.mjs:44-55` to source-derived using existing machinery
Rejected as unavailable. The machinery cannot see the target stack — the test's own comment at
`:14-21` records that the extractor parses shapes the rebuild will never emit.

---

### New non-amending records (N1-N5) — `related` only, no reciprocal `amends` key

Reciprocity under README:26 applies to `amends`/`amended_by` and `supersedes`/`superseded_by`.
Verified: `scripts/check-adrs.mjs:23-27` lists exactly those two pairs in
`RECIPROCAL_RELATIONSHIPS`, and `related` is validated only as an inline array (`:248-249`). Each N
record therefore adds its own id to the `related` list of every record it names, **in the same
atomic commit** with the index row — required by README:9's "explicit in both records" in spirit,
though not machine-enforced.

| Draft | Title | Plan slot | `related` | Reason no `amends` |
|---|---|---|---|---|
| **N1** | Effective-dated grants; the fold is computed, never stored | new (T3) | `[ADR-0021, ADR-0002, ADR-0003]` | ADR-0021:55-56 is what makes effective-dating **safe**, not what blocks it. Nothing is amended. |
| **N2** | Object-policy revocation is a catalog status transition (optional) | replaces struck G6 | `[ADR-0021, ADR-0023]` | ADR-0023:153-154 sits under "Follow-ups (named out of scope for this program)" (`:148`) and carries no charter clause. Out-of-scope is silence, not prohibition (README:7). |
| **N3** | 전결규정 routing and capacity-bearing signatures | G3, **re-scoped** | `[ADR-0018, ADR-0023]` | A **delta on ADR-0023's Engine-Gen**, not greenfield. `plan:1339`'s "new; zero ADR hits" is false — ADR-0023:81-82 decides DAGs and the 검토/승인/합의/참조 vocabulary. |
| **N4** | Conserved-quantity mechanism; lineage deferred | G4 | `[ADR-0001, ADR-0002, ADR-0018]` | The incumbents in T8 are schema, not decisions. The pure-predicate requirement reinforces ADR-0001:20 and :23. |
| **N5** | The double-entry voucher is the single money store | G5 | `[ADR-0002, ADR-0003]` | Zero accepted ADRs occupy this ground (verified: no file in `docs/decisions/` matches general ledger / chart of accounts / double-entry / voucher / 전표 / 회계). |

**N1 Decision, in brief** (full text belongs in the drafted record, not restated here):
`user_role_assignments` (`0065:141-152`) gains `valid_from`/`valid_to` with an exclusion constraint
replacing `UNIQUE (org_id, user_id, role_id)` (`:148`); the fold's join
(`authz/src/lib.rs:1270-1284`) gains a time predicate; grants stay in `user_role_assignments` under
`with_audit` and are **not** migrated into `ont_instance_revisions`, because that store's
append-only trigger permits only `valid_to` to be set and refuses any change to `valid_from`
(`0155:128-150`), making 소급 정정 of a mis-entered 발령일 inexpressible; the fold is never
materialized and never cached across requests, **citing ADR-0021:55-56 as the enabling reason** so
no future slice re-litigates materialization as an obstacle; an as-of read is admissible only over
as-of-capable inputs and otherwise returns a typed refusal **naming the unavailable input** —
the rule already executable at `ontology/adapter-postgres/src/instances.rs:1227-1228` and argued at
`:1132-1140`; `delegation_rule as_of D` IS offered (it resolves to a competent unit, `plan:566`),
`effective(person, scope, asof)` is NOT (two of three mutable inputs are historyless).

**N3 acceptance conditions** (not follow-ups): capacity is two nullable additive columns on
`gov_approvals` (`authorizing_grant_id`, `on_behalf_of_party_id`) following the
`0164_bind_consume_four_eyes.sql:32-33` precedent; `CHECK (approver_id <> requested_by)`
(`0153_create_governance.sql:74`) is retained **verbatim**, and the invariant sentence is
"**capacity refines a signature; it never satisfies a four-eyes gate**"; any migration that first
introduces `delegation_rule` must in the same file add the delegation-transitivity arm to
`backend/crates/governance/adapter-postgres/src/lib.rs:585-604` and a
`CHECK (delegator_id <> delegate_id)`.

**N5 blocking prerequisite:** `finance_gl_vouchers.accounting_date DATE NOT NULL` distinct from
`posted_at` (nullable, `0160:41`, stamped from `now_utc()` at
`backend/crates/finance-gl/rest/src/lib.rs:171,246,279`), plus a finance-gl
`assert_period_open(tx, PeriodLockDomain::Accounting, accounting_date)` caller. N5 must **not** claim
profit-as-a-query: `equipment_cost_ledger` (`0015:45-58`), `equipment_3r_dispositions.cost_minor`
/`sale_amount_minor` (`0182:96-97`) and `equipment_3r_rental_cases.monthly_rate_minor` (`0182:33`)
are three already-shipped parallel money records with three encodings. The sharpest single fact in
the theme: `backend/crates/finance-gl/rest/src/lib.rs:28` is
`const VOUCHER_FEATURE: Feature = Feature::PeriodLockManage;` — the GL voucher surface is authorized
by the period-lock capability and never calls the period-lock guard.

---

## 3. Rejections and defers

Each entry: what was proposed, why it failed, what would reopen it. An unrecorded rejection gets
re-proposed.

**R1 — §5.11 G1 as drafted (identity ADR grounded on "ADR-0022 decides identity is org-scoped").**
WITHDRAWN, not amended: its premise is false by direct read (`ADR-0022:25` is Context; the string
"org-scoped" is absent from the file), so there is no clause to amend, and its deliverable claim
(`plan:510`) is unachievable without the PII matching the plan itself rejects. Reopens only if a
different ADR clause is identified as the incumbent.

**R2 — The `party` table and `users.party_id` in Slice 0.** DEFERRED on irreversibility.
**Non-foreclosure constraints the entity model must not violate:** (i) no cross-tenant identifier as
a FOREIGN KEY, and none in any UNIQUE constraint or index whose key does not lead with `org_id`
(both enforced physically below RLS — X4 CONTROL 3, measured); (ii) the authorization path must
never read `employees`; (iii) `0075:16-17`'s CHECK must not be dropped or relaxed; (iv) when the
handle lands it is homed at the sentinel org with the ordinary `org_isolation` policy — no Tier O
entry, no new GUC, no definer-mediated read; (v) any eventual edge FK must be declared
`RESTRICT`/`NO ACTION` so `0196:132` (which filters `confdeltype IN ('a','r')`) consumes it.
Reopens when a consumer exists that is not itself HOLD — an **owner-controlled** premise (§6.3).

**R3 — Matching-based or confidence-scored identity deduplication.** REJECTED permanently, not
deferred. It is already prohibited in executable SQL and would require a cross-tenant read of
`employees` PII through FORCE RLS — the single disclosure the design exists to prevent.

**R4 — Deleting `Feature` / `matrix_row` to unblock the no-code authority canvas.** REJECTED as
framed. Enumerated against shipped code, deleting `matrix_row` unblocks **zero** of the six things a
tenant canvas cannot do: authoring a role ships (`0065`); granting an existing capability ships
(`authorize` unions matrix and grants at `authz/src/lib.rs:1191-1197`, so the matrix is a DEFAULT,
not a gate); minting a capability is blocked by `0065:64-65`; delegating `role_manage` by
`authz:1388`; non-branch/team conditions by `authz:1426`; org-wide reach by role literals in 13
non-test files. Thirteen collisions point at an object that is not load-bearing. `pub enum Feature`
is KEPT permanently as Cedar's action vocabulary (`authz/src/cedar_pbac/engine.rs:430`). Reopens as
**cleanup** only, never as capability.

**R5 — Per-condition ignoring of unevaluated grant conditions.** REJECTED. The plan, two of three
lenses and `0065:101-103` all proposed it; it is a widening dressed as a bug fix. Fail-closed at
`authz/src/lib.rs:1352-1361` is correct; the write path narrows to match it.

**R6 — Materializing the authority fold / a cached effective-authority projection.** REJECTED.
`plan:1116` row 1 ("Materialise per (party, scope), keyed on `policy_versions.version`") is a
cross-request allow/deny cache contradicting its own row 5 at `:1120`, `§4.6:820`, and
ADR-0021:55-56. Under README:4 a plan cannot supersede that. One-row deletion, not a G-pair. Also
mis-keyed: `policy_versions` is `PRIMARY KEY (org_id)` (`0065:177-181`), so keying there makes one
grant edit invalidate every connected client in a 10k-employee tenant — **creating** the org-wide
broadcast problem the theme then has no dimension for.

**R7 — `approval_signature` / `approval_line` as Tier N ontology instances** (`plan:568-569`).
REJECTED as a strict regression. `ont_instance_revisions.attributes` is a JSONB bag with
`ON DELETE CASCADE` on org (`0155:37-56`), so a signature stored there loses the
`(approver_id, org_id) REFERENCES users` FK, the SoD CHECK, the `UNIQUE (org_id, approval_id)`
single-use index and the RESTRICT durability posture that every shipped gate binds against. The plan
would create the exact hole the collisions blame on the CHECK.

**R8 — Retroactive 반려 as a transition on a closed approval line.** REJECTED. It stays the
compensating document already shipped
(`0097_create_workflow_compensating_documents.sql`, `backend/crates/workflow/domain/src/lib.rs:527-531`),
per ADR-0023:81-84 which is thereby SATISFIED, not amended.

**R9 — Making the four-eyes CHECK conditional on capacity.** REJECTED. There are **five** DB-enforced
four-eyes CHECKs (`0153:74`, `0122_create_leave_requests.sql:63`,
`0163_finance_gl_voucher_sod.sql:27`, `0186_payroll_run_lifecycle.sql:39`,
`0191_create_inventory_cycle_counts.sql:46`), and `0163:14` documents itself as mirroring the
gov_approvals idiom. A capacity-scoped predicate is selected by the actor it constrains, so it is
not an invariant. A band whose 전결규정 legitimately requires one human carries `four_eyes: false`
on its action; a document inside the drafter's own authority is raised with zero approval nodes.

**R10 — A universal component / ECS store for concerns (P14's substrate).** REJECTED, with the
reason named inside `DN-0003:195`: "a generic ECS/component store that becomes a **second writer**."
A generic write path asserting approval state or period lock on a projected payroll row is a second
writer for a fact that must not be bypassable, and ADR-0002's gate reasons about call sites, not
rows, so it cannot see it. P14's *goal* is largely already met by shipped declarative machinery
(§4.6).

**R11 — Derived group designation from shareholding/control edges.** REJECTED on authority grounds.
`group_memberships` is the sole input to `group_admin_member_orgs`
(`request-context/src/lib.rs:405`) and therefore the sole input to the cross-entity `{ADMIN}` +
`BranchScope::All` mint, so deriving designation would let a finance clerk's INSERT mint cross-entity
ADMIN on the next request; under 순환출자 the derivation is a strongly-connected component.
Designation stays STORED, EXPLICIT and AUDITED. Control edges may inform a human decision and may
**never** be an input to any authorization resolver. Say this in D2 so a later lane does not
re-litigate it as an optimisation.

**R12 — Quantity-bearing split/merge lineage (`lot`, `lot_split`, `lot_derivation`, recursive
traversal).** DEFERRED; no migration slot in the 0207+ window. Zero lot/batch/BOM/yield/genealogy
tables exist in the migration set and nothing needs a DAG: production's route is linear
(`0173:59`, `:88`), logistics' partial draw is linear on one row (`0179:48-49`), inventory's
consumption is linear (`0156:103`). **Non-foreclosure constraints:** (a) quantity-bearing or lineage
edges may never live in `object_links` — `0102:68` permits one edge per
`(org, src, dst, link_type)` so a repeated draw between the same pair is unrepresentable, and
`:86` grants no UPDATE so an amended edge would destroy its own audit line; (b) if `TRANSFER` is
added to `inventory_movements`, it carries a from/to location pair on ONE row — widening the `kind`
CHECK at `0191:97` while leaving the single `stock_location_id` at `:96` produces two rows whose
pairing is recorded nowhere on an append-only table; (c) no lineage ADR may be accepted until the
N-into-1 merge names its serialization point and lock order — every shipped precedent locks exactly
one row, so a merge introduces a deadlock class that exists nowhere in the repo today.

**R13 — A generalized single-sided managerial ledger as the economics spine.** REJECTED. It would be
a **fourth** money store, and being `amount > 0` single-sided it forecloses revenue by construction,
so it can never be the spine the thesis names.

**R14 — Widening `period_locks.domain` beyond `{payroll, accounting}`.** REJECTED as speculative
(rung 1). The vocabulary is closed in FOUR layers — DDL CHECK `0107:34`, Rust enum
`period_lock.rs:23-26`, fail-closed `parse` `:38-46`, and **three** published wire enums
(`openapi.yaml:13237`, `:32264`, `:32306`) behind the required drift suite — so the collision's
"minor: one CHECK becomes an FK" is wrong by a factor of four. No caller has named a freeze concern
that is not a statutory close cycle. Reopens when one is counted.

**R15 — `detached_at` on `ont_object_policies` as the revocation mechanism.** REJECTED as
mis-specified. `0154:90-99` installs `trg_*_no_update`/`trg_*_no_delete` calling
`cedar_policy_attach_append_only()`, which raises unconditionally (`0154:59-64`). The UPDATE is
blocked by TRIGGER, not privilege, so the proposed grant would not help and the fix silently
requires dropping an append-only trigger.

**R16 — P15's general fold simulator (W17) as drafted.** DEFERRED. `simulate(policies:
&[AuthoredPolicy], request: &SimRequest)` (`backend/crates/.../authoring.rs:706`) holds no DB handle,
and `AUTHORING_ACTIONS` is a closed five-verb set (`:246-252`) denying by omission at `:714`, so it
structurally cannot evaluate an approval-authority verb. **Non-foreclosure:** the deferred simulator
stays reachable, but it now has two named preconditions rather than mid-build discoveries —
`AUTHORING_ACTIONS` must widen first, and P6's single shared fold must land first or every simulator
claim is a fidelity claim over two hand-maintained programs.

**R17 — Removing columns from audited tables to reshape them.** REJECTED. `users`, `regions`,
`branches`, `user_branches`, `audit_events` are in `built_in_audited_tables()`
(`migration-safety/src/lib.rs:164-172`) and DROP COLUMN/DROP TABLE is a gate violation (`:262-312`).
Shape changes land as add-new-column + stop-writing-old + drop-the-read, leaving the deprecated
column in place forever.

**R18 — Dropping the `if:` at `.github/workflows/ci.yml:126`.** REJECTED. It is structurally
load-bearing: `ci.yml:94-95` derives the authority tip as `$CONSOLE_SYNTHETIC_MERGE_SHA^2` and the
candidate as its `^`, and a push to main has no second parent, so the step would fail on every push.
The HOLD property is already covered on push by the regression test at `ci.yml:142-143`.

**R19 — Bridging the `object_types` and `ont_object_types` registries.** DEFERRED, and do not spend
0207 on it. `backend/ci/gates/tenant-isolation/src/lib.rs:43-46`, `:53-59` allowlists `object_types`
as a global no-RLS table precisely because "the kinds themselves are not tenant data"; a
tenant-authored kind there is a cross-tenant name leak. **Non-foreclosure:** nullable
`src_ont_object_type_id`/`dst_ont_object_type_id` on `object_links` plus a CHECK that exactly one
endpoint form is set is additive, needing no FK change and no data migration.

**R20 — Converging the five ways to point at a domain object.** DEFERRED past Slice 0. There are
four incompatible pointers today — `object_links` (`0102:56-59`), `todos.links` JSONB (`0108:21-22`),
`notifications.link` JSONB (`0099:26-27`), and hardcoded FK columns
(`messenger_threads.work_order_id`, `0012:11`) — and `work` adds a fifth without retiring any.
**Non-foreclosure:** `work` must not gain a column that duplicates an edge, guarded by the plan's own
`no_duplicated_fact` probe (`:1514`).

---

## 4. THIRD_WAY decisions — mechanism, and why it beat both positions

**4.1 T2 — the vocabulary is the defect, not the matrix.** Incumbent said "the matrix is the
escalation floor, do not delete it"; pattern said "delete the matrix to free the canvas." Both were
wrong about which code is load-bearing. `matrix_row` (`authz:573`) and
`custom_role_runtime_feature_allowed` (`authz:1388`) are independent functions; the grant-vs-self
ceiling is `ensure_policy_roles_inside_actor_permission_ceiling`
(`backend/crates/identity/rest/src/lib.rs:2043-2082`), which needs *some* source of the actor's held
permissions, not the matrix specifically; and the no-lockout floor was never specified as a grid
(`rbac-configurable.md:260-262` specifies a holder-count rejection, and `system_policy_roles()` at
`identity/rest:1451-1468` already projects the six as `is_system: true` row-shaped roles). The
mechanism that beat both: **make `feature_catalog` and `Feature::ALL` one vocabulary in both
directions and narrow the condition write path to what the resolver evaluates.** 11 variants have no
catalog row — including `payroll_run_read` and `payroll_run_manage`, the named first vertical's own
capabilities — and are advertised by `GET /api/v1/policy/features`
(`identity/rest:742-759` → `:1433-1448`), pass `validate_policy_permissions` (`:1841`), then die on
the FK at `0065:92` → SQLSTATE 23503 → `ErrorKind::Validation`. 13 catalog keys have no variant,
which is exactly why `orgchange/rest:396-436` gates a 2026 조직개편 approval on role literals instead
of `authorize()` — the mechanism by which each new vertical reproduces the demo six.

**4.2 T3 — admissibility, not replay.** Incumbent said the version/digest cache forbids
effective-dating; pattern said build full historical replay. The incumbent's mechanism does not
exist: `bundle_digest` is `hex(sha256(schema_src ‖ policy_src))`
(`authz/src/cedar_pbac/engine.rs:490-495`) with grant rows not an input, `CompiledBundle` is
documented as "bundle identity + compiled artifacts, not a decision cache" (`:138-140`), entities
are built per request at `:449` immediately before `is_authorized` at `:460`, and a time-varying
grant already reaches Cedar as a principal attribute under a wall-clock predicate
(`compliance/adapter-postgres/src/lib.rs:615-623`) **in `CedarOnly` mode**. So effective-dated
authority already ships and already coexists with ADR-0021 §4/§5. The pattern's replay is unbuildable
while `users.team` (`0002:16`) and `user_branches` (`0002:23-27`) have no history. The mechanism that
beat both is **already written in this repo one level down**: `resolve_derived_attributes_tx` reads
referents as of `valid_from`, never at head, and FAILS the write when a referent has no revision at
that instant, for the stated reason that dropping the term would store "a smaller, entirely plausible
total" (`ontology/adapter-postgres/src/instances.rs:1132-1140`, predicate `:1227-1228`). Lifting that
rule to the fold costs one predicate discipline and one error path, zero new tables, and turns the
untemporalized backbone from a blocker into an enforced coverage boundary.

**4.3 T4 — `request_ref` is the node key, so the signature store already exists.** Both sides argued
over whether to build a new signature store. **Verified by direct read:**
`backend/crates/orgchange/adapter-postgres/src/lib.rs:1478-1489` binds `request_ref` to `step_id`
and `requested_by` to `request.drafted_by` on every step, so one org_change_request with eight steps
writes eight immutable `gov_approvals` rows and `UNIQUE (org_id, request_ref)` is one signature per
**NODE**. An N-node 결재 line in the governance spine is shipped and running the newest approval
domain in the repo. Every argument for a new store — including the plan's own blocker row at `:720`
— rests on the opposite reading. Capacity then costs two nullable columns and nothing has to be
relaxed: 전결 by delegated authority is already expressible (two rows, same approver, different
`request_ref`, same `requested_by`), and 전결 where the competent authority *is* the drafter needs
zero approval nodes, which is a stronger representation than a self-signed one.

**4.4 T5 — the field is empty, so there is nothing to amend.** Three of eight collisions describe an
empty field as occupied. Independent enumeration of `CREATE TABLE` across the migration set finds no
org-unit, department, position, team or competence table; `org_change_requests.target_kind='ORG_UNIT'`
with `target_ref TEXT` (`0198:23-24`) is an approval workflow routing four roles at an entity with no
row. An approval table's CHECK list and a backfill UPDATE are not decisions, so no amendment is
required or possible — and the Tier N substrate that answers unit kind, unit lifetime and
transfer-versus-disband already ships and is already runtime-writable
(`0155:44-45`, `:58`, `:73`, `:166`, `:170`, `:173`). Unit kind is a property, unit lifetime is a
revision interval, and transfer-versus-disband is two different link closures rather than one
settlement checkbox.

**4.5 T7 — 인계 완료 is an act, not a query.** Incumbent said tighten referential integrity until the
completeness query is trustworthy; pattern said build the query. Neither is reachable: `resolve_head`
states in its own comment that `Ok(None)` is byte-identical for "absent" and "not visible, so no
existence oracle" (`objects.rs:691-697`), and `DN-0003:85-86` makes omission-including-counts
binding, so a completeness count over heterogeneous artifact edges is **principal-relative** — two
people run it, get different answers, and the delta may not be exposed. Unrepairable by any amount of
referential integrity. The plan already contradicts itself (`§4.5:768` "a query, not a checkbox" vs
`§7:1540` "a gauge rather than a one-shot check"); the gauge framing is correct and already written,
so deleting the word "computable" is the whole fix.

**4.6 T6 — the extension axis is the verb, and the substrate already ships.** The theme's blocking
premise ("the only shipped extension mechanism is a hand-written closure registry, 1 of 7 types
wired") is false. Two type-agnostic declarative systems are executable code:
`sync_property_links_tx` (`ontology/adapter-postgres/src/instances.rs:874`, called `:723`, `:836`)
turns a property's `config.link` into a real effective-dated `ont_links` edge, and its own doc at
`:862` states "No type is named in this function, and no type needs engine code: a new one
contributes a JSON blob to its own declaration"; `resolve_derived_attributes_tx` (`:1142`, called
`:681`, `:769`) computes `config.derive` before canonicalization so the value is fixity-sealed; and
`0165_ontology_object_type_key_revisions.sql:1024-1041` has plpgsql itself INSERT a generic `create`
action on publish with `params_schema` and `edits` generated from the declared properties — zero
Rust, zero migration, zero redeploy. So extensibility is **open in the entity dimension and closed
in the verb dimension**, which is `DN-0003:97-99` invariant 10 already implemented. The seam IS the
invariant, so no incumbent needs amending; the cheap axis is widening the `derive` op set at
`instances.rs:1166`, which buys every tenant class a new declarative computation forever.

**4.7 T9 — the boundary is cut on the wrong axis.** The plan rejects a parallel money spine because
"two records of the same money diverge; that is a certainty, not a risk" (`:1075`). **That divergence
has already shipped three times** — `equipment_3r_dispositions` carries `cost_minor` AND
`sale_amount_minor` on the same row with `completed_at` and a buyer (`0182:90-106`),
`equipment_3r_rental_cases` carries `monthly_rate_minor` + `duration_months` (`0182:27-40`), and
`equipment_cost_ledger` carries `amount_won` + `entry_at` (`0015:45-58`), none of the last two
period-lock guarded. So the real question is not "how do we get a GL" but "which store is
authoritative for a money fact", and the plan's delta fixes the voucher while leaving two
unreconciled records. The third way: adopt the voucher as authoritative, make it conform
(`accounting_date`, line `branch_id`, `amount_minor` + `currency_code` per the pair already at
`0179:68`, `btrim`-normalized `account_code`), and **downgrade the platform's claim from
profit-as-a-query to cost-as-a-query**, naming the three parallel stores as the peer plan's
reconciliation backlog.

**4.8 T10 — forbid is a monotone ratchet, so ceremony is on the wrong write.** An over-broad PERMIT
is already correctable today with zero schema change: `effect` is a required wire enum
(`openapi.yaml:12291-12295`) passed through at `ontology/rest:521-522`, and `0205:280` mints a fresh
catalog `stable_key` per call whose own comment states "two policies on one type is a supported
shape, not a conflict", so the `UNIQUE` at `0154:35` is never hit. What is unfixable is a mistaken
FORBID: permits are OR'd into the base and every forbid is appended as `AND NOT COALESCE(...)`, with
an unconditional forbid lowering to `NOT TRUE` (`residual.rs:210-213`, `:246-248`, test
`unconditional_forbid_collapses_whole_filter_to_false` `:449-457`). **The system is a one-way ratchet
on forbid, and that write carries neither passkey step-up nor an impact receipt, while the fully
reversible write — DELETE + re-INSERT of role assignments — carries both.** Revocation is a catalog
status transition (`cedar_policy_catalog_entries.status` 'enforced' → 'retired', already CHECK-legal
at `0150:15`, `:43-46`; the catalog has no append-only trigger, `0150:106-110` installs only
`enforce_org_id_immutable`), and it must ship **in the same change** as `AND c.status = 'enforced'`
on BOTH `OBJECT_POLICY_SELECT` and `PROPERTY_POLICY_SELECT`
(`backend/crates/platform/authz-rest/src/store.rs:820-826`, `:828-834`) — otherwise revocation is
honoured on the enforcing read and silently ignored on the point-decision read.

**4.9 T11 — an invalidation ping, and a tenant boundary you can unit-test.** Incumbent said push
nothing; pattern said push the fold. The mechanism: one
`RealtimeEvent::AuthorityChanged { org_id, user_id, subject_version, session_generation,
policy_version }` carrying exactly the three counters the token mint already snapshots
(`backend/crates/platform/auth-rest/src/lib.rs:2866-2879`) and no capability material, routed through
the recipient-addressed `dispatch_notification` (`realtime/src/lib.rs:838-851`), explicitly NOT
`dispatch_event`, which returns `Ok(())` for any event lacking both a branch and a thread
(`:875-877`) — so an `authority_changed` event added per the plan's §5.6 cost line would fire the
notifier, return Ok, reach zero clients, log zero warnings, and still pass
`realtime_push_carries_no_capability` (`:1528`), which asserts payload shape and not delivery. Both
counters are carried because assignment writes bump the subject counter
(`identity/adapter-postgres/src/lib.rs:304`, `:672`, `:1606`) while role-definition and role-status
edits bump only the org counter (`:1284`, `:1369`) — keyed on either alone, a whole class of
authority change pings nobody. A `session_generation` change **disconnects** under a new fourth
`DisconnectReason` (the enum at `:406-411` has only `LaggingConsumer | ReplayFailed |
ServerShutdown`) rather than prompting a re-read.

**4.10 T12 — the repo already ships the third form of gate edit.** Both lenses shared a false frame,
"amend the gate" versus "the gate blocks us". `scripts/check-ci-preflight.mjs:430-453` derives its
requirement from a generated BUCK file, and its own comment (`:408-428`) names the hand-maintained
literal arrays as "the four-coupled-places defect this repo has shipped seven times", names why
repo-wide widening is impossible today (163 of 188 postgres-declared itests have no wrapper, measured
2026-07-29, and "a gate nobody can satisfy gets deleted"), and names the correct unit: "a per-crate
decision with the same shape as this one" — which is exactly Phase 3's unit of work. So the CI-wiring
cost is not a toll to pay but a defect to delete, monotonically strengthening, with no ADR. Gates are
then classified by **what they pin**: safety pins never weakened
(`validate-console-truth-ledger.mjs:294` HOLD; the audited-table DROP prohibition), two safety pins
**strengthened before the work they guard** (§5.12), literal-sameness pins on decisions replaced by
derivation per crate, and the GitOps freeze discharged by decision rather than weakened.

---

## 5. EVIDENCE CORRECTIONS

The 75 collisions were produced by a Collide phase that was never checked. This section is the audit
trail. **Reliability summary: of the collisions carried into the twelve themes, at least 12 are
premise-false (withdraw), at least 5 cite a file or line that does not exist, and at least 9 have
substance intact behind a wrong anchor.** Two of the collisions the brief named as the sharpest in
the whole set both fail. Three plan premises and one architect finding also fail.

### 5.1 Premise FALSE — withdraw the collision

| # | Claim | Correction |
|---|---|---|
| 1 | "One physical passkey cannot belong to a person who works at two companies" | `0004:7` is UNIQUE per *credential*; `webauthn.rs:349-353` uses the per-org `users.id` as user handle and `:339-342` builds `exclude_credentials` from that user's own passkeys only. One device against two handles yields two credential ids and nothing is rejected. The second half is inverted too: `0038:76-80` is `LIMIT 1` over a UNIQUE column (deterministic, not arbitrary), and login is discoverable with an EMPTY allowCredentials list (`webauthn.rs:786-796`) — **the passkey choice IS the org choice, and it already ships.** |
| 2 | "`gov_approvals` is INSERT-only with one row per request, so a 결재 line cannot exist in the governance spine" | **DECISIVE.** Verified: `orgchange/adapter-postgres/src/lib.rs:1478-1489` binds `request_ref` to `step_id`. `UNIQUE (org_id, request_ref)` is one signature per NODE. The claimed failure mode cannot occur; no caller inserts `decision='pending'`. Collateral: `plan:720`'s blocker row is wrong about the incumbent, and that wrong fact justifies inventing `approval_signature` at `plan:568`. |
| 3 | "Inbound email links to work through two hardcoded nullable FK columns" | `linked_work_order_id` / `linked_customer_id` are **dead columns**. Verified: zero `.rs`, zero `openapi.yaml`, zero frontend references; they appear only in `0053_create_comms_webmail.sql:94-95`, two partial indexes (`:196`, `:198`), and one `scripts/dev-seed.sql:771` row. Nothing links through them, so nothing needs DROP COLUMN and the migration-safety gate is never engaged. |
| 4 | "`todos` is person-keyed; handover is an UPDATE that rewrites the row's owner and leaves no prior-owner record" | `backend/crates/todos/adapter-postgres/src/lib.rs` has exactly three writers — `create` (`:83`), `set_done` (`:228`), `delete` (`:273`) — each `with_audit`-wrapped, each scoped `WHERE id = $1 AND owner_user_id = $2`. **There is no owner-change path at all.** The described UPDATE was never written. |
| 5 | "The real P7 defect is a silent grant drop … a five-line fix" | Already shipped: `identity/rest:1841-1842` `Feature::from_str(...).map_err(...)` inside `validate_policy_permissions`, called from both production write paths (`:861`, `:911`) and repeated at `:2050-2051`. `.ok()?` at `authz:1371` is defence-in-depth. Its premise is also inverted: `0065:64-65` grants `console_rt` SELECT only, so adding a verb is a migration, never an INSERT. |
| 6 | "Deleting the Feature matrix removes the only token-only authorization path and the only escalation floor" | `matrix_row` (`authz:573`) and `custom_role_runtime_feature_allowed` (`authz:1388`) are independent functions. The ceiling is `identity/rest:2043-2082`, which needs *some* source of held permissions. The no-lockout floor was specified as a holder count (`rbac-configurable.md:260-262`), never as a grid. |
| 7 | "An admin attaches an over-broad permit; there is no API and no SQL privilege that undoes it — the only remediation is a migration" | **Inverted.** A permit is correctable today via `effect: forbid` (`openapi.yaml:12291-12295` → `ontology/rest:521-522`), which wins in the enforcing read (`residual.rs:210-213`). The unrevokable case is the opposite one — a mistaken FORBID. |
| 8 | "`src_id`/`dst_id` are unindexed opaque TEXT, so a split/merge DAG cannot be expressed" | Refuted by the file it cites: the UNIQUE at `0102:68` leads with `(org_id, src_kind, src_id, …)` and `idx_object_links_dst` (`:73-74`) covers the reverse walk; `0102:71-72` says exactly this. Its title is also self-defeating — ignoring quantity the DAG *is* expressible; the constraint bites only on a repeated edge between the same ordered pair, which is the quantity fact the title says to ignore. And it is mis-targeted: `plan:670` stores `derived_from` as a `lot_derivation` row; `object_links` appears once at `plan:677` for `work_artifact`. |
| 9 | "Effective-dated grants defeat the version/digest cache key ADR-0021 makes load-bearing" | The ADR lines read as quoted; the mechanism does not exist. `bundle_digest` = `hex(sha256(schema_src ‖ policy_src))` (`engine.rs:490-495`), grants not an input; `:138-140` documents `CompiledBundle` as "not a decision cache"; entities built per request `:449`; a time-varying grant already arrives as a principal attribute under a wall-clock predicate (`compliance:615-623`) in `CedarOnly` mode. **ADR-0021:56 is what makes effective-dating safe.** |
| 10 | "P14's 'systems light up' has no substrate; 1 of 7 types wired" | `seed.rs` seeds **27** types (15 projected + 12 instance), not 7. `projected_draft` hardcodes `actions: Vec::new()` (`:199`), documented at `:180-183` as read-path only, and `dispatch_target` occurs exactly once in the whole seed (`:124`) as `None` — so the shipped no-code-reachable domain-write count is **0 of 15**, not 1 of 7. And two type-agnostic declarative systems ship as executable code (§4.6). Highest-value correction in T6. |
| 11 | "A closed entity-type allowlist is a foundation-gate rule" (`foundation-gates.md:49`) | MISREAD. That is item 3 of Gate C under an import/export lineage item — a data-class-to-destination-schema containment rule for ingestion, not an enumeration of permissible entity classes. Its mechanical pin (`scripts/check-g006-asset-dispatch-lifecycle.mjs:162`) is `requireIncludes`, a presence assertion, so appending a clause cannot break it. |
| 12 | "ADR-0023:154 says the canvas needs a named charter" | `:148` is the header "Follow-ups (named out of scope for this program)", `:152` is an unrelated Audit-SSE bullet, and the canvas bullet at `:153-154` contains **no charter clause**; "enters as its own charter" is at `:156` on a different bullet. Under README:7 out-of-scope is silence, not prohibition. **This voids G6's amendment premise.** |

### 5.2 Cited file or line does not exist

- `backend/crates/platform/db/migrations/0191_inventory_movement_conservation.sql` — **not in the
  tree.** Real file `0191_create_inventory_cycle_counts.sql`, whose `:119` does carry the
  conservation CHECK. Substance survives, path fabricated.
- `0072_equipment_ownership_transfer.sql` — cited three times across T4 and T5. Real file
  `0072_create_equipment_ownership_transfers.sql`.
- `docs/decisions/ADR-0022-local-identity-only.md` — cited at
  `ecosystem-plan-architect-findings.md:44`; does not exist. Real name
  `ADR-0022-local-identity-no-external-idp.md`, which `:79` has correctly.
- `0060_create_groups.sql` — `architect-findings.md:158`. Real name
  `0060_create_groups_and_membership.sql`.
- `0156_create_inventory_consumption.sql` — corrected independently at `architect-findings.md:296`.
- **Systemic:** two of two lenses in T8 cited migration files by plausible descriptive title rather
  than by reading the directory. Before any of the 75 collisions gates work, run a mechanical check
  that every cited path exists and every cited line still contains the quoted text (§7, X-CITE).

### 5.3 Substance intact, anchor wrong

| Cited | Actual |
|---|---|
| DN-0003 `:55-56` World/Entity; `:88` denied data; `:65-66` component table | `:56` World, `:57` Entity; `:85-86`; `:65-66` correct |
| `rbac-configurable.md:57-60` for R4's cache-key sentence | R4 spans `:55-59`; the sentence is `:55-56`. Cited range misses it and includes R5's opening |
| `authz/src/lib.rs:1367` for the whole-role drop | `:1367` is `let grants = permission_rows`. The drop is `:1352-1360` (`continue;` at `:1359`) |
| `0065` attribute CHECK "18 literals" / "22 literals" | **Exactly 17** (verified verbatim, `0065:110-128`); operator CHECK has 3 (`:129`). `competence`, `org_unit`, 직무, 직급 are NOT among them |
| `RealtimeEvent` at `:317-336`; `_ => Ok(())` at `:604` | `:318`-`:338`; `:603`. Plan repeats `:318-337` at `:130` and `:1122` |
| `listener.listen(...)` at `:585-587` (lens 2) | `:576-578`; `:585-587` is the `shutdown_rx.changed()` arm |
| `ci.yml:899-901` for `check:openapi-app` | `ci.yml:886`; `:899-901` is the employee-import replay contract. Both the script and its `ci.yml` step have since been deleted; the surviving route-inventory half is `check:platform-contract-drift` |
| `ci.yml:955-956` for the command-database gate | `:950` step name, `:951` `run:`. `:952-956` is the NetworkPolicy preflight **and its comment** — a comment offered as evidence for wiring behaviour, the exact forbidden form |
| `0198` second-table lines | Off by one from the second table onward: `step_order` `:56`, `role_key` `:57`, UNIQUE `:63`, `item_key` `:73-74`, `done` `:75`. First table correct (`:19`, `:23`, `:26`) |
| `0034:122` for the composite same-org FK story | `:122` is `users_id_org_key UNIQUE (id, org_id)`. The FK story is `:136-138` and `:141-142` |
| `0153:75` for `UNIQUE (org_id, request_ref)` (plan `:720`) | `:76`. `:75` is `UNIQUE (id, org_id)`. The `:78` approver FK citation is correct |
| `0108:29` composite FK; `:34-38` owner index | `:28`; `:33-34` (`:36-38` are RLS statements) |
| `accounting.md:29` for the multi-currency exclusion | `:30`. `:29` is the HomeTax/e세로 adapter line |
| `check-ci-preflight.mjs:34-47` | `:35-47` (`:34` is `supportDomainUnitCommand`) |
| `OBJECT_TYPE_LIFECYCLE_PATH` at `rest/src/lib.rs:198` | `:201`, routed `:244` |
| DN-0003 invariant 10 at `:98-100` (plan `:1367`) | `:97-99` |

### 5.4 Plan premises that fail

1. **`plan:1334`** — "`ADR-0022:25,33-39` decides identity is local/org-scoped." `:25` is Context;
   the Decision is `:31-39`; **the words "org-scoped" appear nowhere in ADR-0022.** G1's entire
   "unauthorized by ADR" framing is false. Independently reached at `architect-findings.md:77`.
2. **`plan:233-243` (§0.16)** — `authz:1478-1482` "is the sole tenant-side derivation of
   `BranchScope::All`." False; `request-context:421` is the second (verified). G2b is therefore both
   under-triggered and under-scoped.
3. **`plan:720`** — the `gov_approvals` blocker row ("One decision … No capacity column") is wrong
   about the incumbent, and that wrong fact justifies `approval_signature` at `:568`.
4. **`plan:1116` row 1 vs row 5** — P16 is not undecided; it is decided **twice, incompatibly, in one
   table**, and row 1 contradicts `§4.6:820` and ADR-0021:55-56 as well.
5. **`plan:1339`** — G3 "new; zero ADR hits" is false: ADR-0023:81-82 decides arbitrary approval-line
   DAGs and the 검토/승인/합의/참조 vocabulary.
6. **`plan:1577` and `:1619`** — "the 14 CI jobs in `.github/workflows/ci.yml`". **Verified: TEN
   jobs** — `preflight:75`, `support-domain-unit:163`, `postgres-domain-reachability:194`,
   `company-conformance:244`, `generated-face-authority:291`, `backend:340`, `dev-up-smoke:684`,
   `repo-gates:741`, `api-contract:827`, `kubernetes-manifests:906`.
7. **`plan:1066-1068`** — "four non-test" period-lock call sites; there are **five**
   (`financial/adapter-postgres:1254`, `workflow/adapter-postgres:792`, `orgchange/adapter-postgres:611`
   **and `:744`**, `backend/app/src/hr.rs:1706`). `architect-findings.md:261`, `:338` repeat "four".
8. **`plan:1179`, `:1195`** — attribute `output_quantity`/`scrap_quantity` (`0173:81-82`) to
   `production_plans`; they are columns of `production_operations` (`CREATE TABLE` at `0173:75`).
   `production_plans` (`:44`) has only `quantity` (`:50`).
9. **`plan:385`** — cites `0076:49-50` for the identity-resolution columns; `0076:49-50` is the
   backfill's SET list, the DDL is `0075:6,13`.
10. **`plan:699`** — argues `link_type` "is validated only by slug regex (`0102:63`) — so a new edge
    kind needs no migration." `0130_create_link_types.sql` created the registry, seeded twelve
    labels, and added the FK (`:75`) validated by `0132:7-8`; `link_types` is SELECT-only to
    `console_rt` (`0130:52`). The choice of `object_links` survives; **the stated reason is a stale
    comment cited as behaviour.**
11. **`plan:1359-1360`, `:1669`** — one audit-coverage exclusion. Two (§0.3).
12. **`plan:1193` vs `:1213`** — "unviolatable without any procedural code" contradicted by its own
    text nineteen lines later, which states the invariant as an aggregate.
13. **`plan:402-406` vs `:456-461`** — "the systems light up without anyone hand-writing an
    integration per concern" versus the correct opposite. Keep `:456-461`.
14. **"206 migrations"** (§0.8, §5.5) — 205 `.sql` files, highest `0205`; the 206th directory entry is
    `BUCK`.
15. **`plan:881`** — cites ADR-0023:154-155 for text at `:153-154`.
16. **`plan:672-675`** — `work_scope`, `work_origin`, `work_performed_at`, `work_jurisdiction` stored
    as `ont_link`, while `plan:536-540` makes `work` Tier T **projected**. `ont_links` FKs both
    endpoints to `ont_instances(id, org_id)` (`0155:76-77`) and a projected type owns no
    `ont_instances` rows (`instances.rs:1445-1450`, `:1514-1521`). **All four edges are rejected by
    referential integrity.** The plan caught this for `work_artifact` (`:699`) and missed it for the
    four scope edges — so clause 1 of the flagship 인계 완료 query (`:770`) has no storage either,
    not only clause 2. Not in any collision and not in the architect's list.
17. **§5.11 has no gate row at all.** `grep -ci` over all 1,853 lines returns 0 for `ADR-0025`,
    `route-inventory`, `check-ci-preflight`, `migration-safety`, and `command-database`. The
    practical gate on every other theme is absent from the plan.

### 5.5 Architect-findings corrections

- **`architect-findings.md:135` and `:387` quote a string that does not exist.** "There is no
  artefact you can print and hand to an auditor saying 'this is our approval authority as of
  2026-07-01'…" attributed to `research-sap.md:360-365, :412`. Grep finds it only in the findings doc
  itself and a paraphrase at `ecosystem-plan-review.md:618`. The real text is `research-sap.md:937-939`,
  reject **#4** (not #1): "You cannot print 'what is Kim's authority as of **today**' from S/4HANA."
  **The drift from "as of today" to "as of 2026-07-01" silently converts an evidenced current-state
  renderability gap into an unevidenced historical-replay requirement, and T3's replay scope inherits
  it.** Also `:915-919` does not contain the 전결규정 reject; that range is steal #14
  (`research-sap.md:912-915`). A lane sent to verify this finding as cited will conclude it is
  invented; the substance survives at the corrected anchors.
- **`architect-findings.md`'s `hold_rule` premise has zero executable readers.** `grep -rn "hold_rule"`
  over `scripts/`, `backend/`, `tools/` returns nothing; the token appears only as data in
  `docs/program/console-capability-registry.json`. The executable HOLD constraints are
  `validate-console-truth-ledger.mjs:255` (binding only capabilities at
  `rust_status === 'REQUIRED_UNRESOLVED'`) and `:294`. The inadmissibility finding argues from a field
  nothing enforces.
- `:44` cites a nonexistent ADR-0022 filename (§5.2). `:158` cites a nonexistent migration filename.
  `:261`, `:338` repeat the "four period-lock sites" miscount.

### 5.6 Stale prose contradicted by executable code

- **`backend/crates/kernel/core/src/ids.rs:81-85`** documents of `OrgId::platform()`: "no
  `organizations` row has this id." **Verified false:** `0036_platform_onboarding.sql:222-227` inserts
  exactly that row, with its rationale stated at `:217-221` ("a physical row must exist for
  referential integrity"). Same class of defect as the migration header that killed an earlier plan
  premise, and directly load-bearing for D1's sentinel-home design.
- **`0065:100-103`** says unsupported conditions "remain review/audit metadata"; the code drops the
  entire ROLE (`authz:1352-1360`). Prose and code disagree; the prose is struck.
- **`0102:60-62`** "Free-form-but-validated so new link types need no migration" — overtaken by
  `0130`, whose own header states the vocabulary was deliberately closed "instead of stringly-typed
  fiction."
- **`audit-coverage/src/lib.rs:9-11`** and **`kernel/core/src/audit.rs:3`** still say one carve-out.
- **`orgchange/rest/src/lib.rs:7-15`** says the `org_change_*` keys are registered in "migration
  0189"; they are in `0198_create_org_change.sql` (0189 is unrelated). One lens cited that comment as
  evidence — a header comment wrong on a checkable fact.
- **`docs/ideas/no-code-ontology.md:26`, `:32`** stale by two commits (#521, migration 0205): the
  lifecycle route exists and `0205` shipped the policy attach.
- **`docs/specs/accounting.md:26-28`** excludes "Persistent GL tables", contradicted by shipped
  `0160`/`0163`. Spec correction, **not** an ADR amendment; until corrected, `accounting.md:28` must
  not be cited as an incumbent in any collision or G-pair draft.

### 5.7 Two NEW verified defects in gates everyone agreed to protect

**5.7a — the audited-table DROP COLUMN prohibition is bypassable by spelling.** Verified by direct
read: `table_name_after_alter_table` (`backend/ci/gates/migration-safety/src/lib.rs:314-322`)
advances past only `if exists`, and `tokenize_sql` (`:443-460`) emits a token boundary at every
character that is not ASCII-alphanumeric or `_`, **including `.`**. Therefore
`ALTER TABLE ONLY users DROP COLUMN x` resolves the table to `only`, and
`ALTER TABLE public.users DROP COLUMN x` resolves it to `public`; neither is in
`built_in_audited_tables()` and neither raises `DropAuditedColumn`. Latent today (no such spelling
exists in the migration set) and unprotected by unit tests: `#[test]` count in `lib.rs` is 0 and the
sole integration test uses the bare `users` form. **Two lenses recommended trusting this gate
untouched while it silently protects less than it reads.** Harden before any migration 0207+ lands,
with one negative case per spelling.

**5.7b — the route-presentation instrument compels a false record.** See D4 Context. An anti-lying
instrument whose only green state for a real console surface is an empty, all-false register entry is
broken; fixing it is strengthening, not renegotiation.

### 5.8 Corrections to the defending lenses (not to collisions)

- **T4:** the claim that the delegate/finalize self-approval hole "is the current behaviour of shipped
  code" is FALSE. `enforce_finalize_policy` has no caller outside its own re-export
  (`workflow/runtime/src/lib.rs:33`); `.finalize_waiting_task(` and `load_finalize_waiting_task(` have
  **zero** call sites including tests; there is no `workflow/rest` crate; `workorder/rest` uses the
  runtime only as a flag-gated trigger publisher (`:3540-3552`). The defect is real and **unexposed** —
  `completion.rs:59-98` never compares the delegate to `initiated_by` — so it cannot be used as
  urgency evidence, and it is cheapest to fix now.
- **T5:** "`console_rt` has no raw SELECT (`0060:55-57`)" is FALSE — `0060:57` is
  `GRANT SELECT (id, slug, name, status) ON groups TO console_rt`, a column-level grant. The
  owner-only tables are `group_memberships` and `group_role_grants`. The conclusion survives; the
  mechanism citation does not.
- **T5:** "P5 is a scope-type change, from a branch set to a (branch × competence) product" is FALSE as
  a necessity claim. The `"team"` arm (`authz:1421-1425`) is already a non-branch attribute
  implemented as a subject-side predicate that gates the role via `return None` and never touches the
  scope.
- **T8:** "no single FOR UPDATE can serialize a merge, because a merge has no single parent row" is
  overstated — the child lot IS a single lockable row. The accurate finding, stated by no lens and not
  by the architect: a merge requires a **deterministic two-endpoint lock order and introduces a
  deadlock class absent from all three shipped sites**, each of which takes exactly one lock.
- **T8:** `charge_version` + `UNIQUE (org_id, request_id, charge_version)` (`0166:122,141`) is a
  **revision** of one request's charge resolution — `current_charge_resolution_id` FK at `:158-161`
  makes exactly one current — not two accumulating partial draws against one parent.
- **T10:** the ontology write path is neither "dead in production" nor "live in production". It is
  **deployment-configuration-gated**: wired only when `AppRole::Api` + `DATABASE_URL` +
  `ONTOLOGY_COMMAND_DATABASE_URL` are all set (`backend/app/src/lib.rs:1573-1583`), with the composition
  root at `:2925-2930` and a no-command-pool fallback at `:2929`. Both lenses were wrong in opposite
  directions.
- **T10:** `UNIQUE (org_id, object_type_id, cedar_policy_id)` (`0154:35`) does **not** prevent
  neutralizing a permit — each attach mints a new catalog row with a `gen_random_uuid()` discriminator
  (`0205:280`).
- **T11:** the "silent default arm" `_ => Ok(())` at `realtime:603` is **fail-closed**, not fail-open:
  discarding delivers nothing to anyone, it sits in the demux *before* fan-out, and it is unreachable
  today because `PgListener` yields only the three channels subscribed at `:576-578`. The brief's
  escalation to "an information-disclosure risk" is backwards. Separately, "no fan-out dimension above
  branch" is misframed: `org` is already in the path (`:866`, `:890`, `:1096`), and the dimension truly
  absent is above `org`, whose absence is correct under ADR-0018:230-233.
- **T11:** the missing subject bump on role-definition edits is **not** a governance gap —
  ADR-0021:57-60 requires bumping "subject/**policy**" versions, and `update_policy_role` /
  `update_policy_role_status` do bump `policy_versions` (`identity/adapter-postgres:1284`, `:1369`). It
  is a realtime invalidation requirement. Do **not** add a cross-join `UPDATE subject_authz_versions …
  WHERE role_id = $1`.
- **T11:** "delete the reserved ADR-0021 G-slot for this theme" — **no such slot exists.** §5.11's
  table (`plan:1334-1343`) contains G1, G2b, G8, G9, G2, G3, G4, G5, G6, G7; ADR-0021 appears only
  inside G2. The correct action is to widen G2's stated scope.
- **T12:** the KR cardinality rule is **not** single-pinned. `validate-console-truth-ledger.test.mjs:214`
  and `:234` both assert `:290`, wired unconditionally at `ci.yml:142-143`. So the safety-vs-decision
  taxonomy built on that asymmetry dissolves, **and** the opposing lens's "one-line edit" cost is also
  wrong — it is three files.
- **T12:** the GitOps freeze **is** fixable by editing the assertion alone. The frozen set is four
  deploy paths enumerated at `check-command-database-wiring.test.mjs:129-132`; `scripts/` is not among
  them, and the gate file's only executable reference is `ci.yml:951`. There is no ratchet. The
  two-commit discipline is kept on **measurement-independence** grounds, not impossibility.
- **T1/T4/T7:** `platform_force_remove_organization` is a **hybrid**, not catalog-driven: `0196:140-142`
  excludes `audit_events, employees, users, branches, regions` from the loop and removes them by
  hand-written statements (`:200-201`, `:311-313`). It is fail-loud, not silent — `:328-352` RAISEs
  `23503 platform_force_remove.direct_org_fk_requires_explicit_closure` at migration time. Call site
  is `:192`, not `:186`.
- **T6:** the ADR-0001 collision is wrong as stated. `layer-boundary/src/lib.rs:94`, `:128` govern
  *where Rust may live*, not *whether a concern requires Rust*, and the adapter crate holding both
  generic systems is `Layer::Adapter`. Delete the pair.
- **T8/T9:** `ADR-0001:23` is a **Consequences** bullet, not the Decision. The Decision at `:20`
  enforces crate dependency edges and the absence of `sqlx`/`axum`/`tokio` from domain/application
  crates, which a migration CHECK does not touch. So "the plan relocates invariants into SQL" is **not**
  an ADR-0001 governance gap. Also, the brief's "ADR-0001:23 is never named in any idea doc" is false:
  `plan:1346` quotes it verbatim as the whole of G8.

### 5.9 Resolved, previously "could not establish"

`attach_object_policy` **is** audited: `with_audit::<_, Uuid, PgCedarError>(&self.pool, event, …)` at
`backend/crates/platform/authz-rest/src/store.rs:231` wraps the definer call at `:234`, event built at
`:217`. Remove it from the G9 audit-coverage worry list.

### 5.10 Theme misfiling

"Imported person data granting authority is what ADR-0022 forbids" carries `Patterns: P6, P2` in its
own record and is a finding about position→authority derivation, not about a party. Adjudicating it
inside T1 makes `ADR-0022:38` read as a wall in front of the identity handle when `:38` governs
integrations *deciding authority*. The collision is CORRECT and present-tense
(`0063_create_employees.sql:6-11` is a spreadsheet import row), and its shippable artifact — freezing
the property that the authorization path never reads `employees`, true today at `authz/src/lib.rs:239`
which is a doc comment — needs no party, no migration, and only the D1 clause.

### 5.11 Not verified — flagged, never relied upon

1. **The tenant-wide deny hazard.** The claim that one per-org `policy_versions` bump denies every
   already-minted token in a tenant for up to the token TTL via `StaleSubject` (cited as
   `cedar_pbac.rs:68-79`, `:738-760`, `auth-rest/src/lib.rs:45`). If true it is a serious pre-existing
   enrollment hazard, independent of every theme. Nothing here leans on it (§7, X-STALE).
2. **Whether `finance_gl_vouchers` has rows in any environment.** Decides whether
   `accounting_date NOT NULL` needs a backfill and whether the `amount_won` rename is free.
   `plan:1074`'s "no production data claim" is an assertion, not evidence.
3. **Whether any executable purge, DSR or retention-delete path exists in the migration set.** No
   delete was read and nothing was run; the collision's "no erasure anywhere in 206 migrations" is
   unconfirmed. D1's deferral makes it non-load-bearing now.
4. **Whether `overlays/prod` can boot `CONSOLE_APP_ROLE=api`.** `backend/app/src/lib.rs:704-711`
   refuses when `DATABASE_URL` is set and `ONTOLOGY_COMMAND_DATABASE_URL` is absent; no `overlays/prod`
   kustomization includes the `governed-command-database` component; and
   `scripts/check-command-database-wiring.test.mjs:129-138` makes the one-line kustomize fix itself a
   gate violation. **Reached independently in three themes (T5, T6, T10) and never verified in any.**
   It changes no verdict — if true it gates shipping, not deciding — and needs someone who can inspect
   the running deployment.
5. **The "one of five authorization mechanisms gating most routes" route-share arithmetic.** No verdict
   may cite it until quantified.
6. **The blast radius of extending `is_handler_surface` to `app`.** Unmeasured, not zero (§7, X-T7a).

### 5.12 Confirmed exact and unchallenged

`authz/src/lib.rs:1139-1141`, `:1191-1197`, `:1352-1361`, `:1371-1373`, `:1388-1396`, `:1406-1408`,
`:1426`, `:1477-1541`; `kernel/core/src/access_scope.rs:88`; `identity/rest:742-759`, `:1433-1448`,
`:1843-1847`, `:2043-2082`, `:2433-2441`; `identity/adapter-postgres:59-62`, `:291`, `:1137-1148`;
`request-context:383-434`; `0046_add_member_role.sql:16-18`; `0065:12`, `:64-65`, `:92-93`, `:110-129`,
`:141-152`, `:177-181`; `authz/tests/policy.rs:849-850` (`Feature::ALL.len() == 96`);
`cedar_pbac/engine.rs:366`, `:430`, `:490-495`; `0102:47`, `:53`, `:55-59`, `:68`, `:73-74`, `:86`;
`0152:97`, `:156`, `:162`; `0155:37-56`, `:66-68`, `:73`, `:76-77`, `:79-80`, `:128-150`, `:166-173`;
`0205:12-14`, `:248-251`, `:280`, `:300`; `layer-boundary/src/lib.rs:94`, `:128`; `DN-0003:22`,
`:56-57`, `:65-66`, `:85-86`, `:97-99`, `:195`; `authoring.rs:246-252`, `:706`, `:714`;
`0147:20-22`, `:34-36`; `0166:122`, `:141`, `:158-161`, `:962-965`, `:993`, `:1332`, `:1336-1346`;
`0172:10`, `0178:84`, `0179:68`, `0182:35`, `0187:86`; `0160:20-49`, `:56`, `:62`, `:64`, `:78-118`;
`0173:50`, `:59`, `:75`, `:82`, `:88`; `0191:96-97`, `:119`; `0107:34`; `period_lock.rs:23-26`, `:38-46`;
`migration-safety/src/lib.rs:131-141`, `:164-172`, `:174-187`, `:262-312`;
`validate-console-truth-ledger.mjs:255`, `:270`, `:290`, `:294`; `route-inventory.mjs:4-5`;
`route-inventory.test.mjs:14-21`, `:36-42`, `:44-55`; `check-command-database-wiring.test.mjs:93`,
`:106-116`, `:129-132`; `ci.yml:126-127`, `:136-137`, `:142-143`, `:430-431`, `:886`, `:951`;
`check-adrs.mjs:23-27`, `:248-249`, `:399-406`; `ADR-0003:20`, `ADR-0002:20`, `ADR-0022:36`, `:38`
(all four read verbatim this session).

---

## 6. Low-confidence decisions needing a human

**6.1 — [T12, BLOCKING D4] Do ADR-0025's frontend clauses survive the 2026-07-28 pivot?**
Specifically §7's convergence program and `:233`'s "one carbon-copy visual system" end state. This
selects direction (a) — authorize the absence and restate §7's conditions as waived, naming the
waiver authority — versus (b) — charter the rebuild against ADR-0025:133-142's nine-item evidence
list, restating `:198-200` in stack-neutral terms. **It cannot be inferred from the repository.** What
is established: `web/**` is absent from `HEAD`; ADR-0025 is accepted with no `amended_by`; §7's
deletion conditions were not met; `docs/PIVOT-2026-07-28.md` cannot resolve it (README:1-2, :10); and
ADR-0023's `amended_by: [ADR-0025, ADR-0026]` proves the reciprocity mechanism is live and exercised.
Route via AskUserQuestion with exactly those two options before drafting D4 to `accepted`.

**6.2 — [T2, confidence high but the direction is a judgement] Should `feature_catalog` ↔
`Feature::ALL` equality be TWO-WAY or one-way with an allowlist?** Two-way is the recommendation,
because a one-way gate permanently blesses the role-literal bypass that `orgchange/rest:396-436`
already demonstrates. Two-way costs 13 new enum variants (bumping the pinned count at
`authz/tests/policy.rs:849` from 96 to 109) plus 11 catalog rows plus 13 cells in a `matrix_row`
slated for deletion. This is the one item in T2 not settled by code.

**6.3 — [T1, PROGRAM PRECONDITION, not an engineering task] Is a consumer of the identity handle
being sequenced ahead of the things it depends on?** D1 defers on "no consumer that is not itself
HOLD." That premise is owner-controlled. If group-company authority is sequenced first, **re-open the
timing half only** — the mechanism half, the ADR-0022 amendment and the sentinel-home shape all stand
unchanged either way. 27/27 capabilities HOLD; `console-program-ledger.md:823`: "Nothing in the idea
document is approved work."

**6.4 — [T9, confidence medium] Free-text `account_code`, or an FK to an account master?** The
"manageable without developers" constraint argues for a tenant-authored vocabulary; free text permits
that today but makes aggregates unreproducible (`'100'` and `' 100'` are two accounts —
`0160:62` rejects blank but stores untrimmed); an FK makes them reproducible but needs an in-product
maintenance surface. Neither is foreclosed by N5. State which the peer plan optimises for before it
designs the master. T9 is medium-confidence for this reason and because experiment X-T9a is unrun.

**6.5 — [T1, medium] The sentinel-home tenancy shape is unmeasured.** X4 built Variant A and Variant
B and never Variant C. D1's central shape claim rests on X4b (§7), which is schema-only and needs no
build. Confidence rises to high on a passing X4b.

**6.6 — [cross-theme] Migration 0207+ is a strict serial resource and no lane owns a slot.**
`check_migration_versions` (`migration-safety/src/lib.rs:131-141`) enforces gap-free contiguity, so
under parallel-lane fan-out the version space serializes Phase 3 — this, not any of the six T12
collisions, is what actually orders the work. Reservations known so far: D2/T5 up to two (JV/nesting;
optional CHECK widening), T2 one (11 catalog rows), D3/T7 two (`object_types` + `link_types` rows),
N3/T4 one (capacity columns), N5/T9 one (accounting_date + line branch_id + amount_minor), N1/T3 one
(interval + exclusion constraint), T10 one (impact-preview receipt). **That is nine slots against an
unallocated serial resource.** A per-lane slot table must be added to the plan before Phase 3 fans
out.

---

## 7. Experiments the verdicts demand

Every probe below states a falsifiable prediction **and** a known-bad control. Six probes were
defective in one session here; a probe with no demonstrated failure mode is not evidence. None of
these was run in this lane (read-only, no build).

**X4b — the sentinel-home tenancy variant. [Gates D1's central shape claim. Schema-only, no build.]**
Extend `docs/ideas/experiments/x4/probe.sql`. Create
`x4b_party(id, party_kind, status, created_at, org_id UUID NOT NULL REFERENCES organizations(id))`
with ENABLE + FORCE RLS and the standard `org_isolation` policy; seed two rows at
`org_id = '00000000-0000-0000-0000-00000000face'`; add one `party_org_visibility` edge for tenant A
only. **PREDICTION** armed as A: `count(*) = 0`, `EXISTS = false`, `count(DISTINCT id) = 0`, and
`count(*) WHERE id = <known second party id>` = 0 — while A's own edge still returns its `party_id`,
proving the hidden rows are physically present. Armed as `OrgId::platform()`: both rows readable, and
one `with_audits(OrgId::platform())` mutation's `audit_events` row readable back under the unchanged
`0035:107-112` policy. **KNOWN-BAD CONTROL 1:** the same table with RLS never enabled and SELECT
granted to `console_rt` must leak `count(*) = 2` to A, reproducing X4 CONTROL 1 and the Variant-A
cardinality disclosure at `experiment-x4.md:207-214`. **KNOWN-BAD CONTROL 2:** the same mutation
audited with `org_id = NULL` must be UNREADABLE even armed as the sentinel — proving the `0035:108`
`USING` gap is exactly what the sentinel home avoids, and that `0196:393`'s NULL-org receipt is unread
today. **REFUTED IF:** any new GUC is required, any of the 141 policies must change, a SECURITY
DEFINER becomes necessary, or the tenant can observe platform cardinality.

**X-T2f — inert-condition census. [Read-only. Must precede D2 clause 7 shipping.]** Count
`policy_role_conditions` rows where `attribute NOT IN ('branch','team') OR operator = 'not_equals'`.
Every such role grants nothing today. **PREDICTION:** 0 in any environment where Policy Studio's UI
only ever offered branch/team; a non-zero count means live tenants hold inert roles and the fix needs
an operator-facing migration notice, not just a 400. **KNOWN-BAD CONTROL:** insert a role with
`attribute='position', operator='equals'` and confirm `resolve_effective_feature_grants_in_org`
returns zero grants for its holder while `authorize` still passes for the same feature via a built-in
role.

**X-T5a — is the whole-role silent drop asserted as intended? [Must precede D2 clause 7.]**
**PREDICTION:** `backend/crates/platform/authz/tests/policy.rs` (which INSERTs into
`policy_role_conditions` at `:1498`) contains NO assertion that an unsupported attribute or
`not_equals` yields a role granting nothing, so the fix breaks zero tests. **KNOWN-BAD CONTROL:** a
test that inserts `attribute='position'` and asserts an EMPTY grant set. If that control exists, the
drop was deliberate and the fix needs an ADR clause, not just a patch.

**X-T4a — `request_ref` is the node key. [Confirms the correction that overturns `plan:720`.
psql only, no build.]** Drive two org_change requests through 2+ steps each, then
`SELECT request_ref, COUNT(*) FROM gov_approvals GROUP BY request_ref`. **PREDICTION:** N distinct
`request_ref` values equal to the number of DECIDED STEPS, not the number of requests, and zero
unique-violation errors. **KNOWN-BAD CONTROL:** a second INSERT with the SAME `step_id` must raise a
unique violation — if it does not, `request_ref` is not the node key and the reading is wrong.

**X-T7a — the handler-surface predicate's blast radius. [Blocking on D3's gate change, not on D3.]**
Extend `is_handler_surface` (`audit-coverage/src/lib.rs:450-455`) to match path component `app`, then
run the gate over the workspace. **PREDICTION:** zero violations, because every writer under
`backend/app/src/` already routes through `with_audit`/`with_audits` by discipline. **KNOWN-BAD
CONTROL:** plant one temporary handler under `backend/app/src/` with an sqlx INSERT and no wrapper and
confirm it fails; the gate's existing `gate_fails_state_changing_handler_without_audit` test proves
the detector fires. **IF THE PREDICTION IS FALSE** and it flags existing writers, that is more valuable
than the gate change and the count belongs in D3's Consequences.

**X-T6a — the declarative substrate against an authored type. [Cheap; nobody has run it. Proves
T6's central claim.]** On a scratch DB, POST a new instance-backed object type declaring one property
with `config.link = {stable_key, to_type}` and one with
`config.derive = {op:"sum", over, of, to_type}`; publish through the four-eyes
`ontology.schema.publish` approval; execute the auto-created `create` action twice; read `ont_links`
and the revision fixity hash. **PREDICTION:** an `ont_links` row appears bearing the declared
`stable_key`, and the derived property equals the sum of referents effective at `valid_from` and is
included in the fixity hash — with zero Rust changed and zero migrations added. **KNOWN-BAD CONTROLS,
both must 4xx rather than succeed:** (a) `config.derive {op:"count"}` must be refused with "which this
engine does not implement" (`instances.rs:1166-1172`) and must NOT silently fall back to sum;
(b) `over` naming an undeclared property must be refused and must NOT store a hash-sealed 0. If either
control succeeds, the closed-match discipline is broken and T6's verdict fails.

**X-T8a — `conserved_balance_requires_serialized_parent`. [The one measurement N4 rests on.]** Two
concurrent consumptions of 60 against a 100-unit item, each through `fetch_item_for_update_tx`
(`inventory/adapter-postgres/src/lib.rs:1019`) plus `InventoryItemView::consume`
(`inventory/domain/src/lib.rs:439`). **PREDICTION:** exactly one success and one conflict, final
`quantity_on_hand_milli` = 40. **KNOWN-BAD CONTROL, no new code needed:** swap in the non-locking
`fetch_item_tx` that already exists at `:1014-1017` — predicted to admit both writes, each row
satisfying `0156:103`, with 120 consumed from 100. **If the control does NOT overallocate, N4 clause
(5) is wrong and the row CHECK is stronger than claimed**, which reopens §5.8's mechanism as viable.

**X-T9a — the same-question-twice probe. [Decides N5's core claim. Scratch DB, no build.]** Post the
₩100,000 purchase through the finance-gl voucher path AND through `equipment_cost_ledger`. Ask "what
did equipment E cost in June?" from each store on 1 July, land a backdated June correction, ask again
on 20 July. **PREDICTION:** the voucher answer changes (`posted_at` is `now_utc()` and no lock is
consulted); the `equipment_cost_ledger` answer does not (`entry_at` is business-dated and
`financial/adapter-postgres/src/lib.rs:1254` refuses the write into a locked June with 409).
**KNOWN-BAD CONTROL:** repeat with the accounting period lock released — both answers then change,
proving it is the GUARD and not the date column alone that stabilises the figure. **If the voucher
answer is stable, `accounting_date` is unnecessary and N5's premise is wrong.**

**X-T9b — `account_code` whitespace split. [Two lines. Decides whether `btrim` is needed now.]**
Insert two lines with `account_code` `'100'` and `' 100'` on one voucher. **PREDICTION:**
`GROUP BY account_code` returns two rows, so the aggregate the plan's own `economics_is_a_view` probe
(`:1518`) depends on is already non-reproducible. **KNOWN-BAD CONTROL:** `GROUP BY btrim(account_code)`
returns one row. If both forms already collapse, `0160:62` normalises somewhere unfound and N5 item
(d) drops.

**X-RETIRE — object-policy revocation by catalog status. [Gates N2's retire arm only. Scratch DB.]**
Attach an unconditional forbid to a test object type via the shipped route, confirm reads return zero
rows, then `UPDATE cedar_policy_catalog_entries SET status='retired'`. **PREDICTION:**
`load_enforced_object_policy_blocks` (`authz-rest/src/store.rs:538-558`) returns no blocks, the
residual is not applied, rows are visible again. **FALSIFIED IF** rows stay hidden — which would mean
a bundle-digest cache (ADR-0021:55-56) or another consumer keeps the forbid live, and revocation must
then be forbid-append plus a cache-invalidation story. **KNOWN-BAD CONTROL:** the same status flip on
a type carrying only a PERMIT must NOT make rows visible, because permits-empty hits deny-by-omission
at `residual.rs:201-203`. If the control also flips to visible, the test is observing an unfiltered
read and proves nothing.

**X-STALE — the tenant-wide deny hazard. [Read-only; blocks Cedar action enrollment independently of
every theme.]** Read `cedar_pbac.rs`'s freshness comparison and error path plus the token TTL const.
**PREDICTION:** if the comparison is `>=` against a DB-current per-org minimum re-read per request,
then one 대결 grant write denies the whole tenant until TTL. **KNOWN-BAD CONTROL:** the same trace with
only a per-subject counter bumped must NOT produce tenant-wide denial. If confirmed, freshness must
become per-subject-derivable before more actions enroll.

**X-T11a — fold latency under invalidation burst. [Requires a build; cannot run in this lane.]** With
materialization deleted, P6 widens the fold from two sources
(`request-context/src/lib.rs:366-374`) to six. Measure p99 authority-fold latency and pool saturation
for a synthetic 10k-user org under a burst where one 전결규정 edit invalidates a realistic connected
fraction. **PREDICTION:** with per-`(org,user)` keying, one grant edit pings exactly 1 user and p99 is
unchanged from the two-source baseline within noise; with per-org keying the same edit pings every
connected client and p99 exceeds 3× baseline. **KNOWN-BAD CONTROL:** the identical burst keyed on
`policy_versions` only (the plan-as-drafted shape) — if that control does NOT degrade, the
per-subject argument is wrong and materialization deserves reopening.

**PR-0 — the missing red control. [MANDATORY before any T12 cost estimate is trusted.]** In one
throwaway PR, add a deliberately failing assertion to
`scripts/check-command-database-wiring.test.mjs` and a deliberately failing entry to
`check-ci-preflight.mjs`'s literal arrays; confirm both turn the required jobs red; revert.
**PREDICTION:** `ci.yml:951` and `:134` both fail, and `kubernetes-manifests` (`ci.yml:906`) fails only
via its `needs: preflight` edge, not on its own steps — because `check-ci-preflight.mjs:883-891`
asserts of that job only that it exists, needs preflight, and carries no job-level `if:`.
**KNOWN-BAD CONTROL:** the same PR with no assertion changed must be green; if it is red, the
instrument is measuring something other than the edit. Doubles as the `ci.yml`-change control
`architect-findings.md:201` requests for X8/X9.

**X-T12b — does the per-crate BUCK derivation generalize? [The whole Phase 3 saving rests on it.]**
`check-ci-preflight.mjs:430-453` matches `^    name = "(console-ontology-rest-itest-[a-z0-9_]+)",$`
— a crate-specific convention. 156 crate BUCK files carry the
`@generated by tools/buck/gen_first_party.py` header, so the source signal exists. **PREDICTION:** for
the first Phase 3 crate the generated target names follow the same `console-<crate>-itest-<name>`
convention, so the derivation generalizes by parameterising the prefix and nothing more.
**KNOWN-BAD CONTROL:** point the generalized function at a crate with zero itests and confirm it
pushes the "must declare integration-test targets" failure rather than silently passing on an empty
match — the exact absorb-nothing-as-nothing bug `:408-428` was written against. **If the prediction
fails, Phase 3 pays the three-place literal cost for every crate and the plan must say so.**

**X-CITE — mechanical citation audit. [A script, not a review. Highest systemic value.]** Verify that
every path cited across `adr-collisions-raw.md`, `ecosystem-plan-DRAFT.md` and
`ecosystem-plan-architect-findings.md` exists, and that every cited line still contains the quoted
text. **PREDICTION:** it reproduces at least the five nonexistent files and the fifteen anchor drifts
in §5.2-5.3 and finds more. **KNOWN-BAD CONTROL:** seed one deliberately wrong path and one
deliberately drifted line number and confirm both are reported; a run that reports zero is measuring
nothing.

**X-BTREE — confirm `btree_gist` is available before writing N1's migration.** If it is not, the
fallback is a deferred trigger-based non-overlap check, which is materially worse and must be
recorded as such rather than adopted silently.

---

## 8. Sequencing

### 8.1 Mapping onto §5.11's G1-G9

| Slot | Plan framing | Adjudicated | Record |
|---|---|---|---|
| G1 | new identity ADR | **WITHDRAWN** — premise false (§5.4.1) | **D1** amends ADR-0022 instead |
| G2 | `org_id` × `BranchScope`, "no ADR states how they compose" | UPHELD, **scope widened** to the realtime fan-out | **D2** |
| G2b | ADR-0003 amendment, conditional on `Role` deletion | UPHELD but **re-triggered to present tense** and re-scoped to both derivations | folded into **D2** |
| G3 | "new; zero ADR hits" | **FALSE** — re-scoped as a delta on ADR-0023 Engine-Gen | **N3**, `related` only |
| G4 | new | UPHELD, scope narrowed to the mechanism; lineage deferred | **N4**, `related` only |
| G5 | new | UPHELD; claim downgraded to cost-as-a-query | **N5**, `related` only |
| G6 | ADR-0023 charter amendment | **STRUCK** — the charter clause does not exist (§5.1.12) | **N2** optional, non-amending |
| G7 | DN-0003 amendment | **STRUCK** — DN-0003 is `kind: design-note`, `authority: subordinate`, and cannot take a reciprocal ADR pair at all (README:26 governs ADR relationship keys; design notes declare `parent_adr`). On the merits no edit is needed either | none |
| G8 | "argue, not amend" (ADR-0001:23) | UPHELD as argue-only. `ADR-0001:23` is a Consequences bullet, not the Decision, so no ADR question is engaged. Drop §5.8's lot CHECK from its three examples | none |
| G9 | Phase-7 prepwork | **RECLASSIFIED TO BLOCKING** — the incumbent's own sentence is false and Slice 0 ships `work` (`plan:528`) | **D3** amends ADR-0002 |
| — | **absent from the plan entirely** | The CI-gate theme has no G-slot and no architect finding | **D4** amends ADR-0025 |

### 8.2 Dependency order

```
D4 (ADR-0025) ── owner decision §6.1 ──┐   independent of Slice 0; blocks any console surface
                                       │
5.7a migration-safety resolver fix ────┼── PREREQUISITE to migration 0207+, all lanes
5.7b Leptos extractor + reciprocal ────┘   PREREQUISITE to any console surface

D2 (ADR-0003) ──┬── T2's F2 vocabulary gate + orgchange authorize()
                ├── T5's group-nesting migration  (security review, AFTER D2)
                ├── T11's org_id fan-out filter   (BEFORE the AuthorityChanged variant)
                └── N1 (T3 interval grants)       (shares ADR-0003's related list)

D3 (ADR-0002) ──┬── is_handler_surface extension (X-T7a first)
                └── work / work_artifact registration + 2 migration rows

N5 (G5)       ──── accounting_date + line branch_id + assert_period_open caller
N3 (G3)       ──── gov_approvals capacity columns (additive, nullable)
N4 (G4)       ──── mechanism only; no schema
N2 (optional) ──── retire arm + BOTH status filters + openapi rewrite, ONE change or none
D1 (ADR-0022) ──── independent; no migration slot; does NOT block Slice 0
```

### 8.3 What blocks Slice 0

Slice 0 = **the ₩100,000 비품 purchase, terminal at a 현장, recording the capacity it was signed
under.**

**BLOCKING:**

1. **D3 (amends ADR-0002).** Slice 0 ships `work` (`plan:528`), so G9 is not Phase-7 prepwork. D3 owes
   a retroactive reciprocal pair **before** it may add anything of its own, and it carries the two
   migration rows the plan priced at zero (`object_types` for `work`, `link_types` for
   `work_artifact` — `0130:52` grants `console_rt` SELECT only).
2. **N5 (G5 economics).** The ₩100,000 purchase posts a voucher, and
   `finance_gl_vouchers` has **no business date** (`posted_at` nullable at `0160:41`, stamped from
   `now_utc()`), **no line-level `branch_id`** (header has it at `0160:25`; the line `0160:57-68` does
   not), and **no period-lock caller** while its own surface is authorized by
   `Feature::PeriodLockManage` (`finance-gl/rest/src/lib.rs:28`). Without `accounting_date` the
   purchase cannot be asked about by business date; without the line `branch_id` a 현장-attributed cost
   is header-approximated.
3. **D2 (amends ADR-0003) — for the 현장 term specifically.** `AccessScopeLevel::Region`/`Worksite`
   project to `BranchScope::none()` until a DB-backed `BranchProjection` resolver ships
   (`authz:1536-1538`, `access_scope.rs:92-98`). **Shipping the enum arm without the resolver returns
   empty result sets, not errors.** So "terminal at a 현장" is either a `branches` row today or it is
   an unavailable read — that choice must be made explicitly, not discovered.

**NOT BLOCKING, stated so no lane waits on them:**

- **D1** — the party is deferred and consumes no migration slot. Capacity recording does not need it.
- **N3** — the capacity half of Slice 0 needs **no ADR**: `gov_approvals` already carries an N-node
  결재 line in production (`orgchange/adapter-postgres:1478-1489`), and capacity is two nullable
  additive columns following the `0164:32-33` precedent. This is the theme where the corrected
  evidence *removes* work from the critical path.
- **N1, N2, N4, D4** — none is on Slice 0's path.

**Slice 0's honest blocker count is three records and one owner decision, not nine.** Two of the three
(D2, D3) are amendment pairs owed for **already-shipped divergence** and are therefore owed whether
or not the ecosystem plan is ever approved.

### 8.4 Plan edits required before approval, needing no governance instrument

Withdraw G1; strike G6 and G7 with reasons recorded; re-scope G3, G2 and G2b; reclassify G9 as
blocking; add a gate row covering T12; add the per-lane 0207+ migration-slot table (§6.6); delete
`plan:1116` row 1 and `plan:402-406`; correct every item in §5.4; correct the fabricated quotation and
its anchors at `architect-findings.md:135`, `:387`; and add the non-foreclosure constraint that no
migration 0207+ may hard-code `'KR'` or `'KRW'`.
