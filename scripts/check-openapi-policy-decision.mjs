// POST /api/v1/policy/authorize and /simulate 200 DecisionResponse bind gate.
//
// The hole this closes: authorize and simulate already return
// Json<DecisionResponse> whose only field is SimulationOutcome (no
// serde_json::Value). Composed OpenAPI still advertises
// additionalProperties: true, so clients cannot see the existing envelope.
// Same class as #1025 CatalogEntry / #1027 PreflightOutcome — publish the
// existing outcome, do not invent a store or policy catalog.
//
// Chesterton: face YAML from the existing handler struct, then $ref. Reuse
// the already-published SimulationOutcome schema (BulkDecisionResponse already
// does). Both handlers wrap the outcome — do not bind either 200 as a bare
// SimulationOutcome. Do not bind governance lifecycle preflight or
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
import { EXECUTE_PATH } from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = PREFLIGHT_WRITE_FLOOR;

export const AUTHORIZE_PATH = "/api/v1/policy/authorize";
export const SIMULATE_PATH = "/api/v1/policy/simulate";
export const BULK_PATH = "/api/v1/policy/authorize/bulk";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const TENANT_CONTEXT_PATH = "/api/v1/group-admin/tenant-context";

export const DECISION_RESPONSE = "DecisionResponse";
export const SIMULATION_OUTCOME = "SimulationOutcome";
export const BULK_DECISION_RESPONSE = "BulkDecisionResponse";
export const PREFLIGHT_OUTCOME = "PreflightOutcome";
export const BOUND = 2;

export const DECISION_RS_REL = "backend/crates/platform/authz-rest/src/lib.rs";
export const DECISION_STRUCT = "DecisionResponse";
export const OUTCOME_RS_REL = "backend/crates/platform/authz/src/cedar_pbac/authoring.rs";
export const OUTCOME_STRUCT = "SimulationOutcome";

/** Existing Serialize fields on DecisionResponse. Do not invent names. */
export const DECISION_FIELDS = Object.freeze(["outcome"]);

/** Existing Serialize fields on SimulationOutcome. Do not invent names. */
export const OUTCOME_FIELDS = Object.freeze([
  "effect",
  "determining_policies",
  "errors",
  "reason",
]);

/** SimEffect rename_all = snake_case; already on SimulationOutcome.yaml. */
export const EFFECT_ENUM = Object.freeze(["allow", "deny"]);

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
  SIMULATION_OUTCOME,
  BULK_DECISION_RESPONSE,
  PREFLIGHT_OUTCOME,
]);

const TARGET_PATHS = Object.freeze([AUTHORIZE_PATH, SIMULATE_PATH]);

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
  if (objectName === DECISION_RESPONSE || itemName === DECISION_RESPONSE) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${DECISION_RESPONSE}`,
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
      `cannot read existing ${structName} fields; this slice binds the handler type`,
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
export function evaluateOpenapiPolicyDecision({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, DECISION_RESPONSE, DECISION_FIELDS);
  requireSchemaFields(schemas, findings, SIMULATION_OUTCOME, OUTCOME_FIELDS);

  const envelope = own(schemas, DECISION_RESPONSE);
  if (isPlainObject(envelope)) {
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== DECISION_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${DECISION_RESPONSE}/required`,
        `${DECISION_RESPONSE} required must match always-serialized handler fields `
          + `(${DECISION_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    if (schemaRefName(own(properties, "outcome")) !== SIMULATION_OUTCOME) {
      push(
        findings,
        `#/components/schemas/${DECISION_RESPONSE}/properties/outcome`,
        `outcome must $ref existing ${SIMULATION_OUTCOME} (BulkDecisionResponse already does)`,
      );
    }
  }

  const outcome = own(schemas, SIMULATION_OUTCOME);
  if (isPlainObject(outcome)) {
    const required = own(outcome, "required");
    if (!Array.isArray(required) || required.join("\0") !== OUTCOME_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${SIMULATION_OUTCOME}/required`,
        `${SIMULATION_OUTCOME} required must match always-serialized handler fields `
          + `(${OUTCOME_FIELDS.join(", ")})`,
      );
    }
    const properties = own(outcome, "properties");
    const effect = own(properties, "effect");
    const effectEnum = isPlainObject(effect) ? own(effect, "enum") : null;
    if (
      !isPlainObject(effect)
      || own(effect, "type") !== "string"
      || !Array.isArray(effectEnum)
      || effectEnum.join("\0") !== EFFECT_ENUM.join("\0")
    ) {
      push(
        findings,
        `#/components/schemas/${SIMULATION_OUTCOME}/properties/effect`,
        `effect is existing SimEffect snake_case (${EFFECT_ENUM.join(", ")}); `
          + "do not invent a catalog",
      );
    }
  }

  requireRustFields(findings, repoRoot, DECISION_RS_REL, DECISION_STRUCT, DECISION_FIELDS);
  requireRustFields(findings, repoRoot, OUTCOME_RS_REL, OUTCOME_STRUCT, OUTCOME_FIELDS);

  for (const path of TARGET_PATHS) {
    const location = `#/paths/${path}/post`;
    const operation = findOperation(paths, path, "post");
    if (!isPlainObject(operation)) {
      push(findings, location, `POST ${path} must remain published (runtime already serves it)`);
      continue;
    }

    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto POST ${path}`,
      );
    }

    const headers = jsonOkHeaders(operation);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${location}/responses/200/headers/ETag`,
        "authorize/simulate do not send ETag; HTTP ETag stays HOLD — do not stamp it here",
      );
    }

    const body = jsonOkSchema(operation);
    const boundName = schemaRefName(body);
    if (FORBIDDEN_200.includes(boundName)) {
      push(
        findings,
        `${location}/responses/200`,
        `POST ${path} already returns Json<${DECISION_STRUCT}>; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / bare SimulationOutcome drops the outcome wrapper / execute Value)",
      );
    } else if (boundName === DECISION_RESPONSE) {
      bound += 1;
    } else {
      push(
        findings,
        `${location}/responses/200`,
        `POST ${path} already returns Json<${DECISION_STRUCT}>; `
          + `200 must $ref ${DECISION_RESPONSE}, not additionalProperties`,
      );
    }
  }

  refuseForeignBind(
    findings,
    `#/paths/${BULK_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, BULK_PATH, "post")),
    `POST ${BULK_PATH} already returns BulkDecisionResponse (decisions: Vec<SimulationOutcome>)`,
  );
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
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns LifecyclePreflight, not DecisionResponse`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${TENANT_CONTEXT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, TENANT_CONTEXT_PATH, "post")),
    `POST ${TENANT_CONTEXT_PATH} already returns GroupAdminTenantContextStartResponse, not DecisionResponse`,
  );

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiPolicyDecision({ repoRoot });
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
      `openapi policy-decision typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi policy-decision typed-response gate passed `
      + `(${DECISION_RESPONSE} $ref; ${writes} write operations, 0 findings)`,
  );
}
