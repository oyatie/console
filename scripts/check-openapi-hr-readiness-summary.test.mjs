import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  BOUND,
  GET_FLOOR,
  HR_ANNUAL_LEAVE_FIELDS,
  HR_ANNUAL_LEAVE_SUMMARY,
  HR_ATTENDANCE_FIELDS,
  HR_ATTENDANCE_SUMMARY,
  HR_IMPORT_FIELDS,
  HR_IMPORT_SUMMARY,
  HR_PAYROLL_FIELDS,
  HR_PAYROLL_SUMMARY,
  HR_READINESS_FIELDS,
  HR_READINESS_SUMMARY,
  READINESS_GET_PATH,
  evaluateOpenapiHrReadinessSummary,
} from "./check-openapi-hr-readiness-summary.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-hr-readiness-summary.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-hr-readiness-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function padGets(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad/${i}:
    get:
      operationId: padGet${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function propertyLines(fields, indent = "        ") {
  return fields.map((name) => {
    if (name === "imports") {
      return `${indent}${name}: { $ref: '#/components/schemas/${HR_IMPORT_SUMMARY}' }`;
    }
    if (name === "payroll") {
      return `${indent}${name}: { $ref: '#/components/schemas/${HR_PAYROLL_SUMMARY}' }`;
    }
    if (name === "annual_leave") {
      return `${indent}${name}: { $ref: '#/components/schemas/${HR_ANNUAL_LEAVE_SUMMARY}' }`;
    }
    if (name === "attendance") {
      return `${indent}${name}: { $ref: '#/components/schemas/${HR_ATTENDANCE_SUMMARY}' }`;
    }
    if (name === "active_close_runs") {
      return `${indent}${name}: { type: integer, format: int64 }`;
    }
    return `${indent}${name}: { type: integer }`;
  }).join("\n");
}

function schemas() {
  return `    ${HR_READINESS_SUMMARY}:
      type: object
      properties:
${propertyLines(HR_READINESS_FIELDS)}
    ${HR_IMPORT_SUMMARY}:
      type: object
      properties:
${propertyLines(HR_IMPORT_FIELDS)}
    ${HR_PAYROLL_SUMMARY}:
      type: object
      properties:
${propertyLines(HR_PAYROLL_FIELDS)}
    ${HR_ANNUAL_LEAVE_SUMMARY}:
      type: object
      properties:
${propertyLines(HR_ANNUAL_LEAVE_FIELDS)}
    ${HR_ATTENDANCE_SUMMARY}:
      type: object
      properties:
${propertyLines(HR_ATTENDANCE_FIELDS)}`;
}

function spec(extraPaths, extraSchemas = schemas()) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padGets(GET_FLOOR)}
${extraPaths}
components:
  schemas:
${extraSchemas}
`;
}

const UNTYPED_ABSENCE = `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary
      responses:
        '200':
          content:
            application/json:
              schema:
                type: object
                additionalProperties: true
${UNTYPED_ABSENCE}`;

const TYPED = `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${HR_READINESS_SUMMARY}'
${UNTYPED_ABSENCE}`;

describe("check-openapi-hr-readiness-summary", () => {
  it("exports examined-zero floor and the existing DTO identities", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(READINESS_GET_PATH, "/api/v1/hr/readiness-summary");
    assert.equal(ABSENCE_EXIT_GET_PATH, "/api/v1/hr/absence-exit-dashboard");
    assert.equal(HR_READINESS_SUMMARY, "HrReadinessSummary");
    assert.equal(BOUND, 1);
    assert.deepEqual(HR_READINESS_FIELDS, [
      "imports",
      "payroll",
      "annual_leave",
      "attendance",
    ]);
  });

  it("fails while readiness 200 stays additionalProperties", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${READINESS_GET_PATH}/get/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when HrReadinessSummary is missing from composed schemas", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    ${HR_IMPORT_SUMMARY}: { type: object }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${HR_READINESS_SUMMARY}`
          && /HrReadinessSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on HrPayrollReadinessSummary", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        latest_updated_at: { type: integer }",
          "        latest_updated_at: { type: integer }\n        catalog: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${HR_PAYROLL_SUMMARY}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto getHrReadinessSummary (Feature::ALL)", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary`,
            `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary
      permissions:
      - employee_directory_read`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${READINESS_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when getHrReadinessSummary grows invented query params", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary
      responses:`,
            `  ${READINESS_GET_PATH}:
    get:
      operationId: getHrReadinessSummary
      parameters:
      - name: as_of
        in: query
        schema: { type: string }
      responses:`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${READINESS_GET_PATH}/get/parameters`
          && /Query/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when absence-exit-dashboard is bound to HrReadinessSummary", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            "schema: { type: object, additionalProperties: true }",
            `schema: { $ref: '#/components/schemas/${HR_READINESS_SUMMARY}' }`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`
          && /AbsenceExitDashboardResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when 200 $ref the existing HrReadinessSummary", () => {
    const result = evaluateOpenapiHrReadinessSummary({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi hr-readiness-summary typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /HrReadinessSummary|additionalProperties/);
    }
  });
});
