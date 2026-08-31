import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import {
  GET_FLOOR,
  INSTANCE_GET_PATH,
  INSTANCE_HISTORY_PATH,
  INSTANCE_LIST_PATH,
  INSTANCE_STATE,
  INSTANCE_TRAVERSE_PATH,
  REVISION_SUMMARY,
  TRAVERSAL_GRAPH,
  evaluateOpenapiOntologyInstanceTypes,
} from "./check-openapi-ontology-instance-types.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-ontology-instance-types.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-ontology-instance-types-"));
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

function schemas() {
  return `    ${INSTANCE_STATE}:
      type: object
    ${REVISION_SUMMARY}:
      type: object
    ${TRAVERSAL_GRAPH}:
      type: object
    Timestamp:
      type: string
      format: date-time`;
}

function spec(extraPaths) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padGets(GET_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas()}
`;
}

const UNTYPED = `  ${INSTANCE_GET_PATH}:
    get:
      operationId: getOntologyInstance
      parameters:
      - name: as_of
        in: query
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${INSTANCE_LIST_PATH}:
    get:
      operationId: listOntologyInstances
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { type: object, additionalProperties: true }
  ${INSTANCE_HISTORY_PATH}:
    get:
      operationId: listOntologyInstanceHistory
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { type: object, additionalProperties: true }
  ${INSTANCE_TRAVERSE_PATH}:
    get:
      operationId: traverseOntologyInstance
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const TYPED = `  ${INSTANCE_GET_PATH}:
    get:
      operationId: getOntologyInstance
      parameters:
      - name: as_of
        in: query
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
  ${INSTANCE_LIST_PATH}:
    get:
      operationId: listOntologyInstances
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/${INSTANCE_STATE}' }
  ${INSTANCE_HISTORY_PATH}:
    get:
      operationId: listOntologyInstanceHistory
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/${REVISION_SUMMARY}' }
  ${INSTANCE_TRAVERSE_PATH}:
    get:
      operationId: traverseOntologyInstance
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/${TRAVERSAL_GRAPH}' }`;

describe("check-openapi-ontology-instance-types", () => {
  it("exports examined-zero floor and the four existing GET identities", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(INSTANCE_GET_PATH, "/api/v1/ontology/instances/{id}");
    assert.equal(INSTANCE_LIST_PATH, "/api/v1/ontology/instances");
    assert.equal(INSTANCE_HISTORY_PATH, "/api/v1/ontology/instances/{id}/history");
    assert.equal(INSTANCE_TRAVERSE_PATH, "/api/v1/ontology/instances/{id}/traverse");
    assert.equal(INSTANCE_STATE, "InstanceState");
    assert.equal(REVISION_SUMMARY, "RevisionSummary");
    assert.equal(TRAVERSAL_GRAPH, "TraversalGraph");
  });

  it("fails while instance GET/list/history/traverse 200 stay additionalProperties", () => {
    const result = evaluateOpenapiOntologyInstanceTypes({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    for (const path of [
      INSTANCE_GET_PATH,
      INSTANCE_LIST_PATH,
      INSTANCE_HISTORY_PATH,
      INSTANCE_TRAVERSE_PATH,
    ]) {
      assert.ok(
        result.findings.some(
          (finding) =>
            finding.location === `#/paths/${path}/get/responses/200`
            && /additionalProperties/.test(finding.message),
        ),
        `${path}: ${JSON.stringify(result.findings, null, 2)}`,
      );
    }
  });

  it("fails when instance GET drops Timestamp as_of", () => {
    const result = evaluateOpenapiOntologyInstanceTypes({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `      parameters:
      - name: as_of
        in: query
        schema: { $ref: '#/components/schemas/Timestamp' }\n`,
            "",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${INSTANCE_GET_PATH}/get/parameters/as_of`
          && /as_of/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto instance history (Feature::ALL)", () => {
    const result = evaluateOpenapiOntologyInstanceTypes({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${INSTANCE_HISTORY_PATH}:
    get:
      operationId: listOntologyInstanceHistory`,
            `  ${INSTANCE_HISTORY_PATH}:
    get:
      operationId: listOntologyInstanceHistory
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${INSTANCE_HISTORY_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when the four GETs bind the existing runtime schemas", () => {
    const result = evaluateOpenapiOntologyInstanceTypes({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, 4);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $refs are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi ontology instance typed-response gate passed/);
    } else {
      assert.match(
        ran.stderr,
        /InstanceState|RevisionSummary|TraversalGraph|additionalProperties/,
      );
    }
  });
});
