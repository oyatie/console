import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import yaml from 'js-yaml';
import {
  PINNED_RELEASE_REPOSITORY,
  PINNED_RELEASE_REPOSITORY_ID,
  RELEASE_PLEASE_WORKFLOW_ID,
  assertLivePullRequestSnapshot,
  classifyProtectedPrRoute,
  createProtectedGitOps,
  fetchExactPullHead,
  verifyBootstrapGraph,
} from './verify-console-pr-authority-bootstrap.mjs';

const B = 'd'.repeat(40);
const H = 'a'.repeat(40);
const M = 'b'.repeat(40);
const P = 'c'.repeat(40);
const RELEASE_BOT_ID = 41898282;
const RELEASE_HEAD_REF = 'release-please--branches--main--components--console';
const ORDINARY_HEAD_REF = 'feature/ordinary';
const RELEASE_PATHS = ['.release-please-manifest.json', 'CHANGELOG.md'];

const modified = (file) => ({
  path: file,
  status: 'M',
  oldMode: '100644',
  newMode: '100644',
  oldType: 'blob',
  newType: 'blob',
});

function ordinaryOps(overrides = {}) {
  return {
    hasCommit: (sha) => [B, H, P, M].includes(sha),
    parents: (sha) => (sha === H ? [B] : sha === M ? [B, H] : []),
    commitIdentity: () => ({
      authorName: 'Contributor',
      authorEmail: 'contributor@example.invalid',
      committerName: 'Contributor',
      committerEmail: 'contributor@example.invalid',
      subject: 'feat: ordinary change',
    }),
    diff: () => [modified('backend/app/src/lib.rs')],
    isAncestor: (ancestor, descendant) => ancestor === B && descendant === H,
    tree: (sha) => (sha === H || sha === M ? 'tree-head' : 'tree-base'),
    sameTreeDiff: (left, right) => (
      (left === H && right === M) || (left === M && right === H)
    ),
    ...overrides,
  };
}

function ordinaryCoordinates(overrides = {}) {
  return {
    baseSha: B,
    headSha: H,
    prNumber: 761,
    prAuthorId: 12345,
    prAuthorLogin: 'contributor',
    prHeadRef: ORDINARY_HEAD_REF,
    prHeadRepository: 'contributor/console',
    repository: PINNED_RELEASE_REPOSITORY,
    ...overrides,
  };
}

function releaseOps(overrides = {}) {
  return ordinaryOps({
    parents: (sha) => (sha === H ? [B] : sha === M ? [B, H] : []),
    commitIdentity: () => ({
      authorName: 'github-actions[bot]',
      authorEmail: '41898282+github-actions[bot]@users.noreply.github.com',
      committerName: 'GitHub',
      committerEmail: 'noreply@github.com',
      subject: 'chore(main): release 0.3.4',
    }),
    diff: () => RELEASE_PATHS.map(modified),
    ...overrides,
  });
}

function releaseCoordinates(overrides = {}) {
  return ordinaryCoordinates({
    prNumber: 760,
    prAuthorId: RELEASE_BOT_ID,
    prAuthorLogin: 'github-actions[bot]',
    prHeadRef: RELEASE_HEAD_REF,
    prHeadRepository: PINNED_RELEASE_REPOSITORY,
    mergeSha: M,
    releaseAuthorityProof: Object.freeze({
      workflowId: RELEASE_PLEASE_WORKFLOW_ID,
      runId: 31906390000,
      runNumber: 842,
      runAttempt: 1,
      jobId: 95065020000,
      prNumber: 760,
      headSha: H,
      parentSha: B,
    }),
    ...overrides,
  });
}

test('ordinary unsigned single-commit code PR passes without signature, policy, ledger, merge, or API reads', () => {
  const forbidden = (name) => () => { throw new Error(`ordinary route invoked ${name}`); };
  const ops = ordinaryOps({
    readFile: forbidden('readFile'),
    treeEntry: forbidden('treeEntry'),
    verifyCommit: forbidden('verifyCommit'),
    tree: forbidden('tree'),
    sameTreeDiff: forbidden('sameTreeDiff'),
  });
  assert.deepEqual(verifyBootstrapGraph(ops, ordinaryCoordinates()), {
    baseSha: B,
    headSha: H,
    admissionClass: 'ordinary-pr',
  });
});

