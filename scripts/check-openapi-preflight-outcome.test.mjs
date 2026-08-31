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
  ABSENCE_EXIT_GET_PATH,
  BOUND,
  LIFECYCLE_PREFLIGHT_PATH,
  DISPATCH_ENUM,
  DRAFTS_GET_PATH,
  EXECUTE_OUTCOME,
  EXECUTE_PATH,
  GATE_CHAIN_CONFIG,
  GATE_CHAIN_OUTCOME,
  OBJECT_TYPE_GET_PATH,
  PREFLIGHT_FIELDS,
  PREFLIGHT_OUTCOME,
  PREFLIGHT_PATH,
  PREFLIGHT_REQUIRED,
  PREFLIGHT_STRUCT,
  WRITE_FLOOR,
  evaluateOpenapiPreflightOutcome,
} from "./check-openapi-preflight-outcome.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-preflight-outcome.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-preflight-outcome-"));
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
  return `    ${PREFLIGHT_OUTCOME}:
      type: object
      required:
      - dispatch
      - dispatch_target
      - config
      - gates
      - criteria_ok
      - would_execute
      properties:
        dispatch:
          type: string
          enum:
          - instance_revision
          - projected_usecase
        dispatch_target: { type: [string, 'null'] }
        config: { $ref: '#/components/schemas/${GATE_CHAIN_CONFIG}' }
        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }
        criteria_ok: { type: boolean }
        criteria_error: { type: string }
        would_execute: { type: boolean }
    ${GATE_CHAIN_CONFIG}:
      type: object
      properties:
        authority: { type: boolean }
        self_checklist: { type: boolean }
        four_eyes: { type: boolean }
        egress_dlp: { type: boolean }
    ${GATE_CHAIN_OUTCOME}:
      type: object
      properties:
        allow: { type: boolean }
        gates:
          type: array
          items: { type: object, additionalProperties: true }
    ${EXECUTE_OUTCOME}:
      type: object
      properties:
        dispatch: { type: string }
        gates: { type: object, additionalProperties: true }
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

const HOLD_NEIGHBORS = `  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXECUTE_OUTCOME}'
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
              schema: { type: object, additionalProperties: true }
  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-preflight-outcome", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(PREFLIGHT_PATH, "/api/v1/ontology/actions/{action_key}/preflight");
    assert.equal(EXECUTE_PATH, "/api/v1/ontology/actions/{action_key}/execute");
    assert.equal(PREFLIGHT_OUTCOME, "PreflightOutcome");
    assert.equal(PREFLIGHT_STRUCT, "PreflightOutcome");
    assert.equal(EXECUTE_OUTCOME, "OntologyActionExecuteOutcome");
    assert.equal(BOUND, 1);
    assert.deepEqual(PREFLIGHT_FIELDS, [
      "dispatch",
      "dispatch_target",
      "config",
      "gates",
      "criteria_ok",
      "criteria_error",
      "would_execute",
    ]);
    assert.deepEqual(PREFLIGHT_REQUIRED, [
      "dispatch",
      "dispatch_target",
      "config",
      "gates",
      "criteria_ok",
      "would_execute",
    ]);
    assert.deepEqual(DISPATCH_ENUM, ["instance_revision", "projected_usecase"]);
    assert.deepEqual(
      rustStructFields(
        `pub struct PreflightOutcome {
    pub dispatch: ActionDispatch,
    pub dispatch_target: Option<String>,
    pub config: GateChainConfig,
    pub gates: GateChainOutcome,
    pub criteria_ok: bool,
    pub criteria_error: Option<String>,
    pub would_execute: bool,
}
`,
        "PreflightOutcome",
      ),
      PREFLIGHT_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST preflight 200 stays additionalProperties", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${PREFLIGHT_PATH}/post/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when PreflightOutcome is missing from composed schemas", () => {
    const result = evaluateOpenapiPreflightOutcome({
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
          finding.location === `#/components/schemas/${PREFLIGHT_OUTCOME}`
          && /PreflightOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on PreflightOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        would_execute: { type: boolean }",
          "        would_execute: { type: boolean }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${PREFLIGHT_OUTCOME}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST preflight (Feature::ALL)", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction`,
            `  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${PREFLIGHT_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST preflight is bound as a Head", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${PREFLIGHT_PATH}/post/responses/200`
          && /Company|Head HOLD/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST preflight is bound to OntologyActionExecuteOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
            `$ref: '#/components/schemas/${EXECUTE_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${PREFLIGHT_PATH}/post/responses/200`
          && /OntologyActionExecuteOutcome|projected/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when execute 200 is bound to PreflightOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${EXECUTE_OUTCOME}'`,
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXECUTE_PATH}/post/responses/200`
          && /ExecuteOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when governance lifecycle preflight is bound to PreflightOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`,
            `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`
          && /LifecyclePreflight/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to PreflightOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
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

  it("fails when gates is an open bag instead of GateChainOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }`,
          "        gates: { type: object, additionalProperties: true }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${PREFLIGHT_OUTCOME}/properties/gates`
          && /GateChainOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when 200 $ref the existing PreflightOutcome", () => {
    const result = evaluateOpenapiPreflightOutcome({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi preflight-outcome typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /PreflightOutcome|additionalProperties/);
    }
  });
});
