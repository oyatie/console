// Generated API docs gate (ADR-0031 docs slice).
//
// The hole this closes: composed OpenAPI is regenerated in CI (`console-openapi-gen`
// + `git diff --exit-code`) and the TypeScript SDK is generated from that
// document, but no docs artifact is generated from it. PRODUCT requires typed
// objects/links/actions that generate OpenAPI/validators/docs/SDKs. Dual-written
// YAML + a hand-authored docs site is not that.
//
// Chesterton: extend the OpenAPI regen+diff pattern. Do not replace compose.
// This gate reads committed `backend/openapi/openapi.yaml`, generates the
// in-repo HTML docs, and fails closed on drift. Types, links, actions, and
// operations come from the composed document, not a parallel field catalog.
//
// Totality: js-yaml load + own-property reads of composed schemas/paths + byte
// compare of the generated docs file. A walker that visits nothing reports
// nothing, so HEAD_TYPE_FLOOR / INPUT_TYPE_FLOOR / OPERATION_FLOOR lock
// examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { PERSON_FORBIDDEN_FIELDS } from "./check-openapi-semantic-contract.mjs";
import {
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  NESTED_INPUT_SCHEMAS,
} from "./check-openapi-semantic-generate.mjs";
import {
  DOCS_DIR_REL,
  DOCS_FILE_RELS,
  DOCS_REL,
  GENERATED_HEADER,
  OPENAPI_REL,
  collectOperations,
  generateDocsFiles,
} from "./generate-openapi-docs.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export const HEAD_TYPE_FLOOR = HEAD_SCHEMA_NAMES.length;
export const INPUT_TYPE_FLOOR = INPUT_SCHEMA_NAMES.length;
export const NESTED_TYPE_FLOOR = NESTED_INPUT_SCHEMAS.length;
export const OPERATION_FLOOR = 1;

