# Startup credential transport prerequisite repair

Status: proposed bounded harness design and exact test candidate; no implementation, role grant, migration, owner-test change or runtime acceptance. Root is sole repository writer. Tests and design require independent review before admission.

## Observed failure and actual owner

The retained Company entry test `account_browser::deployment_operator_designation::company_setup::designated_account_discovers_mounted_company_setup_without_business_identity` fails first in `backend/app/tests/auth_rest/deployment_operator_designation.rs:66`: `CONSOLE_STARTUP_AUTH_DATABASE_URL` is absent. The parent observed runtime failure; this packet makes no new runtime execution claim. Both supported disposable database harnesses produce auth/runtime/command credentials but omit startup transport. This is a prerequisite defect, not the intended Company owner RED.

The existing owner is `ops/postgres-reconcile-topology.sh`. Lines54–87 distinguish omission from explicit empty startup password and enforce pairwise distinctness against all supplied credentials. Lines369–393 create and validate `console_auth_startup` LOGIN with NOSUPERUSER/NOBYPASSRLS/NOINHERIT/NOCREATEDB/NOCREATEROLE/NOREPLICATION and no inbound/outbound role membership, then set only its supplied password. Line1269 verifies the login. No harness SQL or extra role grant is permitted. Existing `ops/postgres-auth-topology.integration.test.py` already contains true TCP startup provisioning/omission/collision/privilege/membership/session-drain/rollback tests; reuse valid exact evidence, rerun only if these sources or unresolved conditions change.

The existing designation fixture parses the transported URL, requires `console_auth_startup`, a nonempty password, no query/fragment and the same host/port, rewrites only to its SQLx-owned `_sqlx_test_` database, opens a real TCP connection, and checks actual session_user/current_user and restricted flags. `active_account_designation_replays_without_identity_or_company_effects` executes the actual designation owner and verifies durable receipts, exact replay and no Company/identity effects. This is the downstream owner proof, unchanged.

## Exact scope

Implementation may touch only:

- `tools/ci/cargo_needs_postgres.sh`
- `tools/buck/test_needs_postgres.sh`
- `tools/buck/run_test_with_postgres_env.sh`

Exact reviewed tests touch only the corresponding existing three files listed in `test-manifest.json`; `tests.patch` is their full mechanical patch. Source preimages and observed base are manifest-bound. Any concurrent root change requires explicit rebase/readback; this packet is not permission to overwrite it.

## Mechanical implementation guide

1. Each Cargo and Buck disposable cluster generates its own additional startup password using the existing `secret` function. Do not derive it from, reuse, or accept it from any serving/admin/command credential. Add it to Buck's existing pairwise guard. Cargo retains the real topology's existing pairwise guard; do not duplicate the credential owner or invent a second SQL path.
2. Add exactly `CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD` to each existing mode0600 container/topology env file. Preserve file creation, umask, cleanup, readiness, image pin and topology invocation. Docker argv carries paths only.
3. Add exactly `CONSOLE_STARTUP_AUTH_DATABASE_URL=postgres://console_auth_startup:<generated password>@127.0.0.1:<mapped port>/<same disposable database>` to each existing mode0600 test env file. No query, fragment, extra grant, `SET ROLE`, elevated username or source password appears in argv/logging. Cargo's existing private loader exports the value; Buck passes only the env-file path to its executor. A inherited parent startup URL is overwritten by the disposable harness's own value.
4. Extend the Buck strict data parser by one optional key and its duplicate counter. Preserve value regex, unknown-key refusal, malformed/duplicate refusal, mode0600 check and exact test forwarding. The existing mandatory set remains DATABASE_URL plus three APALIS URLs. Four existing AUTH/LEAVE/ONTOLOGY/FORCE keys remain optional; startup is a fifth optional key. Historical four-key and eight-key files remain valid. Never source/eval the test env file. Omission behavior for legacy files remains unchanged.
5. Preserve recursive Cargo child isolation: each child's own generation/file export wins over inherited parent values; no sharing startup credentials across clusters. Existing current isolation selection, timeouts and terminal role-mutating binary rules remain unchanged.

