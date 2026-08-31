// Typed TypeScript SDK generator (ADR-0031 SDK slice).
//
// The hole this closes: #998–#1000 generate OpenAPI Head/Input *schemas* from
// DTO types, but the published contract still stops at YAML. PRODUCT requires
// that typed objects/links/actions generate OpenAPI *and* SDKs. Dual-written
// YAML + hand TS Heads is not that.
//
// Chesterton: read committed `backend/openapi/openapi.yaml` (compose output).
// Do not invent a second OpenAPI writer. Do not hand-catalog Head/Input fields.
// Schema *names* come from the existing 21-name roster; property *shapes* come
// from the composed document. One language (TypeScript). Not published to npm.
//
// Totality: js-yaml load + own-property walks of components.schemas. A walker
// that visits nothing emits nothing, so the check's HEAD/INPUT floors lock
// examined-zero.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  NESTED_INPUT_SCHEMAS,
} from "./check-openapi-semantic-generate.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export const OPENAPI_REL = "backend/openapi/openapi.yaml";
export const SDK_DIR_REL = "sdk/typescript";
export const SDK_PACKAGE_REL = "sdk/typescript/package.json";
export const SDK_INDEX_REL = "sdk/typescript/src/index.ts";
export const SDK_GENERATED_REL = "sdk/typescript/src/generated.ts";

export const SDK_FILE_RELS = Object.freeze([
  SDK_PACKAGE_REL,
  SDK_INDEX_REL,
  SDK_GENERATED_REL,
]);

export const GENERATED_HEADER =
  "Generated from backend/openapi/openapi.yaml by scripts/generate-openapi-ts-sdk.mjs.\n * Do not edit.";

const REF_PREFIX = "#/components/schemas/";
const TS_MAX_DEPTH = 16;

const TS_RESERVED = new Set([
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "for",
  "function",
  "if",
  "implements",
  "import",
  "in",
  "instanceof",
  "interface",
  "let",
  "new",
  "null",
  "package",
  "private",
  "protected",
  "public",
  "return",
  "static",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "try",
  "typeof",
  "var",
  "void",
  "while",
  "with",
  "yield",
  "type",
]);

export function schemaRefName(ref) {
  if (typeof ref !== "string" || !ref.startsWith(REF_PREFIX)) return null;
  const name = ref.slice(REF_PREFIX.length);
  if (!name || name.includes("/")) return null;
  return name;
}

function collectRefNames(node, acc) {
  if (Array.isArray(node)) {
    for (const item of node) collectRefNames(item, acc);
    return;
  }
  if (!isPlainObject(node)) return;
  const ref = schemaRefName(own(node, "$ref"));
  if (ref) acc.add(ref);
  const properties = own(node, "properties");
  if (isPlainObject(properties)) {
    for (const value of Object.values(properties)) collectRefNames(value, acc);
  }
  collectRefNames(own(node, "items"), acc);
  collectRefNames(own(node, "additionalProperties"), acc);
  for (const key of ["oneOf", "anyOf", "allOf", "not"]) {
    collectRefNames(own(node, key), acc);
  }
}

function tsIdent(name) {
  if (typeof name !== "string") return JSON.stringify(String(name));
  if (/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(name) && !TS_RESERVED.has(name)) return name;
  return JSON.stringify(name);
}

function jsdoc(description) {
  if (typeof description !== "string" || description.trim() === "") return "";
  const lines = description.replaceAll("*/", "*\\/").split(/\r?\n/);
  return `/**\n${lines.map((line) => ` * ${line}`.trimEnd()).join("\n")}\n */\n`;
}

function primitive(type) {
  if (type === "string") return "string";
  if (type === "integer" || type === "number") return "number";
  if (type === "boolean") return "boolean";
  if (type === "null") return "null";
  return "unknown";
}

/**
 * @param {unknown} schema
 * @param {number} depth
 * @returns {string}
 */
