// GET /api/audit published-query gate.
//
// The hole this closes: AuditQuery already deserializes target_id and
// trace_id; fetch_audit_records already AND-filters those columns;
// audit_api.rs already asserts the isolation. Composed OpenAPI still
// advertises only limit/offset/target_type/actor, so clients cannot
// discover the filters the handler already honors.
//
// Chesterton: publish the two existing optional string queries. Do not
// invent an action enum / AsyncAPI catalog from audit_events.action TEXT.
// Do not add an action query the handler does not read. Do not type the
// 200 items (no AuditRecord schema). Do not map Feature::ALL permissions.
//
// Totality: js-yaml load + own-property walk of every GET. GET_FLOOR
// locks examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const AUDIT_GET_PATH = "/api/audit";
export const TARGET_ID = "target_id";
export const TRACE_ID = "trace_id";
export const ACTION = "action";
export const BOUND_FILTERS = 2;

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

const KEPT_EXISTING = Object.freeze(["limit", "offset", "target_type", "actor"]);

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

function isCatalogSchema(schema) {
  if (!isPlainObject(schema)) return false;
  if (Array.isArray(own(schema, "enum")) && own(schema, "enum").length > 0) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (Array.isArray(members) && members.length > 0) return true;
  }
  return false;
}

export function isOptionalStringQueryParam(param, name) {
  if (!isPlainObject(param)) return false;
  if (own(param, "name") !== name) return false;
  if (own(param, "in") !== "query") return false;
  if (own(param, "required") === true) return false;
  const schema = own(param, "schema");
  if (!isPlainObject(schema)) return false;
  if (isCatalogSchema(schema)) return false;
  return own(schema, "type") === "string";
}

function namedQuery(parameters, name) {
  return parameters.find((param) => isPlainObject(param) && own(param, "name") === name);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiAuditQueryParams({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
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

  const location = `#/paths/${AUDIT_GET_PATH}/get`;
  const operation = findGet(paths, AUDIT_GET_PATH);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${AUDIT_GET_PATH} must remain published (runtime already serves it)`);
    return { gets, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto GET /api/audit (AuditLogRead stays the runtime gate, unpublished here)",
    );
  }

  const parameters = parameterList(operation);

  for (const name of KEPT_EXISTING) {
    if (!namedQuery(parameters, name)) {
      push(
        findings,
        `${location}/parameters/${name}`,
        `GET ${AUDIT_GET_PATH} already publishes ${name}; do not drop it while adding the unpublished filters`,
      );
    }
  }

  for (const name of [TARGET_ID, TRACE_ID]) {
    if (isOptionalStringQueryParam(namedQuery(parameters, name), name)) {
      bound += 1;
    } else {
      push(
        findings,
        `${location}/parameters/${name}`,
        `GET ${AUDIT_GET_PATH} already filters ${name} in AuditQuery / fetch_audit_records; `
          + `publish it as an optional string query (not enum, not a catalog)`,
      );
    }
  }

  const action = namedQuery(parameters, ACTION);
  if (action) {
    const schema = own(action, "schema");
    if (isCatalogSchema(schema) || !isOptionalStringQueryParam(action, ACTION)) {
      push(
        findings,
        `${location}/parameters/${ACTION}`,
        "audit_events.action is unconstrained TEXT; do not publish an action enum / AsyncAPI catalog. "
          + "If a query is added it must stay type string with no enum",
      );
    }
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiAuditQueryParams({ repoRoot });
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
  if (findings.length > 0 || belowGetFloor || bound !== BOUND_FILTERS) {
    console.error(
      `openapi audit query-params gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND_FILTERS}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi audit query-params gate passed `
      + `(${TARGET_ID} and ${TRACE_ID} optional string queries; `
      + `${gets} GET operations, 0 findings)`,
  );
}
