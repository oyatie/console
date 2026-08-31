// Head FK link → published GET operation gate.
//
// The hole this closes: PRODUCT requires typed objects/links that generate
// OpenAPI. Head schemas already declare schema-level `links` for runtime FKs
// (Employment.person_id / org_unit_id / job_position_id, OrgUnit.parent_id,
// JobPosition.org_unit_id), and Company / OrgUnit / Person / Employment now
// have instance GET. Those links still name `to` + `field` only — annotation,
// not a traversable operation. This gate requires each link whose `to` Head
// already has an instance GET to carry that GET's operationId.
//
// Chesterton: Company.org_id is the RLS cell, not a Palantir/Head FK — do not
// invent Employment→Company. L5-JOB still refuses `/api/v1/job-positions`;
// Employment.job_position_id stays id-only. PayRun stays PayrollRunSummary.
// as_of stays Employment instance GET; linked OrgUnit/Person/Company have no
// valid-time store. Runtime JSON Heads need not embed hrefs.
//
// Totality: js-yaml load + own-property walk of every GET and every canonical
// schema-level link. A walker that visits nothing reports nothing, so
// GET_FLOOR / LINK_FLOOR / TRAVERSABLE_LINK_FLOOR lock examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import {
  FENCED_HEADS,
  FENCED_JOB_POSITION,
  FENCED_PAY_RUN,
  GET_FLOOR,
  REQUIRED_INSTANCE_GET_HEADS,
} from "./check-openapi-head-gets.mjs";
import {
  CANONICAL_LINKS,
  LINK_FLOOR,
} from "./check-openapi-semantic-contract.mjs";
import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export { FENCED_HEADS, FENCED_JOB_POSITION, FENCED_PAY_RUN, GET_FLOOR, LINK_FLOOR };

/** Instance GET paths already published. JobPosition is not among them. */
export const INSTANCE_GET_PATHS = Object.freeze({
  Company: "/api/v1/companies/{id}",
  OrgUnit: "/api/v1/org-units/{id}",
  Person: "/api/v1/persons/{id}",
  Employment: "/api/v1/employments/{id}",
});

/** Head FKs whose `to` already has instance GET — must bind that operationId. */
export const TRAVERSABLE_LINKS = Object.freeze(
  CANONICAL_LINKS.filter((link) => REQUIRED_INSTANCE_GET_HEADS.includes(link.to)),
);

/** Head FKs whose `to` is still fenced — id-only, no operation. */
export const FENCED_LINKS = Object.freeze(
  CANONICAL_LINKS.filter((link) => FENCED_HEADS.includes(link.to)),
);

export const TRAVERSABLE_LINK_FLOOR = TRAVERSABLE_LINKS.length;
export const FENCED_LINK_FLOOR = FENCED_LINKS.length;

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

function push(findings, location, message) {
  findings.push({ location, message });
}

function schemaRefName(schema) {
  if (!isPlainObject(schema)) return null;
  const ref = own(schema, "$ref");
  if (typeof ref !== "string") return null;
  const prefix = "#/components/schemas/";
  if (!ref.startsWith(prefix)) return null;
  return ref.slice(prefix.length);
}

