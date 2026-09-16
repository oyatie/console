---
id: ADR-0045
status: accepted
doc_status: review
date: 2026-09-15
owner: jasonlee
decision: clean-architecture-rings
amends: [ADR-0001]
related: [ADR-0001, ADR-0041]
---

# ADR-0045 — Clean Architecture rings; Rest/Worker may not skip Use Cases

## Status

**Accepted 2026-09-15.** This ADR amends ADR-0001 so the compiler-enforced layering is
Clean Architecture (Entities, Use Cases, Interface Adapters, Frameworks &
Drivers) with dependencies pointing inward, not a hexagonal core blob with
ports around it. Does not clear payment, legal, production, or Intelligence
HOLDs. Does not create missing `*-application` crates in this record.

## Context

ADR-0001 already named “clean-architecture layering” and enumerated
`console-<domain>-{domain,application,adapter-postgres,rest,worker,ui}`. The
layer-boundary gate still allowed `Layer::Rest` / `Layer::Worker` to depend on
Domain and Adapter directly. PRODUCT recorded that REST-over-use-cases was an
aspiration: six non-platform REST crates had no application sibling, and
Rest→Domain / Rest→Adapter edges were legal. That is hexagonal packaging
(controllers and gateways talking to a single core), not the four rings.

The wanted rule is prescriptive **inside** the core:

1. **Entities** — enterprise rules (`Layer::Domain`, plus shared `Layer::Kernel`).
2. **Use Cases** — application policies (`Layer::Application`).
3. **Interface Adapters** — controllers (`Layer::Rest`, `Layer::Worker`) and
   gateways (`Layer::Adapter`). Presenters live with controllers.
4. **Frameworks & Drivers** — Axum, sqlx, Leptos, `Layer::App`, `Layer::Ui`.

Hexagonal adapters remain the outer two rings. They are not a substitute for
splitting Entities from Use Cases.

## Decision

1. **Rings, inward only.** `Layer::Rest` and `Layer::Worker` `allowed_deps` are
   `[Application, Contracts, Platform, Kernel]`. They must not depend on
   `Layer::Domain` (skipping Use Cases) or `Layer::Adapter` (depending on a
   gateway implementation). `Layer::Adapter` may still depend on Domain:
   gateways map Entities to storage. `Layer::Ui` is unchanged (ADR-0041).
2. **Use Cases crate is required for Rest.** A `console-<stem>-rest` crate
   classified as `Layer::Rest` must have a workspace sibling
   `console-<stem>-application`. Platform `*-rest` crates stay `Layer::Platform`
   (path classification before the `-rest` suffix) and are not this rule.
3. **Shrink-only ratchet.** Edges and missing-application crates that exist on
   the tree that accepts this record are listed in
   `KNOWN_REST_OR_WORKER_SKIP_EDGES` and `KNOWN_REST_WITHOUT_APPLICATION` in
   `backend/ci/gates/layer-boundary`. A new skip fails closed. A listed skip
   that disappears without deleting the entry fails as stale. The lists may
   only shrink. Paying off `KNOWN_REST_WITHOUT_APPLICATION` requires a normal
   Rest → `console-<stem>-application` dependency, not an empty sibling crate.
   consulting / facilities / production `sqlx` in REST stays on that list until
   that edge exists.
4. **Not a vertical rewrite.** This record does not move payroll calculate /
   submit / decide into a new `console-payroll-application` crate. That remains
   a later admitted lane from a red named probe.

## Consequences

+ The gate now fails a *new* Rest→Domain or Rest→Adapter edge, which hexagonal
  packaging would have allowed.
+ Existing skippers stay visible as ratchet debt, including payroll (no
  application crate; REST depends on domain and adapter-postgres).
− Controllers still reach `Layer::Platform` and `Layer::Kernel` (request
  context, IDs). Tightening those to “use-case DTO only” is unscheduled.

## Alternatives considered

Keep Rest→Domain legal and document hexagonal ports (rejected: that is the
status quo the owner rejected). Forbid Rest→Platform in the same change
(rejected: every REST crate uses request-context; that is a later ratchet).
Create every missing `*-application` crate empty in this record (rejected:
empty use-case crates would fake the ring without moving behavior).
