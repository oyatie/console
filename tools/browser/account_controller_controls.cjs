'use strict';
// Actual browser controls against owner-served documents/assets. No API population.
const assert = require('node:assert/strict'), crypto = require('node:crypto');
const {observe, validReceipt} = require('./account_company_handoff.cjs');
const {validCheckpointCommand} = require('./company.cjs');
const {NAMES, validRecords} = require('./account_controller_evidence.cjs');
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const paint = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
function installProbe(mode) {
  let form, controls, html, progress, fault = 0, renderHits = 0, capabilityCalls = 0;
  const retired = [], add = EventTarget.prototype.addEventListener, create = Document.prototype.createElement;
  const snapshot = () => form && JSON.stringify({values: [...form.querySelectorAll('input')].map(e =>
    [e.name, e.value, e.checked, e.readOnly, e.disabled, e.selectionStart, e.selectionEnd]),
    focus: controls.indexOf(document.activeElement), x: scrollX, y: scrollY});
  const currentForm = () => document.querySelector('form[data-company-enrollment],form[data-native-action]');
  const retain = () => {form = currentForm();
    controls = [...form.querySelectorAll('input,button,a')]; html = form.outerHTML; progress = snapshot();};
  EventTarget.prototype.addEventListener = function(type, listener, ...rest) {
    if (!form) form = document.querySelector('form[data-company-enrollment]');
    if (form && ((this === form && type === 'submit') ||
        (form.contains(this) && ['click','input'].includes(type)) || (this === window && ['pagehide', 'pageshow'].includes(type)))
        && form.isConnected && !document.querySelector('[data-console-react-account="mounted"]')) retired.push([this,type,listener]);
    if (mode === 'partial-bind-rollback' && !fault && type === 'submit' && form && !form.isConnected &&
        this !== form && this instanceof HTMLFormElement && this.isConnected &&
        this.matches('form[data-company-enrollment]') && currentForm() === this) {
      fault++; throw new Error('INJECTED_ACCOUNT_BIND_FAILURE');
    }
    return add.call(this, type, listener, ...rest);
  };
  Document.prototype.createElement = function(tag, ...rest) {
    if (String(tag).toLowerCase() === 'form') {
      renderHits++;
      if (mode === 'preclaim-focus') {document.querySelector('input[name="name"]')?.focus(); progress = snapshot();}
    }
    return create.call(this, tag, ...rest);
  };
  let resolveCapability;
  if (mode === 'capability-denied') PublicKeyCredential.isConditionalMediationAvailable = () => {
    capabilityCalls++; return new Promise(resolve => {resolveCapability = resolve;});
  };
  Object.defineProperty(window, '__consoleAccountControllerProbe', {value: Object.freeze({
    retain, counts: () => ({fault, renderHits, capabilityCalls}),
    sameNodes: () => !!form && currentForm() === form && controls.every(e => e.isConnected && form.contains(e)),
    same: () => !!form && currentForm() === form && controls.every(e => e.isConnected && form.contains(e)) && form.outerHTML === html,
    preserved: () => progress === snapshot(), snapshot: () => {progress = snapshot();},
    denyCapability: () => resolveCapability(false),
    fireRetired: () => {
      form.elements.namedItem('name').value = '폐기된 화면의 회사'; form.elements.namedItem('slug').value = 'retired-controller';
      for (const [target,type,listener] of retired) {
        const event = new Event(type, {cancelable:true}); if (type === 'pageshow') Object.defineProperty(event, 'persisted', {value:true});
        if (typeof listener === 'function') listener.call(target,event); else listener.handleEvent(event);
      }
      return retired.length;
    },
  })});
}
async function runControllerControls({browser, origin, result:r, out, emit, receive, setStage}) {
  const contexts = browser.contexts(); assert.equal(contexts.length, 2);
  const page = contexts[1].pages()[0]; assert.equal(contexts[1].pages().length, 1);
  const cdp = await contexts[1].newCDPSession(page); await cdp.send('Page.enable'); await cdp.send('DOMStorage.enable');
  const records = [], assets = new Map(), assetWork = [], storage = {controls:0, operations:0};
  const controlKey = 'console-account-controller-observer-' + crypto.randomUUID();
  const storageObserver = (operation,row) => {
    const id = row.storageId; let own = false; try {own = new URL((id?.securityOrigin ?? id?.storageKey).split('^')[0]).origin === origin;} catch {}
    if (!id || !own || typeof id.isLocalStorage !== 'boolean') {storage.operations++; return;}
    if (row.key === controlKey && ['add','remove'].includes(operation) && (operation === 'remove' || row.newValue === 'observer only')) storage.controls++;
    else storage.operations++;
  };
  for (const [event,operation] of [['DOMStorage.domStorageItemAdded','add'],['DOMStorage.domStorageItemRemoved','remove'],
    ['DOMStorage.domStorageItemUpdated','update'],['DOMStorage.domStorageItemsCleared','clear']]) cdp.on(event,row => storageObserver(operation,row));
  const assetResponse = response => {const url = new URL(response.url());
    if (url.origin !== origin || !['/assets/native-account.js','/assets/account.js'].includes(url.pathname)) return;
    assetWork.push((async () => {assert.equal(response.status(),200); const digest = sha(await response.body());
      if (assets.has(url.pathname)) assert.equal(digest, assets.get(url.pathname)); else assets.set(url.pathname,digest);})());
  };
  page.on('response',assetResponse);
  const anonymous = await browser.newContext({serviceWorkers:'block',viewport:{width:320,height:900}});
  observe(anonymous,'N',origin,r); const anonymousPage = await anonymous.newPage(); anonymousPage.on('response',assetResponse);
  await anonymous.route('**/*',route => {if (new URL(route.request().url()).origin === origin) return route.continue();
    r.external_requests++; return route.abort('blockedbyclient');});
  let positiveStorage = false, committed;
  const formLocator = p => p.locator('form[data-company-enrollment]');
  const submitLocator = p => p.getByRole('button',{name:'회사 업무 공간 만들기',exact:true});
  async function checkpoint(phase,name,extra={}) {emit({kind:'CHECKPOINT',phase,control_name:name,account_id:r.o,...extra});
    assert.equal(validCheckpointCommand(await receive(),phase),true); r.checkpoints.push(phase);}
  async function document(p,route) {const response = await p.goto(origin+route,{waitUntil:'commit'});
    assert.equal(response.status(),200); assert.equal(response.request().redirectedFrom(),null); assert.equal(response.url(),origin+route);}
  async function fill(p) {await p.getByRole('textbox',{name:'회사 이름',exact:true}).fill('고정된 관리자 회사 <원본 & 입력>');
    await p.getByRole('textbox',{name:'업무 공간 식별자',exact:true}).fill(`controller-${r.o.replaceAll('-','')}`);
    await p.getByRole('radio',{name:'다른 계정',exact:true}).check();
    await p.getByRole('textbox',{name:'관리할 계정 참조',exact:true}).fill(r.a);}
  try {
    for (const name of NAMES) {
      setStage('controller/'+name); await checkpoint('CONTROLLER_READY',name);
      const p = name === 'capability-denied' ? anonymousPage : page, protocol = p === page ? cdp : await anonymous.newCDPSession(p);
      await protocol.send('Network.enable'); await protocol.send('Network.setCacheDisabled',{cacheDisabled:true});
      await protocol.send('Page.enable'); const injected = (await protocol.send('Page.addScriptToEvaluateOnNewDocument',
        {source:`(${installProbe.toString()})(${JSON.stringify(name)});`})).identifier;
      const beforeCSRF = r.csrf_requests.length, beforePosts = r.mutations.length, beforeDocs = r.documents.length;
      let routing, release, routeSeen, csrfRouting, csrfRelease, lateCallbacks = 0, imeEvents = 0, autofillEvents = 0;
      try {
        if (name !== 'freeze-at-csrf' && name !== 'capability-denied') {
          let seen; routeSeen = new Promise(resolve => {seen=resolve;});
          const blocked = new Promise(resolve => {release=resolve;});
          const target = name === 'react-first' ? '/assets/native-account.js' : '/assets/account.js';
          routing = async route => {seen(); await blocked; await route.continue();}; await p.route(origin+target,routing);
        }
        await document(p,name === 'capability-denied' ? '/account' : '/account/companies/new');
        if (routeSeen) await routeSeen;
        if (name === 'capability-denied') {
          await p.waitForFunction(() => window.__consoleAccountControllerProbe.counts().capabilityCalls === 1);
          await p.evaluate(() => {window.__consoleAccountControllerProbe.retain(); window.__consoleAccountControllerProbe.denyCapability();});
          await p.locator('#native-error[role="alert"]').waitFor({state:'visible'});
          assert.equal(await p.locator('[data-native-submit]').isDisabled(),true);
        } else {
          await formLocator(p).waitFor({state:'visible'});
          if (name === 'react-first') {
            await p.waitForFunction(() => window.__consoleAccountControllerProbe.counts().renderHits > 0);
            assert.equal(await p.locator('[data-console-react-account="mounted"]').count(),0);
          } else await submitLocator(p).waitFor({state:'visible'});
          if (name !== 'react-first') await p.waitForFunction(() => !document.querySelector('form[data-company-enrollment] button[type="submit"]').disabled);
          if (!positiveStorage) {
            await p.evaluate(key => {for (const store of [localStorage,sessionStorage]) {store.setItem(key,'observer only'); store.removeItem(key);}},controlKey);
            await paint(p); assert.equal(storage.controls,4); positiveStorage=true;
          }
          await p.evaluate(() => window.__consoleAccountControllerProbe.retain());
          const input = p.getByRole('textbox',{name:'회사 이름',exact:true});
          if (name === 'keyboard-before-react') {await input.click(); await p.keyboard.type('before-react'); await p.keyboard.press('Shift+ArrowLeft');}
          if (name === 'eventless-value-before-react') await input.evaluate(e => {e.value='복원된 입력값';});
          if (name === 'autofill-before-react') {
            await protocol.send('Autofill.enable'); let filled;
            const witness = new Promise(resolve => {filled=()=>{autofillEvents++; resolve();};}); protocol.on('Autofill.addressFormFilled',filled);
            const {root} = await protocol.send('DOM.getDocument'); const {nodeId} = await protocol.send('DOM.querySelector',
              {nodeId:root.nodeId,selector:'input[name="name"]'}); const {node} = await protocol.send('DOM.describeNode',{nodeId});
            await protocol.send('Autofill.trigger',{fieldId:node.backendNodeId,address:{fields:[{name:'COMPANY_NAME',value:'실제 자동완성 회사'}]}});
            await witness; assert.equal(autofillEvents,1); assert.equal(await input.inputValue(),'실제 자동완성 회사'); protocol.off('Autofill.addressFormFilled',filled);
          }
          if (name === 'ime-before-react') {
            await p.evaluate(() => {window.__consoleControllerIME=0; for (const type of ['compositionstart','compositionupdate','compositionend'])
              document.addEventListener(type,event => {if (event.isTrusted) window.__consoleControllerIME++;});});
            await input.click(); await protocol.send('Input.imeSetComposition',{text:'김하늘',selectionStart:3,selectionEnd:3});
            await protocol.send('Input.insertText',{text:'김하늘 <연구 & 운영>'}); await p.keyboard.press('Shift+ArrowLeft');
            imeEvents=await p.evaluate(() => window.__consoleControllerIME); assert.ok(imeEvents>=2);
          }
          await p.evaluate(() => window.__consoleAccountControllerProbe.snapshot());
        }
        release?.(); await p.waitForLoadState('networkidle'); await paint(p); await Promise.all(assetWork);
        const mounted = await p.locator('[data-console-react-account="mounted"]').count();
        assert.equal(mounted,['native-first','react-first','freeze-at-csrf'].includes(name)?1:0);
        let exactNodes = false, inputPreserved = true;
        if (!mounted && name !== 'capability-denied') {exactNodes=await p.evaluate(() => window.__consoleAccountControllerProbe.same());
          inputPreserved=await p.evaluate(() => window.__consoleAccountControllerProbe.preserved()); assert.equal(exactNodes,true); assert.equal(inputPreserved,true);}
        if (name === 'capability-denied') exactNodes=await p.evaluate(() => window.__consoleAccountControllerProbe.sameNodes());
        assert.equal(exactNodes || mounted===1,true);
        if (name === 'native-first') {
          lateCallbacks=await p.evaluate(() => window.__consoleAccountControllerProbe.fireRetired()); assert.ok(lateCallbacks>=3);
          await paint(p); assert.equal(r.csrf_requests.length,beforeCSRF); assert.equal(r.documents.length,beforeDocs+1);
          await p.evaluate(() => new Promise((resolve,reject) => {const script=document.createElement('script'); script.type='module';
            script.src='/assets/native-account.js?account-controller-replay=1'; script.onload=resolve; script.onerror=reject; document.head.append(script);}));
          await Promise.all(assetWork); assert.equal(await p.locator('[data-console-react-account="mounted"]').count(),1);
        }
        if (['native-first','react-first','partial-bind-rollback'].includes(name)) {
          await fill(p); csrfRouting=route => route.abort('failed'); await p.route(origin+'/api/v2/auth/csrf',csrfRouting);
          await submitLocator(p).click(); await p.locator('#company-error[role="alert"]').waitFor({state:'visible'});
          assert.equal(r.csrf_requests.length,beforeCSRF+1); assert.equal(r.mutations.length,beforePosts);
          assert.equal(await submitLocator(p).isEnabled(),true);
          assert.equal(await p.getByRole('textbox',{name:'관리할 계정 참조',exact:true}).inputValue(),r.a);
          assert.equal(await p.getByRole('textbox',{name:'회사 이름',exact:true}).evaluate(e=>e.readOnly),false);
          assert.equal(await p.getByRole('textbox',{name:'관리할 계정 참조',exact:true}).isEnabled(),true);
          assert.equal(await p.getByRole('textbox',{name:'관리할 계정 참조',exact:true}).isEditable(),true);
        }
        if (name === 'freeze-at-csrf') {
          await fill(p); let proofSeen; const proof = new Promise(resolve=>{proofSeen=resolve;});
          const heldProof = new Promise(resolve=>{csrfRelease=resolve;}); csrfRouting=async route=>{proofSeen(); await heldProof; await route.continue();};
          await p.route(origin+'/api/v2/auth/csrf',csrfRouting); await submitLocator(p).click(); await proof;
          assert.equal(await p.getByRole('textbox',{name:'회사 이름',exact:true}).evaluate(e=>e.readOnly),true);
          assert.equal(await p.getByRole('textbox',{name:'업무 공간 식별자',exact:true}).evaluate(e=>e.readOnly),true);
          for (const locator of [p.getByRole('radio',{name:'내 계정',exact:true}),p.getByRole('radio',{name:'다른 계정',exact:true})])
            assert.equal(await locator.isDisabled(),true);
          assert.equal(await p.getByRole('textbox',{name:'관리할 계정 참조',exact:true}).isEditable(),false);
          const saved=await p.locator('#company-result').getAttribute('href');
          await checkpoint('CONTROLLER_PRE_DISPATCH',name);
          await formLocator(p).evaluate((form,operator)=>{form.elements.namedItem('name').value='변경된 회사'; form.elements.namedItem('slug').value='changed-after-await';
            form.querySelector('input[type="radio"]').checked=true;
            const recipient=[...form.querySelectorAll('input')].find(e=>e.value!==operator && /^[0-9a-f-]{36}$/.test(e.value));
            if (!recipient) throw new Error('selected recipient control unavailable'); recipient.value=operator;},r.o);
          const response=p.waitForResponse(v=>v.url()===origin+'/api/v2/companies/enroll'&&v.request().method()==='POST');
          csrfRelease(); const actual=await response; assert.equal(actual.status(),201);
          const sent=actual.request().postDataJSON(),receipt=await actual.json();
          assert.equal(validReceipt(sent,receipt,r.a,'고정된 관리자 회사 <원본 & 입력>',`controller-${r.o.replaceAll('-','')}`),true);
          assert.equal(saved,receipt.result_path); await p.waitForURL(origin+receipt.result_path); await p.waitForLoadState('networkidle');
          await p.locator('[data-company-outcome="committed"]').waitFor({state:'visible'});
          assert.equal(await p.locator('[data-company-outcome="committed"]').getByText(r.a,{exact:true}).isVisible(),true);
          await checkpoint('CONTROLLER_COMMITTED',name,{administrator_account_id:r.a,command_id:sent.command_id,org_id:receipt.org_id,
            group_id:receipt.group_id,receipt_id:receipt.receipt_id});
          await document(p,receipt.result_path); await p.waitForLoadState('networkidle'); committed={sent,receipt};
        }
        await p.screenshot({path:require('node:path').join(out,'controller-'+name+'.png'),fullPage:true});
        const counts=await p.evaluate(()=>window.__consoleAccountControllerProbe.counts()).catch(()=>({fault:0,renderHits:0,capabilityCalls:0}));
        records.push({name,asset_bytes_unchanged:true,native_sha256:assets.get('/assets/native-account.js'),react_sha256:assets.get('/assets/account.js'),
          react_mounted:mounted===1,exact_original_nodes:exactNodes,input_preserved:inputPreserved,late_callbacks:lateCallbacks,late_callbacks_inert:true,
          csrf_requests:r.csrf_requests.length-beforeCSRF,post_requests:r.mutations.length-beforePosts,detached_render_hits:counts.renderHits,
          autofill_events:autofillEvents,ime_events:imeEvents,rollback_faults:counts.fault,capability_calls:counts.capabilityCalls,selected_input_frozen:name==='freeze-at-csrf'});
        await checkpoint('CONTROLLER_CHECKED',name);
      } finally {
        release?.(); csrfRelease?.(); if (routing) await p.unroute(origin+(name==='react-first'?'/assets/native-account.js':'/assets/account.js'),routing);
        if (csrfRouting) await p.unroute(origin+'/api/v2/auth/csrf',csrfRouting);
        await protocol.send('Page.removeScriptToEvaluateOnNewDocument',{identifier:injected}); if (protocol!==cdp) await protocol.detach();
      }
    }
    assert.equal(validRecords(records),true); assert.equal(storage.operations,0); assert.equal(storage.controls,4);
    return {records,...committed,operator_storage_operations:storage.operations,storage_positive_controls:storage.controls};
  } finally {page.off('response',assetResponse); anonymousPage.off('response',assetResponse); await cdp.detach(); await anonymous.close();}
}
module.exports={runControllerControls};
