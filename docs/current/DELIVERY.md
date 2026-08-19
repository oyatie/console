# Console delivery authority

Status: active delivery authority. Product and roadmap decisions remain in [`PRODUCT.md`](PRODUCT.md) and [`ROADMAP.md`](ROADMAP.md).

## Admission and ownership

Start from an exact immutable base in a clean bounded worktree. Record the outcome and non-goals, owner, allowed and forbidden paths, source-of-truth writer, shared-resource leases, pre-mortem, detection, rollback and stop conditions, verification baseline, reviewers, evidence, candidate SHA, result, and remaining HOLDs. Serialize migrations, lockfiles, generated contracts, CI, and authority records.

Never import ignored or untracked artifacts, local runtime state, workbooks, secrets, or custody material merely because they exist on a developer machine. Establish custody from the exact candidate Git tree and regular-blob identities; path membership alone is insufficient.

## Candidate, review, and merge

Evidence and reviews bind to an exact candidate SHA. High-risk authorization, migration, contract, approval, HR/payroll, release, production, and compliance-sensitive work requires independent adversarial review appropriate to the risk. CI is evidence, not a substitute for review, legal authority, release authority, or production authority.

Merge only the reviewed candidate through the repository's protected integration path. After merge, read back the hosted commit and required checks. A local commit, branch, pull request, or green local run is unpublished evidence and is not proof that work is merged or released.

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
