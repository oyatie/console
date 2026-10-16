import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");
const gate = resolve(here, "check-postgres-cargo-map.mjs");
const realMapPath = resolve(root, "tools/ci/postgres-cargo-map.json");
const realMap = JSON.parse(readFileSync(realMapPath, "utf8"));

const run = (mapPath) => withNativeInventory((fixture) => {
  fixture.map = JSON.parse(readFileSync(mapPath ?? realMapPath, "utf8"));
  fixture.inventory = structuredClone(actualInventory);
  fixture.defaultMap = !mapPath;
});

const withMap = (mutate) => {
  const dir = mkdtempSync(join(tmpdir(), "pg-cargo-map-"));
  try {
    const copy = JSON.parse(JSON.stringify(realMap));
    mutate(copy);
    const path = join(dir, "map.json");
    writeFileSync(path, `${JSON.stringify(copy, null, 2)}\n`);
    return run(path);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
};

test("the committed map satisfies the gate", () => {
  const result = run();
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /postgres-cargo-map OK/);
});

test("declared counts are checked against the file, field by field", () => {
  // Each field is asserted separately: a single aggregate check would let one
  // stale field hide behind another that happened to be right.
  for (const field of [
    "mapped",
    "unmapped",
    "workflow_targets",
    "workflow_mapped",
    "workflow_missing",
  ]) {
    const result = withMap((map) => {
      map.counts[field] = map.counts[field] + 1;
    });
    assert.notEqual(result.status, 0, `counts.${field} drift must fail the gate`);
    assert.match(
      result.stderr,
      new RegExp(`counts\\.${field} declares`),
      `counts.${field} drift must name the field it caught`,
    );
  }
});

