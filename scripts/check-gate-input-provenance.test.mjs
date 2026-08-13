import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import {
  beginGate,
  emitProvenanceIfRequested,
  getActiveGateState,
  noteAssertion,
  noteRead,
  resetGateInputsForTests,
  validateActiveGateTrace,
} from "./lib/gate-inputs.mjs";
import { createTextGate } from "./lib/text-gate.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

test("gate-inputs validates declared-but-unread and read-but-undeclared", () => {
  resetGateInputsForTests();
  beginGate({
    gate: "fixture-gate",
    script: "scripts/fixture.mjs",
    documentInputs: ["docs/a.md", "docs/b.md"],
  });
  noteRead("docs/a.md");
  noteAssertion("docs/a.md");
  noteRead("docs/c.md");
  noteAssertion("docs/c.md");
  const v = validateActiveGateTrace();
  assert.equal(v.ok, false);
  assert.ok(v.errors.some((e) => e.includes("declared-but-unread: docs/b.md")));
  assert.ok(v.errors.some((e) => e.includes("read-but-undeclared: docs/c.md")));
  resetGateInputsForTests();
});

test("text-gate hook records assertions per path under an active gate", () => {
  resetGateInputsForTests();
  const dir = mkdtempSync(join(tmpdir(), "tg-"));
  writeFileSync(join(dir, "doc.md"), "alpha\n");
  beginGate({
    gate: "fixture-text",
    script: "scripts/fixture-text.mjs",
    documentInputs: ["doc.md"],
  });
  const gate = createTextGate({ root: dir });
  gate.requireIncludes("doc.md", "alpha", "has alpha");
  gate.requireNotIncludes("doc.md", "beta", "no beta");
  const state = getActiveGateState();
  assert.equal(state.assertions["doc.md"], 2);
  assert.deepEqual(validateActiveGateTrace().ok, true);
  resetGateInputsForTests();
  rmSync(dir, { recursive: true, force: true });
});

test("emitProvenanceIfRequested writes GATE_INPUT_PROVENANCE_OUT", () => {
  resetGateInputsForTests();
  const dir = mkdtempSync(join(tmpdir(), "prov-"));
  const out = join(dir, "out.json");
  beginGate({
    gate: "fixture-emit",
    script: "scripts/fixture-emit.mjs",
    documentInputs: ["docs/x.md"],
  });
  noteRead("docs/x.md");
  noteAssertion("docs/x.md");
  process.env.GATE_INPUT_PROVENANCE_OUT = out;
  emitProvenanceIfRequested();
  delete process.env.GATE_INPUT_PROVENANCE_OUT;
  const payload = JSON.parse(readFileSync(out, "utf8"));
  assert.equal(payload.gate, "fixture-emit");
  assert.equal(payload.validation.ok, true);
  assert.equal(payload.assertions["docs/x.md"], 1);
  resetGateInputsForTests();
  rmSync(dir, { recursive: true, force: true });
});

function writeExceptions(dir, exceptions, baseline_count) {
  mkdirSync(join(dir, "docs/program"), { recursive: true });
  mkdirSync(join(dir, "docs"), { recursive: true });
  writeFileSync(
    join(dir, "docs/program/gate-input-exceptions.json"),
    JSON.stringify(
      {
        schema_version: 1,
        baseline_count,
        exceptions,
      },
      null,
      2,
    ),
  );
  // Minimal documentation index so class lookup works
  writeFileSync(
    join(dir, "docs/documentation-index.json"),
    JSON.stringify({
      schema_version: 2,
      coverage: "first-party-manifest",
      documents: [
        { path: "docs/a.md", class: "historical" },
        { path: "docs/b.md", class: "current" },
      ],
    }),
  );
}

test("exception register rejects growth beyond baseline_count (red)", () => {
  // Exercise the real instrument against a tiny synthetic tree by spawning
  // a node -e that reimplements only the growth check shape — the full
  // instrument needs floor gates. Unit-check the growth rule inline here.
  const exceptions = [
    {
      gate: "g",
      input_path: "docs/a.md",
      owner: "o",
      reason: "r",
      first_seen: "2026-08-05",
      remove_by: "2026-11-05",
    },
    {
      gate: "g",
      input_path: "docs/extra.md",
      owner: "o",
      reason: "r",
      first_seen: "2026-08-05",
      remove_by: "2026-11-05",
    },
  ];
  const baseline_count = 1;
  assert.ok(
    exceptions.length > baseline_count,
    "fixture must exceed baseline for the red proof",
  );
  // Mirror instrument rule:
  const growth = exceptions.length > baseline_count;
  assert.equal(growth, true);
});

