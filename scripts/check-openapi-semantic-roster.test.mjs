import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
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
  MANIFEST_REL,
  OPENAPI_GEN_REL,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import {
  DOMAIN_REL,
  evaluateSemanticRoster,
} from "./check-openapi-semantic-roster.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-semantic-roster.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-semantic-roster-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function dispatchVariant(actionKey) {
  return actionKey
    .split(/[._]/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function headFields(name) {
  const fields = {
    Company: ["org_id", "legal_name", "reg_no", "version"],
    OrgUnit: ["id", "name", "parent_id", "version"],
    JobPosition: ["job_position_id", "org_unit_id", "version", "attributes"],
    Person: ["id", "display_name", "legal_name", "version"],
    Employment: ["id", "person_id", "org_unit_id", "job_position_id", "appointed_on"],
    PayRun: ["id", "period_start", "period_end", "source_label", "status", "payable"],
  };
  return (fields[name] ?? ["id"]).map((field) => `    pub ${field}: Uuid,`).join("\n");
}

function greenDto() {
  const heads = CANONICAL_OBJECTS.map(
    (spec) => `pub struct ${spec.name} {\n    ${headFields(spec.name)},\n}\n`,
  ).join("\n");
  const inputs = CANONICAL_ACTIONS.map(
    (spec) => `pub struct ${spec.input} {\n    pub marker: String,\n}\n`,
  ).join("\n");
  const headEntries = CANONICAL_OBJECTS.map(
    (spec) => `    HeadEntry { name: "${spec.name}", object_key: "${spec.object_key}" },`,
  ).join("\n");
  const linkEntries = CANONICAL_LINKS.map(
    (spec) =>
      `    LinkEntry { key: "${spec.key}", from: "${spec.from}", to: "${spec.to}", field: "${spec.field}", cardinality: "${spec.cardinality}", option: ${spec.option} },`,
  ).join("\n");
  const actionEntries = CANONICAL_ACTIONS.map(
    (spec) =>
      `    ActionEntry { action_key: "${spec.action_key}", object: "${spec.object}", object_key: "${spec.object_key}", input: "${spec.input}", edits: &["${spec.edits[0]}"] },`,
  ).join("\n");
  return `${heads}
${inputs}
pub struct EmploymentAttributesInput {}
pub struct OrgUnitSourceBinding {}
pub struct Uuid;
pub struct Timestamp;
pub struct IsoDate;
pub struct JsonObject;
pub(super) struct HeadEntry { pub name: &'static str, pub object_key: &'static str }
pub(super) struct LinkEntry { pub key: &'static str, pub from: &'static str, pub to: &'static str, pub field: &'static str, pub cardinality: &'static str, pub option: bool }
pub(super) struct ActionEntry { pub action_key: &'static str, pub object: &'static str, pub object_key: &'static str, pub input: &'static str, pub edits: &'static [&'static str] }

pub(super) const HEADS: &[HeadEntry] = &[
${headEntries}
];
pub(super) const LINKS: &[LinkEntry] = &[
${linkEntries}
];
pub(super) const ACTIONS: &[ActionEntry] = &[
${actionEntries}
];
pub(super) fn dto_objects() -> Result<Vec<Json>, SemanticError> { Ok(Vec::new()) }
pub(super) fn dto_links() -> Vec<Json> { Vec::new() }
pub(super) fn dto_actions() -> Result<Vec<Json>, SemanticError> { Ok(Vec::new()) }
`;
}

function greenDomain() {
  const objects = CANONICAL_OBJECTS.map(
    (spec) => `    ${spec.name} => "${spec.object_key}",`,
  ).join("\n");
  const actions = CANONICAL_ACTIONS.map(
    (spec) => `    ${dispatchVariant(spec.action_key)} => "${spec.action_key}", ${spec.object};`,
  ).join("\n");
  return `object_keys! {
${objects}
}

dispatch_targets! {
${actions}
}
`;
}

function greenOpenApi() {
  const schemas = {};
  for (const spec of CANONICAL_OBJECTS) {
    schemas[spec.name] = {
      type: "object",
      properties: { id: { type: "string" } },
      links: CANONICAL_LINKS.filter((link) => link.from === spec.name),
      actions: CANONICAL_ACTIONS.filter((action) => action.object === spec.name).map(
        (action) => ({ action_key: action.action_key }),
      ),
    };
  }
  return yaml.dump({
    openapi: "3.1.0",
    info: { title: "Fixture", version: "0.0.1" },
    paths: {},
    components: { schemas },
  });
}

const GREEN_EMITTER = `pub fn generated_schema_yaml() -> Result<Vec<(String, String)>, String> {
    let root = parse_json(include_str!("semantic_manifest.json"))?;
    if root.get("objects").is_some() || root.get("links").is_some() || root.get("actions").is_some() {
        return Err("semantic manifest must not carry objects/links/actions; roster is generated from semantic_dtos".into());
    }
    let _ = semantic_dtos::dto_objects();
    let _ = semantic_dtos::dto_links();
    let _ = semantic_dtos::dto_actions();
    Ok(Vec::new())
}
`;

const GREEN_GEN = `use console_contracts::compose_document_with_owned;
fn main() {
    let owned = console_contracts::generated_schema_yaml().unwrap();
    let _ = compose_document_with_owned;
    let _ = owned;
}
`;

function greenFiles(extra = {}) {
  return {
    [MANIFEST_REL]: JSON.stringify({ version: 1 }, null, 2),
    [DTO_RS_REL]: greenDto(),
    [SEMANTIC_RS_REL]: GREEN_EMITTER,
    [OPENAPI_GEN_REL]: GREEN_GEN,
    [DOMAIN_REL]: greenDomain(),
    "backend/openapi/openapi.yaml": greenOpenApi(),
    ...extra,
  };
}

describe("semantic-roster floors", () => {
  it("locks examined-zero to six Heads, thirteen DispatchTargets, five Head FKs", () => {
    assert.equal(OBJECT_FLOOR, 6);
    assert.equal(ACTION_FLOOR, 13);
    assert.equal(LINK_FLOOR, 5);
    assert.equal(CANONICAL_OBJECTS.length, OBJECT_FLOOR);
    assert.equal(CANONICAL_ACTIONS.length, ACTION_FLOOR);
    assert.equal(CANONICAL_LINKS.length, LINK_FLOOR);
  });
});

describe("evaluateSemanticRoster", () => {
  it("fails when the manifest is absent", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": "openapi: 3.1.0\ninfo: {title: x, version: 0}\npaths: {}\n",
    });
    const result = evaluateSemanticRoster({ repoRoot: root });
    assert.equal(result.objects, 0);
    assert.ok(
      result.findings.some((finding) => /semantic manifest is absent/.test(finding.message)),
    );
  });

  it("fails while objects/links/actions remain a hand-authored JSON catalog", () => {
    const root = fixture(
      greenFiles({
        [MANIFEST_REL]: JSON.stringify(
          {
            version: 1,
            objects: CANONICAL_OBJECTS.map((spec) => ({
              name: spec.name,
              object_key: spec.object_key,
              actions: [],
            })),
            links: CANONICAL_LINKS.map((link) => ({ ...link })),
            actions: CANONICAL_ACTIONS.map((action) => ({
              action_key: action.action_key,
              object: action.object,
            })),
          },
          null,
          2,
        ),
      }),
    );
    const result = evaluateSemanticRoster({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /hand-authored JSON catalog/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when the emitter still parses objects from the manifest JSON", () => {
    const root = fixture(
      greenFiles({
        [SEMANTIC_RS_REL]: `pub fn generated_schema_yaml() -> Result<Vec<(String, String)>, String> {
    let root = parse_json(include_str!("semantic_manifest.json"))?;
    let objects = root
        .get("objects")
        .and_then(Json::as_array)
        .ok_or("objects")?;
    let _ = semantic_dtos::dto_objects();
    let _ = semantic_dtos::dto_links();
    let _ = semantic_dtos::dto_actions();
    let _ = objects;
    Ok(Vec::new())
}
`,
      }),
    );
    const result = evaluateSemanticRoster({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) =>
        /still reads objects from the manifest JSON/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("fails when a runtime DispatchTarget is missing from the DTO roster", () => {
    const domain = greenDomain().replace(
      "dispatch_targets! {",
      'dispatch_targets! {\n    ExtraInvent => "inventory.extra", Company;',
    );
    const root = fixture(greenFiles({ [DOMAIN_REL]: domain }));
    const result = evaluateSemanticRoster({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) =>
        /inventory\.extra/.test(finding.message) && /missing from the DTO ACTIONS roster/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("passes a fixture whose roster is emitted from the DTO / DispatchTarget inventory", () => {
    const root = fixture(greenFiles());
    const result = evaluateSemanticRoster({ repoRoot: root });
    assert.deepEqual(result.findings, []);
    assert.equal(result.objects, OBJECT_FLOOR);
    assert.equal(result.actions, ACTION_FLOOR);
    assert.equal(result.links, LINK_FLOOR);
  });
});

describe("cli", () => {
  it("accepts the live roster only when objects/links/actions are generated from DTO inventory", () => {
    const run = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    assert.equal(run.status, 0, run.stderr + run.stdout);
    assert.match(run.stdout, /semantic-roster gate passed/);
  });
});
