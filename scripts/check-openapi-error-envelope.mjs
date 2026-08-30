// HTTP error-envelope parity gate: OpenAPI + the middleware/error paths that
// must match it.
//
// The holes this closes (P0.2 siblings after the null-union slice):
//   * Shared responses promise JSON ErrorBody, but request-context
//     `error_response` still returns axum's plaintext `(status, message)`.
//   * Runtime can emit 408 (TimeoutLayer) and 413 (2 MiB DefaultBodyLimit)
//     while the published document names neither status.
//   * Documented 429 responses omit Retry-After even though the status is
//     produced.
//
// Totality: js-yaml load + own-property walk of every mapping, same primitive as
// scripts/check-openapi-refs.mjs. Authoring style is invisible. A walker that
// visits nothing reports nothing, so PATH_FLOOR / RESPONSE_FLOOR are the
// examined-zero lock. Runtime checks are exact-file reads of the two writers
// that emit these statuses (request-context middleware, app composition root).

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const PATH_FLOOR = 400;
export const RESPONSE_FLOOR = 10;
export const RETRY_AFTER_SECONDS = "60";

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

function jsonContentSchema(response) {
  if (!isPlainObject(response)) return undefined;
  const content = own(response, "content");
  const json = own(content, "application/json");
  return own(json, "schema");
}

function schemaIsErrorBody(schema) {
  if (!isPlainObject(schema)) return false;
  const ref = own(schema, "$ref");
  return ref === "#/components/schemas/ErrorBody";
}

function responseUsesErrorBody(response) {
  if (!isPlainObject(response)) return false;
  const ref = own(response, "$ref");
  if (typeof ref === "string" && ref.startsWith("#/components/responses/")) {
    return true;
  }
  return schemaIsErrorBody(jsonContentSchema(response));
}

function headerNamedRetryAfter(headers) {
  if (!isPlainObject(headers)) return undefined;
  if (hasOwnKey(headers, "Retry-After")) return own(headers, "Retry-After");
  if (hasOwnKey(headers, "retry-after")) return own(headers, "retry-after");
  return undefined;
}

