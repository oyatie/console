import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const root = new URL("..", import.meta.url).pathname;
const script = join(root, "scripts/generate-trivy-dev-codegen-exceptions.mjs");
const canonical = JSON.parse(readFileSync(join(root, "security/node-audit-exceptions.json"), "utf8"));
const expiry = new Date(Date.now() + 14 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
function fixture() {
  const dir = mkdtempSync(join(tmpdir(), "trivy-exception-parity-"));
  const registry = { ...canonical, entries: canonical.entries.map((entry) => ({ ...entry, expires_on: expiry })) };
  const registryPath = join(dir, "registry.json");
  const outputPath = join(dir, "exceptions.yaml");
  writeFileSync(registryPath, JSON.stringify(registry));
  execFileSync(process.execPath, [script, "--write", "--registry", registryPath, "--output", outputPath], { cwd: root, stdio: "pipe" });
  return { registry, registryPath, outputPath };
}
function check(registryPath, outputPath) {
  return () => execFileSync(process.execPath, [script, "--check", "--registry", registryPath, "--output", outputPath], { cwd: root, encoding: "utf8", stdio: "pipe" });
}
test("Trivy YAML is the exact canonical JSON projection", () => {
  const { registryPath, outputPath } = fixture();
  assert.match(check(registryPath, outputPath)(), /PARITY_PASS/);
});
test("Trivy parity rejects a rogue YAML entry", () => {
  const { registryPath, outputPath } = fixture();
  writeFileSync(outputPath, `${readFileSync(outputPath, "utf8")}  - id: GHSA-aaaa-bbbb-cccc\n`);
  assert.throws(check(registryPath, outputPath), /differs from the canonical JSON projection/);
});
test("Trivy parity rejects widened or missing package constraints", () => {
  const { registryPath, outputPath } = fixture();
  const yaml = readFileSync(outputPath, "utf8");
  writeFileSync(outputPath, yaml.replace("pkg:npm/brace-expansion@2.1.2", "pkg:npm/brace-expansion@*"));
  assert.throws(check(registryPath, outputPath), /differs from the canonical JSON projection/);
  const second = fixture();
  writeFileSync(second.outputPath, readFileSync(second.outputPath, "utf8").replace("    paths:\n      - node_modules/@redocly/openapi-core/node_modules/brace-expansion\n", ""));
  assert.throws(check(second.registryPath, second.outputPath), /differs from the canonical JSON projection/);
});
test("Trivy parity rejects expired canonical entries and expiry mismatch", () => {
  const expired = fixture();
  expired.registry.entries[0].expires_on = "2000-01-01";
  writeFileSync(expired.registryPath, JSON.stringify(expired.registry));
  assert.throws(check(expired.registryPath, expired.outputPath), /expired/);
  const mismatch = fixture();
  writeFileSync(mismatch.outputPath, readFileSync(mismatch.outputPath, "utf8").replace(expiry, "2000-01-01"));
  assert.throws(check(mismatch.registryPath, mismatch.outputPath), /differs from the canonical JSON projection/);
});
