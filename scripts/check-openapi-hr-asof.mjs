// HR as-of published-query gate (P0.4).
//
// The hole this closes: Employment already has append-only
// `employment_revisions` history (`valid_from`, half-open window on the head),
// but the published contract exposes `as_of` on only two GET operations, and
// neither is the Employment Head. One is ontology instance GET (RFC3339
// Timestamp, bi-temporal). The other is evidence-register GET (int64 sequence)
// — a different temporal model. SAP-class from/to, corrections, and delta are
// HOLD. This slice requires the instance GET's as_of query to be published on
// every GET whose 200 schema is the canonical Employment Head.
//
// Totality: js-yaml load + own-property walk of every path item / GET, same
// primitive as scripts/check-openapi-refs.mjs. A walker that visits nothing
// reports nothing, so GET_FLOOR is the examined-zero lock.
//
// Chesterton: copy the instance GET parameter (name as_of, query, optional,
// Timestamp $ref). Do not copy the evidence-register integer as_of. Do not
// invent from/to — they are not on the template.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = 200;
export const EMPLOYMENT_ASOF_FLOOR = 1;

export const TEMPLATE_PATH = "/api/v1/ontology/instances/{id}";
export const EVIDENCE_PATH = "/api/v1/evidence/objects";
export const EMPLOYMENT_SCHEMA = "Employment";
export const TIMESTAMP_REF = "#/components/schemas/Timestamp";

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

export function isTimestampAsOfParam(param) {
  if (!isPlainObject(param)) return false;
  if (own(param, "name") !== "as_of") return false;
  if (own(param, "in") !== "query") return false;
  if (own(param, "required") === true) return false;
  const schema = own(param, "schema");
  return schemaRefName(schema) === "Timestamp";
}

export function isIntegerAsOfParam(param) {
  if (!isPlainObject(param)) return false;
  if (own(param, "name") !== "as_of") return false;
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

export function operationReturnsSchema(operation, schemaName) {
  const schema = json200Schema(operation);
  if (schemaRefName(schema) === schemaName) return true;
  if (!isPlainObject(schema)) return false;
  for (const key of ["allOf", "oneOf", "anyOf"]) {
    const members = own(schema, key);
    if (!Array.isArray(members)) continue;
    if (members.some((member) => schemaRefName(member) === schemaName)) return true;
  }
  return false;
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   employmentGets: number,
 *   employmentAsOf: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHrAsof({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const paths = own(document, "paths");
  let gets = 0;
  let employmentGets = 0;
  let employmentAsOf = 0;

  if (!isPlainObject(paths)) {
    findings.push({
      location: "#/paths",
      message: "published document has no paths mapping",
    });
    return { gets: 0, employmentGets: 0, employmentAsOf: 0, findings };
  }

  const templateGet = findGet(paths, TEMPLATE_PATH);
  const templateParams = parameterList(templateGet);
  const templateAsOf = templateParams.find(isTimestampAsOfParam);
  if (!templateAsOf) {
    findings.push({
      location: `#/paths/${TEMPLATE_PATH}/get`,
      message:
        "template GET must keep optional as_of query with Timestamp $ref "
        + "(RFC3339 bi-temporal); do not replace it with the evidence-register integer",
    });
  }

  const evidenceGet = findGet(paths, EVIDENCE_PATH);
  const evidenceAsOf = parameterList(evidenceGet).find(isIntegerAsOfParam);
  if (evidenceGet && !evidenceAsOf) {
    findings.push({
      location: `#/paths/${EVIDENCE_PATH}/get`,
      message:
        "evidence GET as_of is the integer register snapshot, not the HR template; "
        + "do not silently drop it while copying Timestamp as_of elsewhere",
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
      const operation = own(item, method);
      const location = `#/paths/${path}/get`;
      if (!operationReturnsSchema(operation, EMPLOYMENT_SCHEMA)) continue;
      employmentGets += 1;
      const asOf = parameterList(operation).find(isTimestampAsOfParam);
      const integerAsOf = parameterList(operation).find(isIntegerAsOfParam);
      if (integerAsOf && !asOf) {
        findings.push({
          location,
          message:
            "Employment GET as_of must be the instance-GET Timestamp query, "
            + "not the evidence-register integer sequence",
        });
        continue;
      }
      if (!asOf) {
        findings.push({
          location,
          message:
            "GET that returns Employment must publish optional as_of query "
            + `($ref ${TIMESTAMP_REF}); absent = current open head`,
        });
        continue;
      }
      employmentAsOf += 1;
    }
  }

  if (employmentAsOf < EMPLOYMENT_ASOF_FLOOR) {
    findings.push({
      location: `#/components/schemas/${EMPLOYMENT_SCHEMA}`,
      message:
        `Employment Head is published but ${employmentAsOf} of ${EMPLOYMENT_ASOF_FLOOR} `
        + "required GET operations expose Timestamp as_of; history exists on "
        + "employment_revisions and must be a consistent published query. "
        + "from/to, corrections, and delta remain HOLD.",
    });
  }

  return { gets, employmentGets, employmentAsOf, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHrAsof({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, employmentGets, employmentAsOf, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor) {
    console.error(
      `openapi HR as-of gate FAILED: ${findings.length} finding(s), `
        + `${employmentAsOf}/${EMPLOYMENT_ASOF_FLOOR} Employment GET as_of, `
        + `${employmentGets} Employment GET(s), ${gets} GET(s)`,
    );
    process.exit(1);
  }
  console.log(
    `openapi HR as-of gate passed (${employmentAsOf}/${EMPLOYMENT_ASOF_FLOOR} `
      + `Employment GET as_of, ${gets} GET operations, 0 findings)`,
  );
}
