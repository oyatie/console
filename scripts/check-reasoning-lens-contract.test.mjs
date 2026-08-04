import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, it } from "node:test";

import {
  CANONICAL_AGENTS_BODY_V1,
  CANONICAL_LENSES_V1,
  CANONICAL_MANIFEST_BODY_V1,
  LENS_CONTRACT_DIGEST_V1,
  evaluateReasoningLensContract,
} from "./check-reasoning-lens-contract.mjs";

const SHARED_START = "<!-- SHARED:REASONING-LENSES:START -->";
const SHARED_END = "<!-- SHARED:REASONING-LENSES:END -->";
const EVIDENCE_START = "<!-- REASONING-LENS-EVIDENCE:START -->";
const EVIDENCE_END = "<!-- REASONING-LENS-EVIDENCE:END -->";

function git(root, ...args) {
  const result = spawnSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "Reasoning Lens Test",
      GIT_AUTHOR_EMAIL: "reasoning-lens@example.invalid",
      GIT_COMMITTER_NAME: "Reasoning Lens Test",
      GIT_COMMITTER_EMAIL: "reasoning-lens@example.invalid",
    },
  });
  assert.equal(
    result.status,
    0,
    `git ${args.join(" ")} failed\nstdout: ${result.stdout}\nstderr: ${result.stderr}`,
  );
  return result.stdout.trim();
}

function write(root, path, contents) {
  const absolute = join(root, ...path.split("/"));
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, contents);
}

function read(root, path) {
  return readFileSync(join(root, ...path.split("/")), "utf8");
}

function sharedBlock(body) {
  return `${SHARED_START}\n${body}\n${SHARED_END}\n`;
}

function standardRecord(overrides = {}) {
  return {
    lens_contract: "v1",
    lens_contract_digest: LENS_CONTRACT_DIGEST_V1,
    task_class: "planning",
    risk_class: "standard",
    risk_domains: [],
    selected_lenses: ["Cartesian doubt", "Pragmatism"],
    task_fit: {
      "Cartesian doubt": "Separated evidence from assumptions.",
      Pragmatism: "Selected the smallest sufficient outcome.",
    },
    mandatory_lens_exceptions: {},
    findings: ["A structural gate is required."],
    decisions_changed_or_rejected: [],
    lens_set_changes: [],
    ...overrides,
  };
}

function templateRecord() {
  return standardRecord({
    task_fit: {
      "Cartesian doubt": "EXAMPLE: Separated evidence from assumptions.",
      Pragmatism: "EXAMPLE: Selected the smallest sufficient outcome.",
    },
    findings: ["EXAMPLE: A structural gate is required."],
  });
}

function highRiskRecord(overrides = {}) {
  return standardRecord({
    risk_class: "high",
    risk_domains: ["approval", "release", "production"],
    selected_lenses: [
      "Red Team",
      "Operability / Day-2",
      "Blast-radius / cell-based",
      "Zero-trust / defense-in-depth",
    ],
    task_fit: {
      "Red Team": "Modeled hostile failure paths.",
      "Operability / Day-2": "Defined recovery ownership.",
      "Blast-radius / cell-based": "Contained the release boundary.",
      "Zero-trust / defense-in-depth": "Required independent readback.",
    },
    findings: ["Rollback must fail closed."],
    ...overrides,
  });
}

function trivialRecord(overrides = {}) {
  return {
    lens_contract: "v1",
    lens_contract_digest: LENS_CONTRACT_DIGEST_V1,
    task_class: "trivial_read_only",
    risk_domains: [],
    selected_lenses: [],
    task_fit: {},
    mandatory_lens_exceptions: {},
    findings: [],
    decisions_changed_or_rejected: [],
    lens_set_changes: [],
    ...overrides,
  };
}

function evidenceFromPayload(payload) {
  return `${EVIDENCE_START}\n\`\`\`json\n${payload}\`\`\`\n${EVIDENCE_END}\n`;
}

function evidence(record) {
  return evidenceFromPayload(`${JSON.stringify(record, null, 2)}\n`);
}

