// Generated-docs Palantir-class field gate.
//
// The hole this closes: compose-owned OpenAPI already carries Palantir-class
// fields (schema-level `links[].operationId`, `actions[].permissions`, and
// Head GET/list operation `permissions`). The TypeScript SDK copies those
// schema-level fields onto Head definitions. Generated docs still drop them:
// links render from/to/field/cardinality without the bound GET, actions render
// four_eyes without `role_manage`, and the operations table is
// method/path/operationId/summary with no Feature string. Annotation, not the
// composed contract.
//
// Chesterton: do not map Feature::ALL onto ~587 paths. Copy only permissions
// the composed document already declares. Do not invent PayRun Head GET
// permissions, HTTP ETag, linked as_of, or a JobPosition SSR directory.
// Validators stay the generated Rust codecs; this slice is docs drift.
//
// Totality: js-yaml load + own-property walk of every Head schema link/action
// and every path operation. A walker that visits nothing reports nothing, so
// LINK_OPERATION_FLOOR / ACTION_PERMISSION_FLOOR / OPERATION_PERMISSION_FLOOR
// lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { OP_FLOOR } from "./check-openapi-head-get-permissions.mjs";
import {
  ACTION_FLOOR,
  PERMISSION_ROLE_MANAGE,
} from "./check-openapi-semantic-contract.mjs";
import { HEAD_SCHEMA_NAMES } from "./check-openapi-semantic-generate.mjs";
import { DOCS_REL, HTTP_METHODS, OPENAPI_REL } from "./generate-openapi-docs.mjs";
import { isPlainObject, own } from "./own-property.mjs";

export { ACTION_FLOOR, DOCS_REL, OPENAPI_REL, OP_FLOOR, PERMISSION_ROLE_MANAGE };

/** 5 Head FKs + OrgUnit→JobPosition reverse, each already bound to a GET. */
export const LINK_OPERATION_FLOOR = 6;
export const ACTION_PERMISSION_FLOOR = ACTION_FLOOR;
export const OPERATION_PERMISSION_FLOOR = OP_FLOOR;

function push(findings, location, message) {
  findings.push({ location, message });
}

function permissionsList(value) {
  if (!Array.isArray(value)) return null;
  const items = [];
  for (const item of value) {
    if (typeof item !== "string" || item.length === 0) return null;
    items.push(item);
  }
  return items;
}

function attr(tag, name) {
  if (typeof tag !== "string") return null;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = tag.match(new RegExp(`\\s${escaped}="([^"]*)"`));
  return match ? match[1] : null;
}

function schemaSection(html, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = html.match(
    new RegExp(`<section[^>]*id="schema-${escaped}"[\\s\\S]*?</section>`),
  );
  return match ? match[0] : null;
}

function startTag(html, pattern) {
  if (typeof html !== "string") return null;
  const match = html.match(pattern);
  return match ? match[0] : null;
}

