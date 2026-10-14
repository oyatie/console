#!/usr/bin/env node
/**
 * Detector for js-test-reachability.dark-suite.
 *
 * Wraps scripts/dev/mjs-test-reachability.mjs logic against the current tree
 * (not only origin/main) so ProcessHealth / ProductBuild can fail closed on
 * *growth* of dark suites. Existing dark suites are listed; --check fails only
 * when dark suites exist under tools/ci/** or when --strict is passed.
 *
 * Usage:
 *   node tools/ci/check-mjs-dark-suites.mjs           # report JSON; exit 0
 *   node tools/ci/check-mjs-dark-suites.mjs --strict # exit 2 if any dark suite
 *   node tools/ci/check-mjs-dark-suites.mjs --check  # exit 2 if tools/ci dark
 */
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { projectReachableSuites } from "../../scripts/check-js-test-reachability.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const args = new Set(process.argv.slice(2));
const strict = args.has("--strict");
const check = args.has("--check");

const SKIP_DIRS = new Set([
  "node_modules",
  "target",
  "buck-out",
  ".git",
  ".claude",
  ".worktrees",
  "third-party",
]);

function listCandidateFiles(dir, baseRoot, acc = []) {
  // Prefer git inventory so worktrees / scratch clones do not inflate dark counts.
  // Include untracked (e.g. new tools/ci tests) via --others --exclude-standard.
  try {
    const gitOpts = {
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
      stdio: ["ignore", "pipe", "pipe"],
    };
    const tracked = execFileSync(
      "git",
      ["-C", baseRoot, "ls-files"],
      gitOpts,
    );
    const others = execFileSync(
      "git",
      ["-C", baseRoot, "ls-files", "--others", "--exclude-standard"],
      gitOpts,
    );
    const paths = `${tracked}\n${others}`
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean)
      .filter((p) => !p.startsWith(".claude/") && !p.startsWith(".worktrees/"));
    if (paths.length) return [...new Set(paths)].sort();
  } catch {
    /* fall through to filesystem walk (fixtures / non-git) */
  }

  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return acc;
  }
  for (const name of entries) {
    if (SKIP_DIRS.has(name)) continue;
    const full = join(dir, name);
    let st;
    try {
      st = statSync(full);
    } catch {
      continue;
    }
    if (st.isDirectory()) listCandidateFiles(full, baseRoot, acc);
    else {
      acc.push(relative(baseRoot, full).split("\\").join("/"));
    }
  }
  return acc;
}

export function resolveDarkSuites(repoRoot = root) {
  const pkg = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  const scripts = pkg.scripts ?? {};

  // Workflow inventory remains best-effort; the primary owner projects commands.
  const workflowPaths = [];
  const wfDir = join(repoRoot, ".github/workflows");
  try {
    for (const f of readdirSync(wfDir)) {
      if (!/\.ya?ml$/i.test(f)) continue;
      workflowPaths.push(`.github/workflows/${f}`);
    }
  } catch {
    /* no workflows */
  }

  const files = listCandidateFiles(repoRoot, repoRoot);
  const { reached, invoked } = projectReachableSuites({ repoRoot, files, workflowPaths, scripts });
  const suites = files.filter(file => file.endsWith(".test.mjs"));
  const dark = [];
  const wired = [];
  for (const suite of suites) {
    const base = suite.split("/").pop();
    const owner = Object.entries(scripts).find(
      ([, value]) => value.includes(suite) || value.includes(base),
    );
    if (reached.has(suite)) wired.push(suite);
    else {
      dark.push({
        suite,
        npm_script: owner ? owner[0] : null,
        under_tools_ci: suite.startsWith("tools/ci/"),
      });
    }
  }

  const tools_ci_orphan = dark.filter((d) => d.under_tools_ci && !d.npm_script);

  return {
    class_id: "js-test-reachability.dark-suite",
    suite_count: suites.length,
    wired_count: wired.length,
    dark_count: dark.length,
    dark,
    wired,
    tools_ci_orphan,
    npm_scripts_reachable_from_workflow: [...invoked].sort(),
  };
}

function main() {
  const report = resolveDarkSuites(root);
  console.log(JSON.stringify({ ok: true, ...report }, null, 2));

  if (strict && report.dark_count > 0) {
    console.error(
      JSON.stringify(
        {
          class_id: report.class_id,
          message: "js-test-reachability.dark-suite: dark suites remain (strict)",
          dark_count: report.dark_count,
        },
        null,
        2,
      ),
    );
    process.exit(2);
  }
  // Process gate: tools/ci tests must at least have an npm script owner (test:ci-tools).
  // Full CI wiring is product/process work; orphans are process defects.
  if (check && report.tools_ci_orphan.length > 0) {
    console.error(
      JSON.stringify(
        {
          class_id: report.class_id,
          message: "js-test-reachability.dark-suite: tools/ci tests need npm script owner",
          tools_ci_orphan: report.tools_ci_orphan,
        },
        null,
        2,
      ),
    );
    process.exit(2);
  }
}

if (
  import.meta.url === `file://${process.argv[1]}` ||
  process.argv[1]?.endsWith("check-mjs-dark-suites.mjs")
) {
  main();
}
