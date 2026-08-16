import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import yaml from 'js-yaml';
import {
  assertReleasePleaseActionCoreBinding,
  createReleasePushAskpass,
  parseReleasePleaseActionPr,
  redactReleasePushError,
  releasePushInvocation,
  releaseTransportCommitEnvironment,
  sanitizedChildEnvironment,
} from './converge-release-please-doc-custody.mjs';
import {
  RELEASE_PLEASE_BOT_EMAIL,
  RELEASE_PLEASE_BOT_NAME,
  RELEASE_PLEASE_COMMITTER_EMAIL,
  RELEASE_PLEASE_COMMITTER_NAME,
  RELEASE_PLEASE_PATHS,
} from './release-please-bot-candidate.mjs';

const C = 'c'.repeat(40);
const T = 'a'.repeat(40);
const REPOSITORY = 'jason931225/console';
const HEAD_REF = 'release-please--branches--main--components--console';
const NOTES = [
  '## [0.3.7](https://github.com/jason931225/console/compare/v0.3.6...v0.3.7) (2026-08-16)',
  '',
  '',
  '### Bug Fixes',
  '',
  '* **ci:** separate candidate validation from release authority ([abc1234](https://github.com/jason931225/console/commit/abc1234))',
].join('\n');
const PR_BODY = [
  ':robot: I have created a release *beep* *boop*',
  '---',
  '',
  '',
  NOTES,
  '',
  '---',
  'This PR was generated with [Release Please](https://github.com/googleapis/release-please). See [documentation](https://github.com/googleapis/release-please#release-please).',
].join('\n');
const BASE_CHANGELOG = [
  '# Changelog',
  '',
  '## [0.3.6](https://github.com/jason931225/console/compare/v0.3.5...v0.3.6) (2026-08-11)',
  '',
  '',
  '### Bug Fixes',
  '',
  '* previous release',
  '',
].join('\n');
const HEAD_CHANGELOG = `# Changelog\n\n${NOTES}\n\n${BASE_CHANGELOG.slice('# Changelog\n\n'.length)}`;
const actionPr = Object.freeze({
  headBranchName: HEAD_REF,
  baseBranchName: 'main',
  number: 760,
  title: 'chore(main): release 0.3.7',
  body: PR_BODY,
  labels: ['autorelease: pending'],
  files: [],
});
const livePr = Object.freeze({
  number: 760,
  state: 'open',
  title: actionPr.title,
  body: actionPr.body,
  user: { login: 'github-actions[bot]' },
  head: { ref: HEAD_REF, sha: T, repo: { full_name: REPOSITORY } },
  base: { ref: 'main' },
});
const identity = Object.freeze({
  authorName: RELEASE_PLEASE_BOT_NAME,
  authorEmail: RELEASE_PLEASE_BOT_EMAIL,
  committerName: RELEASE_PLEASE_COMMITTER_NAME,
  committerEmail: RELEASE_PLEASE_COMMITTER_EMAIL,
  subject: actionPr.title,
});
const pathChanges = RELEASE_PLEASE_PATHS.map((path) => ({
  path,
  status: 'M',
  oldMode: '100644',
  newMode: '100644',
  oldType: 'blob',
  newType: 'blob',
}));
const fixture = (overrides = {}) => ({
  actionPr,
  livePr,
  repository: REPOSITORY,
  actualHeadSha: T,
  expectedParentSha: C,
  actualParentSha: C,
  identity,
  pathChanges,
  baseManifest: Buffer.from('{\n  ".": "0.3.6"\n}\n'),
  headManifest: Buffer.from('{\n  ".": "0.3.7"\n}\n'),
  baseChangelog: Buffer.from(BASE_CHANGELOG),
  headChangelog: Buffer.from(HEAD_CHANGELOG),
  ...overrides,
});
const protectedReleaseIssuerClosure = Object.freeze([
  ['./converge-release-please-doc-custody.mjs', '8ed1a06fddbe64ef2893f0252502f17d55cd0b5ab1716d20c37c63baf553605c'],
  ['./generate-documentation-manifest.mjs', 'eb353f442a6d7d84b659d43424dd52fbf9243f73d813f840e2d14aa7277fef77'],
  ['./release-please-bot-candidate.mjs', 'ae3d1069165ca4aaa88a36cac8f13d8d45d952f87fb23c510da0d0a957e62fdf'],
  ['./authority-ledger-path.mjs', '756e838e3979508d3be0b7d9974a0e719de9f1a08effbe60c272c2cad25b498e'],
  ['../check-release-metadata.mjs', '534b49d8426a1a3ae86e88ca026cf034162dd1d6289de3f9c4103e447b998b4a'],
]);
const sha256 = (value) => createHash('sha256').update(value).digest('hex');

