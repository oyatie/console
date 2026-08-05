# S-CI2 path-class CI design (console-8x4)

> **DESIGN DRAFT ONLY — NOT IMPLEMENTATION AUTHORITY.**
> This pack designs fail-closed path-class CI so docs-only PRs can finish in
> ≤5 minutes wall clock without forever-Pending required contexts.
> It does **not** change `.github/workflows/ci.yml`, Security, branch
> protection, or `scripts/check-ci-preflight.mjs`. Implementation is a later
> PR after independent review of this design.

| Field | Value |
| --- | --- |
| Bead / slice | S-CI2 · `console-8x4` |
| Worktree | `.worktrees/ci2-path-filters` |
| Branch | `ci/path-filter-design` |
| Base | `origin/main` |
| Authority | Program design under `docs/program/`; subordinate to README → PRODUCT / ROADMAP / DELIVERY |
| Implementation | **HOLD** until this design is reviewed and a separate CI-owned PR lands |

---

## 1. Problem and non-goals

### Problem

Every pull request currently pays the full CI matrix. The expensive leaves
(`postgres-domain-reachability`, `backend`, `company-conformance`,
`generated-face-authority`, `domain-unit`, `dev-up-smoke`) dominate wall clock.
Docs-only and process-doc changes still start Rust toolchains, disposable
PostgreSQL harnesses, and macOS generated-face closure.

### Why not `on.pull_request.paths` / `paths-ignore`

Chesterton's Fence, already encoded in three places:

1. **Workflow comment** (`.github/workflows/ci.yml`): a `paths` filter leaves
   required contexts **Pending forever** when candidate-controlled trust,
   security, action, or tool configuration changes outside an allowlist.
2. **`scripts/check-ci-preflight.mjs`**: fails closed if `push` or
   `pull_request` trigger objects define `paths` or `paths-ignore`.
3. **`scripts/check-foundation-gates.mjs`**: rejects the same pattern on
   `ci.yml` with a dedicated mutation test.

Historical ADR language that once mitigated monorepo CI tax with path filters
is explicitly reconciled: required CI contexts run **without** workflow path
filters so every protected context is created on every PR (see ADR-0012 current
reconciliation). This design must not resurrect trigger-level filters.

### Non-goals (this pack)

- No workflow behavior change.
- No branch-protection mutation.
- No weakening of `Required / CI` success-only aggregation semantics.
- No replacement for independent review, authority trains, or Security.
- No claim that ≤5 min is already measured on `main`; the budget is an
  **acceptance target** for the later implementation PR.

---

## 2. Required-context inventory (as of this worktree)

### Branch-protection relevant contexts

| Context | Workflow | Role |
| --- | --- | --- |
| `authenticate-console-authority` | `console-authority-bootstrap.yml` | Protected-target C/T train; not candidate-controlled |
| `Required / CI` | `ci.yml` job `required-ci` | Same-workflow aggregate over ten CI leaves |
| `Required / Security` | `security.yml` job `required-security` | Same-workflow aggregate over five Security leaves |

`docs/CI-GATES.md` records the shadow/migration posture: leaves may still be
individually required until protection migrates to the three stable contexts.
Path-class work must keep **every currently required name** reporting
success or failure on every PR (never silent Pending).

### `Required / CI` needs (exact, locked)

From `ci.yml` / `requiredCiAggregator` in `check-ci-preflight.mjs`:

1. `preflight`
2. `domain-unit`
3. `postgres-domain-reachability`
4. `company-conformance`
5. `generated-face-authority`
6. `backend`
7. `dev-up-smoke`
8. `repo-gates`
9. `api-contract`
10. `kubernetes-manifests`

Aggregator contract (locked, mutation-tested):

- `if: ${{ always() }}`
- Each `needs.<job>.result` must equal **`success`**
- **Skipped, cancelled, and failed all fail the aggregate**
- Aggregate does not check out candidate content
- Preflight forbids `job-level if` on protected leaves and on `preflight` itself

