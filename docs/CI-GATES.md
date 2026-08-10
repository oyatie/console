> **EXECUTABLE-CONTRACT INVENTORY / NON-AUTHORITY:** This path-stable inventory documents currently present and historical machinery; its exact headings and lists remain machine checked. It does not authorize product scope, roadmap order, issue closure, release, or production readiness. Current authority begins at [`README.md`](../README.md) and the three documents under [`docs/current/`](current/).

# CI Gates

The GitHub Actions workflow in `.github/workflows/ci.yml` is the **executable inventory**
of CI enforcement (this doc is a non-authority mirror). This document mirrors the current gate inventory and splits
the checks into two groups: core local gates that a fresh development session can
run directly, and CI-contextual/heavy gates that need platform-specific runner
setup, services, or deployment access.

The gates encode the project's non-negotiable invariants (clean-architecture
layering, audit-first discipline, 위치정보법/PIPA data handling, multi-tenant
isolation, and the committed API contract) so that a violation fails before
production. Do not treat a lightweight local loop as full CI confidence: a
change is not "done" until the relevant local gates, review evidence, and CI
jobs for the touched surfaces are green.

### Stable required-context migration

At release `0.3.2`, branch protection requires the ten CI leaf contexts, five
Security leaf contexts, and `authenticate-console-authority` as sixteen exact,
GitHub-Actions-app-bound checks. The leaf proofs remain required while the
workflows introduce the shadow contexts `Required / CI` and
`Required / Security`.

`Required / CI` is a same-workflow strict-success aggregate over the exact ten CI
jobs. `Required / Security` applies the same rule to the exact five Security
jobs. Both run under `always()` and fail when any dependency fails, is cancelled,
or is skipped. Their dependency and result sets are mutation-tested and locked by
the repository preflight gates. Neither aggregate checks out repository content
or executes repository scripts; each runs only its locked status comparison.

The protected-target authority job remains a separate required boundary. Do not
combine it with candidate-controlled CI/Security workflows or duplicate an
aggregate display name across workflows. Only after both aggregates pass on the
exact pull-request train and its post-merge `main` commit may branch protection
be atomically migrated to the three stable contexts: `Required / CI`,
`Required / Security`, and `authenticate-console-authority`. Retain all existing
leaf requirements until that shadow evidence exists, bind the replacements to
the GitHub Actions app, preserve strict up-to-date checks, and immediately read
the live rule back after mutation.

## Review evidence gate

## Console authority bootstrap

`Console authority bootstrap` is a protected-`main` `pull_request_target`
gate for console authority trains targeting `main`. It intentionally checks out only
protected `main` code with the complete Git graph, fetches PR Git objects without
checking them out, verifies signed candidate
`C` and direct authority tip `T` against the pinned SSH signer, and treats the
GitHub synthetic merge `M` only as an unsigned structural object. Git commands and
candidate subprocesses run with a sanitized Git environment that ignores inherited
Git config and `HOME`/XDG configuration. Only after that
does it create a detached `C` worktree to run the candidate validator, planner, and
their unit tests.  It has no secrets, cache restore, npm install, or PR executable
step before authentication. This bootstrap must remain protected-target code; a
workflow supplied by the PR cannot establish its own trust root.

The five required Security contexts inspect a candidate checkout and therefore
are not, by themselves, an isolation boundary against deliberately hostile PR
code. Their workflow shape and proof order are locked against accidental drift,
but merge admission additionally depends on `authenticate-console-authority`,
which runs protected-target code and rejects every PR—including forks and
Dependabot—whose exact C/T train is not signed by the pinned authority. Never
remove that required-context composition or treat a green scanner context alone
as authentication of an untrusted contribution.

On a closed, merged same-repository `main` PR, the separate squash-binding job first
checks out exact `S`, verifies that `HEAD` is `S`, then hook-disabled detaches to
protected `S^` before it invokes any repository script. It verifies that the
one-parent squash commit `S` is bound to the signed `C`/`T` authority train: its only
parent is the trusted pre-merge base and its tree is exactly `T`'s tree. It emits the non-release
`console-squash-binding-v1` receipt with `TREE_BOUND_HOLD_PRESERVED` and release
disposition `HOLD`; it never checks out or executes `T` or `M`. `S` is checked out
only as an object/tree source, `HEAD` is verified as `S`, and the job then
hook-disabled detaches to `S^` before any repository code executes, so no repository
code from `S` runs. Before binding,
the protected `S^` process fetches `refs/pull/<number>/head` into a private namespace
and requires its SHA to equal the closed-event authority-tip SHA, so a deleted PR head
branch cannot make the signed `T` object unavailable.

For user-facing features, PR/review evidence must prove the shipped workflow, not
just the transport seam. API endpoint, handler, and contract tests are necessary
contract evidence, but they are **not sufficient** for UI feature claims. UI
evidence belongs with the implementation that owns that surface; this post-pivot
repository no longer contains the former web/mobile clients or their E2E jobs.

The product guardrail is CRUD-first SaaS: database-backed create/read/update/
delete UI and normal editing workflows come before upload/import/Excel paths.
Upload/import/build requests from non-technical staff are product inputs, not
product authority; reviewers should reframe or reject them when they weaken SaaS
maturity or bypass first-class CRUD workflows.

## Local gate runbook

Start with the core local gates for the surfaces you touched. The command list
below intentionally separates always-local commands from CI-parity/heavy surfaces
so a fresh session does not gain false confidence from a partial run.

```bash
# Backend core (from backend/)
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings
SQLX_OFFLINE=true DATABASE_URL=postgres://<user>@localhost/console_dev cargo test
for g in \
  layer-boundary audit-coverage migration-safety tenant-isolation pii-no-logs \
  rls-arming dev-auth-absence iac-tier fabricated-branch \
  personal-data-classification; do
  cargo run -q -p console-gate-$g            # each must exit 0
done
SQLX_OFFLINE=true cargo test -p console-platform-auth-rest --features dev-auth
SQLX_OFFLINE=true cargo test -p console-app --features dev-auth --test dev_auth_persona_guard_feature
SQLX_OFFLINE=true cargo test -p console-platform-provisioning --test dev_principal_upsert_race

# API contract gates (from repo root after npm ci)
npm run check:platform-contract-drift     # platform route inventory vs committed openapi.yaml
npm run test:employee-import-contract
npm run test:ontology-write-precondition

# Root repository gates (from repo root after npm ci)
npm run test:adrs
npm run check:adrs
for s in \
  check:foundation-gates \
  check:executed-tests \
  check:test-credentials \
  check:request-body-contract \
  check:doc-citations \
  check:doc-manifest \
  check:doc-links \
  check:package-lock \
  check:ci-preflight \
  check:g004-identity-foundation \
  check:g005-workflow-lifecycle \
  check:workflow-runtime-spine \
  check:workflow-runtime-m2-strangler \
  check:workflow-runtime-m2-cedar-guards \
  check:workflow-runtime-m2-runtime \
  check:workflow-runtime-m2-drainer \
  check:g006-asset-dispatch-lifecycle \
  check:g007-collaboration-mobile-lifecycle \
  check:g008-payroll-readiness \
  check:people-hr-maturity \
  check:payroll-release-gate \
  check:undeclared-imports; do
  npm run "$s"
done

# Deployment gates
npm run check:k8s                         # render manifests; CI warns if no live cluster
CONSOLE_NETWORKPOLICY_PREFLIGHT=require npm run check:k8s:networkpolicy
CONSOLE_NETWORKPOLICY_EXPECTED_ENFORCER=cilium \
  CONSOLE_NETWORKPOLICY_SMOKE_POSTGRES=auto \
  npm run smoke:k8s:networkpolicy-deny
npm run check:production-hardening
```

