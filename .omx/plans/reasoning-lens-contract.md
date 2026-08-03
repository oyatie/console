# Consensus Plan — Task-Appropriate Reasoning Lens Contract

## Requirements Summary

Adopt the sixteen user-provided reasoning lenses as a repository-wide execution policy. Every substantive planning, investigation, implementation, review, and verification task selects the smallest useful subset before acting, re-evaluates it when evidence changes, and records concise conclusions and tradeoffs rather than private chain-of-thought.

Locked policy:

- `AGENTS.md` is canonical for the ordered names, definitions, routing rules, and evidence boundary.
- `README.md` and `CLAUDE.md` contain compact identifier-only manifests in the same exact order and point to `AGENTS.md`; they do not duplicate semantics.
- Every nontrivial governed artifact selects at least two lenses.
- High-risk authz, migration, contracts, approval, HR/payroll, release, production, and compliance-sensitive work includes Red Team, Operability / Day-2, Blast-radius / cell-based, and Zero-trust / defense-in-depth, or records a lens-specific not-applicable rationale.
- Historical unmarked ledgers and retrospectives are grandfathered. New or modified governed records use `lens_contract: v1`.
- CI proves manifest/evidence structure for durable governed records only. It must not claim universal task compliance, authentic reasoning quality, or access to private reasoning.

## RALPLAN-DR Summary

### Principles

1. Canonicalize semantics once and project identifiers broadly.
2. Apply the smallest risk-appropriate lens set rather than all lenses mechanically.
3. Make durable decision evidence observable without surveilling private reasoning.
4. Enforce forward without rewriting historical evidence.
5. Validate only objective structure; reviewers judge semantic quality.

### Decision Drivers

1. Prevent drift across agent-facing root guidance.
2. Make nontrivial and high-risk lens routing durable and reviewable.
3. Avoid false claims that formatting proves reasoning or compliance.

### Viable Options

**A. Dedicated structural validator with diff-aware forward enforcement — chosen**

- Pros: isolated ownership, precise diagnostics, focused tests, no dependencies, explicit historical grandfathering.
- Cons: adds a script/test pair and needs full Git history in its CI job.

**B. Fold validation into `check-foundation-gates.mjs`**

- Pros: reuses an existing command and job.
- Cons: couples reasoning governance to product-foundation checks, enlarges blast radius, and obscures failure ownership.

**C. Policy and human review only**

- Pros: least ceremony and no Git-diff logic.
- Cons: cannot detect root-manifest drift or missing forward evidence and fails the locked machine-enforcement outcome.

## Public Engineering Interfaces

### Canonical lens vocabulary

The validator owns this exact frozen constant shape and content:

```js
const CANONICAL_LENSES_V1 = [
  { name: "Cartesian doubt", definition: "challenge assumptions and separate evidence, inference, and uncertainty." },
  { name: "Essentialism / YAGNI", definition: "pursue the smallest sufficient outcome and avoid speculative scope." },
  { name: "Chesterton's Fence", definition: "understand why an existing constraint or mechanism exists before removing it." },
  { name: "Contrarian / outside-the-box", definition: "test non-obvious alternatives when the default framing may be wrong." },
  { name: "Socratic", definition: "expose hidden premises with focused questions; ask the user only when the answer materially blocks safe progress." },
  { name: "Pragmatism", definition: "optimize for the real-world outcome under actual constraints." },
  { name: "Red Team", definition: "model misuse, adversaries, hostile inputs, and ways the plan can fail." },
  { name: "Systems Thinking", definition: "trace dependencies, feedback loops, second-order effects, and system boundaries." },
  { name: "Operability / Day-2", definition: "account for deployment, diagnosis, maintenance, recovery, and ownership after launch." },
  { name: "Opportunity Cost", definition: "compare the chosen work against the best alternatives in time, complexity, and value." },
  { name: "Blast-radius / cell-based", definition: "contain changes and failures; prefer independently recoverable boundaries." },
  { name: "Constant-work / anti-fragility", definition: "avoid input-dependent blowups, degrade predictably, and use stress to improve the system." },
  { name: "Shared-nothing / eventual consistency", definition: "minimize coordination and make convergence, conflicts, and stale-state behavior explicit." },
  { name: "FinOps / unit-cost", definition: "reason about cost per useful outcome, including operational and scaling costs." },
  { name: "Telemetry-first", definition: "make important state, decisions, failures, and success criteria observable." },
  { name: "Zero-trust / defense-in-depth", definition: "verify every boundary, minimize privilege, and layer independent safeguards." }
];
```

