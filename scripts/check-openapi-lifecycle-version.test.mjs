import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  CARGO_REL,
  COMPOSE_API_VERSION_TOKEN,
  COMPOSE_EMIT_TOKEN,
  COMPOSE_REL,
  FILE_FLOOR,
  INFO_REL,
  OPENAPI_REL,
  evaluateLifecycleVersion,
  packageVersion,
} from "./check-openapi-lifecycle-version.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-lifecycle-version.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-lifecycle-version-"));
  fixtureRoots.push(root);
  for (const [rel, body] of Object.entries(files)) {
    const absolute = join(root, rel);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, body);
  }
  return root;
}

function cargoToml(version) {
  return `[package]
name = "console-contracts"
version = "${version}"
edition = "2024"
`;
}

function composeSrc({ stamp = true, emit = true } = {}) {
  const stampLine = stamp
    ? `${COMPOSE_API_VERSION_TOKEN};\n`
    : "pub const COMPOSE_API_VERSION: &str = \"hand\";\n";
  const emitLine = emit
    ? `    out.push_str("  version: ");\n    out.${COMPOSE_EMIT_TOKEN};\n`
    : "    out.push_str(&reindent(preamble.info, 2));\n";
  return `//! fixture compose\n${stampLine}fn emit_info(out: &mut String) {\n${emitLine}}\n`;
}

function openapiYaml(version) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: ${version}
paths: {}
`;
}

function greenFiles(extra = {}) {
  return {
    [INFO_REL]: "  title: Console API\n",
    [OPENAPI_REL]: openapiYaml("0.2.0"),
    [CARGO_REL]: cargoToml("0.2.0"),
    [COMPOSE_REL]: composeSrc(),
    ...extra,
  };
}

describe("packageVersion", () => {
  it("reads [package].version and ignores dependency versions", () => {
    const toml = `[package]
name = "console-contracts"
version = "1.2.3"
description = "not a version"
edition.workspace = true

[dependencies]
serde = { version = "9.9.9" }
`;
    assert.equal(packageVersion(toml), "1.2.3");
  });

  it("returns null when [package].version is absent", () => {
    assert.equal(packageVersion("[package]\nname = \"x\"\n"), null);
  });
});

describe("evaluateLifecycleVersion", () => {
  it("fails when shared info.yaml still carries a hand version field", () => {
    const root = fixture(greenFiles({
      [INFO_REL]: "  title: Console API\n  version: 0.2.0\n",
    }));
    const result = evaluateLifecycleVersion({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /face\/hand YAML field/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when composed info.version drifted from the crate version", () => {
    const root = fixture(greenFiles({
      [OPENAPI_REL]: openapiYaml("9.9.9"),
    }));
    const result = evaluateLifecycleVersion({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /drifted from compose crate version/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when compose does not source CARGO_PKG_VERSION", () => {
    const root = fixture(greenFiles({
      [COMPOSE_REL]: composeSrc({ stamp: false }),
    }));
    const result = evaluateLifecycleVersion({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /CARGO_PKG_VERSION/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("fails when compose does not emit COMPOSE_API_VERSION", () => {
    const root = fixture(greenFiles({
      [COMPOSE_REL]: composeSrc({ emit: false }),
    }));
    const result = evaluateLifecycleVersion({ repoRoot: root });
    assert.ok(
      result.findings.some((finding) => /does not emit COMPOSE_API_VERSION/.test(finding.message)),
      JSON.stringify(result.findings),
    );
  });

  it("passes a fixture whose compose owns info.version from the crate", () => {
    const root = fixture(greenFiles());
    const result = evaluateLifecycleVersion({ repoRoot: root });
    assert.deepEqual(result.findings, []);
    assert.equal(result.files, FILE_FLOOR);
    assert.equal(result.crateVersion, "0.2.0");
    assert.equal(result.publishedVersion, "0.2.0");
  });
});

describe("cli", () => {
  it("accepts the live tree only when compose owns info.version", () => {
    const run = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });
    assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
    assert.match(run.stdout, /lifecycle-version gate passed/);
  });
});
