# Console information architecture — surfaces, navigation, routes

> `Status: IDEA ONE-PAGER — pending approval. Planning only; implementation is gated.`
>
> The deep-planning work `docs/ideas/d4-frontend-charter.md` authorizes and deliberately does not
> pre-empt. That charter fixes the stack (Leptos), the rendering shape (SSR shell + island editors), the
> data contract (a Rust contracts crate), and the gate (the engine substrate, three conditions still
> open). This document decides what the console *is*.
>
> Owner input, 2026-07-30: all four daily loops are first-class — approve, do assigned work, look up,
> configure; eventual scope is what business SaaS and omni platforms like SAP cover, with HR + payroll
> first; design the model independently of the WIP visual prototype and reconcile after.

## Problem Statement

**How might we** build a console whose surfaces render *authored* object types — so a business user adds a
type and can immediately work with it — without the result feeling like a database table with a
navigation bar?

## Recommended Direction

**A small fixed set of generic surfaces, arrangement derived from type metadata, with no per-type layout
override.**

The scope decides this before taste does. SAP-scale breadth across four first-class loops cannot be
hand-designed — that is *why* SAP looks like SAP, and Fiori was a retrofit of consistency onto breadth
that never had it. And the ontology already made types **data**, so surfaces must be data-driven or a
developer is required for every type, which contradicts the product's stated constraint.

**The game lens is not decoration here; it is the existence proof.** An RPG inventory panel renders any
item; a quest log renders any quest; a character sheet renders any character. A handful of panels serve
tens of thousands of objects and still feel good. The reason is the opposite of configurability: **the
arrangement is identical every time, which is what makes it learnable.** Configurable layouts are exactly
why enterprise software does not transfer between deployments. Palantir keeps a canonical Object View for
the same reason.

So: **wanting to override a type's arrangement is a signal its metadata is too thin.** The fix is a richer
declaration — a property that knows it is a title, a currency, a reference, a status — not a layout editor.
That keeps the design budget on seven surfaces instead of seventy, and it keeps the no-code promise honest,
because authoring a *type* is the only authoring step.

## The surface inventory

Seven surfaces. Each renders any type; none is domain-specific. "Driven by" names the type metadata that
determines what appears — the contract the ontology must satisfy for the surface to work at all.

| # | Surface | What it is | Driven by |
|---|---|---|---|
| 1 | **Object view** | one instance: properties, links, actions, history, its record and its economics | property declarations (kind, title-property), link types, action types, revision chain |
| 2 | **Collection view** | many instances, with table / board / calendar / map as *presentations* of one concept, plus filters and saved views | the type's properties (which are filterable, sortable, groupable), link types for grouping |
| 3 | **Action form** | executing one action: parameters, validation, dry run, submit | the action's declared parameter schema and submission criteria |
| 4 | **결재 line** | a document's approval line: as-raised beside as-executed, each signature with the capacity it was made under, open obligation loops | the routing rule, the line instance, signature records |
| 5 | **Activity / ledger** | the record spine rendered — self-describing transactions, scoped to an object or a scope | audit events, transactions, the capacity field |
| 6 | **Authoring canvas** | author a graph of typed things: object types, roles, 전결규정, org structure | a node vocabulary per authoring kind over one canvas substrate |
| 7 | **Search & traverse** | global entry, and traversal from any object along its links | the link graph, human-safe identifiers |

**Seven, not eight — a person-centric surface would be a special case, and it is not needed.** An earlier
draft listed a separate composite showing who you are, your authority per scope, your assigned work and
your pending decisions. But `party` is a type, so **that is the object view of a party** — properties,
links, and the fold, which is exactly what surface 1 renders. `/me` is a shortcut to your own party's
object view, not a distinct surface with its own layout to design and maintain.

The collapse is worth more than one fewer surface: a special-cased person screen would have been the first
crack in "every surface renders any type", and the exception would have justified the next one.

Plus **the shell**: navigation, scope switcher, and the nothing-else that a server-rendered frame needs.

**The claim worth testing in surface 6** is that types, roles, 전결규정 and org structure are all *"author a
graph of typed nodes"* and can share one canvas with different node vocabularies. That is elegant and
might be wrong — a 전결규정 rule table may be a grid rather than a graph. Treat it as an assumption, not a
design.

**The claim worth testing in surface 2** is that table, board, calendar and map are *presentations* rather
than surfaces. If a calendar needs metadata no other presentation needs, it is a surface.

