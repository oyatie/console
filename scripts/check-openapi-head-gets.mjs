// Head instance-GET published-object gate.
//
// The hole this closes: PRODUCT requires typed objects that generate OpenAPI.
// Company / OrgUnit / JobPosition / Person / PayRun Head schemas already exist
// (DTO-generated), but a Head schema without an instance GET is not a published
// object. Employment already has list + instance GET. This gate requires an
// instance GET (200 schema is the Head $ref, not an array) for every roster
// Head that already has a runtime get-by-id port, and refuses as_of on Heads
// with no valid-time store.
//
// Chesterton: L5-JOB still refuses inventing `/api/v1/job-positions` — JobPosition
// identity stays action-receipt readback. PayRun REST remains PayrollRunSummary
// (no Head GET, no fake version). OrgUnit / Company / Person ports already
// load by id; publishing GET that returns the existing Head DTO is not a new
// store. as_of stays Employment/ontology-instance only; using created_at as
// valid_from would be a second time model.
//
// Totality: js-yaml load + own-property walk of every path item / GET. A walker
// that visits nothing reports nothing, so GET_FLOOR locks examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  GET_FLOOR as ASOF_GET_FLOOR,
  isTimestampAsOfParam,
} from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

/** Heads whose runtime get-by-id port already exists and must be published. */
export const REQUIRED_INSTANCE_GET_HEADS = Object.freeze([
  "Company",
  "OrgUnit",
  "Person",
  "Employment",
]);

/** L5-JOB fence: do not invent `/api/v1/job-positions`. */
export const FENCED_JOB_POSITION = "JobPosition";
/** PayRun GET stays PayrollRunSummary; no versioned Head GET. */
export const FENCED_PAY_RUN = "PayRun";
export const FENCED_HEADS = Object.freeze([FENCED_JOB_POSITION, FENCED_PAY_RUN]);

/** Only Employment already has the ontology half-open as_of algebra. */
export const TEMPORAL_ASOF_HEADS = Object.freeze(["Employment"]);

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

function hasPathId(operation) {
  return parameterList(operation).some(
    (param) =>
      isPlainObject(param) && own(param, "name") === "id" && own(param, "in") === "path",
  );
}

function directHeadName(schema) {
  const name = schemaRefName(schema);
  if (typeof name !== "string") return null;
  if (!HEAD_SCHEMA_NAMES.includes(name)) return null;
  return name;
}

function arrayItemHeadName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return directHeadName(own(schema, "items"));
}

function allHeadNamesFromSchema(schema) {
  const names = [];
  const direct = directHeadName(schema);
  if (direct) names.push(direct);
  const items = arrayItemHeadName(schema);
  if (items) names.push(items);
  return names;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   instanceGetsByHead: Record<string, number>,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHeadGets({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const instanceGetsByHead = Object.fromEntries(
    HEAD_SCHEMA_NAMES.map((name) => [name, 0]),
  );
  let gets = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { gets: 0, instanceGetsByHead, findings };
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
      const headNames = allHeadNamesFromSchema(schema);
      const direct = directHeadName(schema);

      for (const name of headNames) {
        if (FENCED_HEADS.includes(name)) {
          push(
            findings,
            location,
            name === FENCED_JOB_POSITION
              ? "JobPosition Head must not gain a GET; L5-JOB still refuses inventing /api/v1/job-positions (identity stays action-receipt readback)"
              : "PayRun Head must not become a GET 200 schema; REST stays PayrollRunSummary and version stays absent",
          );
        }
      }

      if (direct && REQUIRED_INSTANCE_GET_HEADS.includes(direct)) {
        if (!hasPathId(operation)) {
          push(
            findings,
            `${location}/parameters/id`,
            `${direct} instance GET must take path parameter id (publishing the object, not a collection)`,
          );
        } else {
          instanceGetsByHead[direct] += 1;
        }
        const temporal = TEMPORAL_ASOF_HEADS.includes(direct);
        const asOf = parameterList(operation).find(isTimestampAsOfParam);
        const anyAsOf = parameterList(operation).some(
          (param) => isPlainObject(param) && own(param, "name") === "as_of",
        );
        if (temporal && !asOf) {
          push(
            findings,
            `${location}/parameters/as_of`,
            `${direct} already has the Employment/ontology as_of algebra; instance GET must keep optional Timestamp as_of`,
          );
        }
        if (!temporal && anyAsOf) {
          push(
            findings,
            `${location}/parameters/as_of`,
            `${direct} has no valid-time store; do not document as_of the runtime cannot honor (created_at is not valid_from)`,
          );
        }
      }
    }
  }

  for (const name of REQUIRED_INSTANCE_GET_HEADS) {
    if (instanceGetsByHead[name] < 1) {
      push(
        findings,
        `#/components/schemas/${name}`,
        `${name} Head is not a 200 schema of any instance GET; a Head schema without an instance GET is not a published object`,
      );
    }
  }

  return { gets, instanceGetsByHead, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHeadGets({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, instanceGetsByHead, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor) {
    console.error(
      `openapi Head instance-GET gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), instance GETs ${JSON.stringify(instanceGetsByHead)}`,
    );
    process.exit(1);
  }
  const published = REQUIRED_INSTANCE_GET_HEADS
    .map((name) => `${name}=${instanceGetsByHead[name]}`)
    .join(", ");
  console.log(
    `openapi Head instance-GET gate passed `
      + `(${published}; JobPosition/PayRun fenced; ${gets} GET operations, 0 findings)`,
  );
}
