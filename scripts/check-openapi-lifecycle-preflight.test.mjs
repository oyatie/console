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
  AUTHORIZE_PATH,
  BOUND,
  DECISION_RESPONSE,
  DRAFTS_GET_PATH,
  GATE_CHAIN_CONFIG,
  GATE_CHAIN_OUTCOME,
  HANDLER_STRUCT,
  LIFECYCLE_PREFLIGHT,
  LIFECYCLE_PREFLIGHT_PATH,
  OBJECT_TYPE_GET_PATH,
  PREFLIGHT_FIELDS,
  PREFLIGHT_OUTCOME,
  SIMULATE_PATH,
  STORE_STRUCT,
  TENANT_CONTEXT_PATH,
  WRITE_FLOOR,
  evaluateOpenapiLifecyclePreflight,
} from "./check-openapi-lifecycle-preflight.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-lifecycle-preflight.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-lifecycle-preflight-"));
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
  return `    ${LIFECYCLE_PREFLIGHT}:
      type: object
      required:
      - configured
      - config
      - outcome
      properties:
        configured: { type: boolean }
        config: { $ref: '#/components/schemas/${GATE_CHAIN_CONFIG}' }
        outcome: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }
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
    ${PREFLIGHT_OUTCOME}:
      type: object
      properties:
        would_execute: { type: boolean }
        dispatch: { type: string }
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

const HOLD_NEIGHBORS = `  ${PREFLIGHT_PATH}:
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
  ${AUTHORIZE_PATH}:
    post:
      operationId: authorizePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DECISION_RESPONSE}'
  ${SIMULATE_PATH}:
    post:
      operationId: simulatePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DECISION_RESPONSE}'
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
  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-lifecycle-preflight", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(LIFECYCLE_PREFLIGHT_PATH, "/api/v1/governance/lifecycle/preflight");
    assert.equal(LIFECYCLE_PREFLIGHT, "LifecyclePreflight");
    assert.equal(STORE_STRUCT, "LifecyclePreflight");
    assert.equal(HANDLER_STRUCT, "PreflightResponse");
    assert.equal(PREFLIGHT_OUTCOME, "PreflightOutcome");
    assert.equal(BOUND, 1);
    assert.deepEqual(PREFLIGHT_FIELDS, ["configured", "config", "outcome"]);
    assert.deepEqual(
      rustStructFields(
        `pub struct LifecyclePreflight {
    pub configured: bool,
    pub config: GateChainConfig,
    pub outcome: GateChainOutcome,
}
`,
        "LifecyclePreflight",
      ),
      PREFLIGHT_FIELDS,
    );
    assert.deepEqual(
      rustStructFields(
        `struct PreflightResponse {
    configured: bool,
    config: GateChainConfig,
    outcome: GateChainOutcome,
}
`,
        "PreflightResponse",
      ),
      PREFLIGHT_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST lifecycle preflight 200 stays additionalProperties", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when LifecyclePreflight is missing from composed schemas", () => {
    const result = evaluateOpenapiLifecyclePreflight({
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
          finding.location === `#/components/schemas/${LIFECYCLE_PREFLIGHT}`
          && /LifecyclePreflight/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        outcome: { $ref: '#/components/schemas/GateChainOutcome' }",
          "        outcome: { $ref: '#/components/schemas/GateChainOutcome' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${LIFECYCLE_PREFLIGHT}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST lifecycle preflight (Feature::ALL)", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition`,
            `  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST lifecycle preflight is bound as a Head", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`
          && /Company|Head HOLD/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST lifecycle preflight is bound to ontology PreflightOutcome", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`
          && /PreflightOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ontology action preflight is bound to LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
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

  it("fails when group-admin tenant-context is bound to LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`,
            `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`
          && /GroupAdminTenantContextStartResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
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

  it("fails when outcome is an open bag instead of GateChainOutcome", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        outcome: { $ref: '#/components/schemas/${GATE_CHAIN_OUTCOME}' }`,
          "        outcome: { type: object, additionalProperties: true }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${LIFECYCLE_PREFLIGHT}/properties/outcome`
          && /GateChainOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when 200 $ref the existing LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecyclePreflight({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi lifecycle-preflight typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /LifecyclePreflight|additionalProperties/);
    }
  });
});
