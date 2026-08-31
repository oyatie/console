import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  GENERATED_FLOOR,
  GENERATED_SCHEMA_NAMES,
  HEAD_SCHEMA_NAMES,
  INPUT_SCHEMA_NAMES,
  MANIFEST_REL,
  NESTED_INPUT_SCHEMAS,
} from "./check-openapi-semantic-generate.mjs";
import { TYPED_ACTION_GENERATED_REL } from "./check-openapi-typed-action-codecs.mjs";
import {
  DTO_RS_REL,
  HEAD_DTO_SOURCES,
  SHAPE_FLOOR,
  evaluateDtoDerivedShapes,
} from "./check-openapi-dto-derived-shapes.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-dto-derived-shapes.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-dto-derived-shapes-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function dtoModule() {
  const structs = GENERATED_SCHEMA_NAMES.map((name) => {
    const field = name === "PayRun" ? "    pub payable: bool,\n" : "    pub id: Uuid,\n";
    const extra =
      name === "Company"
        ? "    pub org_id: Uuid,\n    pub legal_name: Option<String>,\n    pub reg_no: Option<String>,\n    pub version: i64,\n"
        : name === "OrgUnit"
          ? "    pub id: Uuid,\n    pub name: Option<String>,\n    pub parent_id: Option<Uuid>,\n    pub version: i64,\n"
          : name === "JobPosition"
            ? "    pub job_position_id: Uuid,\n    pub org_unit_id: Uuid,\n    pub version: i64,\n    pub attributes: Map,\n"
            : name === "Person"
              ? "    pub id: Uuid,\n    pub display_name: Option<String>,\n    pub legal_name: Option<String>,\n    pub version: i64,\n"
              : name === "Employment"
                ? "    pub id: Uuid,\n    pub person_id: Option<Uuid>,\n    pub org_unit_id: Option<Uuid>,\n    pub job_position_id: Option<Uuid>,\n    pub appointed_on: Timestamp,\n"
                : name === "PayRun"
                  ? "    pub id: Uuid,\n    pub period_start: IsoDate,\n    pub period_end: IsoDate,\n    pub source_label: String,\n    pub status: String,\n    pub payable: bool,\n"
                  : field;
    return `pub struct ${name} {\n${extra}}\n`;
  }).join("\n");
  return `pub fn dto_schema_bags() -> Vec<(&'static str, ())> { Vec::new() }\n${structs}\n`;
}

function headSources() {
  return {
    "backend/crates/ontology/canonical-adapter-postgres/src/company.rs":
      "pub struct CompanyHead {\n    pub org_id: Uuid,\n    pub legal_name: Option<String>,\n    pub reg_no: Option<String>,\n    pub version: i64,\n}\n",
    "backend/crates/ontology/canonical-adapter-postgres/src/org_unit.rs":
      "pub struct OrgUnitHead {\n    pub id: Uuid,\n    pub name: Option<String>,\n    pub parent_id: Option<Uuid>,\n    pub version: i64,\n}\n",
    "backend/crates/ontology/canonical-adapter-postgres/src/job_position.rs":
      "pub struct JobPositionView {\n    pub job_position_id: Uuid,\n    pub org_unit_id: Uuid,\n    pub version: i64,\n    pub attributes: Value,\n}\n",
    "backend/crates/ontology/canonical-adapter-postgres/src/person.rs":
      "pub struct PersonHead {\n    pub id: Uuid,\n    pub display_name: Option<String>,\n    pub legal_name: Option<String>,\n    pub version: i64,\n}\n",
    "backend/crates/ontology/canonical-adapter-postgres/src/employment.rs":
      "pub struct EmploymentHead {\n    pub id: Uuid,\n    pub person_id: Option<Uuid>,\n    pub org_unit_id: Option<Uuid>,\n    pub job_position_id: Option<Uuid>,\n    #[serde(with = \"time::serde::rfc3339\")]\n    pub appointed_on: OffsetDateTime,\n}\n",
  };
}

function publishedDoc() {
  const schemas = {};
  for (const name of GENERATED_SCHEMA_NAMES) {
    const properties = {};
    if (name === "Company") {
      properties.org_id = { type: "string" };
      properties.legal_name = { type: ["string", "null"] };
      properties.reg_no = { type: ["string", "null"] };
      properties.version = { type: "integer" };
    } else if (name === "OrgUnit") {
      properties.id = { type: "string" };
      properties.name = { type: ["string", "null"] };
      properties.parent_id = { type: "string" };
      properties.version = { type: "integer" };
    } else if (name === "JobPosition") {
      properties.job_position_id = { type: "string" };
      properties.org_unit_id = { type: "string" };
      properties.version = { type: "integer" };
      properties.attributes = { type: "object" };
    } else if (name === "Person") {
      properties.id = { type: "string" };
      properties.display_name = { type: ["string", "null"] };
      properties.legal_name = { type: ["string", "null"] };
      properties.version = { type: "integer" };
    } else if (name === "Employment") {
      properties.id = { type: "string" };
      properties.person_id = { type: "string" };
      properties.org_unit_id = { type: "string" };
      properties.job_position_id = { type: "string" };
      properties.appointed_on = { type: "string" };
    } else if (name === "PayRun") {
      properties.id = { type: "string" };
      properties.period_start = { type: "string" };
      properties.period_end = { type: "string" };
      properties.source_label = { type: "string" };
      properties.status = { type: "string" };
      properties.payable = { type: "boolean", const: false };
    } else {
      properties.id = { type: "string" };
    }
    schemas[name] = {
      type: "object",
      additionalProperties: INPUT_SCHEMA_NAMES.includes(name) || NESTED_INPUT_SCHEMAS.includes(name)
        ? false
        : undefined,
      properties,
    };
  }
  return yaml.dump({
    openapi: "3.1.0",
    info: { title: "Fixture", version: "0.0.1" },
    paths: {},
    components: { schemas },
  });
}

