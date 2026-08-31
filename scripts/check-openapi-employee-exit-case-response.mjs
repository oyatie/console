// HR EmployeeExitCaseResponse envelope bind gate.
//
// The hole this closes: report / confirm / approval-draft already return
// Json<EmployeeExitCaseResponse> whose envelope fields are Uuid / String /
// OffsetDateTime / nested settlement + next-actions, with statutory_basis /
// insurance_loss_payload / approval_payload as serde_json::Value on the nested
// settlement package. Composed OpenAPI still advertises those 201/200 bodies as
// additionalProperties: true. Same class as #1034 DraftRecord — publish the
// existing record, do not invent a store, statutory catalog, or ObjectKey.
//
// Chesterton: face YAML from the existing structs, then $ref. Type every
// non-Value field. Value bags stay unconstrained object /
// additionalProperties: true on THOSE FIELDS ONLY. This is not Korea statutory
// product: do not interpret, close, or catalog those bags. Do not bind
// absence-exit-dashboard (different envelope, nested Value). Do not bind
// kill-switch / rollout. Do not map Feature::ALL. Do not stamp HTTP ETag.
//
// Totality: js-yaml load + own-property walk of every GET and write method +
// optional Rust struct field read. GET_FLOOR / WRITE_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
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

export const EXIT_CASES_PATH = "/api/v1/hr/exit-cases";
export const CONFIRM_PATH = "/api/v1/hr/exit-cases/{id}/confirm";
export const APPROVAL_DRAFT_PATH = "/api/v1/hr/exit-cases/{id}/approval-draft";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const KILL_SWITCH_PATH = "/api/v1/console/kill-switch";
export const ROLLOUT_OPT_IN_PATH = "/api/v1/console/rollout/opt-in";
export const ROLLOUT_ORG_FLAG_PATH = "/api/v1/console/rollout/org-flag";
export const DRAFTS_PATH = "/api/v1/policy/drafts";
export const CATALOG_GET_PATH = "/api/v1/policy/catalog";
export const OVERRIDES_PATH = "/api/v1/governance/overrides";
export const READINESS_GET_PATH = "/api/v1/hr/readiness-summary";

export const EXIT_CASE_RESPONSE = "EmployeeExitCaseResponse";
export const SETTLEMENT_PACKAGE = "EmployeeExitSettlementPackageResponse";
export const NEXT_ACTION = "ExitCaseNextAction";
export const DRAFT_RECORD = "DraftRecord";
export const CATALOG_ENTRY = "CatalogEntry";
export const OVERRIDE_SUMMARY = "OverrideSummary";
export const HR_READINESS_SUMMARY = "HrReadinessSummary";
export const BOUND = 3;

export const HR_RS_REL = "backend/app/src/hr.rs";

/** Existing Serialize fields. Do not invent names. */
export const RECORD_FIELDS = Object.freeze([
  "id",
  "employee_id",
  "employee_name",
  "employee_number",
  "company",
  "org_unit",
  "worksite_name",
  "branch_id",
  "branch_name",
  "absence_alert_id",
  "status",
  "effective_exit_date",
  "site_manager_note",
  "reported_by",
  "reported_at",
  "hr_confirmed_by",
  "hr_confirmed_at",
  "hq_confirmed_by",
  "hq_confirmed_at",
  "approval_submitted_by",
  "approval_submitted_at",
  "settlement_package",
  "next_actions",
]);

export const RECORD_REQUIRED = Object.freeze([
  "id",
  "employee_id",
  "employee_name",
  "company",
  "status",
  "effective_exit_date",
  "site_manager_note",
  "reported_by",
  "reported_at",
  "next_actions",
]);

export const SETTLEMENT_FIELDS = Object.freeze([
  "id",
  "status",
  "service_days",
  "average_wage_period_start",
  "average_wage_period_end",
  "average_wage_calendar_days",
  "average_wage_total_won",
  "average_daily_wage_milliwon",
  "severance_pay_won",
  "monthly_ordinary_wage_won",
  "ordinary_daily_wage_won",
  "statutory_daily_wage_milliwon",
  "missing_source_fields",
  "statutory_basis",
  "insurance_loss_payload",
  "approval_payload",
  "certification_status",
  "generated_at",
  "submitted_by",
  "submitted_at",
]);

