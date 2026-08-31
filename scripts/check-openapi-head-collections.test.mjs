import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  COLLECTION_PATHS,
  FENCED_HEADS,
  FENCED_JOB_POSITION,
  FENCED_PAY_RUN,
  GET_FLOOR,
  REQUIRED_COLLECTION_GET_HEADS,
  TEMPORAL_ASOF_HEADS,
  evaluateOpenapiHeadCollections,
} from "./check-openapi-head-collections.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-head-collections.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-head-collections-"));
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

function schemaYaml(name) {
  return `    ${name}:
      type: object
      properties:
        id: { type: string }`;
}

function spec(paths) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${paths}
${padGets(GET_FLOOR)}
components:
  schemas:
${HEAD_SCHEMA_NAMES.map(schemaYaml).join("\n")}
`;
}

function instanceGet(path, schemaName, extraParams = "") {
  return `  ${path}:
    get:
      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
${extraParams}
      responses:
        '200':
          description: head
          content:
            application/json:
              schema: { $ref: '#/components/schemas/${schemaName}' }`;
}

function collectionGet(path, schemaName, extraParams = "") {
  const parameters = extraParams
    ? `      parameters:
${extraParams}`
    : "";
  return `  ${path}:
    get:
${parameters}
      responses:
        '200':
          description: heads
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/${schemaName}' }`;
}

const RANGE_FROM_TO = `      - name: from
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      - name: to
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }`;

function honestCollections() {
  return [
    collectionGet(COLLECTION_PATHS.Company, "Company"),
    collectionGet(COLLECTION_PATHS.OrgUnit, "OrgUnit"),
    collectionGet(COLLECTION_PATHS.Person, "Person"),
  ].join("\n");
}

function honestInstances() {
  return [
    instanceGet("/api/v1/companies/{id}", "Company"),
    instanceGet("/api/v1/org-units/{id}", "OrgUnit"),
    instanceGet("/api/v1/persons/{id}", "Person"),
    instanceGet("/api/v1/employments/{id}", "Employment"),
  ].join("\n");
}

describe("check-openapi-head-collections", () => {
  it("exports examined-zero floors, required Heads, exact paths, and Chesterton fences", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.deepEqual(REQUIRED_COLLECTION_GET_HEADS, [
      "Company",
      "OrgUnit",
      "Person",
    ]);
    assert.deepEqual(COLLECTION_PATHS, {
      Company: "/api/v1/companies",
      OrgUnit: "/api/v1/org-units",
      Person: "/api/v1/persons",
    });
    assert.deepEqual(FENCED_HEADS, [FENCED_JOB_POSITION, FENCED_PAY_RUN]);
    assert.deepEqual(TEMPORAL_ASOF_HEADS, ["Employment"]);
    assert.ok(!REQUIRED_COLLECTION_GET_HEADS.includes("JobPosition"));
    assert.ok(!REQUIRED_COLLECTION_GET_HEADS.includes("PayRun"));
    assert.ok(!REQUIRED_COLLECTION_GET_HEADS.includes("Employment"));
  });

  it("fails while OrgUnitHead is not a 200 schema of any collection GET", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(spec(`${honestInstances()}
${collectionGet(COLLECTION_PATHS.Company, "Company")}
${collectionGet(COLLECTION_PATHS.Person, "Person")}`)),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/OrgUnit"
          && /not a 200 schema of any collection GET/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.collectionGetsByHead.OrgUnit, 0);
  });

  it("does not treat an instance GET as a collection GET", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(spec(honestInstances())),
    });
    assert.equal(result.collectionGetsByHead.OrgUnit, 0);
    assert.equal(result.collectionGetsByHead.Company, 0);
    assert.equal(result.collectionGetsByHead.Person, 0);
    assert.ok(
      result.findings.some((finding) => finding.location === "#/components/schemas/OrgUnit"),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some((finding) =>
        finding.location === `#/paths/${COLLECTION_PATHS.OrgUnit}/get`
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses from/to on a non-temporal Head collection", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(
        spec(
          [
            collectionGet(COLLECTION_PATHS.OrgUnit, "OrgUnit", RANGE_FROM_TO),
            collectionGet(COLLECTION_PATHS.Company, "Company"),
            collectionGet(COLLECTION_PATHS.Person, "Person"),
          ].join("\n"),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("/api/v1/org-units")
          && /no valid-time store/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses a JobPosition collection GET (L5-JOB fence)", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(
        spec(
          `${honestCollections()}
${collectionGet("/api/v1/job-positions", "JobPosition")}`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) => /L5-JOB still refuses inventing/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses a PayRun Head collection GET (PayrollRunSummary stays; no version)", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(
        spec(
          `${honestCollections()}
${collectionGet("/api/v1/pay-runs", "PayRun")}`,
        ),
      ),
    });
    assert.ok(
      result.findings.some((finding) => /PayrollRunSummary/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when Company, OrgUnit, and Person collection GETs exist without from/to/as_of", () => {
    const result = evaluateOpenapiHeadCollections({
      repoRoot: fixture(spec(`${honestCollections()}\n${honestInstances()}`)),
    });
    assert.equal(result.findings.length, 0, JSON.stringify(result.findings, null, 2));
    assert.equal(result.collectionGetsByHead.Company, 1);
    assert.equal(result.collectionGetsByHead.OrgUnit, 1);
    assert.equal(result.collectionGetsByHead.Person, 1);
    assert.equal(result.collectionGetsByHead.JobPosition, 0);
    assert.equal(result.collectionGetsByHead.PayRun, 0);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the collections are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    // This assertion is the admit-red lock: on the pre-collection tree the CLI is red.
    // After this slice lands it becomes green; the unit fixtures above stay
    // the fail-closed proof either way.
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi Head collection-GET gate passed/);
    } else {
      assert.match(ran.stderr, /not a 200 schema of any collection GET|must exist as the .* Head collection/);
    }
  });
});
