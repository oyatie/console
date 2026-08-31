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
} from "./check-openapi-semantic-contract.mjs";
import {
  GENERATED_FLOOR,
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  MANIFEST_REL,
  expectedGeneratedSchema,
  evaluateSemanticGenerate,
} from "./check-openapi-semantic-generate.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-semantic-generate.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-semantic-generate-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function inputSchema(name) {
  return {
    type: "object",
    additionalProperties: false,
    required: ["marker"],
    properties: {
      marker: { type: "string", const: name },
    },
  };
}

function headSchema(name) {
  return {
    type: "object",
    required: ["id"],
    properties: {
      id: { $ref: "#/components/schemas/Uuid" },
    },
    description: `${name} head`,
  };
}

function completeManifest() {
  const schemas = {};
  for (const name of INPUT_SCHEMA_NAMES) schemas[name] = inputSchema(name);
  schemas.EmploymentAttributesInput = inputSchema("EmploymentAttributesInput");
  schemas.OrgUnitSourceBinding = inputSchema("OrgUnitSourceBinding");
  for (const spec of CANONICAL_OBJECTS) schemas[spec.name] = headSchema(spec.name);
  return {
    version: 1,
    objects: CANONICAL_OBJECTS.map((spec) => ({
      name: spec.name,
      object_key: spec.object_key,
      actions: CANONICAL_ACTIONS.filter((action) => action.object === spec.name).map(
        (action) => action.action_key,
      ),
    })),
    links: CANONICAL_LINKS.map((link) => ({ ...link })),
    actions: CANONICAL_ACTIONS.map((action) => ({
      action_key: action.action_key,
      object: action.object,
      object_key: action.object_key,
      input: action.input,
      four_eyes: action.four_eyes,
      edits: [...action.edits],
      permissions: ["role_manage"],
      concurrency: {
        command_id: "tenant_global_idempotency",
        expected_revision: "optional_cas",
      },
    })),
    schemas,
  };
}

function generatedOpenApi(manifest) {
  const schemas = {};
  for (const name of GENERATED_SCHEMA_NAMES) {
    schemas[name] = expectedGeneratedSchema(manifest, name);
  }
  return yaml.dump({
    openapi: "3.1.0",
    info: { title: "Fixture", version: "0.0.1" },
    paths: {},
    components: { schemas },
  });
}

const GREEN_EMITTER = `pub fn generated_schema_yaml() -> Result<Vec<(String, String)>, String> {
    let _ = include_str!("semantic_manifest.json");
    Ok(Vec::new())
}
`;

const GREEN_GEN = `use console_contracts::compose_document_with_owned;
fn main() {
    let owned = console_contracts::semantic::generated_schema_yaml().unwrap();
    let _ = compose_document_with_owned;
    let _ = owned;
}
`;

const GREEN_LIB = `pub struct OwnedNamedYaml { pub name: String, pub body: String }
pub fn compose_document_with_owned() {}
`;

function greenFiles(manifest, { registry = "pub const ALL_FRAGMENTS: &[&Fragment] = &[];\n" } = {}) {
  return {
    [MANIFEST_REL]: JSON.stringify(manifest, null, 2),
    "backend/crates/contracts/src/semantic.rs": GREEN_EMITTER,
    "backend/crates/contracts/src/bin/console_openapi_gen.rs": GREEN_GEN,
    "backend/crates/contracts/src/lib.rs": GREEN_LIB,
    "backend/crates/contracts/src/gen_registry.rs": registry,
    "backend/openapi/openapi.yaml": generatedOpenApi(manifest),
  };
}

describe("semantic-generate floors", () => {
  it("locks examined-zero to 13 inputs + 2 nested + 6 heads", () => {
    assert.equal(INPUT_SCHEMA_NAMES.length, ACTION_FLOOR);
    assert.equal(HEAD_SCHEMA_NAMES.length, 6);
    assert.equal(GENERATED_FLOOR, 21);
    assert.equal(new Set(GENERATED_SCHEMA_NAMES).size, GENERATED_FLOOR);
  });
});

describe("evaluateSemanticGenerate", () => {
  it("fails when the manifest is absent", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": "openapi: 3.1.0\ninfo: {title: x, version: 0}\npaths: {}\n",
    });
    const result = evaluateSemanticGenerate({ repoRoot: root });
    assert.equal(result.generated, 0);
    assert.ok(result.findings.some((finding) => /semantic manifest is absent/.test(finding.message)));
  });

  it("fails while gen_registry include_str's a generated Input YAML", () => {
    const manifest = completeManifest();
    const root = fixture(
      greenFiles(manifest, {
        registry: `
pub const ONTOLOGY_FRAGMENT: Fragment = Fragment {
    schemas: &[
        NamedYaml {
            name: "CompanyReviseInput",
            body: include_str!("../../ontology/rest/openapi/schemas/CompanyReviseInput.yaml"),
        },
    ],
};
`,
      }),
    );
    const result = evaluateSemanticGenerate({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) =>
        /include_str's CompanyReviseInput\.yaml/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("fails when the composed document drifts from the manifest", () => {
    const manifest = completeManifest();
    const files = greenFiles(manifest);
    const drifted = yaml.load(files["backend/openapi/openapi.yaml"]);
    drifted.components.schemas.CompanyReviseInput.properties.marker.type = "integer";
    files["backend/openapi/openapi.yaml"] = yaml.dump(drifted);
    const root = fixture(files);
    const result = evaluateSemanticGenerate({ repoRoot: root });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("CompanyReviseInput")
          && /does not match the semantic manifest/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("passes a fixture whose composed schemas are generated from the manifest", () => {
    const manifest = completeManifest();
    const root = fixture(greenFiles(manifest));
    const result = evaluateSemanticGenerate({ repoRoot: root });
    assert.deepEqual(result.findings, []);
    assert.equal(result.generated, GENERATED_FLOOR);
    assert.equal(result.actions, ACTION_FLOOR);
    assert.equal(result.objects, CANONICAL_OBJECTS.length);
    assert.equal(result.links, CANONICAL_LINKS.length);
  });
});

describe("cli", () => {
  it("accepts the live composed document as generated from the semantic manifest", () => {
    const run = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stdout, /semantic-generate gate passed/);
  });
});
