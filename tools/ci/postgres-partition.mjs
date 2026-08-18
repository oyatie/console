#!/usr/bin/env node
/**
 * Duration-weighted PostgreSQL shard partitioning.
 *
 * Replaces the semantic-family scheme in postgres-shard.mjs, where a shard was
 * "the app package", "the ontology packages", "the platform packages" and the
 * leftover domain bag split in two by ENTRY COUNT. Families are not balanced,
 * and entry count is not a cost. Measured 2026-08-18 (run 32115833327, 209
 * invocations, 3299.4s total):
 *
 *   domain-b 877.1s  domain-a 738.6s  platform 644.5s  app 552.2s  ontology 487.1s
 *
 * a 1.80x spread whose slowest shard is the whole critical path. LPT-packing the
 * same 61 packages by measured seconds gives 660.3s across the same five shards
 * -- 216.8s off the critical path with no new jobs and no extra compile.
 *
 * WHY PACKAGES AND NOT TEST BINARIES. Binary granularity looks better on test
 * time (413.5s at 8 shards vs 552.2s) but a package split across shards must be
 * COMPILED in each of them. Measured on the same data, binary granularity takes
 * the compile surface from 61 package-builds to 131 -- 2.1x -- which at 5-15s
 * per package plausibly cancels the entire gain. Packages are therefore the
 * atom, and the residual floor is the heaviest single package (console-app,
 * 552.2s). That floor is broken by parallelising WITHIN a package (nextest),
 * which costs no extra compile, not by splitting it across shards.
 */

/**
 * Sum measured seconds per package.
 *
 * Entries with no measurement contribute 0 and are reported separately: a new
 * target defaulting to zero lands in the lightest bin, which is harmless, but
 * silently treating an unmeasured package as free is how the entry-count scheme
 * drifted in the first place.
 *
 * @param {Array<{package?:string, measured_seconds?:number, in_workflow_postgres_job?:boolean}>} entries
 * @returns {{weights: Map<string, number>, unmeasured: string[]}}
 */
export function packageWeights(entries) {
  const weights = new Map();
  const unmeasured = new Set();
  for (const entry of entries ?? []) {
    if (!entry?.in_workflow_postgres_job) continue;
    const pkg = String(entry.package ?? "");
    if (!pkg) continue;
    const seconds = entry.measured_seconds;
    if (typeof seconds !== "number" || !Number.isFinite(seconds) || seconds < 0) {
      unmeasured.add(String(entry.name ?? pkg));
      weights.set(pkg, weights.get(pkg) ?? 0);
      continue;
    }
    weights.set(pkg, (weights.get(pkg) ?? 0) + seconds);
  }
  return { weights, unmeasured: [...unmeasured].sort() };
}

/**
 * Longest-processing-time bin packing: heaviest first, into the lightest bin.
 * Within 4/3 of optimal, far inside CI timing noise.
 *
 * Determinism is load-bearing twice over: the assignment decides which packages
 * a shard compiles, so a reshuffle on equal input invalidates every cached
 * target dir, and the map is committed so an unstable sort would churn the diff.
 * Ties therefore break on package name, and the lightest bin ties to the lowest
 * index.
 *
 * @param {Map<string, number>} weights
 * @param {number} shardCount
 * @returns {Array<{index:number, packages:string[], seconds:number}>}
 */
export function packPackages(weights, shardCount) {
  const count = Number(shardCount);
  if (!Number.isInteger(count) || count < 1) {
    throw new Error(`shard count must be a positive integer, got ${shardCount}`);
  }
  const ordered = [...weights.entries()]
    .sort((a, b) => (b[1] !== a[1] ? b[1] - a[1] : a[0].localeCompare(b[0])));
  const bins = Array.from({ length: count }, (_, index) => ({
    index,
    packages: [],
    seconds: 0,
  }));
  for (const [pkg, seconds] of ordered) {
    let target = bins[0];
    for (const bin of bins) {
      if (bin.seconds < target.seconds) target = bin;
    }
    target.packages.push(pkg);
    target.seconds += seconds;
  }
  for (const bin of bins) bin.packages.sort();
  return bins;
}

/**
 * Assign every workflow entry to a shard index via its package.
 *
 * @param {Array<object>} entries
 * @param {number} shardCount
 * @returns {{assignment: Map<string, number>, bins: Array<object>, unmeasured: string[]}}
 *   assignment maps package -> shard index.
 */