test("a missing counts block is drift, not an exemption", () => {
  // The absent-metadata path is the one that silently passes if the check reads
  // `map.counts ?? {}` and compares undefined against undefined.
  const result = withMap((map) => {
    delete map.counts;
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /counts\.mapped declares/);
});

test("adding an entry without updating counts is caught", () => {
  // The exact historical failure: entries grew, counts did not, and the stale
  // number was then printed as if it had been measured.
  const result = withMap((map) => {
    const template = map.entries.find((entry) => entry.in_workflow_postgres_job);
    map.entries.push({ ...template, name: `${template.name}-drift-fixture` });
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /counts\.mapped declares \d+ but the file contains \d+/);
});

test("native inventory counts are checked against actual native rows", () => {
  const result = withMap((map) => {
    map.native = [
      { wrapper: "app-auth-rest-browser-pg", loc: "//backend/app:console-app-itest-auth_rest-browser", reason: "required native" },
      { wrapper: "app-health-readiness-browser-pg", loc: "//backend/app:console-app-itest-health_readiness-browser", reason: "required native" },
    ];
    map.counts.native = 1;
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /counts\.native declares 1 but the file contains 2/);
});


const nativeFields = "^(buck.type|args|deps|test|labels)$";
const innerTargets = [...new Set(realMap.entries.map((entry) => entry.buck_inner))];
const nativeQuery = `set(//tools/buck: ${innerTargets.join(" ")})`;
const isolation = `postgres-map-inventory-test-${process.pid}`;
let graphResult;
let graphCleanup;
let actualInventory;
const readNativeInventory = () => {
  if (graphResult) return actualInventory;
  try {
    graphResult = spawnSync(resolve(root, "tools/buck2"), [
      "--isolation-dir", isolation, "uquery", "--json", "--output-attribute", nativeFields,
      nativeQuery,
    ], { encoding: "utf8", cwd: root, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 });
  } finally {
    graphCleanup = spawnSync(resolve(root, "tools/buck2"), [
      "--isolation-dir", isolation, "kill",
    ], { encoding: "utf8", cwd: root, timeout: 10_000 });
  }
  actualInventory = graphResult.status === 0 ? JSON.parse(graphResult.stdout) : null;
  return actualInventory;
};
const rootLabel = (label) => label.startsWith("root//") ? label : `root${label}`;

// These seven historical Cargo selectors keep their names; only native metadata changes.
const wrapperRepairs = new Map([
  ["auth-rest-lib-pg", "platform-auth-rest-unit-pg"],
  ["ontology-canonical-adapter-postgres-company-port-as-runtime-role-pg", "ontology-canonical-company-port-as-runtime-role-pg"],
  ["ontology-canonical-adapter-postgres-employment-port-as-runtime-role-pg", "ontology-canonical-employment-port-as-runtime-role-pg"],
  ["ontology-canonical-adapter-postgres-job-position-port-as-runtime-role-pg", "ontology-canonical-job-position-port-as-runtime-role-pg"],
  ["ontology-canonical-adapter-postgres-org-unit-port-as-runtime-role-pg", "ontology-canonical-org-unit-port-as-runtime-role-pg"],
  ["ontology-canonical-adapter-postgres-person-port-as-runtime-role-pg", "ontology-canonical-person-port-as-runtime-role-pg"],
  ["ontology-canonical-adapter-postgres-receipt-owner-widening-pg", "ontology-canonical-receipt-owner-widening-pg"],
]);
const missingTransports = [
  "platform-auth-webauthn-registration-deactivated-user-pg",
  "registry-adapter-postgres-branch-under-deactivated-region-pg",
  "platform-db-lib-pg",
  "platform-db-data-import-pay-period-contract-pg",
  "orgchange-preflight-zero-write-pg",
  "orgchange-adapter-postgres-org-reference-surface-pg",
  "writer-ownership-canonical-census-pg",
  "ontology-canonical-adapter-postgres-employment-reassign-as-runtime-role-pg",
  "ontology-canonical-adapter-postgres-employment-reassign-identity-as-runtime-role-pg",
  "ontology-canonical-adapter-postgres-org-unit-kinds-as-runtime-role-pg",
  "orgchange-adapter-postgres-apply-refuses-deactivated-region-pg",
  "payroll-adapter-postgres-roster-materialisation-pg",
  "identity-adapter-postgres-user_lifecycle_noop_replay-pg",
  "identity-adapter-postgres-branch_under_deactivated_region-pg",
];
const affectedNames = [...wrapperRepairs.keys(), ...missingTransports];
const entryNamed = (map, name) => {
  const matches = map.entries.filter((entry) => entry.name === name);
  assert.equal(matches.length, 1, name);
  return matches[0];
};
const fixtureLoader = "root//tools/buck/run_test_with_postgres_env.sh";
const nativeBuckDouble = `#!/usr/bin/env node
const fs = require("node:fs"), path = require("node:path");
const root = path.dirname(__dirname), args = process.argv.slice(2);
fs.appendFileSync(path.join(root, "calls.jsonl"), JSON.stringify(args) + "\\n");
const queryArgs = args[0] === "--isolation-dir" && args[1] ? args.slice(2) : args;
if (queryArgs.length === 1 && queryArgs[0] === "kill") process.exit(0);
const expectedQuery = JSON.parse(fs.readFileSync(path.join(root, "expected-query.json"), "utf8"));
if (queryArgs.length !== 5 || queryArgs[0] !== "uquery"
    || !["--json", "--output-format=json"].includes(queryArgs[1])
    || queryArgs[2] !== "--output-attribute" || queryArgs[3] !== ${JSON.stringify(nativeFields)}
    || queryArgs[4] !== expectedQuery) {
  console.error("unrecognized or incomplete inventory query (test double)");
  process.exit(64);
}
const status = JSON.parse(fs.readFileSync(path.join(root, "query-exit.json"), "utf8"));
process.stdout.write(fs.readFileSync(path.join(root, "inventory.json"), "utf8"));
if (status) console.error("inventory query refused (test double)");
process.exitCode = status;
`;

const withNativeInventory = (mutate = () => {}) => {
  readNativeInventory();
  assert.equal(graphResult.status, 0, graphResult.stderr || String(graphResult.error));
  assert.equal(graphCleanup.status, 0, graphCleanup.stderr || String(graphCleanup.error));
  const fixture = { map: structuredClone(realMap), inventory: structuredClone(actualInventory), queryExit: 0 };
  for (const [name, wrapper] of wrapperRepairs) {
    entryNamed(fixture.map, name).buck_wrapper = `//tools/buck:${wrapper}`;
  }
  // Proposed plain transports only: each inner is an actual graph-resolved Rust test.
  for (const name of missingTransports) {
    const entry = entryNamed(fixture.map, name);
    const inner = fixture.inventory[rootLabel(entry.buck_inner)];
    assert.equal(inner?.["buck.type"], "rust_test", name);
    fixture.inventory[rootLabel(entry.buck_wrapper)] = {
      "buck.type": "sh_test",
      args: [`$(location ${rootLabel(entry.buck_inner)})`],
      deps: [rootLabel(entry.buck_inner)],
      labels: [inner.labels.includes("test.unit") ? "test.unit" : "test.integration", "resource.postgres", "needs-postgres"],
      test: fixtureLoader,
    };
  }
  mutate(fixture);
  const dir = mkdtempSync(join(tmpdir(), "pg-native-map-"));
  try {
    mkdirSync(join(dir, "tools/ci"), { recursive: true });
    mkdirSync(join(dir, ".github/workflows"), { recursive: true });
    copyFileSync(gate, join(dir, "tools/ci/check-postgres-cargo-map.mjs"));
    for (const relative of ["tools/ci/postgres-partition.mjs", ".github/workflows/ci.yml"]) {
      copyFileSync(join(root, relative), join(dir, relative));
    }
    writeFileSync(join(dir, "map.json"), JSON.stringify(fixture.map));
    writeFileSync(join(dir, "tools/ci/postgres-cargo-map.json"), JSON.stringify(fixture.map));
    writeFileSync(join(dir, "expected-query.json"), JSON.stringify(`set(//tools/buck: ${[...new Set(fixture.map.entries.map((entry) => entry.buck_inner))].join(" ")})`));
    writeFileSync(join(dir, "inventory.json"), fixture.queryOutput ?? JSON.stringify(fixture.inventory));
    writeFileSync(join(dir, "query-exit.json"), JSON.stringify(fixture.queryExit));
    writeFileSync(join(dir, "calls.jsonl"), "");
    writeFileSync(join(dir, "tools/buck2"), nativeBuckDouble);
    chmodSync(join(dir, "tools/buck2"), 0o755);
    const result = spawnSync(process.execPath, [join(dir, "tools/ci/check-postgres-cargo-map.mjs"), ...(fixture.defaultMap ? [] : [join(dir, "map.json")])], {
      encoding: "utf8", cwd: dir,
    });
    const calls = readFileSync(join(dir, "calls.jsonl"), "utf8").trim();
    return { ...result, calls: calls ? calls.split("\n").map((line) => JSON.parse(line)) : [] };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
};
const assertNativeRefusal = (result, name) => {
  assert.notEqual(result.status, 0, `native binding must refuse: ${name}`);
  if (name) assert.ok(result.stderr.includes(name), result.stderr);
};

test("the inventory double requires the complete declared metadata query", () => {
  const dir = mkdtempSync(join(tmpdir(), "pg-native-query-contract-"));
  try {
    mkdirSync(join(dir, "tools"));
    const selectedMap = structuredClone(realMap), entry = entryNamed(selectedMap, affectedNames[0]);
    const wrapperLabel = rootLabel(`//tools/buck:${wrapperRepairs.get(entry.name)}`);
    const inventory = JSON.stringify({ [wrapperLabel]: { "buck.type": "sh_test", args: [`$(location ${rootLabel(entry.buck_inner)})`], deps: [rootLabel(entry.buck_inner)], test: fixtureLoader, labels: ["resource.postgres", "needs-postgres"] } });
    writeFileSync(join(dir, "inventory.json"), inventory);
    writeFileSync(join(dir, "query-exit.json"), "0");
    writeFileSync(join(dir, "expected-query.json"), JSON.stringify(nativeQuery));
    writeFileSync(join(dir, "calls.jsonl"), "");
    writeFileSync(join(dir, "tools/buck2"), nativeBuckDouble);
    chmodSync(join(dir, "tools/buck2"), 0o755);
    const complete = ["uquery", "--json", "--output-attribute", nativeFields, nativeQuery];
    for (const args of [complete, ["--isolation-dir", "query-contract-control", "uquery", "--output-format=json", "--output-attribute", nativeFields, nativeQuery]]) {
      const result = spawnSync(join(dir, "tools/buck2"), args, { encoding: "utf8", cwd: dir });
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.stdout, inventory);
    }
    for (const [fault, args] of [
      ["missing-attributes", ["uquery", "--json", nativeQuery]],
      ["only-rule-type", ["uquery", "--json", "--output-attribute", "^buck.type$", nativeQuery]],
      ["missing-loader", ["uquery", "--json", "--output-attribute", nativeFields.replace("|test", ""), nativeQuery]],
      ["missing-json", ["uquery", "--output-attribute", nativeFields, nativeQuery]],
      ["missing-wrapper-package", [...complete.slice(0, -1), `set(${innerTargets.join(" ")})`]],
      ["missing-inner", [...complete.slice(0, -1), `set(//tools/buck: ${innerTargets.slice(1).join(" ")})`]],
      ["unknown-query", [...complete.slice(0, -1), `${nativeQuery} + //backend/app:unknown-query-control`]],
    ]) {
      const result = spawnSync(join(dir, "tools/buck2"), args, { encoding: "utf8", cwd: dir });
      assert.equal(result.status, 64, fault);
      assert.equal(result.stdout, "", fault);
      assert.match(result.stderr, /unrecognized or incomplete inventory query/, fault);
    }
    entry.buck_inner = entryNamed(selectedMap, missingTransports[0]).buck_inner;
    const selectedInners = [...new Set(selectedMap.entries.map((row) => row.buck_inner))];
    assert.equal(selectedInners.length, innerTargets.length - 1);
    const selectedQuery = `set(//tools/buck: ${selectedInners.join(" ")})`;
    writeFileSync(join(dir, "expected-query.json"), JSON.stringify(selectedQuery));
    const selectedResult = spawnSync(join(dir, "tools/buck2"), [...complete.slice(0, -1), selectedQuery], { encoding: "utf8", cwd: dir });
    assert.equal(selectedResult.status, 0, selectedResult.stderr);
    const returnedWrapper = JSON.parse(selectedResult.stdout)[wrapperLabel];
    assert.notEqual(returnedWrapper.args[0], `$(location ${rootLabel(entry.buck_inner)})`, entry.name);
    assert.notDeepEqual(returnedWrapper.deps, [rootLabel(entry.buck_inner)], entry.name);
    const staleResult = spawnSync(join(dir, "tools/buck2"), complete, { encoding: "utf8", cwd: dir });
    assert.equal(staleResult.status, 64, "the original query is outside the selected mutated map");
    assert.equal(staleResult.stdout, "");
    assert.match(staleResult.stderr, /unrecognized or incomplete inventory query/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("one actual unconfigured Buck query resolves every mapped Rust inner and cleans up its own isolation", (t) => {
  readNativeInventory();
  assert.equal(graphResult.status, 0, graphResult.stderr || String(graphResult.error));
  assert.equal(graphCleanup.status, 0, graphCleanup.stderr || String(graphCleanup.error));
  for (const inner of innerTargets) assert.equal(actualInventory[rootLabel(inner)]?.["buck.type"], "rust_test", inner);
  const wrappers = Object.values(actualInventory).filter((row) => row["buck.type"] === "sh_test");
  assert.ok(wrappers.length >= 230);
  t.diagnostic(`actual Buck inventory: ${Object.keys(actualInventory).length} targets; ${innerTargets.length} Rust inners; SHA256 ${createHash("sha256").update(graphResult.stdout).digest("hex")}`);
});

test("the complete seven-label/fourteen-transport fixture preserves logical Cargo contracts", () => {
  const result = withNativeInventory(({ map }) => {
    for (let index = 0; index < realMap.entries.length; index += 1) {
      const { buck_wrapper: proposedWrapper, ...proposed } = map.entries[index];
      const { buck_wrapper: originalWrapper, ...original } = realMap.entries[index];
      assert.deepEqual(proposed, original);
    }
  });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /postgres-cargo-map OK/);
  const queries = result.calls.filter((args) => args.includes("uquery"));
  assert.equal(queries.length, 1, "the CLI must read native inventory in one batch");
  assert.ok(result.calls.every((args) => args.includes("uquery") || args.includes("kill")), "no compiler/build/test command");
  assert.ok(queries[0].includes("--json") || queries[0].includes("--output-format=json"));
  assert.equal(queries[0][queries[0].indexOf("--output-attribute") + 1], nativeFields);
  assert.equal(queries[0].at(-1), nativeQuery);
});

test("the pre-repair map is rejected for all twenty-one native bindings", () => {
  const result = withNativeInventory(({ map, inventory }) => {
    for (const name of wrapperRepairs.keys()) entryNamed(map, name).buck_wrapper = `//tools/buck:${name}`;
    for (const name of missingTransports) delete inventory[rootLabel(entryNamed(map, name).buck_wrapper)];
  });
  assertNativeRefusal(result);
  for (const name of affectedNames) assert.ok(result.stderr.includes(name), name);
});

test("every affected native binding is independently checked", async (t) => {
  for (const name of affectedNames) {
    await t.test(name, () => {
      const result = withNativeInventory(({ map }) => {
        entryNamed(map, name).buck_wrapper = "//tools/buck:no-such-native-wrapper";
      });
      assertNativeRefusal(result, name);
    });
  }
});

test("native metadata fails closed for malformed labels, cells, types, sources and ambiguous canonical labels", async (t) => {
  const name = affectedNames[0];
  for (const fault of ["missing-wrapper", "nonstring-wrapper", "unknown-cell", "wrong-inner", "wrong-location", "wrong-dependency", "wrong-rule", "unknown-wrapper-rule", "wrong-loader", "missing-pg-label", "missing-resource-label", "malformed-args", "missing-rust-inner", "wrong-inner-rule", "ambiguous-root-label"]) {
    await t.test(fault, () => {
      const result = withNativeInventory(({ map, inventory }) => {
        const entry = entryNamed(map, name), wrapper = inventory[rootLabel(entry.buck_wrapper)];
        if (fault === "missing-wrapper") delete entry.buck_wrapper;
        if (fault === "nonstring-wrapper") entry.buck_wrapper = 42;
        if (fault === "unknown-cell") entry.buck_wrapper = entry.buck_wrapper.replace("//", "other//");
        if (fault === "wrong-inner") entry.buck_inner = entryNamed(map, missingTransports[0]).buck_inner;
        if (fault === "wrong-location") wrapper.args = ["$(location root//backend/app:wrong-inner)"];
        if (fault === "wrong-dependency") wrapper.deps = [rootLabel(entryNamed(map, missingTransports[0]).buck_inner)];
        if (fault === "wrong-rule") wrapper["buck.type"] = "export_file";
        if (fault === "unknown-wrapper-rule") wrapper["buck.type"] = "unknown_native_rule";
        if (fault === "wrong-loader") wrapper.test = "root//tools/buck/test_needs_postgres.sh";
        if (fault === "missing-pg-label") wrapper.labels = wrapper.labels.filter((label) => label !== "needs-postgres");
        if (fault === "missing-resource-label") wrapper.labels = wrapper.labels.filter((label) => label !== "resource.postgres");
        if (fault === "malformed-args") wrapper.args = "$(location root//backend/app:wrong-inner)";
        if (fault === "missing-rust-inner") delete inventory[rootLabel(entry.buck_inner)];
        if (fault === "wrong-inner-rule") inventory[rootLabel(entry.buck_inner)]["buck.type"] = "rust_library";
        if (fault === "ambiguous-root-label") inventory[entry.buck_wrapper] = structuredClone(wrapper);
      });
      assertNativeRefusal(result, name);
    });
  }
});

test("known canonical root labels and harmless explicit same-inner aliases remain valid", () => {
  const result = withNativeInventory(({ map, inventory }) => {
    const entry = entryNamed(map, affectedNames[0]);
    inventory["root//tools/buck:explicit-same-inner-alias"] = structuredClone(inventory[rootLabel(entry.buck_wrapper)]);
    entry.buck_wrapper = "//tools/buck:explicit-same-inner-alias";
    for (const [label, row] of Object.entries(inventory)) {
      delete inventory[label];
      inventory[label.replace(/^root(?=\/\/)/, "")] = {
        ...row,
        ...(row.args ? { args: row.args.map((arg) => arg.replace(/root(?=\/\/)/g, "")) } : {}),
        ...(row.deps ? { deps: row.deps.map((dep) => dep.replace(/^root(?=\/\/)/, "")) } : {}),
        ...(row.test ? { test: row.test.replace(/^root(?=\/\/)/, "") } : {}),
      };
    }
  });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.calls.filter((args) => args.includes("uquery")).length, 1);
});

test("failed or malformed inventory cannot fall back to count/workflow success", async (t) => {
  for (const fault of ["query-refused", "malformed-json", "array-instead-of-object", "null-row", "unknown-cell-target"]) {
    await t.test(fault, () => {
      const result = withNativeInventory((fixture) => {
        if (fault === "query-refused") fixture.queryExit = 17;
        if (fault === "malformed-json") fixture.queryOutput = "{";
        if (fault === "array-instead-of-object") fixture.queryOutput = "[]";
        if (fault === "null-row") fixture.inventory[rootLabel(entryNamed(fixture.map, affectedNames[0]).buck_wrapper)] = null;
        if (fault === "unknown-cell-target") {
          const label = rootLabel(entryNamed(fixture.map, affectedNames[0]).buck_wrapper);
          fixture.inventory[label.replace(/^root/, "other")] = fixture.inventory[label];
          delete fixture.inventory[label];
        }
      });
      assertNativeRefusal(result);
    });
  }
});
