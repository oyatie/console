// OpenAPI 3.1 JSON Schema null-union gate over the published document.
//
// The hole this closes: OAS 3.0 `nullable: true` is not a JSON Schema keyword.
// OpenAPI 3.1 types are JSON Schema; a property written `type: string, nullable: true`
// does not admit JSON null. Generated clients and validators that read 3.1 therefore
// treat the field as required-non-null. The 3.1-correct forms already used in this
// document are `type: [T, 'null']` (primitives) and `oneOf: [{ $ref }, { type: 'null' }]`
// ($ref). This walker flags every mapping that still uses `nullable: true` without a
// JSON Schema null union on the same node.
//
// Totality: js-yaml load + own-property walk of every mapping, same primitive as
// scripts/check-openapi-refs.mjs. Authoring style is invisible. A walker that visits
// nothing reports nothing, so the SCHEMA_FLOOR is the examined-zero lock.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const SCHEMA_FLOOR = 9000;

function jsonSchemaAdmitsNull(schema) {
  if (!isPlainObject(schema)) return false;
  const type = own(schema, "type");
  if (type === "null") return true;
  if (Array.isArray(type) && type.includes("null")) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (!Array.isArray(members)) continue;
    const admits = members.some((member) => jsonSchemaAdmitsNull(member));
    if (key === "allOf") {
      if (members.length > 0 && members.every((member) => jsonSchemaAdmitsNull(member))) {
        return true;
      }
      continue;
    }
    if (admits) return true;
  }
  return false;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   schemas: number,
 *   legacyNullable: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiNullable({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  let schemas = 0;
  let legacyNullable = 0;
  const seen = new WeakSet();

  const visit = (node, location) => {
    if (Array.isArray(node)) {
      node.forEach((item, index) => visit(item, `${location}/${index}`));
      return;
    }
    if (!isPlainObject(node)) return;
    if (seen.has(node)) return;
    seen.add(node);

    const looksLikeSchema = ["type", "$ref", "oneOf", "anyOf", "allOf", "properties", "items", "nullable"]
      .some((key) => hasOwnKey(node, key));
    if (looksLikeSchema) schemas += 1;

    if (own(node, "nullable") === true) {
      legacyNullable += 1;
      if (!jsonSchemaAdmitsNull(node)) {
        findings.push({
          location,
          message: "OpenAPI 3.1 `nullable: true` does not admit JSON null; use a JSON Schema "
            + "null union (`type: [T, \"null\"]` or `oneOf: [{ $ref }, { type: \"null\" }]`) "
            + "as this document already does for 3.1-correct fields",
        });
      }
    }

    for (const [key, value] of Object.entries(node)) {
      visit(value, `${location}/${key}`);
    }
  };

  visit(document, "#");
  return { schemas, legacyNullable, findings };
}

export { jsonSchemaAdmitsNull };

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiNullable({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { schemas, legacyNullable, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = schemas < SCHEMA_FLOOR;
  if (belowFloor) {
    console.error(`saw ${schemas} schema-like mappings (floor ${SCHEMA_FLOOR}) — below the floor, `
      + "the walker examined less of the document than it was built to examine");
  }
  if (findings.length > 0 || belowFloor) {
    console.error(`openapi 3.1 null-union gate FAILED: ${findings.length} finding(s), `
      + `${legacyNullable} nullable: true, ${schemas} schema-like mappings`);
    process.exit(1);
  }
  console.log(`openapi 3.1 null-union gate passed (${schemas} schema-like mappings, `
    + `${legacyNullable} leftover nullable: true with a JSON Schema null union, 0 findings)`);
}
