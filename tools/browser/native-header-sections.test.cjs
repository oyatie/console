'use strict';
// Evidence machinery only. These literal records never enter the product and
// are not browser, policy, accessibility, fuzz, chaos, or durability evidence.
const test = require('node:test');
const assert = require('node:assert/strict');
const {validSectionEvidence, validHeaderEvidence, collectSectionSnapshot} = require('./native_header.cjs');
const {expectedNativeHeaders: companyHeaders} = require('./company.cjs');
const {expectedNativeHeaders: peopleHeaders} = require('./people_journey.cjs');
const w = '/companies/00000000-0000-4000-8000-000000000001';
const d = w + '/people', p = w + '/policy', payroll = w + '/payroll';

function detailExpectation() {
  const groups = () => [
    {label: '사람과 조직', landmark: '사람과 조직 탐색', paths: [d, d + '/new']},
    {label: '관리', landmark: '회사 관리', paths: [w, p]},
  ];
  return {phase: 'DETAIL_CONTROL', url: 'https://localhost:1234' + d + '/record',
    paths: ['/account', w, p, d, d + '/new'], currentPath: null, locationPath: d,
    groups: {desktop: groups(), mobile: groups()}, boundTitle: true};
}

// Literal positive snapshot, independent of collector and classifier builders.
function detailSnapshot() {
  const desktop = {
    structure: [
      {tag: 'p', label: '사람과 조직'},
      {tag: 'nav', landmark: '사람과 조직 탐색', paths: [d, d + '/new']},
      {tag: 'p', label: '관리'},
      {tag: 'nav', landmark: '회사 관리', paths: [w, p]},
      {tag: 'nav', landmark: '계정 탐색', paths: ['/account']},
    ],
    group_labels: ['사람과 조직', '관리'],
    landmarks: [
      {tag: 'nav', landmark: '사람과 조직 탐색', paths: [d, d + '/new']},
      {tag: 'nav', landmark: '회사 관리', paths: [w, p]},
      {tag: 'nav', landmark: '계정 탐색', paths: ['/account']},
    ],
    paths: [d, d + '/new', w, p, '/account'],
    markers: [{tag: 'a', href: d, value: 'location'}], shortcuts: [],
  };
  return {kind: 'RAW_NATIVE_SECTION_SNAPSHOT_V2', region_counts: {desktop: 1, mobile: 1},
    desktop, mobile: {...structuredClone(desktop), menu_count: 1, body_count: 1, native_disclosure: true, body_inside_disclosure: true},
    outside_markers: []};
}

function payrollExpectation() {
  return {phase: 'PAYROLL_CONTROL', url: 'https://localhost:1234' + payroll,
    paths: ['/account', payroll], currentPath: payroll, locationPath: null, payrollPath: payroll,
    groups: {desktop: [{label: '사람과 조직', landmark: '사람과 조직 탐색', paths: [payroll]}], mobile: []}, boundTitle: true};
}
function payrollSnapshot() {
  const selected = [{tag: 'a', href: payroll, value: 'page'}];
  return {kind: 'RAW_NATIVE_SECTION_SNAPSHOT_V2', region_counts: {desktop: 1, mobile: 1},
    desktop: {structure: [{tag: 'p', label: '사람과 조직'},
      {tag: 'nav', landmark: '사람과 조직 탐색', paths: [payroll]},
      {tag: 'nav', landmark: '계정 탐색', paths: ['/account']}],
      group_labels: ['사람과 조직'], landmarks: [{tag: 'nav', landmark: '사람과 조직 탐색', paths: [payroll]},
        {tag: 'nav', landmark: '계정 탐색', paths: ['/account']}],
      paths: [payroll, '/account'], markers: structuredClone(selected), shortcuts: []},
    mobile: {structure: [{tag: 'nav', landmark: '계정 탐색', paths: ['/account']}],
      group_labels: [], landmarks: [{tag: 'nav', landmark: '계정 탐색', paths: ['/account']}],
      paths: [payroll, '/account'], markers: structuredClone(selected),
      shortcuts: [{tag: 'a', href: payroll, value: 'page', direct_child: true, inside_disclosure: false}],
      menu_count: 1, body_count: 1, native_disclosure: true, body_inside_disclosure: true}, outside_markers: []};
}

