// GET /api/v1/ontology/object-types/{key} 200 ObjectTypeDetail bind gate.
//
// The hole this closes: get_object_type already returns ObjectTypeDetail whose
// envelope is object_type / title_property_key / backing_table /
// primary_key_property / properties / links / actions / analytics. Nested
// PropertyDefSummary.config, ActionTypeSummary params_schema/edits/
// submission_criteria/side_effects/control_points, and AnalyticSummary
// formula/result_type are serde_json::Value. field_kind is FieldKind with
// Unknown(String). Composed OpenAPI still advertises the whole 200 as
// additionalProperties: true. Same class as #1034 DraftRecord / #1033
// OverrideSummary — publish the existing record, do not invent a store,
// FieldKind catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing structs, then $ref. Type every
// non-Value, non-Unknown field. Value bags stay unconstrained object /
// additionalProperties: true on THOSE FIELDS ONLY. FieldKind::Unknown(String)
// stays unconstrained string, not a closed enum catalog of kinds. List GET
// must remain $ref ObjectTypeSummary. Do not bind {key} to ObjectTypeSummary.
// Do not map Feature::ALL. Do not invent query params. Do not stamp a new
// HTTP ETag (handler already sends the existing write-validator header).
//
// Totality: js-yaml load + own-property walk of every GET and write method +
// optional Rust struct field read. GET_FLOOR / WRITE_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;
export const WRITE_FLOOR = PREFLIGHT_WRITE_FLOOR;

export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const OBJECT_TYPES_LIST_PATH = "/api/v1/ontology/object-types";
export const OBJECT_TYPE_PUT_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const KILL_SWITCH_PATH = "/api/v1/console/kill-switch";
export const ROLLOUT_OPT_IN_PATH = "/api/v1/console/rollout/opt-in";
export const ROLLOUT_ORG_FLAG_PATH = "/api/v1/console/rollout/org-flag";
export const DRAFTS_PATH = "/api/v1/policy/drafts";
export const CATALOG_GET_PATH = "/api/v1/policy/catalog";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const READINESS_GET_PATH = "/api/v1/hr/readiness-summary";
export const EXIT_CASES_PATH = "/api/v1/hr/exit-cases";

export const OBJECT_TYPE_DETAIL = "ObjectTypeDetail";
export const OBJECT_TYPE_SUMMARY = "ObjectTypeSummary";
export const OBJECT_TYPE_RESPONSE = "ObjectTypeResponse";
export const PROPERTY_DEF_SUMMARY = "PropertyDefSummary";
export const LINK_TYPE_SUMMARY = "LinkTypeSummary";
export const ACTION_TYPE_SUMMARY = "ActionTypeSummary";
export const ANALYTIC_SUMMARY = "AnalyticSummary";
export const DRAFT_RECORD = "DraftRecord";
export const CATALOG_ENTRY = "CatalogEntry";
export const OVERRIDE_SUMMARY = "OverrideSummary";
export const HR_READINESS_SUMMARY = "HrReadinessSummary";
export const DASHBOARD_RESPONSE = "AbsenceExitDashboardResponse";
export const EXIT_CASE_RESPONSE = "EmployeeExitCaseResponse";
export const BOUND = 1;

export const STORE_RS_REL = "backend/crates/ontology/adapter-postgres/src/lib.rs";

/** Existing Serialize fields. Do not invent names. */
export const DETAIL_FIELDS = Object.freeze([
  "object_type",
  "title_property_key",
  "backing_table",
  "primary_key_property",
  "properties",
  "links",
  "actions",
  "analytics",
]);

export const DETAIL_REQUIRED = Object.freeze([
  "object_type",
  "properties",
  "links",
  "actions",
  "analytics",
]);

export const PROPERTY_FIELDS = Object.freeze([
  "id",
  "key",
  "title",
  "field_type",
  "field_kind",
  "config",
  "backing_column",
  "required",
  "in_property_policy",
]);

export const PROPERTY_REQUIRED = Object.freeze([
  "id",
  "key",
  "title",
  "field_type",
  "field_kind",
  "config",
  "required",
  "in_property_policy",
]);

