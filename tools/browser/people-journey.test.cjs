'use strict';
// Evidence machinery controls only. These values never enter the product or
// provision business records; they cannot qualify the actual browser journey.
const test = require('node:test');
const assert = require('node:assert/strict');
const {expectedNativeHeaders, completeNativeHeaders, validEvidence, expectedDocuments, deniedProjectionSafe, PHASES, LEGAL_NAME} = require('./people_journey.cjs');
const {completeDocuments, completeMutations, observeDocuments, observeMutations} = require('./company.cjs');
const {EventEmitter} = require('node:events');
const uuid = n => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;
// Classifier-only positive record; never published as browser acceptance evidence.
function headerWitness(expected) {
  return {kind: 'REAL_NATIVE_HEADER_BROWSER_CHECK', phase: expected.phase, url: expected.url,
    allowed_paths: [...expected.paths].sort(), current_path: expected.currentPath ?? null,
    payroll_path: expected.payrollPath ?? null,
    widths: [320, 680, 681, 1280].map(width => ({width, header_height: 90, main_top: 90,
      title_top: 160, no_overflow: true, open_no_overflow: width <= 680 ? true : null,
      routes_exact: true, current_exact: true, inactive_hidden: true})),
    enter_opened: true, space_closed: true, closed_focus_safe: true, resize_focus_safe: true,
    values_preserved: true, location_preserved: true, no_product_script: true,
    unique_ids: true, network_requests: 0};
}