export const SETTLEMENT_REQUIRED = Object.freeze([
  "id",
  "status",
  "missing_source_fields",
  "statutory_basis",
  "insurance_loss_payload",
  "approval_payload",
  "certification_status",
  "generated_at",
]);

export const NEXT_ACTION_FIELDS = Object.freeze(["key", "label", "href"]);

/** serde_json::Value on the wire. Do not close these into catalogs. */
export const VALUE_FIELDS = Object.freeze([
  "statutory_basis",
  "insurance_loss_payload",
  "approval_payload",
]);

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

const UUID_REQUIRED = Object.freeze(["id", "employee_id", "reported_by"]);
const UUID_OPTIONAL = Object.freeze([
  "branch_id",
  "absence_alert_id",
  "hr_confirmed_by",
  "hq_confirmed_by",
  "approval_submitted_by",
]);
const STRING_REQUIRED = Object.freeze([
  "employee_name",
  "company",
  "status",
  "effective_exit_date",
  "site_manager_note",
]);
const STRING_OPTIONAL = Object.freeze([
  "employee_number",
  "org_unit",
  "worksite_name",
  "branch_name",
]);
const TIMESTAMP_REQUIRED = Object.freeze(["reported_at"]);
const TIMESTAMP_OPTIONAL = Object.freeze([
  "hr_confirmed_at",
  "hq_confirmed_at",
  "approval_submitted_at",
]);

const FORBIDDEN_ENVELOPE = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  DRAFT_RECORD,
  CATALOG_ENTRY,
  OVERRIDE_SUMMARY,
  HR_READINESS_SUMMARY,
]);

