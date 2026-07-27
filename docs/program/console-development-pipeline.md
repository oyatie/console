# Console development pipeline

## Authority and scope

Status: active development-pipeline authority
Effective date: 2026-07-25
Product scope: the B2B SaaS Console only

This document governs how Console work is planned, implemented, reviewed,
integrated, verified, released, and rolled back. It is subordinate to:

1. the read-only canonical no-regrets engineering lifecycle;
2. accepted ADRs, including
   [ADR-0023](../decisions/ADR-0023-oyatie-console-authority.md) as amended by
   [ADR-0025](../decisions/ADR-0025-carbon-copy-console-shared-platform-spine.md);
3. the [Console enterprise roadmap](console-enterprise-roadmap.md);
4. the
   [machine-readable capability registry](console-capability-registry.json) and
   [Korea jurisdiction register](console-jurisdiction-register.json); and
5. immutable candidate, review, CI, image, deployment, and readback evidence.

Oyatie and the evolving Claude Design project are references. They are not
Console completion authority. NousResearch Hermes, Hermes Kanban, OMX, OMC,
GJC, untracked runtime state, agent chat, branch names, pull-request
descriptions, and local working directories are not planning, dispatch,
completion, or release authority.

Historical `.omc` documents remain traceability evidence. Do not delete or
rewrite them merely because their process is retired. Treat their commands,
queues, model assignments, `IN FLIGHT` labels, and completion claims as
non-authoritative unless a current repository-native contract and immutable
receipt independently re-establish the same fact.

This document does not claim that every required mechanism is implemented.
Where repository code or CI differs from this contract, the difference is a
pipeline defect or explicit `HOLD`, not permission to weaken the contract.

### Current implementation holds

The following typed holds record repository reality on 2026-07-25. They remain
open until an immutable later candidate supplies the named implementation and
verification evidence.

- **`HOLD-CI-BUCK2-001` — Cargo remains in required CI.**
  `.github/workflows/ci.yml:372-399` directly runs `cargo fmt`, `cargo clippy`,
  and eight `cargo run -p mnt-gate-*` binaries.
  `scripts/check-ci-preflight.mjs` and its tests currently require those Cargo
  steps. Buck targets already exist for the gate crates, including
  `//backend/ci/gates/layer-boundary:mnt-gate-layer-boundary`,
  `//backend/ci/gates/audit-coverage:mnt-gate-audit-coverage`,
  `//backend/ci/gates/migration-safety:mnt-gate-migration-safety`,
  `//backend/ci/gates/tenant-isolation:mnt-gate-tenant-isolation`,
  `//backend/ci/gates/pii-no-logs:mnt-gate-pii-no-logs`,
  `//backend/ci/gates/rls-arming:mnt-gate-rls-arming`,
  `//backend/ci/gates/dev-auth-absence:mnt-gate-dev-auth-absence`, and
  `//backend/ci/gates/iac-tier:mnt-gate-iac-tier`; their generated BUCK files
  also declare applicable unit/integration targets. The workflow and checker
  must migrate to Buck2-owned fmt, clippy, and gate execution, with equivalent
  fail-closed tests, before any Buck2-only Rust completion or release claim.
- **`HOLD-DOC-LINK-001` — no executable repository link gate was found.**
  A local relative-link scan can support review of a documentation commit, but
  it is not an enforced development-pipeline gate. An owned checker, regression
  tests, and cheap-preflight/CI wiring are required before documentation-link
  checking may be reported as enforced.
- **`HOLD-ROUTING-LEDGER-001` — adaptive routing telemetry is not persisted.**
  The quality-weighted routing rule below is desired policy. No
  repository-native route-outcome ledger currently binds task shape, selected
  role/model, estimates, cost, latency, review findings, retries, and terminal
  outcome to immutable revisions. Until that ledger, validator, and reporting
  path exist, do not claim statistical calibration or adaptive-routing
  improvement.

## Target outcome

The pipeline must maximize sustained delivery of complete Console modules while
preserving product quality and exact-revision truth. The optimization order is:

1. correct module behavior and production isolation;
2. graph and artifact locality;
3. safe concurrent progress;
4. fast feedback and low recovery cost; and
5. raw job or commit count.

Throughput is the rate of independently reviewed, fully wired user stories
admitted to an exact candidate. Lines changed, agents started, worktrees
created, and tests launched are load metrics, not product throughput.

