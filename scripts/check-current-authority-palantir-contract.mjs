// Current-authority Palantir-contract catch-up gate.
//
// The hole this closes: origin/dev already generates the six-object HTTP
// contract (semantic_manifest.json → composed OpenAPI, Rust codecs, docs, TS
// SDK; #989–#1018). PRODUCT/ROADMAP still describe that generator as
// unimplemented and the body-gate floor as 53/291. Stale current-product
// authority is a governed-system hole. This slice records landed facts and
// remaining HOLDs. It does not invent Events, PayRun Head GET, Group ObjectKey,
// Feature::ALL, HTTP ETag, or JobPosition SSR.
//
// Chesterton: docs/current is the only current-product writer. Do not mix
// OpenAPI/backend occupancy. Do not clear HOLDs. ADR-0031 stays accepted;
// its 2026-08-03 "unimplemented emitter" observation is historical.
//
// Totality: read PRODUCT + ROADMAP + named generator artifacts + the live
// body-gate constants. A walker that reads nothing reports nothing, so
// ARTIFACT_FLOOR / HOLD_FLOOR lock examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const PRODUCT_REL = "docs/current/PRODUCT.md";
export const ROADMAP_REL = "docs/current/ROADMAP.md";
export const BODY_GATE_REL = "scripts/check-request-body-contract.mjs";

export const GENERATOR_ARTIFACTS = Object.freeze([
  "backend/crates/contracts/src/semantic_manifest.json",
  "backend/crates/ontology/rest/src/typed_action_generated.rs",
  "sdk/typescript/src/generated.ts",
  "sdk/docs/index.html",
]);

export const ARTIFACT_FLOOR = GENERATOR_ARTIFACTS.length;

export const STALE_UNIMPLEMENTED = Object.freeze([
  "ADR-0031 remains accepted and unimplemented",
  "not a claim the generator already exists",
  "That is admitted contract law, not implemented by this record",
  "API contract truth (admitted direction, not implemented)",
  "53 resolved of 291",
  "generated SDKs as a CI program",
]);

/** Phrases current authority must keep naming. Clearing any is a HOLD reopen. */
export const REMAINING_HOLD_PHRASES = Object.freeze([
  "ObjectKey::Group",
  "payable",
  "PayrollRunSummary",
  "as_of",
  "JobPosition SSR",
  "AsyncAPI",
  "ETag",
  "Feature::ALL",
  "/_ui",
  "display_name",
  "Intelligence",
  "Ampere A1",
  "credential-reset",
  "Korea",
]);

export const HOLD_FLOOR = REMAINING_HOLD_PHRASES.length;

function push(findings, location, message) {
  findings.push({ location, message });
}

function readOptional(repoRoot, rel) {
  const path = join(repoRoot, rel);
  if (!existsSync(path)) return null;
  return readFileSync(path, "utf8");
}

