import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  classifyPaths,
  cargoPackageNameFromDomainLib,
  parseArgs,
  planGates,
} from "./local-admission.mjs";

describe("local-admission path classification", () => {
  it("flags tip authority surfaces", () => {
    const f = classifyPaths([
      "docs/program/ledger/2026-08-06-example.md",
      "docs/documentation-index.json",
      "docs/program/executed-tests-baseline.json",
      "backend/crates/policy/domain/src/lib.rs",
    ]);
    assert.equal(f.ledger, true);
    assert.equal(f.docsIndex, true);
    assert.equal(f.baseline, true);
    assert.deepEqual(f.rustDomainLib, ["backend/crates/policy/domain/src/lib.rs"]);
  });

  it("flags CI contract surfaces", () => {
    const f = classifyPaths([".github/workflows/ci.yml", "scripts/check-ci-preflight.mjs"]);
    assert.equal(f.ci, true);
    assert.equal(f.scripts, true);
  });
});

describe("local-admission gate plan", () => {
  it("plans cargo test for domain lib changes", () => {
    const gates = planGates(["backend/crates/inspection/domain/src/lib.rs"]);
    const cargo = gates.filter((g) => g.id.startsWith("cargo-test:"));
    assert.ok(cargo.length >= 1, "expected cargo-test gate");
    assert.ok(cargo[0].cmd.includes("console-inspection-domain"));
  });

  it("plans the lens manifest check only for the files that can move it", () => {
    // The manifest is a projection of AGENTS.md's lens list into CLAUDE.md, so
    // only those two files can cause drift. The retired gate keyed off
    // ledger/docs-index/baseline changes, none of which can.
    for (const changed of [["AGENTS.md"], ["CLAUDE.md"]]) {
      assert.ok(
        planGates(changed).some((g) => g.id === "check:reasoning-lens-manifest"),
        `manifest check must be planned for ${JSON.stringify(changed)}`,
      );
    }
    for (const changed of [["docs/program/ledger/x.md"], ["backend/src/lib.rs"]]) {
      assert.ok(
        !planGates(changed).some((g) => g.id === "check:reasoning-lens-manifest"),
        `manifest check must NOT be planned for ${JSON.stringify(changed)}`,
      );
    }
  });

  it("plans preflight when ci.yml changes", () => {
    const gates = planGates([".github/workflows/ci.yml"]);
    assert.ok(gates.some((g) => g.id === "check:ci-preflight"));
  });

  it("plans process self-tests when scripts/tools.ci change", () => {
    const gates = planGates([
      "scripts/local-admission.mjs",
      "tools/ci/assess-tip-contention.mjs",
    ]);
    assert.ok(gates.some((g) => g.id === "test:local-admission"));
    assert.ok(gates.some((g) => g.id === "test:ci-tools"));
  });

  it("plans nothing heavy for unrelated docs", () => {
    const gates = planGates(["README.md"]);
    assert.equal(gates.length, 0);
  });
});

describe("local-admission helpers", () => {
  it("parses args", () => {
    const o = parseArgs(["--base", "main", "--json", "--dry-run"]);
    assert.equal(o.base, "main");
    assert.equal(o.json, true);
    assert.equal(o.dryRun, true);
  });

  it("reads cargo package name from domain crate", () => {
    const name = cargoPackageNameFromDomainLib(
      "backend/crates/inspection/domain/src/lib.rs",
    );
    assert.equal(name, "console-inspection-domain");
  });
});
