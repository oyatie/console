// GET /api/v1/hr/absence-exit-dashboard 200 AbsenceExitDashboardResponse bind gate.
//
// The hole this closes: get_absence_exit_dashboard already returns
// Json<AbsenceExitDashboardResponse> whose envelope is summary / alerts /
// exit_cases. alerts[].signal_payload is serde_json::Value; nested exit_cases
// are the same EmployeeExitCaseResponse published in #1035 (settlement
// statutory_basis / insurance_loss_payload / approval_payload stay Value).
// Composed OpenAPI still advertises the 200 as additionalProperties: true.
// Same class as #1035 EmployeeExitCaseResponse — publish the existing record,
// do not invent a store, statutory catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing structs, then $ref. Type every
// non-Value field. Value bags stay unconstrained object /
// additionalProperties: true on THOSE FIELDS ONLY. Nested exit cases $ref
// EmployeeExitCaseResponse rather than duplicating. This is not Korea
// statutory product: do not interpret, close, or catalog those bags. Do not
// bind ObjectTypeDetail / kill-switch / rollout. Do not map Feature::ALL.
// Do not stamp HTTP ETag. Do not invent query params.
//
// Totality: js-yaml load + own-property walk of every GET and write method +
// optional Rust struct field read. GET_FLOOR / WRITE_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import {
  EXIT_CASE_RESPONSE,
  SETTLEMENT_PACKAGE,
  VALUE_FIELDS as SETTLEMENT_VALUE_FIELDS,
} from "./check-openapi-employee-exit-case-response.mjs";
import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;
export const WRITE_FLOOR = PREFLIGHT_WRITE_FLOOR;

export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const KILL_SWITCH_PATH = "/api/v1/console/kill-switch";
export const ROLLOUT_OPT_IN_PATH = "/api/v1/console/rollout/opt-in";
export const ROLLOUT_ORG_FLAG_PATH = "/api/v1/console/rollout/org-flag";
export const DRAFTS_PATH = "/api/v1/policy/drafts";
export const CATALOG_GET_PATH = "/api/v1/policy/catalog";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const READINESS_GET_PATH = "/api/v1/hr/readiness-summary";
export const EXIT_CASES_PATH = "/api/v1/hr/exit-cases";

export const DASHBOARD_RESPONSE = "AbsenceExitDashboardResponse";
export const SUMMARY = "AbsenceExitSummary";
export const ALERT_RESPONSE = "EmployeeAbsenceAlertResponse";
export { EXIT_CASE_RESPONSE, SETTLEMENT_PACKAGE, SETTLEMENT_VALUE_FIELDS };
export const DRAFT_RECORD = "DraftRecord";
export const CATALOG_ENTRY = "CatalogEntry";
export const OVERRIDE_SUMMARY = "OverrideSummary";
export const HR_READINESS_SUMMARY = "HrReadinessSummary";
export const BOUND = 1;

export const HR_RS_REL = "backend/app/src/hr.rs";

/** Existing Serialize fields. Do not invent names. */
export const DASHBOARD_FIELDS = Object.freeze(["summary", "alerts", "exit_cases"]);
export const DASHBOARD_REQUIRED = DASHBOARD_FIELDS;

export const SUMMARY_FIELDS = Object.freeze([
  "open_absence_alerts",
  "exit_cases_pending_hr",
  "settlement_needs_source",
  "settlement_ready",
  "approval_drafts",
  "submitted",
]);

export const ALERT_FIELDS = Object.freeze([
  "id",
  "employee_id",
  "employee_name",
  "employee_number",
  "company",
  "org_unit",
  "worksite_name",
  "branch_id",
  "branch_name",
  "work_date",
  "source",
  "status",
  "severity",
  "audience_roles",
  "signal_payload",
  "notification_title",
  "notification_message",
  "link_href",
  "exit_case_id",
  "detected_at",
]);

export const ALERT_REQUIRED = Object.freeze([
  "id",
  "employee_id",
  "employee_name",
  "company",
  "work_date",
  "source",
  "status",
  "severity",
  "audience_roles",
  "signal_payload",
  "notification_title",
  "notification_message",
  "link_href",
  "detected_at",
]);

export const SIGNAL_PAYLOAD = "signal_payload";

/** serde_json::Value on the wire. Do not close these into catalogs. */
export const VALUE_FIELDS = Object.freeze([
  SIGNAL_PAYLOAD,
  ...SETTLEMENT_VALUE_FIELDS,
]);

export const QUERY_PARAMS = Object.freeze(["limit", "offset", "employee_id"]);

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