function commitAll(root, message) {
  git(root, "add", "-A");
  git(root, "commit", "-m", message);
  return git(root, "rev-parse", "HEAD");
}

function createFixture() {
  const root = mkdtempSync(join(tmpdir(), "reasoning-lens-contract-"));
  git(root, "init", "-b", "main");
  write(root, "AGENTS.md", sharedBlock(CANONICAL_AGENTS_BODY_V1));
  write(root, "README.md", sharedBlock(CANONICAL_MANIFEST_BODY_V1));
  write(root, "CLAUDE.md", sharedBlock(CANONICAL_MANIFEST_BODY_V1));
  write(
    root,
    "docs/retros/templates/pre-mortem.md",
    `# Pre-mortem\n\n${evidence(templateRecord())}`,
  );
  write(root, "docs/program/ledger/historical.md", "# Historical unmarked ledger\n");
  write(root, "docs/retros/historical.md", "# Historical unmarked retrospective\n");
  const base = commitAll(root, "fixture baseline");
  return { root, base };
}

function addEvidenceFile(root, path, record = standardRecord()) {
  write(root, path, `# Governed record\n\n${evidence(record)}`);
}

function result(root, changedSince = null) {
  return evaluateReasoningLensContract(root, { changedSince });
}

function assertPass(evaluation) {
  assert.deepEqual(evaluation.failures, []);
}

function assertFailure(evaluation, fragment) {
  assert.ok(
    evaluation.failures.some((failure) => failure.includes(fragment)),
    `expected a failure containing ${JSON.stringify(fragment)}; got:\n${evaluation.failures.join("\n")}`,
  );
}

describe("frozen reasoning-lens vocabulary and root serialization", () => {
  it("has the frozen ordered names and digest", () => {
    assert.deepEqual(
      CANONICAL_LENSES_V1.map(({ name }) => name),
      [
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
      ],
    );
    assert.equal(
      createHash("sha256").update(JSON.stringify(CANONICAL_LENSES_V1)).digest("hex"),
      LENS_CONTRACT_DIGEST_V1,
    );
  });

  it("accepts the exact three root blocks and grandfathers historical unmarked records", () => {
    const { root } = createFixture();
    assertPass(result(root));
  });

  const mutations = [
    {
      name: "missing marker",
      paths: ["AGENTS.md"],
      mutate: (text) => text.replace(SHARED_START, ""),
      failure: "exactly one shared reasoning-lens marker pair",
    },
    {
      name: "duplicate marker pair",
      paths: ["AGENTS.md"],
      mutate: (text) => `${text}\n${sharedBlock(CANONICAL_AGENTS_BODY_V1)}`,
      failure: "exactly one shared reasoning-lens marker pair",
    },
    {
      name: "reordered lenses",
      paths: ["AGENTS.md"],
      mutate: (text) =>
        text.replace(
          /1\. \*\*Cartesian doubt\*\*[^\n]+\n2\. \*\*Essentialism \/ YAGNI\*\*[^\n]+/,
          (pair) => pair.split("\n").reverse().join("\n"),
        ),
      failure: "frozen v1 serialization",
    },
    {
      name: "renamed lens",
      paths: ["README.md"],
      mutate: (text) => text.replace("Cartesian doubt", "Cartesian certainty"),
      failure: "frozen v1 serialization",
    },
    {
      name: "whitespace-normalized block",
      paths: ["AGENTS.md"],
      mutate: (text) => text.replace("1. **Cartesian doubt**", "1.  **Cartesian doubt**"),
      failure: "frozen v1 serialization",
    },
    {
      name: "Unicode lookalike",
      paths: ["CLAUDE.md"],
      mutate: (text) => text.replace("Cartesian doubt", "Cartesian dоubt"),
      failure: "frozen v1 serialization",
    },
    {
      name: "definition drift",
      paths: ["AGENTS.md"],
      mutate: (text) => text.replace("challenge assumptions", "accept assumptions"),
      failure: "frozen v1 serialization",
    },
    {
      name: "coordinated three-file drift",
      paths: ["AGENTS.md", "README.md", "CLAUDE.md"],
      mutate: (text) => text.replaceAll("Cartesian doubt", "Cartesian certainty"),
      failure: "frozen v1 serialization",
    },
  ];

  for (const mutation of mutations) {
    it(`rejects ${mutation.name}`, () => {
      const { root } = createFixture();
      for (const path of mutation.paths) write(root, path, mutation.mutate(read(root, path)));
      assertFailure(result(root), mutation.failure);
    });
  }
});

