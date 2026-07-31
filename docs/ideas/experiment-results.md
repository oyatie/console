# Experiment results — X8, X9 executed

> `Status: EVIDENCE — executed 2026-07-29, read-only`
>
> Results for two of the experiments designed in `docs/ideas/ecosystem-plan-DRAFT.md` §8 Phase 6. Written
> as a separate artifact rather than edited into the plan because the plan was under Architect/Critic
> review at the time and a moving target confuses a reviewer.
>
> **X8's result refutes a premise that was propagated into three places, including this session's own
> reporting.** That correction is the most important line in this document.

## X8 — How do the CI buck2 jobs currently pass? **ANSWERED**

**The premise was wrong.** The claim — repeated in a docs inventory, in two of my own status reports, and
in the plan's own X8 row — was: *"`prelude/` is missing, so the buck2 graph is already broken."*

`prelude/` is indeed absent from the repo root. That is **expected and correct**, not broken:

```
# .buckconfig
[cells]
  prelude = prelude
...
[external_cells]
  prelude = bundled
```

`[external_cells] prelude = bundled` is buck2's mechanism for supplying the prelude **from inside the
buck2 binary itself**. The cell is declared; its contents are bundled. A vendored `prelude/` directory is
what that setting exists to make unnecessary.

**The full chain, each link verified:**

| # | Link | Evidence |
|---|---|---|
| 1 | CI installs the pinned DotSlash runtime | `.github/workflows/ci.yml:176` → `tools/buck/install_dotslash.sh` |
| 2 | `tools/buck2` is a **DotSlash launcher**, not a binary — `#!/usr/bin/env dotslash`, per-platform blake3-pinned digests | `tools/buck2:1-10` |
| 3 | buck2 resolves the prelude from its own bundle | `.buckconfig` `[external_cells] prelude = bundled` |
| 4 | The required job runs a **real** buck2 test invocation | `ci.yml:192` → `tools/buck2 test //backend/crates/support/domain:console-support-domain-unit` |

So buck2 is **fully functional in CI**, hash-pinned and reproducible. The required job
*"Support domain — Buck2 unit reachability"* passes because it genuinely builds and runs a buck2 test.

### Three consequences, and two of them reduce the blocker list

1. **The `hold_rule` deadlock is much weaker than reported.**
   `docs/program/console-capability-registry.json`'s `hold_rule` fails closed on "empty Buck2 target
   sets". That clause was described — by me — as keyed to a dead build system, making HOLD permanent.
   It is keyed to a **working** system. Buck2 target sets are producible today, so the clause is a
   requirement to satisfy, not a deadlock to escape. The governance question that remains is narrower:
   whether `PIVOT-2026-07-28.md` §6's *"cargo, not buck2"* should be executed at all, given buck2 works.
2. **CI-wiring prepwork simplifies.** Phase 7's instruction to target "the CI that exists (buck2 live)"
   is correct and is now positively grounded rather than being a bet against a broken toolchain.
3. **The build-system governance question stands unchanged**, and the plan's framing of it survives:
   `PIVOT-2026-07-28.md` is not in `docs/decisions/`, so under `docs/decisions/README.md` rules 1-2 it
   binds nothing. Neither "cargo" nor "buck2" is an accepted decision. What changed is only that the
   status quo is healthy, so there is no forced migration.

### Method note, stated so this result is not over-trusted

X8 is an **investigation, not a probe**, so the "proven RED on a known-bad input" discipline does not
apply to it in the same form — there is no GREEN to distrust. What makes it trustworthy is that its
answer is a configuration line that can be quoted and a launcher whose digests are pinned in the tree.
The known-bad control listed for X8 in the plan (*"wiring a new test and assuming it runs"*) is really
**X9's** control, and it is used there.

## X9 — Trace one test end to end **ANSWERED, with a worked example**

Subject: `object_policy_attach_as_runtime_role`, added by PR #525. All four links present and nameable:

| # | Link | Evidence |
|---|---|---|
| 1 | Test file | `backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs` |
| 2 | `rust_test` target | `backend/crates/ontology/rest/BUCK:172` — `console-ontology-rest-itest-object_policy_attach_as_runtime_role`, `crate_root` at `:178` |
| 3 | `sh_test` Postgres wrapper | `tools/buck/BUCK:244` — `ontology-object-policy-attach-postgres`, wrapping the rust_test via `args`/`deps` at `:246-247` |
| 4 | Workflow step | `.github/workflows/ci.yml:239` — `//tools/buck:ontology-object-policy-attach-postgres` |

**Use this as the template rather than describing the pattern abstractly.** Phase 3's per-crate CI wiring
step should point at these four citations.

### The fragility this exposes, which is why the trace is mandatory per test

`ontology/rest/BUCK:173` hand-lists **ten** files in `mapped_srcs` — every fixture and harness file the
test crate reads, individually. **Buck2 does not glob.** So the second link is not "add a target", it is
"add a target that enumerates every file the test touches", and a file added later to a shared harness is
invisible until someone edits that list.

That is the mechanical reason five correct-looking tests executed nowhere this week: link 2 or link 3 can
be absent or incomplete while the test passes locally, and nothing fails. The plan's per-crate wiring step
must therefore name all four links **per test**, not per crate.

## Not yet run, and why

| # | Experiment | Blocker |
|---|---|---|
| X1, X2 | edges from an authored type; a published type listing rows | need a database and a build; would contend with lane-1's in-flight 0206 work — a contended run is not evidence in either direction (`LANE-PROTOCOL.md:137-140`) |
| X3, X4, X5 | definer under attack; no second RLS dimension; Cedar deciding alone | need `effective_grants_for` to exist — these are slice-0 experiments, not prepwork |
| X6 | fold cost per request | needs realistic grant counts, so it follows X4 |
| X7 | draft-PR CI coverage | requires pushing a branch — outward-facing, needs explicit authorization |

X4 remains the one to run first once a lane is free: it tests the plan's own central claim (§4.2), and if
it fails the 141-table cost returns and the entity model changes shape.
