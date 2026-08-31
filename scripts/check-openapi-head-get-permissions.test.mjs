import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { REQUIRED_INSTANCE_GET_HEADS } from "./check-openapi-head-gets.mjs";
import {
  FENCED_HEADS,
  FENCED_PAY_RUN,
  GET_FLOOR,
  HEAD_FLOOR,
  HEAD_GET_PERMISSION_OPS,
  HR_AUTHORIZE_DIRECTORY_READ,
  HR_RS_REL,
  OP_FLOOR,
  PERMISSION_EMPLOYEE_DIRECTORY_READ,
  evaluateOpenapiHeadGetPermissions,
} from "./check-openapi-head-get-permissions.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-head-get-permissions.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml, extraFiles = {}) {
  const root = mkdtempSync(join(tmpdir(), "openapi-head-get-permissions-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  for (const [rel, body] of Object.entries(extraFiles)) {
    const path = join(root, rel);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, body);
  }
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

function schemaYaml(name, { permissions = null } = {}) {
  const perm =
    permissions === null
      ? ""
      : `      permissions:
${permissions.map((item) => `        - ${item}`).join("\n")}
`;
  return `    ${name}:
      type: object
      properties:
        id: { type: string }
${perm}`;
}

function opYaml(spec, { permissions = null } = {}) {
  const perm =
    permissions === null
      ? ""
      : `      permissions:
${permissions.map((item) => `      - ${item}`).join("\n")}
`;
  const pathId = spec.path.includes("{id}")
    ? `      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
`
    : "";
  const schema = spec.path.includes("{id}") && spec.operationId !== "listOrgUnitJobPositions"
    ? `          $ref: '#/components/schemas/${spec.head}'`
    : `          type: array
          items:
            $ref: '#/components/schemas/${spec.head}'`;
  return `  ${spec.path}:
    get:
      operationId: ${spec.operationId}
${perm}${pathId}      responses:
        '200':
          content:
            application/json:
              schema:
${schema}`;
}

function spec({ opPermissions = null, schemaPermissions = null, extraPaths = "" } = {}) {
  const paths = HEAD_GET_PERMISSION_OPS.map((item) =>
    opYaml(item, { permissions: opPermissions }),
  ).join("\n");
  const schemas = HEAD_SCHEMA_NAMES.map((name) => {
    if (name === FENCED_PAY_RUN) return schemaYaml(name, { permissions: null });
    return schemaYaml(name, { permissions: schemaPermissions });
  }).join("\n");
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${paths}
${extraPaths}
${padGets(GET_FLOOR)}
components:
  schemas:
${schemas}
`;
}

const HONEST = {
  opPermissions: [PERMISSION_EMPLOYEE_DIRECTORY_READ],
  schemaPermissions: [PERMISSION_EMPLOYEE_DIRECTORY_READ],
};

const DTO_WITH_PERMISSION = `pub(super) fn published_head_get_permission(head: &str) -> Option<&'static str> {
    match head {
        "Company" => Some("${PERMISSION_EMPLOYEE_DIRECTORY_READ}"),
        _ => None,
    }
}
`;

const HR_WITH_AUTHORIZE = `async fn get_org_unit() {
    ${HR_AUTHORIZE_DIRECTORY_READ}?;
}
`;

describe("check-openapi-head-get-permissions", () => {
  it("exports examined-zero floors and the Chesterton fences", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(HEAD_FLOOR, 5);
    assert.equal(OP_FLOOR, 11);
    assert.equal(HEAD_GET_PERMISSION_OPS.length, OP_FLOOR);
    assert.deepEqual(REQUIRED_INSTANCE_GET_HEADS, [
      "Company",
      "OrgUnit",
      "JobPosition",
      "Person",
      "Employment",
    ]);
    assert.deepEqual(FENCED_HEADS, [FENCED_PAY_RUN]);
    assert.equal(PERMISSION_EMPLOYEE_DIRECTORY_READ, "employee_directory_read");
    assert.ok(!HEAD_GET_PERMISSION_OPS.some((item) => item.head === "PayRun"));
  });

  it("fails while Head GET/list omit the Feature the runtime already enforces", () => {
    const result = evaluateOpenapiHeadGetPermissions({
      repoRoot: fixture(spec()),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/paths/api/v1/org-units/{id}/get/permissions"
          || finding.location === "#/paths//api/v1/org-units/{id}/get/permissions"
          || (
            finding.location.includes("/api/v1/org-units/{id}/get/permissions")
            && /EmployeeDirectoryRead/.test(finding.message)
          ),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.ok(
      result.findings.some((finding) =>
        finding.location.includes("OrgUnit") && /permissions/.test(finding.location),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when GET permissions invent role_manage (the execute gate)", () => {
    const result = evaluateOpenapiHeadGetPermissions({
      repoRoot: fixture(
        spec({
          opPermissions: ["role_manage"],
          schemaPermissions: [PERMISSION_EMPLOYEE_DIRECTORY_READ],
        }),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("/permissions")
          && /role_manage|invented permission/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when PayRun Head grows a GET permission", () => {
    const yaml = spec(HONEST).replace(
      "    PayRun:\n      type: object\n      properties:\n        id: { type: string }\n",
      `    PayRun:
      type: object
      properties:
        id: { type: string }
      permissions:
        - ${PERMISSION_EMPLOYEE_DIRECTORY_READ}
`,
    );
    const result = evaluateOpenapiHeadGetPermissions({ repoRoot: fixture(yaml) });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("PayRun")
          && /no Head GET/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto a non-Head GET (Feature::ALL)", () => {
    const extra = `  /api/v1/leave/requests:
    get:
      operationId: listLeaveRequests
      permissions:
      - ${PERMISSION_EMPLOYEE_DIRECTORY_READ}
      responses:
        '200': { description: ok }
`;
    const result = evaluateOpenapiHeadGetPermissions({
      repoRoot: fixture(spec({ ...HONEST, extraPaths: extra })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("/api/v1/leave/requests")
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when generated Head GET/list permissions match EmployeeDirectoryRead", () => {
    const result = evaluateOpenapiHeadGetPermissions({
      repoRoot: fixture(spec(HONEST), {
        "backend/crates/contracts/src/semantic_dtos.rs": DTO_WITH_PERMISSION,
        [HR_RS_REL]: HR_WITH_AUTHORIZE,
      }),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.ops, OP_FLOOR);
    assert.equal(result.heads, HEAD_SCHEMA_NAMES.length);
    assert.equal(result.permissionedOps, OP_FLOOR);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until Head GET permissions are generated", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi Head GET\/list permissions gate passed/);
    } else {
      assert.match(
        ran.stderr,
        /employee_directory_read|EmployeeDirectoryRead|permissions/,
      );
    }
  });
});
