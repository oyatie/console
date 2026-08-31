// Typed execute-path gate: the ontology action execute/preflight request body
// must $ref the thirteen DispatchTarget input schemas on the wire.
//
// The hole this closes: #996 published schema-level `actions[].input` $refs, but
// POST /api/v1/ontology/actions/{action_key}/execute still $refs
// OntologyActionRequest with `params: { additionalProperties: true }`. That is
// type erasure — the generic codec, not the typed contract. PRODUCT requires
// typed objects/links/actions/permissions that generate OpenAPI; this slice
// makes the served execute body name those types. The Rust handler is a
// sibling fail-closed parse (unknown fields / wrong action_key), proven by
// `cargo test -p console-ontology-rest --lib typed_action`.
//
// Chesterton: generic instance-revision actions (`create`, `set_priority`, …)
// stay on this path. A catch-all object member may exist beside the thirteen
// $refs. The thirteen must still appear. Do not leave this gate green while
// twelve actions still use free-form params.
//
// Totality: js-yaml load + own-property reads. A walker that visits nothing
// reports nothing, so ACTION_FLOOR is the examined-zero lock.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { CANONICAL_ACTIONS, ACTION_FLOOR } from "./check-openapi-semantic-contract.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { ACTION_FLOOR };

export const EXECUTE_PATH = "/api/v1/ontology/actions/{action_key}/execute";
export const PREFLIGHT_PATH = "/api/v1/ontology/actions/{action_key}/preflight";
export const TYPED_EXECUTE_PATHS = Object.freeze([EXECUTE_PATH, PREFLIGHT_PATH]);

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

function propertyMap(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? properties : null;
}

function isFreeFormObject(schema) {
  if (!isPlainObject(schema)) return false;
  if (own(schema, "additionalProperties") !== true) return false;
  const properties = propertyMap(schema);
  return !properties || Object.keys(properties).length === 0;
}

function push(findings, location, message) {
  findings.push({ location, message });
}

function resolveRef(schemas, schema) {
  const name = schemaRefName(schema);
  if (name === null) return { name: null, schema };
  return { name, schema: own(schemas, name) };
}

function combinatorMembers(schema) {
  if (!isPlainObject(schema)) return [];
  const members = [];
  for (const key of ["anyOf", "oneOf", "allOf"]) {
    const list = own(schema, key);
    if (Array.isArray(list)) members.push(...list);
  }
  return members;
}

function collectRefNames(schema, acc = new Set()) {
  if (!isPlainObject(schema)) return acc;
  const name = schemaRefName(schema);
  if (name) acc.add(name);
  for (const member of combinatorMembers(schema)) collectRefNames(member, acc);
  return acc;
}

function operationRequestSchema(document, path) {
  const paths = own(document, "paths");
  const item = own(paths, path);
  const post = own(item, "post");
  const media = own(own(own(post, "requestBody"), "content"), "application/json");
  return own(media, "schema");
}

function paramsSchemaOfEnvelope(schemas, envelope) {
  if (!isPlainObject(envelope)) return null;
  const properties = propertyMap(envelope);
  if (!properties) return null;
  const params = own(properties, "params");
  if (!isPlainObject(params)) return null;
  const resolved = resolveRef(schemas, params);
  return resolved.schema ?? params;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   actions: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateTypedExecuteContract({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const components = own(document, "components");
  const schemas = own(components, "schemas");
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { actions: 0, findings };
  }

  const seen = new Set();

  for (const path of TYPED_EXECUTE_PATHS) {
    const loc = `#/paths/${path}/post/requestBody`;
    const raw = operationRequestSchema(document, path);
    if (!isPlainObject(raw)) {
      push(findings, loc, "execute/preflight request body is absent");
      continue;
    }

    const resolved = resolveRef(schemas, raw);
    const envelope = resolved.schema;
    const envelopeName = resolved.name ?? "(inline)";
    const envelopeLoc = resolved.name
      ? `#/components/schemas/${resolved.name}`
      : loc;

    if (!isPlainObject(envelope)) {
      push(findings, loc, "request body $ref does not resolve");
      continue;
    }

    if (own(envelope, "additionalProperties") !== false) {
      push(
        findings,
        `${envelopeLoc}/additionalProperties`,
        "typed execute envelope must set additionalProperties: false (fail-closed unknown fields)",
      );
    }

    const params = paramsSchemaOfEnvelope(schemas, envelope);
    const paramsLoc = `${envelopeLoc}/properties/params`;
    if (!isPlainObject(params)) {
      push(findings, paramsLoc, "execute envelope has no params schema");
      continue;
    }

    if (isFreeFormObject(params)) {
      push(
        findings,
        paramsLoc,
        "OntologyActionRequest.params additionalProperties: true erases the thirteen typed inputs",
      );
      continue;
    }

    const refs = collectRefNames(params);
    const combinators = combinatorMembers(params);
    if (combinators.length === 0 && refs.size === 0) {
      push(
        findings,
        paramsLoc,
        "params must anyOf/oneOf the thirteen typed action input $refs (or a $ref to a union of them)",
      );
      continue;
    }

    for (const action of CANONICAL_ACTIONS) {
      if (!refs.has(action.input)) {
        push(
          findings,
          `${paramsLoc}/${action.action_key}`,
          `must $ref #/components/schemas/${action.input} on the execute wire, not OntologyActionRequest.params`,
        );
        continue;
      }
      seen.add(action.action_key);

      const inputSchema = own(schemas, action.input);
      const inputLoc = `#/components/schemas/${action.input}`;
      if (!isPlainObject(inputSchema)) {
        push(findings, inputLoc, "typed action input schema is absent");
        continue;
      }
      if (own(inputSchema, "type") !== "object") {
        push(findings, inputLoc, "typed action input must be type: object");
      }
      if (own(inputSchema, "additionalProperties") !== false) {
        push(
          findings,
          `${inputLoc}/additionalProperties`,
          "typed action input must set additionalProperties: false (fail-closed unknown fields)",
        );
      }
      if (isFreeFormObject(inputSchema)) {
        push(findings, inputLoc, "typed action input must not be a free-form object");
      }
    }

    if (envelopeName === "OntologyActionRequest" && isFreeFormObject(params)) {
      push(
        findings,
        paramsLoc,
        "generic execute codec still takes OntologyActionRequest.params on the wire",
      );
    }
  }

  const actions = seen.size;
  if (actions > 0 && actions !== ACTION_FLOOR) {
    const missing = CANONICAL_ACTIONS.map((action) => action.action_key).filter(
      (key) => !seen.has(key),
    );
    if (missing.length > 0) {
      push(
        findings,
        "#/paths/~1api~1v1~1ontology~1actions~1{action_key}~1execute/post/requestBody",
        `typed execute roster is ${actions}/${ACTION_FLOOR}; missing ${missing.join(", ")}`,
      );
    }
  }

  return { actions, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateTypedExecuteContract({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { actions, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = actions < ACTION_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${actions}/${ACTION_FLOOR} typed execute actions — below the floor; `
        + `OntologyActionRequest.params is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi typed-execute gate FAILED: ${findings.length} finding(s), `
        + `${actions}/${ACTION_FLOOR} actions`,
    );
    process.exit(1);
  }
  console.log(
    `openapi typed-execute gate passed `
      + `(${actions}/${ACTION_FLOOR} actions on execute+preflight, 0 findings)`,
  );
}
