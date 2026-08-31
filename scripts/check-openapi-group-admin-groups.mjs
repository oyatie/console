// GET /api/v1/group-admin/groups 200 GroupAdminGroupsResponse bind gate.
//
// The hole this closes: list_group_admin_groups already returns
// Json<GroupAdminGroupsResponse> whose nested rows are Uuid / String
// (no serde_json::Value). Composed OpenAPI still advertises
// additionalProperties: true, so clients cannot see the existing wire.
// Same class as #1025 CatalogEntry $ref — derive the schema from the
// existing DTO fields. Do not invent ObjectKey::Group. Do not reuse
// PlatformGroup (member_count / created_at / PlatformOrgStatus).
//
// Chesterton: face YAML from the existing structs, then $ref. Do not
// publish this GET as a Head. Do not map Feature::ALL permissions. Do
// not stamp HTTP ETag. Handler has no Query. status stays unconstrained
// String (SQL TEXT), not PlatformOrgStatus.
//
// Totality: js-yaml load + own-property walk of every GET + optional Rust
// struct field read. GET_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import { rustStructFields } from "./check-openapi-audit-record.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const GET_FLOOR = ASOF_GET_FLOOR;

export const GROUPS_GET_PATH = "/api/v1/group-admin/groups";
export const DRAFTS_GET_PATH = "/api/v1/policy/drafts";
export const OBJECT_TYPE_GET_PATH = "/api/v1/ontology/object-types/{key}";
export const ABSENCE_EXIT_GET_PATH = "/api/v1/hr/absence-exit-dashboard";

export const GROUPS_RESPONSE = "GroupAdminGroupsResponse";
export const GROUP_RESPONSE = "GroupAdminGroupResponse";
export const MEMBER_RESPONSE = "GroupAdminMemberOrgResponse";
export const PLATFORM_GROUP = "PlatformGroup";
export const CATALOG_ENTRY = "CatalogEntry";
export const BOUND = 1;

export const GROUPS_RS_REL = "backend/crates/platform/auth-rest/src/lib.rs";

export const GROUPS_FIELDS = Object.freeze(["groups"]);
export const GROUP_FIELDS = Object.freeze(["id", "slug", "name", "status", "members"]);
export const MEMBER_FIELDS = Object.freeze(["id", "slug", "name", "status"]);
export const STATUS_STRING_FIELDS = Object.freeze(["status"]);

const FORBIDDEN_200 = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  PLATFORM_GROUP,
  CATALOG_ENTRY,
]);

