> **NON-AUTHORITY IMPLEMENTATION RECORD.** This plan implements the approved
> task `console-l23` design. It does not modify product, delivery, release,
> merge, package, or production authority. Current authority remains
> `README.md` and `docs/current/{PRODUCT,ROADMAP,DELIVERY}.md`.

# Release Proof Consistency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the protected Release Please producer tolerate only a bounded stale-old-head GitHub API projection after its custody push, while preserving exact metadata and new-head proof binding.

**Architecture:** Add a synchronous, dependency-injected polling seam to the protected convergence script. A shared validator rejects stable initial PR drift before any transport mutation and validates the complete tuple again before considering the head SHA on every post-push read. The poller accepts only the exact new custody tip, retries only the exact old lease tip, and fails immediately on all other state. The producer emits native proof outputs only after exact-new acceptance, and its job has a finite 10-minute deadline.

**Tech Stack:** Node.js 24 ESM, `node:test`, GitHub CLI REST reads, GitHub Actions, repository documentation-custody generator.

## Global Constraints

- `/root` is the sole writer; `/root/security_review` and `/root/test_contract_review` are read-only independent reviewers.
- Exact starting base is `53925c3981eb8f041a03398f73d747626edc0a9c`; current implementation branch is `codex/release-proof-consistency`.
- The poller permits at most 20 reads and 500 milliseconds between retryable reads, for at most 9.5 seconds of sleeping.
- Validate stable PR metadata before examining `head.sha` on every read.
- Retry only the exact pre-push lease tip; accept only the exact new custody tip; reject any missing, malformed, or third SHA immediately.
- API, JSON, stable-field, repository, creator, base, and bounds failures are terminal and are never retried.
- Emit proof coordinates only after exact-new acceptance.
- Change `.github/workflows/release-please.yml` only to add a 10-minute timeout
  to the protected producer job; do not change triggers, permissions, secrets,
  proof identity, branch protection, signing, packages, images, or production.
- Do not overwrite the existing `RELEASE_PLEASE_TOKEN` secret.
- Do not rerun protected run `31940004916` as a source-fix test; a new main push must exercise the merged protected source.
- Stop on workflow ID/path drift, unexpected main/base/head/metadata movement, duplicate required contexts, wrong check provider, or any manual package/image/production action.

## File Structure

- Modify `scripts/console/converge-release-please-doc-custody.mjs`: define the bounded polling state machine and call it after the custody push.
- Modify `scripts/console/converge-release-please-doc-custody.test.mjs`: exercise every accepted, retryable, terminal, and integration-order state; refresh only the convergence-script closure digest.
- Modify `.github/workflows/release-please.yml`: bound the protected producer
  job to 10 minutes so a hung API read cannot hold the release concurrency lane
  for GitHub's default limit.
- Create `docs/superpowers/plans/2026-08-16-release-proof-consistency.md`: retain this non-authority implementation record.
- Modify `docs/documentation-manifest.seed.json` and `docs/documentation-index.json`: generated custody records for this plan only.

---

### Task 1: Add the fail-closed post-push polling state machine

**Files:**
- Modify: `scripts/console/converge-release-please-doc-custody.test.mjs:1-120`
- Modify: `scripts/console/converge-release-please-doc-custody.test.mjs:120-300`
- Modify: `scripts/console/converge-release-please-doc-custody.mjs:50-88`
- Modify: `scripts/console/converge-release-please-doc-custody.mjs:213-350`

**Interfaces:**
- Consumes: exact Release Please action output, authenticated pre-push PR REST object, protected repository name, triggering parent SHA, old lease SHA, new custody SHA, injected `readPullRequest()` and `sleep(milliseconds)` functions.
- Produces: `pollReleasePleasePostPushHead(options) -> exact accepted PR object`, or a synchronous exception without further reads/sleeps.

- [ ] **Step 1: Extend the exact PR fixture with immutable identity and base fields**

Add constants and complete the existing `livePr` fixture:

```js
const BOT_ID = 41898282;
const REPOSITORY_ID = 1269693002;
const N = 'b'.repeat(40);

const livePr = Object.freeze({
  number: 760,
  state: 'open',
  draft: false,
  title: actionPr.title,
  body: actionPr.body,
  user: { login: 'github-actions[bot]', id: BOT_ID },
  head: {
    ref: HEAD_REF,
    sha: T,
    repo: { full_name: REPOSITORY, id: REPOSITORY_ID },
  },
  base: {
    ref: 'main',
    sha: C,
    repo: { full_name: REPOSITORY, id: REPOSITORY_ID },
  },
});
```

Import `pollReleasePleasePostPushHead` from the convergence module and add helpers that create a fresh nested PR object without sharing mutable fixture state:

```js
const postPushPr = (headSha, overrides = {}) => ({
  ...structuredClone(livePr),
  head: { ...structuredClone(livePr.head), sha: headSha },
  ...overrides,
});

const pollFixture = (overrides = {}) => ({
  actionPr,
  initialPr: postPushPr(T),
  repository: REPOSITORY,
  expectedParentSha: C,
  oldTip: T,
  newTip: N,
  maxReads: 3,
  delayMs: 0,
  readPullRequest: () => postPushPr(N),
  sleep: () => {},
  ...overrides,
});
```

- [ ] **Step 2: Write deterministic RED tests for accepted and bounded-retry states**

Add tests that count reads and sleeps explicitly:

```js
test('post-push proof polling accepts only the exact new tip with bounded old-tip retries', () => {
  for (const [sequence, expectedReads, expectedSleeps] of [
    [[N], 1, 0],
    [[T, N], 2, 1],
    [[T, T, N], 3, 2],
  ]) {
    let reads = 0;
    let sleeps = 0;
    const accepted = pollReleasePleasePostPushHead(pollFixture({
      readPullRequest: () => postPushPr(sequence[reads++]),
      sleep: (milliseconds) => {
        assert.equal(milliseconds, 0);
        sleeps += 1;
      },
    }));
    assert.equal(accepted.head.sha, N);
    assert.equal(reads, expectedReads);
    assert.equal(sleeps, expectedSleeps);
  }
});

test('post-push proof polling times out on a persistently stale old tip', () => {
  let reads = 0;
  let sleeps = 0;
  assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
    readPullRequest: () => { reads += 1; return postPushPr(T); },
    sleep: () => { sleeps += 1; },
  })), /old lease tip.*3 reads|timed out/);
  assert.equal(reads, 3);
  assert.equal(sleeps, 2);
});
```

- [ ] **Step 3: Write deterministic RED tests for hostile SHA and transport failures**

Cover missing, malformed, and third SHAs, API failure, null response, and sleep failure. Each terminal case must assert there is no extra read or sleep:

```js
test('post-push proof polling fails immediately on any non-old, non-new head', () => {
  for (const sha of [undefined, '', 'ABC', 'd'.repeat(40)]) {
    let reads = 0;
    let sleeps = 0;
    assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
      readPullRequest: () => {
        reads += 1;
        const candidate = postPushPr(T);
        candidate.head.sha = sha;
        return candidate;
      },
      sleep: () => { sleeps += 1; },
    })), /head SHA|unexpected post-push PR head/);
    assert.equal(reads, 1);
    assert.equal(sleeps, 0);
  }
});

test('post-push proof polling never retries API, response, or sleep failures', () => {
  const apiFailure = new Error('GitHub API unavailable');
  assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
    readPullRequest: () => { throw apiFailure; },
  })), (error) => error === apiFailure);
  for (const response of [null, [], 'not an object']) {
    assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
      readPullRequest: () => response,
    })), /metadata must be an object/);
  }
  const sleepFailure = new Error('sleep interrupted');
  assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
    readPullRequest: () => postPushPr(T),
    sleep: () => { throw sleepFailure; },
  })), (error) => error === sleepFailure);
});
```

- [ ] **Step 4: Write table-driven RED tests for every frozen field under old and new heads**

Use a mutation table with one nested clone per case:

