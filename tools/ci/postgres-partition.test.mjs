#!/usr/bin/env node
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  balanceSummary,
  packPackages,
  packageWeights,
  partitionByDuration,
  partitionFailures,
} from "./postgres-partition.mjs";

const e = (name, pkg, seconds, inWorkflow = true) => ({
  name,
  package: pkg,
  measured_seconds: seconds,
  in_workflow_postgres_job: inWorkflow,
});

test("postgres-partition", async (t) => {
  await t.test("sums measured seconds per package", () => {
    const { weights } = packageWeights([e("a", "p", 10), e("b", "p", 5), e("c", "q", 3)]);
    assert.equal(weights.get("p"), 15);
    assert.equal(weights.get("q"), 3);
  });

  await t.test("ignores entries outside the workflow set", () => {
    const { weights } = packageWeights([e("a", "p", 10), e("b", "q", 99, false)]);
    assert.equal(weights.has("q"), false);
  });

  await t.test("reports unmeasured entries instead of treating them as free", () => {
    const { weights, unmeasured } = packageWeights([
      e("good", "p", 10),
      { name: "nope", package: "q", in_workflow_postgres_job: true },
      { name: "nan", package: "r", measured_seconds: "12", in_workflow_postgres_job: true },
      { name: "neg", package: "s", measured_seconds: -1, in_workflow_postgres_job: true },
    ]);
    assert.deepEqual(unmeasured, ["nan", "neg", "nope"]);
    // still placed, at zero weight, so the partition stays complete
    for (const pkg of ["q", "r", "s"]) assert.equal(weights.get(pkg), 0);
  });

  await t.test("LPT beats the naive split it replaces", () => {
    const bins = packPackages(new Map([["a", 100], ["b", 60], ["c", 40]]), 2);
    assert.deepEqual(bins.map((b) => b.seconds).sort((x, y) => y - x), [100, 100]);
  });

  await t.test("packing is deterministic and sorted", () => {
    const w = new Map([["d", 10], ["a", 10], ["c", 10], ["b", 10]]);
    const once = packPackages(w, 2).map((b) => b.packages.join(","));
    const twice = packPackages(w, 2).map((b) => b.packages.join(","));
    assert.deepEqual(once, twice, "an unstable order would invalidate every cached target dir");
    assert.deepEqual(once, ["a,c", "b,d"]);
  });

  await t.test("conserves every package and the total weight", () => {
    const w = new Map([["a", 7], ["b", 3], ["c", 11], ["d", 2], ["e", 5]]);
    const bins = packPackages(w, 3);
    assert.deepEqual(bins.flatMap((b) => b.packages).sort(), ["a", "b", "c", "d", "e"]);
    assert.equal(bins.reduce((s, b) => s + b.seconds, 0), 28);
  });

  await t.test("rejects a nonsense shard count", () => {
    for (const bad of [0, -1, 2.5, "x", null]) {
      assert.throws(() => packPackages(new Map([["a", 1]]), bad), /positive integer/);
    }
  });

  await t.test("assigns every package to exactly one shard", () => {
    const entries = [e("a", "p", 10), e("b", "q", 5), e("c", "r", 1)];
    const { assignment, bins } = partitionByDuration(entries, 2);
    assert.equal(assignment.size, 3);
    const all = bins.flatMap((b) => b.packages);
    assert.equal(new Set(all).size, all.length, "a package in two shards would be compiled twice");
  });

  await t.test("a package never splits across shards", () => {
    // Two heavy entries of the SAME package must stay together, even when
    // splitting them would balance better -- that is the compile-duplication
    // this scheme exists to avoid.
    const { assignment } = partitionByDuration([e("a", "heavy", 100), e("b", "heavy", 100)], 2);
    assert.equal(assignment.size, 1);
    assert.equal(assignment.get("heavy"), 0);
  });

  await t.test("fails closed on an empty shard", () => {
    const failures = partitionFailures([e("a", "p", 10)], 3);
    assert.ok(
      failures.some((f) => /shard \d+ is empty/.test(f)),
      `expected an empty-shard failure, got ${JSON.stringify(failures)}`,
    );
  });

  await t.test("fails closed when weights are missing", () => {
    const failures = partitionFailures(
      [{ name: "x", package: "p", in_workflow_postgres_job: true }],
      1,
    );
    assert.ok(failures.some((f) => /no measured_seconds/.test(f)));
  });

  await t.test("fails closed on an entry with no package", () => {
    const failures = partitionFailures(
      [e("a", "p", 5), { name: "orphan", in_workflow_postgres_job: true, measured_seconds: 1 }],
      1,
    );
    assert.ok(failures.some((f) => /not partitioned/.test(f)));
  });

  await t.test("a healthy partition reports no failures", () => {
    const entries = [e("a", "p", 10), e("b", "q", 9), e("c", "r", 8), e("d", "s", 7)];
    assert.deepEqual(partitionFailures(entries, 2), []);
  });

  await t.test("balanceSummary reports the spread that matters", () => {
    const s = balanceSummary([{ seconds: 100 }, { seconds: 50 }]);
    assert.equal(s.max, 100);
    assert.equal(s.min, 50);
    assert.equal(s.spread, 2);
    assert.equal(s.total, 150);
  });

  await t.test("reproduces the measured rebalance on real observed weights", () => {
    // The five shards as actually observed on run 32115833327. Packing the same
    // work by duration must beat the 877.1s critical path it replaces.
    const observed = new Map([
      ["console-app", 552.2],
      ["console-gate-writer-ownership", 318.8],
      ["console-ontology-canonical-adapter-postgres", 177.0],
      ["console-ontology-rest", 161.4],
      ["console-platform-db", 154.0],
    ]);
    const bins = packPackages(observed, 5);
    const { max } = balanceSummary(bins);
    assert.equal(max, 552.2, "console-app alone is the floor at this shard count");
    assert.ok(max < 877.1, "must improve on the observed critical path");
  });
});
