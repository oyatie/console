// PolicyNoCodeCondition.value ConditionValue bind gate.
//
// The hole this closes: create/update policy-draft already deserialize
// NoCodeBlocks.conditions[].value as internally tagged ConditionValue
// (serde tag = "kind", content = "value": literal / subject_attr / bool).
// Named envelopes already $ref PolicyNoCodeCondition, but value stays an
// additionalProperties: true bag with a comment. Publishing the existing
// closed Rust type is the #1039 GateStatus class. Do not invent extra kinds
// (regex, number, …). Do not close an open enum (Unknown(String) / untagged).
// Do not enum PolicyNoCodeBlocks.action (field is String; AUTHORING_ACTIONS is
// not a serde enum). Do not enum attr. Do not bind AuditRecord.action TEXT.
//
// Chesterton: bind value $ref ConditionValue. Variant schemas must match
// serde tag+content. Probe fail-closed if OAS grows kinds the Rust enum does
// not have. Literal(String) is tagged content payload, not FieldKind::Unknown.
// Do not bind kill-switch / rollout. Do not map Feature::ALL. Do not stamp
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
import { DRAFTS_PATH, DRAFT_ID_PATH } from "./check-openapi-draft-record.mjs";
import {
  EXECUTE_PATH,
  KILL_SWITCH_PATH,
  PREFLIGHT_PATH,
  ROLLOUT_OPT_IN_PATH,
  ROLLOUT_ORG_FLAG_PATH,
  WRITE_FLOOR as EXECUTE_WRITE_FLOOR,
} from "./check-openapi-execute-outcome.mjs";
import {
  rustEnumInfo,
  snakeToPascal,
  toSnakeCase,
} from "./check-openapi-gate-outcome.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const WRITE_FLOOR = EXECUTE_WRITE_FLOOR;

export const POLICY_NO_CODE_CONDITION = "PolicyNoCodeCondition";
export const POLICY_NO_CODE_BLOCKS = "PolicyNoCodeBlocks";
export const CONDITION_VALUE = "ConditionValue";
export const CONDITION_OP = "ConditionOp";
export const EFFECT = "Effect";
export const BOUND = 1;

export const AUTHORING_RS_REL =
  "backend/crates/platform/authz/src/cedar_pbac/authoring.rs";
export const CONDITION_STRUCT = "Condition";
export const CONDITION_VALUE_ENUM = "ConditionValue";
export const CONDITION_OP_ENUM = "ConditionOp";
export const EFFECT_ENUM = "Effect";
export const NO_CODE_BLOCKS_STRUCT = "NoCodeBlocks";

/** Existing Serialize fields on Condition. Do not invent names. */
export const CONDITION_FIELDS = Object.freeze(["attr", "op", "value"]);

/**
 * serde tag = "kind", content = "value", rename_all = "snake_case"
 * of the closed ConditionValue enum (Literal / SubjectAttr / Bool).
 * Not a free-form catalog. String tuple payloads are tagged content.
 */
export const CONDITION_VALUE_VARIANTS = Object.freeze([
  "literal",
  "subject_attr",
  "bool",
]);

export const CONDITION_VALUE_STRING_VARIANTS = Object.freeze([
  "literal",
  "subject_attr",
]);

export const CONDITION_VALUE_BOOL_VARIANTS = Object.freeze(["bool"]);

export const CONDITION_VALUE_TAG = "kind";
export const CONDITION_VALUE_CONTENT = "value";

/** Already-published ConditionOp serde names. Extra ops are an invented catalog. */
export const CONDITION_OP_VARIANTS = Object.freeze(["eq", "ne", "contains"]);

/** Already-published Effect serde names. */
export const EFFECT_VARIANTS = Object.freeze(["permit", "forbid"]);

/** Rust String fields — unconstrained TEXT, not an invented catalog. */
export const BLOCKS_STRING_FIELDS = Object.freeze(["action", "resource_type"]);

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
  return (
    Array.isArray(left) &&
    Array.isArray(right) &&
    left.join("\0") === right.join("\0")
  );
}

export function conditionValueVariantSchemaName(wire) {
  return `${CONDITION_VALUE}${snakeToPascal(wire)}`;
}

