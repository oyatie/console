#!/usr/bin/env node
// Local mirror of the CI `preflight` and `backend` jobs.
//
// Why this exists: those two jobs run ~35 commands in a specific order, several
// of which need a PostgreSQL identity that migration 0196 restricts to
// `console_buck_admin`. Nobody can hold that in their head, so defects were reaching
// CI and costing a 45-minute round trip *each*, and because failures mask one
// another the round trips serialise: one masked layer per run.
//
// The plan below is checked against `.github/workflows/ci.yml` on every run. A
// CI step that is not classified here is a hard error, so this file cannot
// quietly fall behind the pipeline it claims to mirror -- which is exactly how
// the workspace-test run went missing for many commits (H-8 in
// docs/program/false-green-gate-holes.md).
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, rmSync, writeFileSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import yaml from "js-yaml";

const WORKFLOW = ".github/workflows/ci.yml";

/**
 * Every job in `ci.yml`, declared exactly once. `true` means its run-steps are
 * classified in PLAN below; a string means deliberately not mirrored and says
 * why. A job present in CI but absent here is a hard error -- otherwise the
 * step-level completeness check silently scopes itself to whatever this list
 * happened to contain, and a whole new job could land uncovered while
 * `npm run verify` still printed "passed".
 */
const JOBS = new Map([
  ["preflight", true],
  ["backend", true],
  ["repo-gates", true],
  ["kubernetes-manifests", true],
  ["domain-unit", "cargo unit tests for the domain crates behind a pinned toolchain; the `fast` tier already runs the workspace suite"],
  ["postgres-domain-reachability", "serialized Buck2 PostgreSQL targets; the `db` tier already exercises that harness"],
  ["generated-face-authority", "needs pinned Java + Reindeer toolchains to rebuild the full generated-face closure"],
  ["dev-up-smoke", "brings up the whole shared `console-dev` compose project; running it locally tears down other lanes' stacks"],
  ["api-contract", "boots a Buck2-built app against a live listener with the CI keypair fixture"],
  ["company-conformance", true],
]);
const MIRRORED_JOBS = [...JOBS].filter(([, v]) => v === true).map(([name]) => name);
const POSTGRES_IMAGE =
  "postgres:18.4@sha256:4aabea78cf39b90e834caf3af7d602a18565f6fe2508705c8d01aa63245c2e20";
const BOOTSTRAP_GUC = "options%5Bconsole.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1";

/**
 * Every `run:` step in the mirrored jobs, keyed by its CI step name.
 *   tier "fast"    -- no Docker, no network
 *   tier "db"      -- needs Docker
 *   tier "ci-only" -- deliberately not mirrored; `why` must say what covers it
 * `run` overrides the CI command when the local equivalent differs.
 */
