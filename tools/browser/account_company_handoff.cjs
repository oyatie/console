"use strict";
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const {configureResponseRetention,validCheckpointCommand,exactVisibleLink,validOrigin}=require('./company.cjs');
const UUID=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const nil='00000000-0000-0000-0000-000000000000';
const phases=['A_ENROLLED','O_ENROLLED','O_LOGGED_OUT','O_LOGGED_IN','HEALTHY_HANDOFF_ENTRY','COMPANY_COMMITTED','HANDOFF_REOPENED'];
function nonnil(v){return typeof v==='string'&&UUID.test(v)&&v!==nil;}
function fact(value,code){if(value!==true)throw Object.assign(new Error(code),{code});}
function expectedDocuments(r,successor=false){if(!nonnil(r.command_id)||!nonnil(r.org_id))return null;
 const saved=`/account/companies/requests/${r.command_id}`,company=`/companies/${r.org_id}`;
 const documents=[['A','/',200],['A','/account/register',200],['A','/account',200],['O','/',200],['O','/account/register',200],['O','/account',200],['O','/',200],['O','/account',200],['O','/account',200],['A','/account',200],['O','/account',200],['O','/account/companies/new',200],['O',saved,200],['O',saved,200],['O',company,404],['O',saved,200],['A','/account',200],['A',company,200],['A',company,200],['A',saved,404],['A','/account/companies/new',404]];
 if(successor){documents.splice(3,0,['B','/',200],['B','/account/register',200],['B','/account',200]);documents.push(['B','/account',200],['B',company,404],['B',saved,404],['B','/account/companies/new',404]);}
 return documents;
}
function expectedMutations(successor=false){const posts=[['A','/api/v2/auth/registration/start'],['A','/api/v2/auth/registration/finish'],['O','/api/v2/auth/registration/start'],['O','/api/v2/auth/registration/finish'],['O','/api/v2/auth/logout'],['O','/api/v2/auth/passkey/login/start'],['O','/api/v2/auth/passkey/login/finish'],['O','/api/v2/companies/enroll']];if(successor)posts.splice(2,0,['B','/api/v2/auth/registration/start'],['B','/api/v2/auth/registration/finish']);return posts;}
function validReceipt(sent,body,a,name,slug){return nonnil(a)&&sent&&body&&Object.keys(sent).sort().join(',')==='administrative_account_id,command_id,group_id,name,slug'&&nonnil(sent.command_id)&&sent.administrative_account_id===a&&sent.group_id===null&&sent.name===name&&sent.slug===slug&&Object.keys(body).sort().join(',')==='administrative_account_id,group_id,org_id,original_command_id,outcome,receipt_id,replayed,result_path'&&body.outcome==='COMMITTED'&&body.original_command_id===sent.command_id&&body.administrative_account_id===a&&body.replayed===false&&[body.org_id,body.group_id,body.receipt_id].every(nonnil)&&body.result_path===`/account/companies/requests/${sent.command_id}`;}

