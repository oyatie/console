'use strict';
// Detached evidence-machinery fixtures only. These never provision business
// records, authentication, owner receipts or browser routes.
const test = require('node:test');
const assert = require('node:assert/strict');
const helper = require('./group_invalid_form_journey.cjs');
const {TEXT, ADOPT_KEYS} = require('./group_process_journey.cjs');
const uuid = number => `00000000-0000-4000-8000-${String(number).padStart(12, '0')}`;
const PROOF = 'IN_MEMORY_ONLY_PROOF_SENTINEL';
function form() {
  const named = {command_id: uuid(5), expected_group_revision: '1', expected_group_incarnation: uuid(6),
    expected_group_identity_policy_revision: '0', process_id: uuid(4), expected_prior_process_revision: '0',
    process_expiry: '2026-11-06T15:23:42', operator_responsibility: '1', csrf_proof: PROOF,
    ...TEXT, title: helper.CORRECTED_TITLE};
  return ADOPT_KEYS.map(key => [key, named[key]]);
}
function fixture() {
  const group = uuid(1), company = uuid(2), account = uuid(3), process = uuid(4), command = uuid(5);
  return {group, company, account, process, command, fields: form().filter(([key]) => key !== 'csrf_proof'),
    checkpoints: helper.PHASES.map(phase => ({phase, group, company, account,
      owner_effects_verified: true, observed_at_us: 1791254622000000,
      ...(phase === 'GROUP_CORRECTION_ENTRY_READY'
        ? {version: 0, head_revision: 0, policy_revision: 0} : {process}),
      ...(phase === 'GROUP_CORRECTED_ADOPTED'
        ? {version: 1, head_revision: 1, terminal_code: 'ADOPTED', result_receipt_id: uuid(7)} : {}),
      ...(phase === 'GROUP_CORRECTED_CURRENT' ? {version: 1, head_revision: 1} : {})})),
    mutations: [422, 303, 303].map((status, index) => ({command,
      path: `/groups/${group}/identity/` + (index === 2 ? `requests/${command}/retry` : 'processes'),
      status, proof_equal: true, wire_exact: true})),
    screenshots: helper.SCREENSHOTS.slice(), facts: Object.fromEntries(helper.FACTS.map(key => [key, true])),
    projection: {command_id: command, intake_receipt_id: uuid(8), result_receipt_id: uuid(7), terminal_code: 'ADOPTED',
      process_id: process, content_version: '1', head_revision: '1', content_digest: 'a'.repeat(64), head_digest: 'b'.repeat(64)}};
}
function rejectedWithoutOutput(event) {
  const persisted = [];
  let failure;
  try { helper.emitCorrection(event, PROOF, value => persisted.push(JSON.stringify(value))); }
  catch (error) { failure = error; }
  assert.equal(failure?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(persisted.length, 0);
  assert.equal(String(failure).includes(PROOF), false);
}
test('safe projection retains exact public form fields and only proof equality', () => {
  const result = helper.publicForm(form(), PROOF);
  assert.deepEqual(result.fields, form().filter(([key]) => key !== 'csrf_proof'));
  assert.equal(result.proof_equal, true);
  assert.equal(JSON.stringify(result).includes(PROOF), false);
});
for (const [name, mutate] of [
  ['unknown key', fields => fields.push(['surprise', 'value'])],
  ['secret key', fields => fields.push(['cookie', PROOF])],
  ['duplicate proof', fields => fields.push(['csrf_proof', PROOF])],
  ['duplicate public key', fields => fields.push(['command_id', uuid(5)])],
  ['secret in public text', fields => fields.find(pair => pair[0] === 'title')[1] = PROOF],
  ['missing proof', fields => fields.splice(fields.findIndex(pair => pair[0] === 'csrf_proof'), 1)],
]) test(`unsafe ${name} is rejected without credential assertion output`, () => {
  const fields = form(); mutate(fields); let error;
  try { helper.publicForm(fields, PROOF); } catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(String(error).includes(PROOF), false);
});
test('changed proof emits only the safe error code', () => {
  let error; try { helper.publicForm(form(), 'DIFFERENT_IN_MEMORY_ONLY_SENTINEL'); } catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_PROOF_CHANGED');
  assert.equal(String(error).includes(PROOF), false);
});
test('safe event emits exact public values once', () => {
  const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), command_id: uuid(5),
    process_id: uuid(4), operation: 1, fields: helper.publicForm(form(), PROOF).fields};
  const output = [];
  helper.emitCorrection(event, PROOF, value => output.push(JSON.stringify(value)));
  assert.deepEqual(output, [JSON.stringify(event)]);
  assert.equal(output[0].includes(PROOF), false);
});
for (const [name, extra] of [
  ['proof-bearing body hash', {body_sha256: 'a'.repeat(64)}],
  ['raw form body', {body: 'csrf_proof=' + PROOF}],
  ['raw assertion actual', {actual: {csrf_proof: PROOF}}],
  ['response headers', {headers: {'set-cookie': PROOF}}],
  ['unknown event key', {unreviewed: true}],
]) test(`logger refuses ${name} before persistence`, () => {
  rejectedWithoutOutput({phase: 'GROUP_INVALID_READY', group_id: uuid(1), ...extra});
});
test('logger refuses proof hidden inside allowed public fields', () => {
  const fields = form().filter(([key]) => key !== 'csrf_proof');
  fields.find(pair => pair[0] === 'title')[1] = PROOF;
  rejectedWithoutOutput({phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields});
});
test('complete detached history is a positive control, not browser acceptance', () => {
  assert.equal(helper.validEvidence(fixture()), true);
  assert.deepEqual(helper.publicEvidence(fixture(), [PROOF]), fixture());
});
for (const [name, mutate] of [
  ['raw result body', r => r.body = 'csrf_proof=' + PROOF],
  ['secret in allowed text', r => r.fields.find(pair => pair[0] === 'title')[1] = PROOF],
  ['whole body hash', r => r.mutations[0].body_sha256 = 'a'.repeat(64)],
  ['secret assertion', r => r.checkpoints[0].actual = {csrf_proof: PROOF}],
]) test(`partial-result publisher refuses ${name}`, () => {
  const result = fixture(); mutate(result); let error;
  try { helper.publicEvidence(result, [PROOF]); } catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(String(error).includes(PROOF), false);
});
for (const fact of helper.FACTS) test(`missing ${fact} does not certify completion`, () => {
  const result = fixture(); delete result.facts[fact]; assert.equal(helper.validEvidence(result), false);
});
for (const [name, mutate] of [
  ['missing owner checkpoint', r => r.checkpoints.splice(3, 1)],
  ['owner effect omission', r => r.checkpoints[3].owner_effects_verified = false],
  ['same command changed', r => r.mutations[1].command = uuid(99)],
  ['extra acceptance', r => r.mutations.push({...r.mutations[1]})],
  ['wrong invalid status', r => r.mutations[0].status = 200],
  ['wrong correction route', r => r.mutations[1].path += '/injected'],
  ['changed wire', r => r.mutations[1].wire_exact = false],
  ['changed proof boolean', r => r.mutations[1].proof_equal = false],
  ['hidden proof key', r => r.fields.push(['csrf_proof', PROOF])],
  ['dropped user text', r => r.fields.find(pair => pair[0] === 'recipient_responsibility')[1] = ''],
  ['changed original pin', r => r.fields.find(pair => pair[0] === 'expected_prior_process_revision')[1] = '1'],
  ['missing current restriction', r => r.facts.current_denied = false],
  ['missing screenshot', r => r.screenshots.pop()],
  ['stale receipt projection', r => r.projection.command_id = uuid(99)],
  ['whole body hash exported', r => r.mutations[0].body_sha256 = 'a'.repeat(64)],
  ['raw assertion exported', r => r.checkpoints[3].actual = {csrf_proof: PROOF}],
]) test(`evidence control rejects ${name}`, () => {
  const result = fixture(); mutate(result); assert.equal(helper.validEvidence(result), false);
});