const PLAN = new Map([
  // ---- preflight ---------------------------------------------------------
  ["Derive exact console C/T/M train", { tier: "fast", derive: true }],
  ["Install pinned DotSlash runtime", {
    tier: "ci-only",
    why: "runner bootstrap; a local checkout already has the tools/buck2 shim",
  }],
  ["Install workspace dependencies", {
    tier: "ci-only",
    why: "`npm ci` deletes node_modules; `Canonical npm lockfile` below covers lockfile drift",
  }],
  ["Cheap Buck2 generated-face admission", { tier: "fast" }],
  ["Foundation gate contract", { tier: "fast" }],
  ["Console truth-ledger exact-M admission", { tier: "fast" }],
  ["Console fanout planner exact-M admission", {
    tier: "fast",
    needsCleanWorktree: true,
    why: "the planner refuses a dirty worktree, which is the normal state mid-change",
  }],
  ["CI preflight contract tests", { tier: "fast" }],
  ["Console route inventory regression", { tier: "fast" }],
  ["Console authority-train regression", { tier: "fast" }],
  // Wired into `ci.yml` by #556, which is the first thing this mirror had to
  // learn after it started running: the step existed on main and was declared
  // nowhere, so the job-completeness check would have failed closed on it.
  ["Console PR authority bootstrap regression", { tier: "fast" }],
  // The guard that keeps THIS file honest. It ran in no workflow until 2026-08-01,
  // which is why `support-domain-unit` outlived its rename to `domain-unit` on main
  // and `npm run verify` was red for everyone.
  ["Local CI mirror contract", { tier: "fast" }],
  ["Console truth-ledger validator exact-M regression", { tier: "fast" }],
  ["Console fanout planner exact-M regression", { tier: "fast" }],
  ["Buck PostgreSQL environment wrapper regression", { tier: "fast" }],
  ["Buck disposable PostgreSQL harness regression", { tier: "fast" }],
  ["CI preflight contract", { tier: "fast" }],
  ["Canonical npm lockfile", { tier: "fast" }],
  ["Cargo.lock consistency", { tier: "fast" }],

  // ---- backend -----------------------------------------------------------
  ["rustfmt check", { tier: "fast" }],
  ["clippy -D warnings", { tier: "fast" }],
  ["Layer-boundary gate", { tier: "fast" }],
  ["Audit-coverage gate", { tier: "fast" }],
  ["Migration-safety gate", { tier: "fast" }],
  ["Tenant-isolation gate", { tier: "fast" }],
  ["PII-no-logs gate", { tier: "fast" }],
  ["RLS-arming gate", { tier: "fast" }],
  ["Dev-auth-absence gate", { tier: "fast" }],
  ["IaC tier-discipline gate", { tier: "fast" }],
  // The eight `cargo run -p console-gate-*` steps above prove only that each
  // gate exits 0 against THIS tree — which a gate scanning an empty directory
  // also does. These five suites plant a violation in a throwaway tree and
  // assert the gate rejects it, so they are what distinguishes "scanned and
  // found nothing" from "scanned nothing". Unsets DATABASE_URL: none of the
  // five touch a database.
  ["Buck2 CI-gate mutation suites — every gate proven to still reject", { tier: "fast" }],
  ["PR 473 migration operational contract tests", { tier: "fast" }],
  ["Reconcile portable PostgreSQL role topology", {
    tier: "db",
    provision: true,
    why: "replaced locally by a disposable container carrying the same seven roles plus console_buck_admin",
  }],
  ["PR 473 migration operational gate", { tier: "db" }],
  ["Boot smoke — migrate + serve + /readyz", {
    tier: "db",
    run: "CONSOLE_APP_ROLE=migrate SQLX_OFFLINE=true cargo run -q -p console-app",
    why: "migrate half only; the serve/readyz half needs the full CI keypair fixture",
  }],
  // These suites moved off direct Cargo onto generator-owned Buck targets and
  // the disposable role-topology harness, which supplies the
  // migration-0196-authorized `console_buck_admin` identity. The harness brings up
  // its own PostgreSQL, so this needs Docker but not a provisioned database.
  ["Buck2 dev-auth feature PostgreSQL suites", { tier: "db" }],
  // Unsets DATABASE_URL and needs no Docker: the crate's unit target is pure
  // residual-lowering and authoring logic. Mirrored locally because it is the
  // only place the row-visibility safety properties execute at all.
  ["Buck2 platform-authz unit suite", { tier: "fast" }],
  ["Buck2 console-app unit suite", { tier: "fast" }],
  // Unsets DATABASE_URL, so it is a no-Docker step despite living among the
  // PostgreSQL suites. It is the only inventory of mounted routes against
  // openapi.yaml, so a local miss here is a client-contract miss.
  ["Buck2 console-app OpenAPI drift suite", { tier: "fast" }],
  ["Buck2 console-app inline PostgreSQL suites", { tier: "db" }],

  // ---- repo-gates --------------------------------------------------------
  // Pure `npm run` gate binaries: no Docker, no Rust toolchain, no network.
  // `Install workspace dependencies`, `Foundation gate contract` and
  // `Canonical npm lockfile` are byte-identical to their preflight twins, so
  // the entries above cover both jobs.
  ["ADR governance tests", { tier: "fast" }],
  ["ADR governance gate", { tier: "fast" }],
  ["Shared text gate unit tests", { tier: "fast" }],
  ["G004 identity group org people policy foundation gate", { tier: "fast" }],
  ["G005 workflow approval Work Hub lifecycle gate", { tier: "fast" }],
  ["Workflow runtime spine gate", { tier: "fast" }],
  ["Workflow runtime M2 strangler dark-landing gate", { tier: "fast" }],
  ["Workflow runtime M2 Cedar-guard observe-and-record gate", { tier: "fast" }],
  ["Workflow runtime M2 flag-ON runtime gate", { tier: "fast" }],
  ["Workflow runtime M2 outbox-drainer transactional-idempotency gate", { tier: "fast" }],
  ["G006 asset equipment dispatch lifecycle gate", { tier: "fast" }],
  ["G007 collaboration mail calendar poll mobile lifecycle gate", { tier: "fast" }],
  ["G008 import HR payroll readiness gate", { tier: "fast" }],
  ["People HR lifecycle maturity gate", { tier: "fast" }],
  ["Payroll release-gate contract", { tier: "fast" }],
  ["Executed-tests ratchet — a test file must have a path from a workflow step", { tier: "fast" }],
  ["Undeclared imports — every bare specifier must be declared", { tier: "fast" }],
  ["Request-body contract — spec fields must exist on the handler", { tier: "fast" }],
  // The step name promises the whole repository; the gate resolves a declared
  // subset. Mirrored as-is so local matches CI exactly; renaming the step is
  // what would make the promise true.
  ["Doc citations — every code citation must resolve", { tier: "fast" }],

  // ---- kubernetes-manifests ---------------------------------------------
  // Mirrored because `check:production-hardening` pins the exact text of the
  // backend topology step: editing ci.yml fails a job most people would never
  // think to run. That cost a full CI cycle to discover.
  ["Install kubectl (for kustomize renderer)", {
    tier: "ci-only",
    why: "downloads a pinned kubectl; local checkouts use the one already on PATH",
  }],
  ["Install kustomize (NetworkPolicy static render proof)", {
    tier: "ci-only",
    why: "downloads a pinned kustomize; local checkouts use the one already on PATH, and the script falls back to `kubectl kustomize`",
  }],
  ["Governed command-database DARK wiring regression", { tier: "fast" }],
  ["Render manifests and NetworkPolicy enforcement preflight", { tier: "fast" }],
  ["Production hardening contract", { tier: "fast" }],
  ["Production hardening regression tests", { tier: "fast" }],

  // ---- company-conformance ------------------------------------------------
  // Mirrored as of the fan-out's last commit. It was declared not-mirrored while
  // the suite was expected RED, because running it locally would have failed
  // every verify for the same reason it was deliberately not a required check.
  // All five lane types now exist and it is green at 12/12, so it is a normal
  // `db`-tier step and the local mirror runs the real thing.
  ["Company conformance against disposable PostgreSQL", { tier: "db" }],
]);

