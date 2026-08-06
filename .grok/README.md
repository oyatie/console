# Console delivery harness (Grok workflows)

Process control plane for parallel backend roadmap delivery.  
**Not product authority** — that stays `docs/current/*`.  
**Not merge authority** — leader squash-merges.

## Bun doctrine

[Rewriting Bun in Rust](https://bun.com/blog/bun-in-rust): when agents fail a *class* of mistake, **edit a reusable workflow or tool** so the next agent cannot repeat it. See `programs/BUN-PARALLEL-DISCIPLINE.md`.

## Dual-track anti-wait (two workflows)

| Workflow | Role |
|----------|------|
| `ci-fleet-tick` | PR/CI classify, CLEAN for leader, fail URLs, tip-sync needs |
| `product-process-tick` | Product or process work **while** CI runs |
| `program-tick` | Runs **both** — forbids CI-only completion |

```text
/workflow program-tick
# or separately:
/workflow ci-fleet-tick
/workflow product-process-tick
```

## Other durable workflows

| Workflow | Role |
|----------|------|
| `domain-increment` | Backend lane SDLC + Admit phase |
| `process-upgrade` | Failure class → permanent control (process allowlist only) |

## Harness

| Path | Purpose |
|------|---------|
| `harness/failure-classes.v1.json` | Class catalog + detectors |
| `harness/learning-loop.v1.json` | When to promote process edits |
| `programs/briefs/console-backend-roadmap-durable.brief.md` | Durable goal + session prompt |
| `programs/ROADMAP-DURABLE-GOAL.md` | Operator spine |

## Fix the process checklist

1. Map red → `failure-classes.v1.json` id  
2. If control missing or class ≥2 → `/workflow process-upgrade` with `args.class_id`  
3. Edit **workflows/harness/tools**, not chat memory  
4. Never one-shot hooks named for a PR  
