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
  AUDIT_RECORD,
  AUDIT_RECORD_FIELDS,
  AUDIT_RECORD_STRUCT,
  BOUND,
  GET_FLOOR,
  evaluateOpenapiAuditRecord,
  rustStructFields,
} from "./check-openapi-audit-record.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-audit-record.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-audit-record-"));
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
  const lines = AUDIT_RECORD_FIELDS.map((name) => {
    if (name === ACTION) return `        ${name}: { type: string }`;
    if (name === "occurred_at") {
      return `        ${name}: { $ref: '#/components/schemas/Timestamp' }`;
    }
    return `        ${name}: { type: string }`;
  });
  return lines.join("\n");
}

function schemas(extra = "") {
  return `    ${AUDIT_RECORD}:
      type: object
      properties:
${recordProperties()}
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

const PAGE = `                type: object
                required: [items, limit, offset]
                properties:
                  items:
                    type: array
                    items:
                      $ref: '#/components/schemas/${AUDIT_RECORD}'
                  limit: { type: integer, format: int64 }
                  offset: { type: integer, format: int64 }`;

const UNTYPED = `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog
      responses:
        '200':
          content:
            application/json:
              schema:
                type: object
                required: [items, limit, offset]
                properties:
                  items:
                    type: array
                    items:
                      type: object
                      additionalProperties: true
                  limit: { type: integer }
                  offset: { type: integer }`;

const TYPED = `  ${AUDIT_GET_PATH}:
    get:
      operationId: listAuditLog
      responses:
        '200':
          content:
            application/json:
              schema:
${PAGE}`;

describe("check-openapi-audit-record", () => {
  it("exports examined-zero floor, path, and the existing struct field names", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(AUDIT_GET_PATH, "/api/audit");
    assert.equal(AUDIT_RECORD, "AuditRecord");
    assert.equal(AUDIT_RECORD_STRUCT, "AuditRecord");
    assert.equal(ACTION, "action");
    assert.equal(BOUND, 1);
    assert.deepEqual(AUDIT_RECORD_FIELDS, [
      "id",
      "actor",
      "action",
      "target_type",
      "target_id",
      "branch_id",
      "before_snap",
      "after_snap",
      "ip",
      "user_agent",
      "auth_method",
      "device",
      "classification_badges",
      "anomaly",
      "reason",
      "trace_id",
      "span_id",
      "occurred_at",
    ]);
    assert.deepEqual(
      rustStructFields(
        "struct AuditRecord {\n    id: uuid::Uuid,\n    action: String,\n}\n",
        "AuditRecord",
      ),
      ["id", "action"],
    );
  });

  it("fails while GET /api/audit 200 items stay additionalProperties", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(spec(UNTYPED)),
    });
    assert.equal(result.bound, 0);
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/paths/${AUDIT_GET_PATH}/get/responses/200`
          && /additionalProperties/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when AuditRecord is missing from composed schemas", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${padGets(GET_FLOOR)}
${TYPED}
components:
  schemas:
    Timestamp: { type: string, format: date-time }
`),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${AUDIT_RECORD}`
          && /AuditRecord/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when action is typed as an enum catalog", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        action: { type: string }",
          "        action: { type: string, enum: [audit.read] }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${AUDIT_RECORD}/properties/${ACTION}`
          && /enum|catalog/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when an invented field appears on AuditRecord", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(
        spec(TYPED).replace(
          "        occurred_at: { $ref: '#/components/schemas/Timestamp' }",
          "        occurred_at: { $ref: '#/components/schemas/Timestamp' }\n        catalog: { type: string }",
        ),
      ),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === `#/components/schemas/${AUDIT_RECORD}/properties`
          && /invent/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when permissions leak onto GET /api/audit (Feature::ALL)", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(
        spec(
          TYPED.replace(
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

  it("passes when items $ref AuditRecord derived from the existing fields", () => {
    const result = evaluateOpenapiAuditRecord({
      repoRoot: fixture(spec(TYPED)),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.bound, BOUND);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until the typed $ref is published", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi audit-record typed-response gate passed/);
    } else {
      assert.match(ran.stderr, /AuditRecord|additionalProperties/);
    }
  });
});
