# Authority and approval — expressing a company's own authority in its own engine

> **SUPERSEDED — 2026-07-29 by `docs/ideas/ecosystem-plan-DRAFT.md`.** Retained as the requirements
> record and as a lesson; **not current intent, and not safe to build from.**
>
> This document was authored in one session by one hand. Its successor's §0 finds **nine further
> errors** in it beyond the four it retracts itself, one marked BLOCKING:
>
> - **§0.1 — a live internal contradiction.** `:89-92` below retracts group-scoped people ("the group
>   is not high enough"), then `:545-546` recommends exactly that, and `:575-579` sizes
>   `app.current_group` across 141 RLS tables as "the largest single engineering cost". That cost is
>   incurred **only by the retracted option**; the conclusion this document actually reaches — party at
>   the platform, tenant owns the edge (`:83-87`) — needs **no second tenancy dimension at all**.
> - **§0.2** — `Feature` cannot be deleted; it is Cedar's action vocabulary (`cedar_pbac/engine.rs:430`)
>   with ~500 call sites. Only `Role` and `matrix_row` are scaffolding.
> - **§0.3** — Cedar's `parents` hierarchy is **unimplemented** here (`engine.rs:392`, `:425`, `:449`),
>   so "the corporate graph becomes the Cedar hierarchy" is a property of the library, not this engine.
> - **§0.4** — `users.id` is the PK; `(id, org_id)` is an added UNIQUE. So `group_role_grants.user_id`
>   is ordinary DDL, **not** a cross-tenant carve-out earned by design, as claimed below.
> - **§0.5** — `org_unit.parent_org_unit_id` exists only in a conformance **test fixture**; org
>   structure is greenfield in production.
> - **§0.6** — `notice_receipts` carries the composite FK `(recipient_user_id, org_id)`, so it has the
>   same cross-company blocker this document rejects `gov_approvals` for — missed while asserting the
>   gap was "narrower than it first appears".
>
> The durable lesson is the one this repo keeps relearning: **a retraction that is not propagated
> through the recommendation and the cost model is not a retraction.** Four claims were retracted in
> place below and the sections downstream of them were left standing.

> idea-refine output, 2026-07-29. Owner requirements plus refined direction.
> Every "what exists" claim cites executable code or DDL, never a header comment.
>
> **Supersedes a deferral.** `docs/ideas/no-code-ontology.md` §232 lists *"Not building
> ObjectType-as-a-Cedar-resource, schema-mutation verbs, or a per-type ownership"* as Not Doing, and
> §81-87 records why: `Feature` is a closed compile-time enum. The requirement that roles be
> **configurable from a no-code canvas** makes that deferral untenable.
>
> **Product framing, corrected by the owner.** This is not field-service software. It is an omni
> platform for business operation, with **HR + payroll as the first vertical slice**, over infra that
> expresses policy, organization and the other building blocks of a company **as if it were a game
> engine**. The `Feature` × 6-role matrix
> (`[MEMBER, RECEPTIONIST, MECHANIC, ADMIN, EXECUTIVE, SUPER_ADMIN]`,
> `platform/authz/src/lib.rs:109`, `:573`) is **demo scaffolding to delete**, not legacy to bound.

## Problem Statement

**How might we** let a real organization author its own authority — who may do what, in which company,
at which layer — from a canvas, and route a fluid 결재 line over it, with none of it compiled?

## The engine already is the model

| Game engine | Ontology, as shipped |
|---|---|
| archetype / prefab | object type |
| entity | instance |
| component | typed property, link with cardinality |
| system | action with dispatch (`ActionDispatch::{ProjectedUsecase, InstanceRevision}`) |
| deterministic replay | effective-dated fixity-chained revisions; **state = fold** |

The owner's chosen authority model is `effective(person, scope) = fold(grants)`. That is the *same
computation the engine already performs*. Authority is therefore not a new subsystem — it is another
archetype, and 결재 is a system over it.

This is also the thesis test. If a company-expressing engine cannot express its own authorization, the
claim is false. **Authority is the dogfood.**

## Requirements, as stated

1. An employee and the position they occupy are **separate entities**.
2. A company can belong to a **group**.
3. **Authority does not follow rank.** An associate at company A may out-rank an executive at company B.
4. **Organizational policy and group policy are separate planes.**
5. Roles are **not fixed** and exist at **different layers**.
6. Roles are **configurable through a no-code canvas**.
7. **전자결재**, with a **highly fluid** 결재 graph.

## What exists, by layer

| Layer | Mechanism | Fixed or data | Scope | Citation |
|---|---|---|---|---|
| Compile-time RBAC | `Feature` × 6 roles, `const fn matrix_row` | **fixed — delete** | none | `authz/src/lib.rs:109`, `:573` |
| Org roles | `policy_roles` + `user_role_assignments` | data | `org_id` | `0065:141` |
| Group roles | `group_role_grants` | data, 3 CHECK literals | **cross-tenant** | `0060:40` |
| Clearance | `clearance_assignments` | data, dotted keys, time-bounded | `org_id` | `0147:14` |
| Cedar PBAC | `cedar_policy_catalog_entries`, `ont_object_policies` | data | org + type | `0170`, `0205` |

Requirements 4 and 5 are **already designed for**. `group_role_grants` carries the gate rationale
*"cross-tenant group-role authorization; runtime resolves own grants only"* and alone among its
neighbours references `users(id)` **without `org_id`**. `clearance_assignments` already has status,
`starts_at`/`expires_at`, `granted_by`/`revoked_by` and a mandatory `grant_reason` — roughly the right
shape for a grant.

## Where employees belong — reasoned from the objects, not the schema

**"Employee" is not an entity.** It is a role a person plays relative to a legal entity — the
employment relationship seen from the company's side. Every existing attempt to make it an entity
loses the human: `employees` is keyed `(org_id, source_key)`, `users` is keyed `(id, org_id)`, and the
ontology carries `person_name` as a string. Three attempts, three failures, one cause.

The real objects:

| Object | Belongs to | Lifetime |
|---|---|---|
| **natural person / party** | nobody — the platform | independent of every company |
| **legal entity (법인)** | the platform | independent |
| **employment** | the **company** (an edge) | a period, with terms |
| **position** | the company | exists unoccupied |
| **assignment** | the company (an edge) | a period |

**The person belongs to the platform; the tenant owns the edge.** This is also the correct solution to
confidentiality — company A must not learn that its employee also works at company B. That is achieved
by scoping the **relationships**, not the person row: a company sees only parties it holds an edge to.
RLS belongs on the edge, which is where the tenant-owned fact actually lives. The engine already makes
links first-class (`LinkCardinality::{OneOne, OneMany, ManyMany}`).

**This revises the earlier "group is the tenancy boundary for people" answer.** The group is not high
enough. A person can work for companies in *different* groups — a contractor, a director on two
unrelated boards, anyone moving from group A to group B. Group-scoping relocates the duplication rather
than removing it, and cannot represent a person before they are grouped.

**Name it `party`, not `employee`.** An omni platform needs one identity for customers, suppliers,
contractors, directors and government contacts. Named `employee`, the sales vertical duplicates it and
procurement duplicates it again. `employment` is one relationship kind; later verticals add
`customer_of`, `supplier_to`, `director_of` against the same party.

## How to implement group / corporate

Real corporate structure does not fit a flat set. Control derives from shareholding and is not binary;
joint ventures place one company under two groups; groups nest (holding → sub-holding → operating);
**순환출자** cross-shareholding forms genuine cycles; and KFTC 기업집단 designation is an
administrative fact that does not perfectly track ownership arithmetic.

Today's model forbids all of it, and stores the fact twice:

- `groups` (`0060:13`) is flat — `id`, `slug`, `name`, `status`, **no parent**
- `organizations.group_id` is a **single-valued nullable FK** (`0060:26`)
- `group_memberships` is **`UNIQUE (org_id)`** (`0060:36`)
- **Two sources of truth for org→group.** Defect independent of this design.

Nothing about ownership, shareholding, control, subsidiaries or legal entities exists anywhere in the
141-migration tree, so this is greenfield — no migration burden.

**Store control edges; derive the group.** `(holder, held, share, control_basis, period)` expresses
joint ventures, nesting and cycles because it is a graph rather than a set. Because closure per
authorization decision is expensive and legal designation is administrative regardless: **control edges
are truth, group designation is a derived, effective-dated, reasoned fact** — precisely the revision
model the engine already implements. Recompute on edge change; keep the reason.

**Authority scope is therefore not "the group."** Authority may cover a set that is no legal group at
all — a region, all manufacturing subsidiaries. A grant binds to a **scope expression** over the
corporate graph: this entity only, this entity and its control-descendants, or a named set. Cedar
expresses this natively through entity `parents`, so the corporate graph *becomes* the Cedar hierarchy.

Requirement 3 then needs no override mechanism:

```
person P:  grant(capability: narrow, scope: company A)      ← "associate at A"
person P:  grant(capability: broad,  scope: group G ⊇ B)    ← out-ranks at B
person Q:  grant(capability: mid,    scope: company B)      ← "executive at B"

effective(P, B) inherits from G  >  effective(Q, B)
```

Rank appears nowhere. It falls out of where the grant attaches.

## 업무 (work) is an entity — and that is what makes handover possible

**The RPG framing is exact.** Work is a **quest**; its related emails, documents and contacts are the
quest's **journal and inventory**; assignment is an **edge** from work to character.

The decisive consequence: if artifacts attach to the **person**, handover (인계) is a copy or migration
— lossy, and leaky. If artifacts attach to the **work**, handover is re-pointing one edge and the
artifacts follow automatically, because they were never the person's. This is Palantir's model: the
document links to the case, not to the analyst.

**Therefore assignment is the fifth grant source.** `effective(person, scope) = fold(grants)` draws
from position, group, direct, delegation **and assignment**. Assignment-derived authority expires by
construction and is least-privilege by construction: authority over what is assigned, for as long as
it is assigned.

| Event | Semantics |
|---|---|
| **연차** (leave) | time-boxed **대리** assignment; original retains ownership; reverts automatically on expiry |
| **퇴사** (departure) | permanent transfer, original's grants revoked, plus an **인계 완료** completeness check that nothing was left behind |

**분배 and 대리 are different and must not collapse.** 분배 (distribution) splits work across several
people — ManyMany assignment, or decomposition into sub-work each assigned. 대리 (proxy) means acting
*on behalf of* another, so the authority's source is the principal, not the actor. Audit must record
**both parties** — "A approved as proxy for B" — which is structurally identical to 대결 in a 결재 line.

**Prior art exists six ways, none of them general.** `docs_equipment_handover_custody` (`0184:36`),
`attendance_substitutions` (`0188:57`), `equipment_substitutions` (`0014:5`), `work_order_assignments`
(`0008:79`), `user_role_assignments`, `clearance_assignments`. Each solves handover or substitution for
one domain. A single 업무 entity with typed relationships subsumes all six — which is the strongest
available argument for modelling it once.

### Inbound email must link to work deterministically or manually

Email is **first-party and self-hosted**, not an integration: `email_accounts`, `email_folders`,
`email_threads`, `email_messages`, `email_attachments` (`0053_create_comms_webmail.sql`), over
`mailbox_domains` and `mailboxes` (`0082_create_mailbox_server_spine.sql`). So linking at ingestion is
fully under our control, and a threading concept already exists.

**The program forbids AI/LLM judgment in the product — deterministic or manual only.** So a classifier
is not available. Deterministic routes: threading headers (`In-Reply-To` / `References`) onto an
existing linked thread; a per-work reply address (`work-1234@`, feasible because the mail server is
ours); sender → party resolution against the party graph. Everything not resolved deterministically
goes to a **manual triage queue** — a human links it, rather than the system guessing.

Once linked, the artifact belongs to the work, so every later handover carries it with no further work.
Because `email_threads` already exists, linking at thread level carries the **entire prior history**,
not just the message that triggered the link.

### The work entity is the join point for provenance

The payoff is that discovery, handover and analysis become **one traversal**. What a person did while
holding a work — not only the documents but the **actions** — links to the work: audit events under
ADR-0002's audit-first discipline, `cedar_decision_log` (`0159`), action executions, and the
effective-dated revision chain. All of it already exists; the work entity is what joins it.

**This makes 인계 완료 computable rather than attested.** Today handover completeness would be a human
checkbox. As a graph it is a query: does the departing person still hold artifacts linked to no work?
Are there works assigned to them with no successor? Omissions become calculable instead of claimed —
which is also exactly the evidence a 결재 audit needs.

**Two hazards this introduces, both real:**

1. **Work-linked and person-linked artifacts must be distinguished.** If handover moves "everything that
   person did", a departing employee's personal correspondence must not move, and the successor gains
   read access to historical material they were never party to. For HR and payroll work that material
   contains compensation, disciplinary and health (병가) information. Under PIPA a change of handler
   needs a basis. So handover transfers **work-linked artifacts only**, and the boundary must be a
   modelled distinction rather than a convention.
2. **Linking is itself an authorization event.** Linking an email to a work grants every assignee of
   that work read access to it, and a retroactive link grants retroactive access. So "who may link" is
   the same class of question as "who may grant authority", and may warrant the same four-eyes
   treatment. A triage queue that any user can resolve is a privilege-granting surface.

## Permanent 부서 and temporary 사업장 are different kinds

`org_unit` today has only `parent_org_unit_id` — one hierarchy, no kind, no lifetime. `branches`
(`0001_create_regions_branches.sql:11`) is `id`, `region_id`, `name` and nothing else: no kind, no
validity period, no legal attributes. Yet `0166_leave_exact_charge_and_home_branch.sql` already routes
leave charging through a `home_branch`, so it is load-bearing while under-specified.

An org unit needs a **kind** (부서 / 팀 / TF / 사업장), a **lifetime** (permanent, or bounded with an
end date for a project site), and — for 사업장 specifically — **legal attributes**. 사업장 is not merely
a temporary department: in Korea it is the unit at which 4대보험 is registered and may carry its own
사업자등록번호, so payroll, 연차 accrual and 퇴직금 all depend on it. Modelling it as a naming variant of
부서 would put the first vertical's payroll correctness on a field that cannot carry it.

## 고용형태 — two conflicting closed vocabularies already exist

| Location | Values |
|---|---|
| `0172_create_employee_employment_profiles.sql:7` | `REGULAR, CONTRACT, PART_TIME, INTERN` |
| `0187_create_recruiting.sql:22` | `REGULAR, RESIDENT_SHIFT, PART_TIME, POOL_DAILY` |

Two closed vocabularies for one concept, both `CHECK` constraints — the same non-configurable disease
as the `Feature` matrix — and **neither includes 파견, 도급/용역, 일용직 or 프리랜서**.

**파견 breaks the employment model outright.** The employer (파견업체) and the worksite
(사용사업주) are *different legal entities*, and direction is exercised by the latter. So employment
must separate **employer** from **worksite**, and this is a second, independent reason the party cannot
be company-scoped. 겸직 (concurrent posts) likewise requires ManyMany assignment rather than one
position per person.

Employment type must therefore be **authored data with per-type rules** (accrual, insurance,
severance), not an enum — which is the same conclusion the authority model reached, reached again from
payroll.

## 전자결재 — the line is resolved by competence, not by rank

A 기안서 may close at **corporate (company) level**, or require **escalation to group**. Within the
group, the 결재 graph varies by the category and type of 결재 required.

**Routing is not climbing a hierarchy.** An earlier draft of this document claimed the line was a path
up the control graph and that escalation meant ascending it. That is wrong, and the counterexample is
ordinary: a 연락사무소 or 지사 given **전담** (exclusive charge) of a region or a work category holds
**terminal** authority for it and does not escalate to 본사 at all. A 사업장 with an office may handle
most work locally; another may handle only simple matters locally and route everything else to the
지사 or 본사 office. The determinant is **competence**, not level, so a resolved competent unit may sit
above, **laterally**, or **below** the raising unit.

A second earlier claim — *"one hierarchy, three readers"* — is likewise false and is retracted.
Authority scope reads the **control** graph; 결재 routing reads the **competence** relation. They are
different graphs.

### 소속, 직급/직책, 직무 and 결재선 are independent dimensions

They are not unrelated, but none implies the others, and no uniform superior/subordinate relation holds
across them. Korean practice already separates 직급 (grade) from 직책 (the post actually held); 직무
(job function) is a third axis; 소속 (affiliation) a fourth.

| Dimension | What it is | Determines authority? |
|---|---|---|
| **소속** | which org unit one belongs to | no |
| **직급 / 직책** | grade; the post held | no |
| **직무** | job function — HR, finance, engineering | no |
| **결재선** | who signs what | this is the **result** |

**These four are the vocabulary for writing grant rules; they imply no authority by themselves.** A rule
may say *"job function = HR and grade ≥ 부장, within group G, gets capability C"* — authored data, whose
predicates are these dimensions. Authority remains an explicit grant. An earlier draft of this document
listed "position" as a grant source, which conflated 직급, 직책 and 직무; that is corrected here.

**Inter-corporate control is a documentary fact, not an authority ordering.** A third earlier claim —
that authority scope simply *reads* the control graph — was also too strong and is retracted. The
control graph defines **which scopes exist**; it does not decide who holds authority within them. For
operational efficiency an HR officer whose 소속 is a subsidiary may hold group-wide HR competence:

```
grant(person P, capability: hr.*, scope: group G)
  where P.소속 = subsidiary A        ← subordinate on paper
  → P is competent for group HR
```

A 결재 line for such a matter then routes **to P's unit** — downward through the control graph. This is
the *same* phenomenon as a 전담 연락사무소 closing a matter 본사 never sees. Two independently stated
requirements covered by one mechanism, which is evidence the model is shaped correctly.

**So none of the four relations is derived from any other.** The control graph supplies the scope
vocabulary; competence (전결규정) supplies decision categories; grants bind persons to scopes; structure
records affiliation. Independent, and each authored.

### Standing comes from the 결재권 graph, not from hierarchy

Delegation does not strip the delegating side — but an earlier draft of this document derived that
retention from **scope descent**, reasoning that a 본사 grant at company scope already covers a
연락사무소's region scope. **That is wrong and is retracted.** The 결재권 graph decides whether 본사 has
standing over a matter. It is not implied by being higher in the corporate or scope hierarchy, and a unit
absent from the graph for a matter may have no standing at all.

The decisive case. In a line **A - B - C - D**:

- when conditions are met, **C holds 종결권** and may either close or escalate — closing authority is
  **conditional**, not positional
- if C closes, **D never sees the matter** — closure **truncates** the line
- yet **B may retroactively 반려 or 거절**, and B is *earlier and lower* than C

The last point is what breaks a hierarchical reading. B has already signed and sits below C, and still
holds power afterwards. So:

| Concept | Nature |
|---|---|
| **membership of the line** | the source of standing — not rank, not scope containment |
| **종결권** | conditional, conferred on a specific node |
| **closure** | truncates the line; does **not** extinguish other members' standing |
| **retroactive 반려 / 거절** | first-class, independent of order and of rank |

Two layers still must not be conflated, and this part stands:

| Layer | Answers | Nature |
|---|---|---|
| **전결규정 (routing)** | where a matter normally goes | a **default**, not a restriction |
| **결재권 graph** | who holds standing over this matter | standing, retained after closure |

**Routing must not be implemented as a capability restriction, and standing must not be implemented as
scope descent.** Both are the operative content of this correction. "Terminal" therefore means *the line
completes here*, never *authority ends here* — but which parties retain authority is read off the graph,
not inferred.

Audit must still distinguish **"outside normal routing"** from **"unauthorised."** A node acting
non-routinely but with graph-conferred standing is lawful; collapsing the two either makes legitimate acts
look like violations or buries real ones among them.

### Retroactive rejection is a tracked obligation loop, not an event

When B rejects retroactively, the fact must reach C and D, who acknowledge it and **report back to B what
they recognised and what action they took**:

```
B 사후 반려
  → notify:    C, D
  → 인지:       each acknowledges
  → 조치 보고:   each reports the substantive action taken
  → B closes the loop
```

Two consequences that change the data model:

1. **D must be notified even though D never saw the matter** — C's closure truncated the line before
   reaching D. So the notification scope is **the line as raised**, not the path actually signed. The
   document must therefore retain both: the **line-as-raised** and the **line-as-executed**. Truncation
   changes whose signature was required; it does not change who must be informed.
2. **A second edge kind is required on the line.** The model so far had only 결재 (signing). C and D
   reporting back to B is a **보고** relation — opposite in direction and different in meaning. 결재,
   협조 and 보고 are three distinct edge kinds, not one with a flag.

The loop has its own small state machine (통지 → 인지 → 조치보고 → 종료) with an actor and timestamp per
transition, and it must be tracked to completion — otherwise a rejection exists that nobody acted on.

**This is where the 업무 entity earns its keep.** The rejection, the notifications, the acknowledgements and
the action reports all link to the same 업무, so the entire loop is one traversal, and an unfinished
obligation transfers with the work at handover. If a participant departs mid-loop the obligation is not
lost. Completeness is the same graph query as 인계 완료: are there rejections with unacknowledged nodes, or
with no action report?

**Generalise the primitive — and most of it already exists.** This is not specific to 반려: it is
**notification requiring acknowledgement, scoped to line membership**. `0162_create_notices.sql` already
implements exactly that, and says so in its own header — *"acknowledgment) tracking. Distinct from the
personal `notifications`"* — with `notice_receipts.acknowledged_at` (`:46`) and an index on
`(notice_id, acknowledged_at)` (`:54`).

So 통지 → 인지 is built. What is missing is narrower than it first appears: `acknowledged_at` is a stamp,
not a report carrying content. The gap is the **조치 보고** return leg (a substantive reply, not an ack) and
**loop closure by the originator**. Extend or compose with `notices`; do not build a second acknowledgement
mechanism beside it.

### What makes a closure final? — unresolved, and blocking

If a line member may retroactively 반려 indefinitely, no downstream act is ever safe: the supplies are
bought, the hire has happened, the payroll has been paid. A settling point is required. Candidates: an
**이의기간** (objection window), finality on downstream execution, or an explicit **확정** step. This must
be decided before slice 0 is built, because it determines whether closure is a terminal state or a
revisable one.

A separate consequence regardless of which is chosen: **retroactive rejection after execution is a
compensating transaction, not an undo.** Paid payroll cannot become unpaid. 반려 must emit a new revision
— a recovery or correction — while the original remains in history. That couples the 결재 model to payroll
and accounting correction, not only to approval state.

### Worked example — a ₩100,000 supply purchase raised at a 현장

Sending a ₩100,000 비품 purchase up to the 본사 office for signature is wasteful. Traced through the
model, with no special case anywhere:

| Step | Resolution |
|---|---|
| Document class | 구매요청 / 지출결의 |
| Routing lookup | (category = 비품구매, amount ≤ band, scope = 현장 R) → competent unit = 현장, terminal |
| Line | one step, at the 현장 unit, mode = terminal-if-전결 |
| Eligible approvers | the grant fold at 현장 scope, filtered to holders of `purchase.approve` |
| Signature record | *(signer, authorising grant, scope)* — showing it was signed under 현장 전결 authority |
| 본사 | still holds `purchase.approve` at company scope by descent; later review is normal authority, not a deviation |
| Amount revised upward past the band | re-lookup → re-route. It happens to go upward here, but the mechanism is the same lookup, not an "escalation" path |

At ₩100,000,000 the same rule resolves to 본사. **One rule table, no special cases** — and it shows
exactly why routing must not be a capability restriction: 본사 *may* approve a ₩100,000 supply purchase;
it is merely wasteful to require it.

**This is also the smallest slice that exercises the whole model.** A single band, a single terminal step,
and one signature touches party, 업무, scope, the grant fold, routing, and capacity-recorded signature —
the full stack at minimum depth. It is a better first vertical slice than any single step of the
sequencing below, and it is what should be built first to prove the model before the model is widened.

### One person holds different authority *and different work* per scope

Group-level 결재권 and 업무 are distinct from 업무 inside the 법인, held **concurrently** by the same
person. Three consequences:

1. **The fold is already scope-parameterised.** `effective(person, scope)` gives
   `effective(P, group G) ≠ effective(P, company A)` for free — a person's authority is a set per scope,
   never one blob. No change needed; worth stating so it is not re-derived.
2. **업무 carries a scope, so handover is partial.** Someone may relinquish a group HR duty while
   remaining 인사팀장 at a subsidiary. Handover must therefore transfer *the work of that scope*, not
   "everything that person holds" — which means the 인계 완료 completeness query is also scope-bounded.
3. **A signature must record the capacity, not just the signer.** The same person may hold several grants
   that could each authorise the same act, and which one applied decides whether 전결규정 was satisfied.
   `gov_approvals` (`0153:65`) records `approver_id` alone with no capacity field, so it could not express
   this even if it were scope-aware. A 결재 signature must carry *(signer, authorising grant, scope)*.

### Three relations, currently conflated

| Relation | Answers | Today |
|---|---|---|
| **Control** (법인 간 지배) | which legal entities form a group | nothing exists |
| **Structure** (조직) | reporting hierarchy, 부서 / 팀 | `org_unit.parent_org_unit_id` only |
| **Competence** (전담) | which unit may **decide** what | **nothing exists** |

`0001_create_regions_branches.sql` states the current model in its own header: *"Regions and branches:
top-level organizational units. Every operational row … carries a non-null `branch_id` — see plan §2.3
branch-scoped authorization."* `regions` is `id, name`; `branches` adds only `region_id`. A flat two
level scheme, mandatory on every operational row, with ADR-0003 built on it — so **operational scope and
decision scope are currently one field**. Splitting them is the change. Branch remains the operational
scope; competence becomes the decision scope.

The only `jurisdiction` in the tree is `compliance_controls.jurisdiction` (`0148:27`) — legal
jurisdiction, unrelated. `org_unit` has no `kind`.

### Three locations, all of which may differ

A single work item has a **발생지** (the 사업장 where it arose), a **수행지** (the office where it is
performed), and a **결재 관할** (the unit competent to decide it). The owner's case — an office that
handles simple matters locally and sends the rest to 지사 or 본사 — is precisely these three coming
apart. Modelling them as one field is why the current scheme cannot express it.

### The mechanism already has a name: 전결규정

Korean corporate practice encodes exactly this as **전결규정** — delegation-of-final-authority rules:
(work category × amount band × scope) → final approving unit. Critically, its *purpose* is to make
matters **terminal at lower levels**, not to funnel everything upward. It is authored data, which is
exactly what the canvas requirement asks for.

**Consequence: the escalation path is per-category, and there is no single reporting line for 결재.**
Finance may route to 본사 재무 while HR routes to 지사 인사 from the very same originating unit. So the
route is a per-category chain resolved by lookup, never one tree walked upward. Competence is an
**edge** — unit ⟶ competent-for(category, band, scope) — not a level.

**It is a graph, not a chain.** Korean practice needs at minimum:

| Form | Shape | Requirement it imposes |
|---|---|---|
| **결재선** | serial chain of approvers | ordered steps |
| **합의** | *parallel* consent from related departments | **concurrent branches** — not expressible with a step index |
| **전결** | delegated final authority; closes the line early | a step may be terminal, conditionally |
| **대결** | proxy approval during absence | approver resolved at decision time, not raise time |

**Model.** A template is an object type per document class. Each step declares a **competent unit** —
resolved through 전결규정 rather than named directly — a **required capability**, and a **mode** (serial
branch, parallel 합의 branch, terminal-if-전결). The template also declares the **routing rule**: the
predicate over document attributes that resolves which unit is competent, since a threshold
(expenditure over ₩X) is a rule and not a category. Deliberately *routing*, not *escalation* — the
resolved unit may be above, lateral to, or below the raising unit.

A raised document is an instance: the rule is evaluated at raise time to resolve the line, and each
step's eligible approvers come from the **grant fold at that unit's scope**. That is what lets a
group-scoped approver sign a company-raised document, and equally what lets a 전담 연락사무소 close a
matter its 본사 never sees.

**Re-routing is a first-class deviation, not an exception.** Both rule-driven (a threshold crossed once
an amount is revised) and discretionary (an approver declares a different unit competent) re-routes
alter the remaining line as audited revisions on the line instance. The fixity chain then yields a
tamper-evident history: the line as raised, every re-route, and the fold that authorized each
signature. Note this cannot be modelled as "extend upward" — a re-route may *shorten* the line by
resolving to a unit with 전결 authority.

**Neither existing mechanism can carry this, for structural reasons rather than missing features:**

- `work_order_approval_steps` (`0008:59`) — `step_order SMALLINT CHECK (step_order BETWEEN 1 AND 3)` is
  a hardcoded **three-step maximum**; `role IN ('MECHANIC','ADMIN','EXECUTIVE')` is the demo vocabulary;
  `UNIQUE (work_order_id, role)` permits each role **once**. Serial only — 합의 is inexpressible.
- `gov_approvals` (`0153:65`) — `UNIQUE (org_id, request_ref)` allows **one decision per request**, and
  both FKs are composite `(user_id, org_id) REFERENCES users(id, org_id)`, so **the approver must be a
  user of that org**. Escalation to group is forbidden by the foreign key, not merely unbuilt. It is a
  four-eyes gate — what #521 uses to publish an object type — and correct for that job.

## The keystone: there is no person

- **`users` is keyed `(id, org_id)`** — `user_role_assignments` and `clearance_assignments` both use
  the composite FK `(user_id, org_id) REFERENCES users(id, org_id)`. The same human at two companies
  is **two unrelated rows with two ids**.
- **`employees` (`0063:2`) is a spreadsheet import row**: `company TEXT`, `name TEXT`,
  `source_filename`, `source_sheet`, `source_row`, `raw_row JSONB`, unique on `(org_id, source_key)`.
  `company` is free text, not a foreign key.
- **In the ontology the person is a title string** — `employment` declares `person_name` as its
  `title_property_key` and links to `job_position` and `org_unit` as real entities
  (`fixtures/employment.rs:142`, `:148`, `:199`, `:207`). The post is an entity; its occupant is text.

| Requirement | Why it is inexpressible today |
|---|---|
| 1 | the occupant has no identity to be separate *with* |
| 3 | needs one identity present in both orgs; there is none |
| 4 | `group_role_grants` binds a **company-bound user**, so nothing human-shaped crosses companies |
| 7 | an approval line of per-company user rows **breaks at the company boundary** — exactly where a group-level 결재 must cross |

**The person/party entity is the keystone. 전자결재 is downstream of it.**

## Recommended Direction

**Grants are ontology instances. Roles are grant bundles. 결재 lines are authored graphs. Cedar reads
the fold.**

A grant binds *(subject, capability, scope)* and carries a **source** — position, group, direct, or
delegation — plus its own validity window and reason. Effective authority is the fold of a person's
grants at a point in time, which makes authority **replayable**: *"what could this person approve on
2026-03-01?"* is answerable from the fixity chain, which is precisely the question 결재 audit asks.
Korean 대결/전결 (proxy and delegated final authority) are not deviations in this model — they are the
`delegation` grant source, time-boxed, which is evidence the model is shaped right.

**People are group-scoped.** Per the owner's choice, the group is the tenancy boundary for people, not
the company. This makes the group mandatory rather than optional — a standalone company needs a
degenerate single-member group — and that is an acceptable invariant.

**A role is a saved bundle of grants, not a third entity.** The layer lives on the grant's *scope*
(group vs company vs org_unit), not on the role. This deletes a whole layer: no role-versus-permission
vocabulary to keep aligned, no role hierarchy to version. `Feature` is deleted rather than extended.

**The canvas is the ontology authoring UI**, not a second bespoke editor. Per the owner it covers role
and permission definitions, 결재 templates per document class, org structure itself, and **four-eyes on
every authority change** — reusing `gov_approvals` / `gov_approval_consumptions`
(`0153`, `0164_bind_consume_four_eyes`). Without that last item the canvas is privilege escalation with
a nice UI.

**결재 is an authored template with audited runtime deviation.** A template is a type; a document's
line is an instance whose steps link to approvers resolved by the grant fold; a deviation (add, skip,
substitute) is a revision on that instance. The fixity chain therefore yields a tamper-evident
approval history for free. Neither existing mechanism is a routable line:
`work_order_approval_steps` (`0008:59`) is work-order specific, and `gov_approval_requests` /
`gov_approval_consumptions` implement a fixed four-eyes — what #521 uses to publish a type.

## The two hard problems

**1. Bootstrap circularity.** If grants are instances and Cedar gates instance reads — all six paths
closed in #525 — then reading a grant requires a grant. This must be broken deliberately, not
discovered. The shape that already exists is `PgCedarPolicyStore::load_enforced_object_policy_blocks`
(`platform/authz-rest/src/store.rs:569-586`): a read path that sits outside per-type gating and
**re-validates on every read**, with `0205:69-74` marking that re-validation a LIVE CONSTRAINT whose
deletion would kill its own justification silently. Authority reads need the same bargain — bypass the
gate they feed, and pay in re-validation that cannot be quietly removed.

**2. A second isolation dimension across 141 tables.** Verified 2026-07-29: the only tenancy GUC is
`app.current_org` (plus `app.audit_rehome`, `app.maintenance_force_remove`,
`app.platform_force_remove_org` as escape hatches), and **141 tables enable row-level security** on
it. Group-scoped people means a second dimension — `app.current_group` — bridged into a floor that 141
tables enforce one way. This is the largest single engineering cost in the chosen model.

## Key Assumptions to Validate

- [ ] **The fold is expressible in Cedar without a second evaluator.** Test: encode four grant sources
      over one person in two companies and confirm Cedar alone decides requirement 3, with no Rust
      fallback. If it needs a companion evaluator, the two will diverge.
- [ ] **Authority reads can bypass their own gate safely.** Test: build the definer, then attempt the
      #525 exploit shape against it — an unauthorized caller minting a grant — and confirm refusal by
      execution, not by argument.
- [ ] **A second RLS dimension is additive, not a rewrite.** Test: add `app.current_group` to three
      representative tables (one org-only, one group-only, one bridging) and measure whether existing
      policies compose or must be rewritten. Three tables answer it; 141 is the risk.
- [ ] **A grant bundle is sufficient where orgs expect a role.** Test: can you answer "everyone with
      role X, for recertification" from bundles alone? If not, the role must become queryable and the
      deleted layer comes back.
- [ ] **Deviation stays auditable under replay.** Test: deviate a line mid-flight, then replay the
      fold at raise time and at decision time and confirm both reconstruct exactly.

## MVP Scope

**Slice 0 — the ₩100,000 비품 purchase, end to end.** Build the worked example above at minimum depth
before widening anything: a party, one 업무, one scope, a two-entry routing table with one band, the
grant fold, one terminal signature recording its capacity. It proves or refutes the whole model in one
vertical slice, and every step below becomes a *widening* of something already proven rather than a bet.
This ordering follows the repo's own discipline — a narrow case executed beats a broad case designed.

The steps below widen it. Two keystones remain ordered: **party**, then **업무** — assignment points a
work at a party, so party is the more foundational of the two.

**Step 1 — party.** Platform-level, above both `users` and `employees`, with per-org user rows linking
to it rather than being replaced, and **tenant visibility mediated by edges** rather than by scoping the
row. Named `party`, not `employee`, so later verticals reuse it. Disjoint from the in-flight
policy-topology work, so it can start immediately.

**Step 2 — 업무 (work) as an entity**, with typed relationships to artifacts and to assignees. This is
what makes handover, discovery and provenance one traversal, and it subsumes six existing
domain-specific handover/substitution tables.

**Step 3 — assignment as a grant source.** The narrowest useful slice of the authority model: authority
that arrives with the work and expires with it. Proves the fold without needing the full grant taxonomy.

**Step 4 — control edges between legal entities**, with group designation derived. Note
`group_memberships` is `UNIQUE (org_id)` (`0060:36`) and `organizations.group_id` is a single-valued FK —
both must give way for joint ventures and nesting, and the duplicate representation should collapse to
one in the same change.

**Step 5 — the remaining grant sources** (position, group, direct, delegation) and scope expressions
over the control graph. Requirement 3 is provable at the end of this step.

**Step 6 — the competence relation and 전결규정.** A third relation alongside control and structure,
which exists nowhere today: unit ⟶ competent-for(category, band, scope), authored as data. This also
requires splitting decision scope from the mandatory operational `branch_id` that ADR-0003 built on, and
separating a work item's 발생지 / 수행지 / 결재 관할. Nothing about 결재 routing is expressible before
this lands.

**Step 7 — 결재 template and line.** Depends on 1, 2, 3, 5 and 6.

**Running parallel to all of it, because the payroll vertical needs them and they touch different files:**
org-unit **kind** and **lifetime** (부서 / 팀 / TF / 사업장, with 사업장's legal attributes), and
employment type as **authored data with per-type rules** rather than either of the two conflicting
`CHECK` vocabularies. Both also need employment to separate **employer** from **worksite** for 파견.

**Out of the MVP:** deleting `Feature` (sequence it only once the data layers carry what it gates), the
full canvas, group→company rollup reporting, and retroactive re-linking of historical artifacts — the
last because retroactive linking is retroactive access-granting and deserves its own design.

Steps 1 and 2 are two new ontology types, so two new fixture slots — and `LANE_TYPES` is `[&str; 5]`
in `company_conformance.rs`, whose module doc forbids lane edits. They need a Phase-0-style
pre-reservation commit widening the index to 7 (`fanout-plan-DRAFT.md` §5, §97), then two lanes.

## Not Doing (and Why)

- **Extending the `Feature` matrix** — demo scaffolding, and a canvas cannot configure a `const fn`.
  Deleting it is the requirement; adding a corporate role column would make **six** authorization
  mechanisms where five is already the problem.
- **A bespoke authority canvas** — the ontology authoring UI is already the program's deliverable.
  A second editor duplicates it and guarantees drift.
- **A permission vocabulary separate from Cedar actions** — two vocabularies diverge. There is already
  a fixed `AUTHORING_ACTIONS` set; do not add a third naming scheme.
- **Authority derived from rank** — rejected by requirement 3 at the outset. Rank may *source* a grant;
  it may never *be* the authority.
- **A global person row with an RLS exception** — person data is where the isolation floor is least
  safe to weaken, and 141 tables currently hold that floor.
- **Computed-at-request 결재 routing** — reorganizations would silently change who approved what, and
  history would stop being replayable. That defeats the point of a fixity chain.

## Open Questions

- **Who may author authority, and in which scope?** Four-eyes answers *how many*, not *which scope*.
  A group-scoped authority editor can grant into every company in the group.
- **`group_role_grants.group_role` is a CHECK over three literals** (`GROUP_ADMIN`, `GROUP_VIEWER`,
  `GROUP_FINANCE`). Configurable group roles require that to become a reference — a migration on an
  owner-only table.
- **Does deleting `Feature` need a coexistence period?** `docs/specs/cedar-pbac-coexistence-map.json`
  is keyed by domain (`identity.policy`, `workflow.guards`) and already maps the overlap; nothing yet
  defines the end state.
- **PII attaches at step 1.** A durable person identity is where PIPA and Korean payroll obligations
  become real. Every jurisdiction binding and Korea control in the program ledger reads `HOLD`.
- **Is HR+payroll-first compatible with authority-first?** The vertical needs employment records, and
  employment records need the person. Authority may be a prerequisite the vertical did not budget for.
