'use strict';
// External readonly oracle proposal. Caller supplies the real authenticated page,
// existing keyboard tabTo helper and exact authorized paths from its test case.
const assert = require('node:assert/strict');

function validHeaderEvidence(record, expected) {
  try {
    assert.equal(record.kind, 'REAL_NATIVE_HEADER_BROWSER_CHECK');
    assert.equal(record.phase, expected.phase);
    assert.equal(record.url, expected.url);
    assert.deepEqual(record.allowed_paths, [...expected.paths].sort());
    assert.equal(record.current_path, expected.currentPath ?? null);
    assert.equal(record.payroll_path, expected.payrollPath ?? null);
    assert.deepEqual(record.denied_prefixes, [...(expected.deniedPrefixes ?? [])].sort());
    assert.deepEqual(record.widths.map(x => x.width), [320, 680, 681, 1280]);
    for (const row of record.widths) {
      assert.equal(row.no_overflow, true);
      assert.equal(row.routes_exact, true);
      assert.equal(row.current_exact, true);
      assert.equal(row.inactive_hidden, true);
      assert.equal(row.visible_landmarks_unique, true);
      assert.equal(row.denied_hrefs_absent, true);
      if (row.width <= 680) {
        assert.equal(row.open_no_overflow, true);
        assert.equal(Number.isFinite(row.header_height) && row.header_height > 0 && row.header_height <= 128, true);
        assert.equal(Number.isFinite(row.main_top) && row.main_top >= 0 && row.main_top <= 128, true);
        if (expected.boundTitle) assert.equal(Number.isFinite(row.title_top) && row.title_top >= 0 && row.title_top <= 220, true);
      }
    }
    for (const key of ['enter_opened', 'space_closed', 'closed_focus_safe', 'resize_focus_safe',
      'values_preserved', 'location_preserved', 'no_product_script', 'unique_ids']) assert.equal(record[key], true);
    assert.equal(record.network_requests, 0);
    return true;
  } catch { return false; }
}