```js
const postPushMetadataMutations = [
  ['number', (pr) => { pr.number = 761; }],
  ['state', (pr) => { pr.state = 'closed'; }],
  ['draft state', (pr) => { pr.draft = true; }],
  ['title', (pr) => { pr.title = `${pr.title} forged`; }],
  ['body', (pr) => { pr.body = `${pr.body}\nforged`; }],
  ['creator login', (pr) => { pr.user.login = 'attacker'; }],
  ['creator id', (pr) => { pr.user.id += 1; }],
  ['head ref', (pr) => { pr.head.ref = `${pr.head.ref}-attacker`; }],
  ['head repository name', (pr) => { pr.head.repo.full_name = 'attacker/console'; }],
  ['head repository id', (pr) => { pr.head.repo.id += 1; }],
  ['base ref', (pr) => { pr.base.ref = 'attacker'; }],
  ['base SHA', (pr) => { pr.base.sha = 'e'.repeat(40); }],
  ['base repository name', (pr) => { pr.base.repo.full_name = 'attacker/console'; }],
  ['base repository id', (pr) => { pr.base.repo.id += 1; }],
];

test('post-push proof polling validates all stable metadata before old/new SHA handling', () => {
  for (const headSha of [T, N]) {
    for (const [label, mutate] of postPushMetadataMutations) {
      let sleeps = 0;
      const candidate = postPushPr(headSha);
      mutate(candidate);
      assert.throws(
        () => pollReleasePleasePostPushHead(pollFixture({
          readPullRequest: () => candidate,
          sleep: () => { sleeps += 1; },
        })),
        new RegExp(`post-push live PR ${label} changed`, 'i'),
        `${label} at ${headSha}`,
      );
      assert.equal(sleeps, 0, label);
    }
  }
});
```

- [ ] **Step 5: Write RED tests for invalid initial snapshots, SHA pairs, dependencies, and bounds**

Add pre-read assertions for: initial head not equal to `oldTip`; missing/non-positive bot or repository IDs; unequal head/base repository IDs; equal old/new SHA; uppercase/short SHA; non-function read/sleep dependencies; `maxReads` outside integer `1..20`; and `delayMs` outside integer `0..500`. Count `readPullRequest` calls and require zero for every invalid input.

```js
const invalidInitialMutations = [
  (pr) => { pr.head.sha = N; },
  (pr) => { delete pr.user.id; },
  (pr) => { pr.user.id = 0; },
  (pr) => { delete pr.head.repo.id; },
  (pr) => { pr.head.repo.id = 0; },
  (pr) => { delete pr.base.repo.id; },
  (pr) => { pr.base.repo.id += 1; },
];

test('post-push proof polling validates all inputs before its first read', () => {
  for (const mutate of invalidInitialMutations) {
    let reads = 0;
    const initialPr = postPushPr(T);
    mutate(initialPr);
    assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
      initialPr,
      readPullRequest: () => { reads += 1; return postPushPr(N); },
    })));
    assert.equal(reads, 0);
  }

  for (const overrides of [
    { oldTip: N },
    { newTip: T },
    { oldTip: 'A'.repeat(40) },
    { newTip: 'short' },
    { actionPr: { ...actionPr, number: 0 } },
    { actionPr: { ...actionPr, baseBranchName: 'attacker' } },
    { repository: 'not-an-owner-name' },
    { readPullRequest: null },
    { sleep: null },
    { maxReads: 0 },
    { maxReads: 21 },
    { maxReads: 1.5 },
    { delayMs: -1 },
    { delayMs: 501 },
    { delayMs: 0.5 },
  ]) {
    let reads = 0;
    assert.throws(() => pollReleasePleasePostPushHead(pollFixture({
      ...overrides,
      ...(Object.hasOwn(overrides, 'readPullRequest') ? {} : {
        readPullRequest: () => { reads += 1; return postPushPr(N); },
      }),
    })));
    assert.equal(reads, 0);
  }
});
```

- [ ] **Step 6: Run the focused tests and verify the intended RED state**

Run:

```bash
node --test scripts/console/converge-release-please-doc-custody.test.mjs
```

Expected: FAIL at module import because `pollReleasePleasePostPushHead` is not exported. Existing unrelated cases must not fail before that import error.

- [ ] **Step 7: Implement the minimum synchronous helper**

Add small private validators plus the exported helper. Preserve the ordering shown here: stable fields first, SHA second.

