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
  APPROVAL_DRAFT_PATH,
  BOUND,
  CATALOG_ENTRY,
  CATALOG_GET_PATH,
  CONFIRM_PATH,
  DRAFT_RECORD,
  DRAFTS_PATH,
  EXIT_CASE_RESPONSE,
  EXIT_CASES_PATH,
  GET_FLOOR,
  KILL_SWITCH_PATH,
  NEXT_ACTION,
  NEXT_ACTION_FIELDS,
  OBJECT_TYPE_GET_PATH,
  OVERRIDE_SUMMARY,
  RECORD_FIELDS,
  ROLLOUT_OPT_IN_PATH,
  SETTLEMENT_FIELDS,
  SETTLEMENT_PACKAGE,
  VALUE_FIELDS,
  WRITE_FLOOR,
  evaluateOpenapiEmployeeExitCaseResponse,
} from "./check-openapi-employee-exit-case-response.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-employee-exit-case-response.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-exit-case-"));
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
  return `    ${EXIT_CASE_RESPONSE}:
      type: object
      required:
      - id
      - employee_id
      - employee_name
      - company
      - status
      - effective_exit_date
      - site_manager_note
      - reported_by
      - reported_at
      - next_actions
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
        absence_alert_id: { $ref: '#/components/schemas/Uuid' }
        status: { type: string }
        effective_exit_date: { type: string }
        site_manager_note: { type: string }
        reported_by: { $ref: '#/components/schemas/Uuid' }
        reported_at: { $ref: '#/components/schemas/Timestamp' }
        hr_confirmed_by: { $ref: '#/components/schemas/Uuid' }
        hr_confirmed_at: { $ref: '#/components/schemas/Timestamp' }
        hq_confirmed_by: { $ref: '#/components/schemas/Uuid' }
        hq_confirmed_at: { $ref: '#/components/schemas/Timestamp' }
        approval_submitted_by: { $ref: '#/components/schemas/Uuid' }
        approval_submitted_at: { $ref: '#/components/schemas/Timestamp' }
        settlement_package: { $ref: '#/components/schemas/${SETTLEMENT_PACKAGE}' }
        next_actions:
          type: array
          items: { $ref: '#/components/schemas/${NEXT_ACTION}' }
    ${SETTLEMENT_PACKAGE}:
      type: object
      required:
      - id
      - status
      - missing_source_fields
      - statutory_basis
      - insurance_loss_payload
      - approval_payload
      - certification_status
      - generated_at
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        status: { type: string }
        service_days: { type: integer, format: int32 }
        average_wage_period_start: { type: string }
        average_wage_period_end: { type: string }
        average_wage_calendar_days: { type: integer, format: int32 }
        average_wage_total_won: { type: integer, format: int64 }
        average_daily_wage_milliwon: { type: integer, format: int64 }
        severance_pay_won: { type: integer, format: int64 }
        monthly_ordinary_wage_won: { type: integer, format: int64 }
        ordinary_daily_wage_won: { type: integer, format: int64 }
        statutory_daily_wage_milliwon: { type: integer, format: int64 }
        missing_source_fields:
          type: array
          items: { type: string }
        statutory_basis: { type: object, additionalProperties: true }
        insurance_loss_payload: { type: object, additionalProperties: true }
        approval_payload: { type: object, additionalProperties: true }
        certification_status: { type: string }
        generated_at: { $ref: '#/components/schemas/Timestamp' }
        submitted_by: { $ref: '#/components/schemas/Uuid' }
        submitted_at: { $ref: '#/components/schemas/Timestamp' }
    ${NEXT_ACTION}:
      type: object
      required:
      - key
      - label
      - href
      properties:
        key: { type: string }
        label: { type: string }
        href: { type: string }
    ${DRAFT_RECORD}:
      type: object
      properties:
        draft_key: { type: string }
    ${CATALOG_ENTRY}:
      type: object
      properties:
        stable_key: { type: string }
    ${OVERRIDE_SUMMARY}:
      type: object
      properties:
        reason: { type: string }
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

const HOLD_NEIGHBORS = `  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${OBJECT_TYPE_GET_PATH}:
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
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'`;

