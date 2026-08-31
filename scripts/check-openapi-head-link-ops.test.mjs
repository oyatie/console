import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { GET_FLOOR as ASOF_GET_FLOOR } from "./check-openapi-hr-asof.mjs";
import {
  CANONICAL_LINKS,
  LINK_FLOOR,
} from "./check-openapi-semantic-contract.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import {
  FENCED_HEADS,
  FENCED_LINK_FLOOR,
  FENCED_LINKS,
  FENCED_PAY_RUN,
  GET_FLOOR,
  INSTANCE_GET_PATHS,
  LINK_FLOOR as OPS_LINK_FLOOR,
  TRAVERSABLE_LINK_FLOOR,
  TRAVERSABLE_LINKS,
  evaluateOpenapiHeadLinkOps,
} from "./check-openapi-head-link-ops.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-head-link-ops.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(specYaml) {
  const root = mkdtempSync(join(tmpdir(), "openapi-head-link-ops-"));
  fixtureRoots.push(root);
  const absolute = join(root, "backend/openapi/openapi.yaml");
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, specYaml);
  return root;
}

function padGets(count) {
  const paths = [];
  for (let i = 0; i < count; i += 1) {
    paths.push(`  /api/v1/pad/${i}:
    get:
      operationId: padGet${i}
      responses:
        '200': { description: ok }`);
  }
  return paths.join("\n");
}

function instanceGet(path, schemaName, operationId) {
  return `  ${path}:
    get:
      operationId: ${operationId}
      parameters:
      - name: id
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Uuid' }
      responses:
        '200':
          description: head
          content:
            application/json:
              schema: { $ref: '#/components/schemas/${schemaName}' }`;
}

function linkYaml(link, extra = "") {
  return [
    `        - key: ${link.key}`,
    `          from: ${link.from}`,
    `          to: ${link.to}`,
    `          field: ${link.field}`,
    `          cardinality: ${link.cardinality}`,
    `          option: ${link.option}`,
    extra,
  ]
    .filter(Boolean)
    .join("\n");
}

function schemaYaml(name, links) {
  const declared = CANONICAL_LINKS.filter((link) => link.from === name);
  const body = declared.map((link) => links(link)).join("\n");
  return `    ${name}:
      type: object
      properties:
        id: { type: string }
      links:
${body || "        []"}`;
}