Compute `lens_contract_digest` as lowercase SHA-256 hex over the UTF-8 bytes of `JSON.stringify(CANONICAL_LENSES_V1)` with no appended newline. The frozen v1 digest is `ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373`. The validator compares root guidance against this constant, not merely against each other, so coordinated drift requires an intentional versioned code change.

Each root file contains exactly one block delimited by:

```text
<!-- SHARED:REASONING-LENSES:START -->
...
<!-- SHARED:REASONING-LENSES:END -->
```

Root block serialization is generated and compared exactly, without Unicode or whitespace normalization:

- `AGENTS.md` body equals `AGENTS_PREAMBLE + "\n\n" + CANONICAL_LENSES_V1.map((lens, index) => `${index + 1}. **${lens.name}** — ${lens.definition}`).join("\n") + "\n\n" + AGENTS_EPILOG`.
- `AGENTS_PREAMBLE` is exactly: `## Task-selected reasoning lenses\n\nAll substantive reasoning, planning, implementation, review, and verification must use the smallest task-appropriate subset. Select at least two lenses before nontrivial work, re-evaluate the set when evidence or risk changes, and do not mechanically apply all lenses.`
- `AGENTS_EPILOG` is exactly: `High-risk authz, migration, contracts, approval, HR/payroll, release, production, and compliance-sensitive work must include Red Team, Operability / Day-2, Blast-radius / cell-based, and Zero-trust / defense-in-depth, or record a lens-specific not-applicable rationale in durable evidence. Report concise conclusions, evidence, decisions, and tradeoffs rather than private chain-of-thought.`
- `README.md` and `CLAUDE.md` bodies are identical and equal `MANIFEST_PREAMBLE + "\n\n" + CANONICAL_LENSES_V1.map((lens, index) => `${index + 1}. ${lens.name}`).join("\n")`.
- `MANIFEST_PREAMBLE` is exactly: `## Reasoning lens manifest\n\nCanonical definitions and routing rules live in [AGENTS.md](AGENTS.md#task-selected-reasoning-lenses). This identifier-only projection is drift-checked and does not duplicate policy.`
- Each body is enclosed immediately by the shared start/end markers, with exactly one newline after the start marker and before the end marker.

Thus `AGENTS.md` carries full semantics and `README.md`/`CLAUDE.md` carry an exact identifier projection only.

### Durable evidence block

Governed Markdown uses exactly one marker-delimited, canonical JSON block:

````text
<!-- REASONING-LENS-EVIDENCE:START -->
```json
{
  "lens_contract": "v1",
  "lens_contract_digest": "<computed-v1-sha256>",
  "task_class": "planning",
  "risk_class": "standard",
  "risk_domains": [],
  "selected_lenses": [
    "Cartesian doubt",
    "Pragmatism"
  ],
  "task_fit": {
    "Cartesian doubt": "Separated repository evidence from inference.",
    "Pragmatism": "Selected the smallest enforceable outcome."
  },
  "mandatory_lens_exceptions": {},
  "findings": [
    "Root manifest drift needs a structural gate."
  ],
  "decisions_changed_or_rejected": [],
  "lens_set_changes": []
}
```
<!-- REASONING-LENS-EVIDENCE:END -->
````

Allowed values:

- `task_class`: `planning`, `investigation`, `implementation`, `review`, `verification`, `trivial_read_only`.
- `risk_class`: `standard` or `high`; absent only for `trivial_read_only`.
- `risk_domains`: canonical-order subset of `authz`, `migration`, `contracts`, `approval`, `hr_payroll`, `release`, `production`, `compliance_sensitive`, `other`.

Schema rules:

