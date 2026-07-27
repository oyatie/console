#!/usr/bin/env node
import assert from "node:assert/strict";
import { test } from "node:test";

import { assertPlanCoversCi } from "./verify.mjs";

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

test("a plan entry whose CI step disappeared fails the guard", () => {
  // Drop one real step: the plan now claims to mirror something CI no longer runs.
  const steps = assertPlanCoversCi().filter((step) => step.name !== "rustfmt check");
  assert.throws(
    () => assertPlanCoversCi(steps),
    /no matching CI step/,
    "a stale mirror entry must fail closed, not silently linger",
  );
});
