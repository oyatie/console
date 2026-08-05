#!/usr/bin/env node
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import {
  SHARD_IDS,
  shardIdForPackage,
  partitionWorkflowEntries,
  partitionFailures,
} from "./postgres-shard.mjs";

test("shardIdForPackage package rules", () => {
  assert.equal(shardIdForPackage("console-app"), "app");
  assert.equal(shardIdForPackage("console-ontology-adapter-postgres"), "ontology");
  assert.equal(shardIdForPackage("console-ontology-rest"), "ontology");
  assert.equal(shardIdForPackage("console-platform-db"), "platform");
  assert.equal(shardIdForPackage("console-platform-auth-rest"), "platform");
  assert.equal(shardIdForPackage("console-docs-rest"), "domain");
  assert.equal(shardIdForPackage("console-workorder-rest"), "domain");
});

test("partition is disjoint and complete on synthetic set", () => {
  const entries = [
    { name: "a", package: "console-app", in_workflow_postgres_job: true },
    { name: "o", package: "console-ontology-rest", in_workflow_postgres_job: true },
    { name: "p", package: "console-platform-db", in_workflow_postgres_job: true },
    { name: "d", package: "console-docs-rest", in_workflow_postgres_job: true },
    { name: "skip", package: "console-app", in_workflow_postgres_job: false },
  ];
  const parts = partitionWorkflowEntries(entries);
  assert.deepEqual(parts.app, ["a"]);
  assert.deepEqual(parts.ontology, ["o"]);
  assert.deepEqual(parts.platform, ["p"]);
  assert.deepEqual(parts.domain, ["d"]);
  assert.equal(partitionFailures(entries).length, 0);
});

test("real postgres-cargo-map workflow set partitions cleanly", () => {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const doc = JSON.parse(
    readFileSync(resolve(root, "tools/ci/postgres-cargo-map.json"), "utf8"),
  );
  const fails = partitionFailures(doc.entries);
  assert.deepEqual(fails, []);
  const parts = partitionWorkflowEntries(doc.entries);
  const n = SHARD_IDS.reduce((s, id) => s + parts[id].length, 0);
  const workflow = doc.entries.filter((e) => e.in_workflow_postgres_job);
  assert.equal(n, workflow.length);
  assert.ok(n >= 180, `expected >=180 workflow targets, got ${n}`);
  // every facet non-empty for the real map (design assumption)
  for (const id of SHARD_IDS) {
    assert.ok(parts[id].length > 0, `shard ${id} empty`);
  }
});