- Reject unknown/missing keys. Extracted JSON payload bytes between the `json` fences must equal `JSON.stringify(parsed, null, 2) + "\n"` exactly; this rejects noncanonical formatting and duplicate JSON keys because reserialization differs.
- Selected lenses are unique and in canonical order. Nontrivial records select 2–16.
- `task_fit` keys equal selected lenses exactly and values are nonblank outcome-level explanations.
- High-risk records select the four mandatory lenses. An omitted mandatory lens requires a nonblank exception keyed only by that missing lens; exceptions for already selected or nonmandatory lenses fail.
- Standard records require `mandatory_lens_exceptions: {}`. For high-risk records, the mandatory-lens set must be a subset of `selected_lenses ∪ Object.keys(mandatory_lens_exceptions)`; exception keys are disjoint from selected lenses and are a subset of the four mandatory lenses.
- Nontrivial `findings` contains at least one concise string. Decision and lens-change arrays may be empty to avoid fabricated activity.
- `risk_class: standard` requires `risk_domains: []`. `risk_class: high` requires a nonempty canonical-order domain subset. `other` means a high-risk domain outside the enumerated taxonomy and is never valid with `standard`.
- `trivial_read_only` omits `risk_class` and has empty `risk_domains`, `selected_lenses`, `task_fit`, exceptions, findings, decisions, and changes.
- Strings beginning `EXAMPLE:` are allowed only under `docs/retros/templates/` and rejected in governed records, preventing unchanged template-copy evidence.
- The validator never scores prose depth, detects chain-of-thought, or claims that reasoning occurred.

## Implementation Steps

1. **Root guidance contract**
   - Extend `AGENTS.md` with the existing shared markers, exact sixteen definitions from the user instruction, risk-based selection, two-lens minimum, high-risk core including approval, re-evaluation rule, and concise-evidence/private-reasoning boundary.
   - Add compact exact-name manifests to `README.md` and `CLAUDE.md`. Preserve `CLAUDE.md`'s no-policy-duplication rule by describing the manifest as an identifier projection only.

2. **Playbook and templates**
   - Replace the playbook's review-only lens paragraph with the all-substantive-task routing lifecycle: select, classify risk, act, re-evaluate, and persist outcomes when a durable artifact exists.
   - Document the distinction between normative all-task policy and CI-provable durable evidence.
   - Add canonical v1 example blocks to pre-mortem, post-mortem, runnable-rollback, and workflow-experiment templates. Use `EXAMPLE:` values. Make the runnable rollback example high-risk with `risk_domains` including `approval`, `release`, and `production` and the four mandatory lenses; other templates may demonstrate standard routing.

3. **Dependency-free validator and tests**
   - Add `scripts/check-reasoning-lens-contract.mjs` and its Node test file.
   - Structural mode validates the three root manifests, canonical `AGENTS.md` definitions, all template examples, and every existing opt-in evidence block while grandfathering unmarked historical records.
   - `--changed-since BASE` must pass `git cat-file -e "$BASE^{commit}"` and `git merge-base --is-ancestor "$BASE" HEAD`, then runs `git diff --name-status -z --no-renames "$BASE" HEAD --`. Missing objects, noncommits, shallow-history gaps, and nonancestor bases fail closed. Added or modified direct children of `docs/program/ledger/*.md` and recursive `docs/retros/**/*.md` outside `docs/retros/templates/**` must contain a valid v1 block. Deletions are ignored; with `--no-renames`, a rename becomes deletion plus addition and the added target is enforced.
   - Failure output includes path, marker line or field, enforcement mode, base SHA, and head SHA. Invalid/unreachable bases fail closed.
   - Use temporary Git repositories for diff-aware tests so the dirty product worktree is never mutated.

4. **Package, CI, and inventory integration**
   - Add `check:reasoning-lens-contract` to `package.json`.
   - Add `docs/retros/**` and the three root guidance files to both CI path filters.
   - In `repo-gates`, set checkout `fetch-depth: 0`, run the validator tests, then select enforcement mode exactly:
     - pull request: `--changed-since ${{ github.event.pull_request.base.sha }}`;
     - non-tag branch push with nonzero `${{ github.event.before }}`: `--changed-since` that SHA;
     - tag push, zero-before branch creation, and workflow dispatch: structural mode.
   - Document the new root package command in `docs/CI-GATES.md`; retain current doc-link integration and allow the existing foundation inventory gate to verify command parity.
   - Extend `scripts/check-ci-preflight.mjs` and `scripts/check-ci-preflight.test.mjs` as the single owner of CI-shape verification. It must assert both root path-filter lists include `docs/retros/**` and the three root guidance files, `repo-gates` uses `fetch-depth: 0`, validator tests and gate steps are present, and the exact PR/normal-push/zero-before/tag/workflow-dispatch mode-selection shell is retained.