### Approximate leaf cost class (design input, not live SLOs)

| Job id | Timeout | Cost class | Primary work |
| --- | --- | --- | --- |
| `preflight` | 10m | Medium (always serial head) | Node, Buck cheap faces, foundation, cargo metadata, many unit gates |
| `domain-unit` | 20m | Heavy | Cargo lib/unit matrix |
| `postgres-domain-reachability` | 80m | Very heavy | Serialized disposable PG / RLS inventory (~183 tests; ROADMAP partition work) |
| `company-conformance` | 35m | Heavy | PG company-conformance suite |
| `generated-face-authority` | 35m | Heavy (macOS) | Full generated-face closure |
| `backend` | 90m | Very heavy | fmt, clippy, gates, PG-backed backend proofs |
| `dev-up-smoke` | 30m | Heavy | compose + migrate + `/readyz` |
| `repo-gates` | 20m | Medium | ADRs, doc-manifest/links/citations, domain maturity gates |
| `api-contract` | 30m | Medium | platform contract / import contracts |
| `kubernetes-manifests` | 15m | Medium | kustomize render, NetworkPolicy, production-hardening |
| `required-ci` | 5m | Cheap | Status comparison only |

Wall-clock today is roughly `preflight + max(parallel leaves) + required-ci`.
Docs-only must shrink **both** preflight and the parallel max, not only one leaf.

---

## 3. Path classes

Classification is over **changed paths** for the PR merge base…head (or push
before…after). Classes are closed and ordered; detection is fail-closed.

### 3.1 Class definitions

| Class id | Meaning | Detection (normative intent) |
| --- | --- | --- |
| `docs-only` | Only first-party documentation / prose surfaces | Every changed path matches the docs allowlist (below) and **none** match any other class |
| `scripts-tools` | Repo automation, Node scripts, Buck tooling, package scripts | `scripts/**`, `tools/**`, root `package.json` / `package-lock.json` (when not already forced to full by other rules), `BUCK` at roots that only gate tools — exact table owned by classifier |
| `backend` | Rust product / API / migrations / OpenAPI | `backend/**`, `third-party/rust/**` when it affects build graph, `backend/openapi/**` |
| `deploy` | Runtime / cluster / IaC / ops topology | `deploy/**`, `ops/**` (compose and topology proofs), deploy-related scripts when classifier maps them here |
| `mixed` | Two or more of {scripts-tools, backend, deploy} or docs+any of those | Union of matches spans multiple non-docs classes, or docs plus any non-docs class |
| `unknown` | Any path outside the closed map, or empty/unreadable diff | Fail closed → **full matrix** |

Additional **always-full** path prefixes (any hit ⇒ treat as `unknown` / full matrix even if other files are docs):

- `.github/**` (workflows, actions, composite free-runner-disk)
- `security/**` (audit exception policy)
- Root lock/toolchain authority that rewrites CI environment:
  `backend/rust-toolchain.toml`, `backend/Cargo.lock` (also backend),
  `backend/deny.toml`, renovate / release-please config that changes release gates
- Classifier script and its tests themselves (prevents self-skip during migration)
- Any symlink, submodule, or non-regular blob in the diff → full matrix

### 3.2 Docs-only allowlist (initial proposal)

Paths that **may** participate in `docs-only` when they are the **only**
changed set:

- `docs/**` (all classes of Markdown/JSON under docs)
- Root prose: `README.md`, `CHANGELOG.md`, `SPEC.md`, `DESIGN.md`, `HANDOFF.md`,
  `AGENTS.md`, `Claude.md`, `Agents.md` (adapter files)
- `deploy/**/*.md` and other deploy prose **only if** the classifier treats pure
  Markdown under deploy as docs; **safer default:** any `deploy/**` change is
  `deploy` class (recommended: **deploy Markdown is deploy**, not docs-only)

