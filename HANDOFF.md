# HANDOFF — Post-pivot Console

## Start here

The canonical restart record is
[`docs/handoffs/2026-08-03-disk-wipe-consolidation.md`](docs/handoffs/2026-08-03-disk-wipe-consolidation.md).
It records the useful local work preserved before the disk wipe, rejected lanes,
ignored/Ultragoal artifacts, secret-recovery boundary, and exact fresh-clone
entrypoint. No old worktree or local branch is a continuation dependency.

## Authority and program

- Canonical product authority: [`docs/PIVOT-2026-07-28.md`](docs/PIVOT-2026-07-28.md).
- Current program: [`docs/program/README.md`](docs/program/README.md).
- Delivery method: [`docs/program/agentic-engineering-playbook.md`](docs/program/agentic-engineering-playbook.md).
- `main` is the sole integration branch; start from the latest `origin/main`.
- Machine-readable program state remains under `docs/program/`; the preserved
  planning execution status record is
  [`.omx/plans/reasoning-lens-contract-execution-handoff.json`](.omx/plans/reasoning-lens-contract-execution-handoff.json).

## Holds

No live production, DNS, TLS, secret, exposure, payment, credential-reset, or
compliance-claim action is authorized. The OCI Ampere A1 must never be destroyed,
terminated, resized, or reprovisioned. Korea compliance remains `HOLD` pending
qualified authority. Historical handoffs, branches, chats, and transient OMX/OMC
state are context only.
