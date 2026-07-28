# Delegation economics — codex / agy in the fan-out

**Status: PENDING APPROVAL — planning artifact.** Date: 2026-07-28
Feeds §7 of `docs/ideas/fanout-plan-DRAFT.md`.

Every number below is labelled by how it was obtained. Nothing here is from training recall.

## 1. Local ground truth (verified by execution, 2026-07-28)

| Fact | Value |
|---|---|
| `codex` | `codex-cli 0.145.0` at `~/.local/bin/codex` |
| `codex` auth | **ChatGPT subscription** (`codex login status` → "Logged in using ChatGPT") |
| `codex` default model | `gpt-5.6-sol`, `model_reasoning_effort = "medium"`, 250k context (`~/.codex/config.toml`) |
| `codex exec` flags | `-m/--model`, `-s/--sandbox {read-only,workspace-write,danger-full-access}`, `-C/--cd`, `--json`, `--skip-git-repo-check` |
| `agy` | `1.1.7` at `~/.local/bin/agy` |
| `agy` models | `gemini-3.6-flash-{high,medium,low}`, `gemini-3.5-flash-*`, `gemini-3.1-pro-{high,low}`, `claude-sonnet-4-6`, `claude-opus-4-6-thinking`, `gpt-oss-120b-medium` |
| `agy` flags | `--print`, `--model=`, `--effort={low,medium,high}`, `--output-format={text,json,stream-json}`, `--json-schema`, `--sandbox`, `--mode={plan,accept-edits}` |
| prior use in repo | 4 mentions in the program ledger; `OMX_TEAM_WORKER_LAUNCH_ARGS` already pins `--model gpt-5.6-sol -c model_reasoning_effort="xhigh"` |

### Two operational traps, both hit while testing

1. **`agy --model <value>` silently falls back to the default model.** The space-separated form is
   accepted and ignored; only `--model=<value>` binds. Measured: `--model gemini-3.6-flash-high`
   answered *"You are currently using Gemini 3.1 Pro"* — which is the **worst model on the DeepSWE
   board** (12% pass, $79/success). A silent downgrade to the worst available model, with a
   confident-looking answer, is the false-green class. **Always use `--model=`, and always have the
   agent state its own model in the reply.**
2. **`agy --print` cannot read files without permission.** Headless mode auto-denies and returns
   *"no output produced"*. Needs an allow-rule in `settings.json` or `--dangerously-skip-permissions`.
   Prefer a scoped allow-rule; if skipping, pair with `--sandbox`.

## 2. DeepSWE v1.1 (verified — fetched from deepswe.datacurve.ai)

Long-horizon, original SWE tasks. Sorted by **cost per success** (`$/task ÷ pass rate`), which is the
honest metric when failures are retried.

| model | pass@1 | $/task | **$/success** | steps | out tok |
|---|---|---|---|---|---|
| grok-4.5 [high] | 54%±2% | $2.42 | **$4.48** | 61 | 36k |
| gpt-5.6-luna [max] | 67%±4% | $3.03 | **$4.52** | 102 | 73k |
| kimi-k3 [max] | 69%±5% | $4.65 | $6.74 | 98 | 81k |
| gpt-5.6-terra [max] | 70%±3% | $4.95 | $7.07 | 76 | 72k |
| gemini-3.6-flash [high] | 49%±5% | $3.53 | $7.20 | 108 | 97k |
| gpt-5.5 [xhigh] | 67%±6% | $7.23 | $10.79 | 82 | 46k |
| **gpt-5.6-sol [max]** | **73%±3%** | $8.39 | $11.49 | **61** | **60k** |
| claude-opus-5 [max] | 74%±4% | $11.84 | $16.00 | 99 | 118k |
| claude-opus-4.8 [max] | 59%±2% | $13.22 | $22.41 | 120 | 135k |
| claude-fable-5 [max] | 70%±4% | $21.63 | $30.90 | 88 | 119k |
| claude-sonnet-5 [max] | 54%±4% | $26.40 | **$48.89** | **268** | 214k |
| gemini-3.1-pro [high] | 12%±2% | $9.48 | **$79.00** | 81 | 196k |

### What actually matters for us — and it is not the dollar column

**Step count is a safety metric, not a speed metric.** Our entire fan-out risk model is collision and
drift: a lane touching a file outside its slice. Every step is a tool call and another chance to do
that. `gpt-5.6-sol` completes at **61 steps / 60k output tokens** against `claude-opus-5`'s 99 / 118k
for a statistically indistinguishable pass rate (73±3 vs 74±4 — the error bars overlap). Roughly half
the surface area for the same result.