## Immutable authority train

Every candidate pull request uses three distinct Git objects:

- **C — product candidate:** a signed full Git SHA containing the exact product,
  tests, build graph, and release inputs under evaluation.
- **T — authority tip:** a signed direct single-parent child of C. It changes
  exactly the three current Console authority documents required by the
  repository verifier and no product path.
- **M — hosted synthetic merge:** the two-parent merge object synthesized by
  the hosting platform for pull-request checks. T is its second parent. Its tree
  and content diff equal T exactly.

The topology is:

```text
trusted base ... -> C -> T
                    \   \
                     \   -> hosted checks derive M
                      ----> M has exactly two parents; parent 2 is T
```

The permanent invariants are:

1. C and T are immutable, signed, full SHAs.
2. T has exactly one parent, C.
3. C..T changes exactly:
   `console-capability-registry.json`,
   `console-jurisdiction-register.json`, and
   `console-program-ledger.md`.
4. M has exactly two parents and T is parent 2.
5. M and T have identical trees and an empty content diff.
6. Candidate facts come from C. Authority declarations come from T. Checks must
   never infer either from a moving branch.
7. The pull-request head is T, not C and not a locally manufactured M.
8. If C changes, create a new forward-only C -> T train. Do not amend, force
   push, or reuse receipts from the previous candidate.

Before publishing a train, create explicit preservation refs for at least:

- the product candidate C;
- the authority tip T; and
- any locally produced structural merge used for rehearsal.

Preservation refs are recovery anchors, not release authority. Record their
exact target SHAs. Never retarget an existing preservation ref to different
content. A hosted M is fetched and verified from the pull-request ref; it is not
reconstructed and presented as hosted evidence.

The executable topology authorities are
`scripts/console/verify-console-authority-train.mjs` and
`scripts/console/verify-console-pr-authority-bootstrap.mjs`. If prose and those
fail-closed contracts diverge, stop the train and reconcile the authority
before implementation or publication continues.

## Pipeline stages

Stages form a conveyor, not one global phase. Research, planning, contract work,
module implementation, review, verification, and fixes may operate
concurrently on different immutable inputs. A single module still moves through
the following ordered gates.

### 0. Research and refresh

Before decomposing a module wave:

- refresh the current Claude Design reference and record immutable digests;
- inspect the current repository behavior, APIs, schemas, tests, ADRs, roadmap,
  TODOs, and module evidence;
- identify observed user workflows, failure costs, Korea controls, and
  competitor patterns;
- classify every input as accepted authority, repository fact, dated external
  observation, inference, or historical evidence; and
- record contradictions and unresolved decisions as holds.

A moving design reference may refine acceptance criteria. It cannot provide
backend, authorization, audit, legal, deployment, or completion evidence.

### 1. Plan and admit work

Decompose work into module verticals in the capability registry. Each admitted
lane declares:

- one outcome and its executable user story;
- exact base SHA and intended product-candidate train;
- one owner;
- private writable roots;
- shared or generated faces it consumes but may not edit;
- API, schema, migration, and generated-client ownership;
- backend, frontend, user-story, security, observability, and rollback gates;
- resource demand; and
- independent-review requirements.

Unassigned ownership, overlapping writable roots, unresolved migrations,
missing user stories, or missing executable gates produce `HOLD`. They do not
produce speculative implementation.

Plan evolution is allowed between epochs. Replanning must preserve immutable
receipts already earned for unaffected exact SHAs and must explicitly
invalidate receipts whose input, contract, or acceptance criteria changed.

### 2. Run cheap preflight

Cheap, deterministic rejection runs before scarce Rust, database, browser,
mobile, or image resources. At minimum, preflight checks:

- clean worktree and exact base;
- signed-commit and authority topology prerequisites;
- package/lockfile consistency;
- generated-face registry and no-write drift;
- Buck2 manifest, cell map, target resolution, and affected-target planning;
- ownership and writable-root collisions;
- schema and OpenAPI source consistency;
- formatting and lintable metadata;
- documentation links only after `HOLD-DOC-LINK-001` is resolved by an
  executable repository checker;
- dev-auth production-exclusion declarations; and
- resource-budget admission.

A preflight failure cancels only dependent work. Independent module research,
contract work, frontend interaction work against typed contracts, and unrelated
review may continue.

### 3. Implement complete module verticals