function headerRecord(expected, sections) {
  return {kind: 'REAL_NATIVE_HEADER_BROWSER_CHECK', phase: expected.phase, url: expected.url,
    allowed_paths: [...expected.paths].sort(), current_path: expected.currentPath ?? null,
    payroll_path: expected.payrollPath ?? null, denied_prefixes: [],
    widths: [320, 680, 681, 1280].map(width => ({width, header_height: 90, main_top: 90, title_top: 160,
      no_overflow: true, open_no_overflow: width <= 680 ? true : null, routes_exact: true, current_exact: true,
      inactive_hidden: true, visible_landmarks_unique: true, denied_hrefs_absent: true, sections: structuredClone(sections)})),
    enter_opened: true, space_closed: true, closed_focus_safe: true, resize_focus_safe: true,
    values_preserved: true, location_preserved: true, no_product_script: true, unique_ids: true, network_requests: 0};
}

test('literal descendant section positive control', () => assert.equal(validSectionEvidence(detailSnapshot(), detailExpectation()), true));
test('literal Payroll-only mobile without empty People group positive control', () => assert.equal(validSectionEvidence(payrollSnapshot(), payrollExpectation()), true));
test('full unchanged header observations plus raw V2 evidence positive control', () => assert.equal(validHeaderEvidence(headerRecord(detailExpectation(), detailSnapshot()), detailExpectation()), true));

for (const mode of ['desktop', 'mobile']) {
  for (const [label, change] of [
    ['missing location', s => s[mode].markers.pop()],
    ['wrong parent', s => s[mode].markers[0].href = d + '/new'],
    ['duplicate location', s => s[mode].markers.push({...s[mode].markers[0]})],
    ['false descendant page', s => s[mode].markers[0].value = 'page'],
    ['unknown current value', s => s[mode].markers[0].value = 'true'],
    ['empty current value', s => s[mode].markers[0].value = ''],
    ['location on account', s => s[mode].markers[0].href = '/account'],
    ['location on non-link', s => {s[mode].markers[0].tag = 'nav'; s[mode].markers[0].href = null;}],
    ['unoffered parent anchor', s => {s[mode].paths.push(w + '/foreign'); s[mode].markers[0].href = w + '/foreign';}],
    ['unknown snapshot field', s => s[mode].claimed_pass = true],
    ['reversed group order', s => {s[mode].structure = s[mode].structure.slice(2, 4).concat(s[mode].structure.slice(0, 2), s[mode].structure.slice(4));}],
    ['reversed destination order', s => s[mode].structure[1].paths.reverse()],
    ['empty group', s => {s[mode].structure[1].paths.length = 0;}],
    ['missing label', s => s[mode].structure.shift()],
    ['missing corresponding navigation', s => s[mode].structure.splice(1, 1)],
    ['extra nested group label', s => s[mode].group_labels.push('관리')],
    ['extra navigation landmark', s => s[mode].landmarks.push({tag: 'nav', landmark: 'fabricated', paths: ['/account']})],
    ['account before groups', s => s[mode].structure.unshift(s[mode].structure.pop())],
    ['link order sorted instead of observed', s => s[mode].paths.sort()],
  ]) test(`${mode} ${label} refused`, () => {
    const snapshot = detailSnapshot(); change(snapshot);
    assert.equal(validSectionEvidence(snapshot, detailExpectation()), false);
  });
  test(`${mode} exact root rejects any extra location`, () => {
    const expected = detailExpectation(), snapshot = detailSnapshot();
    expected.currentPath = d; expected.locationPath = null;
    for (const region of ['desktop', 'mobile']) snapshot[region].markers[0].value = 'page';
    assert.equal(validSectionEvidence(snapshot, expected), true);
    snapshot[mode].markers.push({tag: 'a', href: d + '/new', value: 'location'});
    assert.equal(validSectionEvidence(snapshot, expected), false);
  });
  test(`${mode} explicitly zero selection rejects invented location`, () => {
    const expected = detailExpectation(), snapshot = detailSnapshot(); expected.locationPath = null;
    for (const region of ['desktop', 'mobile']) snapshot[region].markers = [];
    assert.equal(validSectionEvidence(snapshot, expected), true);
    snapshot[mode].markers.push({tag: 'a', href: d, value: 'location'});
    assert.equal(validSectionEvidence(snapshot, expected), false);
  });
}
for (const [label, change] of [
  ['brand location', s => s.outside_markers.push({tag: 'a', href: '/', value: 'location'})],
  ['header container location', s => s.outside_markers.push({tag: 'header', href: null, value: 'location'})],
  ['duplicate desktop region', s => s.region_counts.desktop = 2],
  ['missing mobile region', s => s.region_counts.mobile = 0],
  ['missing mobile snapshot', s => delete s.mobile],
]) test(`${label} refused`, () => {const snapshot = detailSnapshot(); change(snapshot); assert.equal(validSectionEvidence(snapshot, detailExpectation()), false);});

