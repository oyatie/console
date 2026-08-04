#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import yaml from "js-yaml";

import * as verifyModule from "./verify.mjs";

import {
  assertJobsDeclared,
  assertPlanCoversCi,
  reasoningLensLocalRunFromPlan,
  REASONING_LENS_LOCAL_RUN,
  topologyEnvContents,
  writePrivateFile,
} from "./verify.mjs";

const root = new URL("..", import.meta.url);

test("every mirrored CI run-step is classified", () => {
  const steps = assertPlanCoversCi();
  assert.ok(steps.length > 0, "expected mirrored CI steps");
});

test("a new CI step that nobody classified fails the guard", () => {
  const steps = assertPlanCoversCi().concat({
    job: "backend",
    name: "Brand new gate nobody mirrored",
    run: "echo hi",
  });
  assert.throws(
    () => assertPlanCoversCi(steps),
    /missing from scripts\/verify\.mjs/,
    "adding a CI step without classifying it must fail closed",
  );
});

test("every CI job is declared mirrored or explicitly not mirrored", () => {
  const jobs = assertJobsDeclared();
  assert.ok(jobs.length > 0, "expected CI jobs");
});

test("Required / CI is orchestration-only while every leaf proof remains classified", () => {
  assert.equal(typeof verifyModule.jobMirrorDisposition, "function");
  assert.equal(
    verifyModule.jobMirrorDisposition("required-ci"),
    "terminal status aggregate; its exact needs/result contract is enforced by check-ci-preflight",
  );

  const workflow = yaml.load(
    readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8"),
  );
  for (const dependency of workflow.jobs["required-ci"].needs) {
    const disposition = verifyModule.jobMirrorDisposition(dependency);
    assert.ok(
      disposition === true || (typeof disposition === "string" && disposition.length > 0),
      `${dependency} must remain mirrored or carry an explicit non-mirror rationale`,
    );
  }
});

test("a whole new CI job that nobody declared fails the guard", () => {
  // Without this, step-level completeness silently scopes itself to the jobs
  // the list already knew about, and a new job lands with zero local coverage.
  const jobs = assertJobsDeclared().concat("brand-new-job");
  assert.throws(
    () => assertJobsDeclared(jobs),
    /CI jobs missing from scripts\/verify\.mjs/,
    "a new CI job must fail closed, not be silently unmirrored",
  );
});

test("a declared job that CI no longer runs fails the guard", () => {
  const jobs = assertJobsDeclared().filter((name) => name !== "repo-gates");
  assert.throws(
    () => assertJobsDeclared(jobs),
    /Declared jobs with no matching CI job/,
    "a stale job declaration must fail closed",
  );
});

test("a plan entry whose CI step disappeared fails the guard", () => {
  // Drop one real step: the plan now claims to mirror something CI no longer runs.
  const steps = assertPlanCoversCi().filter((step) => step.name !== "rustfmt check");
  assert.throws(
    () => assertPlanCoversCi(steps),
    /no matching CI step/,
    "a stale mirror entry must fail closed, not silently linger",
  );
});

test("domain-unit is part of the local mirror", () => {
  const domain = assertPlanCoversCi().filter((step) => step.job === "domain-unit");
  assert.deepEqual(domain.map((step) => step.name), ["Domain crate unit tests"]);
});

test("reasoning-lens admission uses the merge-base local override", () => {
  assert.equal(
    REASONING_LENS_LOCAL_RUN,
    [
      "set -euo pipefail",
      'reasoning_base="$(git merge-base HEAD "${CONSOLE_VERIFY_BASE:-origin/main}")"',
      'node scripts/check-reasoning-lens-contract.mjs --changed-since "$reasoning_base"',
    ].join("\n"),
  );
  assert.equal(reasoningLensLocalRunFromPlan(), REASONING_LENS_LOCAL_RUN);
  assert.doesNotMatch(REASONING_LENS_LOCAL_RUN, /--changed-since (?:origin\/main|\$\{CONSOLE_VERIFY_BASE)/);
});

test("PostgreSQL provisioning keeps generated credentials out of Docker argv", () => {
  const source = readFileSync(new URL("./verify.mjs", import.meta.url), "utf8");
  for (const oldArgvShape of [
    "-e POSTGRES_PASSWORD=${adminPassword}",
    "-e POSTGRES_ADMIN_PASSWORD=${adminPassword}",
    "-e CONSOLE_APP_POSTGRES_PASSWORD=${bootstrapPassword}",
    "-e PGPASSWORD=${adminPassword}",
  ]) {
    assert.doesNotMatch(source, new RegExp(oldArgvShape.replace(/[${}]/g, "\\$&")));
  }
  assert.match(source, /\[\s*"exec",\s*"--env-file",\s*topologyEnv,\s*name/);
  assert.match(source, /\[\s*"exec",\s*"--env-file",\s*adminEnv,\s*name/);
});

test("secret-bearing temporary files are private at creation", () => {
  const root = mkdtempSync(join(tmpdir(), "console-verify-mode-"));
  const path = join(root, "secret.env");
  try {
    writePrivateFile(path, "PASSWORD=fixture\n");
    assert.equal(statSync(path).mode & 0o777, 0o600);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("topology env assembly requires pairwise-distinct role credentials", () => {
  const credentials = {
    admin: "admin-secret",
    app: "app-secret",
    runtime: "runtime-secret",
    leaveCommand: "leave-secret",
    ontologyCommand: "ontology-secret",
    platformForce: "force-secret",
  };
  const values = topologyEnvContents(credentials)
    .split("\n")
    .filter((line) => /PASSWORD=/.test(line))
    .map((line) => line.slice(line.indexOf("=") + 1));
  assert.equal(new Set(values).size, 6);
  assert.throws(
    () => topologyEnvContents({ ...credentials, runtime: credentials.app }),
    /pairwise distinct/,
  );
});

test("the topology script rejects duplicate role passwords before DB access", () => {
  const runTopology = (env) => spawnSync("bash", ["ops/postgres-reconcile-topology.sh"], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
  const duplicate = runTopology({
    POSTGRES_HOST: "127.0.0.1",
    POSTGRES_DB: "does_not_matter",
    POSTGRES_ADMIN_USER: "postgres",
    POSTGRES_ADMIN_PASSWORD: "same",
    CONSOLE_APP_POSTGRES_PASSWORD: "same",
    CONSOLE_RT_POSTGRES_PASSWORD: "same",
    CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD: "same",
    CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD: "same",
    CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD: "same",
  });
  assert.notEqual(duplicate.status, 0);
  assert.match(duplicate.stderr, /passwords must be pairwise distinct/);

  const unique = Object.fromEntries(
    topologyEnvContents({
      admin: "admin-secret",
      app: "app-secret",
      runtime: "runtime-secret",
      leaveCommand: "leave-secret",
      ontologyCommand: "ontology-secret",
      platformForce: "force-secret",
    }).trim().split("\n").map((line) => line.split(/=(.*)/s).slice(0, 2)),
  );
  // Port 1 is reserved and cannot host PostgreSQL. This proves the script got
  // past the credential preflight without depending on the developer's local
  // database state.
  unique.POSTGRES_PORT = "1";
  const afterPreflight = runTopology(unique);
  assert.notEqual(afterPreflight.status, 0, "fixture intentionally has no database");
  assert.doesNotMatch(afterPreflight.stderr, /passwords must be pairwise distinct/);
});
