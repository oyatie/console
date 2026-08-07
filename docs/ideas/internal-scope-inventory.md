> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Internal scope inventory — what is already decided, and what today's requirements change

Status: ANALYSIS — read-only inventory

> Produced 2026-07-29 by read-only inspection of `docs/decisions/`, `docs/specs/`, `docs/program/`,
> `docs/ideas/`, and the executable code and DDL they describe. No build, test, or migration was run.
> Every behavioural claim cites executable code or DDL. Where a header comment is cited, it is labelled
> as a header comment and is **not** used as evidence of behaviour.
>
> **Coverage, stated honestly.** Read in full: all 26 ADRs' `## Decision` sections plus
> `docs/decisions/README.md` and the DN- note index; `docs/PIVOT-2026-07-28.md`;
> `docs/MISTAKES-LEDGER.md`; and the load-bearing specs `rbac-configurable.md`, `org-hierarchy.md`,
> `no-code-org-ops-editor-approved-plan.md`, `cedar-pbac-coexistence-map.json`. Read targeted:
> `hr-core.md`, `knl-business-os.md`, `foundation-gates.md`, `LANE-PROTOCOL.md`,
> `authority-and-approval-model.md`. Verified directly against code/DDL: the audit shape and seal chain,
> the role/feature matrix, branch-scope resolution, RLS, the groups model, the ontology registry and
> conformance fixtures, and the GL voucher schema. **Not exhaustively read:** the remaining ~30 peripheral
> `docs/specs/` files, `console-program-ledger.md` (227 KB), `console-capability-registry.json` (328 KB),
> and `console-jurisdiction-register.json` — these were covered by mechanical scan (vocabulary and
> stale-reference greps) rather than full reading, so a requirement scoped only in their prose could have
> been missed. Four parallel sweep agents commissioned for that breadth did not return; the findings here
> are all first-hand.

## 0. The headline

Three findings dominate everything below.

1. **Today's requirements are already written down — in a document with no authority to change anything.**
   `docs/ideas/authority-and-approval-model.md` (dated 2026-07-29, i.e. today) already scopes
   substantially every item in the requirement list, section by section, with code citations. But
   `docs/decisions/README.md:10` states that `proposed`, `draft`, `design-note`, plan and prototype
   material **cannot supersede an accepted ADR**. So the delta is not mainly "what is new" — it is
   **"what must be promoted from an idea document into an accepted ADR."**

