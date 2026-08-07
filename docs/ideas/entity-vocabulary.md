> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Entity vocabulary — naming against industry convention

> `Status: IDEA ONE-PAGER — pending approval.`
>
> Naming is cheap to decide now and expensive to change once it is in crate names, type keys and column
> names. This fixes the vocabulary against established convention — principally SAP's enterprise structure
> and the party model — and separates the words that helped us think from the words that may appear in code.
>
> Sources are `docs/ideas/research-sap.md` (URL + confidence per row) and
> `docs/program/benchmark-matrix/lenses/data-model.md`. Not restated here; referenced.

## Problem Statement

**How might we** name our entities so that someone who knows enterprise software recognises them
immediately, and so that no metaphor we used while thinking survives into the schema?

## Rule 1 — the schema uses domain terms only

Analogies used while working through the design — from games or anywhere else — were a way of getting a
point across quickly. They do not appear in a crate name, a type `stable_key`, a column, a route, or
user-facing copy. Every concept ships under its domain name: 법인, 기업집단, org unit, work, grant,
delegation-of-authority rule.

One collision is worth naming because it already caused ambiguity in our own writing: **"party" means an
identity here, never a team.** A task-scoped team is an `org_unit` with a kind and a contract-derived
lifetime.

## Rule 2 — the identity is a party, and that is the standard term

`party` **stays**, in its industry sense: one durable handle for a natural person or a legal entity, reused
across tenants and verticals, holding no personal data. `party_kind ∈ {NATURAL, LEGAL}`.

This is the **party model** (Silverston's canonical formulation) and it is what SAP arrived at
independently. From the research: *"The strategic object model in SAP S/4HANA is the Business Partner"*, and
CVI exists so that a customer who is also a vendor becomes *"a single Business Partner, assigning both a
customer role and a supplier role to that one entry."*

**The consequence we already reached, now with a citation: employee is a ROLE, not an entity.** SAP models
Customer, Vendor and Employee as **BP roles** over one Business Partner. Our earlier conclusion — that
`employees` being a spreadsheet import row is the defect, and that an employee is a party in an employment
relationship — is the standard model, not an invention.

Naming choice: **`party`**, not `business_partner`. `business_partner` is SAP's brand for the same concept
and carries SAP-specific role machinery; `party` is the vendor-neutral term and is what the plan already
uses. Do not introduce both.

## Rule 3 — organisation is several independent dimensions, not one tree

SAP's stated insight, and the most transferable thing in the research: *a business is not one hierarchy but
several independent dimensions over the same transaction, each answering a different question, with a
posting carrying a coordinate in every one.*

That is the same conclusion we reached from 소속 / 직급·직책 / 직무 / 결재선 being independent — arrived at
from authority rather than from accounting, which is mild evidence it is right.

| SAP unit | Confidence | Korean | What it answers |
|---|---|---|---|
| **Client** | CONFIRMED | 그룹 (as tenant) | which corporate group's data is this |
| **Company code** | CONFIRMED | **법인**, one-to-one | which legal entity is accountable; the *only mandatory* FI unit |
| **Company** (≠ company code) | CONFIRMED | consolidation parent | who consolidates, and the counterparty on intercompany postings |
| **Controlling area** | CONFIRMED | — | the closed cost-controlling boundary; 1:n over company codes is what enables cross-entity allocation |
| **Profit centre** | CONFIRMED | 본부 / 사업부 | which internal area owns the result — deliberately independent of legal entity |
| **Cost centre** | CONFIRMED | 부서 as a cost home | where cost is incurred |
| **Plant** | LIKELY | **사업장 as a physical site** | where things physically happen; normally the valuation area |
| **Business place** | CONFIRMED | **사업장 as a tax-filing unit** | who files, below company-code level — **South Korea explicitly named** in SAP's own doc |
| **Segment** | CONFIRMED | external reporting division | IFRS 8 / US-GAAP segment reporting |
| **Business area** | LIKELY | — | **legacy; absent from S/4HANA Cloud Public.** Do not adopt |

### The finding that changes our org model

**사업장 is two SAP units, not one.** `Plant` is the physical site; `Business place` is the tax-registration
unit that exists *because* Korea requires filing below the legal-entity level. SAP separates them
deliberately, and our plan currently has a single `org_unit.kind = 사업장` that would have to carry both.

They come apart in practice: two physical sites can share one 사업자등록번호, and one site can host work
belonging to two registrations. Since 4대보험 and 사업장-based payroll obligations follow the *registration*
and not the *building*, collapsing them puts payroll correctness on the wrong dimension.

**Recommendation: model them separately** — a location dimension and a tax-registration dimension — and let
an `org_unit` reference each independently. The plan's separate `worksite_registration` (Tier T, holding
4대보험 and 사업자등록번호) is already the registration half; what is missing is that the *site* is not the
same object.

