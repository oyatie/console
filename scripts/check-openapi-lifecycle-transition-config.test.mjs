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
  CONFIG_FIELDS,
  CREATE_PATH,
  DECIDE_PATH,
  DECISION_RESPONSE,
  DRAFTS_GET_PATH,
  LIFECYCLE_PREFLIGHT,
  LIFECYCLE_PREFLIGHT_PATH,
  LIFECYCLE_STATE,
  LIFECYCLE_TRANSITION_CONFIG,
  OBJECT_TYPE_GET_PATH,
  OK_CODE,
  OVERRIDES_PATH,
  PREFLIGHT_OUTCOME,
  REQ_STRUCT,
  REQUIREMENT_FIELDS,
  STORE_STRUCT,
  TRANSITION_REQUIREMENTS,
  TRANSITIONS_PATH,
  WRITE_FLOOR,
  evaluateOpenapiLifecycleTransitionConfig,
} from "./check-openapi-lifecycle-transition-config.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-lifecycle-transition-config.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-lifecycle-transition-config-"));
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
  return `    ${LIFECYCLE_TRANSITION_CONFIG}:
      type: object
      required:
      - object_type_id
      - from_state
      - to_state
      - requirements
      properties:
        object_type_id: { $ref: '#/components/schemas/Uuid' }
        from_state: { $ref: '#/components/schemas/${LIFECYCLE_STATE}' }
        to_state: { $ref: '#/components/schemas/${LIFECYCLE_STATE}' }
        requirements: { $ref: '#/components/schemas/${TRANSITION_REQUIREMENTS}' }
    ${TRANSITION_REQUIREMENTS}:
      type: object
      required:
      - requires_reason
      - requires_four_eyes
      - requires_checklist
      properties:
        requires_reason: { type: boolean }
        requires_four_eyes: { type: boolean }
        requires_checklist: { type: boolean }
    ${LIFECYCLE_STATE}:
      type: string
      enum: [DRAFT, ACTIVE, LOCKED, ARCHIVED, DISPOSED]
    Uuid: { type: string, format: uuid }
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
    ${APPROVAL_SUMMARY}:
      type: object
      properties:
        decision: { type: string }
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
  ${OVERRIDES_PATH}:
    post:
      operationId: openGovernanceOverride
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

const UNTYPED = `  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition
      responses:
        '${OK_CODE}':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-lifecycle-transition-config", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(TRANSITIONS_PATH, "/api/v1/governance/lifecycle/transitions");
    assert.equal(LIFECYCLE_TRANSITION_CONFIG, "LifecycleTransitionConfig");
    assert.equal(STORE_STRUCT, "LifecycleTransitionConfig");
    assert.equal(REQ_STRUCT, "TransitionRequirements");
    assert.equal(OK_CODE, "201");
    assert.equal(BOUND, 1);
    assert.deepEqual(CONFIG_FIELDS, [
      "object_type_id",
      "from_state",
      "to_state",
      "requirements",
    ]);
    assert.deepEqual(REQUIREMENT_FIELDS, [
      "requires_reason",
      "requires_four_eyes",
      "requires_checklist",
    ]);
    assert.deepEqual(
      rustStructFields(
        `pub struct LifecycleTransitionConfig {
    pub object_type_id: Uuid,
    pub from_state: LifecycleState,
    pub to_state: LifecycleState,
    pub requirements: TransitionRequirements,
}
`,
        "LifecycleTransitionConfig",
      ),
      CONFIG_FIELDS,
    );
    assert.deepEqual(
      rustStructFields(
        `pub struct TransitionRequirements {
    pub requires_reason: bool,
    pub requires_four_eyes: bool,
    pub requires_checklist: bool,
}
`,
        "TransitionRequirements",
      ),
      REQUIREMENT_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST governance lifecycle transitions 201 stays additionalProperties", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TRANSITIONS_PATH}/post/responses/${OK_CODE}`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when LifecycleTransitionConfig is missing from composed schemas", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
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
          finding.location === `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}`
          && /LifecycleTransitionConfig/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on LifecycleTransitionConfig", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        requirements: { $ref: '#/components/schemas/${TRANSITION_REQUIREMENTS}' }`,
          `        requirements: { $ref: '#/components/schemas/${TRANSITION_REQUIREMENTS}' }\n        invented_store: { type: string }`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when requirements are flattened onto the request-shaped wire", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        requirements: { $ref: '#/components/schemas/${TRANSITION_REQUIREMENTS}' }`,
          "        requires_reason: { type: boolean }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          (
            finding.location === `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties`
            || finding.location
              === `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties/requirements`
          )
          && /flatten|invent|requirements/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST governance lifecycle transitions (Feature::ALL)", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition`,
            `  ${TRANSITIONS_PATH}:
    post:
      operationId: configureLifecycleTransition
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TRANSITIONS_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance lifecycle transitions is bound as a Head", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TRANSITIONS_PATH}/post/responses/${OK_CODE}`
          && /ObjectKey|Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance lifecycle transitions is bound to LifecyclePreflight", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TRANSITIONS_PATH}/post/responses/${OK_CODE}`
          && /LifecyclePreflight/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST governance lifecycle transitions is bound to ontology PreflightOutcome", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
            `$ref: '#/components/schemas/${PREFLIGHT_OUTCOME}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TRANSITIONS_PATH}/post/responses/${OK_CODE}`
          && /PreflightOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when lifecycle preflight is bound to LifecycleTransitionConfig", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'`,
            `$ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
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

  it("fails when decide-approval is bound to LifecycleTransitionConfig", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${APPROVAL_SUMMARY}'`,
            `$ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
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

  it("fails when policy drafts are bound to LifecycleTransitionConfig", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}'`,
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

  it("passes when 201 $ref the existing LifecycleTransitionConfig", () => {
    const result = evaluateOpenapiLifecycleTransitionConfig({
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
        /openapi lifecycle-transition-config typed-response gate passed/,
      );
    } else {
      assert.match(ran.stderr, /LifecycleTransitionConfig|additionalProperties/);
    }
  });
});