export const LINK_FIELDS = Object.freeze([
  "id",
  "stable_key",
  "title",
  "reverse_title",
  "to_object_type_id",
  "cardinality",
  "traversable",
]);

export const LINK_REQUIRED = Object.freeze([
  "id",
  "stable_key",
  "title",
  "cardinality",
  "traversable",
]);

export const ACTION_FIELDS = Object.freeze([
  "id",
  "stable_key",
  "title",
  "params_schema",
  "edits",
  "submission_criteria",
  "side_effects",
  "dispatch",
  "dispatch_target",
  "control_points",
]);

export const ACTION_REQUIRED = Object.freeze([
  "id",
  "stable_key",
  "title",
  "params_schema",
  "edits",
  "submission_criteria",
  "side_effects",
  "dispatch",
  "control_points",
]);

export const ANALYTIC_FIELDS = Object.freeze([
  "id",
  "key",
  "title",
  "formula",
  "result_type",
]);

export const ANALYTIC_REQUIRED = ANALYTIC_FIELDS;

/** serde_json::Value on the wire. Do not close these into catalogs. */
export const VALUE_FIELDS = Object.freeze([
  "config",
  "params_schema",
  "edits",
  "submission_criteria",
  "side_effects",
  "control_points",
  "formula",
  "result_type",
]);

export const PROPERTY_VALUE_FIELDS = Object.freeze(["config"]);
export const ACTION_VALUE_FIELDS = Object.freeze([
  "params_schema",
  "edits",
  "submission_criteria",
  "side_effects",
  "control_points",
]);
export const ANALYTIC_VALUE_FIELDS = Object.freeze(["formula", "result_type"]);

/** FieldKind::Unknown(String) — unconstrained string, not a kind catalog. */
export const UNKNOWN_STRING_FIELD = "field_kind";

export const QUERY_PARAMS = Object.freeze(["key", "version"]);

export const CARDINALITY_ENUM = Object.freeze(["one_one", "one_many", "many_many"]);
export const DISPATCH_ENUM = Object.freeze(["projected_usecase", "instance_revision"]);

const WRITE_METHODS = new Set(["put", "post", "delete", "patch"]);
const HTTP_METHODS = new Set([
  "get",
  "put",
  "post",
  "delete",
  "options",
  "head",
  "patch",
  "trace",
]);

const FORBIDDEN_ENVELOPE = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  OBJECT_TYPE_SUMMARY,
  OBJECT_TYPE_RESPONSE,
  DRAFT_RECORD,
  CATALOG_ENTRY,
  OVERRIDE_SUMMARY,
  HR_READINESS_SUMMARY,
  DASHBOARD_RESPONSE,
  EXIT_CASE_RESPONSE,
]);

function push(findings, location, message) {
  findings.push({ location, message });
}

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

function jsonOkSchema(operation, code) {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function findOperation(paths, path, method) {
  const item = own(paths, path);
  return own(item, method);
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
}

function parameterNames(operation) {
  const parameters = own(operation, "parameters");
  if (!Array.isArray(parameters)) return [];
  return parameters
    .map((parameter) => own(parameter, "name"))
    .filter((name) => typeof name === "string");
}

function arrayItemName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function enumValues(schema) {
  if (!isPlainObject(schema)) return null;
  const listed = own(schema, "enum");
  return Array.isArray(listed) ? listed : null;
}

function isRootBag(schema) {
  if (!isPlainObject(schema)) return true;
  if (schemaRefName(schema)) return false;
  if (own(schema, "type") === "array") {
    return isRootBag(own(schema, "items"));
  }
  return own(schema, "additionalProperties") === true
    || schemaPropertyNames(schema).length === 0;
}

function isClosedCatalog(schema) {
  if (!isPlainObject(schema)) return false;
  if (typeof own(schema, "$ref") === "string") return true;
  if (Array.isArray(own(schema, "enum")) && own(schema, "enum").length > 0) return true;
  if (own(schema, "additionalProperties") === false) return true;
  const properties = own(schema, "properties");
  if (isPlainObject(properties) && Object.keys(properties).length > 0) return true;
  const required = own(schema, "required");
  if (Array.isArray(required) && required.length > 0) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (Array.isArray(members) && members.length > 0) return true;
  }
  if (own(schema, "type") === "array") {
    const items = own(schema, "items");
    if (isClosedCatalog(items)) return true;
    if (isPlainObject(items) && own(items, "type") && own(items, "type") !== "object") {
      return true;
    }
  }
  return false;
}

