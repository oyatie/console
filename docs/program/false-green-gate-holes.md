# False-green gate holes — nine confirmed, one shape

Recorded 2026-07-25 during the wave-4 hotfix pass; H-9 added 2026-07-26. Every item below was a
**confident green over a broken reality**, and not one was caught by a gate —
each was found by a person or agent reading code and checking a claim.

They share a single shape: *the check verifies a proxy for the contract instead
of the contract*. Listing them together because the remedy is a class of gate,
not eight patches.

## H-1 · The drift gate compares path inventories, not request bodies

`openapi_drift` asserts that every configured route appears in `openapi.yaml`.
It never compares an operation's `requestBody` against the `#[derive(Deserialize)]`
struct the handler binds. So a spec can publish a body shape the server rejects
and CI stays green.

**Proven twice.** (a) The equipment handover fragment, applied ahead of its crate
diff, would have 422'd every console handover with no gate objecting. (b)
`PayrollRunStatus` was missing seven statuses across two schemas — green under
every existing gate, caught by reading the migration's CHECK constraint.

**Proposed check.** Diff each operation's `requestBody` required-set and property
names against the handler's bound struct. `#[serde(deny_unknown_fields)]` makes
the request direction strictly decidable: an undocumented field is a guaranteed
422, not a maybe. Enum variants as step two — that is the shape the payroll bug
took.

## H-2 · Hand-written client types bypass the generated ones entirely

`web/src/console/equipment/**` declares its own local interfaces instead of
consuming `clients/ts`. `tsc -b`, `check:api-drift:portable` and
`check:api-drift:swift` therefore all pass while the console posts
`evidenceReference` at a server that now requires `evidenceObjectId`.

This is H-1 from the opposite direction: H-1 is spec-vs-handler, H-2 is
console-vs-spec. A hand-typed client diverges from a *correct* spec with every
gate green.

**Proposed check.** Fail when a console module declares a local interface whose
shape shadows a generated schema, or — cheaper and blunter — require that any
module calling `/api/**` imports its types from the generated client.

## H-3 · Unit tests hand-feed literals the real column never produces

The §61 notice printed `미사용 연차 유급휴가는 13.000000일입니다` to workers.
`employees.leave_remaining` is `NUMERIC(16,6)` (widened by 0166), but **every
unit test hand-fed `notice_body` a `"13.00"` literal**, so only the DB-backed
test ever saw the real column.

**Proposed check.** Not fully mechanisable, but the operational rule is: any
value that reaches a user-visible or legally-operative string must have at least
one assertion sourced from the real column, not a fixture literal. Enforce in
review; the DoD already requires a runtime-role test per module — this says what
that test must *cover*.

## H-4 · The test suite resolves a package the lockfile does not contain

`package-lock.json` pins `react-router@8.3.0` and has **no `react-router-dom`
entry at all**; nothing in the workspace declares it. Local `node_modules`
nonetheless carried `react-router@7.17.0` *and* `react-router-dom@7.17.0` — a
stale tree matching neither. Local `tsc` and `vitest` passed against a phantom
package; CI runs `npm ci` from the lockfile and would fail to resolve, with 9
suites failing to LOAD (0 assertion failures).

Every web "green" in this program to date — including the coordinator's own
2792/2792 — was measured against that phantom tree.

**Proposed check.** A CI step (or pre-push gate) that verifies the installed tree
matches the lockfile, and a lint that fails on importing a package absent from
the manifest. `npm ls <pkg>` exits non-zero for undeclared packages; that alone
would have caught it.

## H-5 · A migration red/green proof can be a stale binary

`sqlx::migrate!` embeds the migration set at **compile time**, and cargo does
not track `.sql` files as inputs. So the standard red-proof ritual — remove the
migration, watch the test go red, restore it, watch it go green — produces a
**valid red and a meaningless green**: the restoring run links the stale binary
that still has the migration embedded, and would have passed either way.

Found first-hand by the L-A1 stage-2 verifier, which hit it while proving its
own work and reported it rather than banking the green.

**Rule for every backend lane.** A cargo red/green over a migration must touch a
crate source file, or `cargo clean -p <crate>`, *between the halves*. Otherwise
the green proves nothing. Buck is unaffected — the migrations tree is a declared
input there, so it rebuilds correctly.

This one is worse than H-1…H-4 in a specific way: those hid defects in the
product. This hides defects in **the evidence** — it can make a lane's proof of
correctness fraudulent without the lane intending anything of the kind.

## H-6 · A gate whose own fixture made it unfalsifiable

