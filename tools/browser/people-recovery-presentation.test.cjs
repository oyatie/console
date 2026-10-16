'use strict';
// Actual committed React renderer, static presentation fixtures only: no owner,
// Account, Company, Person, business command, authorization or workflow evidence.
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const COMPANY = '00000000-0000-0000-0000-000000000101';
const COMMAND = '00000000-0000-0000-0000-000000000102';
const EMPLOYEE = '00000000-0000-0000-0000-000000000103';
const PERSON = '00000000-0000-0000-0000-000000000104';
const RECEIPT = '00000000-0000-0000-0000-000000000105';
const root = `/companies/${COMPANY}/people`, recovery = `${root}/requests/${COMMAND}`;

test('committed People runtime recovery presentation, never business acceptance', {timeout: 90000}, async t => {
  const driver = process.env.CONSOLE_COMPANY_BROWSER_DRIVER;
  assert.equal(typeof driver, 'string', 'existing isolated pinned browser stage required');
  assert.equal(path.isAbsolute(driver), true);
  const runtime = path.join(path.dirname(driver), 'runtime');
  const pin = {
    'darwin-arm64': ['chrome-headless-shell-mac-arm64/chrome-headless-shell', 'a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
    'linux-x64': ['chrome-headless-shell-linux64/chrome-headless-shell', 'ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e'],
  }[process.platform + '-' + process.arch];
  assert.ok(pin, 'reviewed Chromium platform required');
  const executable = path.join(runtime, 'browser', pin[0]);
  assert.equal(crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex'), pin[1]);
  for (const name of ['playwright', 'playwright-core'])
    assert.equal(require(path.join(runtime, `node_modules/${name}/package.json`)).version, '1.63.0');
  const assets = path.resolve(__dirname, '../../backend/crates/payroll/ui/react');
  const files = new Map(['people.js', 'people-guard.js', 'people.css'].map(name =>
    [`/assets/${name}`, fs.readFileSync(path.join(assets, name))]));
  const {chromium} = require(path.join(runtime, 'node_modules/playwright'));
  const browser = await chromium.launch({headless: true, executablePath: executable, timeout: 10000});
  t.after(() => browser.close());
  assert.equal(browser.version(), '153.0.8010.12');
  const context = await browser.newContext({serviceWorkers: 'block'});
  const page = await context.newPage();
  page.setDefaultTimeout(10000);
  const origin = 'http://127.0.0.1';
  let projection, mutations = 0;
  await context.route('**/*', async route => {
    const request = route.request(), url = new URL(request.url());
    if (request.method() !== 'GET') { mutations += 1; return route.abort(); }
    if (url.origin !== origin) return route.abort();
    if (files.has(url.pathname)) return route.fulfill({status: 200,
      contentType: url.pathname.endsWith('.css') ? 'text/css' : 'text/javascript', body: files.get(url.pathname)});
    if (url.pathname !== '/presentation-fixture') return route.abort();
    return route.fulfill({status: 200, contentType: 'text/html', body: `<!doctype html><html><head>
      <link rel="stylesheet" href="/assets/people.css"><script src="/assets/people-guard.js"></script></head><body>
      <div id="console-people-fallback">Static presentation fixture, no owning response.</div>
      <script id="console-people-bootstrap" type="application/json">${JSON.stringify({version: 1, page: projection})}</script>
      <script type="module" src="/assets/people.js"></script></body></html>`});
  });
  for (const read of [false, true]) for (const create of [false, true]) {
    const scope = {company: COMPANY, company_name: '표현 회귀 자료', company_link: true,
      directory_link: read, can_create: create, payroll_link: false, policy_link: false};
    const outcomes = [
      {kind: 'rejected', reason: '이 요청으로 등록하지 않았습니다'}, {kind: 'cancelled'}, {kind: 'expired'},
      {kind: 'pending', proof: 'presentation-only-not-authority'},
      {kind: 'committed', employee_id: EMPLOYEE, person_id: PERSON, receipt: RECEIPT, registered_at: '2026-10-12 10:00'},
    ];
    const cases = [{state: 'not-visible', page: {kind: 'request_not_visible', scope, command: COMMAND}},
      ...outcomes.map(outcome => ({state: outcome.kind, page: {kind: 'request', scope, request: {
        command: COMMAND, legal_name: '표현 자료', employee_number: '표현-1', accepted_at: '2026-10-12 09:00',
        deadline: '2026-10-13 09:00', intake_receipt: RECEIPT, expected_company_epoch: '1', outcome,
      }}}))];
    for (const fixture of cases) await t.test(`${fixture.state} read=${read} create=${create}`, async () => {
      projection = fixture.page;
      await page.goto(origin + '/presentation-fixture', {waitUntil: 'networkidle'});
      await page.locator('[data-console-react-people="mounted"]').waitFor({state: 'visible'});
      assert.equal(await page.locator('#console-people-fallback').count(), 0, 'actual runtime must promote PeopleView');
      const body = page.locator(`[data-people-outcome="${fixture.state}"]`);
      assert.equal(await body.count(), 1);
      const links = await body.locator('a').evaluateAll(elements => elements.map(e => e.getAttribute('href')).sort());
      const expected = [recovery];
      if (create && ['rejected', 'cancelled', 'expired'].includes(fixture.state)) expected.push(root + '/new');
      if (read && fixture.state === 'committed') expected.push(root + '/' + EMPLOYEE);
      assert.deepEqual(links, expected.sort(), 'recovery body must keep exactly its permitted actions and original locator');
      assert.equal(await body.locator('form').count(), fixture.state === 'pending' ? 2 : 0);
      if (fixture.state === 'pending') assert.deepEqual(
        await body.locator('form').evaluateAll(elements => elements.map(e => e.getAttribute('action')).sort()),
        [recovery + '/cancel', recovery + '/execute']);
      if (fixture.state === 'not-visible') assert.match(await body.innerText(), /등록이 실패했거나 취소되었다는 뜻이 아닙니다/);
      const navigation = ['/account', `/companies/${COMPANY}`];
      if (read) navigation.push(root);
      if (create) navigation.push(root + '/new');
      for (const mode of ['desktop', 'mobile']) {
        const region = page.locator(`[data-native-navigation="${mode}"]`);
        assert.equal(await region.count(), 1);
        assert.deepEqual(await region.locator('a').evaluateAll(elements => elements.map(e => e.getAttribute('href')).sort()),
          [...navigation].sort(), 'global authorized navigation must survive the body correction');
        assert.equal(await region.locator(`a[href*="${COMMAND}"]`).count(), 0);
      }
      assert.equal(mutations, 0, 'presentation controls must not dispatch business work');
    });
  }
});