export function schemaToTs(schema, depth = 0) {
  if (depth > TS_MAX_DEPTH) return "unknown";
  if (schema === true) return "unknown";
  if (schema === false) return "never";
  if (!isPlainObject(schema)) return "unknown";

  const ref = schemaRefName(own(schema, "$ref"));
  if (ref) return ref;

  const constValue = own(schema, "const");
  if (constValue !== undefined) return JSON.stringify(constValue);

  const enumValue = own(schema, "enum");
  if (Array.isArray(enumValue) && enumValue.length > 0) {
    return enumValue.map((item) => JSON.stringify(item)).join(" | ");
  }

  const oneOf = own(schema, "oneOf");
  if (Array.isArray(oneOf) && oneOf.length > 0) {
    return oneOf.map((item) => schemaToTs(item, depth + 1)).join(" | ");
  }
  const anyOf = own(schema, "anyOf");
  if (Array.isArray(anyOf) && anyOf.length > 0) {
    return anyOf.map((item) => schemaToTs(item, depth + 1)).join(" | ");
  }
  const allOf = own(schema, "allOf");
  if (Array.isArray(allOf) && allOf.length > 0) {
    return allOf.map((item) => schemaToTs(item, depth + 1)).join(" & ");
  }

  const type = own(schema, "type");
  if (Array.isArray(type)) {
    const parts = type.map((item) => {
      if (item === "object") return objectToTs(schema, depth);
      if (item === "array") return arrayToTs(schema, depth);
      return primitive(item);
    });
    return parts.join(" | ");
  }
  if (type === "array") return arrayToTs(schema, depth);
  if (type === "object" || isPlainObject(own(schema, "properties"))) {
    return objectToTs(schema, depth);
  }
  if (typeof type === "string") return primitive(type);
  return "unknown";
}

function arrayToTs(schema, depth) {
  const items = own(schema, "items");
  return `Array<${schemaToTs(items, depth + 1)}>`;
}

function indentNested(ts) {
  return ts.includes("\n") ? ts.replaceAll("\n", "\n  ") : ts;
}

function objectToTs(schema, depth) {
  const properties = own(schema, "properties");
  const required = own(schema, "required");
  const requiredSet = new Set(Array.isArray(required) ? required.filter((name) => typeof name === "string") : []);
  const lines = [];
  if (isPlainObject(properties)) {
    for (const name of Object.keys(properties)) {
      const optional = requiredSet.has(name) ? "" : "?";
      const rendered = indentNested(schemaToTs(own(properties, name), depth + 1));
      lines.push(`  ${tsIdent(name)}${optional}: ${rendered};`);
    }
  }
  const additional = own(schema, "additionalProperties");
  if (additional === true) {
    lines.push("  [key: string]: unknown;");
  } else if (isPlainObject(additional)) {
    lines.push(`  [key: string]: ${indentNested(schemaToTs(additional, depth + 1))};`);
  }
  if (lines.length === 0) {
    if (additional === false) return "Record<string, never>";
    return "Record<string, unknown>";
  }
  return `{\n${lines.join("\n")}\n}`;
}

function tsLiteral(value, indent) {
  if (value === null) return "null";
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";
    const inner = value.map((item) => `${indent}  ${tsLiteral(item, `${indent}  `)}`).join(",\n");
    return `[\n${inner},\n${indent}]`;
  }
  if (!isPlainObject(value)) return "null";
  const keys = Object.keys(value);
  if (keys.length === 0) return "{}";
  const inner = keys
    .map((key) => `${indent}  ${tsIdent(key)}: ${tsLiteral(own(value, key), `${indent}  `)}`)
    .join(",\n");
  return `{\n${inner},\n${indent}}`;
}

function emitNamedType(name, schema) {
  const description = typeof own(schema, "description") === "string" ? own(schema, "description") : "";
  const body = schemaToTs(schema);
  return `${jsdoc(description)}export type ${name} = ${body};\n`;
}