**Recommended stricter rule (default for implementation):** docs-only =
`docs/**` + root Markdown/agent adapters listed above. Touches under
`deploy/`, `ops/`, `scripts/`, `backend/`, `.github/` are never docs-only.

### 3.3 Class lattice (fail closed)

```
unknown ──────────────► full matrix
mixed ────────────────► full matrix
backend ──────────────► backend matrix (+ shared)
deploy ───────────────► deploy matrix (+ shared)
scripts-tools ────────► scripts/tools matrix (+ shared)
docs-only ────────────► docs matrix (thin)
```

There is **no** “best-effort skip.” Ambiguity upgrades cost, never reduces it.

---

## 4. Recommended mechanism

### 4.1 Decision: always-run workflow + always-run jobs + step-level path gates

**Do not** use trigger-level `paths` / `paths-ignore`.

**Do not** use `job-level if` that leaves protected leaves in `skipped` while
`Required / CI` still requires `result == success` for every leaf. That
combination is red by design today and is mutation-tested against
“skipped accepted” weakenings.

**Recommended:**

1. Workflow still runs on every `push`/`pull_request`/`workflow_dispatch`.
2. Every job id in `exactCiJobIds` still **starts** on every PR.
3. Early in the graph, compute `path_class` (and optional `path_class_reason`)
   from `git diff --name-only` (PR: `base.sha...head`; push: `before...after`).
4. Each heavy job begins with a **path-class gate step** that:
   - reads the class (job output / artifact / recomputation from the same
     locked script),
   - if the job is not applicable: prints an explicit
     `path-class skip proof: <job> not required for class=<class>` line and
     **exits 0**, setting `outputs.run_heavy=false`,
   - if applicable: sets `run_heavy=true`.
5. All expensive steps use `if: steps.<gate>.outputs.run_heavy == 'true'`
   (plus existing `!cancelled()` where already present).
6. Job result remains `success` when the skip proof runs cleanly; remains
   `failure` when heavy work fails.
7. `required-ci` continues to require **strict success** of all ten leaves —
   no acceptance of `skipped`.

This is the “always-run workflow with no-op proof steps that still create the
aggregator inputs” option from the slice brief. It preserves:

- Required context creation (Pending never from silence)
- Current aggregator contract shape
- Preflight ban on job-level `if` for protected jobs (can remain)

### 4.2 Alternatives considered

| Option | Why rejected / deferred |
| --- | --- |
| Trigger `paths` / `paths-ignore` | Forever-Pending; triple-banned |
| Job-level `if` + rewrite aggregator to accept `skipped` for some classes | Possible later, but expands blast radius: branch protection leaf names, preflight locks, and “skipped accepted” mutation tests all change together. Higher false-green risk |
| Separate always-green shim jobs with new ids | Duplicates contexts; confuses protection migration; two writers of “backend green” |
| Skip entire `preflight` for docs-only | Removes foundation / preflight self-check / authority proofs that docs PRs still need in thinned form |

### 4.3 Classification placement

**Preferred:** first steps of `preflight` (already `fetch-depth: 0`) run a locked
Node classifier, export:

- `PATH_CLASS`
- `PATH_CLASS_DIGEST` (hash of sorted changed paths + class rules version)
- Job outputs consumed by siblings via `needs.preflight.outputs.*`

Siblings must not trust candidate-edited free-form strings without either:

- recomputing classification from the same commit graph with the same script, or
- consuming preflight outputs only after preflight success (already `needs: preflight`).

**Push `before == 0{40}`** (new branch): full matrix.
**Missing base SHA / git error / rename edge cases the script does not understand:** full matrix.
**workflow_dispatch:** full matrix (operator intent unknown) unless an explicit
and reviewed input forces a class (default still full).

### 4.4 Detection algorithm (normative sketch)

