import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  evaluateOpenapiNullable,
  jsonSchemaAdmitsNull,
  SCHEMA_FLOOR,
} from "./check-openapi-nullable.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-nullable.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-nullable-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function spec(body) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths: {}
${body}`;
}

describe("openapi 3.1 JSON Schema null-union gate", () => {
  it("reports type: string with nullable: true and no JSON Schema null union", () => {
    const root = fixture(spec(`components:
  schemas:
    Token:
      type: object
      properties:
        refresh_token: { type: string, nullable: true }
`));

    const { findings } = evaluateOpenapiNullable({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].location, /refresh_token/);
    assert.match(findings[0].message, /does not admit JSON null/);
  });

  it("accepts the document's existing primitive union type: [string, 'null']", () => {
    const root = fixture(spec(`components:
  schemas:
    Page:
      properties:
        next_cursor: { type: [string, 'null'] }
`));

    const { findings, legacyNullable } = evaluateOpenapiNullable({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(legacyNullable, 0);
  });

  it("accepts the document's existing $ref oneOf null union", () => {
    const root = fixture(spec(`components:
  schemas:
    Uuid:
      type: string
    Row:
      properties:
        branch_id: { oneOf: [{ $ref: '#/components/schemas/Uuid' }, { type: 'null' }] }
`));

    const { findings } = evaluateOpenapiNullable({ repoRoot: root });

    assert.deepEqual(findings, []);
  });

  it("still reports $ref + sibling nullable: true, which JSON Schema does not treat as null", () => {
    const root = fixture(spec(`components:
  schemas:
    Uuid:
      type: string
    Row:
      properties:
        branch_id: { $ref: '#/components/schemas/Uuid', nullable: true }
`));

    const { findings } = evaluateOpenapiNullable({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].location, /branch_id/);
  });

  it("does not report nullable: true when the same node already has a type null union", () => {
    const root = fixture(spec(`components:
  schemas:
    Token:
      properties:
        refresh_token: { type: [string, 'null'], nullable: true }
`));

    const { findings, legacyNullable } = evaluateOpenapiNullable({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(legacyNullable, 1);
  });

  it("jsonSchemaAdmitsNull matches type arrays, oneOf, and anyOf", () => {
    assert.equal(jsonSchemaAdmitsNull({ type: ["string", "null"] }), true);
    assert.equal(jsonSchemaAdmitsNull({ type: "string" }), false);
    assert.equal(
      jsonSchemaAdmitsNull({ oneOf: [{ $ref: "#/components/schemas/Uuid" }, { type: "null" }] }),
      true,
    );
    assert.equal(
      jsonSchemaAdmitsNull({ anyOf: [{ type: "string" }, { type: "null" }] }),
      true,
    );
    assert.equal(jsonSchemaAdmitsNull({ nullable: true, type: "string" }), false);
  });

  it("exits 1 naming the floor when the document contains almost no schemas", () => {
    const root = fixture(spec(`components:
  schemas:
    Todo:
      type: object
`));

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /below the floor/);
  });

  it("exits 1 loudly when the document is not parseable YAML at all", () => {
    const root = fixture("openapi: 3.1.0\n  bad-indent: {\n");

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot be parsed/);
  });

  // The live document is the hole this lane closes: OAS 3.0 nullable: true without a
  // JSON Schema null union. This assertion is red on origin/dev and green only after
  // those nodes carry type: [T, 'null'] or oneOf-with-null.
  it("exits 0 against this repository, above the schema floor, with no nullable:true holes", () => {
    const { findings, schemas } = evaluateOpenapiNullable({ repoRoot });

    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 8), null, 2));
    assert.ok(schemas >= SCHEMA_FLOOR, `walker degraded: saw ${schemas} schema-like mappings, floor ${SCHEMA_FLOOR}`);

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi 3.1 null-union gate passed/);
  });
});
