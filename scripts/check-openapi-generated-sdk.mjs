// Generated TypeScript SDK gate (ADR-0031 SDK slice).
//
// The hole this closes: composed OpenAPI is regenerated in CI (`console-openapi-gen`
// + `git diff --exit-code`), but no consumer artifact is generated from that
// document. PRODUCT requires typed objects/links/actions that generate
// OpenAPI/validators/docs/SDKs. Dual-written YAML + hand TS is not that.
//
// Chesterton: extend the OpenAPI regen+diff pattern. Do not replace compose.
// This gate reads committed `backend/openapi/openapi.yaml`, generates the
// in-repo TypeScript package, and fails closed on drift. Types for the six
// Heads and thirteen Inputs come from the composed schemas, not a parallel
// field catalog.
//
// Totality: js-yaml load + own-property reads of composed schemas + byte
// compare of generated SDK files. A walker that visits nothing reports
// nothing, so HEAD_TYPE_FLOOR / INPUT_TYPE_FLOOR lock examined-zero.

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
  GENERATED_HEADER,
  OPENAPI_REL,
  SDK_DIR_REL,
  SDK_FILE_RELS,
  SDK_GENERATED_REL,
  generateSdkFiles,
} from "./generate-openapi-ts-sdk.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export const HEAD_TYPE_FLOOR = HEAD_SCHEMA_NAMES.length;
export const INPUT_TYPE_FLOOR = INPUT_SCHEMA_NAMES.length;
export const NESTED_TYPE_FLOOR = NESTED_INPUT_SCHEMAS.length;
export const SDK_TYPE_FLOOR = GENERATED_SCHEMA_NAMES.length;

function push(findings, location, message) {
  findings.push({ location, message });
}

function schemaPropertyNames(schema) {
  const properties = own(schema, "properties");
  return isPlainObject(properties) ? Object.keys(properties) : [];
}

function exportedTypeBlock(source, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(
    new RegExp(String.raw`export type ${escaped}\s*=\s*(\{[\s\S]*?\n\}|[^;]+);`),
  );
  return match ? match[1] : null;
}