Every exposed module is one frontend/backend vertical. Separate lanes may work
on disjoint parts of the same vertical, but the module is admitted only as one
integrated behavior.

#### Backend definition of done

- Business rules live in the domain/application boundary, not HTTP handlers,
  database adapters, or UI code.
- Reads and mutations use real persistence and production-shaped adapters.
- Tenant, organization, role, policy, and object authorization fail closed.
- Sensitive actions produce the required audit, idempotency, atomicity,
  concurrency, and compensation behavior.
- API schemas and errors are typed; generated clients derive from the canonical
  contract.
- Loading, empty, denied, stale, conflict, partial-failure, retry, and rollback
  behavior is defined where applicable.
- Unit tests cover pure domain/application logic without external services.
- Integration tests separately cover adapters, PostgreSQL/RLS, HTTP contracts,
  migrations, concurrency, and cross-boundary behavior.

#### Frontend definition of done

- The module meets the current Claude Design interaction and visual grammar,
  plus production accessibility and responsive requirements.
- Left navigation, workspace, object context, and right communication/activity
  rail preserve the selected object, user intent, drafts, deep links, and
  browser history.
- Reads and writes use the shared typed client against real backend contracts.
- Server authorization remains authoritative; the client denies by omission and
  never invents permission.
- Every visible control performs a real permitted action or is absent.
- Loading, empty, denied, stale, partial-failure, retry, recovery, and
  optimistic-conflict states are intentional and tested.
- Components are module-private until a proven repeated contract justifies a
  shared primitive. Shared visual and generated faces have a serialized owner.

#### User-story definition of done

- A named persona can complete the real workflow from entry to verified
  outcome using production-shaped data.
- The story proves the required source-object traversal, mutation, audit, and
  post-action readback.
- Error recovery preserves user work and produces a truthful state.
- Interaction cost, dead ends, ambiguous states, accessibility, responsive
  behavior, and console errors are measured.
- The permanent regression test replays the workflow against the real backend.

A module is not done when only one of these three definitions passes.

### 4. Review independently

Review begins as soon as an immutable leaf is ready and fans out independently
from unrelated implementation. The reviewer must be distinct from the
implementer and must bind the review to:

- exact base and leaf SHAs;
- exact changed paths and result digest;
- executable checks and their outcomes;
- module acceptance criteria and user stories;
- unresolved findings and explicit holds; and
- reviewer identity and signing authority.

Findings return to the owning lane. Fixes produce a new immutable leaf and a new
exact review. Review text without custody of the exact diff and evidence is
advisory, not admission.

### 5. Consolidate shared faces once

After reviewed leaves are ready, one integration owner uses one clean,
cache-warm integration worktree and the canonical local Buck2 daemon to:

1. replay only approved immutable leaves in declared order;
2. resolve collisions without rewriting leaf history;
3. update shared OpenAPI, generated clients, routes, navigation, translations,
   migrations, and generated Buck faces once;
4. regenerate through registered writers only;
5. run cheap admission again;
6. compute the affected Buck2 closure;
7. execute the bounded verification queue; and
8. produce the next product candidate C.

Do not run one integration daemon per module. Compatible exact-SHA work shares
the canonical daemon and cache. Create an isolated Buck2 daemon only for a
proven incompatible configuration, toolchain, cell map, or untrusted
experiment. Stop isolated daemons when their lane completes.

### 6. Verify the exact candidate

Rust completion evidence is Buck2-only. Cargo manifests and metadata remain
inputs to the generated graph; direct Cargo build, lint, or test output is not
Console completion evidence.

Run, in order:

1. authority and cheap preflight gates;
2. affected unit targets;
3. affected integration targets with isolated external-resource leases;
4. generated-client and frontend unit/interaction targets;
5. real-backend browser user stories;
6. mobile/device shards where the module changes shared behavior;
7. security, production-isolation, observability, and rollback gates; and
8. the required full backstop matrix.

Every generated Rust test target has exactly one `test.unit` or
`test.integration` label and exactly one resource label. Pure unit tests may
shard broadly. Integration tests are grouped by exact SHA and compatible
resource requirements. PostgreSQL, browser, iOS, Android, graph, and CAS pools
use independent capacity limits.

### 7. Authorize image, release, and deployment

Release order is fail closed:

```text
successful exact-SHA candidate CI
  -> immutable multi-architecture images
  -> scan
  -> sign
  -> provenance/SBOM attestation
  -> independent release authorization
  -> protected production promotion
  -> deployment
  -> health and user-story readback
```

