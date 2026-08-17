#!/usr/bin/env node
/**
 * Reasoning-lens MANIFEST drift check.
 *
 * This replaces `check-reasoning-lens-contract.mjs`, which did two jobs. Only
 * one of them was checkable.
 *
 * KEPT — the identifier-only lens manifest projected into CLAUDE.md must not
 * drift from the canonical list. Two lists either match or they do not, so the
 * check is falsifiable, instant, and its red means something specific.
 *
 * REMOVED — the per-record `lens_contract` evidence block. It required every
 * added or modified governed document to carry a JSON object naming the lenses
 * applied, with a prose rationale per lens. That gate could only verify the
 * object's SHAPE: replacing all six rationales in a real ledger with the string
 * "banana banana banana" left it green. It asserted that reasoning happened,
 * which is not decidable from text, and it produced repeated red for missing
 * paperwork rather than for defects -- `git log` carries seven commits whose
 * whole purpose was healing it, one of which was reverted.
 *
 * The lens policy itself survives in AGENTS.md as guidance. What is gone is the
 * machine demanding proof it cannot evaluate.
 */
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export const CANONICAL_LENSES_V1 = Object.freeze([
  "Cartesian doubt",
  "Essentialism / YAGNI",
  "Chesterton's Fence",
  "Contrarian / outside-the-box",
  "Socratic",
  "Pragmatism",
  "Red Team",
  "Systems Thinking",
  "Operability / Day-2",
  "Opportunity Cost",
  "Blast-radius / cell-based",
  "Constant-work / anti-fragility",
  "Shared-nothing / eventual consistency",
  "FinOps / unit-cost",
  "Telemetry-first",
  "Zero-trust / defense-in-depth",
]);

const MANIFEST_PREAMBLE =
  "## Reasoning lens manifest\n\nCanonical definitions and routing rules live in [AGENTS.md](AGENTS.md#task-selected-reasoning-lenses). This identifier-only projection is drift-checked and does not duplicate policy.";

export const CANONICAL_MANIFEST_BODY_V1 = `${MANIFEST_PREAMBLE}\n\n${CANONICAL_LENSES_V1.map(
  (name, index) => `${index + 1}. ${name}`,
).join("\n")}`;

/** @returns {string[]} failure messages; empty means the projection is intact. */
export function manifestFailures(text, repoPath = "CLAUDE.md") {
  if (text.includes(CANONICAL_MANIFEST_BODY_V1)) return [];
  const expected = CANONICAL_MANIFEST_BODY_V1.split("\n");
  const actual = text.split("\n");
  const at = expected.findIndex((line, i) => actual[actual.indexOf(expected[0]) + i] !== line);
  return [
    `${repoPath}: reasoning lens manifest has drifted from the canonical list`
    + (at >= 0 ? ` (first difference at manifest line ${at + 1}: expected ${JSON.stringify(expected[at])})` : ""),
  ];
}

const isMain = process.argv[1] && process.argv[1].endsWith("check-reasoning-lens-manifest.mjs");
if (isMain) {
  const failures = manifestFailures(readFileSync(resolve(root, "CLAUDE.md"), "utf8"));
  if (failures.length) {
    console.error(failures.join("\n"));
    console.error("Regenerate the numbered list in CLAUDE.md from AGENTS.md#task-selected-reasoning-lenses.");
    process.exitCode = 1;
  } else {
    console.log(`reasoning lens manifest OK (${CANONICAL_LENSES_V1.length} lenses)`);
  }
}
