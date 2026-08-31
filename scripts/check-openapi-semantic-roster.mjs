// Semantic-roster generation gate (ADR-0031 slice).
//
// The hole this closes: #998–#1002 generate Head/Input property bags, codecs,
// and bind_canonical_action_params from DTO types + a JSON catalog, but the
// catalog itself (`objects` / `links` / `actions`) is still hand-copied JSON
// that compose and binders merely consume. Dual-written YAML+Rust is not a
// generated contract; a hand-maintained roster JSON is the same class.
// PRODUCT requires the 6 Heads + 13 DispatchTargets + Head FK links to be
// emitted from the same inventory the runtime uses (DTO types / registry).
//
// Chesterton: #989 admitted the typed semantic manifest as direction; the JSON
// roster existed so compose had an explicit allow-list (deny-by-omission).
// Generating it must preserve that — do not auto-publish every Rust type in
// the crate. Prefer the types already tagged as REST Heads and the 13 action
// Inputs in semantic_dtos.rs, drifted against ObjectKey / DispatchTarget.
// Extend generated_schema_yaml / generated_typed_action_rs. Do not add a
// second OpenAPI writer.
//
// Totality: own-property reads of the committed JSON + text scans of the DTO
// inventory, emitter, generator, canonical-domain registries, and js-yaml of
// the published Heads. A walker that visits nothing reports nothing, so
// OBJECT_FLOOR / ACTION_FLOOR / LINK_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  ACTION_FLOOR,
  CANONICAL_ACTIONS,
  CANONICAL_LINKS,
  CANONICAL_OBJECTS,
  LINK_FLOOR,
  OBJECT_FLOOR,
} from "./check-openapi-semantic-contract.mjs";
import {
  DTO_RS_REL,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  MANIFEST_REL,
  NESTED_INPUT_SCHEMAS,
  OPENAPI_GEN_REL,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const DOMAIN_REL = "backend/crates/ontology/canonical-domain/src/lib.rs";

const MARKER_TYPES = Object.freeze(["Uuid", "Timestamp", "IsoDate", "JsonObject"]);
const ENTRY_TYPES = Object.freeze(["HeadEntry", "LinkEntry", "ActionEntry"]);
const PUBLISHED_STRUCTS = Object.freeze([
  ...HEAD_SCHEMA_NAMES,
  ...INPUT_SCHEMA_NAMES,
  ...NESTED_INPUT_SCHEMAS,
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

function rustStructNames(source) {
  const names = [];
  const re = /^\s*pub(?:\s*\([^)]+\))?\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)\b/gm;
  let match;
  while ((match = re.exec(source)) !== null) {
    names.push(match[1]);
  }
  return names;
}

function parseConstSlice(source, constName) {
  const escaped = constName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(
    new RegExp(String.raw`const ${escaped}\b[^=]*=\s*&\[([\s\S]*?)\]\s*;`),
  );
  return match ? match[1] : null;
}

function parseFieldStrings(block, field) {
  if (typeof block !== "string") return [];
  const escaped = field.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const values = [];
  const re = new RegExp(String.raw`${escaped}:\s*"([^"]+)"`, "g");
  let match;
  while ((match = re.exec(block)) !== null) {
    values.push(match[1]);
  }
  return values;
}

function parseObjectKeys(source) {
  const match = source.match(/object_keys!\s*\{([\s\S]*?)\n\}/);
  if (!match) return [];
  const entries = [];
  const re = /([A-Za-z_][A-Za-z0-9_]*)\s*=>\s*"([^"]+)"/g;
  let item;
  while ((item = re.exec(match[1])) !== null) {
    entries.push({ name: item[1], object_key: item[2] });
  }
  return entries;
}

function parseDispatchTargets(source) {
  const match = source.match(/dispatch_targets!\s*\{([\s\S]*?)\n\}/);
  if (!match) return [];
  const entries = [];
  const re =
    /([A-Za-z_][A-Za-z0-9_]*)\s*=>\s*"([^"]+)"\s*,\s*([A-Za-z_][A-Za-z0-9_]*)/g;
  let item;
  while ((item = re.exec(match[1])) !== null) {
    entries.push({ variant: item[1], action_key: item[2], object: item[3] });
  }
  return entries;
}