function push(findings, location, message) {
  findings.push({ location, message });
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function schemaSection(html, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = html.match(
    new RegExp(`<section[^>]*id="schema-${escaped}"[\\s\\S]*?</section>`),
  );
  return match ? match[0] : null;
}

function sectionPropertyNames(section) {
  if (typeof section !== "string") return [];
  return [...section.matchAll(/\sdata-property="([^"]+)"/g)].map((match) => match[1]);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   heads: number,
 *   inputs: number,
 *   nested: number,
 *   operations: number,
 *   files: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateGeneratedDocs({ repoRoot }) {
  const findings = [];
  const openapiPath = join(repoRoot, OPENAPI_REL);
  if (!existsSync(openapiPath)) {
    push(findings, OPENAPI_REL, "composed OpenAPI document is missing; cannot generate docs");
    return { heads: 0, inputs: 0, nested: 0, operations: 0, files: 0, findings };
  }

  let document;
  try {
    document = yaml.load(readFileSync(openapiPath, "utf8"));
  } catch (error) {
    push(findings, OPENAPI_REL, `cannot parse: ${error.message}`);
    return { heads: 0, inputs: 0, nested: 0, operations: 0, files: 0, findings };
  }

  const published = own(own(document, "components"), "schemas");
  if (!isPlainObject(published)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { heads: 0, inputs: 0, nested: 0, operations: 0, files: 0, findings };
  }

  const generated = generateDocsFiles(document);
  let files = 0;
  for (const rel of DOCS_FILE_RELS) {
    const expected = own(generated.files, rel);
    const path = join(repoRoot, rel);
    if (!existsSync(path)) {
      push(findings, rel, "generated docs file is absent; composed OpenAPI has no docs artifact");
      continue;
    }
    const actual = readFileSync(path, "utf8");
    if (actual !== expected) {
      push(
        findings,
        rel,
        "generated docs drifted from composed OpenAPI (regen is not a no-op; do not hand-edit)",
      );
      continue;
    }
    files += 1;
  }

  const html = existsSync(join(repoRoot, DOCS_REL))
    ? readFileSync(join(repoRoot, DOCS_REL), "utf8")
    : own(generated.files, DOCS_REL) ?? "";

  if (!html.includes(GENERATED_HEADER)) {
    push(findings, DOCS_REL, "generated docs must declare they are produced from backend/openapi/openapi.yaml");
  }

  let heads = 0;
  let inputs = 0;
  let nested = 0;

  for (const name of GENERATED_SCHEMA_NAMES) {
    const schema = own(published, name);
    const loc = `#/components/schemas/${name}`;
    if (!isPlainObject(schema)) {
      push(findings, loc, "composed document is missing a Head/Input schema the docs must describe");
      continue;
    }
    const section = schemaSection(html, name);
    if (section === null) {
      push(
        findings,
        `${DOCS_REL}:${name}`,
        "generated docs do not describe this Head/Input from the composed OpenAPI schema",
      );
      continue;
    }

    const openapiFields = schemaPropertyNames(schema);
    const htmlFields = sectionPropertyNames(section);
    if (openapiFields.length > 0) {
      const missing = openapiFields.filter((field) => !htmlFields.includes(field));
      const extra = htmlFields.filter((field) => !openapiFields.includes(field));
      if (missing.length > 0 || extra.length > 0) {
        push(
          findings,
          `${DOCS_REL}:${name}`,
          `docs drifted from composed schema properties: missing [${missing.join(", ")}] extra [${extra.join(", ")}]`,
        );
        continue;
      }
    }

    if (HEAD_SCHEMA_NAMES.includes(name)) {
      heads += 1;
      const actions = own(schema, "actions");
      if (Array.isArray(actions)) {
        for (const action of actions) {
          const key = typeof own(action, "action_key") === "string" ? own(action, "action_key") : "";
          if (key !== "" && !section.includes(`data-action="${key}"`)) {
            push(
              findings,
              `${DOCS_REL}:${name}:${key}`,
              "Head docs must list this action_key from the composed schema",
            );
          }
        }
      }
    } else if (INPUT_SCHEMA_NAMES.includes(name)) {
      inputs += 1;
    } else if (NESTED_INPUT_SCHEMAS.includes(name)) {
      nested += 1;
    }
  }

  const personSection = schemaSection(html, "Person");
  if (personSection) {
    const fields = sectionPropertyNames(personSection);
    for (const forbidden of PERSON_FORBIDDEN_FIELDS) {
      if (fields.includes(forbidden)) {
        push(
          findings,
          `${DOCS_REL}:Person`,
          `Person closed projection must not grow ${forbidden}`,
        );
      }
    }
  }

  const payRunSection = schemaSection(html, "PayRun");
  if (payRunSection && !/data-property="payable"[^>]*data-const="false"/.test(payRunSection)) {
    push(
      findings,
      `${DOCS_REL}:PayRun`,
      "PayRun.payable must stay literal false in the generated docs",
    );
  }

  const operations = collectOperations(document);
  for (const operation of operations) {
    const key = `${operation.method} ${operation.path}`;
    if (!html.includes(`data-operation="${key}"`)) {
      push(
        findings,
        `${DOCS_REL}:${key}`,
        "generated docs omit this composed OpenAPI operation",
      );
    }
  }

  const belowFloor =
    heads < HEAD_TYPE_FLOOR
    || inputs < INPUT_TYPE_FLOOR
    || nested < NESTED_TYPE_FLOOR
    || operations.length < OPERATION_FLOOR
    || files < DOCS_FILE_RELS.length;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      DOCS_DIR_REL,
      `generated ${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${operations.length} operations, `
        + `${files}/${DOCS_FILE_RELS.length} files — below the floor`,
    );
  }

  return { heads, inputs, nested, operations: operations.length, files, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateGeneratedDocs({ repoRoot });
  } catch (error) {
    console.error(`generated docs gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { heads, inputs, nested, operations, files, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    heads < HEAD_TYPE_FLOOR
    || inputs < INPUT_TYPE_FLOOR
    || nested < NESTED_TYPE_FLOOR
    || operations < OPERATION_FLOOR
    || files < DOCS_FILE_RELS.length;
  if (belowFloor) {
    console.error(
      `saw ${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${operations} operations, `
        + `${files}/${DOCS_FILE_RELS.length} files — below the floor; `
        + `composed YAML without generated docs is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi generated-docs gate FAILED: ${findings.length} finding(s), `
        + `${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${operations} operations, `
        + `${files}/${DOCS_FILE_RELS.length} files`,
    );
    process.exit(1);
  }
  console.log(
    `openapi generated-docs gate passed `
      + `(${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
      + `${nested}/${NESTED_TYPE_FLOOR} nested, ${operations} operations, `
      + `${files}/${DOCS_FILE_RELS.length} files from ${OPENAPI_REL}, 0 findings)`,
  );
}
