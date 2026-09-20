# Console delivery authority

Status: active delivery authority. Product and roadmap decisions remain in [`PRODUCT.md`](PRODUCT.md) and [`ROADMAP.md`](ROADMAP.md).

## Admission and ownership

Start from an exact immutable base in a clean bounded worktree. A lane is a named command that is currently red on that base; occupancy of a path, a merge-tree-clean slice, or a worktree is not a lane. Start implementers only through `tools/lanes/fanout.py`: missing, green, or non-executable probe is stop, and the same probe must be green before the lane is success. Record the outcome and non-goals, owner, allowed and forbidden paths, source-of-truth writer, shared-resource leases, pre-mortem, detection, rollback and stop conditions, verification baseline, reviewers, evidence, candidate SHA, result, and remaining HOLDs. Serialize migrations, lockfiles, generated contracts, CI, and authority records. OpenAPI and generated clients remain one writer.

Never import ignored or untracked artifacts, local runtime state, workbooks, secrets, or custody material merely because they exist on a developer machine. Establish custody from the exact candidate Git tree and regular-blob identities; path membership alone is insufficient.

## Candidate, review, and merge

Evidence and reviews bind to an exact candidate SHA. High-risk authorization, migration, contract, approval, HR/payroll, release, production, and compliance-sensitive work requires independent adversarial review appropriate to the risk. CI is evidence, not a substitute for review, legal authority, release authority, or production authority.

`dev` is the canonical integration and source-release branch and the intended repository default. Feature, authority, and release-candidate work enters through its protected pull-request path. Ordered environment mirrors after `dev` are `staging`, `canary`, and `production`: their names, tips, or rulesets cannot authorize product work, source release, live promotion, or exposure. Creating or fast-forwarding those names is not production exposure. Changing this branch model again requires a separately reviewed current-authority candidate and exact readback of every affected required check and protection.

After #987, `authenticate-console-authority` runs on `pull_request_target` to `dev` only (`base.ref === 'dev'`). Authority PRs that need that check target `dev`, not `main`. Hosted GitHub `default_branch` remains `main`. Merge queue remains on `main` only; enqueue into `dev` fails with no merge queue. Until a merge queue exists on `dev`, a reviewed `dev` candidate lands with squash-merge without `--admin`. Never `--admin`. Never force-push. Never delete `origin/main`. Never PATCH default_branch without `administration=write` and a separate reviewed candidate. Gitops files key Argo `targetRevision` on `dev` (#988); live Argo cluster tracking is not proven by that commit and is not this record. Lake, go-live DNS/TLS/exposure, payable/payment execution, and production credential reset remain **HOLD**.

A required status context must have an executable protected workflow producer for every branch where it is enforced. Configure and verify the producer before enabling the requirement; a context inherited from an unrelated commit or branch is not evidence. Required checks `authenticate-console-authority`, `Required / CI`, and `Required / Security` must have producers on `dev` before they are enforced there. CI, Security, Nightly, CAS seed, bootstrap `assertLivePullRequestSnapshot`, and release-please key on `dev`. Argo `targetRevision` keys on `dev`. GitHub default-branch readback and deletion of `origin/main` remain a later API step requiring `administration=write`. Until separately authorized, environment-branch promotion (live DNS, TLS, or production exposure) remains fail-closed and on HOLD.

Merge only the reviewed candidate through the protected `dev` integration path. After merge, read back the hosted commit and required checks. A local commit, branch, pull request, or green local run is unpublished evidence and is not proof that work is merged or released.

## High-risk program method

Tenancy, authorization, migration, contract, HR, payroll, frontend shipping, and Intelligence-seam work uses this sequence. Authority and design records are orchestrator-owned; they are not `fanout.py` lanes.

1. Four independent design-review rounds, consensus, and a revision-bound explicit approval (design SHA).
2. Tests written first against that design SHA, independently reviewed, and explicitly approved (test SHA). Semantic test changes return to test review.
3. Implementation only from the exact approved test commit. Material design change returns to design review.
4. After implementation: coverage, security hardening, simplification and refactor, with review-fix until approve.
5. A 16-lens audit (the task-selected reasoning lenses in [`AGENTS.md`](../../AGENTS.md)) before the merge verdict. High-risk authz, migration, HR, and payroll include Red Team, Operability / Day-2, Blast-radius / cell-based, and Zero-trust / defense-in-depth.
6. Independent COMMENT on the candidate SHA, then merge only through the protected integration path above. `fanout.py` still admits implementation lanes: a named command that is red on the clean tree.

