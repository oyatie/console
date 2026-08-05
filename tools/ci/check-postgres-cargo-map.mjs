#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const map = JSON.parse(readFileSync(resolve(root, "tools/ci/postgres-cargo-map.json"), "utf8"));
const wf = readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8");
const start = wf.indexOf("postgres-domain-reachability:");
const end = wf.indexOf("\n  company-conformance:", start);
const block = wf.slice(start, end);
const failures = [];

const usesCargo = /tools\/ci\/cargo_needs_postgres\.sh\s+--workflow-only/.test(block);
const re = /\/\/tools\/buck:([a-zA-Z0-9_-]+)/g;
const needed = new Set();
let m;
while ((m = re.exec(block))) needed.add(m[1]);
const mapped = new Set(
  (map.entries ?? []).filter((e) => e.in_workflow_postgres_job).map((e) => e.name),
);

if (usesCargo) {
  if (needed.size > 0) {
    failures.push(
      `cargo harness job must not still list Buck wrappers (found ${needed.size}); map is the source of truth`,
    );
  }
  if (mapped.size < 180) {
    failures.push(`postgres-cargo-map in_workflow_postgres_job count ${mapped.size} < 180`);
  }
  for (const e of map.entries ?? []) {
    if (!e.in_workflow_postgres_job) continue;
    if (!Array.isArray(e.cargo_argv) || e.cargo_argv[0] !== "cargo") {
      failures.push(`map entry ${e.name} missing cargo argv`);
    }
  }
  for (const u of map.unmapped ?? []) {
    // unmapped wrappers must not claim workflow membership
    if (mapped.has(u.wrapper) || mapped.has(String(u.wrapper || "").replace(/^.*:/, ""))) {
      failures.push(`unmapped wrapper also marked workflow: ${u.wrapper}`);
    }
  }
} else {
  const missing = [...needed].filter((n) => !mapped.has(n)).sort();
  if (missing.length) failures.push(`workflow wrappers missing from map: ${missing.join(", ")}`);
  for (const u of map.unmapped ?? []) {
    const name = String(u.wrapper || "").replace(/^.*:/, "");
    if (needed.has(name) || needed.has(u.wrapper)) {
      failures.push(`unmapped but required by workflow: ${u.wrapper}`);
    }
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log(
  usesCargo
    ? `postgres-cargo-map OK (cargo harness; ${mapped.size} workflow entries)`
    : `postgres-cargo-map OK (workflow ${needed.size} wrappers fully mapped)`,
);
