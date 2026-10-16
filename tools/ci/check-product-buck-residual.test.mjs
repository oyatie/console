import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import {
  MAX_PRODUCT_BUCK_RESIDUAL,
  classifyResidual,
  extractBuckWrappersFromCiYml,
  extractBuckWrappersFromPgFacetBlock,
  residualFailures,
} from "./check-product-buck-residual.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

test("extracts sorted unique buck wrappers", () => {
  const wf = `
    //tools/buck:b-second
    //tools/buck:a-first
    //tools/buck:a-first
  `;
  assert.deepEqual(extractBuckWrappersFromCiYml(wf), ["a-first", "b-second"]);
});

test("PG facet block with cargo harness has zero buck wrappers on real ci.yml", () => {
  const wf = readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8");
  assert.deepEqual(extractBuckWrappersFromPgFacetBlock(wf), []);
});

test("classifyResidual splits mapped / unmapped / unknown", () => {
  const map = {
    entries: [{ name: "mapped-one", cargo_argv: ["cargo"] }],
    unmapped: [{ wrapper: "dark-one", reason: "missing tests/x.rs" }],
  };
  const r = classifyResidual(["mapped-one", "dark-one", "ghost"], map);
  assert.deepEqual(r.residual_mapped, ["mapped-one"]);
  assert.deepEqual(r.residual_unmapped, ["dark-one"]);
  assert.deepEqual(r.unknown, ["ghost"]);
});

test("real ci.yml residual is within ceiling and fully classified", () => {
  const map = JSON.parse(readFileSync(resolve(root, "tools/ci/postgres-cargo-map.json"), "utf8"));
  const wf = readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8");
  const { failures, residual } = residualFailures(wf, map);
  assert.deepEqual(failures, []);
  assert.ok(residual.length <= MAX_PRODUCT_BUCK_RESIDUAL);
  assert.ok(residual.length >= 1, "expected residual Buck product surface until cutover");
});

test("unknown residual fails closed", () => {
  const map = { entries: [], unmapped: [] };
  const wf = "run: tools/buck/test_needs_postgres.sh //tools/buck:brand-new-wrapper\n";
  const { failures } = residualFailures(wf, map, { maxResidual: 10 });
  assert.ok(failures.some((f) => f.includes("brand-new-wrapper")));
});

test("residual above ceiling fails", () => {
  const map = {
    entries: [{ name: "a" }, { name: "b" }],
    unmapped: [],
  };
  const wf = "//tools/buck:a\n//tools/buck:b\n";
  const { failures } = residualFailures(wf, map, { maxResidual: 1 });
  assert.ok(failures.some((f) => f.includes("exceeds ceiling")));
});

const nativeBrowserRows = [
  { wrapper: "app-auth-rest-browser-pg", loc: "//backend/app:console-app-itest-auth_rest-browser", reason: "Required native browser producer" },
  { wrapper: "app-health-readiness-browser-pg", loc: "//backend/app:console-app-itest-health_readiness-browser", reason: "Required native browser producer" },
];
const nativeWorkflow = nativeBrowserRows.map((row) => `//tools/buck:${row.wrapper}`).join("\n");

test("approved native browsers are classified separately from the legacy ceiling", () => {
  const map = { entries: [{ name: "legacy" }], unmapped: [], native: structuredClone(nativeBrowserRows) };
  const classified = classifyResidual(["legacy", ...nativeBrowserRows.map((row) => row.wrapper)], map);
  assert.deepEqual(classified.native, nativeBrowserRows.map((row) => row.wrapper));
  assert.deepEqual(classified.unknown, []);
  const result = residualFailures(`//tools/buck:legacy\n${nativeWorkflow}`, map, { maxResidual: 1 });
  assert.deepEqual(result.failures, []);
  assert.deepEqual(result.residual, ["legacy"]);
  assert.equal(MAX_PRODUCT_BUCK_RESIDUAL, 7);
});

test("native registration cannot hide unknown targets or bypass the legacy ceiling", () => {
  const base = { entries: [{ name: "legacy" }], unmapped: [], native: structuredClone(nativeBrowserRows) };
  const wf = `//tools/buck:legacy\n${nativeWorkflow}`;
  const faults = [
    ["unknown native", (map) => map.native.push({ wrapper: "ghost", loc: "//backend/app:ghost", reason: "claimed native" }), `${wf}\n//tools/buck:ghost`],
    ["wrong native target", (map) => { map.native[0].loc = "//backend/app:console-app-itest-auth_rest"; }, wf],
    ["empty reason", (map) => { map.native[0].reason = ""; }, wf],
    ["duplicate native", (map) => map.native.push({ ...map.native[0] }), wf],
    ["overlap mapped", (map) => map.entries.push({ name: map.native[0].wrapper }), wf],
    ["overlap unmapped", (map) => map.unmapped.push({ wrapper: map.native[0].wrapper, reason: "legacy" }), wf],
    ["missing native registration", (map) => map.native.pop(), wf],
    ["native absent from CI", () => {}, "//tools/buck:legacy"],
  ];
  for (const [name, mutate, workflow] of faults) {
    const map = structuredClone(base);
    mutate(map);
    assert.notDeepEqual(residualFailures(workflow, map).failures, [], name);
  }
  assert.notDeepEqual(residualFailures(wf, base, { maxResidual: 0 }).failures, []);
});
