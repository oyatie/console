// Canonical public-schema gate over the published OpenAPI document (P0.1).
//
// The hole this closes: PRODUCT names six governed objects (Company, OrgUnit,
// JobPosition, Person, Employment, PayRun) as the typed public contract, but
// the composed document has no schemas under those names. Generic
// CreateObjectTypeDraft children and OntologyActionRequest.params erase types.
//
// This slice is the smallest honest Heads, not a Palantir OSDK generator and
// not a semantic-manifest compiler. Links / actions / permissions / temporal
// slices are HOLD — absence is documented on the schemas, not faked.
//
// Totality: js-yaml load + own-property reads of components.schemas, same
// primitive as scripts/check-openapi-refs.mjs. A walker that visits nothing
// reports nothing, so NAME_FLOOR is the examined-zero lock.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const NAME_FLOOR = 6;

/** JSON Schema null admission, same algebra as check-openapi-nullable.mjs. */
export function jsonSchemaAdmitsNull(schema) {
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
 * Runtime Head fields that already exist under these six names.
 * Write-only catalog keys that are not on the Head (Company founded_on,
 * EmploymentAttributes.employment_status, JobPosition title-inside-attributes)
 * are not invented here. Nullable lists are Option<T> on the Head.
 */
export const CANONICAL_OBJECT_SCHEMAS = Object.freeze([
  Object.freeze({
    name: "Company",
    required: Object.freeze(["org_id", "version", "legal_name", "reg_no"]),
    nullable: Object.freeze(["legal_name", "reg_no"]),
  }),
  Object.freeze({
    name: "OrgUnit",
    required: Object.freeze(["id", "version", "name", "parent_id"]),
    nullable: Object.freeze(["name", "parent_id"]),
  }),
  Object.freeze({
    name: "JobPosition",
    required: Object.freeze(["job_position_id", "org_unit_id", "version", "attributes"]),
    nullable: Object.freeze([]),
  }),
  Object.freeze({
    name: "Person",
    required: Object.freeze(["id", "version", "display_name", "legal_name"]),
    nullable: Object.freeze(["display_name", "legal_name"]),
  }),
  Object.freeze({
    name: "Employment",
    required: Object.freeze([
      "id",
      "version",
      "appointed_on",
      "person_id",
      "org_unit_id",
      "job_position_id",
    ]),
    nullable: Object.freeze(["person_id", "org_unit_id", "job_position_id"]),
  }),
  Object.freeze({
    name: "PayRun",
    required: Object.freeze([
      "id",
      "period_start",
      "period_end",
      "source_label",
      "status",
      "payable",
    ]),
    nullable: Object.freeze([]),
    payableConstFalse: true,
  }),
]);

function requiredList(schema) {
  const required = own(schema, "required");
  return Array.isArray(required) ? required.filter((item) => typeof item === "string") : [];
}

function propertyMap(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? properties : null;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   named: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateCanonicalSchemas({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const components = own(document, "components");
  const schemas = own(components, "schemas");
  if (!isPlainObject(schemas)) {
    findings.push({
      location: "#/components/schemas",
      message: "published document has no components.schemas mapping",
    });
    return { named: 0, findings };
  }

  let named = 0;
  for (const spec of CANONICAL_OBJECT_SCHEMAS) {
    const location = `#/components/schemas/${spec.name}`;
    if (!hasOwnKey(schemas, spec.name)) {
      findings.push({
        location,
        message:
          "canonical object schema is absent; CreateObjectTypeDraft / OntologyActionRequest.params "
          + "are not this contract",
      });
      continue;
    }
    named += 1;
    const schema = own(schemas, spec.name);
    if (!isPlainObject(schema) || own(schema, "type") !== "object") {
      findings.push({
        location,
        message: "canonical object schema must be type: object (runtime Head, not a Palantir graph)",
      });
      continue;
    }
    const required = requiredList(schema);
    for (const field of spec.required) {
      if (!required.includes(field)) {
        findings.push({
          location: `${location}/required`,
          message: `runtime Head field ${field} is not required`,
        });
      }
    }
    const properties = propertyMap(schema);
    if (!properties) {
      findings.push({
        location: `${location}/properties`,
        message: "canonical object schema has no properties mapping",
      });
      continue;
    }
    for (const field of spec.required) {
      if (!hasOwnKey(properties, field)) {
        findings.push({
          location: `${location}/properties/${field}`,
          message: `runtime Head field ${field} is missing`,
        });
      }
    }
    for (const field of spec.nullable) {
      const property = own(properties, field);
      if (!jsonSchemaAdmitsNull(property)) {
        findings.push({
          location: `${location}/properties/${field}`,
          message:
            `Option field ${field} must admit JSON null (type: [T, "null"] or oneOf with type: null)`,
        });
      }
    }
    if (spec.payableConstFalse) {
      const payable = own(properties, "payable");
      if (own(payable, "const") !== false) {
        findings.push({
          location: `${location}/properties/payable`,
          message:
            "PayRun.payable must be const: false — honest scaffold, not SAP-complete, payment HOLD",
        });
      }
    }
  }

  return { named, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateCanonicalSchemas({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { named, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = named < NAME_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${named} of ${NAME_FLOOR} canonical object schemas — below the floor, `
        + "the six PRODUCT names are not published",
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi canonical-schema gate FAILED: ${findings.length} finding(s), `
        + `${named}/${NAME_FLOOR} named schemas`,
    );
    process.exit(1);
  }
  console.log(
    `openapi canonical-schema gate passed (${named}/${NAME_FLOOR} named schemas, 0 findings)`,
  );
}
