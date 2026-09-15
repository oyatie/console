/**
 * Hostile fixtures for process.doc-comment-cfg-test-false-dark.
 *
 * RED shape (pre-fix): `text.includes("#[cfg(test)]")` treats a doc-comment-only
 * paste as a live unit-test binary. GREEN shape: comment-/literal-aware scan.
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { describe, it } from "node:test";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  hasLiveCfgTestAttribute,
  unitTestedCrateSrcRoots,
} from "./check-executed-tests-cfg.mjs";
import { cargoTestKind } from "./lib/cargo-test-kind.mjs";
import { RECOVERY_CASES } from "./lib/recovery-test-invocations.mjs";

const gateSource = readFileSync(
  fileURLToPath(new URL("./check-executed-tests.mjs", import.meta.url)),
  "utf8",
);

const DOC_ONLY = `/// Mask \`#[cfg(test)]\` / \`#[cfg(all(test, …))]\` modules so inline fixture SQL
/// under \`app/src/**\` does not false-positive as unaudited handlers.
pub fn compute_test_mask() {}
`;

const LINE_COMMENT_ONLY = `// #[cfg(test)]
pub fn not_a_test_module() {}
`;

const BLOCK_COMMENT_ONLY = `/* #[cfg(test)] */
pub fn not_a_test_module() {}
`;

const STRING_ONLY = `pub const PROSE: &str = "#[cfg(test)]";
`;

const LIVE = `#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {}
}
`;

const LIVE_IN_SIBLING_STYLE = `pub fn production() {}

#[cfg(test)]
mod tests {
    #[test]
    fn unit() { assert_eq!(1, 1); }
}
`;

describe("hasLiveCfgTestAttribute — false-dark controls", () => {
  it("documents the RED substring defect the durable fix replaces", () => {
    // Pre-fix oracle: raw includes lights up on doc-comment prose alone.
    assert.equal(DOC_ONLY.includes("#[cfg(test)]"), true);
    assert.equal(LINE_COMMENT_ONLY.includes("#[cfg(test)]"), true);
    assert.equal(BLOCK_COMMENT_ONLY.includes("#[cfg(test)]"), true);
    assert.equal(STRING_ONLY.includes("#[cfg(test)]"), true);
  });

  it("does not treat doc-comment-only #[cfg(test)] as live", () => {
    assert.equal(hasLiveCfgTestAttribute(DOC_ONLY), false);
  });

  it("does not treat line- or block-comment-only #[cfg(test)] as live", () => {
    assert.equal(hasLiveCfgTestAttribute(LINE_COMMENT_ONLY), false);
    assert.equal(hasLiveCfgTestAttribute(BLOCK_COMMENT_ONLY), false);
  });

  it("does not treat string-literal-only #[cfg(test)] as live", () => {
    assert.equal(hasLiveCfgTestAttribute(STRING_ONLY), false);
  });

  it("still detects a real #[cfg(test)] attribute (fails closed on live tests)", () => {
    assert.equal(hasLiveCfgTestAttribute(LIVE), true);
    assert.equal(hasLiveCfgTestAttribute(LIVE_IN_SIBLING_STYLE), true);
  });

  it("still detects live attribute when a doc comment also mentions it", () => {
    const mixed = `${DOC_ONLY}\n${LIVE}`;
    assert.equal(hasLiveCfgTestAttribute(mixed), true);
  });
});

describe("unitTestedCrateSrcRoots — inventory + examined-zero", () => {
  it("excludes crates whose only hit is a doc comment", () => {
    const roots = unitTestedCrateSrcRoots([
      ["backend/ci/gates/audit-coverage/src/lib.rs", DOC_ONLY],
    ]);
    assert.equal(roots.size, 0);
  });

  it("includes crates with a live #[cfg(test)] under src/", () => {
    const roots = unitTestedCrateSrcRoots([
      ["backend/crates/example/src/lib.rs", "pub fn ok() {}"],
      ["backend/crates/example/src/foo.rs", LIVE_IN_SIBLING_STYLE],
    ]);
    assert.deepEqual([...roots], ["backend/crates/example/src"]);
  });

  it("fails closed when examined src inventory is empty", () => {
    assert.throws(
      () => unitTestedCrateSrcRoots([]),
      /examined zero backend\/\*\*\/src\/\*\*\/\*\.rs files/,
    );
    assert.throws(
      () => unitTestedCrateSrcRoots([["backend/crates/example/tests/it.rs", LIVE]]),
      /examined zero/,
    );
  });
});

describe("cargoTestKind — cargo metadata lib aliases", () => {
  it("maps default lib and explicit rlib/cdylib to cargo test --lib", () => {
    assert.equal(cargoTestKind(["lib"]), "lib");
    assert.equal(cargoTestKind(["rlib"]), "lib");
    assert.equal(cargoTestKind(["rlib", "cdylib"]), "lib");
    assert.equal(cargoTestKind(["cdylib"]), "lib");
    assert.equal(cargoTestKind(["proc-macro"]), "lib");
  });

  it("keeps integration tests and drops bins", () => {
    assert.equal(cargoTestKind(["test"]), "test");
    assert.equal(cargoTestKind(["bin"]), null);
    assert.equal(cargoTestKind(["custom-build"]), null);
  });
});

describe("gate wiring", () => {
  it("routes definedBinaries through the comment-aware helper, not raw includes", () => {
    assert.match(gateSource, /unitTestedCrateSrcRoots\(files\)/);
    assert.match(gateSource, /from "\.\/check-executed-tests-cfg\.mjs"/);
    assert.match(gateSource, /cargoTestKind\(target\.kind\)/);
    // Live predicate must not be a raw file-text includes call (comment prose may still
    // describe that historical defect without reintroducing it as code).
    const codeLines = gateSource
      .split("\n")
      .filter((line) => !/^\s*\/\//.test(line) && !/^\s*\*/.test(line));
    assert.equal(
      codeLines.some((line) => line.includes('includes("#[cfg(test)]")')),
      false,
    );
  });
});

const repository = fileURLToPath(new URL("../", import.meta.url));
const integrationRoot = "backend/app/tests/auth_rest.rs";
const recoveryRoot = "backend/crates/payroll/adapter-postgres/tests/recovery.rs";
const observerRoot = "backend/crates/payroll/adapter-postgres/tests/durability_observer.rs";
const testSource = "#[test]\nfn reachable() {}\n";

function sourceInventoryFixture(t, files, expectedCount) {
  const root = mkdtempSync(join(tmpdir(), "executed-test-sources-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (path, contents) => {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), contents);
  };
  for (const path of ["scripts/check-executed-tests.mjs", "scripts/check-executed-tests-cfg.mjs", "scripts/lib"]) {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    cpSync(join(repository, path), join(root, path), { recursive: true });
  }
  symlinkSync(join(repository, "node_modules"), join(root, "node_modules"), "dir");
  const anchors = [
    "backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs",
    "backend/crates/attendance/adapter-postgres/tests/concurrency.rs",
    "backend/ci/gates/tenant-isolation/tests/owner_only_acl_postgres18.rs",
    "backend/app/src/lib.rs",
  ];
  for (const path of anchors) write(path, path.endsWith("/lib.rs") ? LIVE : testSource);
  write(observerRoot, readFileSync(join(repository, observerRoot), "utf8"));
  write(recoveryRoot, readFileSync(join(repository, recoveryRoot), "utf8"));
  for (const [path, contents] of Object.entries(files)) write(path, contents);
  write("backend/Cargo.toml", `[package]
