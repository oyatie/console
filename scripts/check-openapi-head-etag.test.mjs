import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { EXECUTE_PATH, PREFLIGHT_PATH } from "./check-openapi-typed-execute.mjs";
import {
  GET_FLOOR,
  GET_TOKEN_VERSION,
  HEAD_FLOOR,
  REQUIRED_GET_TOKEN_VERSION_HEADS,
  WRITE_FIELD,
  WRITE_IN_BODY,
  evaluateOpenapiHeadEtag,
} from "./check-openapi-head-etag.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-head-etag.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-head-etag-"));
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

const VERSIONED = new Set(["Company", "OrgUnit", "JobPosition", "Person", "Employment"]);

function concurrencyYaml(name) {
  const token = VERSIONED.has(name) ? GET_TOKEN_VERSION : "null";
  const versionProp = VERSIONED.has(name)
    ? `        version:
          type: integer
          format: int64
          minimum: 1`
    : "";
  const required = VERSIONED.has(name) ? "[id, version]" : "[id]";
  return `    ${name}:
      type: object
      required: ${required}
      properties:
        id: { type: string }
${versionProp}
      links: []
      actions:
      - action_key: ${name}.revise
        concurrency:
          command_id: tenant_global_idempotency
          expected_revision: optional_cas
      concurrency:
        get_token: ${token}
        write_field: ${WRITE_FIELD}
        write_in: ${WRITE_IN_BODY}`;
}

function honestHeads() {
  return HEAD_SCHEMA_NAMES.map((name) => concurrencyYaml(name)).join("\n");
}

function spec({ heads, extraPaths = "", gets = GET_FLOOR }) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${extraPaths}
${padGets(gets)}
components:
  schemas:
    OntologyActionRequest:
      type: object
      properties:
        expected_revision:
          type: integer
          format: int64
${heads}
`;
}

describe("check-openapi-head-etag", () => {
  it("exports examined-zero floors", () => {
    assert.equal(HEAD_FLOOR, 6);
    assert.equal(GET_FLOOR, 200);
  });

  it("fails when Head schemas omit schema-level concurrency", () => {
    const heads = HEAD_SCHEMA_NAMES.map(
      (name) => `    ${name}:
      type: object
      properties:
        id: { type: string }
      links: []
      actions: []`,
    ).join("\n");
    const result = evaluateOpenapiHeadEtag({ repoRoot: fixture(spec({ heads })) });
    assert.ok(result.findings.length > 0);
    assert.ok(
      result.findings.some((finding) => /schema-level concurrency is absent/.test(finding.message)),
    );
    assert.equal(result.heads, 0);
  });

  it("fails when HTTP ETag duplicates a Head GET token", () => {
    const result = evaluateOpenapiHeadEtag({
      repoRoot: fixture(
        spec({
          heads: honestHeads(),
          extraPaths: `  /api/v1/employments/{id}:
    get:
      responses:
        '200':
          headers:
            ETag:
              schema: { type: string }
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Employment'`,
          gets: GET_FLOOR - 1,
        }),
      ),
    });
    assert.ok(
      result.findings.some((finding) => /HTTP ETag would duplicate/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when execute If-Match duplicates expected_revision", () => {
    const result = evaluateOpenapiHeadEtag({
      repoRoot: fixture(
        spec({
          heads: honestHeads(),
          extraPaths: `  ${EXECUTE_PATH}:
    post:
      parameters:
      - name: If-Match
        in: header
        required: true
        schema: { type: string }
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
      responses:
        '200': { description: ok }
  ${PREFLIGHT_PATH}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
      responses:
        '200': { description: ok }`,
        }),
      ),
    });
    assert.ok(
      result.findings.some((finding) => /HTTP If-Match would duplicate/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when get_token claims version but the Head has no version property", () => {
    const heads = HEAD_SCHEMA_NAMES.map((name) => {
      if (name !== "PayRun") return concurrencyYaml(name);
      return `    PayRun:
      type: object
      required: [id]
      properties:
        id: { type: string }
      links: []
      actions: []
      concurrency:
        get_token: version
        write_field: ${WRITE_FIELD}
        write_in: ${WRITE_IN_BODY}`;
    }).join("\n");
    const result = evaluateOpenapiHeadEtag({ repoRoot: fixture(spec({ heads })) });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("PayRun/concurrency/get_token")
          && /must be null/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("fails when Employment omits version while employment_revisions.version exists", () => {
    assert.deepEqual([...REQUIRED_GET_TOKEN_VERSION_HEADS], ["Employment"]);
    const heads = HEAD_SCHEMA_NAMES.map((name) => {
      if (name !== "Employment") return concurrencyYaml(name);
      return `    Employment:
      type: object
      required: [id]
      properties:
        id: { type: string }
      links: []
      actions: []
      concurrency:
        get_token: null
        write_field: ${WRITE_FIELD}
        write_in: ${WRITE_IN_BODY}`;
    }).join("\n");
    const result = evaluateOpenapiHeadEtag({ repoRoot: fixture(spec({ heads })) });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("Employment/concurrency/get_token")
          && /must be "version"/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
    assert.ok(
      result.findings.some((finding) => finding.location.includes("Employment/properties/version")),
      JSON.stringify(result.findings),
    );
  });

  it("accepts Heads that publish version as get_token and body expected_revision", () => {
    const result = evaluateOpenapiHeadEtag({
      repoRoot: fixture(
        spec({
          heads: honestHeads(),
          extraPaths: `  /api/v1/employments/{id}:
    get:
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Employment'
  ${EXECUTE_PATH}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
      responses:
        '200': { description: ok }
  ${PREFLIGHT_PATH}:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OntologyActionRequest'
      responses:
        '200': { description: ok }`,
          gets: GET_FLOOR - 1,
        }),
      ),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.heads, HEAD_FLOOR);
  });

  it("accepts the live document only when Head concurrency is generated from DTO truth", () => {
    const run = spawnSync(process.execPath, [cli], {
      cwd: repoRoot,
      encoding: "utf8",
    });
    assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
    assert.match(run.stdout, /Head concurrency gate passed/);
  });
});