The PR 473 migration operational gate requires exactly one libtest
selection/result/summary per guarded regression. Its summary pattern ended the
line at `filtered out;` — but libtest **always** closes with ` finished in
<n>s`. That pattern could not match any real run, on any stream, so the gate
could never pass. It also read `completed.stdout`, while Buck2's simple console
(every non-TTY, i.e. every CI run) replays a test's captured stdout on
**stderr** under a `[<ISO8601>] ` prefix per line, defeating its `^` anchors.

The gate's unit test did not catch any of it, because the fixture hand-wrote a
summary line libtest never emits — **H-3, inside a gate**. That is the sharpest
form of this failure: H-3 in a product surface produces one wrong number; H-3 in
a gate silently disarms every check that gate is supposed to enforce.

It stayed invisible because CI preflight had been red on this lineage, so the
backend job had never run. A gate that has never executed is not a gate, and its
presence in a job list is not evidence of anything.

**Rule.** A gate that parses tool output must have at least one fixture captured
from a real run of that tool, pasted verbatim, not reconstructed from memory of
the format. Reconstructed fixtures test the author's belief about the format,
which is exactly the thing in doubt.

## H-7 · A display concern wired into a fetch dependency

Not a gate hole — a defect *class* the gates cannot see, recorded here because
it was found the same way (reading code behind a "flaky test").

The evidence register decorated rows with display names at fetch time, making
`resolveNames` a dependency of `loadList` and therefore of the list effect. When
the user directory resolved, the effect re-fired and re-seeded rows from the
**list** endpoint — which carries no holds, custody or copies. An evidence
object opened before the directory landed was redrawn as unheld, its chain of
custody empty. A legally-operative falsehood, produced by a cache-warming
refetch.

CI called it a flaky test. It was the product losing a race.

**Rule.** Data that a surface presents as authoritative must never be
re-seedable from a lossier endpoint. Derive display concerns at render; keep
fetched wire truth in state untouched. When a test in an authoritative surface
"flakes," suspect the surface before the test.

## H-8 · A migration guard silently took most of CI's tests offline

Found while checking whether a known-red test had ever been caught. It had not,
and the reason reached much further than that one test.

`0196_platform_force_command_and_fk_closure.sql` — which does not exist on
`main` — guards the force-role topology. Applying migrations now requires a
superuser named **exactly** `console_buck_admin`, with `SESSION_USER = CURRENT_USER`,
`mnt.sqlx_test_bootstrap = 'buck-sqlx-superuser-v1'`, a database matching
`^_sqlx_test_[A-Za-z0-9_]{52}$`, and ownership by the applier. The
non-superuser path admits only `console_app`, which cannot create test databases.

CI's service database runs as `postgres`. Under 0196 it therefore cannot apply
a single migration, so **every** `#[sqlx::test]` reached through it dies with
`platform_force_role_topology.superuser_test_bootstrap_required` before
asserting anything. This is not a property of one suite; it is a property of the
account.

**Two casualties, found one CI run apart.** The PR 473 gate's
`cargo test --workspace` was removed in `77768668` — forced, not careless, though
recorded with an empty commit body. The `Dev-auth feature build/tests` step's two
direct-Cargo commands were *not* removed and simply broke; nobody saw it because
the job had always died earlier, at the gate. Fixing the gate exposed it on the
very next run.

After the removal, CI's Rust test execution is:

| | |
|---|---|
| `//backend/crates/support/domain:console-support-domain-unit` | one crate's unit tests |
| `//backend/app:console-app-unit` | app `src` tests, `resource.none` half |
| `//tools/buck:app-inline-postgres` | app `src` tests, `test-postgres` half |
| `//tools/buck:app-dev-auth-persona-guard-postgres` | 1 of 63 story-test files |
| `console-gate-tenant-isolation --test owner_only_acl_postgres18` | one file |
| the Dev-auth step | 2 packages (only after the fix below) |
| the PR 473 gate | 4 disposable-PostgreSQL targets + 11 named regressions |

The two `console-app` targets share `crate_root = backend/app/src/lib.rs` and
partition on the `test-postgres` feature, so the app crate's **170 inline test
functions do run**. Measured against that:

| test functions | count | executed |
|---|---|---|
| `backend/app/src` (inline) | 170 | yes |
| `backend/app/tests` (story) | 254 | 1 file of 63 |
| `backend/crates/**` | 1,491 | essentially none |

So roughly **1,491 domain-crate and ~250 app story test functions execute
nowhere** on any push or PR, across 156 crates and 148 integration-test files.

**The remedy is cheap, and is now proven.** 0196 does not demand a separate
container — only the identity. Creating `console_buck_admin` as a superuser in the
existing CI service and putting
`options%5Bmnt.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1` on `DATABASE_URL`
satisfies the guard: sqlx then creates each `_sqlx_test_*` database owned by
that role. Verified against the exact suites 0196 had broken —
`console-platform-auth-rest --features dev-auth` went 0-passing to 15/15, and
`console-platform-provisioning --test dev_principal_upsert_race` to 1/1. That fix is
in this branch's `ci.yml`.

