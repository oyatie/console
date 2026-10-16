'use strict';
// Test-only native correction journey. Import constants, never the legacy
// submit/replay helpers that retain hashes of proof-containing POST bodies.
const {TEXT, ADOPT_KEYS} = require('./group_process_journey.cjs');
const PHASES = Object.freeze(['GROUP_CORRECTION_ENTRY_READY', 'GROUP_CORRECTION_FORM_READY',
  'GROUP_INVALID_READY', 'GROUP_INVALID_RETURNED', 'GROUP_CORRECTION_FOCUSED',
  'GROUP_CORRECTED_READY', 'GROUP_CORRECTED_ADOPTED', 'GROUP_CORRECTED_REOPENED',
  'GROUP_CORRECTED_CURRENT', 'GROUP_CORRECTED_DESIGNATION_REVOKE_READY',
  'GROUP_CORRECTED_OWN_RECEIPT_AFTER_REVOKE', 'GROUP_CORRECTED_RETRY_FORM_READY',
  'GROUP_CORRECTED_REPLAY_READY', 'GROUP_CORRECTED_REPLAYED', 'GROUP_CORRECTED_CURRENT_DENIED']);
const INVALID_TITLE = '한'.repeat(41);
const CORRECTED_TITLE = '입력을 수정하여 등록한 그룹 신원 확인 절차';
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const PUBLIC_KEYS = ADOPT_KEYS.filter(key => key !== 'csrf_proof');
function fact(value, code = 'GROUP_CORRECTION_EVIDENCE') {
  if (value !== true) { const error = new Error(code); error.code = code; throw error; }
}
function id(value) { fact(typeof value === 'string' && UUID.test(value) && !/^0{8}-0{4}-0{4}-0{4}-0{12}$/.test(value)); return value; }
function same(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function sameNames(left, right) {
  return Array.isArray(left) && Array.isArray(right) && left.length === right.length && Array.from(left).every((name, index) =>
    typeof name === 'string' && name === right[index]);
}
function exactKeys(value, keys) {
  fact(value !== null && typeof value === 'object' && !Array.isArray(value)
    && sameNames(Object.keys(value).sort(), keys.slice().sort()));
}

// Only the already typed correction event/result trees reach this check. It is
// a refusal check, not a sanitizer: decoded opaque values are never re-encoded
// to decide whether they contain a proof. The fixed packet has at most 15
// checkpoints, 3 mutations and 18 public form fields; traversal is bounded.
function assertDecodedProofFree(value, proofOrProofs) {
  const proofs = typeof proofOrProofs === 'string' ? [proofOrProofs] : proofOrProofs;
  fact(Array.isArray(proofs) && proofs.length >= 1 && proofs.length <= 2
    && Array.from(proofs).every(proof => typeof proof === 'string' && proof.length > 0 && proof.length <= 8192));
  let visited = 0;
  function check(decoded, depth) {
    fact(depth <= 5 && ++visited <= 2048);
    if (typeof decoded === 'string') {
      fact(decoded.length <= 32768 && proofs.every(proof => !decoded.includes(proof)));
    } else if (typeof decoded === 'number') fact(Number.isFinite(decoded));
    else if (typeof decoded === 'boolean') return;
    else {
      fact(decoded !== null && typeof decoded === 'object');
      for (const [key, field] of Object.entries(decoded)) {
        check(key, depth + 1); check(field, depth + 1);
      }
    }
  }
  check(value, 0);
}
function publicFields(entries, keys) {
  fact(Array.isArray(entries) && entries.length === keys.length);
  fact(Array.from(entries).every(pair => Array.isArray(pair) && pair.length === 2
    && typeof pair[0] === 'string' && typeof pair[1] === 'string'));
  const names = entries.map(pair => pair[0]);
  fact(new Set(names).size === names.length && sameNames(names.slice().sort(), keys.slice().sort()));
  return entries.map(([key, value]) => [key, value]);
}

// correctionEvidenceV1: sole new form-to-owner/log projection. The original
// proof is compared only in memory. Unknown/duplicate keys are rejected before
// a caller can emit an event; no raw body, DOM or secret-derived digest exists.
function publicForm(entries, originalProof, keys = ADOPT_KEYS) {
  fact(Array.isArray(entries));
  fact(Array.from(entries).every(pair => Array.isArray(pair) && pair.length === 2
    && Array.from(pair).every(value => typeof value === 'string')));
  const names = entries.map(([key]) => key);
  fact(new Set(names).size === names.length && sameNames(names.slice().sort(), keys.slice().sort()));
  const proofs = entries.filter(([key]) => key === 'csrf_proof').map(([, value]) => value);
  fact(proofs.length === 1 && proofs[0].length > 0 && proofs[0].length <= 8192);
  fact(typeof originalProof === 'string' && originalProof.length > 0);
  fact(proofs[0] === originalProof, 'GROUP_CORRECTION_PROOF_CHANGED');
  const fields = entries.filter(([key]) => key !== 'csrf_proof');
  assertDecodedProofFree(fields, originalProof);
  return {fields, proof_equal: true};
}
const EVENT_KEYS = Object.freeze(['phase', 'group_id', 'command_id', 'process_id',
  'operation', 'fields', 'projection']);
function emitCorrection(event, proofOrProofs, emit) {
  fact(event !== null && typeof event === 'object' && !Array.isArray(event));
  fact(Object.keys(event).every(key => EVENT_KEYS.includes(key)));
  fact(PHASES.includes(event.phase)); id(event.group_id);
  for (const key of ['command_id', 'process_id']) if (key in event) id(event[key]);
  if ('operation' in event) fact(event.operation === 1);
  if ('fields' in event) {
    const allowed = event.phase === 'GROUP_CORRECTION_FORM_READY'
      ? PUBLIC_KEYS.filter(key => key !== 'operator_responsibility') : PUBLIC_KEYS;
    publicFields(event.fields, allowed);
  }
  if ('projection' in event) validProjection(event.projection);
  // Copy only the typed projection before a caller can log it. Both the issued
  // form proof and a later retry proof remain protected for subsequent events.
  const projected = Object.fromEntries(Object.keys(event).map(key => [key,
    key === 'fields' ? event.fields.map(([name, value]) => [name, value])
      : key === 'projection' ? {...event.projection} : event[key]]));
  assertDecodedProofFree(projected, proofOrProofs);
  return emit(projected);
}
const PROJECTION_KEYS = Object.freeze(['command_id', 'intake_receipt_id', 'result_receipt_id', 'terminal_code',
  'process_id', 'content_version', 'head_revision', 'content_digest', 'head_digest']);
function validProjection(value) {
  exactKeys(value, PROJECTION_KEYS);
  id(value.command_id); id(value.intake_receipt_id); id(value.result_receipt_id); id(value.process_id);
  fact(value.terminal_code === 'ADOPTED' && value.content_version === '1' && value.head_revision === '1');
  for (const key of ['content_digest', 'head_digest']) fact(typeof value[key] === 'string' && /^[0-9a-f]{64}$/.test(value[key]));
}
const FACTS = Object.freeze(['invalid_wire_exact', 'invalid_no_effects', 'title_error',
  'preserved_values', 'preserved_pins', 'preserved_proof', 'correction_link',
  'no_invalid_receipt_link', 'focused_title', 'focus_visible', 'fragment_only',
  'reflow_320', 'keyboard', 'corrected_only_title', 'corrected_wire_exact',
  'same_command', 'accepted_closure', 'reopened_receipt', 'current_content',
  'receipt_content',
  'own_receipt_after_designation_loss', 'terminal_replay_zero_effects',
  'current_denied', 'no_business_storage']);
const SCREENSHOTS = Object.freeze(['group-correction-invalid-320.png',
  'group-correction-focused-320.png', 'group-correction-corrected-320.png',
  'group-correction-adopted.png', 'group-correction-own-receipt-after-revoke.png']);
const CHECKPOINT_BASE = Object.freeze(['phase', 'group', 'company', 'account',
  'owner_effects_verified', 'observed_at_us']);
function projectCheckpoint(row, result) {
  fact(row !== null && typeof row === 'object' && !Array.isArray(row));
  fact(typeof row.phase === 'string' && PHASES.includes(row.phase));
  const keys = CHECKPOINT_BASE.slice();
  if (row.phase === 'GROUP_CORRECTION_ENTRY_READY') keys.push('version', 'head_revision', 'policy_revision');
  else keys.push('process');
  if (row.phase === 'GROUP_CORRECTED_ADOPTED') keys.push('version', 'head_revision', 'terminal_code', 'result_receipt_id');
  if (row.phase === 'GROUP_CORRECTED_CURRENT') keys.push('version', 'head_revision');
  exactKeys(row, keys);
  for (const key of ['group', 'company', 'account']) { id(row[key]); fact(row[key] === result[key]); }
  fact(row.owner_effects_verified === true && Number.isSafeInteger(row.observed_at_us) && row.observed_at_us > 0);
  if ('process' in row) { id(row.process); fact(row.process === result.process); }
  for (const key of ['version', 'head_revision', 'policy_revision']) if (key in row) {
    fact(Number.isSafeInteger(row[key]) && row[key] === (row.phase === 'GROUP_CORRECTION_ENTRY_READY' ? 0 : 1));
  }
  if ('terminal_code' in row) fact(row.terminal_code === 'ADOPTED');
  if ('result_receipt_id' in row) {
    id(row.result_receipt_id);
    if (result.projection) fact(row.result_receipt_id === result.projection.result_receipt_id);
  }
  return Object.fromEntries(Object.keys(row).map(key => [key, row[key]]));
}
function projectMutation(row, result, index) {
  exactKeys(row, ['command', 'path', 'status', 'proof_equal', 'wire_exact']);
  id(row.command); fact(row.command === result.command);
  const path = `/groups/${result.group}/identity/` + (index === 2
    ? `requests/${result.command}/retry` : 'processes');
  fact(typeof row.path === 'string' && row.path === path && row.status === [422, 303, 303][index]
    && row.proof_equal === true && row.wire_exact === true);
  return {command: row.command, path: row.path, status: row.status,
    proof_equal: row.proof_equal, wire_exact: row.wire_exact};
}
function projectEvidence(result) {
  const allowed = ['group', 'company', 'account', 'process', 'command', 'fields',
    'checkpoints', 'mutations', 'screenshots', 'facts', 'projection'];
  fact(result !== null && typeof result === 'object' && !Array.isArray(result));
  fact(Object.keys(result).every(key => allowed.includes(key)));
  for (const key of ['group', 'company', 'account', 'process', 'command']) id(result[key]);
  const fields = publicFields(result.fields, PUBLIC_KEYS);
  fact(result.facts !== null && typeof result.facts === 'object' && !Array.isArray(result.facts));
  fact(Object.keys(result.facts).every(key => FACTS.includes(key) && typeof result.facts[key] === 'boolean'));
  fact(Array.isArray(result.checkpoints) && result.checkpoints.length <= PHASES.length);
  const checkpoints = Array.from(result.checkpoints).map(row => projectCheckpoint(row, result));
  fact(sameNames(checkpoints.map(row => row.phase), PHASES.slice(0, checkpoints.length)));
  fact(Array.isArray(result.mutations) && result.mutations.length <= 3);
  const mutations = Array.from(result.mutations).map((row, index) => projectMutation(row, result, index));
  fact(Array.isArray(result.screenshots) && result.screenshots.length <= SCREENSHOTS.length
    && Array.from(result.screenshots).every((name, index) => typeof name === 'string' && name === SCREENSHOTS[index]));
  if ('projection' in result) validProjection(result.projection);
  return {group: result.group, company: result.company, account: result.account,
    process: result.process, command: result.command, fields, checkpoints, mutations,
    screenshots: result.screenshots.slice(), facts: {...result.facts},
    ...('projection' in result ? {projection: {...result.projection}} : {})};
}
function publicEvidence(result, proofs) {
  const projected = projectEvidence(result);
  assertDecodedProofFree(projected, proofs);
  return projected;
}
function validEvidence(result) {
  try {
    exactKeys(result, ['group', 'company', 'account', 'process', 'command', 'fields',
      'checkpoints', 'mutations', 'screenshots', 'facts', 'projection']);
    for (const key of ['group', 'company', 'account', 'process', 'command']) id(result[key]);
    exactKeys(result.facts, FACTS); fact(FACTS.every(key => result.facts[key] === true));
    fact(sameNames(result.checkpoints.map(row => row.phase), PHASES));
    projectEvidence(result);
    fact(result.mutations.length === 3);
    for (let index = 0; index < 3; index++) {
      projectMutation(result.mutations[index], result, index);
    }
    fact(Array.isArray(result.fields) && sameNames(result.fields.map(pair => pair[0]).sort(), PUBLIC_KEYS.slice().sort()));
    fact(new Set(result.fields.map(pair => pair[0])).size === PUBLIC_KEYS.length);
    fact(result.fields.every(pair => pair.length === 2 && pair.every(value => typeof value === 'string')));
    const named = Object.fromEntries(result.fields);
    for (const [key, value] of Object.entries({...TEXT, title: CORRECTED_TITLE})) fact(named[key] === value);
    fact(named.command_id === result.command && named.process_id === result.process
      && named.expected_prior_process_revision === '0' && named.expected_group_identity_policy_revision === '0'
      && named.operator_responsibility === '1');
    id(named.expected_group_incarnation);
    fact(/^[1-9][0-9]*$/.test(named.expected_group_revision)
      && /^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}$/.test(named.process_expiry));
    validProjection(result.projection);
    fact(result.projection.command_id === result.command && result.projection.process_id === result.process);
    fact(sameNames(result.screenshots, SCREENSHOTS));
    return true;
  } catch { return false; }
}