```text
paths = git diff --name-only --diff-filter=ACMR <base>...<head>
if command fails OR paths unreadable OR contains unsupported entry:
  return unknown

classes = empty set
for p in paths:
  if p matches always-full prefixes: return unknown
  if p matches backend map: add backend
  else if p matches deploy map: add deploy
  else if p matches scripts-tools map: add scripts-tools
  else if p matches docs-only allowlist: add docs-only
  else: return unknown

if classes == {docs-only}: return docs-only
if |classes| == 1: return that class
if |classes| > 1: return mixed
if classes empty: return unknown   # empty diff is suspicious in PR CI
```

Classifier lives in something like `scripts/ci/path-class.mjs` with unit tests
that plant:

- pure `docs/program/foo.md` → `docs-only`
- `docs/x.md` + `backend/app/src/lib.rs` → `mixed`
- `.github/workflows/ci.yml` alone → `unknown` (full)
- delete-only of docs → still `docs-only` if only docs deleted (or full if
  delete handling is deferred — **implementation must pick one and lock it**)
- hostile path with spaces / unicode / `../` → full

---

## 5. Per-class job policy

### 5.1 Legend

| Policy | Meaning |
| --- | --- |
| **RUN** | Full current proof body |
| **THIN** | Always-start job; path-class skip proof for heavy body; retain class-relevant steps |
| **SKIP-PROOF** | Always-start job; only path-class skip proof (exit 0); no heavy body |
| **FULL** | Forced full matrix (same as RUN for every leaf) |

### 5.2 Matrix summary

See companion TSV: [`ci-path-filter-job-matrix.tsv`](ci-path-filter-job-matrix.tsv).

| Job | docs-only | scripts-tools | backend | deploy | mixed / unknown |
| --- | --- | --- | --- | --- | --- |
| `preflight` | **THIN** | **THIN**/RUN | RUN | RUN | FULL |
| `domain-unit` | SKIP-PROOF | SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `postgres-domain-reachability` | SKIP-PROOF | SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `company-conformance` | SKIP-PROOF | SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `generated-face-authority` | SKIP-PROOF | RUN if faces/tools graph touched else SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `backend` | SKIP-PROOF | SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `dev-up-smoke` | SKIP-PROOF | RUN if `scripts/dev-up.mjs` / compose touched else SKIP-PROOF | RUN | RUN (compose/topology) | FULL |
| `repo-gates` | **THIN** (docs + ADR + citations + manifest) | **THIN**/RUN | RUN | RUN | FULL |
| `api-contract` | SKIP-PROOF | SKIP-PROOF | RUN | SKIP-PROOF | FULL |
| `kubernetes-manifests` | SKIP-PROOF | SKIP-PROOF if no deploy/scripts render tools | SKIP-PROOF | RUN | FULL |
| `required-ci` | RUN (unchanged) | RUN | RUN | RUN | RUN |

### 5.3 Docs-only: what may skip vs must keep

#### May skip (heavy body → SKIP-PROOF)

- `postgres-domain-reachability` (serialized disposable PG inventory)
- `company-conformance`
- `domain-unit` cargo unit matrix
- `backend` fmt / clippy / gate binaries / PG-backed backend proofs
- `generated-face-authority` full face closure
- `dev-up-smoke` compose bootstrap
- `api-contract` platform/OpenAPI-adjacent contracts
- `kubernetes-manifests` render / NetworkPolicy / production-hardening

#### Must keep (still execute real checks)

Inside **thinned `preflight`** (docs-only):

- Checkout with `fetch-depth: 0`
- Path classification itself (and emit outputs)
- Node setup + `npm ci` (needed for doc and foundation scripts)
- Foundation gate contract (`npm run check:foundation-gates`) — still asserts
  CI trigger shape and gate inventory
- Reasoning-lens changed-record admission when ledger/docs under its scope change
- CI preflight contract tests + live `check:ci-preflight` (protects against
  accidental reintroduction of trigger path filters in the same PR if
  `.github` is touched — but `.github` is never docs-only; still keep the
  contract green on docs PRs as a cheap invariant)