test('ordinary multi-commit, fork, Dependabot, and unsigned authority-doc PRs have no C/T requirement', () => {
  const cases = [
    {
      label: 'multi-commit',
      ops: ordinaryOps({
        parents: (sha) => (sha === H ? [P] : sha === P ? [B] : []),
      }),
      args: ordinaryCoordinates(),
    },
    {
      label: 'fork',
      ops: ordinaryOps(),
      args: ordinaryCoordinates({ prHeadRepository: 'outside/fork', prHeadRef: 'patch-1' }),
    },
    {
      label: 'Dependabot',
      ops: ordinaryOps({
        commitIdentity: () => ({
          authorName: 'dependabot[bot]',
          authorEmail: '49699333+dependabot[bot]@users.noreply.github.com',
          committerName: 'GitHub',
          committerEmail: 'noreply@github.com',
          subject: 'chore(deps): bump a dependency',
        }),
      }),
      args: ordinaryCoordinates({
        prAuthorId: 49699333,
        prAuthorLogin: 'dependabot[bot]',
        prHeadRef: 'dependabot/npm_and_yarn/example',
        prHeadRepository: PINNED_RELEASE_REPOSITORY,
      }),
    },
    {
      label: 'unsigned authority documentation',
      ops: ordinaryOps({ diff: () => [modified('docs/program/console-program-ledger.md')] }),
      args: ordinaryCoordinates({ prHeadRef: 'docs/authority-record' }),
    },
  ];
  for (const { label, ops, args } of cases) {
    assert.equal(verifyBootstrapGraph(ops, args).admissionClass, 'ordinary-pr', label);
  }
});

test('ordinary admission binds the exact protected base ancestry and needs no synthetic merge', () => {
  assert.equal(verifyBootstrapGraph(ordinaryOps(), ordinaryCoordinates()).admissionClass, 'ordinary-pr');
  assert.throws(
    () => verifyBootstrapGraph(ordinaryOps({ isAncestor: () => false }), ordinaryCoordinates()),
    /protected base must be an ancestor of the PR head/,
  );
  assert.throws(
    () => verifyBootstrapGraph(ordinaryOps({ hasCommit: (sha) => sha !== H }), ordinaryCoordinates()),
    /PR head object is unavailable/,
  );
});