for (const [label, change] of [
  ['duplicate shortcut', s => s.mobile.shortcuts.push({...s.mobile.shortcuts[0]})],
  ['shortcut in disclosure', s => s.mobile.shortcuts[0].inside_disclosure = true],
  ['shortcut not direct child', s => s.mobile.shortcuts[0].direct_child = false],
  ['false shortcut location', s => s.mobile.shortcuts[0].value = 'location'],
  ['empty mobile People group', s => s.mobile.structure.unshift({tag: 'p', label: '사람과 조직'}, {tag: 'nav', landmark: '사람과 조직 탐색', paths: []})],
  ['Payroll duplicated in mobile menu', s => s.mobile.structure.unshift({tag: 'p', label: '사람과 조직'}, {tag: 'nav', landmark: '사람과 조직 탐색', paths: [payroll]})],
  ['missing native disclosure', s => s.mobile.native_disclosure = false],
  ['duplicate mobile menu body', s => s.mobile.body_count = 2],
]) test(`Payroll ${label} refused`, () => {const snapshot = payrollSnapshot(); change(snapshot); assert.equal(validSectionEvidence(snapshot, payrollExpectation()), false);});

test('caller must explicitly declare null location on roots/errors', () => {const expected = detailExpectation(); delete expected.locationPath; assert.equal(validSectionEvidence(detailSnapshot(), expected), false);});
test('caller must declare groups in both modes', () => {const expected = detailExpectation(); delete expected.groups.mobile; assert.equal(validSectionEvidence(detailSnapshot(), expected), false);});
test('page and location cannot both be declared', () => {const expected = detailExpectation(); expected.currentPath = d; assert.equal(validSectionEvidence(detailSnapshot(), expected), false);});
for (let index = 0; index < 4; index++) test(`width ${index} must retain raw section evidence`, () => {
  const expected = detailExpectation(), record = headerRecord(expected, detailSnapshot()); delete record.widths[index].sections;
  assert.equal(validHeaderEvidence(record, expected), false);
});
test('old page boolean and current_path cannot conceal wrong parent location', () => {
  const expected = detailExpectation(), record = headerRecord(expected, detailSnapshot());
  record.widths[0].sections.mobile.markers[0].href = d + '/new';
  assert.equal(record.current_path, null); assert.equal(record.widths[0].current_exact, true);
  assert.equal(validHeaderEvidence(record, expected), false);
});