/**
 * Read a closed internally tagged + content Rust enum.
 * String tuple payloads (`Literal(String)`) are tagged content, not
 * FieldKind::Unknown. `open` is true for Unknown / untagged / serde(other)
 * or missing tag/content — those must not be closed in OAS.
 *
 * @param {string} source
 * @param {string} enumName
 * @returns {{
 *   open: boolean,
 *   variants: string[],
 *   tag: string | null,
 *   content: string | null,
 * } | null}
 */
export function rustTaggedContentEnumInfo(source, enumName) {
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
  const contentMatch = attrs.match(/content\s*=\s*"([^"]+)"/);
  const open =
    /Unknown\s*\(/.test(body) ||
    /untagged/.test(attrs) ||
    /serde\([^)]*\bother\b/.test(attrs) ||
    !tagMatch ||
    !contentMatch;
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
    content: contentMatch ? contentMatch[1] : null,
  };
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

function requireClosedUnitEnum(findings, repoRoot, enumName, expected, expectedTag) {
  const rustPath = join(repoRoot, AUTHORING_RS_REL);
  if (!existsSync(rustPath)) return;
  const info = rustEnumInfo(readFileSync(rustPath, "utf8"), enumName);
  if (!info) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${enumName}`,
      `cannot read existing ${enumName}; this slice publishes the existing closed enum`,
    );
    return;
  }
  if (info.open) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${enumName}`,
      `${enumName} is open (Unknown / untagged / String tuple); do not close it into an invented catalog`,
    );
    return;
  }
  if (!sameStringList(info.variants, expected)) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${enumName}`,
      `frozen ${enumName} wire list drifted from the Rust enum `
        + `(rust ${info.variants.join(",") || "none"}; frozen ${expected.join(",")}). `
        + "Do not invent variants",
    );
  }
  if (expectedTag !== undefined && info.tag !== expectedTag) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${enumName}`,
      `${enumName} serde tag must stay ${expectedTag === null ? "absent (externally tagged string enum)" : JSON.stringify(expectedTag)}; `
        + "do not invent a discriminator",
    );
  }
}

function requireClosedTaggedContentEnum(findings, repoRoot) {
  const rustPath = join(repoRoot, AUTHORING_RS_REL);
  if (!existsSync(rustPath)) return;
  const info = rustTaggedContentEnumInfo(
    readFileSync(rustPath, "utf8"),
    CONDITION_VALUE_ENUM,
  );
  if (!info) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${CONDITION_VALUE_ENUM}`,
      `cannot read existing ${CONDITION_VALUE_ENUM}; this slice publishes the existing closed enum`,
    );
    return;
  }
  if (info.open) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${CONDITION_VALUE_ENUM}`,
      `${CONDITION_VALUE_ENUM} is open (Unknown / untagged / missing tag+content); do not close it into an invented catalog`,
    );
    return;
  }
  if (!sameStringList(info.variants, CONDITION_VALUE_VARIANTS)) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${CONDITION_VALUE_ENUM}`,
      `frozen ${CONDITION_VALUE_ENUM} wire list drifted from the Rust enum `
        + `(rust ${info.variants.join(",") || "none"}; frozen ${CONDITION_VALUE_VARIANTS.join(",")}). `
        + "Do not invent variants",
    );
  }
  if (info.tag !== CONDITION_VALUE_TAG) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${CONDITION_VALUE_ENUM}`,
      `${CONDITION_VALUE_ENUM} serde tag must stay ${JSON.stringify(CONDITION_VALUE_TAG)}; do not invent a discriminator`,
    );
  }
  if (info.content !== CONDITION_VALUE_CONTENT) {
    push(
      findings,
      `${AUTHORING_RS_REL}:${CONDITION_VALUE_ENUM}`,
      `${CONDITION_VALUE_ENUM} serde content must stay ${JSON.stringify(CONDITION_VALUE_CONTENT)}; do not invent a payload field`,
    );
  }
}

