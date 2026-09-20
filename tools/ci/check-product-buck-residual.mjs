#!/usr/bin/env node
/**
 * ADR-0039 / DN-0005 step 1 (partial): fail-closed residual Buck product surface.
 *
 * After S2, the load-bearing PG facets run via cargo_needs_postgres.sh. Any
 * product wrappers in ci.yml must appear in postgres-cargo-map.json. Reviewed
 * native browser variants are separate from the retained Cargo transition.
 * Unknown wrappers fail closed; the legacy residual ceiling does not increase.
 */
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
/** Ceiling measured 2026-08-05 after S2 (#583): seven residual Buck product wrappers. */
export const MAX_PRODUCT_BUCK_RESIDUAL = 7;

// This finite admission roster is checked against the registry, not inferred
// from a target name. Generator tests verify the native graph and CI producers.
const NATIVE_BROWSER_TARGETS = new Map([
  ["app-auth-rest-browser-pg", "//backend/app:console-app-itest-auth_rest-browser"],
  ["app-health-readiness-browser-pg", "//backend/app:console-app-itest-health_readiness-browser"],
]);

export function extractBuckWrappersFromCiYml(wf) {
  const names = new Set();
  const re = /\/\/tools\/buck:([a-zA-Z0-9_-]+)/g;
  let m;
  while ((m = re.exec(wf))) names.add(m[1]);
  return [...names].sort();
}

export function extractBuckWrappersFromPgFacetBlock(wf) {
  const startFacet = wf.indexOf("postgres-reachability-app:");
  const startLegacy = wf.indexOf("postgres-domain-reachability:");
  const start = startFacet >= 0 ? startFacet : startLegacy;
  const end = wf.indexOf("\n  company-conformance:", start);
  const block = start >= 0 && end > start ? wf.slice(start, end) : "";
  return extractBuckWrappersFromCiYml(block);
}

export function classifyResidual(wrappers, map) {
  const mappedNames = new Set((map.entries ?? []).map((e) => e.name));
  const unmappedNames = new Set(
    (map.unmapped ?? []).map((u) => String(u.wrapper || "").replace(/^.*:/, "")),
  );
  const residual_mapped = [];
  const residual_unmapped = [];
  const nativeNames = new Set((map.native ?? []).map((row) => row.wrapper));
  const native = [];
  const unknown = [];
  for (const name of wrappers) {
    if (mappedNames.has(name)) residual_mapped.push(name);
    else if (unmappedNames.has(name)) residual_unmapped.push(name);
    else if (nativeNames.has(name)) native.push(name);
    else unknown.push(name);
  }
  return { residual_mapped, residual_unmapped, native, unknown };
}

export function residualFailures(wf, map, { maxResidual = MAX_PRODUCT_BUCK_RESIDUAL } = {}) {
  const failures = [];
  const facetBuck = extractBuckWrappersFromPgFacetBlock(wf);
  if (facetBuck.length) {
    failures.push(
      `PG facet block must not list Buck wrappers (found ${facetBuck.length}: ${facetBuck.join(", ")})`,
    );
  }
  const all = extractBuckWrappersFromCiYml(wf);
  const { residual_mapped, residual_unmapped, native, unknown } = classifyResidual(all, map);
  if (unknown.length) {
    failures.push(
      `ci.yml Buck wrappers missing from postgres-cargo-map (mapped, unmapped or native): ${unknown.join(", ")}`,
    );
  }
  const residual = [...residual_mapped, ...residual_unmapped].sort();
  const legacyNames = new Set([
    ...(map.entries ?? []).map((row) => row.name),
    ...(map.unmapped ?? []).map((row) => String(row.wrapper || "").replace(/^.*:/, "")),
  ]);
  const seenNative = new Set();
  for (const row of map.native ?? []) {
    const name = row.wrapper;
    if (!NATIVE_BROWSER_TARGETS.has(name) || row.loc !== NATIVE_BROWSER_TARGETS.get(name)) {
      failures.push(`unapproved native browser wrapper/target: ${name}`);
    }
    if (typeof row.reason !== "string" || !row.reason.trim()) {
      failures.push(`native browser ${name} lacks reason`);
    }
    if (seenNative.has(name) || legacyNames.has(name)) {
      failures.push(`duplicate or overlapping native browser: ${name}`);
    }
    if (!all.includes(name)) {
      failures.push(`registered native browser absent from CI: ${name}`);
    }
    seenNative.add(name);
  }
  if (residual.length > maxResidual) {
    failures.push(
      `product Buck residual ${residual.length} exceeds ceiling ${maxResidual}: ${residual.join(", ")}`,
    );
  }
  // Unmapped residual must match map.unmapped reasons (documented darkness).
  for (const name of residual_unmapped) {
    const row = (map.unmapped ?? []).find(
      (u) => String(u.wrapper || "").replace(/^.*:/, "") === name,
    );
    if (!row?.reason) {
      failures.push(`unmapped residual ${name} lacks reason in postgres-cargo-map.unmapped`);
    }
  }
  return { failures, residual, residual_mapped, residual_unmapped, native, facetBuck };
}

const isMain =
  process.argv[1] &&
  fileURLToPath(import.meta.url) === resolve(process.argv[1]);

if (isMain) {
  const map = JSON.parse(readFileSync(resolve(root, "tools/ci/postgres-cargo-map.json"), "utf8"));
  const wf = readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8");
  const { failures, residual, residual_mapped, residual_unmapped, native } = residualFailures(wf, map);
  if (failures.length) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  console.log(
    `product-buck-residual OK (native ${native.length}; legacy residual ${residual.length}/${MAX_PRODUCT_BUCK_RESIDUAL}; ` +
      `mapped ${residual_mapped.length}; unmapped ${residual_unmapped.length}: ${residual.join(", ") || "none"})`,
  );
}
