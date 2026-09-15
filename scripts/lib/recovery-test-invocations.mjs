// Recognize only the reviewed two-node executor, never arbitrary Python wrappers.
import yaml from "js-yaml";
import { directExecutable, executableWorkflowCommands } from "./ci-workflow-executables.mjs";
import { stripRustCommentsAndStringLiterals } from "../check-executed-tests-cfg.mjs";

export const RECOVERY_SUPERVISOR = "tools/lanes/recovery/supervise_recovery.py";
export const RECOVERY_CASES = new Map([
  ["replay_cannot_acknowledge_a_locally_visible_receipt_before_remote_apply", ""],
  ["lost_commit_transport_reply_replays_one_real_payroll_effect", "after-commit-response"],
  ["fresh_commit_wait_release_requires_explicit_remote_confirmation", ""],
  ["staging_success_and_idempotent_restage_wait_for_remote_confirmation", ""],
  ["required_remote_unknown_is_bounded_and_never_local_fallback", ""],
  ["finite_remote_bound_and_repeated_replay_preserve_exact_rows", ""],
  ["fresh_commit_deadline_retains_capacity_until_replay_resumes", ""],
  ["aborted_fresh_stage_retains_capacity_until_native_wait_ends", ""],
  ["fresh_stage_transport_error_closes_pool_before_reconciliation", ""],
]);
export const OBSERVER_CASE = "real_observer_installer_is_atomic_replay_exact_and_drift_refusing";
export const APP_CASES = [
  "durability_composition_tests::required_app_startup_admits_only_the_installed_observer",
  "durability_composition_tests::required_api_projected_payroll_dispatch_waits_for_remote_apply",
  "durability_composition_tests::required_workflow_spawn_keeps_unknown_staging_pending_until_retry",
  "durability_composition_tests::required_api_completion_unknown_reconciles_same_command_after_replay",
  "durability_composition_tests::required_api_receipt_absent_unknown_renews_approval_with_same_command",
  "durability_composition_tests::required_api_confirmed_owner_audit_failure_replays_and_repairs",
  "durability_composition_tests::required_workflow_typed_unknown_preserves_pending_and_failed_events",
  "durability_composition_tests::required_workflow_provenance_refusal_is_operation_not_unknown",
];
const ownerPrefix = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-payroll-adapter-postgres", "--test"];
const appPrefix = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-app", "--lib", "--features", "test-recovery"];
const supervisedCases = new Map([
  ...[...RECOVERY_CASES].map(([name, cut]) => [name, { prefix: [...ownerPrefix, "recovery"], cut }]),
  [OBSERVER_CASE, { prefix: [...ownerPrefix, "durability_observer"], cut: "" }],
  ...APP_CASES.map((name) => [name, { prefix: appPrefix, cut: "" }]),
]);
const live = "${{ needs.preflight.outputs.run_live_postgres == 'true' }}";
const condition = "${{ !cancelled() && needs.preflight.outputs.run_live_postgres == 'true' && steps.recovery-checkout.outcome == 'success' && steps.recovery-toolchain.outcome == 'success' && steps.recovery-image.outcome == 'success' }}";
const image = "postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280";
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);

