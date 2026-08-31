// OrgUnit → JobPosition reverse-collection published-link gate.
//
// The hole this closes: Palantir-class links are traversable both ways.
// JobPosition.org_unit_id already binds getOrgUnit (#1013). PgJobPositionPort::
// list_for_org_unit already returns current heads under one OrgUnit. The
// published contract still has no GET that is that collection, and OrgUnit Head
// links stay parent_id-only. Instance GET / GET /api/v1/job-positions (tenant
// list) is not the reverse traversal.
//
// Chesterton: this is not a JobPosition SSR directory (shipping screens stay
// `/` `/organization` `/hr` `/payroll`). as_of / from / to stay omitted —
// job_positions has no valid-time columns. Pagination (#273) is unscheduled;
// the port does not page. PayRun stays PayrollRunSummary. Do not invent a
// second FK; field is the existing JobPosition.org_unit_id the port filters on.
// Empty when the unit has no positions or is invisible — same as the port
// (a missing OrgUnit is GET /api/v1/org-units/{id} 404, not this collection).
//
// Totality: js-yaml load + own-property walk of every GET and the OrgUnit
// schema-level links. A walker that visits nothing reports nothing, so
// GET_FLOOR locks examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  FENCED_HEADS,
  FENCED_PAY_RUN,
  GET_FLOOR,
} from "./check-openapi-head-gets.mjs";
import { isTimestampAsOfParam } from "./check-openapi-hr-asof.mjs";
import { isIntegerNamedParam, isTimestampNamedParam } from "./check-openapi-hr-from-to.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { FENCED_HEADS, FENCED_PAY_RUN, GET_FLOOR };

export const REVERSE_PATH = "/api/v1/org-units/{id}/job-positions";
export const REVERSE_OPERATION_ID = "listOrgUnitJobPositions";
export const REVERSE_LINK_KEY = "org_unit_job_positions";
export const REVERSE_FROM = "OrgUnit";
export const REVERSE_TO = "JobPosition";
export const REVERSE_FIELD = "org_unit_id";
export const REVERSE_CARDINALITY = "one-to-many";
export const REVERSE_OPTION = false;

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

const FORBIDDEN_RANGE_NAMES = Object.freeze(["as_of", "from", "to"]);

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

function hasPathId(operation) {
  return parameterList(operation).some(
    (param) =>
      isPlainObject(param) && own(param, "name") === "id" && own(param, "in") === "path",
  );
}

function arrayItemHeadName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
}

