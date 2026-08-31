// HR from/to published-query gate.
//
// The hole this closes: Employment GET already publishes Timestamp as_of
// (same algebra as ontology instance GET). Collection list/search does not:
// there is no GET /api/v1/employments, so from/to cannot be a published query.
// PRODUCT admits asOf / from / to as core HR. This slice requires the
// Employment collection GET to publish optional from and to Timestamp queries
// using that same half-open algebra (`valid_from < to` AND
// `valid_to IS NULL OR from < valid_to`; absent both = current open heads).
//
// EmployeeDetail as_of remains HOLD. Do not add as_of/from/to on
// /api/v1/employees or /api/v1/employees/{id}. Corrections vs new slice and
// delta transmission remain HOLD.
//
// Totality: js-yaml load + own-property walk of every path item / GET, same
// primitive as scripts/check-openapi-hr-asof.mjs. A walker that visits nothing
// reports nothing, so GET_FLOOR is the examined-zero lock.
//
// Chesterton: copy the instance-GET Timestamp parameter shape (query, optional,
// Timestamp $ref) onto names from and to. Do not copy the evidence-register
// integer as_of. Do not invent a second time model.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  EMPLOYMENT_SCHEMA,
  GET_FLOOR,
  TEMPLATE_PATH,
  TIMESTAMP_REF,
  isIntegerAsOfParam,
  isTimestampAsOfParam,
} from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { GET_FLOOR, TIMESTAMP_REF };

export const COLLECTION_PATH = "/api/v1/employments";
export const EMPLOYEES_PATH = "/api/v1/employees";
export const EMPLOYEE_DETAIL_PATH = "/api/v1/employees/{id}";
export const COLLECTION_FLOOR = 1;

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

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

export function isTimestampNamedParam(param, name) {
  if (!isPlainObject(param)) return false;
  if (own(param, "name") !== name) return false;
  if (own(param, "in") !== "query") return false;
  if (own(param, "required") === true) return false;
  const schema = own(param, "schema");
  return schemaRefName(schema) === "Timestamp";
}

export function isIntegerNamedParam(param, name) {
  if (!isPlainObject(param)) return false;
  if (own(param, "name") !== name) return false;
  if (own(param, "in") !== "query") return false;
  const schema = own(param, "schema");
  if (!isPlainObject(schema)) return false;
  return own(schema, "type") === "integer";
}

function parameterList(operation) {
  const parameters = own(operation, "parameters");
  return Array.isArray(parameters) ? parameters : [];
}

function json200Schema(operation) {
  const responses = own(operation, "responses");
  const ok = own(responses, "200") ?? own(responses, 200);
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

export function operationReturnsEmploymentArray(operation) {
  const schema = json200Schema(operation);
  if (!isPlainObject(schema)) return false;
  if (own(schema, "type") !== "array") return false;
  return schemaRefName(own(schema, "items")) === EMPLOYMENT_SCHEMA;
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function pushHoldPii(findings, path, operation) {
  const params = parameterList(operation);
  const forbidden = [];
  for (const name of ["as_of", "from", "to"]) {
    if (
      params.some((param) => isTimestampNamedParam(param, name))
      || params.some((param) => isIntegerNamedParam(param, name))
      || (name === "as_of" && params.some(isTimestampAsOfParam))
      || (name === "as_of" && params.some(isIntegerAsOfParam))
    ) {
      forbidden.push(name);
    }
  }
  if (forbidden.length > 0) {
    findings.push({
      location: `#/paths/${path}/get`,
      message:
        "EmployeeDetail as_of/from/to remains HOLD (PII); do not publish "
        + `${forbidden.join("/")} on ${path}`,
    });
  }
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   collectionFromTo: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHrFromTo({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const paths = own(document, "paths");
  let gets = 0;
  let collectionFromTo = 0;

  if (!isPlainObject(paths)) {
    findings.push({
      location: "#/paths",
      message: "published document has no paths mapping",
    });
    return { gets: 0, collectionFromTo: 0, findings };
  }

  const templateGet = findGet(paths, TEMPLATE_PATH);
  const templateParams = parameterList(templateGet);
  const templateAsOf = templateParams.find(isTimestampAsOfParam);
  if (!templateAsOf) {
    findings.push({
      location: `#/paths/${TEMPLATE_PATH}/get`,
      message:
        "template GET must keep optional as_of query with Timestamp $ref "
        + "(RFC3339 bi-temporal); from/to copy that shape, not a second time model",
    });
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method) || !HTTP_METHODS.has(method)) continue;
      if (method !== "get") continue;
      gets += 1;
    }
  }

  const employeesGet = findGet(paths, EMPLOYEES_PATH);
  if (employeesGet) pushHoldPii(findings, EMPLOYEES_PATH, employeesGet);
  const employeeDetailGet = findGet(paths, EMPLOYEE_DETAIL_PATH);
  if (employeeDetailGet) {
    pushHoldPii(findings, EMPLOYEE_DETAIL_PATH, employeeDetailGet);
  }

  const collectionGet = findGet(paths, COLLECTION_PATH);
  const location = `#/paths/${COLLECTION_PATH}/get`;
  if (!collectionGet) {
    findings.push({
      location,
      message:
        "GET /api/v1/employments must exist as the Employment Head collection; "
        + "from/to cannot be a published query on a missing list/search",
    });
  } else {
    const params = parameterList(collectionGet);
    const fromParam = params.find((param) => isTimestampNamedParam(param, "from"));
    const toParam = params.find((param) => isTimestampNamedParam(param, "to"));
    const integerFrom = params.find((param) => isIntegerNamedParam(param, "from"));
    const integerTo = params.find((param) => isIntegerNamedParam(param, "to"));
    if ((integerFrom || integerTo) && (!fromParam || !toParam)) {
      findings.push({
        location,
        message:
          "Employment collection from/to must be the instance-GET Timestamp query, "
          + "not the evidence-register integer sequence",
      });
    } else if (!fromParam || !toParam) {
      findings.push({
        location,
        message:
          "GET /api/v1/employments must publish optional from and to queries "
          + `($ref ${TIMESTAMP_REF}); absent both = current open heads; `
          + "window overlap uses the same half-open algebra as as_of",
      });
    } else if (!operationReturnsEmploymentArray(collectionGet)) {
      findings.push({
        location,
        message:
          "GET /api/v1/employments 200 must be an array of Employment Heads "
          + "(not EmployeePage / EmployeeDetail)",
      });
    } else {
      collectionFromTo += 1;
    }
  }

  if (collectionFromTo < COLLECTION_FLOOR) {
    findings.push({
      location: `#/components/schemas/${EMPLOYMENT_SCHEMA}`,
      message:
        `Employment collection from/to is published on ${collectionFromTo} of `
        + `${COLLECTION_FLOOR} required GET operations; history exists on `
        + "employment_heads / employment_revisions and must be a consistent "
        + "published range query. EmployeeDetail as_of remains HOLD.",
    });
  }

  return { gets, collectionFromTo, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHrFromTo({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, collectionFromTo, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor) {
    console.error(
      `openapi HR from/to gate FAILED: ${findings.length} finding(s), `
        + `${collectionFromTo}/${COLLECTION_FLOOR} Employment collection from/to, `
        + `${gets} GET(s)`,
    );
    process.exit(1);
  }
  console.log(
    `openapi HR from/to gate passed (${collectionFromTo}/${COLLECTION_FLOOR} `
      + `Employment collection from/to, ${gets} GET operations, 0 findings)`,
  );
}