CI also runs heavier or runner-contextual gates. Reproduce them locally only when
their prerequisites are available:

- `npm run dev:bootstrap`, `/readyz`, and `npm run dev:down` for the dev-up
  smoke. CI also runs the compose contract unit test and PostgreSQL topology
  integration regression before the bootstrap.

The initial **CI preflight** job runs the foundation-gate, CI-preflight,
executed-test reachability, credential-argument, and deterministic-lockfile
contracts before the expensive backend and database jobs begin.

As measured on 2026-08-03, `check:executed-tests` inventories 333 defined test
binaries: workflow commands directly select 320, leaving an exact dark set of
13. Those reachable binaries contain 319 source files and 2,097 lexically
declared test attributes. The attribute count is a source ratchet, not a runtime
case count: it does not evaluate `cfg`, feature selection, macro expansion, or
`ignore`. The SeaweedFS tests are recorded in the dark set as out of pivot; they
are not claimed as executed coverage. Any new or lost declared attribute must
update the exact baseline intentionally in the same change.

`check:test-credentials` has a deliberately narrow static scope: it rejects
literal passwords on workflow commands that spell a test runner, and its
runtime half exercises the opt-in `pgtest.sh` argv guard. It does not prove that
arbitrary scripts or tests cannot construct credentials internally.

`SQLX_OFFLINE=true` uses the committed `.sqlx/` query cache; regenerate it with
`cargo sqlx prepare --workspace -- --all-targets` (note `--all-targets`, so test
queries are cached too) against a database migrated to head.

## Current CI workflow gate inventory

This inventory is sourced from `.github/workflows/ci.yml` and the root
`package.json` scripts. When the workflow changes, update this table and the
runbook together.

`npm run check:foundation-gates` machine-checks the three lists below against the
workflow and package manifests. The lists intentionally track stable command/gate
names only, not incidental workflow prose or runner setup text.

### Backend console-gate binaries run by CI

- `console-gate-audit-coverage`
- `console-gate-dev-auth-absence`
- `console-gate-fabricated-branch`
- `console-gate-iac-tier`
- `console-gate-layer-boundary`
- `console-gate-migration-safety`
- `console-gate-personal-data-classification`
- `console-gate-pii-no-logs`
- `console-gate-rls-arming`
- `console-gate-tenant-isolation`
- `console-gate-writer-ownership`

### Root package scripts run by CI

- `check:adrs`
- `check:ci-preflight`
- `check:console-truth-ledger`
- `check:doc-citations`
- `check:doc-manifest`
- `check:gate-input-provenance`
- `test:gate-input-provenance`
- `check:doc-links`
- `check:executed-tests`
- `check:js-test-reachability`
- `test:js-test-reachability`
- `check:foundation-gates`
- `check:g004-identity-foundation`
- `check:g005-workflow-lifecycle`
- `check:g006-asset-dispatch-lifecycle`
- `check:g007-collaboration-mobile-lifecycle`
- `check:g008-payroll-readiness`
- `check:k8s`
- `check:platform-contract-drift`
- `check:test-credentials`
- `check:package-lock`
- `check:payroll-release-gate`
- `check:people-hr-maturity`
- `check:pr473-migration-operational`
- `check:production-hardening`
- `check:request-body-contract`
- `check:undeclared-imports`
- `check:workflow-runtime-m2-cedar-guards`
- `check:workflow-runtime-m2-drainer`
- `check:workflow-runtime-m2-runtime`
- `check:workflow-runtime-m2-strangler`
- `check:workflow-runtime-spine`
- `test:adrs`
- `test:employee-import-contract`
- `test:executed-tests-baseline`
- `test:ontology-write-precondition`
- `test:production-hardening`
- `test:text-gate`

### Documentation manifest — exact index-tree custody

- `npm run check:doc-manifest` runs `node scripts/console/generate-documentation-manifest.mjs --check`.
- `npm run check:gate-input-provenance` emits the `gate_inputs` relation (separate from document `class`) and enforces the exception register.
- `npm run test:gate-input-provenance` runs hermetic provenance unit tests plus a live instrument pass.
- The gate fails closed when a tracked first-party Markdown blob is unclassified, a recorded `blob_sha` differs from the exact Git index-tree blob OID, the eight-field seed or closed vocabularies drift, schema-v2 generated bytes are stale, or any of the seven entry/authority/transition projections differ.
- Regenerate generated `path`/`blob_sha` fields and `docs/documentation-index.json` with exactly `node scripts/console/generate-documentation-manifest.mjs --write`; semantic fields remain review-owned and missing semantics remain a failure.

- **Domain crates — unit tests**: an explicit, ratchet-checked set of Cargo
  `--lib`, doctest, and named non-PostgreSQL test targets. This is broad selected
  reachability, not a claim that one full-workspace `cargo test` runs.
- **Backend — fmt / clippy / gates**: `cargo fmt --all -- --check`,
  `SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings`, eleven `console-gate-*` binaries
  (`layer-boundary`, `audit-coverage`, `migration-safety`, `tenant-isolation`,
  `pii-no-logs`, `rls-arming`, `dev-auth-absence`, `iac-tier`,
  `fabricated-branch`, `personal-data-classification`, `writer-ownership`), their named mutation
  suites, and the explicitly named PostgreSQL harness targets in this and the
  dedicated reachability jobs. `check:executed-tests` is the inventory ratchet.
- **dev-up.mjs smoke — compose deps + migrate + /readyz**:
  the compose contract unit test, PostgreSQL topology integration regression,
  `node scripts/dev-up.mjs bootstrap`, `/readyz` curl, and unconditional
  `node scripts/dev-up.mjs down` cleanup.
- **Repository gates — governance and domain contracts**: ADR, documentation,
  foundation, package-lock, workflow, domain-maturity, undeclared-import, and
  request-body gates named in the root-script inventory above.
- **API contract — platform route inventory (text-only)**:
  `npm run check:platform-contract-drift` plus the employee-import and
  ontology-write-precondition contract suites. The job builds and boots
  nothing.
- **Kubernetes manifests — render / hardening / NetworkPolicy preflight**:
  `npm run check:k8s` (render plus `scripts/check-networkpolicy-enforcement.sh`)
  and `npm run check:production-hardening`.

---

## Backend gates

### `rustfmt` — formatting

`cargo fmt --all -- --check`. Zero diff required.

### `clippy -D warnings` — lint + compile

`SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings`. **Every** warning
is an error, including in tests and benches. This also doubles as the offline
compile check (it fails if the `.sqlx` cache is stale or a query is malformed).

### `cargo test` execution — selected and named inventory