This method does not authorize live promotion, DNS, TLS, secrets, exposure, payment execution, or cloning Intelligence.

## PR #862 one-time containment

PR #862 merged candidate `c61949c6c57d56755a49722d82f0cec2a471680f` as `ec866c1b0450829bc6b776be570e13f6a18edbd6` with tree `2b5c870eb9494def1d64f024f51c512cf694df0b` at 2026-08-24T08:46:05Z before any independent review; GitHub recorded zero submitted reviews. That merge violated the review-before-merge rule above. This record does not waive, excuse, or retroactively satisfy it.

One bounded prospective containment of that exact tree is admitted: release, promotion, and dependent authority work remains stopped until a candidate based on `ec866c1b0450829bc6b776be570e13f6a18edbd6`, changing only this receipt and the two generated documentation-manifest projections, receives independent adversarial review of the full inherited current-authority tree before merge, enters through protected `main`, and has its hosted tip and required checks read back. Current authority resumes prospectively at that new reviewed tip only. Any material content finding requires a normal reviewed correction or revert. This paragraph is not precedent or a reusable review exception and authorizes no release, environment promotion, production action, or HOLD clearance.

## Verification method

Run the smallest targeted regression first, then the applicable format, lint/type, contract, security, and domain gates. A new clone must install pinned Node tooling and put the pinned DotSlash runtime on `PATH` before the supported repository entrypoint. The installer cannot modify its parent shell (`GITHUB_PATH` exists only in GitHub Actions).

```sh
npm ci
tools/buck/install_dotslash.sh
export PATH="${CONSOLE_DOTSLASH_BIN_DIR:-${RUNNER_TEMP:-${TMPDIR:-/tmp}/console-dotslash}/bin}:$PATH"
npm run verify
```

This is the same sequence as [`README.md`](../../README.md). Buck2-backed steps fail as environment errors, not product regressions, if DotSlash is missing from `PATH`.

For documentation-authority changes, also run the doc-link tests and gate, ADR tests and gate, citation checks, foundation tests and gate, CI-preflight tests and gate, verifier tests, `npm run verify`, and `git diff --check`. Inspect the exact changed-path allowlist and ignored/untracked state before signing a candidate.

Record exact commands, revision, toolchain/environment, discovered and executed counts, failures, artifact hashes where relevant, and validation gaps. A ran-nothing result, stale SHA, superseded candidate, or omitted required surface is not green evidence.

## Issue lifecycle policy

An issue closes only when one of these facts is recorded:

1. the requested outcome is merged, and release is recorded when the issue requires release; or
2. the issue is explicitly identified as a duplicate or is explicitly superseded by another tracked outcome.

Keep the issue open when work is partial, unpublished, on a local branch or unmerged pull request, blocked by a HOLD, awaiting security review or remediation, awaiting recovery/readback evidence, or represented only by an ambiguous roadmap statement. A commit, passing local tests, an implementation claim, or a planned follow-up does not by itself satisfy closure.

## Non-authority

Historical plans, evidence, branches, chats, handoffs, and transient runtime state may explain or support delivery facts. They never replace current authority or exact-candidate proof. Document classes (`current`, `decision`, `executable-contract`, `evidence`, `historical`, `quarry`) describe custody, not permission to ship aspirations.

## First client release acceptance and qualification

The release inventory is the `release_inventory` section of [`../program/console-capability-registry.json`](../program/console-capability-registry.json). Each required leaf records its owner, dependencies, public reference or explicit user requirement, journey, actual executable boundary, and separately evidenced planned, implemented, integration accepted, production-qualified and released states. Missing source coverage, support-matrix entries, commands, fixtures, reviews or evidence are blockers, not implied passes. Existing HOLD records and historical receipts remain intact. This is one inventory governed by PRODUCT and ROADMAP, not independent shipping authority.

Before implementation: exact design/test candidates, clear owner/interfaces, reviewed fixtures/oracles, executable RED at the actual boundary, functioning prerequisites and migration/recovery design. Reuse valid SHA-bound approvals and evidence; semantic test changes return to independent review. Infrastructure failures, unrelated missing schemas and zero executed tests do not admit a lane.

