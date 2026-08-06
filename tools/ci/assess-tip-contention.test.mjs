import test from "node:test";
import assert from "node:assert/strict";
import {
  pathIsTipSerial,
  classifyPrFiles,
  assessTipContention,
  TIP_SERIAL_PATH_PREFIXES,
} from "./assess-tip-contention.mjs";

test("pathIsTipSerial matches manifest/baseline/ledger/ci roots", () => {
  assert.equal(pathIsTipSerial("docs/documentation-index.json"), true);
  assert.equal(pathIsTipSerial("docs/program/ledger/2026-08-06.md"), true);
  assert.equal(pathIsTipSerial(".github/workflows/ci.yml"), true);
  assert.equal(pathIsTipSerial("backend/Cargo.lock"), true);
  assert.equal(pathIsTipSerial("backend/crates/leave/domain/src/lib.rs"), false);
  assert.ok(TIP_SERIAL_PATH_PREFIXES.length >= 8);
});

test("classifyPrFiles marks tip writers", () => {
  const tip = classifyPrFiles([
    "backend/crates/leave/domain/src/lib.rs",
    "docs/program/console-program-ledger.md",
  ]);
  assert.equal(tip.is_tip_writer, true);
  assert.ok(tip.tip_files.includes("docs/program/console-program-ledger.md"));

  const pure = classifyPrFiles(["backend/crates/leave/domain/src/lib.rs"]);
  assert.equal(pure.is_tip_writer, false);
});

test("assessTipContention emits ops.tip-serial-contention when >=2 writers", () => {
  const report = assessTipContention([
    {
      number: 1,
      title: "tip a",
      mergeStateStatus: "CLEAN",
      files: ["docs/documentation-index.json"],
    },
    {
      number: 2,
      title: "tip b",
      mergeStateStatus: "BEHIND",
      files: [".github/workflows/ci.yml"],
    },
    {
      number: 3,
      title: "pure domain",
      mergeStateStatus: "CLEAN",
      files: ["backend/crates/leave/domain/src/lib.rs"],
    },
  ]);
  assert.equal(report.tip_writers, 2);
  assert.ok(report.class_ids.includes("ops.tip-serial-contention"));
  assert.ok(report.class_ids.includes("ops.missed-tip-sync"));
  assert.equal(report.behind_count, 1);
  assert.deepEqual(
    report.writers.map((w) => w.number),
    [1, 2],
  );
});

test("assessTipContention clean when zero or one tip writer", () => {
  const zero = assessTipContention([
    {
      number: 9,
      title: "domain",
      mergeStateStatus: "CLEAN",
      files: ["backend/crates/policy/domain/src/lib.rs"],
    },
  ]);
  assert.equal(zero.tip_writers, 0);
  assert.equal(zero.class_ids.includes("ops.tip-serial-contention"), false);

  const one = assessTipContention([
    {
      number: 10,
      title: "tip",
      mergeStateStatus: "CLEAN",
      files: ["package-lock.json"],
    },
  ]);
  assert.equal(one.tip_writers, 1);
  assert.equal(one.class_ids.includes("ops.tip-serial-contention"), false);
});
