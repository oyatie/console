import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { rustStructFields } from "./check-openapi-audit-record.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  BOUND,
  CATALOG_ENTRY,
  DRAFTS_GET_PATH,
  GET_FLOOR,
  GROUP_FIELDS,
  GROUP_RESPONSE,
  GROUPS_FIELDS,
  GROUPS_GET_PATH,
  GROUPS_RESPONSE,
  MEMBER_FIELDS,
  MEMBER_RESPONSE,
  OBJECT_TYPE_GET_PATH,
  PLATFORM_GROUP,
  evaluateOpenapiGroupAdminGroups,
} from "./check-openapi-group-admin-groups.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-group-admin-groups.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-group-admin-groups-"));
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

function schemas(extra = "") {
  return `    ${GROUPS_RESPONSE}:
      type: object
      properties:
        groups:
          type: array
          items:
            $ref: '#/components/schemas/${GROUP_RESPONSE}'
    ${GROUP_RESPONSE}:
      type: object
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        slug: { type: string }
        name: { type: string }
        status: { type: string }
        members:
          type: array
          items:
            $ref: '#/components/schemas/${MEMBER_RESPONSE}'
    ${MEMBER_RESPONSE}:
      type: object
      properties:
        id: { $ref: '#/components/schemas/Uuid' }
        slug: { type: string }
        name: { type: string }
        status: { type: string }
    Uuid:
      type: string
      format: uuid
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

const HOLD_NEIGHBORS = `  ${DRAFTS_GET_PATH}:
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
  ${OBJECT_TYPE_GET_PATH}:
    get:
      operationId: getObjectType
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
  ${ABSENCE_EXIT_GET_PATH}:
    get:
      operationId: getHrAbsenceExitDashboard
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }`;

const UNTYPED = `  ${GROUPS_GET_PATH}:
    get:
      operationId: listGroupAdminGroups
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${GROUPS_GET_PATH}:
    get:
      operationId: listGroupAdminGroups
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${GROUPS_RESPONSE}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-group-admin-groups", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(GROUPS_GET_PATH, "/api/v1/group-admin/groups");
    assert.equal(GROUPS_RESPONSE, "GroupAdminGroupsResponse");
    assert.equal(GROUP_RESPONSE, "GroupAdminGroupResponse");
    assert.equal(MEMBER_RESPONSE, "GroupAdminMemberOrgResponse");
    assert.equal(PLATFORM_GROUP, "PlatformGroup");
    assert.equal(CATALOG_ENTRY, "CatalogEntry");
    assert.equal(BOUND, 1);
    assert.deepEqual(GROUPS_FIELDS, ["groups"]);
    assert.deepEqual(GROUP_FIELDS, ["id", "slug", "name", "status", "members"]);
    assert.deepEqual(MEMBER_FIELDS, ["id", "slug", "name", "status"]);
    assert.deepEqual(
      rustStructFields(
        "struct GroupAdminGroupsResponse {\n    groups: Vec<GroupAdminGroupResponse>,\n}\n",
        "GroupAdminGroupsResponse",
      ),
      ["groups"],
    );
  });

  it("fails while GET /api/v1/group-admin/groups 200 stays additionalProperties", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${GROUPS_GET_PATH}/get/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GroupAdminGroupsResponse is missing from composed schemas", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    Uuid: { type: string, format: uuid }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GROUPS_RESPONSE}`
          && /GroupAdminGroupsResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when status is typed as PlatformOrgStatus", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        status: { type: string }\n        members:",
          "        status: { $ref: '#/components/schemas/PlatformOrgStatus' }\n        members:",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GROUP_RESPONSE}/properties/status`
          && /PlatformOrgStatus/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on GroupAdminGroupResponse", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        members:\n          type: array",
          "        member_count: { type: integer }\n        members:\n          type: array",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${GROUP_RESPONSE}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET /api/v1/group-admin/groups (Feature::ALL)", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${GROUPS_GET_PATH}:
    get:
      operationId: listGroupAdminGroups`,
            `  ${GROUPS_GET_PATH}:
    get:
      operationId: listGroupAdminGroups
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${GROUPS_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET /api/v1/group-admin/groups is bound as a Head", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${GROUPS_RESPONSE}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${GROUPS_GET_PATH}/get/responses/200`
          && /ObjectKey/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET /api/v1/group-admin/groups is bound to PlatformGroup", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${GROUPS_RESPONSE}'`,
            `$ref: '#/components/schemas/${PLATFORM_GROUP}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${GROUPS_GET_PATH}/get/responses/200`
          && /PlatformGroup|ObjectKey/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to GroupAdminGroupsResponse", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${GROUPS_RESPONSE}'`,
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

  it("passes when 200 $ref the existing GroupAdminGroupsResponse", () => {
    const result = evaluateOpenapiGroupAdminGroups({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi group-admin-groups typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /GroupAdminGroupsResponse|additionalProperties/);
    }
  });
});