async function runInvalidFormJourney({page, group, company, account, exchange, capture,
  tabTo, expectDocument, expectMutation, secretFree, clearBrowserCache, retainEvidence}) {
  for (const value of [group, company, account]) id(value);
  const origin = new URL(page.url()).origin, base = `/groups/${group}/identity`, action = base + '/processes';
  const result = {group, company, account, checkpoints: [], mutations: [], screenshots: [], facts: {}};
  // Only this closure retains issued proof. It is never stored on result.
  let proof = '', originalFields, invalidFields;
  const proofs = [];
  function publish() { if (result.fields) retainEvidence(publicEvidence(result, proofs)); }
  async function witness(phase, values = {}) {
    const event = {phase, group_id: group, ...values};
    const answer = proof ? await emitCorrection(event, proofs, exchange) : await exchange(event);
    fact(answer.phase === phase && answer.group === group && answer.company === company
      && answer.account === account && answer.owner_effects_verified === true);
    result.checkpoints.push(answer); publish(); return answer;
  }
  async function open(path, link, status = 200) {
    expectDocument('GET', path, status, false);
    let response;
    if (link) {
      fact(await link.count() === 1 && await link.isVisible() && await link.getAttribute('href') === path);
      await tabTo(link, 64);
      const waiting = page.waitForResponse(row => row.url() === origin + path && row.request().isNavigationRequest());
      await page.keyboard.press('Enter'); response = await waiting;
      await page.waitForURL(origin + path, {waitUntil: 'domcontentloaded'});
    } else { await clearBrowserCache(); response = await page.goto(origin + path, {waitUntil: 'domcontentloaded'}); }
    fact(response.status() === status && response.request().redirectedFrom() === null && page.url() === origin + path);
    await secretFree();
  }
  async function form(path, keys) {
    const found = page.getByRole('main').locator(`form[action="${path}"]`);
    fact(await found.count() === 1 && await found.isVisible(), 'GROUP_PROCESS_FORM_MISSING');
    const state = await found.evaluate(element => ({
      method: Object.getOwnPropertyDescriptor(HTMLFormElement.prototype, 'method').get.call(element),
      enctype: element.enctype, target: element.target,
      names: [...element.elements].filter(row => row.name).map(row => row.name),
      disabled: [...element.elements].some(row => row.disabled),
      proofHidden: [...element.elements].filter(row => row.name === 'csrf_proof').every(row => row.type === 'hidden')
    }));
    fact(state.method === 'post' && state.enctype === 'application/x-www-form-urlencoded'
      && state.target === '' && state.disabled === false && state.proofHidden === true
      && same(state.names.slice().sort(), keys.slice().sort())); return found;
  }
  async function formValues(found, keys = ADOPT_KEYS) {
    const entries = await found.evaluate(element => [...new FormData(element).entries()]);
    return publicForm(entries, proof, keys);
  }
  async function noStorage() {
    fact(await page.evaluate(values => {
      const stored = JSON.stringify({local: Object.entries(localStorage), session: Object.entries(sessionStorage)});
      return values.every(value => !stored.includes(value));
    }, [INVALID_TITLE, CORRECTED_TITLE, ...Object.values(TEXT)]));
    result.facts.no_business_storage = true;
  }
  async function photograph(name) { await capture(name.slice(0, -4)); result.screenshots.push(name); publish(); }
  async function reflow() {
    await page.setViewportSize({width: 320, height: 900});
    fact(await page.evaluate(() => document.documentElement.clientWidth === 320
      && Math.max(document.documentElement.scrollWidth, document.body.scrollWidth) <= 321));
    result.facts.reflow_320 = true;
  }
  async function terminal() {
    const panel = page.getByRole('main').locator('[data-group-process-terminal-code]');
    fact(await panel.count() === 1 && await panel.isVisible());
    const projection = await panel.evaluate(element => ({
      command_id: element.getAttribute('data-group-process-command'),
      result_receipt_id: element.getAttribute('data-group-process-result-receipt'),
      terminal_code: element.getAttribute('data-group-process-terminal-code'),
      process_id: element.getAttribute('data-group-process-id'),
      content_version: element.getAttribute('data-group-process-version'),
      head_revision: element.getAttribute('data-group-process-head-revision'),
      content_digest: element.getAttribute('data-group-process-digest'),
      head_digest: element.getAttribute('data-group-process-head-digest')}));
    const originalRecord = page.getByRole('main').locator('details').filter({has: page.getByText('원래 요청의 접수 기록', {exact: true})});
    fact(await originalRecord.count() === 1);
    if (!(await originalRecord.evaluate(element => element.open))) {
      await tabTo(originalRecord.locator('summary'), 64); await page.keyboard.press('Enter');
    }
    fact(await originalRecord.evaluate(element => element.open));
    const meta = await originalRecord.locator('dl').evaluate(element => Object.fromEntries(
      [...element.querySelectorAll('dt')].map(term => [term.textContent, term.nextElementSibling.textContent])));
    for (const [key, value] of [['요청', result.command], ['그룹', group], ['요청한 계정', account],
      ['절차', result.process], ['그룹 식별 버전', original.expected_group_incarnation],
      ['기대한 그룹 버전', original.expected_group_revision], ['기대한 정책 버전', original.expected_group_identity_policy_revision]]) {
      fact(meta[key] === value);
    }
    projection.intake_receipt_id = id(meta['접수 기록']);
    fact(await page.getByRole('main').getByRole('heading', {name: CORRECTED_TITLE, exact: true, level: 3}).isVisible());
    fact(same(await page.getByRole('main').locator('.group-procedure dd').allTextContents(),
      ['대면·계정·자료 확인', ...Object.values(TEXT).slice(2)]));
    validProjection(projection); fact(projection.command_id === result.command && projection.process_id === result.process);
    result.facts.receipt_content = true;
    return projection;
  }
  async function submit(found, path, expectedStatus, keys = ADOPT_KEYS, expectedFields) {
    expectMutation(path); expectDocument('POST', path, expectedStatus, false);
    if (expectedStatus === 303) expectDocument('GET', receipt, 200, true);
    const waiting = page.waitForResponse(row => row.url() === origin + path && row.request().method() === 'POST');
    const button = found.getByRole('button'); fact(await button.count() === 1);
    await tabTo(button, 64); await page.keyboard.press('Enter'); result.facts.keyboard = true;
    const response = await waiting; fact(response.status() === expectedStatus);
    const wire = publicForm([...new URLSearchParams(response.request().postData()).entries()], proof, keys);
    const normalizeWire = fields => fields.map(([key, value]) => [key, value.replace(/\r\n/g, '\n')]);
    fact(same(normalizeWire(wire.fields), normalizeWire(expectedFields)), 'GROUP_CORRECTION_WIRE_CHANGED');
    result.mutations.push({command: result.command, path, status: expectedStatus, proof_equal: true, wire_exact: true});
    if (expectedStatus === 303) await page.waitForURL(origin + receipt, {waitUntil: 'domcontentloaded'});
    else await page.waitForURL(origin + path, {waitUntil: 'domcontentloaded'});
    await secretFree(); return response;
  }

  await open('/account');
  await open(base, page.getByRole('link', {name: '그룹 신원 확인', exact: true}));
  const initial = await witness('GROUP_CORRECTION_ENTRY_READY');
  fact(initial.policy_revision === 0 && initial.version === 0 && initial.head_revision === 0);
  await open(base + '/processes/new', page.getByRole('main').getByRole('link', {name: '확인 절차 등록', exact: true}));
  let active = await form(action, ADOPT_KEYS);
  proof = await active.locator('[name="csrf_proof"]').inputValue();
  proofs.push(proof);
  originalFields = (await formValues(active, ADOPT_KEYS.filter(key => key !== 'operator_responsibility'))).fields;
  const original = Object.fromEntries(originalFields);
  result.command = id(original.command_id); result.process = id(original.process_id);
  const receipt = `${base}/requests/${result.command}`;
  await witness('GROUP_CORRECTION_FORM_READY', {command_id: result.command, process_id: result.process, fields: originalFields});
  for (const [key, value] of Object.entries({...TEXT, title: INVALID_TITLE})) {
    const control = active.locator(`[name="${key}"]`); fact(await control.count() === 1);
    fact(await control.evaluate(element => element.labels?.length > 0));
    if (key === 'method') await control.selectOption(value); else await control.fill(value);
  }
  const expiry = new Date(Number(initial.observed_at_us) / 1000 + (30 * 86400 + 9 * 3600) * 1000).toISOString().slice(0, 19);
  await active.locator('[name="process_expiry"]').fill(expiry);
  await active.locator('[name="operator_responsibility"]').check();
  fact(Buffer.byteLength(INVALID_TITLE, 'utf8') === 123);
  fact(await active.evaluate(element => element.checkValidity()));
  invalidFields = (await formValues(active)).fields;
  result.fields = invalidFields; publish();
  await noStorage(); await witness('GROUP_INVALID_READY', {command_id: result.command, process_id: result.process, operation: 1, fields: invalidFields});
  await submit(active, action, 422, ADOPT_KEYS, invalidFields);
  active = await form(action, ADOPT_KEYS);
  const returned = await formValues(active);
  fact(same(returned.fields, invalidFields), 'GROUP_CORRECTION_INPUT_DROPPED');
  result.facts.preserved_values = true; result.facts.preserved_pins = true; result.facts.preserved_proof = returned.proof_equal;
  fact(await active.locator('#title[aria-invalid="true"]').count() === 1);
  const validation = page.getByRole('main').locator('.group-validation[role="alert"]');
  fact(await validation.count() === 1 && await validation.isVisible());
  fact(await validation.locator('ul a[href="#title"]').count() === 1);
  result.facts.title_error = true; result.facts.invalid_wire_exact = true;
  await witness('GROUP_INVALID_RETURNED', {command_id: result.command, process_id: result.process, fields: returned.fields});
  result.facts.invalid_no_effects = true;
  await reflow(); await photograph('group-correction-invalid-320.png');
  // Actual product RED: it follows the real 422 and complete no-effect checkpoint.
  const correction = validation.getByRole('link', {name: '입력 내용 수정으로 이동', exact: true});
  fact(await correction.count() === 1 && await correction.isVisible()
    && await correction.getAttribute('href') === '#title'
    && await validation.getByRole('link', {name: '원래 요청 결과 확인', exact: true}).count() === 0,
  'GROUP_INVALID_CORRECTION_ACTION_MISSING');
  result.facts.correction_link = true; result.facts.no_invalid_receipt_link = true;
  await tabTo(correction, 64); await page.keyboard.press('Enter');
  fact(page.url() === origin + action + '#title');
  const title = active.locator('#title');
  fact(await title.evaluate(element => {
    const rect = element.getBoundingClientRect(), style = getComputedStyle(element);
    const header = document.querySelector('header.native-workspace-header');
    const headerStyle = header && getComputedStyle(header);
    const headerBottom = header && ['sticky', 'fixed'].includes(headerStyle.position)
      ? Math.max(0, header.getBoundingClientRect().bottom) : 0;
    return document.activeElement === element && element.matches(':focus-visible')
      && rect.width > 0 && rect.height > 0 && rect.left >= 0 && rect.right <= innerWidth + 1
      && rect.top >= headerBottom + 2 && rect.bottom <= innerHeight + 1 && style.outlineStyle !== 'none'
      && parseFloat(style.outlineWidth) >= 2 && style.outlineColor !== 'transparent'
      && !/rgba\([^)]*,\s*0\)/.test(style.outlineColor);
  }), 'GROUP_CORRECTION_FOCUS_MISSING');
  fact(same((await formValues(active)).fields, invalidFields));
  result.facts.focused_title = true; result.facts.focus_visible = true; result.facts.fragment_only = true;
  await witness('GROUP_CORRECTION_FOCUSED', {command_id: result.command, process_id: result.process, fields: invalidFields});
  await photograph('group-correction-focused-320.png');
  await page.setViewportSize({width: 1440, height: 1000}); fact(same((await formValues(active)).fields, invalidFields));
  await reflow(); fact(same((await formValues(active)).fields, invalidFields));
  await page.keyboard.press('ControlOrMeta+A'); await page.keyboard.type(CORRECTED_TITLE);
  const corrected = (await formValues(active)).fields;
  fact(same(corrected, invalidFields.map(([key, value]) => [key, key === 'title' ? CORRECTED_TITLE : value])));
  fact(Object.fromEntries(corrected).command_id === original.command_id
    && Object.fromEntries(corrected).process_id === original.process_id);
  result.fields = corrected; result.facts.corrected_only_title = true; result.facts.same_command = true;
  await noStorage(); await photograph('group-correction-corrected-320.png');
  await witness('GROUP_CORRECTED_READY', {command_id: result.command, process_id: result.process, operation: 1, fields: corrected});
  await submit(active, action, 303, ADOPT_KEYS, corrected); result.facts.corrected_wire_exact = true;
  result.projection = await terminal();
  const accepted = await witness('GROUP_CORRECTED_ADOPTED', {command_id: result.command, process_id: result.process, projection: result.projection});
  fact(accepted.result_receipt_id === result.projection.result_receipt_id && accepted.terminal_code === 'ADOPTED');
  result.facts.accepted_closure = true; await photograph('group-correction-adopted.png');
  await open(receipt); fact(same(await terminal(), result.projection));
  await witness('GROUP_CORRECTED_REOPENED', {command_id: result.command, process_id: result.process, projection: result.projection});
  result.facts.reopened_receipt = true;
  await open(base);
  const current = page.getByRole('main').locator('[data-group-process-state]');
  fact(await current.count() === 1 && await current.getAttribute('data-group-process-state') === 'ACTIVE'
    && await current.getAttribute('data-group-process-id') === result.process
    && await current.getAttribute('data-group-process-version') === '1'
    && await current.getAttribute('data-group-process-head-revision') === '1');
  fact(await page.getByRole('main').getByRole('heading', {name: CORRECTED_TITLE, exact: true}).isVisible());
  fact(same(await page.getByRole('main').locator('.group-procedure dd').allTextContents(),
    ['대면·계정·자료 확인', ...Object.values(TEXT).slice(2)]));
  fact(!(await page.getByRole('main').innerText()).includes(INVALID_TITLE));
  await witness('GROUP_CORRECTED_CURRENT'); result.facts.current_content = true;
  await witness('GROUP_CORRECTED_DESIGNATION_REVOKE_READY');
  await open(receipt); fact(same(await terminal(), result.projection));
  await witness('GROUP_CORRECTED_OWN_RECEIPT_AFTER_REVOKE', {command_id: result.command, process_id: result.process, projection: result.projection});
  result.facts.own_receipt_after_designation_loss = true;
  await photograph('group-correction-own-receipt-after-revoke.png');
  const retryPath = receipt + '/retry';
  await open(retryPath, page.getByRole('main').getByRole('link', {name: '원래 요청 결과 확인', exact: true}));
  active = await form(retryPath, ['csrf_proof']);
  // Retry GET legitimately issues a new proof. It remains closure-local too.
  proof = await active.locator('[name="csrf_proof"]').inputValue();
  proofs.push(proof);
  await formValues(active, ['csrf_proof']);
  await witness('GROUP_CORRECTED_RETRY_FORM_READY', {command_id: result.command});
  await witness('GROUP_CORRECTED_REPLAY_READY', {command_id: result.command});
  await submit(active, retryPath, 303, ['csrf_proof'], []);
  fact(same(await terminal(), result.projection));
  await witness('GROUP_CORRECTED_REPLAYED', {command_id: result.command});
  result.facts.terminal_replay_zero_effects = true;
  await open(base, null, 404);
  const visible = await page.getByRole('main').innerText();
  fact([group, result.process, CORRECTED_TITLE, ...Object.values(TEXT)].every(value => !visible.includes(value)));
  fact(await page.locator('[data-group-process-state], [data-group-process-terminal-code], main form').count() === 0);
  await witness('GROUP_CORRECTED_CURRENT_DENIED'); result.facts.current_denied = true;
  await noStorage(); fact(validEvidence(result)); publish(); return publicEvidence(result, proofs);
}
module.exports = {PHASES, FACTS, SCREENSHOTS, INVALID_TITLE, CORRECTED_TITLE,
  publicForm, emitCorrection, publicEvidence, validEvidence, runInvalidFormJourney};
