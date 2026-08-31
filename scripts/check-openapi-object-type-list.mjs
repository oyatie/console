// GET /api/v1/ontology/object-types 200 ObjectTypeSummary bind gate.
//
// The hole this closes: list_object_types already returns Json<Vec<ObjectTypeSummary>>
// and ObjectTypeSummary already exists in composed OpenAPI (POST/PUT/lifecycle 201/200).
// The list path still advertises additionalProperties: true, so clients cannot see
// the existing summary fields. Same class as #1020 InstanceState list $ref — bind
// the runtime type. Do not invent ObjectTypeDetail (GET by key returns that nested
// shape; nested PropertyDefSummary/LinkTypeSummary/ActionTypeSummary/AnalyticSummary
// schemas do not exist). Do not invent query filters (handler has no Query). Do not
// stamp Feature::ALL permissions. Do not put HTTP ETag on the list (handler does
// not send it).
//
// Chesterton: $ref the existing schema. Do not reuse ObjectTypeResponse (registry
// kind + active_count, a different wire). Do not bind GET
// /api/v1/ontology/object-types/{key} to ObjectTypeSummary — that handler serializes
// ObjectTypeDetail.
//
// Totality: js-yaml load + own-property walk of every GET. GET_FLOOR locks
// examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const OBJECT_TYPES_LIST_PATH = "/api/v1/ontology/object-types";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const OBJECT_TYPE_SUMMARY = "ObjectTypeSummary";
export const OBJECT_TYPE_RESPONSE = "ObjectTypeResponse";
export const BOUND = 1;

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

function arrayItemName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
}

function requireSchema(schemas, findings, name) {
  if (!isPlainObject(own(schemas, name))) {
    push(
      findings,
      `#/components/schemas/${name}`,
      `${name} must already exist — this slice binds the runtime type, it does not invent a store`,
    );
  }
}

function requireGet(paths, findings, path) {
  const location = `#/paths/${path}/get`;
  const operation = findGet(paths, path);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${path} must remain published (runtime already serves it)`);
    return null;
  }
  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto ontology object-type GETs (RoleManage stays the REST gate, unpublished here)",
    );
  }
  return { location, operation };
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiObjectTypeList({ repoRoot }) {
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

  requireSchema(schemas, findings, OBJECT_TYPE_SUMMARY);

  const list = requireGet(paths, findings, OBJECT_TYPES_LIST_PATH);
  if (list) {
    const name = arrayItemName(json200Schema(list.operation));
    if (name !== OBJECT_TYPE_SUMMARY) {
      push(
        findings,
        `${list.location}/responses/200`,
        `GET ${OBJECT_TYPES_LIST_PATH} already returns Json<Vec<ObjectTypeSummary>>; `
          + `200 must be an array of ${OBJECT_TYPE_SUMMARY}, not additionalProperties `
          + `(do not reuse ${OBJECT_TYPE_RESPONSE} — that is the registry kind + active_count wire)`,
      );
    } else {
      bound += 1;
    }
    if (parameterList(list.operation).length > 0) {
      push(
        findings,
        `${list.location}/parameters`,
        "list_object_types has no Query; do not invent as_of, pagination, or filter params",
      );
    }
    const headers = json200Headers(list.operation);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${list.location}/responses/200/headers/ETag`,
        "list_object_types does not send ETag; HTTP ETag stays HOLD — do not stamp it on this GET",
      );
    }
  }

  const detail = requireGet(paths, findings, OBJECT_TYPE_GET_PATH);
  if (detail) {
    const schema = json200Schema(detail.operation);
    const objectName = schemaRefName(schema);
    const itemName = arrayItemName(schema);
    if (objectName === OBJECT_TYPE_SUMMARY || itemName === OBJECT_TYPE_SUMMARY) {
      push(
        findings,
        `${detail.location}/responses/200`,
        `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested properties/links/actions/analytics); `
          + `do not bind it to ${OBJECT_TYPE_SUMMARY} — that would invent a false wire`,
      );
    }
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiObjectTypeList({ repoRoot });
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
      `openapi object-type list typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi object-type list typed-response gate passed `
      + `(${OBJECT_TYPE_SUMMARY} items $ref; ${gets} GET operations, 0 findings)`,
  );
}
