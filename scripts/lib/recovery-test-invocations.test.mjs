import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import yaml from "js-yaml";
import { recoveryTestInvocations, RECOVERY_SUPERVISOR, RECOVERY_CASES, OBSERVER_CASE, APP_CASES, supervisedSourceCases, appRecoveryModuleGuard } from "./recovery-test-invocations.mjs";

const workflow = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");
const source = readFileSync(new URL("../../backend/crates/payroll/adapter-postgres/tests/recovery.rs", import.meta.url), "utf8");
const observerSource = readFileSync(new URL("../../backend/crates/payroll/adapter-postgres/tests/durability_observer.rs", import.meta.url), "utf8");
const appSource = readFileSync(new URL("../../backend/app/src/durability_composition_tests.rs", import.meta.url), "utf8");
const appRecoveryCases = value => supervisedSourceCases(value, APP_CASES, "durability_composition_tests::");
const appLibSource = readFileSync(new URL("../../backend/app/src/lib.rs", import.meta.url), "utf8");
test("actual required recovery commands resolve nine owners, the isolated installer and three app cases", () => {
  const result = recoveryTestInvocations(workflow, source, observerSource, appSource, appLibSource);
  assert.deepEqual(result.failures, []);
  assert.equal(result.invocations.length, 13);
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
    const result = recoveryTestInvocations(yaml.dump(parsed, { lineWidth: -1 }), source, observerSource, appSource, appLibSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}
test("an additional real owner test cannot silently fall outside selected filters", () => {
  const result = recoveryTestInvocations(workflow, source + '\n#[sqlx::test]\nasync fn third_case() {}\n', observerSource, appSource, appLibSource);
  assert.equal(result.invocations.length, 0);
  assert.match(result.failures.join(' '), /discovered/);
});

for (const name of [...RECOVERY_CASES.keys(), OBSERVER_CASE, ...APP_CASES]) {
  test(`new recovery case ${name} must have a real required executor`, () => {
    const parsed = yaml.load(workflow);
    const steps = parsed.jobs['postgres-reachability-domain-b'].steps;
    const index = steps.findIndex(s => s.run?.includes(` ${name} `));
    assert.ok(index >= 0);
    steps.splice(index, 1);
    const result = recoveryTestInvocations(yaml.dump(parsed, { lineWidth: -1 }), source, observerSource, appSource, appLibSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}

for (const suffix of [' --no-run', ' --list', ' --skip real_observer', ' --ignored']) {
  test(`observer execution refuses ${suffix}`, () => {
    const parsed = yaml.load(workflow);
    parsed.jobs['postgres-reachability-domain-b'].steps.find(s => s.run?.includes(` ${OBSERVER_CASE} `)).run += suffix;
    const result = recoveryTestInvocations(yaml.dump(parsed), source, observerSource, appSource, appLibSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}

test("an additional real app case cannot silently fall outside selected filters", () => {
  const result = recoveryTestInvocations(workflow, source, observerSource, appSource + '\n#[sqlx::test]\nasync fn dark_app_case() {}\n', appLibSource);
  assert.equal(result.invocations.length, 0);
  assert.match(result.failures.join(' '), /discovered supervised/);
});

for (const name of APP_CASES) {
  for (const [label, alter] of Object.entries({
    "ordinary feature": run => run.replace('--features test-recovery', '--features test-postgres'),
    "missing feature": run => run.replace(' --features test-recovery', ''),
    "substring filter": run => run.replace(name, name.split('::')[1]),
    "list only": run => run.replace('-- --exact --test-threads=1 --nocapture', '-- --list'),
    "all lib tests": run => run.replace(` ${name} -- --exact`, ' --'),
  })) {
    test(`app ${name} refuses ${label}`, () => {
      const parsed = yaml.load(workflow);
      const step = parsed.jobs['postgres-reachability-domain-b'].steps.find(s => s.run?.includes(` ${name} `));
      step.run = alter(step.run);
      const result = recoveryTestInvocations(yaml.dump(parsed), source, observerSource, appSource, appLibSource);
      assert.equal(result.invocations.length, 0);
      assert.ok(result.failures.length > 0);
    });
  }
}

test("app cases cannot share one supervisor or one failure-sensitive step", () => {
  const parsed = yaml.load(workflow);
  const steps = parsed.jobs['postgres-reachability-domain-b'].steps;
  const first = steps.find(s => s.run?.includes(` ${APP_CASES[0]} `));
  const secondIndex = steps.findIndex(s => s.run?.includes(` ${APP_CASES[1]} `));
  first.run += '\n' + steps[secondIndex].run;
  steps.splice(secondIndex, 1);
  const result = recoveryTestInvocations(yaml.dump(parsed), source, observerSource, appSource, appLibSource);
  assert.equal(result.invocations.length, 0);
  assert.ok(result.failures.length > 0);
});

test("app order must follow the isolated observer in the reviewed order", () => {
  const parsed = yaml.load(workflow);
  const steps = parsed.jobs['postgres-reachability-domain-b'].steps;
  const index = steps.findIndex(s => s.run?.includes(` ${APP_CASES[0]} `));
  const [app] = steps.splice(index, 1);
  steps.splice(steps.findIndex(s => s.run?.includes(` ${OBSERVER_CASE} `)), 0, app);
  const result = recoveryTestInvocations(yaml.dump(parsed), source, observerSource, appSource, appLibSource);
  assert.equal(result.invocations.length, 0);
  assert.ok(result.failures.length > 0);
});

for (const [label, alter] of Object.entries({
  "extra tokio test": source => source + '\n#[tokio::test]\nasync fn uncovered_case() {}\n',
  "extra ordinary test": source => source + '\n#[test]\nfn uncovered_case() {}\n',
  "module cfg": source => '#![cfg(any())]\n' + source,
  "module cfg_attr": source => '#![cfg_attr(test, cfg(any()))]\n' + source,
  "selected ignore": source => source.replace('#[sqlx::test', '#[ignore]\n#[sqlx::test'),
  "selected cfg": source => source.replace('#[sqlx::test', '#[cfg(any())]\n#[sqlx::test'),
  "selected cfg_attr": source => source.replace('#[sqlx::test', '#[cfg_attr(test, ignore)]\n#[sqlx::test'),
  "nested cases": source => 'mod nested {\n' + source + '\n}',
})) {
  test(`app source refuses ${label}`, () => {
    const result = recoveryTestInvocations(workflow, source, observerSource, alter(appSource), appLibSource);
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}

test("app source ignores fake attributes and names in comments and literals", () => {
  const spoof = '\n// #[ignore]\n/* #[tokio::test] async fn extra() {} */\nconst SPOOF: &str = r#"#![cfg(any())] #[test] fn extra() {}"#;\n';
  assert.deepEqual(appRecoveryCases(appSource + spoof), [...APP_CASES].sort());
  const selected = APP_CASES[0].split('::')[1];
  const changed = appSource.replace(`async fn ${selected}`, 'async fn unselected_name');
  assert.throws(() => appRecoveryCases(changed + `\n// #[sqlx::test]\n// async fn ${selected}() {}\n`));
});

for (const [label, alter] of Object.entries({
  "commented correct guard": source => source.replace('#[cfg(all(test, feature = "test-recovery"))]', '// #[cfg(all(test, feature = "test-recovery"))]'),
  "literal correct guard": source => source.replace('#[cfg(all(test, feature = "test-recovery"))]', 'const SPOOF: &str = r#"#[cfg(all(test, feature = "test-recovery"))] mod durability_composition_tests;"#;'),
  "additional disabling guard": source => source.replace('#[cfg(all(test, feature = "test-recovery"))]', '#[cfg(any())]\n#[cfg(all(test, feature = "test-recovery"))]'),
  "ordinary feature guard": source => source.replace('feature = "test-recovery"', 'feature = "test-postgres"'),
})) {
  test(`app module refuses ${label}`, () => {
    assert.throws(() => appRecoveryModuleGuard(alter(appLibSource)));
    const result = recoveryTestInvocations(workflow, source, observerSource, appSource, alter(appLibSource));
    assert.equal(result.invocations.length, 0);
    assert.ok(result.failures.length > 0);
  });
}

test("comment/literal module examples cannot create a second live declaration", () => {
  appRecoveryModuleGuard(appLibSource + '\n// mod durability_composition_tests;\nconst EXAMPLE: &str = "mod durability_composition_tests;";\n');
});

for (const [family, original, expected] of [
  ["owner", source, [...RECOVERY_CASES.keys()]],
  ["observer", observerSource, [OBSERVER_CASE]],
]) {
  for (const [label, alter] of Object.entries({
    "extra tokio test": value => value + '\n#[tokio::test]\nasync fn uncovered_case() {}\n',
    "extra ordinary test": value => value + '\n#[test]\nfn uncovered_case() {}\n',
    "module cfg": value => '#![cfg(any())]\n' + value,
    "module cfg_attr": value => '#![cfg_attr(test, cfg(any()))]\n' + value,
    "selected ignore": value => value.replace('#[sqlx::test', '#[ignore]\n#[sqlx::test'),
    "selected cfg": value => value.replace('#[sqlx::test', '#[cfg(any())]\n#[sqlx::test'),
    "selected cfg_attr": value => value.replace('#[sqlx::test', '#[cfg_attr(test, ignore)]\n#[sqlx::test'),
    "nested cases": value => 'mod nested {\n' + value + '\n}',
  })) {
    test(`${family} source refuses ${label}`, () => {
      const changed = alter(original);
      const result = recoveryTestInvocations(workflow, family === 'owner' ? changed : source,
        family === 'observer' ? changed : observerSource, appSource, appLibSource);
      assert.equal(result.invocations.length, 0);
      assert.ok(result.failures.length > 0);
    });
  }
  test(`${family} census ignores spoofed comments/literals without accepting a missing real case`, () => {
    const spoof = '\n// #[ignore]\n/* #[tokio::test] async fn extra() {} */\nconst SPOOF: &str = r#"#![cfg(any())] #[test] fn extra() {}"#;\n';
    assert.deepEqual(supervisedSourceCases(original + spoof, expected), [...expected].sort());
    const changed = original.replace(`async fn ${expected[0]}`, 'async fn unselected_name');
    assert.throws(() => supervisedSourceCases(changed + `\n// #[sqlx::test]\n// async fn ${expected[0]}() {}\n`, expected));
  });
}
