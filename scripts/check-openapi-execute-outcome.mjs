// POST /api/v1/ontology/actions/{action_key}/execute 200
// OntologyActionExecuteOutcome bind gate.
//
// The hole this closes: action_execute already returns Json<ExecuteOutcome>
// (dispatch / GateChainOutcome / Option<InstanceState> / Option<Value> /
// Option<CommandReceipt>). Composed OpenAPI already $ref
// OntologyActionExecuteOutcome, but gates and instance stay unpublished
// additionalProperties bags. projected is serde_json::Value and must stay
// unconstrained on that field only. Must not $ref PreflightOutcome (preflight
// is a different wire: would_execute / config / criteria, no instance /
// receipt / projected).
//
// Chesterton: type every non-Value field from the existing struct. Reuse
// already-published GateChainOutcome / InstanceState /
// OntologyActionCommandReceipt. Leave projected unconstrained object /
// additionalProperties: true. Do not invent a projected catalog, GateKind,
// store, or ObjectKey. Do not bind execute 200 to PreflightOutcome.
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

export { EXECUTE_PATH, PREFLIGHT_PATH };

export const KILL_SWITCH_PATH = "/api/v1/console/kill-switch";
export const ROLLOUT_OPT_IN_PATH = "/api/v1/console/rollout/opt-in";
export const ROLLOUT_ORG_FLAG_PATH = "/api/v1/console/rollout/org-flag";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";

export const EXECUTE_OUTCOME = "OntologyActionExecuteOutcome";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const GATE_CHAIN_OUTCOME = "GateChainOutcome";
export const INSTANCE_STATE = "InstanceState";
export const COMMAND_RECEIPT = "OntologyActionCommandReceipt";
export const BOUND = 1;

export const EXECUTE_RS_REL = "backend/crates/ontology/rest/src/lib.rs";
export const EXECUTE_STRUCT = "ExecuteOutcome";
export const RECEIPT_STRUCT = "CommandReceipt";

/** Existing Serialize fields on ExecuteOutcome. Do not invent names. */
export const EXECUTE_FIELDS = Object.freeze([
  "dispatch",
  "gates",
  "instance",
  "projected",
  "receipt",
]);

/** Always serialized (instance / projected / receipt skip_serializing_if). */
export const EXECUTE_REQUIRED = Object.freeze(["dispatch", "gates"]);

export const RECEIPT_FIELDS = Object.freeze([
  "command_id",
  "payload_digest",
  "instance",
  "gates",
]);

export const RECEIPT_REQUIRED = RECEIPT_FIELDS;

/** ActionDispatch rename_all = snake_case; already on PreflightOutcome. */
export const DISPATCH_ENUM = Object.freeze([
  "instance_revision",
  "projected_usecase",
]);

/** serde_json::Value on the wire. Do not close into a catalog. */
export const VALUE_FIELD = "projected";

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
  PREFLIGHT_OUTCOME,
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