function json200Schema(operation) {
  const responses = own(operation, "responses");
  const ok = own(responses, "200") ?? own(responses, 200);
  const content = own(ok, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function parameterList(operation) {
  const parameters = own(operation, "parameters");
  return Array.isArray(parameters) ? parameters : [];
}

function hasPathId(operation) {
  return parameterList(operation).some(
    (param) =>
      isPlainObject(param) && own(param, "name") === "id" && own(param, "in") === "path",
  );
}

function directHeadName(schema) {
  const name = schemaRefName(schema);
  if (typeof name !== "string") return null;
  return name;
}

function findLink(list, key) {
  if (!Array.isArray(list)) return null;
  return list.find((item) => isPlainObject(item) && own(item, "key") === key) ?? null;
}

function linkMentionsAsOf(link) {
  if (hasOwnKey(link, "as_of")) return true;
  const parameters = own(link, "parameters");
  if (isPlainObject(parameters) && hasOwnKey(parameters, "as_of")) return true;
  const href = own(link, "href");
  if (typeof href === "string" && href.includes("as_of")) return true;
  return false;
}

function operationIdOf(link) {
  const value = own(link, "operationId");
  if (typeof value !== "string" || value.length === 0) return null;
  return value;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   gets: number,
 *   links: number,
 *   traversable: number,
 *   fenced: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiHeadLinkOps({ repoRoot }) {
  const findings = [];
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const paths = own(document, "paths");
  const schemas = own(own(document, "components"), "schemas");
  let gets = 0;
  const instanceGetByHead = Object.create(null);
  const operationById = Object.create(null);

  if (!isPlainObject(paths)) {
    push(findings, "#/paths", "published document has no paths mapping");
  } else {
    for (const path of Object.keys(paths)) {
      if (!hasOwnKey(paths, path)) continue;
      const item = own(paths, path);
      if (!isPlainObject(item)) continue;
      if (path === "/api/v1/job-positions" || path.startsWith("/api/v1/job-positions/")) {
        push(
          findings,
          `#/paths/${path}`,
          "JobPosition Head must not gain a GET; L5-JOB still refuses inventing /api/v1/job-positions (identity stays action-receipt readback)",
        );
      }
      for (const method of Object.keys(item)) {
        if (!hasOwnKey(item, method)) continue;
        if (!HTTP_METHODS.has(method)) continue;
        if (method !== "get") continue;
        const operation = own(item, method);
        if (!isPlainObject(operation)) continue;
        gets += 1;
        const location = `#/paths/${path}/get`;
        const operationId = own(operation, "operationId");
        if (typeof operationId === "string" && operationId.length > 0) {
          operationById[operationId] = { path, operation };
        }
        const schema = json200Schema(operation);
        const direct = directHeadName(schema);
        if (direct && FENCED_HEADS.includes(direct)) {
          push(
            findings,
            location,
            direct === FENCED_JOB_POSITION
              ? "JobPosition Head must not gain a GET; L5-JOB still refuses inventing /api/v1/job-positions (identity stays action-receipt readback)"
              : "PayRun Head must not become a GET 200 schema; REST stays PayrollRunSummary and version stays absent",
          );
        }
        if (
          direct
          && REQUIRED_INSTANCE_GET_HEADS.includes(direct)
          && hasPathId(operation)
          && typeof operationId === "string"
          && operationId.length > 0
        ) {
          instanceGetByHead[direct] = { path, operationId };
        }
      }
    }
  }

  let links = 0;
  let traversable = 0;
  let fenced = 0;

  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { gets, links, traversable, fenced, findings };
  }

  for (const spec of CANONICAL_LINKS) {
    const schemaLoc = `#/components/schemas/${spec.from}`;
    const schema = own(schemas, spec.from);
    const declared = isPlainObject(schema) ? own(schema, "links") : undefined;
    const found = findLink(declared, spec.key);
    const linkLoc = `${schemaLoc}/links/${spec.key}`;
    if (!found) {
      push(findings, linkLoc, `runtime Head FK ${spec.field} is not declared as a link type`);
      continue;
    }
    links += 1;
    if (linkMentionsAsOf(found)) {
      push(
        findings,
        `${linkLoc}/as_of`,
        `${spec.to} has no valid-time store on this link; do not document as_of the runtime cannot honor (created_at is not valid_from). Employment as_of stays on GET /api/v1/employments/{id}`,
      );
    }

    const operationId = operationIdOf(found);
    const published = instanceGetByHead[spec.to];
    const fencedTarget = FENCED_HEADS.includes(spec.to);

    if (fencedTarget) {
      fenced += 1;
      if (operationId) {
        push(
          findings,
          `${linkLoc}/operationId`,
          spec.to === FENCED_JOB_POSITION
            ? "JobPosition GET is fenced (L5-JOB); employment_job_position stays id-only / no operation — do not invent getJobPosition"
            : "PayRun is not a versioned Head GET; do not bind a PayRun operationId",
        );
      }
      continue;
    }

    if (!REQUIRED_INSTANCE_GET_HEADS.includes(spec.to)) {
      if (operationId) {
        push(
          findings,
          `${linkLoc}/operationId`,
          `${spec.to} has no published instance GET; leave this Head FK id-only`,
        );
      }
      continue;
    }

    if (!published) {
      push(
        findings,
        `#/components/schemas/${spec.to}`,
        `${spec.to} Head is not a 200 schema of any instance GET; cannot bind a traversable link`,
      );
      continue;
    }

    const expectedPath = own(INSTANCE_GET_PATHS, spec.to);
    if (published.path !== expectedPath) {
      push(
        findings,
        `#/paths/${published.path}/get`,
        `${spec.to} instance GET path must stay ${expectedPath} (got ${published.path})`,
      );
    }

    if (!operationId) {
      push(
        findings,
        `${linkLoc}/operationId`,
        `${spec.from}→${spec.to} via ${spec.field} must reference published GET ${published.operationId} (${published.path}); schema-only to/field is not a traversable operation`,
      );
      continue;
    }

    if (operationId !== published.operationId) {
      push(
        findings,
        `${linkLoc}/operationId`,
        `must be ${published.operationId} (existing GET ${published.path}), got ${JSON.stringify(operationId)}`,
      );
      continue;
    }

    const bound = own(operationById, operationId);
    if (!isPlainObject(bound)) {
      push(
        findings,
        `${linkLoc}/operationId`,
        `operationId ${operationId} is not a published operation; do not invent a GET to make the link look complete`,
      );
      continue;
    }
    traversable += 1;
  }

  return { gets, links, traversable, fenced, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiHeadLinkOps({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { gets, links, traversable, fenced, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    gets < GET_FLOOR
    || links < LINK_FLOOR
    || traversable < TRAVERSABLE_LINK_FLOOR
    || fenced < FENCED_LINK_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${gets} GET operations, ${links}/${LINK_FLOOR} links, `
        + `${traversable}/${TRAVERSABLE_LINK_FLOOR} traversable, `
        + `${fenced}/${FENCED_LINK_FLOOR} fenced — below the floor; the walker examined nothing useful`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi Head link-ops gate FAILED: ${findings.length} finding(s), `
        + `${gets} GET(s), ${links} link(s), ${traversable} traversable, ${fenced} fenced`,
    );
    process.exit(1);
  }
  const bound = TRAVERSABLE_LINKS.map((link) => link.key).join(", ");
  const held = FENCED_LINKS.map((link) => link.key).join(", ");
  console.log(
    `openapi Head link-ops gate passed `
      + `(traversable ${bound}; fenced ${held}; ${gets} GET operations, 0 findings)`,
  );
}
