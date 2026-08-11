# Console / Oyatie / Grok → Cursor process synthesis

Date: 2026-08-09  
Sources: Console study, Oyatie study, mm-* study, Cursor hooks map, existing plan `cursor_agentic_parity_8fa3c09c` (chat evidence; not in-repo paths).

## Diagnosis

Multi-round PR fix loops in this program were **process failures**, usually one of:

1. **Incomplete inventory** — fixed the last critic's finding, not all already-visible threads (`process.known-blockers-skipped`).
2. **Wrong convergence** — treated unproven bot opinions as merge bars after every push (`process.bot-thread-treadmill`).
3. **Wrong transport** — `mm-role` CLI critics (auth workaround) instead of native Task (`transport.mm-role-default`).
4. **Missing restack / forge preflight** — tip PRs conflicted or `gh` auth died mid-merge.

It is less often "the critic failed to catch everything at once." Catching A while B was already on the PR, then opening a round for B, then a round for new unproven C, **is** the bug.

See `.cursor/failure-classes-2026-08-10.md` for the measured table and controls.
## What to keep (portable process)

| Pattern | Origin | Cursor artifact |
|---------|--------|-----------------|
| Proof-gated convergence (`major` blocks only if proven) | lane-fanout `isBlocking` | rules + critic agent + receipt validator |
| Standing lenses non-overridable | lane-fanout `STANDING_LENSES` | `.cursor/rules/console-standing-lenses.mdc` |
| Schema-required fields | BUILD_SCHEMA | `scripts/cursor/validate-lane-receipt.mjs` |
| Third spelling → mechanism | BASE_LOCK | `console-base-lock.mdc` |
| One writer / path-disjoint | oyatie parallelism + lane-fanout | rules + git-lock hook |
| Trial before scale | oyatie `deliver.js` / console-complete | base-lock + sessionStart |
| Failure class → edit process | oyatie/console `process-upgrade` | edit rules/hooks, not chat memory |
| Soft-red silence forbidden | work-manager / lane-board | board + push admission |
| Tip-serial mutex | work-graph | planner discipline |
| Role split (plan ≠ implement ≠ critic) | mm model-routing | `.cursor/agents/*` + orchestrator rule |

## What to drop (transport accidents)

- `.grok/bin/mm-role` / `claude -p` / `codex exec` as default critic path → **hook-denied** unless `CURSOR_ALLOW_MM_ROLE=1`
- Rhai workflow nesting as Cursor runtime
- `MM_ROLE_OK` harvest lines
- Credential sync for shelling out of Grok

## Installed Cursor ratchet (this change)

```
.cursor/
  hooks.json
  hooks/          # hard gates
  rules/          # alwaysApply BASE_LOCK + standing lenses
  agents/         # lane-implementer, lane-critic
  receipts/       # schema-valid JSON
  PROCESS-SYNTHESIS.md
scripts/cursor/validate-lane-receipt.mjs
```

Hard vs soft:
- **Hard:** hooks deny destructive git, mm-role, `--workflow-only`; push/PR admission; stop follow-up without valid receipt
- **Soft:** alwaysApply rules + agent prompts
- **Medium:** receipt JSON must validate before "done"

## Operator notes

- Escape hatches: `CURSOR_ALLOW_MM_ROLE=1`, `CURSOR_ALLOW_GIT_DANGEROUS=1`, `SKIP_LOCAL_ADMISSION=1`
- Land process files on a **chore/cursor-swarm-ratchet** PR from clean `main` once product trains quiesce — do not squash process into P4 product PRs
- Extend existing `cursor_agentic_parity` plan: add BASE_LOCK content (done here), keep shared `tools/hooks/` migration as follow-up

## Orchestrator observations (2026-08-10 wave-1)

Measured while running Beads-to-Zero; controls landed in agents/rules/scripts the same day.

1. **Local ≠ landed.** Admission commits that are not on `origin/admission/*` do not exist for CI or reviewers. Run `scripts/cursor/check-admit-sync.sh` in the admit worktree after every rebuild; unpublished tip = soft-red, not babysit-idle.
2. **Lanes are local-first by design.** Individual `lane/*` branches usually are not pushed; evidence lives in worktrees + receipts. Custody/prune must not delete a worktree whose leaf is unadmitted.
3. **Signing is setup, not a nice-to-have.** Smoke `git commit -S` before product. Admission re-sign is a safety net, not the plan (`process.lane-unsigned-product`).
4. **Fmt can lie about gates.** If CI fails fmt first, gate tests did not run — require local gate pins on the tip you push (`process.ci-fmt-masks-gates`).
5. **Class findings need sweep beads.** Prototype-chain / trait-default / unsigned release candidates are classes; filing only the instance recreates the treadmill (`i91`, `h3e`, `9ry`).
6. **Hook fix ⇒ doc fix same turn.** `process.doc-vs-hook-drift` burned a round; BASE_LOCK/agents must move with the hook.
7. **Receipts ride the lane.** Hub copies go stale within one fix round (`process.receipt-location-drift`).
8. **Train scope freeze is real.** Do not fold train-2 leaves (ann/soe/mbl/…) into an open wave-1 PR mid-convergence — new admit train after merge.
9. **Release-please vs authority train.** Bot candidates cannot carry trusted SSH signatures (`process.release-candidate-unsigned` / bead `9ry`) — pick a mechanism before the next version bump.
10. **Babysit-only is banned.** When waiting on CI/admit push, keep ≥1 productive lane/critic/audit in flight.
11. **Worktrees under the hub.** Cursor External-File Protection treats sibling checkouts (`../console-lane-*`) as outside the workspace when the hub folder is open → constant allow-edit prompts. Provision only via `scripts/cursor/provision-lane-worktree.sh` / `<hub>/.worktrees/<name>` (`process.worktree-outside-workspace`). Do not rely on multi-root `.code-workspace`.