// Consume original owner bytes and decoded current DOM; emit booleans only.
function deniedProjectionSafe(material,forbidden){
 if(!material||!Array.isArray(forbidden)||forbidden.length===0||forbidden.some(v=>typeof v!=='string'||!v.length)||
  !['html','visible','text'].every(k=>typeof material[k]==='string'&&material[k].length<=262144)||
  !Array.isArray(material.attributes)||material.attributes.length>4096||!material.attributes.every(v=>typeof v==='string'&&v.length<=262144))return false;
 const escape=v=>v.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&#39;');
 // Match the current Rust inert-projection serializer, including JSON quoting.
 const projectionJSON=v=>JSON.stringify(v).slice(1,-1).replaceAll('&','\\u0026').replaceAll('<','\\u003c').replaceAll('>','\\u003e').replaceAll('\u2028','\\u2028').replaceAll('\u2029','\\u2029');
 return forbidden.every(v=>![material.html,material.visible,material.text,...material.attributes].some(text=>text.includes(v)||text.includes(escape(v))||text.includes(projectionJSON(v))));
}
async function deniedDocumentSafe(page,response,forbidden){
 const material=await page.evaluate(()=>{const elements=[...document.querySelectorAll('*')];if(elements.length>1024)return null;
  const attributes=elements.flatMap(e=>[...e.attributes].map(a=>a.value));if(attributes.length>4096)return null;
  return {visible:document.body?.innerText??'',text:document.documentElement.textContent??'',attributes};});
 if(material)material.html=await response.text();return deniedProjectionSafe(material,forbidden);
}
const MALFORMED_REFERENCE='not-an-account';
const storageControls=()=>['local','session'].flatMap(store=>['add:set','update:set','remove:set','add:property','remove:property','add:clear','clear'].map(operation=>store+':'+operation));
function storageSnapshotSafe(snapshot,forbidden){return Array.isArray(forbidden)&&forbidden.length>0&&forbidden.every(f=>typeof f==='string'&&f.length>0)&&!!snapshot&&Object.keys(snapshot).sort().join(',')==='local,session'&&
 ['local','session'].every(store=>Array.isArray(snapshot[store])&&snapshot[store].length<=128&&snapshot[store].every(row=>Array.isArray(row)&&row.length===2&&row.every(v=>typeof v==='string'&&v.length<=262144&&!forbidden.some(f=>v.includes(f)))));}
function storageTrace(origin,keys){
 const trace={controls:[],product_operations:0,observation_failures:0};
 trace.observe=(operation,row)=>{
  const id=row?.storageId,store=id?.isLocalStorage===true?'local':id?.isLocalStorage===false?'session':null;
  let own=false;try{const identity=id?.securityOrigin??id?.storageKey;own=typeof identity==='string'&&new URL(identity.split('^')[0]).origin===origin;}catch{}
  if(!store||!own){trace.observation_failures++;return;}
  if(operation==='clear'){
   if(trace.controls.at(-1)===store+':add:clear')trace.controls.push(store+':clear');else trace.product_operations++;
   return;
  }
  const purpose=Object.keys(keys).find(p=>row.key===keys[p]);
  const expectedValue=operation==='update'?'changed observer only':'observer only';
  if(purpose&&['add','update','remove'].includes(operation)&&(operation==='remove'||row.newValue===expectedValue))trace.controls.push(store+':'+operation+':'+purpose);
  else trace.product_operations++;
 };
 return trace;
}
function validStorageEvidence(e,role,documents){return !!e&&Object.keys(e).sort().join(',')==='control_events,forbidden_snapshot_absent,installed_documents,observation_failures,product_operations,role,state_preserved'&&e.role===role&&e.installed_documents===documents&&e.product_operations===0&&e.observation_failures===0&&e.state_preserved===true&&e.forbidden_snapshot_absent===true&&JSON.stringify(e.control_events)===JSON.stringify(storageControls());}
function completeStorageEvidence(r,successor=false){const documents=expectedDocuments(r,successor),roles=successor?['A','B','O']:['A','O'];return !!documents&&Array.isArray(r.storage)&&r.storage.length===roles.length&&r.storage.every((e,i)=>validStorageEvidence(e,roles[i],documents.filter(row=>row[0]===roles[i]).length));}
async function startStorageObserver(page,cdp,role,origin){
 const prefix='console-handoff-observer-'+crypto.randomUUID(),keys={set:prefix+'-set',property:prefix+'-property',clear:prefix+'-clear'},trace=storageTrace(origin,keys);let installed=0,initial;
 for(const [event,operation] of [['DOMStorage.domStorageItemAdded','add'],['DOMStorage.domStorageItemUpdated','update'],['DOMStorage.domStorageItemRemoved','remove'],['DOMStorage.domStorageItemsCleared','clear']])cdp.on(event,row=>trace.observe(operation,row));
 cdp.on('Runtime.bindingCalled',row=>{if(row.name==='__consoleHandoffStorageDocument'){if(row.payload==='DOCUMENT')installed++;else trace.observation_failures++;}});
 await cdp.send('Page.enable');await cdp.send('Runtime.enable');await cdp.send('DOMStorage.enable');await cdp.send('Runtime.addBinding',{name:'__consoleHandoffStorageDocument'});
 const script=(await cdp.send('Page.addScriptToEvaluateOnNewDocument',{source:'if (self===top) window.__consoleHandoffStorageDocument("DOCUMENT");'})).identifier;
 const snapshot=()=>page.evaluate(()=>({local:Object.entries(localStorage).sort(),session:Object.entries(sessionStorage).sort()}));
 async function drain(){await cdp.send('Runtime.evaluate',{expression:'void 0'});}
 return {
  positiveControl:async()=>{
   initial=await snapshot();fact(initial.local.length===0&&initial.session.length===0,'BUSINESS_STORAGE');
   const perform=async store=>{await page.evaluate(({keys,store})=>{const storage=store==='local'?localStorage:sessionStorage;
    storage.setItem(keys.set,'observer only');storage.setItem(keys.set,'changed observer only');storage.removeItem(keys.set);
    storage[keys.property]='observer only';delete storage[keys.property];storage.setItem(keys.clear,'observer only');storage.clear();
   },{keys,store});await drain();};
   await perform('local');fact(JSON.stringify(trace.controls)===JSON.stringify(storageControls().slice(0,7))&&trace.product_operations===0&&trace.observation_failures===0&&JSON.stringify(await snapshot())===JSON.stringify(initial),'BUSINESS_STORAGE');
   await perform('session');fact(JSON.stringify(trace.controls)===JSON.stringify(storageControls())&&trace.product_operations===0&&trace.observation_failures===0&&JSON.stringify(await snapshot())===JSON.stringify(initial),'BUSINESS_STORAGE');
  },
  evidence:async forbidden=>{await drain();const current=await snapshot();return {role,installed_documents:installed,control_events:[...trace.controls],product_operations:trace.product_operations,observation_failures:trace.observation_failures,state_preserved:JSON.stringify(current)===JSON.stringify(initial),forbidden_snapshot_absent:storageSnapshotSafe(current,forbidden)};},
  stop:async()=>{await cdp.send('Page.removeScriptToEvaluateOnNewDocument',{identifier:script});await cdp.send('Runtime.removeBinding',{name:'__consoleHandoffStorageDocument'});await cdp.send('DOMStorage.disable');}
 };
}

