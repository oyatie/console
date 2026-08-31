// GET /api/v1/hr/readiness-summary 200 HrReadinessSummary bind gate.
//
// The hole this closes: get_hr_readiness_summary already returns
// Json<HrReadinessSummary> whose nested counters are typed i64 / String /
// Option<OffsetDateTime> (no serde_json::Value). Composed OpenAPI still
// advertises payroll.active_close_runs plus additionalProperties: true, so
// clients cannot see the existing imports / annual_leave / attendance wires.
// Same class as #1022 AuditRecord $ref — derive the schema from the existing
// DTO fields. Do not invent product fields. Do not invent AbsenceExitDashboard.
//
// Chesterton: face YAML from the existing structs, then $ref. Do not bind
// GET /api/v1/hr/absence-exit-dashboard (nested Value bags, no schema). Do not
// map Feature::ALL permissions. Do not stamp HTTP ETag (handler does not send
// it). Do not invent query params (handler has no Query).
//
// Totality: js-yaml load + own-property walk of every GET + optional Rust
// struct field read. GET_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const READINESS_GET_PATH = "/api/v1/hr/readiness-summary";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const HR_READINESS_SUMMARY = "HrReadinessSummary";
export const HR_IMPORT_SUMMARY = "HrImportReadinessSummary";
export const HR_PAYROLL_SUMMARY = "HrPayrollReadinessSummary";
export const HR_ANNUAL_LEAVE_SUMMARY = "HrAnnualLeaveReadinessSummary";
export const HR_ATTENDANCE_SUMMARY = "HrAttendanceReadinessSummary";
export const BOUND = 1;

export const HR_RS_REL = "backend/app/src/hr.rs";

/** Existing Serialize fields. Do not invent names. */
export const HR_READINESS_FIELDS = Object.freeze([
  "imports",
  "payroll",
  "annual_leave",
  "attendance",
]);
export const HR_IMPORT_FIELDS = Object.freeze([
  "runs",
  "applied_runs",
  "input_rows",
  "candidate_rows",
  "preserved_rows",
  "ledger_rows",
  "latest_import_at",
]);
export const HR_PAYROLL_FIELDS = Object.freeze([
  "draft_runs",
  "blocked_runs",
  "calculation_enabled_runs",
  "active_close_runs",
  "draft_lines",
  "payroll_source_rows",
  "attendance_source_rows",
  "attendance_event_links",
  "attendance_material_refs",
  "gross_pay_source_lines",
  "net_pay_source_lines",
  "latest_status",
  "latest_source_label",
  "latest_period_start",
  "latest_period_end",
  "latest_updated_at",
]);
export const HR_ANNUAL_LEAVE_FIELDS = Object.freeze([
  "obligations",
  "usage_promotion_required",
  "payout_review_required",
  "needs_review",
  "remaining_days",
]);
export const HR_ATTENDANCE_FIELDS = Object.freeze([
  "durable_events",
  "self_service_records",
  "payroll_material_refs",
]);

export const NESTED_REFS = Object.freeze({
  imports: HR_IMPORT_SUMMARY,
  payroll: HR_PAYROLL_SUMMARY,
  annual_leave: HR_ANNUAL_LEAVE_SUMMARY,
  attendance: HR_ATTENDANCE_SUMMARY,
});