test('real Git one- and multi-commit ordinary graphs pass without a merge or signing key', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'console-ordinary-graph-'));
  const run = (args) => spawnSync('git', ['-C', directory, ...args], { encoding: 'utf8' });
  const commit = (file, body, subject) => {
    writeFileSync(path.join(directory, file), body);
    assert.equal(run(['add', '--', file]).status, 0);
    assert.equal(run(['commit', '-m', subject]).status, 0);
    return run(['rev-parse', 'HEAD']).stdout.trim();
  };
  try {
    assert.equal(spawnSync('git', ['init', directory], { encoding: 'utf8' }).status, 0);
    assert.equal(run(['config', 'user.name', 'Ordinary Test']).status, 0);
    assert.equal(run(['config', 'user.email', 'ordinary@example.invalid']).status, 0);
    const base = commit('base.txt', 'base\n', 'base');
    const one = commit('one.txt', 'one\n', 'feat: one commit');
    const verify = (head, label) => assert.deepEqual(
      verifyBootstrapGraph(createProtectedGitOps(directory), ordinaryCoordinates({ baseSha: base, headSha: head })),
      { baseSha: base, headSha: head, admissionClass: 'ordinary-pr' },
      label,
    );
    verify(one, 'one commit');
    assert.equal(run(['reset', '--hard', base]).status, 0);
    commit('middle.txt', 'middle\n', 'feat: middle commit');
    const multi = commit('last.txt', 'last\n', 'feat: last commit');
    verify(multi, 'multiple commits');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('malformed release claims cannot fall through to ordinary admission', () => {
  const nearReleaseIdentity = ordinaryOps({
    commitIdentity: () => ({
      authorName: 'github-actions[bot]',
      authorEmail: 'wrong@example.invalid',
      committerName: 'GitHub',
      committerEmail: 'noreply@github.com',
      subject: 'chore(main): release 0.3.4',
    }),
  });
  const cases = [
    [nearReleaseIdentity, ordinaryCoordinates(), 'bot commit envelope'],
    [ordinaryOps(), ordinaryCoordinates({ prAuthorId: RELEASE_BOT_ID, prAuthorLogin: 'github-actions[bot]' }), 'bot creator'],
    [ordinaryOps(), ordinaryCoordinates({ prHeadRef: 'release-please--malformed' }), 'release ref'],
    [ordinaryOps({ diff: () => [modified('.release-please-manifest.json')] }), ordinaryCoordinates(), 'release manifest'],
    [releaseOps(), releaseCoordinates({ prAuthorId: 12345, prAuthorLogin: 'human' }), 'exact bytes with wrong creator'],
  ];
  for (const [ops, args, label] of cases) {
    assert.throws(
      () => verifyBootstrapGraph(ops, args),
      /malformed protected release claim/,
      label,
    );
  }
});

test('exact Release Please tip requires native protected-workflow proof and exact structural merge', () => {
  const args = releaseCoordinates();
  assert.deepEqual(classifyProtectedPrRoute(releaseOps(), args), {
    admissionClass: 'release-please-bot',
    candidateSha: B,
  });
  assert.deepEqual(verifyBootstrapGraph(releaseOps(), args), {
    baseSha: B,
    headSha: H,
    mergeSha: M,
    admissionClass: 'release-please-bot',
  });
  assert.throws(
    () => verifyBootstrapGraph(releaseOps(), releaseCoordinates({ releaseAuthorityProof: undefined })),
    /release authority proof/,
  );
  assert.throws(
    () => verifyBootstrapGraph(releaseOps({ parents: (sha) => (sha === H ? [B] : sha === M ? [P, H] : []) }), args),
    /merge parents must equal the protected base and exact release head/,
  );
});

test('live PR snapshot pins repository identity, state, base, head, and release routing metadata', () => {
  const live = {
    number: 760,
    state: 'open',
    draft: false,
    user: { id: RELEASE_BOT_ID, login: 'github-actions[bot]' },
    base: {
      ref: 'main',
      sha: B,
      repo: { id: PINNED_RELEASE_REPOSITORY_ID, full_name: PINNED_RELEASE_REPOSITORY },
    },
    head: {
      ref: RELEASE_HEAD_REF,
      sha: H,
      repo: { id: PINNED_RELEASE_REPOSITORY_ID, full_name: PINNED_RELEASE_REPOSITORY },
    },
  };
  const expected = {
    repository: PINNED_RELEASE_REPOSITORY,
    prNumber: 760,
    baseSha: B,
    headSha: H,
  };
  assert.deepEqual(assertLivePullRequestSnapshot(live, expected, 'before'), {
    prNumber: 760,
    prAuthorId: RELEASE_BOT_ID,
    prAuthorLogin: 'github-actions[bot]',
    prHeadRef: RELEASE_HEAD_REF,
    prHeadRepository: PINNED_RELEASE_REPOSITORY,
  });
  for (const mutation of [
    { ...live, state: 'closed' },
    { ...live, draft: true },
    { ...live, number: 761 },
    { ...live, base: { ...live.base, sha: P } },
    { ...live, head: { ...live.head, sha: P } },
    { ...live, base: { ...live.base, repo: { ...live.base.repo, id: 1 } } },
  ]) {
    assert.throws(() => assertLivePullRequestSnapshot(mutation, expected, 'after'), /live PR snapshot/);
  }
});

const expectedProtectedWorkflow = Object.freeze({
  name: 'Console authority bootstrap',
  on: {
    pull_request_target: {
      types: ['opened', 'synchronize', 'reopened', 'edited', 'ready_for_review'],
      branches: ['main'],
    },
  },
  permissions: { actions: 'read', contents: 'read', 'pull-requests': 'read' },
  jobs: {
    'authenticate-console-authority': {
      'runs-on': 'ubuntu-latest',
      'timeout-minutes': 12,
      steps: [
        {
          name: 'Checkout protected target code only',
          uses: 'actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0',
          with: {
            ref: '${{ github.event.pull_request.base.sha }}',
            'persist-credentials': false,
            'fetch-depth': 0,
          },
        },
        {
          name: 'Verify exact PR coordinates from protected code',
          env: {
            HOME: '${{ runner.temp }}/hostile-home',
            GITHUB_TOKEN: '${{ secrets.GITHUB_TOKEN }}',
            PR_NUMBER: '${{ github.event.pull_request.number }}',
            PR_HEAD_SHA: '${{ github.event.pull_request.head.sha }}',
            PR_BASE_SHA: '${{ github.event.pull_request.base.sha }}',
            PR_BASE_REF: '${{ github.event.pull_request.base.ref }}',
            REPOSITORY: '${{ github.repository }}',
          },
          run: [
            'node scripts/console/verify-console-pr-authority-bootstrap.mjs \\',
            '--pr-number "$PR_NUMBER" \\',
            '--head "$PR_HEAD_SHA" \\',
            '--base-sha "$PR_BASE_SHA" \\',
            '--base "$PR_BASE_REF" \\',
            '--repository "$REPOSITORY"',
            '',
          ].join('\n'),
        },
      ],
    },
  },
});

function assertProtectedWorkflowContract(source) {
  assert.deepEqual(yaml.load(source), expectedProtectedWorkflow);
}

test('workflow emits one unconditional protected-code-only required context', () => {
  const workflow = readFileSync(new URL('../../.github/workflows/console-authority-bootstrap.yml', import.meta.url), 'utf8');
  assertProtectedWorkflowContract(workflow);
  assert.doesNotMatch(workflow, /^\s+if:/m);
  for (const forbidden of ['pull_request.merge_commit_sha', 'event.sender', 'secrets.RELEASE', 'id-token: write', 'contents: write', 'npm ci', 'actions/cache', 'refs/pull/${{']) {
    assert.doesNotMatch(workflow, new RegExp(forbidden.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  const mutations = [
    workflow.replace(
      '      - name: Verify exact PR coordinates from protected code',
      '      - run: node scripts/console/attack.mjs\n      - name: Verify exact PR coordinates from protected code',
    ),
    workflow.replace(
      '    timeout-minutes: 12',
      '    timeout-minutes: 12\n    if: github.actor == github.repository_owner',
    ),
  ];
  for (const mutation of mutations) assert.throws(() => assertProtectedWorkflowContract(mutation));
});

const protectedExecutableClosure = Object.freeze([
  ['./verify-console-pr-authority-bootstrap.mjs', 'df55666f5d348a24e650cf9fc94e90d8cb9cb783dfa815d80b00d010490b9b83'],
  ['./authority-ledger-path.mjs', '756e838e3979508d3be0b7d9974a0e719de9f1a08effbe60c272c2cad25b498e'],
  ['./release-please-bot-candidate.mjs', 'ae3d1069165ca4aaa88a36cac8f13d8d45d952f87fb23c510da0d0a957e62fdf'],
]);
const sha256 = (value) => createHash('sha256').update(value).digest('hex');

test('protected-target executable closure is bound to reviewed capabilities', () => {
  const verifier = readFileSync(new URL('./verify-console-pr-authority-bootstrap.mjs', import.meta.url), 'utf8');
  for (const forbidden of [
    'verify-commit',
    'allowed_signers',
    'POLICY_PATH',
    'TRUSTED_FINGERPRINT',
    'writeFileSync',
    'spawnSync',
    "worktree', 'add",
  ]) {
    assert.doesNotMatch(verifier, new RegExp(forbidden.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  for (const [relativePath, expectedDigest] of protectedExecutableClosure) {
    const source = readFileSync(new URL(relativePath, import.meta.url));
    assert.equal(sha256(source), expectedDigest, relativePath);
    assert.notEqual(
      sha256(Buffer.concat([source, Buffer.from('\n// candidate execution mutation\n')])),
      expectedDigest,
      `${relativePath} digest must reject any source mutation`,
    );
  }
});

test('fetches an exact non-reachable PR head without requiring a synthetic merge ref', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'console-protected-head-fetch-'));
  const remote = path.join(directory, 'remote.git');
  const source = path.join(directory, 'source');
  const checkout = path.join(directory, 'checkout');
  const run = (cwd, args) => spawnSync('git', ['-C', cwd, ...args], { encoding: 'utf8' });
  try {
    assert.equal(spawnSync('git', ['init', '--bare', remote], { encoding: 'utf8' }).status, 0);
    assert.equal(spawnSync('git', ['init', source], { encoding: 'utf8' }).status, 0);
    assert.equal(run(source, ['config', 'user.name', 'Bootstrap Test']).status, 0);
    assert.equal(run(source, ['config', 'user.email', 'bootstrap@example.invalid']).status, 0);
    writeFileSync(path.join(source, 'base.txt'), 'base\n');
    assert.equal(run(source, ['add', 'base.txt']).status, 0);
    assert.equal(run(source, ['commit', '-m', 'base']).status, 0);
    const base = run(source, ['rev-parse', 'HEAD']).stdout.trim();
    assert.equal(run(source, ['remote', 'add', 'origin', remote]).status, 0);
    assert.equal(run(source, ['push', 'origin', 'HEAD:refs/heads/main']).status, 0);
    assert.equal(spawnSync('git', ['clone', '--branch', 'main', remote, checkout], { encoding: 'utf8' }).status, 0);
    writeFileSync(path.join(source, 'head.txt'), 'head\n');
    assert.equal(run(source, ['add', 'head.txt']).status, 0);
    assert.equal(run(source, ['commit', '-m', 'head']).status, 0);
    const head = run(source, ['rev-parse', 'HEAD']).stdout.trim();
    assert.equal(run(source, ['push', 'origin', 'HEAD:refs/pull/42/head']).status, 0);
    assert.equal(run(source, ['reset', '--hard', base]).status, 0);
    assert.notEqual(run(checkout, ['cat-file', '-e', `${head}^{commit}`]).status, 0);
    assert.equal(fetchExactPullHead(checkout, 42, head), head);
    assert.equal(run(checkout, ['rev-parse', 'refs/console-bootstrap/42/head']).stdout.trim(), head);
    assert.throws(() => fetchExactPullHead(checkout, 42, base), /head ref does not match event SHA/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('candidate compatibility fixture executes the actual planner CLI flag contract', () => {
  const candidate = spawnSync('git', ['rev-parse', 'HEAD^{commit}'], { encoding: 'utf8' }).stdout.trim();
  assert.match(candidate, /^[0-9a-f]{40}$/);
  const directory = mkdtempSync(path.join(tmpdir(), 'console-candidate-planner-'));
  try {
    assert.equal(spawnSync('git', ['worktree', 'add', '--detach', '--no-checkout', directory], { encoding: 'utf8' }).status, 0);
    assert.equal(spawnSync('git', ['-C', directory, 'checkout', '--detach', candidate], { encoding: 'utf8' }).status, 0);
    const accepted = spawnSync('node', ['scripts/console/plan-fanout.mjs', '--candidate', candidate, '--authority-tip', H, '--synthetic-merge', M], { cwd: directory, encoding: 'utf8' });
    assert.doesNotMatch(`${accepted.stdout}${accepted.stderr}`, /unknown argument/);
    const rejected = spawnSync('node', ['scripts/console/plan-fanout.mjs', '--candidate-sha', candidate, '--authority-tip-sha', H, '--synthetic-merge-sha', M], { cwd: directory, encoding: 'utf8' });
    assert.notEqual(rejected.status, 0);
    assert.match(`${rejected.stdout}${rejected.stderr}`, /unknown argument: --candidate-sha/);
  } finally {
    spawnSync('git', ['worktree', 'remove', '--force', directory], { encoding: 'utf8' });
    rmSync(directory, { recursive: true, force: true });
  }
});
