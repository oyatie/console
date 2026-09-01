import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  ALERT_FIELDS,
  ALERT_RESPONSE,
  BOUND,
  CATALOG_ENTRY,
  CATALOG_GET_PATH,
  DASHBOARD_FIELDS,
  DASHBOARD_RESPONSE,
  DRAFT_RECORD,
  DRAFTS_PATH,
  EXIT_CASE_RESPONSE,
  EXIT_CASES_PATH,
  GET_FLOOR,
  KILL_SWITCH_PATH,
  OBJECT_TYPE_GET_PATH,
  QUERY_PARAMS,
  ROLLOUT_OPT_IN_PATH,
  SETTLEMENT_PACKAGE,
  SETTLEMENT_VALUE_FIELDS,
  SIGNAL_PAYLOAD,
  SUMMARY,
  SUMMARY_FIELDS,
  VALUE_FIELDS,
  WRITE_FLOOR,
  evaluateOpenapiAbsenceExitDashboard,
} from "./check-openapi-absence-exit-dashboard.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-absence-exit-dashboard.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-absence-exit-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function padWrites(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad/${i}:
    post:
      operationId: padPost${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function padGets(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad-get/${i}:
    get:
      operationId: padGet${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function schemas(extra = "") {
  return `    ${DASHBOARD_RESPONSE}:
      type: object
      required:
      - summary
      - alerts
      - exit_cases
      properties:
        summary: { $ref: '#/components/schemas/${SUMMARY}' }
        alerts:
          type: array
          items: { $ref: '#/components/schemas/${ALERT_RESPONSE}' }
        exit_cases:
          type: array
          items: { $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}' }
    ${SUMMARY}:
      type: object
      required:
      - open_absence_alerts
      - exit_cases_pending_hr
      - settlement_needs_source
      - settlement_ready
      - approval_drafts
      - submitted
      properties:
        open_absence_alerts: { type: integer, format: int64 }
        exit_cases_pending_hr: { type: integer, format: int64 }
        settlement_needs_source: { type: integer, format: int64 }
        settlement_ready: { type: integer, format: int64 }
        approval_drafts: { type: integer, format: int64 }
        submitted: { type: integer, format: int64 }
    ${ALERT_RESPONSE}:
      type: object
      required:
      - id
      - employee_id
      - employee_name
      - company
      - work_date
      - source
      - status
      - severity
      - audience_roles
      - signal_payload
      - notification_title
      - notification_message
      - link_href
      - detected_at
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        employee_id: { $ref: '#/components/schemas/Uuid' }
        employee_name: { type: string }
        employee_number: { type: string }
        company: { type: string }
        org_unit: { type: string }
        worksite_name: { type: string }
        branch_id: { $ref: '#/components/schemas/Uuid' }
        branch_name: { type: string }
        work_date: { type: string }
        source: { type: string }
        status: { type: string }
        severity: { type: string }
        audience_roles:
          type: array
          items: { type: string }
        signal_payload: { type: object, additionalProperties: true }
        notification_title: { type: string }
        notification_message: { type: string }
        link_href: { type: string }
        exit_case_id: { $ref: '#/components/schemas/Uuid' }
        detected_at: { $ref: '#/components/schemas/Timestamp' }
    ${EXIT_CASE_RESPONSE}:
      type: object
      properties:
        settlement_package: { $ref: '#/components/schemas/${SETTLEMENT_PACKAGE}' }
    ${SETTLEMENT_PACKAGE}:
      type: object
      properties:
        statutory_basis: { type: object, additionalProperties: true }
        insurance_loss_payload: { type: object, additionalProperties: true }
        approval_payload: { type: object, additionalProperties: true }
    ${DRAFT_RECORD}:
      type: object
      properties:
        draft_key: { type: string }
    ${CATALOG_ENTRY}:
      type: object
      properties:
        stable_key: { type: string }
    Timestamp: { type: string, format: date-time }
    Uuid: { type: string, format: uuid }
    Company:
      type: object
      properties:
        org_id: { type: string }
    InventedStatutory:
      type: object
      required: [article]
      properties:
        article: { type: string, enum: [LSA_26] }
${extra}`;
}

function spec(extraPaths, extraSchemas = "") {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padWrites(WRITE_FLOOR)}
${padGets(GET_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas(extraSchemas)}
`;
}

const QUERY = `      parameters:
      - name: limit
        in: query
      - name: offset
        in: query
      - name: employee_id
        in: query`;

const HOLD_NEIGHBORS = `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${ROLLOUT_OPT_IN_PATH}:
    put:
      operationId: updateConsoleRolloutOptIn
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${DRAFTS_PATH}:
    post:
      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'
  ${EXIT_CASES_PATH}:
    post:
      operationId: reportEmployeeExitCase
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`;

const UNTYPED = `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
${QUERY}
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
${QUERY}
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DASHBOARD_RESPONSE}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-absence-exit-dashboard", () => {
  it("exports examined-zero floors, paths, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(ABSENCE_EXIT_GET_PATH, "/api/v1/hr/absence-exit-dashboard");
    assert.equal(DASHBOARD_RESPONSE, "AbsenceExitDashboardResponse");
    assert.deepEqual(VALUE_FIELDS, [
      "signal_payload",
      "statutory_basis",
      "insurance_loss_payload",
      "approval_payload",
    ]);
    assert.deepEqual(SETTLEMENT_VALUE_FIELDS, [
      "statutory_basis",
      "insurance_loss_payload",
      "approval_payload",
    ]);
    assert.equal(BOUND, 1);
    assert.deepEqual(DASHBOARD_FIELDS, ["summary", "alerts", "exit_cases"]);
    assert.equal(SUMMARY_FIELDS.length, 6);
    assert.equal(ALERT_FIELDS.length, 20);
    assert.deepEqual(QUERY_PARAMS, ["limit", "offset", "employee_id"]);
    assert.deepEqual(
      rustStructFields(
        `struct AbsenceExitDashboardResponse {
    summary: AbsenceExitSummary,
    alerts: Vec<EmployeeAbsenceAlertResponse>,
    exit_cases: Vec<EmployeeExitCaseResponse>,
}
`,
        "AbsenceExitDashboardResponse",
      ),
      DASHBOARD_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while GET absence-exit-dashboard 200 stays a root additionalProperties bag", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when AbsenceExitDashboardResponse is missing from composed schemas", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padWrites(WRITE_FLOOR)}
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    Uuid: { type: string, format: uuid }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DASHBOARD_RESPONSE}`
          && /AbsenceExitDashboardResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on AbsenceExitDashboardResponse", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        exit_cases:\n          type: array\n          items: { $ref: '#/components/schemas/EmployeeExitCaseResponse' }",
          "        exit_cases:\n          type: array\n          items: { $ref: '#/components/schemas/EmployeeExitCaseResponse' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DASHBOARD_RESPONSE}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when exit_cases duplicates the envelope instead of $ref EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        exit_cases:\n          type: array\n          items: { $ref: '#/components/schemas/EmployeeExitCaseResponse' }",
          "        exit_cases:\n          type: array\n          items:\n            type: object\n            additionalProperties: true",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DASHBOARD_RESPONSE}/properties/exit_cases`
          && /EmployeeExitCaseResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when signal_payload is given a closed enum catalog", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        signal_payload: { type: object, additionalProperties: true }",
          "        signal_payload: { type: string, enum: [LSA_26, LSA_34] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ALERT_RESPONSE}/properties/${SIGNAL_PAYLOAD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when signal_payload is a closed nested object schema", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        signal_payload: { type: object, additionalProperties: true }",
          "        signal_payload:\n          type: object\n          additionalProperties: false\n          properties:\n            form: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ALERT_RESPONSE}/properties/${SIGNAL_PAYLOAD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when signal_payload $ref an invented statutory schema", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        signal_payload: { type: object, additionalProperties: true }",
          "        signal_payload: { $ref: '#/components/schemas/InventedStatutory' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ALERT_RESPONSE}/properties/${SIGNAL_PAYLOAD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when signal_payload is an array of invented nested schemas", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        signal_payload: { type: object, additionalProperties: true }",
          "        signal_payload:\n          type: array\n          items:\n            type: object\n            properties:\n              article: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ALERT_RESPONSE}/properties/${SIGNAL_PAYLOAD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when nested statutory_basis is given a closed enum catalog", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        statutory_basis: { type: object, additionalProperties: true }",
          "        statutory_basis: { type: string, enum: [LSA_26, LSA_34] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/statutory_basis`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when status is given an invented enum catalog", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        status: { type: string }\n        severity",
          "        status: { type: string, enum: [OPEN, CLOSED] }\n        severity",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ALERT_RESPONSE}/properties/status`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET absence-exit-dashboard (Feature::ALL)", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard`,
            `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      permissions:
      - hr_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET absence-exit-dashboard is bound as a Head", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${DASHBOARD_RESPONSE}'`,
            `$ref: '#/components/schemas/Company'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`
          && /Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET absence-exit-dashboard is bound to EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${DASHBOARD_RESPONSE}'`,
            `$ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`
          && /EmployeeExitCaseResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch is bound to AbsenceExitDashboardResponse", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`,
            `  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DASHBOARD_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${KILL_SWITCH_PATH}/post/responses/200`
          && /kill-switch/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented query param appears", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            "      - name: employee_id\n        in: query",
            "      - name: employee_id\n        in: query\n      - name: as_of\n        in: query",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/parameters`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when GET 200 $ref the existing envelope and Value fields stay open", () => {
    const result = evaluateOpenapiAbsenceExitDashboard({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(
        ran.stdout,
        /openapi absence-exit-dashboard typed-response gate passed/,
      );
    } else {
      assert.match(
        ran.stderr,
        /AbsenceExitDashboardResponse|additionalProperties|root additionalProperties bag/,
      );
    }
  });
});
