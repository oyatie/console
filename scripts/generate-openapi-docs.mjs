// Generated API docs (ADR-0031 docs slice).
//
// The hole this closes: composed OpenAPI is regenerated in CI and the in-repo
// TypeScript SDK is generated from that document, but PRODUCT also requires
// generated docs. Dual-written YAML + a hand-authored docs site is not that.
//
// Chesterton: extend the OpenAPI regen+diff pattern. Do not replace compose.
// This generator reads committed `backend/openapi/openapi.yaml` and emits one
// static HTML artifact. Schema *names* reuse the existing 21-name roster;
// property *shapes*, links, actions, and operations come from the composed
// document. Not published.
//
// Totality: js-yaml load + own-property walks of components.schemas and paths.
// A walker that visits nothing emits nothing, so the check's HEAD/INPUT floors
// and the operations examined-zero lock close that class.

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
import { OPENAPI_REL, schemaRefName } from "./generate-openapi-ts-sdk.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export { OPENAPI_REL };

export const DOCS_DIR_REL = "sdk/docs";
export const DOCS_REL = "sdk/docs/index.html";

export const DOCS_FILE_RELS = Object.freeze([DOCS_REL]);

export const GENERATED_HEADER =
  "Generated from backend/openapi/openapi.yaml by scripts/generate-openapi-docs.mjs. Do not edit.";

export const HTTP_METHODS = Object.freeze([
  "get",
  "post",
  "put",
  "patch",
  "delete",
  "head",
  "options",
  "trace",
]);

const HTML_MAX_DEPTH = 16;

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
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

function schemaKind(name) {
  if (HEAD_SCHEMA_NAMES.includes(name)) return "head";
  if (INPUT_SCHEMA_NAMES.includes(name)) return "input";
  if (NESTED_INPUT_SCHEMAS.includes(name)) return "nested";
  return "ref";
}

/**
 * @param {unknown} schema
 * @param {number} depth
 * @returns {string}
 */
export function schemaToHtml(schema, depth = 0) {
  if (depth > HTML_MAX_DEPTH) return "unknown";
  if (schema === true) return "unknown";
  if (schema === false) return "never";
  if (!isPlainObject(schema)) return "unknown";

  const ref = schemaRefName(own(schema, "$ref"));
  if (ref) return `<a href="#schema-${escapeHtml(ref)}">${escapeHtml(ref)}</a>`;

  const constValue = own(schema, "const");
  if (constValue !== undefined) return `<code>${escapeHtml(JSON.stringify(constValue))}</code>`;

  const enumValue = own(schema, "enum");
  if (Array.isArray(enumValue) && enumValue.length > 0) {
    return enumValue.map((item) => `<code>${escapeHtml(JSON.stringify(item))}</code>`).join(" | ");
  }

  const oneOf = own(schema, "oneOf");
  if (Array.isArray(oneOf) && oneOf.length > 0) {
    return oneOf.map((item) => schemaToHtml(item, depth + 1)).join(" | ");
  }
  const anyOf = own(schema, "anyOf");
  if (Array.isArray(anyOf) && anyOf.length > 0) {
    return anyOf.map((item) => schemaToHtml(item, depth + 1)).join(" | ");
  }
  const allOf = own(schema, "allOf");
  if (Array.isArray(allOf) && allOf.length > 0) {
    return allOf.map((item) => schemaToHtml(item, depth + 1)).join(" &amp; ");
  }

  const type = own(schema, "type");
  if (Array.isArray(type)) {
    return type.map((item) => {
      if (item === "object") return objectToHtml(schema, depth);
      if (item === "array") return arrayToHtml(schema, depth);
      return escapeHtml(typeof item === "string" ? item : "unknown");
    }).join(" | ");
  }
  if (type === "array") return arrayToHtml(schema, depth);
  if (type === "object" || isPlainObject(own(schema, "properties"))) {
    return objectToHtml(schema, depth);
  }
  if (typeof type === "string") {
    const format = own(schema, "format");
    if (typeof format === "string" && format !== "") {
      return `${escapeHtml(type)} <code>${escapeHtml(format)}</code>`;
    }
    return escapeHtml(type);
  }
  return "unknown";
}