test('binds an exact release action output to the generated core tip', () => {
  assert.deepEqual(parseReleasePleaseActionPr(JSON.stringify(actionPr)), actionPr);
  assert.deepEqual(assertReleasePleaseActionCoreBinding(fixture()), {
    prNumber: 760,
    headRef: HEAD_REF,
    headSha: T,
    parentSha: C,
    version: '0.3.7',
  });
});

test('rejects a raced live PR or a tip detached from the triggering main SHA', () => {
  assert.throws(
    () => assertReleasePleaseActionCoreBinding(fixture({
      livePr: { ...livePr, head: { ...livePr.head, sha: 'b'.repeat(40) } },
    })),
    /live PR head.*action-bound tip|head SHA/,
  );
  assert.throws(
    () => assertReleasePleaseActionCoreBinding(fixture({ actualParentSha: 'd'.repeat(40) })),
    /triggering main SHA|parent/,
  );
});

test('rejects PR metadata, repository, action body, or bot identity drift', () => {
  for (const [overrides, pattern] of [
    [{ livePr: { ...livePr, body: `${PR_BODY}\nforged` } }, /body/],
    [{ livePr: { ...livePr, title: 'chore(main): release 9.9.9' } }, /title/],
    [{ livePr: { ...livePr, user: { login: 'jason931225' } } }, /creator/],
    [{ livePr: { ...livePr, head: { ...livePr.head, repo: { full_name: 'attacker/console' } } } }, /repository/],
    [{ actionPr: { ...actionPr, headBranchName: 'evil' } }, /head ref/],
    [{ identity: { ...identity, committerName: 'Jason Lee' } }, /tip committer/],
  ]) {
    assert.throws(() => assertReleasePleaseActionCoreBinding(fixture(overrides)), pattern);
  }
});

test('rejects release core bytes not deterministically represented by the action output', () => {
  for (const [overrides, pattern] of [
    [{ headManifest: Buffer.from('{\n  ".": "0.3.8"\n}\n') }, /0\.3\.8|title|body/],
    [{ headChangelog: Buffer.from(HEAD_CHANGELOG.replace('separate candidate', 'forged candidate')) }, /CHANGELOG|release notes/],
    [{ headChangelog: Buffer.from(`${HEAD_CHANGELOG}\nforged`) }, /CHANGELOG|parent/],
    [{ pathChanges: [...pathChanges, { path: 'README.md', status: 'M', oldMode: '100644', newMode: '100644', oldType: 'blob', newType: 'blob' }] }, /exactly|release core/],
    [{ pathChanges: pathChanges.map((entry, index) => index === 0 ? { ...entry, newMode: '100755' } : entry) }, /mode-100644/],
  ]) {
    assert.throws(() => assertReleasePleaseActionCoreBinding(fixture(overrides)), pattern);
  }
});

test('rejects missing, malformed, or ambiguous release action output', () => {
  for (const raw of ['', 'null', '[]', '{}', '{']) {
    assert.throws(() => parseReleasePleaseActionPr(raw), /release action PR output/);
  }
});

test('strips scheduling-capable push tokens from every non-transport child environment', () => {
  assert.deepEqual(sanitizedChildEnvironment({
    PATH: '/usr/bin',
    GH_TOKEN: 'protected-read-token',
    GITHUB_TOKEN: 'protected-read-token',
    CONSOLE_RELEASE_PUSH_TOKEN: 'push-token',
    RELEASE_PLEASE_TOKEN: 'legacy-push-token',
  }), {
    PATH: '/usr/bin',
    GH_TOKEN: 'protected-read-token',
    GITHUB_TOKEN: 'protected-read-token',
  });
});

