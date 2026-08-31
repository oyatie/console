// POST /api/v1/governance/lifecycle/preflight 200 LifecyclePreflight bind gate.
//
// The hole this closes: lifecycle_preflight already returns Json whose fields
// are configured / GateChainConfig / GateChainOutcome (no serde_json::Value).
// The store type is LifecyclePreflight; the handler DTO is PreflightResponse
// with the same three fields. Composed OpenAPI still advertises
// additionalProperties: true, so clients cannot see the existing wire. Same
// class as #1027 PreflightOutcome / #1028 DecisionResponse — publish the
// existing outcome, do not invent a store or GateKind catalog.
//
// Chesterton: face YAML from the existing struct, then $ref. Reuse already
// published GateChainConfig / GateChainOutcome (LifecycleOutcome /
// PreflightOutcome). Must NOT $ref ontology PreflightOutcome — different type
// (dispatch / would_execute / gates vs configured / outcome). Do not bind
// group-admin tenant-context. Do not bind policy-draft Value bags. Do not map
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

export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const TENANT_CONTEXT_PATH = "/api/v1/group-admin/tenant-context";
export const AUTHORIZE_PATH = "/api/v1/policy/authorize";
export const SIMULATE_PATH = "/api/v1/policy/simulate";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const LIFECYCLE_PREFLIGHT = "LifecyclePreflight";
export const GATE_CHAIN_CONFIG = "GateChainConfig";
export const GATE_CHAIN_OUTCOME = "GateChainOutcome";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const DECISION_RESPONSE = "DecisionResponse";
export const BOUND = 1;

export const STORE_RS_REL = "backend/crates/governance/adapter-postgres/src/lib.rs";
export const STORE_STRUCT = "LifecyclePreflight";
export const HANDLER_RS_REL = "backend/crates/governance/rest/src/lib.rs";
export const HANDLER_STRUCT = "PreflightResponse";

/** Existing fields on LifecyclePreflight / PreflightResponse. Do not invent names. */
export const PREFLIGHT_FIELDS = Object.freeze([
  "configured",
  "config",
  "outcome",
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
  if (objectName === LIFECYCLE_PREFLIGHT || itemName === LIFECYCLE_PREFLIGHT) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${LIFECYCLE_PREFLIGHT}`,
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
export function evaluateOpenapiLifecyclePreflight({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, LIFECYCLE_PREFLIGHT, PREFLIGHT_FIELDS);

  const envelope = own(schemas, LIFECYCLE_PREFLIGHT);
  if (isPlainObject(envelope)) {
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== PREFLIGHT_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_PREFLIGHT}/required`,
        `${LIFECYCLE_PREFLIGHT} required must match always-serialized handler fields `
          + `(${PREFLIGHT_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    if (schemaRefName(own(properties, "config")) !== GATE_CHAIN_CONFIG) {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_PREFLIGHT}/properties/config`,
        `config must $ref existing ${GATE_CHAIN_CONFIG} (LifecycleOutcome already does)`,
      );
    }
    if (schemaRefName(own(properties, "outcome")) !== GATE_CHAIN_OUTCOME) {
      push(
        findings,
        `#/components/schemas/${LIFECYCLE_PREFLIGHT}/properties/outcome`,
        `outcome must $ref existing ${GATE_CHAIN_OUTCOME}; do not invent GateKind / GateOutcome`,
      );
    }
  }

  requireRustFields(findings, repoRoot, STORE_RS_REL, STORE_STRUCT, PREFLIGHT_FIELDS);
  requireRustFields(findings, repoRoot, HANDLER_RS_REL, HANDLER_STRUCT, PREFLIGHT_FIELDS);

  const location = `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post`;
  const operation = findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${LIFECYCLE_PREFLIGHT_PATH} must remain published (runtime already serves it)`,
    );
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${LIFECYCLE_PREFLIGHT_PATH}`,
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "lifecycle_preflight does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_200.includes(boundName)) {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight `
        + `(configured / config / outcome); do not bind it to ${boundName} `
        + "(Head HOLD / ontology PreflightOutcome is a different type / policy envelope)",
    );
  } else if (boundName === LIFECYCLE_PREFLIGHT) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight; `
        + `200 must $ref ${LIFECYCLE_PREFLIGHT}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post")),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not LifecyclePreflight`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post")),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${AUTHORIZE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, AUTHORIZE_PATH, "post")),
    `POST ${AUTHORIZE_PATH} already returns DecisionResponse, not LifecyclePreflight`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${SIMULATE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, SIMULATE_PATH, "post")),
    `POST ${SIMULATE_PATH} already returns DecisionResponse, not LifecyclePreflight`,
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
    `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, TENANT_CONTEXT_PATH, "post")),
    `POST ${TENANT_CONTEXT_PATH} already returns GroupAdminTenantContextStartResponse, not LifecyclePreflight`,
  );

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiLifecyclePreflight({ repoRoot });
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
      `openapi lifecycle-preflight typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi lifecycle-preflight typed-response gate passed `
      + `(${LIFECYCLE_PREFLIGHT} $ref; ${writes} write operations, 0 findings)`,
  );
}