function consumesJsonArray(source, key) {
  const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(
    String.raw`root(?:\s|\n)*\.get\("${escaped}"\)[\s\S]{0,80}and_then\(Json::as_array\)`,
  ).test(source);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   objects: number,
 *   actions: number,
 *   links: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateSemanticRoster({ repoRoot }) {
  const findings = [];
  const manifestLoad = loadJson(repoRoot, MANIFEST_REL);
  if (manifestLoad.missing) {
    push(
      findings,
      MANIFEST_REL,
      "semantic manifest is absent; cannot tell a generated roster from a missing catalog",
    );
    return { objects: 0, actions: 0, links: 0, findings };
  }
  if (manifestLoad.error || !isPlainObject(manifestLoad.value)) {
    push(
      findings,
      MANIFEST_REL,
      `semantic manifest is not a JSON object${manifestLoad.error ? `: ${manifestLoad.error}` : ""}`,
    );
    return { objects: 0, actions: 0, links: 0, findings };
  }

  const manifest = manifestLoad.value;
  for (const key of ["objects", "links", "actions"]) {
    if (hasOwnKey(manifest, key)) {
      push(
        findings,
        `${MANIFEST_REL}#/${key}`,
        "roster array is still a hand-authored JSON catalog; DTO inventory / DispatchTarget registry must generate this",
      );
    }
  }

  const dto = readText(repoRoot, DTO_RS_REL);
  let objectCount = 0;
  let actionCount = 0;
  let linkCount = 0;
  let headNames = [];
  let actionKeys = [];
  let linkKeys = [];

  if (dto.missing) {
    push(
      findings,
      DTO_RS_REL,
      "DTO inventory is absent; objects/links/actions cannot be generated from runtime types",
    );
  } else {
    for (const fn of ["dto_objects", "dto_links", "dto_actions"]) {
      if (!dto.text.includes(`fn ${fn}`)) {
        push(
          findings,
          `${DTO_RS_REL}:${fn}`,
          "DTO inventory must emit the published roster so compose does not consume hand JSON arrays",
        );
      }
    }

    const headBlock = parseConstSlice(dto.text, "HEADS");
    const actionBlock = parseConstSlice(dto.text, "ACTIONS");
    const linkBlock = parseConstSlice(dto.text, "LINKS");
    headNames = parseFieldStrings(headBlock, "name");
    const headKeys = parseFieldStrings(headBlock, "object_key");
    actionKeys = parseFieldStrings(actionBlock, "action_key");
    const actionObjects = parseFieldStrings(actionBlock, "object");
    const actionInputs = parseFieldStrings(actionBlock, "input");
    linkKeys = parseFieldStrings(linkBlock, "key");
    const linkFrom = parseFieldStrings(linkBlock, "from");
    const linkTo = parseFieldStrings(linkBlock, "to");
    const linkFields = parseFieldStrings(linkBlock, "field");

    const declaredStructs = rustStructNames(dto.text);

    for (const spec of CANONICAL_OBJECTS) {
      const loc = `${DTO_RS_REL}:HEADS/${spec.name}`;
      if (!declaredStructs.includes(spec.name)) {
        push(findings, loc, "DTO inventory does not declare this Head struct");
        continue;
      }
      const index = headNames.indexOf(spec.name);
      if (index < 0) {
        push(
          findings,
          loc,
          "Head struct is not in the published HEADS roster (deny-by-omission requires an explicit tag, not every type)",
        );
        continue;
      }
      if (headKeys[index] !== spec.object_key) {
        push(
          findings,
          loc,
          `object_key drifted: DTO has ${headKeys[index] ?? "<missing>"}, runtime ObjectKey is ${spec.object_key}`,
        );
        continue;
      }
      objectCount += 1;
    }

    for (const spec of CANONICAL_ACTIONS) {
      const loc = `${DTO_RS_REL}:ACTIONS/${spec.action_key}`;
      if (!declaredStructs.includes(spec.input)) {
        push(findings, loc, `DTO inventory does not declare Input struct ${spec.input}`);
        continue;
      }
      const index = actionKeys.indexOf(spec.action_key);
      if (index < 0) {
        push(
          findings,
          loc,
          "DispatchTarget Input is not in the published ACTIONS roster",
        );
        continue;
      }
      if (actionObjects[index] !== spec.object || actionInputs[index] !== spec.input) {
        push(
          findings,
          loc,
          `action object/input drifted: DTO has ${actionObjects[index]}/${actionInputs[index]}`,
        );
        continue;
      }
      actionCount += 1;
    }

    for (const spec of CANONICAL_LINKS) {
      const loc = `${DTO_RS_REL}:LINKS/${spec.key}`;
      const index = linkKeys.indexOf(spec.key);
      if (index < 0) {
        push(findings, loc, "runtime Head FK is missing from the published LINKS roster");
        continue;
      }
      if (
        linkFrom[index] !== spec.from
        || linkTo[index] !== spec.to
        || linkFields[index] !== spec.field
      ) {
        push(
          findings,
          loc,
          `link drifted: DTO has ${linkFrom[index]}.${linkFields[index]} -> ${linkTo[index]}`,
        );
        continue;
      }
      const fields = rustStructFields(dto.text, spec.from);
      if (!fields || !fields.includes(spec.field)) {
        push(
          findings,
          loc,
          `link field ${spec.field} is not a field on DTO Head ${spec.from}`,
        );
        continue;
      }
      linkCount += 1;
    }

    for (const name of [...NESTED_INPUT_SCHEMAS, ...MARKER_TYPES]) {
      if (headNames.includes(name) || actionKeys.includes(name)) {
        push(
          findings,
          `${DTO_RS_REL}:${name}`,
          "deny-by-omission: nested write bags and marker types must not be published as objects or actions",
        );
      }
    }

    const extraHeads = headNames.filter(
      (name) => !CANONICAL_OBJECTS.some((spec) => spec.name === name),
    );
    if (extraHeads.length > 0) {
      push(
        findings,
        `${DTO_RS_REL}:HEADS`,
        `deny-by-omission: extra Head ${extraHeads.join(", ")} is not an ObjectKey`,
      );
    }
    const extraActions = actionKeys.filter(
      (key) => !CANONICAL_ACTIONS.some((spec) => spec.action_key === key),
    );
    if (extraActions.length > 0) {
      push(
        findings,
        `${DTO_RS_REL}:ACTIONS`,
        `deny-by-omission: extra action ${extraActions.join(", ")} is not a DispatchTarget`,
      );
    }

    const autoPublished = declaredStructs.filter(
      (name) =>
        !PUBLISHED_STRUCTS.includes(name)
        && !MARKER_TYPES.includes(name)
        && !ENTRY_TYPES.includes(name)
        && (headNames.includes(name) || actionKeys.includes(name)),
    );
    if (autoPublished.length > 0) {
      push(
        findings,
        DTO_RS_REL,
        `deny-by-omission: untagged struct ${autoPublished.join(", ")} was published`,
      );
    }
  }

  const semanticRs = readText(repoRoot, SEMANTIC_RS_REL);
  if (semanticRs.missing) {
    push(
      findings,
      SEMANTIC_RS_REL,
      "contracts crate has no semantic emitter; the roster cannot be generated",
    );
  } else {
    for (const fn of ["dto_objects", "dto_links", "dto_actions"]) {
      if (!semanticRs.text.includes(fn)) {
        push(
          findings,
          SEMANTIC_RS_REL,
          `semantic emitter must call ${fn} so objects/links/actions are not parsed from JSON`,
        );
      }
    }
    for (const key of ["objects", "links", "actions"]) {
      if (consumesJsonArray(semanticRs.text, key)) {
        push(
          findings,
          SEMANTIC_RS_REL,
          `emitter still reads ${key} from the manifest JSON; DTO inventory must generate the roster`,
        );
      }
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

  const domain = readText(repoRoot, DOMAIN_REL);
  if (domain.missing) {
    push(
      findings,
      DOMAIN_REL,
      "canonical-domain is missing; cannot drift the DTO roster against ObjectKey / DispatchTarget",
    );
  } else {
    const objectKeys = parseObjectKeys(domain.text);
    const dispatchTargets = parseDispatchTargets(domain.text);
    if (objectKeys.length === 0) {
      push(findings, DOMAIN_REL, "could not read object_keys! roster");
    } else if (headNames.length > 0) {
      for (const entry of objectKeys) {
        const index = headNames.indexOf(entry.name);
        if (index < 0) {
          push(
            findings,
            `${DTO_RS_REL}:HEADS/${entry.name}`,
            `runtime ObjectKey ${entry.name} is missing from the DTO HEADS roster`,
          );
        }
      }
      for (const name of headNames) {
        if (!objectKeys.some((entry) => entry.name === name)) {
          push(
            findings,
            `${DOMAIN_REL}:ObjectKey/${name}`,
            "DTO HEADS entry is not an ObjectKey (do not auto-publish extra types)",
          );
        }
      }
    }
    if (dispatchTargets.length === 0) {
      push(findings, DOMAIN_REL, "could not read dispatch_targets! roster");
    } else if (actionKeys.length > 0) {
      for (const entry of dispatchTargets) {
        if (!actionKeys.includes(entry.action_key)) {
          push(
            findings,
            `${DTO_RS_REL}:ACTIONS/${entry.action_key}`,
            `runtime DispatchTarget ${entry.action_key} is missing from the DTO ACTIONS roster`,
          );
        }
      }
      for (const key of actionKeys) {
        if (!dispatchTargets.some((entry) => entry.action_key === key)) {
          push(
            findings,
            `${DOMAIN_REL}:DispatchTarget/${key}`,
            "DTO ACTIONS entry is not a DispatchTarget (do not auto-publish extra types)",
          );
        }
      }
    }
  }

  const documentPath = join(repoRoot, "backend/openapi/openapi.yaml");
  if (!existsSync(documentPath)) {
    push(findings, "backend/openapi/openapi.yaml", "published document is missing");
    return { objects: objectCount, actions: actionCount, links: linkCount, findings };
  }

  let document;
  try {
    document = yaml.load(readFileSync(documentPath, "utf8"));
  } catch (error) {
    push(findings, "backend/openapi/openapi.yaml", `cannot parse: ${error.message}`);
    return { objects: objectCount, actions: actionCount, links: linkCount, findings };
  }

  const published = own(own(document, "components"), "schemas");
  if (!isPlainObject(published)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { objects: objectCount, actions: actionCount, links: linkCount, findings };
  }

  for (const spec of CANONICAL_OBJECTS) {
    const schema = own(published, spec.name);
    const loc = `#/components/schemas/${spec.name}`;
    if (!isPlainObject(schema)) {
      push(findings, loc, "composed document is missing a generated Head");
      continue;
    }
    if (!Array.isArray(own(schema, "links")) || !Array.isArray(own(schema, "actions"))) {
      push(
        findings,
        loc,
        "Head schema must carry generated links/actions from the DTO roster",
      );
    }
  }

  const belowFloor =
    objectCount < OBJECT_FLOOR
    || actionCount < ACTION_FLOOR
    || linkCount < LINK_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      DTO_RS_REL,
      `generated ${objectCount}/${OBJECT_FLOOR} objects, ${actionCount}/${ACTION_FLOOR} actions, ${linkCount}/${LINK_FLOOR} links — below the floor`,
    );
  }

  return { objects: objectCount, actions: actionCount, links: linkCount, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const result = evaluateSemanticRoster({ repoRoot });
  const { objects, actions, links, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    objects < OBJECT_FLOOR || actions < ACTION_FLOOR || links < LINK_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
        + `${links}/${LINK_FLOOR} links — below the floor; hand JSON roster is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi semantic-roster gate FAILED: ${findings.length} finding(s), `
        + `${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
        + `${links}/${LINK_FLOOR} links`,
    );
    process.exit(1);
  }
  console.log(
    `openapi semantic-roster gate passed `
      + `(${objects}/${OBJECT_FLOOR} objects, ${actions}/${ACTION_FLOOR} actions, `
      + `${links}/${LINK_FLOOR} links from DTO/DispatchTarget inventory, 0 findings)`,
  );
}
