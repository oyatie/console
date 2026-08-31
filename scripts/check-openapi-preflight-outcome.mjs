// POST /api/v1/ontology/actions/{action_key}/preflight 200 PreflightOutcome bind gate.
//
// The hole this closes: action_preflight already returns Json<PreflightOutcome>
// whose fields are ActionDispatch / Option<String> / GateChainConfig /
// GateChainOutcome / bool (no serde_json::Value). Composed OpenAPI still
// advertises additionalProperties: true, so clients cannot see the existing
// wire. Same class as #997 typed execute params / #1026 GroupAdminGroupsResponse
// — publish the existing outcome, do not invent a store or GateKind catalog.
//
// Chesterton: face YAML from the existing struct, then $ref. Reuse the already
// published GateChainConfig / GateChainOutcome schemas (LifecycleOutcome). Leave
// GateChainOutcome items as the existing open bag — do not invent GateOutcome /
// GateKind / GateStatus schemas here. Do not bind execute 200 (ExecuteOutcome
// carries projected: Value). Do not bind policy-draft / ObjectTypeDetail /
// absence-exit Value bags. Do not map Feature::ALL permissions. Do not stamp
// HTTP ETag.
//
// Totality: js-yaml load + own-property walk of every write method + optional
// Rust struct field read. WRITE_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = 300;

export { EXECUTE_PATH, PREFLIGHT_PATH };

export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";

export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const GATE_CHAIN_CONFIG = "GateChainConfig";
export const GATE_CHAIN_OUTCOME = "GateChainOutcome";
export const EXECUTE_OUTCOME = "OntologyActionExecuteOutcome";
export const BOUND = 1;

export const PREFLIGHT_RS_REL = "backend/crates/ontology/rest/src/lib.rs";
export const PREFLIGHT_STRUCT = "PreflightOutcome";

/** Existing Serialize fields on PreflightOutcome. Do not invent names. */
export const PREFLIGHT_FIELDS = Object.freeze([
  "dispatch",
  "dispatch_target",
  "config",
  "gates",
  "criteria_ok",
  "criteria_error",
  "would_execute",
]);

/** Always serialized (dispatch_target is Option without skip_serializing_if). */
export const PREFLIGHT_REQUIRED = Object.freeze([
  "dispatch",
  "dispatch_target",
  "config",
  "gates",
  "criteria_ok",
  "would_execute",
]);