CI does not currently execute a single full-workspace `cargo test`. The
`Domain crates — unit tests` context runs the selected non-database library,
doctest, and integration targets enumerated in `.github/workflows/ci.yml`.
Database-backed suites run through explicitly named disposable-PostgreSQL
targets, some serialized because they mutate cluster-global roles. The
`check:executed-tests` ratchet must reject newly dark test roots; neither that
ratchet nor clippy is represented here as execution of every workspace test.

### `console-gate-layer-boundary` — clean-architecture + manifest hygiene

Source: `backend/ci/gates/layer-boundary/`. Enforces the dependency direction
([ADR-0001](decisions/ADR-0001-modularmonolith-cargo-workspace-with-compilerenforced-cleanarchitecture.md)):

```
kernel      → (nothing)
domain      → kernel
application → domain, kernel
adapter/platform → application, domain, kernel
rest/worker → adapter, platform, application, domain, kernel
app         → everything
```

Plus:
- **Purity:** `domain` and `application` crates may not depend on `sqlx`, `axum`,
  or `tokio` (no I/O in the pure core).
- **Manifest hygiene:** every workspace crate name starts with `console-`, uses
  `edition.workspace = true`, opts into non-publishability with
  `publish.workspace = true` (inheriting workspace `publish = false`) or direct
  `publish = false`, and carries `[lints] workspace = true`.
- **Conflict-marker scan:** rejects any git-tracked file containing unresolved
  merge markers (`<<<<<<<`, `=======`, `>>>>>>>`). Added after MFL-0001
  (a merge commit shipped with unresolved markers); see
  [MISTAKES-LEDGER.md](MISTAKES-LEDGER.md).

### `console-gate-audit-coverage` — audit-first discipline

Source: `backend/ci/gates/audit-coverage/`. Every state-changing handler marked
`// console-gate: state-changing-handler` must construct an `AuditEvent` and route
its mutation through `with_audit` / `with_audits` / `insert_audit_event`, so the
audit row is written in the **same transaction** as the mutation
([ADR-0002](decisions/ADR-0002-auditfirst-transactional-discipline-audit-event-in.md)).

The **sole** carve-out is LocationPing ingestion: raw GPS coordinates must remain
destructible and must never enter `audit_events`
([ADR-0014](decisions/ADR-0014-locationping-destructible-store-carved-out-of.md),
위치정보법). That exemption is **path-bound** to the
single real writer (`crates/compliance/adapter-postgres/src/lib.rs ::
record_location_ping`) — the same exemption reason on any other file/function is
rejected. (Path binding was hardened in `fix/harden-1`; previously the exemption
matched on reason only, which could silently apply to the wrong handler. The
historical review path was `.omc/review/security-compliance.md`; runtime state is
not repository authority.)

### `console-gate-migration-safety` — append-only audit trail

Source: `backend/ci/gates/migration-safety/`. Migrations are append-only and may
not erode the audit trail. It rejects:
- `DROP TABLE` on an audited table,
- `ALTER TABLE … DROP COLUMN` on an audited table,
- `GRANT UPDATE`/`GRANT DELETE` on `audit_events`,
- `DISABLE TRIGGER` on `audit_events`.

The append-only protection on `audit_events` (REVOKE UPDATE/DELETE + trigger) is
thus immune to being silently undone by a later migration.

### personal-data classification — two checks, neither subsuming the other

Field-level classification is guarded by a **pair** of checks that read
different things, and both must run. They are not equal partners: the catalog
assertion carries the column-membership class, and the text gate is defence in
depth for what only a text reader can do. Earlier versions of this section have
been falsified by the next probe three rounds running, so what follows is
written as measurements with their exit codes rather than as guarantees.

| | reads | CI step |
| --- | --- | --- |
| `console-gate-personal-data-classification` | migration **text** | `Personal-data-classification gate` (`cargo run -p console-gate-personal-data-classification`) |
| `every_application_column_is_classified_or_its_table_is_declared` | the **catalog** of a migrated database | `Serialized disposable PostgreSQL integration targets` → `//tools/buck:platform-db-personal-data-classification-pg` |

**The gate**, `backend/ci/gates/personal-data-classification/`, parses every
migration into a post-migration column set and requires each column, in a table
not listed in `unclassified-tables.txt`, to carry a
`COMMENT ON COLUMN … IS 'pd:<tokens> …'` marker drawn from a closed vocabulary
(`none`, `personal`, `sensitive/*`, `unique-id/*`, `credit`, `pseudonymous`,
`undeclared`).

**The catalog assertion**,
`backend/crates/platform/db/tests/personal_data_classification.rs`, enumerates
every column of every application table out of
`pg_attribute`/`pg_class`/`pg_namespace` after the migrations have run and
requires the same, against its own Rust-side baseline — the per-table SET of
unclassified column names, generated from the live catalog. Migration 0211 reads
the same markers back out of `pg_attribute` so the 접속기록 retention floor is
derived from the schema rather than hand-maintained.

**What each one cannot see, and which of them the class now depends on.** The
gate re-implements PostgreSQL's parser, so it reads some constructs wrong and
passes on them: schema qualification is dropped (`shadow.employees` registers as
`employees`), quoted identifiers are case-folded (`"Employees"."RAW_ROW"` reads
as `employees.raw_row`), a plpgsql body that assembles a keyword out of fragments
— `DO $$ BEGIN EXECUTE 'ALTER TA' || 'BLE leave_requests ADD COLUMN
medical_certificate_no TEXT'; END $$;` — is read as building nothing because the
body scan looks for `table` within four words of `alter` and finds neither
spelled whole, and a multi-action `ALTER TABLE` is judged from its FIRST action,
so this repository's own house idiom with one comma appended —
`EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY, ADD COLUMN
medical_certificate_no TEXT', 'leave_requests')`, the shape used in 25 migrations
— is read as column-neutral with nothing concatenated and every keyword spelled
whole. All were planted and the gate exited 0 on each.

**The parser is no longer what this class depends on, and is not being hardened
further.** Six rounds of hardening it each closed one spelling and each was
followed by another, and every one of those criticals needed TWO things: a
spelling the parser misreads, AND a net unclassified count that did not move,
because the catalog side's baseline pinned a per-table COUNT and a count is
payable — classify one existing column in the same migration that adds an
unclassified one and the number is unchanged. The last of them measured gate
EXIT=0, catalog EXIT=0, `3268 / 668 / 2600` against a clean `3267 / 667 / 2600`,
all 243 pins unmoved, the column confirmed live in `pg_attribute`. Those trios
are from that measurement and are not today's: a clean tree now reads
`3288 / 667 / 2621`, because `information_schema` entered the sweep afterwards.

That baseline is now the per-table **SET of unclassified column NAMES** — 247
tables, 2,621 names, generated from the live catalog by the `#[ignore]`d
`print_unclassified_baseline` in the same file and never typed by hand. A set has
nothing to pay with. Adding an unclassified column makes the set gain a name;
classifying one in the same change makes membership move. Both fail, in both
directions, with a message naming which columns landed unclassified and which
baseline names must be removed in the same change — a stale entry is a slot the
next column takes silently. Replanted against it and measured: the multi-action
house idiom, the `CREATE FUNCTION … AS 'BEGIN ALTER TABLE …'` body, and the
concatenation-split `'ALTER TA' || 'BLE …'` form each give **gate EXIT=0,
catalog EXIT=101**, each naming `medical_certificate_no` and `reason`. The last
of those is the residual the parser could never reach, and it is the proof the
new pin does not care how the DDL was spelled. A column added AND classified in
the same migration passes both, exit 0 each — without that the control would be
routed around within a week.

