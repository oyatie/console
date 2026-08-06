# Console backend roadmap — durable goal (operator view)

**Brief (session paste):** [`briefs/console-backend-roadmap-durable.brief.md`](briefs/console-backend-roadmap-durable.brief.md)  
**Process doctrine:** [`BUN-PARALLEL-DISCIPLINE.md`](BUN-PARALLEL-DISCIPLINE.md)  
**Authority:** `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md` only.

## Interview locks (2026-08-06)

| Knob | Choice |
|------|--------|
| Horizon | **Full backend roadmap** (ROADMAP 1→6 + backend unlocks for 7) |
| HOLDs | Prerequisites → evidence → **stop**; never clear by implication |
| Merge | **Leader-only** squash-merge |
| Closed loop | Fix **process** (workflows/harness/tools), not invent product scope |
| Anti-wait | **Two workflows**: `ci-fleet-tick` + `product-process-tick` (orchestrated by `program-tick`) |

## Dual-track anti-wait (required)

```text
Every autonomous wake / program-tick:
  ┌─────────────────────┐     ┌──────────────────────────┐
  │  ci-fleet-tick      │     │  product-process-tick    │
  │  CI/PR classify     │     │  product OR process work │
  │  CLEAN for leader   │     │  NEVER pure "wait CI"    │
  │  fail URLs / behind │     │  domain-increment or     │
  │  tip-sync report    │     │  process-upgrade         │
  └──────────┬──────────┘     └────────────┬─────────────┘
             │  if waiting_ci              │
             └──────────── must pair ──────┘
```

**Endless waiting-for-CI** is a **process defect** (class `ops.passive-wait`).  
Fix: edit workflows so a tick that only narrates WAIT is incomplete (`program-tick` always runs both legs by default).

## Workflow catalog (edit these to fix process)

| Workflow | Purpose |
|----------|---------|
| **program-tick** | Orchestrator: fleet + product/process in one run |
| **ci-fleet-tick** | Fleet half only |
| **product-process-tick** | Product/process half only (pair when fleet waits) |
| **domain-increment** | One backend lane SDLC + **Admit** phase |
| **process-upgrade** | One failure class → permanent control |

## Goal spine (G001–G009)

```text
G001 Substrate / docs custody + Required CI health
G002 Cargo-first product tests (DN-0005 sequence; residual Buck cutover)
G003 Process intelligence always-on (this harness; never "done")
G004 Architecture foundations (policy, preflight, distinct-human)
G005 Ontology / object engine backend increments
G006 Org/HR owning ports + single-writer (HOLD-gated fan-out)
G007 Payroll projection (no second writer)
G008 Backend unlocks for Leptos HOLD (contracts/gates — not UI claim)
G009 Human-blocked ops (prod/secrets/erase — prepare only)
```

## Activate (session start)

```bash
cd ~/Developer/console
bd prime
bd ready
# optional: pin brief into session / Hindsight
# Every wake:
#   /workflow program-tick
# While CI red/waiting: do not stop after fleet — product-process leg is mandatory
```

## Durable goal prompt (copy-paste)

See brief § Shared constraints + § Session prompt.
