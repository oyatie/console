import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  AUTHORITY_DOCUMENTS,
  candidateShasIn,
  inferCurrentCandidate,
  main,
  rebindText,
} from "./rebind-candidate.mjs";

const OLD = "8fc82a9b330c7f94cff74823a1cc0dd1a4826d0a";
const NEW = "e6783147a715c02170203a65b64a0cb53667e4f3";
const ANCHOR = "7ce8d90f953ab2f90e7053dea2965e87f6bfb1f9";

function registry(sha) {
  return {
    governing_lifecycle: { sha256: ANCHOR },
    capabilities: [
      { candidate_evidence: { candidate_sha: sha, contract: { source_sha: sha } } },
      { jurisdiction_bindings: [{ candidate_sha: sha }, { candidate_sha: sha }] },
    ],
  };
}

function seedRoot(registrySha, registerSha = registrySha) {
  const root = mkdtempSync(join(tmpdir(), "rebind-"));
  const shas = [registrySha, registerSha];
  for (const [index, relative] of AUTHORITY_DOCUMENTS.entries()) {
    const path = resolve(root, relative);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, `${JSON.stringify(registry(shas[index]), null, 1)}\n`);
  }
  return root;
}

test("collects only candidate_sha and source_sha leaves", () => {
  const found = candidateShasIn(registry(OLD));
  assert.deepEqual([...found], [OLD], "the lifecycle digest is not a candidate");
});

test("rebinds every occurrence and leaves unrelated SHAs alone", () => {
  const root = seedRoot(OLD);
  const total = main(["--to", NEW], { root, log: () => {} });
  assert.equal(total, 8, "four leaves per document");
  for (const relative of AUTHORITY_DOCUMENTS) {
    const text = readFileSync(resolve(root, relative), "utf8");
    assert.ok(!text.includes(OLD), `${relative} still carries the old candidate`);
    assert.equal(text.split(NEW).length - 1, 4);
    assert.ok(text.includes(ANCHOR), `${relative} lost an unrelated anchor SHA`);
  }
});

test("infers the current candidate when --from is omitted", () => {
  assert.equal(inferCurrentCandidate([JSON.stringify(registry(OLD))]), OLD);
});

test("refuses to guess when the documents disagree", () => {
  const root = seedRoot(OLD, NEW);
  assert.throws(
    () => main(["--to", "0".repeat(40)], { root, log: () => {} }),
    /must agree on exactly one candidate SHA, found 2/,
    "a half-applied bind must not be finished in an arbitrary direction",
  );
});

test("rejects a short SHA rather than binding an unverifiable prefix", () => {
  const root = seedRoot(OLD);
  assert.throws(() => main(["--to", NEW.slice(0, 8)], { root, log: () => {} }), /full 40-hex/);
});

test("rejects a no-op rebind", () => {
  const root = seedRoot(OLD);
  assert.throws(() => main(["--to", OLD], { root, log: () => {} }), /already bound/);
});

test("refuses a substitution that would produce invalid JSON", () => {
  assert.throws(() => rebindText(`{"candidate_sha": "${OLD}"}`, '"', "x"), /JSON/);
});

test("reports an absent candidate instead of writing an unchanged document", () => {
  assert.throws(() => rebindText(`{"candidate_sha": "${NEW}"}`, OLD, NEW), /is not present/);
});

test("the checked-in authority documents agree on one candidate", () => {
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
  const contents = AUTHORITY_DOCUMENTS.map((relative) =>
    readFileSync(resolve(repoRoot, relative), "utf8"),
  );
  assert.match(inferCurrentCandidate(contents), /^[0-9a-f]{40}$/);
});
