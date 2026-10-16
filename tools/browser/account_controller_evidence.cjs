'use strict';
const assert = require('node:assert/strict');
const {validEvidence, validReceipt} = require('./account_company_handoff.cjs');
const NAMES = Object.freeze(['native-first', 'react-first', 'keyboard-before-react', 'autofill-before-react',
  'eventless-value-before-react', 'ime-before-react', 'preclaim-focus', 'partial-bind-rollback',
  'capability-denied', 'freeze-at-csrf']);
const SUCCESS = new Set(['native-first', 'react-first', 'freeze-at-csrf']);
const PREPARATION = new Set(['native-first', 'react-first', 'partial-bind-rollback']);
const SHA = /^[0-9a-f]{64}$/;
function validRecords(records) {
  try {
    assert.deepEqual(records.map(row => row.name), NAMES);
    for (const row of records) {
      assert.deepEqual(Object.keys(row).sort(), ['asset_bytes_unchanged', 'autofill_events', 'capability_calls',
        'csrf_requests', 'detached_render_hits', 'exact_original_nodes', 'ime_events', 'input_preserved',
        'late_callbacks', 'late_callbacks_inert', 'name', 'native_sha256', 'post_requests',
        'react_mounted', 'react_sha256', 'rollback_faults', 'selected_input_frozen']);
      assert.match(row.native_sha256, SHA); assert.match(row.react_sha256, SHA);
      for (const flag of ['asset_bytes_unchanged', 'input_preserved', 'late_callbacks_inert']) assert.equal(row[flag], true);
      assert.equal(row.react_mounted, SUCCESS.has(row.name));
      assert.equal(row.exact_original_nodes, !SUCCESS.has(row.name));
      assert.equal(row.csrf_requests, PREPARATION.has(row.name) || row.name === 'freeze-at-csrf' ? 1 : 0);
      assert.equal(row.post_requests, row.name === 'freeze-at-csrf' ? 1 : 0);
      assert.equal(row.selected_input_frozen, row.name === 'freeze-at-csrf');
      assert.equal(row.autofill_events, row.name === 'autofill-before-react' ? 1 : 0);
      assert.equal(row.rollback_faults, row.name === 'partial-bind-rollback' ? 1 : 0);
      assert.equal(row.capability_calls, row.name === 'capability-denied' ? 1 : 0);
      assert.equal(row.ime_events >= 2, row.name === 'ime-before-react');
      assert.equal(row.late_callbacks >= 3, row.name === 'native-first');
      if (['react-first', 'preclaim-focus', 'partial-bind-rollback'].includes(row.name)) assert.ok(row.detached_render_hits > 0);
      for (const count of ['autofill_events', 'capability_calls', 'csrf_requests', 'detached_render_hits',
        'ime_events', 'late_callbacks', 'post_requests', 'rollback_faults']) assert.ok(Number.isSafeInteger(row[count]) && row[count] >= 0);
      assert.equal(row.native_sha256, records[0].native_sha256);
      assert.equal(row.react_sha256, records[0].react_sha256);
    }
    return true;
  } catch { return false; }
}
function expectedSuffix(r) {
  const committed = `/account/companies/requests/${r.controller?.sent?.command_id}`;
  return [...NAMES.map(name => name === 'capability-denied' ? ['N', '/account', 200] : ['O', '/account/companies/new', 200]),
    ['O', committed, 200], ['O', committed, 200]];
}
function validCompleteEvidence(r) {
  try {
    assert.equal(validEvidence(r.handoff), true);
    assert.equal(validRecords(r.controller.records), true);
    assert.equal(r.observation_failures, 0); assert.equal(r.external_requests, 0);
    assert.equal(r.relay_failure || r.tls_client_error || false, false);
    assert.deepEqual(r.documents.slice(0, r.handoff.documents.length), r.handoff.documents);
    const suffix = expectedSuffix(r), offset = r.handoff.documents.length;
    assert.equal(r.documents.length, offset + suffix.length);
    for (let i = 0; i < suffix.length; i++) {
      assert.deepEqual(r.documents[offset+i], {ordinal: offset+i+1, role: suffix[i][0], path: suffix[i][1],
        status: suffix[i][2], method: 'GET', redirected: false, url_exact: true});
    }
    assert.deepEqual(r.mutations.slice(0, r.handoff.mutations.length), r.handoff.mutations);
    assert.deepEqual(r.mutations.slice(r.handoff.mutations.length), [{ordinal: 9, role: 'O', method: 'POST', path: '/api/v2/companies/enroll'}]);
    assert.deepEqual(r.csrf_requests, ['O', 'O', 'O', 'O', 'O', 'O']);
    assert.deepEqual(r.checkpoints, [...r.handoff.checkpoints, ...NAMES.flatMap(name => name === 'freeze-at-csrf'
      ? ['CONTROLLER_READY', 'CONTROLLER_PRE_DISPATCH', 'CONTROLLER_COMMITTED', 'CONTROLLER_CHECKED']
      : ['CONTROLLER_READY', 'CONTROLLER_CHECKED'])]);
    assert.equal(r.controller.operator_storage_operations, 0);
    assert.equal(r.controller.storage_positive_controls, 4);
    assert.equal(validReceipt(r.controller.sent, r.controller.receipt, r.handoff.a,
      '고정된 관리자 회사 <원본 & 입력>', `controller-${r.handoff.o.replaceAll('-', '')}`), true);
    assert.notEqual(r.controller.sent.command_id, r.handoff.command_id);
    assert.notEqual(r.controller.receipt.org_id, r.handoff.org_id);
    return true;
  } catch { return false; }
}
module.exports = {NAMES, validRecords, expectedSuffix, validCompleteEvidence};
