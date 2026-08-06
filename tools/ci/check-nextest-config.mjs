#!/usr/bin/env node
/**
 * DN-0005 P3: fail-closed presence of nextest serial-group config.
 * Does not run nextest — only asserts the durable process control exists.
 */
import { readFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const path = resolve(root, ".config/nextest.toml");

const REQUIRED_FILTERS = [
  "leave_migration_expand_contract",
  "key_revision_migration_upgrade",
  "attendance_console_migration_contract",
  "apalis_adapter",
  "apalis_schema_contract",
];

export function checkNextestConfig(tomlText) {
  const failures = [];
  if (!tomlText.includes("[test-groups.cluster-global]")) {
    failures.push("missing [test-groups.cluster-global]");
  }
  if (!/max-threads\s*=\s*1/.test(tomlText)) {
    failures.push("cluster-global must set max-threads = 1");
  }
  if (!tomlText.includes("test-group = 'cluster-global'") && !tomlText.includes('test-group = "cluster-global"')) {
    failures.push("override must assign test-group = cluster-global");
  }
  for (const name of REQUIRED_FILTERS) {
    if (!tomlText.includes(name)) {
      failures.push(`filter missing serial suite marker: ${name}`);
    }
  }
  if (!tomlText.includes("0.9.138")) {
    failures.push("pin cargo-nextest 0.9.138 must be documented in nextest.toml comments");
  }
  return failures;
}

const isMain =
  process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1]);

if (isMain) {
  if (!existsSync(path)) {
    console.error(`missing ${path}`);
    process.exit(1);
  }
  const failures = checkNextestConfig(readFileSync(path, "utf8"));
  if (failures.length) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  console.log("nextest-config OK (cluster-global serial group + pin 0.9.138 documented)");
}
