// POST /api/v1/governance/overrides 201 OverrideSummary bind gate.
//
// The hole this closes: open_override already returns Json<OverrideSummary>
// whose envelope fields are Uuid / String / UserId / OffsetDateTime, with
// before_snapshot: serde_json::Value. Composed OpenAPI still advertises the
// whole 201 as additionalProperties: true, so clients cannot see the existing
// envelope. Same class as #1031 ApprovalSummary / #1032
// LifecycleTransitionConfig — publish the existing summary, do not invent a
// store, snapshot catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing struct, then $ref. Type every
// non-Value field. before_snapshot stays unconstrained object /
// additionalProperties: true on THAT FIELD ONLY (not an enum, not a nested
// invented schema). Do not bind decide-approval (ApprovalSummary has no
// Value). Do not bind lifecycle transitions (no Value). Do not bind create-
// approval / drafts / ObjectTypeDetail (different Value bags). Do not map
// Feature::ALL permissions. Do not stamp HTTP ETag.
//
// Totality: js-yaml load + own-property walk of every write method + optional
// Rust struct field read. WRITE_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = PREFLIGHT_WRITE_FLOOR;

export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const DECIDE_PATH = "/api/v1/governance/approvals/decide";
export const CREATE_PATH = "/api/v1/governance/approvals";
export const TRANSITIONS_PATH = "/api/v1/governance/lifecycle/transitions";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const OVERRIDE_SUMMARY = "OverrideSummary";
export const OPEN_OVERRIDE_REQUEST = "GovernanceOpenOverrideRequest";
export const APPROVAL_SUMMARY = "ApprovalSummary";
export const LIFECYCLE_TRANSITION_CONFIG = "LifecycleTransitionConfig";
export const LIFECYCLE_PREFLIGHT = "LifecyclePreflight";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const DECISION_RESPONSE = "DecisionResponse";
export const BOUND = 1;
export const OK_CODE = "201";

export const STORE_RS_REL = "backend/crates/governance/application/src/lib.rs";
export const STORE_STRUCT = "OverrideSummary";

/** Existing Serialize fields on OverrideSummary. Do not invent names. */
export const SUMMARY_FIELDS = Object.freeze([
  "id",
  "target_type",
  "target_id",
  "actor",
  "reason",
  "before_snapshot",
  "created_at",
]);

/** serde_json::Value on the wire. Do not close this into a catalog. */
export const VALUE_FIELD = "before_snapshot";

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

const UUID_FIELDS = Object.freeze(["id", "target_id", "actor"]);
const STRING_FIELDS = Object.freeze(["target_type", "reason"]);

const FORBIDDEN_201 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  OPEN_OVERRIDE_REQUEST,
  APPROVAL_SUMMARY,
  LIFECYCLE_TRANSITION_CONFIG,
  LIFECYCLE_PREFLIGHT,
  PREFLIGHT_OUTCOME,
  DECISION_RESPONSE,
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

