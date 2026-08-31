// GET /api/v1/policy/catalog 200 CatalogEntry bind gate.
//
// The hole this closes: list_catalog already returns Json<Vec<CatalogEntry>>
// whose fields are Uuid / String / OffsetDateTime (no serde_json::Value).
// Composed OpenAPI still advertises array items as additionalProperties: true,
// so clients cannot see the existing wire. Same class as #1022 AuditRecord /
// #1024 HrReadinessSummary $ref — derive the schema from the existing DTO
// fields. Do not invent a store, status enum, or draft Value bag.
//
// Chesterton: face YAML from the existing struct, then $ref. Do not bind
// GET /api/v1/policy/drafts (DraftRecord has nested serde_json::Value). Do not
// publish GET /api/v1/group-admin/groups as an ontology Head (Group ObjectKey
// HOLD). Do not map Feature::ALL permissions. Do not stamp HTTP ETag. Keep the
// existing status query as unconstrained string (handler CatalogQuery.status
// is Option<String>).
//
// Totality: js-yaml load + own-property walk of every GET + optional Rust
// struct field read. GET_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const CATALOG_GET_PATH = "/api/v1/policy/catalog";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const GROUP_ADMIN_GROUPS_GET_PATH = "/api/v1/group-admin/groups";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const CATALOG_ENTRY = "CatalogEntry";
export const STATUS = "status";
export const BOUND = 1;

export const CATALOG_ENTRY_RS_REL = "backend/crates/platform/authz-rest/src/store.rs";
export const CATALOG_ENTRY_STRUCT = "CatalogEntry";

/** Existing Serialize fields on CatalogEntry. Do not invent names. */
export const CATALOG_ENTRY_FIELDS = Object.freeze([
  "id",
  "stable_key",
  "title",
  "effect",
  "status",
  "source",
  "validation_status",
  "updated_at",
]);

/** Rust `String` fields — unconstrained TEXT, not an invented enum catalog. */
export const CATALOG_STRING_FIELDS = Object.freeze([
  "stable_key",
  "title",
  "effect",
  "status",
  "source",
  "validation_status",
]);

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

function json200Schema(operation) {
  const responses = own(operation, "responses");
  const ok = own(responses, "200") ?? own(responses, 200);
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function json200Headers(operation) {
  const responses = own(operation, "responses");
  const ok = own(responses, "200") ?? own(responses, 200);
  return own(ok, "headers");
}

function parameterList(operation) {
  const parameters = own(operation, "parameters");
  return Array.isArray(parameters) ? parameters : [];
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
}

function arrayItemName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
}