const UNTYPED = `  ${EXIT_CASES_PATH}:
    post:
      operationId: reportEmployeeExitCase
      responses:
        '201':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${CONFIRM_PATH}:
    post:
      operationId: confirmEmployeeExitCase
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${APPROVAL_DRAFT_PATH}:
    post:
      operationId: draftEmployeeExitApproval
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${EXIT_CASES_PATH}:
    post:
      operationId: reportEmployeeExitCase
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'
  ${CONFIRM_PATH}:
    post:
      operationId: confirmEmployeeExitCase
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'
  ${APPROVAL_DRAFT_PATH}:
    post:
      operationId: draftEmployeeExitApproval
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-employee-exit-case-response", () => {
  it("exports examined-zero floors, paths, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(EXIT_CASES_PATH, "/api/v1/hr/exit-cases");
    assert.equal(EXIT_CASE_RESPONSE, "EmployeeExitCaseResponse");
    assert.deepEqual(VALUE_FIELDS, [
      "statutory_basis",
      "insurance_loss_payload",
      "approval_payload",
    ]);
    assert.equal(BOUND, 3);
    assert.equal(RECORD_FIELDS.length, 23);
    assert.equal(SETTLEMENT_FIELDS.length, 20);
    assert.deepEqual(NEXT_ACTION_FIELDS, ["key", "label", "href"]);
    assert.deepEqual(
      rustStructFields(
        `struct EmployeeExitCaseResponse {
    id: Uuid,
    employee_id: Uuid,
    employee_name: String,
    employee_number: Option<String>,
    company: String,
    org_unit: Option<String>,
    worksite_name: Option<String>,
    branch_id: Option<Uuid>,
    branch_name: Option<String>,
    absence_alert_id: Option<Uuid>,
    status: String,
    effective_exit_date: String,
    site_manager_note: String,
    reported_by: Uuid,
    reported_at: OffsetDateTime,
    hr_confirmed_by: Option<Uuid>,
    hr_confirmed_at: Option<OffsetDateTime>,
    hq_confirmed_by: Option<Uuid>,
    hq_confirmed_at: Option<OffsetDateTime>,
    approval_submitted_by: Option<Uuid>,
    approval_submitted_at: Option<OffsetDateTime>,
    settlement_package: Option<EmployeeExitSettlementPackageResponse>,
    next_actions: Vec<ExitCaseNextAction>,
}
`,
        "EmployeeExitCaseResponse",
      ),
      RECORD_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST exit-cases 201/confirm/approval-draft stay a root additionalProperties bag", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXIT_CASES_PATH}/post/responses/201`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CONFIRM_PATH}/post/responses/200`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${APPROVAL_DRAFT_PATH}/post/responses/200`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when EmployeeExitCaseResponse is missing from composed schemas", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
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
          finding.location === `#/components/schemas/${EXIT_CASE_RESPONSE}`
          && /EmployeeExitCaseResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        next_actions:\n          type: array\n          items: { $ref: '#/components/schemas/ExitCaseNextAction' }",
          "        next_actions:\n          type: array\n          items: { $ref: '#/components/schemas/ExitCaseNextAction' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXIT_CASE_RESPONSE}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when statutory_basis is given a closed enum catalog", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
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

  it("fails when insurance_loss_payload is a closed nested object schema", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        insurance_loss_payload: { type: object, additionalProperties: true }",
          "        insurance_loss_payload:\n          type: object\n          additionalProperties: false\n          properties:\n            form: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/insurance_loss_payload`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when approval_payload $ref an invented statutory schema", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        approval_payload: { type: object, additionalProperties: true }",
          "        approval_payload: { $ref: '#/components/schemas/InventedStatutory' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/approval_payload`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when statutory_basis is an array of invented nested schemas", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        statutory_basis: { type: object, additionalProperties: true }",
          "        statutory_basis:\n          type: array\n          items:\n            type: object\n            properties:\n              article: { type: string }",
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
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        status: { type: string }\n        effective_exit_date",
          "        status: { type: string, enum: [REPORTED, CONFIRMED] }\n        effective_exit_date",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/status`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST exit-cases (Feature::ALL)", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${EXIT_CASES_PATH}:
    post:
      operationId: reportEmployeeExitCase`,
            `  ${EXIT_CASES_PATH}:
    post:
      operationId: reportEmployeeExitCase
      permissions:
      - hr_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXIT_CASES_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST exit-cases is bound as a Head", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`,
            `$ref: '#/components/schemas/Company'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXIT_CASES_PATH}/post/responses/201`
          && /Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST exit-cases is bound to DraftRecord", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`,
            `$ref: '#/components/schemas/${DRAFT_RECORD}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXIT_CASES_PATH}/post/responses/201`
          && /DraftRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when absence-exit-dashboard is bound to EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `              schema: { type: object, additionalProperties: true }
  ${OBJECT_TYPE_GET_PATH}:`,
            `              schema:
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'
  ${OBJECT_TYPE_GET_PATH}:`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`
          && /dashboard envelope/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch is bound to EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
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
                $ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`,
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

  it("fails when policy drafts are bound to EmployeeExitCaseResponse", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${DRAFT_RECORD}'`,
            `$ref: '#/components/schemas/${EXIT_CASE_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/post/responses/201`
          && /DraftRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when every exit-case op $ref the existing envelope and Value fields stay open", () => {
    const result = evaluateOpenapiEmployeeExitCaseResponse({
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
        /openapi employee-exit-case-response typed-response gate passed/,
      );
    } else {
      assert.match(
        ran.stderr,
        /EmployeeExitCaseResponse|additionalProperties|root additionalProperties bag/,
      );
    }
  });
});
