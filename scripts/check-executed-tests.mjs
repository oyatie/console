#!/usr/bin/env node
// Which Rust test FILES actually execute in CI, and which are merely defined.
//
// WHY THIS EXISTS
//
// Five times this repository has shipped a correct-looking test that executed nowhere.
// A `rust_test` target is not wiring; a crate is not wiring; only a path from a workflow
// step to a test file is wiring. Nothing computed that path, so nobody could answer
// "does this test run?" without tracing it by hand — and tracing by hand is what failed
// five times.
//
// This resolves the chain that actually exists today:
//
//   .github/workflows/ci.yml
//     ├─ //tools/buck:<wrapper>          → tools/buck/BUCK sh_test
//     │     └─ args = ["$(location //crate:target)"]
//     │           └─ <crate>/BUCK rust_test → crate_root  ← the test FILE
//     ├─ //backend/...:<target> (direct)  → same rust_test lookup
//     └─ cargo test -p <pkg> [--test <name>] → crate_root by package + test name
//
// SILENT DEGRADATION IS THE FAILURE MODE, not a wrong number. A resolver that quietly
// stops resolving reports a smaller executed set and a larger gap, which reads as
// "we found more problems" rather than "the tool broke". So every link that fails to
// resolve is reported as UNRESOLVED and exits non-zero, and named anchors below must
// resolve or the run fails regardless of counts.
//
// Usage:
//   node scripts/check-executed-tests.mjs            # report, exit 1 on unresolved
//   node scripts/check-executed-tests.mjs --json

import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const CI = join(ROOT, ".github/workflows/ci.yml");
const WRAPPER_BUCK = join(ROOT, "tools/buck/BUCK");

// Each of these MUST resolve end to end. They are chosen to cover one instance of every
// link type, so a resolver that degrades on any single shape fails loudly here rather
// than returning a plausible smaller number.
const ANCHORS = [
  // wrapper → itest target → tests/*.rs
  "backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs",
  // wrapper → adapter itest
  "backend/crates/attendance/adapter-postgres/tests/concurrency.rs",
  // cargo test -p … --test … → tests/*.rs
  "backend/ci/gates/tenant-isolation/tests/owner_only_acl_postgres18.rs",
];

function walk(dir, out = []) {
  for (const entry of readdirSync(dir)) {
    if (entry === "node_modules" || entry === ".git" || entry === "buck-out" || entry === "target") continue;
    const p = join(dir, entry);
    const st = statSync(p);
    if (st.isDirectory()) walk(p, out);
    else if (entry === "BUCK") out.push(p);
  }
  return out;
}

/** Every rust_test in the repo: fully-qualified target → its crate_root file. */
function indexRustTests() {
  const byTarget = new Map();
  for (const file of walk(join(ROOT, "backend"))) {
    const text = readFileSync(file, "utf8");
    const cellPath = file.slice(ROOT.length + 1).replace(/\/BUCK$/, "");
    for (const m of text.matchAll(/rust_test\(([\s\S]*?)\n\)/g)) {
      const body = m[1];
      const name = body.match(/name\s*=\s*"([^"]+)"/)?.[1];
      const root = body.match(/crate_root\s*=\s*"([^"]+)"/)?.[1];
      if (!name || !root) continue;
      byTarget.set(`//${cellPath}:${name}`, root);
    }
  }
  return byTarget;
}

/** Wrapper name → the //backend target it locates. */
function indexWrappers() {
  const byName = new Map();
  const text = readFileSync(WRAPPER_BUCK, "utf8");
  for (const m of text.matchAll(/sh_test\(([\s\S]*?)\n\)/g)) {
    const body = m[1];
    const name = body.match(/name\s*=\s*"([^"]+)"/)?.[1];
    const target = body.match(/\$\(location (\/\/[^)]+)\)/)?.[1];
    if (name && target) byName.set(name, target);
  }
  return byName;
}

const rustTests = indexRustTests();
const wrappers = indexWrappers();
const ciRaw = readFileSync(CI, "utf8");
// Shell line-continuations are joined FIRST. Matching `cargo test[^\n]*` against the raw
// text consumes the trailing backslash, so a continuation group can never fire and every
// flag on a following line is silently invisible — the resolver reports a smaller executed
// set, which reads as a finding rather than as a broken tool.
const ci = ciRaw.replace(/\\\s*\n\s*/g, " ");

const executed = new Map();   // crate_root file → how it is reached
const unresolved = [];

// 1. //tools/buck:<wrapper>
for (const m of ci.matchAll(/\/\/tools\/buck:([a-z0-9_-]+)/g)) {
  const wrapper = m[1];
  const target = wrappers.get(wrapper);
  if (!target) { unresolved.push(`ci.yml names //tools/buck:${wrapper}, absent from tools/buck/BUCK`); continue; }
  const root = rustTests.get(target);
  if (!root) { unresolved.push(`wrapper ${wrapper} locates ${target}, which is not a rust_test`); continue; }
  executed.set(root, `//tools/buck:${wrapper}`);
}

