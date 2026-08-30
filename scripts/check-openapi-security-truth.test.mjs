import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  CREDENTIAL_RESET,
  OP_FLOOR,
  PUBLIC_OPERATIONS,
  evaluateOpenapiSecurityTruth,
} from "./check-openapi-security-truth.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-security-truth.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-security-truth-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function spec({ security = "", paths }) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
${security}paths:
${paths}
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
`;
}

const PUBLIC_PATHS = PUBLIC_OPERATIONS.map(([method, path]) => {
  return `  ${path}:
    ${method}:
      security: []
      responses:
        '200': { description: ok }`;
}).join("\n");

const HOLD_RESET = `  ${CREDENTIAL_RESET.path}:
    ${CREDENTIAL_RESET.method}:
      deprecated: true
      summary: Admin credential-reset (HOLD — not generally available)
      description: HOLD — not generally available. Live credential-reset remains HOLD.
      security:
      - bearerAuth: []
      responses:
        '200': { description: HOLD — not generally available }`;

const HONEST = spec({
  security: `security:
- bearerAuth: []
`,
  paths: `${PUBLIC_PATHS}
${HOLD_RESET}`,
});

describe("openapi security-truth gate", () => {
  it("keeps the public-operation set closed at the audited 18", () => {
    assert.equal(PUBLIC_OPERATIONS.length, 18);
    assert.equal(new Set(PUBLIC_OPERATIONS.map(([m, p]) => `${m} ${p}`)).size, 18);
  });

  it("reports a document with no top-level bearer security", () => {
    const root = fixture(spec({
      paths: `  /healthz:
    get:
      security: []
      responses:
        '200': { description: ok }
${HOLD_RESET}`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) => finding.location === "#/security"),
      JSON.stringify(findings, null, 2),
    );
  });

  it("reports a public operation that omits security instead of declaring security: []", () => {
    const root = fixture(spec({
      security: `security:
- bearerAuth: []
`,
      paths: `  /healthz:
    get:
      responses:
        '200': { description: ok }
${HOLD_RESET}`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) =>
        finding.location === "#/paths//healthz/get"
        && /security: \[\]/.test(finding.message)
      ),
      JSON.stringify(findings, null, 2),
    );
  });

  it("reports security: [] on a path that is not in the closed public set", () => {
    const root = fixture(spec({
      security: `security:
- bearerAuth: []
`,
      paths: `${PUBLIC_PATHS}
  /api/v1/payroll/runs:
    get:
      security: []
      responses:
        '200': { description: leaked }
${HOLD_RESET}`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) =>
        finding.location === "#/paths//api/v1/payroll/runs/get"
        && /closed public-operation/.test(finding.message)
      ),
      JSON.stringify(findings, null, 2),
    );
  });

  it("reports credential-reset that still looks generally available", () => {
    const root = fixture(spec({
      security: `security:
- bearerAuth: []
`,
      paths: `${PUBLIC_PATHS}
  ${CREDENTIAL_RESET.path}:
    ${CREDENTIAL_RESET.method}:
      summary: Reset a user's credentials for account recovery (admin)
      description: Account-recovery escape hatch that mints a fresh one-time code.
      security:
      - bearerAuth: []
      responses:
        '200': { description: The fresh one-time code and its expiry. }`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) =>
        finding.location
          === `#/paths/${CREDENTIAL_RESET.path}/${CREDENTIAL_RESET.method}`
        && /not look generally available/.test(finding.message)
      ),
      JSON.stringify(findings, null, 2),
    );
  });

  it("reports credential-reset declared public with security: []", () => {
    const root = fixture(spec({
      security: `security:
- bearerAuth: []
`,
      paths: `${PUBLIC_PATHS}
  ${CREDENTIAL_RESET.path}:
    ${CREDENTIAL_RESET.method}:
      deprecated: true
      summary: Admin credential-reset (HOLD — not generally available)
      description: HOLD — not generally available.
      security: []
      responses:
        '200': { description: HOLD }`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) =>
        finding.location
          === `#/paths/${CREDENTIAL_RESET.path}/${CREDENTIAL_RESET.method}`
        && /not a public/.test(finding.message)
      ),
      JSON.stringify(findings, null, 2),
    );
  });

  it("accepts top-level bearer, explicit public overrides, and HOLD credential-reset", () => {
    const root = fixture(HONEST);
    const { findings, publicDeclared } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
    assert.equal(publicDeclared, 18);
  });

  it("accepts a protected operation that omits security once document-level bearer exists", () => {
    const root = fixture(spec({
      security: `security:
- bearerAuth: []
`,
      paths: `${PUBLIC_PATHS}
  /api/v1/payroll/runs:
    get:
      responses:
        '200': { description: inherits bearer }
${HOLD_RESET}`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("reports omitted security as public when the document has no inheritable bearer", () => {
    const root = fixture(spec({
      paths: `  /api/v1/payroll/runs:
    get:
      responses:
        '200': { description: looks public }
${HOLD_RESET}`,
    }));

    const { findings } = evaluateOpenapiSecurityTruth({ repoRoot: root });
    assert.ok(
      findings.some((finding) =>
        finding.location === "#/paths//api/v1/payroll/runs/get"
        && /treats it as public/.test(finding.message)
      ),
      JSON.stringify(findings, null, 2),
    );
  });

  // The live document is the hole this lane closes. This assertion is red on
  // origin/dev (no top-level bearer, 18 ops omit security, credential-reset
  // looks GA) and green only after those three siblings match the published
  // security truth.
  it("exits 0 against this repository, above the floors, with no security-truth holes", () => {
    const { findings, operations } = evaluateOpenapiSecurityTruth({ repoRoot });

    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 8), null, 2));
    assert.ok(
      operations >= OP_FLOOR,
      `walker degraded: saw ${operations} operations, floor ${OP_FLOOR}`,
    );

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi security-truth gate passed/);
  });
});
