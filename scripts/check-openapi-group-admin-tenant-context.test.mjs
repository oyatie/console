import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  ABSENCE_EXIT_GET_PATH,
  ACTING_ROLE,
  BOUND,
  DRAFTS_GET_PATH,
  GROUPS_RESPONSE,
  HANDLER_STRUCT,
  LIFECYCLE_PREFLIGHT,
  LIFECYCLE_PREFLIGHT_PATH,
  OBJECT_TYPE_GET_PATH,
  PLATFORM_ACTING_ROLE,
  PLATFORM_START_RESPONSE,
  PLATFORM_TENANT_CONTEXT_PATH,
  START_FIELDS,
  START_RESPONSE,
  TENANT_CONTEXT_EXIT_PATH,
  TENANT_CONTEXT_PATH,
  TOKEN_TYPE,
  WRITE_FLOOR,
  evaluateOpenapiGroupAdminTenantContext,
} from "./check-openapi-group-admin-tenant-context.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-group-admin-tenant-context.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-group-admin-tenant-context-"));
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

function schemas(extra = "") {
  return `    ${START_RESPONSE}:
      type: object
      required:
      - access_token
      - token_type
      - acting_org_id
      - acting_org_name
      - acting_role
      - expires_at
      properties:
        access_token: { type: string, minLength: 1 }
        token_type: { type: string, enum: [${TOKEN_TYPE}] }
        acting_org_id: { $ref: '#/components/schemas/Uuid' }
        acting_org_name: { type: string }
        acting_role: { type: string, enum: [${ACTING_ROLE}] }
        expires_at: { $ref: '#/components/schemas/Timestamp' }
    ${PLATFORM_START_RESPONSE}:
      type: object
      properties:
        acting_role: { type: string, enum: [${PLATFORM_ACTING_ROLE}] }
    ${LIFECYCLE_PREFLIGHT}:
      type: object
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
${padWrites(WRITE_FLOOR)}
${extraPaths}
components:
  schemas:
${schemas(extraSchemas)}
`;
}

const HOLD_NEIGHBORS = `  ${PLATFORM_TENANT_CONTEXT_PATH}:
    post:
      operationId: startPlatformTenantContext
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${PLATFORM_START_RESPONSE}'
  ${TENANT_CONTEXT_EXIT_PATH}:
    post:
      operationId: exitGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema:
                type: object
                required: [ended]
                properties:
                  ended: { type: boolean }
  ${LIFECYCLE_PREFLIGHT_PATH}:
    post:
      operationId: preflightLifecycleTransition
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${LIFECYCLE_PREFLIGHT}'
  ${DRAFTS_GET_PATH}:
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

const UNTYPED = `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema: { type: object, additionalProperties: true }
${HOLD_NEIGHBORS}`;

const TYPED = `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${START_RESPONSE}'
${HOLD_NEIGHBORS}`;

describe("check-openapi-group-admin-tenant-context", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(WRITE_FLOOR, 300);
    assert.equal(WRITE_FLOOR, PREFLIGHT_WRITE_FLOOR);
    assert.equal(TENANT_CONTEXT_PATH, "/api/v1/group-admin/tenant-context");
    assert.equal(START_RESPONSE, "GroupAdminTenantContextStartResponse");
    assert.equal(HANDLER_STRUCT, "GroupAdminTenantContextStartResponse");
    assert.equal(PLATFORM_START_RESPONSE, "PlatformTenantContextStartResponse");
    assert.equal(ACTING_ROLE, "GROUP_ADMIN_DELEGATED_ADMIN");
    assert.equal(PLATFORM_ACTING_ROLE, "SUPER_ADMIN");
    assert.equal(TOKEN_TYPE, "Bearer");
    assert.equal(GROUPS_RESPONSE, "GroupAdminGroupsResponse");
    assert.equal(BOUND, 1);
    assert.deepEqual(START_FIELDS, [
      "access_token",
      "token_type",
      "acting_org_id",
      "acting_org_name",
      "acting_role",
      "expires_at",
    ]);
    assert.deepEqual(
      rustStructFields(
        `struct GroupAdminTenantContextStartResponse {
    access_token: String,
    token_type: &'static str,
    acting_org_id: Uuid,
    acting_org_name: String,
    acting_role: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: OffsetDateTime,
}
`,
        "GroupAdminTenantContextStartResponse",
      ),
      START_FIELDS,
    );
    assert.ok(HEAD_SCHEMA_NAMES.includes("Company"));
  });

  it("fails while POST group-admin tenant-context 200 stays additionalProperties", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GroupAdminTenantContextStartResponse is missing from composed schemas", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padWrites(WRITE_FLOOR)}
${TYPED}
components:
  schemas:
    Uuid: { type: string, format: uuid }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${START_RESPONSE}`
          && /GroupAdminTenantContextStartResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on GroupAdminTenantContextStartResponse", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        expires_at: { $ref: '#/components/schemas/Timestamp' }",
          "        expires_at: { $ref: '#/components/schemas/Timestamp' }\n        invented_store: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${START_RESPONSE}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when acting_role is typed as platform SUPER_ADMIN", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(TYPED).replace(
          `        acting_role: { type: string, enum: [${ACTING_ROLE}] }`,
          `        acting_role: { type: string, enum: [${PLATFORM_ACTING_ROLE}] }`,
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${START_RESPONSE}/properties/acting_role`
          && /SUPER_ADMIN/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto POST group-admin tenant-context (Feature::ALL)", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext`,
            `  ${TENANT_CONTEXT_PATH}:
    post:
      operationId: startGroupAdminTenantContext
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TENANT_CONTEXT_PATH}/post/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST group-admin tenant-context is bound as a Head", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${START_RESPONSE}'`,
            "$ref: '#/components/schemas/Company'",
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`
          && /ObjectKey|Company/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when POST group-admin tenant-context is bound to PlatformTenantContextStartResponse", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${START_RESPONSE}'`,
            `$ref: '#/components/schemas/${PLATFORM_START_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`
          && /PlatformTenantContextStartResponse|ObjectKey/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when platform tenant-context is bound to GroupAdminTenantContextStartResponse", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `$ref: '#/components/schemas/${PLATFORM_START_RESPONSE}'`,
            `$ref: '#/components/schemas/${START_RESPONSE}'`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${PLATFORM_TENANT_CONTEXT_PATH}/post/responses/200`
          && /PlatformTenantContextStartResponse/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when policy drafts are bound to GroupAdminTenantContextStartResponse", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(
        spec(
          TYPED.replace(
            `                items:
                  type: object
                  additionalProperties: true`,
            `                items:
                  $ref: '#/components/schemas/${START_RESPONSE}'`,
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

  it("passes when 200 $ref the existing GroupAdminTenantContextStartResponse", () => {
    const result = evaluateOpenapiGroupAdminTenantContext({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.writes >= WRITE_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(
        ran.stdout,
        /openapi group-admin-tenant-context typed-response gate passed/,
      );
    } else {
      assert.match(ran.stderr, /GroupAdminTenantContextStartResponse|additionalProperties/);
    }
  });
});
