import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const root = new URL("..", import.meta.url).pathname;
const gate = join(root, "scripts/check-node-audit-exceptions.mjs");
const registry = JSON.parse(readFileSync(join(root, "security/node-audit-exceptions.json"), "utf8"));
const expiry = new Date(Date.now() + 14 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
const report = {
  auditReportVersion: 2,
  metadata: { vulnerabilities: { high: 1, critical: 0 } },
  vulnerabilities: {
    "brace-expansion": {
      name: "brace-expansion", severity: "high", nodes: registry.entries.map((entry) => entry.path),
      via: [{ url: "https://github.com/advisories/GHSA-mh99-v99m-4gvg" }],
    },
  },
};
function run(mode, registryValue = registry, reportValue = report, preserveExpiry = false) {
  const dir = mkdtempSync(join(tmpdir(), "node-audit-exception-"));
  const registryPath = join(dir, "registry.json");
  const reportPath = join(dir, "audit.json");
  writeFileSync(registryPath, JSON.stringify({ ...registryValue, entries: registryValue.entries.map((entry) => ({ ...entry, expires_on: preserveExpiry ? entry.expires_on : expiry })) }));
  writeFileSync(reportPath, JSON.stringify(reportValue));
  return () => execFileSync(process.execPath, [gate, "--mode", mode, "--registry", registryPath, "--audit-report", reportPath], { cwd: root, encoding: "utf8", stdio: "pipe" });
}
test("dev/codegen audit accepts only exact live exception entries", () => {
  assert.match(run("dev-codegen")(), /NODE_AUDIT_DEV_CODEGEN_PASS/);
});
test("dev/codegen audit rejects a new or path-mismatched high finding", () => {
  const changed = structuredClone(report);
  changed.vulnerabilities.postcss = { name: "postcss", severity: "high", nodes: ["node_modules/postcss"], via: [{ url: "https://github.com/advisories/GHSA-r28c-9q8g-f849" }] };
  assert.throws(run("dev-codegen", registry, changed), /unmatched high finding/);
});
test("dev/codegen audit rejects expired and stale exceptions", () => {
  const expired = structuredClone(registry);
  expired.entries[0].expires_on = "2000-01-01";
  assert.throws(run("dev-codegen", expired, report, true), /expired/);
  assert.throws(run("dev-codegen", { ...registry, entries: registry.entries.slice(0, 1) }), /unmatched high finding|stale exception/);
});
test("production audit forbids every high exception", () => {
  assert.throws(run("production"), /exceptions are forbidden/);
});
test("audit execution errors cannot be treated as a clean report", () => {
  assert.throws(run("production", registry, { error: { summary: "registry unavailable" } }), /report is incomplete/);
});
