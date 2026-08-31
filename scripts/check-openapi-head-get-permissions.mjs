// Head GET/list permissions published-contract gate.
//
// The hole this closes: PRODUCT's typed semantic manifest includes required
// permissions that generate OpenAPI. Roster actions already emit
// `permissions: ["role_manage"]` (ontology REST Feature::RoleManage). Published
// Head GET/list already authorize `Feature::EmployeeDirectoryRead` in
// `backend/app/src/hr.rs`, but the composed document only declares
// `security: [bearerAuth]` — no Feature string. Bearer is not the directory
// read grant.
//
// Chesterton: do not map Feature::ALL onto the ~587-path surface. Emit only
// the Feature those Head GET/list operations already enforce
// (`employee_directory_read`). Do not invent payroll_run_read on a PayRun Head
// GET (PayRun REST stays PayrollRunSummary). Do not swap in role_manage (that
// is the execute gate). HTTP ETag stays HOLD. as_of stays Employment-only.
//
// Totality: js-yaml load + own-property walk of every path item / GET and the
// six Head schemas. A walker that visits nothing reports nothing, so GET_FLOOR
// / HEAD_FLOOR / OP_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  FENCED_HEADS,
  FENCED_PAY_RUN,
  GET_FLOOR,
  REQUIRED_INSTANCE_GET_HEADS,
} from "./check-openapi-head-gets.mjs";
import { COLLECTION_PATHS } from "./check-openapi-head-collections.mjs";
import { REVERSE_OPERATION_ID, REVERSE_PATH } from "./check-openapi-orgunit-job-positions.mjs";
import { DTO_RS_REL, HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { FENCED_HEADS, FENCED_PAY_RUN, GET_FLOOR };

export const HEAD_FLOOR = REQUIRED_INSTANCE_GET_HEADS.length;

export const PERMISSION_EMPLOYEE_DIRECTORY_READ = "employee_directory_read";

export const HR_RS_REL = "backend/app/src/hr.rs";
export const HR_AUTHORIZE_DIRECTORY_READ =
  "authorize_hr_org_wide(&principal, Feature::EmployeeDirectoryRead)";

/** Published Head GET/list whose runtime already gates EmployeeDirectoryRead. */
export const HEAD_GET_PERMISSION_OPS = Object.freeze([
  Object.freeze({
    path: "/api/v1/companies",
    operationId: "listCompanies",
    head: "Company",
  }),
  Object.freeze({
    path: "/api/v1/companies/{id}",
    operationId: "getCompany",
    head: "Company",
  }),
  Object.freeze({
    path: "/api/v1/org-units",
    operationId: "listOrgUnits",
    head: "OrgUnit",
  }),
  Object.freeze({
    path: "/api/v1/org-units/{id}",
    operationId: "getOrgUnit",
    head: "OrgUnit",
  }),
  Object.freeze({
    path: REVERSE_PATH,
    operationId: REVERSE_OPERATION_ID,
    head: "JobPosition",
  }),
  Object.freeze({
    path: COLLECTION_PATHS.JobPosition,
    operationId: "listJobPositions",
    head: "JobPosition",
  }),
  Object.freeze({
    path: "/api/v1/job-positions/{id}",
    operationId: "getJobPosition",
    head: "JobPosition",
  }),
  Object.freeze({
    path: COLLECTION_PATHS.Person,
    operationId: "listPersons",
    head: "Person",
  }),
  Object.freeze({
    path: "/api/v1/persons/{id}",
    operationId: "getPerson",
    head: "Person",
  }),
  Object.freeze({
    path: "/api/v1/employments",
    operationId: "listEmployments",
    head: "Employment",
  }),
  Object.freeze({
    path: "/api/v1/employments/{id}",
    operationId: "getEmployment",
    head: "Employment",
  }),
]);

export const OP_FLOOR = HEAD_GET_PERMISSION_OPS.length;

const HTTP_METHODS = new Set([
  "get",
  "put",
  "post",
  "delete",
  "options",
  "head",
  "patch",
  "trace",
]);

const REQUIRED_OP_KEYS = new Set(
  HEAD_GET_PERMISSION_OPS.map((spec) => `${spec.path}\0${spec.operationId}`),
);

function push(findings, location, message) {
  findings.push({ location, message });
}

function permissionsList(value) {
  if (!Array.isArray(value)) return null;
  return value.filter((item) => typeof item === "string");
}

function findGet(paths, path) {
  const item = own(paths, path);
  return own(item, "get");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   heads: number,
 *   ops: number,
 *   permissionedOps: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHeadGetPermissions({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let gets = 0;
  let heads = 0;
  let ops = 0;
  let permissionedOps = 0;

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
    return { gets: 0, heads: 0, ops: 0, permissionedOps: 0, findings };
  }
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { gets: 0, heads: 0, ops: 0, permissionedOps: 0, findings };
  }

  for (const path of Object.keys(paths)) {
    if (!hasOwnKey(paths, path)) continue;
    const item = own(paths, path);
    if (!isPlainObject(item)) continue;
    for (const method of Object.keys(item)) {
      if (!hasOwnKey(item, method)) continue;
      if (!HTTP_METHODS.has(method)) continue;
      if (method !== "get") continue;
      const operation = own(item, method);
      if (!isPlainObject(operation)) continue;
      gets += 1;
      const location = `#/paths/${path}/get`;
      const listed = permissionsList(own(operation, "permissions"));
      if (listed) {
        permissionedOps += 1;
        if (!REQUIRED_OP_KEYS.has(`${path}\0${own(operation, "operationId")}`)) {
          push(
            findings,
            location,
            "operation-level permissions are admitted only on published Head GET/list; "
              + "do not map Feature::ALL onto the path surface",
          );
        }
      }
    }
  }

  for (const spec of HEAD_GET_PERMISSION_OPS) {
    const location = `#/paths/${spec.path}/get`;
    const operation = findGet(paths, spec.path);
    if (!isPlainObject(operation)) {
      push(
        findings,
        location,
        `${spec.head} ${spec.operationId} is unpublished; cannot emit the Feature the runtime already enforces`,
      );
      continue;
    }
    ops += 1;
    if (own(operation, "operationId") !== spec.operationId) {
      push(
        findings,
        `${location}/operationId`,
        `must be ${spec.operationId}`,
      );
    }
    const listed = permissionsList(own(operation, "permissions"));
    if (!listed) {
      push(
        findings,
        `${location}/permissions`,
        `${spec.operationId} already gates Feature::EmployeeDirectoryRead; `
          + `emit permissions: ["${PERMISSION_EMPLOYEE_DIRECTORY_READ}"] `
          + "(bearerAuth is not that grant)",
      );
      continue;
    }
    if (!listed.includes(PERMISSION_EMPLOYEE_DIRECTORY_READ)) {
      push(
        findings,
        `${location}/permissions`,
        `must include ${PERMISSION_EMPLOYEE_DIRECTORY_READ} (Feature::EmployeeDirectoryRead as_str)`,
      );
    }
    for (const item of listed) {
      if (item !== PERMISSION_EMPLOYEE_DIRECTORY_READ) {
        push(
          findings,
          `${location}/permissions`,
          `invented permission ${item}; Head GET/list only emit the Feature those handlers already enforce`,
        );
      }
    }
  }

  if (permissionedOps > OP_FLOOR) {
    push(
      findings,
      "#/paths",
      `saw ${permissionedOps} GET operations with permissions — above the Head GET/list cap ${OP_FLOOR}; `
        + "do not map Feature::ALL onto the path surface",
    );
  }

  for (const name of HEAD_SCHEMA_NAMES) {
    const location = `#/components/schemas/${name}`;
    const schema = own(schemas, name);
    if (!isPlainObject(schema)) {
      push(findings, location, "canonical Head schema is absent");
      continue;
    }
    heads += 1;
    const listed = permissionsList(own(schema, "permissions"));
    const required = REQUIRED_INSTANCE_GET_HEADS.includes(name);
    if (name === FENCED_PAY_RUN) {
      if (hasOwnKey(schema, "permissions")) {
        push(
          findings,
          `${location}/permissions`,
          "PayRun has no Head GET; do not emit a GET permission (REST stays PayrollRunSummary)",
        );
      }
      continue;
    }
    if (!required) continue;
    if (!listed) {
      push(
        findings,
        `${location}/permissions`,
        `${name} Head GET/list already gate Feature::EmployeeDirectoryRead; `
          + `schema-level permissions: ["${PERMISSION_EMPLOYEE_DIRECTORY_READ}"] is the generated contract`,
      );
      continue;
    }
    if (!listed.includes(PERMISSION_EMPLOYEE_DIRECTORY_READ)) {
      push(
        findings,
        `${location}/permissions`,
        `must include ${PERMISSION_EMPLOYEE_DIRECTORY_READ} (Feature::EmployeeDirectoryRead as_str)`,
      );
    }
    for (const item of listed) {
      if (item !== PERMISSION_EMPLOYEE_DIRECTORY_READ) {
        push(
          findings,
          `${location}/permissions`,
          `invented permission ${item}; do not copy action role_manage onto Head GET`,
        );
      }
    }
  }

  const dtoPath = join(repoRoot, DTO_RS_REL);
  if (existsSync(dtoPath)) {
    const dtoText = readFileSync(dtoPath, "utf8");
    if (!dtoText.includes(`"${PERMISSION_EMPLOYEE_DIRECTORY_READ}"`)) {
      push(
        findings,
        DTO_RS_REL,
        `DTO roster must generate Head GET permissions from Feature::EmployeeDirectoryRead `
          + `("${PERMISSION_EMPLOYEE_DIRECTORY_READ}"); hand-copied path YAML is not generation`,
      );
    }
  }

  const hrPath = join(repoRoot, HR_RS_REL);
  if (existsSync(hrPath)) {
    const hrText = readFileSync(hrPath, "utf8");
    if (!hrText.includes(HR_AUTHORIZE_DIRECTORY_READ)) {
      push(
        findings,
        HR_RS_REL,
        "Head GET/list must keep authorize_hr_org_wide Feature::EmployeeDirectoryRead; "
          + "do not invent a permission the runtime does not enforce",
      );
    }
  }

  return { gets, heads, ops, permissionedOps, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHeadGetPermissions({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, heads, ops, permissionedOps, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowGetFloor = gets < GET_FLOOR;
  const belowHeadFloor = heads < HEAD_FLOOR;
  const belowOpFloor = ops < OP_FLOOR;
  if (belowGetFloor) {
    console.error(
      `saw ${gets} GET operations — below the floor ${GET_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (belowHeadFloor) {
    console.error(
      `saw ${heads} Head schemas — below the floor ${HEAD_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (belowOpFloor) {
    console.error(
      `saw ${ops} Head GET/list operations — below the floor ${OP_FLOOR}, the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowGetFloor || belowHeadFloor || belowOpFloor) {
    console.error(
      `openapi Head GET/list permissions gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), ${heads} Head(s), ${ops} Head GET/list op(s), `
        + `${permissionedOps} permissioned GET(s)`,
    );
    process.exit(1);
  }
  console.log(
    `openapi Head GET/list permissions gate passed `
      + `(${OP_FLOOR} ops ${PERMISSION_EMPLOYEE_DIRECTORY_READ}; `
      + `${HEAD_FLOOR} Heads; PayRun fenced; ${gets} GET operations, 0 findings)`,
  );
}
