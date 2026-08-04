#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

export const CANONICAL_LENSES_V1 = Object.freeze([
  { name: "Cartesian doubt", definition: "challenge assumptions and separate evidence, inference, and uncertainty." },
  { name: "Essentialism / YAGNI", definition: "pursue the smallest sufficient outcome and avoid speculative scope." },
  { name: "Chesterton's Fence", definition: "understand why an existing constraint or mechanism exists before removing it." },
  { name: "Contrarian / outside-the-box", definition: "test non-obvious alternatives when the default framing may be wrong." },
  { name: "Socratic", definition: "expose hidden premises with focused questions; ask the user only when the answer materially blocks safe progress." },
  { name: "Pragmatism", definition: "optimize for the real-world outcome under actual constraints." },
  { name: "Red Team", definition: "model misuse, adversaries, hostile inputs, and ways the plan can fail." },
  { name: "Systems Thinking", definition: "trace dependencies, feedback loops, second-order effects, and system boundaries." },
  { name: "Operability / Day-2", definition: "account for deployment, diagnosis, maintenance, recovery, and ownership after launch." },
  { name: "Opportunity Cost", definition: "compare the chosen work against the best alternatives in time, complexity, and value." },
  { name: "Blast-radius / cell-based", definition: "contain changes and failures; prefer independently recoverable boundaries." },
  { name: "Constant-work / anti-fragility", definition: "avoid input-dependent blowups, degrade predictably, and use stress to improve the system." },
  { name: "Shared-nothing / eventual consistency", definition: "minimize coordination and make convergence, conflicts, and stale-state behavior explicit." },
  { name: "FinOps / unit-cost", definition: "reason about cost per useful outcome, including operational and scaling costs." },
  { name: "Telemetry-first", definition: "make important state, decisions, failures, and success criteria observable." },
  { name: "Zero-trust / defense-in-depth", definition: "verify every boundary, minimize privilege, and layer independent safeguards." },
]);

export const LENS_CONTRACT_DIGEST_V1 =
  "ac1e7d6b8150808ef73e5e3cd1a1e54d2f37eb43e84aaa1370dbbaaff3c44373";

const computedDigest = createHash("sha256")
  .update(JSON.stringify(CANONICAL_LENSES_V1), "utf8")
  .digest("hex");
if (computedDigest !== LENS_CONTRACT_DIGEST_V1) {
  throw new Error(
    `reasoning-lens v1 constant digest mismatch: expected ${LENS_CONTRACT_DIGEST_V1}, got ${computedDigest}`,
  );
}

const AGENTS_PREAMBLE =
  "## Task-selected reasoning lenses\n\nAll substantive reasoning, planning, implementation, review, and verification must use the smallest task-appropriate subset. Select at least two lenses before nontrivial work, re-evaluate the set when evidence or risk changes, and do not mechanically apply all lenses.";
const AGENTS_EPILOG =
  "High-risk authz, migration, contracts, approval, HR/payroll, release, production, and compliance-sensitive work must include Red Team, Operability / Day-2, Blast-radius / cell-based, and Zero-trust / defense-in-depth, or record a lens-specific not-applicable rationale in durable evidence. Report concise conclusions, evidence, decisions, and tradeoffs rather than private chain-of-thought.";
const MANIFEST_PREAMBLE =
  "## Reasoning lens manifest\n\nCanonical definitions and routing rules live in [AGENTS.md](AGENTS.md#task-selected-reasoning-lenses). This identifier-only projection is drift-checked and does not duplicate policy.";

export const CANONICAL_AGENTS_BODY_V1 = `${AGENTS_PREAMBLE}\n\n${CANONICAL_LENSES_V1.map(
  (lens, index) => `${index + 1}. **${lens.name}** — ${lens.definition}`,
).join("\n")}\n\n${AGENTS_EPILOG}`;

export const CANONICAL_MANIFEST_BODY_V1 = `${MANIFEST_PREAMBLE}\n\n${CANONICAL_LENSES_V1.map(
  (lens, index) => `${index + 1}. ${lens.name}`,
).join("\n")}`;

const SHARED_START = "<!-- SHARED:REASONING-LENSES:START -->";
const SHARED_END = "<!-- SHARED:REASONING-LENSES:END -->";
const EVIDENCE_START = "<!-- REASONING-LENS-EVIDENCE:START -->";
const EVIDENCE_END = "<!-- REASONING-LENS-EVIDENCE:END -->";
const EVIDENCE_OPEN = "\n```json\n";
const EVIDENCE_CLOSE = "```\n";

