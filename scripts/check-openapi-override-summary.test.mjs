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
  DRAFTS_GET_PATH,
  LIFECYCLE_PREFLIGHT,
  LIFECYCLE_PREFLIGHT_PATH,
  LIFECYCLE_TRANSITION_CONFIG,
  OBJECT_TYPE_GET_PATH,
  OK_CODE,
  OPEN_OVERRIDE_REQUEST,
  OVERRIDE_SUMMARY,
  OVERRIDES_PATH,
  PREFLIGHT_OUTCOME,
  STORE_STRUCT,
  SUMMARY_FIELDS,
  TRANSITIONS_PATH,
  VALUE_FIELD,
  WRITE_FLOOR,
  evaluateOpenapiOverrideSummary,
} from "./check-openapi-override-summary.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-override-summary.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-override-summary-"));
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
  return `    ${OVERRIDE_SUMMARY}:
      type: object
      required:
      - id
      - target_type
      - target_id
      - actor
      - reason
      - before_snapshot
      - created_at
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        target_type: { type: string }
        target_id: { $ref: '#/components/schemas/Uuid' }
        actor: { $ref: '#/components/schemas/Uuid' }
        reason: { type: string }
        before_snapshot: { type: object, additionalProperties: true }
        created_at: { $ref: '#/components/schemas/Timestamp' }
    ${OPEN_OVERRIDE_REQUEST}:
      type: object
      properties:
        target_type: { type: string }
        before_snapshot: { type: object, additionalProperties: true }
    ${APPROVAL_SUMMARY}:
      type: object
      properties:
        decision: { type: string }
    ${LIFECYCLE_TRANSITION_CONFIG}:
      type: object
      properties:
        object_type_id: { type: string }
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
    Timestamp: { type: string, format: date-time }
    Uuid: { type: string, format: uuid }
    Company:
      type: object
      properties:
        org_id: { type: string }
    InventedSnapshot:
      type: object
      required: [kind]
      properties:
        kind: { type: string, enum: [draft, live] }
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

const HOLD_NEIGHBORS = `  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${APPROVAL_SUMMARY}'
  ${CREATE_PATH}:
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
  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'
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

const UNTYPED = `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OVERRIDE_SUMMARY}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-override-summary", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(OVERRIDES_PATH, "/api/v1/governance/overrides");
    assert.equal(OVERRIDE_SUMMARY, "OverrideSummary");
    assert.equal(STORE_STRUCT, "OverrideSummary");
    assert.equal(VALUE_FIELD, "before_snapshot");
    assert.equal(OK_CODE, "201");
    assert.equal(BOUND, 1);
    assert.deepEqual(SUMMARY_FIELDS, [
      "id",
      "target_type",
      "target_id",
      "actor",
      "reason",
      "before_snapshot",
      "created_at",
    ]);
    assert.deepEqual(
      rustStructFields(
        `pub struct OverrideSummary {
    pub id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub actor: UserId,
    pub reason: String,
    pub before_snapshot: serde_json::Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
`,
        "OverrideSummary",
      ),
      SUMMARY_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST governance overrides 201 stays a root additionalProperties bag", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/responses/${OK_CODE}`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when OverrideSummary is missing from composed schemas", () => {
    const result = evaluateOpenapiOverrideSummary({
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
          finding.location === `#/components/schemas/${OVERRIDE_SUMMARY}`
          && /OverrideSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on OverrideSummary", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        created_at: { $ref: '#/components/schemas/Timestamp' }",
          "        created_at: { $ref: '#/components/schemas/Timestamp' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${OVERRIDE_SUMMARY}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when before_snapshot is given a closed enum catalog", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        before_snapshot: { type: object, additionalProperties: true }",
          "        before_snapshot: { type: string, enum: [draft, live] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${VALUE_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when before_snapshot is a closed nested object schema", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        before_snapshot: { type: object, additionalProperties: true }",
          "        before_snapshot:\n          type: object\n          additionalProperties: false\n          properties:\n            kind: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${VALUE_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when before_snapshot $ref an invented snapshot schema", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        before_snapshot: { type: object, additionalProperties: true }",
          "        before_snapshot: { $ref: '#/components/schemas/InventedSnapshot' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${VALUE_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST governance overrides (Feature::ALL)", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride`,
            `  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance overrides is bound as a Head", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/responses/${OK_CODE}`
          && /ObjectKey|Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance overrides is bound to the request schema", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
            `$ref: '#/components/schemas/${OPEN_OVERRIDE_REQUEST}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/responses/${OK_CODE}`
          && /GovernanceOpenOverrideRequest/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance overrides is bound to ApprovalSummary", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
            `$ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OVERRIDES_PATH}/post/responses/${OK_CODE}`
          && /ApprovalSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when decide-approval is bound to OverrideSummary", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
            `$ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DECIDE_PATH}/post/responses/201`
          && /ApprovalSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to OverrideSummary", () => {
    const result = evaluateOpenapiOverrideSummary({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
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

  it("passes when 201 $ref the existing OverrideSummary and before_snapshot stays open", () => {
    const result = evaluateOpenapiOverrideSummary({
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
        /openapi override-summary typed-response gate passed/,
      );
    } else {
      assert.match(ran.stderr, /OverrideSummary|additionalProperties|root additionalProperties bag/);
    }
  });
});