- Package-lock check if `package-lock` unchanged is fast; keep
- Doc-adjacent console exact-M admission steps that are PR-only and already
  conditional on event — **retain** for PR docs trains that carry authority tips
- **Skip in docs-only thin preflight:** Rust toolchain install, `cargo metadata`,
  Buck DotSlash/preflight.sh cheap faces, Buck postgres harness regressions,
  executed-tests ratchet that requires cargo metadata — unless classifier
  version bumps force full

Inside **thinned `repo-gates`** (docs-only):

- ADR governance tests + gate (docs/decisions)
- Documentation link tests + gate
- Documentation manifest gate
- Doc citations gate
- Optionally foundation gate again if not fully covered in preflight
- **Skip:** G004–G008 lifecycle gates, workflow-runtime gates, people-hr /
  payroll release gates, undeclared-imports, request-body contract — these are
  backend/product maturity, not docs

#### Security (`Required / Security`)

Security is a **separate workflow** and separate required context. Docs can
still embed secrets. Recommendation for the later implementation PR:

| Security job | docs-only | Notes |
| --- | --- | --- |
| `filesystem` (Trivy vuln+secret) | **RUN** (or THIN only if measured &lt; budget and secret scan retained) | Secrets in Markdown are real |
| `iac` | SKIP-PROOF if no deploy/IaC paths | |
| `rust-advisories` | SKIP-PROOF if no Cargo/rust paths | |
| `rust-supply-chain` | SKIP-PROOF if no Cargo/rust paths | |
| `node-advisories` | SKIP-PROOF if no package-lock / package.json | |
| `required-security` | RUN | Still strict success of five leaves via skip-proof pattern |

Security preflight/hardening locks (`check-workflow-hardening`) must gain the
same “no trigger path filters; skip-proof ≠ skipped job” discipline.

#### `authenticate-console-authority`

Always runs for `main` PRs; independent of path class. Docs-only authority
trains still need signed C/T. **No path skip.**

---

## 6. ≤5 minute wall-clock budget

### 6.1 Target

For a PR whose path class is `docs-only`, time from first required check
queued → all of `{Required / CI, Required / Security, authenticate-console-authority}`
reporting success ≤ **5 minutes** under normal GitHub-hosted capacity.

### 6.2 Budget allocation (design)

| Segment | Budget | Notes |
| --- | --- | --- |
| Queue + runner alloc | ≤60s | Not fully controllable; measure and document |
| Thinned preflight | ≤150s | npm ci + doc/foundation/preflight contracts |
| Parallel thin leaves | ≤90s | repo-gates thin + many SKIP-PROOF checkouts |
| Security filesystem | ≤120s | Dominant security cost; measure Trivy on docs tree |
| Authority bootstrap | ≤60s | Already graph-check heavy; must fit |
| Aggregators | ≤30s | Status-only |
| **Slack** | ~30s | |

If measured SKIP-PROOF jobs still dominate via N× checkout+boot, implementation
may:

1. Collapse skip-proofs into fewer jobs **only after** protection no longer
   requires individual leaf names, **or**
2. Use a single reusable composite action that exits 0 in &lt;15s after
   classifying, minimizing setup actions.

Until branch protection drops leaf names, prefer fast skip-proof over job
deletion.

### 6.3 Explicit non-claims

- This design pack does not measure current docs-only wall clock.
- macOS `generated-face-authority` skip-proof still consumes a macOS minute
  budget if the job starts; if macOS queue is slow, later PR may move
  skip-proof-only faces to ubuntu with an explicit identity rename plan
  (protection-sensitive).

---

## 7. Interaction with cargo / PostgreSQL heavy path (#577 and kin)