/** ActionDispatch rename_all = snake_case; already on OntologyActionExecuteOutcome. */
export const DISPATCH_ENUM = Object.freeze([
  "instance_revision",
  "projected_usecase",
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

const FORBIDDEN_200 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  EXECUTE_OUTCOME,
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

function jsonOkSchema(operation, code = "200") {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function jsonOkHeaders(operation, code = "200") {
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
  if (objectName === PREFLIGHT_OUTCOME || itemName === PREFLIGHT_OUTCOME) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${PREFLIGHT_OUTCOME}`,
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
export function evaluateOpenapiPreflightOutcome({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, PREFLIGHT_OUTCOME, PREFLIGHT_FIELDS);

  const outcome = own(schemas, PREFLIGHT_OUTCOME);
  if (isPlainObject(outcome)) {
    const required = own(outcome, "required");
    if (!Array.isArray(required) || required.join("\0") !== PREFLIGHT_REQUIRED.join("\0")) {
      push(
        findings,
        `#/components/schemas/${PREFLIGHT_OUTCOME}/required`,
        `${PREFLIGHT_OUTCOME} required must match always-serialized handler fields `
          + `(${PREFLIGHT_REQUIRED.join(", ")}); criteria_error is skip_serializing_if`,
      );
    }

    const properties = own(outcome, "properties");
    const dispatch = own(properties, "dispatch");
    const dispatchEnum = isPlainObject(dispatch) ? own(dispatch, "enum") : null;
    if (
      !isPlainObject(dispatch)
      || own(dispatch, "type") !== "string"
      || !Array.isArray(dispatchEnum)
      || dispatchEnum.join("\0") !== DISPATCH_ENUM.join("\0")
    ) {
      push(
        findings,
        `#/components/schemas/${PREFLIGHT_OUTCOME}/properties/dispatch`,
        `dispatch is existing ActionDispatch snake_case (${DISPATCH_ENUM.join(", ")}); `
          + "do not invent a catalog",
      );
    }

    const target = own(properties, "dispatch_target");
    const targetType = isPlainObject(target) ? own(target, "type") : null;
    const admitsNull = Array.isArray(targetType) && targetType.includes("null")
      && targetType.includes("string");
    if (!admitsNull) {
      push(
        findings,
        `#/components/schemas/${PREFLIGHT_OUTCOME}/properties/dispatch_target`,
        "dispatch_target is Option<String> without skip_serializing_if; "
          + "3.1 wire is type: [string, 'null']",
      );
    }

    if (schemaRefName(own(properties, "config")) !== GATE_CHAIN_CONFIG) {
      push(
        findings,
        `#/components/schemas/${PREFLIGHT_OUTCOME}/properties/config`,
        `config must $ref existing ${GATE_CHAIN_CONFIG} (LifecycleOutcome already does)`,
      );
    }
    if (schemaRefName(own(properties, "gates")) !== GATE_CHAIN_OUTCOME) {
      push(
        findings,
        `#/components/schemas/${PREFLIGHT_OUTCOME}/properties/gates`,
        `gates must $ref existing ${GATE_CHAIN_OUTCOME}; do not invent GateKind / GateOutcome`,
      );
    }
  }

  const rustPath = join(repoRoot, PREFLIGHT_RS_REL);
  if (existsSync(rustPath)) {
    const source = readFileSync(rustPath, "utf8");
    const fields = rustStructFields(source, PREFLIGHT_STRUCT);
    if (!Array.isArray(fields) || fields.length === 0) {
      push(
        findings,
        `${PREFLIGHT_RS_REL}:${PREFLIGHT_STRUCT}`,
        `cannot read existing ${PREFLIGHT_STRUCT} fields; this slice binds the handler type`,
      );
    } else if (fields.join("\0") !== PREFLIGHT_FIELDS.join("\0")) {
      push(
        findings,
        `${PREFLIGHT_RS_REL}:${PREFLIGHT_STRUCT}`,
        "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
      );
    }
  }

  const location = `#/paths/${PREFLIGHT_PATH}/post`;
  const operation = findOperation(paths, PREFLIGHT_PATH, "post");
  if (!isPlainObject(operation)) {
    push(findings, location, `POST ${PREFLIGHT_PATH} must remain published (runtime already serves it)`);
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${PREFLIGHT_PATH}`,
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "action_preflight does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_200.includes(boundName)) {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${PREFLIGHT_PATH} already returns Json<${PREFLIGHT_STRUCT}>; `
        + `do not bind it to ${boundName} (Head HOLD / execute projected Value)`,
    );
  } else if (boundName === PREFLIGHT_OUTCOME) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${PREFLIGHT_PATH} already returns Json<${PREFLIGHT_STRUCT}>; `
        + `200 must $ref ${PREFLIGHT_OUTCOME}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post")),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, DRAFTS_GET_PATH, "get")),
    `GET ${DRAFTS_GET_PATH} already returns DraftRecord (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_GET_PATH, "get")),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, ABSENCE_EXIT_GET_PATH, "get")),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns nested serde_json::Value bags`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post")),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not action PreflightOutcome`,
  );

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiPreflightOutcome({ repoRoot });
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
      `openapi preflight-outcome typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi preflight-outcome typed-response gate passed `
      + `(${PREFLIGHT_OUTCOME} $ref; ${writes} write operations, 0 findings)`,
  );
}
