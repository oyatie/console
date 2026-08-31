// Policy DraftRecord envelope bind gate.
//
// The hole this closes: list/create/get/update/review/submit/validate already
// return Json<DraftRecord> / Json<Vec<DraftRecord>> whose envelope fields are
// Uuid / String / OffsetDateTime, with normalized_row and validation_errors
// as serde_json::Value and reviewer_id as Option<Uuid>. Composed OpenAPI still
// advertises those 200/201 bodies as additionalProperties: true, so clients
// cannot see the existing envelope. Same class as #1033 OverrideSummary —
// publish the existing record, do not invent a store, policy catalog, or
// ObjectKey.
//
// Chesterton: face YAML from the existing struct, then $ref. Type every
// non-Value field. normalized_row and validation_errors stay unconstrained
// object / additionalProperties: true on THOSE FIELDS ONLY (not enums, not
// nested invented schemas). Do not bind catalog / OverrideSummary /
// DecisionResponse / ObjectTypeDetail / absence-exit. Do not map
// Feature::ALL permissions. Do not stamp HTTP ETag.
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

export const DRAFTS_PATH = "/api/v1/policy/drafts";
export const DRAFT_ID_PATH = "/api/v1/policy/drafts/{draft_id}";
export const REVIEW_PATH = "/api/v1/policy/drafts/{draft_id}/review";
export const SUBMIT_PATH = "/api/v1/policy/drafts/{draft_id}/submit";
export const VALIDATE_PATH = "/api/v1/policy/drafts/{draft_id}/validate";
export const CATALOG_GET_PATH = "/api/v1/policy/catalog";
export const AUTHORIZE_PATH = "/api/v1/policy/authorize";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const DECIDE_PATH = "/api/v1/governance/approvals/decide";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const DRAFT_RECORD = "DraftRecord";
export const CATALOG_ENTRY = "CatalogEntry";
export const OVERRIDE_SUMMARY = "OverrideSummary";
export const DECISION_RESPONSE = "DecisionResponse";
export const APPROVAL_SUMMARY = "ApprovalSummary";
export const POLICY_CREATE_DRAFT_REQUEST = "PolicyCreateDraftRequest";
export const POLICY_UPDATE_DRAFT_REQUEST = "PolicyUpdateDraftRequest";
export const BOUND = 7;

export const STORE_RS_REL = "backend/crates/platform/authz-rest/src/store.rs";
export const STORE_STRUCT = "DraftRecord";

/** Existing Serialize fields on DraftRecord. Do not invent names. */
export const RECORD_FIELDS = Object.freeze([
  "id",
  "draft_key",
  "title",
  "normalized_row",
  "generated_policy_text",
  "validation_status",
  "validation_errors",
  "review_status",
  "reviewer_id",
  "created_by",
  "created_at",
  "updated_at",
]);

/** serde_json::Value on the wire. Do not close these into catalogs. */
export const VALUE_FIELDS = Object.freeze([
  "normalized_row",
  "validation_errors",
]);

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

const UUID_FIELDS = Object.freeze(["id", "created_by"]);
const STRING_FIELDS = Object.freeze([
  "draft_key",
  "title",
  "generated_policy_text",
  "validation_status",
  "review_status",
]);
const TIMESTAMP_FIELDS = Object.freeze(["created_at", "updated_at"]);
const NULLABLE_UUID_FIELD = "reviewer_id";

const FORBIDDEN_ENVELOPE = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  CATALOG_ENTRY,
  OVERRIDE_SUMMARY,
  DECISION_RESPONSE,
  APPROVAL_SUMMARY,
  POLICY_CREATE_DRAFT_REQUEST,
  POLICY_UPDATE_DRAFT_REQUEST,
]);