function isUnconstrainedJsonObject(schema) {
  if (!isPlainObject(schema)) return false;
  if (isClosedCatalog(schema)) return false;
  if (own(schema, "type") !== "object") return false;
  if (own(schema, "additionalProperties") !== true) return false;
  return true;
}

function isUnconstrainedString(schema) {
  return isPlainObject(schema) && own(schema, "type") === "string" && !enumValues(schema);
}

function isOpenString(schema) {
  if (!isPlainObject(schema) || enumValues(schema)) return false;
  const listed = own(schema, "type");
  if (listed === "string") return true;
  return Array.isArray(listed)
    && listed.includes("string")
    && listed.includes("null")
    && listed.length === 2;
}

function isUuidWire(schema) {
  if (!isPlainObject(schema)) return false;
  if (schemaRefName(schema) === "Uuid") return true;
  return own(schema, "type") === "string" && own(schema, "format") === "uuid";
}

function isUuidOrNull(schema) {
  if (isUuidWire(schema)) return true;
  if (!isPlainObject(schema)) return false;
  const listed = own(schema, "type");
  if (Array.isArray(listed) && listed.includes("string") && listed.includes("null")
    && listed.length === 2 && own(schema, "format") === "uuid") {
    return true;
  }
  const members = own(schema, "oneOf");
  if (!Array.isArray(members) || members.length !== 2) return false;
  const hasUuid = members.some((member) => isUuidWire(member));
  const hasNull = members.some((member) => isPlainObject(member) && own(member, "type") === "null");
  return hasUuid && hasNull;
}

function requireSchemaFields(schemas, findings, name, fields) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) {
    push(
      findings,
      `#/components/schemas/${name}`,
      `${name} must be derived from the existing handler struct — do not leave additionalProperties`,
    );
    return;
  }
  const names = schemaPropertyNames(schema);
  const missing = fields.filter((field) => !names.includes(field));
  const extra = names.filter((field) => !fields.includes(field));
  if (missing.length > 0 || extra.length > 0) {
    push(
      findings,
      `#/components/schemas/${name}/properties`,
      `${name} properties must match the existing Rust fields `
        + `(missing ${missing.join(",") || "none"}; extra ${extra.join(",") || "none"}). `
        + "Do not invent fields",
    );
  }
}

function requireRequiredList(schemas, findings, name, fields) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) return;
  const required = own(schema, "required");
  if (!Array.isArray(required) || required.join("\0") !== fields.join("\0")) {
    push(
      findings,
      `#/components/schemas/${name}/required`,
      `${name} required must match always-serialized non-null handler fields `
        + `(${fields.join(", ")})`,
    );
  }
}

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  if (objectName === OBJECT_TYPE_DETAIL || itemName === OBJECT_TYPE_DETAIL) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${OBJECT_TYPE_DETAIL}`,
    );
  }
}

function requireRustFields(findings, repoRoot, rel, structName, fields) {
  const rustPath = join(repoRoot, rel);
  if (!existsSync(rustPath)) return;
  const read = rustStructFields(readFileSync(rustPath, "utf8"), structName);
  if (!Array.isArray(read) || read.length === 0) {
    push(
      findings,
      `${rel}:${structName}`,
      `cannot read existing ${structName} fields; this slice binds the existing type`,
    );
  } else if (read.join("\0") !== fields.join("\0")) {
    push(
      findings,
      `${rel}:${structName}`,
      "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
    );
  }
}

function requireUnconstrainedValue(findings, schemas, schemaName, field) {
  const listed = own(own(own(schemas, schemaName), "properties"), field);
  if (isClosedCatalog(listed) || !isUnconstrainedJsonObject(listed)) {
    push(
      findings,
      `#/components/schemas/${schemaName}/properties/${field}`,
      `${field} is serde_json::Value; leave it unconstrained object / `
        + "additionalProperties: true on that field only — do not invent a nested catalog",
    );
  }
}

