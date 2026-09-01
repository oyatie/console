import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { KILL_SWITCH_PATH } from "./check-openapi-execute-outcome.mjs";
import {
  COMPOSED_CONTRACT_SCHEMAS,
  CONTRACT_SCHEMA_FLOOR,
  DOCS_REL,
  PATH_SCHEMA_FLOOR,
  SDK_GENERATED_REL,
  evaluateGeneratedPathSchemas,
} from "./check-openapi-generated-path-schemas.mjs";
import {
  DOCS_REL as GENERATED_DOCS_REL,
  generateDocsFiles,
} from "./generate-openapi-docs.mjs";
import {
  SDK_GENERATED_REL as GENERATED_SDK_REL,
  generateSdkFiles,
} from "./generate-openapi-ts-sdk.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-generated-path-schemas.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-path-schemas-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function contractDoc({ omitSdk = false, omitDocs = false, extraPaths = "" } = {}) {
  const schemas = {};
  const paths = [];
  for (const name of COMPOSED_CONTRACT_SCHEMAS) {
    schemas[name] = {
      type: "object",
      properties: { id: { type: "string" } },
    };
    paths.push(`  /api/v1/contract/${name}:
    get:
      operationId: get${name}
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/${name}"`);
  }
  const document = yaml.load(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${paths.join("\n")}
${extraPaths}
components:
  schemas: ${JSON.stringify(schemas)}
`);
  const generatedSdk = generateSdkFiles(document);
  const generatedDocs = generateDocsFiles(document);
  const files = {
    "backend/openapi/openapi.yaml": yaml.dump(document),
  };
  if (!omitSdk) Object.assign(files, generatedSdk.files);
  if (!omitDocs) Object.assign(files, generatedDocs.files);
  if (omitSdk) {
    files[SDK_GENERATED_REL] = "export type Company = { id: string };\n";
    files["sdk/typescript/package.json"] = generatedSdk.files["sdk/typescript/package.json"];
    files["sdk/typescript/src/index.ts"] = generatedSdk.files["sdk/typescript/src/index.ts"];
  }
  if (omitDocs) {
    files[DOCS_REL] = "<!DOCTYPE html><html><body></body></html>\n";
  }
  return files;
}

describe("generated path-schema floors", () => {
  it("locks examined-zero to the composed Palantir-class names already on origin/dev", () => {
    assert.equal(CONTRACT_SCHEMA_FLOOR, 10);
    assert.equal(COMPOSED_CONTRACT_SCHEMAS.length, CONTRACT_SCHEMA_FLOOR);
    assert.equal(PATH_SCHEMA_FLOOR, CONTRACT_SCHEMA_FLOOR);
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("GateOutcome"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("ConditionValue"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("OverrideSummary"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("DraftRecord"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("ObjectTypeDetail"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("OntologyActionExecuteOutcome"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("EmployeeExitCaseResponse"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("AbsenceExitDashboardResponse"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("GateKind"));
    assert.ok(COMPOSED_CONTRACT_SCHEMAS.includes("GateStatus"));
  });
});

describe("evaluateGeneratedPathSchemas", () => {
  it("fails when generated SDK drops a composed path schema", () => {
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture(contractDoc({ omitSdk: true })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes(`${SDK_GENERATED_REL}:GateOutcome`)
          && /drops this composed path schema/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when generated docs drop a composed path schema", () => {
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture(contractDoc({ omitDocs: true })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes(`${DOCS_REL}:ConditionValue`)
          && /#1018/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails examined-zero when paths $ref no schemas", () => {
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture({
        "backend/openapi/openapi.yaml": `openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths: {}
components:
  schemas: {}
`,
        [SDK_GENERATED_REL]: "export type Company = { id: string };\n",
        [DOCS_REL]: "<!DOCTYPE html><html><body></body></html>\n",
      }),
    });
    assert.ok(
      result.findings.some((finding) => /below the floor/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.pathSchemas, 0);
    assert.equal(result.contract, 0);
  });

  it("fails when kill-switch 200 binds a Palantir-class schema", () => {
    const extraPaths = `  ${KILL_SWITCH_PATH}:
    post:
      operationId: postKillSwitch
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/GateOutcome"
`;
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture(contractDoc({ extraPaths })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes(KILL_SWITCH_PATH)
          && /kill-switch/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when kill-switch grows Feature::ALL permissions", () => {
    const extraPaths = `  ${KILL_SWITCH_PATH}:
    post:
      operationId: postKillSwitch
      permissions:
        - employee_directory_read
      responses:
        "200": { description: ok }
`;
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture(contractDoc({ extraPaths })),
    });
    assert.ok(
      result.findings.some((finding) => /Feature::ALL/.test(finding.message)),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when SDK and docs copy composed path schemas", () => {
    const result = evaluateGeneratedPathSchemas({
      repoRoot: fixture(contractDoc()),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.contract, CONTRACT_SCHEMA_FLOOR);
    assert.ok(result.pathSchemas >= PATH_SCHEMA_FLOOR);
    assert.equal(result.sdk, result.pathSchemas);
    assert.equal(result.docs, result.pathSchemas);
    assert.equal(GENERATED_SDK_REL, SDK_GENERATED_REL);
    assert.equal(GENERATED_DOCS_REL, DOCS_REL);
  });
});

describe("check-openapi-generated-path-schemas CLI", () => {
  it("exits non-zero on the repository tree until SDK/docs emit composed path schemas", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /generated path-schema gate passed/);
    } else {
      assert.match(ran.stderr, /generated path-schema gate FAILED|drops this composed path schema/);
    }
  });
});