describe("v1 evidence schema", () => {
  it("accepts a canonical nontrivial opt-in record", () => {
    const { root } = createFixture();
    addEvidenceFile(root, "docs/program/ledger/new.md");
    assertPass(result(root));
  });

  const invalidRecords = [
    {
      name: "unknown fields",
      record: () => standardRecord({ surprise: true }),
      failure: "surprise",
    },
    {
      name: "missing fields",
      record: () => {
        const value = standardRecord();
        delete value.findings;
        return value;
      },
      failure: "findings",
    },
    {
      name: "wrong digest",
      record: () => standardRecord({ lens_contract_digest: "0".repeat(64) }),
      failure: "frozen v1 digest",
    },
    {
      name: "unknown lenses",
      record: () =>
        standardRecord({
          selected_lenses: ["Cartesian doubt", "Imaginary lens"],
          task_fit: {
            "Cartesian doubt": "Separated evidence.",
            "Imaginary lens": "Invented a category.",
          },
        }),
      failure: "unknown value",
    },
    {
      name: "duplicate lenses",
      record: () =>
        standardRecord({
          selected_lenses: ["Cartesian doubt", "Cartesian doubt"],
          task_fit: { "Cartesian doubt": "Separated evidence." },
        }),
      failure: "unique and appear in canonical v1 order",
    },
    {
      name: "out-of-order lenses",
      record: () =>
        standardRecord({
          selected_lenses: ["Pragmatism", "Cartesian doubt"],
          task_fit: {
            Pragmatism: "Selected the smallest outcome.",
            "Cartesian doubt": "Separated evidence.",
          },
        }),
      failure: "unique and appear in canonical v1 order",
    },
    {
      name: "one-lens nontrivial records",
      record: () =>
        standardRecord({
          selected_lenses: ["Cartesian doubt"],
          task_fit: { "Cartesian doubt": "Separated evidence." },
        }),
      failure: "must select 2 to 16 lenses",
    },
    {
      name: "mismatched task_fit",
      record: () =>
        standardRecord({
          task_fit: {
            "Cartesian doubt": "Separated evidence.",
            "Red Team": "Modeled misuse.",
          },
        }),
      failure: "keys must equal selected_lenses",
    },
    {
      name: "malformed arrays",
      record: () => standardRecord({ risk_domains: {} }),
      failure: "risk_domains",
    },
    {
      name: "empty findings",
      record: () => standardRecord({ findings: [] }),
      failure: "at least one finding",
    },
  ];

  for (const invalid of invalidRecords) {
    it(`rejects ${invalid.name}`, () => {
      const { root } = createFixture();
      addEvidenceFile(root, "docs/program/ledger/new.md", invalid.record());
      assertFailure(result(root), invalid.failure);
    });
  }

  it("rejects duplicate JSON keys through canonical reserialization", () => {
    const { root } = createFixture();
    const canonical = JSON.stringify(standardRecord(), null, 2);
    const duplicate = canonical.replace(
      '  "lens_contract": "v1",',
      '  "lens_contract": "v1",\n  "lens_contract": "v1",',
    );
    write(
      root,
      "docs/program/ledger/new.md",
      `# Duplicate\n\n${evidenceFromPayload(`${duplicate}\n`)}`,
    );
    assertFailure(result(root), "duplicate keys are forbidden");
  });

  it("rejects otherwise valid but noncanonical JSON formatting", () => {
    const { root } = createFixture();
    write(
      root,
      "docs/program/ledger/new.md",
      `# Minified\n\n${evidenceFromPayload(`${JSON.stringify(standardRecord())}\n`)}`,
    );
    assertFailure(result(root), "not canonical JSON.stringify");
  });

  it("allows EXAMPLE: sentinels in templates and rejects them in governed records", () => {
    const { root } = createFixture();
    assertPass(result(root));
    addEvidenceFile(
      root,
      "docs/program/ledger/new.md",
      standardRecord({ findings: ["EXAMPLE: Placeholder escaped the template."] }),
    );
    assertFailure(result(root), "allowed only under docs/retros/templates/");
  });
});

