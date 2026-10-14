#!/usr/bin/env node
/**
 * S3′ — candidate-bound JavaScript .test.mjs reachability ratchet.
 *
 * Unlike scripts/dev/mjs-test-reachability.mjs (diagnostic, reads origin/main),
 * this gate:
 * - enumerates suites via `git ls-files` at HEAD (candidate-bound)
 * - expands npm scripts from the candidate package.json to a fixed point
 * - counts a suite as reached only by exact path match in executable surface
 * - treats basename-only hits as diagnostic (stderr), never as green coverage
 * - fails closed when a workflow-invoked npm script cannot be resolved
 * - ratchets unexplained dark suites against docs/program/js-test-reachability-baseline.json
 */

import { execFileSync } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import process from "node:process";

import { directExecutable, executableWorkflowCommands, shellCommandTokens } from "./lib/ci-workflow-executables.mjs";
import { sweepReachableText } from "../tools/ci/gate-sweep.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const baselinePath = "docs/program/js-test-reachability-baseline.json";
const jsonMode = process.argv.includes("--json");

function gitLsFiles() {
  return execFileSync("git", ["-C", root, "ls-files", "-z"], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  })
    .split("\0")
    .filter(Boolean);
}

function loadJson(rel) {
  return JSON.parse(readFileSync(join(root, rel), "utf8"));
}

function shellCommands(text) {
  const commands = [];
  for (const surface of shellCommandTokens(text)) {
    if (surface.malformed) continue;
    let tokens = [];
    for (const token of [...surface.tokens, ";"]) {
      if ([";", "|", "&", "||", "&&"].includes(token)) {
        if (tokens.length) commands.push({ tokens });
        tokens = [];
      } else tokens.push(token);
    }
  }
  return commands;
}

function nodeFiles(tokens) {
  let testMode = false;
  const valueOptions = new Set(["--test-name-pattern", "--test-skip-pattern", "--test-reporter",
    "--test-reporter-destination", "--test-concurrency", "--test-timeout", "--test-shard",
    "--require", "-r", "--import", "--loader", "--experimental-loader", "--title", "--conditions", "-C"]);
  for (let index = 1; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (/^-[epc]/.test(token) || ["-h", "-v"].includes(token)
        || /^--(?:eval|print|check|help|version|v8-options)(?:=|$)/.test(token)) return [];
    if (token === "--test") testMode = true;
    else if (valueOptions.has(token)) index += 1;
    else if (token === "--") return testMode ? tokens.slice(index + 1) : tokens.slice(index + 1, index + 2);
    else if (!token.startsWith("-")) return testMode ? tokens.slice(index) : [token];
    else if (!token.includes("=") && !["--test-only", "--test-force-exit", "--experimental-test-coverage", "--enable-source-maps"].includes(token)) return [];
  }
  return [];
}

