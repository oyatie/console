import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import {
  ACTION,
  AUDIT_GET_PATH,
  BOUND_FILTERS,
  GET_FLOOR,
  TARGET_ID,
  TRACE_ID,
  evaluateOpenapiAuditQueryParams,
  isOptionalStringQueryParam,
} from "./check-openapi-audit-query-params.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-audit-query-params.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-audit-query-params-"));
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
    Uuid:
      type: string
      format: uuid
`;
}

const EXISTING = `      parameters:
      - name: limit
        in: query
        schema: { type: integer }
      - name: offset
        in: query
        schema: { type: integer }
      - name: target_type
        in: query
        schema: { type: string }
      - name: actor
        in: query
        schema: { $ref: '#/components/schemas/Uuid' }`;

const UNPUBLISHED = `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog
${EXISTING}
      responses:
        '200': { description: ok }`;

const PUBLISHED = `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog
${EXISTING}
      - name: ${TARGET_ID}
        in: query
        required: false
        schema: { type: string }
      - name: ${TRACE_ID}
        in: query
        required: false
        schema: { type: string }
      responses:
        '200': { description: ok }`;

describe("check-openapi-audit-query-params", () => {
  it("exports examined-zero floor and the existing GET identities", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(AUDIT_GET_PATH, "/api/audit");
    assert.equal(TARGET_ID, "target_id");
    assert.equal(TRACE_ID, "trace_id");
    assert.equal(ACTION, "action");
    assert.equal(BOUND_FILTERS, 2);
    assert.equal(
      isOptionalStringQueryParam(
        { name: TARGET_ID, in: "query", schema: { type: "string" } },
        TARGET_ID,
      ),
      true,
    );
    assert.equal(
      isOptionalStringQueryParam(
        { name: TARGET_ID, in: "query", schema: { type: "string", enum: ["x"] } },
        TARGET_ID,
      ),
      false,
    );
  });

  it("fails while GET /api/audit omits target_id and trace_id", () => {
    const result = evaluateOpenapiAuditQueryParams({
      repoRoot: fixture(spec(UNPUBLISHED)),
    });
    assert.equal(result.bound, 0);
    for (const name of [TARGET_ID, TRACE_ID]) {
      assert.ok(
        result.findings.some(
          (finding) =>
            finding.location === `#/paths/${AUDIT_GET_PATH}/get/parameters/${name}`
            && finding.message.includes(name),
        ),
        `${name}: ${JSON.stringify(result.findings, null, 2)}`,
      );
    }
  });

  it("fails when target_id is published as an enum catalog", () => {
    const result = evaluateOpenapiAuditQueryParams({
      repoRoot: fixture(
        spec(
          PUBLISHED.replace(
            `      - name: ${TARGET_ID}
        in: query
        required: false
        schema: { type: string }`,
            `      - name: ${TARGET_ID}
        in: query
        required: false
        schema: { type: string, enum: [work_order] }`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUDIT_GET_PATH}/get/parameters/${TARGET_ID}`
          && /enum|catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when action is published as an enum catalog", () => {
    const result = evaluateOpenapiAuditQueryParams({
      repoRoot: fixture(
        spec(
          PUBLISHED.replace(
            `      - name: ${TRACE_ID}`,
            `      - name: ${ACTION}
        in: query
        schema: { type: string, enum: [audit.read] }
      - name: ${TRACE_ID}`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUDIT_GET_PATH}/get/parameters/${ACTION}`
          && /catalog|enum/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET /api/audit (Feature::ALL)", () => {
    const result = evaluateOpenapiAuditQueryParams({
      repoRoot: fixture(
        spec(
          PUBLISHED.replace(
            `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog`,
            `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog
      permissions:
      - role_manage`,
          ),
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUDIT_GET_PATH}/get/permissions`
          && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when the two existing filters are optional string queries", () => {
    const result = evaluateOpenapiAuditQueryParams({
      repoRoot: fixture(spec(PUBLISHED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND_FILTERS);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the filters are published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi audit query-params gate passed/);
    } else {
      assert.match(ran.stderr, /target_id|trace_id/);
    }
  });
});
