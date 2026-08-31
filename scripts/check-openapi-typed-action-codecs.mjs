// Typed-action codec generation gate (ADR-0031 slice).
//
// The hole this closes: #998 generates the thirteen DispatchTarget Input
// schemas (and two nested write bags) into OpenAPI from semantic_manifest.json,
// but ontology REST still dual-maintains hand-written serde codecs in
// typed_action.rs. PRODUCT requires one source that generates OpenAPI and
// validators. Dual-written JSON + Rust is a second contract.
//
// Chesterton: extend console-openapi-gen / typed_action.rs. Do not add a
// second runtime binder. bind_canonical_action_params stays the HTTP trust
// boundary; the Input structs and decode_dispatch_target arms are generated
// from the same manifest that emits the OpenAPI bodies.
//
// Totality: own-property reads of the manifest + text scans of the emitter,
// binder, generator, and generated Rust. A walker that visits nothing reports
// nothing, so ACTION_FLOOR / CODEC_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  ACTION_FLOOR,
  CANONICAL_ACTIONS,
} from "./check-openapi-semantic-contract.mjs";
import {
  INPUT_SCHEMA_NAMES,
  MANIFEST_REL,
  NESTED_INPUT_SCHEMAS,
  OPENAPI_GEN_REL,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export const TYPED_ACTION_RS_REL = "backend/crates/ontology/rest/src/typed_action.rs";
export const TYPED_ACTION_GENERATED_REL =
  "backend/crates/ontology/rest/src/typed_action_generated.rs";

export const CODEC_SCHEMA_NAMES = Object.freeze([
  ...INPUT_SCHEMA_NAMES,
  ...NESTED_INPUT_SCHEMAS,
]);

export const CODEC_FLOOR = CODEC_SCHEMA_NAMES.length;

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

function hasStruct(source, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(String.raw`\bstruct\s+${escaped}\b`).test(source);
}

function hasField(source, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(String.raw`\b${escaped}\s*:`).test(source);
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
 *   codecs: number,
 *   actions: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateTypedActionCodecs({ repoRoot }) {
  const findings = [];
  const manifestLoad = loadJson(repoRoot, MANIFEST_REL);
  if (manifestLoad.missing) {
    push(
      findings,
      MANIFEST_REL,
      "semantic manifest is absent; hand-written typed_action.rs is not a generator",
    );
    return { codecs: 0, actions: 0, findings };
  }
  if (manifestLoad.error || !isPlainObject(manifestLoad.value)) {
    push(
      findings,
      MANIFEST_REL,
      `semantic manifest is not a JSON object${manifestLoad.error ? `: ${manifestLoad.error}` : ""}`,
    );
    return { codecs: 0, actions: 0, findings };
  }

  const manifest = manifestLoad.value;
  const actions = own(manifest, "actions");
  const schemas = own(manifest, "schemas");

  let actionCount = 0;
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

  const semanticRs = readText(repoRoot, SEMANTIC_RS_REL);
  if (semanticRs.missing) {
    push(
      findings,
      SEMANTIC_RS_REL,
      "contracts crate has no semantic emitter; codecs cannot be generated from the manifest",
    );
  } else {
    if (!semanticRs.text.includes("semantic_manifest.json")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must include_str the JSON manifest (one source, not a second roster)",
      );
    }
    if (!semanticRs.text.includes("generated_typed_action_rs")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must export generated_typed_action_rs so the 13 codecs are not dual-maintained",
      );
    }
  }

  const genBin = readText(repoRoot, OPENAPI_GEN_REL);
  if (genBin.missing) {
    push(findings, OPENAPI_GEN_REL, "console-openapi-gen is missing");
  } else {
    const text = genBin.text;
    if (!text.includes("generated_typed_action_rs")) {
      push(
        findings,
        OPENAPI_GEN_REL,
        "console-openapi-gen must write generated typed-action codecs; a second binder beside typed_action.rs is refused",
      );
    }
    if (!text.includes("typed_action_generated.rs")) {
      push(
        findings,
        OPENAPI_GEN_REL,
        "console-openapi-gen must write ontology/rest/src/typed_action_generated.rs (same writer as openapi.yaml)",
      );
    }
  }

  const binder = readText(repoRoot, TYPED_ACTION_RS_REL);
  if (binder.missing) {
    push(
      findings,
      TYPED_ACTION_RS_REL,
      "typed_action.rs binder is missing; do not invent a second runtime binder",
    );
  } else {
    if (!binder.text.includes("bind_canonical_action_params")) {
      push(
        findings,
        TYPED_ACTION_RS_REL,
        "typed_action.rs must keep bind_canonical_action_params (extend the binder, do not replace it)",
      );
    }
    if (!binder.text.includes("typed_action_generated.rs")) {
      push(
        findings,
        TYPED_ACTION_RS_REL,
        "typed_action.rs must include! generated codecs from the manifest; hand-written Input structs are a second contract",
      );
    }
    for (const name of CODEC_SCHEMA_NAMES) {
      if (hasStruct(binder.text, name)) {
        push(
          findings,
          `${TYPED_ACTION_RS_REL}:${name}`,
          `typed_action.rs still hand-defines ${name} — dual-maintained codecs are not generation`,
        );
      }
    }
  }

  const generated = readText(repoRoot, TYPED_ACTION_GENERATED_REL);
  let codecs = 0;
  if (generated.missing) {
    push(
      findings,
      TYPED_ACTION_GENERATED_REL,
      "generated typed-action codecs are absent; typed_action.rs structs are dual-maintained with the manifest",
    );
  } else {
    const text = generated.text;
    if (!text.includes("semantic_manifest.json") && !text.includes("@generated")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated codec file must declare it is emitted from semantic_manifest.json",
      );
    }
    if (!text.includes("decode_dispatch_target")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated codecs must include decode_dispatch_target so the 13 arms are not dual-maintained",
      );
    }
    if (!isPlainObject(schemas)) {
      push(
        findings,
        `${MANIFEST_REL}#/schemas`,
        "schemas must map generated names to OpenAPI schema objects",
      );
    } else {
      for (const name of CODEC_SCHEMA_NAMES) {
        const schema = own(schemas, name);
        const loc = `${TYPED_ACTION_GENERATED_REL}:${name}`;
        if (!isPlainObject(schema)) {
          push(
            findings,
            `${MANIFEST_REL}#/schemas/${name}`,
            "codec schema body is absent from the semantic manifest",
          );
          continue;
        }
        if (!hasStruct(text, name)) {
          push(
            findings,
            loc,
            "generated Rust is missing a codec struct the manifest declares",
          );
          continue;
        }
        if (own(schema, "additionalProperties") === false && !text.includes("deny_unknown_fields")) {
          push(
            findings,
            loc,
            "generated codecs must serde(deny_unknown_fields) when the manifest sets additionalProperties: false",
          );
        }
        const properties = schemaPropertyNames(schema);
        const missingFields = properties.filter((field) => !hasField(text, field));
        if (missingFields.length > 0) {
          push(
            findings,
            loc,
            `generated Rust drifted from the manifest; missing fields ${missingFields.join(", ")}`,
          );
          continue;
        }
        codecs += 1;
      }
    }
    for (const spec of CANONICAL_ACTIONS) {
      const variant = spec.action_key
        .split(/[._]/)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join("");
      if (!text.includes(`DispatchTarget::${variant}`)) {
        push(
          findings,
          `${TYPED_ACTION_GENERATED_REL}:${spec.action_key}`,
          `generated decode_dispatch_target must arm DispatchTarget::${variant} from the manifest action_key`,
        );
      }
    }
  }

  const belowFloor = codecs < CODEC_FLOOR || actionCount < ACTION_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      MANIFEST_REL,
      `generated ${codecs}/${CODEC_FLOOR} codecs, ${actionCount}/${ACTION_FLOOR} actions — below the floor`,
    );
  }

  return { codecs, actions: actionCount, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const result = evaluateTypedActionCodecs({ repoRoot });
  const { codecs, actions, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = codecs < CODEC_FLOOR || actions < ACTION_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${codecs}/${CODEC_FLOOR} generated codecs, ${actions}/${ACTION_FLOOR} actions — below the floor; `
        + `dual-maintained typed_action.rs is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi typed-action-codecs gate FAILED: ${findings.length} finding(s), `
        + `${codecs}/${CODEC_FLOOR} codecs, ${actions}/${ACTION_FLOOR} actions`,
    );
    process.exit(1);
  }
  console.log(
    `openapi typed-action-codecs gate passed `
      + `(${codecs}/${CODEC_FLOOR} codecs from the manifest, `
      + `${actions}/${ACTION_FLOOR} actions, 0 findings)`,
  );
}