5. **Independent integration and verification**
   - Execution begins only from `/Users/jasonlee/Developer/console-post-pivot`. Fail before any source write unless `pwd -P` equals that path, `git branch --show-current` equals `codex/post-pivot-wave0`, `git rev-parse HEAD` equals `9200e875b5362ef88b9a1af20dfc43ed3f07a970`, and `git merge-base HEAD origin/main` equals the same SHA.
   - Before edits, preserve `git status --porcelain=v1`, `git diff --binary`, and `git diff --cached --binary` as `.omx/context/reasoning-lens-preexecution-*` artifacts inside the target worktree. Also enumerate `git ls-files -z` plus `git ls-files --others --exclude-standard -z`, sort paths by raw UTF-8 byte order, and write a JSON manifest containing each path's tracked/untracked class, file type, mode, byte length, and SHA-256 of its working-tree bytes. Copy every pre-existing untracked file, preserving relative paths and bytes, into `.omx/context/reasoning-lens-preexecution-untracked/` and archive that directory. Planning artifacts under `.omx/**` are excluded from the manifest itself.
   - After execution, regenerate the manifest and require byte-for-byte equality for every path outside the union of the three lane writable allowlists. Reject changed, deleted, or newly created tracked/untracked paths outside that union. Keep the untracked content archive as recovery evidence until integration completes.
   - Never reset, checkout, clean, stash, rebase, or broadly rewrite the existing Wave-0 diff.
   - Preserve the existing Wave-0 dirty diff and edit only the lane allowlists below.
   - Run targeted validator tests/gate, then doc links, ADR governance, foundation inventory, CI preflight tests/gate, package-lock consistency, and diff checks.
   - Independently review for false compliance claims, copied example acceptance, Git-event mistakes, hidden dependency additions, and unrelated diff changes.

## Acceptance Criteria and Test Plan

### Unit

- Root block: missing, duplicate, reordered, renamed, whitespace-normalized, Unicode-lookalike, coordinated three-file drift, definition drift, and duplicate marker cases fail.
- Evidence JSON: unknown/missing keys, duplicate keys, noncanonical formatting, wrong digest, unknown/duplicate/out-of-order lenses, one-lens nontrivial selection, mismatched `task_fit`, malformed arrays, and empty required findings fail.
- High risk: all four mandatory lenses pass; missing lens without keyed rationale fails; valid keyed exception passes; exception for selected/nonmandatory lens fails; approval risk is high.
- Trivial read-only: exact empty shape passes; risk class or evidence content fails.
- `EXAMPLE:` passes only in template paths and fails in governed records.

### Integration

- Temporary Git histories prove A/M enforcement, D ignore, rename-as-D+A behavior, invalid/unreachable base failure, shallow-history failure, recursive retro scanning, flat-ledger-only scanning, template exclusion, and historical unmarked grandfathering.
- `scripts/check-ci-preflight.test.mjs` proves PR, normal push, zero-before push, tag, and workflow-dispatch modes plus path filters, checkout depth, and validator steps.

### End-to-end repository verification

```bash
node --test scripts/check-reasoning-lens-contract.test.mjs
npm run check:reasoning-lens-contract
npm run check:doc-links
node --test scripts/check-doc-links.test.mjs
npm run test:adrs
npm run check:adrs
npm run check:foundation-gates
node --test scripts/check-ci-preflight.test.mjs
npm run check:ci-preflight
npm run check:package-lock
git diff --check
git status --short
```

When validating a real diff base, additionally run `npm run check:reasoning-lens-contract -- --changed-since <reachable-base-sha>`.

## Risks and Mitigations