/** Path + method + success code + whether the 2xx is an array of DraftRecord. */
export const BINDINGS = Object.freeze([
  Object.freeze({
    path: DRAFTS_PATH,
    method: "get",
    code: "200",
    array: true,
  }),
  Object.freeze({
    path: DRAFTS_PATH,
    method: "post",
    code: "201",
    array: false,
  }),
  Object.freeze({
    path: DRAFT_ID_PATH,
    method: "get",
    code: "200",
    array: false,
  }),
  Object.freeze({
    path: DRAFT_ID_PATH,
    method: "put",
    code: "200",
    array: false,
  }),
  Object.freeze({
    path: REVIEW_PATH,
    method: "post",
    code: "200",
    array: false,
  }),
  Object.freeze({
    path: SUBMIT_PATH,
    method: "post",
    code: "200",
    array: false,
  }),
  Object.freeze({
    path: VALIDATE_PATH,
    method: "post",
    code: "200",
    array: false,
  }),
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

function jsonOkHeaders(operation, code) {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  return own(ok, "headers");
}

function findOperation(paths, path, method) {
  const item = own(paths, path);
  return own(item, method);
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
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

function isNullableUuid(schema) {
  if (!isPlainObject(schema)) return false;
  const members = own(schema, "oneOf");
  if (!Array.isArray(members) || members.length !== 2) return false;
  let hasUuid = false;
  let hasNull = false;
  for (const member of members) {
    if (schemaRefName(member) === "Uuid") hasUuid = true;
    if (isPlainObject(member) && own(member, "type") === "null") hasNull = true;
  }
  return hasUuid && hasNull;
}

function boundNameOf(schema, array) {
  if (array) return arrayItemName(schema);
  return schemaRefName(schema);
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

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  if (objectName === DRAFT_RECORD || itemName === DRAFT_RECORD) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${DRAFT_RECORD}`,
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

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   writes: number,
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiDraftRecord({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, DRAFT_RECORD, RECORD_FIELDS);

  const envelope = own(schemas, DRAFT_RECORD);
  if (isPlainObject(envelope)) {
    if (own(envelope, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${DRAFT_RECORD}`,
        `${DRAFT_RECORD} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only, not the whole 200/201 bag`,
      );
    }
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== RECORD_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${DRAFT_RECORD}/required`,
        `${DRAFT_RECORD} required must match always-serialized handler fields `
          + `(${RECORD_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    for (const field of UUID_FIELDS) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${DRAFT_RECORD}/properties/${field}`,
          `${field} must $ref existing Uuid`,
        );
      }
    }
    for (const field of STRING_FIELDS) {
      const listed = own(properties, field);
      if (!isPlainObject(listed) || own(listed, "type") !== "string" || enumValues(listed)) {
        push(
          findings,
          `#/components/schemas/${DRAFT_RECORD}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    for (const field of VALUE_FIELDS) {
      const listed = own(properties, field);
      if (isClosedCatalog(listed) || !isUnconstrainedJsonObject(listed)) {
        push(
          findings,
          `#/components/schemas/${DRAFT_RECORD}/properties/${field}`,
          `${field} is serde_json::Value; leave it unconstrained object / `
            + "additionalProperties: true on that field only — do not invent a catalog",
        );
      }
    }
    if (!isNullableUuid(own(properties, NULLABLE_UUID_FIELD))) {
      push(
        findings,
        `#/components/schemas/${DRAFT_RECORD}/properties/${NULLABLE_UUID_FIELD}`,
        `${NULLABLE_UUID_FIELD} is Option<Uuid>; oneOf existing Uuid $ref and JSON null — `
          + "do not drop null or invent a catalog",
      );
    }
    for (const field of TIMESTAMP_FIELDS) {
      if (schemaRefName(own(properties, field)) !== "Timestamp") {
        push(
          findings,
          `#/components/schemas/${DRAFT_RECORD}/properties/${field}`,
          `${field} must $ref existing Timestamp (rfc3339 OffsetDateTime)`,
        );
      }
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, STORE_STRUCT, RECORD_FIELDS);

  for (const binding of BINDINGS) {
    const location = `#/paths/${binding.path}/${binding.method}`;
    const operation = findOperation(paths, binding.path, binding.method);
    if (!isPlainObject(operation)) {
      push(
        findings,
        location,
        `${binding.method.toUpperCase()} ${binding.path} must remain published `
          + "(runtime already serves DraftRecord)",
      );
      continue;
    }

    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto ${binding.method.toUpperCase()} ${binding.path}`,
      );
    }

    const headers = jsonOkHeaders(operation, binding.code);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${location}/responses/${binding.code}/headers/ETag`,
        "policy draft handlers do not send ETag; HTTP ETag stays HOLD — do not stamp it here",
      );
    }

    const body = jsonOkSchema(operation, binding.code);
    const boundName = boundNameOf(body, binding.array);
    const responseLoc = `${location}/responses/${binding.code}`;
    if (FORBIDDEN_ENVELOPE.includes(boundName)) {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns DraftRecord; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / CatalogEntry is the enforced catalog / "
          + "OverrideSummary is a governance override / "
          + "DecisionResponse is authorize/simulate / request schema hides id)",
      );
    } else if (boundName === DRAFT_RECORD) {
      bound += 1;
    } else if (isRootBag(body)) {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns DraftRecord; `
          + `${binding.code} must $ref ${DRAFT_RECORD}`
          + `${binding.array ? " as array items" : ""}, not a root additionalProperties bag`,
      );
    } else {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns DraftRecord; `
          + `${binding.code} must $ref ${DRAFT_RECORD}`
          + `${binding.array ? " as array items" : ""}, not additionalProperties`,
      );
    }
  }

  refuseForeignBind(
    findings,
    `#/paths/${CATALOG_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, CATALOG_GET_PATH, "get"), "200"),
    `GET ${CATALOG_GET_PATH} already returns CatalogEntry, not DraftRecord`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${AUTHORIZE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, AUTHORIZE_PATH, "post"), "200"),
    `POST ${AUTHORIZE_PATH} already returns DecisionResponse, not DraftRecord`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OVERRIDES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, OVERRIDES_PATH, "post"), "201"),
    `POST ${OVERRIDES_PATH} already returns OverrideSummary, not DraftRecord`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DECIDE_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DECIDE_PATH, "post"), "201"),
    `POST ${DECIDE_PATH} already returns ApprovalSummary, not DraftRecord`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not DraftRecord`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post"), "200"),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_GET_PATH, "get"), "200"),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, ABSENCE_EXIT_GET_PATH, "get"), "200"),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns nested serde_json::Value bags`,
  );

  return { writes, gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiDraftRecord({ repoRoot });
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
      `openapi draft-record typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), ${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi draft-record typed-response gate passed `
      + `(${DRAFT_RECORD} $ref; ${VALUE_FIELDS.join(" / ")} unconstrained; `
      + `${writes} write operations, ${gets} GET operations, 0 findings)`,
  );
}
