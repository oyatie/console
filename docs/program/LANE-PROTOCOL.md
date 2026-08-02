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

Rationale, from measurement: this repo reached **653 worktrees / 890 directories under `/private/tmp`**
(since consolidated to **6**: main + five lanes).
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

Three mechanisms, in order of preference. **Prefer the earlier one — later ones rely on discipline,
and discipline is what fails.**

1. **Not shared at all** — the lane's work touches only files it owns. Best; nothing to enforce.
2. **Pre-reserved** — every lane's slot is created in ONE commit *before* fan-out, so a lane edits
   only its own. Structural, not procedural.
3. **Serialised** — one owner, others request. A real bottleneck; use only where 1 and 2 fail.

### Measured collision surface for adding a catalog type (verified 2026-07-28)

| Shared file | Class | Mechanism |
|---|---|---|
| `backend/openapi/openapi.yaml` | **① NOT SHARED** | verified: 8 generic ontology paths already cover `/instances`, `/actions/{action_key}/execute`, `/object-types/{key}`. An `Instance`-backed type adds **zero** routes. A lane proposing a bespoke route must escalate — that is `CATALOG.md`'s named anti-pattern |
| `backend/Cargo.toml` `members` | **① NOT SHARED** | verified: **39 glob members** already cover crates under existing groups. Still true: **never create a crate dir without a valid `Cargo.toml` in the same change** — an unmatched glob breaks the build for every lane |
| `docs/specs/cedar-pbac-coexistence-map.json` | **① NOT SHARED** | verified: keyed by **domain**, not object type; no entry is per-type. Per-type policy is authored at runtime from the registry (`ontology/rest/src/lib.rs:426-437`). An earlier revision of this table wrongly serialised it |
| per-crate `BUCK` files | **① NOT SHARED** | **generated** by `tools/buck/gen_first_party.py`. Never hand-edit — the drift gate rejects it. Change the generator; regenerate |
| conformance `fixtures/<type>.rs` | **② PRE-RESERVED** ✅ | all five `pub mod` lines and files already exist. A lane edits only its own |
| migrations (`platform/db/migrations`) | **② PRE-RESERVED** | single global sequence, highest **`0204`**. Blocks assigned per lane in the Phase-0 commit; take the number immediately before push |
| `seed.rs` `BUILTIN_CATALOG_VERSION` (`:68`) | **③ SERIALISED — the real lock** | ONE constant + one 27-draft manifest. Two lanes adding a type both change it. **Fix is per-lane catalog versions**: migration `0204` made installs additive and version-keyed, and the allowlist PK is `catalog_version`, so lanes can ship disjoint versions. Until that lands, this is the one true bottleneck |
| `.github/workflows/ci.yml`, `tools/buck/BUCK` | **③ SERIALISED** | hand-maintained; rarely touched by a type lane. One owner |
| `third-party/rust/BUCK` | **③ SERIALISED** | reindeer-generated; serialise dependency additions |
| the three authority documents | **③ SERIALISED by design** | the C→T train. **Batch the landing** — see §4's landing model |
| the conformance suite (drivers + assertions) | outside the lanes | a lane wanting to change it must escalate |

**Acceptance test for fan-out:** two lanes land a type slice with **zero overlapping file edits**,
demonstrated on a real branch. `tools/lanes/fanout.py run` measures this automatically and fails on
any out-of-slice write.

**Status: isolation itself is PROVEN** (2026-07-28) — 4 concurrent lanes, 37s, 0 of 4 touched
anything outside their slice, 4/4 outputs correct. What remains is the `BUILTIN_CATALOG_VERSION`
lock above.

### Build in a lane, never the main checkout

Measured: an agent that ran `cargo check` in the **main checkout** while another built there took
**47 minutes**, the log reading `Blocking waiting for file lock on build directory`. CI for the same
change is **20 minutes**. Lane worktrees have their own `backend/target` and do not contend.

Corollary, and the reason this matters: **implementation must never wait on CI.** CI is asynchronous
verification — 20 minutes of wall clock nobody should be blocking on. What actually serialises
parallel work is `strict: true` (every merge forces every other open PR to rebase → new C → full CI
re-run). The rebase used to cost a register rebind on top of that; since the candidate SHA left the
registers it costs nothing in them. So: **parallelise the work in lanes; the landing is serialised
by CI, not by a shared file.**

### One writer per lane — the rule tonight cost us (2026-07-29)

A worktree isolates **files**, not **runtime**. That distinction is well documented elsewhere; the
failure modes usually named are shared ports, shared databases, shared caches and shared build
artifacts. Tested against this harness rather than adopted on faith — two lanes running different
suites **concurrently**, measured:

| resource | isolated here? | mechanism |
|---|---|---|
| ports | yes — 32823 vs 32824 | `-p 127.0.0.1::5432`, dynamic |
| database | yes | per-run container **and** per-run database name |
| credentials | yes | per-run generated passwords, mode-0600 env file |
| build artifacts | yes | one `backend/target` per worktree |
| container/volume | yes | `--rm` + `docker rm -fv` + per-run leak assertion |

Result: **both lanes green in 9 s, zero leaks.** So the generic warning does not apply to this
setup, and no runtime-isolation layer needs adopting. Do not take that on the strength of the table
— re-run the experiment if you change the harness.

What DID fail was not a runtime gap at all. **Two agents wrote in one worktree.** A lane was assigned
to a peer and then built in by the assigner:

* a verification run made while the other agent was building in the same tree reported a test binary
  **0 passed / 3 failed**; the identical command on an uncontended tree reported **3 passed / 0
  failed** minutes later. A contended run is not evidence, in either direction — and this one nearly
  caused a correct result to be rejected;