function isCatalogSchema(schema) {
  if (!isPlainObject(schema)) return false;
  if (Array.isArray(own(schema, "enum")) && own(schema, "enum").length > 0) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (Array.isArray(members) && members.length > 0) {
      if (members.some((member) => isCatalogSchema(member))) return true;
    }
  }
  return false;
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function queryParam(operation, name) {
  return parameterList(operation).find((parameter) => {
    if (!isPlainObject(parameter)) return false;
    return own(parameter, "name") === name && own(parameter, "in") === "query";
  }) ?? null;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiPolicyCatalog({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let gets = 0;
  let bound = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { gets: 0, bound: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      if (method !== "get") continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      gets += 1;
    }
  }

  const record = own(schemas, CATALOG_ENTRY);
  if (!isPlainObject(record)) {
    push(
      findings,
      `#/components/schemas/${CATALOG_ENTRY}`,
      `${CATALOG_ENTRY} must be derived from the existing handler struct — do not leave items additionalProperties`,
    );
  } else {
    const names = schemaPropertyNames(record);
    const missing = CATALOG_ENTRY_FIELDS.filter((name) => !names.includes(name));
    const extra = names.filter((name) => !CATALOG_ENTRY_FIELDS.includes(name));
    if (missing.length > 0 || extra.length > 0) {
      push(
        findings,
        `#/components/schemas/${CATALOG_ENTRY}/properties`,
        `${CATALOG_ENTRY} properties must match the existing Rust fields `
          + `(missing ${missing.join(",") || "none"}; extra ${extra.join(",") || "none"}). `
          + "Do not invent fields",
      );
    }
    const properties = own(record, "properties");
    for (const name of CATALOG_STRING_FIELDS) {
      const field = own(properties, name);
      if (isCatalogSchema(field) || (isPlainObject(field) && own(field, "type") !== "string")) {
        push(
          findings,
          `#/components/schemas/${CATALOG_ENTRY}/properties/${name}`,
          `CatalogEntry.${name} is String; do not type it as an enum / catalog`,
        );
      }
    }
    if (schemaRefName(own(properties, "id")) !== "Uuid") {
      push(
        findings,
        `#/components/schemas/${CATALOG_ENTRY}/properties/id`,
        "id is Uuid; $ref Uuid like sibling identity schemas, do not invent an identifier",
      );
    }
    if (schemaRefName(own(properties, "updated_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${CATALOG_ENTRY}/properties/updated_at`,
        "updated_at is OffsetDateTime; $ref Timestamp like sibling audit schemas, do not invent a clock",
      );
    }
  }

  const rustPath = join(repoRoot, CATALOG_ENTRY_RS_REL);
  if (existsSync(rustPath)) {
    const fields = rustStructFields(readFileSync(rustPath, "utf8"), CATALOG_ENTRY_STRUCT);
    if (!Array.isArray(fields) || fields.length === 0) {
      push(
        findings,
        `${CATALOG_ENTRY_RS_REL}:${CATALOG_ENTRY_STRUCT}`,
        "cannot read existing CatalogEntry fields; this slice binds the handler type",
      );
    } else if (fields.join("\0") !== CATALOG_ENTRY_FIELDS.join("\0")) {
      push(
        findings,
        `${CATALOG_ENTRY_RS_REL}:${CATALOG_ENTRY_STRUCT}`,
        "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
      );
    }
  }

  const location = `#/paths/${CATALOG_GET_PATH}/get`;
  const operation = findGet(paths, CATALOG_GET_PATH);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${CATALOG_GET_PATH} must remain published (runtime already serves it)`);
    return { gets, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto GET /api/v1/policy/catalog "
        + "(RoleManage stays the REST gate, unpublished here)",
    );
  }

  const statusParam = queryParam(operation, STATUS);
  if (!isPlainObject(statusParam)) {
    push(
      findings,
      `${location}/parameters/${STATUS}`,
      "list_catalog already filters on CatalogQuery.status: Option<String>; do not drop the published status query",
    );
  } else {
    const schema = own(statusParam, "schema");
    if (isCatalogSchema(schema) || (isPlainObject(schema) && own(schema, "type") !== "string")) {
      push(
        findings,
        `${location}/parameters/${STATUS}`,
        "CatalogQuery.status is Option<String>; do not type the status query as an enum / catalog",
      );
    }
  }

  const extraParams = parameterList(operation).filter((parameter) => {
    if (!isPlainObject(parameter)) return true;
    return !(own(parameter, "name") === STATUS && own(parameter, "in") === "query");
  });
  if (extraParams.length > 0) {
    push(
      findings,
      `${location}/parameters`,
      "list_catalog Query is status only; do not invent as_of, pagination, or extra filters",
    );
  }

  const headers = json200Headers(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "list_catalog does not send ETag; HTTP ETag stays HOLD — do not stamp it on this GET",
    );
  }

  if (arrayItemName(json200Schema(operation)) === CATALOG_ENTRY) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `GET ${CATALOG_GET_PATH} already returns Json<Vec<${CATALOG_ENTRY}>>; `
        + `200 items must $ref ${CATALOG_ENTRY}, not additionalProperties`,
    );
  }

  const drafts = findGet(paths, DRAFTS_GET_PATH);
  if (isPlainObject(drafts)) {
    const schema = json200Schema(drafts);
    const objectName = schemaRefName(schema);
    const itemName = arrayItemName(schema);
    if (objectName === CATALOG_ENTRY || itemName === CATALOG_ENTRY) {
      push(
        findings,
        `#/paths/${DRAFTS_GET_PATH}/get/responses/200`,
        `GET ${DRAFTS_GET_PATH} already returns DraftRecord (nested serde_json::Value); `
          + `do not bind it to ${CATALOG_ENTRY}`,
      );
    }
  }

  const groups = findGet(paths, GROUP_ADMIN_GROUPS_GET_PATH);
  if (isPlainObject(groups)) {
    const schema = json200Schema(groups);
    const objectName = schemaRefName(schema);
    const itemName = arrayItemName(schema);
    if (objectName === CATALOG_ENTRY || itemName === CATALOG_ENTRY) {
      push(
        findings,
        `#/paths/${GROUP_ADMIN_GROUPS_GET_PATH}/get/responses/200`,
        `GET ${GROUP_ADMIN_GROUPS_GET_PATH} is not an ontology Head (Group ObjectKey HOLD); `
          + `do not bind it to ${CATALOG_ENTRY}`,
      );
    }
  }

  const detail = findGet(paths, OBJECT_TYPE_GET_PATH);
  if (isPlainObject(detail)) {
    const schema = json200Schema(detail);
    const objectName = schemaRefName(schema);
    const itemName = arrayItemName(schema);
    if (objectName === CATALOG_ENTRY || itemName === CATALOG_ENTRY) {
      push(
        findings,
        `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
        `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail; do not bind it to ${CATALOG_ENTRY}`,
      );
    }
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiPolicyCatalog({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, bound, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor || bound !== BOUND) {
    console.error(
      `openapi policy-catalog typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi policy-catalog typed-response gate passed `
      + `(${CATALOG_ENTRY} items $ref; ${gets} GET operations, 0 findings)`,
  );
}
