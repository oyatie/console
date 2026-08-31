// POST /api/v1/group-admin/tenant-context 200 GroupAdminTenantContextStartResponse bind gate.
//
// The hole this closes: start_group_admin_tenant_context already returns
// Json<GroupAdminTenantContextStartResponse> whose fields are String / Uuid /
// OffsetDateTime (no serde_json::Value). Composed OpenAPI still advertises
// additionalProperties: true, so clients cannot see the existing wire. Same
// class as #1026 GroupAdminGroupsResponse — vendor-tier HTTP only. Do not
// invent ObjectKey::Group. Do not reuse PlatformTenantContextStartResponse
// (acting_role SUPER_ADMIN). Do not mix this POST into ontology Heads.
//
// Chesterton: face YAML from the existing struct, then $ref. Do not publish
// this POST as a Head. Do not map Feature::ALL permissions. Do not stamp
// HTTP ETag. Request body stays the existing inline org_id Uuid (already
// typed). Exit stays {ended: boolean}.
//
// Totality: js-yaml load + own-property walk of every write method + optional
// Rust struct field read. WRITE_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { WRITE_FLOOR as PREFLIGHT_WRITE_FLOOR } from "./check-openapi-preflight-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = PREFLIGHT_WRITE_FLOOR;

export const TENANT_CONTEXT_PATH = "/api/v1/group-admin/tenant-context";
export const TENANT_CONTEXT_EXIT_PATH = "/api/v1/group-admin/tenant-context/exit";
export const PLATFORM_TENANT_CONTEXT_PATH = "/api/platform/tenant-context";
export const LIFECYCLE_PREFLIGHT_PATH = "/api/v1/governance/lifecycle/preflight";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const START_RESPONSE = "GroupAdminTenantContextStartResponse";
export const PLATFORM_START_RESPONSE = "PlatformTenantContextStartResponse";
export const GROUPS_RESPONSE = "GroupAdminGroupsResponse";
export const LIFECYCLE_PREFLIGHT = "LifecyclePreflight";
export const ACTING_ROLE = "GROUP_ADMIN_DELEGATED_ADMIN";
export const PLATFORM_ACTING_ROLE = "SUPER_ADMIN";
export const TOKEN_TYPE = "Bearer";
export const BOUND = 1;

export const HANDLER_RS_REL = "backend/crates/platform/auth-rest/src/lib.rs";
export const HANDLER_STRUCT = "GroupAdminTenantContextStartResponse";

