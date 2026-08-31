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
  APPROVAL_SUMMARY,
  AUTHORIZE_PATH,
  BOUND,
  CATALOG_ENTRY,
  CATALOG_GET_PATH,
  DECIDE_PATH,
  DECISION_RESPONSE,
  DRAFT_ID_PATH,
  DRAFT_RECORD,
  DRAFTS_PATH,
  GET_FLOOR,
  OBJECT_TYPE_GET_PATH,
  OVERRIDE_SUMMARY,
  OVERRIDES_PATH,
  POLICY_CREATE_DRAFT_REQUEST,
  RECORD_FIELDS,
  REVIEW_PATH,
  STORE_STRUCT,
  SUBMIT_PATH,
  VALIDATE_PATH,
  VALUE_FIELDS,
  WRITE_FLOOR,
  evaluateOpenapiDraftRecord,
} from "./check-openapi-draft-record.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-draft-record.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-draft-record-"));
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
  return `    ${DRAFT_RECORD}:
      type: object
      required:
      - id
      - draft_key
      - title
      - normalized_row
      - generated_policy_text
      - validation_status
      - validation_errors
      - review_status
      - reviewer_id
      - created_by
      - created_at
      - updated_at
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        draft_key: { type: string }
        title: { type: string }
        normalized_row: { type: object, additionalProperties: true }
        generated_policy_text: { type: string }
        validation_status: { type: string }
        validation_errors: { type: object, additionalProperties: true }
        review_status: { type: string }
        reviewer_id:
          oneOf:
          - $ref: '#/components/schemas/Uuid'
          - type: 'null'
        created_by: { $ref: '#/components/schemas/Uuid' }
        created_at: { $ref: '#/components/schemas/Timestamp' }
        updated_at: { $ref: '#/components/schemas/Timestamp' }
    ${CATALOG_ENTRY}:
      type: object
      properties:
        stable_key: { type: string }
    ${OVERRIDE_SUMMARY}:
      type: object
      properties:
        reason: { type: string }
    ${DECISION_RESPONSE}:
      type: object
      properties:
        outcome: { type: object, additionalProperties: true }
    ${APPROVAL_SUMMARY}:
      type: object
      properties:
        decision: { type: string }
    ${POLICY_CREATE_DRAFT_REQUEST}:
      type: object
      properties:
        draft_key: { type: string }
    Timestamp: { type: string, format: date-time }
    Uuid: { type: string, format: uuid }
    Company:
      type: object
      properties:
        org_id: { type: string }
    InventedRow:
      type: object
      required: [kind]
      properties:
        kind: { type: string, enum: [permit, forbid] }
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

const HOLD_NEIGHBORS = `  ${CATALOG_GET_PATH}:
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
  ${AUTHORIZE_PATH}:
    post:
      operationId: authorizePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DECISION_RESPONSE}'
  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OVERRIDE_SUMMARY}'
  ${DECIDE_PATH}:
    post:
      operationId: decideGovernanceApproval
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${APPROVAL_SUMMARY}'
  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
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
  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${DRAFTS_PATH}:
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
    post:
      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${DRAFT_ID_PATH}:
    get:
      operationId: getPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
    put:
      operationId: updatePolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${REVIEW_PATH}:
    post:
      operationId: reviewPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${SUBMIT_PATH}:
    post:
      operationId: submitPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${VALIDATE_PATH}:
    post:
      operationId: validatePolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${DRAFTS_PATH}:
    get:
      operationId: listPolicyDrafts
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/${DRAFT_RECORD}'
    post:
      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
  ${DRAFT_ID_PATH}:
    get:
      operationId: getPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
    put:
      operationId: updatePolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
  ${REVIEW_PATH}:
    post:
      operationId: reviewPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
  ${SUBMIT_PATH}:
    post:
      operationId: submitPolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
  ${VALIDATE_PATH}:
    post:
      operationId: validatePolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-draft-record", () => {
  it("exports examined-zero floors, paths, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(DRAFTS_PATH, "/api/v1/policy/drafts");
    assert.equal(DRAFT_RECORD, "DraftRecord");
    assert.equal(STORE_STRUCT, "DraftRecord");
    assert.deepEqual(VALUE_FIELDS, ["normalized_row", "validation_errors"]);
    assert.equal(BOUND, 7);
    assert.deepEqual(RECORD_FIELDS, [
      "id",
      "draft_key",
      "title",
      "normalized_row",
      "generated_policy_text",
      "validation_status",
      "validation_errors",
      "review_status",
      "reviewer_id",
      "created_by",
      "created_at",
      "updated_at",
    ]);
    assert.deepEqual(
      rustStructFields(
        `pub struct DraftRecord {
    pub id: Uuid,
    pub draft_key: String,
    pub title: String,
    pub normalized_row: serde_json::Value,
    pub generated_policy_text: String,
    pub validation_status: String,
    pub validation_errors: serde_json::Value,
    pub review_status: String,
    pub reviewer_id: Option<Uuid>,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
`,
        "DraftRecord",
      ),
      RECORD_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while GET/POST policy drafts 200/201 stay a root additionalProperties bag", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/get/responses/200`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/post/responses/201`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when DraftRecord is missing from composed schemas", () => {
    const result = evaluateOpenapiDraftRecord({
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
          finding.location === `#/components/schemas/${DRAFT_RECORD}`
          && /DraftRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on DraftRecord", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        updated_at: { $ref: '#/components/schemas/Timestamp' }",
          "        updated_at: { $ref: '#/components/schemas/Timestamp' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DRAFT_RECORD}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when normalized_row is given a closed enum catalog", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        normalized_row: { type: object, additionalProperties: true }",
          "        normalized_row: { type: string, enum: [permit, forbid] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/normalized_row`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when validation_errors is a closed nested object schema", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        validation_errors: { type: object, additionalProperties: true }",
          "        validation_errors:\n          type: object\n          additionalProperties: false\n          properties:\n            code: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/validation_errors`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when normalized_row $ref an invented row schema", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        normalized_row: { type: object, additionalProperties: true }",
          "        normalized_row: { $ref: '#/components/schemas/InventedRow' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/normalized_row`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when validation_errors is an array of invented nested schemas", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        validation_errors: { type: object, additionalProperties: true }",
          "        validation_errors:\n          type: array\n          items:\n            type: object\n            properties:\n              code: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/validation_errors`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when review_status is given an invented enum catalog", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        review_status: { type: string }",
          "        review_status: { type: string, enum: [draft, approved] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/review_status`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when reviewer_id drops the JSON null union", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        reviewer_id:\n          oneOf:\n          - $ref: '#/components/schemas/Uuid'\n          - type: 'null'",
          "        reviewer_id: { $ref: '#/components/schemas/Uuid' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${DRAFT_RECORD}/properties/reviewer_id`
          && /Option<Uuid>/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST policy drafts (Feature::ALL)", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${DRAFTS_PATH}:
    get:
      operationId: listPolicyDrafts`,
            `  ${DRAFTS_PATH}:
    get:
      operationId: listPolicyDrafts
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET policy drafts is bound as a Head", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  $ref: '#/components/schemas/${DRAFT_RECORD}'`,
            `                items:
                  $ref: '#/components/schemas/Company'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/get/responses/200`
          && /Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST policy drafts is bound to the request schema", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'`,
            `      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${POLICY_CREATE_DRAFT_REQUEST}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/post/responses/201`
          && /PolicyCreateDraftRequest/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST policy drafts is bound to OverrideSummary", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DRAFT_RECORD}'`,
            `      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_PATH}/post/responses/201`
          && /OverrideSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy catalog is bound to DraftRecord", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'`,
            `                items:
                  $ref: '#/components/schemas/${DRAFT_RECORD}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CATALOG_GET_PATH}/get/responses/200`
          && /CatalogEntry/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when governance overrides are bound to DraftRecord", () => {
    const result = evaluateOpenapiDraftRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OVERRIDE_SUMMARY}'`,
            `$ref: '#/components/schemas/${DRAFT_RECORD}'`,
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

  it("passes when every draft op $ref the existing DraftRecord and Value fields stay open", () => {
    const result = evaluateOpenapiDraftRecord({
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
        /openapi draft-record typed-response gate passed/,
      );
    } else {
      assert.match(ran.stderr, /DraftRecord|additionalProperties|root additionalProperties bag/);
    }
  });
});