**So, precisely:** for any relation the catalog sweep reads, a parser blind spot
no longer composes into a silent live unclassified column. That is a claim about
the composite, not about the parser, and it stops where the sweep stops. What it
excludes, named rather than implied:

- **Relations created at RUNTIME.** In no migration text and in no catalog built
  from `./migrations`. This already ships:
  `0005_create_compliance_location_store.sql:90-121` creates a `location_pings`
  day partition per day with
  `EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF location_pings …')`.
  All ten `location_pings` columns are classified `pd:personal — 개인위치정보`, but
  every ping row lands in a partition neither reader sees, and nothing proves the
  child inherited the parent's markers.
- **The retention reader uses the same catalog universe.** Migration 0211's
  `personal_data_columns()` now reads the same non-`pg_catalog`, non-temporary
  `r/p/m/f` relations across schemas and returns schema-qualified identities.
  PostgreSQL tests strip the shipped markers, then independently plant a
  sensitive marker in a non-public table, a materialized view, and a foreign
  table; each must derive the two-year floor. This closes the former
  relkind/schema under-retention gap and mutation-locks the alignment.

That list had a third entry that was in the query and in no text.
`application_columns` also filtered `n.nspname <> 'information_schema'`, a
predicate its own doc comment never explained, and as the initdb superuser — who
is exactly who migrates in the pgtest and CI containers, though not the
`console_app` role production uses — `CREATE TABLE information_schema.pd_leak
(rrn TEXT, employee_name TEXT)` landed two live columns and left the catalog
counts byte-identical to a clean tree. The predicate is gone rather than written
down. What PostgreSQL 18.4 ships there under `relkind IN ('r','p','m','f')` is
four real tables — `sql_features`, `sql_implementation_info`, `sql_parts`,
`sql_sizing`, 21 columns, none commented, everything else in that schema being
`relkind = 'v'` — and they are declared in the baseline like any other
unclassified table. Replanted after the fix: **catalog EXIT=101**, naming
`information_schema.pd_leak` and both columns.

A fourth entry was in the query for the same reason and was closed the same way.
`application_columns` also filtered `n.nspname NOT LIKE 'pg\_%'`, justified in its
own doc by the claim that the prefix is reserved so nothing can hide behind it.
That claim is false: `allow_system_table_mods` is a `superuser`-context GUC, and
with it set, `CREATE SCHEMA pg_evil` succeeds where it otherwise raises
`42939 unacceptable schema name`. A migration planting
`pg_evil.pd_leak (rrn, employee_name)` with a stored 주민등록번호 gave gate EXIT=0
and catalog EXIT=0 with output byte-identical to a clean tree. The prefix pattern
is replaced by `n.nspname <> 'pg_catalog' AND c.relpersistence <> 't'` — the 64
system catalogs by name, session-temporary relations by PostgreSQL's own tag
rather than by the shape of a schema name — which adds zero baseline entries and
is *stricter* than the enumeration it replaces: a planted `pg_toast.pd_leak` is
also caught, because a clean `pg_toast` holds only `relkind` `t` and `i` and so
was excluded by the `relkind` predicate already. `pg_temp_N` needs no predicate;
`CREATE TABLE pg_temp_99.pd_leak` raises `42P16 cannot create relations in
temporary schemas of other sessions` even with the GUC on.

**The boundary, named once.** Both of those escapes needed **superuser**, and at
superuser the migrating session can `INSERT INTO pg_description` and forge a `pd:`
marker instead of hiding a table — measured, it succeeds. So this control assumes
a migration that does not exercise superuser catalog-write privilege. Production
migrates as `console_app` (`deploy/apps/console/base/migrate-job.yaml`), created
`NOSUPERUSER`, which reaches none of it: `42501 permission denied to set parameter
"allow_system_table_mods"`, `42939` on `CREATE SCHEMA pg_evil` with no way to lift
it, `42501 permission denied for schema pg_catalog`, `42501 permission denied for
table pg_description`. The pgtest and CI containers migrate as the initdb
superuser, which is why the probes land there. That boundary is not a reason to
leave a predicate unjustified — the `pg_` filter was closed anyway, because
closing it was free. The full statement lives in the module doc of
`backend/crates/platform/db/tests/personal_data_classification.rs`.

**The gate stays, as defence in depth.** It is the only reader that sees a column
at WRITE time, before any database exists, in milliseconds on a developer's
machine, and the catalog is consulted only for relations it knows. A newly found
parser escape is written into the residual register in its crate doc and left
there.

Two forms have left the blind-spot list, and the distance between them is the
lesson.
`DO 'BEGIN CREATE TABLE …; END'` was closed by refusing a `DO` that carries a
single-quoted literal — a refusal written at the STATEMENT HEAD. The identical
block one head over,
`CREATE FUNCTION f() RETURNS void AS 'BEGIN ALTER TABLE … ADD COLUMN …; END'
LANGUAGE plpgsql`, then passed the gate AND the catalog assertion, exit 0 each,
over a column live in `pg_attribute` on PostgreSQL 18.4;
`CREATE OR REPLACE FUNCTION` likewise. The fix was right in kind and one level
too high. The body scan now reads `Tok::Str` alongside `Tok::Body`, below the
head, so a body carrying table DDL is `unsupported-ddl` under either quoting and
under any head. The `DO` refusal is kept on top of it, because
`DO 'BEGIN PERFORM 1; END'` carries no DDL for a scan to find; it is not extended
to `CREATE FUNCTION`, whose literals include parameter defaults
(`0064_platform_group_accounts.sql` writes `DEFAULT ARRAY['MEMBER']` and
`DEFAULT 'GROUP_ADMIN'`). Cost of the widened scan on the corpus, measured: 3,488
top-level single-quoted literals across 210 migrations, zero flagged.

Three properties carry the weight:

- **Unparseable means FAIL, by default rather than by list.** `apply_statement`
  has no fallthrough arm: the forms it recognises are the whole allow-list, and
  anything else raises `unsupported-ddl` naming the head it could not read. An
  earlier version enumerated the dangerous constructs instead, which is the same
  fail-open shape one step along — it tested `head[1] == "table"`, so
  `CREATE UNLOGGED/TEMP/GLOBAL TEMPORARY TABLE` and
  `CREATE SCHEMA x CREATE TABLE …` all walked past. A quoted body — dollar- or
  single-quoted, the scan does not distinguish — is refused the same way unless
  the DDL inside it is one of the column-neutral `ALTER TABLE` actions the parser
  already recognises, so a table built in a `DO` block or a plpgsql function body
  now fails instead of being invisible.
  `UNSUPPORTED_WAIVERS` records what the corpus still cannot parse — one entry,
  0005's per-day `location_pings` partitions — with a reason each, and a waiver
  that matches nothing is itself a violation.
