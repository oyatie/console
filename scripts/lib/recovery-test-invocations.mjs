// Recognize only the reviewed two-node executor, never arbitrary Python wrappers.
import yaml from "js-yaml";
import { directExecutable, executableWorkflowCommands } from "./ci-workflow-executables.mjs";

export const RECOVERY_SUPERVISOR = "tools/lanes/recovery/supervise_recovery.py";
export const RECOVERY_CASES = new Map([
  ["replay_cannot_acknowledge_a_locally_visible_receipt_before_remote_apply", ""],
  ["lost_commit_transport_reply_replays_one_real_payroll_effect", "after-commit-response"],
  ["fresh_commit_wait_release_requires_explicit_remote_confirmation", ""],
  ["staging_success_and_idempotent_restage_wait_for_remote_confirmation", ""],
  ["required_remote_unknown_is_bounded_and_never_local_fallback", ""],
  ["finite_remote_bound_and_repeated_replay_preserve_exact_rows", ""],
]);
export const OBSERVER_CASE = "real_observer_installer_is_atomic_replay_exact_and_drift_refusing";
const supervisedCases = new Map([
  ...[...RECOVERY_CASES].map(([name, cut]) => [name, { binary: "recovery", cut }]),
  [OBSERVER_CASE, { binary: "durability_observer", cut: "" }],
]);
const live = "${{ needs.preflight.outputs.run_live_postgres == 'true' }}";
const condition = "${{ !cancelled() && needs.preflight.outputs.run_live_postgres == 'true' && steps.recovery-checkout.outcome == 'success' && steps.recovery-toolchain.outcome == 'success' && steps.recovery-image.outcome == 'success' }}";
const image = "postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280";
const prefix = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-payroll-adapter-postgres", "--test"];
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);

/** Return Cargo invocations only after the entire required scenario union verifies. */
export function recoveryTestInvocations(workflow, source, observerSource) {
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
    const discovered = [...source.matchAll(/#\[sqlx::test[^\n]*\]\s*async fn ([A-Za-z0-9_]+)/g)].map((m) => m[1]).sort();
    if (!equal(discovered, [...RECOVERY_CASES.keys()].sort())) refuse("selected cases differ from discovered owner cases");
    const observerCases = [...observerSource.matchAll(/#\[sqlx::test[^\n]*\]\s*async fn ([A-Za-z0-9_]+)/g)].map((m) => m[1]);
    if (!equal(observerCases, [OBSERVER_CASE])) refuse("selected observer differs from discovered installer cases");
    const found = new Map();
    const usedSteps = new Set();
    for (const command of executableWorkflowCommands(workflow)) {
      if (command.malformed || command.controlFlow) continue;
      const tokens = directExecutable(command.tokens).tokens;
      if (!equal(tokens.slice(0, 2), ["python3", RECOVERY_SUPERVISOR])) continue;
      if (command.job !== "postgres-reachability-domain-b" || tokens[2] !== "$GITHUB_WORKSPACE" || tokens[3] !== "--") refuse("unbound executor or checkout");
      const cargo = tokens.slice(4);
      const name = cargo[9];
      if (!supervisedCases.has(name) || !equal(cargo.slice(0, 8), prefix)
        || cargo[8] !== supervisedCases.get(name).binary
        || !equal(cargo.slice(10), ["--", "--exact", "--test-threads=1", "--nocapture"])) refuse("nonexecuting or altered Cargo invocation");
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
    return { invocations: [...found.values()], failures };
  } catch (error) {
    failures.push(error.message);
    return { invocations: [], failures };
  }
}