Image Release is triggered by successful completed CI for the exact main SHA,
not by polling. A GitHub release, mutable tag, or deployment record must not be
published as successful before required candidate CI and immutable-image
authorization pass. A failed scan, signature, provenance, authorization,
current-main check, deployment health check, or readback leaves the release
held and preserves the last known-good deployment.

Production promotion consumes immutable image digests. It never rebuilds from a
branch, retries a raced main update, or mutates production from a normal push.

## Concurrency and resource admission

Concurrency is bounded by the scarcest real resource and by collision risk, not
by the number of available agents. For each epoch, record:

- `F`: available parent-process file descriptors after safety reserve;
- `C`: CPU capacity available after interactive and system reserve;
- `R`: RAM capacity available after system reserve;
- `W`: maximum disjoint source writers;
- `V`: independent reviewer capacity;
- `B`: Buck2 compile capacity;
- `P`: disposable PostgreSQL leases;
- `X`: browser workers;
- `I`: iOS simulator/device slots;
- `A`: Android emulator/device slots; and
- per-lane demand vectors for the same resources.

Admit a set of lanes only when:

```text
sum(demand[r]) <= budget[r] for every resource r
and writable roots are pairwise disjoint
and reviewer capacity covers the expected completed leaves
```

Reserve file descriptors for the parent application, Git, the canonical Buck2
daemon, live frontend/backend processes, and recovery tooling before admitting
agents or test workers. If process creation approaches the inherited descriptor
ceiling, stop idle tool servers and completed daemons before reducing test
coverage. Shell-local `ulimit` cannot repair a parent process that already
cannot create children.

The planner applies a slight quality bias. Among equally safe lane sets, prefer
the set with:

1. executable user stories and complete backend/frontend contracts;
2. available independent review;
3. high unlock value for downstream modules;
4. cache and exact-SHA affinity;
5. low shared-face and migration pressure; and
6. lower expected resource cost.

Do not admit more writers than review, integration, and verification can drain.
Excess work in progress increases stale-base rework and lowers completed-module
throughput.

## Desired execution-role and model routing

Automation is an execution mechanism, never work-item or completion authority.
The capability registry, immutable Git objects, executable gates, and signed
receipts remain authoritative regardless of which human or agent performs the
work.

This section defines the target routing policy. `HOLD-ROUTING-LEDGER-001`
prevents any claim that the estimates are currently persisted, calibrated, or
improving statistically.

Route bounded tasks by their dominant need:

- repository lookup to a fast exploration lane;
- official external evidence to a research lane;
- architecture and dependency trade-offs to architecture specialists;
- bounded code changes to implementation lanes;
- failures to debugging lanes;
- test design and flaky-test repair to test lanes;
- exact-diff acceptance to independent review lanes; and
- repository documentation to a documentation lane after behavior is verified.

Use only configured GPT-5.6-family models for agent-assisted Console work. Do
not silently fall back to GPT-5.5 or an unrecorded model. Model unavailability
holds the affected automated lane; it does not weaken review or verification.

For each candidate route `m` for task `t`, maintain observed estimates for
completion probability, severe-finding probability, latency, cost, and
resource demand. Select among capacity-admissible routes by minimizing:

```text
expected_loss(t, m)
  = quality_weight * failure_cost(t) * P(severe_failure | t, m)
  + latency_weight * E[latency | t, m]
  + economy_weight * E[cost | t, m]
  + congestion_weight * resource_pressure(t, m)
```

Set `quality_weight` slightly above economy and latency weights. Increase it
for authorization, data integrity, Korea controls, migrations, authority
trains, release, and rollback work. Use a lower-cost route only when its
observed quality remains inside the task's loss budget. Record the chosen role,
model family, reasoning class, estimate basis, exact input, result, review
findings, retries, and terminal disposition after the persisted route-outcome
ledger is implemented. Then update estimates from observed outcomes; do not
treat external benchmark rankings as repository evidence.

## Worktree and write-collision rules

- One immutable base and one owned branch per leaf.
- One writer per file or declared private root.
- No concurrent writes to shared routes, navigation, translations, OpenAPI,
  generated clients, migrations, capability authority, or generated Buck
  faces.
- Leaves emit registration intent or narrow source changes; the integration
  owner performs shared-face writes once.