function codecRust() {
  return CODEC_NAMES_STUB();
}

function CODEC_NAMES_STUB() {
  const names = [...INPUT_SCHEMA_NAMES, ...NESTED_INPUT_SCHEMAS];
  return names
    .map(
      (name) => `struct ${name} {
    id: String,
}
`,
    )
    .join("\n");
}

function greenFiles(extra = {}) {
  return {
    [MANIFEST_REL]: JSON.stringify({
      version: 1,
      objects: [],
      links: [],
      actions: [],
    }),
    [DTO_RS_REL]: dtoModule(),
    "backend/crates/contracts/src/semantic.rs":
      "mod semantic_dtos;\nfn generated_schema_yaml() { let _ = dto_schema_bags(); }\n",
    "backend/crates/contracts/src/bin/console_openapi_gen.rs":
      "fn main() { let _ = generated_schema_yaml(); let _ = compose_document_with_owned; }\n",
    "backend/openapi/openapi.yaml": publishedDoc(),
    [TYPED_ACTION_GENERATED_REL]: codecRust(),
    ...headSources(),
    ...extra,
  };
}

describe("dto-derived-shapes floors", () => {
  it("locks examined-zero to 13 inputs + 2 nested + 6 heads", () => {
    assert.equal(INPUT_SCHEMA_NAMES.length, 13);
    assert.equal(HEAD_SCHEMA_NAMES.length, 6);
    assert.equal(NESTED_INPUT_SCHEMAS.length, 2);
    assert.equal(SHAPE_FLOOR, 21);
    assert.equal(HEAD_DTO_SOURCES.length, 5);
    assert.equal(new Set(GENERATED_SCHEMA_NAMES).size, GENERATED_FLOOR);
  });
});

describe("evaluateDtoDerivedShapes", () => {
  it("fails while the manifest still holds hand-authored property bags", () => {
    const files = greenFiles();
    files[MANIFEST_REL] = JSON.stringify({
      version: 1,
      objects: [],
      links: [],
      actions: [],
      schemas: {
        Company: { type: "object", properties: { org_id: { type: "string" } } },
      },
    });
    const root = fixture(files);
    const result = evaluateDtoDerivedShapes({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /hand-authored JSON literal/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when the DTO inventory is absent", () => {
    const files = greenFiles();
    delete files[DTO_RS_REL];
    const root = fixture(files);
    const result = evaluateDtoDerivedShapes({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /DTO inventory is absent/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when the emitter still reads property bags from the manifest", () => {
    const files = greenFiles({
      "backend/crates/contracts/src/semantic.rs":
        'fn generated_schema_yaml() { let _ = root.get("schemas").and_then(as_object); let _ = dto_schema_bags(); }\n',
    });
    const root = fixture(files);
    const result = evaluateDtoDerivedShapes({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /still reads property bags from the manifest/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when a Head DTO field is missing from the inventory", () => {
    const files = greenFiles();
    files[DTO_RS_REL] = files[DTO_RS_REL].replace(
      "    pub legal_name: Option<String>,\n    pub reg_no: Option<String>,\n    pub version: i64,\n",
      "    pub legal_name: Option<String>,\n    pub version: i64,\n",
    );
    const root = fixture(files);
    const result = evaluateDtoDerivedShapes({ repoRoot: root });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("Company")
          && /drifted from CompanyHead/.test(finding.message)
          && /reg_no/.test(finding.message),
      ),
      JSON.stringify(result.findings),
    );
  });

  it("passes a fixture whose property bags are generated from DTO types", () => {
    const root = fixture(greenFiles());
    const result = evaluateDtoDerivedShapes({ repoRoot: root });
    assert.deepEqual(result.findings, []);
    assert.equal(result.shapes, SHAPE_FLOOR);
    assert.equal(result.heads, HEAD_DTO_SOURCES.length);
  });
});

describe("cli", () => {
  it("accepts the live document only when Head/Input shapes are generated from DTO types", () => {
    const run = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stdout, /dto-derived-shapes gate passed/);
  });
});
