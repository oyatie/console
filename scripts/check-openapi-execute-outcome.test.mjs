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
  BOUND,
  COMMAND_RECEIPT,
  DISPATCH_ENUM,
  EXECUTE_FIELDS,
  EXECUTE_OUTCOME,
  EXECUTE_PATH,
  EXECUTE_REQUIRED,
  EXECUTE_STRUCT,
  GATE_CHAIN_OUTCOME,
  INSTANCE_STATE,
  KILL_SWITCH_PATH,
  LIFECYCLE_PREFLIGHT_PATH,
  PREFLIGHT_OUTCOME,
  PREFLIGHT_PATH,
  RECEIPT_FIELDS,
  VALUE_FIELD,
  WRITE_FLOOR,
  evaluateOpenapiExecuteOutcome,
} from "./check-openapi-execute-outcome.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-execute-outcome.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-execute-outcome-"));
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
  return `    ${EXECUTE_OUTCOME}:
      type: object
      required:
      - dispatch
      - gates
      properties:
        dispatch:
          type: string
          enum:
          - instance_revision
          - projected_usecase
        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }
        instance: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
        projected: { type: object, additionalProperties: true }
        receipt: { $ref: '#/components/schemas/${COMMAND_RECEIPT}' }
    ${COMMAND_RECEIPT}:
      type: object
      required:
      - command_id
      - payload_digest
      - instance
      - gates
      properties:
        command_id: { type: string, format: uuid }
        payload_digest: { type: string }
        instance: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }
    ${GATE_CHAIN_OUTCOME}:
      type: object
      properties:
        allow: { type: boolean }
        gates:
          type: array
          items: { type: object, additionalProperties: true }
    ${INSTANCE_STATE}:
      type: object
      properties:
        instance: { type: object }
        revision: { type: object }
    ${PREFLIGHT_OUTCOME}:
      type: object
      properties:
        dispatch: { type: string }
        would_execute: { type: boolean }
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

const HOLD_NEIGHBORS = `  ${PREFLIGHT_PATH}:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'
  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
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
  /api/v1/ontology/object-types/{key}:
    get:
      operationId: getObjectType
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED_WRAPPER = `  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXECUTE_OUTCOME}'
${HOLD_NEIGHBORS}`;

function unpublishedSchemas() {
  return spec(UNTYPED_WRAPPER).replace(
    `        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }
        instance: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
        projected: { type: object, additionalProperties: true }`,
    `        gates: { type: object, additionalProperties: true }
        instance: { type: object, additionalProperties: true }
        projected: { type: object, additionalProperties: true }`,
  ).replace(
    `        instance: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
        gates: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }`,
    `        instance: { type: object, additionalProperties: true }
        gates: { type: object, additionalProperties: true }`,
  );
}

const TYPED = `  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${EXECUTE_OUTCOME}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-execute-outcome", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(EXECUTE_PATH, "/api/v1/ontology/actions/{action_key}/execute");
    assert.equal(PREFLIGHT_PATH, "/api/v1/ontology/actions/{action_key}/preflight");
    assert.equal(EXECUTE_OUTCOME, "OntologyActionExecuteOutcome");
    assert.equal(EXECUTE_STRUCT, "ExecuteOutcome");
    assert.equal(PREFLIGHT_OUTCOME, "PreflightOutcome");
    assert.equal(VALUE_FIELD, "projected");
    assert.equal(BOUND, 1);
    assert.deepEqual(EXECUTE_FIELDS, [
      "dispatch",
      "gates",
      "instance",
      "projected",
      "receipt",
    ]);
    assert.deepEqual(EXECUTE_REQUIRED, ["dispatch", "gates"]);
    assert.deepEqual(RECEIPT_FIELDS, [
      "command_id",
      "payload_digest",
      "instance",
      "gates",
    ]);
    assert.deepEqual(DISPATCH_ENUM, ["instance_revision", "projected_usecase"]);
    assert.deepEqual(
      rustStructFields(
        `pub struct ExecuteOutcome {
    pub dispatch: ActionDispatch,
    pub gates: GateChainOutcome,
    pub instance: Option<InstanceState>,
    pub projected: Option<Value>,
    pub receipt: Option<CommandReceipt>,
}
`,
        "ExecuteOutcome",
      ),
      EXECUTE_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while execute 200 wrapper leaves gates/instance unpublished bags", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(unpublishedSchemas()),
    });
    assert.equal(result.bound, 1);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}/properties/gates`
          && /GateChainOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}/properties/instance`
          && /InstanceState/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when OntologyActionExecuteOutcome is missing from composed schemas", () => {
    const result = evaluateOpenapiExecuteOutcome({
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
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}`
          && /OntologyActionExecuteOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on OntologyActionExecuteOutcome", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        receipt: { $ref: '#/components/schemas/OntologyActionCommandReceipt' }",
          "        receipt: { $ref: '#/components/schemas/OntologyActionCommandReceipt' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when projected is closed into a catalog", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        projected: { type: object, additionalProperties: true }",
          "        projected:\n          type: object\n          additionalProperties: false\n          properties:\n            kind: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}/properties/${VALUE_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when projected $ref PreflightOutcome", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        projected: { type: object, additionalProperties: true }",
          `        projected: { $ref: '#/components/schemas/${PREFLIGHT_OUTCOME}' }`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${EXECUTE_OUTCOME}/properties/${VALUE_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST execute is bound to PreflightOutcome", () => {
    const result = evaluateOpenapiExecuteOutcome({
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
          && /PreflightOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST preflight is bound to OntologyActionExecuteOutcome", () => {
    const result = evaluateOpenapiExecuteOutcome({
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
          && /PreflightOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST execute (Feature::ALL)", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction`,
            `  ${EXECUTE_PATH}:
    post:
      operationId: executeOntologyAction
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXECUTE_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST execute is bound as a Head", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${EXECUTE_OUTCOME}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${EXECUTE_PATH}/post/responses/200`
          && /Company|Head HOLD/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch is bound to OntologyActionExecuteOutcome", () => {
    const result = evaluateOpenapiExecuteOutcome({
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
                $ref: '#/components/schemas/${EXECUTE_OUTCOME}'`,
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

  it("passes when 200 $ref the existing ExecuteOutcome and projected stays open", () => {
    const result = evaluateOpenapiExecuteOutcome({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until non-Value fields are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi execute-outcome typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /GateChainOutcome|InstanceState|projected|additionalProperties/);
    }
  });
});
