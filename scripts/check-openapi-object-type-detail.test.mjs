import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  ACTION_FIELDS,
  ACTION_TYPE_SUMMARY,
  ACTION_VALUE_FIELDS,
  ANALYTIC_FIELDS,
  ANALYTIC_SUMMARY,
  BOUND,
  CATALOG_ENTRY,
  CATALOG_GET_PATH,
  DETAIL_FIELDS,
  DETAIL_REQUIRED,
  GET_FLOOR,
  KILL_SWITCH_PATH,
  LINK_FIELDS,
  LINK_TYPE_SUMMARY,
  OBJECT_TYPE_DETAIL,
  OBJECT_TYPE_GET_PATH,
  OBJECT_TYPE_RESPONSE,
  OBJECT_TYPE_SUMMARY,
  OBJECT_TYPES_LIST_PATH,
  PROPERTY_DEF_SUMMARY,
  PROPERTY_FIELDS,
  QUERY_PARAMS,
  UNKNOWN_STRING_FIELD,
  VALUE_FIELDS,
  WRITE_FLOOR,
  evaluateOpenapiObjectTypeDetail,
} from "./check-openapi-object-type-detail.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-object-type-detail.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-object-type-detail-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function padWrites(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad/${i}:
    post:
      operationId: padPost${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function padGets(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad-get/${i}:
    get:
      operationId: padGet${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function schemas(extra = "") {
  return `    ${OBJECT_TYPE_DETAIL}:
      type: object
      required:
      - object_type
      - properties
      - links
      - actions
      - analytics
      properties:
        object_type: { $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}' }
        title_property_key: { type: [string, 'null'] }
        backing_table: { type: [string, 'null'] }
        primary_key_property: { type: [string, 'null'] }
        properties:
          type: array
          items: { $ref: '#/components/schemas/${PROPERTY_DEF_SUMMARY}' }
        links:
          type: array
          items: { $ref: '#/components/schemas/${LINK_TYPE_SUMMARY}' }
        actions:
          type: array
          items: { $ref: '#/components/schemas/${ACTION_TYPE_SUMMARY}' }
        analytics:
          type: array
          items: { $ref: '#/components/schemas/${ANALYTIC_SUMMARY}' }
    ${PROPERTY_DEF_SUMMARY}:
      type: object
      required:
      - id
      - key
      - title
      - field_type
      - field_kind
      - config
      - required
      - in_property_policy
      properties:
        id: { type: string, format: uuid }
        key: { type: string }
        title: { type: string }
        field_type: { type: string }
        field_kind: { type: string }
        config: { type: object, additionalProperties: true }
        backing_column: { type: [string, 'null'] }
        required: { type: boolean }
        in_property_policy: { type: boolean }
    ${LINK_TYPE_SUMMARY}:
      type: object
      required:
      - id
      - stable_key
      - title
      - cardinality
      - traversable
      properties:
        id: { type: string, format: uuid }
        stable_key: { type: string }
        title: { type: string }
        reverse_title: { type: [string, 'null'] }
        to_object_type_id: { type: [string, 'null'], format: uuid }
        cardinality: { type: string, enum: [one_one, one_many, many_many] }
        traversable: { type: boolean }
    ${ACTION_TYPE_SUMMARY}:
      type: object
      required:
      - id
      - stable_key
      - title
      - params_schema
      - edits
      - submission_criteria
      - side_effects
      - dispatch
      - control_points
      properties:
        id: { type: string, format: uuid }
        stable_key: { type: string }
        title: { type: string }
        params_schema: { type: object, additionalProperties: true }
        edits: { type: object, additionalProperties: true }
        submission_criteria: { type: object, additionalProperties: true }
        side_effects: { type: object, additionalProperties: true }
        dispatch: { type: string, enum: [projected_usecase, instance_revision] }
        dispatch_target: { type: [string, 'null'] }
        control_points: { type: object, additionalProperties: true }
    ${ANALYTIC_SUMMARY}:
      type: object
      required:
      - id
      - key
      - title
      - formula
      - result_type
      properties:
        id: { type: string, format: uuid }
        key: { type: string }
        title: { type: string }
        formula: { type: object, additionalProperties: true }
        result_type: { type: object, additionalProperties: true }
    ${OBJECT_TYPE_SUMMARY}:
      type: object
      properties:
        stable_key: { type: string }
    ${OBJECT_TYPE_RESPONSE}:
      type: object
      properties:
        kind: { type: string }
    AbsenceExitDashboardResponse:
      type: object
      properties:
        summary: { type: object }
    ${CATALOG_ENTRY}:
      type: object
      properties:
        stable_key: { type: string }
    DraftRecord:
      type: object
      properties:
        draft_key: { type: string }
    OverrideSummary:
      type: object
      properties:
        reason: { type: string }
    HrReadinessSummary:
      type: object
      properties:
        ready: { type: boolean }
    EmployeeExitCaseResponse:
      type: object
      properties:
        status: { type: string }
    Company:
      type: object
      properties:
        org_id: { type: string }
    InventedKind:
      type: object
      required: [kind]
      properties:
        kind: { type: string, enum: [text, integer] }
${extra}`;
}

function spec(extraPaths, extraSchemas = "") {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${padWrites(WRITE_FLOOR)}
${padGets(GET_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas(extraSchemas)}
`;
}

const PARAMS = `      parameters:
      - name: key
        in: path
      - name: version
        in: query`;

const HOLD_NEIGHBORS = `  ${OBJECT_TYPES_LIST_PATH}:
    get:
      operationId: listObjectTypes
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}'
    post:
      operationId: createObjectType
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}'
  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AbsenceExitDashboardResponse'
  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${CATALOG_GET_PATH}:
    get:
      operationId: listPolicyCatalog
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/${CATALOG_ENTRY}'`;

const KEY_PUT = `    put:
      operationId: stageObjectTypeRevision
      responses:
        '201':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}'`;

const UNTYPED = `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
${PARAMS}
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${KEY_PUT}
${HOLD_NEIGHBORS}`;

const TYPED = `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
${PARAMS}
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OBJECT_TYPE_DETAIL}'
${KEY_PUT}
${HOLD_NEIGHBORS}`;

describe("check-openapi-object-type-detail", () => {
  it("exports examined-zero floors, paths, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(OBJECT_TYPE_GET_PATH, "/api/v1/ontology/object-types/{key}");
    assert.equal(OBJECT_TYPE_DETAIL, "ObjectTypeDetail");
    assert.deepEqual(VALUE_FIELDS, [
      "config",
      "params_schema",
      "edits",
      "submission_criteria",
      "side_effects",
      "control_points",
      "formula",
      "result_type",
    ]);
    assert.deepEqual(ACTION_VALUE_FIELDS, [
      "params_schema",
      "edits",
      "submission_criteria",
      "side_effects",
      "control_points",
    ]);
    assert.equal(UNKNOWN_STRING_FIELD, "field_kind");
    assert.equal(BOUND, 1);
    assert.deepEqual(DETAIL_FIELDS, [
      "object_type",
      "title_property_key",
      "backing_table",
      "primary_key_property",
      "properties",
      "links",
      "actions",
      "analytics",
    ]);
    assert.deepEqual(DETAIL_REQUIRED, [
      "object_type",
      "properties",
      "links",
      "actions",
      "analytics",
    ]);
    assert.equal(PROPERTY_FIELDS.length, 9);
    assert.equal(LINK_FIELDS.length, 7);
    assert.equal(ACTION_FIELDS.length, 10);
    assert.equal(ANALYTIC_FIELDS.length, 5);
    assert.deepEqual(QUERY_PARAMS, ["key", "version"]);
    assert.deepEqual(
      rustStructFields(
        `struct ObjectTypeDetail {
    object_type: ObjectTypeSummary,
    title_property_key: Option<String>,
    backing_table: Option<String>,
    primary_key_property: Option<String>,
    properties: Vec<PropertyDefSummary>,
    links: Vec<LinkTypeSummary>,
    actions: Vec<ActionTypeSummary>,
    analytics: Vec<AnalyticSummary>,
}
`,
        "ObjectTypeDetail",
      ),
      DETAIL_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while GET object-types/{key} 200 stays a root additionalProperties bag", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`
          && /root additionalProperties bag/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when ObjectTypeDetail is missing from composed schemas", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padWrites(WRITE_FLOOR)}
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    ${OBJECT_TYPE_SUMMARY}: { type: object }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${OBJECT_TYPE_DETAIL}`
          && /must be derived/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when field_kind is closed to a FieldKind catalog", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        field_kind: { type: string }",
          "        field_kind: { type: string, enum: [Text, Integer, Unknown] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/${UNKNOWN_STRING_FIELD}`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when config is given a closed nested object schema", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        config: { type: object, additionalProperties: true }",
          "        config:\n          type: object\n          additionalProperties: false\n          properties:\n            link: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/config`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when params_schema $ref an invented nested schema", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        params_schema: { type: object, additionalProperties: true }",
          "        params_schema: { $ref: '#/components/schemas/InventedKind' }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ACTION_TYPE_SUMMARY}/properties/params_schema`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when formula is an array of invented nested schemas", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        formula: { type: object, additionalProperties: true }",
          "        formula:\n          type: array\n          items:\n            type: object\n            properties:\n              op: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location
            === `#/components/schemas/${ANALYTIC_SUMMARY}/properties/formula`
          && /catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET {key} is bound to ObjectTypeSummary", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OBJECT_TYPE_DETAIL}'`,
            `$ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`
          && /ObjectTypeSummary/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET object-types/{key} (Feature::ALL)", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType`,
            `  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET object-types/{key} is bound as a Head", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${OBJECT_TYPE_DETAIL}'`,
            `$ref: '#/components/schemas/Company'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`
          && /Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch is bound to ObjectTypeDetail", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`,
            `  ${KILL_SWITCH_PATH}:
    post:
      operationId: updateConsoleLegacyKillSwitch
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${OBJECT_TYPE_DETAIL}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${KILL_SWITCH_PATH}/post/responses/200`
          && /kill-switch/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when list GET is bound to ObjectTypeDetail", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  $ref: '#/components/schemas/${OBJECT_TYPE_SUMMARY}'`,
            `                items:
                  $ref: '#/components/schemas/${OBJECT_TYPE_DETAIL}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPES_LIST_PATH}/get/responses/200`
          && /ObjectTypeDetail/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented query param appears", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            "      - name: version\n        in: query",
            "      - name: version\n        in: query\n      - name: as_of\n        in: query",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${OBJECT_TYPE_GET_PATH}/get/parameters`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when GET 200 $ref the existing envelope and Value/Unknown stay open", () => {
    const result = evaluateOpenapiObjectTypeDetail({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(
        ran.stdout,
        /openapi object-type-detail typed-response gate passed/,
      );
    } else {
      assert.match(
        ran.stderr,
        /ObjectTypeDetail|additionalProperties|root additionalProperties bag/,
      );
    }
  });
});
