# Bun rewrite lessons — Console delivery process

Canonical process rules for multi-PR / multi-lane Console delivery.  
**Doctrine source:** [Rewriting Bun in Rust](https://bun.com/blog/bun-in-rust) — prep first, trial before fan-out, dual adversarial review, **edit the process** so agents cannot repeat the mistake.

Workflows (edit these when fixing process):

| Workflow | Role |
|----------|------|
| `program-tick` | Fleet heartbeat + failure-class retro + next actions |
| `domain-increment` | One backend lane SDLC; admit before handoff |
| `open-pr-fleet` | Open-PR status, tip-sync needs, anti-passive wait |
| `process-upgrade` | Map a failure class → harness/workflow/tool PR only |

Authority remains `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md`. This file is **process**, not product.

## What "fix the process" means

**Means:** edit a **reusable workflow / harness / tool** so the next agent cannot make the same *class* of mistake without a hard fail, forced step, or explicit waiver.

| Fix class | Do this | Not this |
|-----------|---------|----------|
| **Workflow** | New phase in `domain-increment` / `open-pr-fleet`; admit gate; tip-sync step | Chat note "remember to run lens check" |
| **Tool / script** | Extend `npm run check:*` / `admit` / residual gate | One-line fix on one PR and stop |
| **Harness catalog** | Add row to `failure-classes.v1.json` + control path | One-shot hook named for a PR number |
| **Memory tip** | `.grok/memory/tips/{class}.md` after repeat ≥2 | Session-only folklore |

**Does not mean:** expanding product scope, clearing PRODUCT HOLDs, auto-merge to main (Console: **leader-only** merge), or inventing features to avoid a process gate.

When a mistake class repeats **twice**, promote a process edit (see `learning-loop.v1.json`).

## Six Bun rules (Console)

1. **Prep contract first** — admission + OWNERSHIP/allowlist before multi-agent write.
2. **One representative trial before fan-out** — do not open N domain lanes while the trial path is red.
3. **Dual split-context review** — implementer ≠ reviewer context (`domain-increment` Review phase).
4. **Fail closed on missing evidence** — no handoff without admit green; no "complete" without Required CI on exact head when claiming merge-ready.
5. **Edit process when systematic** — if agents keep doing X, change workflow/tool so X fails closed.
6. **Do not expand while trial red** — no second CI graph writer while #ci tip is red; no product fan-out without prep pack.

## Systematic mistakes → required process edits

| Class ID | Observed | Control (must live in tool/workflow) |
|----------|----------|--------------------------------------|
| `auth.unsigned-tip` | C/T not pinned SSH after rebase | Signed train checklist in handoff; authority bootstrap |
| `lens.noncanonical-json` | Hosted preflight lens fail | `check-reasoning-lens-contract` in admit for ledger paths |
| `docs.tip-blob-prebind` | Manifest ≠ tip blob | `check:doc-manifest` on tip content before push |
| `ci.verify-job-ids` | verify.mjs / preflight digests | `check:ci-preflight` + `test:verify` |
| `ci.residual-buck-growth` | New Buck wrappers in ci.yml | `check:product-buck-residual` |
| `ops.passive-wait` | Watch-only CI turns | `program-tick` / `open-pr-fleet` anti-passive; dual track |
| `ops.mid-run-push` | Push while CI in_progress cancels cone | Fleet rule: one push → wait complete |
| `ops.missed-tip-sync` | PR BEHIND after main moves | `open-pr-fleet` tip-sync report (leader restacks) |
| `ops.skip-admit` | Push without local gates | PreToolUse / admit phase in workflows |

## Anti-passive rule

`waiting_ci` is **not** idle. Every autonomous wake must: merge-report · fix · restack/preflight · fan-out ready work · or advance tracker.  
Unchanged WAIT narration alone is forbidden (Hindsight `autonomous-drive-no-passive-watch`).

## Leader merge (Console difference from Oyatie)

Agents **open PRs, babysit, report CLEAN**. Humans/leader **squash-merge**. Workflows must never call `gh pr merge` unless the operator explicitly overrides in a future process edit with dual-SSOT.

## Parallelism

- **Serialize:** authority tip, `ci.yml` / preflight digests, migrations, Cargo.lock, OpenAPI faces, `docs/current/**`.
- **Parallel:** path-disjoint backend crates after prep pack; process-upgrade PRs vs product PRs when paths disjoint.
