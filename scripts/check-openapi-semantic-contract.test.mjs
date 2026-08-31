import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  ACTION_FLOOR,
  CANONICAL_ACTIONS,
  CANONICAL_LINKS,
  CANONICAL_OBJECTS,
  evaluateSemanticContract,
  LINK_FLOOR,
  OBJECT_FLOOR,
  RESULT_REF,
} from "./check-openapi-semantic-contract.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-semantic-contract.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-semantic-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function spec(body) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths: {}
${body}`;
}

function yamlEscape(value) {
  if (typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}

function linkYaml(link, indent) {
  const pad = " ".repeat(indent);
  return [
    `${pad}- key: ${link.key}`,
    `${pad}  from: ${link.from}`,
    `${pad}  to: ${link.to}`,
    `${pad}  field: ${link.field}`,
    `${pad}  cardinality: ${link.cardinality}`,
    `${pad}  option: ${link.option}`,
  ].join("\n");
}

function actionYaml(action, indent) {
  const pad = " ".repeat(indent);
  const edits = action.edits.map((table) => `${pad}    - ${table}`).join("\n");
  return [
    `${pad}- action_key: ${yamlEscape(action.action_key)}`,
    `${pad}  object_key: ${action.object_key}`,
    `${pad}  input:`,
    `${pad}    $ref: '#/components/schemas/${action.input}'`,
    `${pad}  result:`,
    `${pad}    $ref: '${RESULT_REF}'`,
    `${pad}  permissions:`,
    `${pad}    - role_manage`,
    `${pad}  four_eyes: ${action.four_eyes}`,
    `${pad}  edits:`,
    edits,
    `${pad}  concurrency:`,
    `${pad}    command_id: tenant_global_idempotency`,
    `${pad}    expected_revision: optional_cas`,
  ].join("\n");
}

function headProperties(name) {
  switch (name) {
    case "Company":
      return `        org_id: { type: string }
        version: { type: integer }
        legal_name: { type: [string, 'null'] }
        reg_no: { type: [string, 'null'] }`;
    case "OrgUnit":
      return `        id: { type: string }
        version: { type: integer }
        name: { type: [string, 'null'] }
        parent_id: { type: [string, 'null'] }`;
    case "JobPosition":
      return `        job_position_id: { type: string }
        org_unit_id: { type: string }
        version: { type: integer }
        attributes: { type: object }`;
    case "Person":
      return `        id: { type: string }
        version: { type: integer }
        display_name: { type: [string, 'null'] }
        legal_name: { type: [string, 'null'] }`;
    case "Employment":
      return `        id: { type: string }
        version: { type: integer }
        appointed_on: { type: string }
        person_id: { type: [string, 'null'] }
        org_unit_id: { type: [string, 'null'] }
        job_position_id: { type: [string, 'null'] }`;
    case "PayRun":
      return `        id: { type: string }
        period_start: { type: string }
        period_end: { type: string }
        source_label: { type: string }
        status: { type: string }
        payable: { type: boolean, const: false }`;
    default:
      return "";
  }
}

function requiredFields(name) {
  switch (name) {
    case "Company":
      return ["org_id", "version", "legal_name", "reg_no"];
    case "OrgUnit":
      return ["id", "version", "name", "parent_id"];
    case "JobPosition":
      return ["job_position_id", "org_unit_id", "version", "attributes"];
    case "Person":
      return ["id", "version", "display_name", "legal_name"];
    case "Employment":
      return ["id", "version", "appointed_on", "person_id", "org_unit_id", "job_position_id"];
    case "PayRun":
      return ["id", "period_start", "period_end", "source_label", "status", "payable"];
    default:
      return [];
  }
}

function inputSchemaYaml(action) {
  const extra = extraInputProperties(action.action_key);
  return `    ${action.input}:
      type: object
      required: ${JSON.stringify(extra.required)}
      properties:
${extra.properties}`;
}

function extraInputProperties(actionKey) {
  switch (actionKey) {
    case "company.revise":
      return {
        required: ["attributes"],
        properties: `        attributes:
          type: object
          required: [legal_name]
          properties:
            legal_name: { type: string }
            reg_no: { type: [string, 'null'] }`,
      };
    case "organization.create_org_unit":
      return {
        required: ["attributes"],
        properties: `        source:
          oneOf:
            - type: object
              required: [kind, id]
              properties:
                kind: { type: string }
                id: { type: string }
            - type: 'null'
        attributes:
          type: object
          required: [name, kind]
          properties:
            name: { type: string }
            kind: { type: string, enum: [site, department, team] }
            parent_id: { type: [string, 'null'] }`,
      };
    case "organization.revise_org_unit":
      return {
        required: ["org_unit_id", "attributes"],
        properties: `        org_unit_id: { type: string }
        source:
          oneOf:
            - type: object
              required: [kind, id]
              properties:
                kind: { type: string }
                id: { type: string }
            - type: 'null'
        attributes:
          type: object
          required: [name, kind]
          properties:
            name: { type: string }
            kind: { type: string, enum: [site, department, team] }
            parent_id: { type: [string, 'null'] }`,
      };
    case "organization.create_job_position":
      return {
        required: ["org_unit_id", "attributes"],
        properties: `        org_unit_id: { type: string }
        attributes:
          type: object
          required: [title]
          properties:
            title: { type: string }`,
      };
    case "organization.revise_job_position":
      return {
        required: ["job_position_id", "attributes"],
        properties: `        job_position_id: { type: string }
        org_unit_id:
          type: [string, 'null']
        attributes:
          type: object
          required: [title]
          properties:
            title: { type: string }`,
      };
    case "people.create_person":
      return {
        required: ["attributes"],
        properties: `        employee_id:
          type: [string, 'null']
        attributes:
          type: object
          properties:
            legal_name: { type: [string, 'null'] }
            display_name: { type: [string, 'null'] }`,
      };
    case "people.revise_person":
      return {
        required: ["person_id", "attributes"],
        properties: `        person_id: { type: string }
        employee_id:
          type: [string, 'null']
        attributes:
          type: object
          properties:
            legal_name: { type: [string, 'null'] }
            display_name: { type: [string, 'null'] }`,
      };
    case "hr.appoint":
      return {
        required: ["employee_id", "valid_from", "attributes"],
        properties: `        employee_id: { type: string }
        valid_from: { type: string, format: date-time }
        attributes:
          $ref: '#/components/schemas/EmploymentAttributesInput'`,
      };
    case "hr.promote":
    case "hr.transfer":
      return {
        required: ["employment_id", "valid_from", "attributes"],
        properties: `        employment_id: { type: string }
        valid_from: { type: string, format: date-time }
        attributes:
          $ref: '#/components/schemas/EmploymentAttributesInput'`,
      };
    case "payroll.create_run":
      return {
        required: ["run_id", "period_start", "period_end"],
        properties: `        run_id: { type: string }
        period_start: { type: string, format: date }
        period_end: { type: string, format: date }
        connector: { type: [string, 'null'] }
        job: { type: [string, 'null'] }`,
      };
    case "payroll.submit_run":
      return {
        required: ["run_id"],
        properties: `        run_id: { type: string }`,
      };
    case "payroll.decide_run":
      return {
        required: ["run_id", "decision"],
        properties: `        run_id: { type: string }
        decision: { type: string, enum: [APPROVE, REJECT] }
        reason: { type: [string, 'null'] }`,
      };
    default:
      return { required: ["x"], properties: "        x: { type: string }" };
  }
}

function honestDocument() {
  const objectBlocks = CANONICAL_OBJECTS.map((entry) => {
    const links = CANONICAL_LINKS.filter((link) => link.from === entry.name);
    const actions = CANONICAL_ACTIONS.filter((action) => action.object === entry.name);
    const required = requiredFields(entry.name)
      .map((field) => `        - ${field}`)
      .join("\n");
    const linksYaml =
      links.length === 0
        ? "      links: []"
        : `      links:\n${links.map((link) => linkYaml(link, 8)).join("\n")}`;
    const actionsYaml = `      actions:\n${actions.map((action) => actionYaml(action, 8)).join("\n")}`;
    return `    ${entry.name}:
      type: object
      required:
${required}
      properties:
${headProperties(entry.name)}
${linksYaml}
${actionsYaml}`;
  });

  const inputBlocks = CANONICAL_ACTIONS.map((action) => inputSchemaYaml(action));
  return spec(`components:
  schemas:
    OntologyActionExecuteOutcome:
      type: object
    OntologyActionRequest:
      type: object
      properties:
        params: { type: object, additionalProperties: true }
    EmploymentAttributesInput:
      type: object
      required: [company, employment_status]
      properties:
        company: { type: string }
        org_unit_id: { type: [string, 'null'] }
        job_position_id: { type: [string, 'null'] }
        employment_status: { type: string, enum: [ACTIVE, EXITED, UNKNOWN] }
