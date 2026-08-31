// POST /api/v1/governance/approvals/decide 201 ApprovalSummary bind gate.
//
// The hole this closes: decide_approval already returns Json<ApprovalSummary>
// whose fields are Uuid / String / UserId / ApprovalDecision / OffsetDateTime
// (no serde_json::Value). record_decision refuses pending. Composed OpenAPI
// still advertises additionalProperties: true, so clients cannot see the
// existing wire. Same class as #1029 LifecyclePreflight / #1030
// GroupAdminTenantContextStartResponse — publish the existing summary, do not
// invent a store, GateKind catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing struct, then $ref. decision enum is
// approved/rejected (the request already publishes those two; this 201 refuses
// pending). Do not bind create-approval (ApprovalRequestSummary.payload_summary
// is Value). Do not bind overrides (OverrideSummary.before_snapshot is Value).
// Do not bind lifecycle transitions (LifecycleTransitionConfig is a different
// type). Do not bind policy-draft Value bags. Do not map Feature::ALL
// permissions. Do not stamp HTTP ETag.
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

export const DECIDE_PATH = "/api/v1/governance/approvals/decide";
export const CREATE_PATH = "/api/v1/governance/approvals";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const TRANSITIONS_PATH = "/api/v1/governance/lifecycle/transitions";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const APPROVAL_SUMMARY = "ApprovalSummary";
export const LIFECYCLE_PREFLIGHT = "LifecyclePreflight";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const DECISION_RESPONSE = "DecisionResponse";
export const BOUND = 1;
export const OK_CODE = "201";

export const STORE_RS_REL = "backend/crates/governance/application/src/lib.rs";
export const STORE_STRUCT = "ApprovalSummary";

/** Existing Serialize fields on ApprovalSummary. Do not invent names. */
export const SUMMARY_FIELDS = Object.freeze([
  "id",
  "request_ref",
  "kind",
  "requested_by",
  "approver_id",
  "decision",
  "decided_at",
]);

/** Values record_decision accepts and this 201 emits. Do not add pending. */
export const DECISION_VALUES = Object.freeze(["approved", "rejected"]);

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

const UUID_FIELDS = Object.freeze([
  "id",
  "request_ref",
  "requested_by",
  "approver_id",
]);

const FORBIDDEN_201 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
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
  if (objectName === APPROVAL_SUMMARY || itemName === APPROVAL_SUMMARY) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${APPROVAL_SUMMARY}`,
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
export function evaluateOpenapiApprovalSummary({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, APPROVAL_SUMMARY, SUMMARY_FIELDS);

  const envelope = own(schemas, APPROVAL_SUMMARY);
  if (isPlainObject(envelope)) {
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== SUMMARY_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${APPROVAL_SUMMARY}/required`,
        `${APPROVAL_SUMMARY} required must match always-serialized handler fields `
          + `(${SUMMARY_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    for (const field of UUID_FIELDS) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${APPROVAL_SUMMARY}/properties/${field}`,
          `${field} must $ref existing Uuid (UserId is a transparent UUID newtype)`,
        );
      }
    }
    const kind = own(properties, "kind");
    if (!isPlainObject(kind) || own(kind, "type") !== "string" || enumValues(kind)) {
      push(
        findings,
        `#/components/schemas/${APPROVAL_SUMMARY}/properties/kind`,
        "kind is unconstrained TEXT on the wire; do not invent a kind catalog",
      );
    }
    const decision = own(properties, "decision");
    const listed = enumValues(decision);
    if (
      !isPlainObject(decision)
      || own(decision, "type") !== "string"
      || !listed
      || listed.join("\0") !== DECISION_VALUES.join("\0")
    ) {
      push(
        findings,
        `#/components/schemas/${APPROVAL_SUMMARY}/properties/decision`,
        "decision must be the existing approved/rejected enum this 201 emits "
          + "(record_decision refuses pending); do not invent a GateKind catalog",
      );
    }
    if (schemaRefName(own(properties, "decided_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${APPROVAL_SUMMARY}/properties/decided_at`,
        "decided_at must $ref existing Timestamp (rfc3339 OffsetDateTime)",
      );
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, STORE_STRUCT, SUMMARY_FIELDS);

  const location = `#/paths/${DECIDE_PATH}/post`;
  const operation = findOperation(paths, DECIDE_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${DECIDE_PATH} must remain published (runtime already serves it)`,
    );
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${DECIDE_PATH}`,
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/${OK_CODE}/headers/ETag`,
      "decide_approval does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_201.includes(boundName)) {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${DECIDE_PATH} already returns ApprovalSummary `
        + `(id / request_ref / kind / requested_by / approver_id / decision / decided_at); `
        + `do not bind it to ${boundName} `
        + "(Head HOLD / LifecyclePreflight is a different type / policy envelope)",
    );
  } else if (boundName === APPROVAL_SUMMARY) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${DECIDE_PATH} already returns ApprovalSummary; `
        + `201 must $ref ${APPROVAL_SUMMARY}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${CREATE_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, CREATE_PATH, "post")),
    `POST ${CREATE_PATH} already returns ApprovalRequestSummary (payload_summary: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OVERRIDES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, OVERRIDES_PATH, "post")),
    `POST ${OVERRIDES_PATH} already returns OverrideSummary (before_snapshot: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${TRANSITIONS_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, TRANSITIONS_PATH, "post")),
    `POST ${TRANSITIONS_PATH} already returns LifecycleTransitionConfig, not ApprovalSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post"), "200"),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not ApprovalSummary`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not ApprovalSummary`,
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
    result = evaluateOpenapiApprovalSummary({ repoRoot });
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
      `openapi approval-summary typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi approval-summary typed-response gate passed `
      + `(${APPROVAL_SUMMARY} $ref; ${writes} write operations, 0 findings)`,
  );
}