function jsonOkSchema(operation, code = OK_CODE) {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function jsonOkHeaders(operation, code = OK_CODE) {
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
  return false;
}

function isUnconstrainedJsonObject(schema) {
  if (!isPlainObject(schema)) return false;
  if (isClosedCatalog(schema)) return false;
  if (own(schema, "type") !== "object") return false;
  if (own(schema, "additionalProperties") !== true) return false;
  return true;
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
  if (objectName === OVERRIDE_SUMMARY || itemName === OVERRIDE_SUMMARY) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${OVERRIDE_SUMMARY}`,
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
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiOverrideSummary({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let writes = 0;
  let bound = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { writes: 0, bound: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      if (!WRITE_METHODS.has(method)) continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      writes += 1;
    }
  }

  requireSchemaFields(schemas, findings, OVERRIDE_SUMMARY, SUMMARY_FIELDS);

  const envelope = own(schemas, OVERRIDE_SUMMARY);
  if (isPlainObject(envelope)) {
    if (own(envelope, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${OVERRIDE_SUMMARY}`,
        `${OVERRIDE_SUMMARY} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELD} only, not the whole 201 bag`,
      );
    }
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== SUMMARY_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${OVERRIDE_SUMMARY}/required`,
        `${OVERRIDE_SUMMARY} required must match always-serialized handler fields `
          + `(${SUMMARY_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    for (const field of UUID_FIELDS) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${field}`,
          `${field} must $ref existing Uuid (UserId is a transparent UUID newtype)`,
        );
      }
    }
    for (const field of STRING_FIELDS) {
      const listed = own(properties, field);
      if (!isPlainObject(listed) || own(listed, "type") !== "string" || enumValues(listed)) {
        push(
          findings,
          `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    const snapshot = own(properties, VALUE_FIELD);
    if (isClosedCatalog(snapshot) || !isUnconstrainedJsonObject(snapshot)) {
      push(
        findings,
        `#/components/schemas/${OVERRIDE_SUMMARY}/properties/${VALUE_FIELD}`,
        `${VALUE_FIELD} is serde_json::Value; leave it unconstrained object / `
          + "additionalProperties: true on that field only — do not invent a snapshot catalog",
      );
    }
    if (schemaRefName(own(properties, "created_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${OVERRIDE_SUMMARY}/properties/created_at`,
        "created_at must $ref existing Timestamp (rfc3339 OffsetDateTime)",
      );
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, STORE_STRUCT, SUMMARY_FIELDS);

  const location = `#/paths/${OVERRIDES_PATH}/post`;
  const operation = findOperation(paths, OVERRIDES_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${OVERRIDES_PATH} must remain published (runtime already serves it)`,
    );
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${OVERRIDES_PATH}`,
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/${OK_CODE}/headers/ETag`,
      "open_override does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_201.includes(boundName)) {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${OVERRIDES_PATH} already returns OverrideSummary `
        + `(id / target_type / target_id / actor / reason / before_snapshot / created_at); `
        + `do not bind it to ${boundName} `
        + "(Head HOLD / request schema hides id/actor/created_at / "
        + "ApprovalSummary is a four-eyes decision / "
        + "LifecycleTransitionConfig is a different type)",
    );
  } else if (boundName === OVERRIDE_SUMMARY) {
    bound += 1;
  } else if (isRootBag(body)) {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${OVERRIDES_PATH} already returns OverrideSummary; `
        + `201 must $ref ${OVERRIDE_SUMMARY}, not a root additionalProperties bag`,
    );
  } else {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${OVERRIDES_PATH} already returns OverrideSummary; `
        + `201 must $ref ${OVERRIDE_SUMMARY}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${DECIDE_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DECIDE_PATH, "post")),
    `POST ${DECIDE_PATH} already returns ApprovalSummary, not OverrideSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${CREATE_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, CREATE_PATH, "post")),
    `POST ${CREATE_PATH} already returns ApprovalRequestSummary (payload_summary: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${TRANSITIONS_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, TRANSITIONS_PATH, "post")),
    `POST ${TRANSITIONS_PATH} already returns LifecycleTransitionConfig, not OverrideSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post"), "200"),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not OverrideSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not OverrideSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post"), "200"),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, DRAFTS_GET_PATH, "get"), "200"),
    `GET ${DRAFTS_GET_PATH} already returns DraftRecord (nested serde_json::Value)`,
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

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiOverrideSummary({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { writes, bound, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowWriteFloor = writes < WRITE_FLOOR;
  if (belowWriteFloor) {
    console.error(
      `saw ${writes} write operations — below the floor ${WRITE_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowWriteFloor || bound !== BOUND) {
    console.error(
      `openapi override-summary typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi override-summary typed-response gate passed `
      + `(${OVERRIDE_SUMMARY} $ref; ${VALUE_FIELD} unconstrained; ${writes} write operations, 0 findings)`,
  );
}
