/**
 * Hostile fixtures for process.doc-comment-cfg-test-false-dark.
 *
 * RED shape (pre-fix): `text.includes("#[cfg(test)]")` treats a doc-comment-only
 * paste as a live unit-test binary. GREEN shape: comment-/literal-aware scan.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import {
  hasLiveCfgTestAttribute,
  unitTestedCrateSrcRoots,
} from "./check-executed-tests-cfg.mjs";
import { cargoTestKind } from "./lib/cargo-test-kind.mjs";
import { directExecutable, executableWorkflowCommands } from "./lib/ci-workflow-executables.mjs";
import { countDeclaredTestAttributes } from "./lib/executed-tests-baseline.mjs";

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

// Executable workflow reachability is distinct from a Buck target declaration.
const SDK_CI_SOURCE = "backend/crates/platform/authz/tests/cedar_sdk_identity.rs";
const SDK_CI_ARGV = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-platform-authz", "--test", "cedar_sdk_identity"];
function sdkIdentityWorkflowInvocations(workflow) {
  const commands = executableWorkflowCommands(workflow);
  return commands.filter((command, index) => {
    if (command.job !== "domain-unit" || command.malformed || command.controlFlow) return false;
    const direct = directExecutable(command.tokens);
    const next = commands[index + 1];
    return !direct.malformed
      && JSON.stringify(direct.tokens) === JSON.stringify(SDK_CI_ARGV)
      && next?.job === command.job && next.step === command.step
      && !next.malformed && !next.controlFlow
      && JSON.stringify(next.tokens) === JSON.stringify(["check_status", "cedar_sdk_identity"]);
  });
}

describe("Cedar SDK identity workflow reachability", () => {
  it("domain-unit invokes the complete SDK identity binary and retains the existing Cedar invocations", () => {
    const workflow = readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
    assert.equal(sdkIdentityWorkflowInvocations(workflow).length, 1,
      "domain-unit must execute cedar_sdk_identity once with the failure-collecting check_status");
    const cedar = executableWorkflowCommands(workflow)
      .filter((command) => command.job === "domain-unit" && !command.malformed && !command.controlFlow)
      .map((command) => directExecutable(command.tokens).tokens)
      .filter((tokens) => tokens[0] === "cargo" && tokens[1] === "test" && tokens.includes("console-platform-authz"));
    for (const target of ["cedar_pbac_readiness_cases", "cedar_pbac_legacy_only_observe_and_record", "cedar_diagnostic_fail_closed"]) {
      assert.equal(cedar.filter((tokens) => tokens.some((token, index) => token === "--test" && tokens[index + 1] === target)).length, 1,
        `retained Cedar target ${target} must still execute once`);
    }
  });

  it("the static attribute baseline records the two existing SDK identity tests", () => {
    const baseline = JSON.parse(readFileSync(new URL("../docs/program/executed-tests-baseline.json", import.meta.url), "utf8"));
    const source = readFileSync(new URL(`../${SDK_CI_SOURCE}`, import.meta.url), "utf8");
    assert.equal(countDeclaredTestAttributes(source), 2, "source still declares exactly the two reviewed SDK identity tests");
    assert.equal(baseline.test_attribute_baseline[SDK_CI_SOURCE], 2,
      "reachable-source baseline must include both SDK identity tests; this is not runtime evidence");
  });

  it("rejects nonexecuting and partial workflow claims", () => {
    const workflow = (body, fields = "", job = "domain-unit") => `jobs:\n  ${job}:\n    steps:\n      - name: SDK identity\n${fields}        run: |\n${body.split("\n").map((line) => `          ${line}`).join("\n")}\n`;
    const cargo = `SQLX_OFFLINE=true ${SDK_CI_ARGV.join(" ")}`;
    const body = `${cargo}\ncheck_status "cedar_sdk_identity"`;
    assert.equal(sdkIdentityWorkflowInvocations(workflow(body)).length, 1);
    for (const invalid of [
      workflow(`echo ${body}`), workflow(`# ${body}`),
      workflow(body, "        if: false\n"), workflow(body, "        continue-on-error: true\n"),
      workflow(body, "", "other-job"), workflow(`exit 0\n${body}`),
      workflow(`${cargo} -- --list\ncheck_status "cedar_sdk_identity"`),
      workflow(`${cargo} compiled_bundle_keys\ncheck_status "cedar_sdk_identity"`),
      workflow(`${cargo}\ncheck_status "another_binary"`), workflow(cargo),
    ]) assert.equal(sdkIdentityWorkflowInvocations(invalid).length, 0);
  });
});