function arrayToHtml(schema, depth) {
  return `array of ${schemaToHtml(own(schema, "items"), depth + 1)}`;
}

function objectToHtml(schema, depth) {
  const properties = own(schema, "properties");
  if (!isPlainObject(properties) || Object.keys(properties).length === 0) {
    const additional = own(schema, "additionalProperties");
    if (additional === false) return "object (closed)";
    if (isPlainObject(additional)) return `object of ${schemaToHtml(additional, depth + 1)}`;
    return "object";
  }
  const required = own(schema, "required");
  const requiredSet = new Set(Array.isArray(required) ? required.filter((name) => typeof name === "string") : []);
  const fields = Object.keys(properties).map((name) => {
    const mark = requiredSet.has(name) ? "" : "?";
    return `<code>${escapeHtml(name)}${mark}</code>: ${schemaToHtml(own(properties, name), depth + 1)}`;
  });
  return `{ ${fields.join("; ")} }`;
}

function propertyRow(name, schema, requiredSet) {
  const required = requiredSet.has(name);
  const constValue = isPlainObject(schema) ? own(schema, "const") : undefined;
  const attrs = [`data-property="${escapeHtml(name)}"`];
  if (required) attrs.push('data-required="true"');
  if (constValue !== undefined) attrs.push(`data-const="${escapeHtml(JSON.stringify(constValue))}"`);
  const description = isPlainObject(schema) && typeof own(schema, "description") === "string"
    ? own(schema, "description")
    : "";
  return `<tr ${attrs.join(" ")}><th>${escapeHtml(name)}</th><td>${required ? "required" : "optional"}</td><td>${schemaToHtml(schema)}</td><td>${escapeHtml(description)}</td></tr>`;
}

function renderLinks(links) {
  if (!Array.isArray(links) || links.length === 0) {
    return '<p class="empty">No outgoing Head links.</p>';
  }
  const items = [];
  for (const link of links) {
    if (!isPlainObject(link)) continue;
    const key = typeof own(link, "key") === "string" ? own(link, "key") : "";
    const from = typeof own(link, "from") === "string" ? own(link, "from") : "";
    const to = typeof own(link, "to") === "string" ? own(link, "to") : "";
    const field = typeof own(link, "field") === "string" ? own(link, "field") : "";
    const cardinality = typeof own(link, "cardinality") === "string" ? own(link, "cardinality") : "";
    items.push(
      `<li data-link="${escapeHtml(key)}"><code>${escapeHtml(key)}</code> ${escapeHtml(from)} → ${escapeHtml(to)} via <code>${escapeHtml(field)}</code> (${escapeHtml(cardinality)})</li>`,
    );
  }
  return items.length > 0 ? `<ul class="links">${items.join("")}</ul>` : '<p class="empty">No outgoing Head links.</p>';
}

function renderConcurrency(concurrency) {
  if (!isPlainObject(concurrency)) {
    return '<p class="empty">No Head concurrency token.</p>';
  }
  const getToken = own(concurrency, "get_token");
  const writeField = typeof own(concurrency, "write_field") === "string"
    ? own(concurrency, "write_field")
    : "";
  const writeIn = typeof own(concurrency, "write_in") === "string"
    ? own(concurrency, "write_in")
    : "";
  const token = getToken === null ? "null" : String(getToken ?? "");
  return `<p class="concurrency" data-get-token="${escapeHtml(token)}" data-write-field="${escapeHtml(writeField)}" data-write-in="${escapeHtml(writeIn)}">GET token <code>${escapeHtml(token)}</code>; writes <code>${escapeHtml(writeField)}</code> in <code>${escapeHtml(writeIn)}</code> (not HTTP If-Match).</p>`;
}

