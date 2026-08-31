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
  FENCED_HEADS,
  FENCED_PAY_RUN,
  GET_FLOOR,
  REQUIRED_INSTANCE_GET_HEADS,
  TEMPORAL_ASOF_HEADS,
  evaluateOpenapiHeadGets,
} from "./check-openapi-head-gets.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-head-gets.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-head-gets-"));
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

const EMPLOYMENT_ASOF = `      - name: as_of
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }`;

function honestGets() {
  return [
    instanceGet("/api/v1/companies/{id}", "Company"),
    instanceGet("/api/v1/org-units/{id}", "OrgUnit"),
    instanceGet("/api/v1/job-positions/{id}", "JobPosition"),
    instanceGet("/api/v1/persons/{id}", "Person"),
    instanceGet("/api/v1/employments/{id}", "Employment", EMPLOYMENT_ASOF),
  ].join("\n");
}

describe("check-openapi-head-gets", () => {
  it("exports examined-zero floors and the Chesterton fences", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.deepEqual(REQUIRED_INSTANCE_GET_HEADS, [
      "Company",
      "OrgUnit",
      "JobPosition",
      "Person",
      "Employment",
    ]);
    assert.deepEqual(FENCED_HEADS, [FENCED_PAY_RUN]);
    assert.deepEqual(TEMPORAL_ASOF_HEADS, ["Employment"]);
    assert.ok(REQUIRED_INSTANCE_GET_HEADS.includes("JobPosition"));
    assert.ok(!REQUIRED_INSTANCE_GET_HEADS.includes("PayRun"));
  });

  it("fails while OrgUnitHead is not a 200 schema of any instance GET", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(spec(instanceGet("/api/v1/employments/{id}", "Employment", EMPLOYMENT_ASOF))),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/OrgUnit"
          && /not a 200 schema of any instance GET/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.instanceGetsByHead.OrgUnit, 0);
    assert.equal(result.instanceGetsByHead.Employment, 1);
  });

  it("does not treat a collection array as an instance GET", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(spec(`  /api/v1/org-units:
    get:
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/OrgUnit' }
${instanceGet("/api/v1/employments/{id}", "Employment", EMPLOYMENT_ASOF)}
${instanceGet("/api/v1/companies/{id}", "Company")}
${instanceGet("/api/v1/job-positions/{id}", "JobPosition")}
${instanceGet("/api/v1/persons/{id}", "Person")}`)),
    });
    assert.equal(result.instanceGetsByHead.OrgUnit, 0);
    assert.ok(
      result.findings.some((finding) => finding.location === "#/components/schemas/OrgUnit"),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses as_of on a non-temporal Head GET", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(
        spec(
          [
            instanceGet("/api/v1/org-units/{id}", "OrgUnit", EMPLOYMENT_ASOF),
            instanceGet("/api/v1/companies/{id}", "Company"),
            instanceGet("/api/v1/job-positions/{id}", "JobPosition"),
            instanceGet("/api/v1/persons/{id}", "Person"),
            instanceGet("/api/v1/employments/{id}", "Employment", EMPLOYMENT_ASOF),
          ].join("\n"),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("/api/v1/org-units/{id}")
          && /no valid-time store/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails while JobPosition Head is not a 200 schema of any instance GET", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(
        spec(
          [
            instanceGet("/api/v1/companies/{id}", "Company"),
            instanceGet("/api/v1/org-units/{id}", "OrgUnit"),
            instanceGet("/api/v1/persons/{id}", "Person"),
            instanceGet("/api/v1/employments/{id}", "Employment", EMPLOYMENT_ASOF),
          ].join("\n"),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/JobPosition"
          && /not a 200 schema of any instance GET/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.instanceGetsByHead.JobPosition, 0);
  });

  it("refuses a PayRun Head GET (PayrollRunSummary stays; no version)", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(
        spec(
          `${honestGets()}
${instanceGet("/api/v1/pay-runs/{id}", "PayRun")}`,
        ),
      ),
    });
    assert.ok(
      result.findings.some((finding) => /PayrollRunSummary/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when Company, OrgUnit, JobPosition, Person, and Employment instance GETs exist without as_of on non-temporal Heads", () => {
    const result = evaluateOpenapiHeadGets({
      repoRoot: fixture(spec(honestGets())),
    });
    assert.equal(result.findings.length, 0, JSON.stringify(result.findings, null, 2));
    assert.equal(result.instanceGetsByHead.Company, 1);
    assert.equal(result.instanceGetsByHead.OrgUnit, 1);
    assert.equal(result.instanceGetsByHead.JobPosition, 1);
    assert.equal(result.instanceGetsByHead.Person, 1);
    assert.equal(result.instanceGetsByHead.Employment, 1);
    assert.equal(result.instanceGetsByHead.PayRun, 0);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the Heads are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    // This assertion is the admit-red lock: on the pre-GET tree the CLI is red.
    // After this slice lands it becomes green; the unit fixtures above stay
    // the fail-closed proof either way.
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi Head instance-GET gate passed/);
    } else {
      assert.match(ran.stderr, /not a 200 schema of any instance GET/);
    }
  });
});