ROADMAP delivery substrate work partitions the exact **183-test** disposable
PostgreSQL reachability inventory across isolated databases while retaining a
strict compatibility aggregate (issue track references include the long-running
serialized PG job; cargo/PG membership proofs remain open work). Path-class CI
must **not** pretend that work is done.

Rules:

1. Any path class that includes **`backend`** (or `mixed` / `unknown`) runs
   **full** `postgres-domain-reachability`, `company-conformance`, and
   `backend` cargo/clippy/PG bodies — including whatever #577 cargo-PG
   membership / reachability proofs land as.
2. Path-class skips never delete PG targets from the locked inventory; they
   only omit **execution** when the class does not touch backend.
3. When PG sharding / cargo membership PRs land, they update:
   - the full-matrix job bodies, and
   - preflight ordered contracts / executed-tests ratchet,
   - **not** the docs-only skip allowlist (unless a new job id appears — then
     extend the TSV and aggregator locks in the same PR).
4. False-green hazard: a backend change misclassified as docs-only would skip
   PG. Mitigations: closed path map, always-full prefixes, mutation tests that
   plant `backend/**` files, and classifier digest logged on every skip proof.

---

## 8. Preflight contract migration plan

Implementation is a later PR. This section is the **contract change map** so
reviewers can size the CI-owned lane.

### 8.1 Must remain forever (unless a later design supersedes with equal safety)

| Assertion | Rationale |
| --- | --- |
| `push` / `pull_request` must not define `paths` or `paths-ignore` | Required contexts always created |
| Foundation gate mirror of the same ban | Defense in depth |
| `required-ci` requires `success` (not merely `!= failure`) for every leaf | Rejects skipped/cancelled false greens |
| Aggregate does not checkout candidate code | Trust boundary |
| Protected jobs must `needs: preflight` | Ordering |
| Protected jobs must not use `continue-on-error: true` at job level | |

### 8.2 Must change for path-class

| Current assertion | Migration |
| --- | --- |
| Many preflight commands are **unconditional** (`requiredPreflightCommands`) | Split into `requiredAlwaysPreflightCommands` vs `requiredWhenPathClassIn(...)` maps; docs-only omits cargo/buck/postgres harness commands |
| Protected jobs must not define job-level `if` | **Keep** under recommended design; step-level path gates instead |
| `requiredJobRunContracts` exact ordered step multisets | Extend each heavy job with a locked **first** path-class gate step; heavy steps may gain locked `if:` tied to gate output |
| `requiredJobMetadataSha256` envelopes | Will churn when outputs / env added — update digests in same PR with mutation tests |
| Exact job-id set | Prefer **no new job ids** in v1; if a `path-class` job is added, it must be non-required or folded into preflight |
| Domain-unit / backend “unconditional cargo” locks | Become “unconditional when `run_heavy`” with explicit skip-proof step locked |

### 8.3 New locks to add

1. Classifier script golden tests (class → path fixtures).
2. Workflow mutation: introducing trigger `paths` still fails.
3. Workflow mutation: deleting path-class gate step fails preflight.
4. Workflow mutation: skip-proof step `continue-on-error: true` fails.
5. Workflow mutation: heavy step without `if:` when class is docs-only is OK
   only if it is in the always-run thin set; heavy cargo without gate fails.
6. Aggregator mutation: accepting `skipped` still fails.
7. End-to-end fixture (optional hermetic): synthetic workflow fragment evaluates
   class docs-only ⇒ gate outputs false.

### 8.4 Rollout sequence (later implementation PR series)

1. Land classifier + unit tests (no workflow change) — optional prep PR.
2. Land workflow step gates + preflight lock updates + Security skip-proofs in
   **one CI-owned PR** (single writer for `.github/workflows/ci.yml` and
   preflight locks).
3. Measure three PRs: docs-only, backend-only, docs+backend mixed.
4. Only after green measurement and review, claim S-CI2 acceptance.

---

## 9. Blast radius, rollback, stop conditions, pre-mortem

### 9.1 Blast radius

