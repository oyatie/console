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
  GATE_CHAIN_OUTCOME,
  GATE_KIND,
  GATE_KIND_ENUM,
  GATE_KIND_VARIANTS,
  GATE_OUTCOME,
  GATE_OUTCOME_FIELDS,
  GATE_OUTCOME_STRUCT,
  GATE_RS_REL,
  GATE_STATUS,
  GATE_STATUS_ENUM,
  GATE_STATUS_TAG,
  GATE_STATUS_VARIANTS,
  WRITE_FLOOR,
  evaluateOpenapiGateOutcome,
  gateStatusVariantSchemaName,
  rustEnumInfo,
  toSnakeCase,
} from "./check-openapi-gate-outcome.mjs";
import { KILL_SWITCH_PATH } from "./check-openapi-execute-outcome.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-gate-outcome.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml, rustSource) {
  const root = mkdtempSync(join(tmpdir(), "openapi-gate-outcome-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  if (typeof rustSource === "string") {
    const rustPath = join(root, GATE_RS_REL);
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

const CLOSED_RUST = `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateKind {
    Authority,
    SelfChecklist,
    FourEyes,
    EgressDlp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GateStatus {
    NotRequired,
    Satisfied,
    Pending { reason: String },
    Denied { reason: String },
}

pub struct GateOutcome {
    pub gate: GateKind,
    pub status: GateStatus,
}
`;

function statusVariantsYaml() {
  return GATE_STATUS_VARIANTS.map((wire) => {
    const name = gateStatusVariantSchemaName(wire);
    const reason = wire === "pending" || wire === "denied"
      ? `
        reason: { type: string }`
      : "";
    const required = wire === "pending" || wire === "denied"
      ? `[status, reason]`
      : `[status]`;
    return `    ${name}:
      type: object
      required: ${required}
      properties:
        status: { type: string, enum: [${wire}] }${reason}`;
  }).join("\n");
}

function gateSchemas({ items = `$ref: '#/components/schemas/${GATE_OUTCOME}'`, extraKind = "" } = {}) {
  const kindEnum = [...GATE_KIND_VARIANTS, extraKind].filter(Boolean);
  return `    ${GATE_CHAIN_OUTCOME}:
      type: object
      required: [allow, gates]
      properties:
        allow: { type: boolean }
        gates:
          type: array
          items: { ${items} }
    ${GATE_OUTCOME}:
      type: object
      required:
      - gate
      - status
      properties:
        gate: { $ref: '#/components/schemas/${GATE_KIND}' }
        status: { $ref: '#/components/schemas/${GATE_STATUS}' }
    ${GATE_KIND}:
      type: string
      enum:
${kindEnum.map((value) => `      - ${value}`).join("\n")}
    ${GATE_STATUS}:
      oneOf:
${GATE_STATUS_VARIANTS.map((wire) => `      - $ref: '#/components/schemas/${gateStatusVariantSchemaName(wire)}'`).join("\n")}
      discriminator:
        propertyName: status
        mapping:
${GATE_STATUS_VARIANTS.map((wire) => `          ${wire}: '#/components/schemas/${gateStatusVariantSchemaName(wire)}'`).join("\n")}
${statusVariantsYaml()}
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
${gateSchemas(schemaOptions)}
`;
}

const HOLD_NEIGHBORS = `  /api/v1/ontology/actions/{action_key}/preflight:
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
  /api/v1/governance/lifecycle/preflight:
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
  /api/audit:
    get:
      operationId: getAudit
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const TYPED = `  /api/v1/ontology/actions/{action_key}/preflight:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS.replace(`  /api/v1/ontology/actions/{action_key}/preflight:
    post:
      operationId: preflightOntologyAction
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }\n`, "")}`;

describe("check-openapi-gate-outcome", () => {
  it("exports examined-zero floor and the existing serde wire names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(GATE_CHAIN_OUTCOME, "GateChainOutcome");
    assert.equal(GATE_OUTCOME, "GateOutcome");
    assert.equal(GATE_KIND, "GateKind");
    assert.equal(GATE_STATUS, "GateStatus");
    assert.equal(GATE_STATUS_TAG, "status");
    assert.equal(BOUND, 1);
    assert.deepEqual(GATE_OUTCOME_FIELDS, ["gate", "status"]);
    assert.deepEqual(GATE_KIND_VARIANTS, [
      "authority",
      "self_checklist",
      "four_eyes",
      "egress_dlp",
    ]);
    assert.deepEqual(GATE_STATUS_VARIANTS, [
      "not_required",
      "satisfied",
      "pending",
      "denied",
    ]);
    assert.equal(toSnakeCase("EgressDlp"), "egress_dlp");
    assert.deepEqual(rustEnumInfo(CLOSED_RUST, GATE_KIND_ENUM), {
      open: false,
      variants: [...GATE_KIND_VARIANTS],
      tag: null,
    });
    assert.deepEqual(rustEnumInfo(CLOSED_RUST, GATE_STATUS_ENUM), {
      open: false,
      variants: [...GATE_STATUS_VARIANTS],
      tag: GATE_STATUS_TAG,
    });
    assert.equal(rustEnumInfo(CLOSED_RUST.replace("EgressDlp", "Unknown(String)"), GATE_KIND_ENUM).open, true);
    assert.deepEqual(
      rustStructFields(CLOSED_RUST, GATE_OUTCOME_STRUCT),
      GATE_OUTCOME_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while GateChainOutcome.gates items stay unpublished bags", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(
        spec(TYPED, { items: "type: object, additionalProperties: true" }),
        CLOSED_RUST,
      ),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${GATE_CHAIN_OUTCOME}/properties/gates/items`
          && /GateOutcome/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GateKind is missing from composed schemas", () => {
    const yaml = spec(TYPED).replace(
      new RegExp(`    ${GATE_KIND}:[\\s\\S]*?    ${GATE_STATUS}:`),
      `    ${GATE_STATUS}:`,
    );
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(yaml, CLOSED_RUST),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GATE_KIND}`
          && /GateKind/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented GateKind variant appears", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(spec(TYPED, { extraKind: "kill_switch" }), CLOSED_RUST),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GATE_KIND}/enum`
          && /invented catalog/.test(finding.message)
          && /kill_switch/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented GateStatus mapping variant appears", () => {
    const yaml = spec(TYPED).replace(
      "          denied: '#/components/schemas/GateStatusDenied'",
      "          denied: '#/components/schemas/GateStatusDenied'\n          rollback: '#/components/schemas/GateStatusDenied'",
    );
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(yaml, CLOSED_RUST),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GATE_STATUS}/discriminator/mapping`
          && /rollback/.test(finding.message)
          && /invented catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GateKind is open in Rust", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(
        spec(TYPED),
        CLOSED_RUST.replace(
          "    EgressDlp,\n}",
          "    EgressDlp,\n    Unknown(String),\n}",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `${GATE_RS_REL}:${GATE_KIND_ENUM}`
          && /open/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch is bound to GateOutcome", () => {
    const result = evaluateOpenapiGateOutcome({
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
                $ref: '#/components/schemas/${GATE_OUTCOME}'`,
          ),
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

  it("fails when AuditRecord.action is bound to GateKind", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        action: { type: string }",
          `        action: { $ref: '#/components/schemas/${GATE_KIND}' }`,
        ),
        CLOSED_RUST,
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/AuditRecord/properties/action`
          && /TEXT/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST preflight (Feature::ALL)", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  /api/v1/ontology/actions/{action_key}/preflight:
    post:
      operationId: preflightOntologyAction`,
            `  /api/v1/ontology/actions/{action_key}/preflight:
    post:
      operationId: preflightOntologyAction
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
          finding.location
            === "#/paths//api/v1/ontology/actions/{action_key}/preflight/post/permissions"
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when gates[] $ref GateOutcome and enums match serde", () => {
    const result = evaluateOpenapiGateOutcome({
      repoRoot: fixture(spec(TYPED), CLOSED_RUST),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until items are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi gate-outcome typed-items gate passed/);
    } else {
      assert.match(ran.stderr, /GateOutcome|additionalProperties|invented catalog/);
    }
  });
});
