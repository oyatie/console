'use strict';
// Failure injection against real served assets/DOM; never business/API population.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const CONTROL_NAMES = Object.freeze(['delayed-ime-input', 'delayed-scroll', 'precommit-focus', 'failed-load', 'failed-render']);
function validControlEvidence(records) {
  try {
    assert.equal(Array.isArray(records), true);
    assert.deepEqual(records.map(row => row.name), CONTROL_NAMES);
    for (const row of records) {
      assert.match(row.command, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
      assert.notEqual(row.command, '00000000-0000-0000-0000-000000000000');
      assert.match(row.form_sha256, /^[0-9a-f]{64}$/);
      for (const key of ['fallback_retained', 'form_preserved', 'focus_preserved', 'scroll_preserved',
        'selection_preserved', 'no_mutation', 'owner_effects_verified']) assert.equal(row[key], true);
      assert.equal(row.react_mounted, false);
      if (row.name === 'delayed-ime-input') assert.equal(row.ime_events >= 2, true);
      if (row.name === 'delayed-scroll') assert.equal(row.scroll_y > 0, true);
      if (row.name === 'precommit-focus' || row.name === 'failed-render') assert.equal(row.render_probe_hits > 0, true);
    }
    assert.equal(new Set(records.map(row => row.command)).size, CONTROL_NAMES.length);
    return true;
  } catch { return false; }
}
async function snapshot(page) {
  return page.locator('form[data-people-operation="prepare"]').evaluate(form => {
    const focus = document.activeElement;
    return {fields: [...new FormData(form)].map(([name, value]) => [name, value]),
      focus_id: focus?.id ?? null, selection_start: focus?.selectionStart ?? null,
      selection_end: focus?.selectionEnd ?? null, scroll_x: scrollX, scroll_y: scrollY};
  });
}
// Diagnostic-only projection: fixed counts/enums/booleans, never form material.
function reactControlFailureSnapshot() {
  const probe = window.__consoleReactRenderProbe;
  let guardCanPromote = null;
  try { if (typeof window.__consolePeopleGuard?.canPromote === 'function') guardCanPromote = window.__consolePeopleGuard.canPromote() === true; } catch {}
  const tag = document.activeElement?.tagName?.toLowerCase();
  return {fallbackCount: document.querySelectorAll('#console-people-fallback').length,
    reactMountCount: document.querySelectorAll('[data-console-react-people="mounted"]').length,
    renderProbe: Number.isSafeInteger(probe) && probe >= 0 ? probe : null, guardCanPromote,
    focusTag: ['html','body','a','button','input','summary','main','section','p'].includes(tag) ? tag : 'other',
    scrollXNonzero: window.scrollX !== 0, scrollYNonzero: window.scrollY !== 0};
}
async function runReactControls({page, origin, company, account, expectDocument, exchange, capture, clearBrowserCache}) {
  const path = `/companies/${company}/people/new`, runtime = origin + '/assets/people.js';
  const cdp = await page.context().newCDPSession(page);
  const result = [];
  const initialViewport = page.viewportSize();
  let diagnosticControl = 'none', diagnosticStep = 'start', failedStep = null, failedError = null;
  try {
    for (const name of CONTROL_NAMES) {
      diagnosticControl = name; failedStep = null; failedError = null;
      diagnosticStep = 'owner-ready';
      const ready = await exchange({phase: 'PEOPLE_REACT_CONTROL_READY', control_name: name});
      assert.equal(ready.owner_effects_verified, true);
      diagnosticStep = 'clear-browser-cache';
      await clearBrowserCache();
      diagnosticStep = 'viewport';
      await page.setViewportSize({width: 320, height: 400});
      let release, routing, injected;
      let before, imeEvents = 0, hits = 0, mutations = 0;
      const changed = request => { if (!['GET', 'HEAD', 'OPTIONS'].includes(request.method())) mutations += 1; };
      page.context().on('request', changed);
      try {
        if (name === 'delayed-ime-input' || name === 'delayed-scroll') {
          const pending = new Promise(resolve => { release = resolve; });
          routing = async route => { await pending; await route.continue(); };
          diagnosticStep = 'install-delayed-route';
          await page.route(runtime, routing);
        } else if (name === 'failed-load') {
          diagnosticStep = 'install-failed-load-route';
          routing = route => route.abort('failed'); await page.route(runtime, routing);
        } else {
          // A real DOM-render fault/focus event inside React's host rendering.
          // The original backend response and business code are never replaced.
          const source = `(() => { const original = Document.prototype.createElement;
            window.__consoleReactRenderProbe = 0;
            Document.prototype.createElement = function(tag, ...rest) {
              if (String(tag).toLowerCase() === 'form') {
                window.__consoleReactRenderProbe += 1;
                ${name === 'failed-render' ? "throw new Error('INJECTED_REACT_HOST_RENDER_FAILURE');" : "document.getElementById('people-name')?.focus();"}
              }
              return original.call(this, tag, ...rest);
            }; })();`;
          diagnosticStep = 'install-render-probe';
          injected = (await cdp.send('Page.addScriptToEvaluateOnNewDocument', {source})).identifier;
        }
        expectDocument('GET', path, 200, false);
        diagnosticStep = 'document-navigation';
        const response = await page.goto(origin + path, {waitUntil: 'commit'});
        assert.equal(response.status(), 200);
        diagnosticStep = 'ssr-response-check';
        assert.equal(await page.evaluate(html => !!new DOMParser().parseFromString(html, 'text/html')
          .querySelector('[data-console-react-people="mounted"]'), await response.text()), false);
        const form = page.locator('#console-people-fallback form[data-people-operation="prepare"]');
        diagnosticStep = 'fallback-form-visible';
        await form.waitFor({state: 'visible'});
        if (name === 'delayed-ime-input') {
          diagnosticStep = 'ime-listeners';
          await page.evaluate(() => {
            window.__consoleImeEvents = 0;
            for (const type of ['compositionstart', 'compositionupdate', 'compositionend']) {
              document.addEventListener(type, event => { if (event.isTrusted) window.__consoleImeEvents += 1; });
            }
          });
          const input = form.locator('input[name="legal_name"]');
          diagnosticStep = 'ime-focus';
          await input.click();
          diagnosticStep = 'ime-compose';
          await cdp.send('Input.imeSetComposition', {text: '김하늘', selectionStart: 3, selectionEnd: 3});
          diagnosticStep = 'ime-insert';
          await cdp.send('Input.insertText', {text: '김하늘 <연구 & 운영>'});
          diagnosticStep = 'ime-selection';
          await page.keyboard.press('ArrowLeft'); await page.keyboard.press('Shift+ArrowLeft');
          diagnosticStep = 'ime-input-check';
          assert.equal(await input.inputValue(), '김하늘 <연구 & 운영>');
          diagnosticStep = 'ime-trusted-events';
          imeEvents = await page.evaluate(() => window.__consoleImeEvents);
          assert.equal(imeEvents >= 2, true);
        } else if (name === 'delayed-scroll') {
          diagnosticStep = 'scroll-wheel';
          await page.mouse.wheel(0, 200);
          diagnosticStep = 'scroll-nonzero';
          await page.waitForFunction(() => scrollY > 0);
        }
        diagnosticStep = 'snapshot-before';
        if (name.startsWith('delayed-')) before = await snapshot(page);
        if (release) release();
        diagnosticStep = 'network-idle';
        await page.waitForLoadState('networkidle');
        diagnosticStep = 'snapshot-after-load';
        if (!before) before = await snapshot(page);
        // A post-load paint cannot lose fallback state after an attempted mount.
        diagnosticStep = 'paint-after-load';
        await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        diagnosticStep = 'snapshot-after-paint';
        const after = await snapshot(page);
        assert.equal(JSON.stringify(after) === JSON.stringify(before), true, 'React promotion discarded SSR working context');
        diagnosticStep = 'mounted-count';
        assert.equal(await page.locator('[data-console-react-people="mounted"]').count(), 0);
        diagnosticStep = 'fallback-count';
        assert.equal(await page.locator('#console-people-fallback').count(), 1);
        diagnosticStep = 'prepare-form-count';
        assert.equal(await page.locator('form[data-people-operation="prepare"]').count(), 1);
        if (name === 'precommit-focus' || name === 'failed-render') {
          diagnosticStep = 'render-probe-count';
          hits = await page.evaluate(() => window.__consoleReactRenderProbe);
          assert.equal(hits > 0, true, 'React render-fault control never reached real host rendering');
        }
        const command = before.fields.find(([key]) => key === 'command_id')?.[1];
        diagnosticStep = 'owner-checked';
        const witness = await exchange({phase: 'PEOPLE_REACT_CONTROL_CHECKED', control_name: name, command_id: command});
        assert.equal(witness.owner_effects_verified, true);
        assert.equal(witness.company, company); assert.equal(witness.account, account);
        assert.equal(mutations, 0);
        diagnosticStep = 'capture';
        await capture('react-people-' + name + '-320');
        result.push({name, command, form_sha256: crypto.createHash('sha256').update(JSON.stringify(before.fields)).digest('hex'),
          fallback_retained: true, form_preserved: true, focus_preserved: true, selection_preserved: true,
          scroll_preserved: true, no_mutation: true, react_mounted: false, owner_effects_verified: true,
          ime_events: imeEvents, scroll_y: before.scroll_y, render_probe_hits: hits});
      } catch (error) {
        failedStep = diagnosticStep; failedError = error; throw error;
      } finally {
        if (release) release();
        diagnosticStep = 'cleanup-route';
        if (routing) await page.unroute(runtime, routing);
        diagnosticStep = 'cleanup-render-probe';
        if (injected) await cdp.send('Page.removeScriptToEvaluateOnNewDocument', {identifier: injected});
        page.context().off('request', changed);
      }
    }
    assert.equal(validControlEvidence(result), true); return result;
  } catch (error) {
    const diagnostic = {kind: 'REACT_PEOPLE_CONTROL_FAILURE_DIAGNOSTIC_V1', diagnosticOnly: true,
      controlName: diagnosticControl, awaitedStep: error === failedError ? failedStep : diagnosticStep,
      fallbackCount: null, reactMountCount: null, renderProbe: null, guardCanPromote: null,
      focusTag: 'unknown', scrollXNonzero: null, scrollYNonzero: null, collectionFailed: false};
    try { Object.assign(diagnostic, await page.evaluate(reactControlFailureSnapshot)); }
    catch { diagnostic.collectionFailed = true; }
    try { Object.defineProperty(error, 'consoleReactControlDiagnostic', {value: diagnostic}); } catch {}
    throw error;
  } finally {
    await cdp.detach();
    if (initialViewport) await page.setViewportSize(initialViewport);
  }
}
const HOSTILE_TEXT = '</script><img src=x onerror="window.__consoleHostileExecuted=true">&김하늘\u2028\u2029';
const DECODER_CONTROLS = Object.freeze({
  registration: Object.freeze([
    ['unmodified', true], ['invalid-json', false], ['unknown-version', false],
    ['extra-envelope-field', false], ['unknown-page-kind', false], ['extra-page-field', false],
    ['extra-scope-field', false], ['invalid-scope-boolean', false], ['malformed-company-id', false],
    ['nil-company-id', false], ['uppercase-company-id', false], ['malformed-command-id', false], ['malformed-expectation-id', false],
    ['numeric-revision', false], ['noncanonical-revision', false], ['large-revision-string', true], ['unicode-scalar-boundary', true], ['unicode-scalar-overflow', false], ['oversized-name', false],
    ['oversized-number', false], ['extra-form-field', false], ['extra-expectation-field', false],
    ['invalid-nullable-field', false], ['invalid-proof-type', false],
    ['registration-conflict-valid', true], ['registration-conflict-extra-field', false],
    ['request-not-visible-valid', true], ['request-not-visible-extra-field', false], ['uncertain-valid', true], ['uncertain-extra-field', false], ['hostile-text', true],
  ].map(([name, mounted]) => Object.freeze({name, mounted}))),
  terminal: Object.freeze([
    ['unmodified', true], ['unknown-outcome-kind', false], ['extra-terminal-field', false],
    ['terminal-proof-injection', false], ['rejected-valid', true], ['rejected-proof-injection', false],
    ['conflicting-valid', true], ['conflicting-proof-injection', false], ['cancelled-valid', true], ['cancelled-proof-injection', false],
    ['expired-valid', true], ['expired-proof-injection', false], ['malformed-terminal-id', false],
  ].map(([name, mounted]) => Object.freeze({name, mounted}))),
  directory: Object.freeze([
    ['unmodified', true], ['nullable-record-fields', true], ['oversized-record-collection', false], ['malformed-record-id', false],
    ['extra-record-field', false], ['numeric-record-revision', false], ['invalid-record-nullable-field', false],
    ['invalid-after-cursor', false], ['unsafe-next-href', false], ['detail-valid', true], ['detail-extra-field', false],
  ].map(([name, mounted]) => Object.freeze({name, mounted}))),
});
function corruptProjection(value, name) {
  const p = value.page;
  switch (name) {
    case 'unmodified': break;
    case 'unknown-version': value.version = 2; break;
    case 'extra-envelope-field': value.unowned = true; break;
    case 'unknown-page-kind': p.kind = 'unowned'; break;
    case 'extra-page-field': p.unowned = true; break;
    case 'extra-scope-field': p.scope.unowned = true; break;
    case 'invalid-scope-boolean': p.scope.can_create = 'true'; break;
    case 'malformed-company-id': p.scope.company = '../outside-company'; break;
    case 'nil-company-id': p.scope.company = '00000000-0000-0000-0000-000000000000'; break;
    case 'uppercase-company-id': p.scope.company = 'FFFFFFFF-FFFF-4FFF-8FFF-FFFFFFFFFFFF'; break;
    case 'malformed-command-id': p.form.command = p.form.command.toUpperCase() + '/execute'; break;
    case 'malformed-expectation-id': p.form.expected.object_type_id = 'not-an-object'; break;
    case 'numeric-revision': p.form.expected.company_epoch = 9007199254740992; break;
    case 'noncanonical-revision': p.form.expected.action_revision = '1e3'; break;
    case 'large-revision-string': p.form.expected.company_epoch = '9007199254740993'; break;
    case 'unicode-scalar-boundary': p.form.legal_name = '𠮷'.repeat(200); p.form.employee_number = '𠮷'.repeat(64); break;
    case 'unicode-scalar-overflow': p.form.legal_name = '𠮷'.repeat(201); p.form.employee_number = '𠮷'.repeat(65); break;
    case 'nullable-record-fields': p.records[0].legal_name = null; p.records[0].employee_number = null; break;
    case 'oversized-name': p.form.legal_name = '한'.repeat(201); break;
    case 'oversized-number': p.form.employee_number = '한'.repeat(65); break;
    case 'extra-form-field': p.form.unowned = true; break;
    case 'extra-expectation-field': p.form.expected.unowned = true; break;
    case 'invalid-nullable-field': p.form.name_error = {message: 'unowned'}; break;
    case 'invalid-proof-type': p.form.proof = true; break;
    case 'registration-conflict-valid':
    case 'registration-conflict-extra-field': value.page = {kind:'registration_conflict', scope:p.scope, command:p.form.command, legal_name:p.form.legal_name, employee_number:p.form.employee_number, ...(name.endsWith('-extra-field') ? {unowned:true} : {})}; break;
    case 'request-not-visible-valid':
    case 'request-not-visible-extra-field': value.page = {kind:'request_not_visible', scope:p.scope, command:p.form.command, ...(name.endsWith('-extra-field') ? {unowned:true} : {})}; break;
    case 'uncertain-valid':
    case 'uncertain-extra-field': value.page = {kind:'uncertain', company:p.scope.company, command:p.form.command, ...(name.endsWith('-extra-field') ? {unowned:true} : {})}; break;
    case 'hostile-text': p.form.legal_name = HOSTILE_TEXT; break;
    case 'unknown-outcome-kind': p.request.outcome.kind = 'unowned'; break;
    case 'extra-terminal-field': p.request.outcome.unowned = true; break;
    case 'terminal-proof-injection': p.request.outcome.proof = 'INJECTED_NONAUTHORITY_PROOF'; break;
    case 'rejected-valid':
    case 'rejected-proof-injection': p.request.outcome = {kind:'rejected', reason:'이미 등록된 사번입니다.', ...(name.endsWith('-proof-injection') ? {proof:'INJECTED_NONAUTHORITY_PROOF'} : {})}; break;
    case 'conflicting-valid':
    case 'conflicting-proof-injection': p.request.outcome = {kind:'conflicting', reason:'회사 설정이 변경되었습니다.', ...(name.endsWith('-proof-injection') ? {proof:'INJECTED_NONAUTHORITY_PROOF'} : {})}; break;
    case 'cancelled-valid':
    case 'cancelled-proof-injection': p.request.outcome = {kind:'cancelled', ...(name.endsWith('-proof-injection') ? {proof:'INJECTED_NONAUTHORITY_PROOF'} : {})}; break;
    case 'expired-valid':
    case 'expired-proof-injection': p.request.outcome = {kind:'expired', ...(name.endsWith('-proof-injection') ? {proof:'INJECTED_NONAUTHORITY_PROOF'} : {})}; break;
    case 'oversized-record-collection': p.records = Array.from({length:101},()=>({...p.records[0]})); break;
    case 'malformed-record-id': p.records[0].person_id = '../outside-person'; break;
    case 'extra-record-field': p.records[0].unowned = true; break;
    case 'numeric-record-revision': p.records[0].person_version = 9007199254740992; break;
    case 'invalid-record-nullable-field': p.records[0].legal_name = true; break;
    case 'invalid-after-cursor': p.after_cursor = 'false'; break;
    case 'unsafe-next-href': p.next_href = 'javascript:alert(1)'; break;
    case 'detail-valid':
    case 'detail-extra-field': value.page = {kind:'detail',scope:p.scope,record:p.records[0],...(name.endsWith('-extra-field') ? {unowned:true} : {})}; break;
    case 'malformed-terminal-id': p.request.outcome.employee_id = '../outside-person'; break;
    default: throw new Error('UNKNOWN_DECODER_CONTROL');
  }
  return value;
}
function validDecoderEvidence(records) {
  try {
    assert.equal(Array.isArray(records), true);
    const expected = Object.entries(DECODER_CONTROLS).flatMap(([group, controls]) => controls.map(c => ({group, ...c})));
    assert.deepEqual(records.map(r => [r.group, r.name]), expected.map(r => [r.group, r.name]));
    for (const group of Object.keys(DECODER_CONTROLS)) {
      const rows = records.filter(r => r.group === group), baseline = rows[0];
      assert.equal(baseline.name, 'unmodified'); assert.equal(baseline.react_mounted, true);
      assert.match(baseline.runtime_sha256, /^[0-9a-f]{64}$/);
      for (const row of rows) assert.equal(row.runtime_sha256, baseline.runtime_sha256);
    }
    assert.equal(new Set(records.filter(r=>r.group==='registration').map(r=>r.command)).size, DECODER_CONTROLS.registration.length);
    assert.equal(new Set(records.filter(r=>r.group==='terminal').map(r=>r.command)).size, 1);
    for (let i = 0; i < records.length; i++) {
      const row = records[i], control = expected[i];
      assert.equal(row.react_mounted, control.mounted);
      assert.equal(row.fallback_retained, !control.mounted);
      assert.equal(row.rejection_status, !control.mounted);
      for (const flag of ['runtime_http_200', 'original_response_unchanged', 'no_mutation', 'owner_effects_verified',
        'current_context_preserved', 'no_business_storage']) assert.equal(row[flag], true);
      assert.match(row.original_bootstrap_sha256, /^[0-9a-f]{64}$/);
      assert.match(row.fallback_sha256, /^[0-9a-f]{64}$/);
      if (row.group !== 'directory') {
        assert.match(row.command, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
        assert.notEqual(row.command, '00000000-0000-0000-0000-000000000000');
      } else assert.equal(row.command, null);
      if (row.name === 'hostile-text') assert.equal(row.hostile_text_only, true);
      if (row.name === 'large-revision-string') assert.equal(row.exact_large_revision, true);
      if (row.name === 'unicode-scalar-boundary') assert.equal(row.exact_scalar_limits, true);
      if (row.name === 'nullable-record-fields') assert.equal(row.nullable_record_rendered, true);
      if (row.group === 'terminal') assert.equal(row.no_actionable_form, true);
      assert.equal(row.outside_action_control_verified, true);
      if (!control.mounted) assert.equal(row.global_actions_preserved, true);
    }
    return true;
  } catch { return false; }
}
async function runDecoderControls({page, origin, company, account, group, command, expectDocument,
  exchange, capture, clearBrowserCache, mounted}) {
  assert.ok(DECODER_CONTROLS[group]);
  const path = group === 'registration' ? `/companies/${company}/people/new` : group === 'directory' ? `/companies/${company}/people` : `/companies/${company}/people/requests/${command}`;
  const runtime = origin + '/assets/people.js', records = [];
  const initialViewport = page.viewportSize();
  try {
    await page.setViewportSize({width: 320, height: 900});
    for (const control of DECODER_CONTROLS[group]) {
      const ready = await exchange({phase: 'PEOPLE_REACT_DECODER_READY', decoder_group: group, control_name: control.name});
      assert.equal(ready.owner_effects_verified, true);
      await clearBrowserCache();
      let release, mutations = 0;
      const pending = new Promise(resolve => { release = resolve; });
      const routing = async route => { await pending; await route.continue(); };
      const mutation = request => { if (!['GET', 'HEAD', 'OPTIONS'].includes(request.method())) mutations += 1; };
      page.context().on('request', mutation);
      await page.route(runtime, routing);
      try {
        const asset = page.waitForResponse(r => r.url() === runtime && r.request().resourceType() === 'script');
        expectDocument('GET', path, 200, false);
        const response = await page.goto(origin + path, {waitUntil: 'commit'});
        assert.equal(response.status(), 200);
        const originalResponse = await response.text();
        assert.equal(await page.evaluate(html => !!new DOMParser().parseFromString(html, 'text/html')
          .querySelector('[data-console-react-people="mounted"]'), originalResponse), false);
        const fallback = page.locator('#console-people-fallback');
        await fallback.waitFor({state: 'visible'});
        const bootstrap = page.locator('#console-people-bootstrap');
        assert.equal(await bootstrap.count(), 1, 'REACT_PEOPLE_DECODER_BOOTSTRAP_MISSING');
        const original = await bootstrap.textContent(), value = JSON.parse(original);
        const issuedCommand = group === 'registration' ? value.page.form.command : group === 'directory' ? null : command;
        assert.equal(value.version, 1);
        assert.equal(value.page.kind, group === 'registration' ? 'registration' : group === 'directory' ? 'directory' : 'request');
        if (group === 'directory') assert.equal(value.page.records.length, 1);
        if (group === 'terminal') assert.equal(value.page.request.outcome.kind, 'committed');
        await page.waitForFunction(() => document.readyState !== 'loading');
        const originalFallback = await fallback.evaluate(e => e.outerHTML);
        const actionsBefore = await actionCensus(page);
        assert.equal(await proveOutsideActionObserver(page), true);
        const context = await page.evaluate(() => ({focus: document.activeElement?.id ?? null, x: scrollX, y: scrollY}));
        const modified = control.name === 'invalid-json' ? '{' : JSON.stringify(corruptProjection(value, control.name));
        if (control.name !== 'unmodified') await bootstrap.evaluate((e, raw) => { e.textContent = raw; }, modified);
        release();
        const runtimeResponse = await asset; assert.equal(runtimeResponse.status(), 200);
        const runtimeSha = crypto.createHash('sha256').update(await runtimeResponse.body()).digest('hex');
        if (records.length) assert.equal(runtimeSha, records[0].runtime_sha256);
        await page.waitForLoadState('networkidle');
        let hostileTextOnly = false, exactLargeRevision = false, exactScalarLimits = false, nullableRecordRendered = false;
        if (control.mounted) {
          await mounted(path, response);
          if (control.name === 'unicode-scalar-boundary') {
            assert.equal(await page.locator('input[name="legal_name"]').inputValue(), '𠮷'.repeat(200));
            assert.equal(await page.locator('input[name="employee_number"]').inputValue(), '𠮷'.repeat(64));
            exactScalarLimits = true;
          }
          if (control.name === 'nullable-record-fields') {
            const record = page.locator('[data-people-record]');
            assert.equal(await record.count(), 1); assert.equal(await record.isVisible(), true);
            assert.equal(await record.getByText('이름 미등록', {exact:true}).isVisible(), true);
            assert.equal(await record.getByText('사번 미등록', {exact:true}).isVisible(), true);
            nullableRecordRendered = true;
          }
          if (control.name === 'large-revision-string') {
            assert.equal(await page.locator('input[name="expected_company_epoch"]').inputValue(), '9007199254740993');
            exactLargeRevision = true;
          }
          if (control.name === 'hostile-text') {
            assert.equal(await page.locator('input[name="legal_name"]').inputValue(), HOSTILE_TEXT);
            assert.equal(await page.locator('img[src="x"],[onerror]').count(), 0);
            assert.equal(await page.evaluate(() => window.__consoleHostileExecuted === undefined), true);
            hostileTextOnly = true;
          }
        } else {
          if (await page.locator('[data-console-react-people="mounted"]').count() !== 0)
            throw Object.assign(new Error('REACT_PEOPLE_DECODER_ACCEPTED_INVALID_PROJECTION'), {code:'REACT_PEOPLE_DECODER_ACCEPTED_INVALID_PROJECTION'});
          assert.equal(sameActions(actionsBefore, await actionCensus(page)), true, 'REACT_PEOPLE_DECODER_ADDED_ACTION');
          assert.equal(await fallback.count(), 1); assert.equal(await fallback.isVisible(), true);
          assert.equal((await fallback.evaluate(e => e.outerHTML)) === originalFallback, true, 'REACT_PEOPLE_DECODER_DAMAGED_SSR');
          const status = page.locator('[role="status"][data-console-people-render-status="invalid-projection"]');
          try { await status.waitFor({state:'visible', timeout:15000}); }
          catch { throw Object.assign(new Error('REACT_PEOPLE_DECODER_REJECTION_STATUS_MISSING'), {code:'REACT_PEOPLE_DECODER_REJECTION_STATUS_MISSING'}); }
          assert.equal(await status.count(), 1);
          assert.equal(await status.isVisible(), true);
          assert.equal(await status.textContent(), '화면을 표시하지 못했습니다. 현재 내용을 유지합니다.');
          await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
          assert.equal(await page.locator('[data-console-react-people="mounted"]').count(), 0);
          assert.equal((await fallback.evaluate(e => e.outerHTML)) === originalFallback, true, 'REACT_PEOPLE_DECODER_DAMAGED_SSR');
          assert.equal(sameActions(actionsBefore, await actionCensus(page)), true, 'REACT_PEOPLE_DECODER_ADDED_ACTION');
        }
        assert.deepEqual(await page.evaluate(() => ({focus: document.activeElement?.id ?? null, x: scrollX, y: scrollY})), context);
        if (group === 'terminal') assert.equal(await page.locator('form').count(), 0);
        await page.evaluate(() => window.__consoleStorageObserver.positiveControl());
        assert.equal(mutations, 0);
        assert.equal((await response.text()) === originalResponse, true, 'test must not replace the owning backend document');
        const witness = await exchange({phase: 'PEOPLE_REACT_DECODER_CHECKED', decoder_group: group,
          control_name: control.name, command_id: issuedCommand});
        assert.equal(witness.owner_effects_verified, true); assert.equal(witness.company, company); assert.equal(witness.account, account);
        await capture(`react-people-decoder-${group}-${control.name}-320`);
        records.push({group, name: control.name, command: issuedCommand,
          react_mounted: control.mounted, fallback_retained: !control.mounted, rejection_status: !control.mounted,
          runtime_http_200: true, runtime_sha256: runtimeSha,
          original_bootstrap_sha256: crypto.createHash('sha256').update(original).digest('hex'),
          fallback_sha256: crypto.createHash('sha256').update(originalFallback).digest('hex'), original_response_unchanged: true,
          no_mutation: true, owner_effects_verified: true, current_context_preserved: true, no_business_storage: true, outside_action_control_verified: true, global_actions_preserved: !control.mounted,
          hostile_text_only: hostileTextOnly, exact_large_revision: exactLargeRevision, exact_scalar_limits: exactScalarLimits, nullable_record_rendered: nullableRecordRendered, no_actionable_form: group === 'terminal'});
      } catch (error) { error.code ??= 'REACT_PEOPLE_DECODER_CONTROL_FAILED'; throw error; }
      finally { release(); await page.unroute(runtime, routing); page.context().off('request', mutation); }
    }
    return records;
  } finally { if (initialViewport) await page.setViewportSize(initialViewport); }
}
async function actionCensus(page) {
  return page.evaluate(() => [...document.querySelectorAll('form,a,button,input,textarea,select,summary,[role="button"],[role="link"]')].map(e => e.outerHTML));
}
function sameActions(before, after) { return Array.isArray(before) && Array.isArray(after) && before.every(x=>typeof x==='string') && after.every(x=>typeof x==='string') && JSON.stringify(before) === JSON.stringify(after); }
async function proveOutsideActionObserver(page) {
  const before = await actionCensus(page);
  await page.evaluate(() => { const f=document.createElement('form'); f.id='console-observer-form-control'; f.hidden=true;
    f.method='post'; f.action=location.pathname; const b=document.createElement('button'); b.type='button'; b.textContent='observer only'; f.append(b); document.body.append(f); });
  let detected;
  try { detected = !sameActions(before, await actionCensus(page)); }
  finally { await page.locator('#console-observer-form-control').evaluate(e=>e.remove()); }
  return detected && sameActions(before, await actionCensus(page));
}
const STORAGE_METHODS = Object.freeze(['cache:delete','cache:open','idb:deleteDatabase','idb:open','storage:removeItem','storage:setItem']);
function installStorageObserver() {
  if (window.__consoleStorageObserver) return;
  let exemption=null, injection=null; const undo=[];
  const emit=(kind,operation,injected=false)=>window.__consoleStorageObserverEvent(JSON.stringify({kind,operation,injected}));
  const wrap=(proto,method,operation)=>{const original=proto[method]; undo.push(()=>{proto[method]=original;}); proto[method]=function(...args){const owned=exemption?.target===this&&exemption.operation===operation&&exemption.key===args[0];
    const injected=injection?.target===this&&injection.operation===operation&&injection.key===args[0];
    emit(owned?'control':'product',operation,injected);return original.apply(this,args);};};
  for(const name of ['setItem','removeItem','clear'])wrap(Storage.prototype,name,'storage:'+name);
  for(const name of ['open','deleteDatabase'])wrap(IDBFactory.prototype,name,'idb:'+name);
  for(const name of ['open','delete','match'])wrap(CacheStorage.prototype,name,'cache:'+name);
  const snapshot=async()=>({local:Object.entries(localStorage),session:Object.entries(sessionStorage),
    idb:(await indexedDB.databases()).map(x=>({name:x.name,version:x.version})).sort((a,b)=>String(a.name).localeCompare(String(b.name))),cache:(await caches.keys()).sort()});
  const request=r=>new Promise((resolve,reject)=>{r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(new Error('STORAGE_OBSERVER_CONTROL_FAILED'));});
  const invoke=(target,method,key,value,slot)=>{const token={target,operation:(target===indexedDB?'idb':target===caches?'cache':'storage')+':'+method,key};
    if(slot==='control')exemption=token;else injection=token;
    try{return method==='setItem'?target[method](key,value):target[method](key);}finally{if(slot==='control')exemption=null;else injection=null;}};
  window.__consoleStorageObserver={snapshot,restore:()=>{for(const restore of undo)restore();delete window.__consoleStorageObserver;},positiveControl:async()=>{
    const before=await snapshot(),key='console-observer-only-'+crypto.randomUUID();
    try{invoke(localStorage,'setItem',key,'observer only','control');invoke(sessionStorage,'setItem',key,'observer only','control');
      invoke(localStorage,'removeItem',key,undefined,'control');invoke(sessionStorage,'removeItem',key,undefined,'control');
      const pending=invoke(indexedDB,'open',key,undefined,'control');
      if(pending.readyState!=='pending')throw new Error('STORAGE_OBSERVER_PENDING_CONTROL_NOT_EXECUTED');
      invoke(localStorage,'setItem',key+'-unrelated','observer only','injection');
      invoke(localStorage,'removeItem',key+'-unrelated',undefined,'injection');
      const db=await request(pending);db.close();await request(invoke(indexedDB,'deleteDatabase',key,undefined,'control'));
      await invoke(caches,'open',key,undefined,'control');await invoke(caches,'delete',key,undefined,'control');
      if(JSON.stringify(await snapshot())!==JSON.stringify(before))throw new Error('STORAGE_OBSERVER_STATE_CHANGED');
      emit('positive','verified');
    }finally{exemption=null;injection=null;}
  }};
  emit('installed','document');
}
function validStorageEvidence(r) {
  try {assert.equal(r.product_operations,0);assert.equal(r.positive_controls>0,true);assert.equal(r.installed_documents>0,true);
    assert.equal(r.state_preserved,true);assert.deepEqual(r.methods_verified,STORAGE_METHODS);
    assert.equal(r.injected_write_detections>=2,true);assert.deepEqual(r.injection_methods_verified,['storage:removeItem','storage:setItem']);return true;}catch{return false;}
}
async function startStorageObserver(page) {
  const cdp=await page.context().newCDPSession(page);let documents=0,positives=0;const methods=new Set(),injectedMethods=new Set(),product=[];let injectedWrites=0;
  cdp.on('Runtime.bindingCalled',event=>{if(event.name!=='__consoleStorageObserverEvent')return;
    const row=JSON.parse(event.payload);if(row.kind==='installed')documents++;else if(row.kind==='control')methods.add(row.operation);
    else if(row.kind==='positive')positives++;else if(row.kind==='product'&&row.injected===true){injectedWrites++;injectedMethods.add(row.operation);}else product.push(row.operation);});
  await cdp.send('Runtime.enable');await cdp.send('Runtime.addBinding',{name:'__consoleStorageObserverEvent'});
  const source='('+installStorageObserver.toString()+')()';
  const injected=(await cdp.send('Page.addScriptToEvaluateOnNewDocument',{source})).identifier;
  await page.evaluate(installStorageObserver);const initial=await page.evaluate(()=>window.__consoleStorageObserver.snapshot());
  return {check:async()=>{
    await cdp.send('Runtime.evaluate',{expression:'void 0'});
    assert.equal(product.length,0,'PEOPLE_PERSISTENT_STORAGE_WRITE');
    const current=await page.evaluate(()=>window.__consoleStorageObserver.snapshot());
    assert.equal(JSON.stringify(current)===JSON.stringify(initial),true,'PEOPLE_PERSISTENT_STORAGE_CHANGED');
  },evidence:async()=>{
    await cdp.send('Runtime.evaluate',{expression:'void 0'});
    const current=await page.evaluate(()=>window.__consoleStorageObserver.snapshot());
    return {product_operations:product.length,positive_controls:positives,installed_documents:documents,
      state_preserved:JSON.stringify(current)===JSON.stringify(initial),injected_write_detections:injectedWrites,injection_methods_verified:[...injectedMethods].sort(),methods_verified:[...methods].filter(x=>STORAGE_METHODS.includes(x)).sort()};
  },stop:async()=>{await page.evaluate(()=>window.__consoleStorageObserver?.restore());await cdp.send('Page.removeScriptToEvaluateOnNewDocument',{identifier:injected});await cdp.send('Runtime.removeBinding',{name:'__consoleStorageObserverEvent'});await cdp.detach();}};
}
module.exports = {CONTROL_NAMES, validControlEvidence, runReactControls, DECODER_CONTROLS,
  HOSTILE_TEXT, corruptProjection, validDecoderEvidence, runDecoderControls, sameActions, validStorageEvidence, STORAGE_METHODS, startStorageObserver};
