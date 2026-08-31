import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  APPROVAL_SUMMARY,
  BOUND,
  CREATE_PATH,
  DECIDE_PATH,
  DECISION_RESPONSE,
  DECISION_VALUES,
  DRAFTS_GET_PATH,
  LIFECYCLE_PREFLIGHT,
  LIFECYCLE_PREFLIGHT_PATH,
  OBJECT_TYPE_GET_PATH,
  OK_CODE,
  OVERRIDES_PATH,
  PREFLIGHT_OUTCOME,
  STORE_STRUCT,
  SUMMARY_FIELDS,
  TRANSITIONS_PATH,
  WRITE_FLOOR,
  evaluateOpenapiApprovalSummary,
} from "./check-openapi-approval-summary.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-approval-summary.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-approval-summary-"));
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

function schemas(extra = "") {
  return `    ${APPROVAL_SUMMARY}:
      type: object
      required:
      - id
      - request_ref
      - kind
      - requested_by
      - approver_id
      - decision
      - decided_at
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        request_ref: { $ref: '#/components/schemas/Uuid' }
        kind: { type: string }
        requested_by: { $ref: '#/components/schemas/Uuid' }
        approver_id: { $ref: '#/components/schemas/Uuid' }
        decision: { type: string, enum: [${DECISION_VALUES.join(", ")}] }
        decided_at: { $ref: '#/components/schemas/Timestamp' }
    Uuid: { type: string, format: uuid }
    Timestamp: { type: string, format: date-time }
    ${LIFECYCLE_PREFLIGHT}:
      type: object
      properties:
        configured: { type: boolean }
    ${PREFLIGHT_OUTCOME}:
      type: object
      properties:
        would_execute: { type: boolean }
    ${DECISION_RESPONSE}:
      type: object
      properties:
        outcome: { type: object, additionalProperties: true }
    Company:
      type: object
      properties:
        org_id: { type: string }
${extra}`;
}

function spec(extraPaths, extraSchemas = "") {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padWrites(WRITE_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas(extraSchemas)}
`;
}

const HOLD_NEIGHBORS = `  ${CREATE_PATH}:
    post:
      operationId: createGovernanceApproval
      responses:
        '201':
          content:
            application/json:
              schema:
                type: object
                properties:
                  payload_summary: { type: object, additionalProperties: true }
  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '201':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition
      responses:
        '201':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'
  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'
  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${DRAFTS_GET_PATH}:
    get:
      operationId: listPolicyDrafts
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  type: object
                  additionalProperties: true
  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${APPROVAL_SUMMARY}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-approval-summary", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(DECIDE_PATH, "/api/v1/governance/approvals/decide");
    assert.equal(APPROVAL_SUMMARY, "ApprovalSummary");
    assert.equal(STORE_STRUCT, "ApprovalSummary");
    assert.equal(OK_CODE, "201");
    assert.equal(BOUND, 1);
    assert.deepEqual(SUMMARY_FIELDS, [
      "id",
      "request_ref",
      "kind",
      "requested_by",
      "approver_id",
      "decision",
      "decided_at",
    ]);
    assert.deepEqual(DECISION_VALUES, ["approved", "rejected"]);
    assert.deepEqual(
      rustStructFields(
        `pub struct ApprovalSummary {
    pub id: Uuid,
    pub request_ref: Uuid,
    pub kind: String,
    pub requested_by: UserId,
    pub approver_id: UserId,
    pub decision: ApprovalDecision,
    pub decided_at: OffsetDateTime,
}
`,
        "ApprovalSummary",
      ),
      SUMMARY_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST governance approvals decide 201 stays additionalProperties", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DECIDE_PATH}/post/responses/${OK_CODE}`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ApprovalSummary is missing from composed schemas", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padWrites(WRITE_FLOOR)}
${TYPED}
components:
  schemas:
    Uuid: { type: string, format: uuid }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${APPROVAL_SUMMARY}`
          && /ApprovalSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on ApprovalSummary", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        decided_at: { $ref: '#/components/schemas/Timestamp' }",
          "        decided_at: { $ref: '#/components/schemas/Timestamp' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${APPROVAL_SUMMARY}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when decision publishes pending (this 201 refuses it)", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        decision: { type: string, enum: [${DECISION_VALUES.join(", ")}] }`,
          "        decision: { type: string, enum: [pending, approved, rejected] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${APPROVAL_SUMMARY}/properties/decision`
          && /pending|GateKind/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST governance approvals decide (Feature::ALL)", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval`,
            `  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DECIDE_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance approvals decide is bound as a Head", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DECIDE_PATH}/post/responses/${OK_CODE}`
          && /ObjectKey|Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance approvals decide is bound to LifecyclePreflight", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DECIDE_PATH}/post/responses/${OK_CODE}`
          && /LifecyclePreflight/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when create-approval is bound to ApprovalSummary", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `              schema:
                type: object
                properties:
                  payload_summary: { type: object, additionalProperties: true }`,
            `              schema:
                $ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CREATE_PATH}/post/responses/201`
          && /ApprovalRequestSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when overrides 201 is bound to ApprovalSummary", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '201':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`,
            `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/responses/201`
          && /OverrideSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to ApprovalSummary", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_GET_PATH}/get/responses/200`
          && /DraftRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when 201 $ref the existing ApprovalSummary", () => {
    const result = evaluateOpenapiApprovalSummary({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(
        ran.stdout,
        /openapi approval-summary typed-response gate passed/,
      );
    } else {
      assert.match(ran.stderr, /ApprovalSummary|additionalProperties/);
    }
  });
});