function evidence() {
  const r = {company: uuid(1), account: uuid(2), command: uuid(3), employee: uuid(4), person: uuid(4),
    read_assignment: uuid(5), create_assignment: uuid(6), checkpoints: [], mutations: [],
    screenshots: ['people-directory-320.png', 'people-workspace-next-task.png',
      'people-workspace-next-task-320.png', 'people-registration-desktop.png', 'people-pending-320.png',
      'people-detail-desktop.png', 'people-detail-320.png', 'people-search-match-320.png',
      'people-search-miss-320.png']};
  r.checkpoints = PHASES.map(phase => ({phase, company: r.company, account: r.account, owner_effects_verified: true}));
  for (const flag of ['keyboard', 'reflow_320', 'read_only', 'pending_reopened', 'receipt_reopened',
    'detail_reopened', 'search_match', 'search_detail_reopened', 'search_miss', 'search_clear',
    'search_denied', 'search_asset_referrer_absent', 'search_detail_referrer_absent',
    'search_response_policy', 'read_revoked', 'create_only_receipt', 'create_revoked',
    'no_local_business_storage']) r[flag] = true;
  const p = `/companies/${r.company}/policy/people-directory`, d = `/companies/${r.company}/people`;
  r.mutations = [p + '/catalog', p + '/read/grants', p + '/create/grants', d + '/requests',
    d + `/requests/${r.command}/execute`, p + `/read/grants/${r.read_assignment}/revoke`,
    p + `/create/grants/${r.create_assignment}/revoke`].map((path, i) => ({path, command: i === 3 || i === 4 ? r.command : uuid(10 + i), status: 303, body_sha256: 'a'.repeat(64)}));
  r.header_origin = 'https://localhost:1234';
  r.native_headers = expectedNativeHeaders(r).map(headerWitness);
  return r;
}
test('positive People evidence control', () => assert.equal(validEvidence(evidence()), true));
for (let i = 0; i < PHASES.length; i++) {
  test(`missing ${PHASES[i]} refused`, () => { const r = evidence(); r.checkpoints.splice(i, 1); assert.equal(validEvidence(r), false); });
  test(`unverified ${PHASES[i]} refused`, () => { const r = evidence(); r.checkpoints[i].owner_effects_verified = false; assert.equal(validEvidence(r), false); });
}
for (let i = 0; i < 7; i++) {
  test(`omitted mutation ${i} refused`, () => { const r = evidence(); r.mutations.splice(i, 1); assert.equal(validEvidence(r), false); });
  test(`wrong mutation destination ${i} refused`, () => { const r = evidence(); r.mutations[i].path += '/wrong'; assert.equal(validEvidence(r), false); });
}
test('retry with newly allocated command refused', () => { const r = evidence(); r.mutations[4].command = uuid(99); assert.equal(validEvidence(r), false); });
test('wrong Account witness refused', () => { const r = evidence(); r.checkpoints[0].account = uuid(99); assert.equal(validEvidence(r), false); });
test('missing screenshot refused', () => { const r = evidence(); r.screenshots.pop(); assert.equal(validEvidence(r), false); });
test('unobserved permission loss refused', () => { const r = evidence(); r.create_revoked = 'true'; assert.equal(validEvidence(r), false); });
test('document plan binds exact command, record and action identities', () => {
  const r = evidence(), rows = expectedDocuments(r);
  assert.equal(rows.length, 46);
  assert.equal(rows.filter(x => x.method === 'POST').length, 7);
  assert.equal(rows.filter(x => x.redirected).length, 7);
  assert.equal(rows.filter(x => x.status === 404).length, 6);
  assert.equal(rows.filter(x => x.path.includes('?employee_number=')).length, 4);
  assert.equal(rows.filter(x => x.path.includes('?employee_number=') && x.status === 404).length, 1);
});
function aggregate() {
  const people = evidence(), company = people.company, command = uuid(90);
  const prefix = ['/', '/account/register', '/account', '/account', '/account/companies/new',
    `/account/companies/requests/${command}`, `/account/companies/requests/${command}`,
    `/companies/${company}`, `/account/companies/requests/${command}`, '/account',
    `/companies/${company}`, `/companies/${company}/policy/payroll-read/install`];
  const baseMutations = ['/api/v2/auth/registration/start', '/api/v2/auth/registration/finish', '/api/v2/companies/enroll'];
  const policyMutations = [1, 2, 3].map(n => '/policy-control/' + n);
  const paths = baseMutations.concat(policyMutations, people.mutations.map(m => m.path));
  const r = {people_entry: true, policy_entry: true, org_id: company, command_id: command, people,
    document_failures: 0, unexpected_mutations: 0, policy_documents: [],
    people_documents: expectedDocuments(people), policy_expected_mutations: policyMutations,
    people_expected_mutations: people.mutations.map(m => m.path)};
  r.documents = prefix.map(path => ({method: 'GET', path, status: 200, redirected: false}))
    .concat(r.people_documents).map((row, i) => ({...row, ordinal: i + 1, url_exact: true}));
  r.mutations = paths.map((path, i) => ({ordinal: i + 1, method: 'POST', path}));
  r.posts = Object.fromEntries(paths.map(path => [path, 1])); return r;
}
test('combined census positive control', () => { const r = aggregate(); assert.equal(completeDocuments(r), true); assert.equal(completeMutations(r), true); });
for (let i = 0; i < 46; i++) test(`missing People document ${i} refused`, () => {
  const r = aggregate(); r.documents.splice(12 + i, 1); assert.equal(completeDocuments(r), false);
});
test('document observer binds the complete planned search query and refuses extra keys', () => {
  const origin = 'https://localhost:1234', planned = aggregate();
  const index = planned.documents.findIndex(row => row.path.includes('?employee_number='));
  assert.ok(index >= 12);
  function observed(suffix) {
    const r = aggregate(), page = new EventEmitter(), frame = {};
    r.documents.length = index;
    page.mainFrame = () => frame;
    observeDocuments(page, origin, r);
    const request = {frame: () => frame, resourceType: () => 'document',
      isNavigationRequest: () => true, url: () => origin + planned.documents[index].path + suffix,
      method: () => 'GET', redirectedFrom: () => null};
    const response = {request: () => request, status: () => 200, url: request.url};
    page.emit('request', request); page.emit('response', response);
    return r;
  }
  const exact = observed('');
  assert.equal(exact.document_failures, 0);
  assert.equal(exact.documents.at(-1).path, planned.documents[index].path);
  const extra = observed('&extra=1');
  assert.ok(extra.document_failures > 0);
  assert.equal(extra.documents.at(-1).path, '<unexpected>');
});
test('missing aggregate mutation refused', () => { const r = aggregate(); r.mutations.pop(); assert.equal(completeMutations(r), false); });
test('unplanned extra People mutation refused', () => { const r = aggregate(); r.mutations.push({...r.mutations.at(-1), ordinal: 14}); assert.equal(completeMutations(r), false); });
test('People mutation observer refuses external origins even with matching path', () => {
  const context = new EventEmitter(), r = aggregate(), path = r.people_expected_mutations[0];
  observeMutations(context, 'https://localhost:1234', r);
  context.emit('request', {method: () => 'POST', url: () => 'https://outside.invalid' + path});
  assert.equal(r.unexpected_mutations, 1); assert.equal(r.mutations.at(-1).path, '<unexpected>');
});
test('denied projection checks decoded text, attributes and escaped response content', () => {
  const safe = {html: '<p>열 수 없습니다</p>', visible: '열 수 없습니다', text: '열 수 없습니다', attributes: []};
  assert.equal(deniedProjectionSafe(safe, [LEGAL_NAME, '회사 이름']), true);
  for (const field of ['html', 'visible', 'text']) {
    assert.equal(deniedProjectionSafe({...safe, [field]: LEGAL_NAME}, [LEGAL_NAME]), false);
    assert.equal(deniedProjectionSafe({...safe, [field]: '김하늘 &lt;연구 &amp; 운영&gt;'}, [LEGAL_NAME]), false);
    assert.equal(deniedProjectionSafe({...safe, [field]: '회사 이름'}, [LEGAL_NAME, '회사 이름']), false);
  }
  assert.equal(deniedProjectionSafe({...safe, attributes: [LEGAL_NAME]}, [LEGAL_NAME]), false);
  assert.equal(deniedProjectionSafe({...safe, text: undefined}, [LEGAL_NAME]), false);
});

