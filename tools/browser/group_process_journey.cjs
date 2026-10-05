'use strict';
// New native aggregate, reusing the actual Account/Company browser producer.
// No request injection, API population, synthetic business receipts or route mocks.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const id = value => { assert.match(value, UUID); assert.notEqual(value, '00000000-0000-0000-0000-000000000000'); return value; };
const PHASES = Object.freeze(['GROUP_ENTRY_READY', 'GROUP_ADOPT_FORM_READY', 'GROUP_ADOPT_READY',
  'GROUP_ADOPTED', 'GROUP_ADOPT_REOPENED', 'GROUP_CURRENT_ACTIVE', 'GROUP_SUSPEND_FORM_READY',
  'GROUP_SUSPEND_READY', 'GROUP_SUSPENDED', 'GROUP_SUSPEND_REOPENED', 'GROUP_CURRENT_SUSPENDED',
  'GROUP_REPLACE_FORM_READY', 'GROUP_REPLACE_READY', 'GROUP_REPLACED', 'GROUP_REPLACE_REOPENED',
  'GROUP_CURRENT_REPLACED', 'GROUP_ORIGINAL_RECEIPT_REOPENED', 'GROUP_DESIGNATION_REVOKE_READY',
  'GROUP_OWN_RECEIPT_AFTER_REVOKE', 'GROUP_RETRY_FORM_READY', 'GROUP_TERMINAL_REPLAY_READY',
  'GROUP_TERMINAL_REPLAYED', 'GROUP_CURRENT_DENIED']);
const TEXT = Object.freeze({title: '직접 확인한 계정의 그룹 신원 확인 절차',
  method: 'ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1',
  intended_claimant_matching_procedure: '신청자와 직접 대화하고 제출한 확인 자료의 성명과 신청 의사를 대조한다.',
  account_possession_procedure: '신청자가 본인의 패스키로 로그인하고 계정 화면을 직접 여는지 확인한다.',
  physical_human_evidence_procedure: '대면 확인 자료의 종류와 확인한 사실을 최소 범위로 기록한다. 계정만으로 사람의 동일성을 추정하지 않는다.',
  duplicate_contradictory_claim_procedure: '중복 또는 상충하는 주장은 승인하지 않고 담당자에게 전달한다.',
  qualification_criteria_instruction: '계정 소유와 신청 의사를 모두 확인한 경우에만 검증 업무를 제안한다.',
  escalation_adjudication_procedure: '의심이나 상충이 남으면 독립 담당자의 검토를 요청하고 그 근거를 기록한다.',
  evidence_minimization_retention_description: '불필요한 원본을 복사하지 않는다. 확인한 종류와 근거만 접근 제한 아래 보관한다.',
  recipient_responsibility: '검증 담당자는 확인한 사실과 불확실성을 구분하고 본인 계정을 검증하지 않는다.'});
const REPLACEMENT = Object.freeze({...TEXT, title: '수정하여 재등록한 그룹 신원 확인 절차',
  duplicate_contradictory_claim_procedure: '중복 또는 상충하는 주장은 보류하고 독립 담당자의 결정을 기록한 후 처리한다.'});
const REASON = '중복 주장 처리 절차를 보완할 때까지 새 검증 업무의 진행을 중단한다.';
const ADOPT_KEYS = Object.freeze(['command_id', 'expected_group_revision', 'expected_group_incarnation',
  'expected_group_identity_policy_revision', 'process_id', 'expected_prior_process_revision',
  'process_expiry', 'operator_responsibility', 'csrf_proof', ...Object.keys(TEXT)]);
const SUSPEND_KEYS = Object.freeze(['command_id', 'expected_group_revision', 'expected_group_incarnation',
  'expected_group_identity_policy_revision', 'csrf_proof', 'process_version', 'process_digest',
  'expected_process_head_revision', 'expected_process_head_digest', 'reason']);