2. **The single largest governance gap is not any of the five suspicions.** It is that the
   2026-07-28 pivot deleted `web/`, `clients/`, `ios/`, `android/` and narrowed scope, and
   **not one ADR records it**. `grep` for the pivot across `docs/decisions/` returns nothing, and only
   `ADR-0017` carries a `superseded_by`. `docs/PIVOT-2026-07-28.md:105` prescribes the remedy ("Add a
   `superseded_by` / status note pointing here") and that remedy was never executed. Per
   `docs/decisions/README.md:12` — *"code diverged from an ADR; that is a governance gap, not silent
   supersession. Reconcile it through a new decision"* — this is eight-plus accepted ADRs out of
   reconciliation.

3. **Two of your five suspicions are wrong as framed, and being wrong about them is load-bearing.**
   Suspicion 2 (open sets conflict with compiler enforcement) is refuted outright. Suspicion 3 (a
   single missing `capacity` field) is wrong in both directions at once: ADR-0002 mandates no field
   shape at all, so no ADR blocks the column — but three fields are missing, not one, and the real
   obstacle is the tamper-evidence hash, which the brief does not mention. Details in Part C.

---

## Part A — What is already decided

### A1. Decisions table

`docs/decisions/README.md` is the maintained index and asserts its own authority rules
(`README.md:5-13`). The binding-decision column below was read from each ADR's own `## Decision`
section, not from the index or the title.

| ADR | Binding decision | In force? | Affected today? | Citation |
|---|---|---|---|---|
| 0001 | One Rust cargo workspace, one crate family per domain; dependency direction enforced **twice** — crate visibility (compiler) **and** a CI layer-boundary gate that also fails on `sqlx`/`axum`/`tokio` in domain/application crates | accepted | **No conflict** — see C2 | `ADR-0001-…md:20` |
| 0002 | Every state mutation runs `SELECT FOR UPDATE → validate → UPDATE → INSERT audit_events → COMMIT` via `with_audit`; `audit_events` append-only, UPDATE/DELETE revoked **and** trigger-blocked; audit-coverage CI gate with exactly one exclusion (LocationPing) | accepted | **Yes — amendment**, see C3 | `ADR-0002-…md:20` |
| 0003 | `Branch`/`Region` first-class day 1; principals carry `BranchScope` — `All` for SUPER_ADMIN/EXECUTIVE rollups, explicit branch set otherwise; repositories filter by scope by default (default-deny) | accepted | **Yes — supersede**, see C1 | `ADR-0003-…md:20` |
| 0004 | webauthn-rs passkey ceremonies on all surfaces; short-lived ES256/EdDSA access JWTs; opaque hashed refresh tokens with rotation-on-use, family tracking, reuse-detection → family revocation; OTP bootstrap | accepted | **Yes — amendment**, see C4 | `ADR-0004-…md:20` |
| 0005 | SeaweedFS self-hosted primary behind the S3 storage impl, hardened; every evidence object must reach a retention-locked copy in an independent failure domain; WorkOrder cannot reach FINAL_COMPLETED with unverified evidence | accepted, amended by 0024 | **Out of scope** post-pivot (evidence/WORM) | `README.md:36` |
| 0006 | P1 broadcast→accept dispatch, ≥2 accepts → auto-assign by live-GPS × priority score; separate FSM in the `dispatch` domain; push best-effort, escalation chain is the guarantee | accepted | **Out of scope**; `crates/dispatch` is on disk but **not a workspace member** | `README.md:37`, `PIVOT:94` |
| 0007 | Messages persist to Postgres **before** fan-out (DB row is truth); LISTEN/NOTIFY carries IDs only (8000-byte ceiling); explicitly **not** E2EE | accepted | **Out of scope** (comms) | `README.md:38` |
| 0008 | Use umya-spreadsheet 3.0.0 in `console-platform-excel` | accepted | No | `ADR-0008-…md` §Decision |
| 0009 | Parity enforced structurally: one utoipa `openapi.yaml`; **CI generates ts/swift/kotlin clients and fails on drift**; both apps build from every release tag | accepted | **Subject deleted** — `clients/` absent; PIVOT:22 says the three drift gates no longer exist | `ADR-0009-…md:20` |
| 0010 | `AiAssistantPort` stays a port definition only — no mock adapter, no UI affordance until a real adapter is owned | accepted, amended by 0022 | No | `README.md:41` |
| 0011 | All job scheduling through our own `JobQueue` trait; apalis-postgres is one adapter; RC admitted only after a timer-reliability soak | accepted | No | `README.md:42` |
| 0012 | **"One repository: `backend/`, `web/`, `ios/`, `android/`, `docs/`, `ops/`"**; contract and generated clients version atomically | accepted | **Subject deleted** — three of the six named dirs are gone | `ADR-0012-…md:20` |
| 0013 | Never issued — plan-only APNs placeholder; number reserved, do not reuse | n/a | No | `README.md:44` |
| 0014 | `location_pings` in a separate destructible day-partitioned store; coordinates **never** enter `audit_events`; consent lifecycle events are audited; the sole audit-coverage exclusion | accepted | No — but it is the one precedent for an audit carve-out | `ADR-0014-…md` §Decision |
| 0015 | Continuous WAL archiving + PITR mandatory, RPO ≤5min / RTO ≤1h proven by restore; VM-down degraded mode contract | accepted, amended by 0024 | No | `README.md:46` |
| 0016 | Define `AiAssistantPort` in `console-workorder-application`: `diagnose`, `draft_report`; no state transitions, no oyatie call, no feature surface | accepted | No | `ADR-0016-…md` §Decision |
| 0017 | Supersede itself; do not restore `IdentityProviderPort` | **superseded** by 0022 | No | `ADR-0017-…md:4` |
| 0018 | Build a clean-room Rust corporate workflow engine: versioned definitions (draft/simulate/publish/rollback), a **visual no-code canvas**, human tasks including **approval, rejection, delegation, escalation, sign-off**, RBAC/PBAC/ABAC with **group/org/department/team/position/custom-role scope**, passkey step-up for approval/signature, and **전자결재 with approval line, delegation, rejection, resubmission** | accepted | **Yes — already covers much of the Approvals block** | `ADR-0018-…md:70,87,109,115,218` |
| 0019 | Stalwart as benchmark only; build clean-room Rust mailbox, public MX disabled until gates pass | accepted, amended, **reconciliation required** | Out of scope (comms) | `README.md:50` |
| 0020 | Repo-native connector coverage factory; every connector starts non-live; official rails primary | accepted, **fixture-only** | Out of scope | `README.md:51` |
| 0021 | Cedar PBAC strangler behind an `AuthzEngine` boundary. **(1) PBAC via Cedar, not role-string RBAC** — built-in and custom roles are *subject inputs*, not authoritative allow decisions. **(2) RLS remains the row boundary** — Cedar cannot widen `org_id` or bypass RLS | **accepted target only — no live enforcement switch** | **Yes — this is the intended successor to the matrix** | `ADR-0021-…md:42,46,49`; `README.md:52` |
| 0022 | No speculative external IdP; passkey accounts remain the production identity source; **HR/attendance/payroll integrations must not authenticate users, assert sessions, grant roles, or decide account status** | accepted | **Yes** — constrains where `party` may come from | `ADR-0022-…md:38` |
| 0023 | Oyatie Console design is UI/UX authority; `/overview` is the canonical landing route; Work Hub is the behavioural contract | accepted, amended by 0025/0026 | **Subject deleted** (frontend) | `README.md:54` |
| 0024 | Self-host first; cloud-agnostic core through ports/adapters; providers first-class behind replaceable context boundaries | accepted | No | `README.md:55` |
| 0025 | Isolated carbon-copy console **rooted at `web/src/console/`**, mounted `/console/*`, one shared nonvisual platform spine, staged rollout | accepted | **Subject deleted** — `web/` absent | `ADR-0025-…md:51` |
| 0026 | Retire `coss-rn` as a public product surface; remove from npm workspace | accepted | No | `README.md:57` |

Design notes (subordinate, per `README.md:70-74`): **DN-0001** and **DN-0002** are DARK (self-host HA
scheduling; on-prem VIP/ingress). **DN-0003** is **IN PROGRESS** — "Palantir-derived operational object
runtime, deterministic Actions, object-focused tooling"; explicitly *not* release evidence.

**No ADR anywhere mentions** a compile-time `Feature` enum, a six-role matrix, `party`, 파견/worksite,
전결, 전담/competence, quantity lineage, or GL posting dimensions. The one `party` hit in
`docs/decisions/` is `DN-0003…md:98` — *"first-party tools are compile-time allowlisted"* — an unrelated
sense of the word.

### A2. Governance rules that constrain the answer

These are asserted by the decision directory itself and they determine the shape of the whole delta:

- `README.md:8` — only another **accepted** ADR may amend or supersede an accepted ADR.
- `README.md:9` — a later number does **not** win automatically; supersession must be explicit in **both** records.
- `README.md:10` — `proposed`, `draft`, `design-note`, plan and prototype material **cannot** supersede an accepted ADR.
- `README.md:12` — **"Current implementation/live evidence may show that code diverged from an ADR; that is a governance gap, not silent supersession. Reconcile it through a new decision."**

`README.md:12` is the operative sentence for this whole inventory. Divergence has already happened in
code (Part C1), and the directory's own rule says that requires a decision, not a plan paragraph.

### A3. `docs/PIVOT-2026-07-28.md` — what the pivot changed

Declares itself "the **single source of truth** … When any other document disagrees with this one,
**this one is right**" (`PIVOT:3-6`).

- Frontend entirely deleted, 1,820 tracked files: `web/`, `clients/{ts,kotlin,swift}`, `e2e/`, `android/`, `ios/`, build config (`PIVOT:9-21`). Verified: `web/` and `clients/` are absent.
- Repo renamed `maintenance` → `console`; `mnt_rt` deprecated (`PIVOT:26-32`).
- Frontend stack decided: **Leptos 0.9**, not React; returns **last**, as the acceptance test (`PIVOT:34-48`).
- Scope narrowed hard: **Ontology · Foundry · Policy** as one governed object engine, then **Organization + Employee**, then **HR + Payroll**. Explicitly **out**: ERP modules, field ops, dispatch, comms, compliance, ingest, evidence/WORM, office editor (`PIVOT:59-66`).
- **The engine is not greenfield — it largely exists** (`PIVOT:76-88`).
- ADR policy: never rewrite; add a `superseded_by`/status note pointing at the pivot (`PIVOT:105`).

**Two of the pivot's own claims do not survive checking:**

| Pivot claim | Verified state |
|---|---|
| `PIVOT:68-74` "Build system: cargo, not buck2" | `.buckconfig`, root `BUCK`, `buck-out/` and thousands of `BUCK` files are all still present. `LANE-PROTOCOL.md:264`, committed **20 hours after** the pivot, says *"buck2 is retained and is the parallel-build path"*. `docs/specs/foundation-gates.md:15` pins a **"Buck2-only"** evidence policy as a durable contract verified by gate **G002**. Commit `294f0234c` (#508, post-pivot) hardens Buck target resolution. |
| `PIVOT:82` ontology engine "17,825 LOC" | `backend/crates/ontology/` measures **8,221** non-test + **19,088** test = **27,309** total. The stated figure matches neither. (`platform/authz` at 7,912 vs the stated 7,861 is ordinary drift.) |

So the canonical truth document is itself partly stale, on the one axis where it is contradicted by a
**later** commit. That is not a nit: the pivot claims arbiter status, and a later change disagreed with
it without amending it. **This needs a human decision, not a guess** — I am not resolving which of
`PIVOT:68` and `LANE-PROTOCOL.md:264` is intended to win.

### A4. `docs/specs/` — what is scoped, and its build state

The two specs that carry most of today's requirements:

**`rbac-configurable.md`** — Status: *"DESIGN / TARGET STATE — adversarial security-review DONE (verdict
**NOT-YET — HIGH**: 3 CRITICAL + 4 HIGH + 3 MEDIUM)"* (`:3-4`). Partially built: *"Current shipped
increment is §9 plus early P1 guardrails"* (`:5`). It already scopes, in binding language:

- `:116-119` — *"The built-in 6 remain as **immutable bootstrap system roles**; custom policy layers on top. The authz engine resolves a principal's effective permissions from the **effective policy** (system defaults ∪ tenant custom roles ∪ responsibility / attribute rules) **instead of the static matrix**."* — this **is** `effective(person, scope) = fold(grants)`, already scoped.
- `:29` — **R1: "Ban role-string authorization."** *No authorization decision may branch on a `Role` variant or role string.* 18 enumerated sites; a new CI gate must fail on `matches!(*, Role::…)` outside the authz resolver. It names the dangerous ones: the "grant ≤ self" guard, the OTP/credential-reset privileged-target gates, and a money-path approval router.
- `:126-131` — the multi-layer role model: **job function**, **department/team**, **position/level** (*"authority band used for approval thresholds and escalation"*), **responsibility assignment**, and **scope: platform, group, subsidiary/org, department, branch/site, object, self**.
- `:96-97` — the thing that breaks on deletion: *"`permission_for(role, feature)` indexes the immutable system-role floor"* and *"A user's system roles ride in the **verified JWT** (`AccessClaims.roles` → `Role::from_str`)"*.

**`org-hierarchy.md`** — Status: *"P0 schema/resolvers IMPLEMENTED; P1 AccessScope kernel bridge
IMPLEMENTED; P2 JWT claims IMPLEMENTED; P3 consolidated-read helper IMPLEMENTED; P4a platform-operator
bridge LOCAL/UNRELEASED; P4b+ … remains open"* (`:3-6`). Scopes `Group → 법인(Org=RLS boundary) → Region
→ Branch → Worksite/Site` (`:1,47`). Critically:

- `:6-8` — *"the per-법인 `app.current_org` boundary is **UNCHANGED**. This spec adds a controlled cross-entity scope *above* that boundary; it never punches a hole in it."*
- `:241,246-247` — *"The per-법인 matrix is UNCHANGED. A separate parallel capability set… Effective authority = (group gate if a group endpoint) AND (per-법인 RLS, one member) AND (per-법인 `authorize()` role×branch) — all default-deny, all must pass"*, and `:235` *"the group layer ADDS a marking, never REMOVES a per-법인 restriction"*.
- `:163` — *"`group_id` NULLABLE → an ungrouped 법인 is unchanged"*. Group designation is **stored**.
- `:297` — an already-open question: *"법인 must NOT self-join a group. Confirm."*

**`no-code-org-ops-editor-approved-plan.md`** — Status: **APPROVED PLAN ARTIFACT / PLANNING ONLY**;
*"No code, schema, endpoint, migration, or production policy change is authorized by this document"*
(`:4-6`). This is the widest already-scoped surface:

- `:12` — target product models *"subsidiaries, departments, teams, worksites/사업장 cells, employees, **positions**, reporting lines, cross-organization work assignments, inherited policies, local rules"*.
- `:71` — *"Department / Team / OrgUnit … Tree/canvas/table create, rename, move, nest, assign manager position, deactivate/archive/restore, **cycle checks**"* — canvas plus cycle detection.
- `:254` — the subject-input list already enumerates **every** grant source today's requirement names: *"department/team, position, responsibilities, worksite reach, **group grants, delegations, active assignments**, policy version"*.
- `:44` — the hard constraint: *"Group/HQ, worksite/cell, assignment, policy, role, or scope ids **never arm RLS**."*
- `:244` — *"Runtime decisions evaluate capabilities, relationships, assignments, object attributes, action purpose, policy/ruleset version, and context — **not role strings alone**."*

**`cedar-pbac-coexistence-map.json`** — `status: "design_no_live_switch"`,
`invariants.noLiveSwitchInG001: true`, and **two** entries, both `currentMode: "legacy_only"`. Its
`invariants.rlsHardBoundary` states: *"Cedar decides capabilities/actions; Postgres `console_rt`/FORCE RLS
decides row visibility and mutation reach."* **That invariant is exactly the "split operational scope
from decision scope" the new model asks for — already articulated, in an accepted-ADR-backed artifact.**

Other specs, briefly: `hr-core.md:8,10` promotes *"org unit/department, job, position, worksite"* to
first-class **columns** and derives an org chart *"company → org unit/worksite → position → employees"*
— the right vocabulary in the wrong shape (columns, not entities). `knl-business-os.md:19,146,162` names
a **인력 파견·용역 agency** as a target tenant *vertical* — not the employer≠worksite relation.
`accounting.md`, `payroll.md`, `erp.md`, `mes.md`, `cx-reporting-bi.md` are thin and aspirational;
`erp.md` is 1.1 KB and now out of scope.

### A5. `docs/program/` and `docs/ideas/`

`docs/program/` holds the operational registers: `console-program-ledger.md` (227 KB),
`console-capability-registry.json` (328 KB), `console-jurisdiction-register.json` (52 KB),
`console-enterprise-roadmap.md`, `LANE-PROTOCOL.md`, `ontology-deployment-gap.md`. The registry and
ledger both predate today's requirements and **register none of the new vocabulary** — 전결, 전담,
순환출자, 연락사무소 and 결재권 appear in **no** program document.

`docs/ideas/` is where today's requirements actually live:

| File | Status | Live? |
|---|---|---|
| `authority-and-approval-model.md` | idea-refine output, 2026-07-29 (`:3`) | **the pivotal document** |
| `no-code-ontology.md` | idea, 2026-07-29 (#520) | live; its §232 deferral is explicitly overturned |
| `ecosystem-plan-DRAFT.md` | DRAFT (68 KB) | live draft |
| `execution-plan-DRAFT.md` | approved 2026-07-28 (`fa086a0a2`) | live |
| `fanout-plan-DRAFT.md`, `delegation-economics.md`, `process-minimum-viable-provenance.md` | DRAFT | live |
| `governed-object-engine.md` + `-PLAN.md` | plan | live |
| `lane-assembly-line.md` | 2026-07-29 | live |
| `enterprise-role-workflows.md`, `build-strategy.md` | pre-pivot | partly stale |
| `observability-oci-freetier.md` | 2026-07-10 | stale (pre-pivot infra) |

A mechanical scan settles where the new vocabulary lives: **순환출자, 연락사무소 and 결재권 appear
nowhere in the repository except `docs/ideas/authority-and-approval-model.md` and
`docs/ideas/ecosystem-plan-DRAFT.md`.** 전결, 파견 and 반려 appear mostly in
`docs/design/oyatie-console/` — the **superseded** design authority. The domain knowledge exists; it has
simply never been promoted into a spec or an ADR.

`authority-and-approval-model.md` already contains, with citations, an analysis that reaches the same
conclusions this inventory reaches independently. Its section map alone answers most of Part B: §139
업무 as entity, §212 permanent 부서 vs temporary 사업장, §225 고용형태's two conflicting closed
vocabularies, §245 전자결재 resolved by competence not rank, §262 소속/직급직책/직무/결재선 as
independent dimensions, §299 standing from the 결재권 graph, §340 retroactive 반려 as an obligation
loop, §383 *"What makes a closure final? — **unresolved, and blocking**"*, §436 three relations
currently conflated, §454 three locations all of which may differ, §461 전결규정, §511 *"The keystone:
there is no person"*.

---

## Part B — The delta

Legend: **BUILT** = running code/DDL. **SCOPED** = specified in a document with standing. **DRAFT-ONLY**
= exists only in `docs/ideas/`, which per `README.md:10` cannot bind. **NEW** = nowhere. **CONTRADICTS X**
= cannot be built without reversing X.

### Identity and organisation

| Requirement | Verdict | Evidence |
|---|---|---|
| Platform-level `party` above org-scoped `users` | **NEW + CONTRADICTS ADR-0003's substrate** | `users.org_id SET NOT NULL` (`0029_enforce_org_id.sql:29-33`); `users_id_org_key UNIQUE (id, org_id)` (`0034:122`). Same human at two companies = two unrelated rows. No `home_org`/`primary_org` column exists anywhere. |
| Tenant visibility by **edges**, not row scoping | **CONTRADICTS `org-hierarchy.md:6-8` and `no-code-…-approved-plan.md:44`** | RLS is `org_id = current_setting('app.current_org')` on every tenant table (`0030_enable_rls.sql:33-42`). Both documents forbid anything but `org_id` arming RLS. |
| Employee and position as **separate entities** | **BUILT (ontology), partially** | `job_position` and `employment` are distinct authored object types with an `employment_position` link (`ontology/rest/tests/company_conformance/fixtures/job_position.rs:122`, `employment.rs:140,199`). But `employment.title_property_key = "person_name"` and `person_name` is a text property (`employment.rs:142,148`) — **the post is an entity, its occupant is a string**. In SQL, `position` is a TEXT column on `employees` (`0066_hr_core_employee_fields.sql:26`). |
| Employment separating **employer** from **worksite** (파견) | **NEW** | `employment` declares only `person_name`, `job_position_id`, `org_unit_id`, `base_salary` (`employment.rs:148-179`). No employer link, no worksite link. `hr-core.md:8` has *worksite* as a column; `knl-business-os.md:19` uses 파견 to mean a tenant vertical, not the relation. |
| Employment type as authored data with per-type rules | **NEW — and the current state is actively broken** | Two closed `CHECK` vocabularies that **disagree**: `employment_type IN ('REGULAR','CONTRACT','PART_TIME','INTERN')` (`0172_create_employee_employment_profiles.sql:7`) vs `IN ('REGULAR','RESIDENT_SHIFT','PART_TIME','POOL_DAILY')` (`0187_create_recruiting.sql:22`). A hire and a requisition cannot agree on a type today. |
| Org-unit **kind** (부서/팀/TF/사업장/…) | **NEW** | `org_unit` declares only `name` and `parent_org_unit_id` (`org_unit.rs:137,146`). No `kind`. No org-unit table exists in SQL at all — `p_org_unit` is a TEXT parameter (`0183_leave_api_create_employee.sql:129`). |
| Org-unit **lifetime**, temporary unit's lifetime from a **contract** | **NEW** | No lifetime, validity or contract reference on `org_unit`. |
| **Control edges** between legal entities; group designation **derived** | **CONTRADICTS `org-hierarchy.md:163`** | Group membership is **stored**: `organizations.group_id` (`0060_create_groups_and_membership.sql:27`) plus `group_memberships` (`:32-38`). |
| Joint ventures, nesting, 순환출자 cycles | **CONTRADICTS a hard DB constraint** | `group_memberships` carries **`UNIQUE (org_id)`** (`0060:37`) — an org may belong to **at most one** group. Joint ventures, nesting and cycles are all excluded by that one constraint. `org-hierarchy.md:297` already flags the adjacent open question. |

### Authority

| Requirement | Verdict | Evidence |
|---|---|---|
| Roles not fixed, multiple layers | **SCOPED** | `rbac-configurable.md:126-131`; `ADR-0018:109` (group/org/department/team/position/custom-role scope). |
| Configurable from a **no-code canvas** | **SCOPED** | `ADR-0018:70` (visual no-code canvas, node palette incl. approval/decision/human task); `no-code-…-approved-plan.md:39,71`. |
| `effective(person, scope) = fold(grants)` | **SCOPED, partly BUILT** | `rbac-configurable.md:116-119` specifies exactly this fold. Built: `EffectiveFeatureGrant` resolution from `policy_roles`/`user_role_assignments` (`platform/authz/src/lib.rs:1367-1380`). |
| Sources: position / group / direct / delegation / **assignment** | **SCOPED; delegation and assignment UNBUILT** | Enumerated at `no-code-…-approved-plan.md:254`. But **no delegation table exists** anywhere in 206 migrations. Assignment exists only as `work_order_assignments.mechanic_id → users(id)` with a two-value CHECK role (`0008_create_work_orders.sql:79-86`). |
| Authority not following rank | **DRAFT-ONLY** | `authority-and-approval-model.md` §262, §299. Today rank *is* authority: two role variants alone produce org-wide scope (see C1). |
| Org policy and group policy as separate planes | **SCOPED and BUILT** | `org-hierarchy.md:241` ("a separate parallel capability set"); `group_role_grants` is a distinct plane (`0060:40-50`). **But composed with AND, not fold — see the conflict below.** |
| Deleting the compile-time `Feature` × 6-role matrix | **CONTRADICTS `rbac-configurable.md:116`** | See C5. |
| **Competence (전담)** as a third relation | **NEW** | Zero occurrences of 전담 in `docs/decisions/` or `docs/specs/`. `authority-and-approval-model.md` §436 states it plainly: Control — *"nothing exists"*; Competence — *"**nothing exists**"*. |

**An unrequested conflict worth more than most of the requested ones.** `org-hierarchy.md:246-247`
mandates that effective authority is an **AND** across planes — *"all default-deny, all must pass"* —
and `:235` that the group layer *"ADDS a marking, never REMOVES a per-법인 restriction."* A `fold(grants)`
with delegation and assignment as sources is **additive**: a delegation must be able to confer reach the
recipient's own memberships do not have. The kernel enforces the intersective reading in code:
`BranchScope::intersect` is documented *"never widens: `All` behaves as the identity"* and implemented as
a set intersection (`kernel/core/src/branch.rs:37-50`). **Additive folding and never-widening
intersection are incompatible.** This is a genuine contradiction that nobody has flagged.

### Approvals

| Requirement | Verdict | Evidence |
|---|---|---|
| 결재 routing as 전결규정 lookup, resolving above / laterally / **below** | **NEW** | Zero occurrences of 전결 in `docs/decisions/` or `docs/specs/`. Prior art exists only in the **superseded** `docs/design/oyatie-console/`. `ADR-0018:218` requires *"전자결재 with approval line, delegation, rejection, resubmission"* but says nothing about **lookup-based** or downward routing. Existing approvals are fixed step chains: `work_order_approval_steps` (`0008:59`), `org_change_approval_steps`, `gov_approval_requests`. |
| Standing from the 결재권 graph, not hierarchy | **DRAFT-ONLY** | `authority-and-approval-model.md` §299. |
| Closure truncating the line without extinguishing standing | **DRAFT-ONLY and flagged BLOCKING** | `authority-and-approval-model.md` §383: *"unresolved, and blocking … This must be decided before slice 0 is built."* |
| Signatures recording **capacity** (which grant authorised it) | **NEW** | See C3 — no capacity field on any audit or approval row. |
| Retroactive 반려 as a tracked obligation loop | **DRAFT-ONLY** | §340. The doc adds the consequence the requirement omits: *"retroactive rejection after execution is a compensating transaction, not an undo… Paid payroll cannot become unpaid."* |

### Work, quantity, economics

| Requirement | Verdict | Evidence |
|---|---|---|
| 업무 first-class, with artifacts **and actions** linked to the work | **The pattern is already BUILT for one vertical** | `evidence_media.work_order_id NOT NULL` — artifacts hang off the work, with `uploaded_by` as provenance only (`0009_create_evidence_media.sql:7,17`). `work_order_status_history` links actions to the work with an actor (`0008:93-99`). What is new is generalising this off the work-order vertical. |
| Work metrics and a work ledger | **NEW** | No work-ledger table. |
| **Structural lineage**: quantity-bearing split/merge DAG | **NEW; and the open-set mechanism cannot express it directly** | No lineage or provenance table in 206 migrations. `ont_links` carries `from_instance_id`, `to_instance_id`, `link_type_id`, validity — **and no properties** (`0155_create_ontology_instances.sql:66-78`). An ontology edge cannot carry a quantity, so every quantity-bearing edge must be **reified** as an instance. That is expressible without a developer, but it is a modelling convention that needs deciding, not a capability that exists. |
| Cost/revenue/profit as **queries** over one spine | **PARTIALLY BUILT** | `finance_gl_vouchers.source_object_type` + `source_object_id` is a polymorphic business-object reference, indexed for exactly this drill (`0160_create_finance_gl_vouchers.sql:33-35,53-54`). |
| Business object as a **dimension on GL postings** | **PARTIALLY BUILT — at the wrong granularity** | The dimension is on the voucher **header**. `finance_gl_voucher_lines` carries only `account_code`, `side`, `amount_won`, `memo` (`0160:57-70`) — **no** business-object dimension, no quantity, and `amount_won BIGINT` is KRW-integer-only. One voucher spanning two projects cannot be split per project. |

### Cross-cutting

| Requirement | Verdict | Evidence |
|---|---|---|
| Entity classes an **open set**, addable without a developer | **BUILT** | `ont_object_types`, `ont_property_defs`, `ont_link_types`, `ont_action_types` are rows (`0152_create_ontology_registry.sql:17,49,67,87`). `FieldKind::Unknown(String)` exists so unrecognised authored tags *"degrade to Unknown (forward-compatible; **never fails**)"* (`ontology/domain/src/lib.rs:331-334`). |
| Cross-cutting **concerns** an open set | **NEW** | The concerns themselves (audit, RLS, approval, policy) are all compile-time crates and gates. |
| Realtime visibility of authority and work state | **Substrate BUILT, application NEW** | ADR-0007's LISTEN/NOTIFY fan-out exists but governs the out-of-scope messenger. |

---

## Part C — The five suspicions, adjudicated

### C1. ADR-0003 — **confirmed, but your evidence is invalid and the real conflict is elsewhere**

**Your cited evidence does not hold.** `0001_create_regions_branches.sql:2-3` — *"Every operational row
(work orders, users, equipment, KPIs, chat channels) carries a non-null `branch_id`"* — is a **header
comment**, and the DDL beneath it creates no `branch_id` column at all: `regions` is `id, name,
created_at`; `branches` adds `region_id, name` (`0001:5-17`). Per your own discipline rule this cannot be
cited for behaviour.

**And the claim it makes is false in practice.** Across the migrations there are **61** `NOT NULL`
`branch_id` declarations and **21** deliberately nullable ones, several with explicit rationale —
`audit_events.branch_id` is nullable and commented *"NULL = organization-global event"*
(`0003_create_audit_events.sql:19-20`); `0053_create_comms_webmail.sql:23` says *"NULL = org-wide"*. **The
split between operational scope and org-wide scope has already happened, 21 times, without an ADR.** By
`README.md:12` that is already an unreconciled governance gap.

**The real conflict is two sentences elsewhere, and it is worse than the one you suspected.**

1. `ADR-0003:20` — *"Principals carry a `BranchScope` (kernel type): **`All` for SUPER_ADMIN/EXECUTIVE
   rollups**, an explicit branch set otherwise."* This ties scope-widening to two **named role
   variants**. The executable code does exactly that:

   ```rust
   // backend/crates/platform/authz/src/lib.rs:1478-1483
   if roles.iter().any(|role| matches!(role, Role::SuperAdmin | Role::Executive)) {
       return Ok(BranchScope::All);
   }
   ```

   This is the **only** producer of org-wide scope from a principal — every other non-test
   `BranchScope::All` reference in the workspace *consumes* the value by matching on it. Deleting the six
   roles removes the producer with no successor, and `rbac-configurable.md:29` R1 already declares this
   pattern banned.

2. `kernel/core/src/branch.rs:37-50` — `intersect` *"never widens"*. Scope composition is monotonically
   narrowing, so no grant, delegation or assignment can confer reach a principal's memberships lack.

**A latent consequence, found while verifying the above.** `authorize_org_wide`
(`authz/src/lib.rs:1151-1166`) offers two ways to pass: a built-in role permission
(`has_builtin_org_wide_permission`, `:1158-1160`) **or** a tenant custom grant carrying
`branch_scope == BranchScope::All` (`has_custom_org_wide_permission`, `:1163-1166`). But the function
opens with a hard precondition — `if principal.branch_scope != BranchScope::All { return Err(...) }`
(`:1152-1156`) — and the principal's scope can only become `All` through the role check at `:1478-1483`.
**So the custom-grant branch is unreachable for any principal that is not already SUPER_ADMIN or
EXECUTIVE.** A tenant custom role can never actually exercise org-wide access today. The doc comment at
`:1148-1150` describes the intended behaviour, but the guard above it prevents it. This is dead policy
surface, and it means the configurable-role work is *less* complete than `rbac-configurable.md:5`'s
"shipped increment" claim implies.

**What a superseding ADR must decide:** that a principal carries **two** independent scopes — an
operational scope (branch/worksite, where work happens) and a **decision scope** (competence, what a unit
may decide) — that scope may be **widened** by an explicit grant rather than only narrowed, and that
org-wide reach derives from a **capability**, never from a role variant. It must also ratify the 21
nullable `branch_id` columns that already exist. Note that `cedar-pbac-coexistence-map.json`'s
`invariants.rlsHardBoundary` already articulates the split — the ADR would be *ratifying an existing
invariant*, which makes it cheap to argue.

### C2. ADR-0001 — **refuted. There is no conflict.**

Open-set entity classes and compiler-enforced architecture operate on different objects, and the codebase
already demonstrates it.

`ADR-0001:20` enforces the dependency direction between **crates** — crate visibility plus a CI gate that
fails on illegal edges and on `sqlx`/`axum`/`tokio` appearing in domain/application crates. It says
nothing about how many *kinds of business object* the system may hold.

The ontology engine is itself a conforming crate family — `ontology/{domain,application,adapter-postgres,rest}`
— and the entity classes it interprets are **rows**: `ont_object_types`, `ont_property_defs`,
`ont_link_types`, `ont_action_types` (`0152:17,49,67,87`). Adding an object type is an INSERT. The engine
is even explicitly built for authored types it does not know: `FieldKind::Unknown(String)` carries the raw
tag so an unrecognised type *"never fails"* (`ontology/domain/src/lib.rs:331-334`).

**The boundary, stated precisely:** the compiler enforces the **layering of the interpreter** and the
**shape of a cross-cutting concern** (audit, RLS, policy evaluation). Authored data supplies **which
object types, properties, links and actions exist**. Today's requirements cross that boundary in exactly
one place — *"cross-cutting concerns are an open set"* — because concerns are crates and gates, not rows.
That sub-requirement is the only part of the open-set ask that touches ADR-0001, and it is the one I would
push back on.

### C3. ADR-0002 — **the specific claim is refuted; the underlying problem is real and differently located**

**ADR-0002 mandates no field shape whatsoever.** Its Decision (`ADR-0002:20`) specifies only the
transaction sequence, `with_audit`, append-only enforcement, and the audit-coverage gate. There is no
event schema in the ADR. **Therefore no superseding ADR is required to add a `capacity` column**, and the
claim that a missing field is blocked by an ADR is wrong.

**The "single missing field" arithmetic is also wrong — three are missing.** The actual shape
(`kernel/core/src/audit.rs:83-109`) carries: `actor`, `action`, `target_type`, `target_id`, `branch_id`,
`org_id`, `before`, `after`, `request_context` (ip/user_agent/auth_method/device),
`classification` (badges/anomaly/reason), `trace`, `occurred_at`.

| Field asked about | Present? |
|---|---|
| actor | **yes** (`audit.rs:86`) |
| scope | **yes** — `branch_id` + `org_id` (`:93,98`) |
| timestamp | **yes** (`:108`) |
| capacity / authorising grant | **no** |
| quantity | **no** |
| amount | **no** |

**And adding the column is trivially precedented.** `0149_add_audit_context_classification.sql:6-14`
added **seven** nullable columns to `audit_events` — with the rationale *"All columns are nullable for
backward compatibility with existing append-only rows"* — and no ADR was written for it.

**The real obstacle is the one your brief does not mention: the tamper-evidence hash.**
`ADR-0002:17` makes tamper-evidence a mandated quality attribute (*"audit records must be
tamper-evident"*), and it is implemented by a seal chain over a **frozen** column list:

```
// backend/crates/platform/audit-chain/src/lib.rs:473-474
const SELECT_BATCH_COLUMNS: &str = "id, actor, action, target_type, target_id, branch_id, \
     org_id, before_snap, after_snap, trace_id, span_id, occurred_at, created_at";
```

`row_hash` hashes exactly those thirteen fields (`audit-chain/src/lib.rs:363-379`) — which means **all
seven columns added by 0149 sit outside the tamper-evident chain.** A `capacity` field added the same
cheap way would be append-only (UPDATE/DELETE are trigger-blocked, `0003:38-53`) but **not
hash-protected** — and capacity is precisely a field whose integrity matters, since it is the record of
*what authorised an act*. Bringing it inside the hash changes `row_hash`, which invalidates every
existing seal and requires a `row_hash` v2 with an epoch boundary or a re-seal.

**Verdict:** the claim that one missing field blocks four requirements, and that this is the highest-leverage
*schema* decision, is **not supported**. The schema change is a routine additive migration with a direct
precedent. The genuine decision — and it *is* ADR-worthy, as an **amendment** to ADR-0002 rather than a
supersession — is whether capacity enters the sealed preimage, and how the chain is versioned if it does.

### C4. ADR-0004 — **confirmed: binding assumes an org-scoped user. Amendment, not reversal.**

`auth_webauthn_credentials.user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE`
(`0004_create_auth.sql:6`), and `users.org_id` is `NOT NULL` (`0029_enforce_org_id.sql:29-33`). Refresh
token families are bound the same way (`0004:34,46`). **A passkey therefore binds to an (org, user)
pair.** One human at three companies is three `users` rows, three user handles, three separate
enrollments of the same physical authenticator, and no way to authenticate once and then choose an
employer.

**With a `party` above `users`, the credential should bind to the `party`** — one stable WebAuthn user
handle per human — and org selection becomes a post-authentication authorisation step over the party's
edges rather than a separate credential.

**Does ADR-0004 need to change?** Its Decision (`:20`) never states what a credential binds to; it
specifies ceremonies and token mechanics only. So strictly it is not contradicted — this is an
**amendment** recording that the credential subject is the party, plus a revised Consequences note
(`:24` currently reasons about per-user enrollment logistics, which per-party binding materially
improves). The **binding constraint lives in ADR-0022, not ADR-0004**: `ADR-0022:38` forbids HR,
attendance or payroll integrations from authenticating users, asserting sessions or granting roles. A
`party` sourced from HR data would collide with that sentence, so the superseding decision must state
that `party` is **platform-owned identity**, not an HR-sourced record.

### C5. Specs pinning the `Feature` enum or six roles — **confirmed, in one place, and it is narrower than feared**

The matrix is real: `Role::ALL: [Self; 6]` (`platform/authz/src/lib.rs:55`), `Feature::ALL: [Self; 96]`
(`:372`), and `const fn matrix_row(self) -> [PermissionLevel; 6]` over four levels
(Allow/Deny/Limited/RequestOnly) with column order
`[MEMBER, RECEPTIONIST, MECHANIC, ADMIN, EXECUTIVE, SUPER_ADMIN]` (`:573-576`). Blast radius: **592**
`Feature::` and **380** `Role::` references across **65** files.

The one spec sentence that deletion breaks is `rbac-configurable.md:116` — *"The built-in 6 remain as
**immutable bootstrap system roles**"* — reinforced by `:96-97`, which pins both
`permission_for(role, feature)` as *"the immutable system-role floor"* and the **JWT claim shape**
(`AccessClaims.roles → Role::from_str`). Deleting the matrix therefore breaks a wire contract, not just
an internal type.

Three things make this much cheaper than it looks:

- The same spec already commits to removing the *dangerous* half: `:29` R1 bans role-string
  authorization at 18 named sites and requires a CI gate failing on `matches!(*, Role::…)`.
- `ADR-0021:46` already demotes roles to *"subject inputs/policy bundle generators, **not** authoritative
  allow decisions by themselves."*
- The matrix already has data-driven escape hatches: seven `Feature` variants are documented
  *"Custom-grant only: no built-in role fallback"* (`authz/src/lib.rs:170-190`), and
  `custom_role_runtime_feature_allowed` gates runtime custom grants (`:1373`).

**But there is no live successor.** `cedar-pbac-coexistence-map.json` is
`status: "design_no_live_switch"` with both entries `legacy_only`, and `README.md:52` records ADR-0021 as
*"accepted target only … no live enforcement switch."* Deleting the matrix today removes the **only
enforcing engine**. The sequencing is forced: enrol and promote actions in Cedar first, then delete.

---

## Part D3 — Superseding ADRs required

Ordered by what blocks the most. Each names the decision it reverses and why a plan paragraph cannot do
the job (`README.md:8,10`).

**ADR-0027 — Party identity above org-scoped users.** *Amends ADR-0004, ADR-0022; supersedes part of
ADR-0003's substrate.* Must decide: a platform-level `party` is the subject of a WebAuthn credential
(reversing the de-facto `credential → (org, user)` binding at `0004:6` + `0029:29-33`); tenant
participation is an **edge** from party to org, not a column on the row; and `party` is
platform-owned identity, **not** HR-sourced — required by `ADR-0022:38`. **This is the keystone**: four
other requirements (employee/position separation, one human at several companies, cross-tenant group
grants, group-level 결재 lines) are inexpressible until a person has an identity.

**ADR-0028 — Operational scope and decision scope are separate.** *Supersedes ADR-0003.* Must reverse
`ADR-0003:20`'s *"`All` for SUPER_ADMIN/EXECUTIVE rollups"* — org-wide reach must derive from a
capability, not a role variant (the code at `authz/src/lib.rs:1478-1483` is the only producer today) —
and must reverse the never-widening composition at `kernel/core/src/branch.rs:37-50` so that a grant may
widen reach. Must also ratify the 21 already-nullable `branch_id` columns, and reconcile the additive
`fold(grants)` model with the AND-composition mandated at `org-hierarchy.md:246-247`. **Cheap to argue**:
`cedar-pbac-coexistence-map.json`'s `rlsHardBoundary` invariant already states the split.

**ADR-0029 — Competence as a third relation.** *New; no ADR to reverse.* Control (법인 간 지배),
Structure (조직), and Competence (전담) as three independent relations; 전결규정 as authored data mapping
(category × amount band × scope) → competent unit, resolvable **above, laterally or below** the raising
unit. Needed because nothing exists: 전담 appears in no ADR or spec, and existing approvals are fixed step
chains (`0008:59`).

**ADR-0030 — Group designation is derived from control edges.** *Supersedes part of ADR-0003's group
model and amends `org-hierarchy.md:163`.* Must reverse the **`UNIQUE (org_id)`** constraint at
`0060_create_groups_and_membership.sql:37`, which permits at most one group per org and thereby excludes
joint ventures, nesting and 순환출자 outright. Must also decide `org-hierarchy.md:297`'s already-open
question (*"법인 must NOT self-join a group"*), which under derived designation becomes a cycle-policy
question, not a validation rule.

**ADR-0031 — Delete the compile-time role matrix; Cedar becomes the enforcing engine.** *Amends
ADR-0021; supersedes `rbac-configurable.md:116`.* Must reverse *"The built-in 6 remain as immutable
bootstrap system roles"* and re-specify the JWT claim shape pinned at `rbac-configurable.md:96-97`.
**Must be sequenced after** Cedar enrolment: today `cedar-pbac-coexistence-map.json` is
`design_no_live_switch` with both entries `legacy_only`, so deletion would leave no enforcing engine.

**Amendment to ADR-0002 — capacity in the audit preimage.** *Not a supersession.* ADR-0002 mandates no
field shape, so the column itself needs no decision (precedent: `0149:6-14` added seven columns with no
ADR). The decision needed is narrow and real: does `capacity` enter the sealed preimage at
`audit-chain/src/lib.rs:473-474` and `363-379`, and if so how is `row_hash` versioned without
invalidating existing seals? ADR-0002's tamper-evidence mandate (`:17`) is what makes this ADR-worthy.

**Amendment to ADR-0002 or a new ADR — quantity-bearing edges.** `ont_links` carries no properties
(`0155:66-78`), so the split/merge DAG must either reify allocations as instances (possible today, no
developer needed) or gain link properties (an engine change). Pick one explicitly; the reification
convention is otherwise invented per-team.

**Reconciliation ADR — the pivot.** Per `README.md:12` and `PIVOT:105`, one decision recording that
ADR-0009 and ADR-0012 have lost their subject (`clients/`, `web/`, `ios/`, `android/` deleted), that
ADR-0023 and ADR-0025 point at a deleted path, and that ADR-0005/0006/0007/0014/0019/0020 govern
out-of-scope surfaces. Also required: resolve `PIVOT:68` ("cargo, not buck2") against
`LANE-PROTOCOL.md:264` ("buck2 is retained") and `foundation-gates.md:15` (a Buck2-only gate contract).
**I am not guessing which wins — this needs a human decision.**

---

## Part D4 — Documents that describe a state no longer true

The recurring failure is real and much larger than four instances. Recent commits confirm the pattern:
#523 *"the job is required now, so stop saying it is not"*, #519 *"the suite's own doc described an engine
that no longer exists"*, #499 *"land the rename remnants #497 merged without"*, #504 *"drop the
openapi_drift helper orphaned by the client-test removal"*.

**Highest priority — live and load-bearing:**

| Document | Stale claim | Why it matters now |
|---|---|---|
| `docs/program/LANE-PROTOCOL.md:264` | *"buck2 is retained and is the parallel-build path"* | Contradicts `PIVOT:68-74`, and was committed **20 h after** the pivot (`451591ee8`, #517). This is the protocol an agent is following in a lane **right now**. |
| `docs/specs/foundation-gates.md:15` | A **"Buck2-only"** evidence policy as a durable contract verified by gate **G002** | A CI gate contract pins the build system the pivot says was replaced. |
| `docs/PIVOT-2026-07-28.md:82` | Ontology engine "17,825 LOC" | Actual: 8,221 non-test / 27,309 total. The canonical truth document has a wrong number in its own verification table. |
| `docs/MISTAKES-LEDGER.md` | *"Every mistake gets a row"* (`:3`) — yet the last row is **MFL-0004, 2026-06-12** | The ledger promises to record the recurring failure and records none of it. The prevention mechanism is itself stale. |
| `docs/decisions/` (whole directory) | No ADR references the pivot; only ADR-0017 has a `superseded_by` | `PIVOT:105` prescribed exactly this remedy and it was never applied. |

**Accepted ADRs whose subject was deleted or descoped** (governance gaps per `README.md:12`):
ADR-0009 (`:20` client-drift gates — `PIVOT:22` says they no longer exist), ADR-0012 (`:20` names
`web/`, `ios/`, `android/` — all absent), ADR-0023 and ADR-0025 (`:51` roots the console at
`web/src/console/` — absent), and ADR-0005/0006/0007/0014/0019/0020 (evidence-WORM, dispatch, comms,
location, mailbox, connectivity — all explicitly out per `PIVOT:59-66`).

**Mechanically detected, in the document classes `PIVOT:107` says *must* be updated** (`docs/specs/`,
`docs/program/`, `docs/reference/`, `docs/runbooks/`, root `docs/*.md`) — evidence and retros correctly
excluded, since `PIVOT:106` says do not edit those:

- **45 documents** reference React / `web/src` / Playwright. Notably `docs/specs/knl-business-os.md`, `no-code-org-ops-editor-approved-plan.md`, `roadmap-to-production.md`, `master-parallel-build-plan.md`, `issue-55-approval-command-center.md`, `review-fix-merge-governance.md`, `data-exchange-import-export.md`; `docs/program/console-enterprise-roadmap.md`, `console-program-ledger.md`, `parity-matrix.md`, `ontology-coverage-matrix.md`, the entire `benchmark-matrix/` tree (18 files), `console-capability-registry.json`; and root `docs/parity-checklist.md`, `CI-GATES.md`, `i18n.md`, `DESIGN-DOCTRINE.md`, `web-console-overhaul-spec.md`.
- **12 documents** reference buck2, including `docs/program/console-buck2-scale-playbook.md`, which is wholly about a build system the pivot says was replaced.
- `docs/i18n.md` describes cross-platform string parity — `PIVOT:23` states `check-i18n` no longer exists.

---

## Part D5 — Already scoped but unbuilt (claim credit, do not re-decide)

1. **`effective(person, scope) = fold(grants)`** — specified verbatim at `rbac-configurable.md:116-119`, and partly built (`authz/src/lib.rs:1367-1380`).
2. **Removing role-string authorization** — `rbac-configurable.md:29` R1 already binds it, names all 18 sites, and specifies the CI gate. Do not re-derive this list.
3. **Roles at multiple layers** — `rbac-configurable.md:126-131` and `ADR-0018:109` already enumerate job function / department / position / responsibility / scope / context attributes.
4. **The no-code canvas** — `ADR-0018:70` (accepted ADR) and `no-code-…-approved-plan.md:39,71`, including cycle checks on the org tree.
5. **All five grant sources** — `no-code-…-approved-plan.md:254` already lists group grants, delegations and active assignments as subject inputs.
6. **Org policy vs group policy as separate planes** — `org-hierarchy.md:241`, built as `group_role_grants` (`0060:40`). Only the *composition rule* needs changing.
7. **Splitting operational from decision scope** — already an invariant: `cedar-pbac-coexistence-map.json` `invariants.rlsHardBoundary`.
8. **Company / org_unit / job_position / employment / pay_run as authored ontology types** — **built and conformance-verified**: `ontology/rest/tests/company_conformance/fixtures/{company,org_unit,job_position,employment,pay_run}.rs`, reaching 12/12 as of #516. Employee/position separation is therefore *already built*, minus the person.
9. **Entity classes as an open set** — built (`0152:17,49,67,87`; `FieldKind::Unknown` at `ontology/domain/src/lib.rs:331-334`).
10. **Artifacts and actions linked to the work, not the person** — the pattern is built for work orders (`0009:7,17`; `0008:93-99`); generalising it is the new part.
11. **Business object as a GL dimension** — built at voucher-header granularity with the drill index (`0160:33-35,53-54`). Only line-level granularity is missing.
12. **Grant shape** — `clearance_assignments` already has status, `starts_at`/`expires_at`, `granted_by`/`revoked_by` and a mandatory `grant_reason` (`0147:14`) — close to the right shape for a grant row.
13. **Adding audit columns** — precedent set by `0149:6-14`; no ADR needed for the column itself.

---

## Part D6 — Open questions

1. **buck2 or cargo?** `PIVOT:68` and `LANE-PROTOCOL.md:264` disagree, the later commit favours buck2, and `foundation-gates.md:15` pins a Buck2-only gate contract. Blocks any honest build-system statement. **Needs a human decision.**
2. **What makes a 결재 closure final?** `authority-and-approval-model.md` §383 marks this *unresolved and blocking* before slice 0. Candidates: an 이의기간 window, finality on downstream execution, or an explicit 확정 step.
3. **Does `capacity` enter the sealed audit preimage?** If yes, how is `row_hash` versioned without invalidating existing seals (`audit-chain/src/lib.rs:363-379`)?
4. **Additive fold or intersective AND?** `org-hierarchy.md:246-247` and `branch.rs:37-50` mandate intersection; `fold(grants)` implies union. Which wins, and does a delegation ever grant reach the delegate lacks?
5. **Quantity-bearing edges: reify or extend?** `ont_links` has no properties (`0155:66-78`).
6. **Is `party` platform-owned or HR-derived?** `ADR-0022:38` forbids HR data granting roles or asserting sessions, which constrains the answer.
7. **Cross-cutting concerns as an open set** — the only sub-requirement that genuinely touches ADR-0001. Concerns are crates and CI gates, not rows. What would "adding a concern without a developer" mean concretely?
8. **What replaces `AccessClaims.roles` in the JWT?** (`rbac-configurable.md:96-97`) — a wire contract, so it needs a migration story.
9. **Which of the 45 stale documents get updated vs marked superseded?** `PIVOT:107` says specs/program must be updated; at 45 documents that is a work item, not a cleanup.
10. **Two disagreeing `employment_type` vocabularies** (`0172:7` vs `0187:22`) — which is canonical during migration to authored types?
11. **Is the unreachable custom org-wide grant path (`authz/src/lib.rs:1152` vs `:1163-1166`) a bug to fix now, or does it disappear with ADR-0028?** Fixing it early would let tenant custom roles hold org-wide reach before the matrix is deleted, decoupling two otherwise-sequenced changes.