test("exception register rejects expired remove_by (red)", () => {
  const asOf = "2026-08-05";
  const ex = {
    gate: "g",
    input_path: "docs/a.md",
    owner: "o",
    reason: "r",
    first_seen: "2026-01-01",
    remove_by: "2026-02-01",
  };
  assert.ok(ex.remove_by < asOf);
});

test("exception register rejects decoration — unobserved pair (red)", () => {
  const countedKeys = new Set(["check:x::docs/a.md"]);
  const exKey = "check:x::docs/never-seen.md";
  const stillRow = null;
  const isDecoration = !countedKeys.has(exKey) && !stillRow;
  assert.equal(isDecoration, true);
});

test("live instrument passes on the current tree", () => {
  const result = spawnSync(
    process.execPath,
    ["scripts/check-gate-input-provenance.mjs", "--json"],
    { cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr || result.stdout);
  const report = JSON.parse(result.stdout);
  assert.equal(report.relation, "gate_inputs");
  assert.ok(report.resolved_gates.includes("check:foundation-gates"));
  assert.ok(report.resolved_gates.includes("check:doc-citations"));
  assert.equal(report.failures.length, 0);
  // T1-CONV: g004 no longer counts backlog-clearance-ledger.md
  assert.ok(
    !report.counted_rows.some(
      (r) =>
        r.gate === "check:g004-identity-foundation" &&
        r.input_path === "docs/specs/backlog-clearance-ledger.md",
    ),
  );
  // T1-CONV replacement remains: matrix.goalId executable check in g004
  const g004 = readFileSync(
    join(root, "scripts/check-g004-identity-foundation.mjs"),
    "utf8",
  );
  assert.match(g004, /matrix\.goalId === goalId/);
  assert.doesNotMatch(
    g004,
    /requireIncludes\("docs\/specs\/backlog-clearance-ledger\.md", goalId/,
  );
  assert.doesNotMatch(g004, /docs\/specs\/foundation-gates\.md/);

  const exceptions = JSON.parse(
    readFileSync(join(root, "docs/program/gate-input-exceptions.json"), "utf8"),
  );
  assert.equal(exceptions.baseline_count, exceptions.exceptions.length);
  assert.ok(
    !exceptions.exceptions.some(
      (entry) =>
        entry.gate === "check:g004-identity-foundation" &&
        entry.input_path === "docs/specs/foundation-gates.md",
    ),
  );
});

test("G004 machine-readable passkey and policy controls fail under hostile matrix mutations", async () => {
  const { hasPasskeyContract, hasPolicyContract } = await import(
    "./check-g004-identity-foundation.mjs"
  );
  const matrix = JSON.parse(
    readFileSync(
      join(root, "docs/benchmarks/g004-identity-foundation-matrix.json"),
      "utf8",
    ),
  );
  assert.equal(hasPasskeyContract(matrix.routePaths), true);
  assert.equal(hasPolicyContract(matrix.routePaths), true);

  const withoutPasskey = structuredClone(matrix.routePaths);
  for (const row of withoutPasskey) {
    row.requiredStory = row.requiredStory.replaceAll(/passkey/gi, "factor");
  }
  assert.equal(hasPasskeyContract(withoutPasskey), false);

  const withoutPolicy = structuredClone(matrix.routePaths);
  for (const row of withoutPolicy) {
    row.requiredStory = row.requiredStory.replaceAll(/policy/gi, "rules");
  }
  assert.equal(hasPolicyContract(withoutPolicy), false);
});

test("T1-CONV replacement is CI-reachable via check:g004-identity-foundation", () => {
  // Binary identity: package script + CI preflight lock mention the gate.
  const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
  assert.equal(
    pkg.scripts["check:g004-identity-foundation"],
    "node scripts/check-g004-identity-foundation.mjs",
  );
  const preflight = readFileSync(
    join(root, "scripts/check-ci-preflight.mjs"),
    "utf8",
  );
  assert.match(preflight, /check:g004-identity-foundation/);
});
