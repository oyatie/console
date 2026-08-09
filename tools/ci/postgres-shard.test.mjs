#!/usr/bin/env node
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import {
  SHARD_IDS,
  packageFamily,
  shardIdForPackage,
  domainSubshardByPackage,
  partitionWorkflowEntries,
  partitionFailures,
} from "./postgres-shard.mjs";

test("packageFamily package rules", () => {
  assert.equal(packageFamily("console-app"), "app");
  assert.equal(packageFamily("console-ontology-adapter-postgres"), "ontology");
  assert.equal(packageFamily("console-ontology-rest"), "ontology");
  assert.equal(packageFamily("console-platform-db"), "platform");
  assert.equal(packageFamily("console-platform-auth-rest"), "platform");
  assert.equal(packageFamily("console-docs-rest"), "domain");
  assert.equal(packageFamily("console-workorder-rest"), "domain");
});

test("partition is disjoint and complete on synthetic set", () => {
  const entries = [
    { name: "a", package: "console-app", in_workflow_postgres_job: true },
    { name: "o", package: "console-ontology-rest", in_workflow_postgres_job: true },
    { name: "p", package: "console-platform-db", in_workflow_postgres_job: true },
    { name: "d1", package: "console-docs-rest", in_workflow_postgres_job: true },
    { name: "d2", package: "console-workorder-rest", in_workflow_postgres_job: true },
    { name: "skip", package: "console-app", in_workflow_postgres_job: false },
  ];
  const parts = partitionWorkflowEntries(entries);
  assert.deepEqual(parts.app, ["a"]);
  assert.deepEqual(parts.ontology, ["o"]);
  assert.deepEqual(parts.platform, ["p"]);
  // two domain packages → greedy assigns one to each half when counts equal by name order
  assert.equal(parts["domain-a"].length + parts["domain-b"].length, 2);
  assert.equal(partitionFailures(entries).length, 0);
});

test("domain greedy split balances entry counts on synthetic uneven packages", () => {
  const entries = [
    { name: "h1", package: "console-heavy", in_workflow_postgres_job: true },
    { name: "h2", package: "console-heavy", in_workflow_postgres_job: true },
    { name: "h3", package: "console-heavy", in_workflow_postgres_job: true },
    { name: "l1", package: "console-light", in_workflow_postgres_job: true },
  ];
  // mark as domain family only
  const map = domainSubshardByPackage(entries);
  assert.equal(map.get("console-heavy"), "domain-a"); // first, heavier
  assert.equal(map.get("console-light"), "domain-b");
  const parts = partitionWorkflowEntries(entries);
  assert.equal(parts["domain-a"].length, 3);
  assert.equal(parts["domain-b"].length, 1);
});

test("real postgres-cargo-map workflow set partitions cleanly with balanced domain halves", () => {
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
  for (const id of SHARD_IDS) {
    assert.ok(parts[id].length > 0, `shard ${id} empty`);
  }
  // S2: domain halves should be near-equal (within 5 entries)
  const da = parts["domain-a"].length;
  const db = parts["domain-b"].length;
  assert.ok(Math.abs(da - db) <= 5, `domain halves unbalanced: ${da} vs ${db}`);
  // Inventory tripwire, not a safety property: the balance assertion above is the invariant.
  // 78 -> 80 when identity-rest org_setup and production-rest production_lifecycle_http were
  // repaired and joined the workflow set (P1 dark-test wiring, console-5lh.6).
  // 80 -> 81 when orgchange-preflight-zero-write-pg joined it: the P3 proof that org-change
  // preflight persists nothing, which needs a real database because it works by fingerprinting
  // every base table either side of the call (console-tai / leaf 52a003cba).
  assert.equal(da + db, 81, `expected 81 domain entries, got ${da + db}`);
});

test("shardIdForPackage with domain map resolves domain packages", () => {
  const map = new Map([
    ["console-docs-rest", "domain-b"],
    ["console-workorder-rest", "domain-a"],
  ]);
  assert.equal(shardIdForPackage("console-docs-rest", map), "domain-b");
  assert.equal(shardIdForPackage("console-workorder-rest", map), "domain-a");
  assert.equal(shardIdForPackage("console-app", map), "app");
});
