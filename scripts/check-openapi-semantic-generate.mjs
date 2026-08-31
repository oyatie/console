// Semantic-manifest generation gate (ADR-0031 slice).
//
// The hole this closes: #996/#997 typed the thirteen DispatchTarget inputs and
// the six Heads' links/actions, but the composed document still include_str's
// hand-written YAML beside independently authored Rust codecs. PRODUCT requires
// one machine-readable semantic source that generates OpenAPI. Dual-written
// YAML + Rust is not that.
//
// Chesterton: extend console-openapi-gen / compose. Do not add a second
// OpenAPI emitter. The contracts crate loads the manifest, emits schema YAML,
// and compose merges those bodies. Face fragments must not also own the names.
//
// Totality: own-property reads of the manifest + js-yaml of the published
// document + a text scan of gen_registry.rs. A walker that visits nothing
// reports nothing, so ACTION_FLOOR / OBJECT_FLOOR / GENERATED_FLOOR lock
// examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  ACTION_FLOOR,
  CANONICAL_ACTIONS,
  CANONICAL_LINKS,
  CANONICAL_OBJECTS,
  CONCURRENCY_COMMAND_ID,
  CONCURRENCY_EXPECTED_REVISION,
  LINK_FLOOR,
  OBJECT_FLOOR,
  PERMISSION_ROLE_MANAGE,
  RESULT_REF,
} from "./check-openapi-semantic-contract.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const MANIFEST_REL = "backend/crates/contracts/src/semantic_manifest.json";
export const GEN_REGISTRY_REL = "backend/crates/contracts/src/gen_registry.rs";
export const OPENAPI_GEN_REL = "backend/crates/contracts/src/bin/console_openapi_gen.rs";
export const COMPOSE_LIB_REL = "backend/crates/contracts/src/lib.rs";
export const SEMANTIC_RS_REL = "backend/crates/contracts/src/semantic.rs";

export const NESTED_INPUT_SCHEMAS = Object.freeze([
  "EmploymentAttributesInput",
  "OrgUnitSourceBinding",
]);

export const INPUT_SCHEMA_NAMES = Object.freeze(
  CANONICAL_ACTIONS.map((action) => action.input),
);

export const HEAD_SCHEMA_NAMES = Object.freeze(
  CANONICAL_OBJECTS.map((object) => object.name),
);

export const GENERATED_SCHEMA_NAMES = Object.freeze([
  ...INPUT_SCHEMA_NAMES,
  ...NESTED_INPUT_SCHEMAS,
  ...HEAD_SCHEMA_NAMES,
]);

export const GENERATED_FLOOR = GENERATED_SCHEMA_NAMES.length;

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

function deepEqual(left, right) {
  if (Object.is(left, right)) return true;
  if (Array.isArray(left) && Array.isArray(right)) {
    if (left.length !== right.length) return false;
    return left.every((item, index) => deepEqual(item, right[index]));
  }
  if (!isPlainObject(left) || !isPlainObject(right)) return false;
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  if (leftKeys.length !== rightKeys.length) return false;
  return leftKeys.every(
    (key) => hasOwnKey(right, key) && deepEqual(own(left, key), own(right, key)),
  );
}

function actionContract(action) {
  return {
    action_key: action.action_key,
    object_key: action.object_key,
    input: { $ref: `#/components/schemas/${action.input}` },
    result: { $ref: RESULT_REF },
    permissions: [...(action.permissions ?? [PERMISSION_ROLE_MANAGE])],
    four_eyes: action.four_eyes,
    edits: [...action.edits],
    concurrency: {
      command_id: own(action.concurrency, "command_id") ?? CONCURRENCY_COMMAND_ID,
      expected_revision:
        own(action.concurrency, "expected_revision") ?? CONCURRENCY_EXPECTED_REVISION,
    },
  };
}

/**
 * Schema body the composed document must carry for `name`, derived only from
 * the semantic manifest (not from face YAML).
 */
export function expectedGeneratedSchema(manifest, name) {
  const schemas = own(manifest, "schemas");
  const base = own(schemas, name);
  if (!isPlainObject(base)) return null;
  if (!HEAD_SCHEMA_NAMES.includes(name)) {
    return structuredClone(base);
  }
  const objects = own(manifest, "objects");
  const links = own(manifest, "links");
  const actions = own(manifest, "actions");
  const object = Array.isArray(objects)
    ? objects.find((item) => isPlainObject(item) && own(item, "name") === name)
    : null;
  const declaredLinks = Array.isArray(links)
    ? links.filter((item) => isPlainObject(item) && own(item, "from") === name)
    : [];
  const actionKeys = isPlainObject(object) && Array.isArray(own(object, "actions"))
    ? own(object, "actions")
    : [];
  const declaredActions = Array.isArray(actions)
    ? actionKeys
        .map((key) => actions.find((item) => isPlainObject(item) && own(item, "action_key") === key))
        .filter(Boolean)
        .map(actionContract)
    : [];
  return {
    ...structuredClone(base),
    links: declaredLinks.map((link) => ({
      key: own(link, "key"),
      from: own(link, "from"),
      to: own(link, "to"),
      field: own(link, "field"),
      cardinality: own(link, "cardinality"),
      option: own(link, "option"),
    })),
    actions: declaredActions,
  };
}

