// GateChainOutcome.gates[] items GateOutcome bind gate.
//
// The hole this closes: evaluate_gate_chain already serializes Vec<GateOutcome>
// (GateKind snake_case + internally tagged GateStatus). Named envelopes already
// $ref GateChainOutcome, but items stay additionalProperties: true bags with a
// comment `{ gate, status }`. Publishing the existing closed Rust types is the
// #1017 / #1027 class. Do not invent extra GateKind / GateStatus variants.
// Do not close an open enum (Unknown(String) / untagged / free-form TEXT).
//
// Chesterton: bind items $ref GateOutcome. GateKind / GateStatus schemas must
// match serde. Probe fail-closed if OAS grows variants the Rust enum does not
// have. Do not bind kill-switch / rollout. Do not type AuditRecord.action as
// GateKind (store is unconstrained TEXT). Do not map Feature::ALL. Do not stamp
// HTTP ETag.
//
// Totality: js-yaml load + own-property walk of every write method + optional
// Rust enum/struct read. WRITE_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  ACTION as AUDIT_ACTION_FIELD,
  AUDIT_GET_PATH,
  AUDIT_RECORD,
  rustStructFields,
} from "./check-openapi-audit-record.mjs";
import {
  EXECUTE_PATH,
  KILL_SWITCH_PATH,
  LIFECYCLE_PREFLIGHT_PATH,
  PREFLIGHT_PATH,
  ROLLOUT_OPT_IN_PATH,
  ROLLOUT_ORG_FLAG_PATH,
  WRITE_FLOOR as EXECUTE_WRITE_FLOOR,
} from "./check-openapi-execute-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = EXECUTE_WRITE_FLOOR;

export const GATE_CHAIN_OUTCOME = "GateChainOutcome";
export const GATE_OUTCOME = "GateOutcome";
export const GATE_KIND = "GateKind";
export const GATE_STATUS = "GateStatus";
export const BOUND = 1;

export const GATE_RS_REL = "backend/crates/governance/domain/src/lib.rs";
export const GATE_OUTCOME_STRUCT = "GateOutcome";
export const GATE_KIND_ENUM = "GateKind";
export const GATE_STATUS_ENUM = "GateStatus";

/** Existing Serialize fields on GateOutcome. Do not invent names. */
export const GATE_OUTCOME_FIELDS = Object.freeze(["gate", "status"]);

/**
 * serde rename_all = "snake_case" of the closed GateKind enum
 * (Authority / SelfChecklist / FourEyes / EgressDlp). Not a free-form catalog.
 */
export const GATE_KIND_VARIANTS = Object.freeze([
  "authority",
  "self_checklist",
  "four_eyes",
  "egress_dlp",
]);

/**
 * serde tag = "status", rename_all = "snake_case" of the closed GateStatus enum.
 */
export const GATE_STATUS_VARIANTS = Object.freeze([
  "not_required",
  "satisfied",
  "pending",
  "denied",
]);

export const GATE_STATUS_UNIT_VARIANTS = Object.freeze([
  "not_required",
  "satisfied",
]);

export const GATE_STATUS_REASON_VARIANTS = Object.freeze(["pending", "denied"]);

export const GATE_STATUS_TAG = "status";

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

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function enumValues(schema) {
  if (!isPlainObject(schema)) return null;
  const listed = own(schema, "enum");
  return Array.isArray(listed) ? listed.map(String) : null;
}

function sameStringList(left, right) {
  return Array.isArray(left)
    && Array.isArray(right)
    && left.join("\0") === right.join("\0");
}

export function toSnakeCase(ident) {
  return ident
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .toLowerCase();
}

export function snakeToPascal(snake) {
  return String(snake)
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

export function gateStatusVariantSchemaName(wire) {
  return `${GATE_STATUS}${snakeToPascal(wire)}`;
}

/**
 * Read a closed Rust enum's serde snake_case variants.
 * Returns null when the enum is missing. `open` is true for Unknown(String),
 * serde untagged, or #[serde(other)] — those must not be closed in OAS.
 *
 * @param {string} source
 * @param {string} enumName
 * @returns {{
 *   open: boolean,
 *   variants: string[],
 *   tag: string | null,
 * } | null}
 */
export function rustEnumInfo(source, enumName) {
  const escaped = enumName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(
    new RegExp(
      String.raw`((?:#\[[^\]]+\]\s*)*)(?:pub\s+)?enum ${escaped}\s*\{([\s\S]*?)\n\}`,
    ),
  );
  if (!match) return null;
  const attrs = match[1];
  const body = match[2]
    .replace(/\/\/[^\n]*/g, "")
    .replace(/\/\*[\s\S]*?\*\//g, "");
  const tagMatch = attrs.match(/tag\s*=\s*"([^"]+)"/);
  const open = /Unknown\s*\(/.test(body)
    || /untagged/.test(attrs)
    || /serde\([^)]*\bother\b/.test(attrs)
    || /\b[A-Z][A-Za-z0-9]*\s*\(\s*String\s*\)/.test(body);
  const variants = [];
  const variantRe = /^\s*([A-Z][A-Za-z0-9]*)\b/gm;
  let variantMatch;
  while ((variantMatch = variantRe.exec(body)) !== null) {
    variants.push(toSnakeCase(variantMatch[1]));
  }
  return {
    open,
    variants,
    tag: tagMatch ? tagMatch[1] : null,
  };
}

function requireSchemaFields(schemas, findings, name, fields) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) {
    push(
      findings,
      `#/components/schemas/${name}`,
      `${name} must be derived from the existing Rust type — do not leave additionalProperties`,
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
      `${name} required must match always-serialized Rust fields (${fields.join(", ")})`,
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
      "frozen OAS field list drifted from the Rust struct; do not invent or drop wire fields",
    );
  }
}