/** Fail closed when ci.yml grows or loses a job, in either direction. */
export function assertJobsDeclared(jobNames = Object.keys(yaml.load(readFileSync(WORKFLOW, "utf8")).jobs ?? {})) {
  const undeclared = jobNames.filter((name) => !JOBS.has(name));
  const vanished = [...JOBS.keys()].filter((name) => !jobNames.includes(name));
  const problems = [];
  if (undeclared.length) {
    problems.push(
      `CI jobs missing from scripts/verify.mjs (mirror it, or declare why not):\n  ${undeclared.join("\n  ")}`,
    );
  }
  if (vanished.length) {
    problems.push(`Declared jobs with no matching CI job:\n  ${vanished.join("\n  ")}`);
  }
  if (problems.length) throw new Error(problems.join("\n\n"));
  return jobNames;
}

function ciSteps() {
  const doc = yaml.load(readFileSync(WORKFLOW, "utf8"));
  assertJobsDeclared(Object.keys(doc.jobs ?? {}));
  const steps = [];
  for (const job of MIRRORED_JOBS) {
    const defined = doc.jobs?.[job]?.steps;
    if (!Array.isArray(defined)) throw new Error(`${WORKFLOW} has no ${job} job steps`);
    const jobCwd = doc.jobs[job].defaults?.run?.["working-directory"] ?? ".";
    for (const step of defined) {
      if (typeof step.run !== "string") continue;
      if (!step.name) throw new Error(`${job}: every mirrored run-step needs a name`);
      // Honour the same working directory CI uses; the backend job defaults to
      // `backend/`, so running these from the repo root finds no Cargo.toml.
      const cwd = step["working-directory"] ?? jobCwd;
      steps.push({ job, name: step.name, run: step.run.trim(), cwd });
    }
  }
  return steps;
}