export function bodyGateConstants(source) {
  if (typeof source !== "string") return { resolvedFloor: null, censusFloor: null, undecidableMax: null };
  const resolved = source.match(/const RESOLVED_FLOOR = (\d+);/);
  const census = source.match(/const CENSUS_FLOOR = (\d+);/);
  const undecidable = source.match(/const BODY_UNDECIDABLE_MAX = (\d+);/);
  return {
    resolvedFloor: resolved ? Number(resolved[1]) : null,
    censusFloor: census ? Number(census[1]) : null,
    undecidableMax: undecidable ? Number(undecidable[1]) : null,
  };
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   artifacts: number,
 *   holds: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateCurrentAuthorityPalantirContract({ repoRoot }) {
  const findings = [];
  const product = readOptional(repoRoot, PRODUCT_REL);
  const roadmap = readOptional(repoRoot, ROADMAP_REL);
  const bodyGate = readOptional(repoRoot, BODY_GATE_REL);
  const authority = `${product ?? ""}\n${roadmap ?? ""}`;

  if (product === null) {
    push(findings, PRODUCT_REL, "current product authority is missing");
  }
  if (roadmap === null) {
    push(findings, ROADMAP_REL, "current roadmap authority is missing");
  }

  let artifacts = 0;
  for (const rel of GENERATOR_ARTIFACTS) {
    if (!existsSync(join(repoRoot, rel))) {
      push(
        findings,
        rel,
        "six-object generator artifact is missing; do not claim the generator exists without this file",
      );
      continue;
    }
    artifacts += 1;
  }

  if (product !== null && artifacts >= ARTIFACT_FLOOR) {
    for (const phrase of STALE_UNIMPLEMENTED) {
      if (authority.includes(phrase)) {
        push(
          findings,
          phrase.startsWith("53 ") || phrase.startsWith("API ") || phrase.startsWith("generated SDK")
            ? ROADMAP_REL
            : PRODUCT_REL,
          `stale unimplemented claim ${JSON.stringify(phrase)} while generator artifacts exist`,
        );
      }
    }
    if (!/semantic_manifest\.json/.test(product)) {
      push(
        findings,
        PRODUCT_REL,
        "must name backend/crates/contracts/src/semantic_manifest.json as the landed six-object generator source",
      );
    }
    if (!/#99[89]\b/.test(authority) && !/#10(0[0-9]|1[0-8])\b/.test(authority)) {
      push(
        findings,
        PRODUCT_REL,
        "must record landed Palantir-contract PRs (#989–#1018), not only admitted direction",
      );
    }
  }

  const constants = bodyGateConstants(bodyGate ?? "");
  if (
    constants.resolvedFloor === null
    || constants.censusFloor === null
    || constants.undecidableMax === null
  ) {
    push(findings, BODY_GATE_REL, "cannot read live body-gate RESOLVED_FLOOR / CENSUS_FLOOR / BODY_UNDECIDABLE_MAX");
  } else if (roadmap !== null) {
    const floorPhrase = `${constants.resolvedFloor} resolved of ${constants.censusFloor}`;
    if (!roadmap.includes(floorPhrase)) {
      push(
        findings,
        ROADMAP_REL,
        `must record the live body-gate floor "${floorPhrase}" from ${BODY_GATE_REL}, not a stale 53/291 snapshot`,
      );
    }
    if (!roadmap.includes(String(constants.undecidableMax))) {
      push(
        findings,
        ROADMAP_REL,
        `must record BODY_UNDECIDABLE_MAX ${constants.undecidableMax} (leftover is non-Head; do not tighten MAX here)`,
      );
    }
  }

  let holds = 0;
  if (product !== null && roadmap !== null) {
    for (const phrase of REMAINING_HOLD_PHRASES) {
      if (!authority.includes(phrase)) {
        push(
          findings,
          PRODUCT_REL,
          `remaining HOLD ${JSON.stringify(phrase)} is missing; do not clear Palantir-class HOLDs`,
        );
        continue;
      }
      holds += 1;
    }
  }

  const belowFloor = artifacts < ARTIFACT_FLOOR || holds < HOLD_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      PRODUCT_REL,
      `saw ${artifacts}/${ARTIFACT_FLOOR} artifacts, ${holds}/${HOLD_FLOOR} HOLD phrases — below the floor`,
    );
  }

  return { artifacts, holds, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  let result;
  try {
    result = evaluateCurrentAuthorityPalantirContract({ repoRoot });
  } catch (error) {
    console.error(`current-authority Palantir-contract gate cannot run: ${error.message}`);
    process.exit(1);
  }
  const { artifacts, holds, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = artifacts < ARTIFACT_FLOOR || holds < HOLD_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${artifacts}/${ARTIFACT_FLOOR} artifacts, ${holds}/${HOLD_FLOOR} HOLD phrases — below the floor`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `current-authority Palantir-contract gate FAILED: ${findings.length} finding(s), `
        + `${artifacts}/${ARTIFACT_FLOOR} artifacts, ${holds}/${HOLD_FLOOR} HOLD phrases`,
    );
    process.exit(1);
  }
  console.log(
    `current-authority Palantir-contract gate passed `
      + `(${artifacts}/${ARTIFACT_FLOOR} artifacts, ${holds}/${HOLD_FLOOR} HOLD phrases, 0 findings)`,
  );
}
