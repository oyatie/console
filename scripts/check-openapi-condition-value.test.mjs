import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  AUTHORING_RS_REL,
  BOUND,
  CONDITION_FIELDS,
  CONDITION_OP_VARIANTS,
  CONDITION_STRUCT,
  CONDITION_VALUE,
  CONDITION_VALUE_CONTENT,
  CONDITION_VALUE_ENUM,
  CONDITION_VALUE_TAG,
  CONDITION_VALUE_VARIANTS,
  EFFECT_VARIANTS,
  POLICY_NO_CODE_BLOCKS,
  POLICY_NO_CODE_CONDITION,
  WRITE_FLOOR,
  conditionValueVariantSchemaName,
  evaluateOpenapiConditionValue,
  rustTaggedContentEnumInfo,
} from "./check-openapi-condition-value.mjs";
import { KILL_SWITCH_PATH } from "./check-openapi-execute-outcome.mjs";
import { toSnakeCase } from "./check-openapi-gate-outcome.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-condition-value.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml, rustSource) {
  const root = mkdtempSync(join(tmpdir(), "openapi-condition-value-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  if (typeof rustSource === "string") {
    const rustPath = join(root, AUTHORING_RS_REL);
    mkdirSync(dirname(rustPath), { recursive: true });
    writeFileSync(rustPath, rustSource);
  }
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

const CLOSED_RUST = `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Permit,
    Forbid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOp {
    Eq,
    Ne,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum ConditionValue {
    Literal(String),
    SubjectAttr(String),
    Bool(bool),
}

pub struct Condition {
    pub attr: String,
    pub op: ConditionOp,
    pub value: ConditionValue,
}

pub struct NoCodeBlocks {
    pub effect: Effect,
    pub action: String,
    pub resource_type: String,
    pub conditions: Vec<Condition>,
}
`;

function valueVariantsYaml() {
  return CONDITION_VALUE_VARIANTS.map((wire) => {
    const name = conditionValueVariantSchemaName(wire);
    const payloadType = wire === "bool" ? "boolean" : "string";
    return `    ${name}:
      type: object
      required: [kind, value]
      properties:
        kind: { type: string, enum: [${wire}] }
        value: { type: ${payloadType} }`;
  }).join("\n");
}

function conditionSchemas({ value = `$ref: '#/components/schemas/${CONDITION_VALUE}'`, extraKind = "" } = {}) {
  const kindEnum = [...CONDITION_VALUE_VARIANTS, extraKind].filter(Boolean);
  const mappingWires = extraKind
    ? [...CONDITION_VALUE_VARIANTS, extraKind]
    : [...CONDITION_VALUE_VARIANTS];
  return `    ${POLICY_NO_CODE_BLOCKS}:
      type: object
      required: [effect, action, resource_type]
      properties:
        effect: { type: string, enum: [permit, forbid] }
        action: { type: string }
        resource_type: { type: string }
        conditions:
          type: array
          items: { $ref: '#/components/schemas/${POLICY_NO_CODE_CONDITION}' }
    ${POLICY_NO_CODE_CONDITION}:
      type: object
      required: [attr, op, value]
      properties:
        attr: { type: string }
        op: { type: string, enum: [eq, ne, contains] }
        value: { ${value} }
    ${CONDITION_VALUE}:
      oneOf:
${CONDITION_VALUE_VARIANTS.map((wire) => `      - $ref: '#/components/schemas/${conditionValueVariantSchemaName(wire)}'`).join("\n")}
      discriminator:
        propertyName: kind
        mapping:
${mappingWires.map((wire) => `          ${wire}: '#/components/schemas/${conditionValueVariantSchemaName(wire)}'`).join("\n")}
${valueVariantsYaml()}
    AuditRecord:
      type: object
      properties:
        action: { type: string }
    Company:
      type: object
      properties:
        org_id: { type: string }
`;
}

function spec(extraPaths, schemaOptions) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padWrites(WRITE_FLOOR)}
${extraPaths}
components:
  schemas:
${conditionSchemas(schemaOptions)}
`;
}

const HOLD_NEIGHBORS = `  /api/v1/policy/drafts:
    post:
      operationId: createPolicyDraft
      responses:
        '201':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/DraftRecord' }
  /api/v1/policy/drafts/{draft_id}:
    put:
      operationId: updatePolicyDraft
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/DraftRecord' }
  /api/v1/ontology/actions/{action_key}/preflight:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  /api/v1/ontology/actions/{action_key}/execute:
    post:
      operationId: executeOntologyAction
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
  /api/v1/console/rollout/opt-in:
    put:
      operationId: putConsoleRolloutOptIn
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  /api/v1/console/rollout/org-flag:
    put:
      operationId: putConsoleRolloutOrgFlag
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  /api/audit:
    get:
      operationId: getAudit
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

