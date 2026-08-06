import test from "node:test";
import assert from "node:assert/strict";
import { copyFile, mkdtemp, mkdir, readFile, symlink, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
const run = promisify(execFile);
const script = join(process.cwd(), "scripts/check-doc-links.mjs");
const generator = join(process.cwd(), "scripts/console/generate-documentation-manifest.mjs");
const classVocabulary = [
  "current",
  "decision",
  "executable-contract",
  "evidence",
  "historical",
  "quarry",
];
const manifestFields = [
  "path",
  "class",
  "owner",
  "status",
  "replacement",
  "retention",
  "blob_sha",
  "archive_tag",
];
const excludedDocumentationPrefixes = [
  "buck-out/",
  "node_modules/",
  "target/",
  "third-party/",
  ".grok/",
];

const authorityPaths = {
  product: "docs/current/PRODUCT.md",
  roadmap: "docs/current/ROADMAP.md",
  delivery: "docs/current/DELIVERY.md",
};

function entry() {
  return {
    path: "README.md",
    class: "current",
    owner: "repository maintainers",
    status: "active",
    replacement: null,
    retention: "retain",
  };
}

function authority(concern, path = authorityPaths[concern]) {
  return {
    concern,
    path,
    class: "current",
    owner: "repository maintainers",
    status: "active",
    replacement: null,
    retention: "retain",
  };
}

function validReadme(extraAuthority = "") {
  return `# Entry

## Current authority

1. [Product](docs/current/PRODUCT.md)
2. [Roadmap](docs/current/ROADMAP.md)
3. [Delivery](docs/current/DELIVERY.md)
${extraAuthority}`;
}

function transition(path, status, replacement, retention) {
  return {
    path,
    class: "historical",
    owner: "repository maintainers",
    status,
    replacement,
    retention,
  };
}

function manifestRecord(record) {
  return {
    path: record.path,
    class: record.class,
    owner: record.owner,
    status: record.status,
    replacement: record.replacement,
    retention: record.retention,
    blob_sha: null,
    archive_tag: null,
  };
}

function validIndex(overrides = {}) {
  const entryRecord = entry();
  const authorities = Object.entries(authorityPaths).map(
    ([concern, path]) => authority(concern, path),
  );
  const transitions = [
    transition("SPEC.md", "redirect", "docs/current/PRODUCT.md", "one-release redirect"),
    transition("DESIGN.md", "redirect", "docs/current/PRODUCT.md", "one-release redirect"),
    transition(
      "docs/PIVOT-2026-07-28.md",
      "frozen",
      "docs/current/PRODUCT.md",
      "retain as historical reconciliation",
    ),
  ];
  return {
    schema_version: 2,
    coverage: "first-party-manifest",
    class_vocabulary: [...classVocabulary],
    future_full_manifest_fields: [...manifestFields],
    entry: entryRecord,
    authorities,
    transitions,
    documents: [
      manifestRecord(entryRecord),
      ...authorities.map(manifestRecord),
      ...transitions.map(manifestRecord),
    ].sort((left, right) => (left.path > right.path) - (left.path < right.path)),
    ...overrides,
  };
}

async function makeIndexedRepo(index, extraFiles = {}) {
  const root = await mkdtemp(join(tmpdir(), "doc-links-index-"));
  await run("git", ["init", "-q"], { cwd: root });
  const files = {
    "README.md": validReadme(),
    "SPEC.md": "# Product redirect\n",
    "DESIGN.md": "# Product redirect\n",
    "docs/PIVOT-2026-07-28.md": "# Historical reconciliation\n",
    "docs/current/PRODUCT.md": "# Product\n",
    "docs/current/ROADMAP.md": "# Roadmap\n",
    "docs/current/DELIVERY.md": "# Delivery\n",
    ...extraFiles,
  };
  for (const [path, contents] of Object.entries(files)) {
    await mkdir(join(root, path, ".."), { recursive: true });
    await writeFile(join(root, path), contents);
  }
  await run("git", ["add", "."], { cwd: root });
  for (const record of Array.isArray(index.documents) ? index.documents : []) {
    if (record?.blob_sha !== null) continue;
    const { stdout } = await run("git", ["ls-files", "--stage", "--", record.path], { cwd: root });
    const oid = stdout.trim().split(/\s+/)[1];
    if (oid) record.blob_sha = oid;
  }
  await mkdir(join(root, "docs"), { recursive: true });
  await writeFile(
    join(root, "docs/documentation-index.json"),
    `${JSON.stringify(index, null, 2)}\n`,
  );
  await run("git", ["add", "docs/documentation-index.json"], { cwd: root });
  return root;
}

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

test("accepts one tracked entry record plus three current authorities", async () => {
  const root = await makeIndexedRepo(validIndex());
  await run(process.execPath, [script, root]);
});

test("rejects unknown root completeness and document-manifest fields", async () => {
  for (const [field, value] of [["complete", true], ["manifest", []]]) {
    const root = await makeIndexedRepo(validIndex({ [field]: value }));
    await assert.rejects(
      run(process.execPath, [script, root]),
      new RegExp(`root record has unexpected field: ${field}`),
    );
  }
});

test("rejects unknown entry, authority, and transition record fields", async () => {
  const entryIndex = validIndex();
  entryIndex.entry.complete = true;
  const entryRoot = await makeIndexedRepo(entryIndex);
  await assert.rejects(
    run(process.execPath, [script, entryRoot]),
    /entry record has unexpected field: complete/,
  );

  const authorityIndex = validIndex();
  authorityIndex.authorities[0].documents = [];
  const authorityRoot = await makeIndexedRepo(authorityIndex);
  await assert.rejects(
    run(process.execPath, [script, authorityRoot]),
    /authority record has unexpected field: documents/,
  );

  const transitionIndex = validIndex({
    transitions: [{
      path: "SPEC.md",
      class: "historical",
      owner: "repository maintainers",
      status: "redirect",
      replacement: "docs/current/PRODUCT.md",
      retention: "one-release redirect",
      complete: true,
    }],
  });
  const transitionRoot = await makeIndexedRepo(transitionIndex, { "SPEC.md": "# Redirect\n" });
  await assert.rejects(
    run(process.execPath, [script, transitionRoot]),
    /transition record has unexpected field: complete/,
  );
});

test("rejects a fourth README current authority", async () => {
  const root = await makeIndexedRepo(validIndex(), {
    "README.md": validReadme("4. [Security](docs/current/SECURITY.md)\n"),
    "docs/current/SECURITY.md": "# Security\n",
  });
  await assert.rejects(
    run(process.execPath, [script, root]),
    /README\.md: current authority list must contain exactly/,
  );
});

test("rejects a prose declaration of a fourth README current authority", async () => {
  const root = await makeIndexedRepo(validIndex(), {
    "README.md": `${validReadme()}
The additional current security authority is [Security](docs/current/SECURITY.md).
`,
    "docs/current/SECURITY.md": "# Security\n",
  });
  await assert.rejects(
    run(process.execPath, [script, root]),
    /README\.md: current authority list must contain exactly/,
  );
});

test("rejects reordered README current authorities", async () => {
  const root = await makeIndexedRepo(validIndex(), {
    "README.md": `# Entry

## Current authority

1. [Roadmap](docs/current/ROADMAP.md)
2. [Product](docs/current/PRODUCT.md)
3. [Delivery](docs/current/DELIVERY.md)
`,
  });
  await assert.rejects(
    run(process.execPath, [script, root]),
    /README\.md: current authority list must contain exactly/,
  );
});

test("reads index and Markdown from the exact Git index tree, not unstaged worktree bytes", async () => {
  const root = await makeIndexedRepo(validIndex());

  await writeFile(join(root, "docs/documentation-index.json"), "{ not staged JSON\n");
  await writeFile(join(root, "README.md"), "[unstaged missing](does-not-exist.md)\n");
  await run(process.execPath, [script, root]);

  await writeFile(join(root, "docs/documentation-index.json"), "{ staged invalid JSON\n");
  await run("git", ["add", "docs/documentation-index.json"], { cwd: root });
  await writeFile(
    join(root, "docs/documentation-index.json"),
    `${JSON.stringify(validIndex(), null, 2)}\n`,
  );
  await assert.rejects(
    run(process.execPath, [script, root]),
    /documentation-index\.json: invalid JSON/,
  );
});

test("rejects a Git worktree with no documentation index", async () => {
  const root = await mkdtemp(join(tmpdir(), "doc-links-index-missing-"));
  await run("git", ["init", "-q"], { cwd: root });
  await writeFile(join(root, "README.md"), "# Entry\n");
  await run("git", ["add", "README.md"], { cwd: root });
  await assert.rejects(
    run(process.execPath, [script, root]),
    /docs\/documentation-index\.json: required in a Git worktree/,
  );
});

test("rejects an untracked or ignored documentation index", async () => {
  for (const ignored of [false, true]) {
    const root = await mkdtemp(join(tmpdir(), "doc-links-index-uncustodied-"));
    await run("git", ["init", "-q"], { cwd: root });
    const files = {
      "README.md": "# Entry\n",
      "docs/current/PRODUCT.md": "# Product\n",
      "docs/current/ROADMAP.md": "# Roadmap\n",
      "docs/current/DELIVERY.md": "# Delivery\n",
      "docs/documentation-index.json": `${JSON.stringify(validIndex(), null, 2)}\n`,
    };
    for (const [path, contents] of Object.entries(files)) {
      await mkdir(join(root, path, ".."), { recursive: true });
      await writeFile(join(root, path), contents);
    }
    if (ignored) {
      await writeFile(join(root, ".gitignore"), "docs/documentation-index.json\n");
      await run("git", ["add", ".gitignore"], { cwd: root });
    }
    await run("git", ["add", "README.md", "docs/current"], { cwd: root });
    await assert.rejects(
      run(process.execPath, [script, root]),
      /documentation-index\.json: index must be a regular blob in the exact Git index tree/,
    );
  }
});

test("cannot be redirected to an attacker-selected Git index", async () => {
  const root = await makeIndexedRepo(validIndex());
  const actualIndex = join(root, ".git/index");
  const alternateIndex = join(root, "alternate.index");
  await copyFile(actualIndex, alternateIndex);
  await run("git", ["rm", "--cached", "docs/documentation-index.json"], { cwd: root });

  await assert.rejects(
    run(process.execPath, [script, root], {
      env: { ...process.env, GIT_INDEX_FILE: alternateIndex },
    }),
    /documentation-index\.json: index must be a regular blob in the exact Git index tree/,
  );
});

test("rejects a tracked entry or authority symlink to uncustodied bytes", async () => {
  for (const [path, kind] of [
    ["README.md", "entry"],
    ["docs/current/PRODUCT.md", "authority"],
  ]) {
    const root = await makeIndexedRepo(validIndex());
    await writeFile(join(root, "external.md"), "# External untracked authority\n");
    await unlink(join(root, path));
    await symlink(join(root, "external.md"), join(root, path));
    await run("git", ["add", path], { cwd: root });
    await assert.rejects(
      run(process.execPath, [script, root]),
      new RegExp(`${kind} path must be a regular blob in the exact Git index tree`),
    );
  }
});

test("rejects an extensionless link whose tracked target is a symlink", async () => {
  const root = await makeIndexedRepo(validIndex());
  await writeFile(join(root, "outside.md"), "# Untracked outside bytes\n");
  await symlink(join(root, "outside.md"), join(root, "outside"));
  await writeFile(join(root, "README.md"), "[outside](outside)\n");
  await run("git", ["add", "README.md", "outside"], { cwd: root });

  await assert.rejects(
    run(process.execPath, [script, root]),
    /README\.md:1: missing target: outside/,
  );
});

test("rejects a missing current authority path", async () => {
  const index = validIndex();
  index.authorities[1].path = "docs/current/MISSING.md";
  const root = await makeIndexedRepo(index);
  await assert.rejects(
    run(process.execPath, [script, root]),
    /documentation-index\.json: authority path is not in the exact Git index tree: docs\/current\/MISSING\.md/,
  );
});

test("rejects duplicate concerns and admitted paths", async () => {
  const duplicateConcern = validIndex();
  duplicateConcern.authorities[1].concern = "product";
  const concernRoot = await makeIndexedRepo(duplicateConcern);
  await assert.rejects(run(process.execPath, [script, concernRoot]), /duplicate concern: product/);

  const duplicatePath = validIndex({
    transitions: [{
      path: "README.md",
      class: "historical",
      owner: "repository maintainers",
      status: "redirect",
      replacement: "docs/current/PRODUCT.md",
      retention: "one-release redirect",
    }],
  });
  const pathRoot = await makeIndexedRepo(duplicatePath);
  await assert.rejects(run(process.execPath, [script, pathRoot]), /duplicate admitted path: README\.md/);
});

test("rejects authority outside the allowed three concern/path pairs", async () => {
  const index = validIndex();
  index.authorities.push(authority("operations", "docs/current/OPERATIONS.md"));
  const root = await makeIndexedRepo(index, { "docs/current/OPERATIONS.md": "# Operations\n" });
  await assert.rejects(run(process.execPath, [script, root]), /unexpected authority concern: operations/);
});

test("rejects invalid redirect and frozen replacements", async () => {
  const badRedirect = validIndex({
    transitions: [{
      path: "SPEC.md",
      class: "historical",
      owner: "repository maintainers",
      status: "redirect",
      replacement: "docs/current/MISSING.md",
      retention: "one-release redirect",
    }],
  });
  const redirectRoot = await makeIndexedRepo(badRedirect, { "SPEC.md": "# Redirect\n" });
  await assert.rejects(run(process.execPath, [script, redirectRoot]), /redirect replacement is not an active authority/);

  const badFrozen = validIndex({
    transitions: [{
      path: "docs/PIVOT-2026-07-28.md",
      class: "historical",
      owner: "repository maintainers",
      status: "frozen",
      replacement: null,
      retention: "retain as historical reconciliation",
    }],
  });
  const frozenRoot = await makeIndexedRepo(badFrozen, { "docs/PIVOT-2026-07-28.md": "# Frozen\n" });
  await assert.rejects(run(process.execPath, [script, frozenRoot]), /frozen replacement is not an active authority/);
});

test("rejects untracked and ignored path admission", async () => {
  const transition = {
    path: "docs/private.md",
    class: "historical",
    owner: "repository maintainers",
    status: "redirect",
    replacement: "docs/current/PRODUCT.md",
    retention: "one-release redirect",
  };
  const untrackedRoot = await makeIndexedRepo(validIndex({ transitions: [transition] }));
  await writeFile(join(untrackedRoot, "docs/private.md"), "# Untracked\n");
  await assert.rejects(run(process.execPath, [script, untrackedRoot]), /admitted path is not in the exact Git index tree: docs\/private\.md/);

  const ignoredRoot = await makeIndexedRepo(validIndex({ transitions: [transition] }));
  await writeFile(join(ignoredRoot, ".gitignore"), "docs/private.md\n");
  await writeFile(join(ignoredRoot, "docs/private.md"), "# Ignored\n");
  await run("git", ["add", ".gitignore"], { cwd: ignoredRoot });
  await assert.rejects(run(process.execPath, [script, ignoredRoot]), /admitted path is not in the exact Git index tree: docs\/private\.md/);
});

test("rejects every premature complete-coverage claim", async () => {
  const root = await makeIndexedRepo(validIndex({ coverage: "complete" }), {
    "docs/reference.md": "# Reference\n",
  });
  await assert.rejects(
    run(process.execPath, [script, root]),
    /coverage complete is not accepted without a signed-archive validation contract/,
  );
});

async function makeGeneratorRepo() {
  const index = validIndex();
  index.documents.push({
    path: "docs/CI-GATES.md",
    class: "executable-contract",
    owner: "repository maintainers",
    status: "active",
    replacement: null,
    retention: "retain",
    blob_sha: null,
    archive_tag: null,
  });
  index.documents.sort((left, right) => (left.path > right.path) - (left.path < right.path));
  const root = await makeIndexedRepo(index, { "docs/CI-GATES.md": "# CI gates\n" });
  await mkdir(join(root, "scripts/console"), { recursive: true });
  await copyFile(generator, join(root, "scripts/console/generate-documentation-manifest.mjs"));
  await writeFile(
    join(root, "docs/documentation-manifest.seed.json"),
    `${JSON.stringify(index.documents, null, 2)}\n`,
  );
  await run(
    "git",
    ["add", "scripts/console/generate-documentation-manifest.mjs", "docs/documentation-manifest.seed.json"],
    { cwd: root },
  );
  return root;
}

function literalStringArray(source, name) {
  const match = source.match(new RegExp(`const ${name} = \\[([\\s\\S]*?)\\n\\];`));
  assert.ok(match, `${name} literal must remain inspectable`);
  return [...match[1].matchAll(/"([^"]+)"/g)].map((item) => item[1]);
}

test("accepts a green schema-v2 exact first-party manifest fixture", async () => {
  const root = await makeIndexedRepo(validIndex());
  await run(process.execPath, [script, root]);
});

test("duplicated first-party universe and class constants cannot drift", async () => {
  const [checkerSource, generatorSource] = await Promise.all([
    readFile(script, "utf8"),
    readFile(generator, "utf8"),
  ]);
  assert.deepEqual(
    literalStringArray(checkerSource, "excludedDocumentationPrefixes"),
    excludedDocumentationPrefixes,
  );
  assert.deepEqual(
    literalStringArray(generatorSource, "excludedDocumentationPrefixes"),
    excludedDocumentationPrefixes,
  );
  assert.deepEqual(literalStringArray(checkerSource, "classVocabulary"), classVocabulary);
  assert.deepEqual(literalStringArray(generatorSource, "classVocabulary"), classVocabulary);
});

test("all seven authority-slice records require exact document projections", async () => {
  for (let recordIndex = 0; recordIndex < 7; recordIndex += 1) {
    const index = validIndex();
    const records = [index.entry, ...index.authorities, ...index.transitions];
    records[recordIndex].class = recordIndex < 4 ? "historical" : "current";
    const root = await makeIndexedRepo(index);
    await assert.rejects(
      run(process.execPath, [script, root]),
      /document projection differs at class/,
    );
  }
});

test("P1 rejects a newly staged unclassified Markdown blob by path", async () => {
  const root = await makeGeneratorRepo();
  await writeFile(join(root, "docs/tmp-unclassified.md"), "# Unclassified\n");
  await run("git", ["add", "docs/tmp-unclassified.md"], { cwd: root });
  await assert.rejects(
    run(process.execPath, ["scripts/console/generate-documentation-manifest.mjs", "--check"], { cwd: root }),
    /docs\/tmp-unclassified\.md must have exactly one record/,
  );
});

test("P2 rejects staged Markdown blob drift until regeneration", async () => {
  const root = await makeGeneratorRepo();
  await writeFile(join(root, "docs/CI-GATES.md"), "# Changed CI gates\n");
  await run("git", ["add", "docs/CI-GATES.md"], { cwd: root });
  await assert.rejects(
    run(process.execPath, ["scripts/console/generate-documentation-manifest.mjs", "--check"], { cwd: root }),
    /docs\/CI-GATES\.md blob_sha does not match/,
  );
});

test("P3 rejects a missing full-manifest document record by path", async () => {
  const index = validIndex();
  index.documents = index.documents.filter((record) => record.path !== "DESIGN.md");
  const root = await makeIndexedRepo(index);
  await assert.rejects(
    run(process.execPath, [script, root]),
    /DESIGN\.md must have exactly one document record/,
  );
});

test("P4 keeps complete coverage fail-closed", async () => {
  const root = await makeIndexedRepo(validIndex({ coverage: "complete" }));
  await assert.rejects(
    run(process.execPath, [script, root]),
    /coverage complete is not accepted without a signed-archive validation contract/,
  );
});

test("P5 rejects non-null archive tags without signed-archive validation", async () => {
  const index = validIndex();
  index.documents[0].archive_tag = "archive-v1";
  const root = await makeIndexedRepo(index);
  await assert.rejects(
    run(process.execPath, [script, root]),
    /archive_tag must be null until signed-archive validation exists/,
  );
});

test("P6 root field allowlist drift still rejects documents", async () => {
  const root = await makeIndexedRepo(validIndex());
  await mkdir(join(root, "scripts"), { recursive: true });
  const source = await readFile(script, "utf8");
  const mutated = source.replace(
    '  "transitions",\n  "documents",\n];',
    '  "transitions",\n];',
  );
  assert.notEqual(mutated, source);
  const mutatedScript = join(root, "scripts/check-doc-links.mjs");
  await writeFile(mutatedScript, mutated);
  await assert.rejects(
    run(process.execPath, [mutatedScript, root]),
    /root record has unexpected field: documents/,
  );
});

test("P7 write preserves a missing semantic class and check remains red", async () => {
  const root = await makeGeneratorRepo();
  const seedPath = join(root, "docs/documentation-manifest.seed.json");
  const seed = JSON.parse(await readFile(seedPath, "utf8"));
  delete seed[0].class;
  await writeFile(seedPath, `${JSON.stringify(seed, null, 2)}\n`);
  await run("git", ["add", "docs/documentation-manifest.seed.json"], { cwd: root });

  await run(
    process.execPath,
    ["scripts/console/generate-documentation-manifest.mjs", "--write"],
    { cwd: root },
  );
  const afterWrite = JSON.parse(await readFile(seedPath, "utf8"));
  assert.equal(Object.hasOwn(afterWrite[0], "class"), false);
  await assert.rejects(
    run(
      process.execPath,
      ["scripts/console/generate-documentation-manifest.mjs", "--check"],
      { cwd: root },
    ),
    /is missing class/,
  );
});

test("custom manifest diagnostics preserve the exact custom scope command", async () => {
  const root = await makeGeneratorRepo();
  const seed = JSON.parse(
    await readFile(join(root, "docs/documentation-manifest.seed.json"), "utf8"),
  );
  delete seed[0].class;
  await mkdir(join(root, "fixtures"), { recursive: true });
  await writeFile(
    join(root, "fixtures/custom scope.json"),
    `${JSON.stringify(seed, null, 2)}\n`,
  );
  await assert.rejects(
    run(
      process.execPath,
      [
        "scripts/console/generate-documentation-manifest.mjs",
        "--check",
        "--file",
        "fixtures/custom scope.json",
      ],
      { cwd: root },
    ),
    /Regenerate with: node scripts\/console\/generate-documentation-manifest\.mjs --write --file 'fixtures\/custom scope\.json'/,
  );
});