/** Fail closed when CI and this plan disagree, in either direction. */
export function assertPlanCoversCi(steps = ciSteps()) {
  const seen = new Set();
  const unclassified = [];
  for (const step of steps) {
    seen.add(step.name);
    if (!PLAN.has(step.name)) unclassified.push(`${step.job}: ${step.name}`);
  }
  const stale = [...PLAN.keys()].filter((name) => !seen.has(name));
  const problems = [];
  if (unclassified.length) {
    problems.push(
      `CI steps missing from scripts/verify.mjs (classify each as fast/db/ci-only):\n  ${unclassified.join("\n  ")}`,
    );
  }
  if (stale.length) {
    problems.push(`Plan entries with no matching CI step:\n  ${stale.join("\n  ")}`);
  }
  if (problems.length) throw new Error(problems.join("\n\n"));
  return steps;
}

function run(command, env, cwd = ".") {
  const result = spawnSync("bash", ["-o", "pipefail", "-c", command], {
    stdio: "inherit",
    env,
    cwd,
  });
  return result.status ?? 1;
}

function capture(command) {
  const result = spawnSync("bash", ["-o", "pipefail", "-c", command], { encoding: "utf8" });
  if (result.status !== 0) throw new Error(`failed: ${command}\n${result.stderr}`);
  return result.stdout.trim();
}

/** Reproduce CI's C/T/M derivation against a synthetic merge, so the console
 *  authority gates can be checked before pushing rather than after. */
function consoleTrainEnv() {
  const tip = capture("git rev-parse HEAD");
  const candidate = capture("git rev-parse HEAD^");
  const base = capture(
    `git merge-base HEAD "${process.env.CONSOLE_VERIFY_BASE ?? "origin/main"}"`,
  );
  const merge = capture(
    `git commit-tree "${tip}^{tree}" -p "${base}" -p "${tip}" -m verify-synthetic-merge`,
  );
  return {
    CONSOLE_CANDIDATE_SHA: candidate,
    CONSOLE_AUTHORITY_TIP_SHA: tip,
    CONSOLE_SYNTHETIC_MERGE_SHA: merge,
  };
}

/** Disposable PostgreSQL carrying the seven app roles plus the `console_buck_admin`
 *  superuser that migration 0196 requires before any migration may be applied.
 *  Always `--rm`, always torn down: a previous incident left 707 orphaned
 *  volumes and filled the machine. */
