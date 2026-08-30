import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  EMPLOYMENT_ASOF_FLOOR,
  GET_FLOOR,
  TEMPLATE_PATH,
  evaluateOpenapiHrAsof,
  isIntegerAsOfParam,
  isTimestampAsOfParam,
  operationReturnsSchema,
} from "./check-openapi-hr-asof.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-hr-asof.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-hr-asof-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function padGets(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad/${i}:
    get:
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

const INSTANCE_GET = `  ${TEMPLATE_PATH}:
    get:
      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
      - name: as_of
        in: query
        required: false
        description: RFC3339 instant for a bi-temporal as-of read; absent = current head.
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          description: Instance state.
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const EVIDENCE_GET = `  /api/v1/evidence/objects:
    get:
      parameters:
      - name: as_of
        in: query
        required: false
        schema: { type: integer, format: int64 }
      responses:
        '200': { description: ok }`;

const EMPLOYMENT_SCHEMA = `    Employment:
      type: object
      required: [id, appointed_on, person_id, org_unit_id, job_position_id]
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        appointed_on: { $ref: '#/components/schemas/Timestamp' }
        person_id: { oneOf: [{ $ref: '#/components/schemas/Uuid' }, { type: 'null' }] }
        org_unit_id: { oneOf: [{ $ref: '#/components/schemas/Uuid' }, { type: 'null' }] }
        job_position_id: { oneOf: [{ $ref: '#/components/schemas/Uuid' }, { type: 'null' }] }`;

const COMPONENTS = `components:
  schemas:
    Uuid: { type: string, format: uuid }
    Timestamp: { type: string, format: date-time }
${EMPLOYMENT_SCHEMA}`;

function spec(paths) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${INSTANCE_GET}
${EVIDENCE_GET}
${paths}
${padGets(GET_FLOOR)}
${COMPONENTS}
`;
}

const HONEST_EMPLOYMENT_GET = `  /api/v1/employments/{id}:
    get:
      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
      - name: as_of
        in: query
        required: false
        description: RFC3339 instant for a bi-temporal as-of read; absent = current head.
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          description: Employment head.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Employment' }`;

describe("openapi HR as-of published-query gate (P0.4)", () => {
  it("recognizes the instance-GET Timestamp as_of and rejects the evidence integer", () => {
    assert.equal(
      isTimestampAsOfParam({
        name: "as_of",
        in: "query",
        required: false,
        schema: { $ref: "#/components/schemas/Timestamp" },
      }),
      true,
    );
    assert.equal(
      isIntegerAsOfParam({
        name: "as_of",
        in: "query",
        schema: { type: "integer", format: "int64" },
      }),
      true,
    );
    assert.equal(
      isTimestampAsOfParam({
        name: "as_of",
        in: "query",
        schema: { type: "integer", format: "int64" },
      }),
      false,
    );
    assert.equal(
      operationReturnsSchema(
        { responses: { "200": { content: { "application/json": { schema: { $ref: "#/components/schemas/Employment" } } } } } },
        "Employment",
      ),
      true,
    );
  });

  it("reports Employment Head published with no GET as_of", () => {
    const root = fixture(spec(`  /api/v1/employees/{id}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/EmployeeDetail' }`));

    const { findings, employmentAsOf, employmentGets } = evaluateOpenapiHrAsof({ repoRoot: root });

    assert.equal(employmentGets, 0);
    assert.equal(employmentAsOf, 0);
    assert.ok(
      findings.some((finding) => /employment_revisions/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects an Employment GET that copies the evidence-register integer as_of", () => {
    const root = fixture(spec(`  /api/v1/employments/{id}:
    get:
      parameters:
      - name: as_of
        in: query
        schema: { type: integer, format: int64 }
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Employment' }`));

    const { findings } = evaluateOpenapiHrAsof({ repoRoot: root });

    assert.ok(
      findings.some((finding) => /not the evidence-register integer/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("accepts an Employment GET that copies the instance-GET Timestamp as_of", () => {
    const root = fixture(spec(HONEST_EMPLOYMENT_GET));

    const { findings, employmentAsOf, employmentGets } = evaluateOpenapiHrAsof({
      repoRoot: root,
    });

    assert.equal(employmentGets, 1);
    assert.equal(employmentAsOf, EMPLOYMENT_ASOF_FLOOR);
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("does not require from/to on the Employment GET when the template lacks them", () => {
    const root = fixture(spec(HONEST_EMPLOYMENT_GET));
    const { findings } = evaluateOpenapiHrAsof({ repoRoot: root });
    assert.equal(
      findings.some((finding) => /\bfrom\b|\bto\b/.test(finding.message)),
      false,
      JSON.stringify(findings, null, 2),
    );
  });

  it("exits 1 against a document whose Employment GET omits as_of", () => {
    const root = fixture(spec(`  /api/v1/employments/{id}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Employment' }`));

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /HR as-of gate FAILED/);
  });

  it("exits 1 loudly when the document is not parseable YAML at all", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });

  // The live document is the hole this lane closes: Employment Head is
  // published (P0.1) and employment_revisions history exists, but no GET
  // returns that Head with the instance-GET as_of query. Red on origin/dev;
  // green only after GET /api/v1/employments/{id}?as_of= is published.
  it("exits 0 against this repository with Employment GET Timestamp as_of", () => {
    const { findings, employmentAsOf, gets } = evaluateOpenapiHrAsof({ repoRoot });

    assert.ok(gets >= GET_FLOOR, `GET operations examined: ${gets}`);
    assert.equal(employmentAsOf, EMPLOYMENT_ASOF_FLOOR, `Employment GET as_of: ${employmentAsOf}`);
    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 8), null, 2));

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi HR as-of gate passed/);
  });
});
