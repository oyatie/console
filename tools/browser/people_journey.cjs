'use strict';
// Actual browser continuation. Account, Company and Payroll prerequisites are
// supplied by company.cjs. Database checkpoints are independently acknowledged.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const {assertNativeHeader, validHeaderEvidence} = require('./native_header.cjs');
const PHASES = Object.freeze(['PEOPLE_INSTALLED', 'PEOPLE_READ_GRANTED', 'PEOPLE_READ_ONLY',
  'PEOPLE_CREATE_GRANTED', 'PEOPLE_DIRECTORY_CREATE_READY', 'PEOPLE_REGISTRATION_OPENED',
  'PEOPLE_WORKSPACE_REOPENED', 'PEOPLE_REGISTRATION_REOPENED', 'PEOPLE_PREPARED', 'PEOPLE_PENDING_REOPENED', 'PEOPLE_COMMITTED',
  'PEOPLE_RECEIPT_REOPENED', 'PEOPLE_DETAIL', 'PEOPLE_DETAIL_REOPENED', 'PEOPLE_LIST',
  'PEOPLE_SEARCH_MATCH', 'PEOPLE_SEARCH_DETAIL', 'PEOPLE_SEARCH_DETAIL_REOPENED',
  'PEOPLE_SEARCH_MATCH_REOPENED', 'PEOPLE_SEARCH_MISS', 'PEOPLE_SEARCH_CLEAR',
  'PEOPLE_READ_REVOKED', 'PEOPLE_READ_DENIED', 'PEOPLE_SEARCH_READ_DENIED', 'PEOPLE_OWN_RECEIPT',
  'PEOPLE_CREATE_REVOKED', 'PEOPLE_RECEIPT_DENIED']);
const LEGAL_NAME = '김하늘 <연구 & 운영>';
const EMPLOYEE_NUMBER = 'UI-사람-001';
const NONMATCHING_NUMBER = 'UI-사람-999';
const searchPath = (directory, number) => `${directory}?${new URLSearchParams({employee_number: number})}`;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
function id(value) {
  assert.equal(typeof value, 'string'); assert.match(value, UUID);
  assert.notEqual(value, '00000000-0000-0000-0000-000000000000'); return value;
}
function deniedProjectionSafe(material, forbidden) {
  if (!material || !Array.isArray(forbidden) || forbidden.length === 0 ||
    forbidden.some(value => typeof value !== 'string' || value.length === 0)) return false;
  if (!['html', 'visible', 'text'].every(key => typeof material[key] === 'string') ||
    !Array.isArray(material.attributes) || !material.attributes.every(x => typeof x === 'string')) return false;
  const escape = value => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#39;');
  return forbidden.every(value => ![material.html, material.visible, material.text, ...material.attributes]
    .some(text => text.includes(value) || text.includes(escape(value))));
}
// Fixed route/permission expectations for actual checkpoints, independent of DOM.
function expectedNativeHeaders(r) {
  assert.match(r.header_origin, /^https:\/\/localhost:[1-9][0-9]*$/);
  const w = `/companies/${id(r.company)}`, d = w + '/people';
  const base = ['/account', w, w + '/policy'];
  const item = (phase, path, read, create, currentPath) => ({phase, url: r.header_origin + path,
    paths: [...base, ...(read ? [d] : []), ...(create ? [d + '/new'] : [])], currentPath, boundTitle: true});
  return [item('PEOPLE_HEADER_READ_ONLY', d, true, false, d),
    item('PEOPLE_HEADER_REGISTRATION', d + '/new', true, true, d + '/new'),
    item('PEOPLE_HEADER_PENDING', d + '/requests/' + r.command, true, true),
    item('PEOPLE_HEADER_DETAIL', d + '/' + r.employee, true, true),
    item('PEOPLE_HEADER_CREATE_ONLY', d + '/requests/' + r.command, false, true)];
}
function completeNativeHeaders(r) {
  try {
    const expected = expectedNativeHeaders(r);
    return Array.isArray(r.native_headers) && r.native_headers.length === expected.length &&
      r.native_headers.every((row, i) => validHeaderEvidence(row, expected[i]));
  } catch { return false; }
}

