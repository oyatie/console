# Account14 / App8 CI enrollment map

Read-only map by /root/inventory_repair at immutable
59c38d607745e00c8e8e89d9d6e15ed163688aa3. Root remains sole CI/generated/authority
writer and runtime operator. No shared changes, gate executions, Cargo or DB calls.
This map is not lane admission. Lenses: Essentialism, Systems Thinking, Red Team,
Operability, Blast radius, Zero trust. Source bindings and exact current names are
attached. Finish and freeze the ordinary Operation test before freezing CI selectors.

## Exact seven-file candidate scope

1. `.github/workflows/ci.yml:849`: retain the existing13 supervisor steps; append
   four new App7 cases, then the exact approved Operation case for App8. Each case
   needs its own step and fresh supervisor, in the same roster order. Reuse:
   `CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py
   "$GITHUB_WORKSPACE" -- cargo test --locked --manifest-path backend/Cargo.toml
   -p console-app --lib --features test-recovery durability_composition_tests::<name>
   -- --exact --test-threads=1 --nocapture`.
   Preserve the exact existing !cancelled + run_live_postgres + checkout/toolchain/
   image outcome condition and no continue-on-error. Preserve domain-b job metadata,
   the ordinary target command, setup actions, and both required aggregators.
2. `scripts/lib/recovery-test-invocations.mjs:19`: append the same exact names to
   APP_CASES. It constructs the ordered9 owner +1 observer +N App union at26 and
   validates the whole actual source and workflow at92–119. One stale roster rejects
   the entire recovery union. Preserve top-level SQLx/cfg/ignore/module checks,
   exact Cargo argv/fault-mode/step identity/order, and all failure propagation.
3. `scripts/lib/recovery-test-invocations.test.mjs:13`: update the explicit union
   count13 to17 for App7 or18 for App8, and its descriptive title. Existing per-case
   deletion and5 altered-command mutations derive from APP_CASES automatically.
   Do not replace the literal count with a tautology or remove source checks.
4. `tools/buck/test_preparation_wiring.py:35`: update its separate APP_CASES list
   in exactly the same order. SUPERVISED_CASES40, fixture workflow127, source oracle
   141 and execution comparison393 derive the rest. Mutation loops also grow from
   the roster; no new general probe machinery is needed.
5. `scripts/check-ci-preflight.mjs:43`: append exact commands to recoveryCommands;
   add corresponding named proofRun entries after1191 using indexes13…16 for
   App7 and17 for the final Operation case. The allowed-run checks at2633 derive
   from this list. Keep recoveryRunCondition41 unchanged. Adding steps alone does
   not change requiredJobMetadataSha2561306 (steps excluded), global envelope
   digest1325, existing setup-action indexes, or the unchanged preparation-step
   raw digest1035. Recompute a digest only if its actual bound text/envelope changes.
6. `scripts/check-ci-preflight.test.mjs:952,1012`: exact counts change together:
   App7 => domain-b21, total155, bypass465; App8 =>22,156,468. This is +one run
   step and its three independent bypass mutations per new supervised case.
   Retain all existing required job counts and mutation classes.
7. `docs/program/executed-tests-baseline.json:27,184,414`: pin Account source6→14;
   App aggregate source192→196 for four additions, then197 for the fifth. The
   ontology/rest aggregate baseline19 also becomes21 for the already-added
   diagnostic-prefix negative and typed-display positive controls; otherwise the
   full baseline gate remains red after wiring. Update the three-case explanation
   to the final count. Preserve the named dark set, dark_baseline1, deferred sets,
   test-recovery feature pin, and every unrelated count. These numbers are static
   attribute accounting; root should confirm the actual final resolver output.

## Existing enrollment that requires no new target

Account target already exists at `tools/ci/postgres-cargo-map.json:2187`,
`backend/crates/identity/adapter-postgres/BUCK:89`, and
`scripts/check-ci-preflight.mjs:774`. Its complete, unfiltered
`cargo test --locked --manifest-path backend/Cargo.toml
-p console-identity-adapter-postgres --test deactivate_revokes_credentials
-- --test-threads=1` naturally discovers all14. Preserve the current map entry,
workflow flag and wrapper. Weighted package placement comes from
`tools/ci/postgres-partition.mjs`, not a new hardcoded identity shard; existing
measured_seconds3.7 is historical evidence, not an estimate to rewrite from count.

Both retained lifecycle/freshness targets also already have workflow map entries
(2259 and5603); fixture setup amendments do not change their three-case rosters.
App recovery remains outside ordinary Buck/test-postgres targets and the ordinary
Postgres map. Keep Cargo feature closure and the exact module cfg unchanged.
No generated BUCK/map/lockfile changes are implied by adding cases to existing files.

## Checker coupling and named existing probe

`node --test scripts/lib/recovery-test-invocations.test.mjs` is the narrow existing
probe recommended for the next CI lane. At this source snapshot the live App file
has7 cases but APP_CASES has3, so the existing first case rejects the source union;
this is a static prediction, not a freshly observed RED. Root must run it on the
clean final test-roster head and bind the actual exit/output before admission.
Do not claim lane RED from this map or an unexecuted name.

The exact existing preflight preparation command includes:
`PYTHONDONTWRITEBYTECODE=1 python3 tools/buck/test_preparation_wiring.py -v`,
`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/lanes/recovery
-p 'test_*.py' -v`, and the Node probe above (`ci.yml:204`). The Python source
case also predicts RED for the current7-versus3 mismatch.

`npm run check:executed-tests` runs regression tests and then the resolver. At
`scripts/check-executed-tests.mjs:283` the complete supervised union must validate;
its failure becomes UNRESOLVED and aborts at605 before the baseline check. Updating
baseline counts alone cannot fix that. Its fixture tests import APP_CASES and
use1+APP_CASES.length (`scripts/check-executed-tests.test.mjs:215,264`), so no direct
edit is currently indicated there. After topology is valid, the static ratchet at
646 rejects both losses and unpinned gains (`scripts/lib/executed-tests-baseline.mjs`).

Required follow-through after same probe GREEN: full preparation wiring/regressions,
`node --test scripts/check-ci-preflight.test.mjs`, normal preflight contract entry,
and `npm run check:executed-tests`. Record actual discovered/executed counts, even
where loops generate tests. No runtime count is supplied by this read-only map.
The existing recovery fixture supervisor/test implementation needs no change solely
for enrolling another exact App case.

Pre-mortem: one stale roster rejects all supervised reachability; a grouped command
can hide later cases; only updating baseline conceals missing selectors. Blast
radius: seven CI/accounting files owned serially by root. Detection: same existing
probe, all mutation suites and exact baseline comparison. Rollback: reject the
private map or inverse only an approved future CI patch. Stop if final test name
is not frozen, source bindings drift, a probe is not actual RED, or scope expands.
HOLD: final Operation source/approval, root observed admission RED, independent CI
review, same probe GREEN, exact-head CI/runtime closure and production authority.