- Do not leave required work only in an uncommitted worktree. Create signed,
  reviewable commits and explicit preservation refs before interruption or
  restart.
- Never discard or rewrite another lane's changes. Rebase or replay only from
  immutable reviewed commits.
- A dirty, stale, or ambiguous worktree is not an integration surface.

## Development authentication and live loopback

Keep a working frontend and backend available on loopback so the user can
observe progress and provide course correction. The live stack is development
evidence only unless its binaries, assets, data, and story receipt are bound to
the exact candidate.

Dev-auth is allowed only in explicitly selected local development builds:

- bind frontend and backend listeners to loopback;
- build the dedicated backend dev-auth target rather than changing production
  defaults;
- seed only disposable local development data;
- keep role switching explicit and visually identified as development-only;
- never include dev-auth routes, feature activation, seed data, secrets, or UI
  entrypoints in production backend binaries, web assets, or images; and
- prove production route and asset absence with Buck2-owned gates on the exact
  candidate.

A healthy dev-auth stack does not waive production authentication, policy, or
release evidence. Production absence is a mechanical gate, not a convention.

The minimum live smoke checks are:

- frontend root or target route returns successfully on loopback;
- backend readiness returns successfully on loopback;
- a dev-auth persona obtains a local session only in the dev-auth build;
- one changed module story reaches a real backend read and mutation;
- post-action readback matches the expected persisted state; and
- frontend/backend logs contain no unhandled error for the story.

## No-stub enforcement

Product-reachable code and shipped artifacts contain no stub, placeholder,
fixture-only path, filler text, simulated success, dead control, empty route,
hard-coded operational result, or disabled assertion used to imply completion.

Test doubles may exist only inside tests or explicit local harnesses that are
mechanically absent from production. If a required backend does not exist,
implement it in the module vertical. If a workflow cannot be completed, keep
the route dark and the capability in `HOLD`.

Final candidate sweeps search for suspicious markers and then inspect every
match in context. A zero string count alone is not proof; a renamed fake path is
still fake.

## Metrics

Record metrics per epoch and per exact candidate:

| Dimension | Required measures |
|---|---|
| Flow | planned, admitted, implemented, reviewed, integrated, verified, held; age and wait time at each state |
| Quality | first-pass review rate, findings by severity, reopened defects, user-story pass rate, escaped defects |
| Concurrency | active writers, reviewers, verification jobs, peak FD/CPU/RAM, budget rejections, collision incidents |
| Build graph | affected targets, action count, graph time, cache hit/miss, bytes uploaded/downloaded, critical path |
| Verification | queue time, execution time, shard balance, retries, flaky outcomes, resource-lease utilization |
| Product | task success, clicks/steps, latency, error recovery, denied/empty/stale states, accessibility and console errors |
| Release | CI-to-image time, scan/sign/attestation result, authorization latency, deploy/readback result, rollback time |

Optimize completed reviewed user stories per unit of constrained resource. Never
improve a metric by hiding failures, shrinking required coverage, inflating
timeouts, weakening assertions, relabeling integration tests as unit tests, or
publishing before authorization.

## Failure containment and rollback

The smallest affected scope stops first:

- leaf failure returns to the leaf owner;
- review failure invalidates only that leaf and its dependent receipts;
- shared-face failure stops consolidation;
- exact-candidate failure blocks the train but does not erase approved leaf
  commits;
- image failure blocks signing/promotion;
- deployment or readback failure rolls back to the last known-good immutable
  digests and routing decision.

Every train preserves exact commits, receipts, build reports, test artifacts,
image digests, deployment inputs, and the previous known-good state. Rollback is
rehearsed before expanding production exposure. After rollback, verify
readiness, authentication, authorization, the affected user story, data
integrity, and observability before declaring service restored.

## Stop conditions

Continue safe implementation, review, integration, and verification while
independent work remains. Stop or hold a lane when:

- authority, ownership, or exact input is ambiguous;
- the next action is destructive or would overwrite another lane;
- a required qualified Korea decision is unavailable;
- production credentials or protected-environment approval are missing;
- the module cannot meet its backend/frontend/user-story contract;
- evidence is not bound to the exact candidate; or
- rollback cannot preserve data and service integrity.

Completion requires every roadmap/README/TODO obligation in scope to be mapped
to an implemented module, an accepted explicit descope, or a typed `HOLD`, with
documentation updated to match verified reality.