- **The gate's baseline is shrink-only, enforced.** CI checks out a single
  commit with no history, so the gate cannot diff against the previous baseline.
  Migration numbers supply the clock instead: a column introduced after
  `BASELINE_FROZEN_AFTER_MIGRATION` is not sheltered by a baseline entry,
  whether it arrived on a new table or on one already listed. That clock is a
  filename prefix, so the gate also checks that no number at or below the freeze
  is reused or vacant — otherwise a new migration could name itself `0042_…` and
  be read as pre-freeze. What is *not* achieved: nothing here proves the
  baseline file is a subset of yesterday's, because yesterday's is not in the
  checkout. It proves that every way of adding an entry today is already a
  violation.
- **The catalog assertion's baseline is a set of column names, and two-sided.**
  It declares 247 of 286 tables, so a table-granular baseline would have
  sheltered 86% of the schema whole. Each entry therefore names that table's
  unclassified columns — 2,621 names today, of 3,288 columns. A name in the
  catalog that the entry does not list fails, saying it *landed unclassified*; a
  listed name that is no longer unclassified fails too, requiring the baseline be
  updated in the same change, because a stale entry is a slot the next column
  takes silently. Set membership rather than a count is what makes the two
  failures independent: classifying one column does not pay for adding another.
  Separately, the closed vocabulary is applied where `classified` is decided and
  across every application schema, so `COMMENT ON COLUMN x IS 'pd:lol'` is a
  violation, and being in the baseline does not shelter it — an entry admits a
  MISSING marker, never a wrong one.

Coverage is partial by design and the numbers are countable, not claimed: both
checks print their totals on every run. Listing a table in either baseline is an
admission that nobody has classified it — **not** a statement that it holds no
personal data. Neither check asserts anything about whether any statutory
obligation is met, and neither moves a compliance control off HOLD.

### `console-gate-pii-no-logs` — PIPA log hygiene

Source: `backend/ci/gates/pii-no-logs/`. Scans the bodies of logging macros
(`info!`/`debug!`/`warn!`/`error!`/etc.) and rejects:
- Korean mobile phone-number patterns,
- GPS coordinate pairs (two plausible lat/long floats together),
- resident-registration-number (주민등록번호) patterns.

PII/location data may be persisted (audited or destructible per policy) but must
never be written to logs.

### `console-gate-tenant-isolation` — RLS tenant-scope coverage

Source: `backend/ci/gates/tenant-isolation/`. Statically scans database
migrations and the audit GUC source path to ensure tenant-scoped tables are
classified, carry a non-null `org_id` unless explicitly allowlisted, enable and
force Row Level Security, and use policies bound to
`current_setting('app.current_org')`. It also rejects session-level or non-local
GUC arming that could bleed tenant context across requests.

The static scan is a fast source-level lint, not a reimplementation of
PostgreSQL privilege resolution. During the PostgreSQL 18 boot smoke, CI also
runs `owner_only_acl_postgres18` immediately after migrations execute as the
production owner role (`console_app`). That contract reuses the gate's owner-only
table allowlist and asks PostgreSQL for the effective `console_rt` table and column
privileges, so direct, `PUBLIC`, role-inherited, column-level, schema-wide, and
default-privilege grants are evaluated by the database itself. It also rejects
roles that `console_rt` can assume with `SET ROLE`, case-distinct table-like
relation shadows in `public`, and proves adversarial ACL mutations are
observable before rolling them back.

### `console-gate-rls-arming` — production queries use an armed org context

Source: `backend/ci/gates/rls-arming/`. Scans adapter/rest data-layer code for
query execution on a bare pool where no per-transaction `app.current_org` GUC is
armed. Legitimately global reads must carry an inline `// rls-arming: ok
<reason>` marker so each exception is reviewed and path-local.

### `console-gate-fabricated-branch` — defence in depth, NOT a control

Source: `backend/ci/gates/fabricated-branch/`. `authorize(principal, action,
resource_branch)` checks `principal.branch_scope.allows(resource_branch)`. Fed a
branch derived from the principal (`All => BranchId::new()`,
`Branches(b) => b.iter().next()/.any()`), that check is a tautology on both arms
and the branch dimension silently disappears. The gate scans `BranchScope::`
match arms for those shapes; a legitimate representative-branch pick (audit-row
actor branch, default branch for row creation) needs an inline
`// fabricated-branch: ok <reason>` marker, like `rls-arming` above.

**Read the module doc before trusting a GREEN.** This gate greps, and three blind
spots are known and unpatchable by more patterns:

1. a fabrication moved one function away scans clean — it reasons about match-arm
   bodies, never about what a caller does with the returned `Option`;
2. the `Branches` rule matches literal substrings only, so
   `branches.first().copied()`, `.iter().copied().next()`, `.nth(0)` and a plain
   `for` loop are invisible;
3. detection keys on the literal prefix `BranchScope::`, so any import alias
   defeats it entirely.

The control that would close all three is a `ResourceBranch` newtype
constructible only from a row read, so `authorize` cannot receive a
principal-derived value at all. That is a cross-lane signature change and is
queued separately; its absence is known, not overlooked
([DN-0004](decisions/notes/DN-0004-adr-0028-branchless-capability-authorization.md)).

### `console-gate-dev-auth-absence` — dev auth stays out of release defaults

Source: `backend/ci/gates/dev-auth-absence/`. Uses `cargo metadata` to prove the
`console-app` default feature set does not transitively enable `dev-auth`, so the
local role-switch endpoint cannot ship in the default release binary. HTTP-level
absence tests complement this feature-graph proof.

### `console-gate-writer-ownership` — one writer per canonical object

Two halves that read different things. Only the second one is load-bearing, and
they are wired into different jobs, so a reader who finds the `cargo run` step
and stops has found the weaker half.

| | reads | CI step |
| --- | --- | --- |
| `console-gate-writer-ownership` | production Rust **source**, parsed with `syn` | `Writer-ownership gate` (`cargo run -p console-gate-writer-ownership`) |
| `ops/postgres-reconcile-topology.sh` canonical block | the **capabilities** of a migrated database | PostgreSQL reachability facets → `writer-ownership-canonical-census-pg` in `tools/ci/postgres-cargo-map.json` |

**The database half is total.** For every canonical relation it asks
`has_table_privilege` for each non-expected role against each DML verb, pins the
table OWNER, and walks `pg_inherits` with a reachability CTE so a partition child
or an injected parent cannot carry a grant the roster never names. It fails
closed when it examined a set of tables that is not exactly the canonical roster
— "more than zero tables" was the previous spelling, and renaming one table away
shrank the scope from eight to seven while still passing.
`backend/ci/gates/writer-ownership/tests/census_executes_against_postgres.rs`
executes it against a real PostgreSQL and mutates the script to prove each probe
flips, because assertions about the script's *characters* survive
`IF leaked IS NOT NULL AND false THEN`.

**The source half is best-effort by construction.** It answers "which crate
writes this table" from parsed Rust, and two residual shapes are pinned by
`known_residual_` tests rather than claimed fixed. A SQL lexer is the total fix
(bead `console-tai.1`). Treat it as defence in depth; the census is the control.

Its mutation suite, `tests/gate_detects_violation.rs`, runs under **Domain crates
— unit tests** and not in the backend job's Buck2 mutation-suite step with the
other eight gates. Three of its 41 tests assert about *this* repository — one
walks the whole backend crate tree, two read
`ops/postgres-reconcile-topology.sh` — and a Buck2 action materializes only a
target's own `mapped_srcs`, so those three fail there with `os error 2`. The
requirement is a real checkout, which `domain-unit` has.