function requireClosedRustEnum(findings, repoRoot, enumName, expected, expectedTag) {
  const rustPath = join(repoRoot, GATE_RS_REL);
  if (!existsSync(rustPath)) return;
  const info = rustEnumInfo(readFileSync(rustPath, "utf8"), enumName);
  if (!info) {
    push(
      findings,
      `${GATE_RS_REL}:${enumName}`,
      `cannot read existing ${enumName}; this slice publishes the existing closed enum`,
    );
    return;
  }
  if (info.open) {
    push(
      findings,
      `${GATE_RS_REL}:${enumName}`,
      `${enumName} is open (Unknown / untagged / String tuple); do not close it into an invented catalog`,
    );
    return;
  }
  if (!sameStringList(info.variants, expected)) {
    push(
      findings,
      `${GATE_RS_REL}:${enumName}`,
      `frozen ${enumName} wire list drifted from the Rust enum `
        + `(rust ${info.variants.join(",") || "none"}; frozen ${expected.join(",")}). `
        + "Do not invent variants",
    );
  }
  if (expectedTag !== undefined && info.tag !== expectedTag) {
    push(
      findings,
      `${GATE_RS_REL}:${enumName}`,
      `${enumName} serde tag must stay ${expectedTag === null ? "absent (externally tagged string enum)" : JSON.stringify(expectedTag)}; `
        + "do not invent a discriminator",
    );
  }
}

function requireExactStringEnum(schemas, findings, name, variants) {
  const schema = own(schemas, name);
  if (!isPlainObject(schema)) {
    push(
      findings,
      `#/components/schemas/${name}`,
      `${name} must be the existing closed Rust enum on the wire — do not leave it unpublished`,
    );
    return;
  }
  if (own(schema, "type") !== "string") {
    push(
      findings,
      `#/components/schemas/${name}`,
      `${name} is a serde snake_case string enum; do not publish a different shape`,
    );
  }
  const listed = enumValues(schema);
  if (!sameStringList(listed, variants)) {
    const extra = (listed ?? []).filter((value) => !variants.includes(value));
    const missing = variants.filter((value) => !(listed ?? []).includes(value));
    push(
      findings,
      `#/components/schemas/${name}/enum`,
      `${name} must match the existing Rust enum exactly `
        + `(missing ${missing.join(",") || "none"}; extra ${extra.join(",") || "none"}). `
        + "Extra variants are an invented catalog — fail closed",
    );
  }
}

