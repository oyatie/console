// Head concurrency published-contract gate.
//
// The hole this closes: Company / OrgUnit / JobPosition / Person already
// serialize `version`, and every roster action already declares
// `concurrency.expected_revision: optional_cas` plus OntologyActionRequest
// `expected_revision` on execute/preflight. Head GET still does not publish
// that token as a generated contract. Dual-written YAML ETag/If-Match headers
// would be a second token beside the JSON fields the runtime already uses.
//
// Chesterton: ontology object-type key writes already speak HTTP ETag/If-Match
// (`key_write_etag`). Canonical Head writes speak JSON `expected_revision`.
// ROADMAP lists uniform HTTP ETag as unscheduled. Five of six Heads have no
// GET operation to hang a header on; Employment GET does not serialize
// version. Generate the GET token from the DTO `version` field (null when the
// Head does not serialize one) and document writes as the existing body field.
// Do not invent a parallel ETag type or a new revision store.
//
// Totality: js-yaml load + own-property walk of every Head schema and GET, plus
// a text scan of the DTO inventory and semantic emitter when those files exist.
// A walker that visits nothing reports nothing, so HEAD_FLOOR / GET_FLOOR lock
// examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  CONCURRENCY_EXPECTED_REVISION,
} from "./check-openapi-semantic-contract.mjs";
import {
  DTO_RS_REL,
  HEAD_SCHEMA_NAMES,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import {
  EXECUTE_PATH,
  PREFLIGHT_PATH,
} from "./check-openapi-typed-execute.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const HEAD_FLOOR = HEAD_SCHEMA_NAMES.length;
export const GET_FLOOR = 200;
export const WRITE_IN_BODY = "body";
export const GET_TOKEN_VERSION = "version";
export const WRITE_FIELD = "expected_revision";

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

const WRITE_PATHS = Object.freeze([EXECUTE_PATH, PREFLIGHT_PATH]);

function push(findings, location, message) {
  findings.push({ location, message });
}

function readText(repoRoot, rel) {
  const path = join(repoRoot, rel);
  if (!existsSync(path)) return { path, missing: true, text: "" };
  return { path, missing: false, text: readFileSync(path, "utf8") };
}

function rustStructFields(source, structName) {
  const escaped = structName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(new RegExp(String.raw`pub struct ${escaped}\s*\{([\s\S]*?)\n\}`));
  if (!match) return null;
  const fields = [];
  const fieldRe = /^\s*(?:#\[[^\]]+\]\s*)*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/gm;
  let fieldMatch;
  while ((fieldMatch = fieldRe.exec(match[1])) !== null) {
    fields.push(fieldMatch[1]);
  }
  return fields;
}

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

function json200(operation) {
  const responses = own(operation, "responses");
  return own(responses, "200") ?? own(responses, 200);
}

function json200Schema(operation) {
  const content = own(json200(operation), "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function headNamesFromSchema(schema) {
  const names = [];
  const direct = schemaRefName(schema);
  if (HEAD_SCHEMA_NAMES.includes(direct)) names.push(direct);
  if (isPlainObject(schema) && own(schema, "type") === "array") {
    const items = schemaRefName(own(schema, "items"));
    if (HEAD_SCHEMA_NAMES.includes(items)) names.push(items);
  }
  return names;
}

function parameterList(operation) {
  const parameters = own(operation, "parameters");
  return Array.isArray(parameters) ? parameters : [];
}

function hasIfMatchHeader(operation) {
  return parameterList(operation).some(
    (param) =>
      isPlainObject(param)
      && own(param, "name") === "If-Match"
      && own(param, "in") === "header",
  );
}

function hasEtagHeader(operation) {
  const headers = own(json200(operation), "headers");
  if (!isPlainObject(headers)) return false;
  return hasOwnKey(headers, "ETag") || hasOwnKey(headers, "etag");
}

function envelopeHasExpectedRevision(schemas, envelope) {
  const name = schemaRefName(envelope);
  const schema = name ? own(schemas, name) : envelope;
  const properties = propertyMap(schema);
  return isPlainObject(properties) && hasOwnKey(properties, WRITE_FIELD);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   heads: number,
 *   gets: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHeadEtag({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const components = own(document, "components");
  const schemas = own(components, "schemas");
  const paths = own(document, "paths");
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { heads: 0, gets: 0, findings };
  }

  const dto = readText(repoRoot, DTO_RS_REL);
  const dtoVersionByHead = new Map();
  if (!dto.missing) {
    for (const name of HEAD_SCHEMA_NAMES) {
      const fields = rustStructFields(dto.text, name);
      if (!fields) {
        push(
          findings,
          `${DTO_RS_REL}:${name}`,
          "DTO inventory does not declare this Head struct; cannot derive the GET concurrency token",
        );
        continue;
      }
      dtoVersionByHead.set(name, fields.includes("version"));
    }
  }

  const semantic = readText(repoRoot, SEMANTIC_RS_REL);
  if (!semantic.missing) {
    if (
      !semantic.text.includes("get_token")
      || !semantic.text.includes("write_field")
      || !semantic.text.includes("write_in")
    ) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "generated_schema_yaml must inject Head concurrency from DTO version fields "
          + `(get_token / write_field=${WRITE_FIELD} / write_in=${WRITE_IN_BODY}); `
          + "hand YAML ETag/If-Match headers are a second token",
      );
    }
  }

  let heads = 0;
  for (const name of HEAD_SCHEMA_NAMES) {
    const location = `#/components/schemas/${name}`;
    if (!hasOwnKey(schemas, name)) {
      push(findings, location, "canonical Head schema is absent");
      continue;
    }
    const schema = own(schemas, name);
    if (!isPlainObject(schema) || own(schema, "type") !== "object") {
      push(findings, location, "canonical Head schema must be type: object");
      continue;
    }

    const properties = propertyMap(schema) ?? {};
    if (hasOwnKey(properties, "concurrency")) {
      push(
        findings,
        `${location}/properties/concurrency`,
        "concurrency must be a schema-level ontology keyword, not an instance property",
      );
    }

    const hasVersionProperty = hasOwnKey(properties, "version");
    const expectedToken = dtoVersionByHead.has(name)
      ? (dtoVersionByHead.get(name) ? GET_TOKEN_VERSION : null)
      : (hasVersionProperty ? GET_TOKEN_VERSION : null);

    if (dtoVersionByHead.has(name) && dtoVersionByHead.get(name) !== hasVersionProperty) {
      push(
        findings,
        `${location}/properties/version`,
        dtoVersionByHead.get(name)
          ? "DTO Head serializes version; the published schema must carry that GET concurrency token"
          : "DTO Head does not serialize version; do not invent a GET concurrency token or revision store",
      );
    }

    const concurrency = own(schema, "concurrency");
    if (!isPlainObject(concurrency)) {
      push(
        findings,
        `${location}/concurrency`,
        "schema-level concurrency is absent; Head GET must publish the DTO version token "
          + `(or null when the Head does not serialize version) and writes use ${WRITE_FIELD} in the JSON body`,
      );
      continue;
    }
    heads += 1;

    const getToken = own(concurrency, "get_token");
    if (getToken !== expectedToken) {
      push(
        findings,
        `${location}/concurrency/get_token`,
        `must be ${JSON.stringify(expectedToken)} from DTO/runtime version presence, got ${JSON.stringify(getToken)}`,
      );
    }
    if (own(concurrency, "write_field") !== WRITE_FIELD) {
      push(
        findings,
        `${location}/concurrency/write_field`,
        `must be ${WRITE_FIELD} (OntologyActionRequest body field the runtime already CAS-checks)`,
      );
    }
    if (own(concurrency, "write_in") !== WRITE_IN_BODY) {
      push(
        findings,
        `${location}/concurrency/write_in`,
        `must be ${WRITE_IN_BODY}; HTTP If-Match would duplicate expected_revision`,
      );
    }

    const declaredActions = own(schema, "actions");
    if (Array.isArray(declaredActions)) {
      for (const [index, action] of declaredActions.entries()) {
        if (!isPlainObject(action)) continue;
        const actionKey = own(action, "action_key");
        const actionLoc = `${location}/actions/${typeof actionKey === "string" ? actionKey : index}`;
        const actionConcurrency = own(action, "concurrency");
        if (!isPlainObject(actionConcurrency)) {
          push(
            findings,
            `${actionLoc}/concurrency`,
            "roster action must keep expected_revision CAS",
          );
          continue;
        }
        if (own(actionConcurrency, "expected_revision") !== CONCURRENCY_EXPECTED_REVISION) {
          push(
            findings,
            `${actionLoc}/concurrency/expected_revision`,
            `must stay ${CONCURRENCY_EXPECTED_REVISION}; do not replace the body field with If-Match`,
          );
        }
      }
    }
  }

  let gets = 0;
  if (isPlainObject(paths)) {
    for (const path of Object.keys(paths)) {
      const item = own(paths, path);
      if (!isPlainObject(item)) continue;
      for (const method of HTTP_METHODS) {
        if (method !== "get") continue;
        const operation = own(item, method);
        if (!isPlainObject(operation)) continue;
        gets += 1;
        const location = `#/paths/${path}/get`;
        const headNames = headNamesFromSchema(json200Schema(operation));
        if (headNames.length === 0) continue;
        if (hasEtagHeader(operation)) {
          push(
            findings,
            `${location}/responses/200/headers/ETag`,
            `HTTP ETag would duplicate Head ${headNames.join(", ")} concurrency.get_token / version; `
              + "the published token is the JSON field generated from the DTO",
          );
        }
      }

      const post = own(item, "post");
      if (!isPlainObject(post)) continue;
      if (!WRITE_PATHS.includes(path)) continue;
      const location = `#/paths/${path}/post`;
      if (hasIfMatchHeader(post)) {
        push(
          findings,
          `${location}/parameters/If-Match`,
          `HTTP If-Match would duplicate OntologyActionRequest.${WRITE_FIELD}; `
            + "roster writes already CAS that body field",
        );
      }
      const media = own(own(own(post, "requestBody"), "content"), "application/json");
      const envelope = own(media, "schema");
      if (!envelopeHasExpectedRevision(schemas, envelope)) {
        push(
          findings,
          `${location}/requestBody/${WRITE_FIELD}`,
          `mutating roster path must keep ${WRITE_FIELD} on the JSON envelope (If-Match equivalent)`,
        );
      }
    }
  }

  if (heads < HEAD_FLOOR && !findings.some((finding) => finding.location.endsWith("/concurrency"))) {
    push(
      findings,
      "#/components/schemas",
      `Head concurrency is published on ${heads} of ${HEAD_FLOOR} Heads; `
        + "GET token must be generated from DTO version presence",
    );
  }

  return { heads, gets, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHeadEtag({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { heads, gets, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor || heads < HEAD_FLOOR) {
    console.error(
      `openapi Head concurrency gate FAILED: ${findings.length} finding(s), `
        + `${heads}/${HEAD_FLOOR} Heads, ${gets} GET(s)`,
    );
    process.exit(1);
  }
  console.log(
    `openapi Head concurrency gate passed `
      + `(${heads}/${HEAD_FLOOR} Heads from DTO version/expected_revision, `
      + `${gets} GET operations, 0 findings; HTTP ETag/If-Match not duplicated)`,
  );
}