function includeStrMentions(source, schemaName) {
  const escaped = schemaName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(String.raw`include_str!\([^)]*[/"]${escaped}\.yaml`);
  return re.test(source);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   generated: number,
 *   actions: number,
 *   objects: number,
 *   links: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateSemanticGenerate({ repoRoot }) {
  const findings = [];
  const manifestLoad = loadJson(repoRoot, MANIFEST_REL);
  if (manifestLoad.missing) {
    push(
      findings,
      MANIFEST_REL,
      "semantic manifest is absent; hand-written Input YAML is not a generator",
    );
    return { generated: 0, actions: 0, objects: 0, links: 0, findings };
  }
  if (manifestLoad.error || !isPlainObject(manifestLoad.value)) {
    push(
      findings,
      MANIFEST_REL,
      `semantic manifest is not a JSON object${manifestLoad.error ? `: ${manifestLoad.error}` : ""}`,
    );
    return { generated: 0, actions: 0, objects: 0, links: 0, findings };
  }

  const manifest = manifestLoad.value;
  const objects = own(manifest, "objects");
  const links = own(manifest, "links");
  const actions = own(manifest, "actions");
  const schemas = own(manifest, "schemas");

  let objectCount = 0;
  let actionCount = 0;
  let linkCount = 0;

  if (!Array.isArray(objects)) {
    push(findings, `${MANIFEST_REL}#/objects`, "objects must be an array of six Heads");
  } else {
    for (const spec of CANONICAL_OBJECTS) {
      const found = objects.find((item) => isPlainObject(item) && own(item, "name") === spec.name);
      if (!found) {
        push(
          findings,
          `${MANIFEST_REL}#/objects/${spec.name}`,
          "canonical object is missing from the semantic manifest",
        );
        continue;
      }
      objectCount += 1;
    }
  }

  if (!Array.isArray(links)) {
    push(findings, `${MANIFEST_REL}#/links`, "links must be an array of Head FK declarations");
  } else {
    for (const spec of CANONICAL_LINKS) {
      const found = links.find((item) => isPlainObject(item) && own(item, "key") === spec.key);
      if (!found) {
        push(
          findings,
          `${MANIFEST_REL}#/links/${spec.key}`,
          "runtime Head FK is missing from the semantic manifest",
        );
        continue;
      }
      linkCount += 1;
    }
  }

  if (!Array.isArray(actions)) {
    push(
      findings,
      `${MANIFEST_REL}#/actions`,
      "actions must be an array of thirteen DispatchTarget contracts",
    );
  } else {
    for (const spec of CANONICAL_ACTIONS) {
      const found = actions.find(
        (item) => isPlainObject(item) && own(item, "action_key") === spec.action_key,
      );
      if (!found) {
        push(
          findings,
          `${MANIFEST_REL}#/actions/${spec.action_key}`,
          "dispatch target is missing from the semantic manifest",
        );
        continue;
      }
      actionCount += 1;
    }
  }

  if (!isPlainObject(schemas)) {
    push(
      findings,
      `${MANIFEST_REL}#/schemas`,
      "schemas must map generated names to OpenAPI schema objects",
    );
  } else {
    for (const name of GENERATED_SCHEMA_NAMES) {
      const schema = own(schemas, name);
      if (!isPlainObject(schema)) {
        push(
          findings,
          `${MANIFEST_REL}#/schemas/${name}`,
          "generated schema body is absent from the semantic manifest",
        );
        continue;
      }
      if (INPUT_SCHEMA_NAMES.includes(name) || NESTED_INPUT_SCHEMAS.includes(name)) {
        if (own(schema, "additionalProperties") !== false) {
          push(
            findings,
            `${MANIFEST_REL}#/schemas/${name}/additionalProperties`,
            "typed input must set additionalProperties: false",
          );
        }
      }
    }
  }

  const semanticRs = join(repoRoot, SEMANTIC_RS_REL);
  if (!existsSync(semanticRs)) {
    push(
      findings,
      SEMANTIC_RS_REL,
      "contracts crate has no semantic emitter; compose cannot generate Input schemas from the manifest",
    );
  } else {
    const text = readFileSync(semanticRs, "utf8");
    if (!text.includes("semantic_manifest.json")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must include_str the JSON manifest (one source, not a second roster)",
      );
    }
    if (!text.includes("generated_schema_yaml")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must export generated_schema_yaml for compose",
      );
    }
  }

  const genBin = join(repoRoot, OPENAPI_GEN_REL);
  if (!existsSync(genBin)) {
    push(findings, OPENAPI_GEN_REL, "console-openapi-gen is missing");
  } else {
    const text = readFileSync(genBin, "utf8");
    if (!text.includes("generated_schema_yaml") && !text.includes("compose_document_with_owned")) {
      push(
        findings,
        OPENAPI_GEN_REL,
        "console-openapi-gen must compose owned schemas from generated_schema_yaml; a second emitter beside compose is refused",
      );
    }
  }

  const composeLib = join(repoRoot, COMPOSE_LIB_REL);
  if (existsSync(composeLib)) {
    const text = readFileSync(composeLib, "utf8");
    if (!text.includes("compose_document_with_owned") && !text.includes("OwnedNamedYaml")) {
      push(
        findings,
        COMPOSE_LIB_REL,
        "compose must accept owned schema bodies so the manifest can generate without a second OpenAPI writer",
      );
    }
  }

  const registryPath = join(repoRoot, GEN_REGISTRY_REL);
  if (!existsSync(registryPath)) {
    push(findings, GEN_REGISTRY_REL, "gen_registry.rs is missing");
  } else {
    const registry = readFileSync(registryPath, "utf8");
    for (const name of GENERATED_SCHEMA_NAMES) {
      if (includeStrMentions(registry, name)) {
        push(
          findings,
          `${GEN_REGISTRY_REL}:${name}`,
          `gen_registry still include_str's ${name}.yaml — dual-written YAML is not generation`,
        );
      }
    }
  }

  let generated = 0;
  const documentPath = join(repoRoot, "backend/openapi/openapi.yaml");
  if (!existsSync(documentPath)) {
    push(findings, "backend/openapi/openapi.yaml", "published document is missing");
    return {
      generated,
      actions: actionCount,
      objects: objectCount,
      links: linkCount,
      findings,
    };
  }

  let document;
  try {
    document = yaml.load(readFileSync(documentPath, "utf8"));
  } catch (error) {
    push(findings, "backend/openapi/openapi.yaml", `cannot parse: ${error.message}`);
    return {
      generated,
      actions: actionCount,
      objects: objectCount,
      links: linkCount,
      findings,
    };
  }

  const published = own(own(document, "components"), "schemas");
  if (!isPlainObject(published)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return {
      generated,
      actions: actionCount,
      objects: objectCount,
      links: linkCount,
      findings,
    };
  }

  if (isPlainObject(schemas)) {
    for (const name of GENERATED_SCHEMA_NAMES) {
      const expected = expectedGeneratedSchema(manifest, name);
      const actual = own(published, name);
      const loc = `#/components/schemas/${name}`;
      if (!isPlainObject(expected)) {
        continue;
      }
      if (!isPlainObject(actual)) {
        push(findings, loc, "composed document is missing a schema the manifest generates");
        continue;
      }
      if (!deepEqual(expected, actual)) {
        push(
          findings,
          loc,
          "composed schema does not match the semantic manifest (hand-written YAML drifted, or compose did not emit this body)",
        );
        continue;
      }
      generated += 1;
    }
  }

  const belowFloor =
    generated < GENERATED_FLOOR
    || actionCount < ACTION_FLOOR
    || objectCount < OBJECT_FLOOR
    || linkCount < LINK_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      MANIFEST_REL,
      `generated ${generated}/${GENERATED_FLOOR} schemas, ${objectCount}/${OBJECT_FLOOR} objects, ${actionCount}/${ACTION_FLOOR} actions, ${linkCount}/${LINK_FLOOR} links — below the floor`,
    );
  }

  return {
    generated,
    actions: actionCount,
    objects: objectCount,
    links: linkCount,
    findings,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const result = evaluateSemanticGenerate({ repoRoot });
  const { generated, actions, objects, links, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    generated < GENERATED_FLOOR
    || actions < ACTION_FLOOR
    || objects < OBJECT_FLOOR
    || links < LINK_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${generated}/${GENERATED_FLOOR} generated schemas, ${objects}/${OBJECT_FLOOR} objects, `
        + `${actions}/${ACTION_FLOOR} actions, ${links}/${LINK_FLOOR} links — below the floor; `
        + `dual-written YAML is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi semantic-generate gate FAILED: ${findings.length} finding(s), `
        + `${generated}/${GENERATED_FLOOR} generated, ${objects}/${OBJECT_FLOOR} objects, `
        + `${actions}/${ACTION_FLOOR} actions, ${links}/${LINK_FLOOR} links`,
    );
    process.exit(1);
  }
  console.log(
    `openapi semantic-generate gate passed `
      + `(${generated}/${GENERATED_FLOOR} schemas from the manifest, `
      + `${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
      + `${links}/${LINK_FLOOR} links, 0 findings)`,
  );
}
