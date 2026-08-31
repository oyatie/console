import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR, TEMPLATE_PATH } from "./check-openapi-hr-asof.mjs";
import {
  COLLECTION_FLOOR,
  COLLECTION_PATH,
  EMPLOYEE_DETAIL_PATH,
  EMPLOYEES_PATH,
  GET_FLOOR,
  evaluateOpenapiHrFromTo,
  isTimestampNamedParam,
  operationReturnsEmploymentArray,
} from "./check-openapi-hr-from-to.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-hr-from-to.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-hr-from-to-"));
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
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          description: Instance state.
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const EMPLOYMENT_SCHEMA = `    Employment:
      type: object
      required: [id, appointed_on, person_id, org_unit_id, job_position_id]
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
    Timestamp:
      type: string
      format: date-time
    Uuid:
      type: string
      format: uuid`;

const HONEST_COLLECTION = `  ${COLLECTION_PATH}:
    get:
      parameters:
      - name: from
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      - name: to
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          description: Employment heads overlapping [from, to).
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/Employment' }`;

const EMPLOYEES_GET = `  ${EMPLOYEES_PATH}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/EmployeePage' }
  ${EMPLOYEE_DETAIL_PATH}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/EmployeeDetail' }`;

function spec(extraPaths) {
  return `openapi: 3.1.0
paths:
${INSTANCE_GET}
${EMPLOYEES_GET}
${extraPaths}
${padGets(GET_FLOOR)}
components:
  schemas:
${EMPLOYMENT_SCHEMA}
    EmployeePage:
      type: object
    EmployeeDetail:
      type: object
`;
}

describe("check-openapi-hr-from-to", () => {
  it("locks GET_FLOOR to the as_of gate so both walkers examine the same document", () => {
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(COLLECTION_FLOOR, 1);
  });

  it("accepts optional Timestamp from/to and an Employment array 200", () => {
    const fromParam = {
      name: "from",
      in: "query",
      required: false,
      schema: { $ref: "#/components/schemas/Timestamp" },
    };
    const toParam = {
      name: "to",
      in: "query",
      required: false,
      schema: { $ref: "#/components/schemas/Timestamp" },
    };
    assert.equal(isTimestampNamedParam(fromParam, "from"), true);
    assert.equal(isTimestampNamedParam(toParam, "to"), true);
    assert.equal(
      operationReturnsEmploymentArray({
        responses: {
          200: {
            content: {
              "application/json": {
                schema: {
                  type: "array",
                  items: { $ref: "#/components/schemas/Employment" },
                },
              },
            },
          },
        },
      }),
      true,
    );
  });

  it("fails while the Employment collection path is missing", () => {
    const root = fixture(spec(`  /api/v1/employments/{id}:
    get:
      parameters:
      - name: as_of
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Employment' }`));

    const { findings, collectionFromTo } = evaluateOpenapiHrFromTo({ repoRoot: root });
    assert.equal(collectionFromTo, 0);
    assert.ok(
      findings.some((finding) => /missing list\/search/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("fails when the collection exists but omits from/to", () => {
    const root = fixture(spec(`  ${COLLECTION_PATH}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/Employment' }`));

    const { findings } = evaluateOpenapiHrFromTo({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /optional from and to/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects integer from/to copied from the evidence register", () => {
    const root = fixture(spec(`  ${COLLECTION_PATH}:
    get:
      parameters:
      - name: from
        in: query
        schema: { type: integer, format: int64 }
      - name: to
        in: query
        schema: { type: integer, format: int64 }
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/Employment' }`));

    const { findings } = evaluateOpenapiHrFromTo({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /not the evidence-register integer/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects a collection that returns EmployeePage instead of Employment Heads", () => {
    const root = fixture(spec(`  ${COLLECTION_PATH}:
    get:
      parameters:
      - name: from
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      - name: to
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200':
          content:
            application/json:
              schema: { $ref: '#/components/schemas/EmployeePage' }`));

    const { findings } = evaluateOpenapiHrFromTo({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /array of Employment Heads/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("rejects EmployeeDetail as_of (HOLD)", () => {
    const root = fixture(`openapi: 3.1.0
paths:
${INSTANCE_GET}
  ${EMPLOYEES_PATH}:
    get:
      responses:
        '200': { description: ok }
  ${EMPLOYEE_DETAIL_PATH}:
    get:
      parameters:
      - name: as_of
        in: query
        required: false
        schema: { $ref: '#/components/schemas/Timestamp' }
      responses:
        '200': { description: ok }
${HONEST_COLLECTION}
${padGets(GET_FLOOR)}
components:
  schemas:
${EMPLOYMENT_SCHEMA}
`);

    const { findings } = evaluateOpenapiHrFromTo({ repoRoot: root });
    assert.ok(
      findings.some((finding) => /EmployeeDetail as_of\/from\/to remains HOLD/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("accepts Timestamp from/to on the Employment collection", () => {
    const root = fixture(spec(HONEST_COLLECTION));
    const { findings, collectionFromTo, gets } = evaluateOpenapiHrFromTo({
      repoRoot: root,
    });
    assert.equal(collectionFromTo, COLLECTION_FLOOR);
    assert.ok(gets >= GET_FLOOR);
    assert.deepEqual(findings, [], JSON.stringify(findings, null, 2));
  });

  it("exits 1 against origin/dev-shaped documents that omit the collection", () => {
    const root = fixture(spec(""));
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /HR from\/to gate FAILED/);
  });

  it("exits 1 loudly when the document is not parseable YAML at all", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });
});