function validEvidence(r,successor=false){try{
 const docs=expectedDocuments(r,successor),posts=expectedMutations(successor);
 const expectedPhases=successor?[phases[0],'B_ENROLLED',...phases.slice(1)]:phases;
 const expectedMounts=['A_ACCOUNT','O_SETUP','O_HISTORY','O_HISTORY_REOPENED','A_ACCOUNT_REOPENED',...(successor?['B_ACCOUNT_REOPENED']:[])];
 return docs!==null&&validOrigin(r.origin)&&nonnil(r.a)&&nonnil(r.o)&&r.a!==r.o&&r.browser_version==='153.0.8010.12'&&r.observation_failures===0&&r.external_requests===0&&!r.relay_failure&&!r.tls_client_error&&r.checkpoints.join(',')===expectedPhases.join(',')&&
 r.documents.length===docs.length&&r.documents.every((row,i)=>Object.keys(row).sort().join(',')==='method,ordinal,path,redirected,role,status,url_exact'&&row.ordinal===i+1&&row.role===docs[i][0]&&row.path===docs[i][1]&&row.status===docs[i][2]&&row.method==='GET'&&row.redirected===false&&row.url_exact===true)&&
 r.mutations.length===posts.length&&r.mutations.every((row,i)=>Object.keys(row).sort().join(',')==='method,ordinal,path,role'&&row.ordinal===i+1&&row.method==='POST'&&row.role===posts[i][0]&&row.path===posts[i][1])&&
 JSON.stringify(r.csrf_requests)===JSON.stringify(['O','O'])&&JSON.stringify(r.react_mounts)===JSON.stringify(expectedMounts)&&
 completeStorageEvidence(r,successor)&&r.shared_reference===r.a&&
 (!successor||(r.kind==='REAL_REACT_COMPANY_INFORMATION_MANAGER_CURRENT_V1'&&nonnil(r.b)&&r.b!==r.a&&r.b!==r.o&&['b_registration','b_company_denied','b_foreign_history_denied','b_operator_denied'].every(k=>r[k]===true)))&&
 ['a_registration','o_registration','o_logout_login','own_reference','default_self','malformed_rejected','company_wire','o_history','o_company_denied','a_company_reopened','a_foreign_history_denied','a_operator_denied','input_not_stored','reflow_320'].every(k=>r[k]===true)&&
 validReceipt(r.sent,r.receipt,r.a,r.company_name,r.company_slug)&&r.command_id===r.sent.command_id&&r.org_id===r.receipt.org_id;
 }catch{return false;}}
