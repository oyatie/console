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
  REVERSE_CARDINALITY,
  REVERSE_FIELD,
  REVERSE_FROM,
  REVERSE_LINK_KEY,
  REVERSE_OPERATION_ID,
  REVERSE_OPTION,
  REVERSE_PATH,
  REVERSE_TO,
  evaluateOpenapiOrgUnitJobPositions,
} from "./check-openapi-orgunit-job-positions.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-orgunit-job-positions.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-orgunit-job-positions-"));
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
      operationId: padGet${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function schemaYaml(name, extraLinks = "") {
  const parent =
    name === "OrgUnit"
      ? `        - key: org_unit_parent
          from: OrgUnit
          to: OrgUnit
          field: parent_id
          cardinality: many-to-one
          option: true
          operationId: getOrgUnit
${extraLinks}`
      : "        []";
  return `    ${name}:
      type: object
      properties:
        id: { type: string }
      links:
${parent}`;
}

function spec(paths, orgUnitExtraLinks = "") {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${paths}
${padGets(GET_FLOOR)}
components:
  schemas:
${HEAD_SCHEMA_NAMES.map((name) => schemaYaml(name, name === "OrgUnit" ? orgUnitExtraLinks : "")).join("\n")}
`;
}

function reverseGet(extraParams = "", operationId = REVERSE_OPERATION_ID) {
  return `  ${REVERSE_PATH}:
    get:
      operationId: ${operationId}
      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
${extraParams}
      responses:
        '200':
          description: positions under the unit
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/${REVERSE_TO}' }`;
}

const HONEST_LINK = `        - key: ${REVERSE_LINK_KEY}
          from: ${REVERSE_FROM}
          to: ${REVERSE_TO}
          field: ${REVERSE_FIELD}
          cardinality: ${REVERSE_CARDINALITY}
          option: ${REVERSE_OPTION}
          operationId: ${REVERSE_OPERATION_ID}`;

describe("check-openapi-orgunit-job-positions", () => {
  it("exports examined-zero floor, reverse identity, and the PayRun fence", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(REVERSE_PATH, "/api/v1/org-units/{id}/job-positions");
    assert.equal(REVERSE_OPERATION_ID, "listOrgUnitJobPositions");
    assert.equal(REVERSE_LINK_KEY, "org_unit_job_positions");
    assert.equal(REVERSE_FROM, "OrgUnit");
    assert.equal(REVERSE_TO, "JobPosition");
    assert.equal(REVERSE_FIELD, "org_unit_id");
    assert.equal(REVERSE_CARDINALITY, "one-to-many");
    assert.equal(REVERSE_OPTION, false);
    assert.deepEqual(FENCED_HEADS, [FENCED_PAY_RUN]);
  });

  it("fails while GET /api/v1/org-units/{id}/job-positions is unpublished", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(spec("  /api/v1/job-positions:\n    get:\n      responses:\n        '200': { description: tenant list }")),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${REVERSE_PATH}/get`
          && /list_for_org_unit/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.reverse, false);
  });

  it("fails while OrgUnit Head has no reverse collection link", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(spec(reverseGet())),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/OrgUnit/links/${REVERSE_LINK_KEY}`
          && /listOrgUnitJobPositions/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.reverse, true);
  });

  it("fails while the reverse link binds getJobPosition (instance, not collection)", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(
        spec(
          reverseGet(),
          `        - key: ${REVERSE_LINK_KEY}
          from: ${REVERSE_FROM}
          to: ${REVERSE_TO}
          field: ${REVERSE_FIELD}
          cardinality: ${REVERSE_CARDINALITY}
          option: ${REVERSE_OPTION}
          operationId: getJobPosition`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/OrgUnit/links/${REVERSE_LINK_KEY}/operationId`
          && /getJobPosition/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses as_of on the reverse collection (JobPosition has no valid-time store)", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(
        spec(
          reverseGet(`      - name: as_of
        in: query
        schema: { $ref: '#/components/schemas/Timestamp' }`),
          HONEST_LINK,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes(`${REVERSE_PATH}/get/parameters`)
          && /no valid-time store/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.reverse, false);
  });

  it("refuses a PayRun Head as the reverse 200 schema", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(
        spec(`  /api/v1/org-units/{id}/pay-runs:
    get:
      responses:
        '200':
          description: invented
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/PayRun' }`),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/paths//api/v1/org-units/{id}/pay-runs/get"
          && /PayRun Head/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when the reverse collection GET and OrgUnit Head link bind listOrgUnitJobPositions", () => {
    const result = evaluateOpenapiOrgUnitJobPositions({
      repoRoot: fixture(spec(reverseGet(), HONEST_LINK)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.reverse, true);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the reverse collection is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi OrgUnit→JobPosition reverse-collection gate passed/);
    } else {
      assert.match(ran.stderr, /org-units\/\{id\}\/job-positions|list_for_org_unit|listOrgUnitJobPositions/);
    }
  });
});
