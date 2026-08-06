# Durable goal brief — Console backend roadmap (closed-loop)

**Artifact class:** durable multi-session goal for Grok workflows + Beads  
**Not merge authority.** Leader squash-merges when CLEAN.  
**Orchestration:** `.grok/workflows/*` only for drive loops.  
**Doctrine:** [Bun rewrite](https://bun.com/blog/bun-in-rust) — edit reusable workflows/tools so mistakes cannot recur; trial before fan-out; dual-track while CI runs.

---

## Shared constraints (every goal inherits)

### A. Mission

Implement the **Console backend product roadmap** end-to-end under live `docs/current/PRODUCT.md` + `ROADMAP.md` + `DELIVERY.md`: substrate → architecture foundations → org/HR ports → payroll projection → backend unlocks for frontend HOLD — via small, path-disjoint, merge-safe increments. Not a single-session tasklist.

### B. Authority (fail-closed)

1. PRODUCT / ROADMAP / DELIVERY are sole active product law.  
2. Historical plans, chats, and session todos are queue signals only.  
3. HOLDs: implement prerequisites + evidence; **stop**; never clear by implication.  
4. Out of product scope: ERP expansion, comms verticals as product claims, live prod/DNS/TLS, Korea compliance conclusions, frontend UI until HOLD clears.

### C. Dual-track anti-wait (two workflows)

**Problem:** agents fall into endless "still waiting for CI" loops.

**Rule:** every autonomous wake runs **both**:

| Workflow | Owns |
|----------|------|
| `ci-fleet-tick` | Open PRs, exact-head CI/Security, CLEAN list for leader, fail URLs, BEHIND tip-sync needs |
| `product-process-tick` | Backend lane or process-upgrade work that does **not** require waiting on that CI |

Orchestrator: **`program-tick`** runs both legs. Completing only fleet with `waiting_ci` and no product/process action is **ops.passive-wait** (fail closed).

### D. Merge autonomy

- Agents: open PR, babysit, report CLEAN, restack when BEHIND.  
- **Leader only:** `gh pr merge --squash`.  
- No mid-run push thrash while Required CI in_progress.

### E. Process closed loop (not product)

On hosted/local red:

1. Map to `.grok/harness/failure-classes.v1.json` class.  
2. If control missing or class repeats ≥2 → **`process-upgrade`** (allowlist: workflows/harness/tools/ci/scripts).  
3. Forbidden: one-shot hooks named for a PR; expanding product scope to "work around" process pain.

### F. Parallelism

- Serialize: authority tip, ci.yml/preflight, migrations, Cargo.lock, OpenAPI faces, docs/current.  
- Parallel: path-disjoint backend crates after prep pack; process-upgrade vs product when paths disjoint.  
- Trial before multi-lane fan-out.

### G. Delivery mechanics (every implement slice)

1. Worktree from current `origin/main`.  
2. Admission + allowlist.  
3. `domain-increment` including **Admit** phase.  
4. Dual review on high risk.  
5. Signed C/T when authority/docs tip requires.  
6. One push → wait complete run.  
7. `program-tick` while waiting.  
8. Leader merge → tip-sync remaining PRs.

---

## Goal graph

@goal: G001 Substrate and Required CI health
Keep Required / CI + Security meaningful; preflight fail-closed; authority train healthy; docs custody intact. Drain open process/substrate PRs with dual-track ticks.

@goal: G002 Cargo-first product tests (DN-0005)
Equivalence residual gate → nextest serial groups → switch residual Buck product jobs → drop Buck product CI steps (files retained). Faces carved out. No Buck-as-driver-until-RE.

@goal: G003 Process intelligence (always-on)
Maintain failure-classes, learning-loop, dual-track workflows; land process-upgrade PRs on repeat classes. Never "done."

@goal: G004 Architecture foundations
Branchless capability / temporal grants / true preflight / distinct-human approval as PRODUCT item 4 — backend only.

@goal: G005 Ontology backend increments
Wave ontology/foundry backend under allowlist; no UI.

@goal: G006 Org/HR ports and single-writer
Owning ports + proofs; projection fan-out remains HOLD until conditions met.

@goal: G007 Payroll projection
Project PayRun from existing writer; no second write path.

@goal: G008 Backend unlocks for frontend HOLD
Contracts/gates/SSR prerequisites only; do not claim Leptos UI complete under HOLD.

@goal: G009 Human-blocked ops
Prod/secrets/erase: prepare evidence only; human_blocked.

---

## Session prompt (paste each wake)

```text
You are driving Console durable goal: full backend roadmap under PRODUCT/ROADMAP/DELIVERY.

HARD RULES
- Leader-only merge. You open/babysit/report CLEAN.
- HOLDs fail-closed; never clear by implication.
- Dual-track every wake: run /workflow program-tick (ci-fleet-tick + product-process-tick).
  If CI is waiting, you MUST still advance product or process work. Pure WAIT is forbidden.
- Fix process via reusable workflows/harness/tools (process-upgrade), not one-shot hooks.
- One writer per root; serialize tip/ci.yml/migrations/lockfile.
- Authority: docs/current only for product law.

START
1) bd prime; bd ready
2) /workflow program-tick
3) If clean_merge_ready → report for leader merge
4) If waiting_ci → execute product-process dispatch (domain-increment or process-upgrade)
5) If failed → map failure-classes.v1.json → fix + process-upgrade if class control missing

Doctrine: .grok/programs/BUN-PARALLEL-DISCIPLINE.md
Brief: .grok/programs/briefs/console-backend-roadmap-durable.brief.md
```
