import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  ARTIFACT_FLOOR,
  BODY_GATE_REL,
  GENERATOR_ARTIFACTS,
  HOLD_FLOOR,
  PRODUCT_REL,
  REMAINING_HOLD_PHRASES,
  ROADMAP_REL,
  bodyGateConstants,
  evaluateCurrentAuthorityPalantirContract,
} from "./check-current-authority-palantir-contract.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-current-authority-palantir-contract.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "authority-palantir-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

const ARTIFACT_BODIES = Object.freeze({
  [GENERATOR_ARTIFACTS[0]]: "{}\n",
  [GENERATOR_ARTIFACTS[1]]: "// generated\n",
  [GENERATOR_ARTIFACTS[2]]: "export type Company = {};\n",
  [GENERATOR_ARTIFACTS[3]]: "<html></html>\n",
  [BODY_GATE_REL]: `const RESOLVED_FLOOR = 82;
const CENSUS_FLOOR = 291;
const BODY_UNDECIDABLE_MAX = 199;
`,
});

const HOLD_BLOCK = REMAINING_HOLD_PHRASES.map((phrase) => `- ${phrase}`).join("\n");

function landedProduct() {
  return `# Product
The published HTTP contract is generated from backend/crates/contracts/src/semantic_manifest.json (#998–#1018).
${HOLD_BLOCK}
`;
}

function landedRoadmap() {
  return `# Roadmap
The current body-gate floor is 82 resolved of 291 census; BODY_UNDECIDABLE_MAX is 199.
`;
}

describe("check-current-authority-palantir-contract", () => {
  it("reads live body-gate constants from the gate source", () => {
    const constants = bodyGateConstants(ARTIFACT_BODIES[BODY_GATE_REL]);
    assert.deepEqual(constants, {
      resolvedFloor: 82,
      censusFloor: 291,
      undecidableMax: 199,
    });
  });

  it("fails when PRODUCT still claims the generator is unimplemented", () => {
    const result = evaluateCurrentAuthorityPalantirContract({
      repoRoot: fixture({
        ...ARTIFACT_BODIES,
        [PRODUCT_REL]: `# Product\nADR-0031 remains accepted and unimplemented\n${HOLD_BLOCK}\n`,
        [ROADMAP_REL]: landedRoadmap(),
      }),
    });
    assert.ok(
      result.findings.some((finding) =>
        finding.message.includes("ADR-0031 remains accepted and unimplemented"),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ROADMAP still claims the stale 53/291 body-gate floor", () => {
    const result = evaluateCurrentAuthorityPalantirContract({
      repoRoot: fixture({
        ...ARTIFACT_BODIES,
        [PRODUCT_REL]: landedProduct(),
        [ROADMAP_REL]: "The current body-gate floor is 53 resolved of 291 census; 206 leftover.\n",
      }),
    });
    assert.ok(
      result.findings.some((finding) => /53 resolved of 291/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some((finding) => /82 resolved of 291/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when a remaining Palantir-class HOLD phrase is missing", () => {
    const result = evaluateCurrentAuthorityPalantirContract({
      repoRoot: fixture({
        ...ARTIFACT_BODIES,
        [PRODUCT_REL]: "# Product\nsemantic_manifest.json (#998)\n",
        [ROADMAP_REL]: landedRoadmap(),
      }),
    });
    assert.ok(
      result.findings.some((finding) => /remaining HOLD/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when current authority records the landed generator and remaining HOLDs", () => {
    const result = evaluateCurrentAuthorityPalantirContract({
      repoRoot: fixture({
        ...ARTIFACT_BODIES,
        [PRODUCT_REL]: landedProduct(),
        [ROADMAP_REL]: landedRoadmap(),
      }),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.artifacts, ARTIFACT_FLOOR);
    assert.equal(result.holds, HOLD_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until current authority records the generator", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /current-authority Palantir-contract gate passed/);
    } else {
      assert.match(
        ran.stderr,
        /unimplemented|53 resolved of 291|semantic_manifest|HOLD|Palantir-contract gate FAILED/,
      );
    }
  });
});