function withPostgres(body) {
  const name = `console-verify-pg-${process.pid}`;
  const scratch = mkdtempSync(join(tmpdir(), "console-verify-"));
  const adminPassword = capture("openssl rand -hex 16");
  const bootstrapPassword = capture("openssl rand -hex 16");
  try {
    capture(
      `docker run -d --rm --name ${name} -p 127.0.0.1:0:5432 ` +
        `-e POSTGRES_DB=console_ci -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=${adminPassword} ` +
        `${POSTGRES_IMAGE}`,
    );
    capture(
      `for i in $(seq 1 60); do docker exec ${name} pg_isready -U postgres -d console_ci >/dev/null 2>&1 && exit 0; sleep 1; done; exit 1`,
    );
    const port = capture(`docker port ${name} 5432/tcp | head -1 | sed 's/.*://'`);

    capture(`docker cp ops/postgres-reconcile-topology.sh ${name}:/topology.sh`);
    capture(
      `docker exec -e POSTGRES_HOST=127.0.0.1 -e POSTGRES_DB=console_ci ` +
        `-e POSTGRES_ADMIN_USER=postgres -e POSTGRES_ADMIN_PASSWORD=${adminPassword} ` +
        `-e CONSOLE_APP_POSTGRES_PASSWORD=${bootstrapPassword} -e CONSOLE_RT_POSTGRES_PASSWORD=${bootstrapPassword} ` +
        `-e CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=${bootstrapPassword} ` +
        `-e CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=${bootstrapPassword} ` +
        `-e CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=${bootstrapPassword} ` +
        `${name} bash /topology.sh`,
    );

    // The password reaches psql through a mode-0600 file, never argv.
    const sql = join(scratch, "buck-admin.sql");
    writeFileSync(sql, `CREATE ROLE console_buck_admin SUPERUSER LOGIN PASSWORD '${bootstrapPassword}';\n`);
    chmodSync(sql, 0o600);
    capture(`docker cp ${sql} ${name}:/buck-admin.sql`);
    capture(
      `docker exec -e PGPASSWORD=${adminPassword} ${name} psql -U postgres -d postgres -v ON_ERROR_STOP=1 -f /buck-admin.sql`,
    );

    const buckAdminUrl = `postgres://console_buck_admin:${bootstrapPassword}@127.0.0.1:${port}/console_ci?${BOOTSTRAP_GUC}`;
    return body({
      DATABASE_URL: buckAdminUrl,
      CONSOLE_BUCK_ADMIN_DATABASE_URL: buckAdminUrl,
    });
  } finally {
    spawnSync("docker", ["rm", "-f", name], { stdio: "ignore" });
    rmSync(scratch, { recursive: true, force: true });
  }
}

function main() {
  const requested = process.argv[2] ?? "fast";
  if (!["fast", "db", "all"].includes(requested)) {
    console.error("usage: node scripts/verify.mjs [fast|db|all]");
    return 2;
  }
  const steps = assertPlanCoversCi();
  const wanted = requested === "all" ? ["fast", "db"] : [requested];

  const consoleEnv = consoleTrainEnv();
  const dirtyWorktree = capture("git status --porcelain").length > 0;
  const skipped = [];
  const failures = [];
  let baseEnv = { ...process.env, ...consoleEnv, SQLX_OFFLINE: "true" };

  // preflight and repo-gates both run `check:foundation-gates` and
  // `check:package-lock`. Same command, same cwd -- run it once. Keyed on the
  // command, not the step name, so a future divergence still runs both.
  const ran = new Set();

  const execute = (dbEnv) => {
    for (const step of steps) {
      const plan = PLAN.get(step.name);
      if (plan.tier === "ci-only") {
        skipped.push(`${step.name} -- ${plan.why}`);
        continue;
      }
      if (!wanted.includes(plan.tier)) continue;
      // These two CI steps are *setup*, not checks; their local equivalents ran
      // before the loop (consoleTrainEnv / withPostgres).
      if (plan.derive || plan.provision) continue;
      if (plan.needsCleanWorktree && dirtyWorktree) {
        skipped.push(`${step.name} -- ${plan.why}`);
        continue;
      }
      const command = plan.run ?? step.run;
      const key = `${step.cwd} ${command}`;
      if (ran.has(key)) continue;
      ran.add(key);
      console.log(`\n── [${plan.tier}] ${step.name}  (cwd: ${step.cwd})`);
      const status = run(command, { ...baseEnv, ...(dbEnv ?? {}) }, step.cwd);
      if (status !== 0) failures.push(step.name);
    }
  };

  if (wanted.includes("db")) {
    withPostgres((dbEnv) => execute(dbEnv));
  } else {
    execute(undefined);
  }

  if (skipped.length) {
    console.log(`\nNot mirrored locally (${skipped.length}):`);
    for (const line of skipped) console.log(`  • ${line}`);
  }
  if (failures.length) {
    console.error(`\nFAILED (${failures.length}):`);
    for (const name of failures) console.error(`  ✗ ${name}`);
    return 1;
  }
  console.log(`\nverify(${requested}) passed.`);
  return 0;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  try {
    process.exit(main());
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