function enumValues(schema) {
  if (!isPlainObject(schema)) return null;
  const listed = own(schema, "enum");
  return Array.isArray(listed) ? listed : null;
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

function isUnpublishedBag(schema) {
  if (!isPlainObject(schema)) return true;
  if (schemaRefName(schema)) return false;
  return own(schema, "additionalProperties") === true
    || schemaPropertyNames(schema).length === 0;
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

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  if (objectName === EXECUTE_OUTCOME || itemName === EXECUTE_OUTCOME) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${EXECUTE_OUTCOME}`,
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
export function evaluateOpenapiExecuteOutcome({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, EXECUTE_OUTCOME, EXECUTE_FIELDS);
  requireRequiredList(schemas, findings, EXECUTE_OUTCOME, EXECUTE_REQUIRED);
  requireSchemaFields(schemas, findings, COMMAND_RECEIPT, RECEIPT_FIELDS);
  requireRequiredList(schemas, findings, COMMAND_RECEIPT, RECEIPT_REQUIRED);

  const outcome = own(schemas, EXECUTE_OUTCOME);
  if (isPlainObject(outcome)) {
    if (own(outcome, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${EXECUTE_OUTCOME}`,
        `${EXECUTE_OUTCOME} envelope must stay typed; additionalProperties: true belongs `
          + "on projected only, not the whole 200 bag",
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
        `#/components/schemas/${EXECUTE_OUTCOME}/properties/dispatch`,
        `dispatch is existing ActionDispatch snake_case (${DISPATCH_ENUM.join(", ")}); `
          + "do not invent a catalog",
      );
    }

    if (schemaRefName(own(properties, "gates")) !== GATE_CHAIN_OUTCOME) {
      push(
        findings,
        `#/components/schemas/${EXECUTE_OUTCOME}/properties/gates`,
        `gates is GateChainOutcome; $ref the existing schema — do not leave an unpublished bag `
          + "or invent GateKind / GateOutcome",
      );
    }

    if (schemaRefName(own(properties, "instance")) !== INSTANCE_STATE) {
      push(
        findings,
        `#/components/schemas/${EXECUTE_OUTCOME}/properties/instance`,
        `instance is Option<InstanceState> (skip_serializing_if); $ref existing ${INSTANCE_STATE} `
          + "— do not leave an unpublished bag",
      );
    }

    const projected = own(properties, VALUE_FIELD);
    if (isClosedCatalog(projected) || !isUnconstrainedJsonObject(projected)) {
      push(
        findings,
        `#/components/schemas/${EXECUTE_OUTCOME}/properties/${VALUE_FIELD}`,
        "projected is serde_json::Value; leave it unconstrained object / "
          + "additionalProperties: true on that field only — do not invent a projected catalog",
      );
    }

    if (schemaRefName(own(properties, "receipt")) !== COMMAND_RECEIPT) {
      push(
        findings,
        `#/components/schemas/${EXECUTE_OUTCOME}/properties/receipt`,
        `receipt is Option<CommandReceipt> (skip_serializing_if); $ref existing ${COMMAND_RECEIPT}`,
      );
    }
  }

  const receipt = own(schemas, COMMAND_RECEIPT);
  if (isPlainObject(receipt)) {
    if (own(receipt, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${COMMAND_RECEIPT}`,
        `${COMMAND_RECEIPT} envelope must stay typed; additionalProperties: true belongs on Value fields only`,
      );
    }
    const properties = own(receipt, "properties");
    if (schemaRefName(own(properties, "instance")) !== INSTANCE_STATE) {
      push(
        findings,
        `#/components/schemas/${COMMAND_RECEIPT}/properties/instance`,
        `instance is InstanceState; $ref existing ${INSTANCE_STATE} — do not leave an unpublished bag`,
      );
    }
    if (schemaRefName(own(properties, "gates")) !== GATE_CHAIN_OUTCOME) {
      push(
        findings,
        `#/components/schemas/${COMMAND_RECEIPT}/properties/gates`,
        `gates is GateChainOutcome; $ref existing ${GATE_CHAIN_OUTCOME}; do not invent GateKind`,
      );
    }
  }

  requireRustFields(findings, repoRoot, EXECUTE_RS_REL, EXECUTE_STRUCT, EXECUTE_FIELDS);
  requireRustFields(findings, repoRoot, EXECUTE_RS_REL, RECEIPT_STRUCT, RECEIPT_FIELDS);

  const location = `#/paths/${EXECUTE_PATH}/post`;
  const operation = findOperation(paths, EXECUTE_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${EXECUTE_PATH} must remain published (runtime already serves Json<${EXECUTE_STRUCT}>)`,
    );
  } else {
    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto POST ${EXECUTE_PATH}`,
      );
    }

    const headers = jsonOkHeaders(operation);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${location}/responses/200/headers/ETag`,
        "action_execute does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
      );
    }

    const body = jsonOkSchema(operation);
    const boundName = schemaRefName(body);
    const responseLoc = `${location}/responses/200`;
    if (FORBIDDEN_200.includes(boundName)) {
      push(
        findings,
        responseLoc,
        `POST ${EXECUTE_PATH} already returns Json<${EXECUTE_STRUCT}>; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / PreflightOutcome is the preflight wire, not execute)",
      );
    } else if (boundName === EXECUTE_OUTCOME) {
      bound += 1;
    } else if (isUnpublishedBag(body)) {
      push(
        findings,
        responseLoc,
        `POST ${EXECUTE_PATH} already returns Json<${EXECUTE_STRUCT}>; `
          + `200 must $ref ${EXECUTE_OUTCOME}, not a root additionalProperties bag`,
      );
    } else {
      push(
        findings,
        responseLoc,
        `POST ${EXECUTE_PATH} already returns Json<${EXECUTE_STRUCT}>; `
          + `200 must $ref ${EXECUTE_OUTCOME}, not additionalProperties`,
      );
    }
  }

  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post")),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not execute`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post")),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not execute`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${KILL_SWITCH_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, KILL_SWITCH_PATH, "post")),
    `POST ${KILL_SWITCH_PATH} is console kill-switch, not ontology execute`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_OPT_IN_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_OPT_IN_PATH, "put")),
    `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not ontology execute`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_ORG_FLAG_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_ORG_FLAG_PATH, "put")),
    `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not ontology execute`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_GET_PATH, "get")),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail, not execute`,
  );

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiExecuteOutcome({ repoRoot });
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
      `openapi execute-outcome typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi execute-outcome typed-response gate passed `
      + `(${EXECUTE_OUTCOME} $ref; ${VALUE_FIELD} unconstrained; `
      + `${writes} write operations, 0 findings)`,
  );
}
