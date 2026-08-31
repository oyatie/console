// DTO-derived Head/Input property-shape gate (ADR-0031 slice).
//
// The hole this closes: #998/#999 generate OpenAPI bodies and execute codecs
// from semantic_manifest.json, but the property bags inside that JSON are still
// hand-copied schema literals. Dual-written JSON + Rust is not a generated
// contract. PRODUCT requires one source: the runtime Head/Input types (or the
// compose crate's DTO inventory of those types) generate the property bags.
//
// Chesterton: extend generated_schema_yaml / generated_typed_action_rs. Do not
// add a second OpenAPI writer. objects/links/actions stay in the manifest;
// property shapes must not.
//
// Totality: own-property reads of the manifest + text scans of the DTO module
// and emitter + js-yaml of the published document + adapter Head field reads.
// A walker that visits nothing reports nothing, so SHAPE_FLOOR locks
// examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  CANONICAL_OBJECTS,
  PERSON_FORBIDDEN_FIELDS,
} from "./check-openapi-semantic-contract.mjs";
import {
  GENERATED_FLOOR,
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  MANIFEST_REL,
  NESTED_INPUT_SCHEMAS,
  OPENAPI_GEN_REL,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import { CODEC_SCHEMA_NAMES, TYPED_ACTION_GENERATED_REL } from "./check-openapi-typed-action-codecs.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export const DTO_RS_REL = "backend/crates/contracts/src/semantic_dtos.rs";
export const SHAPE_FLOOR = GENERATED_FLOOR;

/** Runtime Head structs the published object schemas must not drift from. */
export const HEAD_DTO_SOURCES = Object.freeze([
  Object.freeze({
    name: "Company",
    rel: "backend/crates/ontology/canonical-adapter-postgres/src/company.rs",
    structName: "CompanyHead",
  }),
  Object.freeze({
    name: "OrgUnit",
    rel: "backend/crates/ontology/canonical-adapter-postgres/src/org_unit.rs",
    structName: "OrgUnitHead",
  }),
  Object.freeze({
    name: "JobPosition",
    rel: "backend/crates/ontology/canonical-adapter-postgres/src/job_position.rs",
    structName: "JobPositionView",
  }),
  Object.freeze({
    name: "Person",
    rel: "backend/crates/ontology/canonical-adapter-postgres/src/person.rs",
    structName: "PersonHead",
  }),
  Object.freeze({
    name: "Employment",
    rel: "backend/crates/ontology/canonical-adapter-postgres/src/employment.rs",
    structName: "EmploymentHead",
  }),
]);

function push(findings, location, message) {
  findings.push({ location, message });
}

function loadJson(repoRoot, rel) {
  const path = join(repoRoot, rel);
  if (!existsSync(path)) return { path, missing: true, value: null };
  try {
    return { path, missing: false, value: JSON.parse(readFileSync(path, "utf8")) };
  } catch (error) {
    return { path, missing: false, value: null, error: error.message };
  }
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

function dtoDeclaredNames(source) {
  const names = [];
  const re = /^\s*(?:pub\s+)?(?:struct|const)\s+([A-Za-z_][A-Za-z0-9_]*)\b/gm;
  let match;
  while ((match = re.exec(source)) !== null) {
    names.push(match[1]);
  }
  return names;
}

function dtoShapeFieldNames(source, schemaName) {
  const escaped = schemaName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const constMatch = source.match(
    new RegExp(String.raw`const ${escaped.toUpperCase().replace(/[A-Z]/g, "")}|name:\s*"${escaped}"[\s\S]*?fields:\s*&\[([\s\S]*?)\]`),
  );
  if (constMatch) {
    const fields = [];
    const fieldRe = /name:\s*"([A-Za-z_][A-Za-z0-9_]*)"/g;
    let fieldMatch;
    while ((fieldMatch = fieldRe.exec(constMatch[1] ?? constMatch[0])) !== null) {
      if (fieldMatch[1] !== schemaName) fields.push(fieldMatch[1]);
    }
    if (fields.length > 0) return fields;
  }
  const structFields = rustStructFields(source, schemaName);
  if (structFields && structFields.length > 0) return structFields;
  const shapeMatch = source.match(
    new RegExp(String.raw`"${escaped}"[\s\S]{0,400}fields:\s*&\[([\s\S]*?)\]`),
  );
  if (!shapeMatch) return null;
  const fields = [];
  const fieldRe = /name:\s*"([A-Za-z_][A-Za-z0-9_]*)"/g;
  let fieldMatch;
  while ((fieldMatch = fieldRe.exec(shapeMatch[1])) !== null) {
    fields.push(fieldMatch[1]);
  }
  return fields;
}

function schemaPropertyNames(schema) {
  if (!isPlainObject(schema)) return [];
  const properties = own(schema, "properties");
  if (!isPlainObject(properties)) return [];
  return Object.keys(properties);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   shapes: number,
 *   heads: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateDtoDerivedShapes({ repoRoot }) {
  const findings = [];
  const manifestLoad = loadJson(repoRoot, MANIFEST_REL);

  if (manifestLoad.missing) {
    push(
      findings,
      MANIFEST_REL,
      "semantic manifest is absent; cannot tell objects/links/actions from hand property bags",
    );
    return { shapes: 0, heads: 0, findings };
  }
  if (manifestLoad.error || !isPlainObject(manifestLoad.value)) {
    push(
      findings,
      MANIFEST_REL,
      `semantic manifest is not a JSON object${manifestLoad.error ? `: ${manifestLoad.error}` : ""}`,
    );
    return { shapes: 0, heads: 0, findings };
  }

  const manifest = manifestLoad.value;
  const schemas = own(manifest, "schemas");
  if (isPlainObject(schemas)) {
    for (const name of GENERATED_SCHEMA_NAMES) {
      const schema = own(schemas, name);
      if (!isPlainObject(schema)) continue;
      if (isPlainObject(own(schema, "properties"))) {
        push(
          findings,
          `${MANIFEST_REL}#/schemas/${name}`,
          "property bag is still a hand-authored JSON literal; DTO types must generate this shape",
        );
      }
    }
  }

  const dto = readText(repoRoot, DTO_RS_REL);
  if (dto.missing) {
    push(
      findings,
      DTO_RS_REL,
      "DTO inventory is absent; Head/Input property bags are still hand JSON in the manifest",
    );
  } else {
    const declared = dtoDeclaredNames(dto.text);
    for (const name of GENERATED_SCHEMA_NAMES) {
      const hasName =
        declared.includes(name)
        || dto.text.includes(`"${name}"`)
        || dto.text.includes(`name: "${name}"`);
      if (!hasName) {
        push(
          findings,
          `${DTO_RS_REL}:${name}`,
          "DTO inventory does not declare this Head/Input shape",
        );
      }
    }
    if (!dto.text.includes("dto_schema_bags") && !dto.text.includes("fn schema")) {
      push(
        findings,
        DTO_RS_REL,
        "DTO inventory must export dto_schema_bags (or per-type schema) so compose generates property bags from types, not from JSON literals",
      );
    }
  }

  const semantic = readText(repoRoot, SEMANTIC_RS_REL);
  if (semantic.missing) {
    push(findings, SEMANTIC_RS_REL, "semantic emitter is missing");
  } else {
    if (!semantic.text.includes("semantic_dtos") && !semantic.text.includes("dto_schema_bags")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "generated_schema_yaml must take property bags from the DTO inventory, not from semantic_manifest.json schemas",
      );
    }
    const usesManifestSchemas = /\.get\("schemas"\)[\s\S]{0,120}and_then/.test(
      semantic.text,
    );
    if (usesManifestSchemas) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "emitter still reads property bags from the manifest; that JSON is a second hand-authored contract",
      );
    }
  }

  const genBin = readText(repoRoot, OPENAPI_GEN_REL);
  if (genBin.missing) {
    push(findings, OPENAPI_GEN_REL, "console-openapi-gen is missing");
  } else if (
    !genBin.text.includes("generated_schema_yaml")
    || !genBin.text.includes("compose_document_with_owned")
  ) {
    push(
      findings,
      OPENAPI_GEN_REL,
      "console-openapi-gen must keep composing via generated_schema_yaml; a second OpenAPI writer is refused",
    );
  }

  let heads = 0;
  for (const spec of HEAD_DTO_SOURCES) {
    const source = readText(repoRoot, spec.rel);
    if (source.missing) {
      push(findings, spec.rel, `runtime Head ${spec.structName} source is missing`);
      continue;
    }
    const fields = rustStructFields(source.text, spec.structName);
    if (!fields || fields.length === 0) {
      push(
        findings,
        `${spec.rel}:${spec.structName}`,
        "could not read pub fields from the runtime Head struct",
      );
      continue;
    }
    heads += 1;
    if (!dto.missing) {
      const dtoFields = dtoShapeFieldNames(dto.text, spec.name);
      if (!dtoFields || dtoFields.length === 0) {
        push(
          findings,
          `${DTO_RS_REL}:${spec.name}`,
          `DTO inventory does not declare fields for runtime ${spec.structName}`,
        );
      } else {
        const missing = fields.filter((field) => !dtoFields.includes(field));
        const extra = dtoFields.filter((field) => !fields.includes(field));
        if (missing.length > 0 || extra.length > 0) {
          push(
            findings,
            `${DTO_RS_REL}:${spec.name}`,
            `DTO fields drifted from ${spec.structName}: missing [${missing.join(", ")}] extra [${extra.join(", ")}]`,
          );
        }
      }
    }
  }

  let shapes = 0;
  const documentPath = join(repoRoot, "backend/openapi/openapi.yaml");
  if (!existsSync(documentPath)) {
    push(findings, "backend/openapi/openapi.yaml", "published document is missing");
    return { shapes, heads, findings };
  }

  let document;
  try {
    document = yaml.load(readFileSync(documentPath, "utf8"));
  } catch (error) {
    push(findings, "backend/openapi/openapi.yaml", `cannot parse: ${error.message}`);
    return { shapes, heads, findings };
  }

  const published = own(own(document, "components"), "schemas");
  if (!isPlainObject(published)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { shapes, heads, findings };
  }

  for (const name of GENERATED_SCHEMA_NAMES) {
    const schema = own(published, name);
    const loc = `#/components/schemas/${name}`;
    if (!isPlainObject(schema)) {
      push(findings, loc, "composed document is missing a generated Head/Input schema");
      continue;
    }
    const properties = schemaPropertyNames(schema);
    if (properties.length === 0) {
      push(findings, loc, "generated schema has no properties");
      continue;
    }
    if ((INPUT_SCHEMA_NAMES.includes(name) || NESTED_INPUT_SCHEMAS.includes(name))
      && own(schema, "additionalProperties") !== false) {
      push(findings, `${loc}/additionalProperties`, "typed input must set additionalProperties: false");
    }
    if (!dto.missing) {
      const dtoFields = dtoShapeFieldNames(dto.text, name);
      if (dtoFields && dtoFields.length > 0) {
        const missing = dtoFields.filter((field) => !properties.includes(field));
        if (missing.length > 0) {
          push(
            findings,
            loc,
            `composed schema drifted from DTO fields: missing ${missing.join(", ")}`,
          );
          continue;
        }
      }
    }
    shapes += 1;
  }

  const person = own(published, "Person");
  if (isPlainObject(person)) {
    const properties = schemaPropertyNames(person);
    for (const forbidden of PERSON_FORBIDDEN_FIELDS) {
      if (properties.includes(forbidden)) {
        push(
          findings,
          "#/components/schemas/Person",
          `Person closed projection must not grow ${forbidden}`,
        );
      }
    }
  }

  const payRun = own(published, "PayRun");
  if (isPlainObject(payRun)) {
    const payable = own(own(payRun, "properties"), "payable");
    if (!isPlainObject(payable) || own(payable, "const") !== false) {
      push(
        findings,
        "#/components/schemas/PayRun/properties/payable",
        "PayRun.payable must stay const: false",
      );
    }
  }

  const generatedCodecs = readText(repoRoot, TYPED_ACTION_GENERATED_REL);
  if (!generatedCodecs.missing && !dto.missing) {
    for (const name of CODEC_SCHEMA_NAMES) {
      const dtoFields = dtoShapeFieldNames(dto.text, name);
      if (!dtoFields || dtoFields.length === 0) continue;
      const missing = dtoFields.filter(
        (field) => !new RegExp(String.raw`\b${field}\s*:`).test(generatedCodecs.text),
      );
      if (missing.length > 0) {
        push(
          findings,
          `${TYPED_ACTION_GENERATED_REL}:${name}`,
          `generated codec drifted from DTO fields: missing ${missing.join(", ")}`,
        );
      }
    }
  }

  const belowFloor = shapes < SHAPE_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      DTO_RS_REL,
      `generated ${shapes}/${SHAPE_FLOOR} DTO-derived shapes — below the floor`,
    );
  }

  return { shapes, heads, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const result = evaluateDtoDerivedShapes({ repoRoot });
  const { shapes, heads, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = shapes < SHAPE_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${shapes}/${SHAPE_FLOOR} DTO-derived shapes, ${heads}/${HEAD_DTO_SOURCES.length} Head DTOs — below the floor; `
        + `hand-authored JSON property bags are not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi dto-derived-shapes gate FAILED: ${findings.length} finding(s), `
        + `${shapes}/${SHAPE_FLOOR} shapes, ${heads}/${HEAD_DTO_SOURCES.length} Head DTOs`,
    );
    process.exit(1);
  }
  console.log(
    `openapi dto-derived-shapes gate passed `
      + `(${shapes}/${SHAPE_FLOOR} shapes from DTO types, `
      + `${heads}/${HEAD_DTO_SOURCES.length} Head DTOs, 0 findings)`,
  );
}
