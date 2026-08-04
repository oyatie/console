import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, mkdir, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
const run = promisify(execFile);
const script = join(process.cwd(), "scripts/check-doc-links.mjs");

test("accepts local links and ignores external/anchor links", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await mkdir(join(root, "docs"));
  await writeFile(join(root, "README.md"), "[guide](docs/guide.md) [anchor](#top) [web](https://example.com)\n");
  await writeFile(join(root, "docs/guide.md"), "# Guide\n");
  await run(process.execPath, [script, root]);
});

test("fails with file and line for missing local target", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(join(root, "README.md"), "[missing](docs/nope.md)\n");
  await assert.rejects(run(process.execPath, [script, root]), /README\.md:1: missing target: docs\/nope\.md/);
});

test("ignores fenced code", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(join(root, "README.md"), "```md\n[missing](nope.md)\n```\n");
  await run(process.execPath, [script, root]);
});

test("ignores link-shaped examples inside inline code", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(
    join(root, "README.md"),
    "Manifest text: `[AGENTS.md](AGENTS.md#task-selected-reasoning-lenses)`.\n",
  );
  await run(process.execPath, [script, root]);
});

test("still checks links beside inline code", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(join(root, "README.md"), "`[example](ignored.md)` [missing](real-missing.md)\n");
  await assert.rejects(run(process.execPath, [script, root]), /missing target: real-missing\.md/);
});

test("ignores Buck output trees, including dangling artifact symlinks", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(join(root, "README.md"), "# Clean tracked documentation\n");
  await mkdir(join(root, "buck-out"));
  await symlink(join(root, "missing-artifact.md"), join(root, "buck-out", "artifact.md"));
  await run(process.execPath, [script, root]);
});

test("rejects missing extensionless and reference-style targets", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-"));
  await writeFile(join(root, "README.md"), "[direct](missing)\n[ref]: absent-dir\n");
  await assert.rejects(run(process.execPath, [script, root]), /missing target: (missing|absent-dir)/);
});
