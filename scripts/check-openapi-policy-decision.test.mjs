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
  BULK_DECISION_RESPONSE,
  BULK_PATH,
  DECISION_FIELDS,
  DECISION_RESPONSE,
  DECISION_STRUCT,
  DRAFTS_GET_PATH,
  EFFECT_ENUM,
  LIFECYCLE_PREFLIGHT_PATH,
  OBJECT_TYPE_GET_PATH,
  OUTCOME_FIELDS,
  OUTCOME_STRUCT,
  PREFLIGHT_OUTCOME,
  SIMULATE_PATH,
  SIMULATION_OUTCOME,
  TENANT_CONTEXT_PATH,
  WRITE_FLOOR,
  evaluateOpenapiPolicyDecision,
} from "./check-openapi-policy-decision.mjs";
import { EXECUTE_PATH } from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-policy-decision.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-policy-decision-"));
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
  return `    ${DECISION_RESPONSE}:
      type: object
      required:
      - outcome
      properties:
        outcome: { $ref: '#/components/schemas/${SIMULATION_OUTCOME}' }
    ${SIMULATION_OUTCOME}:
      type: object
      required:
      - effect
      - determining_policies
      - errors
      - reason
      properties:
        effect:
          type: string
          enum:
          - allow
          - deny
        determining_policies: { type: array, items: { type: string } }
        errors: { type: array, items: { type: string } }
        reason: { type: string }
    ${BULK_DECISION_RESPONSE}:
      type: object
      required: [decisions]
      properties:
        decisions: { type: array, items: { $ref: '#/components/schemas/${SIMULATION_OUTCOME}' } }
    ${PREFLIGHT_OUTCOME}:
      type: object
      properties:
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

const HOLD_NEIGHBORS = `  ${BULK_PATH}:
    post:
      operationId: authorizeBulk
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${BULK_DECISION_RESPONSE}'
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
              schema: { type: object, additionalProperties: true }
  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
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

const UNTYPED = `  ${AUTHORIZE_PATH}:
    post:
      operationId: authorizePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${SIMULATE_PATH}:
    post:
      operationId: simulatePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${AUTHORIZE_PATH}:
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
${HOLD_NEIGHBORS}`;

describe("check-openapi-policy-decision", () => {
  it("exports examined-zero floor, paths, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(AUTHORIZE_PATH, "/api/v1/policy/authorize");
    assert.equal(SIMULATE_PATH, "/api/v1/policy/simulate");
    assert.equal(DECISION_RESPONSE, "DecisionResponse");
    assert.equal(DECISION_STRUCT, "DecisionResponse");
    assert.equal(SIMULATION_OUTCOME, "SimulationOutcome");
    assert.equal(OUTCOME_STRUCT, "SimulationOutcome");
    assert.equal(BOUND, 2);
    assert.deepEqual(DECISION_FIELDS, ["outcome"]);
    assert.deepEqual(OUTCOME_FIELDS, [
      "effect",
      "determining_policies",
      "errors",
      "reason",
    ]);
    assert.deepEqual(EFFECT_ENUM, ["allow", "deny"]);
    assert.deepEqual(
      rustStructFields(
        "struct DecisionResponse {\n    outcome: SimulationOutcome,\n}\n",
        "DecisionResponse",
      ),
      DECISION_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST authorize/simulate 200 stay additionalProperties", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUTHORIZE_PATH}/post/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${SIMULATE_PATH}/post/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when DecisionResponse is missing from composed schemas", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padWrites(WRITE_FLOOR)}
${TYPED}
components:
  schemas:
    ${SIMULATION_OUTCOME}:
      type: object
      required: [effect, determining_policies, errors, reason]
      properties:
        effect: { type: string, enum: [allow, deny] }
        determining_policies: { type: array, items: { type: string } }
        errors: { type: array, items: { type: string } }
        reason: { type: string }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DECISION_RESPONSE}`
          && /DecisionResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when DecisionResponse.outcome is not SimulationOutcome", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        outcome: { $ref: '#/components/schemas/${SIMULATION_OUTCOME}' }`,
          "        outcome: { type: object, additionalProperties: true }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DECISION_RESPONSE}/properties/outcome`
          && /SimulationOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when simulate 200 is bound to bare SimulationOutcome", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${SIMULATE_PATH}:
    post:
      operationId: simulatePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${DECISION_RESPONSE}'`,
            `  ${SIMULATE_PATH}:
    post:
      operationId: simulatePolicyDecision
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${SIMULATION_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${SIMULATE_PATH}/post/responses/200`
          && /SimulationOutcome|outcome wrapper/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        outcome: { $ref: '#/components/schemas/${SIMULATION_OUTCOME}' }`,
          `        outcome: { $ref: '#/components/schemas/${SIMULATION_OUTCOME}' }\n        invented_store: { type: string }`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${DECISION_RESPONSE}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST authorize (Feature::ALL)", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${AUTHORIZE_PATH}:
    post:
      operationId: authorizePolicyDecision`,
            `  ${AUTHORIZE_PATH}:
    post:
      operationId: authorizePolicyDecision
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUTHORIZE_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST authorize is bound as a Head", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${DECISION_RESPONSE}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUTHORIZE_PATH}/post/responses/200`
          && /Company|Head HOLD/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when governance lifecycle preflight is bound to DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
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
                $ref: '#/components/schemas/${DECISION_RESPONSE}'`,
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

  it("fails when group-admin tenant-context is bound to DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
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
                $ref: '#/components/schemas/${DECISION_RESPONSE}'`,
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

  it("fails when policy drafts are bound to DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${DECISION_RESPONSE}'`,
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

  it("fails when bulk authorize is bound to DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${BULK_DECISION_RESPONSE}'`,
            `$ref: '#/components/schemas/${DECISION_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${BULK_PATH}/post/responses/200`
          && /BulkDecisionResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when both 200s $ref the existing DecisionResponse", () => {
    const result = evaluateOpenapiPolicyDecision({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi policy-decision typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /DecisionResponse|additionalProperties/);
    }
  });
});
