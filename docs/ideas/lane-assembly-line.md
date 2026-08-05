# Lane assembly line — the delta over the approved fan-out plan

> idea-refine output, 2026-07-29. **Read `docs/ideas/fanout-plan-DRAFT.md` first** — it is APPROVED
> and already specifies the reservation scheme (§5), the three CI gates that replace a human
> coordinator (§5.1), the structural guards (§6), the pipeline audit (§6.5) and the Bun mechanisms
> (§7). `docs/program/LANE-PROTOCOL.md` specifies the worktree pool, ownership and the landing model.
>
> This file records only what those two do **not** cover. Everything else was deliberately deleted
> rather than restated: §243 of the fan-out plan names cross-document restatement as an anti-pattern,
> having already cost 11 corrections when four planning docs disagreed about derived facts.

## Two premises retired

**"Make the 10-phase slice pipeline an assembly line where each phase moves to the next safe task."**
There is no barrier to remove. The slice script runs three fan-outs (Explore ×5, Design ×N, Prove ×2)
and seven strictly serial single-agent stages, each a bare `await agent(...)`. `pipeline()` over one
item is identical to sequential, and no worker is idle — each stage is a fresh agent that exits. The
chain is serial because `Implement` needs `Red`'s tests and the reviewers need a committed diff.
Overlapping the stages is the task fragmentation Bun grouped by crate to avoid, and LANE-PROTOCOL §6's
diff-only invariant forbids a reviewer starting before the author's diff exists.

**"The queue is starving because nobody feeds it."** Half right. On 2026-07-29 no second slice could
run beside the live one: the version-orphaning work collides with the policy-topology work on
`store.rs`, `rest/src/lib.rs` and the postgres adapter. That made "manufacture disjointness" look like
the hard problem. It is not — see the landing shape below, which makes collisions cheap instead of
forbidden.

## The delta: a permanently-open draft PR on the consolidation branch

LANE-PROTOCOL §194 and fan-out §220 both say *parallelise the work, serialise the landing* — lanes
collect onto one integration branch, one C with one T, one trip through the train per batch. Neither
says how that branch gets CI while it accumulates.

**Mechanism, verified in the trigger block:** `.github/workflows/ci.yml` fires on `pull_request:` for
all branches and `push:` only for `main`. So a **draft PR** on the consolidation branch runs all 14
required contexts on every push, cannot be merged by accident, and — because `strict: true` only
bites at merge time — produces **no rebase thrash while it is open**. #525 sat as a draft until it was
merged on 2026-07-29, so the pattern is already in use here.

Consequence: merging a lane into the consolidation branch costs **seconds** instead of a 20-minute CI
cycle plus an authority rebind. Collisions stop being a scheduling constraint the coordinator must
prevent and become ordinary git conflicts. Slices no longer need to be incapable of colliding — only
capable of merging cleanly, which git already answers mechanically.

**Assumption to validate before relying on it:** push a throwaway branch, open it as draft, and
confirm all 14 contexts execute. Then land a deliberately failing test on it and confirm it goes red.
A draft that silently skipped CI would recreate the false-green this repo has hit five times. Do not
infer either from the trigger block.

## Bun, from the primary source

Read directly at <https://bun.com/blog/bun-in-rust>, because the in-repo distillations had drifted.
Numbers quoted, not paraphrased.

| | Bun | Here, 2026-07-29 |
|---|---|---|
| Landing | one branch (`claude/phase-a-port`), 11 days, **one PR**, *"no incremental merges during the rewrite period"* | 8 PRs in ~19 h, one authority rebind each |
| Volume per PR | 6,502 commits, **+1,009,272** lines | #525: 23 files, +5,437 |
| Worktrees | **4** shards, one worktree each | 5 lanes exist, 1 in use |
| Agents | 16 per workflow, **~64 concurrent** at peak | 5 explorers, then 1 at a time |
| Loop | implementer → 2+ adversarial reviewers → **1 fixer** | 10 phases, **fixer specified in §7 but absent from the script** |
| Cost | 690 M output tokens, 5.9 B uncached input, **~$165,000**, 11 days | — |

Two things the primary source adds that the distillations lost:

- **Their answer to the never-executed test was a human.** *"I manually verified the tests were in
  fact running and not being skipped."* Not a gate. Five instances here in one week, and the reflex
  each time has been to add another gate.
- **Their loop is four stages; ours is ten.** Each of ours is defended in the ledger by a specific
  failure, so this is not an argument that six are waste — but the reference implementation shipped a
  million lines with four, and that asymmetry deserves an explicit decision rather than drift.

## Not doing

- **Long-lived per-stage workers** — a persistent reviewer accumulates the previous slice's author
  reasoning and carries it into the next review. LANE-PROTOCOL §6 credits diff-only isolation with
  catching 38 fabrications; this would delete that property while every run still looked green.
- **A GitHub merge queue** — one consolidation PR already serialises the exit.
- **Competing implementations** — 2× cost, marginal gain. Competing *designs* already earn their keep:
  the judge caught both #525 designs making the same conformance error.
- **64 concurrent agents** — Bun's scale had ~16,000 mechanical compiler errors to grind. The binding
  constraint here is review quality on security-sensitive slices, not throughput. Copying the agent
  count would copy the number and miss the reason.
- **A backlog steward that judges disjointness** — fan-out §5 already computes it from a reservation
  table, and §5.1's three CI gates already replace a human coordinator. A judging agent would be a
  second, softer mechanism next to a mechanical one.

## Open questions

- Is the pre-merge 390-reference rebind worth keeping at all? Fan-out §242 shows it binds a SHA that
  squash-merge destroys, while `bind-merged-console-authority-squash` already binds the surviving one
  post-merge (green on #506–#508). Its recommendation — one rebind per batch, not per lane — is
  unexecuted.
- How long may a consolidation branch live before it is a liability? Bun accepted 11 days with one
  human watching. Nothing here defines the ceiling.
- Should the slice script grow the fixer role that §7 already specifies?

## Measured 2026-07-30 — the pre-merge rebind bound a commit that no longer exists

`fanout-plan-DRAFT.md` §6.5 argues the ~390-reference pre-merge authority rebind is waste, because the
repo is squash-only so the commit the registers bind is destroyed by the very merge it authorizes, while
`bind-merged-console-authority-squash` binds the surviving SHA afterwards. It cited #506–#508 as evidence
and recommended reducing the pre-merge rebind to one per batch.

PR #526 supplied direct evidence on our own work:

- The train was built and verified: 390 references rebound to `f9ea7b7e0`, `authenticate-console-authority`
  **SUCCESS**, shape verified as T being C's single-parent child touching only the three authority documents.
- The PR squash-merged to `a9e51e7b7`. **`f9ea7b7e0` is not in `main`.** Every one of the 390 bindings now
  points at a commit reachable only from a merged branch ref.
- `bind-merged-console-authority-squash` reported **SUCCESS** on close, binding the surviving squash SHA.

So both halves of §6.5's claim are now measured rather than inferred: the pre-merge rebind is paid to bind a
doomed commit, and the post-merge binder — which is the real provenance anchor — works. The recommendation
remains unexecuted, and this is the second time in one session it cost real effort.

The cost was never the rebind itself, and as of 2026-08-01 there is no rebind: the candidate SHA left
both registers and `rebind-authority-train.mjs` is deleted. The cost is that **every
PR needs a signed authority tip whose content is a governance claim**, which is a human decision per PR
rather than per batch. That is what makes it a per-lane tax rather than a per-batch one, and it is the
concrete argument for the consolidation-branch shape this document proposes.