function requireClosedEnum(findings, schemas, schemaName, field, expected) {
  const listed = own(own(own(schemas, schemaName), "properties"), field);
  const values = enumValues(listed);
  if (!isPlainObject(listed) || own(listed, "type") !== "string"
    || !values || values.join("\0") !== expected.join("\0")) {
    push(
      findings,
      `#/components/schemas/${schemaName}/properties/${field}`,
      `${field} is an existing closed domain enum (${expected.join(", ")}); `
        + "type it from the Rust enum — do not invent extra kinds",
    );
  }
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   writes: number,
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiObjectTypeDetail({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let writes = 0;
  let gets = 0;
  let bound = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { writes: 0, gets: 0, bound: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      if (method === "get") gets += 1;
      if (WRITE_METHODS.has(method)) writes += 1;
    }
  }

  requireSchemaFields(schemas, findings, OBJECT_TYPE_DETAIL, DETAIL_FIELDS);
  requireSchemaFields(schemas, findings, PROPERTY_DEF_SUMMARY, PROPERTY_FIELDS);
  requireSchemaFields(schemas, findings, LINK_TYPE_SUMMARY, LINK_FIELDS);
  requireSchemaFields(schemas, findings, ACTION_TYPE_SUMMARY, ACTION_FIELDS);
  requireSchemaFields(schemas, findings, ANALYTIC_SUMMARY, ANALYTIC_FIELDS);
  requireRequiredList(schemas, findings, OBJECT_TYPE_DETAIL, DETAIL_REQUIRED);
  requireRequiredList(schemas, findings, PROPERTY_DEF_SUMMARY, PROPERTY_REQUIRED);
  requireRequiredList(schemas, findings, LINK_TYPE_SUMMARY, LINK_REQUIRED);
  requireRequiredList(schemas, findings, ACTION_TYPE_SUMMARY, ACTION_REQUIRED);
  requireRequiredList(schemas, findings, ANALYTIC_SUMMARY, ANALYTIC_REQUIRED);

  const envelope = own(schemas, OBJECT_TYPE_DETAIL);
  if (isPlainObject(envelope)) {
    if (own(envelope, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}`,
        `${OBJECT_TYPE_DETAIL} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only, not the whole 200 bag`,
      );
    }
    const properties = own(envelope, "properties");
    if (schemaRefName(own(properties, "object_type")) !== OBJECT_TYPE_SUMMARY) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/object_type`,
        `object_type is ${OBJECT_TYPE_SUMMARY}; $ref the existing nested struct — `
          + "do not bind the {key} GET itself to ObjectTypeSummary",
      );
    }
    if (arrayItemName(own(properties, "properties")) !== PROPERTY_DEF_SUMMARY) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/properties`,
        `properties is Vec<${PROPERTY_DEF_SUMMARY}>; array items must $ref the existing nested struct`,
      );
    }
    if (arrayItemName(own(properties, "links")) !== LINK_TYPE_SUMMARY) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/links`,
        `links is Vec<${LINK_TYPE_SUMMARY}>; array items must $ref the existing nested struct`,
      );
    }
    if (arrayItemName(own(properties, "actions")) !== ACTION_TYPE_SUMMARY) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/actions`,
        `actions is Vec<${ACTION_TYPE_SUMMARY}>; array items must $ref the existing nested struct`,
      );
    }
    if (arrayItemName(own(properties, "analytics")) !== ANALYTIC_SUMMARY) {
      push(
        findings,
        `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/analytics`,
        `analytics is Vec<${ANALYTIC_SUMMARY}>; array items must $ref the existing nested struct`,
      );
    }
    for (const field of ["title_property_key", "backing_table", "primary_key_property"]) {
      if (!isOpenString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${OBJECT_TYPE_DETAIL}/properties/${field}`,
          `${field} is Option<String> on the wire; open string (nullable), do not invent a catalog`,
        );
      }
    }
  }

  const property = own(schemas, PROPERTY_DEF_SUMMARY);
  if (isPlainObject(property)) {
    if (own(property, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${PROPERTY_DEF_SUMMARY}`,
        `${PROPERTY_DEF_SUMMARY} envelope must stay typed; additionalProperties: true belongs `
          + "on config only, not the whole property",
      );
    }
    const properties = own(property, "properties");
    if (!isUuidWire(own(properties, "id"))) {
      push(
        findings,
        `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/id`,
        "id is a transparent UUID newtype; type string format uuid (same as ObjectTypeSummary.id)",
      );
    }
    for (const field of ["key", "title", "field_type"]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    const fieldKind = own(properties, UNKNOWN_STRING_FIELD);
    if (isClosedCatalog(fieldKind) || !isUnconstrainedString(fieldKind)) {
      push(
        findings,
        `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/${UNKNOWN_STRING_FIELD}`,
        "field_kind is FieldKind with Unknown(String); leave it unconstrained string — "
          + "do not close FieldKind to a finite catalog of kinds",
      );
    }
    requireUnconstrainedValue(findings, schemas, PROPERTY_DEF_SUMMARY, "config");
    if (!isOpenString(own(properties, "backing_column"))) {
      push(
        findings,
        `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/backing_column`,
        "backing_column is Option<String>; open string (nullable), do not invent a catalog",
      );
    }
    for (const field of ["required", "in_property_policy"]) {
      const listed = own(properties, field);
      if (!isPlainObject(listed) || own(listed, "type") !== "boolean" || enumValues(listed)) {
        push(
          findings,
          `#/components/schemas/${PROPERTY_DEF_SUMMARY}/properties/${field}`,
          `${field} is bool on the wire; type boolean and do not invent a catalog`,
        );
      }
    }
  }

  const link = own(schemas, LINK_TYPE_SUMMARY);
  if (isPlainObject(link)) {
    if (own(link, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${LINK_TYPE_SUMMARY}`,
        `${LINK_TYPE_SUMMARY} envelope must stay typed; additionalProperties: true belongs on Value fields only`,
      );
    }
    const properties = own(link, "properties");
    if (!isUuidWire(own(properties, "id"))) {
      push(
        findings,
        `#/components/schemas/${LINK_TYPE_SUMMARY}/properties/id`,
        "id is a transparent UUID newtype; type string format uuid",
      );
    }
    for (const field of ["stable_key", "title"]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${LINK_TYPE_SUMMARY}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    if (!isOpenString(own(properties, "reverse_title"))) {
      push(
        findings,
        `#/components/schemas/${LINK_TYPE_SUMMARY}/properties/reverse_title`,
        "reverse_title is Option<String>; open string (nullable), do not invent a catalog",
      );
    }
    if (!isUuidOrNull(own(properties, "to_object_type_id"))) {
      push(
        findings,
        `#/components/schemas/${LINK_TYPE_SUMMARY}/properties/to_object_type_id`,
        "to_object_type_id is Option<ObjectTypeId>; uuid or null — do not invent a catalog",
      );
    }
    requireClosedEnum(findings, schemas, LINK_TYPE_SUMMARY, "cardinality", CARDINALITY_ENUM);
    const traversable = own(properties, "traversable");
    if (!isPlainObject(traversable) || own(traversable, "type") !== "boolean" || enumValues(traversable)) {
      push(
        findings,
        `#/components/schemas/${LINK_TYPE_SUMMARY}/properties/traversable`,
        "traversable is bool on the wire; type boolean and do not invent a catalog",
      );
    }
  }

  const action = own(schemas, ACTION_TYPE_SUMMARY);
  if (isPlainObject(action)) {
    if (own(action, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${ACTION_TYPE_SUMMARY}`,
        `${ACTION_TYPE_SUMMARY} envelope must stay typed; additionalProperties: true belongs `
          + "on params_schema / edits / submission_criteria / side_effects / control_points only",
      );
    }
    const properties = own(action, "properties");
    if (!isUuidWire(own(properties, "id"))) {
      push(
        findings,
        `#/components/schemas/${ACTION_TYPE_SUMMARY}/properties/id`,
        "id is a transparent UUID newtype; type string format uuid",
      );
    }
    for (const field of ["stable_key", "title"]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${ACTION_TYPE_SUMMARY}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    for (const field of ACTION_VALUE_FIELDS) {
      requireUnconstrainedValue(findings, schemas, ACTION_TYPE_SUMMARY, field);
    }
    requireClosedEnum(findings, schemas, ACTION_TYPE_SUMMARY, "dispatch", DISPATCH_ENUM);
    if (!isOpenString(own(properties, "dispatch_target"))) {
      push(
        findings,
        `#/components/schemas/${ACTION_TYPE_SUMMARY}/properties/dispatch_target`,
        "dispatch_target is Option<String>; open string (nullable), do not invent a catalog",
      );
    }
  }

  const analytic = own(schemas, ANALYTIC_SUMMARY);
  if (isPlainObject(analytic)) {
    if (own(analytic, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${ANALYTIC_SUMMARY}`,
        `${ANALYTIC_SUMMARY} envelope must stay typed; additionalProperties: true belongs `
          + "on formula / result_type only",
      );
    }
    const properties = own(analytic, "properties");
    if (!isUuidWire(own(properties, "id"))) {
      push(
        findings,
        `#/components/schemas/${ANALYTIC_SUMMARY}/properties/id`,
        "id is a transparent UUID newtype; type string format uuid",
      );
    }
    for (const field of ["key", "title"]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${ANALYTIC_SUMMARY}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    for (const field of ANALYTIC_VALUE_FIELDS) {
      requireUnconstrainedValue(findings, schemas, ANALYTIC_SUMMARY, field);
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, OBJECT_TYPE_DETAIL, DETAIL_FIELDS);
  requireRustFields(findings, repoRoot, STORE_RS_REL, PROPERTY_DEF_SUMMARY, PROPERTY_FIELDS);
  requireRustFields(findings, repoRoot, STORE_RS_REL, LINK_TYPE_SUMMARY, LINK_FIELDS);
  requireRustFields(findings, repoRoot, STORE_RS_REL, ACTION_TYPE_SUMMARY, ACTION_FIELDS);
  requireRustFields(findings, repoRoot, STORE_RS_REL, ANALYTIC_SUMMARY, ANALYTIC_FIELDS);

  const location = `#/paths/${OBJECT_TYPE_GET_PATH}/get`;
  const operation = findOperation(paths, OBJECT_TYPE_GET_PATH, "get");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `GET ${OBJECT_TYPE_GET_PATH} must remain published `
        + `(runtime already serves ${OBJECT_TYPE_DETAIL})`,
    );
  } else {
    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto GET ${OBJECT_TYPE_GET_PATH}`,
      );
    }

    const listedParams = parameterNames(operation);
    if (listedParams.join("\0") !== QUERY_PARAMS.join("\0")) {
      push(
        findings,
        `${location}/parameters`,
        `GET ${OBJECT_TYPE_GET_PATH} already publishes ${QUERY_PARAMS.join(", ")}; `
          + "do not invent as_of or other query params",
      );
    }

    const body = jsonOkSchema(operation, "200");
    const boundName = schemaRefName(body);
    const responseLoc = `${location}/responses/200`;
    if (FORBIDDEN_ENVELOPE.includes(boundName)) {
      push(
        findings,
        responseLoc,
        `GET ${OBJECT_TYPE_GET_PATH} already returns ${OBJECT_TYPE_DETAIL}; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / ObjectTypeSummary is the list GET / ObjectTypeResponse is registry kind + active_count / "
          + "DraftRecord is a policy draft / CatalogEntry is the enforced catalog / "
          + "OverrideSummary is a governance override / readiness and HR envelopes are different wires)",
      );
    } else if (boundName === OBJECT_TYPE_DETAIL) {
      bound += 1;
    } else if (isRootBag(body)) {
      push(
        findings,
        responseLoc,
        `GET ${OBJECT_TYPE_GET_PATH} already returns ${OBJECT_TYPE_DETAIL}; `
          + `200 must $ref ${OBJECT_TYPE_DETAIL}, not a root additionalProperties bag`,
      );
    } else {
      push(
        findings,
        responseLoc,
        `GET ${OBJECT_TYPE_GET_PATH} already returns ${OBJECT_TYPE_DETAIL}; `
          + `200 must $ref ${OBJECT_TYPE_DETAIL}, not additionalProperties`,
      );
    }
  }

  const list = findOperation(paths, OBJECT_TYPES_LIST_PATH, "get");
  const listSchema = jsonOkSchema(list, "200");
  const listItem = arrayItemName(listSchema);
  if (listItem !== OBJECT_TYPE_SUMMARY) {
    push(
      findings,
      `#/paths/${OBJECT_TYPES_LIST_PATH}/get/responses/200`,
      `GET ${OBJECT_TYPES_LIST_PATH} must remain an array of ${OBJECT_TYPE_SUMMARY}; `
        + `do not bind the list to ${OBJECT_TYPE_DETAIL}`,
    );
  }
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPES_LIST_PATH}/get/responses/200`,
    listSchema,
    `GET ${OBJECT_TYPES_LIST_PATH} already returns Vec<ObjectTypeSummary>`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_PUT_PATH}/put/responses/201`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_PUT_PATH, "put"), "201"),
    `PUT ${OBJECT_TYPE_PUT_PATH} already returns ObjectTypeSummary, not ObjectTypeDetail`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, ABSENCE_EXIT_GET_PATH, "get"), "200"),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns AbsenceExitDashboardResponse`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${KILL_SWITCH_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, KILL_SWITCH_PATH, "post"), "200"),
    `POST ${KILL_SWITCH_PATH} is console kill-switch, not an ontology detail`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_OPT_IN_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_OPT_IN_PATH, "put"), "200"),
    `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not an ontology detail`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_ORG_FLAG_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_ORG_FLAG_PATH, "put"), "200"),
    `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not an ontology detail`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DRAFTS_PATH, "post"), "201"),
    `POST ${DRAFTS_PATH} already returns DraftRecord, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${CATALOG_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, CATALOG_GET_PATH, "get"), "200"),
    `GET ${CATALOG_GET_PATH} already returns CatalogEntry, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OVERRIDES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, OVERRIDES_PATH, "post"), "201"),
    `POST ${OVERRIDES_PATH} already returns OverrideSummary, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${READINESS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, READINESS_GET_PATH, "get"), "200"),
    `GET ${READINESS_GET_PATH} already returns HrReadinessSummary, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXIT_CASES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, EXIT_CASES_PATH, "post"), "201"),
    `POST ${EXIT_CASES_PATH} already returns ${EXIT_CASE_RESPONSE}, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not ${OBJECT_TYPE_DETAIL}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post"), "200"),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );

  return { writes, gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiObjectTypeDetail({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { writes, gets, bound, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowWriteFloor = writes < WRITE_FLOOR;
  const belowGetFloor = gets < GET_FLOOR;
  if (belowWriteFloor) {
    console.error(
      `saw ${writes} write operations — below the floor ${WRITE_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowWriteFloor || belowGetFloor || bound !== BOUND) {
    console.error(
      `openapi object-type-detail typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), ${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi object-type-detail typed-response gate passed `
      + `(${OBJECT_TYPE_DETAIL} $ref; ${VALUE_FIELDS.join(" / ")} unconstrained; `
      + `${UNKNOWN_STRING_FIELD} unconstrained string; `
      + `${writes} write operations, ${gets} GET operations, 0 findings)`,
  );
}
