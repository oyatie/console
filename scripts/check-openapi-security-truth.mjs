// OpenAPI security-truth gate over the published document (P0.5).
//
// The holes this closes:
//   * No document-level `security`, so OpenAPI treats every operation that omits
//     `security` as public. That is not deny-by-omission.
//   * Deliberately public operations never declare `security: []`, so public vs
//     inherited-bearer cannot be told apart.
//   * POST /api/v1/auth/admin/credential-reset is documented as a live admin
//     recovery API while PRODUCT marks credential-reset HOLD.
//
// Totality: js-yaml load + own-property walk of every path item / HTTP method,
// same primitive as scripts/check-openapi-refs.mjs. Authoring style is invisible.
// A walker that visits nothing reports nothing, so OP_FLOOR is the examined-zero
// lock.
//
// Chesterton: `deprecated: true` plus a description that the operation is HOLD /
// not generally available is the existing held/not-implemented vocabulary
// (see GovernanceDecideApprovalRequest.requested_by and payable: Always false
// until the release gate). Do not invent an x-console-hold extension.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const OP_FLOOR = 400;

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

export const CREDENTIAL_RESET = Object.freeze({
  method: "post",
  path: "/api/v1/auth/admin/credential-reset",
});

// Closed set of operations that may, and must, declare `security: []`.
// Anything else with an empty security array is an accidental public surface.
export const PUBLIC_OPERATIONS = Object.freeze([
  ["get", "/.well-known/apple-app-site-association"],
  ["get", "/.well-known/assetlinks.json"],
  ["post", "/api/v1/auth/device-login/approve"],
  ["post", "/api/v1/auth/device-login/poll"],
  ["post", "/api/v1/auth/device-login/start"],
  ["post", "/api/v1/auth/logout"],
  ["post", "/api/v1/auth/otp/redeem"],
  ["post", "/api/v1/auth/passkey/login/finish"],
  ["post", "/api/v1/auth/passkey/login/start"],
  ["post", "/api/v1/auth/signup"],
  ["post", "/api/v1/auth/token/refresh"],
  ["post", "/api/v1/storefront/inquiries"],
  ["get", "/api/v1/storefront/listings"],
  ["get", "/api/v1/storefront/listings/{id}"],
  ["get", "/api/v1/storefront/listings/{id}/media/{media_id}"],
  ["post", "/api/v1/support/intake"],
  ["get", "/healthz"],
  ["get", "/readyz"],
]);

const PUBLIC_KEYS = new Set(PUBLIC_OPERATIONS.map(([method, path]) => `${method} ${path}`));

function opKey(method, path) {
  return `${method} ${path}`;
}

function requirementHasBearer(requirement) {
  return isPlainObject(requirement) && hasOwnKey(requirement, "bearerAuth");
}

function documentRequiresBearer(document) {
  const security = own(document, "security");
  if (!Array.isArray(security) || security.length === 0) return false;
  return security.some(requirementHasBearer);
}

function isExplicitlyPublic(security) {
  return Array.isArray(security) && security.length === 0;
}

function describesHoldNotGa(text) {
  if (typeof text !== "string") return false;
  return /\bHOLD\b/.test(text) && /not generally available/i.test(text);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   operations: number,
 *   publicDeclared: number,
 *   omittedSecurity: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiSecurityTruth({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const paths = own(document, "paths");
  let operations = 0;
  let publicDeclared = 0;
  let omittedSecurity = 0;
  const seenPublic = new Set();

  if (!documentRequiresBearer(document)) {
    findings.push({
      location: "#/security",
      message: "document-level `security` must require bearerAuth so omitted "
        + "operation security inherits deny-by-omission; `security: []` is the "
        + "only explicit public override",
    });
  }

  if (!isPlainObject(paths)) {
    return { operations, publicDeclared, omittedSecurity, findings };
  }

  for (const [path, item] of Object.entries(paths)) {
    if (!isPlainObject(item)) continue;
    for (const [method, operation] of Object.entries(item)) {
      if (!HTTP_METHODS.has(method) || !isPlainObject(operation)) continue;
      operations += 1;
      const location = `#/paths/${path}/${method}`;
      const key = opKey(method, path);
      const hasSecurity = hasOwnKey(operation, "security");
      const security = own(operation, "security");

      if (!hasSecurity) {
        omittedSecurity += 1;
        if (PUBLIC_KEYS.has(key)) {
          findings.push({
            location,
            message: "deliberately public operation must declare `security: []`; "
              + "omission is not an explicit public override once document-level "
              + "bearerAuth is set",
          });
        } else if (!documentRequiresBearer(document)) {
          findings.push({
            location,
            message: "operation omits `security` while the document has no "
              + "inheritable bearer requirement, so OpenAPI treats it as public",
          });
        }
      } else if (isExplicitlyPublic(security)) {
        publicDeclared += 1;
        if (!PUBLIC_KEYS.has(key)) {
          findings.push({
            location,
            message: "`security: []` is allowed only on the closed public-operation "
              + "set; this path is not a deliberately public surface",
          });
        } else {
          seenPublic.add(key);
        }
      }

      if (method === CREDENTIAL_RESET.method && path === CREDENTIAL_RESET.path) {
        if (isExplicitlyPublic(security)) {
          findings.push({
            location,
            message: "credential-reset is HOLD and admin-gated, not a public "
              + "operation; do not declare `security: []`",
          });
        }
        const summary = own(operation, "summary");
        const description = own(operation, "description");
        const deprecated = own(operation, "deprecated") === true;
        if (!deprecated || !describesHoldNotGa(`${summary ?? ""}\n${description ?? ""}`)) {
          findings.push({
            location,
            message: "credential-reset must not look generally available while "
              + "PRODUCT marks it HOLD: use the existing held/not-implemented "
              + "vocabulary (`deprecated: true` plus description HOLD — not "
              + "generally available)",
          });
        }
      }
    }
  }

  for (const [method, path] of PUBLIC_OPERATIONS) {
    const key = opKey(method, path);
    if (seenPublic.has(key)) continue;
    const item = own(paths, path);
    const operation = own(item, method);
    if (!isPlainObject(operation)) {
      findings.push({
        location: `#/paths/${path}/${method}`,
        message: "closed public-operation set names a missing path; do not shrink "
          + "the published public surface by deleting it from the document",
      });
    }
  }

  return { operations, publicDeclared, omittedSecurity, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateOpenapiSecurityTruth({ repoRoot });
  } catch (error) {
    console.error(`backend/openapi/openapi.yaml cannot be parsed: ${error.message}`);
    process.exit(1);
  }
  const { operations, publicDeclared, omittedSecurity, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = operations < OP_FLOOR;
  if (belowFloor) {
    console.error(`saw ${operations} operations (floor ${OP_FLOOR}) — below the floor, `
      + "the walker examined less of the document than it was built to examine");
  }
  if (findings.length > 0 || belowFloor) {
    console.error(`openapi security-truth gate FAILED: ${findings.length} finding(s), `
      + `${operations} operations, ${publicDeclared} explicit public, `
      + `${omittedSecurity} omitted security`);
    process.exit(1);
  }
  console.log(`openapi security-truth gate passed (${operations} operations, `
    + `${publicDeclared} explicit public, ${omittedSecurity} omitted security, 0 findings)`);
}