**`claude-sonnet-5` is the trap.** The instinct "use Sonnet for the cheap mechanical lanes" picks the
**worst** option on the board: 54% pass, 268 steps, 214k output tokens, $48.89/success — 11× worse
than luna. Do not reach for it here.

**Subscription vs metered inverts the cost column.** `codex` is authenticated by ChatGPT
subscription, so its marginal cost is bounded by plan rate limits rather than metered per task. The
DeepSWE dollars are API pricing and are the wrong basis for our decision.

**The strongest economic argument is budget-pool separation, not price.** This session lost **2 of 7
agents in one workflow and 1 of 6 in another to session limits** — and in the first, the casualty was
the *only adversarial verifier*, so a lane shipped unverified. Work sent to `codex` draws from a
different pool than the one that keeps dying. That is a reliability argument that happens to also be
cheaper.

## 3. Other benchmarks (REPORTED — not verified by me)

LiveBench and Artificial Analysis are client-rendered; I could not extract their tables directly and
these come from secondary reporting. Treat as directional.

- **Claude Fable 5 — #1 on LiveBench Coding at 86.0.**
- **Artificial Analysis Coding Agent Index:** Sol 80, Terra 77, Luna 75.
- **Terminal-Bench 2.1:** Sol 88.8% (91.9% ultra), Fable 5 88.0%, Luna 84.3%, Opus 4.8 82.7%,
  Sonnet 5 80.4%.

**The split these reveal is the useful part.** Fable 5 tops *authoring* benchmarks (LiveBench Coding
86.0) while sitting second-worst on DeepSWE cost-per-success ($30.90). Strong at writing code,
expensive at finishing long autonomous tasks. Sol inverts it: fewer wins on authoring benchmarks,
best-in-class on completing long-horizon agentic work efficiently.

That maps cleanly onto our phases: **authoring judgment in phases 1–2, autonomous completion in
phase 3.**

## 3.5 Runtime reality — measured, and it overrides everything below

Three delegation paths were tested against the identical bounded task. **Only one currently works.**