async function assertNativeHeader(page, tabTo, expected) {
  assert.equal(typeof expected.phase, 'string');
  assert.equal(expected.phase.length > 0, true);
  assert.equal(page.url(), expected.url);
  assert.equal(new Set(expected.paths).size, expected.paths.length);
  assert.equal(expected.paths.includes('/account'), true);
  if (expected.payrollPath) assert.equal(expected.paths.includes(expected.payrollPath), true);
  const originalViewport = page.viewportSize();
  await page.waitForLoadState('networkidle');
  const before = await page.evaluate(() => JSON.stringify([...document.forms].map(f =>
    [f.method, f.action, [...new FormData(f)].map(([k, v]) => [k, typeof v === 'string' ? v : [v.name, v.size, v.type]])])));
  const record = {kind: 'REAL_NATIVE_HEADER_BROWSER_CHECK', phase: expected.phase, url: expected.url,
    allowed_paths: [...expected.paths].sort(), current_path: expected.currentPath ?? null,
    payroll_path: expected.payrollPath ?? null, denied_prefixes: [...(expected.deniedPrefixes ?? [])].sort(), widths: []};
  let requests = 0;
  const observedRequest = () => { requests += 1; };
  page.on('request', observedRequest);
  const banner = page.getByRole('banner');
  const summary = banner.locator('summary').filter({hasText: /^업무 탐색$/});
  const disclosure = summary.locator('..');
  const same = (a, b, message) => assert.equal(JSON.stringify(a) === JSON.stringify(b), true, message);
  const visibleLinks = () => banner.getByRole('link').evaluateAll(links => links.map(a => a.getAttribute('href')).sort());
  const visibleCurrent = () => banner.getByRole('link').evaluateAll(links => links
    .filter(a => a.getAttribute('aria-current') === 'page').map(a => a.getAttribute('href')).sort());
  const uniqueVisibleLandmarks = async () => {
    const names = await page.getByRole('navigation').evaluateAll(elements => elements.map(e =>
      e.getAttribute('aria-label') ?? (e.getAttribute('aria-labelledby') ?? '').split(/\s+/)
        .filter(Boolean).map(id => document.getElementById(id)?.textContent.trim() ?? '').join(' ')));
    assert.equal(names.every(name => name.length > 0) && new Set(names).size === names.length, true,
      'visible navigation landmark names must be unique');
  };
  const focusIsVisible = () => page.evaluate(() => {
    const e = document.activeElement;
    return e && e !== document.body && e.getClientRects().length > 0 &&
      getComputedStyle(e).visibility !== 'hidden' &&
      !e.closest('[hidden],[inert]');
  });
  async function openMenu() {
    assert.equal(await disclosure.count(), 1);
    assert.equal(await disclosure.evaluate(e => e.open), false);
    await tabTo(summary, 48);
    await page.keyboard.press('Enter');
    assert.equal(await disclosure.evaluate(e => e.open), true);
  }
  try {
    for (const width of [320, 680, 681, 1280]) {
      await page.setViewportSize({width, height: 900});
      await page.evaluate(() => window.scrollTo(0, 0));
      assert.equal(await banner.count(), 1);
      const header = await banner.boundingBox();
      const main = await page.getByRole('main').boundingBox();
      const title = await page.getByRole('heading', {level: 1}).boundingBox();
      assert.equal(header !== null && main !== null && title !== null, true);
      // First boundary failure on today's real267px header; not a missingfuturetype.
      if (width <= 680) {
        assert.equal(header.height <= 128, true, 'native mobile header must be at most128px');
        assert.equal(main.y >= 0 && main.y <= 128, true, 'main must start within128px');
        if (expected.boundTitle) assert.equal(title.y >= 0 && title.y <= 220, true, 'baseline subject must remain near taskentry');
      }
      const desktop = banner.locator('[data-native-navigation="desktop"]');
      const mobile = banner.locator('[data-native-navigation="mobile"]');
      assert.equal(await desktop.count(), 1); assert.equal(await mobile.count(), 1);
      assert.equal(await (width <= 680 ? desktop : mobile).evaluate(e => getComputedStyle(e).display), 'none');
      assert.notEqual(await (width <= 680 ? mobile : desktop).evaluate(e => getComputedStyle(e).display), 'none');
      assert.equal(await banner.locator('form,input').count(), 0);
      const everyHref = await banner.locator('a[href]').evaluateAll(links => links.map(a => a.getAttribute('href')));
      assert.equal(everyHref.every(path => ['/', ...expected.paths].includes(path)), true, 'hiddenpresentation must not expose unpermittedroutes');
      const rawHrefs = await page.locator('a[href]').evaluateAll(links => links.map(a => a.getAttribute('href')));
      assert.equal(rawHrefs.every(path => !(expected.deniedPrefixes ?? []).some(prefix =>
        path === prefix || path.startsWith(prefix + '/') || path.startsWith(prefix + '?'))), true,
        'every DOM anchor must omit denied destinations, including hidden copies and main cards');
      await uniqueVisibleLandmarks();
      let openNoOverflow = null;
      if (width <= 680) {
        assert.equal(await summary.count(), 1);
        assert.equal(await disclosure.evaluate(e => e.open), false);
        same(await visibleLinks(), ['/', ...(expected.payrollPath ? [expected.payrollPath] : [])].sort(), 'closedmenu linkset');
        same(await visibleCurrent(), expected.currentPath === expected.payrollPath && expected.payrollPath ? [expected.payrollPath] : [], 'closedmenu currentpage');
        await openMenu();
        openNoOverflow = await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth);
        assert.equal(openNoOverflow, true, 'open mobile menu overflowed viewport');
        same(await visibleLinks(), ['/', ...expected.paths].sort(), 'openedmenu linkset');
        await uniqueVisibleLandmarks();
        same(await visibleCurrent(), expected.currentPath ? [expected.currentPath] : [], 'openedmenu currentpage');
        if (expected.payrollPath) assert.equal(await disclosure.locator(`a[href="${expected.payrollPath}"]`).count(), 0);
        await tabTo(summary, 48);
        await page.keyboard.press('Space');
        assert.equal(await disclosure.evaluate(e => e.open), false);
        // Traverse real keyboard focus; no programmatic focus/open shortcut.
        for (let i = 0; i < 48; i += 1) {
          await page.keyboard.press('Tab');
          assert.equal(await page.evaluate(() => {
            const e = document.activeElement;
            const d = e?.closest('details');
            return !d || d.open || e.tagName === 'SUMMARY';
          }), true, 'closed details descendant received keyboardfocus');
        }
      } else {
        same(await visibleLinks(), ['/', ...expected.paths].sort(), 'desktop linkset');
        same(await visibleCurrent(), expected.currentPath ? [expected.currentPath] : [], 'desktop currentpage');
      }
      const noOverflow = await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth);
      assert.equal(noOverflow, true);
      record.widths.push({width, header_height: header.height, main_top: main.y, title_top: title.y,
        no_overflow: true, open_no_overflow: openNoOverflow, routes_exact: true, current_exact: true, inactive_hidden: true, visible_landmarks_unique: true, denied_hrefs_absent: true});
    }
    await page.setViewportSize({width: 680, height: 900});
    await openMenu();
    const account = banner.getByRole('link', {name: '내 계정 · 업무 공간 선택', exact: true});
    await tabTo(account, 48);
    await page.setViewportSize({width: 681, height: 900});
    await page.keyboard.press('Tab');
    assert.equal(await focusIsVisible(), true, 'resize frommobile trapped hiddenfocus');
    await tabTo(account, 48);
    await page.setViewportSize({width: 680, height: 900});
    await page.keyboard.press('Tab');
    assert.equal(await focusIsVisible(), true, 'resize fromdesktop trapped hiddenfocus');
    assert.equal(await disclosure.evaluate(e => e.open), true);
    await tabTo(summary, 48); await page.keyboard.press('Space');
    assert.equal(await disclosure.evaluate(e => e.open), false);
    if (originalViewport) await page.setViewportSize(originalViewport);
    await page.waitForLoadState('networkidle');
    const after = await page.evaluate(() => JSON.stringify([...document.forms].map(f =>
      [f.method, f.action, [...new FormData(f)].map(([k, v]) => [k, typeof v === 'string' ? v : [v.name, v.size, v.type]])])));
    // Compare booleans only: snapshots may contain hiddenproofs; never printthem.
    assert.equal(before === after, true, 'navigation changed formstate');
    assert.equal(page.url() === expected.url, true);
    assert.equal(await page.locator('script').count(), 0);
    assert.equal(await page.evaluate(() => {
      const ids = [...document.querySelectorAll('[id]')].map(e => e.id);
      return ids.length === new Set(ids).size;
    }), true);
    Object.assign(record, {enter_opened: true, space_closed: true, closed_focus_safe: true,
      resize_focus_safe: true, values_preserved: true, location_preserved: true,
      no_product_script: true, unique_ids: true, network_requests: requests});
    assert.equal(validHeaderEvidence(record, expected), true);
    return record;
  } finally {
    page.off('request', observedRequest);
    if (originalViewport) await page.setViewportSize(originalViewport);
  }
}
module.exports = {assertNativeHeader, validHeaderEvidence};