function hasForbiddenRangeParam(operation) {
  return parameterList(operation).some((param) => {
    if (!isPlainObject(param)) return false;
    if (isTimestampAsOfParam(param)) return true;
    for (const name of FORBIDDEN_RANGE_NAMES) {
      if (isTimestampNamedParam(param, name) || isIntegerNamedParam(param, name)) {
        return true;
      }
    }
    return own(param, "name") === "as_of" && own(param, "in") === "query";
  });
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function findLink(list, key) {
  if (!Array.isArray(list)) return null;
  return list.find((item) => isPlainObject(item) && own(item, "key") === key) ?? null;
}

function linkMentionsAsOf(link) {
  if (hasOwnKey(link, "as_of")) return true;
  const parameters = own(link, "parameters");
  if (isPlainObject(parameters) && hasOwnKey(parameters, "as_of")) return true;
  const href = own(link, "href");
  if (typeof href === "string" && href.includes("as_of")) return true;
  return false;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   reverse: boolean,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiOrgUnitJobPositions({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let gets = 0;
  let reverse = false;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { gets: 0, reverse, findings };
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
      const location = `#/paths/${path}/get`;
      const itemsHead = arrayItemHeadName(json200Schema(operation));
      if (itemsHead && FENCED_HEADS.includes(itemsHead)) {
        push(
          findings,
          location,
          "PayRun Head must not become a GET 200 schema; REST stays PayrollRunSummary and version stays absent",
        );
      }
    }
  }

  const location = `#/paths/${REVERSE_PATH}/get`;
  const operation = findGet(paths, REVERSE_PATH);
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `GET ${REVERSE_PATH} must exist as the OrgUnit→JobPosition reverse collection; `
        + "PgJobPositionPort::list_for_org_unit is already the port, and GET /api/v1/job-positions is the tenant list not this traversal",
    );
  } else {
    const operationId = own(operation, "operationId");
    if (operationId !== REVERSE_OPERATION_ID) {
      push(
        findings,
        `${location}/operationId`,
        `must be ${REVERSE_OPERATION_ID} (list_for_org_unit), got ${JSON.stringify(operationId)}`,
      );
    }
    if (!hasPathId(operation)) {
      push(
        findings,
        `${location}/parameters/id`,
        "reverse collection must take path parameter id (the OrgUnit)",
      );
    }
    const itemsHead = arrayItemHeadName(json200Schema(operation));
    if (itemsHead !== REVERSE_TO) {
      push(
        findings,
        location,
        `GET ${REVERSE_PATH} 200 must be an array of ${REVERSE_TO} Heads `
          + "(tenant-wide listJobPositions / a different list DTO does not count)",
      );
    }
    if (hasForbiddenRangeParam(operation)) {
      push(
        findings,
        `${location}/parameters`,
        "JobPosition has no valid-time store; do not document as_of/from/to the runtime cannot honor (created_at is not valid_from)",
      );
    }
    if (
      operationId === REVERSE_OPERATION_ID
      && hasPathId(operation)
      && itemsHead === REVERSE_TO
      && !hasForbiddenRangeParam(operation)
    ) {
      reverse = true;
    }
  }

  const schemaLoc = `#/components/schemas/${REVERSE_FROM}`;
  const schema = isPlainObject(schemas) ? own(schemas, REVERSE_FROM) : undefined;
  const declared = isPlainObject(schema) ? own(schema, "links") : undefined;
  const found = findLink(declared, REVERSE_LINK_KEY);
  const linkLoc = `${schemaLoc}/links/${REVERSE_LINK_KEY}`;
  if (!found) {
    push(
      findings,
      linkLoc,
      `OrgUnit Head must declare reverse collection ${REVERSE_LINK_KEY} bound to ${REVERSE_OPERATION_ID}; `
        + "schema-only parent_id is not a traversable OrgUnit→JobPosition link",
    );
  } else {
    for (const [field, value] of [
      ["from", REVERSE_FROM],
      ["to", REVERSE_TO],
      ["field", REVERSE_FIELD],
      ["cardinality", REVERSE_CARDINALITY],
    ]) {
      if (own(found, field) !== value) {
        push(
          findings,
          `${linkLoc}/${field}`,
          `must be ${value} (list_for_org_unit filters JobPosition.${REVERSE_FIELD}), got ${JSON.stringify(own(found, field))}`,
        );
      }
    }
    if (own(found, "option") !== REVERSE_OPTION) {
      push(
        findings,
        `${linkLoc}/option`,
        "must be false — empty array is still a collection, not an optional FK",
      );
    }
    if (linkMentionsAsOf(found)) {
      push(
        findings,
        `${linkLoc}/as_of`,
        "JobPosition has no valid-time store on this reverse; do not document as_of the runtime cannot honor",
      );
    }
    const operationId = own(found, "operationId");
    if (operationId !== REVERSE_OPERATION_ID) {
      push(
        findings,
        `${linkLoc}/operationId`,
        `must be ${REVERSE_OPERATION_ID} (GET ${REVERSE_PATH}), got ${JSON.stringify(operationId)}; `
          + "getJobPosition is the instance GET, not this reverse collection",
      );
    }
  }

  return { gets, reverse, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiOrgUnitJobPositions({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, reverse, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor || !reverse) {
    console.error(
      `openapi OrgUnit→JobPosition reverse-collection gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), reverse=${reverse}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi OrgUnit→JobPosition reverse-collection gate passed `
      + `(${REVERSE_PATH} ${REVERSE_OPERATION_ID}; OrgUnit.links.${REVERSE_LINK_KEY}; `
      + `${gets} GET operations, 0 findings)`,
  );
}