function requireConditionValue(schemas, findings) {
  const schema = own(schemas, CONDITION_VALUE);
  if (!isPlainObject(schema)) {
    push(
      findings,
      `#/components/schemas/${CONDITION_VALUE}`,
      `${CONDITION_VALUE} must be the existing internally tagged Rust enum — do not leave it unpublished`,
    );
    return;
  }
  const oneOf = own(schema, "oneOf");
  if (!Array.isArray(oneOf) || oneOf.length !== CONDITION_VALUE_VARIANTS.length) {
    push(
      findings,
      `#/components/schemas/${CONDITION_VALUE}/oneOf`,
      `${CONDITION_VALUE} must be a ${CONDITION_VALUE_VARIANTS.length}-member oneOf matching serde(tag = "kind", content = "value")`,
    );
  }
  const discriminator = own(schema, "discriminator");
  const propertyName = isPlainObject(discriminator)
    ? own(discriminator, "propertyName")
    : null;
  const mapping = isPlainObject(discriminator) ? own(discriminator, "mapping") : null;
  if (propertyName !== CONDITION_VALUE_TAG || !isPlainObject(mapping)) {
    push(
      findings,
      `#/components/schemas/${CONDITION_VALUE}/discriminator`,
      `${CONDITION_VALUE} must use discriminator propertyName ${CONDITION_VALUE_TAG} matching serde(tag = "kind")`,
    );
    return;
  }
  const mappingKeys = Object.keys(mapping);
  const extraKeys = mappingKeys.filter(
    (key) => !CONDITION_VALUE_VARIANTS.includes(key),
  );
  const missingKeys = CONDITION_VALUE_VARIANTS.filter(
    (key) => !mappingKeys.includes(key),
  );
  if (extraKeys.length > 0 || missingKeys.length > 0) {
    push(
      findings,
      `#/components/schemas/${CONDITION_VALUE}/discriminator/mapping`,
      `${CONDITION_VALUE} mapping must match the existing Rust enum exactly `
        + `(missing ${missingKeys.join(",") || "none"}; extra ${extraKeys.join(",") || "none"}). `
        + "Extra variants are an invented catalog — fail closed",
    );
  }
  const expectedRefs = CONDITION_VALUE_VARIANTS.map(
    (wire) => `#/components/schemas/${conditionValueVariantSchemaName(wire)}`,
  );
  const oneOfRefs = Array.isArray(oneOf)
    ? oneOf.map((member) => (isPlainObject(member) ? own(member, "$ref") : null))
    : [];
  if (!sameStringList(oneOfRefs, expectedRefs)) {
    push(
      findings,
      `#/components/schemas/${CONDITION_VALUE}/oneOf`,
      `${CONDITION_VALUE} oneOf $refs must be the existing serde variants in Rust declaration order`,
    );
  }
  for (const wire of CONDITION_VALUE_VARIANTS) {
    const variantName = conditionValueVariantSchemaName(wire);
    const mapped = own(mapping, wire);
    if (mapped !== `#/components/schemas/${variantName}`) {
      push(
        findings,
        `#/components/schemas/${CONDITION_VALUE}/discriminator/mapping/${wire}`,
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
    const kind = own(own(variant, "properties"), CONDITION_VALUE_TAG);
    const kindEnum = enumValues(kind);
    if (
      !isPlainObject(kind) ||
      own(kind, "type") !== "string" ||
      !sameStringList(kindEnum, [wire])
    ) {
      push(
        findings,
        `#/components/schemas/${variantName}/properties/${CONDITION_VALUE_TAG}`,
        `${variantName}.kind must be the singleton enum [${wire}] matching serde`,
      );
    }
    const names = schemaPropertyNames(variant);
    const extraProps = names.filter(
      (name) => name !== CONDITION_VALUE_TAG && name !== CONDITION_VALUE_CONTENT,
    );
    if (extraProps.length > 0) {
      push(
        findings,
        `#/components/schemas/${variantName}/properties`,
        `${variantName} extra properties ${extraProps.join(",")} are an invented catalog`,
      );
    }
    const payload = own(own(variant, "properties"), CONDITION_VALUE_CONTENT);
    if (CONDITION_VALUE_BOOL_VARIANTS.includes(wire)) {
      if (!isPlainObject(payload) || own(payload, "type") !== "boolean") {
        push(
          findings,
          `#/components/schemas/${variantName}/properties/${CONDITION_VALUE_CONTENT}`,
          `${variantName}.value is bool on the wire; do not invent a catalog`,
        );
      }
    } else if (CONDITION_VALUE_STRING_VARIANTS.includes(wire)) {
      if (
        !isPlainObject(payload) ||
        own(payload, "type") !== "string" ||
        enumValues(payload)
      ) {
        push(
          findings,
          `#/components/schemas/${variantName}/properties/${CONDITION_VALUE_CONTENT}`,
          `${variantName}.value is unconstrained String on the wire; do not invent a catalog`,
        );
      }
    }
  }
}

function refuseForeignBind(findings, location, schema, label) {
  const objectName = schemaRefName(schema);
  if (
    objectName === CONDITION_VALUE ||
    objectName === CONDITION_OP ||
    objectName === EFFECT
  ) {
    push(findings, location, `${label}; do not bind it to ${objectName}`);
  }
}

function refuseStringCatalog(findings, location, schema, label) {
  if (!isPlainObject(schema)) return;
  if (schemaRefName(schema) || Array.isArray(own(schema, "enum"))) {
    push(
      findings,
      location,
      `${label}; do not invent an enum catalog from unconstrained TEXT`,
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
export function evaluateOpenapiConditionValue({ repoRoot }) {
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

  requireClosedTaggedContentEnum(findings, repoRoot);
  requireClosedUnitEnum(
    findings,
    repoRoot,
    CONDITION_OP_ENUM,
    CONDITION_OP_VARIANTS,
    null,
  );
  requireClosedUnitEnum(findings, repoRoot, EFFECT_ENUM, EFFECT_VARIANTS, null);
  requireRustFields(
    findings,
    repoRoot,
    AUTHORING_RS_REL,
    CONDITION_STRUCT,
    CONDITION_FIELDS,
  );

  const condition = own(schemas, POLICY_NO_CODE_CONDITION);
  if (!isPlainObject(condition)) {
    push(
      findings,
      `#/components/schemas/${POLICY_NO_CODE_CONDITION}`,
      `${POLICY_NO_CODE_CONDITION} must remain published; this slice types its value field`,
    );
  } else {
    const properties = own(condition, "properties");
    const value = own(properties, "value");
    const valueName = schemaRefName(value);
    if (valueName === CONDITION_VALUE) {
      bound += 1;
    } else {
      push(
        findings,
        `#/components/schemas/${POLICY_NO_CODE_CONDITION}/properties/value`,
        `${POLICY_NO_CODE_CONDITION}.value is ConditionValue already on the wire; `
          + `$ref ${CONDITION_VALUE} — do not leave additionalProperties bags or invent a catalog`,
      );
    }
    const attr = own(properties, "attr");
    refuseStringCatalog(
      findings,
      `#/components/schemas/${POLICY_NO_CODE_CONDITION}/properties/attr`,
      attr,
      "condition attr is unconstrained String (declared object properties are not a closed serde enum)",
    );
    const op = own(properties, "op");
    const opListed = enumValues(op);
    if (!sameStringList(opListed, CONDITION_OP_VARIANTS)) {
      const extra = (opListed ?? []).filter(
        (value) => !CONDITION_OP_VARIANTS.includes(value),
      );
      const missing = CONDITION_OP_VARIANTS.filter(
        (value) => !(opListed ?? []).includes(value),
      );
      push(
        findings,
        `#/components/schemas/${POLICY_NO_CODE_CONDITION}/properties/op/enum`,
        `op must match existing ConditionOp exactly `
          + `(missing ${missing.join(",") || "none"}; extra ${extra.join(",") || "none"}). `
          + "Extra variants are an invented catalog — fail closed",
      );
    }
  }

  const blocks = own(schemas, POLICY_NO_CODE_BLOCKS);
  if (isPlainObject(blocks)) {
    const properties = own(blocks, "properties");
    for (const field of BLOCKS_STRING_FIELDS) {
      refuseStringCatalog(
        findings,
        `#/components/schemas/${POLICY_NO_CODE_BLOCKS}/properties/${field}`,
        own(properties, field),
        `${field} is unconstrained String on NoCodeBlocks`,
      );
    }
    const effect = own(properties, "effect");
    const effectListed = enumValues(effect);
    if (!sameStringList(effectListed, EFFECT_VARIANTS)) {
      const extra = (effectListed ?? []).filter(
        (value) => !EFFECT_VARIANTS.includes(value),
      );
      push(
        findings,
        `#/components/schemas/${POLICY_NO_CODE_BLOCKS}/properties/effect/enum`,
        `effect must match existing Effect exactly; extra ${extra.join(",") || "none"} is an invented catalog`,
      );
    }
  }

  requireConditionValue(schemas, findings);

  const audit = own(schemas, AUDIT_RECORD);
  if (isPlainObject(audit)) {
    const action = own(own(audit, "properties"), AUDIT_ACTION_FIELD);
    const actionName = schemaRefName(action);
    if (
      actionName === CONDITION_VALUE ||
      actionName === CONDITION_OP ||
      actionName === EFFECT ||
      (isPlainObject(action) && Array.isArray(own(action, "enum")))
    ) {
      push(
        findings,
        `#/components/schemas/${AUDIT_RECORD}/properties/${AUDIT_ACTION_FIELD}`,
        "audit action is unconstrained TEXT; do not bind it to ConditionValue or invent an action catalog",
      );
    }
  }

  for (const [path, method, code, label] of [
    [
      KILL_SWITCH_PATH,
      "post",
      "200",
      `POST ${KILL_SWITCH_PATH} is console kill-switch, not a policy condition`,
    ],
    [
      ROLLOUT_OPT_IN_PATH,
      "put",
      "200",
      `PUT ${ROLLOUT_OPT_IN_PATH} is console rollout, not a policy condition`,
    ],
    [
      ROLLOUT_ORG_FLAG_PATH,
      "put",
      "200",
      `PUT ${ROLLOUT_ORG_FLAG_PATH} is console rollout, not a policy condition`,
    ],
    [
      AUDIT_GET_PATH,
      "get",
      "200",
      `GET ${AUDIT_GET_PATH} already returns AuditPage, not ConditionValue`,
    ],
  ]) {
    refuseForeignBind(
      findings,
      `#/paths/${path}/${method}/responses/${code}`,
      jsonOkSchema(findOperation(paths, path, method), code),
      label,
    );
  }

  for (const [path, method] of [
    [DRAFTS_PATH, "post"],
    [DRAFT_ID_PATH, "put"],
    [PREFLIGHT_PATH, "post"],
    [EXECUTE_PATH, "post"],
  ]) {
    const operation = findOperation(paths, path, method);
    if (!isPlainObject(operation)) continue;
    if (hasPermissions(operation)) {
      push(
        findings,
        `#/paths/${path}/${method}/permissions`,
        "operation-level permissions are admitted only on Head GET/list; "
          + `do not map Feature::ALL onto ${method.toUpperCase()} ${path}`,
      );
    }
    const headers = jsonOkHeaders(operation, method === "post" && path === DRAFTS_PATH ? "201" : "200");
    if (isPlainObject(headers) && hasOwnKey(headers, "ETag")) {
      push(
        findings,
        `#/paths/${path}/${method}/responses/200/headers/ETag`,
        "HTTP ETag stays HOLD — do not stamp it on a policy-draft envelope",
      );
    }
    const boundName = schemaRefName(
      jsonOkSchema(
        operation,
        method === "post" && path === DRAFTS_PATH ? "201" : "200",
      ),
    );
    if (HEAD_SCHEMA_NAMES.includes(boundName)) {
      push(
        findings,
        `#/paths/${path}/${method}/responses/${method === "post" && path === DRAFTS_PATH ? "201" : "200"}`,
        `${method.toUpperCase()} ${path} is not a Head; do not bind it to ${boundName}`,
      );
    }
  }

  return { writes, bound, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiConditionValue({ repoRoot });
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
      `openapi condition-value typed-field gate FAILED: ${findings.length} finding(s), `
        + `${writes} write(s), bound=${bound}/${BOUND}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi condition-value typed-field gate passed `
      + `(${POLICY_NO_CODE_CONDITION}.value $ref ${CONDITION_VALUE}; `
      + `${CONDITION_VALUE} matches serde tag+content; `
      + `${writes} write operations, 0 findings)`,
  );
}