function validEvidence(result) {
  try {
    id(result.group); id(result.company); id(result.account); id(result.process);
    assert.deepEqual(result.checkpoints.map(row => row.phase), PHASES);
    assert.equal(result.checkpoints.every(row => row.owner_effects_verified === true), true);
    assert.equal(result.mutations.length, 4);
    assert.equal(new Set(result.mutations.map(row => row.command)).size, 3);
    assert.equal(result.mutations[3].command, result.mutations[0].command);
    for (const row of result.mutations) { id(row.command); assert.equal(row.status, 303); assert.match(row.body_sha256, /^[0-9a-f]{64}$/); }
    for (const key of ['keyboard', 'reflow_320', 'no_business_storage', 'old_receipt_preserved',
      'suspended_without_version_write', 'replacement_same_process', 'current_denied',
      'own_receipt_after_designation_loss', 'terminal_replay_zero_effects']) assert.equal(result[key], true);
    assert.deepEqual(result.screenshots, ['group-process-adopt-320.png', 'group-process-adopt-desktop.png',
      'group-process-adopted.png', 'group-process-suspended.png', 'group-process-replacement-320.png',
      'group-process-replaced.png', 'group-process-own-receipt-after-revoke.png']);
    return true;
  } catch { return false; }
}
async function runGroupProcessJourney({page, group, company, account, exchange, capture, tabTo,
  expectDocument, expectMutation, secretFree, clearBrowserCache}) {
  id(group); id(company); id(account);
  const origin = new URL(page.url()).origin;
  const base = `/groups/${group}/identity`;
  const result = {group, company, account, checkpoints: [], mutations: [], screenshots: []};
  let original, originalProjection, current;
  async function witness(phase, fields = {}) {
    const answer = await exchange({phase, group_id: group, ...fields});
    assert.equal(answer.phase, phase); assert.equal(answer.group, group);
    assert.equal(answer.company, company); assert.equal(answer.account, account);
    assert.equal(answer.owner_effects_verified, true);
    result.checkpoints.push(answer); return answer;
  }
  async function open(path, link, status = 200) {
    expectDocument('GET', path, status, false);
    let response;
    if (link) {
      assert.equal(await link.count(), 1); assert.equal(await link.isVisible(), true);
      assert.equal(await link.getAttribute('href'), path);
      await tabTo(link, 64);
      const waiting = page.waitForResponse(row => row.url() === origin + path && row.request().isNavigationRequest());
      await page.keyboard.press('Enter'); response = await waiting;
    } else { await clearBrowserCache(); response = await page.goto(origin + path, {waitUntil: 'domcontentloaded'}); }
    assert.equal(page.url(), origin + path); assert.equal(response.status(), status);
    assert.equal(response.request().redirectedFrom(), null); await secretFree(); return response;
  }
  async function geometry() {
    await page.setViewportSize({width: 320, height: 900});
    assert.equal(await page.evaluate(() => Math.max(document.documentElement.scrollWidth,
      document.body.scrollWidth) <= document.documentElement.clientWidth + 1), true);
    result.reflow_320 = true;
  }
  async function photograph(name) { await capture(name.slice(0, -4)); result.screenshots.push(name); }
  async function noStorage() {
    assert.equal(await page.evaluate(values => {
      const stored = JSON.stringify({local: Object.entries(localStorage), session: Object.entries(sessionStorage)});
      return values.every(value => !stored.includes(value));
    }, [TEXT.title, TEXT.physical_human_evidence_procedure, REPLACEMENT.title, REASON]), true);
    result.no_business_storage = true;
  }
  async function form(action, keys) {
    const found = page.getByRole('main').locator(`form[action="${action}"]`);
    if (await found.count() !== 1 || !(await found.isVisible())) { const error = new Error('GROUP_PROCESS_FORM_MISSING'); error.code = 'GROUP_PROCESS_FORM_MISSING'; throw error; }
    const fields = await found.evaluate(element => {
      const controls = [...element.elements].filter(row => row.name);
      return {method: Object.getOwnPropertyDescriptor(HTMLFormElement.prototype, 'method').get.call(element), enctype: element.enctype, target: element.target,
        keys: controls.map(row => row.name), disabled: controls.some(row => row.disabled),
        named: controls.filter(row => row.name !== 'csrf_proof').map(row => [row.name, row.value]),
        proof: controls.filter(row => row.name === 'csrf_proof').map(row => ({type: row.type,
          valid: typeof row.value === 'string' && row.value.length > 0 && row.value.length <= 8192}))};
    });
    assert.equal(fields.method, 'post'); assert.equal(fields.enctype, 'application/x-www-form-urlencoded');
    assert.equal(fields.target, ''); assert.equal(fields.disabled, false);
    assert.deepEqual(fields.keys.slice().sort(), keys.slice().sort());
    assert.deepEqual(fields.proof, [{type: 'hidden', valid: true}]);
    return found;
  }
  async function fields(found) {
    const named = await found.evaluate(element => [...new FormData(element).entries()]
      .filter(([key]) => key !== 'csrf_proof'));
    assert.equal(new Set(named.map(([key]) => key)).size, named.length);
    return named;
  }
  async function adopt(found, content, expiry) {
    for (const [key, value] of Object.entries(content)) {
      const input = found.locator(`[name="${key}"]`);
      assert.equal(await input.count(), 1);
      if (key === 'method') await input.selectOption(value); else await input.fill(value);
      assert.equal(await input.getAttribute('aria-label') !== null ||
        await input.evaluate(element => element.labels?.length > 0), true);
    }
    await found.locator('[name="process_expiry"]').fill(expiry);
    await found.locator('[name="operator_responsibility"]').check();
    assert.equal(await found.locator('[name="operator_responsibility"]').inputValue(), '1');
    await noStorage();
  }
  async function submit(found, phase, committed, operation, action, expectedCode) {
    const posted = await fields(found); const command = id(Object.fromEntries(posted).command_id);
    await witness(phase, {command_id: command, operation, fields: posted});
    expectMutation(action); expectDocument('POST', action, 303, false);
    const receipt = `${base}/requests/${command}`;
    expectDocument('GET', receipt, 200, true);
    const waiting = page.waitForResponse(row => row.url() === origin + action && row.request().method() === 'POST');
    const button = found.getByRole('button', {name: /.+/}); assert.equal(await button.count(), 1);
    await tabTo(button, 64); await page.keyboard.press('Enter'); result.keyboard = true;
    const response = await waiting; assert.equal(response.status(), 303);
    const wire = new URLSearchParams(response.request().postData());
    assert.deepEqual([...wire.entries()].filter(([key]) => key !== 'csrf_proof'), posted);
    assert.equal(wire.getAll('csrf_proof').length, 1); assert.ok(wire.get('csrf_proof'));
    result.mutations.push({command, path: action, status: 303,
      body_sha256: crypto.createHash('sha256').update(response.request().postData()).digest('hex')});
    await page.waitForURL(origin + receipt);
    const projection = await terminal(expectedCode, command);
    const answer = await witness(committed, {command_id: command, operation, receipt_path: receipt, projection});
    assert.equal(answer.process, result.process); assert.equal(answer.terminal_code, expectedCode);
    assert.equal(projection.result_receipt_id, answer.result_receipt_id);
    return {command, receipt, answer, projection};
  }
  async function terminal(code, command) {
    const panel = page.getByRole('main').locator('[data-group-process-terminal-code]');
    assert.equal(await panel.count(), 1); assert.equal(await panel.isVisible(), true);
    assert.equal(await panel.getAttribute('data-group-process-terminal-code'), code);
    assert.equal(await panel.getAttribute('data-group-process-command'), command);
    return panel.evaluate(element => ({command_id: element.getAttribute('data-group-process-command'),
      result_receipt_id: element.getAttribute('data-group-process-result-receipt'),
      terminal_code: element.getAttribute('data-group-process-terminal-code'),
      process_id: element.getAttribute('data-group-process-id'),
      content_version: element.getAttribute('data-group-process-version'),
      head_revision: element.getAttribute('data-group-process-head-revision'),
      content_digest: element.getAttribute('data-group-process-digest'),
      head_digest: element.getAttribute('data-group-process-head-digest')}));
  }
  async function reopen(saved, phase, code) {
    await open(saved.receipt); const projection = await terminal(code, saved.command);
    assert.deepEqual(projection, saved.projection);
    await witness(phase, {command_id: saved.command, projection}); await noStorage();
  }
  async function currentPage(phase, expectedVersion, expectedHead, state, title) {
    await open(base);
    const head = page.getByRole('main').locator('[data-group-process-state]');
    assert.equal(await head.count(), 1);
    for (const [key, value] of [['id', result.process], ['version', String(expectedVersion)],
      ['head-revision', String(expectedHead)], ['state', state]]) {
      assert.equal(await head.getAttribute(`data-group-process-${key}`), value);
    }
    assert.equal(await page.getByRole('main').getByRole('heading', {name: title, exact: true}).isVisible(), true);
    const answer = await witness(phase); assert.equal(answer.process, result.process);
    assert.equal(answer.version, expectedVersion); assert.equal(answer.head_revision, expectedHead);
    current = head; return answer;
  }
  // Account and Company were already enrolled/created through the real preceding UI.
  await open('/account');
  const entry = page.getByRole('link', {name: '그룹 신원 확인', exact: true});
  if (await entry.count() !== 1 || !(await entry.isVisible()) || await entry.getAttribute('href') !== base) {
    const error = new Error('GROUP_PROCESS_ENTRY_MISSING'); error.code = 'GROUP_PROCESS_ENTRY_MISSING'; throw error;
  }
  await open(base, entry); const initial = await witness('GROUP_ENTRY_READY');
  assert.equal(initial.policy_revision, 0); assert.equal(initial.version, 0); assert.equal(initial.head_revision, 0);
  const create = page.getByRole('main').getByRole('link', {name: '확인 절차 등록', exact: true});
  await open(base + '/processes/new', create);
  let activeForm = await form(base + '/processes', ADOPT_KEYS);
  result.process = id(await activeForm.locator('[name="process_id"]').inputValue());
  await witness('GROUP_ADOPT_FORM_READY', {command_id: await activeForm.locator('[name="command_id"]').inputValue(),
    process_id: result.process, fields: await fields(activeForm)});
  const kstExpiry = new Date(Number(initial.observed_at_us) / 1000 + (30 * 86400 + 9 * 3600) * 1000).toISOString().slice(0, 19);
  await adopt(activeForm, TEXT, kstExpiry); await geometry(); await photograph('group-process-adopt-320.png');
  await page.setViewportSize({width: 1440, height: 1000}); await photograph('group-process-adopt-desktop.png');
  original = await submit(activeForm, 'GROUP_ADOPT_READY', 'GROUP_ADOPTED', 1, base + '/processes', 'ADOPTED');
  originalProjection = original.projection; await photograph('group-process-adopted.png');
  await reopen(original, 'GROUP_ADOPT_REOPENED', 'ADOPTED');
  await currentPage('GROUP_CURRENT_ACTIVE', 1, 1, 'ACTIVE', TEXT.title);
  const suspend = page.getByRole('main').getByRole('link', {name: '확인 절차 중단', exact: true});
  const suspendPath = `${base}/processes/${result.process}/suspend`;
  await open(suspendPath, suspend); activeForm = await form(suspendPath, SUSPEND_KEYS);
  await witness('GROUP_SUSPEND_FORM_READY', {command_id: await activeForm.locator('[name="command_id"]').inputValue(),
    process_id: result.process, fields: await fields(activeForm)});
  await activeForm.locator('[name="reason"]').fill(REASON);
  const suspended = await submit(activeForm, 'GROUP_SUSPEND_READY', 'GROUP_SUSPENDED', 6, suspendPath, 'SUSPENDED');
  await photograph('group-process-suspended.png'); await reopen(suspended, 'GROUP_SUSPEND_REOPENED', 'SUSPENDED');
  await currentPage('GROUP_CURRENT_SUSPENDED', 1, 2, 'SUSPENDED', TEXT.title); result.suspended_without_version_write = true;
  const replace = page.getByRole('main').getByRole('link', {name: '확인 절차 수정 등록', exact: true});
  await open(`${base}/processes/${result.process}/replace`, replace); activeForm = await form(base + '/processes', ADOPT_KEYS);
  assert.equal(await activeForm.locator('[name="process_id"]').inputValue(), result.process);
  assert.equal(await activeForm.locator('[name="expected_prior_process_revision"]').inputValue(), '2');
  await witness('GROUP_REPLACE_FORM_READY', {command_id: await activeForm.locator('[name="command_id"]').inputValue(),
    process_id: result.process, fields: await fields(activeForm)});
  await adopt(activeForm, REPLACEMENT, kstExpiry); await geometry(); await photograph('group-process-replacement-320.png');
  const replaced = await submit(activeForm, 'GROUP_REPLACE_READY', 'GROUP_REPLACED', 1, base + '/processes', 'REPLACED');
  await photograph('group-process-replaced.png'); await reopen(replaced, 'GROUP_REPLACE_REOPENED', 'REPLACED');
  await currentPage('GROUP_CURRENT_REPLACED', 2, 3, 'ACTIVE', REPLACEMENT.title); result.replacement_same_process = true;
  await reopen(original, 'GROUP_ORIGINAL_RECEIPT_REOPENED', 'ADOPTED'); result.old_receipt_preserved = true;
  await witness('GROUP_DESIGNATION_REVOKE_READY');
  await reopen(original, 'GROUP_OWN_RECEIPT_AFTER_REVOKE', 'ADOPTED');
  result.own_receipt_after_designation_loss = true; await photograph('group-process-own-receipt-after-revoke.png');
  const retry = page.getByRole('main').getByRole('link', {name: '원래 요청 결과 확인', exact: true});
  const retryPath = `${base}/requests/${original.command}/retry`;
  await open(retryPath, retry); activeForm = await form(retryPath, ['csrf_proof']);
  await witness('GROUP_RETRY_FORM_READY', {command_id: original.command});
  await witness('GROUP_TERMINAL_REPLAY_READY', {command_id: original.command});
  expectMutation(retryPath); expectDocument('POST', retryPath, 303, false); expectDocument('GET', original.receipt, 200, true);
  const retryResponse = page.waitForResponse(row => row.url() === origin + retryPath && row.request().method() === 'POST');
  await tabTo(activeForm.getByRole('button', {name: '원래 요청 결과 확인', exact: true}), 64); await page.keyboard.press('Enter');
  const replayed = await retryResponse; assert.equal(replayed.status(), 303);
  assert.deepEqual([...new URLSearchParams(replayed.request().postData()).keys()], ['csrf_proof']);
  result.mutations.push({command: original.command, path: retryPath, status: 303,
    body_sha256: crypto.createHash('sha256').update(replayed.request().postData()).digest('hex')});
  await page.waitForURL(origin + original.receipt); assert.deepEqual(await terminal('ADOPTED', original.command), originalProjection);
  await witness('GROUP_TERMINAL_REPLAYED', {command_id: original.command}); result.terminal_replay_zero_effects = true;
  const denied = await open(base, null, 404);
  const html = await denied.text(); const visible = await page.locator('body').innerText();
  for (const secret of [group, result.process, TEXT.title, REPLACEMENT.title, TEXT.physical_human_evidence_procedure]) {
    assert.equal(html.includes(secret) || visible.includes(secret), false);
  }
  assert.equal(await page.locator('[data-group-process-state], [data-group-process-terminal-code], main form').count(), 0);
  await witness('GROUP_CURRENT_DENIED'); result.current_denied = true;
  assert.equal(validEvidence(result), true); return result;
}
module.exports = {PHASES, TEXT, REPLACEMENT, REASON, ADOPT_KEYS, SUSPEND_KEYS, validEvidence, runGroupProcessJourney};
