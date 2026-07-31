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

**2026-07-31 — VERDICT: OPEN, and the check now exists but is NOT wired.**
Reproduced from code, not prose: `backend/app/tests/openapi_drift.rs` contains
zero occurrences of `requestBody` and zero of `deny_unknown_fields`.
`scripts/check-request-body-contract.mjs` implements the proposed check and is
red on a live product defect nobody had reported:

```
POST /api/v1/inventory/items/{item_id}/consumptions: spec property "quantity_consumed_milli" is not a field of ConsumeItemBody (deny_unknown_fields => 422)
POST /api/v1/inventory/items/{item_id}/consumptions: spec property "occurred_at" is not a field of ConsumeItemBody (deny_unknown_fields => 422)
POST /api/v1/inventory/items/{item_id}/consumptions: spec property "idempotency_key" is not a field of ConsumeItemBody (deny_unknown_fields => 422)
request body contract gate FAILED: 3 finding(s), resolved 51, skipped 172
```

`backend/crates/inventory/rest/src/lib.rs:ConsumeItemBody` (line 231) is
`rename_all = "camelCase",
deny_unknown_fields`; the spec publishes snake_case. Every spec-conformant
request to that endpoint 422s today, exactly as this section predicted for the
handover fragment. The sibling receipt body already publishes
`quantityReceivedMilli` — the struct is right and the spec is the outlier.

The gate is deliberately **not** wired into `ci.yml`. Fixing
`backend/openapi/openapi.yaml` is another slice's; allowlisting the operation
would suppress the gate's only true finding, and wiring a permanently-red gate
would break `main` for every lane. H-1 ships as *check written and proven red,
wiring blocked on the spec fix* — not as closed.

**Scope, stated so it is never read as more than it is.** The gate resolves 51
of roughly 288 request bodies. The rest bind no `Json<T>` or carry no
`deny_unknown_fields`, and are undecidable in this direction. Cite this as a
*floor* on correctness coverage. "Request bodies are checked" would re-commit
the meta-finding at the bottom of this document.

**Found while building it, unowned and outside H-1…H-4.** `openapi_drift`'s own
CI step is unprotected: deleting its `run:` line yields zero
`check:ci-preflight` failures. The flagship drift test is one line from silent
removal.

**2026-07-31, later the same day — CORRECTION: the verdict above is already
stale, in this document's characteristic direction. H-1 is CLOSED.** The
paragraph beginning "The gate is deliberately **not** wired" was true when
written and false within hours. `4e7da6b52 fix(openapi): the consume-item
request body 422s every conformant caller` fixed the spec, so the gate's only
true finding is gone and it now passes: `request body contract gate passed
(resolved 51, skipped 172)`. It is wired as `check:request-body-contract` in
`repo-gates`, and its step is locked in
`scripts/check-ci-preflight.mjs:requireOrderedStepContracts`. The original text
stays above because the correction is the point.

**The gate itself carried a false-green path, found only in the coverage pass.**
`scripts/check-request-body-contract.mjs:renameField` reproduces serde's
`rename_all`, and the whole comparison is that one function. Two of its eight
rules were guessed rather than read: serde's `LowerCase` is `field.to_owned()`
and its `UpperCase` is `field.to_ascii_uppercase()`, both keeping the
underscores, while the gate's `words.join("")` form dropped them — `very_tasty`
became `verytasty`. The quiet direction of that error is a spec publishing
`verytasty`, a guaranteed 422 under `deny_unknown_fields`, read as correct.
`rename_all = "lowercase"` already appears twice in `backend/**`. The
replacement is checked against serde's own `rename_fields` fixture table rather
than against a derivation.

**Two things in this slice were green for a reason unrelated to their name**,
which is the meta-finding at the size of a single test. The undeclared-import
suite's "does not mistake prose or SQL inside a string literal" case contained
no line that matched the import patterns at all, so it passed with the
suppression stubbed out; and
`scripts/check-request-body-contract.mjs:evaluateRequestBodyContract` could not
reach its own success branch while the spec was broken — the exit-0 path had
never executed. Both now have assertions that go red when the behaviour is
removed. Coverage on the two gates moved from 85.88%/84.78% branch to
95.40%/87.23%, measured with `node --test --experimental-test-coverage`.

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

**2026-07-31 — VERDICT: MISSTATED. The subject no longer exists. No check
written, deliberately.** `962fb98b7 chore!: clean slate — delete frontend, pivot
to the governed object engine (#503)` removed 899 `web/` paths. Measured today:

```
$ git ls-files | awk '/\.tsx?$/' | wc -l
       0
$ ls web clients
ls: web: No such file or directory
ls: clients: No such file or directory
```

Both sides of the comparison are gone: there is no hand-written client type and
no generated one. A gate here would have zero inputs on both sides and could
therefore never be proven red — an unfalsifiable gate, which is the exact defect
class this document exists to name. Building one so that the count of closed
holes reached four would be self-refuting. **If a client surface returns, H-2
returns with it and this verdict expires.**

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