### Dev-auth feature build/tests — explicit non-default coverage

CI separately runs the non-default dev-auth path so the code remains healthy
without making it part of the release feature set:

```bash
SQLX_OFFLINE=true cargo test -p console-platform-auth-rest --features dev-auth
SQLX_OFFLINE=true cargo test -p console-app --features dev-auth --test dev_auth_persona_guard_feature
SQLX_OFFLINE=true cargo test -p console-platform-provisioning --test dev_principal_upsert_race
```

---

## API contract gates

The committed `backend/openapi/openapi.yaml` remains the reviewed API contract.
The post-pivot repository contains no generated client or frontend workspaces, so
the surviving CI job does not build or boot an application and does not claim a
client round-trip. It checks the committed document and source inventories
directly.

### `check:platform-contract-drift` — spec covers mounted routes

`node scripts/check-platform-contract-drift.mjs` parses `console-platform-rest` router definitions in
`src/lib.rs` and `src/view_as.rs` and fails when any `/api/platform/*`
path+HTTP-method is missing from OpenAPI, or when OpenAPI documents a platform
operation that the backend router does not define. The backend
`openapi_drift.rs` test continues to check each REST crate's exported
`*_ROUTE_PATHS` against OpenAPI path keys and mirrors the stricter platform
operation inventory via `PLATFORM_ROUTE_OPERATIONS`. Together these prevent an
unowned/undocumented HTTP surface (MFL-0002), including method-level platform
drift on already-documented paths.

Verification notes for platform route or DTO changes must name both halves of the
contract check: the route inventory comparison (`npm run check:platform-contract-drift`)
and the backend `openapi_drift.rs` suite.

---

## Deployment and hardening gates

The Kubernetes manifests job runs `npm run check:k8s`, which renders the
production overlays, guards Argo CD targets, and invokes
`scripts/check-networkpolicy-enforcement.sh`. Generic CI has no production
kubeconfig, so that live NetworkPolicy readback runs with
`CONSOLE_NETWORKPOLICY_PREFLIGHT=warn`: CI may prove manifests render, but it must
not be cited as proof that the target cluster enforces NetworkPolicy isolation.

Before deployment, an operator with a kubeconfig for the target cluster must run:

```bash
CONSOLE_NETWORKPOLICY_PREFLIGHT=require npm run check:k8s:networkpolicy
CONSOLE_NETWORKPOLICY_EXPECTED_ENFORCER=cilium \
  CONSOLE_NETWORKPOLICY_SMOKE_POSTGRES=auto \
  npm run smoke:k8s:networkpolicy-deny
```

That required mode reads the selected cluster context, confirms the `maintenance`
namespace has applied NetworkPolicy objects, and fails unless it detects a
policy-capable enforcer such as Cilium, Calico/Canal, Antrea, kube-router, or
OVN-Kubernetes. Plain flannel-only clusters fail the preflight. Use
`CONSOLE_NETWORKPOLICY_EXPECTED_ENFORCER=cilium` (or another supported value) when a
deployment context has a declared CNI owner. The denied-traffic smoke then creates
temporary same-namespace pods: an unlabeled control pod must reach an
`app=console-web` target on TCP/8080; an `app=console-app` client selected by
`default-deny-egress-app-tier` must resolve kube-dns, reach outbound HTTPS on
TCP/443, and reach `console-db-rw:5432` when the CNPG Service exists; that same
app-tier client must fail to reach the temporary HTTP target on TCP/8080. A smoke
PASS is the deny/allow packet evidence required for production isolation; a
preflight or smoke FAIL means wrong context/RBAC, missing policies, public
image-pull blocking (override `CONSOLE_NETWORKPOLICY_SMOKE_*_IMAGE` to approved
mirrors), no approved HTTPS probe, or a CNI/policy regression that must be fixed
before launch.

`scripts/deploy.sh` is the deployment output contract, not just a digest helper.
Default mode must fail closed unless it can produce fresh rollout evidence. A
deployment-complete claim requires all of these signals from the same run: a
successful `image-release.yml` run for the target commit; fresh `console-app` and
`console-web` digest artifacts; the prod overlay/bump revision that Argo should sync;
Argo Application `maintenance` reporting `Synced` at that revision;
`console-app`/`console-web` Rollouts Healthy; `console-worker` Deployment rolled out;
workload template image digests and running/ready pod image IDs or image
references matching the built digests; and HTTP 200 from both public endpoints.
Missing `kubectl`, missing target kubeconfig/RBAC, an unreachable Argo
Application, unavailable argo-rollouts plugin, rollout failure, pod readiness
failure, digest mismatch, or endpoint failure is a failed deploy verification, not
an optional skip.

`scripts/deploy.sh --digest-bump-only` / `--bump-only` is intentionally different:
it updates the desired prod image digests and prints that deployment, rollout,
pod-image, and endpoint verification were **NOT** run. Use it only when an
operator explicitly wants a digest bump from a host without cluster access; the
result must be documented as "desired prod digests updated only" and must not be
cited as deployed, verified, production-ready, or a G008 rollout completion.

After the Kubernetes check, CI/local validation still runs
`npm run check:production-hardening`. That production-hardening contract includes
the SMTP relay fail-closed guard: if the production-like `console-config` ConfigMap
sets non-secret `CONSOLE_EMAIL_*` relay fields (`CONSOLE_EMAIL_SMTP_HOST`,
`CONSOLE_EMAIL_SMTP_PORT`, `CONSOLE_EMAIL_FROM`, or `CONSOLE_EMAIL_FROM_NAME`), the API and
worker manifests must explicitly require `CONSOLE_EMAIL_SMTP_USERNAME` and
`CONSOLE_EMAIL_SMTP_PASSWORD` from `console-secrets` via non-optional `secretKeyRef`
entries. `envFrom` alone is not enough because Kubernetes silently omits missing
Secret keys; local/dev/e2e stub-email configs should omit the whole SMTP relay
group. Local reproduction needs the same renderer tooling that CI installs,
including a compatible `kubectl`/kustomize runtime.

These are manifest and desired-state gates, not live packet-enforcement proof.
They prove that the NetworkPolicy manifests such as
`deploy/apps/console/base/networkpolicy.yaml` render and that the production
hardening contract still points at the intended deployment surfaces. They do not
prove that traffic is isolated in a running cluster. Production NetworkPolicy
isolation requires a policy-capable CNI (the staged on-prem path uses Cilium;
Calico or Canal with Calico policy would be equivalent if explicitly selected).
Plain Talos/flannel renders NetworkPolicy resources inert even when the YAML
renders cleanly.

Security/review evidence for production networking must therefore pair the render
gate and `check:production-hardening` result with live CNI readiness plus
deny/allow DNS, Postgres-if-present, HTTPS, and explicit denied-flow connectivity
evidence from `npm run smoke:k8s:networkpolicy-deny` (or an equivalent recorded
pod-connectivity transcript) before claiming network isolation. Cross-reference
the enforcement notes in
`deploy/apps/console/base/networkpolicy.yaml`, the on-prem CNI stage in
`deploy/apps/cilium/README.md`, and the Talos on-prem substrate notes in
`deploy/talos/on-prem/README.md` when reviewing those gates.