function renderActions(actions) {
  if (!Array.isArray(actions) || actions.length === 0) {
    return '<p class="empty">No actions.</p>';
  }
  const items = [];
  for (const action of actions) {
    if (!isPlainObject(action)) continue;
    const key = typeof own(action, "action_key") === "string" ? own(action, "action_key") : "";
    const inputRef = schemaRefName(own(own(action, "input"), "$ref")) ?? "";
    const fourEyes = typeof own(action, "four_eyes") === "string" ? own(action, "four_eyes") : "";
    const inputHtml = inputRef === ""
      ? "untyped"
      : `<a href="#schema-${escapeHtml(inputRef)}">${escapeHtml(inputRef)}</a>`;
    items.push(
      `<li data-action="${escapeHtml(key)}"><code>${escapeHtml(key)}</code> input ${inputHtml} four_eyes ${escapeHtml(fourEyes)}</li>`,
    );
  }
  return items.length > 0 ? `<ul class="actions">${items.join("")}</ul>` : '<p class="empty">No actions.</p>';
}

function emitSchema(name, schema) {
  const kind = schemaKind(name);
  const description = typeof own(schema, "description") === "string" ? own(schema, "description") : "";
  const properties = own(schema, "properties");
  const required = own(schema, "required");
  const requiredSet = new Set(Array.isArray(required) ? required.filter((item) => typeof item === "string") : []);
  const rows = [];
  if (isPlainObject(properties)) {
    for (const propName of Object.keys(properties)) {
      rows.push(propertyRow(propName, own(properties, propName), requiredSet));
    }
  }
  const propsTable = rows.length > 0
    ? `<table class="properties"><thead><tr><th>Field</th><th>Presence</th><th>Type</th><th>Notes</th></tr></thead><tbody>${rows.join("")}</tbody></table>`
    : '<p class="empty">No properties.</p>';
  let extra = "";
  if (kind === "head") {
    extra = `<h4>Links</h4>${renderLinks(own(schema, "links"))}<h4>Actions</h4>${renderActions(own(schema, "actions"))}<h4>Concurrency</h4>${renderConcurrency(own(schema, "concurrency"))}`;
  }
  return `<section id="schema-${escapeHtml(name)}" data-schema="${escapeHtml(name)}" data-kind="${escapeHtml(kind)}"><h3>${escapeHtml(name)}</h3><p class="description">${escapeHtml(description)}</p>${propsTable}${extra}</section>`;
}

/**
 * @param {unknown} document
 * @returns {{ method: string, path: string, operationId: string, summary: string }[]}
 */
export function collectOperations(document) {
  const operations = [];
  const paths = own(document, "paths");
  if (!isPlainObject(paths)) return operations;
  for (const path of Object.keys(paths)) {
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of HTTP_METHODS) {
      const op = own(item, method);
      if (!isPlainObject(op)) continue;
      operations.push({
        method: method.toUpperCase(),
        path,
        operationId: typeof own(op, "operationId") === "string" ? own(op, "operationId") : "",
        summary: typeof own(op, "summary") === "string" ? own(op, "summary") : "",
      });
    }
  }
  return operations;
}

function renderOperations(operations) {
  const rows = operations.map((operation) => {
    const key = `${operation.method} ${operation.path}`;
    return `<tr data-operation="${escapeHtml(key)}"><th>${escapeHtml(operation.method)}</th><td><code>${escapeHtml(operation.path)}</code></td><td>${escapeHtml(operation.operationId)}</td><td>${escapeHtml(operation.summary)}</td></tr>`;
  });
  return `<section id="operations" data-operation-count="${operations.length}"><h2>Operations</h2><p class="meta">${operations.length} operations from composed <code>paths</code>.</p><table class="operations"><thead><tr><th>Method</th><th>Path</th><th>Operation ID</th><th>Summary</th></tr></thead><tbody>${rows.join("")}</tbody></table></section>`;
}

