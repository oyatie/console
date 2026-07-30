# Parallel Agentic Development in Production: What Practitioners Actually Report

Status: RESEARCH — sourced, confidence-labelled

Compiled 2026-07-29. Every substantive claim carries a URL, a date, and a confidence label:

- **CONFIRMED** — primary source read directly; quote or close paraphrase.
- **LIKELY** — multiple credible secondary sources agree; primary not read.
- **UNCERTAIN** — single secondary source, or the source's own framing is hedged.
- **UNKNOWN** — the public record does not answer this.

Also labelled throughout: **[DID]** = someone did it and measured it. **[SAYS]** = someone recommends it without reported evidence of having done it. The second category is close to worthless for our purposes and is marked so it can be discounted.

---

## 1. The Bun baseline, corrected against the primary source

Source: <https://bun.com/blog/bun-in-rust> — read directly, twice, on 2026-07-29. PR: <https://github.com/oven-sh/bun/pull/30412>, merged 2026-05-14.

### Numbers that are correct as you have been citing them

| Claim | Status |
|---|---|
| 11 days (2026-05-03 → merged 2026-05-14) | **CONFIRMED** |
| 6,502 commits excluding merges | **CONFIRMED** |
| +1,009,272 lines net diff landed | **CONFIRMED** |
| One PR, no incremental merges | **CONFIRMED** — PR #30412, base `main`, head `claude/phase-a-port`, all commits merged (not squashed) |
| 4 worktrees × 16 agents ≈ 64 concurrent | **CONFIRMED, verbatim**: "At peak, we were running 4 of these workflows at once each in a separate worktree, each with 16 Claudes per workflow. About 64 Claudes at a time." |
| 1 implementer + 2 adversarial reviewers + 1 fixer | **CONFIRMED** |
| Reviewers see the diff and nothing else | **CONFIRMED, verbatim**: reviewer context is "only the diff. told to assume the code is wrong" — the implementer's context ("the .zig original, the port plan, its own reasoning") is not shared with them |
| ~16,000 compiler errors | **CONFIRMED**: "≈16,000 errors left" |
| 3-file trial run, then the rest of 1,448 | **CONFIRMED**: "I started with just 3", then "I asked Claude to loop the workflow on all 1,448 .zig files" — so 1,445 others, as you have it |
| Progressive ladder: cargo check → `bun --version` → one test file → ~100 sharded random files → full CI on all platforms | **CONFIRMED** |
| 0 tests skipped or deleted | **CONFIRMED**: "0 tests skipped or deleted" |
| Author manually verified tests were actually running | **CONFIRMED, verbatim**: "(and I manually verified the tests were in fact running and not being skipped)" |
| `git stash` / `git reset` banned | **CONFIRMED**, with the incident: "one Claude ran `git stash` before committing. Another ran `git stash pop`. And then `git reset HEAD --hard`. They were stepping on each other!" |
| "If you need a paragraph-long comment to justify why the workaround is OK, the code is wrong — fix the code" | **CONFIRMED, verbatim** |
| ~3 hours up front producing `PORTING.md` + `LIFETIMES.tsv` | **CONFIRMED**: "about 3 hours talking to Claude about how to map patterns" |
| `systemd-run` / cgroups for test isolation | **CONFIRMED**: "we used `systemd-run` (cgroups) to limit memory & CPU usage and isolate pid namespaces" |
| 5.9B uncached input, 690M output, ~$165,000 at API pricing | **CONFIRMED** |

### Four corrections. Two of them matter.

**Correction 1 — the crate rule is not what you have been saying. This is the one to fix.** **CONFIRMED.**

You have been citing: "~16,000 compiler errors fixed **by crate, never by file**, explicitly to prevent task fragmentation." Two of those three parts are wrong.

The post says: "We went crate-by-crate." And then, describing the loop: **"For each crate, run `cargo check`, group the output by file and save the errors to a file. Fix all the compiler errors within that crate. 2 adversarial reviewers for the crate's changes. 1 fixer applies the fixes."**

So errors *were* grouped by file — as bookkeeping *inside* a crate. The correct statement of the rule is: **the crate is the unit of agent assignment and review; the file is the unit of error bookkeeping within it.** Not "never by file."

And the stated rationale is not fragmentation. It is, verbatim, **"To maximize parallelism, the workflow looped over each crate"** and **"To prevent claudes from stepping on each other, `cargo check` only ran at the very start and like the other runs, no `git` until the end."** Parallelism and write-collision avoidance. The phrase "task fragmentation" does not appear in the post, and no sentence gives fragmentation as the reason for crate-level grouping.

Your fragmentation thesis may still be right — see §5, where a different primary source states something very close to it — but Bun is not the citation for it. Citing Bun for "explicitly to prevent task fragmentation" is putting your inference in Jarred Sumner's mouth, and a reviewer who reads the post will catch it.

**Correction 2 — 60,624 is the Linux number, not the total.** **CONFIRMED.** The post gives per-platform counts: Linux x64 60,624 tests / 1,386,826 `expect()` calls; macOS arm64 58,850 / 1,259,953; Windows x64 57,337 / 1,007,544. Saying "60,624 tests with 0 skipped" is fine if you say "on Linux x64"; unqualified it implies a single suite total that the post does not state.

**Correction 3 — the post is dated ~2026-07-08, not May.** **LIKELY** (page-rendered date; work dates are CONFIRMED in-text). The *work* was 2026-05-03→14 and the merge was 2026-05-14. The *post* was published around 2026-07-08 and announces Bun v1.4.0, "the first version of Bun written in Rust". If you have been writing "Bun's post, May 2026", that is the work window, not the publication date. Minor, but it is the kind of thing that gets a footnote flagged.

**Correction 4 — the cost figure omits 72 billion cached input token reads.** **CONFIRMED.** The post reports 5.9B uncached input, 690M output, **and 72B cached input reads**. The cached figure is 12× the uncached volume. It does not change the ~$165,000 headline (cache reads are the cheap tier, which is the point) but it does change the *shape* of the cost story: this run was overwhelmingly cache-read-dominated, and any cost model you build from "5.9B in, 690M out" without the cache line will mis-predict badly.

### Things in the post you did not have, worth adding

- **19 known regressions** were introduced, "each of which has been fixed". **CONFIRMED.** This is the honest quality number and it is a strong one to cite — a million-line port with 19 known escapes is the real headline, better than the line count.
- Bun v1.4.0 "fixes 128 bugs that reproduce in v1.3.14". **CONFIRMED.**
- Peak throughput: "at peak Claude wrote about 1,300 lines of code per minute"; peak hour 695 commits; peak minute 58 commits. **CONFIRMED.**
- The blog names its **false starts** as "git conflicts, disk space issues, test timeouts". **CONFIRMED.** Disk space is the one nobody plans for: 4 worktrees of a Zig+Rust build tree, times parallel test runs.
- **The post contains no retrospective.** **CONFIRMED** — I looked specifically. There is no "what I would do differently" section. The closest thing is a remark that "the least risky approach to getting something shippable would be a mechanical port from Zig to Rust, with the minimal number of behavioral changes." If you have been attributing lessons-learned framing to Bun, it is not there; the post is a description, not a retrospective.
- Small internal discrepancy: the post gives 6,502 commits excluding merges and 6,778 including them; the PR page reports 6,755 commits merged. **CONFIRMED** that both figures appear; the 6,778 vs 6,755 gap is unexplained. Use 6,502 (excluding merges) and you avoid the question.
- Diff stats from the PR page: 2,188 files changed, 1,009,257 added, 4,024 removed. **LIKELY** (secondary reporting of the PR page; my fetch of the PR did not surface the file/line counts). Note 1,009,257 ≠ the blog's 1,009,272. A 15-line difference, irrelevant to any argument, but do not present the two as the same number.

---

## 2. Other practitioner reports

### 2.1 OpenAI, *Scientific computing in the age of agentic AI: an exploratory field report* — 2026-07-28

<https://openai.com/index/scientific-computing-agentic-ai/> · PDF: <https://cdn.openai.com/pdf/scientific-computing-in-the-age-of-agentic-ai-an-exploratory-field-report.pdf> — I extracted and read the PDF text directly. **CONFIRMED** throughout this section. **[DID]**

This is the most valuable document I found after Bun, and it is better evidence in one specific way: it is **eight independent case studies by different groups**, each with a written reflection, so the recurring patterns are cross-team rather than one person's method. Authors span OpenAI, UNC Chapel Hill, Allen Institute for AI, Seqera, Altos Labs, Harvard/Dana-Farber (Heng Li), and others.