export const BINDINGS = Object.freeze([
  Object.freeze({
    path: EXIT_CASES_PATH,
    method: "post",
    code: "201",
  }),
  Object.freeze({
    path: CONFIRM_PATH,
    method: "post",
    code: "200",
  }),
  Object.freeze({
    path: APPROVAL_DRAFT_PATH,
    method: "post",
    code: "200",
  }),
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
  if (objectName === EXIT_CASE_RESPONSE || itemName === EXIT_CASE_RESPONSE) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${EXIT_CASE_RESPONSE}`,
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

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   writes: number,
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiEmployeeExitCaseResponse({ repoRoot }) {
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

  requireSchemaFields(schemas, findings, EXIT_CASE_RESPONSE, RECORD_FIELDS);
  requireSchemaFields(schemas, findings, SETTLEMENT_PACKAGE, SETTLEMENT_FIELDS);
  requireSchemaFields(schemas, findings, NEXT_ACTION, NEXT_ACTION_FIELDS);
  requireRequiredList(schemas, findings, EXIT_CASE_RESPONSE, RECORD_REQUIRED);
  requireRequiredList(schemas, findings, SETTLEMENT_PACKAGE, SETTLEMENT_REQUIRED);
  requireRequiredList(schemas, findings, NEXT_ACTION, NEXT_ACTION_FIELDS);

  const envelope = own(schemas, EXIT_CASE_RESPONSE);
  if (isPlainObject(envelope)) {
    if (own(envelope, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${EXIT_CASE_RESPONSE}`,
        `${EXIT_CASE_RESPONSE} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only, not the whole 201/200 bag`,
      );
    }
    const properties = own(envelope, "properties");
    for (const field of UUID_REQUIRED) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/${field}`,
          `${field} must $ref existing Uuid`,
        );
      }
    }
    for (const field of UUID_OPTIONAL) {
      if (schemaRefName(own(properties, field)) !== "Uuid") {
        push(
          findings,
          `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/${field}`,
          `${field} is skip_serializing Option<Uuid>; $ref existing Uuid and omit from required — `
            + "do not invent a catalog",
        );
      }
    }
    for (const field of [...STRING_REQUIRED, ...STRING_OPTIONAL]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    for (const field of TIMESTAMP_REQUIRED) {
      if (schemaRefName(own(properties, field)) !== "Timestamp") {
        push(
          findings,
          `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/${field}`,
          `${field} must $ref existing Timestamp (rfc3339 OffsetDateTime)`,
        );
      }
    }
    for (const field of TIMESTAMP_OPTIONAL) {
      if (schemaRefName(own(properties, field)) !== "Timestamp") {
        push(
          findings,
          `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/${field}`,
          `${field} is skip_serializing Option<OffsetDateTime>; $ref existing Timestamp and omit `
            + "from required",
        );
      }
    }
    if (schemaRefName(own(properties, "settlement_package")) !== SETTLEMENT_PACKAGE) {
      push(
        findings,
        `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/settlement_package`,
        `settlement_package is Option<${SETTLEMENT_PACKAGE}>; $ref the existing nested struct — `
          + "do not leave it as a Value bag or invent a catalog",
      );
    }
    if (arrayItemName(own(properties, "next_actions")) !== NEXT_ACTION) {
      push(
        findings,
        `#/components/schemas/${EXIT_CASE_RESPONSE}/properties/next_actions`,
        `next_actions is Vec<${NEXT_ACTION}>; array items must $ref the existing nested struct`,
      );
    }
  }

  const settlement = own(schemas, SETTLEMENT_PACKAGE);
  if (isPlainObject(settlement)) {
    if (own(settlement, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${SETTLEMENT_PACKAGE}`,
        `${SETTLEMENT_PACKAGE} envelope must stay typed; additionalProperties: true belongs `
          + `on ${VALUE_FIELDS.join(" / ")} only`,
      );
    }
    const properties = own(settlement, "properties");
    if (schemaRefName(own(properties, "id")) !== "Uuid") {
      push(
        findings,
        `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/id`,
        "id must $ref existing Uuid",
      );
    }
    for (const field of ["status", "certification_status"]) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a catalog`,
        );
      }
    }
    for (const field of VALUE_FIELDS) {
      const listed = own(properties, field);
      if (isClosedCatalog(listed) || !isUnconstrainedJsonObject(listed)) {
        push(
          findings,
          `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/${field}`,
          `${field} is serde_json::Value; leave it unconstrained object / `
            + "additionalProperties: true on that field only — do not invent a "
            + "Korea statutory catalog",
        );
      }
    }
    if (schemaRefName(own(properties, "generated_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/generated_at`,
        "generated_at must $ref existing Timestamp (rfc3339 OffsetDateTime)",
      );
    }
    if (schemaRefName(own(properties, "submitted_by")) !== "Uuid") {
      push(
        findings,
        `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/submitted_by`,
        "submitted_by is skip_serializing Option<Uuid>; $ref existing Uuid",
      );
    }
    if (schemaRefName(own(properties, "submitted_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${SETTLEMENT_PACKAGE}/properties/submitted_at`,
        "submitted_at is skip_serializing Option<OffsetDateTime>; $ref existing Timestamp",
      );
    }
  }

  const nextAction = own(schemas, NEXT_ACTION);
  if (isPlainObject(nextAction)) {
    const properties = own(nextAction, "properties");
    for (const field of NEXT_ACTION_FIELDS) {
      if (!isUnconstrainedString(own(properties, field))) {
        push(
          findings,
          `#/components/schemas/${NEXT_ACTION}/properties/${field}`,
          `${field} is unconstrained TEXT on the wire; do not invent a next-action catalog`,
        );
      }
    }
  }

  requireRustFields(findings, repoRoot, HR_RS_REL, EXIT_CASE_RESPONSE, RECORD_FIELDS);
  requireRustFields(findings, repoRoot, HR_RS_REL, SETTLEMENT_PACKAGE, SETTLEMENT_FIELDS);
  requireRustFields(findings, repoRoot, HR_RS_REL, NEXT_ACTION, NEXT_ACTION_FIELDS);

  for (const binding of BINDINGS) {
    const location = `#/paths/${binding.path}/${binding.method}`;
    const operation = findOperation(paths, binding.path, binding.method);
    if (!isPlainObject(operation)) {
      push(
        findings,
        location,
        `${binding.method.toUpperCase()} ${binding.path} must remain published `
          + `(runtime already serves ${EXIT_CASE_RESPONSE})`,
      );
      continue;
    }

    if (hasPermissions(operation)) {
      push(
        findings,
        `${location}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto ${binding.method.toUpperCase()} ${binding.path}`,
      );
    }

    const headers = jsonOkHeaders(operation, binding.code);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `${location}/responses/${binding.code}/headers/ETag`,
        "HR exit-case handlers do not send ETag; HTTP ETag stays HOLD — do not stamp it here",
      );
    }

    const body = jsonOkSchema(operation, binding.code);
    const boundName = schemaRefName(body);
    const responseLoc = `${location}/responses/${binding.code}`;
    if (FORBIDDEN_ENVELOPE.includes(boundName)) {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns ${EXIT_CASE_RESPONSE}; `
          + `do not bind it to ${boundName} `
          + "(Head HOLD / DraftRecord is a policy draft / CatalogEntry is the enforced catalog / "
          + "OverrideSummary is a governance override / readiness is a different HR envelope)",
      );
    } else if (boundName === EXIT_CASE_RESPONSE) {
      bound += 1;
    } else if (isRootBag(body)) {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns ${EXIT_CASE_RESPONSE}; `
          + `${binding.code} must $ref ${EXIT_CASE_RESPONSE}, not a root additionalProperties bag`,
      );
    } else {
      push(
        findings,
        responseLoc,
        `${binding.method.toUpperCase()} ${binding.path} already returns ${EXIT_CASE_RESPONSE}; `
          + `${binding.code} must $ref ${EXIT_CASE_RESPONSE}, not additionalProperties`,
      );
    }
  }

  refuseForeignBind(
    findings,
    `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, ABSENCE_EXIT_GET_PATH, "get"), "200"),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns a dashboard envelope with nested Value bags, `
      + `not a bare ${EXIT_CASE_RESPONSE}`,
  );
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
    `POST ${KILL_SWITCH_PATH} is console kill-switch, not an HR exit case`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_OPT_IN_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_OPT_IN_PATH, "put"), "200"),
    `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not an HR exit case`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ROLLOUT_ORG_FLAG_PATH}/put/responses/200`,
    jsonOkSchema(findOperation(paths, ROLLOUT_ORG_FLAG_PATH, "put"), "200"),
    `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not an HR exit case`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, DRAFTS_PATH, "post"), "201"),
    `POST ${DRAFTS_PATH} already returns DraftRecord, not ${EXIT_CASE_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${CATALOG_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, CATALOG_GET_PATH, "get"), "200"),
    `GET ${CATALOG_GET_PATH} already returns CatalogEntry, not ${EXIT_CASE_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OVERRIDES_PATH}/post/responses/201`,
    jsonOkSchema(findOperation(paths, OVERRIDES_PATH, "post"), "201"),
    `POST ${OVERRIDES_PATH} already returns OverrideSummary, not ${EXIT_CASE_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${READINESS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, READINESS_GET_PATH, "get"), "200"),
    `GET ${READINESS_GET_PATH} already returns HrReadinessSummary, not ${EXIT_CASE_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PREFLIGHT_PATH, "post"), "200"),
    `POST ${PREFLIGHT_PATH} already returns PreflightOutcome, not ${EXIT_CASE_RESPONSE}`,
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
    result = evaluateOpenapiEmployeeExitCaseResponse({ repoRoot });
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
      `openapi employee-exit-case-response typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), ${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi employee-exit-case-response typed-response gate passed `
      + `(${EXIT_CASE_RESPONSE} $ref; ${VALUE_FIELDS.join(" / ")} unconstrained; `
      + `${writes} write operations, ${gets} GET operations, 0 findings)`,
  );
}
