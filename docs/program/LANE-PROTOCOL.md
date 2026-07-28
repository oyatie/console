# LANE-PROTOCOL.md — parallel execution discipline

> Adapted from Bun's Zig→Rust rewrite (1,448 files, 535k lines, 11 days, 64 agents across **4**
> worktrees, ~1,300 lines/min at peak, 6,502 commits). What follows copies their *mechanisms* and
> deliberately diverges where our work differs from theirs.
>
> Status: **prep artifact, not yet exercised.** Fan-out is not authorized until §4 passes.

## 1. The lesson that is usually taken wrongly

The naive reading is "run many agents." That is not why it worked. Bun's 64 agents were safe because
every one of the 1,448 files had three things:

1. a **known-correct reference** — the original `.zig` file
2. an **immutable target** — a language-independent TypeScript suite, never rewritten
3. a **mechanical rule set** — `PORTING.md`

Agents could not design the wrong thing because they were not designing. Sumner's instruction was
literally *"do the rewrite that looks like we transpiled our Zig code to Rust."*

**We have none of the three yet.** Pointing this machinery at design work produces N designs and a
merge conflict. Evidence from this repo, 2026-07-28: a documentation sweep run without a fixed
reference failed **9 of 11 areas with 38 fabricated claims**, including deleting requirement text while
reporting it as struck through; an infrastructure classification pass produced **21 cases** of agents
labelling live production resources as safe to delete.

## 2. Design → Expansion

| Phase | Character | Model |
|---|---|---|
| **Design** — substrate understanding + **OrgUnit end to end** | serial, small, high-judgment | **not** Bun. One implementer, deep review |
| **Expansion** — Position, Person, Employment, PayRun | mechanical, repetitive | full Bun machinery below |

The design phase's actual job is to **manufacture the three preconditions**:

| Bun had | We build |
|---|---|
| the `.zig` reference | **OrgUnit**, built by hand, exceptionally well |
| the immutable TS suite | **`company-conformance`** — black-box scenario suite at the application use-case boundary, owned outside the lanes |
| `PORTING.md` | **`docs/program/CATALOG.md`** |

**Deliberate divergence:** Bun rejected incremental because shim code hurts, and could afford big-bang
*because the work was mechanical and verifiable*. Ours is neither yet, so we invert their order —
incremental through design, big-bang through expansion.

## 3. The worktree pool

**Fixed pool of 5 lanes.** Not per-task worktrees.

```
~/Developer/console-lanes/lane-{1..5}
```

Rationale, from measurement: this repo reached **653 worktrees / 890 directories under `/private/tmp`**.
That is not untidiness — it caused a real incident this session. The `maintenance`→`console` rename
orphaned every one of them (their `.git` files pointed at the dead path), silently killing background
agents. This repo has also lost dev Postgres to 707 orphaned Docker volumes. Bun ran the entire
operation on four.

Rules:
- Lanes are **reused**, never created ad hoc. If all 5 are busy, work waits.
- Lanes live **outside `/private/tmp`** — it is OS-ephemeral and was the source of the mass orphaning.
- A lane is returned clean: no uncommitted changes, branch merged or abandoned deliberately.
- `git worktree prune` after every teardown.

## 4. Ownership — the gate on fan-out

**Disjoint roots. No two lanes touch the same file.** This is not a guideline; it was violated twice
this session and both times produced an edit war that cost more than the parallelism gained.

Reserved before any fan-out:

| Shared file | Discipline |
|---|---|
| `backend/Cargo.toml` `members` | pre-reserve entries; **never create a crate dir without a valid `Cargo.toml` in the same change** — an unmatched glob fails the build for every lane |
| `third-party/rust/BUCK` | reindeer-generated; **serialize dependency additions**, one owner |
| `backend/openapi/openapi.yaml` | reserved tag + contiguous path block per lane, merge-queued. **Do not split it** — `include_str!` at `app/src/lib.rs:187` and `app/tests/openapi_drift.rs:6` |
| migrations (`platform/db/migrations`) | single global sequence (highest `0168`); **pre-reserved block per lane**. Numbers have collided before |
| type-registry seed | one owner; lanes emit type definitions as data |
| the conformance suite | owned **outside** the lanes; a lane wanting to change it must escalate |

**Acceptance test for fan-out:** two lanes land a type + endpoint + migration with **zero overlapping
file edits**. Until demonstrated, fan-out is not authorized.

### Rehearsal results (2026-07-28) — measured, not assumed

Three lanes built real crates concurrently. **Isolation holds; landing does not.**

| Lane | Crate | Build | Lock contention | Leaked into main |
|---|---|---|---|---|
| 1 | `leave/domain` | ✓ 4.4s | none | no |
| 2 | `ontology/domain` | ✓ 7.3s | none | no |
| 3 | `platform/authz` | ✓ 18.2s | none | no |

Separate worktrees give separate `target/` directories, so cargo never serialises on the build lock.

**BLOCKER — `main` is not protected.** `gh api repos/.../branches/main/protection` returns
**404 "Branch not protected"**: no required status checks, no "require branches up to date", no merge
queue. Consequences measured per collision class:

| Collision | Caught pre-merge? | Mechanism |
|---|---|---|
| duplicate migration number | **NO** | `console-gate-migration-safety` emits `DuplicateMigrationVersion`, but only when both files are in one tree. PR CI runs against the merge ref, so lane B *would* fail — except nothing forces a re-run when the base advances. Lane B's stale-green stays mergeable; **main turns red post-merge**. Deploy is contained (`image-release` needs a green exact-SHA main run) |
| `Cargo.toml` `members` | **YES** | preflight's `cargo metadata --manifest-path backend/Cargo.toml --locked` resolves the globs and exits 101; all 8 downstream jobs `needs: preflight`, so it fail-fasts in the first minute |
| duplicate `openapi.yaml` keys | **NO** | `openapi_drift.rs` is **never invoked by CI**; `check:openapi-app` only regexes `/api/platform/` paths and then byte-compares the served bytes to the file — tautological, since the handler returns the `include_str!` const verbatim. Nothing in the repo parses the YAML |

**Two in-repo documents are wrong about this.** `docs/program/wave4-migration-slot-ledger.md` §1 says
duplicate migrations are *"not detected by cargo, sqlx, or any CI gate in this repo"*, and
`docs/specs/master-parallel-build-plan.md:220` calls it *"silent DDL loss"*. The gate predates both
(it landed in `845af868`). Work has been planned around a hazard that was already mitigated.

**Silent divergence risk:** `tools/buck/gen_first_party.py` discovers members by walking the
filesystem and never reads the `members` list, so the Buck2 and cargo graphs can diverge indefinitely
with no gate comparing them.

### Landing model: batch, do not fan out into the train

The authority train (§7 of the console governance) requires a two-commit `C → T` branch with the
registers binding C's exact SHA — **390 references**. Every merge moves `main`, so every other lane
must rebase, which changes its C, which invalidates all 390 bindings. That is O(N²) in lanes and
serialises landing regardless of how parallel the work is; the ledger names it *"the authority-train
global lock"* and attributes four consecutive hand-rebuilt releases to it.

**Therefore: parallelise the work, serialise the landing.** Lanes collect onto one integration branch,
which becomes a **single C with one T** — one trip through the train per batch, not per lane. This
costs nothing in expressiveness because the train already collapses history to C+T; batching only
stops paying the rebind N times to reach the same end state.

`scripts/console/rebind-authority-train.mjs` automates the rebind (measured: 390 references in
seconds, with shape verified afterwards) so the hand-rebuild step that lost work across four releases
is no longer manual.

### Pre-fan-out checklist

1. **Branch protection or a merge queue on `main`** — without it, two lanes can both go green and land
   conflicting migrations. This is the one true blocker.
2. **Wire sccache** — measured on this repo: `Cache hits: 0%`, 4,084 commands, **0 cached**. Every lane
   currently recompiles the entire dependency graph from scratch.
3. **Batched landing** per the model above.

## 5. The work queue

Bun grouped ~16,000 compiler errors **by crate, not by file** — explicitly to prevent task
fragmentation — and activated the next crate only when the current one was clean.

1. **Q1** — `cargo check -p <crate>`, grouped by crate. One crate active per lane.
2. **Q2** — the conformance suite, by scenario step.
3. **Q3** — `cargo nextest`, sharded by crate.

A lane is done when **its slice of the conformance suite is green**. Nothing else counts as done — not
a self-report, not a summary.

## 6. Review

**1 implementer + 2 adversarial reviewers + 1 fixer.** Reviewers receive the **diff only**, are told to
assume it is wrong, and have no access to the implementer's reasoning. Bun's reviewers also rejected
solutions that needed paragraph-long comments to justify themselves — a workaround that must be
explained is a defect.

This pattern is the one thing that demonstrably worked here: it caught 38 fabrications and 21
live-infrastructure misclassifications that the producing agents reported as clean.

## 7. Git guardrails

Adopted verbatim from Bun, both learned the hard way:

- **`git stash` and `git reset` are banned.** Commit or abandon. Bun had to edit the workflow to
  forbid them after agents used them to escape trouble.
- **Atomic per-file commits.**
- **"Edit the process, not the outputs."** When a lane produces bad work, fix the prompt and rerun.
  Applied this session: a docs sweep failing 9/11 was reverted and re-run against a hardened prompt
  rather than hand-patched.
- **Lanes write findings to disk as they go.** Three agents went idle without ever returning a report
  this session; every one had to be verified by hand. Do not rely on a final message.

## 8. Consolidation

| Gate | Condition |
|---|---|
| **C1 Contract** | every lane's endpoints + migrations landed; `openapi_drift` green |
| **C2 Build** | merged tree compiles; conformance suite green |
| **C3 Exposure** | per capability, evidence reviewed before any exposure claim moves off `HOLD` |

## 9. Local build

buck2 is retained and is the parallel-build path (`.buckconfig`, root `BUCK`, 168 per-crate `BUCK`
files, reindeer vendoring, driven by `scripts/dev-up.mjs`). Cargo remains the **source of truth** —
root `BUCK` says so outright: *"Rust targets are generated from the current Cargo workspace."*

Complementary, still unconfigured and worth fixing: this repo has **no `.cargo/config.toml` and no
`[profile]` section**, so the inner loop has never been tuned. `sccache`, `debug = "line-tables-only"`,
and a faster linker are free wins that buck2 does not cover.
