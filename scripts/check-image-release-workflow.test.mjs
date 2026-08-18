import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, describe, it } from "node:test";

import yaml from "js-yaml";

const workflowPath = new URL(
  "../.github/workflows/image-release.yml",
  import.meta.url,
);
const workflowText = readFileSync(workflowPath, "utf8");
const workflow = yaml.load(workflowText);
const admission = workflow.jobs["ci-admission"];
const admissionStep = admission.steps.find((step) => step.id === "admit");
const ciWorkflowPath = new URL("../.github/workflows/ci.yml", import.meta.url);
const ciWorkflow = yaml.load(readFileSync(ciWorkflowPath, "utf8"));
const workflowsDirectory = new URL("../.github/workflows/", import.meta.url);
const cacheHygienePath = new URL(
  "../.github/workflows/cache-hygiene.yml",
  import.meta.url,
);
const cacheHygieneText = readFileSync(cacheHygienePath, "utf8");
const cacheHygiene = yaml.load(cacheHygieneText);

const CANDIDATE = "a".repeat(40);
const PARENT = "b".repeat(40);
const OTHER = "c".repeat(40);
const REPOSITORY_ID = 1269693002;
const CI_WORKFLOW_ID = 296023727;
const SECURITY_WORKFLOW_ID = 296023731;
const CI_RUN_ID = 1101;
const SECURITY_RUN_ID = 2201;

const temporaryRoots = [];
after(() => {
  for (const root of temporaryRoots) {
    rmSync(root, { recursive: true, force: true });
  }
});

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, nested]) => [key, canonical(nested)]),
    );
  }
  return value;
}

function productionJobDigest(model = workflow) {
  const protectedJobs = {
    "production-promotion-preflight":
      model.jobs["production-promotion-preflight"],
    "bump-digests": model.jobs["bump-digests"],
  };
  return createHash("sha256")
    .update(JSON.stringify(canonical(protectedJobs)))
    .digest("hex");
}

function releaseCommit() {
  return {
    sha: CANDIDATE,
    parents: [{ sha: PARENT }],
    commit: {
      message: "chore(main): release 1.2.3 (#760)\n\nrelease notes",
      author: {
        name: "github-actions[bot]",
        email: "41898282+github-actions[bot]@users.noreply.github.com",
      },
      committer: { name: "GitHub", email: "noreply@github.com" },
      verification: { verified: true, reason: "valid" },
    },
    author: { login: "github-actions[bot]", id: 41898282, type: "Bot" },
    committer: { login: "web-flow", id: 19864447, type: "User" },
    files: [
      { filename: ".release-please-manifest.json", status: "modified" },
      { filename: "CHANGELOG.md", status: "modified" },
      { filename: "docs/documentation-index.json", status: "modified" },
      {
        filename: "docs/documentation-manifest.seed.json",
        status: "modified",
      },
    ],
  };
}

function workflowRun({
  id,
  workflowId,
  path,
  status = "completed",
  conclusion = "success",
}) {
  return {
    id,
    workflow_id: workflowId,
    path,
    run_attempt: 1,
    event: "push",
    head_branch: "main",
    head_sha: CANDIDATE,
    status,
    conclusion,
    repository: { id: REPOSITORY_ID, full_name: "oyatie/console" },
  };
}

function aggregateJobs(name, conclusion = "success") {
  return {
    total_count: 1,
    jobs: [{ name, status: "completed", conclusion }],
  };
}

function publishedRelease(overrides = {}) {
  return {
    id: 3301,
    tag_name: "v1.2.3",
    target_commitish: CANDIDATE,
    draft: false,
    prerelease: false,
    immutable: true,
    published_at: "2026-08-16T12:00:00Z",
    author: { login: "github-actions[bot]", id: 41898282, type: "Bot" },
    ...overrides,
  };
}

function baseRoutes() {
  const ciRun = workflowRun({
    id: CI_RUN_ID,
    workflowId: CI_WORKFLOW_ID,
    path: ".github/workflows/ci.yml",
  });
  const securityRun = workflowRun({
    id: SECURITY_RUN_ID,
    workflowId: SECURITY_WORKFLOW_ID,
    path: ".github/workflows/security.yml",
  });
  return {
    [`repos/oyatie/console/actions/runs/${CI_RUN_ID}`]: [ciRun],
    [`repos/oyatie/console/actions/workflows/${CI_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`]: [
      { total_count: 1, workflow_runs: [ciRun] },
    ],
    [`repos/oyatie/console/actions/runs/${CI_RUN_ID}/attempts/1/jobs?per_page=100`]: [
      aggregateJobs("Required / CI"),
    ],
    [`repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`]: [
      { total_count: 1, workflow_runs: [securityRun] },
    ],
    [`repos/oyatie/console/actions/runs/${SECURITY_RUN_ID}/attempts/1/jobs?per_page=100`]: [
      aggregateJobs("Required / Security"),
    ],
    [`repos/oyatie/console/git/ref/heads/main`]: [
      { object: { type: "commit", sha: CANDIDATE } },
    ],
    [`repos/oyatie/console/commits/${CANDIDATE}`]: [releaseCommit()],
    [`repos/oyatie/console/contents/.release-please-manifest.json?ref=${CANDIDATE}`]: [
      '{".":"1.2.3"}\n',
    ],
    [`repos/oyatie/console/contents/.release-please-manifest.json?ref=${PARENT}`]: [
      '{".":"1.2.2"}\n',
    ],
    [`repos/oyatie/console/releases/tags/v1.2.3`]: [publishedRelease()],
    [`repos/oyatie/console/releases/3301`]: [publishedRelease()],
    [`repos/oyatie/console/git/ref/tags/v1.2.3`]: [
      { object: { type: "commit", sha: CANDIDATE } },
    ],
  };
}

