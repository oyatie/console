# SPEC — Post-pivot Console

## Product boundary

Console builds one governed company object system in this order:

1. Ontology / Foundry-style object engine / deterministic Policy.
2. Company, OrgUnit, JobPosition, Person, and Employment.
3. HR appointment, promotion, and transfer as the canonical assignment writer.
4. Payroll projected from existing payroll truth as PayRun.
5. A Leptos SSR frontend only after every ADR-0030 substrate gate is freshly green.

ERP and finance modules, communications, compliance products, ingest/evidence, office editing, AI judgment, payment execution, and unrelated verticals are out of scope. Existing code in those areas is historical or maintenance-only and does not authorize expansion.

## Core invariants

- Deterministic commands, expected revisions, replay-safe receipts, auditable mutations, tenant isolation, and deny-by-omission.
- Effective-dated truth uses half-open intervals; history is closed and appended, never overwritten.
- Projected objects have exactly one domain writer. Ontology and adapters do not create alternate write paths.
- Requester and approver are distinct natural persons even when capacities differ.
- Preflight runs the same authorization, policy, state, revision, and input validation as execute and performs no mutation.
- Legal sources are versioned evidence, not transferable compliance conclusions. Production exposure requires separate authority.

## Build and verification

Cargo, sccache, per-lane target directories, and nextest are the target toolchain. Until the dedicated Cargo-convergence train proves equivalent coverage and deletes Buck paths, current CI remains evidence of repository reality rather than product direction.

Run targeted tests first, then formatting, clippy/type checks, relevant gates, and exact candidate verification. Record exact SHA, invocation, discovered/executed counts, environment, and artifact hashes in the lane receipt.

## Authority

[`docs/PIVOT-2026-07-28.md`](docs/PIVOT-2026-07-28.md) is canonical. Delivery details live in [`docs/program/console-enterprise-roadmap.md`](docs/program/console-enterprise-roadmap.md), [`docs/program/console-development-pipeline.md`](docs/program/console-development-pipeline.md), and [`docs/program/agentic-engineering-playbook.md`](docs/program/agentic-engineering-playbook.md).
