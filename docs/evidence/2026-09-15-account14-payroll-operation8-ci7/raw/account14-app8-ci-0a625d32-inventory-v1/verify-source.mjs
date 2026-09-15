import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const root = fileURLToPath(new URL('.', import.meta.url));
const require = createRequire(join(root, 'validation/package.json'));
const yaml = require('js-yaml');
const read = (side, path) => readFileSync(join(root, side, path), 'utf8');
const roster = JSON.parse(readFileSync(join(root, 'roster.json'), 'utf8'));
const checks = [];
const check = (name, body) => { body(); checks.push(name); };
const workflowPath = '.github/workflows/ci.yml';
const oldWorkflow = yaml.load(read('base', workflowPath));
const workflow = yaml.load(read('post', workflowPath));
const steps = workflow.jobs['postgres-reachability-domain-b'].steps;
const additions = steps.filter(step => roster.new_names.some(name => step.run?.includes(` ${name} `)));
check('five separate exact supervised App steps', () => {
  assert.equal(additions.length, 5);
  assert.deepEqual(additions.map(step => roster.new_names.find(name => step.run.includes(` ${name} `))), roster.new_names);
});
const oldLast = oldWorkflow.jobs['postgres-reachability-domain-b'].steps.at(-1);
check('new steps preserve exact executor and failure condition', () => {
  for (const [index, step] of additions.entries()) {
    assert.deepEqual(Object.keys(step), ['name', 'if', 'run']);
    assert.equal(step.if, oldLast.if);
    assert.equal(step.run, oldLast.run.replace(roster.names[2], roster.new_names[index]));
  }
});
check('removing only five steps restores entire parsed workflow', () => {
  const restored = structuredClone(workflow);
  restored.jobs['postgres-reachability-domain-b'].steps = steps.filter(step => !additions.includes(step));
  assert.deepEqual(restored, oldWorkflow);
});
check('original thirteen supervisor commands and order exact', () => {
  const commands = model => model.jobs['postgres-reachability-domain-b'].steps.filter(step => step.run?.includes('supervise_recovery.py')).map(step => step.run);
  assert.equal(commands(oldWorkflow).length, 13);
  assert.equal(commands(workflow).length, 18);
  assert.deepEqual(commands(workflow).slice(0, 13), commands(oldWorkflow));
});
for (const path of ['scripts/lib/recovery-test-invocations.mjs', 'tools/buck/test_preparation_wiring.py']) {
  check(`${path}: only five roster lines added`, () => {
    const restored = read('post', path).split('\n').filter(line => !roster.new_names.some(name => line.trim() === `"${name}",`)).join('\n');
    assert.equal(restored, read('base', path));
  });
}
check('supervisor regression only title and literal union count grow', () => {
  const path = 'scripts/lib/recovery-test-invocations.test.mjs';
  assert.equal(read('post', path).replace('the isolated installer and eight app cases', 'the isolated installer and three app cases')
    .replace('assert.equal(result.invocations.length, 18);', 'assert.equal(result.invocations.length, 13);'), read('base', path));
});
check('preflight only five command strings and five proof entries added', () => {
  const path = 'scripts/check-ci-preflight.mjs';
  const block = /const recoveryCommands = (\[[\s\S]*?\]);/;
  const before = read('base', path);
  const after = read('post', path);
  const oldCommands = JSON.parse(before.match(block)[1]);
  const commands = JSON.parse(after.match(block)[1]);
  assert.equal(commands.length, 18);
  assert.deepEqual(commands.slice(0, 13), oldCommands);
  assert.deepEqual(commands.slice(13), additions.map(step => step.run));
  const restored = after.replace(block, before.match(block)[0]).split('\n')
    .filter(line => !roster.new_names.some(name => line.startsWith(`    proofRun("App recovery ${name.split('::')[1]}"`))).join('\n');
  assert.equal(restored, before);
});
check('preflight test retains every control and grows exact counts', () => {
  const path = 'scripts/check-ci-preflight.test.mjs';
  const restored = read('post', path).replace('observer and eight app cases.', 'observer and three app cases.')
    .replace('"postgres-reachability-domain-b": 22,', '"postgres-reachability-domain-b": 17,')
    .replace('assert.equal(runStepCount, 156,', 'assert.equal(runStepCount, 151,')
    .replace('Five additional app composition proofs extend the matrix: 156*3 = 468.', 'Three app composition proofs extend the matrix: 151*3 = 453.')
    .replace('assert.equal(mutationCount, 468,', 'assert.equal(mutationCount, 453,');
  assert.equal(restored, read('base', path));
});
check('baseline changes only three growth counts and App explanation', () => {
  const path = 'docs/program/executed-tests-baseline.json';
  const before = JSON.parse(read('base', path));
  const after = JSON.parse(read('post', path));
  for (const [source, [oldCount, newCount]] of Object.entries(roster.baseline_changes)) {
    assert.equal(after.test_attribute_baseline[source], newCount);
    after.test_attribute_baseline[source] = oldCount;
  }
  after.why_app_recovery_variant_is_pinned = after.why_app_recovery_variant_is_pinned
    .replace('eight additional live two-node durability cases', 'three additional live two-node durability cases');
  assert.deepEqual(after, before);
});
writeFileSync(join(root, 'source-checks.json'), JSON.stringify({ status: 'PASS', count: checks.length, checks }, null, 2) + '\n');