function validEvidence(r) {
  try {
    id(r.company); id(r.account); id(r.command); id(r.employee); id(r.person);
    assert.equal(completeNativeHeaders(r), true);
    assert.equal(r.employee, r.person);
    assert.deepEqual(r.checkpoints.map(x => x.phase), PHASES);
    assert.equal(r.mutations.length, 7);
    assert.equal(new Set(r.mutations.map(x => x.command)).size, 6);
    const policy = `/companies/${r.company}/policy/people-directory`;
    const directory = `/companies/${r.company}/people`;
    assert.deepEqual(r.mutations.map(x => x.path), [policy + '/catalog', policy + '/read/grants',
      policy + '/create/grants', directory + '/requests', directory + `/requests/${r.command}/execute`,
      policy + `/read/grants/${id(r.read_assignment)}/revoke`,
      policy + `/create/grants/${id(r.create_assignment)}/revoke`]);
    for (const m of r.mutations) {
      id(m.command); assert.equal(m.status, 303); assert.match(m.body_sha256, /^[0-9a-f]{64}$/);
    }
    assert.equal(r.mutations[3].command, r.command); assert.equal(r.mutations[4].command, r.command);
    for (const c of r.checkpoints) {
      assert.equal(c.company, r.company); assert.equal(c.account, r.account);
      assert.equal(c.owner_effects_verified, true);
    }
    for (const flag of ['keyboard', 'reflow_320', 'read_only', 'pending_reopened', 'receipt_reopened',
      'detail_reopened', 'search_match', 'search_detail_reopened', 'search_miss', 'search_clear',
      'search_denied', 'search_asset_referrer_absent', 'search_detail_referrer_absent',
      'search_response_policy', 'read_revoked', 'create_only_receipt', 'create_revoked',
      'no_local_business_storage']) {
      assert.equal(r[flag], true);
    }
    for (const name of ['people-directory-320.png', 'people-workspace-next-task.png',
      'people-workspace-next-task-320.png', 'people-registration-desktop.png',
      'people-pending-320.png', 'people-detail-desktop.png', 'people-detail-320.png',
      'people-search-match-320.png', 'people-search-miss-320.png']) {
      assert.equal(r.screenshots.filter(x => x === name).length, 1);
    }
    return true;
  } catch { return false; }
}

function expectedDocuments(r) {
  const p = `/companies/${r.company}/policy/people-directory`, d = `/companies/${r.company}/people`;
  const w = `/companies/${r.company}`, request = d + `/requests/${r.command}`, detail = d + '/' + r.employee;
  const match = searchPath(d, EMPLOYEE_NUMBER), miss = searchPath(d, NONMATCHING_NUMBER);
  const get = (path, status = 200) => ({method: 'GET', path, status, redirected: false});
  const action = (index, destination) => [{method: 'POST', path: r.mutations[index].path, status: 303, redirected: false},
    {method: 'GET', path: destination, status: 200, redirected: true}];
  const receipt = (kind, index) => p + `/requests/${kind}/${r.mutations[index].command}`;
  return [get(p + '/install'), ...action(0, receipt('install', 0)), get(p + '/read/grant'),
    ...action(1, receipt('grant', 1)), get(d), get(d + '/new', 404), get(w), get(p + '/create/grant'),
    ...action(2, receipt('grant', 2)), get(d), get(d + '/new'), get(w), get(d + '/new'),
    ...action(3, request), get(request),
    ...action(4, request), get(request), get(detail), get(detail), get(d),
    get(match), get(detail), get(detail), get(match), get(miss), get(d),
    get(w), get(p + '/read/revoke'),
    ...action(5, receipt('revoke', 5)), get(d, 404), get(detail, 404), get(match, 404), get(request), get(w), get(p + '/create/revoke'),
    ...action(6, receipt('revoke', 6)), get(w), get(d + '/new', 404), get(request, 404)];
}