```js
function positiveSafeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    fail(`${label} must be a positive safe integer`);
  }
  return value;
}

function assertPostPushStablePr(pr, expected) {
  if (pr === null || Array.isArray(pr) || typeof pr !== 'object') {
    fail('post-push live PR metadata must be an object');
  }
  const checks = [
    [pr.number === expected.number, 'number'],
    [pr.state === 'open', 'state'],
    [pr.draft === false, 'draft state'],
    [pr.title === expected.title, 'title'],
    [pr.body === expected.body, 'body'],
    [pr?.user?.login === expected.creatorLogin, 'creator login'],
    [pr?.user?.id === expected.creatorId, 'creator id'],
    [pr?.head?.ref === expected.headRef, 'head ref'],
    [pr?.head?.repo?.full_name === expected.repository, 'head repository name'],
    [pr?.head?.repo?.id === expected.repositoryId, 'head repository id'],
    [pr?.base?.ref === 'main', 'base ref'],
    [pr?.base?.sha === expected.parentSha, 'base SHA'],
    [pr?.base?.repo?.full_name === expected.repository, 'base repository name'],
    [pr?.base?.repo?.id === expected.repositoryId, 'base repository id'],
  ];
  for (const [valid, label] of checks) {
    if (!valid) fail(`post-push live PR ${label} changed`);
  }
}

export function pollReleasePleasePostPushHead({
  actionPr,
  initialPr,
  repository,
  expectedParentSha,
  oldTip,
  newTip,
  readPullRequest,
  sleep,
  maxReads = 20,
  delayMs = 500,
} = {}) {
  if (!actionPr || typeof actionPr !== 'object') fail('release action PR output is unavailable');
  const repo = nonEmptyString(repository, 'protected repository');
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repo)) {
    fail('protected repository must be an exact owner/name pair');
  }
  const parentSha = exactSha(expectedParentSha, 'triggering main SHA');
  const oldSha = exactSha(oldTip, 'pre-push lease tip SHA');
  const newSha = exactSha(newTip, 'post-push custody tip SHA');
  if (oldSha === newSha) fail('pre-push and post-push tips must differ');
  if (typeof readPullRequest !== 'function') fail('post-push PR reader must be a function');
  if (typeof sleep !== 'function') fail('post-push sleeper must be a function');
  if (!Number.isSafeInteger(maxReads) || maxReads < 1 || maxReads > 20) {
    fail('post-push maximum reads must be an integer from 1 through 20');
  }
  if (!Number.isSafeInteger(delayMs) || delayMs < 0 || delayMs > 500) {
    fail('post-push delay must be an integer from 0 through 500 milliseconds');
  }

  if (initialPr === null || Array.isArray(initialPr) || typeof initialPr !== 'object') {
    fail('pre-push live PR metadata must be an object');
  }
  const prNumber = positiveSafeInteger(actionPr.number, 'release action PR number');
  const title = nonEmptyString(actionPr.title, 'release action PR title');
  const body = nonEmptyString(actionPr.body, 'release action PR body');
  const headRef = nonEmptyString(actionPr.headBranchName, 'release action PR head ref');
  if (!RELEASE_PLEASE_HEAD_REF.test(headRef)) {
    fail('release action PR head ref must match the release-please branch pattern');
  }
  if (actionPr.baseBranchName !== 'main') fail('release action PR base must be main');
  const expected = Object.freeze({
    number: prNumber,
    title,
    body,
    creatorLogin: RELEASE_PLEASE_BOT_NAME,
    creatorId: positiveSafeInteger(initialPr?.user?.id, 'pre-push PR creator id'),
    headRef,
    repository: repo,
    repositoryId: positiveSafeInteger(initialPr?.head?.repo?.id, 'pre-push repository id'),
    parentSha,
  });
  if (initialPr?.base?.repo?.id !== expected.repositoryId) {
    fail('pre-push head and base repository ids must match');
  }
  assertPostPushStablePr(initialPr, expected);
  if (exactSha(initialPr?.head?.sha, 'pre-push live PR head SHA') !== oldSha) {
    fail('pre-push live PR head must equal the lease tip');
  }

  for (let read = 1; read <= maxReads; read += 1) {
    const current = readPullRequest();
    assertPostPushStablePr(current, expected);
    const currentSha = exactSha(current?.head?.sha, 'post-push live PR head SHA');
    if (currentSha === newSha) return current;
    if (currentSha !== oldSha) {
      fail(`unexpected post-push PR head ${currentSha}; expected old lease tip or new custody tip`);
    }
    if (read === maxReads) {
      fail(`post-push PR remained at old lease tip after ${maxReads} reads`);
    }
    sleep(delayMs);
  }
  fail('post-push PR polling exhausted unexpectedly');
}
```

