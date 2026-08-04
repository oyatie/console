# Context — reasoning lens contract

## Task
Implement the approved task-appropriate reasoning-lens contract in the existing post-pivot worktree, but the user then explicitly invoked `$ralplan`, so produce a durable consensus plan only and do not edit source files.

## Desired outcome
Make sixteen named lenses a repository-wide reasoning-routing system. `AGENTS.md` is canonical; `README.md` and `CLAUDE.md` contain compact drift-checked manifests. Every nontrivial task selects at least two lenses. High-risk work includes Red Team, Operability/Day-2, Blast-radius/cell-based, and Zero-trust/defense-in-depth unless a reasoned not-applicable exception is recorded. Persist concise decision evidence, never private chain-of-thought.

## User-locked decisions
- Risk-based subset, not all sixteen on every task.
- Applies to all nontrivial planning, investigation, implementation, and review reasoning.
- Canonical definitions in `AGENTS.md`; compact manifests in `README.md` and `CLAUDE.md`.
- Minimum two lenses; mandatory high-risk core as described above.
- The contract must survive in all three root files.

## Sixteen lenses
Cartesian doubt; Essentialism/YAGNI; Chesterton's Fence; Contrarian/outside-the-box; Socratic; Pragmatism; Red Team; Systems Thinking; Operability/Day-2; Opportunity Cost; Blast-radius/cell-based; Constant-work/anti-fragility; Shared-nothing/eventual consistency; FinOps/unit-cost; Telemetry-first; Zero-trust/defense-in-depth.

## Current repository facts
- Worktree: `/Users/jasonlee/Developer/console-post-pivot`, branch `codex/post-pivot-wave0`, based on `origin/main` `9200e875b`.
- The worktree already has an uncommitted Wave-0 truth-reconciliation diff; preserve it.
- Root guidance and the playbook do not yet contain the sixteen-lens contract.
- Four retrospective templates exist under `docs/retros/templates` and have no lens evidence block.
- Existing doc-link gate and CI wiring are uncommitted in this worktree.
- `package.json` and the documentation CI job are the natural validator integration points.

## Constraints
- Ralplan planning-only boundary: write only `.omx` planning/context/state artifacts.
- Existing ignored worktree content is discovery evidence, not authority.
- No product API or database changes.
- Grandfather historical ledger records; validate new/materially revised artifacts through a v1 marker.
- Machine checks must not demand or expose private chain-of-thought.
