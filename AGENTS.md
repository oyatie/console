# Repository invariants

- `README.md` is the sole entry point. Current product, roadmap, and delivery authority lives only in `docs/current/PRODUCT.md`, `docs/current/ROADMAP.md`, and `docs/current/DELIVERY.md`; proposed or conflicting plans are HOLD, not permission to expand scope.
- Keep one writer per root. Declare ownership, exact base SHA, immutable target, and mechanical guide before fan-out; serialize migrations, lockfiles, generated files, OpenAPI, CI, and authority records.
- Never use destructive shared-workspace Git operations or overwrite another lane's work. Preserve historical evidence.
- Do not skip, delete, quarantine, or weaken tests without an approved receipt and independent review. Record exact invocations and discovered/executed counts.
- Keep facts, inferences, hypotheses, and legal conclusions distinct. Production exposure and legal/compliance claims require separate authority and evidence.
- Every lane records pre-mortem, blast radius, detection, rollback, stop conditions, review identities, head SHA, and remaining HOLDs.

Detailed current method: `docs/current/DELIVERY.md`. The program playbook is retained as historical method and reusable reference only.

<!-- SHARED:REASONING-LENSES:START -->
## Task-selected reasoning lenses

All substantive reasoning, planning, implementation, review, and verification must use the smallest task-appropriate subset. Select at least two lenses before nontrivial work, re-evaluate the set when evidence or risk changes, and do not mechanically apply all lenses.

1. **Cartesian doubt** — challenge assumptions and separate evidence, inference, and uncertainty.
2. **Essentialism / YAGNI** — pursue the smallest sufficient outcome and avoid speculative scope.
3. **Chesterton's Fence** — understand why an existing constraint or mechanism exists before removing it.
4. **Contrarian / outside-the-box** — test non-obvious alternatives when the default framing may be wrong.
5. **Socratic** — expose hidden premises with focused questions; ask the user only when the answer materially blocks safe progress.
6. **Pragmatism** — optimize for the real-world outcome under actual constraints.
7. **Red Team** — model misuse, adversaries, hostile inputs, and ways the plan can fail.
8. **Systems Thinking** — trace dependencies, feedback loops, second-order effects, and system boundaries.
9. **Operability / Day-2** — account for deployment, diagnosis, maintenance, recovery, and ownership after launch.
10. **Opportunity Cost** — compare the chosen work against the best alternatives in time, complexity, and value.
11. **Blast-radius / cell-based** — contain changes and failures; prefer independently recoverable boundaries.
12. **Constant-work / anti-fragility** — avoid input-dependent blowups, degrade predictably, and use stress to improve the system.
13. **Shared-nothing / eventual consistency** — minimize coordination and make convergence, conflicts, and stale-state behavior explicit.
14. **FinOps / unit-cost** — reason about cost per useful outcome, including operational and scaling costs.
15. **Telemetry-first** — make important state, decisions, failures, and success criteria observable.
16. **Zero-trust / defense-in-depth** — verify every boundary, minimize privilege, and layer independent safeguards.

High-risk authz, migration, contracts, approval, HR/payroll, release, production, and compliance-sensitive work must include Red Team, Operability / Day-2, Blast-radius / cell-based, and Zero-trust / defense-in-depth, or record a lens-specific not-applicable rationale in durable evidence. Report concise conclusions, evidence, decisions, and tradeoffs rather than private chain-of-thought.
<!-- SHARED:REASONING-LENSES:END -->
