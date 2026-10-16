'use strict';
// Evidence machinery controls only. These values never enter the product or
// provision business records; they cannot qualify the actual browser journey.
const test = require('node:test');
const assert = require('node:assert/strict');
const {expectedNativeHeaders, completeNativeHeaders, validEvidence, expectedDocuments, expectedMountPaths, deniedProjectionSafe, PHASES, LEGAL_NAME} = require('./people_journey.cjs');
const {completeDocuments, completeMutations, observeDocuments, observeMutations} = require('./company.cjs');
const {EventEmitter} = require('node:events');
const {expectedDenialHeaders, completeDenialHeaders} = require('./people_journey.cjs');
const {CONTROL_NAMES, validControlEvidence, DECODER_CONTROLS, validDecoderEvidence, sameActions, validStorageEvidence, STORAGE_METHODS} = require('./react_people_controls.cjs');
const uuid = n => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;
// Classifier-only positive record; never published as browser acceptance evidence.
// Synthetic classifier witness only; never used by the real DOM collector.
function sectionWitness(expected) {
  const selected = expected.currentPath ?? expected.locationPath;
  const markers = selected ? [{tag: 'a', href: selected,
    value: expected.currentPath ? 'page' : 'location'}] : [];
  const region = mode => {
    const structure = expected.groups[mode].flatMap(group => [
      {tag: 'p', label: group.label},
      {tag: 'nav', landmark: group.landmark, paths: [...group.paths]},
    ]).concat([{tag: 'nav', landmark: '계정 탐색', paths: ['/account']}]);
    const shortcuts = mode === 'mobile' && expected.payrollPath ? [{tag: 'a', href: expected.payrollPath,
      value: expected.currentPath === expected.payrollPath ? 'page' : null,
      direct_child: true, inside_disclosure: false}] : [];
    return {structure, group_labels: expected.groups[mode].map(group => group.label),
      landmarks: structure.filter(node => node.tag === 'nav'),
      paths: shortcuts.map(link => link.href).concat(expected.groups[mode].flatMap(group => group.paths), ['/account']),
      markers: structuredClone(markers), shortcuts,
      ...(mode === 'mobile' ? {menu_count: 1, body_count: 1, native_disclosure: true, body_inside_disclosure: true} : {})};
  };
  return {kind: 'RAW_NATIVE_SECTION_SNAPSHOT_V2', region_counts: {desktop: 1, mobile: 1},
    desktop: region('desktop'), mobile: region('mobile'), outside_markers: []};
}