test('a fixed-clock custody transport commit is guaranteed to differ from the action tip', () => {
  const repository = mkdtempSync(join(tmpdir(), 'console-rp-distinct-tip-test-'));
  const git = (args, env = process.env) => execFileSync(
    'git',
    ['-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', ...args],
    { cwd: repository, encoding: 'utf8', env, stdio: ['ignore', 'pipe', 'pipe'] },
  );
  const fixedIdentity = {
    ...process.env,
    GIT_AUTHOR_NAME: RELEASE_PLEASE_BOT_NAME,
    GIT_AUTHOR_EMAIL: RELEASE_PLEASE_BOT_EMAIL,
    GIT_COMMITTER_NAME: RELEASE_PLEASE_COMMITTER_NAME,
    GIT_COMMITTER_EMAIL: RELEASE_PLEASE_COMMITTER_EMAIL,
  };
  try {
    git(['init', '-q']);
    writeFileSync(join(repository, 'release.txt'), 'base\n');
    git(['add', 'release.txt']);
    git(['commit', '-q', '-m', 'base'], {
      ...fixedIdentity,
      GIT_AUTHOR_DATE: '@1699999999 +0000',
      GIT_COMMITTER_DATE: '@1699999999 +0000',
    });
    writeFileSync(join(repository, 'release.txt'), 'release\n');
    git(['add', 'release.txt']);
    git(['commit', '-q', '-m', actionPr.title], {
      ...fixedIdentity,
      GIT_AUTHOR_DATE: '@1700000000 +0000',
      GIT_COMMITTER_DATE: '@1700000000 +0000',
    });
    const tip = git(['rev-parse', 'HEAD']).trim();
    const parent = git(['rev-parse', 'HEAD^']).trim();
    const sourceCommitterEpoch = git(['show', '-s', '--format=%ct', tip]).trim();
    git(['reset', '--soft', parent]);
    const transportEnvironment = releaseTransportCommitEnvironment({
      sourceCommitterEpoch,
      sourceEnvironment: {
        ...fixedIdentity,
        GIT_AUTHOR_DATE: '@1700000000 +0000',
      },
    });
    git(['commit', '-q', '-m', actionPr.title], transportEnvironment);
    const newTip = git(['rev-parse', 'HEAD']).trim();
    assert.notEqual(newTip, tip);
    assert.equal(git(['show', '-s', '--format=%ct', newTip]).trim(), '1700000001');
    assert.equal(transportEnvironment.GIT_COMMITTER_DATE, '@1700000001 +0000');
    assert.equal(transportEnvironment.CONSOLE_RELEASE_PUSH_TOKEN, undefined);
    assert.equal(transportEnvironment.RELEASE_PLEASE_TOKEN, undefined);
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
  for (const sourceCommitterEpoch of ['', '-1', '01', 'not-a-time', '9999999999']) {
    assert.throws(
      () => releaseTransportCommitEnvironment({ sourceCommitterEpoch }),
      /committer epoch|bounded distinct timestamp/,
    );
  }
});

test('keeps the push token out of argv, process errors, and non-password askpass responses', () => {
  const token = `ghp_${'S'.repeat(40)}`;
  const transportDirectory = mkdtempSync(join(tmpdir(), 'console-rp-transport-test-'));
  try {
    const askpassPath = createReleasePushAskpass(transportDirectory);
    assert.equal(statSync(askpassPath).mode & 0o777, 0o700);
    assert.equal(execFileSync(askpassPath, ['Password for https://x-access-token@github.com:'], {
      encoding: 'utf8',
      env: { CONSOLE_RELEASE_PUSH_TOKEN: token },
    }), `${token}\n`);
    const wrongPrompt = spawnSync(askpassPath, ['Username for https://github.com:'], {
      encoding: 'utf8',
      env: { CONSOLE_RELEASE_PUSH_TOKEN: token },
    });
    assert.notEqual(wrongPrompt.status, 0);
    assert.doesNotMatch(`${wrongPrompt.stdout}${wrongPrompt.stderr}`, new RegExp(token));

    const invocation = releasePushInvocation({
      repository: REPOSITORY,
      headRef: HEAD_REF,
      leaseTip: T,
      token,
      askpassPath,
      sourceEnvironment: {
        PATH: process.env.PATH,
        RELEASE_PLEASE_TOKEN: 'legacy-secret',
      },
    });
    assert.doesNotMatch(JSON.stringify(invocation.args), new RegExp(token));
    assert.match(JSON.stringify(invocation.args), /https:\/\/x-access-token@github\.com\/jason931225\/console\.git/);
    assert.equal(invocation.env.CONSOLE_RELEASE_PUSH_TOKEN, token);
    assert.equal(invocation.env.RELEASE_PLEASE_TOKEN, undefined);
    assert.equal(invocation.env.GIT_ASKPASS, askpassPath);
    assert.equal(invocation.env.GIT_TERMINAL_PROMPT, '0');

    let failed;
    try {
      execFileSync('/usr/bin/false', invocation.args, { env: invocation.env });
    } catch (error) {
      failed = error;
    }
    assert.ok(failed instanceof Error);
    assert.doesNotMatch(String(failed.stack), new RegExp(token));
    assert.doesNotMatch(redactReleasePushError(new Error(`transport failed: ${token}`), [token]), new RegExp(token));
  } finally {
    rmSync(transportDirectory, { recursive: true, force: true });
  }
});

const expectedReleaseWorkflow = Object.freeze({
  name: 'Release Please',
  on: {
    push: { branches: ['main'] },
  },
  permissions: {},
  concurrency: { group: 'release-please', 'cancel-in-progress': false },
  jobs: {
    'release-please': {
      'runs-on': 'ubuntu-latest',
      permissions: { contents: 'write', 'pull-requests': 'write' },
      outputs: {
        pr_number: '${{ steps.converge.outputs.pr_number }}',
        head_sha: '${{ steps.converge.outputs.head_sha }}',
        parent_sha: '${{ steps.converge.outputs.parent_sha }}',
      },
      steps: [
        {
          uses: 'googleapis/release-please-action@8b8fd2cc23b2e18957157a9d923d75aa0c6f6ad5',
          id: 'release',
          with: {
            token: '${{ secrets.GITHUB_TOKEN }}',
            'config-file': 'release-please-config.json',
            'manifest-file': '.release-please-manifest.json',
          },
        },
        {
          uses: 'actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0',
          if: '${{ steps.release.outputs.pr }}',
          with: { 'fetch-depth': 0, 'persist-credentials': false },
        },
        {
          uses: 'actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e',
          if: '${{ steps.release.outputs.pr }}',
          with: { 'node-version': '24.16.0' },
        },
        {
          name: 'Converge documentation custody on release-please tip',
          id: 'converge',
          if: '${{ steps.release.outputs.pr }}',
          env: {
            RELEASE_PLEASE_PR: '${{ steps.release.outputs.pr }}',
            CONSOLE_RELEASE_PUSH_TOKEN: '${{ secrets.RELEASE_PLEASE_TOKEN }}',
            GH_TOKEN: '${{ secrets.GITHUB_TOKEN }}',
            GITHUB_TOKEN: '${{ secrets.GITHUB_TOKEN }}',
          },
          run: 'node scripts/console/converge-release-please-doc-custody.mjs',
        },
      ],
    },
    'release-authority-proof': {
      needs: 'release-please',
      name: 'release-authority-proof pr=${{ needs.release-please.outputs.pr_number }} head=${{ needs.release-please.outputs.head_sha }}',
      if: "${{ needs.release-please.outputs.pr_number != '' && needs.release-please.outputs.head_sha != '' && needs.release-please.outputs.parent_sha != '' }}",
      'runs-on': 'ubuntu-latest',
      'timeout-minutes': 2,
      permissions: {},
      steps: [
        {
          name: 'Assert protected release proof coordinates',
          env: {
            PR_NUMBER: '${{ needs.release-please.outputs.pr_number }}',
            HEAD_SHA: '${{ needs.release-please.outputs.head_sha }}',
            PARENT_SHA: '${{ needs.release-please.outputs.parent_sha }}',
          },
          run: [
            'set -eu',
            'test "$PARENT_SHA" = "$GITHUB_SHA"',
            'test "$HEAD_SHA" != "$PARENT_SHA"',
            'test "${#HEAD_SHA}" -eq 40',
            'test "$PR_NUMBER" -gt 0',
            'case "$HEAD_SHA" in',
            '  *[!0-9a-f]*) exit 1 ;;',
            'esac',
            '',
          ].join('\n'),
        },
      ],
    },
  },
});

test('protected release workflow keeps the PAT transport-only and emits one native proof job', () => {
  const source = readFileSync(
    new URL('../../.github/workflows/release-please.yml', import.meta.url),
    'utf8',
  );
  const model = yaml.load(source);
  assert.deepEqual(model, expectedReleaseWorkflow);
  const releaseConfig = JSON.parse(readFileSync(
    new URL('../../release-please-config.json', import.meta.url),
    'utf8',
  ));
  assert.equal(releaseConfig['always-update'], true);
  assert.deepEqual(model.permissions, {});

  const producer = model.jobs['release-please'];
  assert.deepEqual(producer.permissions, { contents: 'write', 'pull-requests': 'write' });
  assert.deepEqual(producer.outputs, {
    pr_number: '${{ steps.converge.outputs.pr_number }}',
    head_sha: '${{ steps.converge.outputs.head_sha }}',
    parent_sha: '${{ steps.converge.outputs.parent_sha }}',
  });
  const release = producer.steps.find((step) => step.id === 'release');
  assert.equal(release.with.token, '${{ secrets.GITHUB_TOKEN }}');
  assert.equal(Object.hasOwn(release.with, 'bootstrap-sha'), false);
  assert.doesNotMatch(JSON.stringify(release), /RELEASE_PLEASE_TOKEN/);

  const checkout = producer.steps.find((step) => String(step.uses ?? '').startsWith('actions/checkout@'));
  assert.equal(checkout.if, '${{ steps.release.outputs.pr }}');
  assert.equal(checkout.with['persist-credentials'], false);
  assert.equal(Object.hasOwn(checkout.with, 'token'), false);
  const setup = producer.steps.find((step) => String(step.uses ?? '').startsWith('actions/setup-node@'));
  assert.equal(setup.if, '${{ steps.release.outputs.pr }}');

  const converge = producer.steps.find((step) => step.id === 'converge');
  assert.equal(converge.if, '${{ steps.release.outputs.pr }}');
  assert.deepEqual(converge.env, {
    RELEASE_PLEASE_PR: '${{ steps.release.outputs.pr }}',
    CONSOLE_RELEASE_PUSH_TOKEN: '${{ secrets.RELEASE_PLEASE_TOKEN }}',
    GH_TOKEN: '${{ secrets.GITHUB_TOKEN }}',
    GITHUB_TOKEN: '${{ secrets.GITHUB_TOKEN }}',
  });

  const proof = model.jobs['release-authority-proof'];
  assert.equal(proof.needs, 'release-please');
  assert.equal(
    proof.name,
    'release-authority-proof pr=${{ needs.release-please.outputs.pr_number }} head=${{ needs.release-please.outputs.head_sha }}',
  );
  assert.equal(
    proof.if,
    "${{ needs.release-please.outputs.pr_number != '' && needs.release-please.outputs.head_sha != '' && needs.release-please.outputs.parent_sha != '' }}",
  );
  assert.deepEqual(proof.permissions, {});
  assert.equal(proof.steps.length, 1);
  assert.deepEqual(proof.steps[0].env, {
    PR_NUMBER: '${{ needs.release-please.outputs.pr_number }}',
    HEAD_SHA: '${{ needs.release-please.outputs.head_sha }}',
    PARENT_SHA: '${{ needs.release-please.outputs.parent_sha }}',
  });
  assert.match(proof.steps[0].run, /test "\$PARENT_SHA" = "\$GITHUB_SHA"/);
  assert.doesNotMatch(source, /statuses:\s*write|checks:\s*write|upload-artifact|createCommitStatus|external_id/);

  for (const mutation of [
    source.replace(
      '      - name: Converge documentation custody on release-please tip',
      '      - run: echo "$CONSOLE_RELEASE_PUSH_TOKEN"\n      - name: Converge documentation custody on release-please tip',
    ),
    source.replace(
      '          token: ${{ secrets.GITHUB_TOKEN }}',
      '          token: ${{ secrets.RELEASE_PLEASE_TOKEN }}',
    ),
    `${source}\n  attacker-job:\n    runs-on: ubuntu-latest\n    steps: []\n`,
  ]) {
    assert.notEqual(mutation, source);
    assert.notDeepEqual(yaml.load(mutation), expectedReleaseWorkflow);
  }
});

test('protected release authority issuer executable closure is review-pinned', () => {
  for (const [relativePath, expectedDigest] of protectedReleaseIssuerClosure) {
    const source = readFileSync(new URL(relativePath, import.meta.url));
    assert.equal(sha256(source), expectedDigest, relativePath);
    assert.notEqual(
      sha256(Buffer.concat([source, Buffer.from('\n// unreviewed issuer mutation\n')])),
      expectedDigest,
      `${relativePath} must reject any source mutation`,
    );
  }
});