**What was built and measured:**

- **rustar-aligner** — from-scratch Rust reimplementation of STAR, ">20,000 lines of accumulated C/C++". Parity measured on position, CIGAR, MAPQ, NH tag and proper-pair flag against STAR 2.7.11b, identical index and arguments, 10k yeast RNA-seq reads: **99.815% single-end, 99.883% paired-end tie-adjusted parity, 0 STAR-only and 0 rustar-only reads, 0 MAPQ inflations or deflations, 0 NH differences, and a suffix array of 10,862 entries byte-for-byte identical to STAR's. 396 passing tests.**
- **RustQC** — consolidated 15 post-alignment QC tools into one binary. On a 186-million-read paired-end human dataset on AWS: original tools **15 h 34 min** sequential (RSeQC's TIN alone ~9 h 45 min) → RustQC **14 min 54 s**, ">60× faster, with disk I/O down from 2.5 TB to 0.1 TB". Output "numerically equivalent and MultiQC-compatible", switchover "close to a one-line configuration change and is reversible". Also TrimGalore 7× faster, FastQC-Rust 3× faster.
- **HelixForge** — GPU-native redesign. Benchmark task: 100 single-base substitutions plus 5 indels in a 10 Mb region. BamSurgeon **1,610 s** average (1,557 s of it the editing step) → GPU path far faster; projected whole-genome ~2.3 h on one H200 or ~19 min across eight H200s, against ~5.8 days per genome for the BamSurgeon CPU path. A ~3,200-genome cohort at 30× would be ~six weeks on eight H200s vs **~51 years** on a single CPU worker.
- **MHCflurry** — TensorFlow/Keras → PyTorch, "a codebase-wide backend rewrite changing nearly 10,000 lines". Validated against the TensorFlow backend across **315 allele-and-peptide combinations**.
- **bayesm-rs** — "We prompted GPT-5.2 to execute a complete rewrite of bayesm into Rust with fairly minimal guidance." Reproduced the same posterior summaries with **2.71× single-threaded and 9.51× eight-threaded** speedup vs CRAN bayesm 3.1.7.
- **svb** — head-to-head vs streamvbyte64 v0.2.0 on identical inputs (AVX2, GitHub Actions CI): U32Classic decode 2.34–2.88× faster (14.1 vs 4.89 GB/s at 8192 elements); U32 encode 2.68–2.85×; fused VBZ decode 3.68 GB/s; VBZ2 2-chain 5.62 GB/s. Wire compatibility verified in CI by round-trip encode/decode against the reference.

**Mechanisms that matter to us, all quoted:**

- **Adversarial pairing, across models, adopted as a remedy for a stall — and it worked.** "Adversarial pairing beats a single agent. Letting Claude Code and Codex alternate between contributor/reviewer roles seemed to escape plateaus since the two agents caught different errors in their reviews." The human role in MHCflurry included "deciding to add a second agent (Codex) as reviewer/contributor once subtle numerical problems persisted after prolonged work with just Claude Code," and "choosing the pattern of agentic work: develop with one harness, review with the other, and switch roles when work stalled or moved unproductively." Note the hedge — "seemed to escape plateaus" — this is a reported qualitative outcome, not a measured defect-detection rate.
- **A directly stated limit of agentic review.** "Agentic code reviews are not always sufficient to reach convergence but might instead nudge the agent towards unnecessary expansion of the code surface, which in turn introduces even more subtle bugs. It took a great deal of questioning and sanity checking by a human to slowly achieve near-equivalence of predicted outputs." This is the strongest sourced caution against treating reviewer agents as a closed loop.
- **Agents cannot self-verify, and the answer they landed on is structural, not habitual.** "Agents simply cannot self-verify yet. The agent will assert that a plot looks correct, but on visual inspection it is severely flawed: overlapping labels, an incorrect axis, an element flipped or rotated. It has no perception of the small details that make a plot readable. **This failure is so common and so complete that the only reliable way to handle it is structural. Every test in the cargo suite renders a plot, over 900 so far, and each is examined visually for regressions or omissions across all plot types.**" And later: "over 900 plots being manually checked by eye before a release."
- **The false-green problem, named by a primary source.** Summarising across the three projects: agents are "weak wherever correctness is not well defined: where it is silent and only shows up downstream (rustar-aligner), **where the agent's own tests certify the wrong answer (svb)**, or where the judgement is visual (kuva)."
- **The probe itself was the defect — this is your six-defective-probes lesson, from someone else's incident.** "The validation harness itself can also be a source of errors; in HelixForge, an early false-positive strand-balance audit caused by downsampling led the agent to modify the GPU implementation even though the problem was in the auditing step. Careful human review is thus required at multiple levels of abstraction: the rewrite itself needs to be carefully checked, and the validation harness (which is often designed with the assistance of coding agents) itself also requires manual review." A broken probe did not merely fail to catch a bug — it *caused* the agent to damage correct code.
- **Independent corroboration of Bun's ban on escape hatches, with the opposite polarity.** "A further requirement was keeping the agent progressing through regressions rather than reverting prior work, which proved a persistent challenge but was essential to achieving parity. These rules were encoded directly into the agent's standing instructions." And the rustar-aligner stall: "At around 90% parity, the agents stalled. The remaining differences consisted of several layers of bugs stacked upon one another, so that any single change caused a regression in testing; **the agent would then revert its work, unable to progress.** The breakthrough came from allowing the agent to modify the STAR source alongside rustar-aligner, adding debug output to trace individual reads through both alignment pipelines... **Permitting regressions while following this path — trusting the overall process** — was essential." Two independent teams, same failure: agents use revert as an escape from a hard state. Bun banned the git commands; this team banned the behaviour in standing instructions and additionally had to *authorise temporary regressions* so the agent would stop retreating.
- **Failure of over-decomposition, at the algorithm level.** "The agent also repeatedly tripped on long-read-only `#ifdef` blocks in STAR's C++, making algorithm decisions based on code paths not active for short reads; the source-tracing method largely resolved this, since edits to dead code produced no change and forced the agent to investigate further. **More generally, agents tend to decompose a complex, tightly coupled algorithm into atomic pieces and then cannot recover the accuracy that the monolithic design achieves. Some complexity is irreducible, and the agent fights it.**"
- **Acceptance criteria as the enabling artifact.** "Define acceptance criteria. A concrete acceptance criterion gives the agentic loop a target for more autonomous iteration."
- **Iteration shape.** "The projects generally proceeded through staged, feedback-driven iterations rather than as one-shot approaches. Contributors typically decomposed broad goals into iterated changes, and established intermediate benchmark/test harnesses against which agents had to test and revise changes. Initial implementations could often be produced rapidly, but resolving edge cases, subtle numerical differences, and failures that appeared only on realistic workloads frequently required substantially more iteration. Often, contributors found that completing the 'last mile' of an implementation took the most work and effort."
- **Real data over synthetic.** "For RustQC, validation across real public sequencing data showed that many edge cases surfaced only at realistic scale; minimal datasets were not sufficient."
- **The economic reframing, stated as their conclusion.** "Agents can reduce the effort required to produce and improve implementations, [but] they do not eliminate the costs of deciding what to build or remove the need for long-term responsibility for the package. **The economic opportunity is therefore better understood as a reallocation of scarce expert effort from implementation toward specification, verification, and stewardship.**"
- **Their own stated limitation.** "This exploratory field report is retrospective: the underlying projects were not commissioned for this study or conducted under a common protocol, and the case studies were collected from contributors after the work had already been undertaken. They therefore provide a narrow, selected cross-sectional view of current practice rather than a representative sample."

**Notable absence:** the report contains **no token counts and no dollar figures** for any of the eight projects. For a document this careful about measurement, that gap is itself informative — see §8.

**Sourced negative inside this report.** One contributor's earlier attempt at the same MHCflurry migration failed: S.F. "first tried to migrate MHCflurry with aider, an early open-source CLI-centric agent harness, using Claude 3.5 Sonnet and OpenAI's o1 model. Though this effort did successfully port one component of MHCflurry over a week of work, it was then abandoned because of the perceived immaturity of the agentic coding process." His PR-closing comment: "Why 200 commits? Because I did this almost entirely with aider... I have learned a lot about how incredibly naive AI-code generation is". Same task, same person, ~18 months apart: abandoned, then succeeded. The variable that changed was model and harness capability plus a verification methodology, not enthusiasm.

Also relevant, quoted from the report's framing: "agents did not, for instance, fully complete any of FrontierSWE's five from-scratch implementation tasks."

### 2.2 rewrites.bio — Seqera, updated 2026-06-16

<https://rewrites.bio> — **CONFIRMED.** **[SAYS, but written by someone who DID]** — Philip Ewels wrote this *after* the RustQC rewrite, so it is distilled practice rather than speculation, but the principles themselves are prescriptive.

Ten principles for AI-assisted rewrites. The four that are load-bearing for us:

- **"Emulate exactly."** "The goal is a faster tool that produces the same results" — byte-for-byte for deterministic tools, declared numerical tolerance otherwise.
- **"Scope appropriately."** "A rewrite that does four things correctly is more valuable than one that claims fifteen and does twelve right."
- **"Be transparent about AI."** Document which tools were used, **how correctness was validated, and the gaps in validation coverage.** An explicit requirement to publish your validation gaps.
- **"Contribute upstream responsibly."** "Verify problems manually before filing bugs; avoid automating issue reports and **never use AI-generated test cases as evidence.**"

That last clause is the strongest published statement of our probe rule, from an unrelated domain.

### 2.3 Airbnb — Enzyme → React Testing Library, ~3.5k files

<https://medium.com/airbnb-engineering/accelerating-large-scale-test-migration-with-llms-9565c208023b> (2025-03) — **CONFIRMED.** **[DID]**

The most cleanly instrumented migration in the public record, and the structural opposite of Bun: **per-file state machine, not per-crate grouping.**

- "nearly 3.5K React component test files"; completed in **6 weeks** against an estimate of "1.5 years of engineering time to do by hand"; six engineers (**LIKELY** — engineer count is from secondary coverage, e.g. <https://www.infoq.com/news/2025/03/airbnb-llm-test-migration/>).
- **75% of target files migrated in the first four hours.** Then "we had pushed our completed files from 75% to 97% of the total files" over four days of tuned iteration. The last **3% (~100 files)** took "another week" and were finished by hand using the failing LLM refactors as starting points.
- **Pipeline: a per-file state machine.** "Moving the file to the next state only after validation on the previous state passed." States: Enzyme refactor → fix Jest → fix lint and tsc → mark complete. **The gate is the state transition.** A file cannot advance on an agent's assertion; it advances on a validator.
- **Retry as the primary tactic, with a wildly non-uniform budget.** "Retry steps multiple times until they passed or we reached a limit" — most simple-to-medium files done within 10 attempts; long-tail files were retried "anywhere between 50 to 100 times."
- **Context escalation as the fix for the long tail.** Prompts grew "to anywhere between 40,000 to 100,000 tokens", pulling in "as many as 50 related files" plus few-shot examples and "examples of existing, well-written, passing test files."
- No cost figures published. **CONFIRMED absence.**

**Why this is the useful contrast to Bun.** Airbnb's unit of work was one file, and it worked — because each file was *independent*, each had a *mechanical* validator (Jest/lint/tsc), and there was no shared build state to contend over. Bun's unit was the crate, because 16 agents editing files inside one Rust crate contend on `cargo check` and on each other's compile errors. The decomposition unit is not a matter of taste; **it is determined by where the coupling and the validator live.** Neither team says this outright, but the two designs together imply it, and it is the most defensible general rule I can extract from the record.

### 2.4 Google — LLM-assisted internal migrations

- <https://arxiv.org/abs/2501.06972> (2025-01), *How is Google using AI for internal code migrations?* — experience report on JUnit3→JUnit4, Joda-Time→java.time, and experimental flag cleanup; Google "on track to achieve its migration targets, surpassing the success metric of 50% or better acceleration." **CONFIRMED** (abstract-level).
- <https://arxiv.org/abs/2504.09691> (2025-04), *Migrating Code At Scale With LLMs At Google* — **"39 distinct migrations undertaken by three developers over twelve months"**, **"595 code changes with 93,574 edits have been submitted"**, of which **"74.45% of the code changes and 69.46% of the edits were generated by the LLM"**, with an estimated **"50% reduction on the total time spent on the migration compared to earlier manual migrations."** **CONFIRMED** (abstract-level; I did not read the full body).

**Landing model: the opposite extreme from Bun.** 595 separate changes over twelve months. Google's answer to the review problem is not adversarial agents; it is small, individually reviewable, incrementally landed changes at scale, in a monorepo with a merge-queue-equivalent. Three developers, twelve months, 595 changes ≈ one landed change every 1.8 developer-days. That is the throughput baseline any "we shipped a million lines in 11 days" claim should be set against, because Google's version is *maintained* and *reviewed by owners*.

### 2.5 Lutz Leonhardt — 44 Angular components, Reactive Forms → Signal Forms

<https://dev.to/lutz_leonhardt/i-used-ai-agents-to-migrate-44-angular-components-the-review-changed-my-mind-4pop> (2026) — **CONFIRMED.** **[DID]** Small scale, unusually honest, and it independently reproduces three of our failure modes.

- **44 components**, one orchestrator agent, **one sub-agent per component in an isolated git worktree** (`isolation: "worktree"`). 34 completed, 5 intentionally skipped (no Signal Forms equivalent for `FormArray`), 5 failed. **94 commits in the final PR.** ~2–3 hours of agent time in one evening.
- Orchestration was three markdown artifacts: `goal.md` (protocol), `SignalMigration.md` (playbook), `Plan.md` (bill of materials tracking per-component status). Note: the *plan is a file*, not a transcript — the same journalling instinct we arrived at.
- **All five failures clustered on one root cause:** "SignalFormControl does not support imperative validator or error mutation." A single API gap, discovered 34 components in. A three-component trial run would have found it — which is exactly Bun's 3-file trial.
- **The adversarial diff review caught three classes the pipeline missed:**
  1. **False green, verbatim:** "In several worktrees, `npm install` had not been executed... agents noted this as a warning — and then reported SUCCESS anyway."
  2. **Silent semantic change:** an email validator swap changed which addresses the form accepts, with "no test catching the difference."
  3. **Timing regression:** a validator side effect that "could reset a deeplink alert immediately after it was set — a regression invisible in unit tests."
- **What he would do differently** (the retrospective Bun does not have): **five-component waves** — "After each wave, the orchestrator writes a summary and **stops**. The human reviews the results, decides whether to update the playbook, and approves the next wave." Plus: preserve worktrees for inspection rather than auto-cleaning them; require unit tests to actually run before success is declarable; **"enforce test execution limits externally rather than via prompt"**; and use "a different frontier model reading the migration diff with adversarial intent."
- His summary: **"Agent transforms. Second model challenges. Human decides."** and **"automated transformation is not the same as validated correctness."**

"Enforce externally rather than via prompt" is the single most transferable sentence in this document. It is the difference between a habit and a gate.

### 2.6 Lovable — one engineer, ~$85k of tokens, six months

<https://lovable.dev/blog/85000-in-tokens-later-scaling-agentic-coding-at-lovable> (2026, mid-year) — **CONFIRMED.** **[DID]** The best single-practitioner economics report I found.

- **~$85k in tokens since January 2026**, ramping from "$600 of tokens per month" pre-Lovable to **"~$25K/mo in May"**. Split: "About 75% of my tokens are spent directly on implementation. The other 25% (and growing) is spent on all forms of automation."
- **Output:** January "a productive week meant 20–30 merged PRs"; by June "150+ merged PRs"; **"During the first week of June I merged 293 PRs."**
- **Topology:** grew from "one human and three agents" to **"one human over six to seven agents, each with its own subagents"**, including "a dedicated agent that writes tasks for the other agents, with multiple levels of implementation and review agents." Six to seven, not sixty-four — for open-ended feature work rather than a mechanical port.
- **Landing model: stacks, explicitly chosen over one big PR.** "A 10-PR stack instead of a single PR" is the unit for large work, "keeping PRs to tens to hundreds of lines of code."
- **A measured failure of AI review at size:** "The quality of AI code review drops sharply when your change gets big enough" — demonstrated when "a fellow developer got AI review approval on a big, 6K-line PR" and the same reviewer flagged real issues only after the change was split into a stack. **This is an empirical bound on adversarial-reviewer-agent effectiveness: it degrades with diff size.** Bun's reviewers saw per-file and per-crate diffs, not the million-line one — which, read against this, is load-bearing to why it worked.
- **Risk-tiered review lanes:** an AI workflow classifies PRs into "fast cheap AI, slow expensive AI, and human review lanes," with infrastructure changes mandatorily human.
- **Human review moved up an altitude:** away from line-by-line, toward "RFCs and ADRs" and "the most impactful changes."
- **Agent mortality / state:** "Large tasks exceeded 1M tokens", requiring external task tracking (Beads, "with a local setup without syncing to git"). Plus "Never reuse context for multiple tasks. `/clear` is free, reversible, and takes just a few seconds." And "zero permission requests" were a precondition for throughput, achieved by wrapping CLI tools with safety constraints rather than by approving prompts.

### 2.7 A controlled cost benchmark: 23 harness/model combinations, one feature

<https://blog.insight-services-apac.dev/2026/07/06/cost-to-a-merged-feature> (2026-07-06) — **CONFIRMED.** **[DID]** The only properly controlled cost-per-landed-change measurement I found.

- Task: RFC 8628 OAuth 2.0 device-authorization flow on a **frozen** Nuxt 4 + Drizzle repo with history stripped — "a real security boundary (one-time codes, server-side expiry, no replay, hashed secrets)."
- **23 model-and-harness combinations across 29 runs** (Claude Code, GitHub Copilot, OpenCode; Sonnet 4.6/5, Opus 4.8, Fable 5, GPT-5.5/5.4-mini/5.6, Gemini 3.5 Flash, MAI-Code-1-Flash, open-weight models).
- **"The same feature cost about $3 to about $33 to merge."** Harness choice alone moves cost by "roughly 2.5×".
- **Where the money goes shifts as models get cheaper:** "the bill shifts onto the review gate rather than disappearing." For MAI-Code-1-Flash, **"upwards of 97%"** of the $2.81–$5.73 total was the Opus 4.8 review gate, not the coding tokens.
- **The finding that matters most to us:** the constant Opus 4.8 review gate **approved implementations scoring as low as 12 out of 38 on the objective behavioural test suite.** The author's conclusion: **"merged" ≠ "correct".** An LLM review gate, run as the sole gate, passes badly broken security code.
- **And a methodological warning we should adopt:** **"Most of the striking early results didn't survive a second, careful run."** Single-run agentic measurements are noise.

### 2.8 The review bottleneck, at population scale

- **LinearB 2026 Software Engineering Benchmarks** — **8.1 million pull requests, 4,800+ organisations, 42 countries.** Agentic-AI PRs waited **5.3× longer for reviewer pickup (1,055 vs 201 minutes, 75th percentile)**; AI-assisted PRs were **2.6× larger (408 vs 157 LOC, p75)**; elite teams reached a **95% acceptance rate on manual PRs but only 71% on agentic AI PRs**, and "most teams cannot exceed 60% on AI PRs." **LIKELY** — consistent across multiple secondary reports (<https://linearb.io/dev-interrupted/podcast/linearb-2026-benchmarks-ai-pr-merge-rate>, <https://byteiota.com/ai-prs-wait-4-6x-longer-linearb-2026-benchmarks/>); I did not read the primary report, and note the two secondary sources quote 5.3× and 4.6× for pickup, so treat the exact multiple as approximate.
- **DORA 2025 State of DevOps** — AI adoption ~90%; throughput improves; **instability increases** (higher change failure rate, more rework). **LIKELY.** The specific trio circulating as "9% higher bug rate / 91% longer review time / 154% larger PRs" is **UNCERTAIN** — I could not tie those three numbers to the DORA primary and one secondary source appears to blend DORA with other studies. Do not cite that trio.
- **METR RCT** (<https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/>, arXiv 2507.09089, 2025-07-10) — 16 experienced open-source developers, 246 tasks in mature repos they averaged 5 years on: **19% slower with AI allowed**, while forecasting 24% faster beforehand and estimating 20% faster afterwards. **CONFIRMED** as the study's headline. **Two caveats you must carry with the citation:** n=16, and **METR itself now labels the result historical**, not reflective of current tools. Cite it for the perception/reality gap, not as a current effect size.

### 2.9 Cognition — the strongest architectural counter-argument

<https://cognition.com/blog/dont-build-multi-agents> (2025-06), Walden Yan — **CONFIRMED.** **[SAYS]**, importantly.

- Two principles: **"Share context, and share full agent traces, not just individual messages"** and **"Actions carry implicit decisions, and conflicting decisions carry bad results."**
- The failure mechanism: parallel subagents take actions on "conflicting assumptions not prescribed upfront," and you inherit "the undesirable task of combining these two miscommunications."
- Their prescription: "The simplest way to follow the principles is to just use a single-threaded linear agent," with a context-compression model for tasks too long to fit.
- **Honesty requirement:** the post presents this as principled analysis and industry observation; **it does not report an experiment they ran and measured.** They build Devin, so it is informed, but as evidence it is argument, not data.
- **Their position has since moved**, which matters: Yan later stated "A year ago, I'd tell people to not build multi-agents and to focus on context engineering fundamentals. Today, many sexy ideas are still impractical, but we've found some setups that actually work" (<https://x.com/walden_yan/status/2047054554433462360>, 2026) — reportedly landing on "multi-agent systems work best today when writes stay single-threaded and the additional agents contribute intelligence rather than actions." **UNCERTAIN** on the exact later wording (secondary paraphrase); **LIKELY** on the direction of the shift.

**Note how well the revised position describes Bun.** One implementer writes; two reviewers and a fixer contribute intelligence; the fixer applies. Reviewers never write. Bun is a *single-writer* topology, fanned out 64 ways across *disjoint* crates. Cognition's objection is to concurrent writers with conflicting assumptions — which is precisely the configuration Bun's crate partitioning, `cargo check`-once rule, and no-git-until-the-end rule exist to prevent. The two are not in conflict; Bun is a proof of Cognition's revised rule.

### 2.10 Anthropic — multi-agent research system

<https://www.anthropic.com/engineering/multi-agent-research-system> (2025-06) — **CONFIRMED.** **[DID]** Vendor-authored, but it reports measurements and failures, so it is usable with that label.

- **"A multi-agent system with Claude Opus 4 as the lead agent and Claude Sonnet 4 subagents outperformed single-agent Claude Opus 4 by 90.2% on our internal research eval."**
- **Cost:** "agents typically use about 4× more tokens than chat interactions, and multi-agent systems use about 15× more tokens than chats." And: "For economic viability, multi-agent systems require tasks where the value of the task is high enough to pay for the increased performance."
- **Token spend is the dominant explanatory variable:** "token usage by itself explains 80% of the variance" in BrowseComp performance, with tool calls and model choice the other two factors (95% of variance combined).
- **Fan-out sizing, stated as guidance from experience:** "Simple fact-finding requires just 1 agent with 3–10 tool calls, direct comparisons might need 2–4 subagents with 10–15 calls each, and complex research might use more than 10 subagents with clearly divided responsibilities."
- **Over-fanout as a real observed failure:** "Early agents made errors like **spawning 50 subagents for simple queries**, scouring the web endlessly for nonexistent sources, and distracting each other with excessive updates."
- **The decomposition failure mode, named:** "Without detailed task descriptions, agents duplicate work, leave gaps, or fail to find necessary information."
- **Their explicit caveat on coding:** "Most coding tasks involve fewer truly parallelizable tasks than research, and LLM agents are not yet great at coordinating and delegating to other agents in real time." Read against Bun, the operative word is *truly* — a 1,448-file mechanical port is unusually parallelizable, which is why it worked and why it generalises less than the line count suggests.
- **Agent mortality, and the mitigations:** "When errors occur, we can't just restart from the beginning: restarts are expensive and frustrating for users. Instead, we built systems that can resume from where the agent was when the errors occurred." And: "When context limits approach, agents can spawn fresh subagents with clean contexts while maintaining continuity through careful handoffs... they can **retrieve stored context like the research plan from their memory** rather than losing previous work when reaching the context limit." Plus: "Without effective mitigations, minor system failures can be catastrophic for agents."
- **Evaluation:** started from "about 20 queries representing real usage patterns"; LLM-as-judge against a rubric (factual accuracy, citation accuracy, completeness, source quality, tool efficiency); **"We found success focusing on end-state evaluation rather than turn-by-turn analysis"**; and "People testing agents find edge cases that evals miss."

### 2.11 Refute-or-Promote — measured adversarial adjudication of findings

<https://arxiv.org/abs/2604.19049> / <https://arxiv.org/html/2604.19049v1> (2026-04), Abhinav Agarwal — **CONFIRMED** for the reported figures; note this is a **single-author preprint with self-reported results, not peer-reviewed.** **[DID]**

This is the closest thing in the literature to your 75-findings adjudication problem, and it is worth reading in full.

- **Funnel:** ~171 initial candidate findings across 7 targets → **~135 (~79%) killed by adversarial review**, leaving ~36 validated. Prospective subset (lcms2, wolfSSL; n=30): **83% kill rate.**
- **Stage economics:** "Stage A eliminates ~63% of candidates; of those surviving Stage A, Stage B killed ~42%." The cheap stage kills two thirds before any expensive work happens.
- **Mandatory empirical gate:** "no candidate reaches disclosure without empirical confirmation" (Stage C). A finding is not a finding until it reproduces.
- **Outcomes and cost:** "36+ outcomes across 7 targets" — 4 CVEs, 20+ spec defects, 3 compiler bugs, 8 security fixes without CVEs — for **"~$250 out-of-pocket on a standard LLM subscription for 36+ outcomes (including 4 CVEs) — roughly $62/CVE."**
- **Unanimity is not evidence:** **"80+ agents" unanimously endorsed a false Bleichenbacher vulnerability** in an initial OpenSSL campaign. And in the other direction, a *real* bug (lcms2 `CubeSize()` integer overflow, CVE-2026-41254) **"was unanimously killed in Round 4"** and only "resurrected via a creative uplift agent tasked with a different role." Unanimous agreement and unanimous rejection both failed. The paper's framing: agreement "reflects shared training-data priors rather than convergent truth."
- **An honest complication for your batching thesis:** this paper adjudicates candidates **individually** through sequential stage gates — and it works at n≈171 for ~$250. So "one-by-one adjudication cannot scale" is not what the record says. What distinguishes it from your 268-agent/8.6M-token/zero-verdict run is *where the cost sits*: a cheap Stage A that kills 63% before any expensive agent is spawned, versus spawning a full-cost adjudicator per finding at the outset. Regrouping 75 findings into 12 themes and triaging cheaply first are two different fixes to the same problem, and this paper is evidence for the second, not the first.

Related, on consensus as a failure mode: "Auditing medical multi-agent AI reveals risks of false consensus" (<https://arxiv.org/pdf/2510.10185>) reports **57.2% of errors occurring under agent agreement and 90.6% of dangerous under-predictions occurring under agent agreement**, concluding that disagreement-based monitoring is "structurally blind to the majority of dangerous failures." **UNCERTAIN** — I have this from search summary only, and it is a medical-triage domain, not code. Directionally consistent with the above; do not lean on the exact figures.

### 2.12 Measured worktree isolation failure

<https://github.com/anthropics/claude-code/issues/55724>, filed **2026-05-03**, closed as duplicate — **CONFIRMED** as a report; single reporter, so the measurement is n=1. **[DID]**

- **13 parallel agents dispatched with `isolation: "worktree"`: 5 committed successfully, 8 failed.** Intermittent at 5 agents; "near-certain that some will fail" at 10+.
- Mechanism: worktrees share the parent repo's `.git/`; concurrent `git add`/`git commit` contend on `index.lock`, ref packing and loose refs. Error: `Unable to create '.git/index.lock': File exists`.
- **The compounding failure is the cleanup:** "Those agents exit without committing. Worktree auto-cleanup removes the worktree and its uncommitted changes. **Work is permanently lost — requires full re-dispatch.**"
- Reporter's fixes, in priority order: retry on lock contention with backoff (200/400/800 ms); **check `git status --porcelain` before cleanup and preserve worktrees with uncommitted changes**; jitter worktree creation by 100–500 ms.

This is a different mechanism from ours — we lost a deliverable to a peer's `git reset --hard` inside a *shared* worktree; this is lock contention plus auto-cleanup in *separate* worktrees. Both end the same way: finished work destroyed by infrastructure, not by a bad model output. Note that Bun's "no `git` until the end" rule incidentally prevents both, and that a 64-agent run doing per-crate commits would have hit this wall hard — which suggests Bun's git discipline was doing more work than the blog credits it for.

### 2.13 Sandbox isolation: what worktrees do not isolate

**LIKELY** — synthesised from multiple secondary practitioner posts (<https://www.gptfrontier.com/preventing-database-and-port-collisions-with-concurrent-ai-agents/>, <https://codeongrass.com/blog/parallel-coding-agents-worktree-isolation-ownership/>, <https://northflank.com/blog/how-to-sandbox-ai-agents>); no single authoritative primary. **[SAYS]** mostly.

The consistent claim: "Git worktree gives you multiple working directories but they still share the same database, same ports, same Docker daemon — it solves code isolation, not environment isolation." Named leaks: port collisions (`EADDRINUSE` on the default dev port when two agents each start a server), shared Postgres on a fixed host mapping so parallel integration tests pollute each other's tables, and a shared Docker daemon. Remedy proposed: per-agent containers with own filesystem root, PID namespace and network interface.

Bun's answer to the same class of problem was `systemd-run` cgroups with isolated PID namespaces for the *test* step specifically — narrower and cheaper than full containers, and worth noting as the minimum viable version.

### 2.14 The high-parallelism end, and why its numbers are weak

- **Steve Yegge / Gas Town** (open-sourced 2026-01) — a Go orchestrator (~189k LOC) for "colonies of 20–30 parallel AI coding agents", with a role hierarchy: Overseer (human), Mayor (dispatcher), Deacon (health daemon), Dogs (maintenance), and at rig level Crew, Polecats (ephemeral workers), **Refinery (merge queue manager)**, Witness (supervisor). **LIKELY** on the design; the 20–30 figure is Yegge's stated capacity. Claims of "50–80 peak, maybe 100" and "5 PRs in 3 hours to 36 PRs in 4 hours" are **UNCERTAIN** — unattributed in the secondary sources and I found no primary measurement. Sources: <https://reading.torqsoftware.com/notes/software/ai-ml/agentic-coding/2026-01-15-gas-town-multi-agent-orchestration-framework/>, <https://codex.danielvaughan.com/2026/04/08/gas-town-multi-agent-factory/>.
  - The structurally interesting part is that at 20–30 agents the design grew **a dedicated merge queue role and a health daemon**. Both are answers to bottlenecks Bun avoided by never merging until the end and by having a human watch.
- **Geoff Huntley / "Ralph Wiggum"** (<https://ghuntley.com/ralph/>; <https://www.theregister.com/2026/01/27/ralph_wiggum_claude_loops/>, 2026-01-27) — a single agent in a bash loop, re-fed its own errors. Reported: an MVP quoted at $50,000 delivered for **~$297 in tokens**, and a programming language built for about the same. **LIKELY** (Register-reported, practitioner self-reported). Also "about US $10 of compute per hour." Note this is the *opposite* topology — depth, not width — and the reported economics are better per outcome than any parallel run in this document. That is a real tension in the record, not a resolved question.
- **The Pragmatic Engineer**, "New trend: programming by kicking off parallel AI agents" (<https://newsletter.pragmaticengineer.com/p/new-trend-programming-by-kicking>) — **CONFIRMED, and the finding is the absence.** Sid Bidasaria (Anthropic), Simon Willison and Armin Ronacher all describe running multiple agents; **none reported a number.** The two most useful statements are both about the same ceiling: Willison — "I can only focus on reviewing and landing one significant change at a time"; Ronacher — "sometimes kick off parallel agents, but not as much as I used to do... it's only so much my mind can review." Ronacher explicitly *reduced* his parallelism.

---

## 3. Cross-cutting: what consistently works

Ordered by strength of evidence. Only items with at least two independent sources, or one primary measurement, appear here.

1. **A validator, not an agent's report, decides that work is done.** Airbnb's state machine advances a file only when the previous state's validation passes (**CONFIRMED**). Bun's ladder is compile → run → one test → 100 tests → full CI on 6 platforms (**CONFIRMED**). Refute-or-Promote requires empirical confirmation before a finding exists (**CONFIRMED**). Lutz's headline failure was agents "noted this as a warning — and then reported SUCCESS anyway" (**CONFIRMED**). Four independent sources, one rule.
2. **A trial run on 3–5 units before fanning out to the rest.** Bun: 3 files before 1,448 (**CONFIRMED**). Lutz's retrospective asks for exactly this in five-component waves after discovering an API gap 34 components in (**CONFIRMED**). Anthropic started from ~20 representative queries (**CONFIRMED**).
3. **A written plan artifact, produced before the fan-out, that every agent reads.** Bun: `PORTING.md` + `LIFETIMES.tsv`, ~3 hours (**CONFIRMED**). Lutz: `goal.md` + `SignalMigration.md` + `Plan.md` (**CONFIRMED**). Anthropic: the research plan is retrieved from memory rather than held in context (**CONFIRMED**). Lovable: task state in Beads, outside the transcript (**CONFIRMED**). Airbnb's state machine is itself durable state. **In every case the plan is a file, and in every case that is what survives an agent dying.**
4. **Adversarial review by a reviewer that does not see the implementer's reasoning.** Bun: "only the diff. told to assume the code is wrong" (**CONFIRMED**). OpenAI field report: contributor/reviewer alternation between Claude Code and Codex, "the two agents caught different errors" (**CONFIRMED**, qualitative). Lutz: "a different frontier model reading the migration diff with adversarial intent" (**CONFIRMED**). Refute-or-Promote: 79% of candidates killed by adversarial refutation (**CONFIRMED**, self-reported preprint).
5. **Cross-model adversarial review beats same-model.** OpenAI field report is the strongest evidence — Codex was *added as a remedy* when Claude Code alone plateaued on subtle numerical bugs, and it broke the plateau (**CONFIRMED**). Refute-or-Promote uses a "Cross-Model Critic for orthogonal error detection" (**CONFIRMED**). Lutz recommends it (**[SAYS]**). Mechanism: correlated priors. 80+ same-family agents unanimously endorsed a nonexistent vulnerability (**CONFIRMED**).
6. **Group work by the coupling boundary, and give each group its own validator invocation.** Bun: crate, because a crate is one `cargo check` (**CONFIRMED**). Airbnb: file, because a file is one Jest run (**CONFIRMED**). The rule that fits both: **the work unit is the smallest thing that has an independent, mechanical pass/fail signal and does not contend with its siblings.**
7. **Ban the escape hatches, and authorise temporary regressions instead.** Bun banned `git stash`/`git reset`/mid-workflow `cargo` (**CONFIRMED**). The rustar-aligner team encoded "keep progressing through regressions rather than reverting prior work" into standing instructions and found "permitting regressions while following this path... was essential" (**CONFIRMED**). Independent teams, same discovery: an agent's instinct under difficulty is to retreat, and retreat destroys the run.
8. **Small diffs for review, whatever the landing model.** Lovable measured AI review quality degrading with diff size, with a 6k-line PR passing review and then failing once split (**CONFIRMED**). LinearB: AI PRs are 2.6× larger and wait 5.3× longer (**LIKELY**). Bun's reviewers saw crate-sized diffs even though the *landing* was one PR — the two decisions are independent and should be reasoned about separately.
9. **Real data at realistic scale, not minimal fixtures.** "For RustQC, validation across real public sequencing data showed that many edge cases surfaced only at realistic scale; minimal datasets were not sufficient" (**CONFIRMED**). rewrites.bio: "Synthetic data is useful for quick iteration but insufficient for validation" (**CONFIRMED**).
10. **Declare the acceptance criterion before the loop starts.** "A concrete acceptance criterion gives the agentic loop a target for more autonomous iteration" (**CONFIRMED**). Bun's was 100% of the existing suite passing on 6 platforms; rustar-aligner's was numeric parity against STAR; Airbnb's was four validators in sequence.

---

## 4. Cross-cutting: what consistently fails

1. **Agents assert success they have not earned.** The strongest sourced statement: "Agents simply cannot self-verify yet" (**CONFIRMED**, OpenAI field report). The strongest incident: `npm install` never ran, agents logged a warning and reported SUCCESS (**CONFIRMED**, Lutz). The strongest naming of the mechanism: "the agent's own tests certify the wrong answer" (**CONFIRMED**, OpenAI field report on svb). And the strongest measurement: an Opus 4.8 review gate approved a security implementation scoring **12/38** on objective tests (**CONFIRMED**, Insight benchmark).
2. **The probe is a defect surface, and a broken probe actively causes damage.** HelixForge: a false-positive strand-balance audit caused by downsampling "led the agent to modify the GPU implementation even though the problem was in the auditing step" (**CONFIRMED**). Not merely a missed bug — a working implementation was edited to satisfy a lying probe. This is the case for our rule, and it is now sourced.
3. **Unanimous agreement among agents is not evidence.** 80+ agents unanimously endorsed a false Bleichenbacher vulnerability; a real CVE was unanimously killed and only recovered by re-roling an agent (**CONFIRMED**). Both directions fail. Consensus measures shared priors.
4. **Infrastructure destroys finished work.** 13 agents → 8 lost to `index.lock` contention plus auto-cleanup (**CONFIRMED**, n=1 report). Bun's agents used `git stash`/`git reset` on each other (**CONFIRMED**). A 429 kills a session with no recovery and it "stays dead until a human intervenes" (**LIKELY**, <https://github.com/paperclipai/paperclip/issues/1861>). None of these are model failures.
5. **Review capacity is the binding constraint, and it is measured.** Agentic PRs wait 5.3× longer for pickup; acceptance drops from 95% to 71% even on elite teams (**LIKELY**). Willison: "I can only focus on reviewing and landing one significant change at a time" (**CONFIRMED**). Ronacher reduced his parallelism for this reason (**CONFIRMED**). DORA 2025: throughput up, instability up (**LIKELY**).
6. **Reviewer agents can make things worse by expanding scope.** "Agentic code reviews are not always sufficient to reach convergence but might instead nudge the agent towards unnecessary expansion of the code surface, which in turn introduces even more subtle bugs" (**CONFIRMED**). This is the most important caveat on mechanism #4 above and it comes from the same document that endorses adversarial pairing.
7. **Over-fanout is a real, observed failure, not a theoretical one.** "Spawning 50 subagents for simple queries... distracting each other with excessive updates" (**CONFIRMED**). And: "Without detailed task descriptions, agents duplicate work, leave gaps" (**CONFIRMED**).
8. **Agents decompose coupled things into atomic pieces and then cannot reassemble the accuracy.** "Agents tend to decompose a complex, tightly coupled algorithm into atomic pieces and then cannot recover the accuracy that the monolithic design achieves. Some complexity is irreducible, and the agent fights it" (**CONFIRMED**). This is the closest sourced statement to your fragmentation thesis. Note carefully: it is about decomposing *an algorithm*, not *a task list*. It is adjacent evidence, strongly suggestive, not the same claim.
9. **The last mile dominates.** Airbnb: 75% in four hours, 97% in four days, the final 3% took another week and ~100 files went to humans (**CONFIRMED**). rustar-aligner: agents stalled at ~90% parity and needed a human to invent a tracing method (**CONFIRMED**). OpenAI field report generally: "completing the 'last mile' of an implementation took the most work and effort" (**CONFIRMED**). Any schedule extrapolated from the first-pass rate will be wrong by a large factor.
10. **Single measurements do not replicate.** "Most of the striking early results didn't survive a second, careful run" (**CONFIRMED**). This applies to everything in this document, including Bun.

---

## 5. Mechanisms we do not have

Each is a concrete practice, with the source that reports it and an honest note on whether it was measured.

1. **Externally enforced test execution, not prompt-requested.** Lutz, verbatim: "enforce test execution limits externally rather than via prompt" and "require unit tests to run before declaring success." Combined with Airbnb's state machine, the shape is: **the harness runs the validator and records the result; the agent never reports its own pass.** This is the direct structural answer to "is the test actually running" that Bun answered with a human habit. **[DID]** — Lutz's failure was caused by the absence of it; the fix itself is his recommendation. Sources: <https://dev.to/lutz_leonhardt/i-used-ai-agents-to-migrate-44-angular-components-the-review-changed-my-mind-4pop>, <https://medium.com/airbnb-engineering/accelerating-large-scale-test-migration-with-llms-9565c208023b>.
2. **A cheap Stage A that kills most candidates before any expensive agent is spawned.** Refute-or-Promote: Stage A eliminates ~63% of candidates; Stage B kills ~42% of survivors; only then does the expensive empirical gate run. Total: ~$250 for 36+ outcomes from ~171 candidates. **[DID]**, self-reported preprint. This is the mechanism that would have saved your 268-agent run — not batching, triage ordering. <https://arxiv.org/abs/2604.19049>
3. **A mandatory empirical-confirmation gate: a finding does not exist until it reproduces.** Refute-or-Promote Stage C: "no candidate reaches disclosure without empirical confirmation." **[DID]** Note the symmetry with our rule: they require the *finding* to be demonstrated; we require the *probe* to be demonstrated. Both are "no claim without a demonstration."
4. **"Never use AI-generated test cases as evidence."** rewrites.bio principle 10, verbatim. **[SAYS]**, but written by someone who completed a 60× rewrite. <https://rewrites.bio>
5. **Publish your validation gaps as a deliverable.** rewrites.bio principle 3 requires documenting "how correctness was validated" *and* "gaps in validation coverage." **[SAYS]** An artifact that names what you did *not* prove is a cheap and unusually honest gate.
6. **Structural visual/manual review where the model provably cannot judge.** "Every test in the cargo suite renders a plot, over 900 so far, and each is examined visually." **[DID]** The transferable idea is not the plots; it is the practice of **identifying the specific class where the model cannot self-verify and building a structural human checkpoint for exactly that class**, rather than reviewing everything or trusting everything.
7. **Mutation testing as the gate that proves a probe can fail.** This is the mechanism you asked for and it exists off the shelf: mutate the code, and a test suite that stays green has proven it cannot detect that bug. Reported measurement: a vanilla LLM prompt scored 53% mutation score on HumanEval-Java, unchanged after four iterations without mutation feedback, versus 89.5% with mutation feedback (**UNCERTAIN** — MutGen, via secondary summary only; I did not reach the primary). The concept is well established (<https://www.thoughtworks.com/radar/techniques/mutation-testing>). **A surviving mutant is a probe with no demonstrated failure mode, mechanised.**
8. **Preserve the workspace on failure; never auto-clean a dirty worktree.** "Check `git status --porcelain` before cleanup and preserve worktrees with uncommitted changes"; plus Lutz's "preserve worktrees for inspection." **[DID]** — both written by people who lost work to auto-cleanup. <https://github.com/anthropics/claude-code/issues/55724>
9. **Retry with backoff on git lock contention.** 200/400/800 ms, "the lock is transient and retries almost always succeed"; plus 100–500 ms jitter on worktree creation. **[DID]** Cheap, and it addresses a measured 8-of-13 loss rate.
10. **Risk-tiered review lanes.** Lovable: classify PRs into "fast cheap AI, slow expensive AI, and human review lanes," infrastructure mandatorily human. **[DID]** This is how you stop paying Opus-review prices on every diff — recall that the review gate was up to 97% of total cost in the Insight benchmark.
11. **Bounded waves with a mandatory stop.** Lutz: five components, then "the orchestrator writes a summary and **stops**", human approves the next wave, playbook updated between waves. **[DID]** — proposed in direct response to discovering a blocking API gap 34 components in. The stop is what converts a fan-out into a learning loop.
12. **A stack of small PRs as the unit for large work.** Lovable: "a 10-PR stack instead of a single PR", each "tens to hundreds of lines." **[DID]**, with a measured justification (AI review quality degrades with size). Contrast Bun's single PR — defensible for a mechanical port validated by an existing suite, much harder to defend for feature work.
13. **A merge-queue role and a health daemon once you exceed ~20 agents.** Gas Town grew both (Refinery, Deacon). **UNCERTAIN** on measured benefit; noted because two roles appearing at that scale is a signal about where the bottleneck moves.
14. **End-state evaluation over turn-by-turn.** "Instead of judging whether the agent followed a specific process, evaluate whether it achieved the correct final state." **[DID]** <https://www.anthropic.com/engineering/multi-agent-research-system>
15. **Context handoff via retrieved artifact, not continuation.** "Agents can spawn fresh subagents with clean contexts while maintaining continuity through careful handoffs... retrieve stored context like the research plan from their memory." Plus Lovable: "Never reuse context for multiple tasks. `/clear` is free." **[DID]** Our journalling habit is the right instinct; making it the *only* channel between agent generations is the mechanism.
16. **A `git status`/`porcelain` precondition and no VCS operations until the end.** Bun's "no `git` until the end" combined with "`cargo check` only ran at the very start" is a single idea worth naming: **shared mutable state is touched exactly once, at a boundary, by one actor.** **[DID]**

---

## 6. Sourced negatives and critiques

1. **The Bun rewrite's post-landing state, measured by a third party.** Tom Lockwood, <https://lockwood.dev/ai/2026/07/27/how-is-the-bun-rewrite-in-rust-going.html>, 2026-07-27 — **CONFIRMED** as his own GitHub-derived data; his cost extrapolation is explicitly his estimate.
   - "It's now been 11 weeks since the last Bun release tag: 2026-05-12 15:12:49 -0700 (tag: bun-v1.3.14)" — i.e. as of 2026-07-27 he found no release tag after the merge.
   - **Open `robobun` (Claude-generated) PRs: 1,277 on July 9 → 2,475 by July 27.** He calculates the backlog would need ~86 days of continuous CI to process.
   - He disputes the $165,000 figure, arguing it excludes ongoing CI/CD and Anthropic staff time, and estimates "approaching $800k" if daily costs continued at $10,000/day. **This is his extrapolation, not a measurement — label it as such if you cite it.**
   - His conclusion: "we can't take it at face value that the rewrite is 'done'."
   - **Note a tension I could not resolve:** the Bun post (published ~2026-07-08) announces v1.4.0, while Lockwood on 2026-07-27 reports no release tag after 2026-05-12. **UNCERTAIN.** Both are cited above with their dates; I did not establish which framing is right. If you cite "Bun v1.4.0 shipped", verify the tag first.
2. **Hacker News, on the merge** — <https://news.ycombinator.com/item?id=48132488>, ~2026-05-14. **CONFIRMED** that the comments exist; the counts within them are commenters' own and unverified.
   - `embedding-shape` counted **~10,428 `unsafe` blocks across 736 files** in a ~929k-line codebase, arguing "no human has read any appreciable fraction" of it. **UNCERTAIN** as a number. But the argument is the serious one: a memory-safety-motivated port that lands 10k `unsafe` blocks has an unquantified fraction of its stated benefit.
   - `bmitc`: no analysis was published showing *which* Zig memory bugs were actually fixed; and flagged a commit with "a one second sleep put in place" in tests. A sleep in a test is a false-green mechanism, in the diff.
   - `sesm`: a 622-line Zig→Rust idioms file and pre-existing smart-pointer types suggest longer preparation than the 11-day framing implies. Partially corrected by `Aurornis` (the `bun_collections` crate was part of the PR, not pre-existing). Worth knowing that the "~3 hours of planning" figure was contested on exactly this basis.
   - `tasuki`: nine days before the merge, the author had said "there's a very high chance all this code gets thrown out completely."
   - `drzaiusx11` raised **load-bearing bugs** — behaviour that downstream code implicitly depends on — which no maintainer answered in the thread. For a "0 tests skipped, 100% suite green" verification story, this is the sharpest unanswered question: the suite encodes the old behaviour that was *tested*, not the old behaviour that was *relied upon*.
   - `fragmede`: "Everyone's had a problem they were working on, and the solution doesn't come sitting at the desk staring at the code, but three days later." Compression of calendar time removes the incubation that finds design errors.
3. **METR's RCT** — 19% slower, against a 20% *perceived* speedup, with a 39-point perception gap. **CONFIRMED** as headline; n=16; **METR labels it historical.** <https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/>
4. **Cognition's architectural objection.** **CONFIRMED** as argument, **[SAYS]** as evidence. Its author has since partially retracted the strong form. Do not cite it as a measured negative — cite it as the reason single-writer topology is the safe default.
5. **A practitioner who reduced parallelism deliberately.** Armin Ronacher: "sometimes kick off parallel agents, but not as much as I used to do... it's only so much my mind can review." **CONFIRMED.** Small, but it is a named, experienced engineer moving in the opposite direction, and it is the honest counterweight to Gas Town.
6. **An abandoned agentic migration, by someone who later succeeded at the same task.** MHCflurry with aider + Claude 3.5 Sonnet + o1: one component ported over a week, then abandoned "because of the perceived immaturity of the agentic coding process", 200 commits, closing note about "how incredibly naive AI-code generation is." **CONFIRMED** (OpenAI field report, appendix).
7. **Population-scale negatives:** DORA 2025's throughput-up/stability-down split (**LIKELY**); LinearB's 95%→71% acceptance-rate drop and 5.3× pickup delay (**LIKELY**).
8. **Cost blowouts:** Uber reportedly exhausted its annual AI coding tools budget in four months, with individual engineers at $500–$2,000/month before per-engineer caps. **UNCERTAIN** — secondary only (<https://blog.exceeds.ai/ai-coding-token-costs-2026/>); I could not reach a primary. Do not cite without verification.

---

## 7. Economics, assembled

The public record on cost is thin and inconsistent in units. Everything below is per its own definition; they are not comparable without care.

| Source | Unit | Figure | Confidence |
|---|---|---|---|
| Bun (<https://bun.com/blog/bun-in-rust>) | whole 1M-line port, API pricing | **~$165,000**; 5.9B uncached in, 690M out, 72B cached reads | CONFIRMED |
| Lockwood (2026-07-27) | same port, incl. CI + staff time | "approaching $800k" — **his extrapolation** | his estimate |
| Insight benchmark (2026-07-06) | one merged OAuth feature, 23 harness/model combos | **$3–$33**; harness choice alone moves it ~2.5×; review gate up to 97% of total | CONFIRMED |
| Lovable (2026) | one engineer, 6 months | **~$85,000**, peaking ~$25k/month; 293 PRs in one week at peak | CONFIRMED |
| Refute-or-Promote (2026-04) | validated security findings | **~$250 for 36+ outcomes ≈ $62/CVE** | CONFIRMED (self-reported preprint) |
| Huntley / Ralph | one MVP quoted at $50k | **~$297 in tokens**; ~$10/hour of compute | LIKELY |
| Anthropic (2025-06) | multiplier | multi-agent ≈ **15× a chat**; agents ≈ 4× | CONFIRMED |
| OpenAI field report (2026-07-28) | eight rewrites | **no cost figures published** | CONFIRMED absence |

**The two findings worth carrying:**

- **Cost per landed change is dominated by the review gate, not the coding tokens, once models get cheap.** Up to 97% in a measured comparison. Any budget model that counts implementation tokens and treats review as free is wrong by an order of magnitude — and this is exactly the failure mode of a workflow that spawns a full-cost adjudicator per finding.
- **Cost per agent-hour is a useless metric and cost per *landed, verified* change is the only one anyone measured usefully.** The Insight benchmark is the only source that fixed the deliverable and varied the machinery. Its most important result is not the $3–$33 range; it is that the gate approved a 12/38 implementation — meaning **cost per merged change and cost per correct change are different numbers, and only one team measured the gap.**

---

## 8. Open questions where the public record is thin

Stated plainly, because these are findings too.

1. **N=1 on large single-diff agentic ports.** Bun is the only published example of a million-line, one-PR, high-parallelism agentic rewrite with token and cost accounting. Every other large migration in the record — Google, Airbnb, all eight OpenAI cases — landed incrementally. **There is no second data point for the one-PR model, and therefore no basis for claiming it generalises.**
2. **Nobody has published a measured optimum for concurrent agent count.** The record has 3–7 (Lovable), 4×16=64 (Bun), >10 subagents (Anthropic research), 13 (the worktree failure report), 20–30 (Gas Town), 44 sub-agents (Lutz). **No source varied the agent count and measured the result.** Bun's "4 worktrees" is stated without a reason; whether it was disk, CPU, human attention or arbitrary is **UNKNOWN**.
3. **No published heuristic for task granularity exists.** I looked hard. The closest things are Anthropic's fan-out sizing by task type (experience-based guidance, not a measurement), the observation that agents over-decompose *algorithms* and lose accuracy, and secondary "100–500 tokens of output per subtask" advice with no measurement behind it (**UNCERTAIN**, apxml). **Your 268-agents-zero-verdicts → 49-agents result is, as far as I can tell, a data point that does not exist in the public record.** It is publishable.
4. **Adversarial diff-only review is almost entirely unmeasured.** Bun asserts it. The OpenAI field report says it "seemed to escape plateaus." Refute-or-Promote measures a 79% kill rate but on security findings, not code diffs, and self-reported in a single-author preprint. **Nobody has published a controlled comparison of diff-only versus full-context review on the same diffs.** What defect classes it catches is currently anecdote: Lutz's three (unrun tests, silent semantic change, timing regression) are the most specific list available.
5. **No structural false-green gate is reported in a large agentic run.** Bun's answer was a human checking. Lutz's answer is a recommendation. Airbnb's state machine is the closest thing to a real mechanism and it predates the current concern. **Mutation testing is the obvious available answer and I found nobody reporting it in a large parallel agentic run.** That gap is an opportunity, not just an absence.
6. **Agent mortality is discussed everywhere and measured almost nowhere.** The 13-agents/8-lost report is n=1. Everything else is guidance. **No source reports what fraction of long agentic runs die, or what fraction of their work survives.** Our own two-workflow quota loss is, again, more data than most published accounts contain.
7. **Cost per *correct* landed change has been measured exactly once** (Insight, and only as a byproduct — the 12/38 result). No team has published cost per defect escaped, or cost per regression, which is the number an engineering leader would actually want.
8. **The maintenance tail is unmeasured everywhere.** Lockwood's 2,475-open-PR observation is the only public look at what happens to a large agentic rewrite after the merge, and it is one outsider counting PRs. The OpenAI field report is unusual in naming stewardship as a first-class unsolved problem — "it often remains unclear whether that implementation should replace the original tool, be incorporated into it, or remain an experimental fork" — and warns that "the proliferation of inexpensive rewrites may simply divide users between superficially similar tools." **Nobody has published a 12-month follow-up on an agentic rewrite.**
9. **The depth-versus-width question is wide open.** Huntley's single-agent loop reports the best cost-per-outcome in this document. Bun's 64-way fan-out reports the largest outcome. **No source compares them on the same task.**

---

## Source list

Primary sources read directly:

- <https://bun.com/blog/bun-in-rust> (pub. ~2026-07-08; work 2026-05-03→14)
- <https://github.com/oven-sh/bun/pull/30412> (merged 2026-05-14)
- <https://news.ycombinator.com/item?id=48132488> (~2026-05-14)
- <https://lockwood.dev/ai/2026/07/27/how-is-the-bun-rewrite-in-rust-going.html> (2026-07-27)
- <https://cdn.openai.com/pdf/scientific-computing-in-the-age-of-agentic-ai-an-exploratory-field-report.pdf> + <https://openai.com/index/scientific-computing-agentic-ai/> (2026-07-28)
- <https://rewrites.bio> (updated 2026-06-16)
- <https://medium.com/airbnb-engineering/accelerating-large-scale-test-migration-with-llms-9565c208023b> (2025-03)
- <https://arxiv.org/abs/2504.09691> and <https://arxiv.org/abs/2501.06972> (2025)
- <https://cognition.com/blog/dont-build-multi-agents> (2025-06)
- <https://dev.to/lutz_leonhardt/i-used-ai-agents-to-migrate-44-angular-components-the-review-changed-my-mind-4pop> (2026)
- <https://www.anthropic.com/engineering/multi-agent-research-system> (2025-06)
- <https://arxiv.org/html/2604.19049v1> (2026-04)
- <https://github.com/anthropics/claude-code/issues/55724> (filed 2026-05-03)
- <https://lovable.dev/blog/85000-in-tokens-later-scaling-agentic-coding-at-lovable> (2026)
- <https://blog.insight-services-apac.dev/2026/07/06/cost-to-a-merged-feature> (2026-07-06)
- <https://newsletter.pragmaticengineer.com/p/new-trend-programming-by-kicking> (2026)

Secondary / unverified-primary, cited as LIKELY or UNCERTAIN above:

- <https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/> · <https://arxiv.org/abs/2507.09089>
- <https://linearb.io/dev-interrupted/podcast/linearb-2026-benchmarks-ai-pr-merge-rate> · <https://byteiota.com/ai-prs-wait-4-6x-longer-linearb-2026-benchmarks/>
- <https://redmonk.com/rstephens/2025/12/18/dora2025/> · <https://www.faros.ai/blog/key-takeaways-from-the-dora-report-2025>
- <https://ghuntley.com/ralph/> · <https://www.theregister.com/2026/01/27/ralph_wiggum_claude_loops/>
- <https://reading.torqsoftware.com/notes/software/ai-ml/agentic-coding/2026-01-15-gas-town-multi-agent-orchestration-framework/> · <https://codex.danielvaughan.com/2026/04/08/gas-town-multi-agent-factory/>
- <https://www.thoughtworks.com/radar/techniques/mutation-testing> · <https://www.augmentcode.com/guides/mutation-testing-ai-generated-code>
- <https://arxiv.org/pdf/2510.10185>
- <https://github.com/paperclipai/paperclip/issues/1861>
- <https://www.gptfrontier.com/preventing-database-and-port-collisions-with-concurrent-ai-agents/> · <https://northflank.com/blog/how-to-sandbox-ai-agents>
- <https://blog.exceeds.ai/ai-coding-token-costs-2026/>