test('Company and Policy declarations preserve page semantics and identify typed parents', () => {
  const r = {org_id: w.split('/').pop(), policy_entry: true, header_origin: 'https://localhost:1234',
    policy: {mutations: [2, 3, 4].map(n => ({command_id: `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`}))}};
  const expected = companyHeaders(r);
  assert.deepEqual(expected.map(row => row.locationPath), [null, p, p, p, p, null, null, null, p, p, null]);
  assert.deepEqual(expected[3].groups.desktop, [{label: '사람과 조직', landmark: '사람과 조직 탐색', paths: [payroll]},
    {label: '관리', landmark: '회사 업무 탐색', paths: [w, p]}]);
  assert.deepEqual(expected[3].groups.mobile, [{label: '관리', landmark: '회사 업무 탐색', paths: [w, p]}]);
  assert.deepEqual(expected[6].groups, payrollExpectation().groups);
});
test('People declarations preserve roots and retain only authorized selected parents', () => {
  const expected = peopleHeaders({company: w.split('/').pop(), command: '00000000-0000-4000-8000-000000000003',
    employee: '00000000-0000-4000-8000-000000000004', header_origin: 'https://localhost:1234'});
  assert.deepEqual(expected.map(row => row.locationPath), [null, null, d + '/new', d, d + '/new']);
  assert.deepEqual(expected.map(row => row.currentPath ?? null), [d, d + '/new', null, null, null]);
  assert.deepEqual(expected[4].groups.desktop, [{label: '사람과 조직', landmark: '사람과 조직 탐색', paths: [d + '/new']},
    {label: '관리', landmark: '회사 관리', paths: [w, p]}]);
  assert.deepEqual(expected[4].groups.mobile, expected[4].groups.desktop);
});

