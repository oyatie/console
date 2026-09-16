---
id: ADR-0001
status: accepted
doc_status: published
date: 2026-06-12
owner: jasonlee
consensus: ralplan iteration 3 (Planner/Architect/Critic APPROVE, 2026-06-12)
amended_by: [ADR-0041, ADR-0045]
related: [ADR-0012, ADR-0035, ADR-0041, ADR-0045]
---

# ADR-0001 — Modular-monolith Cargo workspace with compiler-enforced clean-architecture layering

## Status
Accepted (consensus-approved plan §2.1).

**Amended 2026-08-25 by ADR-0041**, which adds `ui` to the enumerated crate family and accepts `Layer::Ui` with legal edges `Ui → {Contracts, Ui}` and `App → Ui`. The original layering `kernel ← domain ← application ← adapter ← {rest, worker} ← app` stands; `ui` is an additional family member, not a replacement for rest/worker.

**Amended 2026-09-15 by ADR-0045**, which names the four Clean Architecture rings (Entities = domain+kernel, Use Cases = application, Interface Adapters = rest/worker/adapter, Frameworks = app/ui and drivers) and forbids Rest/Worker → Domain and Rest/Worker → Adapter except a shrink-only ratchet. Hexagonal “one core plus ports” is not the inner-core rule.

## Context
One small team builds a system spanning nine business domains for a 300+ user, multi-branch org. Microservices would multiply operational surface; an unstructured monolith would rot. The reference discipline (oyatie) layers `kernel ← domain ← application ← adapter ← {rest, worker} ← app`.

## Decision
Single deployable Rust binary from a Cargo workspace with one crate family per domain (`console-<domain>-{domain,application,adapter-postgres,rest,worker,ui}` as amended by ADR-0041), shared `console-kernel-core`, cross-cutting `console-platform-*` crates. A `console-<domain>-ui` crate may depend on the shared contracts crate and on other `ui` crates; `app` may depend on `ui`. Dependency direction is enforced twice: by crate visibility (the compiler refuses absent edges) and by a CI layer-boundary gate (T0.2) that fails on illegal edges and on `sqlx`/`axum`/`tokio` appearing in domain/application crates.

As amended by ADR-0045, that direction is Clean Architecture, not hexagonal-as-core-blob: Rest/Worker (controllers) depend on Application (use cases), Contracts, Platform, and Kernel; they do not depend on Domain or Adapter except via the shrink-only ratchet in the layer-boundary gate. Adapter (gateways) may still depend on Domain. A `Layer::Rest` crate must have a sibling `*-application` crate unless listed on that same ratchet.

## Consequences
+ Domain logic stays pure and exhaustively unit-testable; adapters are swappable; later service extraction is per-crate, not a rewrite.
+ One process to deploy/observe/back up on the single OCI VM.
− Workspace compile times grow with crate count; mitigated by per-crate `cargo test -p`.

## Alternatives considered
Microservices (rejected: disproportionate ops for team size); single-crate monolith (rejected: no compiler-enforced boundaries, rot risk).