const STYLE = [
  "body{font-family:ui-sans-serif,system-ui,sans-serif;line-height:1.45;margin:1.25rem auto;max-width:72rem;padding:0 1rem}",
  ".banner{padding:.5rem .75rem;background:#f4f4f5;border:1px solid #d4d4d8}",
  "nav a{margin-right:.75rem}",
  "table{border-collapse:collapse;width:100%;margin:.5rem 0 1.25rem}",
  "th,td{border:1px solid #d4d4d8;padding:.3rem .5rem;text-align:left;vertical-align:top}",
  "code{font-family:ui-monospace,monospace;font-size:.92em}",
  "h2{margin-top:2rem}",
  ".meta,.empty,.description{color:#52525b}",
].join("");

export function generateDocsFiles(document) {
  const info = own(document, "info");
  const title = typeof own(info, "title") === "string" ? own(info, "title") : "API";
  const version = typeof own(info, "version") === "string" ? own(info, "version") : "";
  const published = own(own(document, "components"), "schemas");
  const schemas = isPlainObject(published) ? published : {};
  const names = namesToEmit(schemas);
  const operations = collectOperations(document);

  const schemaHtml = [];
  const exported = [];
  for (const name of names) {
    const schema = own(schemas, name);
    if (!isPlainObject(schema)) continue;
    schemaHtml.push(emitSchema(name, schema));
    exported.push(name);
  }

  const html = [
    "<!DOCTYPE html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="utf-8"/>',
    `<title>${escapeHtml(title)}</title>`,
    `<meta name="generator" content="${escapeHtml("scripts/generate-openapi-docs.mjs")}"/>`,
    `<style>${STYLE}</style>`,
    "</head>",
    "<body>",
    `<!-- ${GENERATED_HEADER} -->`,
    `<p class="banner">${escapeHtml(GENERATED_HEADER)}</p>`,
    `<h1>${escapeHtml(title)}</h1>`,
    `<p class="meta">version ${escapeHtml(version)} · generated from <code>${escapeHtml(OPENAPI_REL)}</code></p>`,
    '<nav><a href="#objects">Objects</a> <a href="#inputs">Inputs</a> <a href="#operations">Operations</a></nav>',
    '<section id="objects"><h2>Typed objects</h2>',
    schemaHtml.filter((_, index) => schemaKind(exported[index]) === "head").join(""),
    "</section>",
    '<section id="inputs"><h2>Action inputs</h2>',
    schemaHtml.filter((_, index) => {
      const kind = schemaKind(exported[index]);
      return kind === "input" || kind === "nested";
    }).join(""),
    "</section>",
    '<section id="refs"><h2>Referenced schemas</h2>',
    schemaHtml.filter((_, index) => schemaKind(exported[index]) === "ref").join(""),
    "</section>",
    renderOperations(operations),
    "</body>",
    "</html>",
    "",
  ].join("\n");

  return {
    files: {
      [DOCS_REL]: html,
    },
    exported,
    operations: operations.length,
    heads: exported.filter((name) => HEAD_SCHEMA_NAMES.includes(name)).length,
    inputs: exported.filter((name) => INPUT_SCHEMA_NAMES.includes(name)).length,
    nested: exported.filter((name) => NESTED_INPUT_SCHEMAS.includes(name)).length,
  };
}

export function loadOpenApiDocument(repoRoot) {
  return yaml.load(readFileSync(join(repoRoot, OPENAPI_REL), "utf8"));
}

export function writeDocsFiles(repoRoot, files) {
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
  const generated = generateDocsFiles(loadOpenApiDocument(repoRoot));
  if (write) {
    writeDocsFiles(repoRoot, generated.files);
    console.log(
      `wrote ${DOCS_DIR_REL} `
        + `(${generated.heads} Heads, ${generated.inputs} Inputs, ${generated.nested} nested, `
        + `${generated.operations} operations)`,
    );
  } else {
    process.stdout.write(generated.files[DOCS_REL]);
  }
}