function spec(paths, linkExtra) {
  const extras = linkExtra ?? {};
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${paths}
${padGets(GET_FLOOR)}
components:
  schemas:
${HEAD_SCHEMA_NAMES.map((name) => schemaYaml(name, (link) => linkYaml(link, extras[link.key] ?? ""))).join("\n")}
`;
}

function honestGets() {
  return [
    instanceGet("/api/v1/companies/{id}", "Company", "getCompany"),
    instanceGet("/api/v1/org-units/{id}", "OrgUnit", "getOrgUnit"),
    instanceGet("/api/v1/job-positions/{id}", "JobPosition", "getJobPosition"),
    instanceGet("/api/v1/persons/{id}", "Person", "getPerson"),
    instanceGet("/api/v1/employments/{id}", "Employment", "getEmployment"),
  ].join("\n");
}

const BOUND = Object.freeze({
  org_unit_parent: "          operationId: getOrgUnit",
  job_position_org_unit: "          operationId: getOrgUnit",
  employment_person: "          operationId: getPerson",
  employment_org_unit: "          operationId: getOrgUnit",
  employment_job_position: "          operationId: getJobPosition",
});

describe("check-openapi-head-link-ops", () => {
  it("exports examined-zero floors and the Chesterton fences", () => {
    assert.equal(GET_FLOOR, 200);
    assert.equal(GET_FLOOR, ASOF_GET_FLOOR);
    assert.equal(OPS_LINK_FLOOR, LINK_FLOOR);
    assert.equal(OPS_LINK_FLOOR, 5);
    assert.equal(TRAVERSABLE_LINK_FLOOR, 5);
    assert.equal(FENCED_LINK_FLOOR, 0);
    assert.deepEqual(
      TRAVERSABLE_LINKS.map((link) => link.key),
      [
        "org_unit_parent",
        "job_position_org_unit",
        "employment_person",
        "employment_org_unit",
        "employment_job_position",
      ],
    );
    assert.deepEqual(
      FENCED_LINKS.map((link) => link.key),
      [],
    );
    assert.deepEqual(INSTANCE_GET_PATHS, {
      Company: "/api/v1/companies/{id}",
      OrgUnit: "/api/v1/org-units/{id}",
      JobPosition: "/api/v1/job-positions/{id}",
      Person: "/api/v1/persons/{id}",
      Employment: "/api/v1/employments/{id}",
    });
    assert.deepEqual(FENCED_HEADS, [FENCED_PAY_RUN]);
    assert.ok(Object.hasOwn(INSTANCE_GET_PATHS, "JobPosition"));
    assert.ok(!Object.hasOwn(INSTANCE_GET_PATHS, "PayRun"));
  });

  it("fails while Employment→Person is schema-only without getPerson", () => {
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(honestGets(), {
        org_unit_parent: "          operationId: getOrgUnit",
        job_position_org_unit: "          operationId: getOrgUnit",
        employment_org_unit: "          operationId: getOrgUnit",
        employment_job_position: "          operationId: getJobPosition",
      })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/Employment/links/employment_person/operationId"
          && /getPerson/.test(finding.message)
          && /schema-only/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.traversable, 4);
  });

  it("fails while Employment→OrgUnit is schema-only without getOrgUnit", () => {
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(honestGets(), {
        org_unit_parent: "          operationId: getOrgUnit",
        job_position_org_unit: "          operationId: getOrgUnit",
        employment_person: "          operationId: getPerson",
        employment_job_position: "          operationId: getJobPosition",
      })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/Employment/links/employment_org_unit/operationId"
          && /getOrgUnit/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("refuses as_of on a linked non-temporal Head", () => {
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(honestGets(), {
        ...BOUND,
        employment_person: "          operationId: getPerson\n          parameters:\n            as_of: '$request.query.as_of'",
      })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location.includes("employment_person/as_of")
          && /no valid-time store/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails while Employment→JobPosition is schema-only without getJobPosition", () => {
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(honestGets(), {
        org_unit_parent: "          operationId: getOrgUnit",
        job_position_org_unit: "          operationId: getOrgUnit",
        employment_person: "          operationId: getPerson",
        employment_org_unit: "          operationId: getOrgUnit",
      })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/Employment/links/employment_job_position/operationId"
          && /getJobPosition/.test(finding.message)
          && /schema-only/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
    assert.equal(result.traversable, 4);
  });

  it("fails while JobPosition instance GET is missing so the link cannot bind", () => {
    const withoutJobPosition = [
      instanceGet("/api/v1/companies/{id}", "Company", "getCompany"),
      instanceGet("/api/v1/org-units/{id}", "OrgUnit", "getOrgUnit"),
      instanceGet("/api/v1/persons/{id}", "Person", "getPerson"),
      instanceGet("/api/v1/employments/{id}", "Employment", "getEmployment"),
    ].join("\n");
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(withoutJobPosition, {
        org_unit_parent: "          operationId: getOrgUnit",
        job_position_org_unit: "          operationId: getOrgUnit",
        employment_person: "          operationId: getPerson",
        employment_org_unit: "          operationId: getOrgUnit",
        employment_job_position: "          operationId: getJobPosition",
      })),
    });
    assert.ok(
      result.findings.some(
        (finding) =>
          finding.location === "#/components/schemas/JobPosition"
          && /not a 200 schema of any instance GET/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when Head FKs with published GET bind that operationId including employment_job_position", () => {
    const result = evaluateOpenapiHeadLinkOps({
      repoRoot: fixture(spec(honestGets(), BOUND)),
    });
    assert.equal(result.findings.length, 0, JSON.stringify(result.findings, null, 2));
    assert.equal(result.links, 5);
    assert.equal(result.traversable, 5);
    assert.equal(result.fenced, 0);
    assert.ok(result.gets >= GET_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until Employment links bind GET operations", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /openapi Head link-ops gate passed/);
    } else {
      assert.match(ran.stderr, /schema-only to\/field is not a traversable operation|below the floor/);
    }
  });
});