**Do not adopt `business area`.** It is legacy and absent from S/4HANA Cloud Public Edition; SAP steers new
builds to profit centre plus segment. Adopting it would import a deprecation.

## Rule 5 — org unit is ONE type; the distinctions are not an enum

The 부서 / 팀 / TF / 사업장 / 연락사무소 list was examples, and checking it against convention shows it
conflates three different things. **Neither SAP nor Workday enumerates department-versus-team as a kind.**

SAP HCM Org Management is a typed relationship graph with four object types — **`O` org unit, `S` position,
`C` job, `P` person** — where existence is infotype 1000, **every relationship is a record in infotype 1001**
(reciprocal, with automatic inverses, carrying **validity periods**), and **object *and relationship* types
are customer-definable**. There is exactly one `O`. A division, a department and a team are all `O`; what
distinguishes them is **position in the graph, validity period, and name** — not a kind column.

Two consequences for us:

**Our engine is already this shape.** Authored object types, authored link types with cardinality,
effective-dated revisions. SAP's answer to "how many org unit kinds" is "define what you need" — which is
what an authored ontology gives for free. A fixed `kind` enum would be *less* capable than the substrate it
sits on, and it is the closed-vocabulary mistake this program has already made three times
(`Feature`, `AUTHORING_ACTIONS`, two employment-type CHECKs).

**And the list splits across three dimensions, not one:**

| Example given | What it actually is | Ships as |
|---|---|---|
| 부서, 팀, TF | the same org-unit type at different graph positions; a TF differs by having an **end date**, not a kind | one `org_unit` type + relationship + validity period |
| 사업장 | **two** dimensions — physical site *and* tax-filing registration (see Rule 3) | a location reference + a registration reference |
| 연락사무소, 지사, 본사 | **legal standing and tax-registration status**, not org structure | a property of the legal entity / registration, not an org-unit kind |

The third row is the one most likely to be modelled wrongly. A 연락사무소 differs from a 지사 in what it may
legally do and how it is registered — so encoding it as an org-unit kind puts a legal fact in a structural
field, where no rule can act on it. **This needs qualified Korean counsel to state precisely, and Korea
controls are `HOLD`** (`console-jurisdiction-register.json:1186` — agents may not invent certainty), so this
document names the shape and asserts nothing about the law.

**Recommendation: no `kind` enum.** One org-unit type, distinctions from the graph and validity period, with
legal and tax facts on the entities that own them. If a genuinely structural discriminator turns out to be
needed, it should be an authored property — not a `CHECK` constraint.

## Rule 6 — position is an entity because it survives vacancy

Our "employee and position are separate entities" conclusion has a better justification than the one we gave
it, from SuccessFactors Position Management (`CONFIRMED`, sourced in `research-sap.md`):

> In a job-based structure, job details live on the employee record and are **lost when they leave**; in a
> position-based structure they live on the **position, which survives vacancy** — which is what makes
> headcount and budget planning possible.

That is the reason, and it is stronger than the symmetry argument: a position must be an entity so it can be
**empty**. An unoccupied post with a budget and a job profile is the thing headcount planning operates on,
and it is inexpressible if the post is a field on a person.

