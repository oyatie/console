# Console delivery harness (Grok-native)

Process control plane for parallel backend roadmap delivery.  
**Not product authority** — that stays `docs/current/*`.

**Does not use** `omc`, `omx`, `gjc`, or `hermes` CLIs. Ideas are incorporated; implementations live under `.grok/`.

## Ideas absorbed

| Source | What we took |
|--------|----------------|
| [Bun rewrite](https://bun.com/blog/bun-in-rust) | Prep contract, dual adversarial review, **edit the process** |
| OMC **ultragoal** | Durable `goals.json` + `ledger.jsonl` + native `/goal` handoff |
| OMC **ralplan** | Planner → Architect → Critic until APPROVE before heavy execute |
| OMC **ralph** | PRD story loop + “boulder never stops” Stop-hook loop |
| [Hermes Agent](https://github.com/NousResearch/hermes-agent) | Closed learning: curated memory, skill drafts, trajectories, persist nudges |
| Oyatie `.grok` | mm-learn / dual-track / soft-red queue patterns |

## Autonomy (default)

Under human **supervision** (intervene only if awry):

1. **`console-drive`** runs the Bun control plane: soft-red tools → fleet fix/merge → **product implement** → process-edit → learn.
2. Soft reds are **inputs to fix loops**, not the product of the drive.
3. When a *class* of drive failure recurs, **edit** `console-drive.rhai` / harness / `tools/ci/*` (not chat memory).
4. Agent review **APPROVE** + Required CI/Security → merge (`autonomy-merge.v1.json`).
5. **Stop hook** loop-back while ultragoal active → re-dispatch `/workflow console-drive`.

## Primary entry (implementation, not board-only)

```text
/workflow console-drive
```

Optional: `{ "skip_product": true }` · `{ "allow_merge": false }`

## Ultragoal + native `/goal` + hooks

```text
# 1) Inject durable plan + arm loop
/workflow ultragoal {"action":"activate","objective":"Drain tip-serial queue and land Wave 0 pure tests","workflow":"console-drive"}

# 2) Session-native goal (Grok)
/goal <same objective printed in handoff>

# 3) Drive implementation
/workflow console-drive
# vague/large first:
/workflow ralplan {"task":"…"}
/workflow ralph

# 4) Learn → process-upgrade if class repeats
/workflow learn
```

Hooks (project trust required — `/hooks-trust`):

| Event | Script | Effect |
|-------|--------|--------|
| `SessionStart` | `bin/console-hook-session-start` | Injects active ultragoal context |
| `Stop` | `bin/console-hook-stop` | **Blocks stop** while `active-goal.live.json.active`; instructs `/workflow …` dispatch |

```json
// .grok/hooks/ultragoal-loop.json
```

CLI helpers:

```bash
.grok/bin/console-goal status|activate|deactivate|checkpoint|handoff
.grok/bin/console-learn from-event --id … --summary '…' --classes 'ops.soft-red-silence'
```

## Workflow catalog

| Workflow | Role |
|----------|------|
| **`console-drive`** | **Implementation control plane**: soft-red tools → fleet fix/merge → product build → process-edit → learn |
| **`program-control`** | Lighter meta heartbeat (prefer console-drive) |
| **`ultragoal`** | Durable goals + activate hook loop + `/goal` handoff |
| **`ralplan`** | Consensus planning (Planner/Architect/Critic) |
| **`ralph`** | PRD story execution loop until APPROVE |
| **`learn`** | Hermes-style reflection/tip/skill promotion |
| **`work-manager`** | Board + soft reds/blocks (no silence) |
| **`implement-lane`** | Claim one item → PR |
| **`pr-babysit`** | Repair → review → merge |
| `program-tick` | Legacy dual-track |
| `domain-increment` | Full backend SDLC + Admit |
| `process-upgrade` | Failure class → permanent control |

## Layout

```text
.grok/
  bin/           console-goal console-learn console-hook-*
  hooks/         ultragoal-loop.json   (SessionStart + Stop)
  ultragoal/     goals.json ledger.jsonl active-goal prd progress brief
  harness/       autonomy-merge lane-board learning-loop hermes-learning ultragoal
  workflows/     *.rhai
  memory/        MEMORY.md reflections/ tips/ trajectories/
  skills/        learned skill drafts
  programs/      BUN-PARALLEL-DISCIPLINE ROADMAP-DURABLE-GOAL
```

## Soft reds & blocks

Every soft red/block → `harness/lane-board.live.json` (`ops.soft-red-silence` if dropped).

## Fix the process

1. Map red → `failure-classes.v1.json`
2. `/workflow learn` then `/workflow process-upgrade` if class ≥2
3. Edit workflows/harness/tools — not chat memory
4. Never one-shot hooks named for a PR
