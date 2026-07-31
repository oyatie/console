// H-1 gate: an operation's requestBody must match the deny_unknown_fields struct its handler binds.
//
// The hole this closes: backend/app/tests/openapi_drift.rs compares PATH INVENTORIES. It contains
// zero occurrences of `requestBody` and zero of `deny_unknown_fields`, so a path can be present in
// both the spec and the router while every field name inside its body disagrees — and with
// `deny_unknown_fields` that disagreement is a guaranteed 422, not a maybe.
//
// SCOPE, stated so it is never read as more than it is: this compares the subset of operations
// whose handler binds a `Json<T>` where T carries `deny_unknown_fields`. Everything else is
// undecidable in this direction and lands in `skipped`. This is a FLOOR on correctness coverage.
// It is NOT "request bodies are checked".
//
// The invariant is the NAMED ANCHORS, not the count. Five separate resolver bugs during this
// gate's construction each exited 0 with `findings: 0` while comparing less and less; a count
// floor cannot catch a single-operation drop, so each anchor must resolve or the gate hard-fails.

import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

// Each anchor exercises a route form that has already broken a resolver silently.
const ANCHORS = [
  "POST /api/v1/equipment-3r/rental-cases/{case_id}/handover", // rustfmt-wrapped path const
  "POST /api/v1/inventory/items/{item_id}/consumptions", // wrapped route, two methods on one line
  "POST /api/v1/inventory/items/{item_id}/receipts", // single-line route
];

// Measured 50 against the inspected route list, not against a first run.
const RESOLVED_FLOOR = 45;

// The `\s*` after `=` is load-bearing: rustfmt wraps long path consts onto the next line, and a
// single-line-only regex silently dropped equipment's handover anchor.
const CONST_PATH = /pub const ([A-Z0-9_]+): &str =\s*"([^"]+)"/g;
// `\)\s*,?\s*\)` closes the route call; an earlier `[^)]*` form consumed the FOLLOWING `.route`'s
// leading dot and lost 258 of 525 routes.
const ROUTE = /\.route\(\s*([A-Z0-9_]+)\s*,([\s\S]*?)\)\s*,?\s*\)/g;
// Over the route's method group. Stopping at the first `)` lost `.post(consume_item)` entirely.
const METHOD = /\b(get|post|put|patch|delete)\(\s*([a-z0-9_]+)/g;
const HANDLER = /async fn ([a-z0-9_]+)\s*\(([\s\S]*?)\)\s*->/g;
const JSON_BODY = /Json\(\s*\w+\s*\)\s*:\s*Json<\s*([A-Za-z0-9_]+)\s*>/;
const STRUCT = /#\[serde\(([^\]]*)\)\]\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z0-9_]+)\s*\{([\s\S]*?)\n\}/g;
// `[a-z]+` cannot match `camelCase`. The capital-letter class is mandatory; `_-` is here so
// snake_case and kebab-case are handled rather than silently read as "no rename".
const RENAME_ALL = /rename_all\s*=\s*"([A-Za-z_-]+)"/;

// Mirrors `RenameRule::apply_to_field` in serde_derive_internals-0.29.1 src/case.rs, which is the
// function that decides what the server actually accepts. Two rules were guessed rather than read:
// serde's `LowerCase` is `field.to_owned()` and its `UpperCase` is `field.to_ascii_uppercase()` —
// both keep the underscores. A `words.join("")` form dropped them, which reads a spec publishing
// `very_tasty` as a mismatch and a spec publishing `verytasty` — a guaranteed 422 under
// deny_unknown_fields — as correct. `lowercase` and `snake_case` are identity and fall to default.
export function renameField(name, style) {
  const pascal = () => name.split("_").filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1)).join("");
  switch (style) {
    case "camelCase":
      return pascal().replace(/^./, (first) => first.toLowerCase());
    case "PascalCase":
      return pascal();
    case "kebab-case":
      return name.replaceAll("_", "-");
    case "SCREAMING-KEBAB-CASE":
      return name.replaceAll("_", "-").toUpperCase();
    case "UPPERCASE":
    case "SCREAMING_SNAKE_CASE":
      return name.toUpperCase();
    default:
      return name;
  }
}

function rustFiles(directory, collected = []) {
  let entries;
  try {
    entries = readdirSync(directory, { withFileTypes: true });
  } catch {
    return collected;
  }
  for (const entry of entries) {
    if (entry.name === "target" || entry.name === "node_modules" || entry.name === ".git") continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) rustFiles(path, collected);
    else if (entry.name.endsWith(".rs")) collected.push(path);
  }
  return collected;
}

function parseStructFields(body) {
  const fields = [];
  let pendingRename = null;
  let pendingDefault = false;
  for (const rawLine of body.split("\n")) {
    const line = rawLine.trim();
    if (line.startsWith("#[")) {
      pendingRename = line.match(/\brename\s*=\s*"([^"]+)"/)?.[1] ?? pendingRename;
      if (/\bdefault\b/.test(line)) pendingDefault = true;
      continue;
    }
    if (line.startsWith("//") || line === "") continue;
    // `r#` is not decoration: attendance's AmendCloseBody declares `r#ref`, and serde publishes
    // it as `ref`. Dropping the field made the gate report `ref` as absent from the struct — a
    // false finding, which is the loud direction of the same degradation the anchors guard.
    const field = line.match(/^(?:pub(?:\([^)]*\))?\s+)?(?:r#)?([a-z0-9_]+)\s*:\s*(.+?),?$/);
    if (field) {
      fields.push({ name: field[1], type: field[2], rename: pendingRename, hasDefault: pendingDefault });
    }
    pendingRename = null;
    pendingDefault = false;
  }
  return fields;
}