// R1/R2 successor controls. No credential, proof-derived digest, runtime owner
// data or browser population is used. Assertions report only refusal codes and
// output counts, including when a producer mistakenly accepts a synthetic canary.
function resultRefusedBeforePersistence(result, proofs = [PROOF]) {
  const persisted = [];
  let error;
  try { persisted.push(JSON.stringify(helper.publicEvidence(result, proofs))); }
  catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(persisted.length, 0);
  assert.equal(proofs.every(proof => !String(error).includes(proof)), true);
}
function eventRefusedBeforePersistence(event, proofs) {
  const persisted = [];
  let error;
  try { helper.emitCorrection(event, proofs, value => persisted.push(JSON.stringify(value))); }
  catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(persisted.length, 0);
  const retained = typeof proofs === 'string' ? [proofs] : proofs;
  assert.equal(retained.every(proof => !String(error).includes(proof)), true);
}
for (const phase of helper.PHASES) test(`typed ${phase} actual-shape prefix exports safely`, () => {
  const result = fixture(), index = helper.PHASES.indexOf(phase);
  result.checkpoints = result.checkpoints.slice(0, index + 1);
  result.mutations = result.mutations.slice(0, index >= 13 ? 3 : index >= 6 ? 2 : index >= 3 ? 1 : 0);
  result.screenshots = result.screenshots.slice(0, index >= 10 ? 5 : index >= 6 ? 4 : index >= 5 ? 3 : index >= 4 ? 2 : index >= 3 ? 1 : 0);
  result.facts = {};
  if (index < 6) delete result.projection;
  if (index < 5) result.fields.find(pair => pair[0] === 'title')[1] = helper.INVALID_TITLE;
  assert.deepEqual(helper.publicEvidence(result, [PROOF]), result);
});
const NESTED_METADATA = [
  ['raw body', () => ({body: 'SYNTHETIC_RAW_BODY_CANARY'})],
  ['body hash', () => ({body_sha256: 'a'.repeat(64)})],
  ['headers', () => ({headers: {'set-cookie': 'SYNTHETIC_COOKIE_CANARY'}})],
  ['assertion', () => ({actual: {csrf_proof: 'SYNTHETIC_ASSERTION_CANARY'}})],
  ['array', () => [{body_sha256: 'a'.repeat(64)}]],
];
const CHECKPOINT_SCALARS = [
  ['phase', 0], ['group', 0], ['company', 0], ['account', 0],
  ['owner_effects_verified', 0], ['observed_at_us', 0], ['process', 1],
  ['version', 0], ['head_revision', 0], ['policy_revision', 0],
  ['terminal_code', 6], ['result_receipt_id', 6],
];
for (const [key, index] of CHECKPOINT_SCALARS) for (const [name, value] of NESTED_METADATA) {
  test(`checkpoint ${key} refuses nested ${name} before partial/final persistence`, () => {
    const result = fixture(); result.checkpoints[index][key] = value();
    resultRefusedBeforePersistence(result);
    assert.equal(helper.validEvidence(result), false);
  });
}
for (const key of ['command', 'path', 'status', 'proof_equal', 'wire_exact']) {
  for (const [name, value] of NESTED_METADATA) {
    test(`partial mutation ${key} refuses nested ${name}`, () => {
      const result = fixture(); result.mutations[0][key] = value();
      resultRefusedBeforePersistence(result);
      assert.equal(helper.validEvidence(result), false);
    });
  }
}
for (const [name, mutate] of [
  ['missing required checkpoint time', r => delete r.checkpoints[0].observed_at_us],
  ['string checkpoint time', r => r.checkpoints[0].observed_at_us = '1791254622000000'],
  ['nonintegral checkpoint time', r => r.checkpoints[0].observed_at_us = 1.5],
  ['nonfinite checkpoint time', r => r.checkpoints[0].observed_at_us = Infinity],
  ['unsafe checkpoint time', r => r.checkpoints[0].observed_at_us = Number.MAX_SAFE_INTEGER + 1],
  ['wrong checkpoint scope', r => r.checkpoints[1].group = uuid(99)],
  ['string revision', r => r.checkpoints[0].version = '0'],
  ['wrong entry revision', r => r.checkpoints[0].head_revision = 1],
  ['wrong accepted revision', r => r.checkpoints[6].version = 2],
  ['wrong accepted result identity', r => r.checkpoints[6].result_receipt_id = uuid(99)],
  ['scalar metadata on wrong phase', r => r.checkpoints[3].policy_revision = 0],
  ['wrong partial mutation path', r => r.mutations[0].path += '/unexpected'],
  ['wrong partial mutation status', r => r.mutations[0].status = 200],
  ['string partial mutation status', r => r.mutations[0].status = '422'],
  ['checkpoint hole', r => delete r.checkpoints[1]],
  ['mutation hole', r => delete r.mutations[0]],
  ['screenshot hole', r => delete r.screenshots[0]],
]) test(`typed exporter refuses ${name}`, () => {
  const result = fixture(); mutate(result); resultRefusedBeforePersistence(result);
  assert.equal(helper.validEvidence(result), false);
});
const OPAQUE_PROOF_CANARIES = [
  ['quote', 'OPAQUE_"_FORM_PROOF'],
  ['backslash', 'OPAQUE_\\_FORM_PROOF'],
  ['newline', 'OPAQUE_\n_FORM_PROOF'],
  ['carriage return', 'OPAQUE_\r_FORM_PROOF'],
  ['tab', 'OPAQUE_\t_FORM_PROOF'],
  ['NUL', 'OPAQUE_\u0000_FORM_PROOF'],
  ['Unicode', 'OPAQUE_한글💠_FORM_PROOF'],
  ['Unicode line separator', 'OPAQUE_\u2028_FORM_PROOF'],
];
for (const [name, proof] of OPAQUE_PROOF_CANARIES) {
  test(`decoded ${name} proof in event text produces zero logger output`, () => {
    const fields = fixture().fields;
    fields.find(pair => pair[0] === 'title')[1] = 'prefix' + proof + 'suffix';
    eventRefusedBeforePersistence({phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields}, proof);
  });
  test(`decoded ${name} proof in partial result text produces zero retained output`, () => {
    const result = fixture();
    result.fields.find(pair => pair[0] === 'title')[1] = 'prefix' + proof + 'suffix';
    resultRefusedBeforePersistence(result, [proof]);
  });
  test(`decoded ${name} proof remains opaque in publicForm`, () => {
    const entries = form();
    entries.find(pair => pair[0] === 'csrf_proof')[1] = proof;
    entries.find(pair => pair[0] === 'title')[1] = 'prefix' + proof + 'suffix';
    let error; try { helper.publicForm(entries, proof); } catch (caught) { error = caught; }
    assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
    assert.equal(String(error).includes(proof), false);
  });
}
const ISSUED_PROOF = 'OPAQUE_"_ISSUED_FORM_PROOF';
const RETRY_PROOF = 'OPAQUE_\\_RETRY_FORM_PROOF';
for (const [name, proof] of [['issued', ISSUED_PROOF], ['retry', RETRY_PROOF]]) {
  test(`${name} proof remains protected at later event export`, () => {
    const fields = fixture().fields; fields.find(pair => pair[0] === 'title')[1] = proof;
    eventRefusedBeforePersistence({phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields}, [ISSUED_PROOF, RETRY_PROOF]);
  });
  test(`${name} proof remains protected at later partial/final result export`, () => {
    const result = fixture(); result.fields.find(pair => pair[0] === 'title')[1] = proof;
    resultRefusedBeforePersistence(result, [ISSUED_PROOF, RETRY_PROOF]);
  });
}
test('two opaque proofs permit a genuinely decoded-safe event', () => {
  const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields: fixture().fields};
  const persisted = [];
  helper.emitCorrection(event, [ISSUED_PROOF, RETRY_PROOF], value => persisted.push(JSON.stringify(value)));
  assert.equal(persisted.length, 1);
  assert.equal(typeof persisted[0], 'string');
});
test('two opaque proofs permit genuinely decoded-safe partial/final evidence', () => {
  assert.deepEqual(helper.publicEvidence(fixture(), [ISSUED_PROOF, RETRY_PROOF]), fixture());
});
for (const [name, proof] of [['quote', ISSUED_PROOF], ['backslash', RETRY_PROOF]]) {
  test(`nested ${name} proof under public text is refused before event export`, () => {
    const fields = fixture().fields;
    fields.find(pair => pair[0] === 'title')[1] = {actual: proof};
    eventRefusedBeforePersistence({phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields}, [proof]);
  });
  test(`nested ${name} proof under checkpoint scalar is refused before retained export`, () => {
    const result = fixture(); result.checkpoints[0].version = {actual: proof};
    resultRefusedBeforePersistence(result, [proof]);
  });
}

