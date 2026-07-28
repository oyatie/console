#!/usr/bin/env node
import assert from "node:assert/strict";
import { test } from "node:test";

import { assertJobsDeclared, assertPlanCoversCi } from "./verify.mjs";

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
