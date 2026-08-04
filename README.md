# Console

Console is a Rust platform for a governed company object engine: **Ontology / Foundry / Policy → Company / OrgUnit / Employee → HR → Payroll**. This is the current product boundary; ERP, finance, communications, compliance products, ingest/evidence, office editing, AI judgment, and unrelated verticals are out of scope.

## Repository map
- `backend/` — Rust workspace, ontology, authorization, HR/payroll substrate, REST application, migrations.
- `docs/PIVOT-2026-07-28.md` — canonical product and repository truth.
- `docs/decisions/` — accepted decisions plus explicit amendment, supersession, and current-reconciliation history.
- `docs/program/` — current delivery method and machine-readable registers; historical records are labelled.
- `scripts/` — executable CI/preflight and verification gates.

## Supported local checks
From `backend/`: `cargo fmt --all --check`, `cargo check --workspace`, and targeted `cargo test` for changed crates. Run repository gate scripts named by the applicable CI workflow. Record the exact command, revision, toolchain, and test counts in the lane receipt.

## Authority
Precedence is Pivot, accepted ADRs consistent with it, current roadmap/pipeline, machine-readable registers, exact candidate evidence, then historical plans and runtime state. See `AGENTS.md` and `docs/program/agentic-engineering-playbook.md`.

<!-- SHARED:REASONING-LENSES:START -->
## Reasoning lens manifest

Canonical definitions and routing rules live in [AGENTS.md](AGENTS.md#task-selected-reasoning-lenses). This identifier-only projection is drift-checked and does not duplicate policy.

1. Cartesian doubt
2. Essentialism / YAGNI
3. Chesterton's Fence
4. Contrarian / outside-the-box
5. Socratic
6. Pragmatism
7. Red Team
8. Systems Thinking
9. Operability / Day-2
10. Opportunity Cost
11. Blast-radius / cell-based
12. Constant-work / anti-fragility
13. Shared-nothing / eventual consistency
14. FinOps / unit-cost
15. Telemetry-first
16. Zero-trust / defense-in-depth
<!-- SHARED:REASONING-LENSES:END -->
