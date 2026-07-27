import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import test from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const checker = resolve(root, "scripts/check-foundation-gates.mjs");
const liveAuthorityContracts = [
  "docs/specs/foundation-gates.md",
  "docs/specs/review-fix-merge-governance.md",
];

function runChecker() {
  return spawnSync(process.execPath, [checker], {
    cwd: root,
    encoding: "utf8",
  });
}

function withInjectedContractText(path, text, callback) {
  const absolutePath = resolve(root, path);
  const original = readFileSync(absolutePath, "utf8");
  writeFileSync(absolutePath, `${original}\n${text}\n`);
  try {
    callback();
  } finally {
    writeFileSync(absolutePath, original);
  }
}

test("foundation gate rejects retired authority reentry in every live authority contract", () => {
  const hostileLiterals = [
    "hermes kanban",
    "Hermes agent authority",
    "hermes session authority",
    "OMC",
    "GJC",
    "NousResearch Hermes",
    "~/.codex/agents executor authority",
  ];

  for (const path of liveAuthorityContracts) {
    for (const hostileLiteral of hostileLiterals) {
      withInjectedContractText(path, hostileLiteral, () => {
        const result = runChecker();
        assert.notEqual(result.status, 0, `${path} accepted ${hostileLiteral}`);
        assert.match(result.stderr, /excludes retired|React Native Hermes JS engine/i);
      });
    }
  }
});

test("foundation gate permits the React Native Hermes JS engine as a technical dependency", () => {
  for (const path of liveAuthorityContracts) {
    withInjectedContractText(path, "React Native Hermes JS engine technical dependency", () => {
      const result = runChecker();
      assert.equal(result.status, 0, `${path} incorrectly rejected the React Native Hermes JS engine`);
    });
  }
});