| Surface | Impact |
| --- | --- |
| Merge admission | Wrong skip ⇒ false green merge of broken backend |
| Required contexts | Wrong job-level skip ⇒ red aggregate or Pending if mis-migrated |
| Runner cost | Correct docs-only skip ⇒ large minute savings |
| Security | Skipping secret scan on docs ⇒ secret merge risk |
| Authority bootstrap | Out of scope for path class; must remain |
| Developers | Faster docs PR feedback; need clear logs explaining skips |

### 9.2 Rollback

1. Revert the implementation PR (workflow + preflight locks + classifier).
2. Confirm trigger filters remain absent.
3. Confirm `Required / CI` again demands success of all full leaves.
4. No data migration; pure CI config rollback.

### 9.3 Stop conditions

Stop or HOLD implementation if:

- Any required context is observed **Pending** on a PR that should have run.
- A planted backend-breaking change classified as docs-only goes green.
- Docs-only wall clock cannot get under 5 minutes without deleting required
  contexts or weakening Security secret scan without a reviewed exception.
- Preflight lock surface becomes too large to mutation-test in one PR (split
  prep PRs; do not ship half-locked gates).
- Branch protection still lists leaf names and skip-proof jobs exceed budget
  due to runner queue — escalate to protection migration **before** deleting
  leaves.

### 9.4 Pre-mortem (what will go wrong)

1. **Mis-map:** `docs/program/foo.md` plus an unnoticed `backend/openapi` edit
   from a bad rebase → must be `mixed`. Mitigation: classify merge commit /
   synthetic merge paths actually checked out by CI.
2. **Empty-diff PR** (title-only): class `unknown` → full matrix, not skip.
3. **Generated docs from code** living under `docs/` that claim API truth:
   docs-only will not run OpenAPI gates; that is accepted — code is source of
   truth; doc drift is caught by citation/link gates, not cargo.
4. **SKIP-PROOF without log line:** reviewers cannot see why PG did not run.
   Require a single structured log line and optional job summary.
5. **Premature job-level if:** someone “simplifies” to `if: class != docs` and
   breaks aggregate. Preflight must keep forbidding job-level if on protected
   jobs until a dedicated redesign of the aggregator lands.
6. **Security thinning too far:** secret in Markdown merged. Default keep Trivy
   filesystem/secret on docs-only.

---

## 10. Verification plan for the later implementation PR

Local (no full monorepo CI required for classifier-only prep):

```bash
node --test scripts/ci/path-class.test.mjs   # when added
node --test scripts/check-ci-preflight.test.mjs
npm run check:ci-preflight
npm run check:foundation-gates
```

Hosted:

1. Docs-only PR: only thin proofs run; skip-proof lines present; wall clock
   measured; all three required contexts success.
2. Backend PR: full cargo/PG matrix runs (including #577-era proofs when
   present).
3. Mixed PR: full matrix.
4. Adversarial PR: change `.github/workflows/ci.yml` alone → full matrix; never
   docs-only.
5. Adversarial PR: add trigger `paths` → preflight red.

---

## 11. Explicit pack boundary

| This pack delivers | This pack does not deliver |
| --- | --- |
| `docs/program/ci-path-filter-design.md` | Any `.github/workflows/*` edit |
| `docs/program/ci-path-filter-job-matrix.tsv` | Preflight assertion code changes |
| Design for fail-closed path-class CI | Branch protection API changes |
| Preflight migration plan | Claiming ≤5 min already achieved |

**Implementation is a later PR after review of this design.**

---

## 12. Recommended mechanism (executive)

Use an **always-on workflow and always-on job ids**, classify
`git diff base...head` into closed path classes with **unknown → full matrix**,
and replace heavy work with **locked no-op skip-proof steps** that still exit
**success** so `Required / CI` / `Required / Security` keep strict
`result == success` aggregation without ever accepting `skipped` or
reintroducing trigger-level `paths` filters.