function typeBlockPropertyNames(block) {
  if (typeof block !== "string" || !block.startsWith("{")) return [];
  const close = block.lastIndexOf("}");
  const inner = close > 0 ? block.slice(1, close) : block.slice(1);
  const names = [];
  let depth = 0;
  for (const line of inner.split("\n")) {
    if (depth === 0) {
      const match = line.match(
        /^\s*(?:readonly\s+)?(?:(["'][^"']+["'])|([A-Za-z_$][A-Za-z0-9_$]*))\??\s*:/,
      );
      if (match) {
        const raw = match[1] ?? match[2];
        names.push(raw.startsWith("\"") || raw.startsWith("'") ? raw.slice(1, -1) : raw);
      }
    }
    depth += (line.match(/\{/g) ?? []).length - (line.match(/\}/g) ?? []).length;
  }
  return names;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   heads: number,
 *   inputs: number,
 *   nested: number,
 *   files: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateGeneratedSdk({ repoRoot }) {
  const findings = [];
  const openapiPath = join(repoRoot, OPENAPI_REL);
  if (!existsSync(openapiPath)) {
    push(findings, OPENAPI_REL, "composed OpenAPI document is missing; cannot generate an SDK");
    return { heads: 0, inputs: 0, nested: 0, files: 0, findings };
  }

  let document;
  try {
    document = yaml.load(readFileSync(openapiPath, "utf8"));
  } catch (error) {
    push(findings, OPENAPI_REL, `cannot parse: ${error.message}`);
    return { heads: 0, inputs: 0, nested: 0, files: 0, findings };
  }

  const published = own(own(document, "components"), "schemas");
  if (!isPlainObject(published)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { heads: 0, inputs: 0, nested: 0, files: 0, findings };
  }

  const generated = generateSdkFiles(document);
  let files = 0;
  for (const rel of SDK_FILE_RELS) {
    const expected = own(generated.files, rel);
    const path = join(repoRoot, rel);
    if (!existsSync(path)) {
      push(findings, rel, "generated SDK file is absent; composed OpenAPI has no consumer artifact");
      continue;
    }
    const actual = readFileSync(path, "utf8");
    if (actual !== expected) {
      push(
        findings,
        rel,
        "generated SDK drifted from composed OpenAPI (regen is not a no-op; do not hand-edit)",
      );
      continue;
    }
    files += 1;
  }

  const generatedTs = existsSync(join(repoRoot, SDK_GENERATED_REL))
    ? readFileSync(join(repoRoot, SDK_GENERATED_REL), "utf8")
    : own(generated.files, SDK_GENERATED_REL) ?? "";

  if (!generatedTs.includes(GENERATED_HEADER.split("\n")[0])) {
    push(findings, SDK_GENERATED_REL, "generated SDK must declare it is produced from backend/openapi/openapi.yaml");
  }

  let heads = 0;
  let inputs = 0;
  let nested = 0;

  for (const name of GENERATED_SCHEMA_NAMES) {
    const schema = own(published, name);
    const loc = `#/components/schemas/${name}`;
    if (!isPlainObject(schema)) {
      push(findings, loc, "composed document is missing a Head/Input schema the SDK must type");
      continue;
    }
    const block = exportedTypeBlock(generatedTs, name);
    if (block === null) {
      push(
        findings,
        `${SDK_GENERATED_REL}:${name}`,
        "typed SDK does not export this Head/Input from the composed OpenAPI schema",
      );
      continue;
    }

    const openapiFields = schemaPropertyNames(schema);
    const tsFields = typeBlockPropertyNames(block);
    if (openapiFields.length > 0) {
      const missing = openapiFields.filter((field) => !tsFields.includes(field));
      const extra = tsFields.filter((field) => !openapiFields.includes(field));
      if (missing.length > 0 || extra.length > 0) {
        push(
          findings,
          `${SDK_GENERATED_REL}:${name}`,
          `SDK type drifted from composed schema properties: missing [${missing.join(", ")}] extra [${extra.join(", ")}]`,
        );
        continue;
      }
    }

    if (HEAD_SCHEMA_NAMES.includes(name)) {
      heads += 1;
      if (!generatedTs.includes(`export const ${name}Definition`)) {
        push(
          findings,
          `${SDK_GENERATED_REL}:${name}Definition`,
          "Head SDK type must export links/actions metadata from the composed schema",
        );
      }
    } else if (INPUT_SCHEMA_NAMES.includes(name)) {
      inputs += 1;
    } else if (NESTED_INPUT_SCHEMAS.includes(name)) {
      nested += 1;
    }
  }

  const personBlock = exportedTypeBlock(generatedTs, "Person");
  if (personBlock) {
    const fields = typeBlockPropertyNames(personBlock);
    for (const forbidden of PERSON_FORBIDDEN_FIELDS) {
      if (fields.includes(forbidden)) {
        push(
          findings,
          `${SDK_GENERATED_REL}:Person`,
          `Person closed projection must not grow ${forbidden}`,
        );
      }
    }
  }

  const payRunBlock = exportedTypeBlock(generatedTs, "PayRun");
  if (payRunBlock && !/\bpayable\s*:\s*false\s*;/.test(payRunBlock)) {
    push(
      findings,
      `${SDK_GENERATED_REL}:PayRun`,
      "PayRun.payable must stay literal false in the generated SDK",
    );
  }

  const belowFloor =
    heads < HEAD_TYPE_FLOOR
    || inputs < INPUT_TYPE_FLOOR
    || nested < NESTED_TYPE_FLOOR
    || files < SDK_FILE_RELS.length;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      SDK_DIR_REL,
      `generated ${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${files}/${SDK_FILE_RELS.length} files — below the floor`,
    );
  }

  return { heads, inputs, nested, files, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateGeneratedSdk({ repoRoot });
  } catch (error) {
    console.error(`generated SDK gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { heads, inputs, nested, files, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    heads < HEAD_TYPE_FLOOR
    || inputs < INPUT_TYPE_FLOOR
    || nested < NESTED_TYPE_FLOOR
    || files < SDK_FILE_RELS.length;
  if (belowFloor) {
    console.error(
      `saw ${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${files}/${SDK_FILE_RELS.length} files — below the floor; `
        + `composed YAML without a generated SDK is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi generated-sdk gate FAILED: ${findings.length} finding(s), `
        + `${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
        + `${nested}/${NESTED_TYPE_FLOOR} nested, ${files}/${SDK_FILE_RELS.length} files`,
    );
    process.exit(1);
  }
  console.log(
    `openapi generated-sdk gate passed `
      + `(${heads}/${HEAD_TYPE_FLOOR} Heads, ${inputs}/${INPUT_TYPE_FLOOR} Inputs, `
      + `${nested}/${NESTED_TYPE_FLOOR} nested, ${files}/${SDK_FILE_RELS.length} files from ${OPENAPI_REL}, 0 findings)`,
  );
}