/** Existing Serialize fields on GroupAdminTenantContextStartResponse. Do not invent names. */
export const START_FIELDS = Object.freeze([
  "access_token",
  "token_type",
  "acting_org_id",
  "acting_org_name",
  "acting_role",
  "expires_at",
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

const FORBIDDEN_200 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  PLATFORM_START_RESPONSE,
  GROUPS_RESPONSE,
  LIFECYCLE_PREFLIGHT,
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

function jsonOkSchema(operation, code = "200") {
  const responses = own(operation, "responses");
  const ok = own(responses, code) ?? own(responses, Number(code));
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function jsonOkHeaders(operation, code = "200") {
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

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  if (objectName === START_RESPONSE || itemName === START_RESPONSE) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${START_RESPONSE}`,
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
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiGroupAdminTenantContext({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let writes = 0;
  let bound = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { writes: 0, bound: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      if (!WRITE_METHODS.has(method)) continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      writes += 1;
    }
  }

  requireSchemaFields(schemas, findings, START_RESPONSE, START_FIELDS);

  const envelope = own(schemas, START_RESPONSE);
  if (isPlainObject(envelope)) {
    const required = own(envelope, "required");
    if (!Array.isArray(required) || required.join("\0") !== START_FIELDS.join("\0")) {
      push(
        findings,
        `#/components/schemas/${START_RESPONSE}/required`,
        `${START_RESPONSE} required must match always-serialized handler fields `
          + `(${START_FIELDS.join(", ")})`,
      );
    }
    const properties = own(envelope, "properties");
    const actingRole = own(properties, "acting_role");
    const roleEnum = enumValues(actingRole);
    if (!roleEnum || roleEnum.join("\0") !== ACTING_ROLE) {
      push(
        findings,
        `#/components/schemas/${START_RESPONSE}/properties/acting_role`,
        `acting_role is the existing ${ACTING_ROLE} constant; do not reuse `
          + `${PLATFORM_ACTING_ROLE} / ${PLATFORM_START_RESPONSE} and do not invent a role catalog`,
      );
    }
    const tokenType = own(properties, "token_type");
    const tokenEnum = enumValues(tokenType);
    if (!tokenEnum || tokenEnum.join("\0") !== TOKEN_TYPE) {
      push(
        findings,
        `#/components/schemas/${START_RESPONSE}/properties/token_type`,
        `token_type is the existing ${TOKEN_TYPE} constant; do not invent token types`,
      );
    }
    if (schemaRefName(own(properties, "acting_org_id")) !== "Uuid") {
      push(
        findings,
        `#/components/schemas/${START_RESPONSE}/properties/acting_org_id`,
        "acting_org_id must $ref existing Uuid (handler Uuid)",
      );
    }
    if (schemaRefName(own(properties, "expires_at")) !== "Timestamp") {
      push(
        findings,
        `#/components/schemas/${START_RESPONSE}/properties/expires_at`,
        "expires_at must $ref existing Timestamp (handler OffsetDateTime RFC3339)",
      );
    }
  }

  requireRustFields(findings, repoRoot, HANDLER_RS_REL, HANDLER_STRUCT, START_FIELDS);

  const location = `#/paths/${TENANT_CONTEXT_PATH}/post`;
  const operation = findOperation(paths, TENANT_CONTEXT_PATH, "post");
  if (!isPlainObject(operation)) {
    push(
      findings,
      location,
      `POST ${TENANT_CONTEXT_PATH} must remain published (runtime already serves it)`,
    );
    return { writes, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + `do not map Feature::ALL onto POST ${TENANT_CONTEXT_PATH} `
        + "(GROUP_ADMIN stays the runtime gate, unpublished here)",
    );
  }

  const headers = jsonOkHeaders(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "start_group_admin_tenant_context does not send ETag; HTTP ETag stays HOLD — do not stamp it here",
    );
  }

  const body = jsonOkSchema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_200.includes(boundName)) {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${TENANT_CONTEXT_PATH} is vendor-tier HTTP (Group ObjectKey HOLD); `
        + `do not bind it to ${boundName}`,
    );
  } else if (boundName === START_RESPONSE) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `POST ${TENANT_CONTEXT_PATH} already returns Json<${START_RESPONSE}>; `
        + `200 must $ref ${START_RESPONSE}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    `#/paths/${PLATFORM_TENANT_CONTEXT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, PLATFORM_TENANT_CONTEXT_PATH, "post")),
    `POST ${PLATFORM_TENANT_CONTEXT_PATH} already returns ${PLATFORM_START_RESPONSE} `
      + `(${PLATFORM_ACTING_ROLE}), not ${START_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${TENANT_CONTEXT_EXIT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, TENANT_CONTEXT_EXIT_PATH, "post")),
    `POST ${TENANT_CONTEXT_EXIT_PATH} already returns {ended: boolean}, not ${START_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${LIFECYCLE_PREFLIGHT_PATH}/post/responses/200`,
    jsonOkSchema(findOperation(paths, LIFECYCLE_PREFLIGHT_PATH, "post")),
    `POST ${LIFECYCLE_PREFLIGHT_PATH} already returns ${LIFECYCLE_PREFLIGHT}, not ${START_RESPONSE}`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${DRAFTS_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, DRAFTS_GET_PATH, "get")),
    `GET ${DRAFTS_GET_PATH} already returns DraftRecord (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${OBJECT_TYPE_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, OBJECT_TYPE_GET_PATH, "get")),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    `#/paths/${ABSENCE_EXIT_GET_PATH}/get/responses/200`,
    jsonOkSchema(findOperation(paths, ABSENCE_EXIT_GET_PATH, "get")),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns nested serde_json::Value bags`,
  );

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiGroupAdminTenantContext({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { writes, bound, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowWriteFloor = writes < WRITE_FLOOR;
  if (belowWriteFloor) {
    console.error(
      `saw ${writes} write operations — below the floor ${WRITE_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowWriteFloor || bound !== BOUND) {
    console.error(
      `openapi group-admin-tenant-context typed-response gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi group-admin-tenant-context typed-response gate passed `
      + `(${START_RESPONSE} $ref; ${writes} write operations, 0 findings)`,
  );
}