| path | result | cause |
|---|---|---|
| **Claude subagents** (`Agent` tool) | **works** — the Architect and Critic passes this session both landed and both found real defects | the proven baseline |
| **`codex exec` CLI** | **sealed** — every tool call denied | `~/.codex/config.toml` sets `hooks = true` with **oh-my-codex** conductor state stuck at *"no usable canonical session cwd"*; codex reports *"the guard rejects all tools before execution"*. Persists with `DISABLE_OMC=1` — that variable governs OMC, not oh-my-codex |
| **`agy --print`** | **hangs / drops stdout** in non-TTY | upstream bugs [#318](https://github.com/google-antigravity/antigravity-cli/issues/318) (hangs headless) and [#76](https://github.com/google-antigravity/antigravity-cli/issues/76) (silently drops stdout on pipe/subprocess) — exactly the shape a Workflow subprocess uses |

**codex is not slow — it was blocked.** Unblocked for one call it answered in **~20 s**, correctly, and
its answer independently confirmed `0204:87`:
`v_digest := public.digest(pg_catalog.convert_to(p_manifest::TEXT, 'UTF8'), 'sha256');`
It then wandered into `omx ralplan preflight`, was denied, and stalled. My earlier ">8 min, zero
bytes" reading was **environment contamination, not model latency** — a second wrong conclusion from
the same experiment, corrected only by reading stderr.

### RESOLVED 2026-07-28 — codex now works. Three blocking layers, peeled in order.

Owner authorized the config change. Each layer had to be removed separately; each *alone* left codex
still sealed, which is why the earlier readings looked like model latency.

| # | Layer | Symptom | Fix |
|---|---|---|---|
| 1 | **oh-my-codex `PreToolUse` hook** | `PROVENANCE_DENIED: Conductor mode is active … no usable canonical session cwd` — every tool call denied | `enabled = false` on the `~/.codex/hooks.json:pre_tool_use:0:0` entry in `~/.codex/config.toml`. **Only that entry**; SessionStart, PostToolUse, notify and the status line untouched. Backup: `config.toml.bak-20260728-preToolUse` |
| 2 | **`developer_instructions`** | *"Blocked by the mandatory preflight stop condition"* — the system prompt orders `omx ralplan preflight --json` before any work and stops | override **per call**: `-c developer_instructions='""'` |
| 3 | **`Stop` hook** | answers correctly, then wanders into *"stale/foreign working-directory session pointer"* recovery | disable **per call**: `-c features.hooks=false` |

**Measured result: ~17 min sealed / zero output → ~10 s, correct, `EXIT=0`, two agent messages.**
The answer independently confirms `0204:87`.

### The delegation invocation contract

```bash
codex exec --model <model> --sandbox read-only --skip-git-repo-check --json \
  -c features.hooks=false -c developer_instructions='""' \
  -C <lane-worktree> "<task>" < /dev/null > out.jsonl 2> err.log
```

Every element is load-bearing, each established by a failed run:

- **`< /dev/null`** — without it codex blocks on `Reading additional input from stdin...`
- **redirect to a file, not a pipe** — pipes are the non-TTY shape that hid output
- **`--json`** — structured events; the answer is an `item.completed` / `agent_message`, machine-readable without scraping prose
- **`-c features.hooks=false`** — bypasses the oh-my-codex layer *for the call*, leaving global config alone
- **`-c developer_instructions='""'`** — strips the OMX orchestration prompt so the agent does the task, not orchestration
- **`--sandbox read-only`** for reviewers and analysts; `workspace-write` only for implementer lanes
- **`-C <lane-worktree>`** — never the main checkout

**For Workflow integration prefer [`@openai/codex-sdk`](https://www.npmjs.com/package/@openai/codex-sdk)
v0.145.0** (matches the installed CLI exactly, Node ≥18) — same JSONL protocol, without shelling out.
The flags above map to its options.

**Note the global fix has independent value:** that guard was denying *every* codex tool call, so
interactive codex in this repo was broken too, not just delegated runs.

## 3.6 What Bun's lesson actually says about model choice

The naive reading is "spend the best model on the biggest fan-out." Bun's experience says the
opposite, and it is the load-bearing insight for this whole section.

**Bun's 64 agents were safe because the harness removed the ability to design wrong** — a
known-correct `.zig` reference, an immutable test suite, and `PORTING.md`. Sumner's instruction was
literally *"do the rewrite that looks like we transpiled our Zig code to Rust."* Safety came from the
harness, not from model capability. They did not use a smarter model for the parallel phase; they
made the parallel phase mechanical.

Two consequences invert the intuitive allocation:

1. **The design phase deserves the strongest model, and it is the phase nobody parallelizes.** Phases
   1–2 build the harness that does not exist yet — the conformance suite and the OrgUnit reference.
   Get these wrong and every downstream lane transliterates a defect faithfully. Bun spent 3 hours
   and PR #30224 serially before fanning out, for exactly this reason.
2. **The mechanical phase is where cheap models become safe — *because* of the harness.** Once
   `CATALOG.md` plus a hand-built OrgUnit reference plus an immutable suite exist, a lane is not
   designing. That is what licenses routing phase 3 by cost and step-count rather than raw capability.

**Corollary for routing: the metric is not pass rate, it is staying in lane.** DeepSWE measures
autonomous task completion; it does not measure whether an agent touched a file outside its slice —
the only property this fan-out's disjointness depends on. Step count is the closest available proxy
(`gpt-5.6-sol` 61 steps vs `claude-opus-5` 99 for indistinguishable pass rates), which is why §6's
calibration measures **files-touched-outside-lane** directly instead of trusting either number.

**Where model diversity pays is review, not implementation.** Bun's reviewers got the diff only and
were told to assume it was wrong. Adding cross-family review to that is the one place capability
diversity beats capability: blind spots correlate *within* a family. This session is the evidence —
an independent reviewer killed a premise I had verified twice, and a second reviewer with a different
lens caught that my first repair fixed one of three commands.

## 3.7 Effort tiers change the answer more than model choice does

Earlier revisions of this document routed on `[max]` alone. CursorBench 3.2 publishes all five effort
tiers, and the picture inverts.

**Max is a bad deal for the frontier models.** Measured, cost per success:

| model | best usable tier | vs its own Max |
|---|---|---|
| Opus 5 | **Low** 62.8% $2.55, 37 steps | Max costs **2.9× more per success** for +7.2 pts |
| Fable 5 | **Low** 62.1% $4.46, 31 steps | Max costs **3.4× more** for +8.4 pts |
| Sol | **Med** 60.0% $1.95, 27 steps | Max costs **2.6× more** for +7.2 pts |
| Terra | **Max** 64.9% $2.89, 47 steps | lower tiers fall below the usability floor |
| Luna | **Max** 61.1% $1.97, 61 steps | same |

*(Usability floor = 60%. Below it, retries plus reviewer time dominate — a plausible-wrong diff is not
free, it costs two adversarial reviewers and a fix cycle.)*

**Effort fragility differs sharply, and it decides which tier is safe to drop to:**

| model | Max → High | verdict |
|---|---|---|
| Opus 5 | −3.3 pts, 78→48 steps, $8.23→$3.91 | **robust** — High is the sweet spot |
| Sol | −3.7 pts, 48→**32** steps | **robust**, and the lowest-step credible option |
| Fable 5 | −4.0 pts, 72→48 steps | robust but expensive at every tier |
| Luna | −4.3 pts | mid tiers collapse below the floor |
| **Terra** | **−10.7 pts** (64.9→54.2) | **fragile — run Terra at Max/XHigh or not at all** |

**Two specific traps:** `Luna Max` burns **88k tokens at 61 steps** for 61.1%, versus XHigh's 22k at
48 — 4× the tokens for 3.4 points; never use it. And `Grok 4.5` is excluded from routing entirely:
CursorBench discloses its scores *"benefited from unintentional inclusion of earlier Cursor codebase
snapshots in training."*

## 3.8 What each model is actually good at — from benchmark divergence

The two benchmarks measure different things, and the **rank divergence** is the task-fit signal:

| model | DeepSWE (long-horizon autonomy) | CursorBench (ambiguous multi-file) | specialism |
|---|---|---|---|
| Opus 5 | **74 · #1** | 70.0 · #2 | autonomy-leaning, strong at both |
| Sol | 73 · #2 | 67.2 · #3 | **autonomy specialist** |
| **Fable 5** | 70 · #3 | **70.5 · #1** | **ambiguity/judgment specialist** |
| Terra | 70 · #4 | 64.9 · #4 | balanced |
| Luna | 67 · #5 | 61.1 · #5 | balanced |

**Fable is the only model that ranks higher on ambiguity than autonomy.** CursorBench's task mix is
edit, refactor, bugfix, codebase understanding, planning, **code review**, instruction following. That
is precisely the adversarial-reviewer job. Sol inverts it — better at sustained unsupervised
completion than at ambiguous judgment, which is precisely the mechanical-lane job.

## 3.9 Our tasks, classified — and the axis that actually matters

Difficulty is the wrong axis. **Verification cost** is the one this session proved decisive: a wrong
*fact* is spotted on sight; a plausible-wrong *diff* costs two reviewers plus a fix cycle.

| task | ambiguity | verification cost | route |
|---|---|---|---|
| **Fact lookup** — "does file X declare Y", "highest migration" | none | instant, on sight | **do it in-session.** Delegation overhead is a ~150× loss (measured: >5 min vs ~2 s) |
| **`.gitattributes`, collateral file edits** | none | instant | in-session; not worth a delegation round-trip |
| **Catalog type transliteration** (Phase 3 lanes) | **removed by the harness** — CATALOG.md + OrgUnit reference + immutable suite | reviewer catches it | **Sol High** — 32 steps, lowest collision surface of any credible tier |
| **CI gate crates** (~380 lines, `rls-arming` as reference) | low — a reference exists | tests catch it | **Sol High / Opus 5 High** |
| **Migration authoring** | low, but blast radius is high | expensive if wrong | **Opus 5 High** — robust at that tier, and errors are costly |
| **Adversarial diff review** | **high** — must find what the author couldn't | the whole point | **Fable 5**, cross-family from the implementer. It tops the ambiguous/review benchmark |
| **Conformance suite design** (Batch 3) | **high** — defines the immutable target | catastrophic if wrong; everything downstream aims at it | **Opus/Fable, high effort.** Few tasks; cost is irrelevant here |
| **OrgUnit reference** (Batch 4) | **highest** — every later type copies it | a defect propagates to all 4 types | **highest effort available**, 2 cross-family reviewers |
| **Debugging a red gate** | high — this session took 3 layers to peel | reasoning-bound, not volume-bound | **Opus/Fable high effort**; cheap tiers reason shallowly here |

### DeepSWE DOES measure Rust — extracted, and it reorders the tables above

**Correction to a claim I repeated without checking:** "no accessible benchmark measures Rust" is true
for LiveBench and **false for DeepSWE**. Its task set is 113 problems:

| ts | go | python | **rust** | js |
|---|---|---|---|---|
| 35 | 34 | 34 | **5** | 5 |

Per-task rollouts are reachable at `/data/v1.1/tasks/<id>` (HTML, ~1 MB each). The five Rust tasks are
real projects — `boa`, `fd`, `oxvg`, `pest`, `wasmi` — 20 trials per model.

**Rust-only results invert the global ranking:**

| model | Rust | global DeepSWE |
|---|---|---|
| **claude-fable-5 [max]** | **85%** | #3 |
| claude-opus-5 [max] | 85% | #1 |
| claude-opus-5 [xhigh] | 80% | — |
| claude-fable-5 [xhigh] | 75% | — |
| claude-opus-5 [high] | 70% | — |
| **gpt-5.6-sol [max]** | **60%** | **#2** |
| **gpt-5.6-terra [max]** | **55%** | #4 |

**Claude takes all five top slots.** Fable — which I had routed to "authoring only, never agentic" —
ties for first on Rust. Sol, which I made the primary implementation lane, falls to tenth.

**Statistical honesty: no pairwise comparison is significant at n=20.** Fable-max vs sol-max is
p=0.155; opus-max vs terra-max p=0.082 (suggestive). So this **reorders priors; it does not settle
lanes.** The family-level clustering across five independent configurations is the stronger signal,
not any single pair.

**One claim it does settle.** Claude effort on Rust is **monotonic** — opus-5 max 85 > xhigh 80 >
high 70 > medium 60 — which **contradicts the LiveBench-derived claim that max effort makes Claude
worse at agentic work** (high 61.6 > xhigh 61.3 > max 59.2). On the benchmark ranked most credible,
for the language that is 93% of this repo, more effort is monotonically better. Do not route Claude
down-tier on that LiveBench claim.

**Residual caveat:** the five tasks are parser/engine-heavy (a JS engine, a CLI finder, an SVG
optimiser, a PEG parser, a wasm interpreter). This repo is web services, SQL, and an ontology engine.
Domain transfer is unproven.

### Rust economics at every effort tier — the routing table that matters

DeepSWE's per-trial records carry `cost_usd`, `n_agent_steps` and `outcome`. Joined across the five
Rust tasks (20 trials per configuration), **cost per success on Rust**:

| model · effort | rust % | $/task | **$/success** | steps |
|---|---|---|---|---|
| terra · medium | 26% | 0.66 | **2.51** | **29** |
| **opus-5 · low** | **53%** | 2.60 | **4.95** | 53 |
| terra · xhigh | 50% | 2.91 | 5.82 | 58 |
| sol · medium | 50% | 3.02 | 6.03 | 48 |
| **opus-5 · medium** | **60%** | 5.27 | **8.79** | 79 |
| **sol · xhigh** | **63%** | 7.01 | 11.10 | **63** |
| opus-5 · high | 70% | 7.97 | 11.39 | 96 |
| luna · max | 60% | 7.21 | 12.02 | 154 |
| opus-5 · xhigh | 80% | 11.93 | 14.92 | 119 |
| terra · max | 55% | 8.23 | 14.97 | 104 |
| **opus-5 · max** | **85%** | 14.87 | 17.49 | 129 |
| fable-5 · xhigh | 75% | 20.42 | 27.22 | 99 |
| **sol · max** | **60%** | 15.86 | **26.43** | 91 |

**Four conclusions that change lanes:**

1. **`sol [max]` is strictly dominated on Rust.** 60% at $26.43/success — worse pass rate *and* worse
   cost-per-success than `opus-5 [max]` (85%, $17.49). My "Sol for the implementation lane" was a
   global-ranking artifact that does not survive language conditioning.
2. **`opus-5 [low]` is the efficiency find:** 53% at **$4.95/success**, 53 steps. The cheapest route
   to >50% on Rust, by a wide margin. This corroborates the CursorBench "Max is a bad deal" finding on
   a different benchmark and the right language.
3. **Above a 60% usability floor, the honest choices are narrow:** `opus-5 medium` (60%, $8.79,
   79 steps) is cheapest; **`sol xhigh` (63%, $11.10, 63 steps) has the fewest steps** — which is the
   *collision-safety* metric for fan-out, not a speed metric.
4. **Buying the top tier costs 3.5× for +32pp.** `opus-5` low→max is $4.95→$17.49 for 53%→85%.
   Justified only where a wrong answer is expensive to detect — the design phases, not the lanes.

**`terra [medium]` at $2.51/success is the cheapest number on the board and a trap:** 26% pass means
~4 attempts, and each failed attempt produces a plausible-wrong diff that consumes two adversarial
reviewers. Cheap only where verification is instant.

**Under subscription the dollars compress but the steps do not.** Claude and codex run on seats, so
`$/success` is closer to a rate-limit proxy than a bill — but **step count still measures collision
surface**, and it is the axis that does not go away.

### QUALITY IS NOT NEGOTIABLE — and that inverts the table above

This is production. Cost-per-success is the wrong objective function on the shipping path, and the
preceding table optimises it. Re-derived with quality as a constraint rather than a variable:

| model · effort | Rust % | production-viable? |
|---|---|---|
| **opus-5 · max** | **85%** | **yes — the ceiling** |
| **opus-5 · xhigh** | **80%** | **yes** |
| fable-5 · xhigh | 75% | yes |
| opus-5 · high | 70% | marginal |
| sol · xhigh | 63% | no — 1 in 3 wrong |
| opus-5 · low | 53% | **no** — was my "efficiency find" |
| terra · medium | 26% | **no** — was the cheapest number on the board |

`terra medium` at $2.51/success and `opus-5 low` at $4.95 are the two most attractive entries in the
economics table and **both are disqualified**: 26% and 53% mean the majority (or near half) of diffs
are wrong. Every wrong diff consumes two adversarial reviewers and a fix cycle, and — worse — a
*plausible* wrong diff on a production path is the failure mode this whole session has been fighting.

**Under subscription, the marginal cost of `opus-5 max` is ~zero.** So there is no economic argument
for running production lanes below the ceiling. Economics governs where verification is instant —
census, lookups, sweeps — and **never the shipping path.**

### Marginal analysis — what the NEXT effort tier buys (Rust, measured)

Average cost-per-success hides the decision. The marginal question is *what does the next tier cost
per additional percentage point*:

| model | tier step | Δpass | Δ$ | **$ per +1pp** |
|---|---|---|---|---|
| opus-5 | low→medium | +7pp | +2.67 | 0.38 |
| opus-5 | medium→high | +10pp | +2.70 | **0.27** |
| opus-5 | high→xhigh | +10pp | +3.96 | 0.40 |
| opus-5 | xhigh→max | +5pp | +2.94 | 0.59 |
| **sol** | **medium→high** | **+0pp** | +2.44 | **pure waste** |
| **sol** | **xhigh→max** | **−3pp** | +8.85 | **pay more, get LESS** |
| terra | xhigh→max | +5pp | +5.32 | 1.06 |
| fable-5 | medium→high | +5pp | +4.61 | 0.92 |

**`opus-5` is the only model whose every tier step is economically rational on Rust** — a flat
$0.27–0.59 per point all the way to max. **`sol` inverts twice**: medium→high buys nothing, and
xhigh→max *costs 3 points*. `sol max` is dominated by `sol xhigh` on both axes.

### Iteration economics — and the trap in them

| model·effort | p(1) | p(2) | p(3) | attempts for 95% | E[$] to 1st success |
|---|---|---|---|---|---|
| terra·medium | 26% | 45% | 59% | **10** | **2.54** |
| opus-5·low | 53% | 78% | 90% | 4 | 4.91 |
| opus-5·medium | 60% | 84% | 94% | 4 | 8.78 |
| opus-5·high | 70% | 91% | 97% | 3 | 11.39 |
| opus-5·xhigh | 80% | 96% | 99% | **2** | 14.91 |
| **opus-5·max** | **85%** | **98%** | **100%** | **2** | 17.49 |

On expected spend alone, **the weakest model wins**: `terra·medium` reaches 95% confidence for $2.54
against `opus-5·max`'s $17.49. **Do not act on that**, for three reasons:

1. **Retries are almost certainly correlated.** A model that fails a task tends to fail it again.
   These are *upper bounds*; independence is assumed, not measured.
2. **Iteration requires DETECTABLE failure.** Without the conformance suite you cannot tell attempt 1
   failed, so k=1 is forced and single-shot pass rate is the only number that matters. **The harness
   does not merely raise quality — it is what makes iteration economically possible at all.**
3. **The decisive one: more attempts = more gate-escape exposure.** "Retry until the gate passes"
   samples until something *satisfies the gate*, which selects for diffs that fool an imperfect gate
   rather than diffs that are correct. That is automated overfitting to the gate — the same failure
   class as "never repair a gate by making it pass," except no human is in the loop to notice.
   `terra·medium` needs 10 attempts to `opus-5·max`'s 2: **5× the exposure.**

**So high single-shot pass rate is worth paying for even when retries look cheaper** — it minimises
the number of chances a wrong diff gets to slip past an imperfect gate. That is the quantitative form
of "quality is not negotiable."

### What we must start recording (not derivable from any benchmark)

Published numbers give single-shot p only. The production questions need our own telemetry:

- `attempts_to_success` per work item — gives the **real** k distribution
- per-attempt outcome, so **retry correlation** can be measured instead of assumed
- **gate-escape events**: a diff that passed the gate and was later found wrong. This is the number
  that decides whether cheap-plus-iterate is ever safe here, and no benchmark can supply it
- steps per attempt, cumulative — collision surface compounds across retries

### The ceiling finding, which matters more than any routing choice

**The best available configuration fails 15% of Rust tasks.** `opus-5 max` gets 17 of 20. Model
selection moves you from 55% → 85%; it cannot move you to production quality **at any price or any
effort tier**.

So the 15% is not a model problem to be shopped around — it is the exact gap the harness exists to
close:

- the **immutable conformance suite**, which a lane cannot edit
- **cross-family adversarial review**, which caught a false premise its author had verified twice
- **probe-red-first**, which catches the case where the grader itself is the defect
- **isolation measured, not assumed** — 0 of 4 lanes out-of-slice

**Quality comes from the gates, not the model.** That is also Bun's actual lesson: 64 agents were safe
because the harness removed the ability to design wrong, not because the model was strong enough to
be trusted. Buying a better model raises the floor; only the harness raises the ceiling.

### ⚠️ The language gap — this invalidates the tables above for most of this repo

**Measured:** console is **479,339 lines of Rust (93% of executable code)**, against 25,922 JS/MJS and
9,506 Python. LiveBench's agentic subset covers **javascript, typescript, python only**; neither
DeepSWE nor CursorBench publishes a Rust split.

**So every table above extrapolates from ~5% of this repo to the 93%.** Routing keyed on
`(family, effort, task_class)` is under-specified — **language belongs in the key.**

### First graded Rust evidence on console — and the grader was the defect

Probe: *"is `attributes` ever UPDATEd in `ontology/adapter-postgres/src/instances.rs`, or is state a
fold? List every UPDATE's columns."* Ground truth established by execution earlier in the session.

| model | verdict | columns | time |
|---|---|---|---|
| `gpt-5.6-sol` | **FOLD** ✓ | lifecycle_state, updated_at, valid_to, current_revision_id, updated_at | 13s |
| `gpt-5.6-luna` | **FOLD** ✓ | *(identical)* | 11s |

**Both scored 2/2 — and my ground-truth label was wrong.** I had recorded three UPDATE columns and
omitted `updated_at`, which genuinely appears in two of the three statements (`:275`, `:1285`). Had I
not re-verified, I would have logged a false negative against both models.

**New failure mode, named:** a graded probe is only as good as its grader. The probe's *label* needs
the same red-first scrutiny as the probe's *code*. This is the fourth defective probe I have produced
in one session — `.length` on an object, `root//pkg:name` vs `//pkg:name`, zsh 1-indexed arrays, and
now an incorrect ground truth.

**What this probe does and does not establish:** both families comprehend real Rust control flow
correctly in ~12s, which retires the worry that absent-from-benchmarks implies absent-competence. It
**does not rank them** — both were perfect, so it does not discriminate. The next Rust probe must be
one a weaker model is expected to fail.

### The correction Bun's lesson forces on all of the above

**CursorBench measures *ambiguous* tasks. Phase 3 is deliberately *unambiguous*** — the harness exists
precisely to remove design. So these scores **understate cheaper tiers for our mechanical lanes**, and
the gap should narrow. That is Bun's lesson quantified: they did not use a smarter model for the
parallel phase, they made the parallel phase mechanical.

**This is a prediction, and §6's calibration is where it gets tested.** Route by measured
files-touched-outside-lane, not by these tables.

## 4. Recommended allocation

**Revised for effort tiers and task-fit divergence (§3.7–3.9).** The earlier version routed on `[max]`
only and is superseded — Max is a 2.6–3.4× cost-per-success penalty on the frontier models.

| Role | Model · **effort** | Why |
|---|---|---|
| **Conformance suite + OrgUnit reference** (Batches 3–4) | **Opus 5 · Max/XHigh** or **Fable 5 · XHigh** | Highest-ambiguity work; a defect propagates to all four later types. Few tasks, so cost is genuinely irrelevant — the one place Max is justified. |
| **Phase 3 lane implementer** | **Sol · High** (`gpt-5.6-sol`) | **32 steps** — lowest collision surface of any credible tier, and Sol is the *autonomy* specialist (DeepSWE #2). Ambiguity is already removed by the harness. Separate budget pool. |
| **Migration authoring / gate crates** | **Opus 5 · High** | 66.7% at 48 steps for $3.91 — robust at High (−3.3 pts), and mistakes here are expensive. |
| **Adversarial reviewer #1** | **Fable 5 · High/XHigh** | **The ambiguity specialist** — CursorBench #1, and code review is one of its measured categories. Finding what the author could not is exactly this axis. |
| **Adversarial reviewer #2** | **Sol · XHigh** via `codex exec` | **Cross-family is the point** — blind spots correlate within a family. Proven twice this session: codex killed a premise I had verified twice; a Claude critic then killed the fix. |
| **Debugging a red gate** | **Opus 5 / Fable 5 · high effort** | Reasoning-bound, not volume-bound. Took three peeled layers this session; cheap tiers reason shallowly here. |
| **Fact lookup / trivial edits** | **in-session, no delegation** | Measured >5 min vs ~2 s. A ~150× loss. |
| **Never** | `Luna · Max`; `Terra · High` or below; `Sonnet 5`; `gemini-3.1-pro` | Luna Max burns 88k tokens for 61.1%. Terra is effort-fragile (**−10.7 pts** Max→High) so its cheap tiers fall below the floor. Sonnet 5 and gemini-3.1-pro are the worst cost-per-success on either board. |
| **Excluded from routing** | `Grok 4.5` | CursorBench discloses training contamination from earlier Cursor codebase snapshots. |

### Delegation contract (mandatory for every delegated task)

Carried over from the failures measured this session:

1. **State the model in the reply.** Guards against `agy`'s silent fallback.
2. **`--sandbox read-only` unless the task is an implementation lane.** Reviewers and lookups never
   need write access.
3. **`-C <lane worktree>`** — never the main checkout. Shared-checkout contamination was measured:
   four lanes in one tree, every one reporting foreign scope violations.
4. **Reproduce the original failure, not the artifact.** A delegated repair must show red→green on the
   *symptom*, not on the file it touched.
5. **Every probe proven RED on a known-bad input** before its GREEN is trusted.
6. **Write findings to disk as you go.** Three agents went idle without ever returning a report this
   session; do not rely on the final message.

## 5. Measured latency — and the recommendation it refuted

**Experiment.** Identical bounded task to both CLIs: *read one migration file, state whether the
sha256 is over a parameter or the whole catalog, cite the line, answer in under 40 words.* A task I
had already answered in-session with one `awk` in roughly **2 seconds**.

| tool | invocation | elapsed | output |
|---|---|---|---|
| `codex exec` | `--model gpt-5.6-sol --sandbox read-only` | **>8 min** | **zero bytes** — never returned |
| `agy` | `--model=gemini-3.6-flash-high --sandbox --dangerously-skip-permissions --print` | **>6 min** | **zero bytes** — never returned |

Both were still running, having emitted nothing at all, when the experiment was closed out. These are
lower bounds on a task that takes ~2 s in-session.

**This refutes §4's "bounded factual lookup → agy" row, which is now retracted.** I wrote that
routing from leaderboard capability numbers without measuring the thing that actually decides it.
Pass rate says which tool *can* answer; latency says whether delegating is worth it at all. For a
2-second `awk`, a 5-minute round trip is a ~150× loss no accuracy figure can recover.

**Corrected principle: delegation overhead is roughly fixed and large, so it only amortises over
long-horizon batched work.** That is precisely the regime DeepSWE measures and precisely what a
phase-3 lane is. It is the opposite of a lookup.

**Rule:** if the task is answerable by a single tool call in-session, do it in-session. Delegate when
the unit of work is a whole lane slice, not a fact.

*(Caveat, stated rather than assumed: these are lower bounds from processes that had not yet
returned, on first invocation, possibly including cold-start and sandbox setup. They are enough to
kill the lookup use case; they are not a steady-state throughput measurement for lane work, which
§6 must establish separately.)*

## 6. Required calibration before trusting any of this

Per the standing instruction — **experiment and verify rather than trust model routing.** §4 is a
hypothesis. Before phase 3 routes real work by it, run the calibration on **one already-solved
slice** (OrgUnit, from phase 2, whose correct answer is known):

1. Give the identical lane brief to `gpt-5.6-sol`, `gpt-5.6-terra`, and Claude.
2. Measure: wall-clock, whether it stayed inside its slice (**files touched outside the lane** — the
   metric that actually matters here, not pass rate), whether it reproduced the failure rather than
   the artifact, and whether its probe went red on a known-bad input.
3. Route by **measured** results. Discard §4's table where it disagrees.

The leaderboard cannot answer the question that decides this fan-out — *does this model stay in its
lane* — because DeepSWE does not measure collision. §5 already shows one leaderboard-derived
recommendation dying on contact with a stopwatch.

## 7. Open / unverified
- **`agy` billing model is unknown** — whether it draws from a separate pool like codex, or is
  metered, is unestablished and changes its economics.
- LiveBench / Artificial Analysis figures are secondary reporting (§3), not extracted from source.
- DeepSWE measures *long-horizon autonomous* tasks. Phase-3 lane work is deliberately **mechanical
  transliteration against a fixed reference and immutable target** — the regime where cheaper models
  should close the gap, because the design judgment has been removed. That is Bun's actual lesson.
  **The allocation above is a starting hypothesis; the two-lane dry run (§10.3 of the fan-out plan) is
  where it gets tested.**
