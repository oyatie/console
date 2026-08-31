import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import { HEAD_GET_PERMISSION_OPS } from "./check-openapi-head-get-permissions.mjs";
import {
  CANONICAL_ACTIONS,
  CANONICAL_LINKS,
} from "./check-openapi-semantic-contract.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { DOCS_REL } from "./generate-openapi-docs.mjs";
import {
  ACTION_PERMISSION_FLOOR,
  LINK_OPERATION_FLOOR,
  OPERATION_PERMISSION_FLOOR,
  PERMISSION_ROLE_MANAGE,
  evaluateGeneratedDocsContract,
} from "./check-openapi-generated-docs-contract.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(
  new URL("./check-openapi-generated-docs-contract.mjs", import.meta.url),
);
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-docs-contract-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

const REVERSE = Object.freeze({
  key: "org_unit_job_positions",
  from: "OrgUnit",
  to: "JobPosition",
  field: "org_unit_id",
  cardinality: "one-to-many",
  operationId: "listOrgUnitJobPositions",
});

const LINK_OPS = Object.freeze({
  org_unit_parent: "getOrgUnit",
  job_position_org_unit: "getOrgUnit",
  employment_person: "getPerson",
  employment_org_unit: "getOrgUnit",
  employment_job_position: "getJobPosition",
  [REVERSE.key]: REVERSE.operationId,
});

function yamlQuote(value) {
  return JSON.stringify(value);
}

function linkYaml(link) {
  const operationId = LINK_OPS[link.key];
  return `        - key: ${link.key}
          from: ${link.from}
          to: ${link.to}
          field: ${link.field}
          cardinality: ${link.cardinality}
          operationId: ${operationId}`;
}

function actionYaml(action) {
  return `        - action_key: ${yamlQuote(action.action_key)}
          permissions:
            - ${PERMISSION_ROLE_MANAGE}`;
}

function schemaYaml(name) {
  const links = [...CANONICAL_LINKS, REVERSE].filter((link) => link.from === name);
  const actions = CANONICAL_ACTIONS.filter((action) => action.object === name);
  const linksBlock = links.length === 0
    ? "      links: []\n"
    : `      links:\n${links.map(linkYaml).join("\n")}\n`;
  const actionsBlock = actions.length === 0
    ? "      actions: []\n"
    : `      actions:\n${actions.map(actionYaml).join("\n")}\n`;
  return `    ${name}:
      type: object
      properties:
        id: { type: string }
${linksBlock}${actionsBlock}`;
}

function opYaml(spec, { permissions = true, extra = "" } = {}) {
  const perm = permissions
    ? `      permissions:
      - employee_directory_read
`
    : "";
  return `  ${spec.path}:
    get:
      operationId: ${spec.operationId}
${perm}${extra}      responses:
        '200': { description: ok }`;
}

function openapiYaml({ opPermissions = true, extraPaths = "" } = {}) {
  const paths = HEAD_GET_PERMISSION_OPS.map((item) =>
    opYaml(item, { permissions: opPermissions }),
  ).join("\n");
  const schemas = HEAD_SCHEMA_NAMES.map(schemaYaml).join("\n");
  return `openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
${paths}
${extraPaths}
components:
  schemas:
${schemas}
`;
}

function linkItem(link, { operationId = true } = {}) {
  const op = LINK_OPS[link.key];
  const attr = operationId ? ` data-operation-id="${op}"` : "";
  return `<li data-link="${link.key}"${attr}><code>${link.key}</code></li>`;
}

function actionItem(action, { permissions = true } = {}) {
  const attr = permissions ? ` data-permissions="${PERMISSION_ROLE_MANAGE}"` : "";
  return `<li data-action="${action.action_key}"${attr}><code>${action.action_key}</code></li>`;
}

function headSection(name, { linkOps = true, actionPerms = true } = {}) {
  const links = [...CANONICAL_LINKS, REVERSE].filter((link) => link.from === name);
  const actions = CANONICAL_ACTIONS.filter((action) => action.object === name);
  return `<section id="schema-${name}" data-schema="${name}" data-kind="head"><h3>${name}</h3><ul class="links">${links.map((link) => linkItem(link, { operationId: linkOps })).join("")}</ul><ul class="actions">${actions.map((action) => actionItem(action, { permissions: actionPerms })).join("")}</ul></section>`;
}

function opRow(spec, { permissions = true, extraPermissions = null } = {}) {
  const key = `GET ${spec.path}`;
  let attr = "";
  if (extraPermissions) attr = ` data-permissions="${extraPermissions}"`;
  else if (permissions) attr = ` data-permissions="employee_directory_read"`;
  return `<tr data-operation="${key}"${attr}><th>GET</th><td>${spec.path}</td></tr>`;
}

