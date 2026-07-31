#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const dotSlashBootstrap = "tools/buck/install_dotslash.sh";
const reindeerToolchainLock = "third-party/rust/reindeer/upstream.lock";
const reindeerToolchainSource = `source ${reindeerToolchainLock}`;
const reindeerToolchainInstall = 'rustup toolchain install "$REINDEER_TOOLCHAIN" --profile minimal';
const strictShellMode = "set -euo pipefail";
const reindeerToolchainOverride = /^(?:export\s+)?REINDEER_TOOLCHAIN\s*=/;
const ciPreflightTestCommand = "node --test scripts/check-ci-preflight.test.mjs";
const consoleRouteInventoryTestCommand = "node --test scripts/console/route-inventory.test.mjs";
const consoleTruthLedgerCommand = "npm run check:console-truth-ledger";
const consoleAuthorityTrainTestCommand = "node --test scripts/console/verify-console-authority-train.test.mjs";
const consoleTruthLedgerTestCommand = "node --test scripts/console/validate-console-truth-ledger.test.mjs";
const consoleFanoutPlannerTestCommand = "node --test scripts/console/plan-fanout.test.mjs";
const consoleFanoutPlannerAdmissionCommand = 'node scripts/console/plan-fanout.mjs --candidate "$CONSOLE_CANDIDATE_SHA" --authority-tip "$CONSOLE_AUTHORITY_TIP_SHA" --synthetic-merge "$CONSOLE_SYNTHETIC_MERGE_SHA"';
const consolePrCondition = "${{ github.event_name == 'pull_request' }}";
const consoleTrainDerivation = [
  "set -euo pipefail",
  'CONSOLE_SYNTHETIC_MERGE_SHA="$(git rev-parse "$GITHUB_SHA^{commit}")"',
  'test "$(git rev-parse HEAD)" = "$CONSOLE_SYNTHETIC_MERGE_SHA"',
  'CONSOLE_AUTHORITY_TIP_SHA="$(git rev-parse "$CONSOLE_SYNTHETIC_MERGE_SHA^2")"',
  'CONSOLE_CANDIDATE_SHA="$(git rev-parse "$CONSOLE_AUTHORITY_TIP_SHA^")"',
  "{",
  "printf 'CONSOLE_CANDIDATE_SHA=%s\\n' \"$CONSOLE_CANDIDATE_SHA\"",
  "printf 'CONSOLE_AUTHORITY_TIP_SHA=%s\\n' \"$CONSOLE_AUTHORITY_TIP_SHA\"",
  "printf 'CONSOLE_SYNTHETIC_MERGE_SHA=%s\\n' \"$CONSOLE_SYNTHETIC_MERGE_SHA\"",
  '} >> "$GITHUB_ENV"',
];
const buckPostgresEnvironmentTestCommand = "tools/buck/run_test_with_postgres_env.test.sh";
const buckPostgresHarnessTestCommand = "tools/buck/test_needs_postgres.test.sh";
// The domain-unit step is now a multi-line run. What must be asserted is not the exact
// text but that every audit-relevant package added on 2026-07-31 is still named — dropping
// one silently returns its tests to executing nowhere, which is the condition
// check:executed-tests exists to measure and this assertion exists to prevent.
const domainUnitPackages = [
  "console-support-domain",
  "console-payroll-domain",
  "console-payroll-adapter-postgres",
  "console-attendance-application",
  "console-compliance-domain",
  "console-governance-domain",
  "console-platform-audit-chain",
  "console-platform-authz",
  "console-policy-application",
  "console-policy-domain",
  // Added 2026-07-31. Every crate below held test code that executed in no workflow step.
  // They are domain and application crates — no database, no fixture, no wrapper target —
  // so the only thing that had ever kept them dark was that nobody named them here.
  "console-action-inbox-application",
  "console-attendance-domain",
  "console-benefit-application",
  "console-benefit-domain",
  "console-comms-application",
  "console-comms-domain",
  "console-dispatch-application",
  "console-dispatch-domain",
  "console-docs-application",
  "console-docs-domain",
  "console-equipment-domain",
  "console-erp-domain",
  "console-evaluation-application",
  "console-evaluation-domain",
  "console-finance-gl-domain",
  "console-identity-domain",
  "console-inbox-application",
  "console-inbox-domain",
  "console-inventory-domain",
  "console-leave-domain",
  "console-logistics-domain",
  "console-messenger-application",
  "console-notices-domain",
  "console-notifications-domain",
  "console-ontology-application",
  "console-ontology-domain",
  "console-orgchange-domain",
  "console-recruiting-application",
  "console-recruiting-domain",
  "console-reporting-application",
  "console-reporting-domain",
  "console-sales-domain",
  "console-support-application",
  "console-todos-domain",
  "console-workflow-domain",
  "console-workorder-application",
  // Second tranche, same day. These are `--lib` unit tests in `rest`, `adapter-postgres`
  // and CI-gate crates. FOUR sibling crates were excluded because their lib carries a
  // `#[sqlx::test]` and needs DATABASE_URL — console-platform-group, console-platform-storage,
  // console-gate-rls-arming, console-support-rest. "--lib means no database" is false here,
  // and the only reason that is known is that running them produced
  // "DATABASE_URL must be set" rather than a green summary.
  "console-analytics-quant-service",
  "console-attendance-adapter-postgres",
  "console-attendance-rest",
  "console-benefit-rest",
  "console-comms-adapter-imap",
  "console-comms-adapter-mox",
  "console-comms-adapter-smtp",
  "console-comms-credential-cipher",
  "console-comms-mailbox",
  "console-comms-rest",
  "console-compliance-rest",
  "console-consulting-rest",
  "console-dispatch-rest",
  "console-docs-adapter-postgres",
  "console-docs-rest",
  "console-evaluation-adapter-postgres",
  "console-facilities-rest",
  "console-financial-rest",
  "console-gate-dev-auth-absence",
  "console-gate-iac-tier",
  "console-gate-layer-boundary",
  "console-gate-tenant-isolation",
  "console-gate-vendor-lockin",
  "console-identity-rest",
  "console-inventory-adapter-postgres",
  "console-inventory-rest",
  "console-leave-adapter-postgres",
  "console-leave-rest",
  "console-ontology-adapter-postgres",
  "console-ontology-rest",
  "console-payroll-rest",
  "console-platform-email",
  "console-platform-jobs",
  "console-platform-push",
  "console-platform-realtime",
  "console-platform-request-context",
  "console-production-rest",
  "console-reporting-adapter-postgres",
  "console-support-adapter-postgres",
  "console-workflow-runtime-adapter-postgres",
  "console-workorder-rest",
];
const domainUnitTestFiles = [
  "attendance_policy",
  "location_consent_fsm",
  "location_ping_policy",
  "cedar_pbac_readiness_cases",
  "cedar_pbac_legacy_only_observe_and_record",
  // `--lib` does not reach an integration test under tests/, so each of these has to be
  // named. A crate appearing in domainUnitPackages above does NOT imply its tests/ files run.
  "range_and_history",
  "quote_and_residual",
  "equipment",
  "mentions",
  "object_code_refs",
  "parity",
  "thread_kind",
  "approval_and_assignment",
  "serde_roundtrips",
  "settlement_fsm",
  "workorder_fsm",
  // Third tranche, 2026-07-31. Integration tests under tests/ that needed no database —
  // confirmed by running them, not by reading their imports.
  "jwt_es256",
  "jwt_verifier",
  "template_fidelity",
  "template_fill_engine",
  "hub",
  "notify_payload",
  "seaweedfs_worm",
  "config",
  "dev_seed_notification_links",
  "openslo_files",
  "workbench_api",
  // `well_known` exists in BOTH console-platform-auth and console-app. This list is bare test
  // names, so one entry covers both and cannot distinguish them — which is exactly why ci.yml
  // gives each package its own cargo invocation. Deleting either invocation still leaves this
  // entry satisfied by the other, so the guard here is weaker than it looks for this one name.
  "well_known",
];
const postgresDomainReachabilityCommands = [
  "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
  "//tools/buck:dispatch-p1-postgres \\",
  "//tools/buck:attendance-cancel-substitution-postgres \\",
  "//tools/buck:attendance-concurrency-postgres \\",
  "//tools/buck:ontology-object-type-lifecycle-postgres \\",
  "//tools/buck:ontology-object-type-cas-postgres \\",
  "//tools/buck:ontology-publish-auto-create-action-postgres \\",
  "//tools/buck:ontology-object-policy-attach-postgres \\",
  "//tools/buck:ontology-action-execute-postgres \\",
  "//tools/buck:ontology-gaps-postgres \\",
  "//tools/buck:ontology-projected-dispatch-postgres \\",
  "//tools/buck:platform-erasure-ledger-postgres",
];
const postgresWrapperContracts = [
  ["dispatch-p1-postgres", "//backend/crates/dispatch/adapter-postgres:console-dispatch-adapter-postgres-itest-p1_dispatch"],
  ["attendance-cancel-substitution-postgres", "//backend/crates/attendance/adapter-postgres:console-attendance-adapter-postgres-itest-cancel_substitution"],
  ["attendance-concurrency-postgres", "//backend/crates/attendance/adapter-postgres:console-attendance-adapter-postgres-itest-concurrency"],
  ["app-inline-postgres", "//backend/app:console-app-itest-inline-postgres"],
  ["app-dev-auth-persona-guard-postgres", "//backend/app:console-app-itest-dev_auth_persona_guard_feature"],
  ["auth-rest-dev-auth-inline-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-dev-auth-postgres"],
  ["auth-rest-dev-auth-session-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-dev_auth_session"],
  ["auth-rest-dev-auth-group-admin-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-group_admin_tenant_context"],
  ["provisioning-dev-principal-upsert-race-postgres", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-dev_principal_upsert_race"],
  ["app-evaluation-cycle-api-postgres", "//backend/app:console-app-itest-evaluation_cycle_api"],
  ["ontology-builtin-catalog-additive-upgrade-postgres", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-builtin_catalog_additive_upgrade_as_runtime_role"],
  ["app-org-change-api-postgres", "//backend/app:console-app-itest-org_change_api"],
  ["app-purchase-request-collection-api-postgres", "//backend/app:console-app-itest-purchase_request_collection_api"],
  ["app-workflow-object-context-api-postgres", "//backend/app:console-app-itest-workflow_object_context_api"],
  ["equipment-3r-http-postgres", "//backend/crates/equipment/rest:console-equipment-rest-itest-equipment_3r_http"],
  ["app-equipment-3r-api-postgres", "//backend/app:console-app-itest-equipment_3r_api"],
  ["ontology-object-type-lifecycle-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_type_lifecycle_over_http"],
  ["ontology-object-type-cas-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_type_cas_as_runtime_role"],
  ["ontology-publish-auto-create-action-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-publish_auto_create_action_as_runtime_role"],
  ["ontology-object-policy-attach-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_policy_attach_as_runtime_role"],
  ["ontology-action-execute-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-action_execute_as_runtime_role"],
  ["ontology-gaps-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-ont_gaps_as_runtime_role"],
  ["ontology-projected-dispatch-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-projected_dispatch_as_runtime_role"],
  ["platform-erasure-ledger-postgres", "//backend/crates/platform/erasure-ledger:console-platform-erasure-ledger-itest-erasure_ledger_as_runtime_role"],
];
const postgresWrapperLoader = "run_test_with_postgres_env.sh";
const postgresWrapperLabels = '["test.integration", "resource.postgres", "needs-postgres"]';
const postgresWrapperBuildFile = readFileSync(new URL("../tools/buck/BUCK", import.meta.url), "utf8");
// GENERATED by tools/buck/gen_first_party.py from the crate's tests/ directory,
// which is what makes the check below a TOTALITY check rather than a second
// hand-kept list: adding a test file adds a target here without anyone deciding to.
const ontologyRestCrateBuildFile = readFileSync(
  new URL("../backend/crates/ontology/rest/BUCK", import.meta.url),
  "utf8",
);
const requiredPreflightCommands = [
  "tools/buck/preflight.sh",
  "npm run check:foundation-gates",
  ciPreflightTestCommand,
  consoleRouteInventoryTestCommand,
  buckPostgresEnvironmentTestCommand,
  buckPostgresHarnessTestCommand,
  "npm run check:ci-preflight",
  "npm run check:package-lock",
  "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null",
];
const protectedJobs = [
  "backend",
  "dev-up-smoke",
  "repo-gates",
  "api-contract",
  "kubernetes-manifests",
  "generated-face-authority",
  "domain-unit",
  "postgres-domain-reachability",
];

function triggerPathEntries(workflow, trigger) {
  const match = workflow.match(new RegExp(`^  ${trigger}:\\n([\\s\\S]*?)(?=^  [A-Za-z0-9_-]+:|^permissions:)`, "m"));
  const paths = match?.[1].match(/^    paths:\n((?:      - "[^"]+"\n)+)/m);
  return paths ? [...paths[1].matchAll(/^      - "([^"]+)"$/gm)].map((entry) => entry[1]) : [];
}

function jobBlock(workflow, job) {
  const jobs = workflow.slice(workflow.indexOf("jobs:\n") + "jobs:\n".length);
  const match = jobs.match(new RegExp(`^  ${job}:\\n([\\s\\S]*?)(?=^  [A-Za-z0-9_-]+:|(?![\\s\\S]))`, "m"));
  return match?.[1] ?? null;
}

function needsPreflight(block) {
  const value = block.match(/^    needs:\s*(.+)$/m)?.[1]?.trim();
  if (!value) return false;
  if (value.startsWith("[") && value.endsWith("]")) {
    return value.slice(1, -1).split(",").map((job) => job.trim()).includes("preflight");
  }
  return value === "preflight";
}

function stepBlocks(block) {
  const steps = block.match(/^    steps:\n([\s\S]*)$/m)?.[1] ?? "";
  return steps.split(/^      - /m).slice(1);
}

function runScalar(step) {
  return step.match(/^        run: ([^\n]+)$/m)?.[1]?.trim();
}

function isUnconditional(step) {
  return !/^        (?:if|continue-on-error):/m.test(step);
}

function multilineRunCommands(step) {
  const run = step.match(/^        run: \|\n((?:          [^\n]*(?:\n|$))*)/m)?.[1] ?? "";
  return run
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function runScript(step) {
  const scalar = runScalar(step);
  if (scalar && scalar !== "|") return scalar;
  const multiline = step.match(/^        run: \|\n((?:          [^\n]*(?:\n|$))*)/m)?.[1];
  return multiline?.replace(/^          /gm, "") ?? "";
}

function stripShellComment(line) {
  let quote = null;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (quote) {
      if (character === quote) quote = null;
    } else if (character === "'" || character === '"') {
      quote = character;
    } else if (character === "#" && (index === 0 || /\s/.test(line[index - 1]))) {
      return line.slice(0, index);
    }
  }
  return line;
}

function shellCommandTokens(script) {
  const commands = [];
  let command = "";
  for (const physicalLine of script.split(/\r?\n/)) {
    const line = stripShellComment(physicalLine.trim());
    if (!line) continue;
    const continued = /\\\s*$/.test(line);
    command += (command ? " " : "") + (continued ? line.replace(/\\\s*$/, "") : line);
    if (!continued) {
      commands.push(command);
      command = "";
    }
  }
  if (command) commands.push(command + "\\");

  return commands.map((surface) => {
    const tokens = [];
    let token = "";
    let quote = null;
    let escaped = false;
    const flush = () => {
      if (token) tokens.push(token);
      token = "";
    };
    for (let index = 0; index < surface.length; index += 1) {
      const character = surface[index];
      if (escaped) {
        token += character;
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (quote) {
        if (character === quote) quote = null;
        else token += character;
      } else if (character === "'" || character === '"') {
        quote = character;
      } else if (/\s/.test(character)) {
        flush();
      } else if (";|&".includes(character)) {
        flush();
        tokens.push(character);
      } else {
        token += character;
      }
    }
    const malformed = quote !== null || escaped;
    if (escaped) token += "\\";
    flush();
    return { tokens, malformed };
  });
}

const maxExecutablePrefixDepth = 12;
const maxExecutableTokens = 512;
const assignmentToken = /^[A-Za-z_][A-Za-z0-9_]*=/;

function mergeCargoAnalysis(left, right) {
  for (const packageName of right.packages) left.packages.add(packageName);
  left.malformed ||= right.malformed;
  return left;
}

function emptyCargoAnalysis(malformed = false) {
  return { packages: new Set(), malformed };
}

function packageArguments(tokens, index) {
  const packages = new Set();
  for (; index < tokens.length; index += 1) {
    const argument = tokens[index];
    let packageName = null;
    if (argument === "-p" || argument === "--package") {
      packageName = tokens[index + 1];
      index += 1;
    } else if (argument.startsWith("-p=")) {
      packageName = argument.slice(3);
    } else if (argument.startsWith("--package=")) {
      packageName = argument.slice("--package=".length);
    }
    if (!packageName) continue;
    packages.add(packageName);
  }
  return { packages, malformed: false };
}

function cargoTestAnalysis(tokens, depth = 0) {
  if (depth > maxExecutablePrefixDepth || tokens.length > maxExecutableTokens) {
    return emptyCargoAnalysis(true);
  }
  let index = 0;
  while (assignmentToken.test(tokens[index] ?? "")) index += 1;
  if (index === tokens.length) return emptyCargoAnalysis();

  if (tokens[index] === "command") {
    index += 1;
    while (tokens[index]?.startsWith("-")) {
      const option = tokens[index];
      if (option === "--") {
        index += 1;
        break;
      }
      if (option === "-v" || option === "-V") return emptyCargoAnalysis();
      if (option === "-p") {
        index += 1;
        continue;
      }
      return emptyCargoAnalysis(true);
    }
    return index < tokens.length
      ? cargoTestAnalysis(tokens.slice(index), depth + 1)
      : emptyCargoAnalysis(true);
  }

  if (tokens[index] === "env") {
    index += 1;
    while (tokens[index]) {
      const option = tokens[index];
      if (option === "--") {
        index += 1;
        break;
      }
      if (option === "-S" || option === "--split-string") {
        const payload = tokens[index + 1];
        if (!payload || index + 2 !== tokens.length) return emptyCargoAnalysis(true);
        const payloads = shellCommandTokens(payload);
        if (payloads.length === 0) return emptyCargoAnalysis(true);
        return payloads.reduce(
          (analysis, surface) => mergeCargoAnalysis(
            analysis,
            surface.malformed
              ? emptyCargoAnalysis(true)
              : cargoTestAnalysis(surface.tokens, depth + 1),
          ),
          emptyCargoAnalysis(),
        );
      }
      if (["-u", "--unset", "-C", "--chdir"].includes(option)) {
        if (!tokens[index + 1]) return emptyCargoAnalysis(true);
        index += 2;
      } else if (["-i", "--ignore-environment", "-v", "--debug"].includes(option)) {
        index += 1;
      } else if (assignmentToken.test(option)) {
        index += 1;
      } else if (option.startsWith("-")) {
        return emptyCargoAnalysis(true);
      } else {
        break;
      }
    }
    return index < tokens.length
      ? cargoTestAnalysis(tokens.slice(index), depth + 1)
      : emptyCargoAnalysis(true);
  }

  if (tokens[index] !== "cargo") return emptyCargoAnalysis();
  index += 1;
  if (tokens[index] === "test") return packageArguments(tokens, index + 1);
  if (tokens[index] === "nextest" && tokens[index + 1] === "run") {
    return packageArguments(tokens, index + 2);
  }
  return emptyCargoAnalysis();
}

function cargoTestAnalysisInStep(step) {
  const analysis = emptyCargoAnalysis();
  for (const surface of shellCommandTokens(runScript(step))) {
    if (surface.malformed) {
      mergeCargoAnalysis(analysis, emptyCargoAnalysis(true));
      continue;
    }
    let segment = [];
    for (const token of [...surface.tokens, ";"]) {
      if (token === ";" || token === "|" || token === "&") {
        mergeCargoAnalysis(analysis, cargoTestAnalysis(segment));
        segment = [];
      } else {
        segment.push(token);
      }
    }
  }
  return analysis;
}

function requireUnconditionalRun(steps, command, job, failures) {
  const matchingSteps = steps.filter((step) => runScalar(step) === command);
  if (matchingSteps.length === 0) {
    failures.push(`${job} must run ${command}`);
  } else if (matchingSteps.some((step) => !isUnconditional(step))) {
    failures.push(`${job} must run ${command} unconditionally without if or continue-on-error`);
  }
}

function requireConsoleExactMergeProof(workflow, steps, failures) {
  const derive = steps.filter((step) => stepName(step) === "Derive exact console C/T/M train");
  if (derive.length !== 1 || !hasOnlyExpectedCondition(derive[0], consolePrCondition) || multilineRunCommands(derive[0]).join("\n") !== consoleTrainDerivation.join("\n")) {
    failures.push("preflight must derive exact C/T/M from the pull-request synthetic merge");
  }
  for (const command of [consoleTruthLedgerCommand, consoleFanoutPlannerAdmissionCommand]) {
    const matching = steps.filter((step) => runScalar(step) === command);
    if (matching.length !== 1 || !hasOnlyExpectedCondition(matching[0], consolePrCondition)) failures.push(`preflight must run ${command} only after exact C/T/M derivation on pull requests`);
  }
  for (const command of [consoleAuthorityTrainTestCommand, consoleTruthLedgerTestCommand, consoleFanoutPlannerTestCommand]) {
    const matching = steps.filter((step) => runScalar(step) === command);
    if (matching.length !== 1 || !isUnconditional(matching[0])) failures.push(`preflight must run ${command} unconditionally on pull requests and main`);
  }
  if (workflow.includes("CONSOLE_INTEGRATION_TIP_SHA")) failures.push("preflight must not reference legacy CONSOLE_INTEGRATION_TIP_SHA");
}

function requirePostgresWrapperContracts(buildFile, failures) {
  for (const [name, binary] of postgresWrapperContracts) {
    const block = buildFile.match(new RegExp(`sh_test\\(\\n    name = "${name}",[\\s\\S]*?\\n\\)`, "m"))?.[0];
    if (!block) {
      failures.push(`tools/buck/BUCK must define PostgreSQL wrapper ${name}`);
      continue;
    }
    const expectedArgs = `args = ["$(location ${binary})"],`;
    const expectedDeps = `deps = ["${binary}"],`;
    const hasExactAttribute = (attribute) => (
      [...block.matchAll(new RegExp(`^    ${attribute.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`, "gm"))].length === 1
    );
    const hasExactlyOneField = (field) => (
      [...block.matchAll(new RegExp(`^    ${field} = .+$`, "gm"))].length === 1
    );
    if (!hasExactAttribute(`test = "${postgresWrapperLoader}",`)
      || !hasExactAttribute(expectedArgs)
      || !hasExactAttribute(expectedDeps)
      || !hasExactAttribute(`labels = ${postgresWrapperLabels},`)
      || !["test", "args", "deps", "labels"].every(hasExactlyOneField)) {
      failures.push(`tools/buck/BUCK must bind PostgreSQL wrapper ${name} to the loader and exact Rust binary`);
    }
  }
}

/**
 * TOTALITY over the crate that decides row visibility: every integration test it
 * has must be wrapped in exactly one PostgreSQL `sh_test` and that wrapper must
 * be named in ci.yml.
 *
 * The lists above are per-target contracts — they check that the entries someone
 * REMEMBERED to add are wired correctly, and say nothing about the ones nobody
 * added. That is the four-coupled-places defect this repo has shipped seven
 * times, most recently to `action_execute`, `ont_gaps` and `projected_dispatch`,
 * all three of which had a `rust_test` target and appeared in no wrapper, no
 * ci.yml step and neither locked list. Two of them were broken for the whole
 * lane that introduced the break, and CI could not see it.
 *
 * Deliberately scoped to this ONE crate and driven off its GENERATED BUCK file
 * rather than a repo-wide sweep. The repo-wide version is not blocked by a lack
 * of signal — `TEST_RESOURCE_REQUIREMENTS` in tools/buck/gen_first_party.py
 * already declares `postgres` per test file, and gen_first_party FAILS on any
 * test file that has no declaration. It is blocked by scale: of the 188
 * postgres-declared integration tests in this repo, 163 have no `sh_test`
 * wrapper (measured 2026-07-29). Turning that into a gate today makes CI
 * permanently red, and a gate nobody can satisfy gets deleted. Widening this is
 * a per-crate decision with the same shape as this one, not a cleverer regex.
 */
function requireOntologyRestItestReachability(buildFile, workflow, failures) {
  const itests = [
    ...ontologyRestCrateBuildFile.matchAll(/^    name = "(console-ontology-rest-itest-[a-z0-9_]+)",$/gm),
  ].map(([, name]) => name);
  if (itests.length === 0) {
    failures.push("backend/crates/ontology/rest/BUCK must declare integration-test targets");
    return;
  }
  const wrappers = [...buildFile.matchAll(/sh_test\(\n([\s\S]*?)\n\)/g)].map(([, body]) => ({
    name: body.match(/^    name = "([^"]+)",$/m)?.[1],
    target: body.match(/^    args = \["\$\(location ([^)]+)\)"\],$/m)?.[1],
  }));
  for (const itest of itests) {
    const target = `//backend/crates/ontology/rest:${itest}`;
    const matching = wrappers.filter((wrapper) => wrapper.target === target);
    if (matching.length !== 1 || !matching[0].name) {
      failures.push(`tools/buck/BUCK must wrap ${itest} in exactly one PostgreSQL sh_test`);
      continue;
    }
    if (!new RegExp(`^\\s+//tools/buck:${matching[0].name}( \\\\)?$`, "m").test(workflow)) {
      failures.push(`ci.yml must execute //tools/buck:${matching[0].name} or ${itest} runs nowhere`);
    }
  }
}

function requireUnconditionalMultilineRun(steps, commands, job, failures) {
  const matchingSteps = steps.filter((step) => multilineRunCommands(step).join("\n") === commands.join("\n"));
  if (matchingSteps.length === 0) {
    failures.push(`${job} must run the locked PostgreSQL reachability targets`);
  } else if (matchingSteps.some((step) => !isUnconditional(step))) {
    failures.push(`${job} must run the locked PostgreSQL reachability targets unconditionally without if or continue-on-error`);
  }
}

function runCommand(step) {
  const scalar = runScalar(step);
  return scalar && scalar !== "|" ? scalar : (multilineRunCommands(step).join("\n") || null);
}

function requireOnlyLockedRuns(steps, commands, job, failures) {
  const actual = steps.map(runCommand).filter(Boolean);
  if (actual.length !== commands.length || actual.some((command, index) => command !== commands[index])) {
    failures.push(`${job} must contain only the locked ordered run steps`);
  }
}

function stepName(step) {
  return step.match(/^name: ([^\n]+)$/m)?.[1] ?? null;
}

function stepWorkingDirectory(step) {
  return step.match(/^        working-directory: ([^\n]+)$/m)?.[1]?.trim() ?? null;
}

function hasOnlyExpectedCondition(step, expectedIf) {
  const conditions = [...step.matchAll(/^        if: ([^\n]+)$/gm)].map((match) => match[1]);
  return (expectedIf === null
    ? conditions.length === 0
    : conditions.length === 1 && conditions[0] === expectedIf)
    && !/^        continue-on-error:/m.test(step);
}

function requireOrderedStepContracts(steps, contracts, job, failures) {
  const indexes = [];
  for (const contract of contracts) {
    const matches = steps
      .map((step, index) => ({ step, index }))
      .filter(({ step }) => stepName(step) === contract.name);
    if (matches.length !== 1
      || (contract.run !== undefined && runCommand(matches[0]?.step) !== contract.run)
      || (contract.workingDirectory !== undefined
        && stepWorkingDirectory(matches[0]?.step ?? "") !== contract.workingDirectory)
      || !hasOnlyExpectedCondition(matches[0]?.step ?? "", contract.if)) {
      failures.push(`${job} must preserve the locked fail-fast step multiset and failure semantics`);
      indexes.push(-1);
    } else {
      indexes.push(matches[0].index);
    }
  }
  if (indexes.some((index) => index < 0) || indexes.some((index, position) => position > 0 && index <= indexes[position - 1])) {
    failures.push(`${job} must preserve the locked fail-fast step order`);
  }
  return indexes;
}

const apiContractCaptureName = "Capture Buck2-built app for contract test";
const apiContractCaptureCommands = [
  "set -euo pipefail",
  'console_app_bin="${GITHUB_WORKSPACE}/.tmp/buck2/api-contract/console-app"',
  'test -x "${console_app_bin}"',
  "printf 'CONSOLE_APP_BIN=%s\\n' \"${console_app_bin}\" >> \"${GITHUB_ENV}\"",
];
// The Swatinem/rust-cache step was REMOVED from this list on 2026-07-31. api-contract
// builds //backend/app:console-app with Buck2 — see scripts/check-openapi-app.mjs, which
// spawns a Buck2-built binary from .tmp/buck2/api-contract/console-app — and Buck2 never
// writes backend/target. The job therefore restored and saved a cache it could not use,
// paying transfer cost on every run and occupying an LRU slot against a 10GB repository
// budget shared with the two jobs that DO run cargo.
const apiContractAllowedSteps = [
  "name: Checkout\n        uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7",
  "name: Install pinned DotSlash runtime\n        run: tools/buck/install_dotslash.sh",
  "name: Free runner disk for API contract\n        uses: ./.github/actions/free-runner-disk",
  "name: Install Rust toolchain (pinned via rust-toolchain.toml)\n        uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable\n        with:\n          toolchain: \"1.97.1\"",
  "name: Set up Node.js\n        uses: actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e # v6.4.0\n        with:\n          node-version: \"24\"\n          cache: npm",
  "name: Install client tooling\n        run: npm ci",
  "name: OpenAPI app-served drift gate\n        if: ${{ !cancelled() }}\n        run: npm run check:openapi-app",
  `name: ${apiContractCaptureName}
        if: \${{ !cancelled() }}
        shell: bash
        run: |
          set -euo pipefail
          # check:openapi-app is the sole Buck2 producer for this handoff.
          console_app_bin="\${GITHUB_WORKSPACE}/.tmp/buck2/api-contract/console-app"
          test -x "\${console_app_bin}"
          printf 'CONSOLE_APP_BIN=%s\\n' "\${console_app_bin}" >> "\${GITHUB_ENV}"`,
  "name: Employee import replay contract\n        if: ${{ !cancelled() }}\n        run: npm run test:employee-import-contract",
  "name: Ontology write precondition contract\n        if: ${{ !cancelled() }}\n        run: npm run test:ontology-write-precondition",
];

function hasOnlyAllowedApiContractSteps(steps) {
  return steps.length === apiContractAllowedSteps.length
    && steps.every((step, index) => step.trimEnd() === apiContractAllowedSteps[index]);
}

function isDesignatedApiContractCapture(step) {
  if (!step.startsWith(`name: ${apiContractCaptureName}\n`)) return false;
  return multilineRunCommands(step).filter((line) => !line.startsWith("#")).join("\n")
    === apiContractCaptureCommands.join("\n");
}

function requireReindeerToolchainBefore(steps, command, failures) {
  const commandIndex = steps.findIndex((step) => runScalar(step) === command);
  const toolchainIndex = steps.findIndex((step) => multilineRunCommands(step).includes(reindeerToolchainInstall));
  if (toolchainIndex < 0) {
    failures.push("generated-face-authority must install the lock-pinned Reindeer Rust toolchain before full generated-face closure");
    return;
  }
  const commands = multilineRunCommands(steps[toolchainIndex]);
  const strictModeIndex = commands.indexOf(strictShellMode);
  const sourceIndex = commands.indexOf(reindeerToolchainSource);
  const installIndex = commands.indexOf(reindeerToolchainInstall);
  if (sourceIndex < 0) {
    failures.push(`generated-face-authority must source ${reindeerToolchainLock} before installing the Reindeer Rust toolchain`);
  } else if (strictModeIndex < 0 || strictModeIndex > sourceIndex) {
    failures.push(`generated-face-authority must enable strict shell mode before sourcing ${reindeerToolchainLock}`);
  } else if (sourceIndex > installIndex) {
    failures.push(`generated-face-authority must source ${reindeerToolchainLock} before installing the Reindeer Rust toolchain`);
  }
  if (commands.some((entry) => reindeerToolchainOverride.test(entry))) {
    failures.push(`generated-face-authority must not override REINDEER_TOOLCHAIN after sourcing ${reindeerToolchainLock}`);
  }
  if (!isUnconditional(steps[toolchainIndex])) {
    failures.push("generated-face-authority must install the Reindeer Rust toolchain unconditionally");
  }
  if (commandIndex >= 0 && toolchainIndex > commandIndex) {
    failures.push("generated-face-authority must install the lock-pinned Reindeer Rust toolchain before full generated-face closure");
  }
}

function requirePreflightRustToolchainBefore(steps, failures) {
  const setupIndex = steps.findIndex((step) =>
    step.startsWith("name: Install Rust toolchain for Cargo.lock consistency\n"),
  );
  if (setupIndex < 0) {
    failures.push("preflight must install the pinned Rust toolchain before Cargo-dependent CI preflight tests");
    return;
  }
  if (!isUnconditional(steps[setupIndex])) {
    failures.push("preflight must install the pinned Rust toolchain unconditionally");
    return;
  }
  for (const command of [
    ciPreflightTestCommand,
    "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null",
  ]) {
    const commandIndex = steps.findIndex((step) => runScalar(step) === command);
    if (commandIndex >= 0 && setupIndex > commandIndex) {
      failures.push(`preflight must install the pinned Rust toolchain before ${command}`);
    }
  }
}

function requireDotSlashBefore(steps, command, job, failures) {
  const commandIndex = steps.findIndex((step) => runScalar(step) === command);
  const dotSlashIndex = steps.findIndex((step) => runScalar(step) === dotSlashBootstrap);
  if (dotSlashIndex < 0) {
    failures.push(`${job} must install pinned DotSlash before Buck2`);
  } else if (!isUnconditional(steps[dotSlashIndex])) {
    failures.push(`${job} must install DotSlash unconditionally`);
  } else if (commandIndex >= 0 && dotSlashIndex > commandIndex) {
    failures.push(`${job} must install DotSlash before ${command}`);
  }
}

function requireEffectiveDotSlashBootstrap(block, job, failures) {
  const workingDirectory = block.match(/^    defaults:\n      run:\n        working-directory: ([^\n]+)$/m)?.[1]?.trim();
  const bootstrap = workingDirectory === "backend" ? `../${dotSlashBootstrap}` : dotSlashBootstrap;
  const steps = stepBlocks(block);
  const bootstrapIndex = steps.findIndex((step) => runScalar(step) === bootstrap);
  if (bootstrapIndex < 0) {
    failures.push(`${job} must install pinned DotSlash from ${bootstrap}`);
    return;
  }
  const firstBuckInvocation = steps.findIndex((step, index) => {
    const run = runScalar(step);
    const command = run === "|" ? multilineRunCommands(step).join("\n") : run ?? "";
    return index !== bootstrapIndex
      && (
        /(?:^|[^A-Za-z0-9_])tools\/buck(?:2|\/)/.test(command)
        || /\bdotslash\b/i.test(command)
        || /(?:^|[\s;&|])node\s+scripts\/dev-up\.mjs\s+bootstrap(?:\s|$)/m.test(command)
      );
  });
  if (firstBuckInvocation >= 0 && bootstrapIndex > firstBuckInvocation) {
    failures.push(`${job} must install pinned DotSlash before its first Buck invocation`);
  }
}

export function evaluateCiPreflight(workflow, buckBuildFile = postgresWrapperBuildFile) {
  const failures = [];
  for (const trigger of ["push", "pull_request"]) {
    if (!triggerPathEntries(workflow, trigger).includes("toolchains/**")) {
      failures.push(`${trigger} must include toolchains/** in CI path filters`);
    }
    if (!triggerPathEntries(workflow, trigger).includes("docs/program/**")) {
      failures.push(`${trigger} must include docs/program/** in CI path filters`);
    }
  }

  const preflight = jobBlock(workflow, "preflight");
  if (!preflight) {
    failures.push("CI must define a preflight job before expensive jobs");
    return { failures };
  }

  const preflightSteps = stepBlocks(preflight);
  if (/^    if:/m.test(preflight)) {
    failures.push("preflight must not define job-level if");
  }
  if (/^    continue-on-error:/m.test(preflight)) {
    failures.push("preflight must not define job-level continue-on-error");
  }
  const checkout = preflightSteps.find((step) => step.startsWith("name: Checkout\n"));
  if (!checkout || !/^        with:\n(?:          [^\n]+\n)*          fetch-depth: 0$/m.test(checkout)) {
    failures.push("preflight checkout must fetch full history with fetch-depth: 0");
  }
  requireDotSlashBefore(preflightSteps, "tools/buck/preflight.sh", "preflight", failures);
  requirePreflightRustToolchainBefore(preflightSteps, failures);
  for (const command of requiredPreflightCommands) {
    requireUnconditionalRun(preflightSteps, command, "preflight", failures);
  }
  requireConsoleExactMergeProof(workflow, preflightSteps, failures);
  // One job, both crates. They share console-kernel-core, so two jobs recompiled
  // the same dependencies and paid two runner startups and two cache restores.
  const domainUnit = jobBlock(workflow, "domain-unit");
  if (domainUnit) {
    const steps = stepBlocks(domainUnit);
    const block = domainUnit;
    for (const pkg of domainUnitPackages) {
      if (!block.includes(`-p ${pkg}`)) failures.push(`domain-unit must run -p ${pkg}`);
    }
    for (const t of domainUnitTestFiles) {
      if (!block.includes(`--test ${t}`)) failures.push(`domain-unit must run --test ${t}`);
    }
    // Match the INVOCATION, not the block. `/--lib/` alone is satisfied by the comment
    // above the step explaining why --lib is load-bearing — the third time in this
    // repository that prose has silently satisfied a code assertion.
    if (!/cargo test[^\n]*--lib/.test(block)) {
      failures.push("domain-unit must pass --lib on its first cargo invocation");
    }
    // The step list stays locked. Rewriting the command assertion to check packages
    // rather than an exact string dropped requireOnlyLockedRuns, which would have let
    // anyone ADD an arbitrary run step to this job unnoticed. One `run:` and no more.
    const runCount = (block.match(/^        run: /gm) || []).length;
    if (runCount !== 1) {
      failures.push(`domain-unit must contain only the locked ordered run steps; found ${runCount} run steps`);
    }
  }

  requirePostgresWrapperContracts(buckBuildFile, failures);
  requireOntologyRestItestReachability(buckBuildFile, workflow, failures);

  const postgresDomainReachability = jobBlock(workflow, "postgres-domain-reachability");
  if (postgresDomainReachability) {
    const steps = stepBlocks(postgresDomainReachability);
    requireUnconditionalMultilineRun(
      steps,
      postgresDomainReachabilityCommands,
      "postgres-domain-reachability",
      failures,
    );
    requireOnlyLockedRuns(
      steps,
      [dotSlashBootstrap, postgresDomainReachabilityCommands.join("\n")],
      "postgres-domain-reachability",
      failures,
    );
  }

  const backend = jobBlock(workflow, "backend");
  if (backend) {
    const steps = stepBlocks(backend);
    const failFastIf = null;
    const pr473ContractTestCommand = "python3 scripts/check-pr473-migration-operational.test.py -v";
    if (steps.some((step) => step.includes("if: ${{ !cancelled() }}"))) {
      failures.push("backend must not use !cancelled() on protected fail-fast steps");
    }
    const sourceGateContracts = [
      ["Layer-boundary gate", "cargo run -p console-gate-layer-boundary"],
      ["Audit-coverage gate", "cargo run -p console-gate-audit-coverage"],
      ["Migration-safety gate", "cargo run -p console-gate-migration-safety"],
      ["Tenant-isolation gate", "cargo run -p console-gate-tenant-isolation"],
      ["PII-no-logs gate", "cargo run -p console-gate-pii-no-logs"],
      ["RLS-arming gate", "cargo run -p console-gate-rls-arming"],
      ["Dev-auth-absence gate", "cargo run -p console-gate-dev-auth-absence"],
      ["IaC tier-discipline gate", "cargo run -p console-gate-iac-tier"],
    ];
    const gateIndexes = requireOrderedStepContracts(
      steps,
      [
        ["clippy -D warnings", "SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings"],
        ...sourceGateContracts,
        ["PR 473 migration operational contract tests", pr473ContractTestCommand],
        ["Reconcile portable PostgreSQL role topology", undefined],
        ["PR 473 migration operational gate", "npm run check:pr473-migration-operational"],
        ["Boot smoke — migrate + serve + /readyz", undefined],
      ].map(([name, run]) => ({ name, run, if: failFastIf })),
      "backend",
      failures,
    );
    if (gateIndexes[1] !== gateIndexes[0] + 1) {
      failures.push("backend must run source-only gates immediately after clippy");
    }
    requireOrderedStepContracts(
      steps,
      [
        {
          name: "Buck2 dev-auth feature PostgreSQL suites",
          run: [
            "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
            "//tools/buck:auth-rest-dev-auth-inline-postgres \\",
            "//tools/buck:auth-rest-dev-auth-session-postgres \\",
            "//tools/buck:auth-rest-dev-auth-group-admin-postgres \\",
            "//tools/buck:provisioning-dev-principal-upsert-race-postgres",
          ].join("\n"),
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          // Locked so the step cannot be deleted silently: the crate's residual
          // lowering decides row visibility, and its unit target executed in no
          // workflow at all until this contract existed.
          name: "Buck2 platform-authz unit suite",
          run: "env -u DATABASE_URL tools/buck2 test //backend/crates/platform/authz:console-platform-authz-unit",
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          name: "Buck2 console-app unit suite",
          run: "env -u DATABASE_URL tools/buck2 test //backend/app:console-app-unit",
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          name: "Buck2 console-app inline PostgreSQL suites",
          run: [
            "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
            "//tools/buck:app-inline-postgres \\",
            "//tools/buck:app-dev-auth-persona-guard-postgres",
          ].join("\n"),
          workingDirectory: ".",
          if: failFastIf,
        },
      ],
      "backend",
      failures,
    );
    const directCargoTestAnalysis = steps.reduce(
      (analysis, step) => mergeCargoAnalysis(analysis, cargoTestAnalysisInStep(step)),
      emptyCargoAnalysis(),
    );
    if (directCargoTestAnalysis.malformed) {
      failures.push("backend must not contain a malformed executable shell surface");
    }
    for (const packageName of ["console-platform-auth-rest", "console-platform-provisioning"]) {
      if (directCargoTestAnalysis.packages.has(packageName)) {
        failures.push("backend must not run direct Cargo PostgreSQL tests for " + packageName);
      }
    }
  }

  const devUpSmoke = jobBlock(workflow, "dev-up-smoke");
  if (devUpSmoke) {
    requireOrderedStepContracts(
      stepBlocks(devUpSmoke),
      [
        { name: "dev-up compose contract unit test", run: "node --test scripts/dev-up-compose.test.mjs", if: null },
        { name: "Install pinned DotSlash runtime", run: dotSlashBootstrap, if: null },
        { name: "Free runner disk for Rust backend", run: null, if: null },
        { name: "Install Rust toolchain (pinned via rust-toolchain.toml)", run: null, if: null },
      ],
      "dev-up-smoke",
      failures,
    );
  }

  const fullGeneratedFaces = jobBlock(workflow, "generated-face-authority");
  if (fullGeneratedFaces) {
    const fullGeneratedFaceSteps = stepBlocks(fullGeneratedFaces);
    const fullGeneratedFaceCommand = "tools/buck/preflight.sh --full-generated-faces";
    const matchingFullGateSteps = fullGeneratedFaceSteps.filter((step) => runScalar(step) === fullGeneratedFaceCommand);
    if (matchingFullGateSteps.length === 0) {
      failures.push("generated-face-authority must run the complete generated-face closure");
    } else if (matchingFullGateSteps.some((step) => !isUnconditional(step))) {
      failures.push("generated-face-authority must run the complete generated-face closure unconditionally");
    }
    requireDotSlashBefore(
      fullGeneratedFaceSteps,
      fullGeneratedFaceCommand,
      "generated-face-authority",
      failures,
    );
    requireReindeerToolchainBefore(fullGeneratedFaceSteps, fullGeneratedFaceCommand, failures);
  }

  // repo-gates was entirely unlocked: deleting `run: npm run check:adrs` from it returned zero
  // preflight failures. A step wired into ci.yml is not thereby protected — it occupies a slot in
  // the job list and reads as coverage while being one line away from silent removal.
  const repoGates = jobBlock(workflow, "repo-gates");
  if (repoGates) {
    requireOrderedStepContracts(
      stepBlocks(repoGates),
      [{
        name: "Undeclared imports — every bare specifier must be declared",
        run: "npm run check:undeclared-imports",
        if: "${{ !cancelled() }}",
      }, {
        // Wired in 4e7da6b52 and unprotected until now: deleting this step returned zero
        // preflight failures, which is the same one-line-from-silent-removal state the
        // undeclared-imports step above was added to escape.
        name: "Request-body contract — spec fields must exist on the handler",
        run: "npm run check:request-body-contract",
        if: "${{ !cancelled() }}",
      }],
      "repo-gates",
      failures,
    );
  }

  const apiContract = jobBlock(workflow, "api-contract");
  if (apiContract) {
    const apiContractSteps = stepBlocks(apiContract);
    if (!hasOnlyAllowedApiContractSteps(apiContractSteps)) {
      failures.push("api-contract must contain only the approved ordered steps");
    }
    requireDotSlashBefore(
      apiContractSteps,
      "npm run check:openapi-app",
      "api-contract",
      failures,
    );
    const openApiGateIndexes = apiContractSteps
      .map((step, index) => (runScalar(step) === "npm run check:openapi-app" ? index : -1))
      .filter((index) => index >= 0);
    if (openApiGateIndexes.length !== 1) {
      failures.push("api-contract must run exactly one npm run check:openapi-app producer");
    }
    const jobOrStepAppBinaryOverride = /^ {6,}CONSOLE_APP_BIN\s*:/m.test(apiContract);
    const captureStepIndexes = apiContractSteps
      .map((step, index) => (step.startsWith(`name: ${apiContractCaptureName}\n`) ? index : -1))
      .filter((index) => index >= 0);
    const captureStepIndex = captureStepIndexes[0] ?? -1;
    const captureIsDesignated = captureStepIndexes.length === 1 && isDesignatedApiContractCapture(apiContractSteps[captureStepIndex]);
    const nonCaptureSteps = apiContractSteps.filter((_, index) => index !== captureStepIndex);
    const shellAppBinaryOverride = nonCaptureSteps.some((step) => step.includes("CONSOLE_APP_BIN"));
    const cargoTargetAppBinaryOverride = apiContract.split(/\r?\n/).some((line) =>
      !line.trimStart().startsWith("#") && line.includes("CONSOLE_APP_BIN:") && (line.includes("backend/target") || line.includes("CARGO_TARGET_DIR")),
    );
    if (jobOrStepAppBinaryOverride || shellAppBinaryOverride) {
      failures.push("api-contract must not override the captured CONSOLE_APP_BIN");
    }
    if (cargoTargetAppBinaryOverride) {
      failures.push("api-contract must not use a Cargo target path for CONSOLE_APP_BIN");
    }
    if (!captureIsDesignated) {
      failures.push("api-contract capture must use the designated verified command grammar");
    }
    if (nonCaptureSteps.some((step) => step.includes("GITHUB_ENV"))) {
      failures.push("api-contract may reference GITHUB_ENV only in the designated capture step");
    }

    const openApiGateIndex = openApiGateIndexes[0] ?? -1;
    if (openApiGateIndex < 0 || captureStepIndex < openApiGateIndex) {
      failures.push("api-contract must capture the Buck2-built console-app path after its sole producer");
    }
  }

  for (const job of ["backend", "dev-up-smoke"]) {
    const block = jobBlock(workflow, job);
    if (block) requireEffectiveDotSlashBootstrap(block, job, failures);
  }

  // The two jobs that actually run cargo must SHARE one rust-cache entry.
  //
  // MEASURED 2026-07-31: six jobs carried Swatinem/rust-cache, all with
  // `workspaces: backend` and none with a shared-key. rust-cache keys on job name by
  // default, so those were six separate near-duplicate caches of one workspace competing
  // for a 10GB repository LRU budget — they evicted each other. Four of the six never ran
  // cargo at all (Buck2 does not write backend/target) and were removed outright.
  //
  // Without this assertion, deleting `shared-key` silently refragments them: CI stays
  // green, nothing reports it, and the caches quietly stop being shared. Verified that a
  // deletion passed every gate before this block existed.
  for (const job of ["domain-unit", "backend"]) {
    const block = jobBlock(workflow, job);
    if (!block) continue;
    if (!/shared-key:\s*backend-cargo/.test(block)) {
      failures.push(`${job} must share the rust-cache entry via shared-key: backend-cargo`);
    }
    if (!/cache-all-crates:\s*"true"/.test(block)) {
      failures.push(`${job} must set cache-all-crates: "true" on rust-cache`);
    }
  }
  // Exactly ONE writer. Sharing a key without deciding who saves it means whichever job
  // finishes first publishes the cache — so domain-unit's three crates could become the
  // entry `backend` restores for a whole-workspace clippy. `backend` runs
  // clippy --all-targets, a strict superset, so it is the writer and every other cargo
  // job is restore-only.
  {
    const writers = ["domain-unit", "backend"].filter((job) => {
      const block = jobBlock(workflow, job);
      return block && !/save-if:\s*false/.test(block);
    });
    if (writers.length !== 1 || writers[0] !== "backend") {
      failures.push(
        `exactly one cargo job may write the shared rust-cache and it must be backend; found: ${writers.join(", ") || "none"}`,
      );
    }
  }
  // Buck2 never writes backend/target, so a rust-cache step on a Buck2-only job is pure
  // transfer cost and an LRU slot taken from the jobs that do use it.
  for (const job of ["postgres-domain-reachability", "company-conformance", "dev-up-smoke", "api-contract"]) {
    const block = jobBlock(workflow, job);
    if (block && /Swatinem\/rust-cache/.test(block)) {
      failures.push(`${job} runs no cargo and must not carry a rust-cache step`);
    }
  }

  for (const job of protectedJobs) {
    const block = jobBlock(workflow, job);
    if (!block) {
      failures.push(`CI must define protected job ${job}`);
    } else if (!needsPreflight(block)) {
      failures.push(`${job} must need preflight`);
    } else if (/^    if:/m.test(block)) {
      failures.push(`${job} must not define job-level if`);
    }
  }

  return { failures };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const { failures } = evaluateCiPreflight(readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8"));
  if (failures.length > 0) {
    console.error("CI preflight contract failed:");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
  }
  console.log("CI preflight contract passed.");
}