* a `git reset --hard` (lane setup) and the authority rebind's own reset **destroyed a peer's
  finished, passing deliverable** that its brief had told it to leave uncommitted.

Three mechanical rules, each earned:

1. **A lane has exactly ONE writer.** Never build, test, or mutate git in a lane you do not own. To
   verify someone else's work, mirror their branch into a lane you *do* own (`git fetch <their
   worktree> <branch>`), never run in theirs.
2. **Commit as you go; stage by path.** Never `git add -A` or `git commit -a` — a blanket add is how
   one agent's work lands inside another's commit. "Leave it uncommitted, the caller lands it" is
   withdrawn: it contradicted the `stash`/`reset` ban and cost a deliverable.
3. **Hygiene is measured on your OWN run.** A global `docker volume ls | wc -l` before/after is
   confounded the moment a peer runs a container — observed reporting `34 → 35` when the new volume
   was someone else's. `tools/lanes/pgtest.sh` now asserts only on its own container name.

### Rehearsal results (2026-07-28) — measured, not assumed

Three lanes built real crates concurrently. **Isolation holds; landing does not.**

| Lane | Crate | Build | Lock contention | Leaked into main |
|---|---|---|---|---|
| 1 | `leave/domain` | ✓ 4.4s | none | no |
| 2 | `ontology/domain` | ✓ 7.3s | none | no |
| 3 | `platform/authz` | ✓ 18.2s | none | no |

Separate worktrees give separate `target/` directories, so cargo never serialises on the build lock.

**RESOLVED 2026-07-28 — `main` IS protected.** Verified live: **12 required contexts**,
`strict: true`, `enforce_admins: true`, force-push and deletion blocked. The collision table below
is kept because the *mechanisms* are still accurate; only the "caught pre-merge?" column changed,
since `strict: true` now forces a rebase before merge and re-runs CI on the moved base.

Note the cost this introduced: `strict: true` is now what serialises parallel PRs — each merge
forces every other open PR to rebase, producing a new C. That rebase used to also invalidate every
stored register binding; since the candidate SHA left the registers it invalidates nothing in them.
The remaining cost is a CI re-run, which is an argument for batching the landing, not for reverting
protection.

Consequences as originally measured, pre-protection:

| Collision | Caught pre-merge? | Mechanism |
|---|---|---|
| duplicate migration number | **NO** | `console-gate-migration-safety` emits `DuplicateMigrationVersion`, but only when both files are in one tree. PR CI runs against the merge ref, so lane B *would* fail — except nothing forces a re-run when the base advances. Lane B's stale-green stays mergeable; **main turns red post-merge**. Deploy is contained (`image-release` needs a green exact-SHA main run) |
| `Cargo.toml` `members` | **YES** | preflight's `cargo metadata --manifest-path backend/Cargo.toml --locked` resolves the globs and exits 101; all 8 downstream jobs `needs: preflight`, so it fail-fasts in the first minute |
| duplicate `openapi.yaml` keys | **NOW YES** | *stale as written* — `openapi_drift` is wired at `ci.yml:597` (`//backend/app:console-app-itest-openapi_drift`) since #506. Originally: `check:openapi-app` only regexes `/api/platform/` paths and then byte-compares the served bytes to the file — tautological, since the handler returns the `include_str!` const verbatim. Nothing in the repo parses the YAML |

**Two in-repo documents are wrong about this.** `docs/program/wave4-migration-slot-ledger.md` §1 says
duplicate migrations are *"not detected by cargo, sqlx, or any CI gate in this repo"*, and
`docs/specs/master-parallel-build-plan.md:220` calls it *"silent DDL loss"*. The gate predates both
(it landed in `845af868`). Work has been planned around a hazard that was already mitigated.

**Silent divergence risk:** `tools/buck/gen_first_party.py` discovers members by walking the
filesystem and never reads the `members` list, so the Buck2 and cargo graphs can diverge indefinitely
with no gate comparing them.

### Landing model: the train no longer holds a global lock

The authority train (§7 of the console governance) still requires a two-commit `C → T` branch, and T
may still modify nothing outside the authority allow-list. What it no longer requires is a **rebind**:
the registers used to store C's exact SHA in every candidate-evidence leaf, so every merge moved
`main`, every other lane had to rebase, its C changed, and all of those bindings went stale at once.
That was O(N²) in lanes and serialised landing regardless of how parallel the work was; the ledger
names it *"the authority-train global lock"* and attributes four consecutive hand-rebuilt releases
to it.

The candidate SHA now reaches the validator from CI's own derivation (`ci.yml` reads the C/T/M train
off Git parentage) instead of from a copy stored in the file being validated. Rebasing a lane changes
C and changes nothing in either register, so `rebind-candidate.mjs` and `rebind-authority-train.mjs`
are deleted rather than automated.

T must modify **at least one** authority document, not all three, and it may **add** a lowercase
`.md` file directly under `docs/program/ledger/` — a flat directory, so no subdirectory and no other
extension. For most lanes that is one new ledger entry file of its own, so two lanes writing the
ledger no longer collide on the same bytes.

Batching lanes onto one integration branch is still reasonable when the work is genuinely
interdependent, but it is now a choice, not a cost-avoidance measure.

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

buck2 is retained and is the parallel-build path (`.buckconfig`, root `BUCK`, 169 per-crate `BUCK`
files, reindeer vendoring, driven by `scripts/dev-up.mjs`). Cargo remains the **source of truth** —
root `BUCK` says so outright: *"Rust targets are generated from the current Cargo workspace."*

Complementary, still unconfigured and worth fixing: this repo has **no `.cargo/config.toml` and no
`[profile]` section**, so the inner loop has never been tuned. `sccache`, `debug = "line-tables-only"`,
and a faster linker are free wins that buck2 does not cover.
