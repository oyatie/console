// Generated SDK/docs path-schema gate (#1018-class).
//
// The hole this closes: #1020–#1040 bound Palantir-class envelopes onto
// composed OpenAPI (`GateOutcome` / `ConditionValue` / `OverrideSummary` /
// `DraftRecord` / `ObjectTypeDetail` / `OntologyActionExecuteOutcome` /
// `EmployeeExitCaseResponse` / `AbsenceExitDashboardResponse`, plus Head GET
// permissions and link operationIds). The in-repo TypeScript SDK and `sdk/docs`
// still start from the 21-name Head/Input roster, so those composed schemas
// never emit. Same class as #1018 docs drift: generator agrees with committed
// bytes and still drops what compose already publishes. Do not invent a name
// catalog. Walk `paths` `$ref`s and that set's schema-body closure.
//
// Chesterton: do not replace compose. Do not bind kill-switch / rollout. Do
// not map Feature::ALL. Do not stamp HTTP ETag. Do not redo #1016–#1040
// enum/envelope binds. Head GET permissions and link operationIds stay on the
// existing Head definitions (#1013/#1017/#1018).
//
// Totality: js-yaml load + own-property walk of every path `$ref` + schema
// closure. A walker that visits nothing reports nothing, so PATH_SCHEMA_FLOOR
// / CONTRACT_SCHEMA_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  KILL_SWITCH_PATH,
  ROLLOUT_OPT_IN_PATH,
  ROLLOUT_ORG_FLAG_PATH,
} from "./check-openapi-execute-outcome.mjs";
import { DOCS_REL, OPENAPI_REL } from "./generate-openapi-docs.mjs";
import { SDK_GENERATED_REL, composedPathSchemaNames } from "./generate-openapi-ts-sdk.mjs";
import { isPlainObject, own } from "./own-property.mjs";

/**
 * Names #1020–#1040 already published in composed OpenAPI. Not a generation
 * catalog: the generator walks `paths`. The probe fails if compose still
 * has these and generated SDK/docs drop them.
 */
export const COMPOSED_CONTRACT_SCHEMAS = Object.freeze([
  "AbsenceExitDashboardResponse",
  "ConditionValue",
  "DraftRecord",
  "EmployeeExitCaseResponse",
  "GateKind",
  "GateOutcome",
  "GateStatus",
  "ObjectTypeDetail",
  "OntologyActionExecuteOutcome",
  "OverrideSummary",
]);

export const CONTRACT_SCHEMA_FLOOR = COMPOSED_CONTRACT_SCHEMAS.length;

/** Examined-zero lock: composed paths must `$ref` the Palantir-class set. */
export const PATH_SCHEMA_FLOOR = CONTRACT_SCHEMA_FLOOR;

export { DOCS_REL, OPENAPI_REL, SDK_GENERATED_REL };

function push(findings, location, message) {
  findings.push({ location, message });
}

function hasExportedType(source, name) {
  if (typeof source !== "string") return false;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`export type ${escaped}\\s*=`).test(source);
}

function hasSchemaSection(html, name) {
  if (typeof html !== "string") return false;
  return html.includes(`id="schema-${name}"`);
}