name = "console-payroll-adapter-postgres"
version = "0.0.0"
edition = "2021"
[lib]
path = "app/src/lib.rs"
[[test]]
name = "recovery"
path = "crates/payroll/adapter-postgres/tests/recovery.rs"
[[test]]
name = "durability_observer"
path = "crates/payroll/adapter-postgres/tests/durability_observer.rs"
`);
  const lock = spawnSync("cargo", ["generate-lockfile", "--offline", "--manifest-path", "backend/Cargo.toml"], {
    cwd: root, encoding: "utf8",
  });
  assert.equal(lock.status, 0, lock.error?.message ?? lock.stderr);
  const targets = [...anchors, integrationRoot];
  write("backend/app/BUCK", targets.map((path, index) => `rust_test(\n    name = "fixture-${index}",\n    crate_root = "${path}",\n    features = [${path.endsWith("/lib.rs") ? '"test-postgres"' : ""}],\n)\n`).join("\n"));
  write("tools/buck/BUCK", "");
  write("tools/ci/postgres-cargo-map.json", '{"entries":[]}\n');
  const workflow = readFileSync(join(repository, ".github/workflows/ci.yml"), "utf8");
  const recoveryJob = workflow.match(/^  postgres-reachability-domain-b:\n[\s\S]*?(?=^  [A-Za-z0-9_-]+:)/m)?.[0];
  assert.ok(recoveryJob, "fixture must retain the real supervised recovery wiring");
  write(".github/workflows/ci.yml", `jobs:\n${recoveryJob}  fixture:\n    steps:\n      - name: Run fixture\n        run: tools/buck2 test ${targets.map((_, index) => `//backend/app:fixture-${index}`).join(" ")}\n`);
  const baseline = {
    dark_baseline: 0,
    deferred_fixture: [],
    test_attribute_baseline: {
      ...Object.fromEntries(anchors.map((path) => [path, 1])),
      [recoveryRoot]: RECOVERY_CASES.size,
      [observerRoot]: 1,
      [integrationRoot]: expectedCount,
    },
  };
  write("docs/program/executed-tests-baseline.json", JSON.stringify(baseline));
  return {
    write,
    compile() {
      const result = spawnSync("rustc", ["--edition=2021", "--test", integrationRoot, "-o", join(root, "fixture-tests")], {
        cwd: root, encoding: "utf8",
      });
      assert.equal(result.status, 0, result.error?.message ?? result.stderr);
      const listing = spawnSync(join(root, "fixture-tests"), ["--list"], { encoding: "utf8" });
      assert.equal(listing.status, 0, listing.stderr);
      assert.match(listing.stdout, new RegExp(`\\n${expectedCount} tests?, 0 benchmarks`));
      const execution = spawnSync(join(root, "fixture-tests"), [], { encoding: "utf8" });
      assert.equal(execution.status, 0, execution.stderr);
      assert.ok(execution.stdout.includes(`test result: ok. ${expectedCount} passed; 0 failed; 0 ignored;`));
    },
    run() {
      const result = spawnSync(process.execPath, ["scripts/check-executed-tests.mjs", "--json"], {
        cwd: root, encoding: "utf8",
      });
      assert.ifError(result.error);
      const report = JSON.parse(result.stdout);
      assert.deepEqual(report.unresolved, [], result.stderr);
      assert.deepEqual(report.dark, [], result.stderr);
      return { ...result, report };
    },
  };
}

describe("reachable Rust source inventory", () => {
  it("follows path and ordinary modules recursively without counting orphan sources", (t) => {
    const fixture = sourceInventoryFixture(t, {
      [integrationRoot]: `#[path = "auth_rest/linked.rs"]\nmod linked;\n${testSource}`,
      "backend/app/tests/auth_rest/linked.rs": `mod nested;\nmod directory;\nmod inline { mod child; }\n${testSource}`,
      "backend/app/tests/auth_rest/nested.rs": `mod child;\n${testSource}`,
      "backend/app/tests/auth_rest/nested/child.rs": testSource,
      "backend/app/tests/auth_rest/directory/mod.rs": `mod leaf;\n${testSource}`,
      "backend/app/tests/auth_rest/directory/leaf.rs": testSource,
      "backend/app/tests/auth_rest/inline/child.rs": testSource,
      "backend/app/tests/auth_rest/orphan.rs": testSource,
    }, 7);
    fixture.compile();
    const result = fixture.run();
    assert.equal(result.report.testAttributes[integrationRoot], 7, result.stderr);
    assert.equal(result.status, 0, result.stderr);
  });

  it("rejects loss of a referenced child test even when an orphan gains a replacement", (t) => {
    const fixture = sourceInventoryFixture(t, {
      [integrationRoot]: `#[path = "auth_rest/linked.rs"]\nmod linked;\n${testSource}`,
      "backend/app/tests/auth_rest/linked.rs": testSource,
    }, 2);
    fixture.compile();
    const before = fixture.run();
    assert.equal(before.status, 0, before.stderr);
    fixture.write("backend/app/tests/auth_rest/linked.rs", "pub fn helper() {}\n");
    fixture.write("backend/app/tests/auth_rest/orphan.rs", testSource);
    const after = fixture.run();
    assert.equal(after.report.testAttributes[integrationRoot], 1);
    assert.equal(after.status, 1);
    assert.match(after.stderr, /auth_rest\.rs: 2 -> 1 \(-1\)/);
    assert.match(after.stderr, /lost declared test attributes/);
  });

  it("ignores test attributes and module declarations in comments and strings", (t) => {
    const fixture = sourceInventoryFixture(t, {
      [integrationRoot]: `${testSource}
/*
#[test]
fn comment_only() {}
#[path = "auth_rest/comment.rs"]
mod comment;
*/
const TEXT: &str = r#"
#[test]
fn string_only() {}
#[path = "auth_rest/string.rs"]
mod string;
"#;
// #[path = "auth_rest/line_comment.rs"] mod line_comment;
`,
      "backend/app/tests/auth_rest/comment.rs": testSource,
      "backend/app/tests/auth_rest/string.rs": testSource,
      "backend/app/tests/auth_rest/line_comment.rs": testSource,
    }, 1);
    fixture.compile();
    const result = fixture.run();
    assert.equal(result.report.testAttributes[integrationRoot], 1, result.stderr);
    assert.equal(result.status, 0, result.stderr);
  });
});