// Additive header requirements do not replace any original evidence controls.
test('People mandatory header positive control', () => assert.equal(completeNativeHeaders(evidence()), true));
test('People header expected policy and exact current routes', () => {
  const r = evidence(), w = '/companies/' + r.company, d = w + '/people';
  const e = expectedNativeHeaders(r);
  assert.deepEqual(e.map(row => row.phase), ['PEOPLE_HEADER_READ_ONLY', 'PEOPLE_HEADER_REGISTRATION',
    'PEOPLE_HEADER_PENDING', 'PEOPLE_HEADER_DETAIL', 'PEOPLE_HEADER_CREATE_ONLY']);
  assert.deepEqual(e.map(row => row.currentPath ?? null), [d, d + '/new', null, null, null]);
  assert.deepEqual(e[0].paths, ['/account', w, w + '/policy', d]);
  assert.deepEqual(e[4].paths, ['/account', w, w + '/policy', d + '/new']);
  assert.deepEqual(e.map(row => row.url), [d, d + '/new', d + '/requests/' + r.command,
    d + '/' + r.employee, d + '/requests/' + r.command].map(path => r.header_origin + path));
});
for (let i = 0; i < 5; i++) {
  test(`missing mandatory People header ${i} refused by full evidence`, () => {
    const r = evidence(); r.native_headers.splice(i, 1); assert.equal(validEvidence(r), false);
  });
  test(`unpreserved form at People header ${i} refused`, () => {
    const r = evidence(); r.native_headers[i].values_preserved = false; assert.equal(validEvidence(r), false);
  });
  test(`unexpected request at People header ${i} refused`, () => {
    const r = evidence(); r.native_headers[i].network_requests = 1; assert.equal(validEvidence(r), false);
  });
  test(`wrong People header ${i} location refused`, () => {
    const r = evidence(); r.native_headers[i].url += '/wrong'; assert.equal(validEvidence(r), false);
  });
  test(`unpermitted People header ${i} route refused`, () => {
    const r = evidence(); r.native_headers[i].allowed_paths.push('/companies/foreign/people'); assert.equal(validEvidence(r), false);
  });
}
test('duplicate People header refused', () => { const r = evidence(); r.native_headers.push(r.native_headers[0]); assert.equal(validEvidence(r), false); });
test('reordered People header refused', () => { const r = evidence(); [r.native_headers[0], r.native_headers[1]] = [r.native_headers[1], r.native_headers[0]]; assert.equal(validEvidence(r), false); });
test('missing entire People header history refused', () => { const r = evidence(); delete r.native_headers; assert.equal(validEvidence(r), false); });