- **Checkbox reasoning:** gate only durable structure; semantic quality remains independent review responsibility.
- **False universal-compliance claim:** docs and output explicitly say CI covers designated records only.
- **Private reasoning leakage:** fields accept conclusions/evidence/tradeoffs only; no trace scoring or CoT detection.
- **Historical churn:** unmarked history is grandfathered; any future A/M governed record opts into v1.
- **Template false green:** reserved examples fail outside template paths.
- **Git event errors:** pin full history, exact event bases, zero-before/tag fallback, and fail-closed reachability.
- **Dirty-worktree collision:** no resets/checkouts/broad formatters; inspect targeted diff before and after.

## ADR

### Decision

Adopt `AGENTS.md` as canonical lens policy, root identifier-only projections, a frozen v1 digest, and a dedicated structural/diff-aware validator for forward durable evidence.

### Drivers

- Cross-agent consistency.
- Risk-proportionate, observable decision evidence.
- Honest enforcement boundaries and preserved history.

### Alternatives considered

- Human-review-only policy.
- Integration into the existing foundation gate.

### Why chosen

The dedicated validator is the smallest design that detects drift and forward evidence omissions without coupling reasoning governance to product gates or pretending to prove private reasoning.

### Consequences

- New/modified governed records carry canonical JSON metadata.
- CI needs full Git history in the repo-gates job.
- Reviewers remain responsible for honest task/risk classification and evidence quality.

### Follow-ups

- Observe false-positive and ritualization signals for two delivery waves.
- Change semantics only through a versioned v2 contract and migration plan.
- Keep historical v1 evidence immutable except through ordinary reviewed edits.

## Available Agent Types and Execution Staffing

Relevant roster: `writer`, `executor`, `test-engineer`, `verifier`, `code-reviewer`, `architect`, `critic`, `explore`, `debugger`, `git-master`, `code-simplifier`, and optional taxonomy advisor `scholastic`.

Recommended Team + Ultragoal staffing:

- Writer lane, high reasoning. Writable allowlist: `AGENTS.md`, `README.md`, `CLAUDE.md`, `docs/program/agentic-engineering-playbook.md`, and `docs/retros/templates/*.md` only.
- Executor/test-engineer lane, high reasoning. Writable allowlist: `scripts/check-reasoning-lens-contract.mjs` and `scripts/check-reasoning-lens-contract.test.mjs` only.
- Integration executor lane, medium reasoning, starting after the validator command stabilizes. Writable allowlist: `package.json`, `.github/workflows/ci.yml`, `docs/CI-GATES.md`, `scripts/check-ci-preflight.mjs`, and `scripts/check-ci-preflight.test.mjs` only.
- Verifier and code-reviewer, high reasoning: read-only sequential verification and adversarial review.
- Ultragoal leader owns durable checkpoints and shared-file integration.

Launch hints:

```bash
cd /Users/jasonlee/Developer/console-post-pivot
$ultragoal /Users/jasonlee/Developer/console/.omx/plans/reasoning-lens-contract.md
$team 3 "Implement /Users/jasonlee/Developer/console/.omx/plans/reasoning-lens-contract.md in /Users/jasonlee/Developer/console-post-pivot with disjoint writer, validator-test, and CI-inventory lanes; preserve the existing Wave-0 diff."
```

Team verification path: each lane reports exact files/tests; leader rejects overlaps; verifier runs the full integration command set at the exact head; code reviewer checks enforcement boundaries and diff containment; Ultragoal records evidence and closes only after all gates pass.

`$ralph` remains an explicit persistent single-owner fallback, not the default. `$autoresearch-goal` and `$performance-goal` are not indicated because this is neither a research nor optimization deliverable.

## Consensus Improvement Changelog

- Restored approval work to the high-risk taxonomy.
- Split normative behavior from CI-provable durable evidence.
- Added frozen v1 digest, exact markers, strict canonical JSON, and schema closure.
- Pinned PR/push/tag/workflow-dispatch Git semantics and fail-closed bases.
- Added template-only sentinels and comprehensive temporary-Git regression coverage.
- Pinned the exact target worktree, branch, base SHA, root-block renderer grammar, and CI-shape test owner.
- Added disjoint lane writable allowlists and byte-complete tracked/untracked containment evidence for the existing dirty Wave-0 worktree.
