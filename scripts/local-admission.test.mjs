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

  it("plans lens when ledger changes", () => {
    const gates = planGates(["docs/program/ledger/x.md"]);
    assert.ok(gates.some((g) => g.id === "check:reasoning-lens-contract"));
  });

  it("plans preflight when ci.yml changes", () => {
    const gates = planGates([".github/workflows/ci.yml"]);
    assert.ok(gates.some((g) => g.id === "check:ci-preflight"));
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