---

## Retired pre-pivot mobile gates (historical)

> **Not current or runnable.** The `web/`, `android/`, and `ios/` trees and their
> workflows/scripts were removed after the pivot. The present-tense descriptions
> below are retained only as context for older review evidence; they do not
> describe the current repository or a merge gate.

### `check-i18n.mjs` — UI string-key parity

`node scripts/check-i18n.mjs` checks that web, Android, and iOS UI string keys
are present and consistent across the three clients (no missing/orphaned keys for
shared surfaces).

### Parity checklist — feature parity

Validates `docs/parity-checklist.md`: each shipped feature row names its Android
target, its iOS implementation, and the evidence commands that prove parity
([ADR-0009](decisions/ADR-0009-dualnative-swiftkotlin-parity-strategy-via-single.md)).

### iOS app — build + behavior tests

`swift build`, `swift test`, and `swift run ConsoleCoreBehaviorTests`
from `ios/`. The behavior runner mirrors the Android unit-test assertions for
shared domain logic (consent state machine, messenger reducer, sync, etc.). These
gates are local on macOS with a compatible Swift toolchain and otherwise rely on
the macOS CI runner.

### iOS UI tests — hermetic real-session XCUITest/accessibility gate

The standalone `.github/workflows/ios-ui-tests.yml` workflow is the CI-only
Simulator gate for SwiftUI post-login flows and accessibility coverage. It runs
every triggered push/tag or untrusted/public pull-request gate on one job-local
GitHub-hosted `macos-26` VM. Public pull-request code does not run on a reusable
self-hosted macOS runner. Any future self-hosted CI lane must use a separately
governed ephemeral/JIT runner group with teardown attestation; that lane is not
implemented or evidenced by the current workflow.

The current merge authority is Xcode 26.6 build `17F113`, Apple Swift 6.3.3 in
strict Swift 6 language mode, and the exact iOS 26.5 Simulator runtime. The job
fails if any version differs. XcodeGen 2.46.0 is downloaded into the job root and
verified against the repository-pinned SHA-256. It generates
`ios/Console.xcodeproj` from `ios/project.yml`; that project is a test
artifact, not a committed archive-capable project, TestFlight proof, or
release-signing gate.

The database, backend, build, and session boundary is job-local:

1. The workflow creates one mode-`0700` directory below `$RUNNER_TEMP` and puts
   `CARGO_HOME`, `RUSTUP_HOME`, `CARGO_TARGET_DIR`, the XcodeGen tool, PostgreSQL,
   backend identity/session state, DerivedData, `.xctestrun`, and test artifacts
   under that owned root.
2. It verifies `git rev-parse HEAD` against `GITHUB_SHA`, downloads PostgreSQL
   18.4 from the official source location, and verifies SHA-256
   `81a81ec695fb0c7901407defaa1d2f7973617154cf27ba74e3a7ab8e64436094`
   before building it. The mode-`0700` cluster and candidate `console-app` backend
   use separate random loopback ports.
3. It applies migrations and deterministic UI fixtures. Before each named shard,
   it generates a new random one-use OTP and stores only its SHA-256 digest in
   the database. The plaintext OTP and minted access/refresh tokens briefly
   reside in runner-local mode-`0700` job files and the mode-`0600` `.xctestrun`.
   GitHub masks those values; the artifact secret scan checks for them; cleanup
   deletes them with the owned job root. The `.xctestrun` remains below job-local
   DerivedData so `__TESTROOT__` resolves the built products.
4. The job executes all named shards even after an earlier shard fails. Each
   shard owns a process session, timeout watchdog, presentation, fresh fixture
   session, `.xcresult`, summary JSON, test-tree JSON, and timing record. A shard
   failure sets the aggregate status but does not hide failures in later shards.
5. `scripts/verify-xcresult-test-results.mjs` aggregates all shard results and
   requires the exact union of XCTest methods discovered from `ios/UITests`, with
   no duplicate, missing, skipped, failed, or errored case.
6. Before upload, the workflow scans the artifact tree for every raw OTP, access
   token, and refresh token minted during the job and fails on any match. The
   artifact upload step copies diagnostic results with seven-day retention.
7. After the upload attempt, unconditional cleanup verifies the backend process
   identity before stopping it, proves PostgreSQL is inactive, restores the
   Simulator to light appearance and `large` content size with exact readback,
   deletes the exact Simulator and proves its UUID is absent, removes generated
   CI files, deletes the complete owned job root, and proves that root no longer
   exists.

#### Named fail-slow shards and budgets

The manifest order, selectors, presentation, and hard budgets are part of the
fail-closed contract:

| Named shard | XCTest selection | Presentation | Budget |
| --- | --- | --- | ---: |
| `preflight` | `PreflightUITests` | light / large | 90 s |
| `login-validation` | `LoginValidationUITests` | light / large | 90 s |
| `accessibility-id-parity` | `ConsoleAccessibilityIDParityTests` | light / large | 45 s |
| `critical-path` | `ConsoleCriticalPathUITests` | light / large | 360 s |
| `messenger` | `MessengerUITests` | light / large | 210 s |
| `camera-capture` | `CameraCaptureUITests` | light / large | 90 s |
| `audit-dynamic-today` | Today Dynamic Type audit method | light / large | 150 s |
| `audit-dynamic-detail` | work-order-detail Dynamic Type audit method | light / large | 150 s |
| `audit-dynamic-messenger` | Messenger Dynamic Type audit method | light / large | 150 s |
| `audit-dynamic-login` | login Dynamic Type audit method | light / large | 120 s |
| `accessibility-standard` | four non-Dynamic-Type standard audit methods | light / large | 360 s |
| `accessibility-largest` | two non-Dynamic-Type AX5 audit methods | light / accessibility extra-extra-extra large | 240 s |
| `accessibility-dark` | two non-Dynamic-Type dark audit methods | dark / large | 240 s |
| `dynamic-type-large` | large Dynamic Type runtime contract | light / large | 150 s |
| `dynamic-type-ax5` | AX5 Dynamic Type runtime contract | light / accessibility extra-extra-extra large | 180 s |

Every shard records `test:<shard-name>` timing with its configured budget and a
terminal `passed`, `failed`, `timeout`, or `setup-failed` status. The measured
budgets total 2,625 seconds; the workflow reserves another 30 minutes for setup,
build, result verification, artifact handling, and cleanup under the 90-minute
job ceiling. Fresh per-shard sessions remain below the backend's 15-minute access
token TTL and keep suite duration independent of refresh success.

#### Shell-owned presentation and runtime proof

The workflow, not the test process, owns global Simulator presentation. Before
each shard it runs supported `xcrun simctl ui` commands for `appearance` and
`content_size`, queries both values, and requires exact equality with the shard
contract before minting the session or launching XCTest. App launch does not use
`-UIPreferredContentSizeCategoryName`, and tests do not mutate
`XCUIDevice.shared.appearance`. This avoids process-local assumptions and makes
dark mode and content-size setup independently observable.

The two runtime shards complement XCTest's audit API with layout evidence:

- `dynamic-type-large` proves that consent remains inline; the Messenger body and
  timestamp are hittable, visible, non-overlapping, and in one horizontal band.