Two further items worth adopting, both sourced: **position hierarchy is the leading hierarchy by default**,
distinct from the employee→manager reporting line — two graphs, which matches our finding that 소속 and
결재선 are independent. And **incumbent tracking**, so a person on leave or a global assignment retains a
right to return to their position — which is exactly the 연차 / 파견 case, and it belongs to the position
rather than to the work.

Naming, per SAP HCM's four object types: **position** (`S`) is the post; **job** (`C`) is the
classification — 직무. Our 직급 / 직책 / 직무 distinction maps onto job-versus-position plus a grade
attribute, and it should not invent a third object type.

## Rule 4 — dimensions are references, not a nesting

Because the dimensions are independent, an entity or a posting carries **a coordinate in each**, rather than
sitting at one place in one tree. This is what our own conclusion — that authority scope, competence and
legal structure are separate graphs — looks like on the accounting side.

Concretely: our `finance_gl_vouchers` header already carries an untyped `source_object_type` /
`source_object_id` pair, and the *line* carries no dimension at all. SAP puts a coordinate on the line.
That is the gap the peer finance plan inherits, and naming it here so it is not rediscovered.

## Key Assumptions to Validate

- [ ] **The two 사업장 dimensions are genuinely both needed in the first vertical.** Test: find one real case
      in the target business where a site and a tax registration do not correspond one-to-one. If none
      exists, one dimension with a documented ceiling is the honest simplification — but do not collapse
      them by assumption.
- [ ] **Dropping the `kind` enum does not lose a distinction someone relies on.** Test: take every 조직
      type the target business actually uses and express each as graph position + validity period + name.
      Any that needs a discriminator to *behave* differently — not just to display differently — is a real
      counterexample, and the discriminator should then be an authored property rather than a CHECK.
- [ ] **직급 / 직책 / 직무 map onto job + position + grade without a third object type.** Test: express a
      real 인사발령 — a 직급 change with no 직책 change, and a 직책 change with no 직급 change — in that
      vocabulary. If either is awkward, the mapping is wrong.
- [ ] **`party` reads clearly to a Korean business user.** Test: it has no natural Korean translation
      ("당사자" is legalistic). If the UI needs a different display term than the model term, decide the
      display term now rather than letting each surface invent one.
- [ ] **Our tenant boundary can carry the Client/Company-code distinction.** `organizations` is currently
      *both* the tenant and the 법인. SAP separates them (Client over Company codes) precisely so a group of
      legal entities can share one data space — which is the requirement. This is the `org_id` ×
      `BranchScope` governance gap in another guise, and it is the largest naming consequence in this
      document.
- [ ] **Controlling area is worth adopting at all.** It exists to allow cost allocation *across* company
      codes within a boundary. That is exactly the group-level allocation question — but it is finance-plan
      scope, and adopting the name without the mechanism would be cargo cult.

## Not Doing (and Why)

- **Adopting SAP's names verbatim** — `BUKRS`, `KOKRS`, `PRCTR` and the four-character-code culture are
  artifacts of a 1970s system. Adopt the *distinctions*, not the identifiers.
- **`business_partner` alongside `party`** — two names for one concept guarantees they diverge.
- **`business area`** — legacy, absent from S/4HANA Cloud Public.
- **Renaming `organizations` in this document** — the Client/Company-code split is a real decision with
  migration consequences across 141 RLS policies and ~100 `org_id`-immutability triggers. Named as an
  assumption to validate, not decided here.
- **A Korean-first schema vocabulary** — the model stays English for crate and column names, as the repo
  already does; Korean is a display concern. Mixed-language identifiers are the worse of both.

## Open Questions

- **Is `profit centre` an `org_unit` kind or its own dimension?** SAP makes it a dimension precisely because
  it is independent of legal entity — a 본부 spanning two 법인 is expressible in SAP and not in a single
  org-unit tree. Ours is currently a tree.
- **Does the group need `Company` (the consolidation parent) as distinct from the group itself?** SAP has
  both because consolidation grouping and trading-partner identity are different questions. We have neither
  yet.
- **What is the display term for `party` in Korean UI?** Model term and display term may legitimately
  differ; the risk is each surface choosing its own.