## The comprehensibility bar

The requirement behind this whole model is that the console feel easy — state visible at a glance, no
manual needed. That is a quality bar on **all seven surfaces**, not a feature of one screen, and it needs
to be testable or it will not survive a deadline:

- **Glanceable.** A user's own state — what is mine, what is waiting on me, what I may do here — is
  readable without navigating or expanding anything.
- **Refusals explain themselves.** A user sees "you may not approve this because…" and, symmetrically,
  "you may, because grant G at scope S". We already log Cedar decisions
  (`0159_create_cedar_decision_log.sql`); in the fold model the explanation is a traversal, so this is a
  feature to build rather than a report to run.
- **Same shape every time.** What the derived arrangement buys: knowing one type's screen means knowing
  every type's screen.
- **State is visible in form, not only in text.** A pill, a chip, a severity stripe — what needs attention
  reads before it is read.
- **No dead ends.** Every object reachable from every object it relates to.

These sit alongside ADR-0025 §7's nine-item bar, which covers correctness and completeness but says nothing
about whether the result is comprehensible.

## Navigation — derived, not authored

The navigation is **not a static tree**. It is the set of (surface × scope) pairs the grant fold permits
for the current party, in the current scope.

Three consequences, each from a decision already made:

1. **A denied entry is ABSENT, not hidden.** `DN-0003` invariant 5 requires denied data omitted *including
   counts and relationship existence*. A greyed-out menu item discloses that the thing exists. SSR is what
   makes this real rather than aspirational — the entry never enters the delivered bytes, which is the
   authorization argument the charter records for choosing SSR at all.
2. **Progressive disclosure is free.** A new member sees few entries and a 그룹 인사 sees many, because both
   are the same fold evaluated against different grants. No separate "simple mode" to build or maintain.
3. **The scope switcher is the org switcher.** One human at two companies switches scope, and the whole
   navigation re-derives. This is the character/account split from the game lens, and the passkey choice
   already *is* the org choice, so the mechanism ships.

**The open cost:** the fold is 4–5 round trips and materialized per `(party, scope)` keyed on
`policy_versions`. Rendering navigation per request means the fold is on the critical path of every page.
That is the strongest argument for the materialization the plan already specifies — and the first thing to
measure, because if nav costs a fold, SSR's per-request cost is the fold's cost.

## Routes — object-addressable, and existence-safe

```
/                       overview — the composite home (ADR-0023: canonical authenticated landing)
/me                     shortcut to /o/party/{self} — not a distinct surface
/inbox                  pending decisions (Work Hub's role-aware action-inbox contract)
/work                   assigned 업무 (mywork's personal queue)
/o/{type}               collection view
/o/{type}/{id}          object view
/o/{type}/{id}/a/{act}  action form
/o/{type}/{id}/line     결재 line, when the object is a document
/author/{kind}          authoring canvas
/search?q=              search and traverse
```

Two properties this shape must have, and one is a finding this repo already paid for:

**A denied id and a missing id must be indistinguishable.** The #525 work found that resolve-by-code
returned a *different* 404 for a policy-hidden row than for a nonexistent one — a distinguishable-error
information leak. A route carrying `{type}` and `{id}` is a probe for exactly that, so the identical-404
property belongs in the acceptance criteria of every object route, not in a security review afterwards.

**`{type}` is a stable key, never a version id.** `list_object_types` is `DISTINCT ON (stable_key) …
published DESC`, so publishing v2 currently orphans v1's instances. A URL that survives a type version
bump must address the stable key. The version-orphaning defect is the plan's own open item; the routes
must not encode the bug.

## The identity dependency, stated rather than discovered

`/me`, the derived navigation and the scope switcher all need an identity to hang off — and the plan
**defers `party`**. In Slice 0 the only identity available is the org-scoped `users` row, which works for
one company and breaks the moment a person is at two, since one human at two companies is two unrelated
rows with two ids.

So the identity-bearing surfaces have a **reduced form** until the handle lands: `/me` resolves to the
current org's user, and the scope switcher has exactly one scope. That is coherent for Slice 0, and it means
the multi-company behaviour this IA describes is **specified but not reachable** until the party decision is
taken. Recorded here so `/me` is not built as though the handle exists.

## Key Assumptions to Validate