The same lever restores workspace-wide coverage: `cargo test --workspace` under
that URL is no longer impossible, merely un-run. It stays a separate charter
only because switching it on will surface a genuine backlog, not because
anything blocks it. The two known failures in `console-ontology-rest
object_type_cas_as_runtime_role` have since been fixed — and fixing them
uncovered H-9, which is what a backlog of un-run tests actually costs.

**Rule.** A guard that narrows *who may apply migrations* narrows *what CI can
execute* — the two are one lever. Any change to bootstrap authority must name,
in its own commit message, every suite it takes offline and where each is re-run.
Otherwise coverage leaves without a single gate turning red, and the loss is
discovered only when some unrelated fix lets the job run far enough to notice.

**Correction history — kept, because it is the point.** This section has been
wrong twice. It first called the workspace deletion careless, when 0196 had made
it impossible. It then put CI's total at "roughly fifteen tests", having grepped
only for `cargo test`/`buck2 test` and so missed everything routed through
`tools/buck/test_needs_postgres.sh` — which is how the app's 170 inline tests
run. Both errors flattered the finding. A document whose thesis is *confirm the
gate ran before citing it* has no standing to estimate its own numbers.

## H-9 · A tenancy test that never armed the tenant

Found by fixing an unrelated 500, which let a test reach assertions that had
never run.

`object_type_cas_as_runtime_role` proves that the Cedar normalization-blocker
queue is tenant-scoped. It seeds one blocker for each of two organizations,
then reads the table as the real `console_rt` role and asserts each tenant sees
exactly its own row. The reads were wrapped in `scope_org(org, …)`, which reads
as arming the tenant.

It does not. `scope_org` sets a **task-local** that adapter code consults when
it opens a connection; the assertions issued raw `sqlx` queries straight at a
pooled connection, which consults nothing. So `current_setting('app.current_org',
true)` returned the empty string, `NULLIF(…)::uuid` returned `NULL`, and the
policy predicate `org_id = NULL` matched nothing. **Both tenants read zero
rows.**

Zero rows satisfies "tenant A does not see tenant B's row". The assertion that
tenant A *does* see its own row is what caught it — and only because a separate
fix let execution reach it. Had the test been written with just the negative
half, it would pass today against a table with **no row-level security at all**,
against a dropped policy, against a revoked grant.

The neighbouring tests are not wrong: they call adapters, which arm the GUC
internally. This table has no Rust read path — it is written by migrations and
read by operators — so there was no adapter to inherit the arming from, and
nothing in the test made that absence visible.

**Swept for siblings; none confirmed.** All 76 test files using `scope_org`
were checked for the same shape — a `scope_org` block issuing raw `sqlx` at a
non-owner pool with nothing arming the GUC. Every candidate resolved as either
the `#[sqlx::test]` owner fixture (superuser, RLS bypassed, so the GUC is not
what makes the read work) or an adapter call that arms the GUC itself.
`policy/adapter-postgres/tests/draft_storage.rs` is the pattern done right: it
runs a store against an `console_rt` pool and asserts *both* that org A sees its row
and that org B does not. So this is one instance, not a class — but the
assertion asymmetry that hid it is a class.

**Rule.** A tenancy proof must be able to fail in the *permissive* direction. An
isolation test whose assertions are all of the form "X cannot see Y" is
satisfied by a read path that returns nothing to anyone; pair every one with a
positive assertion that the legitimate tenant *does* see its own data. And where
a test reaches past the adapters to raw SQL, the arming that production performs
must be performed explicitly — a scope helper that only sets a task-local looks
identical at the call site whether or not anything downstream reads it.

## The meta-finding

`openapi_drift`, the api-drift checks, `tsc`, `vitest`, `eslint` and
`check-ui-strings` were all green across the entire window in which every one of
these defects existed. Gate coverage is not the same as correctness coverage,
and this program has been reading the former as the latter.

H-6 sharpens that into something worse than a blind spot. A gate can be *worse
than absent*: an absent gate is visibly absent, while a gate that cannot pass —
or that has simply never executed, because an earlier job failed and everything
downstream reported `skipping` — occupies its slot in the job list and reads as
coverage. Every defect fixed on this branch was only reachable after CI
preflight went green for the first time on this lineage — and H-8 was only
reachable after that, by reading a gate that had just started passing.

Until H-1…H-4 have checks, "green" here means *green on what we thought to look
at* — and any claim of readiness should say so explicitly rather than cite a
gate list. The stronger form of the same discipline: before citing a gate as
evidence, confirm it *ran*, and that it has ever been observed to fail.
