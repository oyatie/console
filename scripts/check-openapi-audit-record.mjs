// GET /api/audit 200 AuditRecord bind gate.
//
// The hole this closes: audit_log already returns Json<AuditPage> whose items
// are the typed AuditRecord row the handler selects. Composed OpenAPI still
// advertises items as additionalProperties: true, so clients cannot see the
// existing wire fields. Same class as #1020 InstanceState $ref — bind the
// runtime type. Do not invent fields. Do not type action as an enum (store is
// unconstrained TEXT). Do not publish an action query / AsyncAPI catalog.
//
// Chesterton: face YAML schema derived from the existing struct fields, then
// $ref. Do not reuse AuditStreamRecord / PolicyAuditEventResponse (different
// columns). Do not map Feature::ALL permissions. Do not add AuditPage as a
// second named wrapper — the page object already exists inline.
//
// Totality: js-yaml load + own-property walk of every GET + optional Rust
// struct field read. GET_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const AUDIT_GET_PATH = "/api/audit";
export const AUDIT_RECORD = "AuditRecord";
export const ACTION = "action";
export const BOUND = 1;

export const AUDIT_RECORD_RS_REL = "backend/app/src/lib.rs";
export const AUDIT_RECORD_STRUCT = "AuditRecord";

/** Existing Serialize fields on AuditRecord. Do not invent names. */
export const AUDIT_RECORD_FIELDS = Object.freeze([
  "id",
  "actor",
  "action",
  "target_type",
  "target_id",
  "branch_id",
  "before_snap",
  "after_snap",
  "ip",
  "user_agent",
  "auth_method",
  "device",
  "classification_badges",
  "anomaly",
  "reason",
  "trace_id",
  "span_id",
  "occurred_at",
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

const PAGE_REQUIRED = Object.freeze(["items", "limit", "offset"]);

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

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

function hasPermissions(operation) {
  const listed = own(operation, "permissions");
  return Array.isArray(listed) && listed.length > 0;
}

function isCatalogSchema(schema) {
  if (!isPlainObject(schema)) return false;
  if (Array.isArray(own(schema, "enum")) && own(schema, "enum").length > 0) return true;
  for (const key of ["oneOf", "anyOf", "allOf"]) {
    const members = own(schema, key);
    if (Array.isArray(members) && members.length > 0) {
      if (members.some((member) => isCatalogSchema(member))) return true;
    }
  }
  return false;
}

function pageItemsRef(schema) {
  if (!isPlainObject(schema)) return null;
  const properties = own(schema, "properties");
  const items = own(properties, "items");
  if (!isPlainObject(items) || own(items, "type") !== "array") return null;
  return schemaRefName(own(items, "items"));
}

export function rustStructFields(source, structName) {
  const escaped = structName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(
    new RegExp(String.raw`(?:pub\s+)?struct ${escaped}\s*\{([\s\S]*?)\n\}`),
  );
  if (!match) return null;
  const fields = [];
  const fieldRe =
    /^\s*(?:#\[[^\]]+\]\s*)*(?:pub(?:\s*\([^)]+\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:/gm;
  let fieldMatch;
  while ((fieldMatch = fieldRe.exec(match[1])) !== null) {
    fields.push(fieldMatch[1]);
  }
  return fields;
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiAuditRecord({ repoRoot }) {
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

  const record = own(schemas, AUDIT_RECORD);
  if (!isPlainObject(record)) {
    push(
      findings,
      `#/components/schemas/${AUDIT_RECORD}`,
      `${AUDIT_RECORD} must be derived from the existing handler struct — do not leave items additionalProperties`,
    );
  } else {
    const names = schemaPropertyNames(record);
    const missing = AUDIT_RECORD_FIELDS.filter((name) => !names.includes(name));
    const extra = names.filter((name) => !AUDIT_RECORD_FIELDS.includes(name));
    if (missing.length > 0 || extra.length > 0) {
      push(
        findings,
        `#/components/schemas/${AUDIT_RECORD}/properties`,
        `${AUDIT_RECORD} properties must match the existing Rust fields `
          + `(missing ${missing.join(",") || "none"}; extra ${extra.join(",") || "none"}). `
          + "Do not invent fields",
      );
    }
    const action = own(own(record, "properties"), ACTION);
    if (isCatalogSchema(action) || (isPlainObject(action) && own(action, "type") !== "string")) {
      push(
        findings,
        `#/components/schemas/${AUDIT_RECORD}/properties/${ACTION}`,
        "audit_events.action is unconstrained TEXT; do not type action as an enum / catalog",
      );
    }
    const occurred = own(own(record, "properties"), "occurred_at");
    if (schemaRefName(occurred) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${AUDIT_RECORD}/properties/occurred_at`,
        "occurred_at is OffsetDateTime; $ref Timestamp like sibling audit schemas, do not invent a clock",
      );
    }
  }

  const rustPath = join(repoRoot, AUDIT_RECORD_RS_REL);
  if (existsSync(rustPath)) {
    const fields = rustStructFields(readFileSync(rustPath, "utf8"), AUDIT_RECORD_STRUCT);
    if (!Array.isArray(fields) || fields.length === 0) {
      push(
        findings,
        `${AUDIT_RECORD_RS_REL}:${AUDIT_RECORD_STRUCT}`,
        "cannot read existing AuditRecord fields; this slice binds the handler type",
      );
    } else if (fields.join("\0") !== AUDIT_RECORD_FIELDS.join("\0")) {
      push(
        findings,
        `${AUDIT_RECORD_RS_REL}:${AUDIT_RECORD_STRUCT}`,
        "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
      );
    }
  }

  const location = `#/paths/${AUDIT_GET_PATH}/get`;
  const operation = findGet(paths, AUDIT_GET_PATH);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${AUDIT_GET_PATH} must remain published (runtime already serves it)`);
    return { gets, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto GET /api/audit (AuditLogRead stays the runtime gate, unpublished here)",
    );
  }

  const page = json200Schema(operation);
  const required = own(page, "required");
  const requiredList = Array.isArray(required) ? required.filter((name) => typeof name === "string") : [];
  for (const name of PAGE_REQUIRED) {
    if (!requiredList.includes(name)) {
      push(
        findings,
        `${location}/responses/200/schema/required/${name}`,
        `GET ${AUDIT_GET_PATH} already returns AuditPage.{${PAGE_REQUIRED.join(", ")}}; do not drop ${name}`,
      );
    }
  }

  if (pageItemsRef(page) === AUDIT_RECORD) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `GET ${AUDIT_GET_PATH} already returns Json<AuditPage> of typed ${AUDIT_RECORD}; `
        + `items must $ref ${AUDIT_RECORD}, not additionalProperties`,
    );
  }

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiAuditRecord({ repoRoot });
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
      `openapi audit-record typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi audit-record typed-response gate passed `
      + `(${AUDIT_RECORD} items $ref; ${gets} GET operations, 0 findings)`,
  );
}