// R3: sparse arrays are malformed inputs, including absent pair members and
// missing proof/name slots. Refusal must precede logger or retention callbacks.
function sparseRefusedBeforePersistence(produce) {
  const persisted = [];
  let error;
  try { produce(value => persisted.push(value)); }
  catch (caught) { error = caught; }
  assert.equal(error?.code, 'GROUP_CORRECTION_EVIDENCE');
  assert.equal(persisted.length, 0);
  assert.equal(String(error).includes(PROOF), false);
}
const SPARSE_PROOFS = [
  ['one absent proof', () => Array(1)],
  ['two absent proofs', () => Array(2)],
  ['absent first proof', () => { const proofs = [PROOF, RETRY_PROOF]; delete proofs[0]; return proofs; }],
  ['absent second proof', () => { const proofs = [PROOF, RETRY_PROOF]; delete proofs[1]; return proofs; }],
];
for (const [name, vector] of SPARSE_PROOFS) {
  test(`logger refuses ${name} before output`, () => {
    const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields: fixture().fields};
    sparseRefusedBeforePersistence(emit => helper.emitCorrection(event, vector(), emit));
  });
  test(`result publisher refuses ${name} before retention`, () => {
    sparseRefusedBeforePersistence(retain => retain(helper.publicEvidence(fixture(), vector())));
  });
}
test('logger refuses absent proof vector even when public text contains the known canary', () => {
  const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields: fixture().fields};
  event.fields.find(pair => pair[0] === 'title')[1] = PROOF;
  sparseRefusedBeforePersistence(emit => helper.emitCorrection(event, Array(1), emit));
});
for (const [name, index] of [
  ['first', entries => 0],
  ['title', entries => entries.findIndex(pair => pair[0] === 'title')],
  ['last', entries => entries.length - 1],
]) {
  test(`publicForm refuses absent ${name} entry before retention`, () => {
    const entries = form(); delete entries[index(entries)];
    sparseRefusedBeforePersistence(retain => retain(helper.publicForm(entries, PROOF)));
  });
  test(`logger refuses absent ${name} public field before output`, () => {
    const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields: fixture().fields};
    delete event.fields[index(event.fields)];
    sparseRefusedBeforePersistence(emit => helper.emitCorrection(event, PROOF, emit));
  });
  test(`result publisher refuses absent ${name} public field before retention`, () => {
    const result = fixture(); delete result.fields[index(result.fields)];
    sparseRefusedBeforePersistence(retain => retain(helper.publicEvidence(result, [PROOF])));
    assert.equal(helper.validEvidence(result), false);
  });
}
for (const [name, sparsePair] of [
  ['absent key', pair => { delete pair[0]; }],
  ['absent value', pair => { delete pair[1]; }],
  ['both absent members', pair => { delete pair[0]; delete pair[1]; }],
]) {
  test(`publicForm refuses pair with ${name} before retention`, () => {
    const entries = form(); sparsePair(entries.find(pair => pair[0] === 'title'));
    sparseRefusedBeforePersistence(retain => retain(helper.publicForm(entries, PROOF)));
  });
  test(`logger refuses public pair with ${name} before output`, () => {
    const event = {phase: 'GROUP_INVALID_READY', group_id: uuid(1), fields: fixture().fields};
    sparsePair(event.fields.find(pair => pair[0] === 'title'));
    sparseRefusedBeforePersistence(emit => helper.emitCorrection(event, PROOF, emit));
  });
  test(`result publisher refuses public pair with ${name} before retention`, () => {
    const result = fixture(); sparsePair(result.fields.find(pair => pair[0] === 'title'));
    sparseRefusedBeforePersistence(retain => retain(helper.publicEvidence(result, [PROOF])));
    assert.equal(helper.validEvidence(result), false);
  });
}
for (const name of [ADOPT_KEYS.slice().sort()[0], ADOPT_KEYS.slice().sort().at(-1)]) {
  test(`publicForm refuses matching absent ${name} entry and expected-name slots`, () => {
    const entries = form(), keys = ADOPT_KEYS.slice();
    delete entries[entries.findIndex(pair => pair[0] === name)];
    delete keys[keys.indexOf(name)];
    sparseRefusedBeforePersistence(retain => retain(helper.publicForm(entries, PROOF, keys)));
  });
}