**2026-07-31 — VERDICT: MISSTATED as an open hole. The defect is fixed. No check
written, deliberately.** The rendering defect no longer exists in code:
`backend/crates/leave/adapter-postgres/src/lib.rs:unused_leave_days` (line 1321)
reads

```sql
SELECT trim_scale(leave_remaining)::text FROM employees WHERE id = $1
```

`trim_scale` drops the storage scale's trailing zeros without changing the
value, and `::text` keeps it off a float, so `13.000000일` cannot be produced by
this path. This section's own **Proposed check** already concedes the hole is
"not fully mechanisable" and prescribes review enforcement rather than a gate.
Building a gate anyway would produce an unfalsifiable one — the failure this
document's meta-finding warns about — so H-3 stays a review rule.

**What is genuinely still open here is not a gate.** Nothing mechanically
prevents the *next* legally-operative string from being asserted only against a
fixture literal. That is the H-3 class, and it remains a review obligation
because no executable check for it can be proven red without inventing its own
subject.

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

**2026-07-31 — VERDICT: MISSTATED as narrated, OPEN as a class. Check written,
wired, and CLOSED.** The react-router story is dead with the frontend: the
lockfile now contains three entries in total.

```
$ node -e 'console.log(Object.keys(require("./package-lock.json").packages))'
[ '', 'node_modules/argparse', 'node_modules/js-yaml' ]
```

The **class** reproduced immediately, in this repository, today. Line 4 of
scripts/lib/kotlin-discriminator-unions.mjs — orphaned by #503, deleted here in
`3065d347e`, and therefore deliberately not cited as a live path — imported
`openapi-typescript`, present in neither `package.json` nor `package-lock.json`.
Its sibling test produced the exact H-4 signature, a LOAD failure with zero
assertion failures:

```
$ node --test scripts/lib/kotlin-discriminator-unions.test.mjs
    code: 'ERR_MODULE_NOT_FOUND'
ℹ tests 1
ℹ pass 0
ℹ fail 1
```

`scripts/check-undeclared-imports.mjs` (npm script `check:undeclared-imports`,
run by `repo-gates`) now fails when any bare specifier is undeclared in the
nearest `package.json`. It was observed red naming that file before the orphan
pair was deleted, and green after. Its step is locked in
`scripts/check-ci-preflight.mjs`, because before that lock `repo-gates`
protected none of its steps — deleting `run: npm run check:adrs` returned zero
preflight failures.

**One named exception, recorded because a silent one would be the meta-finding.**
The gate excludes `docs/evidence/**` as archived evidence and prints the excluded
set with its count on every run.
`docs/evidence/console/wave4/L-F1/browser-window-host.mjs:chromium` (line 30)
imports `playwright`, which web/package.json declared until `962fb98b7` deleted
it. That file is the instrument of a recorded verification result — four
citations rest on it, one recording `10/10 checks passed` — so deleting it to
obtain a green would trade audit evidence for a green light. The exclusion is a
named export covered by two tests, one of which disables it and observes the
gate go red on that exact file. **An archived evidence artifact that CI executes
would make this exception a real defect**; nothing under `docs/evidence/**` is
invoked by any workflow today.

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

**2026-07-31 — wrong a third time, and this one is left standing above rather
than edited away.** The opening sentence of this section says
`0196_platform_force_command_and_fk_closure.sql` "does not exist on `main`". It
exists on `main` right now, at
`backend/crates/platform/db/migrations/0196_platform_force_command_and_fk_closure.sql`.
The original text is kept because the correction is worth more than a clean
page: this document's characteristic failure is describing a problem in the
present tense after it has been fixed, which is the same direction as every
error above. The H-8 *analysis* of what CI's Rust execution consists of is
unaffected and still stands; only the existence claim was stale.

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

**2026-07-31 — H-1…H-4 adjudicated; two got checks, and that is the correct
number.** The sentence above assumed all four were open and all four wanted
gates. Verified from executable code, one was open (H-1) and three were
misstated: H-2's subject was deleted, H-3's defect is fixed, H-4's narration was
dead but its class reproduced here. Two executable checks shipped —
`check:undeclared-imports` (wired, step locked) and
`check-request-body-contract` (written, red on a live 422, wiring blocked on a
cross-slice spec fix). H-2 and H-3 got dated corrections and **no gate**: with
zero inputs on both sides, or with the defect already fixed and the section's own
remedy saying "not fully mechanisable", any gate built there could never be
observed failing. Shipping two unfalsifiable gates so the count reached four
would have instantiated this document's meta-finding inside the response to it.

The stale claim that made this re-reading necessary is itself the pattern:
this document stated that `0196_platform_force_command_and_fk_closure.sql` "does
not exist on main". It exists on main. **The documented failure direction here is
that the docs describe problems already fixed** — so read the gate, never the
prose about the gate.