function requireGateStatus(schemas, findings) {
  const schema = own(schemas, GATE_STATUS);
  if (!isPlainObject(schema)) {
    push(
      findings,
      `#/components/schemas/${GATE_STATUS}`,
      `${GATE_STATUS} must be the existing internally tagged Rust enum — do not leave it unpublished`,
    );
    return;
  }
  const oneOf = own(schema, "oneOf");
  if (!Array.isArray(oneOf) || oneOf.length !== GATE_STATUS_VARIANTS.length) {
    push(
      findings,
      `#/components/schemas/${GATE_STATUS}/oneOf`,
      `${GATE_STATUS} must be a ${GATE_STATUS_VARIANTS.length}-member oneOf matching serde(tag = "status")`,
    );
  }
  const discriminator = own(schema, "discriminator");
  const propertyName = isPlainObject(discriminator) ? own(discriminator, "propertyName") : null;
  const mapping = isPlainObject(discriminator) ? own(discriminator, "mapping") : null;
  if (propertyName !== GATE_STATUS_TAG || !isPlainObject(mapping)) {
    push(
      findings,
      `#/components/schemas/${GATE_STATUS}/discriminator`,
      `${GATE_STATUS} must use discriminator propertyName ${GATE_STATUS_TAG} matching serde(tag = "status")`,
    );
    return;
  }
  const mappingKeys = Object.keys(mapping);
  const extraKeys = mappingKeys.filter((key) => !GATE_STATUS_VARIANTS.includes(key));
  const missingKeys = GATE_STATUS_VARIANTS.filter((key) => !mappingKeys.includes(key));
  if (extraKeys.length > 0 || missingKeys.length > 0) {
    push(
      findings,
      `#/components/schemas/${GATE_STATUS}/discriminator/mapping`,
      `${GATE_STATUS} mapping must match the existing Rust enum exactly `
        + `(missing ${missingKeys.join(",") || "none"}; extra ${extraKeys.join(",") || "none"}). `
        + "Extra variants are an invented catalog — fail closed",
    );
  }
  const expectedRefs = GATE_STATUS_VARIANTS.map(
    (wire) => `#/components/schemas/${gateStatusVariantSchemaName(wire)}`,
  );
  const oneOfRefs = Array.isArray(oneOf)
    ? oneOf.map((member) => (isPlainObject(member) ? own(member, "$ref") : null))
    : [];
  if (!sameStringList(oneOfRefs, expectedRefs)) {
    push(
      findings,
      `#/components/schemas/${GATE_STATUS}/oneOf`,
      `${GATE_STATUS} oneOf $refs must be the existing serde variants in Rust declaration order`,
    );
  }
  for (const wire of GATE_STATUS_VARIANTS) {
    const variantName = gateStatusVariantSchemaName(wire);
    const mapped = own(mapping, wire);
    if (mapped !== `#/components/schemas/${variantName}`) {
      push(
        findings,
        `#/components/schemas/${GATE_STATUS}/discriminator/mapping/${wire}`,
        `${wire} must map to existing ${variantName} — do not invent a variant schema`,
      );
    }
    const variant = own(schemas, variantName);
    if (!isPlainObject(variant)) {
      push(
        findings,
        `#/components/schemas/${variantName}`,
        `${variantName} must exist as the serde internally tagged variant`,
      );
      continue;
    }
    const status = own(own(variant, "properties"), GATE_STATUS_TAG);
    const statusEnum = enumValues(status);
    if (
      !isPlainObject(status)
      || own(status, "type") !== "string"
      || !sameStringList(statusEnum, [wire])
    ) {
      push(
        findings,
        `#/components/schemas/${variantName}/properties/${GATE_STATUS_TAG}`,
        `${variantName}.status must be the singleton enum [${wire}] matching serde`,
      );
    }
    const names = schemaPropertyNames(variant);
    const extraProps = names.filter((name) => name !== GATE_STATUS_TAG && name !== "reason");
    if (extraProps.length > 0) {
      push(
        findings,
        `#/components/schemas/${variantName}/properties`,
        `${variantName} extra properties ${extraProps.join(",")} are an invented catalog`,
      );
    }
    if (GATE_STATUS_REASON_VARIANTS.includes(wire)) {
      const reason = own(own(variant, "properties"), "reason");
      if (!isPlainObject(reason) || own(reason, "type") !== "string" || enumValues(reason)) {
        push(
          findings,
          `#/components/schemas/${variantName}/properties/reason`,
          `${variantName}.reason is unconstrained String on the wire; do not invent a catalog`,
        );
      }
    } else if (names.includes("reason")) {
      push(
        findings,
        `#/components/schemas/${variantName}/properties/reason`,
        `${variantName} is a unit variant; do not invent a reason field`,
      );
    }
  }
}

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  if (
    objectName === GATE_OUTCOME
    || objectName === GATE_KIND
    || objectName === GATE_STATUS
  ) {
    push(
      findings,
      location,
      `${label}; do not bind it to ${objectName}`,
    );
  }
}