Before GREEN, adjust implementation details only where a RED test demonstrates a mismatch; do not broaden retryable state.

- [ ] **Step 8: Refresh the protected source digest for the isolated helper**

Run:

```bash
shasum -a 256 scripts/console/converge-release-please-doc-custody.mjs
```

Replace only the digest beside `./converge-release-please-doc-custody.mjs` in `protectedReleaseIssuerClosure`. Task 2 will refresh that same one digest once more after wiring the helper into `main()`.

- [ ] **Step 9: Run the focused tests and verify GREEN**

Run:

```bash
node --test scripts/console/converge-release-please-doc-custody.test.mjs
```

Expected: all focused tests PASS, including the pre-existing token, workflow, commit-identity, and closure tests.

- [ ] **Step 10: Commit the isolated polling state machine**

```bash
git add scripts/console/converge-release-please-doc-custody.mjs \
  scripts/console/converge-release-please-doc-custody.test.mjs
git commit -m "fix: bound release proof head polling"
```

Expected: one commit containing only the helper, its deterministic unit tests, and the exact updated convergence-source digest.

---

### Task 2: Integrate exact-new acceptance before proof output

**Files:**
- Modify: `scripts/console/converge-release-please-doc-custody.mjs:480-505`
- Modify: `scripts/console/converge-release-please-doc-custody.test.mjs:450-475`

**Interfaces:**
- Consumes: `pollReleasePleasePostPushHead(options)` from Task 1 and the existing `ghJson`, `actionPr`, `pr`, `repository`, `expectedParentSha`, `tip`, and `newTip` values.
- Produces: exact proof-output ordering: successful force-with-lease, exact-new polling acceptance, then `emitProofOutputs({ prNumber, headSha, parentSha })`.

- [ ] **Step 1: Add a RED source-order integration test**

Extend the test file with a protected-source assertion:

```js
test('protected producer accepts the exact post-push head before emitting proof outputs', () => {
  const source = readFileSync(
    new URL('./converge-release-please-doc-custody.mjs', import.meta.url),
    'utf8',
  );
  const push = source.indexOf("run('git', invocation.args, { cwd: work, env: invocation.env });");
  const poll = source.indexOf('pollReleasePleasePostPushHead({', push);
  const output = source.indexOf('emitProofOutputs({ prNumber: binding.prNumber', poll);
  assert.ok(push >= 0 && poll > push && output > poll);
  assert.doesNotMatch(source.slice(push, output), /const finalPr = ghJson/);
});
```

- [ ] **Step 2: Run the focused test and verify the intended RED state**

Run:

```bash
node --test --test-name-pattern='protected producer accepts' \
  scripts/console/converge-release-please-doc-custody.test.mjs
```

Expected: FAIL because `main()` still uses the one-shot `finalPr` read.

- [ ] **Step 3: Replace the one-shot read with the production poller call**

Replace the `finalPr` block with:

```js
    pollReleasePleasePostPushHead({
      actionPr,
      initialPr: pr,
      repository,
      expectedParentSha,
      oldTip: tip,
      newTip,
      readPullRequest: () => ghJson([
        'api', `repos/${repository}/pulls/${actionPr.number}`,
      ]),
      sleep: (milliseconds) => {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
      },
    });
    emitProofOutputs({ prNumber: binding.prNumber, headSha: newTip, parentSha: parent });
```

Keep the existing `emitProofOutputs` call and returned coordinates unchanged. Do not catch poller errors or add a fallback output. Extract the initial stable-field validation and invoke it before worktree/transport mutation, then add only the producer timeout described above to workflow YAML.

- [ ] **Step 4: Refresh only the protected convergence-source digest**

Compute the final source digest:

```bash
shasum -a 256 scripts/console/converge-release-please-doc-custody.mjs
```

Replace only the digest beside `./converge-release-please-doc-custody.mjs` in `protectedReleaseIssuerClosure`. Leave all other closure paths and digests byte-for-byte unchanged.