const TASK_CLASSES = new Set([
  "planning",
  "investigation",
  "implementation",
  "review",
  "verification",
  "trivial_read_only",
]);
const RISK_CLASSES = new Set(["standard", "high"]);
const RISK_DOMAINS = Object.freeze([
  "authz",
  "migration",
  "contracts",
  "approval",
  "hr_payroll",
  "release",
  "production",
  "compliance_sensitive",
  "other",
]);
const LENS_NAMES = Object.freeze(CANONICAL_LENSES_V1.map(({ name }) => name));
const MANDATORY_HIGH_RISK_LENSES = Object.freeze([
  "Red Team",
  "Operability / Day-2",
  "Blast-radius / cell-based",
  "Zero-trust / defense-in-depth",
]);
const NONTRIVIAL_KEYS = Object.freeze([
  "lens_contract",
  "lens_contract_digest",
  "task_class",
  "risk_class",
  "risk_domains",
  "selected_lenses",
  "task_fit",
  "mandatory_lens_exceptions",
  "findings",
  "decisions_changed_or_rejected",
  "lens_set_changes",
]);
const TRIVIAL_KEYS = Object.freeze(NONTRIVIAL_KEYS.filter((key) => key !== "risk_class"));
const WALK_EXCLUSIONS = new Set([
  ".git",
  ".omx",
  "buck-out",
  "dist",
  "node_modules",
  "target",
  "third-party",
  "vendor",
]);

function addFailure(failures, path, location, message) {
  failures.push({ path, location: String(location), message });
}

function countOccurrences(text, token) {
  let count = 0;
  let offset = 0;
  while (true) {
    const found = text.indexOf(token, offset);
    if (found === -1) return count;
    count += 1;
    offset = found + token.length;
  }
}

function lineAt(text, offset) {
  return text.slice(0, Math.max(0, offset)).split("\n").length;
}

function displayPath(root, absolutePath) {
  const fromRoot = relative(root, absolutePath);
  if (fromRoot !== ".." && !fromRoot.startsWith(`..${sep}`) && !isAbsolute(fromRoot)) {
    return (fromRoot || ".").split(sep).join("/");
  }
  return absolutePath;
}

function readRepositoryFile(root, repoPath, failures) {
  const absolutePath = join(root, ...repoPath.split("/"));
  try {
    return readFileSync(absolutePath, "utf8");
  } catch {
    addFailure(failures, repoPath, 1, "file is missing or unreadable");
    return null;
  }
}

function firstDifferentLine(actual, expected) {
  const actualLines = actual.split("\n");
  const expectedLines = expected.split("\n");
  const limit = Math.max(actualLines.length, expectedLines.length);
  for (let index = 0; index < limit; index += 1) {
    if (actualLines[index] !== expectedLines[index]) return index;
  }
  return 0;
}

function validateSharedBlock(root, repoPath, expectedBody, failures) {
  const text = readRepositoryFile(root, repoPath, failures);
  if (text === null) return;

  const startCount = countOccurrences(text, SHARED_START);
  const endCount = countOccurrences(text, SHARED_END);
  if (startCount !== 1 || endCount !== 1) {
    addFailure(
      failures,
      repoPath,
      "marker",
      `expected exactly one shared reasoning-lens marker pair; found start=${startCount}, end=${endCount}`,
    );
    return;
  }

  const start = text.indexOf(SHARED_START);
  const end = text.indexOf(SHARED_END);
  if (end <= start) {
    addFailure(failures, repoPath, lineAt(text, start), "shared reasoning-lens markers are out of order");
    return;
  }

  const bodyStart = start + SHARED_START.length;
  if (text[bodyStart] !== "\n" || text[end - 1] !== "\n") {
    addFailure(
      failures,
      repoPath,
      lineAt(text, start),
      "shared reasoning-lens block must have exactly one newline after the start marker and before the end marker",
    );
    return;
  }

  const actualBody = text.slice(bodyStart + 1, end - 1);
  if (actualBody !== expectedBody) {
    const offset = firstDifferentLine(actualBody, expectedBody);
    addFailure(
      failures,
      repoPath,
      lineAt(text, start) + 1 + offset,
      "shared reasoning-lens body differs from the frozen v1 serialization",
    );
  }
}