- [ ] **Derived arrangement is good enough that nobody asks for a layout override.** Test: render the five
      shipped conformance types (`company`, `org_unit`, `job_position`, `employment`, `pay_run`) in the
      object view from metadata alone, and show them to someone who knows the domain. If three of five need
      hand-arranging, the metadata is too thin — enrich the declaration, do not add a layout editor.
- [ ] **Every entity in the model has a natural home on one of the seven surfaces.** Test: walk the plan's
      sixteen entities against the inventory. An entity with no home is a signal the *entity* is wrong or
      the inventory is incomplete. This is the sharpest test in the list, because it fails in both
      directions and either failure is informative.
- [ ] **The fold is affordable on the navigation path.** Test: measure fold cost at realistic grant counts,
      materialized versus on-demand, with the known-bad control being a fold whose cost grows with total
      org grants rather than with the person's. This is experiment X6 and it is unrun.
- [ ] **Four authoring kinds fit one canvas.** Test: sketch a 전결규정 rule and an object type in the same
      node vocabulary. If the rule wants a grid, surface 7 splits.
- [ ] **A type authored end-to-end produces a working object view with zero code.** The whole thesis in one
      test, and it is currently blocked: X1 confirmed a relationship needs a property's `config.link`, X2
      confirmed a published type lists `[]` until a policy is attached, and projected-type actions still
      need a hand-written Rust closure per action.

## MVP Scope

**In — one surface, one type, end to end:** the **object view** for `employment`, rendered from type
metadata, with its links traversable and its actions listed. Plus the shell with fold-derived navigation
and the identical-404 property. That is the smallest thing that tests the central claim, and it is the
frontend analogue of the ₩100,000 비품 slice: narrow, but it exercises metadata-driven rendering, the fold,
SSR authorization, and the contracts crate at once.

**Out of the MVP:** the authoring canvas (it needs the largest design and depends on the type-metadata
richness the MVP measures), the 결재 line (needs the routing model, which is a plan widening), the ledger
(no read model exists), map and calendar presentations, and anything that would require a layout override.

**Not implementable yet regardless of scope:** everything above is gated. Three engine conditions are open
— projected-type dispatch, a read model, a real dry run — plus the contracts crate, which is new authoring.
This MVP is what the frontend team plans toward, not what it builds next week.

## Not Doing (and Why)

- **A per-type layout override** — it is the superset that cannot exist before the derived version, and the
  game lens argues against it on the merits: identical arrangement is what makes a generic surface
  learnable. Revisit only if the first assumption above fails, and even then prefer richer metadata.
- **Hand-designed screens per domain** — arithmetically impossible at SAP-scale breadth with four
  first-class loops, and it would make "add a type" a visibly second-class experience, which is the
  no-code promise failing where users would notice most.
- **A static navigation tree** — it cannot express the fold, and a greyed-out entry violates the
  omit-including-counts invariant. Derived navigation is not an optimisation; it is the only correct one.
- **Route-level version ids** — encodes the version-orphaning defect into every URL and every bookmark.
- **Reconciling with the WIP visual prototype yet** — per owner decision, the model is settled on its own
  merits first. The prototype informs the visual layer, and this document should be checked against it
  before either is built.
- **Offline or mobile** — an internal business console on-network. Not a decision to revisit until someone
  names the user who needs it.

## Open Questions

- **Is `/overview` a composite of the other surfaces, or its own?** ADR-0023 makes it the canonical
  landing route with Work Hub as the action-inbox contract and `mywork` as the personal queue. If overview
  is a composition of inbox + work + your own party's object view, it needs no surface of its own — which is the
  cheaper and probably better answer, but it is not decided here.
- **What renders an entity that is neither Tier N nor projected?** The plan has four storage tiers and the
  object view is described in ontology terms. A Tier T table like `work` is projected into the ontology by
  one match arm; a Tier O table like the group-grant store is definer-mediated. Does the object view render
  those, and if so through what metadata?
- **What is the unit of a saved view, and who may share one?** A saved collection view is authored
  configuration, so it is subject to the same authorization question as any other authored thing — and a
  shared view could disclose the existence of rows its recipient cannot read.
- **Does a party's object view disclose authority the viewer may not see?** Rendering someone else's party
  means reading their grants. That is a legitimate need for an HR officer and a disclosure otherwise — so a
  party's object view is an authorization surface, not a profile page, and the fold has to be evaluated
  for the *subject* while being authorized for the *viewer*. Two folds, one screen.