const STRUCT_FIELDS = Object.freeze([
  Object.freeze({ schema: GROUPS_RESPONSE, structName: GROUPS_RESPONSE, fields: GROUPS_FIELDS }),
  Object.freeze({ schema: GROUP_RESPONSE, structName: GROUP_RESPONSE, fields: GROUP_FIELDS }),
  Object.freeze({ schema: MEMBER_RESPONSE, structName: MEMBER_RESPONSE, fields: MEMBER_FIELDS }),
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

function arrayItemName(schema) {
  if (!isPlainObject(schema) || own(schema, "type") !== "array") return null;
  return schemaRefName(own(schema, "items"));
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

function requireStatusString(schemas, findings, name) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) return;
  const status = own(own(schema, "properties"), "status");
  if (schemaRefName(status) === "PlatformOrgStatus") {
    push(
      findings,
      `#/components/schemas/${name}/properties/status`,
      `${name}.status is handler String / SQL TEXT; do not reuse PlatformOrgStatus`,
    );
    return;
  }
  if (isCatalogSchema(status) || (isPlainObject(status) && own(status, "type") !== "string")) {
    push(
      findings,
      `#/components/schemas/${name}/properties/status`,
      `${name}.status is unconstrained String; do not type it as an enum / catalog`,
    );
  }
}

function refuseForeignBind(findings, path, schema, label) {
  const objectName = schemaRefName(schema);
  const itemName = arrayItemName(schema);
  for (const name of [GROUPS_RESPONSE, GROUP_RESPONSE, MEMBER_RESPONSE]) {
    if (objectName === name || itemName === name) {
      push(
        findings,
        `#/paths/${path}/get/responses/200`,
        `${label}; do not bind it to ${name}`,
      );
    }
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
export function evaluateOpenapiGroupAdminGroups({ repoRoot }) {
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
  requireStatusString(schemas, findings, GROUP_RESPONSE);
  requireStatusString(schemas, findings, MEMBER_RESPONSE);

  const parent = own(schemas, GROUPS_RESPONSE);
  if (isPlainObject(parent)) {
    const groups = own(own(parent, "properties"), "groups");
    if (arrayItemName(groups) !== GROUP_RESPONSE) {
      push(
        findings,
        `#/components/schemas/${GROUPS_RESPONSE}/properties/groups`,
        `groups must be an array of $ref ${GROUP_RESPONSE} (the nested handler struct)`,
      );
    }
  }

  const group = own(schemas, GROUP_RESPONSE);
  if (isPlainObject(group)) {
    const members = own(own(group, "properties"), "members");
    if (arrayItemName(members) !== MEMBER_RESPONSE) {
      push(
        findings,
        `#/components/schemas/${GROUP_RESPONSE}/properties/members`,
        `members must be an array of $ref ${MEMBER_RESPONSE} (the nested handler struct)`,
      );
    }
  }

  const rustPath = join(repoRoot, GROUPS_RS_REL);
  if (existsSync(rustPath)) {
    const source = readFileSync(rustPath, "utf8");
    for (const entry of STRUCT_FIELDS) {
      const fields = rustStructFields(source, entry.structName);
      if (!Array.isArray(fields) || fields.length === 0) {
        push(
          findings,
          `${GROUPS_RS_REL}:${entry.structName}`,
          `cannot read existing ${entry.structName} fields; this slice binds the handler type`,
        );
      } else if (fields.join("\0") !== entry.fields.join("\0")) {
        push(
          findings,
          `${GROUPS_RS_REL}:${entry.structName}`,
          "frozen OAS field list drifted from the handler struct; do not invent or drop wire fields",
        );
      }
    }
  }

  const location = `#/paths/${GROUPS_GET_PATH}/get`;
  const operation = findGet(paths, GROUPS_GET_PATH);
  if (!isPlainObject(operation)) {
    push(findings, location, `GET ${GROUPS_GET_PATH} must remain published (runtime already serves it)`);
    return { gets, bound, findings };
  }

  if (hasPermissions(operation)) {
    push(
      findings,
      `${location}/permissions`,
      "operation-level permissions are admitted only on Head GET/list; "
        + "do not map Feature::ALL onto GET /api/v1/group-admin/groups "
        + "(GROUP_ADMIN stays the runtime gate, unpublished here)",
    );
  }

  if (parameterList(operation).length > 0) {
    push(
      findings,
      `${location}/parameters`,
      "list_group_admin_groups has no Query; do not invent as_of, pagination, or filter params",
    );
  }

  const headers = json200Headers(operation);
  if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
    push(
      findings,
      `${location}/responses/200/headers/ETag`,
      "list_group_admin_groups does not send ETag; HTTP ETag stays HOLD — do not stamp it on this GET",
    );
  }

  const body = json200Schema(operation);
  const boundName = schemaRefName(body);
  if (FORBIDDEN_200.includes(boundName)) {
    push(
      findings,
      `${location}/responses/200`,
      `GET ${GROUPS_GET_PATH} is not an ontology Head (Group ObjectKey HOLD); `
        + `do not bind it to ${boundName}`,
    );
  } else if (boundName === GROUPS_RESPONSE) {
    bound += 1;
  } else {
    push(
      findings,
      `${location}/responses/200`,
      `GET ${GROUPS_GET_PATH} already returns Json<${GROUPS_RESPONSE}>; `
        + `200 must $ref ${GROUPS_RESPONSE}, not additionalProperties`,
    );
  }

  refuseForeignBind(
    findings,
    DRAFTS_GET_PATH,
    json200Schema(findGet(paths, DRAFTS_GET_PATH)),
    `GET ${DRAFTS_GET_PATH} already returns DraftRecord (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    OBJECT_TYPE_GET_PATH,
    json200Schema(findGet(paths, OBJECT_TYPE_GET_PATH)),
    `GET ${OBJECT_TYPE_GET_PATH} already returns ObjectTypeDetail (nested serde_json::Value)`,
  );
  refuseForeignBind(
    findings,
    ABSENCE_EXIT_GET_PATH,
    json200Schema(findGet(paths, ABSENCE_EXIT_GET_PATH)),
    `GET ${ABSENCE_EXIT_GET_PATH} already returns nested serde_json::Value bags`,
  );

  return { gets, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiGroupAdminGroups({ repoRoot });
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
      `openapi group-admin-groups typed-response gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi group-admin-groups typed-response gate passed `
      + `(${GROUPS_RESPONSE} $ref; ${gets} GET operations, 0 findings)`,
  );
}