function gitText(root, args) {
  return execFileSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

function prepareGitContext(root, changedSince, failures) {
  const context = {
    mode: changedSince === null ? "structural" : "changed-since",
    base: changedSince === null ? "NONE" : changedSince,
    head: "UNKNOWN",
    changes: [],
  };

  try {
    context.head = gitText(root, ["rev-parse", "--verify", "HEAD"]);
  } catch {
    addFailure(failures, "<git>", "HEAD", "cannot resolve the repository HEAD commit");
  }

  if (changedSince === null) return context;

  try {
    execFileSync("git", ["-C", root, "cat-file", "-e", `${changedSince}^{commit}`], {
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch {
    addFailure(
      failures,
      "<git>",
      "base",
      `changed-since base ${JSON.stringify(changedSince)} is missing, not a commit, or unavailable in this checkout`,
    );
    return context;
  }

  try {
    context.base = gitText(root, ["rev-parse", "--verify", `${changedSince}^{commit}`]);
  } catch {
    addFailure(failures, "<git>", "base", "cannot canonicalize the changed-since base commit");
    return context;
  }

  try {
    execFileSync("git", ["-C", root, "merge-base", "--is-ancestor", context.base, "HEAD"], {
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch {
    addFailure(
      failures,
      "<git>",
      "base",
      `changed-since base ${context.base} is not an ancestor of HEAD ${context.head}`,
    );
    return context;
  }

  try {
    const output = execFileSync(
      "git",
      ["-C", root, "diff", "--name-status", "-z", "--no-renames", context.base, "HEAD", "--"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );
    const fields = output.split("\0");
    if (fields.at(-1) === "") fields.pop();
    for (let index = 0; index < fields.length; ) {
      const status = fields[index++];
      const path = fields[index++];
      if (!status || path === undefined) {
        addFailure(failures, "<git>", "diff", "malformed NUL-delimited Git diff output");
        break;
      }
      context.changes.push({ status: status[0], path });
      if (status[0] === "R" || status[0] === "C") index += 1;
    }
  } catch {
    addFailure(failures, "<git>", "diff", "cannot enumerate the changed-since diff");
  }

  return context;
}

function fallbackMarkdownFiles(root) {
  const result = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (!WALK_EXCLUSIONS.has(entry.name)) visit(join(directory, entry.name));
      } else if (entry.isFile() && entry.name.toLowerCase().endsWith(".md")) {
        result.push(displayPath(root, join(directory, entry.name)));
      }
    }
  };
  visit(root);
  return result;
}

function repositoryMarkdownFiles(root) {
  try {
    const output = execFileSync(
      "git",
      ["-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );
    return [...new Set(output.split("\0").filter(Boolean))]
      .filter((path) => path.toLowerCase().endsWith(".md"))
      .filter((path) => path !== ".omx" && !path.startsWith(".omx/"))
      .sort();
  } catch {
    return fallbackMarkdownFiles(root).sort();
  }
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function sameArray(left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function validateExactKeys(value, expected, repoPath, failures) {
  const actual = Object.keys(value);
  for (const key of expected) {
    if (!Object.hasOwn(value, key)) addFailure(failures, repoPath, key, "missing required field");
  }
  for (const key of actual) {
    if (!expected.includes(key)) addFailure(failures, repoPath, key, "unknown field");
  }
}

function validateStringArray(value, field, repoPath, failures, { nonempty = false } = {}) {
  if (!Array.isArray(value)) {
    addFailure(failures, repoPath, field, "must be an array");
    return false;
  }
  if (nonempty && value.length === 0) {
    addFailure(failures, repoPath, field, "must contain at least one item");
  }
  for (const [index, item] of value.entries()) {
    if (typeof item !== "string" || item.trim() === "") {
      addFailure(failures, repoPath, `${field}[${index}]`, "must be a nonblank string");
    }
  }
  return true;
}

function validateCanonicalSubset(value, canonical, field, repoPath, failures) {
  if (!validateStringArray(value, field, repoPath, failures)) return false;
  let previous = -1;
  for (const [index, item] of value.entries()) {
    const canonicalIndex = canonical.indexOf(item);
    if (canonicalIndex === -1) {
      addFailure(failures, repoPath, `${field}[${index}]`, `unknown value ${JSON.stringify(item)}`);
      continue;
    }
    if (canonicalIndex <= previous) {
      addFailure(
        failures,
        repoPath,
        field,
        "values must be unique and appear in canonical v1 order",
      );
      return false;
    }
    previous = canonicalIndex;
  }
  return true;
}

function collectExamplePaths(value, prefix = "$") {
  const found = [];
  if (typeof value === "string") {
    if (value.startsWith("EXAMPLE:")) found.push(prefix);
  } else if (Array.isArray(value)) {
    for (const [index, item] of value.entries()) {
      found.push(...collectExamplePaths(item, `${prefix}[${index}]`));
    }
  } else if (isPlainObject(value)) {
    for (const [key, item] of Object.entries(value)) {
      found.push(...collectExamplePaths(item, `${prefix}.${key}`));
    }
  }
  return found;
}

function extractEvidenceBlock(text, repoPath, failures) {
  const startCount = countOccurrences(text, EVIDENCE_START);
  const endCount = countOccurrences(text, EVIDENCE_END);
  if (startCount === 0 && endCount === 0) {
    return { optedIn: false, payload: null, line: 1 };
  }
  if (startCount !== 1 || endCount !== 1) {
    addFailure(
      failures,
      repoPath,
      "marker",
      `expected exactly one reasoning-lens evidence marker pair; found start=${startCount}, end=${endCount}`,
    );
    return { optedIn: true, payload: null, line: 1 };
  }

  const start = text.indexOf(EVIDENCE_START);
  const end = text.indexOf(EVIDENCE_END);
  const startLine = lineAt(text, start);
  if (end <= start) {
    addFailure(failures, repoPath, startLine, "reasoning-lens evidence markers are out of order");
    return { optedIn: true, payload: null, line: startLine };
  }

  const between = text.slice(start + EVIDENCE_START.length, end);
  if (!between.startsWith(EVIDENCE_OPEN) || !between.endsWith(EVIDENCE_CLOSE)) {
    addFailure(
      failures,
      repoPath,
      startLine,
      "evidence block must use exact marker, json-fence, and newline serialization",
    );
    return { optedIn: true, payload: null, line: startLine };
  }

  return {
    optedIn: true,
    payload: between.slice(EVIDENCE_OPEN.length, -EVIDENCE_CLOSE.length),
    line: startLine,
  };
}

function validateEvidencePayload(payload, repoPath, isTemplate, failures) {
  let value;
  try {
    value = JSON.parse(payload);
  } catch {
    addFailure(failures, repoPath, "json", "evidence payload is not valid JSON");
    return false;
  }

  if (!isPlainObject(value)) {
    addFailure(failures, repoPath, "json", "evidence payload must be a JSON object");
    return false;
  }

  if (payload !== `${JSON.stringify(value, null, 2)}\n`) {
    addFailure(
      failures,
      repoPath,
      "json",
      "evidence payload is not canonical JSON.stringify(value, null, 2) plus one newline; duplicate keys are forbidden",
    );
  }

  const trivial = value.task_class === "trivial_read_only";
  validateExactKeys(value, trivial ? TRIVIAL_KEYS : NONTRIVIAL_KEYS, repoPath, failures);

  if (value.lens_contract !== "v1") {
    addFailure(failures, repoPath, "lens_contract", "must equal v1");
  }
  if (value.lens_contract_digest !== LENS_CONTRACT_DIGEST_V1) {
    addFailure(failures, repoPath, "lens_contract_digest", "does not match the frozen v1 digest");
  }
  if (!TASK_CLASSES.has(value.task_class)) {
    addFailure(failures, repoPath, "task_class", `invalid task class ${JSON.stringify(value.task_class)}`);
  }

  const riskDomainsValid = validateCanonicalSubset(
    value.risk_domains,
    RISK_DOMAINS,
    "risk_domains",
    repoPath,
    failures,
  );
  const selectedValid = validateCanonicalSubset(
    value.selected_lenses,
    LENS_NAMES,
    "selected_lenses",
    repoPath,
    failures,
  );

  if (!isPlainObject(value.task_fit)) {
    addFailure(failures, repoPath, "task_fit", "must be an object");
  } else {
    const fitKeys = Object.keys(value.task_fit);
    if (Array.isArray(value.selected_lenses) && !sameArray(fitKeys, value.selected_lenses)) {
      addFailure(
        failures,
        repoPath,
        "task_fit",
        "keys must equal selected_lenses in canonical order",
      );
    }
    for (const [key, explanation] of Object.entries(value.task_fit)) {
      if (typeof explanation !== "string" || explanation.trim() === "") {
        addFailure(failures, repoPath, `task_fit.${key}`, "must be a nonblank outcome-level explanation");
      }
    }
  }

  if (!isPlainObject(value.mandatory_lens_exceptions)) {
    addFailure(failures, repoPath, "mandatory_lens_exceptions", "must be an object");
  }

  const findingsValid = validateStringArray(value.findings, "findings", repoPath, failures);
  validateStringArray(
    value.decisions_changed_or_rejected,
    "decisions_changed_or_rejected",
    repoPath,
    failures,
  );
  validateStringArray(value.lens_set_changes, "lens_set_changes", repoPath, failures);

  if (trivial) {
    for (const field of [
      "risk_domains",
      "selected_lenses",
      "findings",
      "decisions_changed_or_rejected",
      "lens_set_changes",
    ]) {
      if (!Array.isArray(value[field]) || value[field].length !== 0) {
        addFailure(failures, repoPath, field, "must be an empty array for trivial_read_only");
      }
    }
    for (const field of ["task_fit", "mandatory_lens_exceptions"]) {
      if (!isPlainObject(value[field]) || Object.keys(value[field]).length !== 0) {
        addFailure(failures, repoPath, field, "must be an empty object for trivial_read_only");
      }
    }
  } else {
    if (!RISK_CLASSES.has(value.risk_class)) {
      addFailure(failures, repoPath, "risk_class", `invalid risk class ${JSON.stringify(value.risk_class)}`);
    }
    if (!Array.isArray(value.selected_lenses) || value.selected_lenses.length < 2 || value.selected_lenses.length > 16) {
      addFailure(failures, repoPath, "selected_lenses", "nontrivial records must select 2 to 16 lenses");
    }
    if (!findingsValid || value.findings.length === 0) {
      addFailure(failures, repoPath, "findings", "nontrivial records must contain at least one finding");
    }

    if (value.risk_class === "standard") {
      if (!Array.isArray(value.risk_domains) || value.risk_domains.length !== 0) {
        addFailure(failures, repoPath, "risk_domains", "standard risk requires an empty risk_domains array");
      }
      if (
        !isPlainObject(value.mandatory_lens_exceptions) ||
        Object.keys(value.mandatory_lens_exceptions).length !== 0
      ) {
        addFailure(
          failures,
          repoPath,
          "mandatory_lens_exceptions",
          "standard risk requires an empty exceptions object",
        );
      }
    }

    if (value.risk_class === "high") {
      if (!riskDomainsValid || value.risk_domains.length === 0) {
        addFailure(failures, repoPath, "risk_domains", "high risk requires a nonempty canonical domain subset");
      }
      if (isPlainObject(value.mandatory_lens_exceptions)) {
        const exceptionKeys = Object.keys(value.mandatory_lens_exceptions);
        for (const key of exceptionKeys) {
          if (!MANDATORY_HIGH_RISK_LENSES.includes(key)) {
            addFailure(
              failures,
              repoPath,
              `mandatory_lens_exceptions.${key}`,
              "exceptions are allowed only for the four mandatory high-risk lenses",
            );
          }
          if (Array.isArray(value.selected_lenses) && value.selected_lenses.includes(key)) {
            addFailure(
              failures,
              repoPath,
              `mandatory_lens_exceptions.${key}`,
              "cannot except a lens that is selected",
            );
          }
          const explanation = value.mandatory_lens_exceptions[key];
          if (typeof explanation !== "string" || explanation.trim() === "") {
            addFailure(
              failures,
              repoPath,
              `mandatory_lens_exceptions.${key}`,
              "must be a nonblank lens-specific rationale",
            );
          }
        }
        for (const lens of MANDATORY_HIGH_RISK_LENSES) {
          if (
            (!Array.isArray(value.selected_lenses) || !value.selected_lenses.includes(lens)) &&
            !Object.hasOwn(value.mandatory_lens_exceptions, lens)
          ) {
            addFailure(
              failures,
              repoPath,
              "mandatory_lens_exceptions",
              `high-risk record must select ${lens} or provide its keyed rationale`,
            );
          }
        }
      }
    }
  }

  if (!selectedValid && Array.isArray(value.selected_lenses)) {
    // The canonical-subset diagnostic above owns the detailed ordering or value failure.
  }

  const examplePaths = collectExamplePaths(value);
  if (!isTemplate && examplePaths.length > 0) {
    addFailure(
      failures,
      repoPath,
      examplePaths[0],
      "EXAMPLE: sentinel strings are allowed only under docs/retros/templates/",
    );
  }
  if (isTemplate && examplePaths.length === 0) {
    addFailure(failures, repoPath, "example", "template evidence must contain EXAMPLE: sentinel values");
  }

  return true;
}

function isTemplatePath(path) {
  return path.startsWith("docs/retros/templates/");
}

function isDiffGovernedPath(path) {
  const flatLedger = /^docs\/program\/ledger\/[^/]+\.md$/.test(path);
  const retrospective = /^docs\/retros\/.+\.md$/.test(path) && !isTemplatePath(path);
  return flatLedger || retrospective;
}

function formatFailure(failure, context) {
  return `${failure.path}:${failure.location}: [mode=${context.mode} base=${context.base} head=${context.head}] ${failure.message}`;
}

export function evaluateReasoningLensContract(
  rootInput = process.cwd(),
  { changedSince = null } = {},
) {
  const root = resolve(rootInput);
  const failures = [];
  const context = prepareGitContext(root, changedSince, failures);

  validateSharedBlock(root, "AGENTS.md", CANONICAL_AGENTS_BODY_V1, failures);
  validateSharedBlock(root, "README.md", CANONICAL_MANIFEST_BODY_V1, failures);
  validateSharedBlock(root, "CLAUDE.md", CANONICAL_MANIFEST_BODY_V1, failures);

  const markdownPaths = repositoryMarkdownFiles(root);
  const evidenceValidity = new Map();
  let evidenceCount = 0;
  let templateCount = 0;

  for (const repoPath of markdownPaths) {
    const text = readRepositoryFile(root, repoPath, failures);
    if (text === null) continue;
    const template = isTemplatePath(repoPath);
    if (template) templateCount += 1;
    const before = failures.length;
    const extracted = extractEvidenceBlock(text, repoPath, failures);
    if (!extracted.optedIn) {
      evidenceValidity.set(repoPath, false);
      if (template) {
        addFailure(failures, repoPath, "evidence", "retrospective template must contain one v1 example block");
      }
      continue;
    }
    evidenceCount += 1;
    if (extracted.payload !== null) {
      validateEvidencePayload(extracted.payload, repoPath, template, failures);
    }
    evidenceValidity.set(repoPath, extracted.payload !== null && failures.length === before);
  }

  if (!existsSync(join(root, "docs", "retros", "templates")) || templateCount === 0) {
    addFailure(
      failures,
      "docs/retros/templates",
      "directory",
      "missing retrospective templates with canonical v1 examples",
    );
  }

  if (changedSince !== null) {
    for (const { status, path } of context.changes) {
      if ((status !== "A" && status !== "M") || !isDiffGovernedPath(path)) continue;
      if (!evidenceValidity.get(path)) {
        addFailure(
          failures,
          path,
          "evidence",
          "added or modified governed record must contain one valid lens_contract v1 evidence block",
        );
      }
    }
  }

  return {
    root,
    mode: context.mode,
    base: context.base,
    head: context.head,
    evidenceCount,
    failures: failures.map((failure) => formatFailure(failure, context)),
  };
}

function parseArguments(argv) {
  let root = process.cwd();
  let changedSince = null;
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--changed-since") {
      if (changedSince !== null || index + 1 >= argv.length) {
        throw new Error("--changed-since requires exactly one base revision");
      }
      changedSince = argv[++index];
    } else if (argument === "--root") {
      if (index + 1 >= argv.length) throw new Error("--root requires a directory");
      root = resolve(argv[++index]);
    } else {
      throw new Error(`unknown argument ${JSON.stringify(argument)}`);
    }
  }
  return { root, changedSince };
}

export function main(argv = process.argv.slice(2)) {
  let options;
  try {
    options = parseArguments(argv);
  } catch (error) {
    console.error(`reasoning lens contract usage error: ${error.message}`);
    process.exitCode = 2;
    return;
  }

  const result = evaluateReasoningLensContract(options.root, {
    changedSince: options.changedSince,
  });
  if (result.failures.length > 0) {
    console.error(result.failures.join("\n"));
    process.exitCode = 1;
    return;
  }

  console.log(
    `reasoning lens contract OK (mode=${result.mode} base=${result.base} head=${result.head} evidence_blocks=${result.evidenceCount})`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main();
}
