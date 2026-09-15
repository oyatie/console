import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import yaml from "js-yaml";
import { recoveryTestInvocations, RECOVERY_SUPERVISOR, RECOVERY_CASES, OBSERVER_CASE } from "./recovery-test-invocations.mjs";

const workflow = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");
const source = readFileSync(new URL("../../backend/crates/payroll/adapter-postgres/tests/recovery.rs", import.meta.url), "utf8");
const observerSource = readFileSync(new URL("../../backend/crates/payroll/adapter-postgres/tests/durability_observer.rs", import.meta.url), "utf8");
test("actual required recovery commands resolve nine owner cases and the isolated installer", () => {
  const result = recoveryTestInvocations(workflow, source, observerSource);
  assert.deepEqual(result.failures, []);
  assert.equal(result.invocations.length, 10);
});

const mutations = {
  "wrong checkout": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run = steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run.replace('"$GITHUB_WORKSPACE"', '/tmp/stale'); },
  "no run": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run += ' --no-run'; },
  "list only": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run += ' --list'; },
  "skip selected test": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run += ' --skip replay'; },
  "success-only second case": (steps) => { delete steps.filter(s => s.run?.includes(RECOVERY_SUPERVISOR))[1].if; },
  "continued failure": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR))["continue-on-error"] = true; },
  "echo shell": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).shell = 'echo {0}'; },
  "echo command": (steps) => { steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run = 'echo ' + steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)).run; },
  "missing scenario": (steps) => { steps.splice(steps.findIndex(s => s.run?.includes(RECOVERY_SUPERVISOR)), 1); },
  "duplicate scenario": (steps) => { steps.push(structuredClone(steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)))); },
  "wrong cut": (steps) => { const step = steps.filter(s => s.run?.includes(RECOVERY_SUPERVISOR))[1]; step.run = step.run.replace('after-commit-response', 'before-send'); },
  "late image setup": (steps) => { const [image] = steps.splice(steps.findIndex(s => s.id === 'recovery-image'), 1); steps.push(image); },
  "fake setup id": (steps) => { steps.find(s => s.id === 'recovery-image').id = 'different'; },
  "unbound checkout ref": (steps) => { steps.find(s => s.id === 'recovery-checkout').with.ref = 'stale'; },
  "wrong setup guard": (steps) => { const step = steps.find(s => s.run?.includes(RECOVERY_SUPERVISOR)); step.if = step.if.replace('recovery-image', 'unrelated'); },
};
for (const [name, mutate] of Object.entries(mutations)) {
  test(`recovery executor refuses ${name}`, () => {
    const parsed = yaml.load(workflow);
    mutate(parsed.jobs['postgres-reachability-domain-b'].steps);
    const result = recoveryTestInvocations(yaml.dump(parsed, { lineWidth: -1 }), source, observerSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}
test("an additional real owner test cannot silently fall outside selected filters", () => {
  const result = recoveryTestInvocations(workflow, source + '\n#[sqlx::test]\nasync fn third_case() {}\n', observerSource);
  assert.equal(result.invocations.length, 0);
  assert.match(result.failures.join(' '), /discovered/);
});

for (const name of [...RECOVERY_CASES.keys(), OBSERVER_CASE]) {
  test(`new recovery case ${name} must have a real required executor`, () => {
    const parsed = yaml.load(workflow);
    const steps = parsed.jobs['postgres-reachability-domain-b'].steps;
    const index = steps.findIndex(s => s.run?.includes(` ${name} `));
    assert.ok(index >= 0);
    steps.splice(index, 1);
    const result = recoveryTestInvocations(yaml.dump(parsed, { lineWidth: -1 }), source, observerSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}

for (const suffix of [' --no-run', ' --list', ' --skip real_observer', ' --ignored']) {
  test(`observer execution refuses ${suffix}`, () => {
    const parsed = yaml.load(workflow);
    parsed.jobs['postgres-reachability-domain-b'].steps.find(s => s.run?.includes(` ${OBSERVER_CASE} `)).run += suffix;
    const result = recoveryTestInvocations(yaml.dump(parsed), source, observerSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}