async function runPeopleJourney({page, company, companyName, account, exchange, capture, tabTo,
  expectDocument, expectMutation, secretFree}) {
  id(company); id(account);
  const origin = new URL(page.url()).origin;
  const workspace = `/companies/${company}`;
  const directory = workspace + '/people';
  const policy = workspace + '/policy/people-directory';
  const result = {company, account, checkpoints: [], mutations: [], screenshots: [],
    header_origin: origin, native_headers: []};
  async function header(phase) {
    const expected = expectedNativeHeaders(result).filter(row => row.phase === phase);
    assert.equal(expected.length, 1);
    result.native_headers.push(await assertNativeHeader(page, tabTo, expected[0]));
  }
  async function directoryNavigation() {
    const banner = page.getByRole('banner');
    if (page.viewportSize().width <= 680) {
      const summary = banner.locator('summary').filter({hasText: /^업무 탐색$/});
      assert.equal(await summary.count(), 1);
      const menu = summary.locator('..');
      assert.equal(await menu.evaluate(e => e.open), false);
      await tabTo(summary, 48); await page.keyboard.press('Enter');
      assert.equal(await menu.evaluate(e => e.open), true);
    }
    return banner.getByRole('link', {name: '사람', exact: true});
  }
  async function nextTask(name, destination) {
    const task = page.getByRole('main').getByRole('region', {name: '다음 업무', exact: true});
    assert.equal(await task.count(), 1);
    assert.equal(await task.locator('a,button,[role="button"]').count(), 1);
    const link = task.getByRole('link', {name, exact: true});
    assert.equal(await link.count(), 1);
    assert.equal(await link.getAttribute('href'), destination);
    return link;
  }
  let pendingPath, detailPath;
  async function noBusinessStorage() {
    assert.equal(await page.evaluate(({name, number}) => {
      const data = JSON.stringify({local: Object.entries(localStorage), session: Object.entries(sessionStorage)});
      return !data.includes(name) && !data.includes(number);
    }, {name: LEGAL_NAME, number: EMPLOYEE_NUMBER}), true);
  }
  async function witness(phase, values = {}) {
    const reply = await exchange({phase, ...values});
    assert.equal(reply.phase, phase); assert.equal(reply.company, company); assert.equal(reply.account, account);
    assert.equal(reply.owner_effects_verified, true);
    result.checkpoints.push(reply); return reply;
  }
  async function open(path, status = 200, link) {
    expectDocument('GET', path, status, false);
    let response;
    if (link) {
      if (await link.count() !== 1 || !(await link.isVisible())) {
        const error = new Error('PEOPLE_ENTRY'); error.code = 'PEOPLE_ENTRY'; throw error;
      }
      assert.equal(await link.getAttribute('href'), path);
      await tabTo(link, 48);
      const waiting = page.waitForResponse(r => r.url() === origin + path && r.request().isNavigationRequest());
      await page.keyboard.press('Enter'); response = await waiting;
    } else { response = await page.goto(origin + path); }
    await page.waitForURL(origin + path);
    assert.equal(response.status(), status); assert.equal(response.request().redirectedFrom(), null);
    if (status === 404) {
      const material = await page.evaluate(() => ({visible: document.body.innerText, text: document.documentElement.textContent,
        attributes: [...document.querySelectorAll('*')].flatMap(e => [...e.attributes].map(a => a.value))}));
      material.html = await response.text();
      assert.equal(deniedProjectionSafe(material, [company, companyName, account, LEGAL_NAME, EMPLOYEE_NUMBER,
        result.command, result.employee, result.person].filter(Boolean)), true);
      assert.equal(await page.locator('form,[data-people-record],[data-people-command]').count(), 0);
    }
    return response;
  }
  async function requestView(outcome) {
    await noBusinessStorage();
    const panel = page.locator(`[data-people-outcome="${outcome}"]`);
    assert.equal(await panel.count(), 1); assert.equal(await panel.isVisible(), true);
    assert.equal(await panel.getAttribute('data-people-command'), result.command);
    assert.equal(await panel.getByText(LEGAL_NAME, {exact: true}).isVisible(), true);
    assert.equal(await panel.getByText(EMPLOYEE_NUMBER, {exact: true}).isVisible(), true);
    if (outcome === 'committed') {
      assert.equal(await panel.getAttribute('data-people-employee'), result.employee);
      assert.equal(await panel.getAttribute('data-people-person'), result.person);
      assert.equal(await panel.getByText('사람 목록에 등록했습니다. 이 등록으로 고용이나 발령이 생성되지는 않았습니다.', {exact: true}).isVisible(), true);
    } else {
      assert.equal(await panel.locator('form[data-people-operation="execute"] input[name="command_id"]').inputValue(), result.command);
    }
  }
  async function detailView() {
    await noBusinessStorage();
    const record = page.locator('[data-people-record]');
    assert.equal(await record.count(), 1); assert.equal(await record.isVisible(), true);
    assert.equal(await record.getAttribute('data-people-record'), result.employee);
    assert.equal(await record.getAttribute('data-people-person'), result.person);
    assert.equal(await page.getByRole('heading', {name: LEGAL_NAME, exact: true, level: 1}).isVisible(), true);
    assert.equal(await record.getByText(EMPLOYEE_NUMBER, {exact: true}).isVisible(), true);
  }
  async function shot(name, width) {
    await page.setViewportSize({width, height: 900}); await secretFree();
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), true);
    await capture(name); result.screenshots.push(name + '.png');
  }
  function searchPolicy(response) {
    assert.equal(response.headers()['cache-control'], 'no-store');
    assert.equal(response.headers()['referrer-policy'], 'no-referrer');
  }
  async function noReferer(request) {
    assert.equal(Object.hasOwn(await request.allHeaders(), 'referer'), false);
  }
  async function search(number, currentValue, coldAsset = false) {
    const form = page.getByRole('main').locator('form[method="get"]');
    if (await form.count() !== 1) {
      const error = new Error('PEOPLE_SEARCH_FORM_MISSING');
      error.code = 'PEOPLE_SEARCH_FORM_MISSING';
      throw error;
    }
    assert.equal(await form.getAttribute('action'), directory);
    const field = form.locator('input[name="employee_number"]');
    assert.equal(await field.count(), 1);
    const label = await field.evaluateHandle(input => [...input.labels].find(label =>
      label.textContent.trim().includes('사번')) ?? null);
    assert.equal(await label.asElement()?.isVisible(), true);
    await label.dispose();
    assert.equal(await field.inputValue(), currentValue);
    const submit = form.locator('button:not([type]),button[type="submit"],input[type="submit"]');
    assert.equal(await submit.count(), 1); assert.equal(await submit.isVisible(), true);
    const path = searchPath(directory, number);
    await field.fill(number);
    const asset = coldAsset ? page.waitForResponse(r => r.url() === origin + '/assets/workspace.css' &&
      r.request().resourceType() === 'stylesheet') : null;
    if (coldAsset) {
      const cdp = await page.context().newCDPSession(page);
      await cdp.send('Network.clearBrowserCache');
      await cdp.detach();
    }
    expectDocument('GET', path, 200, false);
    const waiting = page.waitForResponse(r => r.url() === origin + path && r.request().isNavigationRequest());
    await tabTo(submit, 48); await page.keyboard.press('Enter');
    const response = await waiting;
    await page.waitForURL(origin + path);
    assert.equal(response.status(), 200); assert.equal(response.request().redirectedFrom(), null);
    assert.deepEqual([...new URL(page.url()).searchParams], [['employee_number', number]]);
    searchPolicy(response);
    assert.equal(await page.locator('form[method="get"] input[name="employee_number"]').inputValue(), number);
    if (asset) {
      const response = await asset;
      assert.equal(response.status(), 200);
      await noReferer(response.request());
      result.search_asset_referrer_absent = true;
    }
    return path;
  }
  async function submit(selector, label, path, destination, phase, values = {}) {
    const form = page.locator(selector); assert.equal(await form.count(), 1);
    assert.equal(await form.getAttribute('method'), 'post'); assert.equal(await form.getAttribute('action'), path);
    const fields = await form.evaluate(f => [...new FormData(f)].filter(([name]) => name !== 'csrf_proof').sort());
    assert.equal(new Set(fields.map(([name]) => name)).size, fields.length);
    assert.equal(fields.every(([, value]) => typeof value === 'string'), true);
    const command = id(fields.find(([name]) => name === 'command_id')?.[1]);
    const recovery = destination(command);
    await exchange({phase: 'PEOPLE_ACTION_READY', action_phase: phase, command_id: command, fields});
    expectMutation(path); expectDocument('POST', path, 303, false); expectDocument('GET', recovery, 200, true);
    const waiting = page.waitForResponse(r => r.url() === origin + path && r.request().method() === 'POST');
    const button = form.getByRole('button', {name: label, exact: true});
    await tabTo(button, 48); await page.keyboard.press('Enter');
    const response = await waiting;
    assert.equal(response.status(), 303); assert.equal(response.request().redirectedFrom(), null);
    const raw = response.request().postData(); assert.equal(typeof raw, 'string');
    const sent = new URLSearchParams(raw);
    assert.equal(sent.getAll('csrf_proof').length, 1); assert.ok(sent.get('csrf_proof').length > 0);
    assert.deepEqual([...sent].filter(([name]) => name !== 'csrf_proof').sort(), fields);
    assert.equal(Buffer.byteLength(raw), Math.min(Buffer.byteLength(raw), 8192));
    await page.waitForURL(origin + recovery);
    result.mutations.push({path, command, status: response.status(), body_sha256: crypto.createHash('sha256').update(raw).digest('hex')});
    const w = await witness(phase, {command_id: command, ...values});
    return {command, path: recovery, witness: w};
  }
  async function policyCommand(kind, action, entryLink) {
    const install = kind === 'install';
    const revoke = kind === 'revoke';
    const actionLabel = action === 'read' ? '열람' : '등록';
    const formPath = install ? policy + '/install' : policy + `/${action}/${kind}`;
    const label = install ? '사람 등록·열람 권한' : `사람 ${actionLabel} 권한 ${revoke ? '회수' : '연결'}`;
    await open(formPath, 200, entryLink ?? page.getByRole('link', {name: label, exact: true}));
    if (!install && !revoke) {
      await page.getByLabel('종료 날짜와 시각 (한국 표준시, UTC+09:00)', {exact: true})
        .fill(new Date(Date.now() + 86400000 + 9 * 3600000).toISOString().slice(0, 16));
    }
    const operation = install ? 'InstallPeopleDirectoryCatalogV1' : revoke ? 'RevokePeopleDirectoryV1' : 'GrantPeopleDirectoryV1';
    const phase = install ? 'PEOPLE_INSTALLED' : `PEOPLE_${action.toUpperCase()}_${revoke ? 'REVOKED' : 'GRANTED'}`;
    const path = install ? policy + '/catalog' : revoke ? policy + `/${action}/grants/${id(result[action + '_assignment'])}/revoke` : policy + `/${action}/grants`;
    const submitted = await submit(`form[data-policy-operation="${operation}"]`,
      install ? '권한 설정 준비' : label, path, command => policy + `/requests/${kind}/${command}`, phase, {action: action ?? null});
    assert.equal(await page.locator('[data-policy-outcome="committed"]').isVisible(), true);
    if (kind === 'grant') result[action + '_assignment'] = id(submitted.witness.assignment_id);
    return submitted;
  }

  // Starts on the genuine Company workspace after the unchanged Payroll journey.
  await exchange({phase: 'PEOPLE_READY'});
  assert.equal(page.url(), origin + workspace);
  assert.equal(await page.getByRole('link', {name: '사람', exact: true}).count(), 0);
  assert.equal(await page.getByRole('main').getByRole('link', {name: '사람 등록', exact: true}).count(), 0);
  assert.equal(await page.getByRole('main').locator(`a[href="${directory}/new"]`).count(), 0);
  await policyCommand('install', undefined, await nextTask('사람 권한 설정 시작', policy + '/install'));
  await policyCommand('grant', 'read');
  await open(directory, 200, await directoryNavigation());
  assert.equal(await page.getByRole('heading', {name: '사람', exact: true, level: 1}).isVisible(), true);
  assert.equal(await page.getByRole('link', {name: '사람 등록', exact: true}).count(), 0);
  assert.equal(await page.locator('[data-people-record]').count(), 0);
  assert.equal(await page.getByRole('main').getByRole('link', {name: '목록 처음부터 보기', exact: true}).count(), 0);
  assert.equal(await page.locator('.people-empty').locator('a,button,[role="button"]').count(), 0);
  await header('PEOPLE_HEADER_READ_ONLY');
  await shot('people-directory-320', 320);
  await open(directory + '/new', 404); await witness('PEOPLE_READ_ONLY'); result.read_only = true;
  await open(workspace);
  await policyCommand('grant', 'create', await nextTask('사람 등록 권한 연결', policy + '/create/grant'));
  await open(directory, 200, await directoryNavigation());
  assert.equal(await page.locator('.people-empty').count(), 1);
  assert.equal(await page.getByRole('main').getByRole('link', {name: '목록 처음부터 보기', exact: true}).count(), 0);
  const emptyCreate = page.getByRole('main').getByRole('link', {name: '사람 등록', exact: true});
  assert.equal(await emptyCreate.count(), 1);
  assert.equal(await emptyCreate.getAttribute('href'), directory + '/new');
  await witness('PEOPLE_DIRECTORY_CREATE_READY');
  await open(directory + '/new', 200, emptyCreate);
  await witness('PEOPLE_REGISTRATION_OPENED');
  await open(workspace);
  await shot('people-workspace-next-task', 1440);
  await shot('people-workspace-next-task-320', 320);
  await witness('PEOPLE_WORKSPACE_REOPENED');
  await open(directory + '/new', 200, await nextTask('사람 등록', directory + '/new'));
  await witness('PEOPLE_REGISTRATION_REOPENED');
  await page.getByLabel('이름', {exact: true}).fill(LEGAL_NAME);
  await page.getByLabel('사번', {exact: true}).fill(EMPLOYEE_NUMBER);
  await noBusinessStorage();
  assert.equal(await page.getByText('사람 목록에 등록합니다. 고용과 발령은 별도로 승인해야 합니다.', {exact: true}).isVisible(), true);
  await header('PEOPLE_HEADER_REGISTRATION');
  await shot('people-registration-desktop', 1440);
  const prepared = await submit('form[data-people-operation="prepare"]', '등록 내용 확인', directory + '/requests',
    command => directory + '/requests/' + command, 'PEOPLE_PREPARED');
  result.command = prepared.command; pendingPath = prepared.path;
  await requestView('pending');
  await open(pendingPath); await requestView('pending');
  await witness('PEOPLE_PENDING_REOPENED', {command_id: result.command}); result.pending_reopened = true;
  await header('PEOPLE_HEADER_PENDING');
  await shot('people-pending-320', 320);
  const committed = await submit('form[data-people-operation="execute"]', '등록 확정', pendingPath + '/execute',
    () => pendingPath, 'PEOPLE_COMMITTED');
  result.employee = id(committed.witness.employee_id); result.person = id(committed.witness.person_id);
  await requestView('committed');
  await open(pendingPath); await requestView('committed');
  await witness('PEOPLE_RECEIPT_REOPENED', {command_id: result.command}); result.receipt_reopened = true;
  detailPath = directory + '/' + result.employee;
  await open(detailPath, 200, page.getByRole('link', {name: '등록한 사람 보기', exact: true}));
  await detailView();
  await witness('PEOPLE_DETAIL'); await shot('people-detail-desktop', 1440); await shot('people-detail-320', 320);
  await open(detailPath); await detailView(); await witness('PEOPLE_DETAIL_REOPENED'); result.detail_reopened = true;
  await header('PEOPLE_HEADER_DETAIL');
  await open(directory, 200, await directoryNavigation());
  const entry = page.locator(`[data-people-record="${result.employee}"]`);
  assert.equal(await page.locator('[data-people-record]').count(), 1); assert.equal(await entry.isVisible(), true);
  assert.equal(await entry.getByRole('link', {name: LEGAL_NAME, exact: true}).getAttribute('href'), detailPath);
  assert.equal(await entry.getByText(EMPLOYEE_NUMBER, {exact: true}).isVisible(), true);
  await witness('PEOPLE_LIST');
  const matchPath = await search(EMPLOYEE_NUMBER, '', true);
  const match = page.locator(`[data-people-record="${result.employee}"]`);
  assert.equal(await page.locator('[data-people-record]').count(), 1);
  assert.equal(await match.getByText(EMPLOYEE_NUMBER, {exact: true}).isVisible(), true);
  const matchLink = match.getByRole('link', {name: LEGAL_NAME, exact: true});
  assert.equal(await matchLink.getAttribute('href'), detailPath);
  await shot('people-search-match-320', 320);
  await witness('PEOPLE_SEARCH_MATCH'); result.search_match = true;
  const searchedDetail = await open(detailPath, 200, matchLink);
  await noReferer(searchedDetail.request()); result.search_detail_referrer_absent = true;
  await detailView(); await witness('PEOPLE_SEARCH_DETAIL');
  await open(detailPath); await detailView();
  await witness('PEOPLE_SEARCH_DETAIL_REOPENED'); result.search_detail_reopened = true;
  const searchedAgain = await open(matchPath);
  searchPolicy(searchedAgain);
  assert.equal(await page.locator(`[data-people-record="${result.employee}"]`).count(), 1);
  await witness('PEOPLE_SEARCH_MATCH_REOPENED');
  await search(NONMATCHING_NUMBER, EMPLOYEE_NUMBER);
  assert.equal(await page.locator('[data-people-record]').count(), 0);
  const emptySearch = page.locator('.people-empty');
  assert.equal(await emptySearch.count(), 1);
  assert.match(await emptySearch.innerText(), /(?:현재|이) 회사/);
  assert.match(await emptySearch.innerText(), /일치/);
  await shot('people-search-miss-320', 320);
  await witness('PEOPLE_SEARCH_MISS'); result.search_miss = true;
  const clear = page.getByRole('main').getByRole('link', {name: '검색 지우기', exact: true});
  assert.equal(await clear.count(), 1);
  await open(directory, 200, clear);
  assert.equal(await page.locator(`[data-people-record="${result.employee}"]`).count(), 1);
  assert.equal(await page.locator('form[method="get"] input[name="employee_number"]').inputValue(), '');
  assert.equal(await page.getByRole('main').getByRole('link', {name: '검색 지우기', exact: true}).count(), 0);
  await witness('PEOPLE_SEARCH_CLEAR'); result.search_clear = true;
  await open(workspace); await policyCommand('revoke', 'read');
  await open(directory, 404); await open(detailPath, 404); await witness('PEOPLE_READ_DENIED'); result.read_revoked = true;
  const deniedSearch = await open(matchPath, 404);
  searchPolicy(deniedSearch);
  await witness('PEOPLE_SEARCH_READ_DENIED'); result.search_denied = true;
  result.search_response_policy = true;
  await open(pendingPath);
  await requestView('committed');
  assert.equal(await page.getByRole('link', {name: '등록한 사람 보기', exact: true}).count(), 0);
  await witness('PEOPLE_OWN_RECEIPT', {command_id: result.command}); result.create_only_receipt = true;
  await header('PEOPLE_HEADER_CREATE_ONLY');
  await open(workspace); await policyCommand('revoke', 'create');
  await open(workspace);
  assert.equal(await page.getByRole('main').getByRole('link', {name: '사람 등록', exact: true}).count(), 0);
  assert.equal(await page.getByRole('main').locator(`a[href="${directory}/new"]`).count(), 0);
  await nextTask('사람 등록 권한 연결', policy + '/create/grant');
  await open(directory + '/new', 404);
  await open(pendingPath, 404); await witness('PEOPLE_RECEIPT_DENIED'); result.create_revoked = true;
  await noBusinessStorage(); result.no_local_business_storage = true;
  result.keyboard = true; result.reflow_320 = true;
  assert.equal(validEvidence(result), true); return result;
}
module.exports = {expectedNativeHeaders, completeNativeHeaders, runPeopleJourney, validEvidence, expectedDocuments, deniedProjectionSafe, PHASES, LEGAL_NAME, EMPLOYEE_NUMBER};
