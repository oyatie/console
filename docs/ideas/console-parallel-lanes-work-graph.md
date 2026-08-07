> **QUARRY / NON-AUTHORITY.** Planning graph for multi-agent dispatch only. Cannot dispatch product work, clear HOLDs, or override `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md`. Executable projections: Beads issues + `.grok/harness/work-graph.v1.json` (process).

# Console parallel lanes — work-order graph

## Problem statement

How might we structure the entire Console completion backlog as a dependency-ordered work graph whose nodes are narrow, path-disjoint agent lanes, so we can fan out safely without tip thrash, dual writers, HOLD clearance, or sprawl?

## Recommended direction

**Phase-gated, cell-parallel, tip-serial (V8 hybrid).**

- Consumer: multi-agent dispatch  
- Lane success: path-disjoint allowlist + `must_read` + admit green + PR or stacked commit  
- Specialists: `.grok/bin/mm-role` → `codex exec` / `claude -p` only  
- Tip-serial paths are a **global mutex** (manifest/ledger/baseline/ci.yml/lock/OpenAPI)

## Product completion order

```text
Ontology + policy
  → Company → OrgUnit → JobPosition → Person/Employment
  → HR single assignment writer
  → PayRun projection
  → Leptos SSR (after ADR-0030 / H1)
```

## Phases and lanes

| Phase | Lanes | Parallelism |
|-------|--------|-------------|
| **P0** Unblock | L0-611 docs fence merge; L0-SSF close beads epic | yes |
| **P1** Substrate | L1-PG-PART, L1-JS-REACH, L1-CARGO-MEM, L1-MIG-PARSE, L1-BUCK-RES, L1-PROC; L1-DOC-INDEX tip-serial if needed | high within phase; one tip writer |
| **P2** Arch | L2-POL (console-a80), L2-APR (console-66n), L2-CAP, L2-API (OpenAPI lease) | 3–4 after P1 |
| **P3** Ontology | L3-ONT-CELL cells; L3-SSF-CLOSE | path-disjoint crates |
| **P4** Ports H2 | L4-PORT-DES serial; then L4-PORT-{CO,OU,JP,PE,EM,PR} parallel | design then fan-out |
| **P5** Org/HR/Pay | L5-ORG, L5-JOB → L5-HR serial → L5-PAY | gated by ports |
| **P6** UI H1 | L6-ADR → L6-SSR → L6-UI-* | all HOLD-gated |
| **P∞** Always | L∞-PROC process; L∞-HOLD-PREP custody | never product expansion |

Full machine graph: `.grok/harness/work-graph.v1.json` (process checkout). Beads: created from `.grok/harness/work-graph.beads.json`.

## Lane packet contract

```text
lane_id, objective, phase, depends_on, allowlist (prefer ≤3),
forbidden (docs/current/**, foreign writers), must_read, hold_touch,
tip_serial, verification[], admit, success, not_doing,
hindsight_recall (attempt), hindsight_retain (on land/class≥2)
```

## Hindsight (cross-session)

- **Recall** before inventing soft-red/process/fleet state: Hindsight MCP when up; else `bd` + lane-board + project memory.
- **Retain** after land / class≥2: class_id, lane_id, head SHA, control path (Hindsight or project memory fallback).
- Fail open if MCP down — do not block product admit solely on Hindsight outage.
- Config: `[mcp_servers.hindsight]` → `http://localhost:8888/mcp/`.

## Key assumptions

- [ ] Crate path-disjoint ≈ merge-safe except tip-serial set  
- [ ] Admit + CI enforce allowlists (agents alone will drift)  
- [ ] Port design can wait until after P1–P2 substrate  
- [ ] Empty own-author GitHub `reviewDecision` is process, not code  

## MVP (first executable wave)

P0 + ready P1 lanes with full packets + L2-POL/L2-APR admissions. No P5–P6 implementation.

## Not doing

- ERP/comms/compliance/ingest/AI product expansion  
- Bulk docs graveyard (ROADMAP HOLD)  
- Multi-tip product PRs  
- Grok native Sol/Claude models  
- UI before H1  
- Dual writers for projected types  
- New sprawl assessment docs each week  

## Open questions

- Exact allowlists for L1-PG-PART / L1-CARGO-MEM (fill on first admission pack)  
- Whether L2-CAP shares types with L2-POL (lease shared crate if yes)  