- [ ] **Step 5: Run the complete focused release-authority suite**

Run:

```bash
node --test \
  scripts/console/verify-console-pr-authority-bootstrap.test.mjs \
  scripts/console/release-please-bot-candidate.test.mjs \
  scripts/console/release-authority-proof.test.mjs \
  scripts/console/converge-release-please-doc-custody.test.mjs
```

Expected: all discovered tests PASS, zero skipped or failed. Record the discovered/executed count from the runner.

- [ ] **Step 6: Run syntax, diff, and CI-contract gates**

Run:

```bash
node --check scripts/console/converge-release-please-doc-custody.mjs
node --check scripts/console/converge-release-please-doc-custody.test.mjs
node --test scripts/check-ci-preflight.test.mjs
node scripts/check-ci-preflight.mjs
git diff --check
```

Expected: both syntax checks exit 0, all CI-preflight tests PASS, preflight prints success, and the diff check is empty.

- [ ] **Step 7: Commit the protected integration and closure pin**

```bash
git add scripts/console/converge-release-please-doc-custody.mjs \
  scripts/console/converge-release-please-doc-custody.test.mjs
git commit -m "test: pin release proof polling integration"
```

Expected: one commit containing the producer call-site replacement, source-order guard, and exact updated source digest.

---

### Task 3: Verify the complete candidate and obtain independent review

**Files:**
- Verify: all files changed from `53925c3981eb8f041a03398f73d747626edc0a9c..HEAD`
- Verify: `docs/documentation-manifest.seed.json`
- Verify: `docs/documentation-index.json`

**Interfaces:**
- Consumes: completed Task 1 and Task 2 commits plus the already committed design and implementation plan.
- Produces: a clean, locally verified branch with independent security and test-contract READY verdicts bound to an immutable head/tree/patch identity.

- [ ] **Step 1: Run repository verification with the pinned DotSlash binary available**

Install the repository-pinned, checksum-verified DotSlash 0.5.7 into a fresh task-specific temporary directory, prepend it to `PATH`, and run:

```bash
task_dotslash_root="$(mktemp -d)"
CONSOLE_DOTSLASH_BIN_DIR="${task_dotslash_root}/bin" tools/buck/install_dotslash.sh
PATH="${task_dotslash_root}/bin:${PATH}" npm run verify
```

Expected: exit 0. Record every discovered/executed count. If any gate fails, use `superpowers:systematic-debugging`; do not skip, quarantine, or weaken it.

- [ ] **Step 2: Record immutable review coordinates**

Run:

```bash
git rev-parse HEAD
git write-tree
git diff 53925c3981eb8f041a03398f73d747626edc0a9c...HEAD | shasum -a 256
git status --short
```

Expected: one immutable head SHA, one tree SHA, one patch SHA-256, and an empty status.

- [ ] **Step 3: Obtain independent security and test-contract review**

Send both read-only reviewers the exact head/tree/patch coordinates, approved design, implementation plan, focused/full commands with counts, and these review questions:

```text
Security: Can any response other than the exact old lease tip or exact new
custody tip reach retry or acceptance? Is every stable identity/base/repository
field validated before SHA handling? Can API/sleep failures or malformed data
produce proof output? Is the retry work strictly bounded?

Test contract: Do RED/GREEN evidence, field-mutation coverage, source ordering,
closure pinning, CI-preflight contracts, and full verification prove the stated
design without skipped or weakened tests?
```

Expected: both reviewers report READY against the same immutable identity with no Critical or Important finding. Repair any finding with a fresh RED test, rerun all gates, compute a new identity, and request a fresh review.

---

### Task 4: Merge the repair and validate the new protected producer

**Files:**
- GitHub branch: `codex/release-proof-consistency`
- GitHub workflow: `.github/workflows/release-please.yml`, workflow ID `296023729`
- GitHub release PR: `jason931225/console#760`

**Interfaces:**
- Consumes: the locally verified, independently reviewed exact implementation head.
- Produces: merged repair on `main`, a fresh successful protected Release Please run, a new exact custody tip for PR #760, and three exact-head required checks from GitHub Actions app `15368`.

- [ ] **Step 1: Confirm main and branch-protection preconditions before publishing**

Run read-only checks:

```bash
git fetch --prune origin
git rev-parse origin/main
gh api repos/jason931225/console/branches/main/protection
gh api repos/jason931225/console/actions/workflows/296023729
gh secret list --repo jason931225/console
```

Expected: main either remains `53925c3981eb8f041a03398f73d747626edc0a9c` or the branch is first rebased and fully reverified; strict protection still requires exactly `authenticate-console-authority`, `Required / CI`, and `Required / Security`, each pinned to app `15368`; workflow ID/path is unchanged and active; `RELEASE_PLEASE_TOKEN` remains present. Never print or replace its value.

If `origin/main` moved, rebase only this sole-writer branch and stop on any conflict:

```bash
git rebase origin/main
```

After a clean rebase, repeat every Task 2 and Task 3 verification command, recompute head/tree/patch coordinates, and obtain fresh READY verdicts bound to them before publishing.

- [ ] **Step 2: Push the reviewed branch and create the repair PR**

```bash
repair_head_sha="$(git rev-parse HEAD)"
repair_tree_sha="$(git write-tree)"
repair_patch_sha="$(git diff origin/main...HEAD | shasum -a 256 | awk '{print $1}')"
git push -u origin codex/release-proof-consistency
gh pr create --repo jason931225/console \
  --base main \
  --head codex/release-proof-consistency \
  --title "fix(ci): tolerate stale release proof readback" \
  --body "Task: console-l23

Exact candidate: head ${repair_head_sha}; tree ${repair_tree_sha}; patch SHA-256 ${repair_patch_sha}; base $(git rev-parse origin/main).

Root cause: the protected producer treated one stale post-push Pulls API projection as terminal after a successful force-with-lease. This patch rejects stable initial drift before transport, revalidates every stable field on each read, retries only the exact old lease SHA for at most 20 reads/9.5 seconds of deliberate sleep, accepts only the exact new custody SHA, and emits no proof on any other state. A 10-minute protected-producer deadline bounds a hung API call.

Review lenses: Red Team, Operability/Day-2, Blast-radius, Zero-trust, YAGNI. Writer: /root. Independent reviewers: /root/security_review and /root/test_contract_review.

Verification: focused release-authority suite, CI-preflight contracts, syntax/diff gates, documentation custody/links, and npm run verify with repository-pinned DotSlash all pass with counts recorded in task evidence.

Pre-mortem: stale API state, third-SHA movement, stable metadata drift, token exposure, wrong proof provider, or base movement. Detection: exact read/sleep assertions, closure digest, exact-head hosted checks, native proof identity, and final PR/base/tree/path readback. Rollback: ordinary revert of this helper and digest. Stop: any unexpected SHA/metadata/provider/workflow/base drift.

Blast radius is limited to Release Please pre/post-push validation and a finite producer timeout. No workflow trigger/permission, secret, branch-protection, package, image, or production change. Image/package/production work remains HOLD."
repair_pr_number="$(gh pr view codex/release-proof-consistency \
  --repo jason931225/console --json number --jq .number)"
```

Expected: `repair_pr_number` is a positive integer and the PR body contains no credential or token value.

- [ ] **Step 3: Require exact repair-head hosted checks and merge**

Read the PR head SHA immediately before every decision:

```bash
repair_head_sha="$(gh pr view "${repair_pr_number}" --repo jason931225/console \
  --json headRefOid --jq .headRefOid)"
gh pr view "${repair_pr_number}" --repo jason931225/console \
  --json state,isDraft,baseRefOid,headRefOid,mergeStateStatus,statusCheckRollup
gh pr checks "${repair_pr_number}" --repo jason931225/console \
  --json name,state,bucket,link,workflow
gh api "repos/jason931225/console/commits/${repair_head_sha}/check-runs"
gh api "repos/jason931225/console/commits/${repair_head_sha}/status"
```

Expected: open/non-draft, current main base, exactly one success for each required name, all three from app `15368`, no same-name legacy status, no unresolved conversation, no unexpected path, and mergeable/up-to-date state. Then merge only that head:

```bash
gh pr merge "${repair_pr_number}" --repo jason931225/console \
  --squash --match-head-commit "${repair_head_sha}"
```

Stop instead of merging on any stale, duplicate, missing, neutral, skipped, wrong-provider, or non-success required context.