function operationKey(method, path) {
  return `${method} ${path}`;
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   links: number,
 *   actions: number,
 *   operations: number,
 *   permissionedOps: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateGeneratedDocsContract({ repoRoot }) {
  const findings = [];
  const openapiPath = join(repoRoot, OPENAPI_REL);
  const docsPath = join(repoRoot, DOCS_REL);
  if (!existsSync(openapiPath)) {
    push(findings, OPENAPI_REL, "composed OpenAPI document is missing");
    return { links: 0, actions: 0, operations: 0, permissionedOps: 0, findings };
  }
  if (!existsSync(docsPath)) {
    push(findings, DOCS_REL, "generated docs artifact is missing");
    return { links: 0, actions: 0, operations: 0, permissionedOps: 0, findings };
  }

  let document;
  try {
    document = yaml.load(readFileSync(openapiPath, "utf8"));
  } catch (error) {
    push(findings, OPENAPI_REL, `cannot parse: ${error.message}`);
    return { links: 0, actions: 0, operations: 0, permissionedOps: 0, findings };
  }
  const html = readFileSync(docsPath, "utf8");
  const schemas = own(own(document, "components"), "schemas");
  if (!isPlainObject(schemas)) {
    push(findings, "#/components/schemas", "published document has no components.schemas mapping");
    return { links: 0, actions: 0, operations: 0, permissionedOps: 0, findings };
  }

  let links = 0;
  let actions = 0;

  for (const name of HEAD_SCHEMA_NAMES) {
    const schema = own(schemas, name);
    const loc = `#/components/schemas/${name}`;
    const section = schemaSection(html, name);
    if (!isPlainObject(schema)) {
      push(findings, loc, "composed document is missing a Head schema the docs must describe");
      continue;
    }
    if (section === null) {
      push(findings, `${DOCS_REL}:${name}`, "generated docs omit this Head");
      continue;
    }

    const declaredLinks = own(schema, "links");
    if (Array.isArray(declaredLinks)) {
      for (const link of declaredLinks) {
        if (!isPlainObject(link)) continue;
        const key = own(link, "key");
        const operationId = own(link, "operationId");
        if (typeof key !== "string" || key.length === 0) continue;
        if (typeof operationId !== "string" || operationId.length === 0) continue;
        links += 1;
        const tag = startTag(
          section,
          new RegExp(`<li\\s[^>]*data-link="${key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"[^>]*>`),
        );
        if (tag === null) {
          push(
            findings,
            `${DOCS_REL}:${name}/links/${key}`,
            `docs omit composed link ${key} (operationId ${operationId})`,
          );
          continue;
        }
        if (attr(tag, "data-operation-id") !== operationId) {
          push(
            findings,
            `${DOCS_REL}:${name}/links/${key}/operationId`,
            `docs must generate composed link operationId ${operationId}`,
          );
        }
      }
    }

    const declaredActions = own(schema, "actions");
    if (Array.isArray(declaredActions)) {
      for (const action of declaredActions) {
        if (!isPlainObject(action)) continue;
        const key = own(action, "action_key");
        const listed = permissionsList(own(action, "permissions"));
        if (typeof key !== "string" || key.length === 0) continue;
        if (listed === null || listed.length === 0) continue;
        actions += 1;
        const tag = startTag(
          section,
          new RegExp(`<li\\s[^>]*data-action="${key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"[^>]*>`),
        );
        if (tag === null) {
          push(
            findings,
            `${DOCS_REL}:${name}/actions/${key}`,
            `docs omit composed action ${key}`,
          );
          continue;
        }
        const rendered = (attr(tag, "data-permissions") ?? "").split(" ").filter(Boolean);
        if (rendered.join(" ") !== listed.join(" ")) {
          push(
            findings,
            `${DOCS_REL}:${name}/actions/${key}/permissions`,
            `docs must generate composed action permissions [${listed.join(", ")}]`,
          );
        }
      }
    }
  }

  const paths = own(document, "paths");
  let operations = 0;
  let permissionedOps = 0;
  if (isPlainObject(paths)) {
    for (const path of Object.keys(paths)) {
      const item = own(paths, path);
      if (!isPlainObject(item)) continue;
      for (const method of HTTP_METHODS) {
        const op = own(item, method);
        if (!isPlainObject(op)) continue;
        operations += 1;
        const key = operationKey(method.toUpperCase(), path);
        const listed = permissionsList(own(op, "permissions")) ?? [];
        const tag = startTag(
          html,
          new RegExp(
            `<tr\\s[^>]*data-operation="${key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"[^>]*>`,
          ),
        );
        if (tag === null) {
          push(findings, `${DOCS_REL}:${key}`, "generated docs omit this composed OpenAPI operation");
          continue;
        }
        const rendered = (attr(tag, "data-permissions") ?? "").split(" ").filter(Boolean);
        if (listed.length > 0) {
          permissionedOps += 1;
          if (rendered.join(" ") !== listed.join(" ")) {
            push(
              findings,
              `${DOCS_REL}:${key}/permissions`,
              `docs must generate composed operation permissions [${listed.join(", ")}]`,
            );
          }
        } else if (rendered.length > 0) {
          push(
            findings,
            `${DOCS_REL}:${key}/permissions`,
            "operation-level permissions are admitted only when composed OpenAPI declares them; "
              + "do not map Feature::ALL onto the docs operations table",
          );
        }
      }
    }
  }

  if (links < LINK_OPERATION_FLOOR) {
    push(
      findings,
      "#/components/schemas",
      `saw ${links} Head links with operationId — below the floor ${LINK_OPERATION_FLOOR}`,
    );
  }
  if (actions < ACTION_PERMISSION_FLOOR) {
    push(
      findings,
      "#/components/schemas",
      `saw ${actions} Head actions with permissions — below the floor ${ACTION_PERMISSION_FLOOR}`,
    );
  }
  if (permissionedOps < OPERATION_PERMISSION_FLOOR) {
    push(
      findings,
      "#/paths",
      `saw ${permissionedOps} operations with permissions — below the floor ${OPERATION_PERMISSION_FLOOR}`,
    );
  }

  return { links, actions, operations, permissionedOps, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateGeneratedDocsContract({ repoRoot });
  } catch (error) {
    console.error(`generated-docs Palantir-class field gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { links, actions, operations, permissionedOps, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor =
    links < LINK_OPERATION_FLOOR
    || actions < ACTION_PERMISSION_FLOOR
    || permissionedOps < OPERATION_PERMISSION_FLOOR;
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi generated-docs Palantir-class field gate FAILED: ${findings.length} finding(s), `
        + `${links} link operationIds, ${actions} action permissions, `
        + `${permissionedOps} operation permissions, ${operations} operations`,
    );
    process.exit(1);
  }
  console.log(
    `openapi generated-docs Palantir-class field gate passed `
      + `(${links} link operationIds, ${actions} action permissions, `
      + `${permissionedOps} operation permissions, ${operations} operations, 0 findings)`,
  );
}