const STRUCT_FIELDS = Object.freeze([
  Object.freeze({ schema: HR_READINESS_SUMMARY, structName: HR_READINESS_SUMMARY, fields: HR_READINESS_FIELDS }),
  Object.freeze({ schema: HR_IMPORT_SUMMARY, structName: HR_IMPORT_SUMMARY, fields: HR_IMPORT_FIELDS }),
  Object.freeze({ schema: HR_PAYROLL_SUMMARY, structName: HR_PAYROLL_SUMMARY, fields: HR_PAYROLL_FIELDS }),
  Object.freeze({ schema: HR_ANNUAL_LEAVE_SUMMARY, structName: HR_ANNUAL_LEAVE_SUMMARY, fields: HR_ANNUAL_LEAVE_FIELDS }),
  Object.freeze({ schema: HR_ATTENDANCE_SUMMARY, structName: HR_ATTENDANCE_SUMMARY, fields: HR_ATTENDANCE_FIELDS }),
]);

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

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
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

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHrReadinessSummary({ repoRoot }) {
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

  for (const entry of STRUCT_FIELDS) {
    requireSchemaFields(schemas, findings, entry.schema, entry.fields);
  }

  const parent = own(schemas, HR_READINESS_SUMMARY);
  if (isPlainObject(parent)) {
    const properties = own(parent, "properties");
    for (const [field, schemaName] of Object.entries(NESTED_REFS)) {
      const name = schemaRefName(own(properties, field));
      if (name !== schemaName) {
        push(
          findings,
          `#/components/schemas/${HR_READINESS_SUMMARY}/properties/${field}`,
          `${field} must $ref existing ${schemaName} (the nested handler struct)`,
        );
      }
    }
  }

  const payroll = own(schemas, HR_PAYROLL_SUMMARY);
  if (isPlainObject(payroll)) {
    const active = own(own(payroll, "properties"), "active_close_runs");
    if (!isPlainObject(active) || own(active, "type") !== "integer") {
      push(
        findings,
        `#/components/schemas/${HR_PAYROLL_SUMMARY}/properties/active_close_runs`,
        "active_close_runs is already published as int64; do not drop or retag it",
      );
    }
  }

  const rustPath = join(repoRoot, HR_RS_REL);
  if (existsSync(rustPath)) {
    const source = readFileSync(rustPath, "utf8");
    for (const entry of STRUCT_FIELDS) {
      const fields = rustStructFields(source, entry.structName);
      if (!Array.isArray(fields) || fields.length === 0) {
        push(
          findings,
          `${HR_RS_REL}:${entry.structName}`,
          `cannot read existing ${entry.structName} fields; this slice binds the handler type`,
        );
      } else if (fields.join("\0") !== entry.fields.join("\0")) {
        push(
          findings,
          `${HR_RS_REL}:${entry.structName}`,
          "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
        );
      }
    }
  }

  const location = `#/paths/${READINESS_GET_PATH}/get`;
  const operation = findGet(paths, READINESS_GET_PATH);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${READINESS_GET_PATH} must remain published (runtime already serves it)`);
    return { gets, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto GET /api/v1/hr/readiness-summary "
        + "(EmployeeDirectoryRead stays the REST gate, unpublished here)",
    );
  }

  if (parameterList(operation).length > 0) {
    push(
      findings,
      `${location}/parameters`,
      "get_hr_readiness_summary has no Query; do not invent as_of, pagination, or filter params",
    );
  }

  const headers = json200Headers(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "get_hr_readiness_summary does not send ETag; HTTP ETag stays HOLD — do not stamp it on this GET",
    );
  }

  if (schemaRefName(json200Schema(operation)) === HR_READINESS_SUMMARY) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `GET ${READINESS_GET_PATH} already returns Json<${HR_READINESS_SUMMARY}>; `
        + `200 must $ref ${HR_READINESS_SUMMARY}, not additionalProperties`,
    );
  }

  const absence = findGet(paths, ABSENCE_EXIT_GET_PATH);
  if (isPlainObject(absence)) {
    const name = schemaRefName(json200Schema(absence));
    if (name === HR_READINESS_SUMMARY) {
      push(
        findings,
        `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
        `GET ${ABSENCE_EXIT_GET_PATH} already returns AbsenceExitDashboardResponse `
          + `(nested serde_json::Value); do not bind it to ${HR_READINESS_SUMMARY}`,
      );
    }
  }

  const detail = findGet(paths, OBJECT_TYPE_GET_PATH);
  if (isPlainObject(detail) && schemaRefName(json200Schema(detail)) === HR_READINESS_SUMMARY) {
    push(
      findings,
      `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
      `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail; do not bind it to ${HR_READINESS_SUMMARY}`,
    );
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHrReadinessSummary({ repoRoot });
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
      `openapi hr-readiness-summary typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi hr-readiness-summary typed-response gate passed `
      + `(${HR_READINESS_SUMMARY} $ref; ${gets} GET operations, 0 findings)`,
  );
}