function resolveResponse(document, response) {
  if (!isPlainObject(response)) return undefined;
  const ref = own(response, "$ref");
  if (typeof ref !== "string") return response;
  const prefix = "#/components/responses/";
  if (!ref.startsWith(prefix)) return undefined;
  const name = ref.slice(prefix.length);
  const responses = own(own(document, "components"), "responses");
  return own(responses, name);
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   paths: number,
 *   responses: number,
 *   documented408: number,
 *   documented413: number,
 *   documented429: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateOpenapiErrorEnvelope({ repoRoot }) {
  const document = yaml.load(
    readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
  );
  const findings = [];
  const responses = own(own(document, "components"), "responses");
  const responseCount = isPlainObject(responses) ? Object.keys(responses).length : 0;

  const requestTimeout = own(responses, "RequestTimeout");
  if (!schemaIsErrorBody(jsonContentSchema(requestTimeout))) {
    findings.push({
      location: "#/components/responses/RequestTimeout",
      message: "runtime TimeoutLayer emits 408; document it as JSON ErrorBody "
        + "(shared response RequestTimeout), matching Unauthorized/TooManyRequests",
    });
  }

  const payloadTooLarge = own(responses, "PayloadTooLarge");
  if (!schemaIsErrorBody(jsonContentSchema(payloadTooLarge))) {
    findings.push({
      location: "#/components/responses/PayloadTooLarge",
      message: "runtime DefaultBodyLimit emits 413; document it as JSON ErrorBody "
        + "(shared response PayloadTooLarge), matching Unauthorized/TooManyRequests",
    });
  }

  const tooMany = own(responses, "TooManyRequests");
  if (!isPlainObject(tooMany) || headerNamedRetryAfter(own(tooMany, "headers")) === undefined) {
    findings.push({
      location: "#/components/responses/TooManyRequests",
      message: "429 responses must declare the Retry-After header when that status is produced",
    });
  }

  let paths = 0;
  let documented408 = 0;
  let documented413 = 0;
  let documented429 = 0;
  const pathItems = own(document, "paths");
  if (isPlainObject(pathItems)) {
    for (const [path, item] of Object.entries(pathItems)) {
      if (!isPlainObject(item)) continue;
      paths += 1;
      for (const [method, operation] of Object.entries(item)) {
        if (!HTTP_METHODS.has(method) || !isPlainObject(operation)) continue;
        const opResponses = own(operation, "responses");
        if (!isPlainObject(opResponses)) continue;
        const at = `#/paths/${path}/${method}/responses`;

        for (const status of ["408", "413", "429"]) {
          if (!hasOwnKey(opResponses, status)) continue;
          const raw = own(opResponses, status);
          const resolved = resolveResponse(document, raw);
          if (status === "408") documented408 += 1;
          if (status === "413") documented413 += 1;
          if (status === "429") documented429 += 1;
          if (!responseUsesErrorBody(raw) && !schemaIsErrorBody(jsonContentSchema(resolved))) {
            findings.push({
              location: `${at}/${status}`,
              message: `HTTP ${status} must use JSON ErrorBody (shared envelope), not a bare description or plaintext`,
            });
          }
          if (status === "429") {
            const headers = own(resolved, "headers");
            if (headerNamedRetryAfter(headers) === undefined) {
              findings.push({
                location: `${at}/429`,
                message: "documented 429 must include Retry-After",
              });
            }
          }
        }
      }
    }
  }

  if (documented408 < 1) {
    findings.push({
      location: "#/paths",
      message: "no operation documents HTTP 408, but TimeoutLayer is reachable at runtime",
    });
  }
  if (documented413 < 1) {
    findings.push({
      location: "#/paths",
      message: "no operation documents HTTP 413, but DefaultBodyLimit is reachable at runtime",
    });
  }

  return {
    paths,
    responses: responseCount,
    documented408,
    documented413,
    documented429,
    findings,
  };
}

function plaintextStatusMessageResponse(source) {
  return /\(\s*status\s*,\s*message\.to_owned\(\)\s*\)\s*\.into_response\(\)/.test(source);
}

function emitsJsonErrorBody(source) {
  return source.includes("Json(ErrorBody")
    && source.includes("error: ErrorPayload");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{ findings: { location: string, message: string }[] }}
 */
export function evaluateRuntimeErrorEnvelope({ repoRoot }) {
  const findings = [];
  const requestContext = readFileSync(
    join(repoRoot, "backend/crates/platform/request-context/src/lib.rs"),
    "utf8",
  );
  if (plaintextStatusMessageResponse(requestContext) || !emitsJsonErrorBody(requestContext)) {
    findings.push({
      location: "backend/crates/platform/request-context/src/lib.rs",
      message: "middleware auth failures must return JSON ErrorBody; plaintext "
        + "`(status, message).into_response()` while shared Unauthorized promises application/json",
    });
  }

  const app = readFileSync(join(repoRoot, "backend/app/src/lib.rs"), "utf8");
  const requestContextHasRetryAfter = /RETRY_AFTER|Retry-After/.test(requestContext);
  const appHasRetryAfter = /RETRY_AFTER|Retry-After/.test(app);
  if (!requestContextHasRetryAfter && !appHasRetryAfter) {
    findings.push({
      location: "backend/crates/platform/request-context/src/lib.rs",
      message: "429 responses must include Retry-After when that status is produced "
        + `(delay-seconds ${RETRY_AFTER_SECONDS} matches the auth rate-limit window)`,
    });
  }

  return { findings };
}

export function evaluateErrorEnvelope({ repoRoot }) {
  const openapi = evaluateOpenapiErrorEnvelope({ repoRoot });
  const runtime = evaluateRuntimeErrorEnvelope({ repoRoot });
  return {
    ...openapi,
    findings: [...openapi.findings, ...runtime.findings],
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateErrorEnvelope({ repoRoot });
  } catch (error) {
    console.error(`error-envelope gate cannot evaluate: ${error.message}`);
    process.exit(1);
  }
  const { paths, responses, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = paths < PATH_FLOOR || responses < RESPONSE_FLOOR;
  if (belowFloor) {
    console.error(`saw ${paths} paths (floor ${PATH_FLOOR}) and ${responses} shared responses `
      + `(floor ${RESPONSE_FLOOR}) — below the floor, the walker examined less of the document `
      + "than it was built to examine");
  }
  if (findings.length > 0 || belowFloor) {
    console.error(`openapi error-envelope gate FAILED: ${findings.length} finding(s), `
      + `${paths} paths, ${responses} shared responses`);
    process.exit(1);
  }
  console.log(`openapi error-envelope gate passed (${paths} paths, ${responses} shared responses, `
    + "0 findings)");
}
