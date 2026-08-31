// Ontology instance GET/list/history/traverse typed-response gate.
//
// The hole this closes: PgInstanceStore::{get_as_of, history, list_instances,
// traverse} and the REST handlers already return InstanceState /
// RevisionSummary / TraversalGraph. Those schemas already exist in composed
// OpenAPI. The path items still advertise additionalProperties: true, so the
// published contract is untyped where the runtime is not.
//
// Chesterton: bind the existing $refs. Do not invent an Employment history
// store, Head history GET, HTTP ETag, Feature::ALL permissions on these
// routes, or an action catalog. Instance GET keeps Timestamp as_of. PayRun
// stays PayrollRunSummary.
//
// Totality: js-yaml load + own-property walk of every GET. GET_FLOOR locks
// examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR, isTimestampAsOfParam } from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const INSTANCE_GET_PATH = "/api/v1/ontology/instances/{id}";
export const INSTANCE_LIST_PATH = "/api/v1/ontology/instances";
export const INSTANCE_HISTORY_PATH = "/api/v1/ontology/instances/{id}/history";
export const INSTANCE_TRAVERSE_PATH = "/api/v1/ontology/instances/{id}/traverse";

export const INSTANCE_STATE = "InstanceState";
export const REVISION_SUMMARY = "RevisionSummary";
export const TRAVERSAL_GRAPH = "TraversalGraph";

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
        + "do not map Feature::ALL onto ontology instance GETs (RoleManage stays the REST gate, unpublished here)",
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
export function evaluateOpenapiOntologyInstanceTypes({ repoRoot }) {
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

  requireSchema(schemas, findings, INSTANCE_STATE);
  requireSchema(schemas, findings, REVISION_SUMMARY);
  requireSchema(schemas, findings, TRAVERSAL_GRAPH);

  const instanceGet = requireGet(paths, findings, INSTANCE_GET_PATH);
  if (instanceGet) {
    const name = schemaRefName(json200Schema(instanceGet.operation));
    if (name !== INSTANCE_STATE) {
      push(
        findings,
        `${instanceGet.location}/responses/200`,
        `GET ${INSTANCE_GET_PATH} already returns Json<InstanceState> (current or as_of); `
          + `200 must $ref ${INSTANCE_STATE}, not additionalProperties`,
      );
    } else {
      bound += 1;
    }
    if (!parameterList(instanceGet.operation).some(isTimestampAsOfParam)) {
      push(
        findings,
        `${instanceGet.location}/parameters/as_of`,
        "instance GET must keep Timestamp as_of (get_as_of already reconstructs the slice)",
      );
    }
  }

  const instanceList = requireGet(paths, findings, INSTANCE_LIST_PATH);
  if (instanceList) {
    const name = arrayItemName(json200Schema(instanceList.operation));
    if (name !== INSTANCE_STATE) {
      push(
        findings,
        `${instanceList.location}/responses/200`,
        `GET ${INSTANCE_LIST_PATH} already returns Json<Vec<InstanceState>>; `
          + `200 must be an array of ${INSTANCE_STATE}, not additionalProperties`,
      );
    } else {
      bound += 1;
    }
  }

  const history = requireGet(paths, findings, INSTANCE_HISTORY_PATH);
  if (history) {
    const name = arrayItemName(json200Schema(history.operation));
    if (name !== REVISION_SUMMARY) {
      push(
        findings,
        `${history.location}/responses/200`,
        `GET ${INSTANCE_HISTORY_PATH} already returns Json<Vec<RevisionSummary>> from Instances::history; `
          + `200 must be an array of ${REVISION_SUMMARY}, not additionalProperties`,
      );
    } else {
      bound += 1;
    }
  }

  const traverse = requireGet(paths, findings, INSTANCE_TRAVERSE_PATH);
  if (traverse) {
    const name = schemaRefName(json200Schema(traverse.operation));
    if (name !== TRAVERSAL_GRAPH) {
      push(
        findings,
        `${traverse.location}/responses/200`,
        `GET ${INSTANCE_TRAVERSE_PATH} already returns Json<TraversalGraph>; `
          + `200 must $ref ${TRAVERSAL_GRAPH}, not additionalProperties`,
      );
    } else {
      bound += 1;
    }
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiOntologyInstanceTypes({ repoRoot });
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
  if (findings.length > 0 || belowGetFloor || bound !== 4) {
    console.error(
      `openapi ontology instance typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/4`,
    );
    process.exit(1);
  }
  console.log(
    `openapi ontology instance typed-response gate passed `
      + `(${INSTANCE_STATE} get/list, ${REVISION_SUMMARY} history, ${TRAVERSAL_GRAPH} traverse; `
      + `${gets} GET operations, 0 findings)`,
  );
}