function itemsSchema(arraySchema) {
  if (!isPlainObject(arraySchema) || own(arraySchema, "type") !== "array") return null;
  return own(arraySchema, "items");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   writes: number,
 *   bound: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiGateOutcome({ repoRoot }) {
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

  requireClosedRustEnum(findings, repoRoot, GATE_KIND_ENUM, GATE_KIND_VARIANTS, null);
  requireClosedRustEnum(
    findings,
    repoRoot,
    GATE_STATUS_ENUM,
    GATE_STATUS_VARIANTS,
    GATE_STATUS_TAG,
  );
  requireRustFields(
    findings,
    repoRoot,
    GATE_RS_REL,
    GATE_OUTCOME_STRUCT,
    GATE_OUTCOME_FIELDS,
  );

  const chain = own(schemas, GATE_CHAIN_OUTCOME);
  if (!isPlainObject(chain)) {
    push(
      findings,
      `#/components/schemas/${GATE_CHAIN_OUTCOME}`,
      `${GATE_CHAIN_OUTCOME} must remain published; this slice types its gates[] items`,
    );
  } else {
    const gates = own(own(chain, "properties"), "gates");
    const items = itemsSchema(gates);
    const itemName = schemaRefName(items);
    if (itemName === GATE_OUTCOME) {
      bound += 1;
    } else {
      push(
        findings,
        `#/components/schemas/${GATE_CHAIN_OUTCOME}/properties/gates/items`,
        `${GATE_CHAIN_OUTCOME}.gates items are Vec<GateOutcome> already on the wire; `
          + `$ref ${GATE_OUTCOME} — do not leave additionalProperties bags or invent a catalog`,
      );
    }
  }

  requireSchemaFields(schemas, findings, GATE_OUTCOME, GATE_OUTCOME_FIELDS);
  requireRequiredList(schemas, findings, GATE_OUTCOME, GATE_OUTCOME_FIELDS);
  const outcome = own(schemas, GATE_OUTCOME);
  if (isPlainObject(outcome)) {
    if (own(outcome, "additionalProperties") === true) {
      push(
        findings,
        `#/components/schemas/${GATE_OUTCOME}`,
        `${GATE_OUTCOME} envelope must stay typed; additionalProperties: true is the unpublished bag`,
      );
    }
    const properties = own(outcome, "properties");
    if (schemaRefName(own(properties, "gate")) !== GATE_KIND) {
      push(
        findings,
        `#/components/schemas/${GATE_OUTCOME}/properties/gate`,
        `gate is closed GateKind; $ref ${GATE_KIND} — do not leave an unconstrained string`,
      );
    }
    if (schemaRefName(own(properties, "status")) !== GATE_STATUS) {
      push(
        findings,
        `#/components/schemas/${GATE_OUTCOME}/properties/status`,
        `status is internally tagged GateStatus; $ref ${GATE_STATUS} matching serde(tag = "status")`,
      );
    }
  }

  requireExactStringEnum(schemas, findings, GATE_KIND, GATE_KIND_VARIANTS);
  requireGateStatus(schemas, findings);

  const audit = own(schemas, AUDIT_RECORD);
  if (isPlainObject(audit)) {
    const action = own(own(audit, "properties"), AUDIT_ACTION_FIELD);
    const actionName = schemaRefName(action);
    if (
      actionName === GATE_KIND
      || actionName === GATE_STATUS
      || (isPlainObject(action) && Array.isArray(own(action, "enum")))
    ) {
      push(
        findings,
        `#/components/schemas/${AUDIT_RECORD}/properties/${AUDIT_ACTION_FIELD}`,
        "audit action is unconstrained TEXT; do not bind it to GateKind or invent an action catalog",
      );
    }
  }

  for (const [path, method, code, label] of [
    [KILL_SWITCH_PATH, "post", "200", `POST ${KILL_SWITCH_PATH} is console kill-switch, not a gate outcome`],
    [ROLLOUT_OPT_IN_PATH, "put", "200", `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not a gate outcome`],
    [ROLLOUT_ORG_FLAG_PATH, "put", "200", `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not a gate outcome`],
    [AUDIT_GET_PATH, "get", "200", `GET ${AUDIT_GET_PATH} already returns AuditPage, not GateOutcome`],
  ]) {
    refuseForeignBind(
      findings,
      `#/paths/${path}/${method}/responses/${code}`,
      jsonOkSchema(findOperation(paths, path, method), code),
      label,
    );
  }

  for (const path of [PREFLIGHT_PATH, EXECUTE_PATH, LIFECYCLE_PREFLIGHT_PATH]) {
    const operation = findOperation(paths, path, "post");
    if (!isPlainObject(operation)) continue;
    if (hasPermissions(operation)) {
      push(
        findings,
        `#/paths/${path}/post/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto POST ${path}`,
      );
    }
    const headers = jsonOkHeaders(operation);
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `#/paths/${path}/post/responses/200/headers/ETag`,
        "HTTP ETag stays HOLD — do not stamp it on a gate-chain envelope",
      );
    }
    const boundName = schemaRefName(jsonOkSchema(operation));
    if (HEAD_SCHEMA_NAMES.includes(boundName)) {
      push(
        findings,
        `#/paths/${path}/post/responses/200`,
        `POST ${path} is not a Head; do not bind it to ${boundName}`,
      );
    }
  }

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiGateOutcome({ repoRoot });
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
      `openapi gate-outcome typed-items gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi gate-outcome typed-items gate passed `
      + `(${GATE_CHAIN_OUTCOME}.gates[] $ref ${GATE_OUTCOME}; `
      + `${GATE_KIND} / ${GATE_STATUS} match serde; `
      + `${writes} write operations, 0 findings)`,
  );
}