function collectSources(repoRoot) {
  const consts = new Map();
  const structs = new Map();
  const handlers = new Map();
  const routes = [];

  for (const file of rustFiles(join(repoRoot, "backend"))) {
    const source = readFileSync(file, "utf8");
    const key = relative(repoRoot, file);

    for (const match of source.matchAll(CONST_PATH)) consts.set(match[1], match[2]);

    for (const match of source.matchAll(STRUCT)) {
      const attrs = match[1];
      structs.set(`${key}::${match[2]}`, {
        name: match[2],
        denyUnknown: /\bdeny_unknown_fields\b/.test(attrs),
        renameAll: attrs.match(RENAME_ALL)?.[1] ?? null,
        fields: parseStructFields(match[3]),
      });
    }

    for (const match of source.matchAll(HANDLER)) {
      handlers.set(`${key}::${match[1]}`, match[2].match(JSON_BODY)?.[1] ?? null);
    }

    for (const match of source.matchAll(ROUTE)) {
      for (const method of match[2].matchAll(METHOD)) {
        routes.push({ file: key, constName: match[1], method: method[1], handler: method[2] });
      }
    }
  }
  return { consts, structs, handlers, routes };
}

function jsonRequestSchema(document, path, method) {
  const schema = document.paths?.[path]?.[method]?.requestBody?.content?.["application/json"]?.schema;
  if (!schema) return null;
  if (!schema.$ref) return schema;
  const name = schema.$ref.replace("#/components/schemas/", "");
  return document.components?.schemas?.[name] ?? null;
}

/**
 * @param {{ repoRoot: string, openApiPath?: string }} options
 * @returns {{
 *   resolved: number,
 *   skipped: number,
 *   findings: { operation: string, message: string }[],
 *   unresolvedAnchors: string[],
 * }}
 */
export function evaluateRequestBodyContract({ repoRoot, openApiPath = "backend/openapi/openapi.yaml" }) {
  const document = yaml.load(readFileSync(join(repoRoot, openApiPath), "utf8"));
  const { consts, structs, handlers, routes } = collectSources(repoRoot);
  const findings = [];
  const resolvedOperations = new Set();
  let skipped = 0;

  for (const route of routes) {
    const path = consts.get(route.constName);
    if (!path) continue;
    const bodyType = handlers.get(`${route.file}::${route.handler}`);
    if (!bodyType) continue;
    const operation = `${route.method.toUpperCase()} ${path}`;
    const schema = jsonRequestSchema(document, path, route.method);
    if (!schema?.properties) {
      skipped += 1;
      continue;
    }
    // `AssignBody` exists in two crates; the handler's own file decides which one it binds.
    const struct = structs.get(`${route.file}::${bodyType}`)
      ?? [...structs.values()].find((candidate) => candidate.name === bodyType);
    if (!struct || !struct.denyUnknown) {
      skipped += 1;
      continue;
    }
    resolvedOperations.add(operation);

    const wireName = (field) => field.rename ?? renameField(field.name, struct.renameAll);
    const wireNames = new Set(struct.fields.map(wireName));
    const specRequired = new Set(schema.required ?? []);

    for (const property of Object.keys(schema.properties)) {
      if (!wireNames.has(property)) {
        findings.push({
          operation,
          message: `spec property "${property}" is not a field of ${struct.name} (deny_unknown_fields => 422)`,
        });
      }
    }
    for (const field of struct.fields) {
      if (field.type.startsWith("Option<") || field.hasDefault) continue;
      if (specRequired.has(wireName(field))) continue;
      // When the struct's own rust name is itself a spec property, the loop above already
      // reported that the spec publishes this field under a name the struct rejects. The absent
      // `required[]` entry is the same defect seen from the other side, not a second one.
      if (schema.properties[field.name] !== undefined) continue;
      findings.push({
        operation,
        message: `${struct.name}.${field.name} is required by the handler but not in spec required[]`,
      });
    }
  }

  return {
    resolved: resolvedOperations.size,
    skipped,
    findings,
    unresolvedAnchors: ANCHORS.filter((anchor) => !resolvedOperations.has(anchor)),
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const { resolved, skipped, findings, unresolvedAnchors } = evaluateRequestBodyContract({ repoRoot });
  for (const finding of findings) console.error(`${finding.operation}: ${finding.message}`);
  for (const anchor of unresolvedAnchors) {
    console.error(`anchor operation ${anchor} no longer resolves — the resolver has silently degraded`);
  }
  if (resolved < RESOLVED_FLOOR) {
    console.error(`resolved ${resolved} operations, below the floor of ${RESOLVED_FLOOR} — `
      + "the resolver compared less of the surface than it was built to compare");
  }
  if (findings.length > 0 || unresolvedAnchors.length > 0 || resolved < RESOLVED_FLOOR) {
    console.error(`request body contract gate FAILED: ${findings.length} finding(s), `
      + `resolved ${resolved}, skipped ${skipped}`);
    process.exit(1);
  }
  console.log(`request body contract gate passed (resolved ${resolved}, skipped ${skipped})`);
}