function fakeExecutable(path, body) {
  writeFileSync(path, `#!${process.execPath}\n${body}`);
  chmodSync(path, 0o755);
}

function runAdmission({ routes = baseRoutes(), env = {} } = {}) {
  const root = mkdtempSync(join(tmpdir(), "console-image-admission-"));
  temporaryRoots.push(root);
  const bin = join(root, "bin");
  const fixturePath = join(root, "fixture.json");
  const statePath = join(root, "state.json");
  const logPath = join(root, "requests.jsonl");
  const sleepLogPath = join(root, "sleep.log");
  const outputPath = join(root, "github-output");
  const summaryPath = join(root, "github-summary");
  mkdirSync(bin);
  writeFileSync(fixturePath, JSON.stringify(routes));
  writeFileSync(statePath, "{}");
  writeFileSync(logPath, "");
  writeFileSync(sleepLogPath, "");
  writeFileSync(outputPath, "");
  writeFileSync(summaryPath, "");
  symlinkSync("/usr/bin/jq", join(bin, "jq"));

  fakeExecutable(
    join(bin, "gh"),
    String.raw`
const fs = require("node:fs");
const args = process.argv.slice(2);
if (args[0] !== "api") process.exit(96);
const required = [
  "api", "-H", "X-GitHub-Api-Version: 2022-11-28",
  "-H", "Cache-Control: no-cache",
];
if (JSON.stringify(args.slice(0, required.length)) !== JSON.stringify(required)) {
  process.stderr.write("unexpected gh argv: " + JSON.stringify(args) + "\n");
  process.exit(97);
}
const endpoint = args.at(-1);
const ordinaryArgv = [...required, endpoint];
const rawArgv = [
  ...required,
  "-H", "Accept: application/vnd.github.raw+json",
  endpoint,
];
const expectedArgv = endpoint.includes("/contents/") ? rawArgv : ordinaryArgv;
if (JSON.stringify(args) !== JSON.stringify(expectedArgv)) {
  process.stderr.write("unexpected extra gh argv: " + JSON.stringify(args) + "\n");
  process.exit(97);
}
fs.appendFileSync(process.env.FAKE_GH_LOG, JSON.stringify(args) + "\n");
const fixtures = JSON.parse(fs.readFileSync(process.env.FAKE_GH_FIXTURE, "utf8"));
const states = JSON.parse(fs.readFileSync(process.env.FAKE_GH_STATE, "utf8"));
if (!Object.hasOwn(fixtures, endpoint)) {
  process.stderr.write("unexpected endpoint: " + endpoint + "\n");
  process.exit(98);
}
const sequence = fixtures[endpoint];
const index = states[endpoint] ?? 0;
states[endpoint] = index + 1;
fs.writeFileSync(process.env.FAKE_GH_STATE, JSON.stringify(states));
const response = sequence[Math.min(index, sequence.length - 1)];
if (response && typeof response === "object" && Object.hasOwn(response, "exit")) {
  if (response.stderr) process.stderr.write(response.stderr);
  process.exit(response.exit);
}
process.stdout.write(typeof response === "string" ? response : JSON.stringify(response));
`,
  );
  fakeExecutable(
    join(bin, "timeout"),
    `const { spawnSync } = require("node:child_process");\nconst args = process.argv.slice(2);\nif (args[0] !== "20s" || args[1] !== "gh" || args[2] !== "api") {\n  process.stderr.write("unexpected timeout argv: " + JSON.stringify(args) + "\\n");\n  process.exit(95);\n}\nconst result = spawnSync(args[1], args.slice(2), { stdio: "inherit" });\nprocess.exit(result.status ?? 99);\n`,
  );
  fakeExecutable(
    join(bin, "sleep"),
    `require("node:fs").appendFileSync(process.env.FAKE_SLEEP_LOG, process.argv.slice(2).join(" ") + "\\n");\n`,
  );

  const result = spawnSync("/bin/bash", ["-c", admissionStep.run], {
    cwd: root,
    encoding: "utf8",
    env: {
      PATH: bin,
      EVENT_NAME: "workflow_run",
      REPO: "oyatie/console",
      EXPECTED_REPOSITORY_ID: String(REPOSITORY_ID),
      GH_TOKEN: "fixture-token",
      TRIGGER_RUN_ID: String(CI_RUN_ID),
      TRIGGER_WORKFLOW_NAME: "CI",
      TRIGGER_WORKFLOW_CONCLUSION: "success",
      TRIGGER_WORKFLOW_EVENT: "push",
      TRIGGER_WORKFLOW_HEAD_BRANCH: "main",
      TRIGGER_WORKFLOW_HEAD_SHA: CANDIDATE,
      DISPATCH_CANDIDATE_SHA: "",
      DISPATCH_REF: "",
      RUN_ATTEMPT: "1",
      ADMISSION_MAX_POLLS: "2",
      ADMISSION_POLL_SECONDS: "0",
      GITHUB_OUTPUT: outputPath,
      GITHUB_STEP_SUMMARY: summaryPath,
      FAKE_GH_FIXTURE: fixturePath,
      FAKE_GH_STATE: statePath,
      FAKE_GH_LOG: logPath,
      FAKE_SLEEP_LOG: sleepLogPath,
      HTTP_PROXY: "http://127.0.0.1:9",
      HTTPS_PROXY: "http://127.0.0.1:9",
      ALL_PROXY: "http://127.0.0.1:9",
      NO_PROXY: "",
      ...env,
    },
  });

  return {
    ...result,
    output: readFileSync(outputPath, "utf8"),
    summary: readFileSync(summaryPath, "utf8"),
    sleeps: readFileSync(sleepLogPath, "utf8").trim().split("\n").filter(Boolean),
    requests: readFileSync(logPath, "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line)),
  };
}

describe("Image Release protected workflow shape", () => {
  it("keeps admission read-only and gates every privileged publication stage", () => {
    assert.deepEqual(workflow.permissions, {});
    assert.deepEqual(admission.permissions, { contents: "read", actions: "read" });
    assert.equal(admission["timeout-minutes"], 10);
    assert.equal(admission["continue-on-error"], undefined);
    assert.equal(admissionStep["continue-on-error"], undefined);
    assert.equal(admissionStep.if, undefined);
    assert.equal(admission.outputs.eligible, "${{ steps.admit.outputs.eligible }}");
    assert.equal(
      admission.outputs.release_sha,
      "${{ steps.admit.outputs.release_sha }}",
    );
    assert.equal(
      admission.outputs.release_tag,
      "${{ steps.admit.outputs.release_tag }}",
    );
    assert.equal(
      admission.outputs.release_version,
      "${{ steps.admit.outputs.release_version }}",
    );
    assert.equal(workflow.jobs.build.if, "needs.ci-admission.outputs.eligible == 'true'");
    assert.equal(workflow.jobs.build.needs, "ci-admission");
    assert.deepEqual(workflow.jobs.merge.needs, ["ci-admission", "build"]);
    const productionJobs = new Set([
      "production-promotion-preflight",
      "bump-digests",
    ]);
    const nonProductionJobs = Object.entries(workflow.jobs)
      .filter(([name]) => !productionJobs.has(name));
    for (const [name, job] of nonProductionJobs) {
      const permissions = job.permissions;
      assert.ok(
        permissions === undefined
          || (permissions !== null
            && typeof permissions === "object"
            && !Array.isArray(permissions)),
        `${name} must use mapping-form permissions; scalar write-all is forbidden`,
      );
    }
    const nonProductionWriteJobs = nonProductionJobs
      .filter(([, job]) =>
        Object.values(job.permissions ?? {}).some((access) => access === "write"))
      .map(([name]) => name)
      .sort();
    assert.deepEqual(nonProductionWriteJobs, ["build", "merge"]);
    assert.deepEqual(workflow.jobs.build.permissions, {
      contents: "read",
      packages: "write",
    });
    assert.deepEqual(workflow.jobs.merge.permissions, {
      contents: "read",
      packages: "write",
      "id-token": "write",
      attestations: "write",
    });
    for (const name of nonProductionWriteJobs) {
      const job = workflow.jobs[name];
      assert.equal(job["continue-on-error"], undefined);
      if (name === "build") {
        assert.equal(job.if, "needs.ci-admission.outputs.eligible == 'true'");
      } else {
        assert.equal(job.if, undefined);
        assert.ok(
          (Array.isArray(job.needs) ? job.needs : [job.needs]).includes("build"),
          `${name} must remain downstream of the exactly guarded build job`,
        );
      }
    }
    for (const name of ["build", "merge", "release-probe"]) {
      const job = workflow.jobs[name];
      assert.equal(job["continue-on-error"], undefined);
      for (const step of job.steps) {
        assert.equal(
          step["continue-on-error"],
          undefined,
          `${name}/${step.name ?? step.id ?? step.uses} must fail closed`,
        );
      }
    }
    assert.equal(workflow.jobs["release-probe"].if, undefined);
    for (const name of ["build", "merge"]) {
      for (const step of workflow.jobs[name].steps) {
        assert.equal(
          step.if,
          undefined,
          `${name}/${step.name ?? step.id ?? step.uses} must not be conditionally skipped`,
        );
      }
    }
    assert.deepEqual(
      workflow.jobs["release-probe"].steps
        .filter((step) => step.if !== undefined)
        .map((step) => ({ name: step.name, if: step.if })),
      [{ name: "Stop the probe container", if: "always()" }],
    );
    assert.equal(
      workflow.jobs.merge.steps.some(
        (step) => step.uses === "./.github/actions/setup-trivy",
      ),
      false,
    );
    assert.equal(
      workflow.jobs.merge.steps.some((step) =>
        String(step.uses ?? "").startsWith("actions/checkout@"),
      ),
      false,
    );
    assert.match(
      workflow.jobs.merge.steps.map((step) => step.run ?? "").join("\n"),
      /3cbae37cd440cd8676e5ce9207fe460b5641c7579a17e9d00f8894928c41a88d/,
    );
    const buildMetadata = workflow.jobs.build.steps.find((step) => step.id === "meta");
    const mergeMetadata = workflow.jobs.merge.steps.find((step) => step.id === "meta");
    assert.deepEqual(
      mergeMetadata.with.tags.trim().split("\n"),
      [
        "type=semver,pattern={{version}},value=${{ needs.ci-admission.outputs.release_tag }}",
        "type=semver,pattern={{major}}.{{minor}},value=${{ needs.ci-admission.outputs.release_tag }}",
        "type=raw,value=main",
        "type=raw,value=edge",
        "type=raw,value=sha-${{ needs.ci-admission.outputs.release_sha }}",
      ],
    );
    assert.equal(mergeMetadata.with.flavor, "latest=false");
    for (const metadata of [buildMetadata, mergeMetadata]) {
      assert.match(
        metadata.with.labels,
        /org\.opencontainers\.image\.version=\$\{\{ needs\.ci-admission\.outputs\.release_version \}\}/,
      );
      assert.match(
        metadata.with.labels,
        /org\.opencontainers\.image\.revision=\$\{\{ needs\.ci-admission\.outputs\.release_sha \}\}/,
      );
    }
    assert.deepEqual(workflow.jobs["release-probe"].needs, [
      "ci-admission",
      "merge",
    ]);
    const executableAdmissionShell = admissionStep.run
      .split("\n")
      .filter((line) => !line.trimStart().startsWith("#"))
      .join("\n");
    assert.doesNotMatch(
      executableAdmissionShell,
      /(?:^|[\s;|&()])(?:\/(?:[^/\s;|&()]+\/)+)?(?:curl|wget|git|node|python3?|eval)(?=$|[\s;|&()])/m,
    );
    assert.doesNotMatch(
      executableAdmissionShell,
      /(?:^|[;|&()]\s*)(?:(?:builtin|command)\s+)?source(?=$|\s)/m,
    );
    assert.doesNotMatch(admissionStep.run, /\$\{\{/);
    assert.equal(
      admission.steps.some((step) => /checkout|cache|artifact/i.test(step.uses ?? "")),
      false,
    );
  });

  it("does not alter either protected production-promotion job", () => {
    assert.equal(
      productionJobDigest(),
      "03b816c09d40906b50523f530551fda517cbb52a98017da9ca75ea728285eebd",
    );
  });

  it("keeps cache deletion on its independent schedule or manual trigger", () => {
    assert.deepEqual(Object.keys(cacheHygiene.on).sort(), [
      "schedule",
      "workflow_dispatch",
    ]);
    assert.doesNotMatch(cacheHygieneText, /github\.event\.workflow_run/);

    for (const filename of readdirSync(workflowsDirectory).filter((entry) =>
      entry.endsWith(".yml"))) {
      const candidate = yaml.load(
        readFileSync(new URL(filename, workflowsDirectory), "utf8"),
      );
      const subscriptions = candidate?.on?.workflow_run?.workflows ?? [];
      assert.equal(
        Array.isArray(subscriptions) && subscriptions.includes("Image Release"),
        false,
        `${filename} must not subscribe write-capable follow-up work to Image Release`,
      );
    }
  });

  it("installs locked dependencies before the dependency-backed hardening suite", () => {
    const steps = ciWorkflow.jobs["kubernetes-manifests"].steps;
    const installIndex = steps.findIndex(
      (step) =>
        step.name === "Install production-hardening test dependencies" &&
        step.run === "npm ci --ignore-scripts",
    );
    const testIndex = steps.findIndex(
      (step) =>
        step.name === "Production hardening regression tests" &&
        step.run === "npm run test:production-hardening",
    );
    assert.ok(installIndex >= 0, "dependency install step is absent");
    assert.ok(testIndex > installIndex, "hardening tests run before dependency install");
    assert.equal(
      steps[installIndex].if,
      "${{ !cancelled() && needs.preflight.outputs.run_heavy == 'true' }}",
    );
  });
});

describe("Image Release live admission shell", () => {
  it("admits one immutable exact-SHA release with exact CI and Security", () => {
    const result = runAdmission();
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=true$/m);
    assert.match(result.output, new RegExp(`^release_sha=${CANDIDATE}$`, "m"));
    assert.match(result.output, /^release_tag=v1\.2\.3$/m);
    assert.match(result.output, /^release_version=1\.2\.3$/m);
    assert.match(result.output, /^ci_run_id=1101$/m);
    assert.match(result.output, /^security_run_id=2201$/m);
  });

  it("classifies an unchanged manifest as an ordinary green no-op", () => {
    const routes = baseRoutes();
    routes[
      `repos/oyatie/console/contents/.release-please-manifest.json?ref=${PARENT}`
    ] = ['{".":"1.2.3"}\n'];
    const result = runAdmission({ routes });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=false$/m);
    assert.doesNotMatch(result.output, /^release_sha=/m);
    assert.equal(
      result.requests.some((args) => args.at(-1).includes("/releases/")),
      false,
    );
  });

  it("cleanly ignores an automatic stale candidate", () => {
    const routes = baseRoutes();
    routes["repos/oyatie/console/git/ref/heads/main"] = [
      { object: { type: "commit", sha: OTHER } },
    ];
    const result = runAdmission({ routes });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=false$/m);
  });

  it("fails red when current-main readback is unavailable during polling", () => {
    const routes = baseRoutes();
    routes["repos/oyatie/console/git/ref/heads/main"] = [
      { object: { type: "commit", sha: CANDIDATE } },
      { exit: 1, stderr: "connection refused\n" },
    ];
    const result = runAdmission({ routes });
    assert.notEqual(result.status, 0);
    assert.doesNotMatch(result.output, /^eligible=false$/m);
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("fails red instead of treating malformed main refs as stale", () => {
    for (const sha of ["not-a-full-lowercase-sha", `${CANDIDATE}\n`]) {
      const routes = baseRoutes();
      routes["repos/oyatie/console/git/ref/heads/main"] = [
        { object: { type: "commit", sha } },
      ];
      const result = runAdmission({ routes });
      assert.notEqual(result.status, 0, `accepted malformed main SHA ${JSON.stringify(sha)}`);
      assert.doesNotMatch(result.output, /^eligible=false$/m);
      assert.doesNotMatch(result.output, /^eligible=true$/m);
    }
  });

  it("fails closed on a mutable, draft, or prerelease source release", () => {
    for (const mutation of [
      { immutable: false },
      { draft: true },
      { prerelease: true },
    ]) {
      const routes = baseRoutes();
      const hostileRelease = publishedRelease(mutation);
      routes["repos/oyatie/console/releases/tags/v1.2.3"] = [hostileRelease];
      routes["repos/oyatie/console/releases/3301"] = [hostileRelease];
      const result = runAdmission({ routes });
      assert.notEqual(result.status, 0, result.stdout);
      assert.doesNotMatch(result.output, /^eligible=true$/m);
    }
  });

  it("rejects every exact-release snapshot disagreement", () => {
    const byTagMutations = [
      ["tag", { tag_name: "v1.2.4" }],
      ["target", { target_commitish: OTHER }],
      ["published timestamp", { published_at: null }],
      ["release ID type", { id: "3301" }],
      ["author login", { author: { ...publishedRelease().author, login: "jason931225" } }],
      ["author ID", { author: { ...publishedRelease().author, id: 56489493 } }],
      ["author type", { author: { ...publishedRelease().author, type: "User" } }],
    ];
    for (const [label, mutation] of byTagMutations) {
      const routes = baseRoutes();
      const hostileRelease = publishedRelease(mutation);
      routes["repos/oyatie/console/releases/tags/v1.2.3"] = [hostileRelease];
      routes["repos/oyatie/console/releases/3301"] = [hostileRelease];
      const result = runAdmission({ routes });
      assert.notEqual(result.status, 0, `accepted wrong release ${label}`);
      assert.doesNotMatch(result.output, /^eligible=true$/m);
    }

    const mismatchedById = baseRoutes();
    mismatchedById["repos/oyatie/console/releases/3301"] = [
      publishedRelease({ published_at: "2026-08-16T12:00:01Z" }),
    ];
    const result = runAdmission({ routes: mismatchedById });
    assert.notEqual(result.status, 0, "accepted mismatched release-by-ID evidence");
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("rejects every release-commit authority disagreement", () => {
    const mutations = [
      ["candidate SHA", (commit) => ({ ...commit, sha: OTHER })],
      [
        "parent count",
        (commit) => ({
          ...commit,
          parents: [...commit.parents, { sha: OTHER }],
        }),
      ],
      [
        "subject",
        (commit) => ({
          ...commit,
          commit: { ...commit.commit, message: "chore(main): release 1.2.4 (#760)" },
        }),
      ],
      [
        "subject NUL",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            message: "chore(main): release 1.2.3 (#760)\u0000\n\nrelease notes",
          },
        }),
      ],
      [
        "parent SHA",
        (commit) => ({
          ...commit,
          parents: [{ sha: `${PARENT}\n` }],
        }),
      ],
      [
        "commit author name",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            author: { ...commit.commit.author, name: "github-actions" },
          },
        }),
      ],
      [
        "commit author email",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            author: { ...commit.commit.author, email: "actions@github.com" },
          },
        }),
      ],
      [
        "commit committer name",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            committer: { ...commit.commit.committer, name: "github-actions[bot]" },
          },
        }),
      ],
      [
        "commit committer email",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            committer: { ...commit.commit.committer, email: "actions@github.com" },
          },
        }),
      ],
      [
        "verified bit",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            verification: { ...commit.commit.verification, verified: false },
          },
        }),
      ],
      [
        "verification reason",
        (commit) => ({
          ...commit,
          commit: {
            ...commit.commit,
            verification: { ...commit.commit.verification, reason: "unsigned" },
          },
        }),
      ],
      ["API author ID", (commit) => ({ ...commit, author: { ...commit.author, id: 7 } })],
      [
        "API author login",
        (commit) => ({ ...commit, author: { ...commit.author, login: "github-actions" } }),
      ],
      [
        "API author type",
        (commit) => ({ ...commit, author: { ...commit.author, type: "User" } }),
      ],
      [
        "API committer ID",
        (commit) => ({ ...commit, committer: { ...commit.committer, id: 7 } }),
      ],
      [
        "API committer login",
        (commit) => ({ ...commit, committer: { ...commit.committer, login: "github" } }),
      ],
      [
        "file envelope",
        (commit) => ({
          ...commit,
          files: [
            ...commit.files,
            { filename: "backend/src/lib.rs", status: "modified" },
          ],
        }),
      ],
    ];
    for (const [label, mutate] of mutations) {
      const routes = baseRoutes();
      routes[`repos/oyatie/console/commits/${CANDIDATE}`] = [
        mutate(releaseCommit()),
      ];
      const result = runAdmission({ routes });
      assert.notEqual(result.status, 0, `accepted wrong release-commit ${label}`);
      assert.doesNotMatch(result.output, /^eligible=true$/m);
    }
  });

  it("fails closed when exact Security is terminally red", () => {
    const routes = baseRoutes();
    routes[
      `repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`
    ] = [
      {
        total_count: 1,
        workflow_runs: [
          workflowRun({
            id: SECURITY_RUN_ID,
            workflowId: SECURITY_WORKFLOW_ID,
            path: ".github/workflows/security.yml",
            conclusion: "failure",
          }),
        ],
      },
    ];
    const result = runAdmission({ routes });
    assert.notEqual(result.status, 0, result.stdout);
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("fails closed when a workflow attempt advances during its jobs read", () => {
    const routes = baseRoutes();
    const attemptOne = workflowRun({
      id: SECURITY_RUN_ID,
      workflowId: SECURITY_WORKFLOW_ID,
      path: ".github/workflows/security.yml",
    });
    const attemptTwo = {
      ...attemptOne,
      run_attempt: 2,
      status: "in_progress",
      conclusion: null,
    };
    routes[
      `repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`
    ] = [
      { total_count: 1, workflow_runs: [attemptOne] },
      { total_count: 1, workflow_runs: [attemptTwo] },
    ];
    const result = runAdmission({ routes });
    assert.notEqual(result.status, 0, result.stdout);
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("fails closed if the immutable tag moves during the final readback", () => {
    const routes = baseRoutes();
    routes["repos/oyatie/console/git/ref/tags/v1.2.3"] = [
      { object: { type: "commit", sha: CANDIDATE } },
      { object: { type: "commit", sha: OTHER } },
    ];
    const result = runAdmission({ routes });
    assert.notEqual(result.status, 0, result.stdout);
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("polls bounded pending Security evidence and then admits it", () => {
    const routes = baseRoutes();
    routes[
      `repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`
    ] = [
      {
        total_count: 1,
        workflow_runs: [
          workflowRun({
            id: SECURITY_RUN_ID,
            workflowId: SECURITY_WORKFLOW_ID,
            path: ".github/workflows/security.yml",
            status: "in_progress",
            conclusion: null,
          }),
        ],
      },
      {
        total_count: 1,
        workflow_runs: [
          workflowRun({
            id: SECURITY_RUN_ID,
            workflowId: SECURITY_WORKFLOW_ID,
            path: ".github/workflows/security.yml",
          }),
        ],
      },
    ];
    const result = runAdmission({ routes });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=true$/m);
    assert.deepEqual(result.sleeps, ["0"]);
  });

  it("polls a not-yet-visible release only for an explicit 404", () => {
    const routes = baseRoutes();
    routes["repos/oyatie/console/releases/tags/v1.2.3"] = [
      { exit: 1, stderr: "gh: Not Found (HTTP 404)\n" },
      publishedRelease(),
    ];
    const result = runAdmission({ routes });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=true$/m);
    assert.deepEqual(result.sleeps, ["0"]);

    const networkRoutes = baseRoutes();
    networkRoutes["repos/oyatie/console/releases/tags/v1.2.3"] = [
      { exit: 1, stderr: "connection refused\n" },
    ];
    const networkFailure = runAdmission({ routes: networkRoutes });
    assert.notEqual(networkFailure.status, 0);
    assert.deepEqual(networkFailure.sleeps, []);
  });

  it("uses the same immutable release, CI, and Security proof for manual recovery", () => {
    const result = runAdmission({
      env: {
        EVENT_NAME: "workflow_dispatch",
        TRIGGER_RUN_ID: "",
        TRIGGER_WORKFLOW_NAME: "",
        TRIGGER_WORKFLOW_CONCLUSION: "",
        TRIGGER_WORKFLOW_EVENT: "",
        TRIGGER_WORKFLOW_HEAD_BRANCH: "",
        TRIGGER_WORKFLOW_HEAD_SHA: "",
        DISPATCH_CANDIDATE_SHA: CANDIDATE,
        DISPATCH_REF: "refs/heads/main",
      },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=true$/m);
    assert.equal(
      result.requests.some((args) =>
        args.at(-1).endsWith(`/actions/runs/${CI_RUN_ID}`),
      ),
      false,
    );
  });

  it("turns a terminally failed source CI wake-up into a green no-op", () => {
    const routes = baseRoutes();
    routes[`repos/oyatie/console/actions/runs/${CI_RUN_ID}`] = [
      workflowRun({
        id: CI_RUN_ID,
        workflowId: CI_WORKFLOW_ID,
        path: ".github/workflows/ci.yml",
        conclusion: "failure",
      }),
    ];
    const result = runAdmission({
      routes,
      env: { TRIGGER_WORKFLOW_CONCLUSION: "failure" },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=false$/m);
    assert.equal(result.requests.length, 1);
  });

  it("turns a successful source CI rerun wake-up into a green no-op", () => {
    const routes = baseRoutes();
    routes[`repos/oyatie/console/actions/runs/${CI_RUN_ID}`] = [
      {
        ...workflowRun({
          id: CI_RUN_ID,
          workflowId: CI_WORKFLOW_ID,
          path: ".github/workflows/ci.yml",
        }),
        run_attempt: 2,
      },
    ];
    const result = runAdmission({ routes });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.output, /^eligible=false$/m);
    assert.equal(result.requests.length, 1);
  });

  it("rejects malformed release intent and protected-workflow identity", () => {
    const malformedManifest = baseRoutes();
    malformedManifest[
      `repos/oyatie/console/contents/.release-please-manifest.json?ref=${CANDIDATE}`
    ] = ['{"component":"1.2.3"}\n'];
    assert.notEqual(runAdmission({ routes: malformedManifest }).status, 0);

    for (const hostileVersion of [
      '{".":"1.2.3\\n"}\n',
      '{".":"1.2.3\\r"}\n',
      '{".":"1.2.3\\u0000"}\n',
      '{".":"1.2.\x003"}\n',
    ]) {
      const normalizedVersion = baseRoutes();
      normalizedVersion[
        `repos/oyatie/console/contents/.release-please-manifest.json?ref=${CANDIDATE}`
      ] = [hostileVersion];
      assert.notEqual(runAdmission({ routes: normalizedVersion }).status, 0);
    }

    const annotatedTag = baseRoutes();
    annotatedTag["repos/oyatie/console/git/ref/tags/v1.2.3"] = [
      { object: { type: "tag", sha: OTHER } },
    ];
    assert.notEqual(runAdmission({ routes: annotatedTag }).status, 0);

    const wrongTrigger = baseRoutes();
    wrongTrigger[`repos/oyatie/console/actions/runs/${CI_RUN_ID}`] = [
      workflowRun({
        id: CI_RUN_ID,
        workflowId: SECURITY_WORKFLOW_ID,
        path: ".github/workflows/security.yml",
      }),
    ];
    assert.notEqual(runAdmission({ routes: wrongTrigger }).status, 0);

    const duplicateSecurity = baseRoutes();
    const securityRun = workflowRun({
      id: SECURITY_RUN_ID,
      workflowId: SECURITY_WORKFLOW_ID,
      path: ".github/workflows/security.yml",
    });
    duplicateSecurity[
      `repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`
    ] = [{ total_count: 2, workflow_runs: [securityRun, { ...securityRun, id: 2202 }] }];
    assert.notEqual(runAdmission({ routes: duplicateSecurity }).status, 0);
  });

  it("rejects every generic CI and Security run-identity disagreement", () => {
    const lanes = [
      {
        id: CI_RUN_ID,
        workflowId: CI_WORKFLOW_ID,
        path: ".github/workflows/ci.yml",
      },
      {
        id: SECURITY_RUN_ID,
        workflowId: SECURITY_WORKFLOW_ID,
        path: ".github/workflows/security.yml",
      },
    ];
    const mutations = [
      ["repository", (run) => ({ ...run, repository: { ...run.repository, id: 7 } })],
      ["workflow", (run) => ({ ...run, workflow_id: 7 })],
      ["path", (run) => ({ ...run, path: ".github/workflows/untrusted.yml" })],
      ["event", (run) => ({ ...run, event: "workflow_dispatch" })],
      ["branch", (run) => ({ ...run, head_branch: "not-main" })],
      ["head", (run) => ({ ...run, head_sha: OTHER })],
      ["status control", (run) => ({ ...run, status: "completed\u0000" })],
      ["conclusion control", (run) => ({ ...run, conclusion: "success\u0000" })],
    ];
    for (const lane of lanes) {
      const endpoint = `repos/oyatie/console/actions/workflows/${lane.workflowId}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`;
      for (const [label, mutate] of mutations) {
        const routes = baseRoutes();
        const trusted = workflowRun(lane);
        routes[endpoint] = [
          { total_count: 1, workflow_runs: [mutate(trusted)] },
        ];
        const result = runAdmission({ routes });
        assert.notEqual(
          result.status,
          0,
          `${lane.path} accepted a mismatched ${label}`,
        );
        assert.doesNotMatch(result.output, /^eligible=true$/m);
      }
    }
  });

  it("rejects missing, duplicate, or red CI and Security aggregate jobs", () => {
    const lanes = [
      { id: CI_RUN_ID, name: "Required / CI" },
      { id: SECURITY_RUN_ID, name: "Required / Security" },
    ];
    for (const lane of lanes) {
      const endpoint = `repos/oyatie/console/actions/runs/${lane.id}/attempts/1/jobs?per_page=100`;
      const cases = [
        ["missing", { total_count: 0, jobs: [] }],
        [
          "duplicate",
          {
            total_count: 2,
            jobs: [
              { name: lane.name, status: "completed", conclusion: "success" },
              { name: lane.name, status: "completed", conclusion: "success" },
            ],
          },
        ],
        ["red", aggregateJobs(lane.name, "failure")],
        [
          "normalized status",
          {
            total_count: 1,
            jobs: [{ name: lane.name, status: "completed\u0000", conclusion: "success" }],
          },
        ],
        [
          "normalized conclusion",
          {
            total_count: 1,
            jobs: [{ name: lane.name, status: "completed", conclusion: "success\u0000" }],
          },
        ],
        [
          "normalized total count",
          {
            total_count: "1\u0000",
            jobs: [{ name: lane.name, status: "completed", conclusion: "success" }],
          },
        ],
      ];
      for (const [label, jobs] of cases) {
        const routes = baseRoutes();
        routes[endpoint] = [jobs];
        const result = runAdmission({ routes });
        assert.notEqual(result.status, 0, `${lane.name} accepted ${label} aggregate evidence`);
        assert.doesNotMatch(result.output, /^eligible=true$/m);
      }
    }
  });

  it("rejects a shell-normalizable workflow run count before extraction", () => {
    const routes = baseRoutes();
    const endpoint = `repos/oyatie/console/actions/workflows/${SECURITY_WORKFLOW_ID}/runs?event=push&branch=main&head_sha=${CANDIDATE}&per_page=100`;
    routes[endpoint] = [
      {
        total_count: "1\u0000",
        workflow_runs: [
          workflowRun({
            id: SECURITY_RUN_ID,
            workflowId: SECURITY_WORKFLOW_ID,
            path: ".github/workflows/security.yml",
          }),
        ],
      },
    ];
    const result = runAdmission({ routes });
    assert.notEqual(result.status, 0);
    assert.doesNotMatch(result.output, /^eligible=true$/m);
  });

  it("rejects Image Release reruns before any API read", () => {
    const result = runAdmission({ env: { RUN_ATTEMPT: "2" } });
    assert.notEqual(result.status, 0);
    assert.deepEqual(result.requests, []);
  });
});