test('raw section DOM collector controls under pinned Chromium, never business acceptance', async t => {
  const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
  const driver = process.env.CONSOLE_COMPANY_BROWSER_DRIVER;
  assert.equal(typeof driver, 'string', 'source-bound browser stage prerequisite required');
  assert.equal(path.isAbsolute(driver), true);
  const runtime = path.join(path.dirname(driver), 'runtime');
  const pin = {
    'darwin-arm64': ['chrome-headless-shell-mac-arm64/chrome-headless-shell', 'a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
    'linux-x64': ['chrome-headless-shell-linux64/chrome-headless-shell', 'ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e'],
  }[process.platform + '-' + process.arch];
  assert.ok(pin, 'reviewed Chromium platform required');
  const executable = path.join(runtime, 'browser', pin[0]);
  assert.equal(crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex'), pin[1]);
  assert.equal(require(path.join(runtime, 'node_modules/playwright/package.json')).version, '1.63.0');
  assert.equal(require(path.join(runtime, 'node_modules/playwright-core/package.json')).version, '1.63.0');
  const {chromium} = require(path.join(runtime, 'node_modules/playwright'));
  const browser = await chromium.launch({headless: true, executablePath: executable});
  t.after(() => browser.close());
  const page = await browser.newPage();
  // Synthetic navigation markup tests observation machinery only. No Account,
  // Company, Person, command, API, source, or database is provisioned.
  const navigation = `<p class="nav-group">사람과 조직</p><nav aria-label="사람과 조직 탐색">
    <a href="${d}" aria-current="location">사람</a><a href="${d}/new">사람 등록</a></nav>
    <p class="nav-group">관리</p><nav aria-label="회사 관리"><a href="${w}">회사 업무 공간</a><a href="${p}">권한 관리</a></nav>
    <nav class="native-account-nav" aria-label="계정 탐색"><a href="/account">내 계정 · 업무 공간 선택</a></nav>`;
  const fixture = `<header><a class="brand" href="/">Console</a><div data-native-navigation="desktop">${navigation}</div>
    <div data-native-navigation="mobile"><details class="native-mobile-menu"><summary>업무 탐색</summary>
    <div class="native-mobile-menu-body">${navigation}</div></details></div></header>`;
  const collect = () => page.locator('header').evaluate(collectSectionSnapshot);
  await t.test('literal markup yields the independent literal snapshot', async () => {
    await page.setContent(fixture); assert.deepEqual(await collect(), detailSnapshot());
  });
  for (const mode of ['desktop', 'mobile']) {
    for (const change of ['missing-location', 'wrong-parent', 'duplicate-location', 'false-page', 'non-link-location',
      'account-location', 'empty-group', 'reversed-groups', 'reversed-destinations', 'nested-label']) {
      await t.test(`${mode} DOM ${change} is observed and refused`, async () => {
        await page.setContent(fixture);
        await page.locator(`[data-native-navigation="${mode}"]`).evaluate((outer, {change, d}) => {
          const body = outer.querySelector('.native-mobile-menu-body') ?? outer;
          const selected = body.querySelector('a[aria-current]');
          const registration = body.querySelector(`a[href="${d}/new"]`);
          if (change === 'missing-location') selected.removeAttribute('aria-current');
          if (change === 'wrong-parent') {selected.removeAttribute('aria-current'); registration.setAttribute('aria-current', 'location');}
          if (change === 'duplicate-location') registration.setAttribute('aria-current', 'location');
          if (change === 'false-page') selected.setAttribute('aria-current', 'page');
          if (change === 'non-link-location') {selected.removeAttribute('aria-current'); body.querySelector('nav').setAttribute('aria-current', 'location');}
          if (change === 'account-location') {selected.removeAttribute('aria-current'); body.querySelector('a[href="/account"]').setAttribute('aria-current', 'location');}
          if (change === 'empty-group') body.querySelector('nav').replaceChildren();
          if (change === 'reversed-groups') {const label = body.children[2], nav = body.children[3]; body.prepend(label, nav);}
          if (change === 'reversed-destinations') body.querySelector('nav').prepend(registration);
          if (change === 'nested-label') {const extra = document.createElement('span'); extra.className = 'nav-group'; extra.textContent = '관리'; body.querySelector('nav').append(extra);}
        }, {change, d});
        assert.equal(validSectionEvidence(await collect(), detailExpectation()), false);
      });
    }
  }
  for (const change of ['detached-body', 'wrong-details-owner', 'nested-details-owner']) {
    await t.test(`mobile DOM ${change} is observed and refused`, async () => {
      await page.setContent(fixture);
      const before = await collect();
      assert.equal(before.mobile.body_inside_disclosure, true);
      assert.equal(validSectionEvidence(before, detailExpectation()), true);
      await page.locator('[data-native-navigation="mobile"]').evaluate((outer, change) => {
        const menu = outer.querySelector('.native-mobile-menu');
        const body = outer.querySelector('.native-mobile-menu-body');
        if (change === 'detached-body') outer.append(body);
        else {
          const wrongMenu = document.createElement('details');
          const summary = document.createElement('summary'); summary.textContent = 'unowned';
          wrongMenu.append(summary, body);
          if (change === 'wrong-details-owner') outer.append(wrongMenu);
          else menu.append(wrongMenu);
        }
      }, change);
      const after = await collect();
      assert.equal(after.mobile.native_disclosure, true);
      assert.equal(after.mobile.menu_count, 1); assert.equal(after.mobile.body_count, 1);
      assert.equal(after.mobile.body_inside_disclosure, false);
      const withoutOwnership = snapshot => {
        const retained = structuredClone(snapshot); delete retained.mobile.body_inside_disclosure; return retained;
      };
      assert.deepEqual(withoutOwnership(after), withoutOwnership(before));
      assert.equal(validSectionEvidence(after, detailExpectation()), false);
    });
  }
  await t.test('DOM marker outside the regions cannot disappear', async () => {
    await page.setContent(fixture); await page.locator('.brand').evaluate(element => element.setAttribute('aria-current', 'location'));
    const snapshot = await collect(); assert.deepEqual(snapshot.outside_markers, [{tag: 'a', href: '/', value: 'location'}]);
    assert.equal(validSectionEvidence(snapshot, detailExpectation()), false);
  });
});

// Additive V2 controls for the independently reproduced detached-body false
// acceptance. Every V1 test and business expectation above remains retained.
for (const value of [false, null, 0, 1, 'true', undefined]) test(`exact disclosure body ownership ${String(value)} refused`, () => {
  const snapshot = detailSnapshot();
  if (value === undefined) delete snapshot.mobile.body_inside_disclosure;
  else snapshot.mobile.body_inside_disclosure = value;
  assert.equal(validSectionEvidence(snapshot, detailExpectation()), false);
});
