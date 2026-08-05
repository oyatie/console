/**
 * Deterministic package → PostgreSQL reachability facet (shard) assignment.
 * Pure module: unit-tested; used by cargo_needs_postgres.sh and map checker.
 *
 * Facets (display names elsewhere):
 *   app | platform | ontology | domain
 */
export const SHARD_IDS = Object.freeze([
  "app",
  "platform",
  "ontology",
  "domain",
]);

/**
 * @param {string} packageName cargo package (e.g. console-app)
 * @returns {"app"|"platform"|"ontology"|"domain"}
 */
export function shardIdForPackage(packageName) {
  const p = String(packageName || "");
  if (p === "console-app") return "app";
  if (p.includes("ontology")) return "ontology";
  if (p.startsWith("console-platform") || p === "console-platform-db") {
    return "platform";
  }
  // platform packages also appear as console-platform-* already covered
  if (p.startsWith("console-platform-")) return "platform";
  return "domain";
}

/**
 * @param {Array<{name:string,package:string,in_workflow_postgres_job?:boolean}>} entries
 * @param {{ workflowOnly?: boolean }} [opts]
 * @returns {Record<string, string[]>} shardId → entry names
 */
export function partitionWorkflowEntries(entries, opts = {}) {
  const workflowOnly = opts.workflowOnly !== false;
  /** @type {Record<string, string[]>} */
  const out = Object.fromEntries(SHARD_IDS.map((id) => [id, []]));
  for (const e of entries || []) {
    if (workflowOnly && !e.in_workflow_postgres_job) continue;
    const id = shardIdForPackage(e.package);
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
    console.log(`partition ok (${doc.counts?.workflow_targets ?? "?"} workflow targets)`);
  } else {
    console.error("usage: node tools/ci/postgres-shard.mjs --check [map.json]");
    process.exit(2);
  }
}
