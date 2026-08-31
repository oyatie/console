import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { CANONICAL_ACTIONS } from "./check-openapi-semantic-contract.mjs";
import {
  ACTION_FLOOR,
  EXECUTE_PATH,
  PREFLIGHT_PATH,
  evaluateTypedExecuteContract,
} from "./check-openapi-typed-execute.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-typed-execute.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-typed-execute-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function yamlEscape(value) {
  return JSON.stringify(value);
}

function inputSchemaYaml(action, extra = "") {
  return `    ${action.input}:
      type: object
      additionalProperties: false
      required:
      - marker
      properties:
        marker:
          type: string
${extra}`;
}

function paramsUnionYaml(inputs) {
  const refs = inputs
    .map((name) => `            - $ref: '#/components/schemas/${name}'`)
    .join("\n");
  return `        params:
          anyOf:
${refs}
            - type: object
              additionalProperties: true`;
}

function document({ inputs = CANONICAL_ACTIONS.map((action) => action.input), extraInputs = "" } = {}) {
  const uniqueInputs = [...new Set(inputs)];
  const inputBlock = CANONICAL_ACTIONS
    .filter((action) => uniqueInputs.includes(action.input))
    .map((action) => inputSchemaYaml(action))
    .join("\n");
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
  ${yamlEscape(EXECUTE_PATH)}:
    post:
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
  ${yamlEscape(PREFLIGHT_PATH)}:
    post:
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
components:
  schemas:
    OntologyActionRequest:
      type: object
      additionalProperties: false
      required:
      - object_type_id
      properties:
        object_type_id:
          type: string
${paramsUnionYaml(uniqueInputs)}
${inputBlock}${extraInputs}`;
}

describe("openapi typed-execute gate (execute request body $refs)", () => {
  it("locks the thirteen DispatchTarget input schema names", () => {
    assert.equal(CANONICAL_ACTIONS.length, ACTION_FLOOR);
    assert.deepEqual(
      CANONICAL_ACTIONS.map((action) => action.action_key),
      [
        "company.revise",
        "organization.create_org_unit",
        "organization.revise_org_unit",
        "organization.create_job_position",
        "organization.revise_job_position",
        "people.create_person",
        "people.revise_person",
        "hr.appoint",
        "hr.promote",
        "hr.transfer",
        "payroll.create_run",
        "payroll.submit_run",
        "payroll.decide_run",
      ],
    );
  });

  it("rejects OntologyActionRequest.params additionalProperties: true (type erasure)", () => {
    const root = fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
  ${yamlEscape(EXECUTE_PATH)}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
  ${yamlEscape(PREFLIGHT_PATH)}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
components:
  schemas:
    OntologyActionRequest:
      type: object
      properties:
        object_type_id: { type: string }
        params:
          type: object
          additionalProperties: true
`);
    const { findings, actions } = evaluateTypedExecuteContract({ repoRoot: root });
    assert.equal(actions, 0);
    assert.ok(
      findings.some((finding) => /erases the thirteen typed inputs/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("stays red when twelve of thirteen inputs are $ref'd (free-form remainder)", () => {
    const twelve = CANONICAL_ACTIONS.slice(0, 12).map((action) => action.input);
    const root = fixture(document({ inputs: twelve }));
    const { findings, actions } = evaluateTypedExecuteContract({ repoRoot: root });
    assert.ok(actions < ACTION_FLOOR, `actions=${actions}`);
    assert.ok(
      findings.some((finding) => /payroll.decide_run/.test(finding.location)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects typed inputs that omit additionalProperties: false", () => {
    const honest = document();
    const sabotaged = honest.replace(
      /CompanyReviseInput:\n      type: object\n      additionalProperties: false/,
      "CompanyReviseInput:\n      type: object",
    );
    const root = fixture(sabotaged);
    const { findings } = evaluateTypedExecuteContract({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /CompanyReviseInput\/additionalProperties/.test(finding.location)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("accepts execute+preflight envelopes whose params anyOf $refs all thirteen inputs", () => {
    const root = fixture(document());
    const { findings, actions } = evaluateTypedExecuteContract({ repoRoot: root });
    assert.equal(actions, ACTION_FLOOR, JSON.stringify(findings, null, 2));
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("exits 1 against the free-form execute codec", () => {
    const root = fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
  ${yamlEscape(EXECUTE_PATH)}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
components:
  schemas:
    OntologyActionRequest:
      type: object
      properties:
        params:
          type: object
          additionalProperties: true
`);
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /typed-execute gate FAILED/);
  });

  it("exits 1 loudly when the document is not parseable YAML", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });

  // The live document is the hole this lane closes: execute still $refs
  // OntologyActionRequest.params with additionalProperties: true.
  it("exits 0 against this repository only after typed execute params land", () => {
    const { findings, actions } = evaluateTypedExecuteContract({ repoRoot });
    assert.equal(
      actions,
      ACTION_FLOOR,
      `actions: ${actions}; ${JSON.stringify(findings.slice(0, 8), null, 2)}`,
    );
    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 12), null, 2));

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi typed-execute gate passed/);
  });
});