const UUID_REQUIRED = Object.freeze(["id", "employee_id"]);
const UUID_OPTIONAL = Object.freeze(["branch_id", "exit_case_id"]);
const STRING_REQUIRED = Object.freeze([
  "employee_name",
  "company",
  "work_date",
  "source",
  "status",
  "severity",
  "notification_title",
  "notification_message",
  "link_href",
]);
const STRING_OPTIONAL = Object.freeze([
  "employee_number",
  "org_unit",
  "worksite_name",
  "branch_name",
]);

const FORBIDDEN_ENVELOPE = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  DRAFT_RECORD,
  CATALOG_ENTRY,
  OVERRIDE_SUMMARY,
  HR_READINESS_SUMMARY,
  EXIT_CASE_RESPONSE,
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

function jsonOkSchema(operation, code) {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function jsonOkHeaders(operation, code) {
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

function parameterNames(operation) {
  const parameters = own(operation, "parameters");
  if (!Array.isArray(parameters)) return [];
  return parameters
    .map((parameter) => own(parameter, "name"))
    .filter((name) => typeof name === "string");
}

function arrayItemName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function enumValues(schema) {
  if (!isPlainObject(schema)) return null;
  const listed = own(schema, "enum");
  return Array.isArray(listed) ? listed : null;
}

function isRootBag(schema) {
  if (!isPlainObject(schema)) return true;
  if (schemaRefName(schema)) return false;
  if (own(schema, "type") === "array") {
    return isRootBag(own(schema, "items"));
  }
  return own(schema, "additionalProperties") === true
    || schemaPropertyNames(schema).length === 0;
}

function isClosedCatalog(schema) {
  if (!isPlainObject(schema)) return false;
  if (typeof own(schema, "$ref") === "string") return true;
  if (Array.isArray(own(schema, "enum")) && own(schema, "enum").length > 0) return true;
  if (own(schema, "additionalProperties") === false) return true;
  const properties = own(schema, "properties");
  if (isPlainObject(properties) && Object.keys(properties).length > 0) return true;
  const required = own(schema, "required");
  if (Array.isArray(required) && required.length > 0) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (Array.isArray(members) && members.length > 0) return true;
  }
  if (own(schema, "type") === "array") {
    const items = own(schema, "items");
    if (isClosedCatalog(items)) return true;
    if (isPlainObject(items) && own(items, "type") && own(items, "type") !== "object") {
      return true;
    }
  }
  return false;
}

function isUnconstrainedJsonObject(schema) {
  if (!isPlainObject(schema)) return false;
  if (isClosedCatalog(schema)) return false;
  if (own(schema, "type") !== "object") return false;
  if (own(schema, "additionalProperties") !== true) return false;
  return true;
}

function isUnconstrainedString(schema) {
  return isPlainObject(schema) && own(schema, "type") === "string" && !enumValues(schema);
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

function requireRequiredList(schemas, findings, name, fields) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) return;
  const required = own(schema, "required");
  if (!Array.isArray(required) || required.join("\0") !== fields.join("\0")) {
    push(
      findings,
      `#/components/schemas/${name}/required`,
      `${name} required must match always-serialized handler fields `
        + `(${fields.join(", ")})`,
    );
  }
}

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  if (objectName === DASHBOARD_RESPONSE || itemName === DASHBOARD_RESPONSE) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${DASHBOARD_RESPONSE}`,
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
      `cannot read existing ${structName} fields; this slice binds the existing type`,
    );
  } else if (read.join("\0") !== fields.join("\0")) {
    push(
      findings,
      `${rel}:${structName}`,
      "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
    );
  }
}

function requireUnconstrainedValue(findings, schemas, schemaName, field) {
  const listed = own(own(own(schemas, schemaName), "properties"), field);
  if (isClosedCatalog(listed) || !isUnconstrainedJsonObject(listed)) {
    push(
      findings,
      `#/components/schemas/${schemaName}/properties/${field}`,
      `${field} is serde_json::Value; leave it unconstrained object / `
        + "additionalProperties: true on that field only — do not invent a "
        + "Korea statutory catalog",
    );
  }
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   writes: number,
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiAbsenceExitDashboard({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let writes = 0;
  let gets = 0;
  let bound = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { writes: 0, gets: 0, bound: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      if (method === "get") gets += 1;
      if (WRITE_METHODS.has(method)) writes += 1;
    }
  }

  requireSchemaFields(schemas, findings, DASHBOARD_RESPONSE, DASHBOARD_FIELDS);
  requireSchemaFields(schemas, findings, SUMMARY, SUMMARY_FIELDS);
  requireSchemaFields(schemas, findings, ALERT_RESPONSE, ALERT_FIELDS);
  requireRequiredList(schemas, findings, DASHBOARD_RESPONSE, DASHBOARD_REQUIRED);
  requireRequiredList(schemas, findings, SUMMARY, SUMMARY_FIELDS);
  requireRequiredList(schemas, findings, ALERT_RESPONSE, ALERT_REQUIRED);

  const envelope = own(schemas, DASHBOARD_RESPONSE);
  if (isPlainObject(envelope)) {
    if (own(envelope, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${DASHBOARD_RESPONSE}`,
        `${DASHBOARD_RESPONSE} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only, not the whole 200 bag`,
      );
    }
    const properties = own(envelope, "properties");
    if (schemaRefName(own(properties, "summary")) !== SUMMARY) {
      push(
        findings,
        `#/components/schemas/${DASHBOARD_RESPONSE}/properties/summary`,
        `summary is ${SUMMARY}; $ref the existing nested struct`,
      );
    }
    if (arrayItemName(own(properties, "alerts")) !== ALERT_RESPONSE) {
      push(
        findings,
        `#/components/schemas/${DASHBOARD_RESPONSE}/properties/alerts`,
        `alerts is Vec<${ALERT_RESPONSE}>; array items must $ref the existing nested struct`,
      );
    }
    if (arrayItemName(own(properties, "exit_cases")) !== EXIT_CASE_RESPONSE) {
      push(
        findings,
        `#/components/schemas/${DASHBOARD_RESPONSE}/properties/exit_cases`,
        `exit_cases is Vec<${EXIT_CASE_RESPONSE}> already published in #1035; `
          + `$ref that envelope rather than duplicating or inventing a catalog`,
      );
    }
  }

  const summary = own(schemas, SUMMARY);
  if (isPlainObject(summary)) {
    if (own(summary, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${SUMMARY}`,
        `${SUMMARY} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only`,
      );
    }
    const properties = own(summary, "properties");
    for (const field of SUMMARY_FIELDS) {
      const listed = own(properties, field);
      if (!isPlainObject(listed) || own(listed, "type") !== "integer" || enumValues(listed)) {
        push(
          findings,
          `#/components/schemas/${SUMMARY}/properties/${field}`,
          `${field} is i64 on the wire; type integer and do not invent a catalog`,
        );
      }
    }
  }

  const alert = own(schemas, ALERT_RESPONSE);
  if (isPlainObject(alert)) {
    if (own(alert, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${ALERT_RESPONSE}`,
        `${ALERT_RESPONSE} envelope must stay typed; additionalProperties: true belongs `
          + `on ${SIGNAL_PAYLOAD} only, not the whole alert`,
      );
    }
    const properties = own(alert, "properties");
    for (const field of UUID_REQUIRED) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${ALERT_RESPONSE}/properties/${field}`,
          `${field} must $ref existing Uuid`,
        );
      }
    }
    for (const field of UUID_OPTIONAL) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${ALERT_RESPONSE}/properties/${field}`,
          `${field} is skip_serializing Option<Uuid>; $ref existing Uuid and omit from required — `
            + "do not invent a catalog",
        );
      }
    }
    for (const field of [...STRING_REQUIRED, ...STRING_OPTIONAL]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${ALERT_RESPONSE}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    const roles = own(properties, "audience_roles");
    const roleItems = isPlainObject(roles) ? own(roles, "items") : null;
    if (
      !isPlainObject(roles)
      || own(roles, "type") !== "array"
      || !isUnconstrainedString(roleItems)
    ) {
      push(
        findings,
        `#/components/schemas/${ALERT_RESPONSE}/properties/audience_roles`,
        "audience_roles is Vec<String>; unconstrained string items — do not invent a role catalog",
      );
    }
    requireUnconstrainedValue(findings, schemas, ALERT_RESPONSE, SIGNAL_PAYLOAD);
    if (schemaRefName(own(properties, "detected_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${ALERT_RESPONSE}/properties/detected_at`,
        "detected_at must $ref existing Timestamp (rfc3339 OffsetDateTime)",
      );
    }
  }

  for (const field of SETTLEMENT_VALUE_FIELDS) {
    requireUnconstrainedValue(findings, schemas, SETTLEMENT_PACKAGE, field);
  }

  requireRustFields(findings, repoRoot, HR_RS_REL, DASHBOARD_RESPONSE, DASHBOARD_FIELDS);
  requireRustFields(findings, repoRoot, HR_RS_REL, SUMMARY, SUMMARY_FIELDS);
  requireRustFields(findings, repoRoot, HR_RS_REL, ALERT_RESPONSE, ALERT_FIELDS);

  const location = `#/paths/${ABSENCE_EXIT_GET_PATH}/get`;
  const operation = findOperation(paths, ABSENCE_EXIT_GET_PATH, "get");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `GET ${ABSENCE_EXIT_GET_PATH} must remain published `
        + `(runtime already serves ${DASHBOARD_RESPONSE})`,
    );
  } else {
    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto GET ${ABSENCE_EXIT_GET_PATH}`,
      );
    }

    const listedParams = parameterNames(operation);
    if (listedParams.join("\0") !== QUERY_PARAMS.join("\0")) {
      push(
        findings,
        `${location}/parameters`,
        `GET ${ABSENCE_EXIT_GET_PATH} already publishes ${QUERY_PARAMS.join(", ")}; `
          + "do not invent as_of, version, or other query params",
      );
    }

    const headers = jsonOkHeaders(operation, "200");
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${location}/responses/200/headers/ETag`,
        "HR absence-exit-dashboard handler does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
      );
    }

    const body = jsonOkSchema(operation, "200");
    const boundName = schemaRefName(body);
    const responseLoc = `${location}/responses/200`;
    if (FORBIDDEN_ENVELOPE.includes(boundName)) {
      push(
        findings,
        responseLoc,
        `GET ${ABSENCE_EXIT_GET_PATH} already returns ${DASHBOARD_RESPONSE}; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / DraftRecord is a policy draft / CatalogEntry is the enforced catalog / "
          + "OverrideSummary is a governance override / readiness is a different HR envelope / "
          + `${EXIT_CASE_RESPONSE} is the nested exit_cases item, not the dashboard envelope)`,
      );
    } else if (boundName === DASHBOARD_RESPONSE) {
      bound += 1;
    } else if (isRootBag(body)) {
      push(
        findings,
        responseLoc,
        `GET ${ABSENCE_EXIT_GET_PATH} already returns ${DASHBOARD_RESPONSE}; `
          + `200 must $ref ${DASHBOARD_RESPONSE}, not a root additionalProperties bag`,
      );
    } else {
      push(
        findings,
        responseLoc,
        `GET ${ABSENCE_EXIT_GET_PATH} already returns ${DASHBOARD_RESPONSE}; `
          + `200 must $ref ${DASHBOARD_RESPONSE}, not additionalProperties`,
      );
    }
  }

  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_GET_PATH, "get"), "200"),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${KILL_SWITCH_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, KILL_SWITCH_PATH, "post"), "200"),
    `POST ${KILL_SWITCH_PATH} is console kill-switch, not an HR dashboard`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_OPT_IN_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_OPT_IN_PATH, "put"), "200"),
    `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not an HR dashboard`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_ORG_FLAG_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_ORG_FLAG_PATH, "put"), "200"),
    `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not an HR dashboard`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DRAFTS_PATH, "post"), "201"),
    `POST ${DRAFTS_PATH} already returns DraftRecord, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${CATALOG_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, CATALOG_GET_PATH, "get"), "200"),
    `GET ${CATALOG_GET_PATH} already returns CatalogEntry, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OVERRIDES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, OVERRIDES_PATH, "post"), "201"),
    `POST ${OVERRIDES_PATH} already returns OverrideSummary, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${READINESS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, READINESS_GET_PATH, "get"), "200"),
    `GET ${READINESS_GET_PATH} already returns HrReadinessSummary, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXIT_CASES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, EXIT_CASES_PATH, "post"), "201"),
    `POST ${EXIT_CASES_PATH} already returns ${EXIT_CASE_RESPONSE}, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not ${DASHBOARD_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${EXECUTE_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, EXECUTE_PATH, "post"), "200"),
    `POST ${EXECUTE_PATH} already returns ExecuteOutcome (projected: serde_json::Value)`,
  );

  return { writes, gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiAbsenceExitDashboard({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { writes, gets, bound, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowWriteFloor = writes < WRITE_FLOOR;
  const belowGetFloor = gets < GET_FLOOR;
  if (belowWriteFloor) {
    console.error(
      `saw ${writes} write operations — below the floor ${WRITE_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowWriteFloor || belowGetFloor || bound !== BOUND) {
    console.error(
      `openapi absence-exit-dashboard typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), ${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi absence-exit-dashboard typed-response gate passed `
      + `(${DASHBOARD_RESPONSE} $ref; ${VALUE_FIELDS.join(" / ")} unconstrained; `
      + `${writes} write operations, ${gets} GET operations, 0 findings)`,
  );
}
