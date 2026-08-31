// Head collection-GET published-object gate.
//
// The hole this closes: PRODUCT requires typed objects that generate OpenAPI.
// Company / OrgUnit / Person already have instance GET (#1008) and runtime
// list/search ports (`PgCompanyPort::list`, `PgOrgUnitPort::list`,
// `PgPersonPort::list`). Instance GET alone is not a published collection.
// Employment already has GET /api/v1/employments. This gate requires a
// collection GET (200 schema is an array of the Head $ref) for every roster
// Head that already has a list port, except the L5-JOB / PayRun fences.
//
// Chesterton: publishing GET /api/v1/org-units (etc.) that returns the existing
// Head DTO is not a new store. Leptos org-chart / org-entities list via a
// different path and DTO; that does not invent these ports. as_of / from / to
// stay Employment-only — these Heads have no valid-time columns. Pagination
// (#273) is unscheduled; the list ports do not page.
//
// Totality: js-yaml load + own-property walk of every path item / GET. A walker
// that visits nothing reports nothing, so GET_FLOOR locks examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  FENCED_HEADS,
  FENCED_JOB_POSITION,
  FENCED_PAY_RUN,
  GET_FLOOR,
  TEMPORAL_ASOF_HEADS,
} from "./check-openapi-head-gets.mjs";
import { isTimestampAsOfParam } from "./check-openapi-hr-asof.mjs";
import { isIntegerNamedParam, isTimestampNamedParam } from "./check-openapi-hr-from-to.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { FENCED_HEADS, FENCED_JOB_POSITION, FENCED_PAY_RUN, GET_FLOOR, TEMPORAL_ASOF_HEADS };

/** Heads whose runtime list port already exists and must be published. */
export const REQUIRED_COLLECTION_GET_HEADS = Object.freeze([
  "Company",
  "OrgUnit",
  "Person",
]);

/** Exact collection paths — same shape as Employment GET /api/v1/employments. */
export const COLLECTION_PATHS = Object.freeze({
  Company: "/api/v1/companies",
  OrgUnit: "/api/v1/org-units",
  Person: "/api/v1/persons",
});

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
  const name = schemaRefName(own(schema, "items"));
  if (typeof name !== "string") return null;
  if (!HEAD_SCHEMA_NAMES.includes(name)) return null;
  return name;
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

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   collectionGetsByHead: Record<string, number>,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHeadCollections({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const collectionGetsByHead = Object.fromEntries(
    HEAD_SCHEMA_NAMES.map((name) => [name, 0]),
  );
  let gets = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { gets: 0, collectionGetsByHead, findings };
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
      const schema = json200Schema(operation);
      const itemsHead = arrayItemHeadName(schema);

      if (itemsHead && FENCED_HEADS.includes(itemsHead)) {
        push(
          findings,
          location,
          itemsHead === FENCED_JOB_POSITION
            ? "JobPosition Head must not gain a collection GET; L5-JOB still refuses inventing /api/v1/job-positions (identity stays action-receipt readback)"
            : "PayRun Head must not become a collection GET 200 schema; REST stays PayrollRunSummary and version stays absent",
        );
      }

      // Instance GET (direct Head $ref, typically with path id) is not a collection.
      if (itemsHead && !hasPathId(operation)) {
        collectionGetsByHead[itemsHead] += 1;
      }
    }
  }

  for (const name of REQUIRED_COLLECTION_GET_HEADS) {
    const collectionPath = own(COLLECTION_PATHS, name);
    const location = `#/paths/${collectionPath}/get`;
    const operation = findGet(paths, collectionPath);
    if (!operation) {
      push(
        findings,
        location,
        `GET ${collectionPath} must exist as the ${name} Head collection; `
          + "instance GET alone does not publish the list port",
      );
      continue;
    }
    if (hasPathId(operation)) {
      push(
        findings,
        `${location}/parameters/id`,
        `${name} collection GET must not take path parameter id (that is the instance GET)`,
      );
    }
    const itemsHead = arrayItemHeadName(json200Schema(operation));
    if (itemsHead !== name) {
      push(
        findings,
        location,
        `GET ${collectionPath} 200 must be an array of ${name} Heads `
          + "(instance GET / a different list DTO does not count)",
      );
    }
    const temporal = TEMPORAL_ASOF_HEADS.includes(name);
    if (!temporal && hasForbiddenRangeParam(operation)) {
      push(
        findings,
        `${location}/parameters`,
        `${name} has no valid-time store; do not document as_of/from/to the runtime cannot honor (created_at is not valid_from)`,
      );
    }
    if (itemsHead !== name || hasPathId(operation) || (!temporal && hasForbiddenRangeParam(operation))) {
      continue;
    }
    if (collectionGetsByHead[name] < 1) {
      collectionGetsByHead[name] = 1;
    }
  }

  for (const name of REQUIRED_COLLECTION_GET_HEADS) {
    if (collectionGetsByHead[name] < 1) {
      push(
        findings,
        `#/components/schemas/${name}`,
        `${name} Head is not a 200 schema of any collection GET; instance GET alone does not count`,
      );
    }
  }

  return { gets, collectionGetsByHead, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHeadCollections({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, collectionGetsByHead, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor) {
    console.error(
      `openapi Head collection-GET gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), collection GETs ${JSON.stringify(collectionGetsByHead)}`,
    );
    process.exit(1);
  }
  const published = REQUIRED_COLLECTION_GET_HEADS
    .map((name) => `${name}=${collectionGetsByHead[name]}`)
    .join(", ");
  console.log(
    `openapi Head collection-GET gate passed `
      + `(${published}; JobPosition/PayRun fenced; ${gets} GET operations, 0 findings)`,
  );
}