No changes to topology source, authority, migrations, generated SQL, custody profiles, runtime Account/Company code, Rust test bodies, lockfiles or toolchain. `tools/lanes/pgtest.sh` is a separate historical general Cargo harness that does not currently provide even native auth transport; it is not an approved native Account/Company invocation and is outside this fix. Native probes use the two supported entrypoints. Its existence must not be confused with proof that arbitrary alternate harnesses supply native prerequisites.

## Exact tests and oracles

The existing Cargo transport test still exercises the actual harness and runner for cargo and nextest, faking only Docker/build boundaries. It now requires eight nonempty pairwise-distinct topology values and exact console_auth_startup URL with matching host/port/database, no query/fragment, same password hash across private files, and no credential in recorded argv. An intentionally wrong inherited startup URL proves overwrite. Private file modes and deletion, cleanup calls and build/test phases remain asserted. This is a transport test, not SQL authorization proof.

The existing Buck transport fake validates mode0600 topology file and startup distinctness, then saves only a digest. Its real executor handoff must contain the exact startup role URL and matching digest on the same loopback database. Assertions explicitly terminate fake processes on failure; no preceding successful fake command can mask absence. The fake selects the test handoff by the actual `test` subcommand and explicitly refuses a missing env file; a missing request cannot skip the oracle. The fake build phase has no executor file and does not impersonate a test handoff. Existing failure/cleanup/secret-leak/exact-selector tests remain unchanged. The expected sorted topology-key census gains only the startup key.

The strict loader test retains all prior cases and adds: historical eight-key success; nine-key startup success overriding an incorrect inherited URL; each of the four required keys removed separately is refused as incomplete; duplicate startup refusal; empty/substitution/unknown startup-looking key refusals with exact diagnostic class, a child-execution marker proving no downstream side effect, and no command substitution or credential diagnostics. No raw startup URL is printed as a test artifact.

After review and root test-only commit, named baseline probes:

```
node --test --test-name-pattern='Cargo PostgreSQL harness transports distinct restricted credentials privately' tools/ci/cargo-needs-postgres-args.test.mjs
bash tools/buck/test_needs_postgres.test.sh
bash tools/buck/run_test_with_postgres_env.test.sh
```

Expected defect: first two fail because the real harness omits the startup topology password; strict-loader case reaches the new key and fails unexpected-key. A syntax/missing fixture/tool failure is not eligible RED. These are bounded real harness transport boundaries, even though Docker/compilers are faked.

After admitted implementation, same probes must pass, then existing Cargo argument/isolation suite and relevant credential/preflight checks run once. Preserve complete discovered/executed counts and exact candidate.

On root's dedicated release VM, run the unchanged exact designation proof via the Buck harness, then unchanged Company entry:

```
CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=account_browser::deployment_operator_designation::active_account_designation_replays_without_identity_or_company_effects tools/buck/test_needs_postgres.sh //tools/buck:app-auth-rest-pg
CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=account_browser::deployment_operator_designation::company_setup::designated_account_discovers_mounted_company_setup_without_business_identity tools/buck/test_needs_postgres.sh //tools/buck:app-auth-rest-pg
```

Compiler discovery must confirm these exact identifiers first; never claim a zero-test pass. Designation proof must be green. Company entry is expected still RED at `NATIVE_COMPANY_ENTRY` until its own implementation is admitted: repairing prerequisites does not accept Company UI or enrollment. If the next failure is another prerequisite, repair it under its owner rather than changing tests or falsely admitting Company.

## Risk, detection, rollback and stop conditions

Pre-mortem: credential accidentally reuses admin/auth, is misnamed, leaks into argv, fails strict loader, inherits another cluster URL, or provisions authority by test SQL. Private file digest/role/peer checks plus real topology/designation proofs detect these. Blast radius is disposable test transport only. Rollback is a reviewed revert of harness changes; disposable containers/files are cleaned by existing traps, and no persistent database or production credential is changed. Stop on source-preimage drift, unexpected grant/role changes, raw credential output, non-owner RED, zero tests, missing runtime prerequisites, or any request to weaken original assertions. Reviewer: independent integration_security; root records final candidate, admission and verification receipts. Production exposure, reset execution, Company acceptance and release qualification remain HOLD.

Root reports the discovered module prefix is `account_browser::`; the corrected commands above bind that prefix. A quicker exact Cargo map may run the same unchanged owning test through its actual harness. That proves the Cargo→owner route; it does not replace the still-required Buck rebuild/executor evidence. Preserve each runner's verification gap separately.
