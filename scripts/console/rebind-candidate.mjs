#!/usr/bin/env node
// Rebinds the console authority documents onto a new candidate commit.
//
// Advancing the candidate is a mechanical, total substitution: every
// `candidate_sha` and `source_sha` leaf in the capability registry and the
// jurisdiction register carries the candidate commit, so a bind rewrites the
// same ~389 occurrences every time. Doing that by hand is both error-prone
// (a short SHA or a mistyped digit silently produces an unverifiable bind)
// and the reason concurrent lanes collide on these two files.
//
// Unrelated 40-hex values in the documents — lifecycle digests, upstream
// anchors — are left untouched: only the previous candidate SHA is replaced.
//
// Usage:
//   node scripts/console/rebind-candidate.mjs --from <old-sha> --to <new-sha>
//   node scripts/console/rebind-candidate.mjs --to <new-sha>   # infers --from
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export const AUTHORITY_DOCUMENTS = [
  "docs/program/console-capability-registry.json",
  "docs/program/console-jurisdiction-register.json",
];

const FULL_SHA = /^[0-9a-f]{40}$/;

/** Every SHA-shaped string carried by a `candidate_sha`/`source_sha` leaf. */
export function candidateShasIn(json) {
  const found = new Set();
  const walk = (node) => {
    if (Array.isArray(node)) {
      node.forEach(walk);
    } else if (node && typeof node === "object") {
      for (const [key, value] of Object.entries(node)) {
        if ((key === "candidate_sha" || key === "source_sha") && typeof value === "string") {
          found.add(value);
        }
        walk(value);
      }
    }
  };
  walk(json);
  return found;
}

/**
 * The single SHA currently bound across every authority document. Refuses to
 * guess when the documents disagree: a split candidate means a half-applied
 * bind, and picking either half would finish it in the wrong direction.
 */
export function inferCurrentCandidate(contents) {
  const shas = new Set();
  for (const text of contents) {
    for (const sha of candidateShasIn(JSON.parse(text))) shas.add(sha);
  }
  if (shas.size !== 1) {
    throw new Error(
      `authority documents must agree on exactly one candidate SHA, found ${shas.size}: ` +
        `${[...shas].join(", ") || "(none)"}`,
    );
  }
  return [...shas][0];
}

export function rebindText(text, from, to) {
  const occurrences = text.split(from).length - 1;
  if (occurrences === 0) throw new Error(`candidate SHA ${from} is not present`);
  const rebound = text.replaceAll(from, to);
  JSON.parse(rebound); // fail closed rather than write a corrupt authority document
  return { rebound, occurrences };
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i];
    if (flag !== "--from" && flag !== "--to") throw new Error(`unknown argument ${flag}`);
    args[flag.slice(2)] = argv[i + 1];
  }
  return args;
}

export function main(argv, { root = REPO_ROOT, log = console.log } = {}) {
  const args = parseArgs(argv);
  const paths = AUTHORITY_DOCUMENTS.map((relative) => resolve(root, relative));
  const contents = paths.map((path) => readFileSync(path, "utf8"));

  const to = args.to;
  if (!FULL_SHA.test(to ?? "")) {
    throw new Error(`--to must be a full 40-hex commit SHA, got ${to ?? "(missing)"}`);
  }
  const from = args.from ?? inferCurrentCandidate(contents);
  if (!FULL_SHA.test(from)) {
    throw new Error(`--from must be a full 40-hex commit SHA, got ${from}`);
  }
  if (from === to) throw new Error("candidate is already bound to that commit");

  let total = 0;
  for (const [index, path] of paths.entries()) {
    const { rebound, occurrences } = rebindText(contents[index], from, to);
    writeFileSync(path, rebound);
    total += occurrences;
    log(`${AUTHORITY_DOCUMENTS[index]}: rebound ${occurrences} occurrences`);
  }
  log(`candidate ${from} -> ${to} (${total} occurrences)`);
  return total;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`rebind-candidate: ${error.message}`);
    process.exit(1);
  }
}