- `dynamic-type-ax5` proves that consent moves to a system sheet; every consent
  control is visible and hittable; the grant button is at least 44 points high;
  and the Messenger timestamp moves below the body while both remain clear of
  navigation and tab chrome.

#### Xcode 26 Dynamic Type compatibility ledger

Xcode 26.6 reports a bounded set of synthesized SwiftUI-node Dynamic Type issues
that the runtime shards test directly. The handler accepts an issue only when all
of these fields match: audit type `.dynamicType`, compact description
`Dynamic Type font sizes are partially unsupported`, detailed description
`User will not be able to change the font size of this SwiftUI.AccessibilityNode`,
non-null element, exact accessibility identifier, and exact element type. The
observed sorted multiset must equal the expected sorted multiset, so a changed,
new, missing, or duplicate issue fails. All non-Dynamic-Type audit types run
handler-free through `.all.subtracting(.dynamicType)`.

The exact compatibility ledger is:

| Screen | Accepted synthesized nodes |
| --- | --- |
| Today | static text `locationConsent.title`; static text `locationConsent.stateLabel`; static text `locationConsent.stateValue`; static text `locationConsent.collectionLabel`; static text `locationConsent.collectionValue`; button `locationConsent.grant` |
| Work-order detail | static text `detail.symptom.label`; static text `detail.symptom.value` |
| Messenger | none |
| Login | none |

This is a toolchain compatibility ledger, not a general suppression list. Remove
an entry when the pinned merge-authority toolchain no longer emits that exact
issue and the affected Dynamic Type audit passes without it, while both runtime
proof shards remain green. Remove the helper only after all four methods pass
with empty ledgers and no issue handler. Make either change atomically with the
expected entries, fail-closed checker/mutation coverage, and this document. A
future Xcode or Swift version is not merge authority until its own pinned
workflow and evidence replace the versions above.

Missing configuration, fixtures, keychain entitlement, presentation readback,
session material, expected-test evidence, secret-scan evidence, or cleanup proof
fails the gate. No `-skip-testing`, `XCTSkip`, optional-session branch, external
backend/session secret, or fork-specific reduced suite is valid.

The job-local backend uses a CI-only loopback ATS allowance. Production
`ios/Sources/ConsoleApp/Info.plist`, TestFlight/archive settings, and
release networking policy remain unchanged. The shared keychain entitlement is
still required so the test process can seed the same production Keychain layout
that the app restores. The test resolves the granted group directly; a locally
supplied `CONSOLE_IOS_KEYCHAIN_GROUP` remains a diagnostic override, not a CI input
or session credential.

### Android app — build, unit/accessibility, and screenshots

From `android/`, CI runs `./gradlew build -x testReleaseUnitTest -x
testDebugUnitTest`, `./gradlew testDebugUnitTest`, and `./gradlew
verifyRoborazziDebug`. The first gate assembles/lint-checks without duplicate
unit-test execution, the second runs the Robolectric Compose UI/accessibility
tests, and the third verifies committed Roborazzi screenshot goldens.

### Android instrumented E2E — emulator-backed post-login workflow

The `android-instrumented` job in `.github/workflows/ci.yml` runs on a Linux
runner with KVM and Gradle Managed Device setup. It starts PostgreSQL 18.4,
verifies the checkout equals `GITHUB_SHA`, builds that candidate's `console-app`,
migrates and seeds an isolated database, boots the API on loopback, and redeems a
random short-lived mechanic OTP. The fresh token pair is stored in a mode-0600
runner-temp androidTest asset before `./gradlew fieldApi34DebugAndroidTest` runs.
The workflow deliberately avoids external backend/session secrets, GitHub step
outputs, and raw Gradle CLI arguments for token values.

The former `check:android-e2e-fail-closed` package script was the lightweight
regression guard for issue #359: it statically inspects the workflow and Android debug/release network
boundaries, while its mutation suite proves that external secrets, a non-PG18
database, missing exact-SHA verification, deterministic/unhashed OTPs, credential
leaks, skip-permitting result gates, and release cleartext regressions fail. It
does not start a GitHub runner or boot the Gradle Managed Device; the live
`android-instrumented` job is the full post-login evidence.

Runner-local inputs and artifacts are:

- `FIELD_E2E_SESSION_ASSETS_DIR` is an in-step runner-local handoff, not a
  repository secret. Gradle wires that directory into the
  `androidTest` assets and `WorkOrderFlowTest` reads
  `field-e2e-session.properties` for `FIELD_E2E_ACCESS_TOKEN` and
  `FIELD_E2E_REFRESH_TOKEN`. The workflow removes the source fixture, generated
  copies, and androidTest APKs after the run.
- `E2E_AUTH_DIR`, database/role passwords, the plaintext OTP, and JWT keys exist
  only under the job process or `$RUNNER_TEMP`. The OTP and minted tokens are
  masked; only the token asset path reaches Gradle.
- Debug permits cleartext only to the emulator host alias `10.0.2.2`; the base
  policy denies other cleartext destinations and release retains its HTTPS API.

Every context runs the same required hermetic gate. A missing fixture, failed OTP
exchange, unreachable protected API, absent `WorkOrderFlowTest` result, skipped
case, failure, or error fails the job; there is no optional self-skip path.

---

## CI-contextual dev-up gate

The dev-up smoke is local only when its service/runtime dependencies are
available:

- **dev-up smoke:** `node scripts/dev-up.mjs bootstrap`, `/readyz`, cleanup with
  `node scripts/dev-up.mjs down`. CI precedes bootstrap with the compose contract
  unit test and PostgreSQL topology integration regression.

---

## Notes

- The ten `console-gate-*` binaries exit non-zero on the first violation with a
  `file:detail` message; run an individual gate locally to see what it caught.
- When a change touches OpenAPI routes/schemas, run
  `npm run check:platform-contract-drift` and the backend `openapi_drift.rs` suite, plus
  the employee-import or ontology-write contract suite when that surface moves.
  A backend-only internal change that does not move API surfaces still needs the
  backend fmt/clippy/test/gate binaries and any touched-surface CI-contextual
  gates.
- Gate provenance and the incidents that motivated several checks are recorded in
  [MISTAKES-LEDGER.md](MISTAKES-LEDGER.md).

## Node dependency advisories

`security.yml` first runs `npm audit --omit=dev --audit-level=high` through the
repository-owned gate with **zero exceptions**. It then evaluates the full
workspace audit against `security/node-audit-exceptions.json`. That registry is
fail-closed: every entry must match the exact GHSA, package, installed version,
and lockfile path; it must name an owner/tracker/rationale, be `dev-codegen`
scoped, and expire within 30 days. A stale entry, an unmatched advisory, or any
new HIGH/CRITICAL result fails CI. An empty `entries` array is the canonical
state when the locked dependency graph has no HIGH/CRITICAL findings. The full
Trivy filesystem scan uses only the
matching, explicit `security/trivy-dev-codegen-exceptions.yaml`. That YAML is a
byte-for-byte deterministic projection of the canonical JSON registry and CI
verifies the projection immediately before passing it to Trivy, so rogue,
widened, missing, stale, or expiry-mismatched YAML entries fail before scanning.
Production npm audit is intentionally unfiltered.