Before workflow acceptance: exercise real browser → authentication → current policy → owning transaction → durable effects → response → reopening. Require independent correctness, security, failure, usability and performance evidence. Compare each journey against relevant interviews, prototypes, images and industry references; implement against the real owner/database; exercise realistic data and restricted permissions; capture screenshots; obtain independent visual/interaction critique; record adopt/adapt/reject against task outcomes. Representative users across distinct tasks/permissions test discoverability, consequence comprehension, completion, assistance and recovery. Formative studies find defects; they do not establish population-wide usability. Progress feedback must remain truthful and preserve work over persistent entities.

Required histories include cross-tenant denial and separately governed cross-Company access; salary fields, confidential links, search/previews/exports and subscription revocation; distinct Accounts for one natural person in approval checks; stale/concurrent/duplicate requests, lost responses, interrupted commits, cancellation and unknown outcomes; temporary/permanent task transfer with no eligible replacement; partial settlements, rejection, duplicate statement imports, unmatched payments, corrected instructions and ambiguous delivery; pipeline checkpoints, schema drift, backfills, sandbox isolation and starvation; sheet same-cell/independent/structural/formula/reconnect edits; model/application promotion, rollback, dependency/source-policy changes; populated migrations, mixed-version readers/writers, resumable backfills, retention/deletion/legal holds; Korean IME, keyboard use, screen-reader semantics, long text, large datasets, 320px reflow and 200% zoom.

Buck2-native qualification requires candidate-bound action and artifact evidence for application/worker binaries, Leptos SSR and hydration WASM, compiled gates, required Rust test variants and release-image inputs. Generated target presence, successful Cargo execution or a wrapper invoking Cargo cannot establish native qualification. Record actual compiler/tool inputs and resulting artifact identities, preserve test discovery/execution counts and PostgreSQL credential isolation, and compare native replacements before retiring transitional producers. Cargo metadata, formatting and separately pinned tooling bootstrap are permitted supporting tools, not alternative product artifact producers. Native execution does not itself prove full hermeticity, cross-platform cache reuse or performance; qualify those claims separately.

Positive controls and injected corruption must prove the evidence machinery detects missing requests, omitted effects, incomplete histories and stale artifacts. Verus claims name the executable boundary and trusted assumptions. Record exact invocations and discovered/executed counts; never skip, weaken or quarantine a test without the approved receipt and independent review.

Qualification starts with hypotheses of 80 active sessions, 40 editors, 2,000 attendance submissions within 60 seconds, balanced and heavily skewed Company populations, and twelve months of history. Run mixed work for 60 minutes, then bursts, saturation and recovery, including ingestion, backfills, analytical queries/exports, transforms, model jobs, formula recalculation and subscription catch-up. Measure hot-object and hot-cell contention separately and count rejected/timed-out useful work. Initial server-side targets (not measurements): context reads p95 ≤250 ms / p99 ≤1 second; saves/time submissions p95 ≤500 ms / p99 ≤2 seconds; review attempts ≤5 seconds; Company execution ≤30 seconds; Group completion ≤300 seconds; valid interactive error/timeout rate ≤0.1% within the admitted envelope. Evaluate 50,000- and 1,000,000-person datasets with explicit saturation and architecture-change triggers; headcount alone chooses no mechanism.

Reuse application/workers, CNPG, object storage and observability. Qualify two application replicas and three independently placed database instances with a required synchronous standby and bounded rollout. Resolve and measure the connection-budget defect: authenticated blue/green can demand at least 68 connections against configuration 60 before reserve. Inventory every application pool, worker, observer, migration and rollout allocation; increasing the cap without a tested memory/concurrency budget is not proof. Unify PostgreSQL versions through compatibility testing, retain Rust 1.98.1 and verified TLS, and pin tested application/SSR/WASM/dependency/deployment artifacts.

Require no loss of acknowledged state under qualified primary/failure-domain loss; ordinary-data disaster RPO ≤5 minutes and recovery readmission ≤4 hours; fenced recovery and current-authority reconciliation before resumption; and restoration of database security objects, files, datasets, model artifacts, checkpoints, keys and package metadata. Separate software rollback, data restoration and irreversible-effect reconciliation. These are qualification targets, not guarantees established by this record.

Before client exposure: complete release inventory, exact built artifact, deployment-equivalent qualification, populated migrations, recovery drills, monitoring/support and no unresolved release blockers. Exposure remains a separate decision for the complete assembled release; a subset or internal milestone cannot clear it.
