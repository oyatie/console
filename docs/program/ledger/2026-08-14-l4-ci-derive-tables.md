# Authority tip — L4-CI-DERIVE-TABLES: census scope derived from the topology script

**Date:** 2026-08-14
**Kind:** authority tip ledger bound on candidate C; T adds this ledger entry only
**Base:** `dea1f91bf6c336319fc718f9d9f3eb2c2047f63c` (origin/main)
**Candidate C and Tip T:** both signed by the pinned authority (principal `jason19931225@gmail.com`, ED25519 `SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8`). Commit/tree SHAs are recorded in the post-merge readback ledger update (T cannot name its own SHA).
**Scope:** `backend/ci/gates/writer-ownership/tests/census_executes_against_postgres.rs` only. No production code, no migration, no CI wiring change.
**Not product authority.** Clears no HOLD and authorizes no production, credential, compliance, payment, erase, OCI, frontend, projection, or Oyatie action.

## Summary

- The census scope is now DERIVED at runtime from the `required_tables CONSTANT TEXT[] := ARRAY[...]` statement in `ops/postgres-reconcile-topology.sh`, scoped to that canonical SQL block via its BEGIN/END markers instead of a hand-maintained fixed-arity `const REQUIRED_TABLES: [&str; 20]`.
- The 20-name list survives as `EXPECTED_REQUIRED_TABLES` — a verbatim pin inside the census test, not a second production source. The new no-Docker unit test `derived_required_tables_match_the_verbatim_roster` fails the run whenever the shell array and the pin diverge, so a lane landing a new canonical table edits two places (shell first, then the pin) and the census can no longer silently shrink its scope.
- RED baseline recorded: compile error `required_tables_from_topology not found in this scope` (E0425) before the function existed.

## Verification

Every Cargo command below is runnable from the repository root as `cd backend && …`. Environment: `CARGO_TARGET_DIR=target` `SQLX_OFFLINE=true`.

- `cd backend && CARGO_TARGET_DIR=target SQLX_OFFLINE=true cargo fmt -p console-gate-writer-ownership -- --check`: clean.
- `cd backend && CARGO_TARGET_DIR=target SQLX_OFFLINE=true cargo clippy -p console-gate-writer-ownership --all-targets -- -D warnings`: clean (collapsible-if findings fixed with let-chains).
- `cd backend && CARGO_TARGET_DIR=target SQLX_OFFLINE=true cargo test -p console-gate-writer-ownership --test census_executes_against_postgres derived_required_tables_match_the_verbatim_roster`: 1 passed / 0 failed (4 filtered out; Docker-required census tests run in CI).
- `node scripts/console/generate-documentation-manifest.mjs --check`: OK (441 markdown files).

## Operational receipt (lane-specific)

- **Lane:** lane-1qw5-derive-tables · **Worktree:** `.worktrees/lane-1qw5` · **Owner:** Jason Lee.
- **Pre-mortem:** the change re-sources a gate scope from a shell script; modeled failure = a decoy marker or `ARRAY[` declaration earlier in the script hijacking the parse, or the census scope silently shrinking.
- **Detection:** the decoy unit test (decoy `ARRAY[` before the `DO $canonical$` block returns only the real block's entry), `derived_required_tables_match_the_verbatim_roster`, and the executed-tests baseline ratchet.
- **Rollback:** revert the squash on main; the parser is test-local with no production runtime surface.
- **Stop conditions:** any required check red on the merge ref; unresolved review threads; loss of the pinned signing authority.
- **Review identities:** 7 `chatgpt-codex-connector` review threads resolved 2026-08-14 (severity mixed P1/P2; every finding fixed or evidenced); owner merge sign-off.
- **Head SHA at freeze:** recorded in the post-merge readback update (self-reference).

## Remaining HOLDs / follow-ups

- All current PRODUCT/ROADMAP HOLDs remain unchanged.
- `console-1qw.4` (L4-EMPL-RETARGET) remains open; the P4 epic closes after both 1qw.4 and this lane merge.
- Docker census execution on this workstation is unavailable; CI is the executing evidence.

<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373",
  "task_class": "implementation",
  "risk_class": "high",
  "risk_domains": [
    "release"
  ],
  "selected_lenses": [
    "Cartesian doubt",
    "Essentialism / YAGNI",
    "Chesterton's Fence",
    "Red Team",
    "Systems Thinking",
    "Operability / Day-2",
    "Blast-radius / cell-based",
    "Zero-trust / defense-in-depth"
  ],
  "task_fit": {
    "Cartesian doubt": "Separated what the census scope WAS (two hand-maintained copies of one set) from what it must be (one production source plus one verbatim ratchet), and re-checked the derivation against the actual script bytes.",
    "Essentialism / YAGNI": "Runtime parsing of the existing marker block with std-only string handling — no build.rs, no generated file, no new dependency, no lockfile churn.",
    "Chesterton's Fence": "Kept the verbatim-pin idiom this repository already uses (e.g. thirteen_dispatch_targets_verbatim): the pin is the behavioral regression lock, the shell array is the single production source.",
    "Red Team": "Modeled the failure that motivated the change — a rename silently shrinking the examined scope while a count assertion stayed green — and added the marker-missing and empty-extraction failure modes so the parser fails loudly rather than examining nothing.",
    "Systems Thinking": "Traced the collision the bead found: four port lanes hand-appending the same set to two shared files; the derivation removes the census half of that collision.",
    "Operability / Day-2": "The derivation runs in the same test binary that already locates the topology script; no new runtime surface, and the failure messages name the script path and the missing marker.",
    "Blast-radius / cell-based": "One test file changed; no production or CI surface touched; the Docker-required census behavior is unchanged and still fails closed without Docker.",
    "Zero-trust / defense-in-depth": "The derived scope is compared against the verbatim pin on every run — neither the script nor the pin is trusted alone; divergence fails the census instead of defaulting open."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "The census scope and the topology script's required_tables array were two hand-maintained copies of one set, and a fixed-arity array whose arity had to be bumped on every canonical-table migration.",
    "A rename once shrank the examined scope from eight tables to seven while the run still reported success — the count assertion could not see the shrink."
  ],
  "decisions_changed_or_rejected": [
    "Rejected a build.rs-generated const file: it would add a generated-file serialization point and build-time machinery for a set the test can read directly from its already-located script.",
    "Rejected removing the verbatim pin: the pin is the ratchet that refuses drift, consistent with the repository's existing verbatim idioms."
  ],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
