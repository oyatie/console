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
  CATALOG_ENTRY,
  CATALOG_ENTRY_FIELDS,
  CATALOG_ENTRY_STRUCT,
  CATALOG_GET_PATH,
  CATALOG_STRING_FIELDS,
  DRAFTS_GET_PATH,
  GET_FLOOR,
  GROUP_ADMIN_GROUPS_GET_PATH,
  STATUS,
  evaluateOpenapiPolicyCatalog,
} from "./check-openapi-policy-catalog.mjs";
import { rustStructFields } from "./check-openapi-audit-record.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-policy-catalog.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-policy-catalog-"));
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

function recordProperties() {
  return CATALOG_ENTRY_FIELDS.map((name) => {
    if (name === "id") return `        ${name}: { $ref: '#/components/schemas/Uuid' }`;
    if (name === "updated_at") {
      return `        ${name}: { $ref: '#/components/schemas/Timestamp' }`;
    }
    return `        ${name}: { type: string }`;
  }).join("\n");
}

function schemas(extra = "") {
  return `    ${CATALOG_ENTRY}:
      type: object
      properties:
${recordProperties()}
    Uuid:
      type: string
      format: uuid
    Timestamp:
      type: string
      format: date-time
${extra}`;
}

function spec(extraPaths, extraSchemas = "") {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padGets(GET_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas(extraSchemas)}
`;
}

const UNTYPED_DRAFTS = `  ${DRAFTS_GET_PATH}:
    get:
      operationId: listPolicyDrafts
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  type: object
                  additionalProperties: true
  ${GROUP_ADMIN_GROUPS_GET_PATH}:
    get:
      operationId: listGroupAdminGroups
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog
      parameters:
      - name: ${STATUS}
        in: query
        required: false
        schema: { type: string }
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  type: object
                  additionalProperties: true
${UNTYPED_DRAFTS}`;

const TYPED = `  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog
      parameters:
      - name: ${STATUS}
        in: query
        required: false
        schema: { type: string }
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'
${UNTYPED_DRAFTS}`;

describe("check-openapi-policy-catalog", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(CATALOG_GET_PATH, "/api/v1/policy/catalog");
    assert.equal(DRAFTS_GET_PATH, "/api/v1/policy/drafts");
    assert.equal(GROUP_ADMIN_GROUPS_GET_PATH, "/api/v1/group-admin/groups");
    assert.equal(CATALOG_ENTRY, "CatalogEntry");
    assert.equal(CATALOG_ENTRY_STRUCT, "CatalogEntry");
    assert.equal(STATUS, "status");
    assert.equal(BOUND, 1);
    assert.deepEqual(CATALOG_ENTRY_FIELDS, [
      "id",
      "stable_key",
      "title",
      "effect",
      "status",
      "source",
      "validation_status",
      "updated_at",
    ]);
    assert.deepEqual(CATALOG_STRING_FIELDS, [
      "stable_key",
      "title",
      "effect",
      "status",
      "source",
      "validation_status",
    ]);
    assert.deepEqual(
      rustStructFields(
        "struct CatalogEntry {\n    id: uuid::Uuid,\n    status: String,\n}\n",
        "CatalogEntry",
      ),
      ["id", "status"],
    );
  });

  it("fails while GET /api/v1/policy/catalog 200 items stay additionalProperties", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CATALOG_GET_PATH}/get/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when CatalogEntry is missing from composed schemas", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    Uuid: { type: string, format: uuid }
    Timestamp: { type: string, format: date-time }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${CATALOG_ENTRY}`
          && /CatalogEntry/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when status is typed as an enum catalog", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        status: { type: string }",
          "        status: { type: string, enum: [enforced, shadow] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${CATALOG_ENTRY}/properties/${STATUS}`
          && /enum|catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on CatalogEntry", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        updated_at: { $ref: '#/components/schemas/Timestamp' }",
          "        updated_at: { $ref: '#/components/schemas/Timestamp' }\n        payable: { type: boolean }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${CATALOG_ENTRY}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET /api/v1/policy/catalog (Feature::ALL)", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog`,
            `  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CATALOG_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET /api/v1/policy/catalog grows invented query params", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `      parameters:
      - name: ${STATUS}
        in: query
        required: false
        schema: { type: string }`,
            `      parameters:
      - name: ${STATUS}
        in: query
        required: false
        schema: { type: string }
      - name: as_of
        in: query
        schema: { type: string }`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${CATALOG_GET_PATH}/get/parameters`
          && /as_of/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to CatalogEntry", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${DRAFTS_GET_PATH}/get/responses/200`
          && /DraftRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when 200 items $ref the existing CatalogEntry", () => {
    const result = evaluateOpenapiPolicyCatalog({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi policy-catalog typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /CatalogEntry|additionalProperties/);
    }
  });
});
