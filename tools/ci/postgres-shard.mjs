/**
 * Deterministic package → PostgreSQL reachability facet (shard) assignment.
 * Pure module: unit-tested; used by cargo_needs_postgres.sh and map checker.
 *
 * Facets (display names elsewhere):
 *   app | platform | ontology | domain-a | domain-b
 *
 * S2: split former `domain` bag (78 targets) into two balanced sub-facets by
 * package entry count (greedy bin-pack). Load-bearing aggregator display name
 * is unchanged.
 */
export const SHARD_IDS = Object.freeze([
  "app",
  "platform",
  "ontology",
  "domain-a",
  "domain-b",
]);

/**
 * Top-level family before domain subshard assignment.
 * @param {string} packageName
 * @returns {"app"|"platform"|"ontology"|"domain"}
 */
export function packageFamily(packageName) {
  const p = String(packageName || "");
  if (p === "console-app") return "app";
  if (p.includes("ontology")) return "ontology";
  if (p.startsWith("console-platform") || p === "console-platform-db") {
    return "platform";
  }
  return "domain";
}

/**
 * Greedy balance of domain packages by workflow entry count.
 * Stable: sort by (-count, name); assign each package to the lighter bin.
 *
 * @param {Array<{package:string,in_workflow_postgres_job?:boolean}>} entries
 * @returns {Map<string, "domain-a"|"domain-b">}
 */
export function domainSubshardByPackage(entries) {
  /** @type {Map<string, number>} */
  const counts = new Map();
  for (const e of entries || []) {
    if (!e.in_workflow_postgres_job) continue;
    if (packageFamily(e.package) !== "domain") continue;
    const p = e.package;
    counts.set(p, (counts.get(p) || 0) + 1);
  }
  const ordered = [...counts.entries()].sort((a, b) => {
    if (b[1] !== a[1]) return b[1] - a[1];
    return a[0].localeCompare(b[0]);
  });
  /** @type {Map<string, "domain-a"|"domain-b">} */
  const out = new Map();
  let countA = 0;
  let countB = 0;
  for (const [pkg, n] of ordered) {
    if (countA <= countB) {
      out.set(pkg, "domain-a");
      countA += n;
    } else {
      out.set(pkg, "domain-b");
      countB += n;
    }
  }
  return out;
}

/**
 * @param {string} packageName cargo package (e.g. console-app)
 * @param {Map<string, "domain-a"|"domain-b"> | null | undefined} domainMap
 *   Required for domain packages when partitioning a map; if omitted, domain
 *   packages map to domain-a (tests of non-domain families only).
 * @returns {"app"|"platform"|"ontology"|"domain-a"|"domain-b"}
 */
export function shardIdForPackage(packageName, domainMap = null) {
  const family = packageFamily(packageName);
  if (family !== "domain") return family;
  if (domainMap && domainMap.has(packageName)) {
    return domainMap.get(packageName);
  }
  // Deterministic fallback without map context (single-package probes).
  return "domain-a";
}

/**
 * @param {Array<{name:string,package:string,in_workflow_postgres_job?:boolean}>} entries
 * @param {{ workflowOnly?: boolean }} [opts]
 * @returns {Record<string, string[]>} shardId → entry names
 */
export function partitionWorkflowEntries(entries, opts = {}) {
  const workflowOnly = opts.workflowOnly !== false;
  const domainMap = domainSubshardByPackage(entries);
  /** @type {Record<string, string[]>} */
  const out = Object.fromEntries(SHARD_IDS.map((id) => [id, []]));
  for (const e of entries || []) {
    if (workflowOnly && !e.in_workflow_postgres_job) continue;
    const id = shardIdForPackage(e.package, domainMap);
    out[id].push(e.name);
  }
  return out;
}

/**
 * Fail-closed partition invariants for the workflow set.
 * @param {Array<object>} entries
 * @returns {string[]} failure messages (empty = ok)
 */
export function partitionFailures(entries) {
  const failures = [];
  const workflow = (entries || []).filter((e) => e.in_workflow_postgres_job);
  const parts = partitionWorkflowEntries(workflow, { workflowOnly: false });
  const seen = new Map();
  let total = 0;
  for (const id of SHARD_IDS) {
    for (const name of parts[id]) {
      total += 1;
      if (seen.has(name)) {
        failures.push(
          `entry ${name} in both ${seen.get(name)} and ${id}`,
        );
      }
      seen.set(name, id);
    }
  }
  if (total !== workflow.length) {
    failures.push(
      `partition size ${total} != workflow set ${workflow.length}`,
    );
  }
  for (const e of workflow) {
    if (!seen.has(e.name)) {
      failures.push(`workflow entry not partitioned: ${e.name}`);
    }
  }
  // S2: both domain halves non-empty when domain work exists
  const domainTotal = parts["domain-a"].length + parts["domain-b"].length;
  if (domainTotal > 0) {
    if (parts["domain-a"].length === 0) {
      failures.push("domain-a empty while domain packages exist");
    }
    if (parts["domain-b"].length === 0) {
      failures.push("domain-b empty while domain packages exist");
    }
  }
  return failures;
}

// CLI: node tools/ci/postgres-shard.mjs --check [map path]
const isMain = process.argv[1] && process.argv[1].endsWith("postgres-shard.mjs");
if (isMain) {
  const args = process.argv.slice(2);
  if (args[0] === "--check") {
    const { readFileSync } = await import("node:fs");
    const { resolve, dirname } = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
    const mapPath = args[1]
      ? resolve(args[1])
      : resolve(root, "tools/ci/postgres-cargo-map.json");
    const doc = JSON.parse(readFileSync(mapPath, "utf8"));
    const fails = partitionFailures(doc.entries || []);
    if (fails.length) {
      console.error(fails.join("\n"));
      process.exit(1);
    }
    const parts = partitionWorkflowEntries(doc.entries || []);
    for (const id of SHARD_IDS) {
      console.log(`${id}\t${parts[id].length}`);
    }
    // Report what this run partitioned, not what the file claims it would:
    // reading `doc.counts` here printed `207 workflow targets` while the five
    // shard lines directly above summed to 209. A success line that restates
    // stored metadata is not a measurement.
    const partitioned = SHARD_IDS.reduce((total, id) => total + parts[id].length, 0);
    console.log(`partition ok (${partitioned} workflow targets)`);
  } else {
    console.error("usage: node tools/ci/postgres-shard.mjs --check [map.json]");
    process.exit(2);
  }
}