describe("check-openapi-condition-value", () => {
  it("exports examined-zero floor and the existing serde wire names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(POLICY_NO_CODE_CONDITION, "PolicyNoCodeCondition");
    assert.equal(CONDITION_VALUE, "ConditionValue");
    assert.equal(CONDITION_VALUE_TAG, "kind");
    assert.equal(CONDITION_VALUE_CONTENT, "value");
    assert.equal(BOUND, 1);
    assert.deepEqual(CONDITION_FIELDS, ["attr", "op", "value"]);
    assert.deepEqual(CONDITION_VALUE_VARIANTS, ["literal", "subject_attr", "bool"]);
    assert.deepEqual(CONDITION_OP_VARIANTS, ["eq", "ne", "contains"]);
    assert.deepEqual(EFFECT_VARIANTS, ["permit", "forbid"]);
    assert.equal(toSnakeCase("SubjectAttr"), "subject_attr");
    assert.deepEqual(rustTaggedContentEnumInfo(CLOSED_RUST, CONDITION_VALUE_ENUM), {
      open: false,
      variants: [...CONDITION_VALUE_VARIANTS],
      tag: CONDITION_VALUE_TAG,
      content: CONDITION_VALUE_CONTENT,
    });
    assert.equal(
      rustTaggedContentEnumInfo(
        CLOSED_RUST.replace("Bool(bool)", "Unknown(String)"),
        CONDITION_VALUE_ENUM,
      ).open,
      true,
    );
    assert.deepEqual(
      rustStructFields(CLOSED_RUST, CONDITION_STRUCT),
      CONDITION_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while PolicyNoCodeCondition.value stays an unpublished bag", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(HOLD_NEIGHBORS, { value: "type: object, additionalProperties: true" }),
        CLOSED_RUST,
      ),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${POLICY_NO_CODE_CONDITION}/properties/value`
          && /ConditionValue/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ConditionValue is missing from composed schemas", () => {
    const yaml = spec(HOLD_NEIGHBORS).replace(
      new RegExp(`    ${CONDITION_VALUE}:[\\s\\S]*?    AuditRecord:`),
      "    AuditRecord:",
    );
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(yaml, CLOSED_RUST),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${CONDITION_VALUE}`
          && /unpublished/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails closed on an invented ConditionValue kind", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(spec(HOLD_NEIGHBORS, { extraKind: "regex" }), CLOSED_RUST),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          /invented catalog/.test(finding.message)
          && /regex/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails closed if Rust ConditionValue becomes Unknown(String)", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(HOLD_NEIGHBORS),
        CLOSED_RUST.replace("Bool(bool)", "Unknown(String)"),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.endsWith(`:${CONDITION_VALUE_ENUM}`)
          && /open/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when AuditRecord.action is bound to ConditionValue", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(HOLD_NEIGHBORS).replace(
          `    AuditRecord:
      type: object
      properties:
        action: { type: string }`,
          `    AuditRecord:
      type: object
      properties:
        action: { $ref: '#/components/schemas/${CONDITION_VALUE}' }`,
        ),
        CLOSED_RUST,
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/AuditRecord/properties/action"
          && /TEXT/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when PolicyNoCodeBlocks.action is closed into an invented catalog", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(HOLD_NEIGHBORS).replace(
          `        effect: { type: string, enum: [permit, forbid] }
        action: { type: string }
        resource_type: { type: string }`,
          `        effect: { type: string, enum: [permit, forbid] }
        action: { type: string, enum: [view, edit, read_field] }
        resource_type: { type: string }`,
        ),
        CLOSED_RUST,
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${POLICY_NO_CODE_BLOCKS}/properties/action`
          && /TEXT/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch 200 is bound to ConditionValue", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(HOLD_NEIGHBORS).replace(
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
                $ref: '#/components/schemas/${CONDITION_VALUE}'`,
        ),
        CLOSED_RUST,
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

  it("fails when permissions leak onto POST drafts (Feature::ALL)", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(
        spec(
          HOLD_NEIGHBORS.replace(
            `  /api/v1/policy/drafts:
    post:
      operationId: createPolicyDraft`,
            `  /api/v1/policy/drafts:
    post:
      operationId: createPolicyDraft
      permissions:
      - role_manage`,
          ),
        ),
        CLOSED_RUST,
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/paths//api/v1/policy/drafts/post/permissions"
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when value $ref ConditionValue and enums match serde", () => {
    const result = evaluateOpenapiConditionValue({
      repoRoot: fixture(spec(HOLD_NEIGHBORS), CLOSED_RUST),
    });
    assert.equal(result.bound, BOUND);
    assert.deepEqual(result.findings, []);
    assert.ok(result.writes >= WRITE_FLOOR);
  });
});

describe("check-openapi-condition-value CLI", () => {
  it("exits non-zero on the unpublished-bag fixture", () => {
    const root = fixture(
      spec(HOLD_NEIGHBORS, { value: "type: object, additionalProperties: true" }),
      CLOSED_RUST,
    );
    const ran = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.notEqual(ran.status, 0);
    assert.match(ran.stderr, /FAILED/);
  });
});