export function partitionByDuration(entries, shardCount) {
  const { weights, unmeasured } = packageWeights(entries);
  const bins = packPackages(weights, shardCount);
  const assignment = new Map();
  for (const bin of bins) {
    for (const pkg of bin.packages) assignment.set(pkg, bin.index);
  }
  return { assignment, bins, unmeasured };
}

/**
 * Fail-closed invariants. Mirrors postgres-shard.mjs's partitionFailures so the
 * replacement cannot be weaker than what it replaces.
 *
 * @param {Array<object>} entries
 * @param {number} shardCount
 * @returns {string[]} failure messages (empty = ok)
 */
export function partitionFailures(entries, shardCount) {
  const failures = [];
  const workflow = (entries ?? []).filter((e) => e?.in_workflow_postgres_job);
  let result;
  try {
    result = partitionByDuration(workflow, shardCount);
  } catch (error) {
    return [`partition: ${error.message}`];
  }
  const { assignment, bins, unmeasured } = result;

  // Every workflow entry must land in exactly one shard.
  let placed = 0;
  for (const entry of workflow) {
    const pkg = String(entry.package ?? "");
    if (!assignment.has(pkg)) {
      failures.push(`workflow entry not partitioned: ${entry.name} (package ${pkg || "<none>"})`);
      continue;
    }
    placed += 1;
  }
  if (placed !== workflow.length) {
    failures.push(`partitioned ${placed} entries but the workflow set has ${workflow.length}`);
  }

  // A package must not appear in two shards -- that is the compile-duplication
  // this scheme exists to avoid, and it would be invisible in the timings.
  const seen = new Map();
  for (const bin of bins) {
    for (const pkg of bin.packages) {
      if (seen.has(pkg)) {
        failures.push(`package ${pkg} in both shard ${seen.get(pkg)} and ${bin.index}`);
      }
      seen.set(pkg, bin.index);
    }
  }

  // An empty shard means a job that boots PostgreSQL, compiles, and tests
  // nothing -- pure fixed cost on the critical path.
  for (const bin of bins) {
    if (bin.packages.length === 0) {
      failures.push(`shard ${bin.index} is empty; it would pay full fixed cost for no tests`);
    }
  }

  if (unmeasured.length) {
    failures.push(
      `${unmeasured.length} workflow entr(ies) have no measured_seconds; `
      + `regenerate weights before trusting the balance:\n  ${unmeasured.slice(0, 10).join("\n  ")}`,
    );
  }
  return failures;
}

/**
 * @param {Array<object>} bins
 * @returns {{max:number, min:number, spread:number, total:number}}
 */
export function balanceSummary(bins) {
  const seconds = bins.map((b) => b.seconds);
  const max = Math.max(...seconds);
  const min = Math.min(...seconds);
  return {
    max,
    min,
    spread: min > 0 ? max / min : Infinity,
    total: seconds.reduce((a, b) => a + b, 0),
  };
}

const isMain = process.argv[1] && process.argv[1].endsWith("postgres-partition.mjs");
if (isMain) {
  const { readFileSync } = await import("node:fs");
  const { resolve, dirname } = await import("node:path");
  const { fileURLToPath } = await import("node:url");
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const args = process.argv.slice(2);
  const shardArg = args.find((a) => a.startsWith("--shards="));
  const shardCount = shardArg ? Number(shardArg.slice("--shards=".length)) : 5;
  const mapPath = args.find((a) => !a.startsWith("--"))
    ? resolve(args.find((a) => !a.startsWith("--")))
    : resolve(root, "tools/ci/postgres-cargo-map.json");
  const doc = JSON.parse(readFileSync(mapPath, "utf8"));
  const failures = partitionFailures(doc.entries ?? [], shardCount);
  if (failures.length) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  const { bins } = partitionByDuration(
    (doc.entries ?? []).filter((e) => e.in_workflow_postgres_job),
    shardCount,
  );
  for (const bin of bins) {
    console.log(`shard-${bin.index}\t${bin.seconds.toFixed(1)}s\t${bin.packages.length} packages`);
  }
  const { max, spread, total } = balanceSummary(bins);
  console.log(
    `partition ok (${shardCount} shards, max ${max.toFixed(1)}s, `
    + `spread ${spread.toFixed(2)}x, perfect ${(total / shardCount).toFixed(1)}s)`,
  );
}