// 2. direct //backend/...:target in a workflow step
for (const m of ci.matchAll(/(\/\/backend\/[A-Za-z0-9_\/-]+:[A-Za-z0-9_-]+)/g)) {
  const root = rustTests.get(m[1]);
  if (root) executed.set(root, m[1]);
}

// 3. cargo test -p <pkg> [--test <name>]
for (const m of ci.matchAll(/cargo test[^\n]*/g)) {
  const line = m[0];
  const pkgs = [...line.matchAll(/-p\s+([A-Za-z0-9_-]+)/g)].map((x) => x[1]);
  const tests = [...line.matchAll(/--test\s+([A-Za-z0-9_]+)/g)].map((x) => x[1]);
  for (const pkg of pkgs) {
    // A package's rust_test targets share the crate name with '-' separators.
    const hits = [...rustTests.entries()].filter(([t]) => t.includes(pkg));
    if (hits.length === 0) { unresolved.push(`cargo test -p ${pkg} matches no rust_test target`); continue; }
    for (const [, root] of hits) {
      if (tests.length === 0) { if (root.includes("/src/")) executed.set(root, `cargo -p ${pkg}`); }
      else if (tests.some((t) => root.endsWith(`/tests/${t}.rs`))) executed.set(root, `cargo -p ${pkg} --test`);
    }
  }
}

// Filesystem-derived: every tests/*.rs, plus every src/lib.rs carrying #[cfg(test)].
// Independent of any build system, so it survives buck2's deletion.
function definedTestFiles() {
  const out = new Set();
  const scan = (dir) => {
    for (const entry of readdirSync(dir)) {
      if (["node_modules", ".git", "buck-out", "target"].includes(entry)) continue;
      const p = join(dir, entry);
      if (statSync(p).isDirectory()) { scan(p); continue; }
      if (!entry.endsWith(".rs")) continue;
      const rel = p.slice(ROOT.length + 1);
      if (/\/tests\/[^/]+\.rs$/.test(rel)) out.add(rel);
      else if (/\/src\/lib\.rs$/.test(rel) && readFileSync(p, "utf8").includes("#[cfg(test)]")) out.add(rel);
    }
  };
  scan(join(ROOT, "backend"));
  return [...out].sort();
}
const defined = definedTestFiles();
const executedFiles = [...executed.keys()].sort();
const dark = defined.filter((f) => !executed.has(f));

const missingAnchors = ANCHORS.filter((a) => !executed.has(a));
for (const a of missingAnchors) {
  unresolved.push(`ANCHOR ${a} no longer resolves — the resolver has silently degraded`);
}

if (process.argv.includes("--json")) {
  console.log(JSON.stringify({ defined: defined.length, executed: executedFiles.length, dark, unresolved }, null, 2));
} else {
  console.log(`rust_test crate_roots defined : ${defined.length}`);
  console.log(`reachable from a CI step      : ${executedFiles.length}`);
  console.log(`executed nowhere              : ${dark.length}`);
  for (const f of dark.slice(0, 25)) console.log(`  dark  ${f}`);
  if (dark.length > 25) console.log(`  … and ${dark.length - 25} more`);
  for (const u of unresolved) console.log(`  UNRESOLVED  ${u}`);
}

// THE RATCHET. The dark count may fall and may never rise.
//
// This states plainly what it implies, because a ratchet whose implication is unstated is
// unimplementable: FROM NOW ON, A NEW TEST FILE MUST BE WIRED INTO CI IN THE SAME PULL
// REQUEST THAT ADDS IT. Adding backend/**/tests/foo.rs without a workflow path to it
// raises the count and fails here. That is the intended cost — this repository has shipped
// five tests that executed nowhere, and every one of them was added without wiring.
//
// The baseline is a committed number, not a first-run measurement, so a regression in the
// resolver cannot quietly raise the bar to whatever it currently reports.
const baselinePath = join(ROOT, "docs/program/executed-tests-baseline.json");
if (existsSync(baselinePath)) {
  const baseline = JSON.parse(readFileSync(baselinePath, "utf8")).dark_baseline;
  if (dark.length > baseline) {
    console.error(`\nexecuted-nowhere count rose from ${baseline} to ${dark.length}. A test file was added without a path from any workflow step. Wire it, or the repository has one more test that cannot fail.`);
    process.exit(1);
  }
  if (dark.length < baseline) {
    console.log(`\nexecuted-nowhere fell ${baseline} -> ${dark.length}. Lower docs/program/executed-tests-baseline.json to lock the gain in.`);
  }
}

// Unresolved links are a tool failure and must never read as a finding.
if (unresolved.length > 0) {
  console.error(`\n${unresolved.length} unresolved link(s): the resolver could not follow the chain. Fix the resolver before trusting any count above.`);
  process.exit(1);
}
