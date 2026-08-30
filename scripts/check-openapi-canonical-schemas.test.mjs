import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  CANONICAL_OBJECT_SCHEMAS,
  evaluateCanonicalSchemas,
  jsonSchemaAdmitsNull,
  NAME_FLOOR,
} from "./check-openapi-canonical-schemas.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-canonical-schemas.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-canonical-"));
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

function honestSchema(name) {
  const specFor = CANONICAL_OBJECT_SCHEMAS.find((entry) => entry.name === name);
  const properties = {};
  for (const field of specFor.required) {
    if (specFor.nullable.includes(field)) {
      properties[field] = { type: ["string", "null"] };
    } else if (name === "PayRun" && field === "payable") {
      properties[field] = { type: "boolean", const: false };
    } else if (name === "JobPosition" && field === "attributes") {
      properties[field] = { type: "object" };
    } else {
      properties[field] = { type: "string" };
    }
  }
  return {
    type: "object",
    required: [...specFor.required],
    properties,
  };
}

function honestDocument() {
  const schemas = {};
  for (const entry of CANONICAL_OBJECT_SCHEMAS) {
    schemas[entry.name] = honestSchema(entry.name);
  }
  return spec(`components:
  schemas:
${Object.entries(schemas)
    .map(([name, body]) => `    ${name}:\n${yamlBlock(body, 6)}`)
    .join("")}`);
}

function yamlBlock(node, indent) {
  const pad = " ".repeat(indent);
  const inner = [];
  if (node.type) inner.push(`${pad}type: ${formatType(node.type)}`);
  if (Object.hasOwn(node, "const")) inner.push(`${pad}const: ${node.const}`);
  if (Array.isArray(node.required)) {
    inner.push(`${pad}required:`);
    for (const field of node.required) inner.push(`${pad}- ${field}`);
  }
  if (node.properties) {
    inner.push(`${pad}properties:`);
    for (const [key, value] of Object.entries(node.properties)) {
      inner.push(`${pad}  ${key}:`);
      inner.push(yamlBlock(value, indent + 4));
    }
  }
  return `${inner.join("\n")}\n`;
}

function formatType(type) {
  if (Array.isArray(type)) return `[${type.map((item) => (item === "null" ? "'null'" : item)).join(", ")}]`;
  return type;
}

describe("openapi canonical public-schema gate (P0.1)", () => {
  it("names exactly the six PRODUCT objects", () => {
    assert.equal(CANONICAL_OBJECT_SCHEMAS.length, NAME_FLOOR);
    assert.deepEqual(
      CANONICAL_OBJECT_SCHEMAS.map((entry) => entry.name),
      ["Company", "OrgUnit", "JobPosition", "Person", "Employment", "PayRun"],
    );
  });

  it("reports each missing canonical name on an otherwise empty document", () => {
    const root = fixture(spec(`components:
  schemas:
    CreateObjectTypeDraft:
      type: object
    OntologyActionRequest:
      type: object
      properties:
        params: { type: object, additionalProperties: true }
`));

    const { findings, named } = evaluateCanonicalSchemas({ repoRoot: root });

    assert.equal(named, 0);
    assert.equal(findings.length, NAME_FLOOR, JSON.stringify(findings, null, 2));
    for (const name of ["Company", "OrgUnit", "JobPosition", "Person", "Employment", "PayRun"]) {
      assert.ok(
        findings.some((finding) => finding.location.endsWith(`/${name}`)),
        `missing finding for ${name}: ${JSON.stringify(findings)}`,
      );
    }
  });

  it("does not treat DirectoryPerson / PayrollRunSummary as the canonical names", () => {
    const root = fixture(spec(`components:
  schemas:
    DirectoryPerson: { type: object }
    HrOrgChartCompany: { type: object }
    OrgUnitReference: { type: object }
    PayrollRunSummary: { type: object }
`));

    const { named, findings } = evaluateCanonicalSchemas({ repoRoot: root });

    assert.equal(named, 0);
    assert.equal(findings.length, NAME_FLOOR);
  });

  it("accepts the six honest Head schemas, including PayRun payable const false", () => {
    const root = fixture(honestDocument());

    const { findings, named } = evaluateCanonicalSchemas({ repoRoot: root });

    assert.equal(named, NAME_FLOOR);
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("rejects PayRun.payable when it is a free boolean (SAP-complete fiction)", () => {
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
      required: [id, appointed_on, person_id, org_unit_id, job_position_id]
      properties:
        id: { type: string }
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
        payable: { type: boolean }
`));

    const { findings } = evaluateCanonicalSchemas({ repoRoot: root });

    assert.ok(
      findings.some((finding) => finding.location.includes("PayRun") && /const: false/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects a Company that omits legal_name / reg_no Option fields", () => {
    const root = fixture(spec(`components:
  schemas:
    Company:
      type: object
      required: [org_id, version]
      properties:
        org_id: { type: string }
        version: { type: integer }
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
      required: [id, appointed_on, person_id, org_unit_id, job_position_id]
      properties:
        id: { type: string }
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

    const { findings } = evaluateCanonicalSchemas({ repoRoot: root });

    assert.ok(findings.some((finding) => /legal_name/.test(finding.message)));
    assert.ok(findings.some((finding) => /reg_no/.test(finding.message)));
  });

  it("jsonSchemaAdmitsNull matches type arrays and oneOf", () => {
    assert.equal(jsonSchemaAdmitsNull({ type: ["string", "null"] }), true);
    assert.equal(jsonSchemaAdmitsNull({ type: "string" }), false);
    assert.equal(
      jsonSchemaAdmitsNull({ oneOf: [{ $ref: "#/components/schemas/Uuid" }, { type: "null" }] }),
      true,
    );
  });

  it("exits 1 against a document missing the six names", () => {
    const root = fixture(spec(`components:
  schemas:
    Todo:
      type: object
`));

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /below the floor/);
    assert.match(result.stderr, /canonical-schema gate FAILED/);
  });

  it("exits 1 loudly when the document is not parseable YAML at all", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });

  // The live document is the hole this lane closes: none of the six PRODUCT
  // names exist as components.schemas. This assertion is red on origin/dev and
  // green only after those six Head schemas are published.
  it("exits 0 against this repository with all six canonical Head schemas", () => {
    const { findings, named } = evaluateCanonicalSchemas({ repoRoot });

    assert.equal(named, NAME_FLOOR, `canonical names present: ${named}`);
    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 8), null, 2));

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi canonical-schema gate passed/);
  });
});
