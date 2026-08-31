// Generated API lifecycle gate: composed `info.version` from crate version.
//
// The hole this closes: Palantir/SAP-class lifecycle that generates OpenAPI.
// Shared info YAML used to carry `version:` as a face/hand YAML string. Compose
// re-indented that body into the published document. The crate already has a
// release version (`console-contracts` `CARGO_PKG_VERSION`). Matching by
// coincidence (`0.1.0` == `0.1.0`) is not ownership: a later crate bump leaves
// the published API version stale, and a hand edit of the YAML can drift the
// other way.
//
// Chesterton: do not replace compose. Stamp `info.version` from the version
// compose already has. Do not invent PayRun `version`, ETag, AsyncAPI, or a
// second catalog. `openapi.version` (the OpenAPI spec, `3.1.0`) stays the
// shared preamble field it already is.
//
// Fail-closed: this probe is red on a clean tip while `info.version` is still a
// hand field. Green only when (1) shared info YAML has no `version` key,
// (2) compose source stamps `COMPOSE_API_VERSION` from `CARGO_PKG_VERSION`,
// (3) published `info.version` equals the contracts crate version.
//
// Totality: js-yaml load of info.yaml + published OpenAPI, Cargo.toml
// `[package].version` parse, and a required source token in compose. A walker
// that visits nothing reports nothing, so FILE_FLOOR locks examined-zero.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import yaml from "js-yaml";

import { hasOwnKey, isPlainObject, own } from "./own-property.mjs";

export const OPENAPI_REL = "backend/openapi/openapi.yaml";
export const INFO_REL = "backend/openapi/shared/info.yaml";
export const CARGO_REL = "backend/crates/contracts/Cargo.toml";
export const COMPOSE_REL = "backend/crates/contracts/src/lib.rs";
export const FILE_FLOOR = 4;

export const COMPOSE_API_VERSION_TOKEN =
  'pub const COMPOSE_API_VERSION: &str = env!("CARGO_PKG_VERSION")';
export const COMPOSE_EMIT_TOKEN = "push_str(COMPOSE_API_VERSION)";

function push(findings, location, message) {
  findings.push({ location, message });
}

/**
 * Cargo `[package].version` from the compose crate. Workspace does not declare
 * a version; this is the release version compose already sources.
 *
 * @param {string} toml
 * @returns {string | null}
 */
export function packageVersion(toml) {
  if (typeof toml !== "string") return null;
  const start = toml.search(/^\[package\][ \t]*$/m);
  if (start < 0) return null;
  const afterHeader = toml.slice(start);
  const nl = afterHeader.search(/\r?\n/);
  const body = nl < 0 ? "" : afterHeader.slice(nl + 1);
  const next = body.search(/^\[/m);
  const section = next < 0 ? body : body.slice(0, next);
  const version = section.match(/^[ \t]*version[ \t]*=[ \t]*"([^"]+)"/m);
  return version ? version[1] : null;
}

function loadYaml(repoRoot, rel) {
  return yaml.load(readFileSync(join(repoRoot, rel), "utf8"));
}

function readText(repoRoot, rel) {
  return readFileSync(join(repoRoot, rel), "utf8");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   files: number,
 *   crateVersion: string | null,
 *   publishedVersion: unknown,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateLifecycleVersion({ repoRoot }) {
  const findings = [];
  let files = 0;
  let crateVersion = null;
  let publishedVersion = null;

  let infoDoc;
  try {
    infoDoc = loadYaml(repoRoot, INFO_REL);
    files += 1;
  } catch (error) {
    push(findings, INFO_REL, `cannot parse shared info YAML: ${error.message}`);
    infoDoc = null;
  }

  let cargoToml;
  try {
    cargoToml = readText(repoRoot, CARGO_REL);
    files += 1;
  } catch (error) {
    push(findings, CARGO_REL, `cannot read compose crate manifest: ${error.message}`);
    cargoToml = null;
  }

  let openapiDoc;
  try {
    openapiDoc = loadYaml(repoRoot, OPENAPI_REL);
    files += 1;
  } catch (error) {
    push(
      findings,
      OPENAPI_REL,
      `cannot parse composed OpenAPI: ${error.message}`,
    );
    openapiDoc = null;
  }

  let composeSrc;
  try {
    composeSrc = readText(repoRoot, COMPOSE_REL);
    files += 1;
  } catch (error) {
    push(findings, COMPOSE_REL, `cannot read compose source: ${error.message}`);
    composeSrc = null;
  }

  if (isPlainObject(infoDoc) && hasOwnKey(infoDoc, "version")) {
    push(
      findings,
      `${INFO_REL}:version`,
      "info.version is still a face/hand YAML field; compose must stamp it from "
        + "CARGO_PKG_VERSION so it cannot drift from the crate version",
    );
  }

  crateVersion = cargoToml ? packageVersion(cargoToml) : null;
  if (!crateVersion) {
    push(
      findings,
      `${CARGO_REL}:version`,
      "compose crate [package].version is missing; that is the release version "
        + "compose already sources",
    );
  }

  const info = isPlainObject(openapiDoc) ? own(openapiDoc, "info") : undefined;
  publishedVersion = isPlainObject(info) ? own(info, "version") : undefined;
  if (typeof publishedVersion !== "string" || publishedVersion.length === 0) {
    push(
      findings,
      `${OPENAPI_REL}:info.version`,
      "composed OpenAPI info.version is missing",
    );
  } else if (crateVersion && publishedVersion !== crateVersion) {
    push(
      findings,
      `${OPENAPI_REL}:info.version`,
      `composed info.version ${JSON.stringify(publishedVersion)} drifted from `
        + `compose crate version ${JSON.stringify(crateVersion)}`,
    );
  }

  if (typeof composeSrc === "string") {
    if (!composeSrc.includes(COMPOSE_API_VERSION_TOKEN)) {
      push(
        findings,
        `${COMPOSE_REL}:COMPOSE_API_VERSION`,
        "compose does not source info.version from env!(\"CARGO_PKG_VERSION\"); "
          + "a hand-edited YAML string can drift from the crate version",
      );
    }
    if (!composeSrc.includes(COMPOSE_EMIT_TOKEN)) {
      push(
        findings,
        `${COMPOSE_REL}:emit`,
        "compose does not emit COMPOSE_API_VERSION into info.version",
      );
    }
  }

  if (files < FILE_FLOOR && findings.length === 0) {
    push(
      findings,
      "lifecycle",
      `examined ${files}/${FILE_FLOOR} lifecycle inputs — below the floor`,
    );
  }

  return { files, crateVersion, publishedVersion, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateLifecycleVersion({ repoRoot });
  } catch (error) {
    console.error(`openapi lifecycle-version gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { files, crateVersion, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  if (files < FILE_FLOOR) {
    console.error(
      `saw ${files}/${FILE_FLOOR} lifecycle inputs — below the floor; `
        + "a walker that visits nothing reports nothing",
    );
  }
  if (findings.length > 0 || files < FILE_FLOOR) {
    console.error(
      `openapi lifecycle-version gate FAILED: ${findings.length} finding(s), `
        + `${files}/${FILE_FLOOR} files, crate ${JSON.stringify(crateVersion)}`,
    );
    process.exit(1);
  }
  console.log(
    `openapi lifecycle-version gate passed `
      + `(${files}/${FILE_FLOOR} files, crate ${crateVersion}, compose-owned info.version, `
      + "0 findings)",
  );
}