/** Fixed supervised sources contain only reviewed top-level SQLx cases. */
export function supervisedSourceCases(source, expected, modulePrefix = "") {
  // Every supervised command runs cargo test: this exact outer predicate is
  // always true. Keep rejecting inner, compound, negated and conditional guards.
  const code = stripRustCommentsAndStringLiterals(source, { preserveLines: true })
    .replace(/#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g, attribute => attribute.replace(/[^\n]/g, " "));
  if (/#\s*!?\s*\[\s*(?:cfg|cfg_attr|ignore)\b/.test(code)) throw new Error("supervised source cfg/ignore enrollment changed");
  if (/\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*[;{]|\binclude\s*!/.test(code)) throw new Error("supervised source module topology changed");
  const attributes = [...code.matchAll(/#\s*\[\s*(?:(tokio|sqlx)\s*::\s*)?test\b[^\]]*\]/g)];
  const cases = [...code.matchAll(/#\s*\[\s*sqlx\s*::\s*test\b[^\]]*\]\s*async\s+fn\s+([A-Za-z0-9_]+)/g)];
  const topLevel = ({ index }) => {
    const prefix = code.slice(0, index);
    return (prefix.match(/\{/g) ?? []).length === (prefix.match(/\}/g) ?? []).length;
  };
  const discovered = cases.map((match) => `${modulePrefix}${match[1]}`).sort();
  if (attributes.length !== expected.length || attributes.some(match => match[1] !== "sqlx")
    || cases.some(match => !topLevel(match)) || !equal(discovered, [...expected].sort())) {
    throw new Error("selected cases differ from discovered supervised cases");
  }
  return discovered;
}

/** Verify the live module declaration, rather than a cfg example in a comment/literal. */
export function appRecoveryModuleGuard(source) {
  const code = stripRustCommentsAndStringLiterals(source, { preserveLines: true });
  const declarations = [...code.matchAll(/(?:#\s*\[[^\]]*\]\s*)*\bmod\s+durability_composition_tests\s*;/g)];
  if (declarations.length !== 1) throw new Error("app recovery requires one live module declaration");
  const declaration = declarations[0];
  const prefix = code.slice(0, declaration.index);
  if ((prefix.match(/\{/g) ?? []).length !== (prefix.match(/\}/g) ?? []).length) throw new Error("app recovery module must be top level");
  const live = stripRustCommentsAndStringLiterals(source.slice(declaration.index, declaration.index + declaration[0].length), { preserveStrings: true });
  if (!/^#\s*\[\s*cfg\s*\(\s*all\s*\(\s*test\s*,\s*feature\s*=\s*"test-recovery"\s*\)\s*\)\s*\]\s*mod\s+durability_composition_tests\s*;\s*$/.test(live)) {
    throw new Error("app recovery module requires only the reviewed test-recovery cfg");
  }
}

/** Return Cargo invocations only after the entire required scenario union verifies. */
export function recoveryTestInvocations(workflow, source, observerSource, appSource, appLibSource) {
  const failures = [];
  const refuse = (message) => { throw new Error(`recovery reachability: ${message}`); };
  try {
    const parsed = yaml.load(workflow);
    const job = parsed.jobs?.["postgres-reachability-domain-b"];
    if (!job || job.if !== undefined || job["continue-on-error"] !== undefined || job.needs !== "preflight") refuse("required facet changed");
    const steps = job.steps;
    if (!Array.isArray(steps)) refuse("missing steps");
    const setup = (id, predicate) => {
      const matches = steps.map((step, index) => ({ step, index })).filter(({ step }) => step.id === id);
      if (matches.length !== 1) refuse(`missing or duplicate ${id}`);
      const { step, index } = matches[0];
      if (step.if !== live || step["continue-on-error"] !== undefined || !predicate(step)) refuse(`invalid ${id} setup`);
      return index;
    };
    const checkout = setup("recovery-checkout", (s) => s.uses === "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0"
      && s.with?.["persist-credentials"] === false && ["ref", "repository", "path"].every((k) => s.with?.[k] === undefined));
    const toolchain = setup("recovery-toolchain", (s) => s.uses === "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8" && s.with?.toolchain === "1.98.1");
    const imageIndex = setup("recovery-image", (s) => s.run?.trim() === `docker pull ${image}`);
    const ordinary = steps.map((step, index) => ({ step, index })).filter(({ step }) => step.name === "Run disposable PostgreSQL integration targets");
    if (ordinary.length !== 1 || !(checkout < toolchain && toolchain < imageIndex && imageIndex < ordinary[0].index)) refuse("setup must precede ordinary tests");
    supervisedSourceCases(source, [...RECOVERY_CASES.keys()]);
    supervisedSourceCases(observerSource, [OBSERVER_CASE]);
    supervisedSourceCases(appSource, APP_CASES, "durability_composition_tests::");
    appRecoveryModuleGuard(appLibSource);
    const found = new Map();
    const usedSteps = new Set();
    for (const command of executableWorkflowCommands(workflow)) {
      if (command.malformed || command.controlFlow) continue;
      const tokens = directExecutable(command.tokens).tokens;
      if (!equal(tokens.slice(0, 2), ["python3", RECOVERY_SUPERVISOR])) continue;
      if (command.job !== "postgres-reachability-domain-b" || tokens[2] !== "$GITHUB_WORKSPACE" || tokens[3] !== "--") refuse("unbound executor or checkout");
      const cargo = tokens.slice(4);
      const name = cargo.at(-5);
      const scenario = supervisedCases.get(name);
      if (!scenario || !equal(cargo, [...scenario.prefix, name, "--", "--exact", "--test-threads=1", "--nocapture"])) refuse("nonexecuting or altered Cargo invocation");
      if (!equal(command.tokens, [`CONSOLE_RECOVERY_CUT=${supervisedCases.get(name).cut}`, ...tokens])) refuse("unbound fault mode or wrapper");
      const candidates = steps.map((step, index) => ({ step, index })).filter(({ step }) => step.run?.includes(`${RECOVERY_SUPERVISOR} `) && step.run.includes(` ${name} `));
      if (candidates.length !== 1) refuse("scenario must have one distinct step");
      const { step, index } = candidates[0];
      const shell = step.shell ?? job.defaults?.run?.shell ?? parsed.defaults?.run?.shell;
      if (step.if !== condition || step["continue-on-error"] !== undefined || step["working-directory"] !== undefined
        || ![undefined, "bash"].includes(shell) || ![null, "bash"].includes(command.shell)) refuse("scenario lacks failure-propagating execution conditions");
      if (index <= ordinary[0].index || usedSteps.has(index) || found.has(name)) refuse("scenario ordering or duplication");
      usedSteps.add(index);
      found.set(name, cargo);
    }
    if (found.size !== supervisedCases.size) refuse("all supervised cases must execute");
    if (!equal([...found.keys()], [...supervisedCases.keys()])) refuse("reviewed scenario order changed");
    return { invocations: [...found.values()], failures };
  } catch (error) {
    failures.push(error.message);
    return { invocations: [], failures };
  }
}
