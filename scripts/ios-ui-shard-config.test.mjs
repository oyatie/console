import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { shardConfig, shardDefaults } from "./ios-ui-shard-config.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const WORKFLOW = readFileSync(
  resolve(REPO_ROOT, ".github/workflows/ios-ui-tests.yml"),
  "utf8",
);

test("inherits the defaults assigned before the case block", () => {
  assert.deepEqual(shardDefaults(WORKFLOW), {
    fixtureProfile: "full",
    contentSize: "large",
  });
  const camera = shardConfig("camera-capture", WORKFLOW);
  assert.equal(camera.fixtureProfile, "full");
  assert.equal(camera.contentSize, "large");
});

test("reads a shard's own overrides", () => {
  const ax5 = shardConfig("dynamic-type-ax5", WORKFLOW);
  assert.equal(ax5.fixtureProfile, "accessibility-audit-one-row");
  assert.equal(ax5.contentSize, "accessibility-extra-extra-extra-large");
  assert.deepEqual(ax5.selectors, [
    "MaintenanceFieldUITests/DynamicTypeRuntimeUITests/testAccessibilityExtraExtraExtraLargeRuntimeContract",
  ]);
});

test("reads every selector of a multi-test shard", () => {
  const shard = shardConfig("messenger-mutation", WORKFLOW);
  assert.equal(shard.selectors.length, 2);
  assert.ok(shard.selectors.every((s) => s.startsWith("MaintenanceFieldUITests/")));
});

test("only camera-capture resets camera privacy", () => {
  assert.equal(shardConfig("camera-capture", WORKFLOW).resetsCameraPrivacy, true);
  for (const other of ["dynamic-type-ax5", "critical-today", "messenger-render"]) {
    assert.equal(shardConfig(other, WORKFLOW).resetsCameraPrivacy, false);
  }
});

test("the timeout tracks the workflow rather than a local copy", () => {
  // A hardcoded expectation here would reintroduce the drift this module exists
  // to prevent, so assert the relationship instead of the number.
  const shard = shardConfig("critical-report", WORKFLOW);
  const declared = new RegExp(
    `critical-report\\)\\s*\\n\\s*SHARD_TIMEOUT_SECONDS=${shard.timeoutSeconds}\\b`,
  );
  assert.match(WORKFLOW, declared);
});

test("every shard named in the matrix resolves", () => {
  const manifests = [...WORKFLOW.matchAll(/shards:\s*"([^"]+)"/g)].map((m) => m[1]);
  // Derived from the matrix, never hardcoded: a literal batch count here is the
  // same drift this module exists to prevent, and it broke the moment the
  // matrix was repartitioned from seven batches to five.
  const declaredBatches = [...WORKFLOW.matchAll(/^\s*- batch:\s*[a-z0-9-]+$/gm)].length;
  assert.equal(manifests.length, declaredBatches, "one shard manifest per batch");
  for (const shard of manifests.join(" ").split(/\s+/).filter(Boolean)) {
    const config = shardConfig(shard, WORKFLOW);
    assert.ok(config.selectors.length > 0, `${shard} resolved no selectors`);
    assert.ok(Number.isInteger(config.timeoutSeconds), `${shard} resolved no timeout`);
  }
});

test("rejects an unknown shard rather than running a default", () => {
  assert.throws(() => shardConfig("not-a-shard", WORKFLOW), /not declared/);
});

test("rejects a shell-injecting shard name", () => {
  assert.throws(() => shardConfig("a; rm -rf /", WORKFLOW), /invalid shard name/);
});