function observe(context,role,origin,result){const owners=new WeakMap();
 context.on('request',request=>{try{
  const url=new URL(request.url()),canonical=url.origin===origin&&!url.search&&!url.hash&&!url.username&&!url.password;
  if(request.method()==='GET'&&url.pathname==='/api/v2/auth/csrf'){if(!canonical)result.observation_failures++;result.csrf_requests.push(role);}
  if(!['GET','HEAD','OPTIONS'].includes(request.method()))result.mutations.push({ordinal:result.mutations.length+1,role,method:request.method(),path:canonical?url.pathname:'<unexpected>'});
  if(request.resourceType()==='document'||request.isNavigationRequest()){
   const row={ordinal:result.documents.length+1,role,method:request.method(),path:canonical?url.pathname:'<unexpected>',redirected:request.redirectedFrom()!==null,status:null,url_exact:false};
   if(request.resourceType()!=='document'||!request.isNavigationRequest()||owners.has(request))result.observation_failures++;
   owners.set(request,row);result.documents.push(row);
  }
 }catch{result.observation_failures++;}});
 context.on('response',response=>{try{const request=response.request();if(request.resourceType()!=='document'&&!request.isNavigationRequest())return;const row=owners.get(request);if(!row||row.status!==null){result.observation_failures++;return;}row.status=response.status();row.url_exact=response.url()===origin+row.path;row.redirected ||= request.redirectedFrom()!==null;}catch{result.observation_failures++;}});
 context.on('requestfailed',request=>{if(request.resourceType()==='document'||request.isNavigationRequest()||!['GET','HEAD','OPTIONS'].includes(request.method()))result.observation_failures++;});
}
async function runHandoff({browser,origin,result:r,out,emit,receive,setStage,successor=false}){
 const participants=[];r.react_mounts=[];r.storage=[];
 async function exchange(phase,id,extra={}){emit({kind:'CHECKPOINT',phase,account_id:id,...extra});fact(validCheckpointCommand(await receive(),phase),'OWNER_PROTOCOL');r.checkpoints.push(phase);}
 async function capture(page,name){await page.screenshot({path:path.join(out,name),fullPage:true});r.screenshots.push(name);}
 async function reflow(page){return page.evaluate(()=>document.documentElement.clientWidth===320&&Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)<=321);}
 async function tabTo(page,locator,max=24){fact(await locator.count()===1,'KEYBOARD_DISCOVERY');for(let i=0;i<max;i++){await page.keyboard.press('Tab');if(await locator.evaluate(e=>document.activeElement===e)){fact(await locator.evaluate(e=>{const s=getComputedStyle(e),b=e.getBoundingClientRect();return e.matches(':focus-visible')&&s.outlineStyle!=='none'&&parseFloat(s.outlineWidth)>=2&&b.width>0&&b.height>0;}),'KEYBOARD_FOCUS');return;}}fact(false,'KEYBOARD_DISCOVERY');}
 function forbiddenInputs(p){return [p.id,r.a,r.b,r.o,r.shared_reference,MALFORMED_REFERENCE,r.company_name,r.company_slug].filter(v=>typeof v==='string'&&v.length>0);}
 async function secretFree(p){const cookies=await p.context.cookies(origin),secrets=cookies.filter(c=>c.name.startsWith('__Host-console_account_')).map(c=>c.value),html=await p.page.content(),snapshot=await p.page.evaluate(()=>({local:Object.entries(localStorage),session:Object.entries(sessionStorage)})),storage=JSON.stringify(snapshot);fact(secrets.every(s=>s.length>0&&!html.includes(s)&&!storage.includes(s)),'SECRET_DISCLOSURE');fact(storageSnapshotSafe(snapshot,forbiddenInputs(p)),'BUSINESS_STORAGE');return cookies;}
 async function mounted(page,phase){const root=page.locator('[data-console-react-account="mounted"]');try{await root.waitFor({state:'visible'});}catch{fact(false,'REACT_ACCOUNT_MOUNT_MISSING');}fact(await root.count()===1&&await root.isVisible(),'REACT_ACCOUNT_MOUNT_MISSING');r.react_mounts.push(phase);}
 async function document(page,p,status=200){const response=await page.goto(origin+p,{waitUntil:'load',timeout:8000});fact(response?.status()===status&&response.request().method()==='GET'&&response.request().redirectedFrom()===null&&response.url()===origin+p,'ACCOUNT_SSR');return response;}
 for(const role of successor?['A','B','O']:['A','O']){
  setStage(role.toLowerCase()+'_registration');
  const context=await browser.newContext({serviceWorkers:'block',viewport:{width:320,height:900}});observe(context,role,origin,r);
  await context.route('**/*',route=>{if(new URL(route.request().url()).origin===origin)return route.continue();r.external_requests++;return route.abort('blockedbyclient');});
  const page=await context.newPage();page.setDefaultTimeout(8000);const cdp=await context.newCDPSession(page);await configureResponseRetention(cdp);await cdp.send('WebAuthn.enable',{enableUI:false});
  const auth=await cdp.send('WebAuthn.addVirtualAuthenticator',{options:{protocol:'ctap2',transport:'internal',hasResidentKey:true,hasUserVerification:true,isUserVerified:true,automaticPresenceSimulation:true}});
  const storage=await startStorageObserver(page,cdp,role,origin);const p={context,page,cdp,auth,storage};participants.push(p);await document(page,'/');await storage.positiveControl();
  const register=page.getByRole('link',{name:'계정 만들기',exact:true});fact(await exactVisibleLink(register,'/account/register'),'UI_LINK_MISSING');await tabTo(page,register);await page.keyboard.press('Enter');await page.waitForURL(origin+'/account/register');
  for(const title of ['테스트 서비스 약관','테스트 개인정보 안내']){const box=page.getByRole('checkbox',{name:title,exact:true});fact(await box.count()===1&&!await box.isChecked(),'TERMS_CONTROL');await tabTo(page,box);await page.keyboard.press('Space');fact(await box.isChecked(),'KEYBOARD_TERMS');}
  const start=page.waitForResponse(v=>v.url()===origin+'/api/v2/auth/registration/start'&&v.request().method()==='POST'),finish=page.waitForResponse(v=>v.url()===origin+'/api/v2/auth/registration/finish'&&v.request().method()==='POST');
  await tabTo(page,page.getByRole('button',{name:'패스키로 계정 만들기',exact:true}));await page.keyboard.press('Enter');
  const sr=await start;fact(sr.status()===200,'REGISTRATION_WIRE');const creation=(await sr.json()).public_key_options?.publicKey;
  fact(creation?.authenticatorSelection?.residentKey==='required'&&creation.authenticatorSelection.requireResidentKey===true&&creation.authenticatorSelection.userVerification==='required','REGISTRATION_WIRE');
  const handle=Buffer.from(creation.user.id,'base64url');fact(handle.length===16,'REGISTRATION_WIRE');const h=handle.toString('hex');p.id=`${h.slice(0,8)}-${h.slice(8,12)}-${h.slice(12,16)}-${h.slice(16,20)}-${h.slice(20)}`;
  const fr=await finish;fact(fr.status()===201&&(await fr.json()).account?.account_id===p.id,'REGISTRATION_EFFECT');await page.waitForURL(origin+'/account');await page.locator('[data-account-state="active"]').waitFor();fact(await page.locator('[data-context-state="empty"]').count()===1,'ACCOUNT_SSR');
  const credentials=await cdp.send('WebAuthn.getCredentials',{authenticatorId:auth.authenticatorId});fact(credentials.credentials.length===1&&credentials.credentials[0].isResidentCredential===true&&credentials.credentials[0].rpId==='localhost'&&Buffer.from(credentials.credentials[0].userHandle,'base64').equals(handle),'RESIDENT');credentials.credentials.length=0;
  const cookies=await secretFree(p),access=cookies.find(c=>c.name==='__Host-console_account_session'),refresh=cookies.find(c=>c.name==='__Host-console_account_refresh');fact(!!access&&!!refresh&&access.httpOnly&&refresh.httpOnly&&access.secure&&refresh.secure&&access.path==='/'&&refresh.path==='/'&&access.sameSite==='Lax'&&refresh.sameSite==='Strict','COOKIE_SECURITY');
  r[role.toLowerCase()]=p.id;r[role.toLowerCase()+'_registration']=true;await exchange(role+'_ENROLLED',p.id);
 }
 const [a,o]=successor?[participants[0],participants[2]]:participants;const b=successor?participants[1]:null;fact(a.id!==o.id&&(!b||(b.id!==a.id&&b.id!==o.id)),'ACCOUNT_SSR');
 setStage('o_logout');let response=o.page.waitForResponse(v=>v.url()===origin+'/api/v2/auth/logout'&&v.request().method()==='POST');await o.page.getByRole('button',{name:'로그아웃',exact:true}).click();const loggedOut=await response;fact(loggedOut.status()===200&&(await loggedOut.json()).outcome==='COMMITTED','LOGOUT_EFFECT');await o.page.waitForURL(origin+'/');await exchange('O_LOGGED_OUT',o.id);
 setStage('o_login');await o.page.getByRole('link',{name:'로그인',exact:true}).click();await o.page.waitForURL(origin+'/account');const start=o.page.waitForResponse(v=>v.url()===origin+'/api/v2/auth/passkey/login/start'&&v.request().method()==='POST'),finish=o.page.waitForResponse(v=>v.url()===origin+'/api/v2/auth/passkey/login/finish'&&v.request().method()==='POST');
 await o.page.getByRole('button',{name:'패스키로 로그인',exact:true}).click();const sr=await start,login=await sr.json();fact(sr.status()===200&&login.public_key_options?.mediation==='conditional'&&login.public_key_options?.publicKey?.userVerification==='required'&&login.public_key_options.publicKey.allowCredentials.length===0,'LOGIN_WIRE');await o.page.getByRole('textbox',{name:'패스키 계정 선택',exact:true}).focus();const fr=await finish;fact(fr.status()===200&&(await fr.json()).account?.account_id===o.id,'LOGIN_EFFECT');await o.page.waitForURL(origin+'/account');await o.page.locator('[data-account-state="active"]').waitFor();await secretFree(o);r.o_logout_login=true;await exchange('O_LOGGED_IN',o.id);
 setStage('healthy_handoff_entry');await document(a.page,'/account');fact(await a.page.locator('[data-account-state="active"]').count()===1,'ACCOUNT_SSR');await document(o.page,'/account');const setup=o.page.getByRole('link',{name:'회사 업무 공간 만들기',exact:true});fact(await exactVisibleLink(setup,'/account/companies/new'),'COMPANY_ENTRY');await tabTo(o.page,setup);await o.page.keyboard.press('Enter');await o.page.waitForURL(origin+'/account/companies/new');fact(await o.page.locator('[data-company-setup]').count()===1&&await o.page.getByRole('textbox',{name:'회사 이름',exact:true}).count()===1&&await o.page.getByRole('textbox',{name:'업무 공간 식별자',exact:true}).count()===1,'COMPANY_FORM');await exchange('HEALTHY_HANDOFF_ENTRY',o.id);
 setStage('own_account_reference');const reference=a.page.getByRole('textbox',{name:'내 계정 참조',exact:true});fact(await reference.count()===1,'ACCOUNT_REFERENCE_MISSING');fact(await reference.isVisible()&&await reference.inputValue()===a.id&&await reference.evaluate(e=>e.readOnly&&!e.disabled),'ACCOUNT_REFERENCE_INVALID');await mounted(a.page,'A_ACCOUNT');await tabTo(a.page,reference);await a.page.keyboard.press(process.platform==='darwin'?'Meta+A':'Control+A');fact(await reference.evaluate(e=>e.selectionStart===0&&e.selectionEnd===e.value.length),'ACCOUNT_REFERENCE_INVALID');const sharedReference=await reference.inputValue();fact(sharedReference===a.id,'ACCOUNT_REFERENCE_INVALID');r.shared_reference=sharedReference;r.own_reference=true;await capture(a.page,'01-own-account-reference.png');
 setStage('separate_administrator');const self=o.page.getByRole('radio',{name:'내 계정',exact:true}),other=o.page.getByRole('radio',{name:'다른 계정',exact:true}),recipient=o.page.getByRole('textbox',{name:'관리할 계정 참조',exact:true});fact(await self.count()===1&&await other.count()===1,'SEPARATE_ADMIN_CONTROL_MISSING');await mounted(o.page,'O_SETUP');fact(await self.isChecked()&&!await other.isChecked(),'ADMINISTRATOR_INPUT_INVALID');r.default_self=true;await other.check();fact(await recipient.count()===1&&await recipient.isVisible(),'SEPARATE_ADMIN_CONTROL_MISSING');
 r.company_name='브라우저로 연결한 독립 관리 회사 <연구 & 본사>';r.company_slug=`handoff-${o.id.replaceAll('-','')}`;const name=o.page.getByRole('textbox',{name:'회사 이름',exact:true}),slug=o.page.getByRole('textbox',{name:'업무 공간 식별자',exact:true}),submit=o.page.getByRole('button',{name:'회사 업무 공간 만들기',exact:true});await name.fill(r.company_name);await slug.fill(r.company_slug);await recipient.fill(MALFORMED_REFERENCE);const mutations=r.mutations.length,proofs=r.csrf_requests.length;await submit.click();fact(await recipient.inputValue()===MALFORMED_REFERENCE&&(await recipient.evaluate(e=>!e.validity.valid)||await o.page.locator('#company-error[role="alert"]').isVisible())&&r.mutations.length===mutations&&r.csrf_requests.length===proofs,'ADMINISTRATOR_INPUT_INVALID');r.malformed_rejected=true;await recipient.fill(sharedReference);await capture(o.page,'02-selected-administrator.png');fact(await reflow(a.page)&&await reflow(o.page),'REFLOW_320');r.reflow_320=true;
 setStage('company_submit');response=o.page.waitForResponse(v=>v.url()===origin+'/api/v2/companies/enroll'&&v.request().method()==='POST');await tabTo(o.page,submit);await o.page.keyboard.press('Enter');const created=await response;fact(created.status()===201&&!('set-cookie' in await created.allHeaders()),'HANDOFF_RECEIPT');r.sent=created.request().postDataJSON();r.receipt=await created.json();fact(validReceipt(r.sent,r.receipt,a.id,r.company_name,r.company_slug),'HANDOFF_RECEIPT');r.command_id=r.sent.command_id;r.org_id=r.receipt.org_id;r.company_wire=true;const saved=r.receipt.result_path,company=`/companies/${r.org_id}`;await o.page.waitForURL(origin+saved);
 async function history(){const panel=o.page.locator('[data-company-outcome="committed"]');await panel.waitFor({state:'visible'});fact(await panel.count()===1&&await panel.getByRole('heading',{name:'생성 완료',exact:true,level:1}).isVisible()&&await panel.getByText(a.id,{exact:true}).isVisible()&&await o.page.locator(`a[href="${company}"]`).count()===0,'HANDOFF_DISCLOSURE');}
 await history();await mounted(o.page,'O_HISTORY');await capture(o.page,'03-operator-creation-history.png');await exchange('COMPANY_COMMITTED',o.id,{administrator_account_id:a.id,command_id:r.command_id,org_id:r.org_id,group_id:r.receipt.group_id,receipt_id:r.receipt.receipt_id});
 setStage('handoff_reopen');await document(o.page,saved);await history();await mounted(o.page,'O_HISTORY_REOPENED');r.o_history=true;const deniedCompany=await document(o.page,company,404);fact(await deniedDocumentSafe(o.page,deniedCompany,[r.company_name,r.company_slug]),'HANDOFF_DISCLOSURE');r.o_company_denied=true;await document(o.page,saved);await history();await document(a.page,'/account');await mounted(a.page,'A_ACCOUNT_REOPENED');fact(await a.page.getByRole('textbox',{name:'내 계정 참조',exact:true}).inputValue()===a.id&&await a.page.locator('a[href="/account/companies/new"]').count()===0,'HANDOFF_DISCLOSURE');const link=a.page.locator(`a[href="${company}"]`);fact(await exactVisibleLink(link,company),'HANDOFF_DISCLOSURE');await tabTo(a.page,link);await a.page.keyboard.press('Enter');await a.page.waitForURL(origin+company);fact(await a.page.getByRole('main').getByRole('heading',{name:r.company_name,exact:true}).isVisible(),'HANDOFF_DISCLOSURE');await document(a.page,company);fact(await a.page.getByRole('main').getByRole('heading',{name:r.company_name,exact:true}).isVisible(),'HANDOFF_DISCLOSURE');r.a_company_reopened=true;await capture(a.page,'04-administrator-company-reopened.png');const deniedHistory=await document(a.page,saved,404);fact(await deniedDocumentSafe(a.page,deniedHistory,[o.id]),'HANDOFF_DISCLOSURE');r.a_foreign_history_denied=true;await document(a.page,'/account/companies/new',404);fact(await a.page.locator('[data-company-setup]').count()===0,'HANDOFF_DISCLOSURE');r.a_operator_denied=true;
 if(b){await document(b.page,'/account');await mounted(b.page,'B_ACCOUNT_REOPENED');fact(await b.page.getByRole('textbox',{name:'내 계정 참조',exact:true}).inputValue()===b.id&&await b.page.locator(`a[href="${company}"]`).count()===0,'HANDOFF_DISCLOSURE');const denied=await document(b.page,company,404);fact(await deniedDocumentSafe(b.page,denied,[r.company_name,r.company_slug]),'HANDOFF_DISCLOSURE');r.b_company_denied=true;const foreign=await document(b.page,saved,404);fact(await deniedDocumentSafe(b.page,foreign,[a.id,o.id]),'HANDOFF_DISCLOSURE');r.b_foreign_history_denied=true;await document(b.page,'/account/companies/new',404);fact(await b.page.locator('[data-company-setup]').count()===0,'HANDOFF_DISCLOSURE');r.b_operator_denied=true;}
 await exchange('HANDOFF_REOPENED',o.id);for(const p of participants){await secretFree(p);r.storage.push(await p.storage.evidence(forbiddenInputs(p)));await p.storage.stop();await p.cdp.send('WebAuthn.removeVirtualAuthenticator',{authenticatorId:p.auth.authenticatorId});await p.cdp.detach();}r.input_not_stored=completeStorageEvidence(r,successor);fact(r.input_not_stored,'BUSINESS_STORAGE');
}
module.exports={nonnil,expectedDocuments,expectedMutations,validReceipt,validEvidence,observe,runHandoff,deniedProjectionSafe,storageSnapshotSafe,storageControls,storageTrace,validStorageEvidence,completeStorageEvidence};
