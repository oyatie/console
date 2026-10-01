'use strict';
// PRIVATE EXECUTABLE TEST PROPOSAL. Call only from the genuine source-bound
// Company browser/DB harness. No fixtures, response stubs or test-only UI state.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const {assertRecoveryForms} = require('./recovery_controls.cjs');
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
function id(value) { assert.equal(typeof value, 'string'); assert.match(value, UUID); assert.notEqual(value, '00000000-0000-0000-0000-000000000000'); return value; }
const PHASES = ['CATALOG_INSTALLED', 'GRANT_COMMITTED', 'GRANT_REOPENED', 'REVOKE_COMMITTED', 'REVOKE_REOPENED'];
const ACTIONS = ['InstallPayrollReadCatalogV1', 'GrantPayrollReadV1', 'RevokePayrollReadV1'];
const COPY = Object.freeze({
  title: '급여 목록 열람 권한',
  heading: '이 권한의 공개 범위',
  consequence: '권한이 연결되어 유효하고 현재 정책이 허용할 때, 대상 관리 계정이 선택한 회사의 급여 목록과 목록에 포함된 모든 항목을 확인할 수 있습니다. 근태 마감 증빙 전체와 결정 사유도 포함됩니다.',
  scope: '급여의 상세 내역·수정·지급·내보내기 권한은 연결되지 않습니다. 다른 회사에는 적용되지 않습니다.',
  expiry: '종료 날짜와 시각 (한국 표준시, UTC+09:00)',
});
const TASK_SITES = ['INSTALL_PREFLIGHT','CATALOG_INSTALLED','GRANT_PREFLIGHT','GRANT_COMMITTED','GRANT_REOPENED','GRANT_RETURNED','REVOKE_PREFLIGHT','REVOKE_COMMITTED','REVOKE_REOPENED'];
const TARGET_RESTRICTION = '회사 등록 시 지정된 관리 계정에 한해 연결할 수 있습니다.';
const SAME_ACCOUNT = '현재 로그인 계정이 권한 대상입니다';
function taskIssues(row, scope) {
  const issues = [];
  const equal = (a,b) => JSON.stringify(a) === JSON.stringify(b);
  if(row?.heading !== COPY.heading || row?.description !== COPY.consequence || row?.limit !== COPY.scope || row?.heading_visible !== true || row?.description_visible !== true || row?.limit_visible !== true || row?.consequence_text !== [COPY.heading,COPY.consequence,COPY.scope].join(' ')) issues.push('CONDITIONAL_SCOPE');
  const primary = [['회사',scope.company_name],['권한 대상','회사 등록 시 지정된 관리 계정'],['작업 담당','현재 로그인 계정']];
  const prose = [TARGET_RESTRICTION];
  if(scope.recipient === scope.operator && row?.primary_prose?.includes(SAME_ACCOUNT)) prose.push(SAME_ACCOUNT);
  if(!equal(row?.primary,primary) || !equal(row?.primary_prose,prose) || row?.primary_visible !== true || row?.primary_text !== primary.flat().concat(prose).join(' ')) issues.push('PRIMARY_CONTEXT');
  const identifiers = [['회사',scope.company],['그룹',scope.group],['권한 대상',scope.recipient],['현재 담당 계정',scope.operator]];
  const attributes = [['data-policy-company',scope.company],['data-policy-recipient',scope.recipient],['data-policy-operator',scope.operator]];
  if(row?.details_count !== 1 || row?.details_native !== true || row?.details_title !== '회사·계정 식별 정보' || row?.initially_closed !== true || row?.summary_tab_index !== 0 || row?.summary_visible !== true ||
    !equal(row?.identifiers,identifiers) || !equal(row?.identity_attributes,attributes) || row?.attributes_in_details !== true || row?.identifiers_hidden !== true) issues.push('IDENTIFIER_DETAILS');
  if(row?.tab_discovered !== true || row?.focus_visible !== true || row?.keyboard_opened !== true || row?.keyboard_closed !== true || row?.identifiers_visible !== true || row?.network_requests !== 0 ||
    row?.values_preserved !== true || row?.location_preserved !== true || row?.unique_ids !== true || row?.no_product_script !== true) issues.push('DISCLOSURE_INTERACTION');
  return issues;
}
function taskEvidenceIssues(e) {
  try {
    for(const key of ['company','group','recipient','operator']) id(e[key]);
    assert.equal(typeof e.company_name,'string'); assert.ok(e.company_name.length > 0);
    assert.equal(new URL(e.origin).origin,e.origin); assert.match(e.origin,/^https:\/\/localhost:\d+$/);
    const commands=e.mutations.map(m=>id(m.command_id)); assert.equal(commands.length,3);
    const base=`${e.origin}/companies/${e.company}/policy/payroll-read`;
    const install=base+'/requests/install/'+commands[0], grant=base+'/requests/grant/'+commands[1], revoke=base+'/requests/revoke/'+commands[2];
    const urls=[base+'/install',install,base+'/grant',grant,grant,grant,base+'/revoke',revoke,revoke];
    const states=['NONE','NONE','NONE','ACTIVE','ACTIVE','ACTIVE','ACTIVE','REVOKED','REVOKED'];
    assert.deepEqual(e.ui_observations.map(r=>r.site),TASK_SITES);
    return e.ui_observations.flatMap((row,i)=>[
      ...(row.url===urls[i] && row.current_state===states[i] ? [] : [`${row.site}:CURRENT_LOCATION`]),
      ...taskIssues(row,e).map(issue=>`${row.site}:${issue}`),
    ]);
  } catch { return ['TASK_HISTORY_INCOMPLETE']; }
}
function validTaskEvidence(e) { return taskEvidenceIssues(e).length === 0; }
function collectTask(site) {
  const visible=e=>e?.checkVisibility({checkOpacity:true,checkVisibilityCSS:true})===true;
  const normalize=text=>String(text ?? '').trim().replace(/\s+/g,' ');
  const panel=[...document.querySelectorAll('section.panel')].find(e=>e.querySelector('h2')?.textContent==='대상과 업무 범위');
  const details=panel ? [...panel.querySelectorAll('details')].filter(e=>e.querySelector('summary')?.textContent==='회사·계정 식별 정보') : [];
  const selected=details.length===1 ? details[0] : null;
  const summary=selected?.querySelector('summary');
  const primaryNodes=panel ? [...panel.querySelectorAll('dt,p')].filter(e=>!e.closest('details')) : [];
  const pairs=root=>root ? [...root.querySelectorAll('dt')].filter(dt=>root===selected || !dt.closest('details')).map(dt=>[dt.textContent,dt.nextElementSibling?.textContent]) : [];
  const attrs=['data-policy-company','data-policy-recipient','data-policy-operator'].flatMap(name=>[...document.querySelectorAll(`[${name}]`)].map(e=>[name,e.getAttribute(name)]));
  const consequence=document.querySelector('.policy-consequences');
  const heading=consequence?.querySelector('h2'),description=consequence?.querySelector('p'),limit=consequence?.querySelector('.policy-limit');
  const body=panel?.querySelector('.policy-panel-body');
  const walker=body && document.createTreeWalker(body,NodeFilter.SHOW_TEXT);
  const primaryText=[];
  if(walker)while(walker.nextNode()){
    const node=walker.currentNode;
    if(!node.parentElement.closest('details') && visible(node.parentElement))primaryText.push(node.textContent);
  }
  const ids=[...document.querySelectorAll('[id]')].map(e=>e.id);
  return {site,url:location.href,current_state:document.querySelector('[data-policy-current-state]')?.getAttribute('data-policy-current-state') ?? null,
    heading:heading?.textContent ?? null,description:description?.textContent ?? null,limit:limit?.textContent ?? null,
    heading_visible:visible(heading),description_visible:visible(description),limit_visible:visible(limit),consequence_text:normalize(consequence?.innerText),
    primary:pairs(panel),primary_prose:panel ? [...panel.querySelectorAll('p')].filter(p=>!p.closest('details')).map(p=>p.textContent) : [],
    primary_visible:primaryNodes.length>0 && primaryNodes.every(e=>visible(e) && (e.tagName!=='DT' || visible(e.nextElementSibling))),primary_text:normalize(primaryText.join(' ')),
    details_count:details.length,details_native:selected?.tagName==='DETAILS',details_title:selected?.querySelector('summary')?.textContent ?? null,
    initially_closed:selected?.open===false,summary_tab_index:summary?.tabIndex ?? null,summary_visible:visible(summary),identifiers:pairs(selected),identity_attributes:attrs,
    attributes_in_details:attrs.length===3 && ['data-policy-company','data-policy-recipient','data-policy-operator'].every(name=>[...document.querySelectorAll(`[${name}]`)].every(e=>selected?.contains(e))),
    identifiers_hidden:selected!==null && [...selected.querySelectorAll('dd')].every(e=>!visible(e)),
    unique_ids:new Set(ids).size===ids.length,no_product_script:document.scripts.length===0};
}
async function observeTask(page, context, site, tabTo) {
  // New presentation findings are evidence, not an early gate: retain every
  // immediate owner/transport assertion and the complete ADMIN_REOPENED history.
  const row = await page.evaluate(collectTask,site);
  row.tab_discovered=false; row.focus_visible=false; row.keyboard_opened=false; row.keyboard_closed=false; row.identifiers_visible=false; row.network_requests=0;
  const snapshot=()=>page.evaluate(()=>JSON.stringify({url:location.href,forms:[...document.forms].map(f=>[...new FormData(f)])}));
  const before=await snapshot(); const url=page.url();
  const request=()=>row.network_requests++;
  context.on('request',request);
  try {
    if(row.details_count===1 && row.details_native && row.initially_closed) {
      const summary=page.locator('section.panel details > summary').filter({hasText:/^회사·계정 식별 정보$/});
      if(await summary.count()===1) {
        try {await tabTo(summary,48);row.tab_discovered=await summary.evaluate(e=>document.activeElement===e);}
        catch(error){if(!['KEYBOARD_DISCOVERY','KEYBOARD_FOCUS'].includes(error?.code))throw error;}
        if(row.tab_discovered){
          row.focus_visible=await summary.evaluate(e=>e.matches(':focus-visible'));
          await page.keyboard.press('Enter');row.keyboard_opened=await summary.evaluate(e=>e.parentElement.open===true);
          row.identifiers_visible=await summary.evaluate(e=>[...e.parentElement.querySelectorAll('dd')].length===4 && [...e.parentElement.querySelectorAll('dd')].every(d=>d.checkVisibility({checkOpacity:true,checkVisibilityCSS:true})));
          await page.keyboard.press('Space');row.keyboard_closed=await summary.evaluate(e=>e.parentElement.open===false);
        }
      }
    }
    await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    row.values_preserved=(await snapshot())===before; row.location_preserved=page.url()===url;
  } finally {context.off('request',request);}
  return row;
}
function validEvidence(e) {
  try {
    id(e.company); id(e.recipient); id(e.operator);
    assert.equal(e.unexpected_mutations, 0); assert.equal(e.mutation_failures, 0);
    assert.deepEqual(e.checkpoints.map(x => x.phase), PHASES);
    assert.equal(e.mutations.length, 3);
    assert.deepEqual(e.mutations.map(x => x.operation), ACTIONS);
    assert.equal(new Set(e.mutations.map(x => x.command_id)).size, 3);
    for (const m of e.mutations) {
      id(m.command_id); assert.equal(m.method, 'POST'); assert.equal(m.status, 303);
      assert.equal(m.same_origin, true); assert.equal(m.request_count, 1);
      assert.match(m.body_sha256, /^[0-9a-f]{64}$/);
      assert.match(m.expected_company_epoch, /^[1-9][0-9]*$/);
    }
    for (let i = 0; i < e.checkpoints.length; i++) {
      const w = e.checkpoints[i]; const m = e.mutations[i === 0 ? 0 : i >= 3 ? 2 : 1];
      assert.equal(w.company, e.company); assert.equal(w.recipient, e.recipient);
      assert.equal(w.actor_account_id,e.operator);
      assert.equal(w.command_id, m.command_id); id(w.receipt_id);
      assert.equal(w.owner_effects_verified, true); assert.equal(w.unrelated_bytes_preserved, true);
      assert.equal(w.original_discovery_preserved, true);
      assert.match(w.company_epoch, /^[1-9][0-9]*$/);
      if (i !== 2 && i !== 4) assert.equal(BigInt(w.company_epoch), BigInt(m.expected_company_epoch) + 1n);
    }
    assert.equal(new Set([e.checkpoints[0].receipt_id,e.checkpoints[1].receipt_id,e.checkpoints[3].receipt_id]).size,3);
    assert.equal(e.mutations[1].expected_company_epoch,e.checkpoints[0].company_epoch);
    assert.equal(e.mutations[2].expected_company_epoch,e.checkpoints[2].company_epoch);
    assert.equal(e.checkpoints[2].receipt_id, e.checkpoints[1].receipt_id);
    assert.equal(e.checkpoints[2].company_epoch, e.checkpoints[1].company_epoch);
    id(e.checkpoints[1].assignment_id);
    assert.equal(e.checkpoints[1].assignment_revision, '1');
    assert.equal(e.checkpoints[2].assignment_id, e.checkpoints[1].assignment_id);
    assert.equal(e.checkpoints[2].assignment_revision, '1');
    assert.equal(e.checkpoints[3].assignment_id, e.checkpoints[1].assignment_id);
    assert.equal(e.checkpoints[3].assignment_revision, '2');
    assert.equal(e.checkpoints[1].state, 'ACTIVE'); assert.equal(e.checkpoints[2].state, 'ACTIVE');
    assert.equal(e.checkpoints[3].state, 'REVOKED');
    const revoked=e.checkpoints[3], reopened=e.checkpoints[4];
    for(const key of ['receipt_id','command_id','company_epoch','assignment_id','assignment_revision']) assert.equal(reopened[key],revoked[key]);
    assert.equal(reopened.state,'REVOKED');assert.equal(reopened.current_state,'REVOKED');
    assert.equal(reopened.reload_had_no_effects,true);
    assert.equal(e.install_did_not_grant, true);
    assert.equal(e.grant_receipt_reopened, true); assert.equal(e.revoked_receipt_reopened, true);
    assert.equal(e.reflow_320, true); assert.equal(e.keyboard_submit, true);
    return true;
  } catch { return false; }
}
async function visibleText(page, text) { assert.equal(await page.getByText(text, { exact: true }).isVisible(), true); }
function exactRecoveryHref(href, {origin,company,operation,command}) {
  try {
    id(company);id(command);assert.ok(['install','grant','revoke'].includes(operation));assert.equal(new URL(origin).origin,origin);
    const path=`/companies/${company}/policy/payroll-read/requests/${operation}/${command}`;
    return href===path && new URL(href,origin).href===origin+path;
  } catch {return false;}
}
const FORBIDDEN_FIELDS=['company','recipient','operator','receipt','epoch'];
function deniedMaterialSafe(material, forbidden) {
  try {
    assert.deepEqual(Object.keys(forbidden).sort(),[...FORBIDDEN_FIELDS].sort());
    for(const field of FORBIDDEN_FIELDS) {
      assert.ok(Array.isArray(forbidden[field])&&forbidden[field].length>0);
      assert.ok(forbidden[field].every(v=>typeof v==='string'&&v.length>0));
    }
    for(const key of ['responseText','domHtml','visibleText','scriptText']) assert.equal(typeof material[key],'string');
    assert.ok(Array.isArray(material.attributeValues));
    const decode=s=>s.replace(/\\u([0-9a-f]{4})/gi,(_,h)=>String.fromCharCode(parseInt(h,16)));
    const complete=[material.responseText,material.domHtml,material.visibleText,material.scriptText].map(decode);
    for(const field of FORBIDDEN_FIELDS)for(const value of forbidden[field]) {
      if(field!=='epoch') { if(complete.some(s=>s.includes(value)))return false; }
      else {
        // Short revision numbers are not globally unique. Inspect visible text,
        // exact decoded attribute values and named serialized revision fields;
        // parent field-projection oracle covers full source semantics below.
        assert.match(value,/^[1-9][0-9]*$/);
        if(new RegExp('(^|[^0-9])'+value+'([^0-9]|$)').test(material.visibleText))return false;
        const named=new RegExp('["\']?(?:company_epoch|expected_company_epoch|epoch)["\']?\\s*:\\s*["\']?'+value+'(?:["\']|[^0-9]|$)');
        if(complete.some(s=>named.test(s)))return false;
      }
      if(material.attributeValues.some(v=>v===value))return false;
    }
    return true;
  } catch {return false;}
}
async function assertOutcome(page, state, expected) {
  const outcome = page.locator(`[data-policy-outcome="${state}"]`);
  await outcome.waitFor({state:'visible'});
  assert.equal(await outcome.count(), 1); assert.equal(await outcome.isVisible(), true);
  if (state === 'denied') assert.equal(await page.locator('form').count(), 0);
  if (['uncertain','pending','expired','not-visible'].includes(state)) await assertRecoveryForms(page,state,expected);
  if (state === 'denied') {
    assert.equal(await page.locator('[data-policy-recipient],[data-policy-company],[data-policy-receipt]').count(), 0);
    await visibleText(page, '이 요청을 열거나 변경할 권한이 없습니다');
    assert.equal(typeof expected?.verifyDeniedProjection,'function');
    assert.ok(expected.response);assert.ok([403,404].includes(expected.response.status()));
    assert.equal(new URL(expected.documentUrl).origin,expected.origin);
    assert.equal(expected.response.url(),expected.documentUrl);assert.equal(page.url(),expected.documentUrl);
    assert.equal(expected.response.request().method(),'GET');
    assert.equal(expected.response.request().resourceType(),'document');
    assert.equal(expected.response.request().isNavigationRequest(),true);
    const material=await page.evaluate(()=>({
      domHtml:document.documentElement.outerHTML,visibleText:document.body.innerText,
      scriptText:[...document.scripts].map(s=>s.textContent).join('\n'),
      attributeValues:[...document.querySelectorAll('*')].flatMap(e=>[...e.attributes].map(a=>a.value)),
    }));
    material.responseText=await expected.response.text();
    assert.equal(deniedMaterialSafe(material,expected.forbidden),true);
    // Mandatory actual parent projection oracle; never derive a proof from marker
    // absence. It receives complete real response/DOM and exact source roster.
    const proof=await expected.verifyDeniedProjection({material,forbidden:expected.forbidden,documentUrl:expected.documentUrl});
    assert.equal(proof.source_projection_verified,true);
    const digest=v=>crypto.createHash('sha256').update(JSON.stringify(v)).digest('hex');
    assert.equal(proof.material_sha256,digest(material));assert.equal(proof.forbidden_sha256,digest(expected.forbidden));
  }
  if (state === 'uncertain') {
    await visibleText(page, '아직 처리 결과를 확인할 수 없습니다');
    const recheck = page.getByRole('link', {name: '같은 요청 결과 다시 확인', exact: true});
    assert.equal(await recheck.isVisible(), true);
    assert.equal(exactRecoveryHref(await recheck.getAttribute('href'),expected),true);
    assert.equal(new URL(page.url()).origin,expected.origin);
  }
}
async function runPolicyJourney({ page, context, company, group, companyName, recipient, operator, expiresAtLocal, checkpoint, capture, beforeSubmit, expectDocument, readPayroll, tabTo }) {
  id(company); id(group); id(recipient); id(operator); assert.equal(typeof companyName,'string'); assert.ok(companyName.length > 0);
  assert.match(expiresAtLocal, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
  assert.equal(typeof checkpoint, 'function'); assert.equal(typeof capture, 'function');
  for (const fn of [beforeSubmit,expectDocument,readPayroll,tabTo]) assert.equal(typeof fn,'function');
  const origin = new URL(page.url()).origin; assert.match(origin, /^https:\/\/localhost:\d+$/);
  const base = `/companies/${company}/policy/payroll-read`;
  const result = {company,group,company_name:companyName,origin,recipient,operator,mutations:[],checkpoints:[],ui_observations:[],unexpected_mutations:0,mutation_failures:0};
  const requests = new Map(); let active = null;
  const onRequest = request => {
    if (['GET','HEAD','OPTIONS'].includes(request.method())) return;
    try {
      const url = new URL(request.url());
      if (!active || request.method() !== 'POST' || url.origin !== origin || url.pathname !== active.path || requests.has(request)) { result.unexpected_mutations++; return; }
      const body = request.postData(); assert.equal(typeof body, 'string');
      const fields = new URLSearchParams(body);
      assert.deepEqual([...fields].filter(([name])=>name!=='csrf_proof').sort(),active.fields);
      assert.equal(fields.getAll('csrf_proof').length,1);
      assert.equal(fields.getAll('command_id').length,1); assert.equal(fields.get('command_id'),active.command_id);
      assert.equal(fields.getAll('expected_company_epoch').length,1);
      assert.equal(fields.get('expected_company_epoch'),active.expected_company_epoch);
      const row = {...active,method:'POST',same_origin:true,status:null,request_count:1,body_sha256:crypto.createHash('sha256').update(body).digest('hex')};
      requests.set(request,row);result.mutations.push(row);
    } catch { result.mutation_failures++; }
  };
  const onResponse = response => { const row=requests.get(response.request()); if(row) { if(row.status!==null) result.mutation_failures++; row.status=response.status(); } };
  const onFailure = request => { if(requests.has(request)) result.mutation_failures++; };
  context.on('request',onRequest);context.on('response',onResponse);context.on('requestfailed',onFailure);
  async function navigate(operation) {
    assert.ok(['install','grant','revoke'].includes(operation));
    const path=base+'/'+operation;
    expectDocument('GET',path,200,false);const response=await page.goto(origin+path);assert.equal(response.status(),200);
    assert.equal(page.url(),origin+path);assert.equal(response.request().redirectedFrom(),null);
    assert.equal(await page.locator('[data-policy-preflight-operation]').getAttribute('data-policy-preflight-operation'),operation);
  }
  async function reopen() {
    const before=page.url();expectDocument('GET',new URL(before).pathname,200,false);const response=await page.reload();assert.equal(response.status(),200);
    assert.equal(page.url(),before);assert.equal(response.request().redirectedFrom(),null);
  }
  async function inspectSubject() {
    assert.equal(await page.getByRole('heading',{name:COPY.title,exact:true,level:1}).isVisible(),true);
    assert.equal(await page.locator('[data-policy-company]').getAttribute('data-policy-company'),company);
    assert.equal(await page.locator('[data-policy-recipient]').getAttribute('data-policy-recipient'),recipient);
    assert.equal(await page.locator('[data-policy-operator]').getAttribute('data-policy-operator'),operator);
    await visibleText(page,COPY.scope);
  }
  async function submit(operation,label,path) {
    const form=page.locator(`form[data-policy-operation="${operation}"]`);
    assert.equal(await form.count(),1);assert.equal(await form.getAttribute('method'),'post');assert.equal(await form.getAttribute('action'),path);
    const command=id(await form.locator('input[name="command_id"]').inputValue());
    active={operation,path,command_id:command,expected_company_epoch:await form.locator('input[name="expected_company_epoch"]').inputValue(),fields:await form.evaluate(f=>[...new FormData(f)].filter(([name])=>name!=='csrf_proof').sort())};
    const names=['command_id','expected_company_epoch',...(operation===ACTIONS[1]?['recipient_account_id','expected_role_revision','assignment_id','expected_assignment_revision','expires_at_local']:operation===ACTIONS[2]?['expected_role_revision','expected_assignment_revision']:[])].sort();
    assert.deepEqual(active.fields.map(([name])=>name),names);
    assert.ok(active.fields.every(([,value])=>typeof value==='string'));
    const kind={InstallPayrollReadCatalogV1:'install',GrantPayrollReadV1:'grant',RevokePayrollReadV1:'revoke'}[operation];
    await beforeSubmit({...active});
    expectDocument('POST',path,303,false);expectDocument('GET',base+'/requests/'+kind+'/'+command,200,true);
    const response=page.waitForResponse(r=>r.url()===origin+path&&r.request().method()==='POST');
    const button=form.getByRole('button',{name:label,exact:true});await button.focus();await page.keyboard.press('Enter');
    assert.equal((await response).status(),303);
    await page.waitForURL(origin+base+'/requests/'+kind+'/'+command);
    active=null; await assertOutcome(page,'committed');
    assert.equal(await page.locator('[data-policy-command]').getAttribute('data-policy-command'),command);
    return command;
  }
  async function witness(phase,command) {
    // Parent MUST independently inspect real DB/source/audit history, including
    // complete expected effect delta, before returning this finite acknowledgement.
    const w=await checkpoint({phase,company,recipient,operator,command_id:command});
    assert.equal(w.phase,phase);assert.equal(w.command_id,command);
    assert.equal(await page.locator('[data-policy-receipt]').getAttribute('data-policy-receipt'),w.receipt_id);
    result.checkpoints.push(w);return w;
  }
  try {
    // Entry harness has already navigated and independently witnessed this preflight.
    assert.equal(page.url(),origin+base+'/install');await inspectSubject();result.ui_observations.push(await observeTask(page,context,'INSTALL_PREFLIGHT',tabTo));
    const install=await submit(ACTIONS[0],'열람 권한 설정 준비',base+'/catalog');
    await visibleText(page,'설정 준비 완료');await visibleText(page,'계정에 열람 권한은 연결되지 않았습니다');
    const installed=await witness(PHASES[0],install); assert.equal(installed.business_assignment_count,0);result.install_did_not_grant=true;
    result.ui_observations.push(await observeTask(page,context,'CATALOG_INSTALLED',tabTo));
    await navigate('grant');await inspectSubject();
    await page.getByLabel(COPY.expiry,{exact:true}).fill(expiresAtLocal);
    result.ui_observations.push(await observeTask(page,context,'GRANT_PREFLIGHT',tabTo));
    await capture('policy-before-grant');
    const grant=await submit(ACTIONS[1],'열람 권한 연결',base+'/grants');
    const granted=await witness(PHASES[1],grant);await visibleText(page,'열람 권한을 연결했습니다');
    assert.equal(Date.parse(granted.valid_until),Date.parse(expiresAtLocal+':00+09:00'));
    assert.equal(granted.role_revision,'1');assert.equal(granted.role_valid_until,null);
    assert.ok(Date.parse(granted.valid_until)>Date.parse(granted.valid_from));
    assert.ok(Date.parse(granted.valid_until)-Date.parse(granted.valid_from)<=30*86400000);
    result.ui_observations.push(await observeTask(page,context,'GRANT_COMMITTED',tabTo));
    await reopen();await assertOutcome(page,'committed');await witness(PHASES[2],grant);result.grant_receipt_reopened=true;
    result.ui_observations.push(await observeTask(page,context,'GRANT_REOPENED',tabTo));
    await readPayroll();
    result.ui_observations.push(await observeTask(page,context,'GRANT_RETURNED',tabTo));
    await capture('policy-grant-reopened');
    await navigate('revoke');await inspectSubject();result.ui_observations.push(await observeTask(page,context,'REVOKE_PREFLIGHT',tabTo));await page.setViewportSize({width:320,height:900});
    result.reflow_320=await page.evaluate(()=>document.documentElement.scrollWidth<=document.documentElement.clientWidth);
    await capture('policy-active-320');
    const revoke=await submit(ACTIONS[2],'열람 권한 회수',base+'/grants/'+id(granted.assignment_id)+'/revoke');
    const revoked=await witness(PHASES[3],revoke);await visibleText(page,'열람 권한을 회수했습니다');
    result.ui_observations.push(await observeTask(page,context,'REVOKE_COMMITTED',tabTo));
    await reopen();await assertOutcome(page,'committed');
    assert.equal(await page.locator('[data-policy-command]').getAttribute('data-policy-command'),revoke);
    assert.equal(await page.locator('[data-policy-receipt]').getAttribute('data-policy-receipt'),revoked.receipt_id);
    assert.equal(await page.locator('[data-policy-current-state]').getAttribute('data-policy-current-state'),'REVOKED');
    const reopened=await witness(PHASES[4],revoke);
    assert.equal(reopened.current_state,'REVOKED');assert.equal(reopened.reload_had_no_effects,true);
    result.revoked_receipt_reopened=true;result.keyboard_submit=true;
    result.ui_observations.push(await observeTask(page,context,'REVOKE_REOPENED',tabTo));
    await capture('policy-revoked-reopened');assert.equal(validEvidence(result),true);return result;
  } finally { context.off('request',onRequest);context.off('response',onResponse);context.off('requestfailed',onFailure); }
}
module.exports={runPolicyJourney,assertOutcome,validEvidence,exactRecoveryHref,deniedMaterialSafe,COPY,TASK_SITES,taskIssues,taskEvidenceIssues,validTaskEvidence,collectTask};