function headerWitness(expected) {
  return {kind: 'REAL_NATIVE_HEADER_BROWSER_CHECK', phase: expected.phase, url: expected.url,
    allowed_paths: [...expected.paths].sort(), current_path: expected.currentPath ?? null,
    payroll_path: expected.payrollPath ?? null, denied_prefixes: [...(expected.deniedPrefixes ?? [])].sort(),
    widths: [320, 680, 681, 1280].map(width => ({width, header_height: 90, main_top: 90,
      title_top: 160, no_overflow: true, open_no_overflow: width <= 680 ? true : null,
      routes_exact: true, current_exact: true, inactive_hidden: true, visible_landmarks_unique: true, denied_hrefs_absent: true, sections: sectionWitness(expected)})),
    enter_opened: true, space_closed: true, closed_focus_safe: true, resize_focus_safe: true,
    values_preserved: true, location_preserved: true, no_product_script: expected.clientScriptContract === undefined,
    ...(expected.clientScriptContract === undefined ? {} : {script_evidence: {guard_before_body: true, scripts: [
      {attributes: [['src', '/assets/people-guard.js']]},
      {attributes: [['id', 'console-people-bootstrap'], ['type', 'application/json']]},
      {attributes: [['src', '/assets/people.js'], ['type', 'module']]},
    ]}}), unique_ids: true, network_requests: 0};
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
  r.denied_headers = expectedDenialHeaders(r).map(headerWitness);
  r.screenshots.push(...CONTROL_NAMES.map(name => 'react-people-' + name + '-320.png'));
  r.react_controls = CONTROL_NAMES.map((name, i) => ({name, command: uuid(100 + i), form_sha256: 'a'.repeat(64),
    fallback_retained: true, form_preserved: true, focus_preserved: true, selection_preserved: true,
    scroll_preserved: true, no_mutation: true, react_mounted: false, owner_effects_verified: true,
    ime_events: 3, scroll_y: 200, render_probe_hits: 1}));
  r.react_decoder_controls = Object.entries(DECODER_CONTROLS).flatMap(([group, controls]) => controls.map((c, i) => ({
    group, name: c.name, command: group === 'terminal' ? r.command : group === 'directory' ? null : uuid(200 + i), react_mounted: c.mounted,
    fallback_retained: !c.mounted, rejection_status: !c.mounted, runtime_http_200: true, runtime_sha256: 'b'.repeat(64),
    original_bootstrap_sha256: 'c'.repeat(64), fallback_sha256: 'd'.repeat(64), original_response_unchanged: true,
    no_mutation: true, owner_effects_verified: true, current_context_preserved: true, no_business_storage: true, outside_action_control_verified: true, global_actions_preserved: !c.mounted,
    hostile_text_only: c.name === 'hostile-text', exact_large_revision: c.name === 'large-revision-string', exact_scalar_limits: c.name === 'unicode-scalar-boundary', nullable_record_rendered: c.name === 'nullable-record-fields', no_actionable_form: group === 'terminal',
  })));
  r.screenshots.push(...r.react_decoder_controls.map(c => `react-people-decoder-${c.group}-${c.name}-320.png`));
  r.react_mounts = expectedMountPaths(r);
  r.persistent_storage_observer = {product_operations:0,positive_controls:1,installed_documents:1,state_preserved:true,methods_verified:[...STORAGE_METHODS],injected_write_detections:2,injection_methods_verified:['storage:removeItem','storage:setItem']};
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
  assert.equal(rows.length, 110);
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
for (let i = 0; i < 110; i++) test(`missing People document ${i} refused`, () => {
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

test('People additive denial-header positive control', () => assert.equal(completeDenialHeaders(evidence()), true));
test('People denied-detail and denied-receipt declarations retain only Account recovery', () => {
  const r = evidence(), w = '/companies/' + r.company, d = w + '/people';
  const expected = expectedDenialHeaders(r);
  assert.deepEqual(expected.map(row => row.phase), ['PEOPLE_HEADER_DENIED_DETAIL', 'PEOPLE_HEADER_DENIED_RECEIPT']);
  assert.deepEqual(expected.map(row => row.url), [d + '/' + r.employee, d + '/requests/' + r.command].map(path => r.header_origin + path));
  for (const row of expected) {
    assert.deepEqual(row.paths, ['/account']); assert.equal(row.currentPath, null); assert.equal(row.locationPath, null);
    assert.deepEqual(row.groups, {desktop: [], mobile: []}); assert.deepEqual(row.deniedPrefixes, [w]);
  }
});
for (let i = 0; i < 2; i++) {
  test(`People omitted denial header ${i} refused by full evidence`, () => {const r = evidence(); r.denied_headers.splice(i, 1); assert.equal(validEvidence(r), false);});
  test(`People duplicated denial header ${i} refused`, () => {const r = evidence(); r.denied_headers.splice(i, 0, r.denied_headers[i]); assert.equal(validEvidence(r), false);});
  for (let width = 0; width < 4; width++) {
    test(`People denial header ${i} missing width ${width} sections refused`, () => {const r = evidence(); delete r.denied_headers[i].widths[width].sections; assert.equal(validEvidence(r), false);});
    for (const mode of ['desktop', 'mobile']) {
      test(`People denial header ${i} width ${width} ${mode} invented selection refused`, () => {
        const r = evidence(); r.denied_headers[i].widths[width].sections[mode].markers.push({tag: 'a', href: '/account', value: 'location'});
        assert.equal(r.denied_headers[i].current_path, null); assert.equal(r.denied_headers[i].widths[width].current_exact, true);
        assert.equal(validEvidence(r), false);
      });
      test(`People denial header ${i} width ${width} ${mode} invented group refused`, () => {
        const r = evidence(); r.denied_headers[i].widths[width].sections[mode].structure.unshift({tag: 'p', label: '사람과 조직'},
          {tag: 'nav', landmark: '사람과 조직 탐색', paths: ['/companies/foreign/people']});
        assert.equal(validEvidence(r), false);
      });
    }
  }
}
test('People missing additive denial history refused', () => {const r = evidence(); delete r.denied_headers; assert.equal(validEvidence(r), false);});
test('People reordered additive denial history refused', () => {const r = evidence(); r.denied_headers.reverse(); assert.equal(validEvidence(r), false);});
test('duplicate People header refused', () => { const r = evidence(); r.native_headers.push(r.native_headers[0]); assert.equal(validEvidence(r), false); });
test('reordered People header refused', () => { const r = evidence(); [r.native_headers[0], r.native_headers[1]] = [r.native_headers[1], r.native_headers[0]]; assert.equal(validEvidence(r), false); });
test('missing entire People header history refused', () => { const r = evidence(); delete r.native_headers; assert.equal(validEvidence(r), false); });

// Mandatory additive raw records propagate through the unchanged business census.
for (let i = 0; i < 5; i++) {
  for (let width = 0; width < 4; width++) test(`People header ${i} omitted raw sections at width ${width} refused`, () => {
    const r = evidence(); delete r.native_headers[i].widths[width].sections;
    assert.equal(validEvidence(r), false);
  });
}

// Mandatory V2 ownership propagates through known and separately denied history.
for (const [history, count] of [['native_headers', 5], ['denied_headers', 2]]) {
  for (let i = 0; i < count; i++) {
    for (let width = 0; width < 4; width++) {
      for (const ownership of [false, undefined]) test(`People ${history} ${i} width ${width} disclosure ownership ${String(ownership)} refused`, () => {
        const r = evidence(); const mobile = r[history][i].widths[width].sections.mobile;
        if (ownership === undefined) delete mobile.body_inside_disclosure;
        else mobile.body_inside_disclosure = ownership;
        assert.equal(validEvidence(r), false);
      });
    }
  }
}

// New independent positive/corruption controls for explicit React execution and failures.
test('React script evidence cannot silently claim script-free', () => { const r=evidence(); r.native_headers[0].no_product_script=true; assert.equal(validEvidence(r),false); });
test('injected extra script refused', () => { const r=evidence(); r.native_headers[0].script_evidence.scripts.push({attributes:[['src','/assets/extra.js']]}); assert.equal(validEvidence(r),false); });
test('inline executable script refused', () => { const r=evidence(); r.native_headers[0].script_evidence.scripts[1].attributes.pop(); assert.equal(validEvidence(r),false); });
test('missing React mount observation refused', () => { const r=evidence(); r.react_mounts.pop(); assert.equal(validEvidence(r),false); });
for(const name of CONTROL_NAMES) {
  test(`missing React failure control ${name} refused`,()=>{const r=evidence();r.react_controls=r.react_controls.filter(row=>row.name!==name);assert.equal(validEvidence(r),false);});
  test(`unverified React failure control ${name} refused`,()=>{const r=evidence();r.react_controls.find(row=>row.name===name).owner_effects_verified=false;assert.equal(validEvidence(r),false);});
}
test('zero real IME events refused',()=>{const r=evidence();r.react_controls[0].ime_events=0;assert.equal(validControlEvidence(r.react_controls),false);});
test('render failure that never reached React refused',()=>{const r=evidence();r.react_controls[4].render_probe_hits=0;assert.equal(validControlEvidence(r.react_controls),false);});
test('fallback controls cannot create business effects',()=>{const r=evidence();r.react_controls[0].no_mutation=false;assert.equal(validControlEvidence(r.react_controls),false);});

for (const [group, controls] of Object.entries(DECODER_CONTROLS)) for (const c of controls) {
  test(`missing real decoder control ${group}/${c.name} refused`, () => {
    const r=evidence(); r.react_decoder_controls=r.react_decoder_controls.filter(x=>!(x.group===group&&x.name===c.name));
    assert.equal(validEvidence(r),false);
  });
  test(`unverified decoder owner census ${group}/${c.name} refused`, () => {
    const r=evidence();r.react_decoder_controls.find(x=>x.group===group&&x.name===c.name).owner_effects_verified=false;
    assert.equal(validEvidence(r),false);
  });
}
for (const flag of ['runtime_http_200','original_response_unchanged','no_mutation','current_context_preserved','no_business_storage']) {
  test(`decoder evidence requires ${flag}`,()=>{const r=evidence();r.react_decoder_controls[1][flag]=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
}
test('decoder negative control cannot mount',()=>{const r=evidence();r.react_decoder_controls[1].react_mounted=true;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('decoder rejection must expose fixed accessible status',()=>{const r=evidence();r.react_decoder_controls[1].rejection_status=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('decoder fallback must survive rejection',()=>{const r=evidence();r.react_decoder_controls[1].fallback_retained=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('decoder baseline must prove real mount first',()=>{const r=evidence();r.react_decoder_controls[0].react_mounted=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('decoder controls require identical actually served runtime bytes',()=>{const r=evidence();r.react_decoder_controls[1].runtime_sha256='e'.repeat(64);assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('hostile text control requires literal rendering witness',()=>{const r=evidence();r.react_decoder_controls.find(x=>x.name==='hostile-text').hostile_text_only=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('large exact revisions cannot silently round',()=>{const r=evidence();r.react_decoder_controls.find(x=>x.name==='large-revision-string').exact_large_revision=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('terminal controls cannot introduce actionable forms',()=>{const r=evidence();r.react_decoder_controls.find(x=>x.group==='terminal').no_actionable_form=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});

test('supplementary Unicode limits require exact scalar-value witness',()=>{const r=evidence();r.react_decoder_controls.find(x=>x.name==='unicode-scalar-boundary').exact_scalar_limits=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('nullable record fields require visible fallback names',()=>{const r=evidence();r.react_decoder_controls.find(x=>x.name==='nullable-record-fields').nullable_record_rendered=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});

for(const flag of ['outside_action_control_verified','global_actions_preserved'])test(`decoder action observation requires ${flag}`,()=>{const r=evidence();r.react_decoder_controls[1][flag]=false;assert.equal(validDecoderEvidence(r.react_decoder_controls),false);});
test('global action census positive control refuses added outside-fallback form',()=>{const before=['<form action="/real/requests">owned</form>'];assert.equal(sameActions(before,before),true);assert.equal(sameActions(before,[...before,'<form hidden action="/real/requests">injected</form>']),false);});
test('global action census refuses hidden changed controls',()=>assert.equal(sameActions(['<button>owned</button>'],['<button hidden>changed</button>']),false));
test('global action census refuses omitted evidence',()=>assert.equal(sameActions(undefined,undefined),false));
test('positive persistent storage evidence control',()=>assert.equal(validStorageEvidence(evidence().persistent_storage_observer),true));
for(const field of ['positive_controls','installed_documents'])test(`persistent storage observer requires ${field}`,()=>{const r=evidence();r.persistent_storage_observer[field]=0;assert.equal(validEvidence(r),false);});
test('persistent storage writes are refused through the complete journey',()=>{const r=evidence();r.persistent_storage_observer.product_operations=1;assert.equal(validEvidence(r),false);});
test('changed browser store state is refused',()=>{const r=evidence();r.persistent_storage_observer.state_preserved=false;assert.equal(validEvidence(r),false);});
for(const method of STORAGE_METHODS)test(`storage observer positive control requires ${method}`,()=>{const r=evidence();r.persistent_storage_observer.methods_verified=r.persistent_storage_observer.methods_verified.filter(x=>x!==method);assert.equal(validEvidence(r),false);});

test('pending observer control must detect unrelated product write',()=>{const r=evidence();r.persistent_storage_observer.injected_write_detections=0;assert.equal(validEvidence(r),false);});
test('pending observer control must classify both transient writes as product',()=>{const r=evidence();r.persistent_storage_observer.injection_methods_verified=['storage:setItem'];assert.equal(validEvidence(r),false);});