function emitHeadDefinition(name, schema) {
  if (!isPlainObject(schema)) return "";
  const links = own(schema, "links");
  const actions = own(schema, "actions");
  if (!Array.isArray(links) && !Array.isArray(actions)) return "";
  const definition = {
    name,
    links: Array.isArray(links) ? links : [],
    actions: Array.isArray(actions) ? actions : [],
  };
  return `export const ${name}Definition = ${tsLiteral(definition, "")} as const;\n`;
}

function namesToEmit(published) {
  const seen = new Set();
  const queue = [...GENERATED_SCHEMA_NAMES];
  while (queue.length > 0) {
    const name = queue.shift();
    if (seen.has(name)) continue;
    seen.add(name);
    const refs = new Set();
    collectRefNames(own(published, name), refs);
    for (const ref of refs) queue.push(ref);
  }
  const extras = [...seen].filter((name) => !GENERATED_SCHEMA_NAMES.includes(name)).sort();
  return [...GENERATED_SCHEMA_NAMES, ...extras];
}

export function generateSdkFiles(document) {
  const published = own(own(document, "components"), "schemas");
  const schemas = isPlainObject(published) ? published : {};
  const names = namesToEmit(schemas);
  const chunks = [
    `/**\n * ${GENERATED_HEADER}\n */\n`,
  ];
  const exportedTypes = [];
  const exportedDefinitions = [];

  for (const name of names) {
    const schema = own(schemas, name);
    if (!isPlainObject(schema)) continue;
    chunks.push(emitNamedType(name, schema));
    exportedTypes.push(name);
    if (HEAD_SCHEMA_NAMES.includes(name)) {
      const definition = emitHeadDefinition(name, schema);
      if (definition) {
        chunks.push(definition);
        exportedDefinitions.push(name);
      }
    }
  }

  const generated = `${chunks.join("\n").replace(/\n{3,}/g, "\n\n").trimEnd()}\n`;
  const index = `/**\n * ${GENERATED_HEADER}\n */\nexport * from "./generated.ts";\n`;
  const pkg = `${JSON.stringify(
    {
      name: "@oyatie/console-openapi",
      private: true,
      version: "0.0.0",
      type: "module",
      description: "In-repo TypeScript SDK generated from backend/openapi/openapi.yaml. Not published.",
      exports: {
        ".": "./src/index.ts",
      },
    },
    null,
    2,
  )}\n`;

  return {
    files: {
      [SDK_PACKAGE_REL]: pkg,
      [SDK_INDEX_REL]: index,
      [SDK_GENERATED_REL]: generated,
    },
    exportedTypes,
    exportedDefinitions,
    heads: exportedTypes.filter((name) => HEAD_SCHEMA_NAMES.includes(name)).length,
    inputs: exportedTypes.filter((name) => INPUT_SCHEMA_NAMES.includes(name)).length,
    nested: exportedTypes.filter((name) => NESTED_INPUT_SCHEMAS.includes(name)).length,
  };
}

export function loadOpenApiDocument(repoRoot) {
  return yaml.load(readFileSync(join(repoRoot, OPENAPI_REL), "utf8"));
}

export function writeSdkFiles(repoRoot, files) {
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(repoRoot, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const write = process.argv.includes("--write");
  const args = process.argv.slice(2).filter((arg) => arg !== "--write");
  const repoRoot = args[0] ?? fileURLToPath(new URL("..", import.meta.url));
  const generated = generateSdkFiles(loadOpenApiDocument(repoRoot));
  if (write) {
    writeSdkFiles(repoRoot, generated.files);
    console.log(
      `wrote ${SDK_DIR_REL} `
        + `(${generated.heads} Heads, ${generated.inputs} Inputs, ${generated.nested} nested, `
        + `${generated.exportedTypes.length} exported types)`,
    );
  } else {
    process.stdout.write(generated.files[SDK_GENERATED_REL]);
  }
}