${objectBlocks.join("\n")}
${inputBlocks.join("\n")}`);
}

describe("openapi semantic-contract gate (links + action contracts)", () => {
  it("locks the six PRODUCT objects, thirteen dispatch targets, and five Head FKs", () => {
    assert.equal(CANONICAL_OBJECTS.length, OBJECT_FLOOR);
    assert.equal(CANONICAL_ACTIONS.length, ACTION_FLOOR);
    assert.equal(CANONICAL_LINKS.length, LINK_FLOOR);
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

  it("reports missing links/actions on Head-only schemas (the #994 hole)", () => {
    const root = fixture(spec(`components:
  schemas:
    Company:
      type: object
      required: [org_id, version, legal_name, reg_no]
      properties:
        org_id: { type: string }
        version: { type: integer }
        legal_name: { type: [string, 'null'] }
        reg_no: { type: [string, 'null'] }
    OrgUnit:
      type: object
      required: [id, version, name, parent_id]
      properties:
        id: { type: string }
        version: { type: integer }
        name: { type: [string, 'null'] }
        parent_id: { type: [string, 'null'] }
    JobPosition:
      type: object
      required: [job_position_id, org_unit_id, version, attributes]
      properties:
        job_position_id: { type: string }
        org_unit_id: { type: string }
        version: { type: integer }
        attributes: { type: object }
    Person:
      type: object
      required: [id, version, display_name, legal_name]
      properties:
        id: { type: string }
        version: { type: integer }
        display_name: { type: [string, 'null'] }
        legal_name: { type: [string, 'null'] }
    Employment:
      type: object
      required: [id, version, appointed_on, person_id, org_unit_id, job_position_id]
      properties:
        id: { type: string }
        version: { type: integer }
        appointed_on: { type: string }
        person_id: { type: [string, 'null'] }
        org_unit_id: { type: [string, 'null'] }
        job_position_id: { type: [string, 'null'] }
    PayRun:
      type: object
      required: [id, period_start, period_end, source_label, status, payable]
      properties:
        id: { type: string }
        period_start: { type: string }
        period_end: { type: string }
        source_label: { type: string }
        status: { type: string }
        payable: { type: boolean, const: false }
`));

    const { findings, objects, actions, links } = evaluateSemanticContract({ repoRoot: root });

    assert.equal(objects, OBJECT_FLOOR);
    assert.equal(actions, 0);
    assert.equal(links, 0);
    assert.ok(findings.some((finding) => finding.location.endsWith("/Company/links")));
    assert.ok(findings.some((finding) => finding.location.endsWith("/Employment/actions")));
    assert.ok(findings.some((finding) => /Heads are not an ontology|schema-level/.test(finding.message)));
  });

  it("rejects links/actions stuffed under properties (instance-field fiction)", () => {
    const root = fixture(spec(`components:
  schemas:
    Company:
      type: object
      properties:
        org_id: { type: string }
        links: { type: array }
        actions: { type: array }
      links: []
      actions: []
    OrgUnit:
      type: object
      properties: { id: { type: string } }
      links: []
      actions: []
    JobPosition:
      type: object
      properties: { job_position_id: { type: string } }
      links: []
      actions: []
    Person:
      type: object
      properties: { id: { type: string } }
      links: []
      actions: []
    Employment:
      type: object
      properties: { id: { type: string } }
      links: []
      actions: []
    PayRun:
      type: object
      properties:
        payable: { type: boolean, const: false }
      links: []
      actions: []
`));

    const { findings } = evaluateSemanticContract({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /instance properties/.test(finding.message)),
      JSON.stringify(findings.slice(0, 6), null, 2),
    );
  });

  it("rejects OntologyActionRequest.params as an action input (type erasure)", () => {
    const honest = honestDocument();
    const sabotaged = honest.replace(
      "#/components/schemas/CompanyReviseInput",
      "#/components/schemas/OntologyActionRequest",
    );
    const root = fixture(sabotaged);
    const { findings } = evaluateSemanticContract({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /CompanyReviseInput/.test(finding.message)),
      JSON.stringify(findings.filter((finding) => /company.revise/.test(finding.location)), null, 2),
    );
  });

  it("accepts the six objects with Head FKs as links and thirteen typed action contracts", () => {
    const root = fixture(honestDocument());
    const { findings, objects, actions, links } = evaluateSemanticContract({ repoRoot: root });
    assert.equal(objects, OBJECT_FLOOR, JSON.stringify(findings, null, 2));
    assert.equal(actions, ACTION_FLOOR, JSON.stringify(findings, null, 2));
    assert.equal(links, LINK_FLOOR, JSON.stringify(findings, null, 2));
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("exits 1 against Head-only schemas", () => {
    const root = fixture(spec(`components:
  schemas:
    Company: { type: object }
    OrgUnit: { type: object }
    JobPosition: { type: object }
    Person: { type: object }
    Employment: { type: object }
    PayRun: { type: object }
`));
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /semantic-contract gate FAILED/);
    assert.match(result.stderr, /below the floor/);
  });

  it("exits 1 loudly when the document is not parseable YAML", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });

  // The live document is the hole this lane closes: six Head schemas exist and
  // lack schema-level links/actions. Red on origin/dev; green only after the
  // thirteen dispatch targets and five Head FKs are declared.
  it("exits 0 against this repository with declared links and action contracts", () => {
    const { findings, objects, actions, links } = evaluateSemanticContract({ repoRoot });

    assert.equal(objects, OBJECT_FLOOR, `objects: ${objects}`);
    assert.equal(actions, ACTION_FLOOR, `actions: ${actions}; ${JSON.stringify(findings.slice(0, 8), null, 2)}`);
    assert.equal(links, LINK_FLOOR, `links: ${links}`);
    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 12), null, 2));

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi semantic-contract gate passed/);
  });
});