describe("risk routing and trivial records", () => {
  it("accepts all four mandatory lenses for high-risk work", () => {
    const { root } = createFixture();
    addEvidenceFile(root, "docs/program/ledger/high.md", highRiskRecord());
    assertPass(result(root));
  });

  it("rejects a missing mandatory high-risk lens without a keyed rationale", () => {
    const { root } = createFixture();
    addEvidenceFile(
      root,
      "docs/program/ledger/high.md",
      highRiskRecord({
        selected_lenses: ["Red Team", "Operability / Day-2"],
        task_fit: {
          "Red Team": "Modeled hostile failures.",
          "Operability / Day-2": "Defined recovery ownership.",
        },
      }),
    );
    assertFailure(result(root), "must select Blast-radius / cell-based or provide its keyed rationale");
  });

  it("accepts a lens-specific rationale for an omitted mandatory lens", () => {
    const { root } = createFixture();
    addEvidenceFile(
      root,
      "docs/program/ledger/high.md",
      highRiskRecord({
        selected_lenses: ["Red Team", "Operability / Day-2", "Blast-radius / cell-based"],
        task_fit: {
          "Red Team": "Modeled hostile failures.",
          "Operability / Day-2": "Defined recovery ownership.",
          "Blast-radius / cell-based": "Contained the release boundary.",
        },
        mandatory_lens_exceptions: {
          "Zero-trust / defense-in-depth": "No trust boundary is crossed by this offline artifact.",
        },
      }),
    );
    assertPass(result(root));
  });

  it("rejects exceptions for selected and nonmandatory lenses", () => {
    const selected = createFixture();
    addEvidenceFile(
      selected.root,
      "docs/program/ledger/high.md",
      highRiskRecord({
        mandatory_lens_exceptions: { "Red Team": "Already selected." },
      }),
    );
    assertFailure(result(selected.root), "cannot except a lens that is selected");

    const nonmandatory = createFixture();
    addEvidenceFile(
      nonmandatory.root,
      "docs/program/ledger/high.md",
      highRiskRecord({
        mandatory_lens_exceptions: { "Cartesian doubt": "Not mandatory." },
      }),
    );
    assertFailure(result(nonmandatory.root), "allowed only for the four mandatory high-risk lenses");
  });

  it("does not permit approval to be classified as standard risk", () => {
    const { root } = createFixture();
    addEvidenceFile(
      root,
      "docs/program/ledger/approval.md",
      standardRecord({ risk_domains: ["approval"] }),
    );
    assertFailure(result(root), "standard risk requires an empty risk_domains array");
  });

  it("accepts the exact empty trivial_read_only shape", () => {
    const { root } = createFixture();
    addEvidenceFile(root, "docs/program/ledger/trivial.md", trivialRecord());
    assertPass(result(root));
  });

  it("rejects risk_class or substantive content on trivial_read_only", () => {
    const withRisk = createFixture();
    addEvidenceFile(
      withRisk.root,
      "docs/program/ledger/trivial.md",
      trivialRecord({ risk_class: "standard" }),
    );
    assertFailure(result(withRisk.root), "risk_class");

    const withContent = createFixture();
    addEvidenceFile(
      withContent.root,
      "docs/program/ledger/trivial.md",
      trivialRecord({ findings: ["Not trivial after all."] }),
    );
    assertFailure(result(withContent.root), "must be an empty array for trivial_read_only");
  });
});

