import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import {
  BOUND,
  GET_FLOOR,
  OBJECT_TYPES_LIST_PATH,
  OBJECT_TYPE_GET_PATH,
  OBJECT_TYPE_RESPONSE,
  OBJECT_TYPE_SUMMARY,
  evaluateOpenapiObjectTypeList,
} from "./check-openapi-object-type-list.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-object-type-list.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-object-type-list-"));
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
  return `    ${OBJECT_TYPE_SUMMARY}:
      type: object
    ${OBJECT_TYPE_RESPONSE}:
      type: object`;
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

const UNTYPED_GET = `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { type: object, additionalProperties: true }
${UNTYPED_GET}`;

const TYPED = `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}' }
${UNTYPED_GET}`;

describe("check-openapi-object-type-list", () => {
  it("exports examined-zero floor and the existing list GET identity", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(OBJECT_TYPES_LIST_PATH, "/api/v1/ontology/object-types");
    assert.equal(OBJECT_TYPE_GET_PATH, "/api/v1/ontology/object-types/{key}");
    assert.equal(OBJECT_TYPE_SUMMARY, "ObjectTypeSummary");
    assert.equal(OBJECT_TYPE_RESPONSE, "ObjectTypeResponse");
    assert.equal(BOUND, 1);
  });

  it("fails while list 200 items stay additionalProperties", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPES_LIST_PATH}/get/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ObjectTypeSummary is missing from composed schemas", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    ${OBJECT_TYPE_RESPONSE}: { type: object }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${OBJECT_TYPE_SUMMARY}`
          && /ObjectTypeSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto listObjectTypes (Feature::ALL)", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes`,
            `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPES_LIST_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when listObjectTypes grows invented query params", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      responses:`,
            `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      parameters:
      - name: as_of
        in: query
        schema: { type: string }
      responses:`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPES_LIST_PATH}/get/parameters`
          && /Query/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET by key is bound to ObjectTypeSummary (wrong runtime type)", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            "schema: { type: object, additionalProperties: true }",
            `schema: { $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}' }`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`
          && /ObjectTypeDetail/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when list items $ref the existing ObjectTypeSummary", () => {
    const result = evaluateOpenapiObjectTypeList({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi object-type list typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /ObjectTypeSummary|additionalProperties/);
    }
  });
});