function jsonOkSchema(operation, code = "200") {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

function findOperation(paths, path, method) {
  const item = own(paths, path);
  return own(item, method);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   pathSchemas: number,
 *   contract: number,
 *   sdk: number,
 *   docs: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateGeneratedPathSchemas({ repoRoot }) {
  const findings = [];
  const openapiPath = join(repoRoot, OPENAPI_REL);
  const sdkPath = join(repoRoot, SDK_GENERATED_REL);
  const docsPath = join(repoRoot, DOCS_REL);
  if (!existsSync(openapiPath)) {
    push(findings, OPENAPI_REL, "composed OpenAPI document is missing");
    return { pathSchemas: 0, contract: 0, sdk: 0, docs: 0, findings };
  }
  if (!existsSync(sdkPath)) {
    push(findings, SDK_GENERATED_REL, "generated TypeScript SDK is missing");
  }
  if (!existsSync(docsPath)) {
    push(findings, DOCS_REL, "generated docs artifact is missing");
  }

  let document;
  try {
    document = yaml.load(readFileSync(openapiPath, "utf8"));
  } catch (error) {
    push(findings, OPENAPI_REL, `cannot parse: ${error.message}`);
    return { pathSchemas: 0, contract: 0, sdk: 0, docs: 0, findings };
  }

  const schemas = own(own(document, "components"), "schemas");
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { pathSchemas: 0, contract: 0, sdk: 0, docs: 0, findings };
  }

  const names = composedPathSchemaNames(document);
  const sdkSource = existsSync(sdkPath) ? readFileSync(sdkPath, "utf8") : "";
  const html = existsSync(docsPath) ? readFileSync(docsPath, "utf8") : "";

  let sdk = 0;
  let docs = 0;
  for (const name of names) {
    const loc = `#/components/schemas/${name}`;
    const inSdk = hasExportedType(sdkSource, name);
    const inDocs = hasSchemaSection(html, name);
    if (inSdk) sdk += 1;
    else {
      push(
        findings,
        `${SDK_GENERATED_REL}:${name}`,
        "generated SDK drops this composed path schema; copy OpenAPI, do not leave the 21-name roster as the emit set",
      );
    }
    if (inDocs) docs += 1;
    else {
      push(
        findings,
        `${DOCS_REL}:${name}`,
        "generated docs drop this composed path schema; same class as #1018 field drift",
      );
    }
    if (!isPlainObject(own(schemas, name))) {
      push(findings, loc, "path $ref names a schema that is not published under components.schemas");
    }
  }

  let contract = 0;
  for (const name of COMPOSED_CONTRACT_SCHEMAS) {
    if (!isPlainObject(own(schemas, name))) {
      push(
        findings,
        `#/components/schemas/${name}`,
        `${name} is already published on origin/dev; do not drop it from composed OpenAPI in this slice`,
      );
      continue;
    }
    contract += 1;
    if (!names.includes(name)) {
      push(
        findings,
        locForMissingPathReach(name),
        `${name} is published but not reachable from paths; generator must not invent a parallel catalog`,
      );
    }
  }

  const paths = own(document, "paths");
  for (const [path, method, label] of [
    [KILL_SWITCH_PATH, "post", "console kill-switch"],
    [ROLLOUT_OPT_IN_PATH, "put", "console rollout"],
    [ROLLOUT_ORG_FLAG_PATH, "put", "console rollout"],
  ]) {
    const operation = findOperation(paths, path, method);
    if (!isPlainObject(operation)) continue;
    const bound = schemaRefName(jsonOkSchema(operation, "200"));
    if (COMPOSED_CONTRACT_SCHEMAS.includes(bound)) {
      push(
        findings,
        `#/paths/${path}/${method}/responses/200`,
        `${label} must not bind ${bound}; do not bind kill-switch / rollout`,
      );
    }
    const listed = own(operation, "permissions");
    if (Array.isArray(listed) && listed.length > 0) {
      push(
        findings,
        `#/paths/${path}/${method}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto ${method.toUpperCase()} ${path}`,
      );
    }
  }

  if (names.length < PATH_SCHEMA_FLOOR) {
    push(
      findings,
      "#/paths",
      `saw ${names.length} composed path schemas — below the floor ${PATH_SCHEMA_FLOOR}`,
    );
  }
  if (contract < CONTRACT_SCHEMA_FLOOR) {
    push(
      findings,
      "#/components/schemas",
      `saw ${contract}/${CONTRACT_SCHEMA_FLOOR} Palantir-class composed schemas — below the floor`,
    );
  }

  return { pathSchemas: names.length, contract, sdk, docs, findings };
}

function locForMissingPathReach(name) {
  return `#/components/schemas/${name}`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateGeneratedPathSchemas({ repoRoot });
  } catch (error) {
    console.error(`generated path-schema gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { pathSchemas, contract, sdk, docs, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    pathSchemas < PATH_SCHEMA_FLOOR || contract < CONTRACT_SCHEMA_FLOOR;
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi generated path-schema gate FAILED: ${findings.length} finding(s), `
        + `${sdk}/${pathSchemas} SDK types, ${docs}/${pathSchemas} docs sections, `
        + `${contract}/${CONTRACT_SCHEMA_FLOOR} Palantir-class composed schemas`,
    );
    process.exit(1);
  }
  console.log(
    `openapi generated path-schema gate passed `
      + `(${sdk}/${pathSchemas} SDK types, ${docs}/${pathSchemas} docs sections, `
      + `${contract}/${CONTRACT_SCHEMA_FLOOR} Palantir-class composed schemas, 0 findings)`,
  );
}
