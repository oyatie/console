> **NON-AUTHORITY DESIGN RECORD.** This document records an implementation
> design approved for task `console-l23`. It does not modify product, delivery,
> release, merge, or production authority. Current authority remains
> `README.md` and `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md`.

# Release proof post-push consistency design

Date: 2026-08-16
Owner/writer: `/root`
Independent reviewers: `/root/security_review`, `/root/test_contract_review`
Starting base: `53925c3981eb8f041a03398f73d747626edc0a9c`
Observed action tip: `6658408890a4c9a6447229ece65bac918fdf6ad0`
Observed custody tip: `b74970dca3f42038d7b4a7067cfe4769c382e112`
Protected workflow/run: `296023729` / `31940004916`, attempt 2

## Problem and evidence

Release Please run `31940004916` attempt 2 used the newly provisioned
transport token and successfully force-with-leased the deterministic custody
tip `b74970d...`. That push scheduled CI, Security, and the protected authority
workflow. The custody tip is a one-parent child of the exact main SHA, uses the
required bot/GitHub identity, and changes exactly the two release paths plus
the two documentation-custody paths.

The producer then made one immediate `pulls/760` API read in the same second as
the push. That read did not yet report the expected head, so the producer
failed with `live PR moved or metadata changed after custody push` and emitted
no native proof. The live PR subsequently reported the exact expected head and
all stable metadata matched. CI and Security passed; authority correctly failed
because the proof job was skipped.

The observed defect is an availability race at an eventually consistent API
boundary. It is not permission to weaken the exact-head proof.

## Goals and non-goals

Goals:

- tolerate only the observed stale-old-head window after a successful push;
- reject stable initial PR drift before creating or pushing a transport commit;
- preserve exact PR identity, metadata, repository, base, and head binding;
- bound work and wall time deterministically;
- emit proof outputs only after the API reports the exact new custody SHA;
- provide deterministic tests for success, timeout, and hostile movement.

Non-goals:

- no generic retry wrapper;
- no retry of API/JSON errors or metadata drift;
- no branch-protection, workflow-trigger, secret, signing, package, image, or
  production changes;
- no acceptance of historical proof or a third head SHA;
- no protected workflow change beyond a finite producer-job timeout; triggers,
  permissions, secrets, and proof-job identity remain byte-for-byte unchanged.

## Considered approaches

1. **Bounded old-tip-only synchronous poll (selected).** Keep the existing
   synchronous producer and inject read/sleep functions into a small helper.
   Retry only while every stable field is valid and `head.sha` is exactly the
   pre-push lease tip. This is the smallest fail-closed repair.
2. **Convert the producer to asynchronous I/O.** Use `setTimeout` and async GitHub
   reads. This avoids a synchronous sleep but expands the call graph and error
   surface without improving the security contract.
3. **Remove the post-push PR readback.** Rely only on Git transport or the later
   consumer. This weakens defense in depth and is rejected.

## State machine

Add an exported helper in
`scripts/console/converge-release-please-doc-custody.mjs`. Its production call
uses protected `GITHUB_TOKEN` reads, a maximum of 20 reads, and 500 milliseconds
between retryable reads (9.5 seconds maximum deliberate sleep). The protected
producer job has a 10-minute deadline so a network call that accepts a
connection but never returns cannot occupy the serialized release lane until
GitHub's default six-hour limit.

The same stable-field validator runs on the authenticated initial Pulls API
snapshot before any worktree creation, token acquisition, commit creation, or
force-with-lease. The post-push loop then repeats that complete validation on
every read.

Before examining the head SHA on every response, the helper requires:

- the expected PR number, open and non-draft state;
- exact Release Please title and body;
- creator login and numeric ID frozen from the authenticated pre-push PR;
- exact head ref and same-repository head name and numeric ID;
- base ref `main`, same-repository base name and numeric ID;
- base SHA exactly equal to the triggering `GITHUB_SHA`.

Timestamps, labels, mergeability fields, and other derived GitHub state are not
frozen.

After stable-field validation:

- exact expected new custody SHA: accept immediately;
- exact old lease SHA: sleep and retry if budget remains;
- missing, malformed, or any third SHA: fail immediately;
- old SHA on the final read: fail with an explicit bounded-timeout error.

An API error, invalid JSON, or stable-field mismatch propagates immediately and
does not consume a retry as if it were eventual consistency. Old and new SHAs
must be distinct canonical lowercase 40-character Git SHAs; retry bounds and
injected functions are validated before the first read.

The existing proof-output call remains after the helper returns. Therefore the
only output coordinates are the original PR number, exact accepted new SHA,
and exact triggering parent SHA.

## Testing

Tests in `scripts/console/converge-release-please-doc-custody.test.mjs` use
injected read and sleep functions, with no network or real delay:

- immediate new tip: one read, zero sleeps;
- old then new, and old until new on the final allowed read;
- persistent old tip: exact maximum reads and one fewer sleeps, then timeout;
- missing, malformed, or third SHA: immediate failure and zero sleeps;
- each frozen field drifted under both old and new heads: immediate failure;
- base SHA drift despite base ref remaining `main`: immediate failure;
- API failure and null/non-object response: immediate failure;
- invalid/equal SHA inputs and invalid bounds: fail before reading;
- source-order guard: proof output remains after exact-head acceptance.

Refresh only the protected producer-source digest in the existing executable
closure test. Run the focused test RED before production code, then GREEN, the
complete authority/release suite, CI-preflight tests/gate, documentation gates,
`git diff --check`, and the repository-supported `npm run verify` with pinned
DotSlash available.

## Rollout, detection, and rollback

Merge the repair as an ordinary PR only after exact-head CI, Security, and
ordinary protected authority checks pass. A new main push must create a new
Release Please run; rerunning the old run would retain old protected source.

Success detection is an exact native proof job whose name binds PR #760 and the
new custody SHA, followed by the three required app-`15368` contexts on that
same SHA. Only then may #760 be squash-merged after a fresh main/base/tree/path
readback.

Rollback is a normal revert of the poll helper and closure digest. Do not delete
or bypass the required context, reuse an old proof, or relax branch protection.

Stop on workflow ID/path drift, secret replacement, base movement not covered
by a fresh main run, unexpected PR metadata or SHA, wrong check provider,
duplicate same-name status/check, unreviewed source change, package/image
publication request, or any production action.

Remaining HOLD: image-release trigger redesign is separate work.
