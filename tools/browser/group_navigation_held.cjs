'use strict';
// Real form submission and retained native Account logout. No request injection,
// business setup API, fixture cookies, route mocks or client authority.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const {ADOPT_KEYS, TEXT} = require('./group_process_journey.cjs');
const PHASES = Object.freeze(['GROUP_HELD_FORM_READY','GROUP_HELD_SUBMITTED',
  'GROUP_HELD_READS_FENCED','GROUP_HELD_LOGGED_OUT']);
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
function id(value) { assert.match(value,UUID); assert.notEqual(value,'00000000-0000-0000-0000-000000000000');return value; }
function validEvidence(result) {
  try {
    assert.deepEqual(Object.keys(result).sort(),['group','company','account','retained_logout',
      'checkpoints','reads','screenshots','command','process','genuine_typed_form','reflow_320',
      'no_business_storage','keyboard_submit','submitted','no_result_or_form_after_fence',
      'logout_csrf','logout_root','logout_cookie_clear','real_ui_logout'].sort());
    for (const key of ['account','company','group','command','process']) id(result[key]);
    assert.deepEqual(result.checkpoints.map(row=>row.phase),PHASES);
    assert.equal(result.checkpoints.every(row=>row.owner_effects_verified===true),true);
    assert.equal(result.retained_logout.path,'/account'); assert.equal(result.retained_logout.status,200);
    assert.deepEqual(result.logout_root,{path:'/',status:200,redirected:false});
    assert.deepEqual(result.logout_csrf,{path:'/api/v2/auth/csrf',status:200});
    const base=`/groups/${result.group}/identity`;
    assert.deepEqual(result.reads,['/account',base,base+'/processes/new',
      `${base}/processes/${result.process}/replace`,`${base}/processes/${result.process}/suspend`,
      `${base}/requests/${result.command}`,`${base}/requests/${result.command}/retry`,'/']
      .map(path=>({path,status:503})));
    assert.equal(result.submitted.status,503); assert.equal(result.submitted.command,result.command);
    assert.match(result.submitted.body_sha256,/^[0-9a-f]{64}$/);
    for (const key of ['genuine_typed_form','keyboard_submit','reflow_320','no_business_storage',
      'no_result_or_form_after_fence','real_ui_logout','logout_cookie_clear']) assert.equal(result[key],true);
    assert.equal(result.checkpoints[0].confirmed_correction,true);
    assert.equal(result.checkpoints[0].fresh_account_positive.actual_projection_wait,true);
    assert.equal(result.checkpoints[0].fresh_account_positive.status,200);
    assert.equal(result.checkpoints[0].held_account_ordering.actual_projection_wait,false);
    assert.equal(result.checkpoints[0].held_account_ordering.status,503);
    assert.equal(result.checkpoints[0].held_account_ordering.blocker_retained_until_held_response,true);
    assert.deepEqual(result.checkpoints[0].fresh_account_after_held,
      {path:'/account',status:200,authorized_account:true,after_held_lock_cleanup:true,complete_rows_unchanged:true});
    assert.deepEqual(result.checkpoints[0].mounted_owner_interruption,
      {path:`/groups/${result.group}/identity/requests/${result.command}`,http_timeout_ms:500,
        actual_owner_selector_wait:true,blocker_retained_until_timeout:true,status:408,
        exact_original_locator:true,owner_quiesced_without_forced_cancel:true,complete_rows_unchanged:true,
        actual_lock_cleanup:true,status_readmission:404,account_readmission:200,authorized_account:true});
    for(const row of [result.checkpoints[0].fresh_account_positive,result.checkpoints[0].held_account_ordering]) {
      assert.equal(row.complete_rows_unchanged,true);assert.equal(row.actual_lock_cleanup,true);
    }
    assert.equal(result.checkpoints[3].real_logout_revocation_verified,true);
    assert.equal(result.checkpoints[3].revoked_session_denied_without_effects,true);
    assert.deepEqual(result.screenshots,['group-held-form-before-correction-320.png',
      'group-held-form-submission-interrupted.png']);
    return true;
  } catch {return false;}
}

async function retainLogoutPage({context,origin,configureResponseRetention}) {
  const page=await context.newPage();
  try {
    const session=await context.newCDPSession(page);
    await configureResponseRetention(session);
    const response=await page.goto(origin+'/account',{waitUntil:'domcontentloaded',timeout:8000});
    assert.equal(response.status(),200); assert.equal(response.request().redirectedFrom(),null);
    assert.equal(page.url(),origin+'/account');
    const control=page.getByRole('button',{name:'로그아웃',exact:true});
    assert.equal(await control.count(),1); assert.equal(await control.isVisible(),true);
    assert.equal(await page.locator('form[data-native-action="logout"]').count(),1);
    return {page,session,evidence:{path:'/account',status:200,redirected:false,native_logout:true}};
  } catch (error) {await page.close();throw error;}
}

