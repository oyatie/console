import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  GENERATED_FLOOR,
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  NESTED_INPUT_SCHEMAS,
} from "./check-openapi-semantic-generate.mjs";
import {
  HEAD_TYPE_FLOOR,
  INPUT_TYPE_FLOOR,
  NESTED_TYPE_FLOOR,
  OPERATION_FLOOR,
  evaluateGeneratedDocs,
} from "./check-openapi-generated-docs.mjs";
import {
  DOCS_FILE_RELS,
  DOCS_REL,
  generateDocsFiles,
} from "./generate-openapi-docs.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-generated-docs.mjs", import.meta.url));
const generateCli = fileURLToPath(new URL("./generate-openapi-docs.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-generated-docs-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function headSchema(name) {
  const properties = {
    Company: {
      org_id: { $ref: "#/components/schemas/Uuid" },
      legal_name: { type: ["string", "null"] },
      reg_no: { type: ["string", "null"] },
      version: { type: "integer" },
    },
    OrgUnit: {
      id: { $ref: "#/components/schemas/Uuid" },
      name: { type: ["string", "null"] },
      parent_id: { $ref: "#/components/schemas/Uuid" },
      version: { type: "integer" },
    },
    JobPosition: {
      job_position_id: { $ref: "#/components/schemas/Uuid" },
      org_unit_id: { $ref: "#/components/schemas/Uuid" },
      version: { type: "integer" },
      attributes: { type: "object" },
    },
    Person: {
      id: { $ref: "#/components/schemas/Uuid" },
      display_name: { type: ["string", "null"] },
      legal_name: { type: ["string", "null"] },
      version: { type: "integer" },
    },
    Employment: {
      id: { $ref: "#/components/schemas/Uuid" },
      person_id: { $ref: "#/components/schemas/Uuid" },
      org_unit_id: { $ref: "#/components/schemas/Uuid" },
      job_position_id: { $ref: "#/components/schemas/Uuid" },
      appointed_on: { $ref: "#/components/schemas/Timestamp" },
      version: { type: "integer" },
    },
    PayRun: {
      id: { $ref: "#/components/schemas/Uuid" },
      period_start: { type: "string", format: "date" },
      period_end: { type: "string", format: "date" },
      source_label: { type: "string" },
      status: { type: "string" },
      payable: { type: "boolean", const: false },
    },
  };
  const fields = properties[name] ?? { id: { type: "string" } };
  return {
    type: "object",
    required: Object.keys(fields),
    properties: fields,
    links: [],
    actions: name === "Company"
      ? [{ action_key: "company.revise", input: { $ref: "#/components/schemas/CompanyReviseInput" } }]
      : [],
  };
}

function inputSchema(name) {
  if (name === "EmploymentAttributesInput") {
    return {
      type: "object",
      additionalProperties: false,
      required: ["company", "employment_status"],
      properties: {
        company: { type: "string" },
        employment_status: { type: "string" },
      },
    };
  }
  if (name === "OrgUnitSourceBinding") {
    return {
      type: "object",
      additionalProperties: false,
      required: ["kind", "id"],
      properties: {
        kind: { type: "string" },
        id: { type: "string" },
      },
    };
  }
  return {
    type: "object",
    additionalProperties: false,
    required: ["id"],
    properties: {
      id: { $ref: "#/components/schemas/Uuid" },
    },
  };
}

function publishedDoc(overrides = {}) {
  const schemas = {
    Uuid: { type: "string", format: "uuid" },
    Timestamp: { type: "string", format: "date-time" },
  };
  for (const name of GENERATED_SCHEMA_NAMES) {
    schemas[name] = HEAD_SCHEMA_NAMES.includes(name) ? headSchema(name) : inputSchema(name);
  }
  Object.assign(schemas, overrides);
  return yaml.dump({
    openapi: "3.1.0",
    info: { title: "Fixture", version: "0.0.1" },
    paths: {
      "/api/v1/ontology/objects/{objectKey}/{id}/actions/{actionKey}/execute": {
        post: {
          operationId: "executeOntologyAction",
          summary: "Execute a typed action",
        },
      },
    },
    components: { schemas },
  });
}

function greenFiles(extra = {}) {
  const openapi = publishedDoc();
  const generated = generateDocsFiles(yaml.load(openapi));
  return {
    "backend/openapi/openapi.yaml": openapi,
    ...generated.files,
    ...extra,
  };
}

describe("generated-docs floors", () => {
  it("locks examined-zero to 6 Heads + 13 Inputs + 2 nested + 1 file", () => {
    assert.equal(HEAD_TYPE_FLOOR, 6);
    assert.equal(INPUT_TYPE_FLOOR, 13);
    assert.equal(NESTED_TYPE_FLOOR, 2);
    assert.equal(OPERATION_FLOOR, 1);
    assert.equal(GENERATED_SCHEMA_NAMES.length, GENERATED_FLOOR);
    assert.equal(HEAD_SCHEMA_NAMES.length, 6);
    assert.equal(INPUT_SCHEMA_NAMES.length, 13);
    assert.equal(NESTED_INPUT_SCHEMAS.length, 2);
    assert.equal(DOCS_FILE_RELS.length, 1);
  });
});

describe("evaluateGeneratedDocs", () => {
  it("fails when the generated docs artifact is absent", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": publishedDoc(),
    });
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /generated docs file is absent/.test(finding.message)),
      JSON.stringify(result.findings),
    );
    assert.equal(result.files, 0);
  });

  it("fails when committed docs bytes drifted from a regen", () => {
    const files = greenFiles();
    files[DOCS_REL] = `${files[DOCS_REL]}\n<!-- hand-written -->\n`;
    const root = fixture(files);
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /drifted from composed OpenAPI/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when a Head section is missing from the docs", () => {
    const openapi = publishedDoc();
    const generated = generateDocsFiles(yaml.load(openapi));
    generated.files[DOCS_REL] = generated.files[DOCS_REL].replace(
      /<section id="schema-Person"[\s\S]*?<\/section>/,
      "",
    );
    const root = fixture({
      "backend/openapi/openapi.yaml": openapi,
      ...generated.files,
    });
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("Person")
          && /do not describe this Head\/Input/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("fails when OpenAPI Head properties are not in the docs", () => {
    const files = greenFiles();
    const doc = yaml.load(files["backend/openapi/openapi.yaml"]);
    doc.components.schemas.Company.properties.nickname = { type: "string" };
    files["backend/openapi/openapi.yaml"] = yaml.dump(doc);
    const root = fixture(files);
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some(
        (finding) =>
          /drifted from composed schema properties/.test(finding.message)
          && /nickname/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("fails when PayRun.payable is not literal false", () => {
    const files = greenFiles();
    const doc = yaml.load(files["backend/openapi/openapi.yaml"]);
    doc.components.schemas.PayRun.properties.payable = { type: "boolean" };
    delete doc.components.schemas.PayRun.properties.payable.const;
    files["backend/openapi/openapi.yaml"] = yaml.dump(doc);
    const generated = generateDocsFiles(doc);
    Object.assign(files, generated.files);
    const root = fixture(files);
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /PayRun\.payable must stay literal false/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when Person grows a forbidden field", () => {
    const files = greenFiles();
    const doc = yaml.load(files["backend/openapi/openapi.yaml"]);
    doc.components.schemas.Person.properties.phone = { type: "string" };
    files["backend/openapi/openapi.yaml"] = yaml.dump(doc);
    const generated = generateDocsFiles(doc);
    Object.assign(files, generated.files);
    const root = fixture(files);
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /must not grow phone/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when a composed operation is omitted from the docs", () => {
    const files = greenFiles();
    files[DOCS_REL] = files[DOCS_REL].replace(
      /<tr data-operation="POST \/api\/v1\/ontology\/objects\/\{objectKey\}\/\{id\}\/actions\/\{actionKey\}\/execute"[\s\S]*?<\/tr>/,
      "",
    );
    const root = fixture(files);
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /omit this composed OpenAPI operation/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("passes a fixture whose docs were generated from the composed OpenAPI", () => {
    const root = fixture(greenFiles());
    const result = evaluateGeneratedDocs({ repoRoot: root });
    assert.deepEqual(result.findings, []);
    assert.equal(result.heads, HEAD_TYPE_FLOOR);
    assert.equal(result.inputs, INPUT_TYPE_FLOOR);
    assert.equal(result.nested, NESTED_TYPE_FLOOR);
    assert.equal(result.operations, 1);
    assert.equal(result.files, DOCS_FILE_RELS.length);
  });
});

describe("cli", () => {
  it("accepts the live document only when the generated docs match composed OpenAPI", () => {
    const run = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
    assert.match(run.stdout, /generated-docs gate passed/);
  });

  it("regen of the live docs is a no-op", () => {
    const run = spawnSync(process.execPath, [generateCli, "--write", repoRoot], {
      encoding: "utf8",
    });
    assert.equal(run.status, 0, run.stderr);
    const result = evaluateGeneratedDocs({ repoRoot });
    assert.deepEqual(result.findings, []);
    const diff = spawnSync(
      "git",
      ["-C", repoRoot, "diff", "--exit-code", "--", "sdk/docs"],
      { encoding: "utf8" },
    );
    assert.equal(diff.status, 0, diff.stdout + diff.stderr);
  });
});

describe("generateDocsFiles", () => {
  it("property rows and operations come from OpenAPI, not a parallel catalog", () => {
    const doc = yaml.load(publishedDoc());
    doc.components.schemas.Company.properties.nickname = { type: "string" };
    doc.paths["/api/v1/extra"] = { get: { operationId: "extraOp" } };
    const generated = generateDocsFiles(doc);
    const html = generated.files[DOCS_REL];
    assert.match(html, /data-property="nickname"/);
    assert.match(html, /data-schema="Company"/);
    assert.match(html, /data-schema="CompanyReviseInput"/);
    assert.match(html, /data-action="company.revise"/);
    assert.match(html, /data-operation="GET \/api\/v1\/extra"/);
    assert.match(html, /data-property="payable"[^>]*data-const="false"/);
  });
});