function docsHtml({
  linkOps = true,
  actionPerms = true,
  opPerms = true,
  extraRows = "",
} = {}) {
  const heads = HEAD_SCHEMA_NAMES.map((name) =>
    headSection(name, { linkOps, actionPerms }),
  ).join("");
  const rows = HEAD_GET_PERMISSION_OPS.map((item) => opRow(item, { permissions: opPerms })).join("");
  return `<!DOCTYPE html><html><body>${heads}<section id="operations"><table>${rows}${extraRows}</table></section></body></html>\n`;
}

function greenFiles(overrides = {}) {
  return {
    "backend/openapi/openapi.yaml": openapiYaml(),
    [DOCS_REL]: docsHtml(),
    ...overrides,
  };
}

describe("check-openapi-generated-docs-contract", () => {
  it("exports examined-zero floors for composed Palantir-class fields", () => {
    assert.equal(LINK_OPERATION_FLOOR, 6);
    assert.equal(ACTION_PERMISSION_FLOOR, 13);
    assert.equal(OPERATION_PERMISSION_FLOOR, 11);
    assert.equal(CANONICAL_LINKS.length + 1, LINK_OPERATION_FLOOR);
    assert.equal(CANONICAL_ACTIONS.length, ACTION_PERMISSION_FLOOR);
    assert.equal(HEAD_GET_PERMISSION_OPS.length, OPERATION_PERMISSION_FLOOR);
    assert.equal(PERMISSION_ROLE_MANAGE, "role_manage");
  });

  it("fails when generated docs omit composed link operationId", () => {
    const result = evaluateGeneratedDocsContract({
      repoRoot: fixture(greenFiles({ [DOCS_REL]: docsHtml({ linkOps: false }) })),
    });
    assert.ok(
      result.findings.some((finding) =>
        finding.location.includes("org_unit_job_positions/operationId")
        && /listOrgUnitJobPositions/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when generated docs omit composed action permissions", () => {
    const result = evaluateGeneratedDocsContract({
      repoRoot: fixture(greenFiles({ [DOCS_REL]: docsHtml({ actionPerms: false }) })),
    });
    assert.ok(
      result.findings.some((finding) =>
        finding.location.includes("company.revise/permissions")
        && /role_manage/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when generated docs omit composed Head GET/list operation permissions", () => {
    const result = evaluateGeneratedDocsContract({
      repoRoot: fixture(greenFiles({ [DOCS_REL]: docsHtml({ opPerms: false }) })),
    });
    assert.ok(
      result.findings.some((finding) =>
        finding.location.includes("GET /api/v1/org-units/{id}/permissions")
        && /employee_directory_read/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("fails when docs invent permissions on a non-Head GET (Feature::ALL)", () => {
    const extraPaths = `  /api/v1/leave/requests:
    get:
      operationId: listLeaveRequests
      responses:
        '200': { description: ok }
`;
    const extraRows = '<tr data-operation="GET /api/v1/leave/requests" data-permissions="employee_directory_read"><th>GET</th><td>/api/v1/leave/requests</td></tr>';
    const result = evaluateGeneratedDocsContract({
      repoRoot: fixture({
        "backend/openapi/openapi.yaml": openapiYaml({ extraPaths }),
        [DOCS_REL]: docsHtml({ extraRows }),
      }),
    });
    assert.ok(
      result.findings.some((finding) =>
        finding.location.includes("GET /api/v1/leave/requests/permissions")
        && /Feature::ALL/.test(finding.message),
      ),
      JSON.stringify(result.findings, null, 2),
    );
  });

  it("passes when generated docs copy composed link, action, and operation Palantir-class fields", () => {
    const result = evaluateGeneratedDocsContract({
      repoRoot: fixture(greenFiles()),
    });
    assert.deepEqual(result.findings, []);
    assert.equal(result.links, LINK_OPERATION_FLOOR);
    assert.equal(result.actions, ACTION_PERMISSION_FLOOR);
    assert.equal(result.permissionedOps, OPERATION_PERMISSION_FLOOR);
  });

  it("CLI exits non-zero on the repository tree until docs generate Palantir-class fields", () => {
    const ran = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    if (ran.status === 0) {
      assert.match(ran.stdout, /Palantir-class field gate passed/);
    } else {
      assert.match(
        ran.stderr,
        /operationId|permissions|Palantir-class field gate FAILED/,
      );
    }
  });
});