- [ ] **Step 4: Identify the fresh protected run from the repair merge**

Read back the repair PR merge commit and current main, then query workflow ID `296023729`. Accept exactly one new run whose event is `push`, branch is `main`, and `head_sha` equals the merged repair SHA:

```bash
gh pr view "${repair_pr_number}" --repo jason931225/console \
  --json state,mergedAt,mergeCommit
gh api repos/jason931225/console/git/ref/heads/main
gh run list --repo jason931225/console --workflow 296023729 \
  --event push --branch main --limit 20 --json databaseId,headSha,status,conclusion,url
```

Expected: the repair PR is merged, main equals its merge commit, and one fresh protected run is queued or running at that exact SHA. Do not rerun `31940004916`.

- [ ] **Step 5: Monitor the fresh producer and bind its native proof**

Poll in intervals shorter than 60 seconds while keeping the user updated. When terminal, require:

- run workflow ID `296023729`, exact path `.github/workflows/release-please.yml`, event `push`, branch `main`, and head SHA equal to the repair merge;
- current attempt conclusion `success`;
- producer job success;
- exactly one native proof job whose ASCII name binds PR `760` and the emitted new custody SHA;
- PR #760 remains open, non-draft, bot-created, same-repository, and based on the exact repair merge;
- the new head is a one-parent child of that base with exact Release Please/GitHub identity and only the two release paths plus all-or-nothing custody pair.

Any failed/pending newer attempt overrides an older success. Stop on a skipped/missing/duplicate proof job, unexpected SHA or metadata, workflow drift, or timeout.

---

### Task 5: Merge PR #760 and close the recovery task

**Files:**
- GitHub PR: `jason931225/console#760`
- Bead: `console-l23`

**Interfaces:**
- Consumes: the fresh exact protected proof and all three current-head required contexts.
- Produces: squash-merged PR #760, exact post-merge readbacks, and closed recovery evidence.

- [ ] **Step 1: Revalidate the exact release head immediately before merge**

Read PR, commit, comparison, diff, checks, check-runs, and legacy statuses. Require:

```text
state=open; draft=false; base SHA=current main; one parent=head^=base;
ahead=1; behind=0; exact bot/GitHub identity; exact allowed mode-100644 paths;
authenticate-console-authority=success;
Required / CI=success;
Required / Security=success;
each required name occurs exactly once and comes from app_id 15368;
legacy status contains no same-name context; auto-merge remains null.
```

Recompute the synthetic merge and require parents `[base, head]` and tree equal to the release head tree. Stop on any mismatch or main movement.

- [ ] **Step 2: Squash-merge only the validated release head**

```bash
validated_release_head="$(gh pr view 760 --repo jason931225/console \
  --json headRefOid --jq .headRefOid)"
gh pr merge 760 --repo jason931225/console \
  --squash --match-head-commit "${validated_release_head}"
```

Do not use admin bypass, delete checks, change protection, or enable auto-merge.

- [ ] **Step 3: Perform exact post-merge readbacks**

```bash
release_merge_sha="$(gh pr view 760 --repo jason931225/console \
  --json mergeCommit --jq .mergeCommit.oid)"
gh pr view 760 --repo jason931225/console \
  --json state,mergedAt,mergedBy,mergeCommit,baseRefOid,headRefOid
gh api repos/jason931225/console/git/ref/heads/main
gh api "repos/jason931225/console/commits/${release_merge_sha}"
gh pr checks 760 --repo jason931225/console \
  --json name,state,bucket,link,workflow
```

Expected: PR #760 is `MERGED`, main equals the reported squash merge commit, the validated head is unchanged, and pre-merge check evidence remains attached to that exact head.

- [ ] **Step 4: Close the Bead and record final workspace state**

```bash
bd close console-l23 --reason="Release proof consistency repair and PR #760 merged with exact required-check readbacks"
bd show console-l23
git status --short
```

Expected: bead is closed and the implementation worktree is clean. Report the repair PR, both merge SHAs, fresh Release Please run/proof URLs, exact test invocations/counts, required checks, secret metadata presence, rollback, and remaining image/package/production HOLD. Do not manually publish, delete, promote, or modify any package/image/production state.