describe("diff-aware forward enforcement", () => {
  it("requires evidence on added and modified direct ledger records", () => {
    const added = createFixture();
    write(added.root, "docs/program/ledger/added.md", "# Added without evidence\n");
    commitAll(added.root, "add unmarked ledger");
    assertFailure(result(added.root, added.base), "added or modified governed record");

    const modified = createFixture();
    write(modified.root, "docs/program/ledger/historical.md", "# Modified historical ledger\n");
    commitAll(modified.root, "modify unmarked ledger");
    assertFailure(result(modified.root, modified.base), "added or modified governed record");
  });

  it("accepts an added governed record with valid v1 evidence", () => {
    const { root, base } = createFixture();
    addEvidenceFile(root, "docs/program/ledger/added.md");
    commitAll(root, "add governed ledger");
    assertPass(result(root, base));
  });

  it("ignores deletions", () => {
    const { root, base } = createFixture();
    git(root, "rm", "docs/program/ledger/historical.md");
    git(root, "commit", "-m", "delete old ledger");
    assertPass(result(root, base));
  });

  it("treats a rename as deletion plus evidence-enforced addition", () => {
    const { root, base } = createFixture();
    renameSync(
      join(root, "docs/program/ledger/historical.md"),
      join(root, "docs/program/ledger/renamed.md"),
    );
    commitAll(root, "rename old ledger");
    assertFailure(result(root, base), "docs/program/ledger/renamed.md:evidence");
  });

  it("scans retrospectives recursively", () => {
    const { root, base } = createFixture();
    write(root, "docs/retros/incidents/one.md", "# Unmarked incident\n");
    commitAll(root, "add recursive retrospective");
    assertFailure(result(root, base), "docs/retros/incidents/one.md:evidence");
  });

  it("enforces only flat ledger children and excludes valid templates from diff enforcement", () => {
    const nested = createFixture();
    write(nested.root, "docs/program/ledger/archive/old.md", "# Nested archive\n");
    commitAll(nested.root, "add nested archive");
    assertPass(result(nested.root, nested.base));

    const template = createFixture();
    write(
      template.root,
      "docs/retros/templates/pre-mortem.md",
      `# Revised template prose\n\n${evidence(templateRecord())}`,
    );
    commitAll(template.root, "revise template");
    assertPass(result(template.root, template.base));
  });

  it("fails closed for missing, noncommit, and nonancestor bases", () => {
    const missing = createFixture();
    assertFailure(result(missing.root, "definitely-not-a-ref"), "missing, not a commit, or unavailable");

    const noncommit = createFixture();
    const blob = git(noncommit.root, "hash-object", "README.md");
    assertFailure(result(noncommit.root, blob), "missing, not a commit, or unavailable");

    const divergent = createFixture();
    git(divergent.root, "switch", "-c", "side");
    write(divergent.root, "side.txt", "side\n");
    const side = commitAll(divergent.root, "side commit");
    git(divergent.root, "switch", "main");
    write(divergent.root, "main.txt", "main\n");
    commitAll(divergent.root, "main commit");
    assertFailure(result(divergent.root, side), "is not an ancestor of HEAD");
  });

  it("fails closed when a shallow checkout lacks the requested base", () => {
    const source = createFixture();
    write(source.root, "second.txt", "second\n");
    commitAll(source.root, "second commit");

    const cloneParent = mkdtempSync(join(tmpdir(), "reasoning-lens-shallow-parent-"));
    const clone = join(cloneParent, "clone");
    const cloneResult = spawnSync(
      "git",
      ["clone", "--depth=1", pathToFileURL(source.root).href, clone],
      { encoding: "utf8" },
    );
    assert.equal(cloneResult.status, 0, cloneResult.stderr);
    assertFailure(result(clone, source.base), "missing, not a commit, or unavailable");
  });

  it("includes path, field, mode, base SHA, and head SHA in diagnostics", () => {
    const { root, base } = createFixture();
    write(root, "docs/program/ledger/new.md", "# Missing evidence\n");
    const head = commitAll(root, "missing evidence");
    const evaluation = result(root, base);
    assert.match(
      evaluation.failures.join("\n"),
      new RegExp(
        `docs/program/ledger/new\\.md:evidence: \\[mode=changed-since base=${base} head=${head}\\]`,
      ),
    );
  });
});
