// POST /api/v1/governance/lifecycle/transitions 201 LifecycleTransitionConfig
// bind gate.
//
// The hole this closes: configure_transition already returns
// Json<LifecycleTransitionConfig> whose fields are Uuid / LifecycleState /
// TransitionRequirements (three bools, no serde_json::Value). Composed OpenAPI
// still advertises additionalProperties: true, so clients cannot see the
// existing wire. Same class as #1029 LifecyclePreflight / #1031
// ApprovalSummary — publish the existing config, do not invent a store,
// GateKind catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing struct, then $ref. Nested
// requirements $ref the existing TransitionRequirements bool bag (do not
// flatten onto the request-shaped wire). from_state / to_state $ref existing
// LifecycleState. Must NOT $ref LifecyclePreflight or ontology
// PreflightOutcome — different types (gate envelope / would_execute vs
// object_type_id / requirements). Do not bind overrides (Value snapshot).
// Do not map Feature::ALL permissions. Do not stamp HTTP ETag.
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

export const TRANSITIONS_PATH = "/api/v1/governance/lifecycle/transitions";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const DECIDE_PATH = "/api/v1/governance/approvals/decide";
export const CREATE_PATH = "/api/v1/governance/approvals";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const LIFECYCLE_TRANSITION_CONFIG = "LifecycleTransitionConfig";
export const TRANSITION_REQUIREMENTS = "TransitionRequirements";
export const LIFECYCLE_STATE = "LifecycleState";
export const LIFECYCLE_PREFLIGHT = "LifecyclePreflight";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const DECISION_RESPONSE = "DecisionResponse";
export const APPROVAL_SUMMARY = "ApprovalSummary";
export const CONFIGURE_REQUEST = "GovernanceConfigureTransitionRequest";
export const BOUND = 1;
export const OK_CODE = "201";

export const STORE_RS_REL = "backend/crates/governance/application/src/lib.rs";
export const STORE_STRUCT = "LifecycleTransitionConfig";
export const REQ_RS_REL = "backend/crates/governance/domain/src/lib.rs";
export const REQ_STRUCT = "TransitionRequirements";

/** Existing Serialize fields on LifecycleTransitionConfig. Do not invent names. */
export const CONFIG_FIELDS = Object.freeze([
  "object_type_id",
  "from_state",
  "to_state",
  "requirements",
]);

/** Existing Serialize fields on TransitionRequirements. Do not flatten. */
export const REQUIREMENT_FIELDS = Object.freeze([
  "requires_reason",
  "requires_four_eyes",
  "requires_checklist",
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

const FORBIDDEN_201 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  LIFECYCLE_PREFLIGHT,
  PREFLIGHT_OUTCOME,
  DECISION_RESPONSE,
  APPROVAL_SUMMARY,
  CONFIGURE_REQUEST,
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
  if (
    objectName === LIFECYCLE_TRANSITION_CONFIG
    || itemName === LIFECYCLE_TRANSITION_CONFIG
  ) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${LIFECYCLE_TRANSITION_CONFIG}`,
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
export function evaluateOpenapiLifecycleTransitionConfig({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, LIFECYCLE_TRANSITION_CONFIG, CONFIG_FIELDS);
  requireSchemaFields(schemas, findings, TRANSITION_REQUIREMENTS, REQUIREMENT_FIELDS);

  const envelope = own(schemas, LIFECYCLE_TRANSITION_CONFIG);
  if (isPlainObject(envelope)) {
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== CONFIG_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/required`,
        `${LIFECYCLE_TRANSITION_CONFIG} required must match always-serialized handler fields `
          + `(${CONFIG_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    if (schemaRefName(own(properties, "object_type_id")) !== "Uuid") {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties/object_type_id`,
        "object_type_id must $ref existing Uuid",
      );
    }
    for (const field of ["from_state", "to_state"]) {
      if (schemaRefName(own(properties, field)) !== LIFECYCLE_STATE) {
        push(
          findings,
          `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties/${field}`,
          `${field} must $ref existing ${LIFECYCLE_STATE}; do not invent a state catalog`,
        );
      }
    }
    if (schemaRefName(own(properties, "requirements")) !== TRANSITION_REQUIREMENTS) {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_TRANSITION_CONFIG}/properties/requirements`,
        `requirements must $ref existing ${TRANSITION_REQUIREMENTS} `
          + "(nested bool bag); do not flatten onto the request-shaped wire "
          + "and do not leave additionalProperties",
      );
    }
  }

  const requirements = own(schemas, TRANSITION_REQUIREMENTS);
  if (isPlainObject(requirements)) {
    const required = own(requirements, "required");
    if (
      !Array.isArray(required)
      || required.join("\0") !== REQUIREMENT_FIELDS.join("\0")
    ) {
      push(
        findings,
        `#/components/schemas/${TRANSITION_REQUIREMENTS}/required`,
        `${TRANSITION_REQUIREMENTS} required must match always-serialized handler fields `
          + `(${REQUIREMENT_FIELDS.join(", ")})`,
      );
    }
    const properties = own(requirements, "properties");
    for (const field of REQUIREMENT_FIELDS) {
      const listed = own(properties, field);
      if (!isPlainObject(listed) || own(listed, "type") !== "boolean") {
        push(
          findings,
          `#/components/schemas/${TRANSITION_REQUIREMENTS}/properties/${field}`,
          `${field} is a bool on the existing struct; do not invent a GateKind catalog`,
        );
      }
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, STORE_STRUCT, CONFIG_FIELDS);
  requireRustFields(findings, repoRoot, REQ_RS_REL, REQ_STRUCT, REQUIREMENT_FIELDS);

  const location = `#/paths/${TRANSITIONS_PATH}/post`;
  const operation = findOperation(paths, TRANSITIONS_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${TRANSITIONS_PATH} must remain published (runtime already serves it)`,
    );
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${TRANSITIONS_PATH}`,
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/${OK_CODE}/headers/ETag`,
      "configure_transition does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_201.includes(boundName)) {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${TRANSITIONS_PATH} already returns LifecycleTransitionConfig `
        + `(object_type_id / from_state / to_state / requirements); `
        + `do not bind it to ${boundName} `
        + "(Head HOLD / LifecyclePreflight is a different type / "
        + "ontology PreflightOutcome is a different type / "
        + "request schema is flat / ApprovalSummary is a four-eyes decision)",
    );
  } else if (boundName === LIFECYCLE_TRANSITION_CONFIG) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/${OK_CODE}`,
      `POST ${TRANSITIONS_PATH} already returns LifecycleTransitionConfig; `
        + `201 must $ref ${LIFECYCLE_TRANSITION_CONFIG}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${DECIDE_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DECIDE_PATH, "post")),
    `POST ${DECIDE_PATH} already returns ApprovalSummary, not LifecycleTransitionConfig`,
  );
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
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post"), "200"),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not LifecycleTransitionConfig`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not LifecycleTransitionConfig`,
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
    result = evaluateOpenapiLifecycleTransitionConfig({ repoRoot });
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
      `openapi lifecycle-transition-config typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi lifecycle-transition-config typed-response gate passed `
      + `(${LIFECYCLE_TRANSITION_CONFIG} $ref; ${writes} write operations, 0 findings)`,
  );
}