function nativeHrChild(files) {
  const launcher = "scripts/run-native-hr-policy-parser-contract.sh";
  const supervisor = "scripts/check-native-hr-policy-parser-contract.mjs";
  const suite = "scripts/check-native-hr-policy-parser-contract.test.mjs";
  if (![launcher, supervisor, suite].every(file => files.has(file))) return [];
  const launches = shellCommands(readFileSync(join(root, launcher), "utf8")).some(entry => {
    const { tokens } = directExecutable(entry.tokens);
    return tokens[0] === "$CONSOLE_HR_POLICY_NODE" && tokens[1] === "$SCRIPT_DIR/check-native-hr-policy-parser-contract.mjs";
  });
  if (!launches) return [];

  const source = readFileSync(join(root, supervisor), "utf8");
  // Mask comments and quoted data positionally: neither can supply executable calls.
  const code = source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`/g,
    text => text.replace(/[^\n]/g, " "));
  const depthAt = index => {
    let depth = 0;
    for (const character of code.slice(0, index)) {
      if (character === "{") depth += 1;
      else if (character === "}") depth -= 1;
    }
    return depth;
  };
  const imported = [...code.matchAll(/^\s*import\s*\{\s*spawn\s*\}\s*from\s+;/gm)]
    .some(match => /from\s*(['"])node:child_process\1/.test(source.slice(match.index, match.index + match[0].length)));
  if (!imported) return [];
  for (const match of code.matchAll(/^\s*const\s+child\s*=\s*spawn\(\s*binding\.node\.path\s*,\s*\[\s*,\s*\]\s*,\s*\{/gm)) {
    const actual = source.slice(match.index, match.index + match[0].length);
    if (!/spawn\(\s*binding\.node\.path\s*,\s*\[\s*(['"])--test\1\s*,\s*(['"])scripts\/check-native-hr-policy-parser-contract\.test\.mjs\2\s*\]\s*,\s*\{/.test(actual)) continue;
    // shortcut: only this declared supervisor topology; review new child shapes separately.
    const fixture = depthAt(match.index) === 0 && /^\s*import\s*\{\s*spawn\s*\}\s*from\s+;\s*const\s+binding\s*=\s*\{\s*node\s*:\s*\{\s*path\s*:\s*process\.execPath\s*\}\s*\}\s*;\s*$/.test(code.slice(0, match.index));
    const owner = depthAt(match.index) === 2
      && /export async function childProcess\(binding, run, nonce, isolation\)\s*\{\s*return await new Promise\(resolve =>\s*\{\s*$/.test(code.slice(0, match.index))
      && [...code.matchAll(/^\s*child\s*=\s*await childProcess\(binding, run, nonce, isolation\);/gm)].some(call => {
        const main = code.slice(0, call.index).match(/export async function main\(\)\s*\{\s*(?:let [^;]*;\s*)+try\s*\{([^}]*)$/);
        return depthAt(call.index) === 2 && main && !/\breturn\b|\bif\s*\(\s*(?:false|0|null)\s*\)/.test(main[1]);
      })
      && [...code.matchAll(/^if \(process\.argv\[1\] === new URL\(import\.meta\.url\)\.pathname\) process\.exitCode = await main\(\);/gm)].some(call => depthAt(call.index) === 0);
    if (fixture || owner) return [suite];
  }
  return [];
}

function main() {
  const failures = [];
  const files = gitLsFiles();
  const suites = files.filter((f) => f.endsWith(".test.mjs")).sort();
  const workflows = files.filter((f) => f.startsWith(".github/workflows/") && f.endsWith(".yml"));

  if (!existsSync(join(root, "package.json"))) {
    failures.push("package.json missing on candidate");
  }
  const pkg = loadJson("package.json");
  const scripts = pkg.scripts ?? {};

  const commands = [];
  for (const wf of workflows) {
    const text = readFileSync(join(root, wf), "utf8");
    try {
      commands.push(...executableWorkflowCommands(text));
    } catch (error) {
      failures.push(`${wf}: cannot extract executable commands: ${error.message}`);
    }
  }

  const invoked = new Set();
  const reached = new Set();
  const tracked = new Set(files);
  let sweepInvoked = false;
  for (let index = 0; index < commands.length; index += 1) {
    const entry = commands[index];
    if (entry.malformed) continue;
    const direct = directExecutable(entry.tokens);
    if (direct.malformed) continue;
    const tokens = direct.tokens;
    if (tokens[0] === "npm") {
      const explicit = ["run", "run-script"].includes(tokens[1]);
      const name = explicit ? tokens[2] : ["test", "start", "stop", "restart"].includes(tokens[1]) ? tokens[1] : null;
      if (name === null) continue; // npm ci/install/audit are not script invocations.
      if (!name || !Object.hasOwn(scripts, name)) {
        failures.push(`unresolvable npm script invoked by workflow surface: ${name ?? "(missing name)"}`);
      } else if (!invoked.has(name)) {
        invoked.add(name);
        commands.push(...shellCommands(scripts[name]));
      }
    } else if (/(?:^|\/)node(?:\.exe)?$/.test(tokens[0] ?? "")) {
      const paths = nodeFiles(tokens);
      for (const file of paths) reached.add(file);
      if (paths[0] === "tools/ci/gate-sweep.mjs" && !sweepInvoked) {
        sweepInvoked = true;
        commands.push(...shellCommands(sweepReachableText(root)));
      }
    } else {
      const script = ["sh", "bash", "/bin/sh", "/bin/bash"].includes(tokens[0]) ? tokens[1] : tokens[0];
      if (script === "scripts/run-native-hr-policy-parser-contract.sh") {
        for (const file of nativeHrChild(tracked)) reached.add(file);
      }
    }
  }

  const wired = [];
  const basenameOnly = [];
  const dark = [];
  for (const suite of suites) {
    const base = suite.split("/").pop();
    const exact = reached.has(suite);
    const byBase = !exact && reached.has(base);
    if (exact) wired.push(suite);
    else if (byBase) {
      basenameOnly.push(suite);
      dark.push(suite); // basename is not coverage
    } else dark.push(suite);
  }

  if (!existsSync(join(root, baselinePath))) {
    failures.push(`${baselinePath}: missing (must ship with deferred buckets)`);
  } else {
    const doc = loadJson(baselinePath);
    if (doc.schema_version !== 1) {
      failures.push(`${baselinePath}: schema_version must be 1`);
    }
    const buckets = Object.entries(doc).filter(([k]) => k.startsWith("deferred_"));
    if (buckets.length === 0) {
      failures.push(`${baselinePath}: names no deferred_* buckets`);
    }
    const accepted = new Set();
    for (const [key, value] of buckets) {
      if (!Array.isArray(value) || !value.every((e) => typeof e === "string")) {
        failures.push(`${baselinePath}: malformed bucket ${key}`);
        continue;
      }
      for (const entry of value) accepted.add(entry);
    }
    // growth: dark not in accepted
    for (const suite of dark) {
      if (!accepted.has(suite)) {
        failures.push(
          `unregistered dark .test.mjs suite (not reached by exact path from CI executable surface): ${suite}`,
        );
      }
    }
    // decoration: accepted path not present on candidate
    for (const entry of accepted) {
      if (!suites.includes(entry)) {
        failures.push(
          `${baselinePath}: decoration — deferred suite not on candidate: ${entry}`,
        );
      }
    }
    // regression: suite was deferred but is now exact-wired — must leave baseline
    for (const suite of wired) {
      if (accepted.has(suite)) {
        failures.push(
          `${baselinePath}: stale deferred entry still listed but now exact-wired: ${suite}`,
        );
      }
    }
  }

  const report = {
    schema_version: 1,
    relation: "js_test_reachability",
    suite_count: suites.length,
    wired_exact: wired,
    basename_only_diagnostic: basenameOnly,
    dark,
    invoked_npm_scripts: [...invoked].sort(),
    failures,
  };

  if (jsonMode) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  } else {
    console.log(
      `js-test-reachability: ${suites.length} suites; exact-wired ${wired.length}; dark ${dark.length}; basename-only diagnostic ${basenameOnly.length}`,
    );
    for (const s of basenameOnly) {
      console.error(`diagnostic: basename-only hit (not coverage): ${s}`);
    }
    if (failures.length) {
      console.error(`js-test-reachability failed (${failures.length}):`);
      for (const f of failures) console.error(`- ${f}`);
    } else {
      console.log("js-test-reachability check passed");
    }
  }

  process.exit(failures.length ? 1 : 0);
}

main();