async function runHeldJourney({page,context,group,company,account,retainedLogout,
  exchange,capture,tabTo,expectDocument,expectMutation,secretFree,clearBrowserCache}) {
  id(group); id(company); id(account);
  const origin=new URL(page.url()).origin,base=`/groups/${group}/identity`;
  const result={group,company,account,retained_logout:retainedLogout.evidence,
    checkpoints:[],reads:[],screenshots:[]};
  async function witness(phase,fields={}) {
    const answer=await exchange({phase,group_id:group,...fields});
    assert.equal(answer.phase,phase);assert.equal(answer.owner_effects_verified,true);
    result.checkpoints.push(answer);return answer;
  }
  async function open(path,status) {
    expectDocument('GET',path,status,false);await clearBrowserCache();
    const response=await page.goto(origin+path,{waitUntil:'domcontentloaded',timeout:8000});
    assert.equal(page.url(),origin+path);assert.equal(response.status(),status);
    assert.equal(response.request().redirectedFrom(),null);await secretFree();
    result.reads.push({path,status});return response;
  }
  async function nondisclosing(response) {
    const html=await response.text();const visible=await page.locator('body').innerText();
    for(const secret of [group,company,result.command,result.process,...Object.values(TEXT)]) {
      assert.equal(html.includes(secret)||visible.includes(secret),false);
    }
    assert.equal(await page.locator('[data-group-process-state], [data-group-process-terminal-code], main form').count(),0);
    assert.equal(html.includes('csrf_proof'),false);
    assert.equal(response.headers()['cache-control'],'no-store');
    result.no_result_or_form_after_fence=true;
  }
  // The typed form is genuinely issued by the original-profile Group owner.
  // Use the actual Group from the preceding visible Company creation receipt.
  expectDocument('GET',base+'/processes/new',200,false);await clearBrowserCache();
  const initial=await page.goto(origin+base+'/processes/new',{waitUntil:'domcontentloaded',timeout:8000});
  assert.equal(initial.status(),200);assert.equal(initial.request().redirectedFrom(),null);
  const form=page.getByRole('main').locator(`form[action="${base}/processes"]`);
  assert.equal(await form.count(),1);assert.equal(await form.isVisible(),true);
  const controls=await form.evaluate(element=>({method:Object.getOwnPropertyDescriptor(HTMLFormElement.prototype, 'method').get.call(element),enctype:element.enctype,target:element.target,
    keys:[...element.elements].filter(row=>row.name).map(row=>row.name),
    disabled:[...element.elements].filter(row=>row.name).some(row=>row.disabled),
    proof:[...element.elements].filter(row=>row.name==='csrf_proof').map(row=>({type:row.type,
      valid:typeof row.value==='string'&&row.value.length>0&&row.value.length<=8192}))}));
  assert.deepEqual(controls.keys.slice().sort(),ADOPT_KEYS.slice().sort());
  assert.equal(controls.method,'post');assert.equal(controls.enctype,'application/x-www-form-urlencoded');
  assert.equal(controls.target,'');assert.equal(controls.disabled,false);
  assert.deepEqual(controls.proof,[{type:'hidden',valid:true}]);
  result.command=id(await form.locator('[name="command_id"]').inputValue());
  result.process=id(await form.locator('[name="process_id"]').inputValue());
  for(const [key,value] of Object.entries(TEXT)) {
    const input=form.locator(`[name="${key}"]`);assert.equal(await input.count(),1);
    if(key==='method')await input.selectOption(value);else await input.fill(value);
    assert.equal(await input.evaluate(element=>element.labels?.length>0||element.hasAttribute('aria-label')),true);
  }
  const expiry=new Date(Date.now()+(30*86400+9*3600)*1000).toISOString().slice(0,19);
  await form.locator('[name="process_expiry"]').fill(expiry);
  await form.locator('[name="operator_responsibility"]').check();
  assert.equal(await form.evaluate(element=>element.checkValidity()),true);
  const named=await form.evaluate(element=>[...new FormData(element).entries()].filter(([key])=>key!=='csrf_proof'));
  assert.equal(new Set(named.map(([key])=>key)).size,named.length);
  assert.equal(named.some(([key])=>key==='csrf_proof'),false);result.genuine_typed_form=true;
  await page.setViewportSize({width:320,height:900});
  assert.equal(await page.evaluate(()=>Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)<=document.documentElement.clientWidth+1),true);
  result.reflow_320=true;
  assert.equal(await page.evaluate(values=>{
    const storage=JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)});
    return values.every(value=>!storage.includes(value));
  },Object.values(TEXT)),true);result.no_business_storage=true;
  await secretFree();await capture('group-held-form-before-correction-320');
  result.screenshots.push('group-held-form-before-correction-320.png');
  const answer=await witness('GROUP_HELD_FORM_READY',{command_id:result.command,process_id:result.process,fields:named});
  assert.equal(answer.confirmed_correction,true);
  // The DOM, actual hidden proof and every input stay alive across the real
  // database correction. Submit via the genuine visible keyboard control.
  expectMutation(base+'/processes');expectDocument('POST',base+'/processes',503,false);
  const waiting=page.waitForResponse(row=>row.url()===origin+base+'/processes'&&row.request().method()==='POST');
  const button=form.getByRole('button',{name:/.+/});assert.equal(await button.count(),1);
  await tabTo(button,64);await page.keyboard.press('Enter');result.keyboard_submit=true;
  const submitted=await waiting;assert.equal(submitted.status(),503);
  assert.equal(submitted.request().redirectedFrom(),null);
  const wire=new URLSearchParams(submitted.request().postData());
  assert.deepEqual([...wire.entries()].filter(([key])=>key!=='csrf_proof'),named);
  assert.equal(wire.getAll('csrf_proof').length,1);assert.ok(wire.get('csrf_proof'));
  await page.waitForURL(origin+base+'/processes');await nondisclosing(submitted);await secretFree();
  const body_sha256=crypto.createHash('sha256').update(submitted.request().postData()).digest('hex');
  result.submitted={command:result.command,status:503,body_sha256};
  await witness('GROUP_HELD_SUBMITTED',{command_id:result.command,process_id:result.process,
    fields:named,body_sha256,status:503});
  await capture('group-held-form-submission-interrupted');result.screenshots.push('group-held-form-submission-interrupted.png');
  for(const path of ['/account',base,base+'/processes/new',
    `${base}/processes/${result.process}/replace`,`${base}/processes/${result.process}/suspend`,
    `${base}/requests/${result.command}`,`${base}/requests/${result.command}/retry`]) {
    await nondisclosing(await open(path,503));
  }
  // One authenticated root-read control completes the eight real browser reads.
  await nondisclosing(await open('/',503));
  await witness('GROUP_HELD_READS_FENCED');
  expectMutation('/api/v2/auth/logout');
  const logoutPage=retainedLogout.page;
  const logout=logoutPage.waitForResponse(row=>row.url()===origin+'/api/v2/auth/logout'&&row.request().method()==='POST');
  const csrf=logoutPage.waitForResponse(row=>row.url()===origin+'/api/v2/auth/csrf'&&row.request().method()==='GET');
  const root=logoutPage.waitForResponse(row=>row.url()===origin+'/'&&row.request().isNavigationRequest());
  const buttonLogout=logoutPage.getByRole('button',{name:'로그아웃',exact:true});
  assert.equal(await buttonLogout.count(),1);assert.equal(await buttonLogout.isVisible(),true);
  await buttonLogout.focus();assert.equal(await buttonLogout.evaluate(element=>document.activeElement===element),true);
  await logoutPage.keyboard.press('Enter');
  const loggedOut=await logout;assert.equal(loggedOut.status(),200);
  const issued=await csrf;assert.equal(issued.status(),200);
  result.logout_csrf={path:'/api/v2/auth/csrf',status:200};
  assert.deepEqual(await loggedOut.json(),{outcome:'COMMITTED'});
  await logoutPage.waitForURL(origin+'/');
  const signedOut=await root;assert.equal(signedOut.status(),200);
  assert.equal(signedOut.request().redirectedFrom(),null);
  result.logout_root={path:'/',status:200,redirected:false};
  result.logout_cookie_clear=(await context.cookies(origin)).every(cookie=>
    !['__Host-console_account_session','__Host-console_account_refresh'].includes(cookie.name));
  assert.equal(result.logout_cookie_clear,true);result.real_ui_logout=true;
  await witness('GROUP_HELD_LOGGED_OUT');
  await retainedLogout.session.detach();
  await logoutPage.close();assert.equal(validEvidence(result),true);return result;
}
module.exports={PHASES,validEvidence,retainLogoutPage,runHeldJourney};
