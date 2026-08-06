import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { checkNextestConfig } from "./check-nextest-config.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

test("repo nextest.toml satisfies DN-0005 P3 control", () => {
  const text = readFileSync(resolve(root, ".config/nextest.toml"), "utf8");
  assert.deepEqual(checkNextestConfig(text), []);
});

test("missing serial group fails", () => {
  assert.ok(checkNextestConfig("[profile.ci]\n").length > 0);
});

test("missing one filter fails", () => {
  const text = `
[test-groups.cluster-global]
max-threads = 1
[[profile.default.overrides]]
filter = 'test(/leave_migration_expand_contract/)'
test-group = 'cluster-global'
# pin 0.9.138
`;
  const fails = checkNextestConfig(text);
  assert.ok(fails.some((f) => f.includes("key_revision")));
});
