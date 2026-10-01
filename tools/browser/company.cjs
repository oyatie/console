'use strict';
// Real Console browser leaf. TLS relay forwards bytes to the parent's actual axum listener.
// Parent owns DB truth and must acknowledge every checkpoint. No mock routes or cookies.
const fs=require('node:fs');const path=require('node:path');const crypto=require('node:crypto');
const tls=require('node:tls');const net=require('node:net');const readline=require('node:readline');
const {execFileSync}=require('node:child_process');const {once}=require('node:events');
const {assertNativeHeader,validHeaderEvidence}=require('./native_header.cjs');
const browserPin={
 'darwin-arm64':['chrome-headless-shell-mac-arm64/chrome-headless-shell','a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
 'linux-x64':['chrome-headless-shell-linux64/chrome-headless-shell','ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e']
}[process.platform+'-'+process.arch];
if(!browserPin)throw Error('UNSUPPORTED_REVIEWED_BROWSER_PLATFORM');
const EXECUTABLE=path.join(__dirname,'runtime','browser',browserPin[0]);
const EXECUTABLE_SHA=browserPin[1];
const PLAYWRIGHT=path.join(__dirname,'runtime','node_modules','playwright');
const TERMS=[{kind:'test.account.service',title:'테스트 서비스 약관'},{kind:'test.account.privacy',title:'테스트 개인정보 안내'}];
const MUTATION_PATHS=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish','/api/v2/companies/enroll'];
const UUID=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
function nonnil(value){return typeof value==='string'&&UUID.test(value)&&value!=='00000000-0000-0000-0000-000000000000';}
function documentPaths(result){if(!nonnil(result.command_id)||!nonnil(result.org_id))return null;const saved=`/account/companies/requests/${result.command_id}`;return ['/','/account/register','/account','/account','/account/companies/new',saved,saved,`/companies/${result.org_id}`,...(result.policy_entry===true?[]:[`/companies/${result.org_id}/policy`,`/companies/${result.org_id}/policy`,`/companies/${result.org_id}`]),saved,...(result.policy_entry===true?['/account',`/companies/${result.org_id}`,`/companies/${result.org_id}/policy/payroll-read/install`]:[])];}
function completeDocumentsOriginal(result){const expected=documentPaths(result);return !!expected&&result.document_failures===0&&Array.isArray(result.documents)&&result.documents.length===expected.length&&result.documents.every((row,index)=>row&&Object.keys(row).sort().join(',')==='method,ordinal,path,redirected,status,url_exact'&&row.ordinal===index+1&&row.method==='GET'&&row.path===expected[index]&&row.redirected===false&&row.status===200&&row.url_exact===true);}
function expectedDocumentRows(result){const paths=documentPaths(result);return paths&&paths.map(path=>({method:'GET',path,status:200,redirected:false})).concat(result.policy_documents||[],result.people_documents||[]);}
function completeDocuments(result){if(!result.policy_entry)return completeDocumentsOriginal(result);const expected=expectedDocumentRows(result);return !!expected&&result.document_failures===0&&result.documents?.length===expected.length&&result.documents.every((row,i)=>Object.keys(row).sort().join(',')==='method,ordinal,path,redirected,status,url_exact'&&row.ordinal===i+1&&row.url_exact===true&&['method','path','status','redirected'].every(k=>row[k]===expected[i][k]));}
function observeDocuments(page,origin,result){
 const owners=new Map();
 function isDocument(request){return request.frame()===page.mainFrame()&&(request.resourceType()==='document'||request.isNavigationRequest());}
 page.on('request',request=>{try{
  if(!isDocument(request))return;
  const url=new URL(request.url());
  const allowed=['/','/account/register','/account','/account/companies/new'].includes(url.pathname)||/^\/account\/companies\/requests\/[0-9a-f-]{36}$/.test(url.pathname)||/^\/companies\/[0-9a-f-]{36}$/.test(url.pathname);
  const policyRoot=result.policy_entry!==true&&url.pathname===`/companies/${result.org_id}/policy`;
  const policyEntry=result.policy_entry===true&&url.pathname===`/companies/${result.org_id}/policy/payroll-read/install`;
  const policyStep=result.policy_entry===true&&result.policy_documents?.some(d=>d.path===url.pathname);
  const fullPath=url.pathname+url.search;
  const peopleStep=result.people_entry===true&&result.people_documents?.some(d=>d.path===fullPath);
  const canonical=(allowed||policyRoot||policyEntry||policyStep||peopleStep)&&url.origin===origin&&(!url.search||peopleStep)&&!url.hash&&!url.username&&!url.password;
  const row={ordinal:result.documents.length+1,method:request.method(),path:canonical?fullPath:'<unexpected>',redirected:request.redirectedFrom()!==null,status:null,url_exact:false};
  if(owners.has(request)||request.resourceType()!=='document'||!request.isNavigationRequest())result.document_failures++;
  owners.set(request,row);result.documents.push(row);
 }catch{result.document_failures++;}});
 page.on('response',response=>{try{const request=response.request();if(!isDocument(request))return;const row=owners.get(request);if(!row||row.status!==null){result.document_failures++;return;}row.status=response.status();row.url_exact=response.url()===origin+row.path;row.redirected ||= request.redirectedFrom()!==null;const expected=result.policy_documents?expectedDocumentRows(result)?.[row.ordinal-1]:{method:'GET',status:200,redirected:false};if(!expected||row.status!==expected.status||row.method!==expected.method||row.redirected!==expected.redirected||!row.url_exact)result.document_failures++;}catch{result.document_failures++;}});
 page.on('requestfailed',request=>{try{if(isDocument(request))result.document_failures++;}catch{result.document_failures++;}});
}
// Observe every HTTP mutation request reported by this BrowserContext, including
// its pages/popups/workers. Safe GET/HEAD/OPTIONS are outside this mutation census.
// Service workers are blocked. This is not operating-system or WebSocket egress proof.
function observeMutations(context,origin,result){
 const seen=new WeakSet();
 context.on('request',request=>{try{
  const method=request.method();if(['GET','HEAD','OPTIONS'].includes(method))return;
  if(seen.has(request)){result.unexpected_mutations++;return;}seen.add(request);
  const url=new URL(request.url());
  const canonical=url.origin===origin&&!url.search&&!url.hash&&!url.username&&!url.password&&(MUTATION_PATHS.includes(url.pathname)||(result.policy_entry===true&&result.policy_expected_mutations?.includes(url.pathname))||(result.people_entry===true&&result.people_expected_mutations?.includes(url.pathname)));
  const allowed=method==='POST'&&canonical;
  result.mutations.push({ordinal:result.mutations.length+1,method:method==='POST'?'POST':'<unexpected>',path:allowed?url.pathname:'<unexpected>'});
  if(allowed)result.posts[url.pathname]=(result.posts[url.pathname]||0)+1;else result.unexpected_mutations++;
 }catch{result.unexpected_mutations++;}});
}
function completeMutationsOriginal(result){return result.unexpected_mutations===0&&Array.isArray(result.mutations)&&result.mutations.length===MUTATION_PATHS.length&&result.mutations.every((row,index)=>row&&Object.keys(row).sort().join(',')==='method,ordinal,path'&&row.ordinal===index+1&&row.method==='POST'&&row.path===MUTATION_PATHS[index])&&MUTATION_PATHS.every(path=>result.posts?.[path]===1);}
function completeMutations(result){if(!result.policy_entry)return completeMutationsOriginal(result);const expected=MUTATION_PATHS.concat(result.policy_expected_mutations||[],result.people_expected_mutations||[]);const count=result.people_entry===true?13:6;return expected.length===count&&result.unexpected_mutations===0&&result.mutations?.length===count&&result.mutations.every((r,i)=>Object.keys(r).sort().join(',')==='method,ordinal,path'&&r.ordinal===i+1&&r.method==='POST'&&r.path===expected[i])&&expected.every(p=>result.posts[p]===1);}
function scrub(env){return Object.fromEntries(Object.entries(env).filter(([k])=>!['DEBUG','PWDEBUG','NODE_DEBUG'].includes(k)));}
function requireFact(value,code){if(value!==true){const e=new Error(code);e.code=code;throw e;}}
function delayLimit(promise,ms){let timer;return Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>{const e=new Error('TIMEOUT');e.code='TIMEOUT';reject(e);},ms);})]).finally(()=>clearTimeout(timer));}
function validOrigin(origin){try{const u=new URL(origin);return u.protocol==='https:'&&u.hostname==='localhost'&&/^\d+$/.test(u.port)&&Number(u.port)>0&&Number(u.port)<=65535&&u.pathname==='/'&&!u.search&&!u.hash&&!u.username&&!u.password;}catch{return false;}}
function validCheckpointCommand(value,phase){return value?.kind==='CONTINUE'&&value.phase===phase&&Object.keys(value).sort().join(',')==='kind,phase';}
function completePolicyPlan(r){try{
 const b=`/companies/${r.org_id}/policy/payroll-read`,payroll=`/companies/${r.org_id}/payroll`;
 const commands=r.policy.mutations.map(m=>m.command_id),assignment=r.policy.checkpoints[1].assignment_id;
 const mutationPaths=[b+'/catalog',b+'/grants',b+'/grants/'+assignment+'/revoke'];
 const install=b+'/requests/install/'+commands[0],grant=b+'/requests/grant/'+commands[1],revoke=b+'/requests/revoke/'+commands[2];
 const get=path=>({method:'GET',path,status:200,redirected:false});
 const post=path=>({method:'POST',path,status:303,redirected:false});
 const redirected=path=>({...get(path),redirected:true});
 const expected=[post(mutationPaths[0]),redirected(install),get(b+'/grant'),post(mutationPaths[1]),redirected(grant),get(grant),get(`/companies/${r.org_id}`),get(payroll),get(payroll),get(grant),get(b+'/revoke'),post(mutationPaths[2]),redirected(revoke),get(revoke),{...get(payroll),status:404},get('/account'),get(`/companies/${r.org_id}`)];
 return JSON.stringify(r.policy_documents)===JSON.stringify(expected)&&JSON.stringify(r.policy_expected_mutations)===JSON.stringify(mutationPaths);
 }catch{return false;}}
async function exactVisibleLink(link,href){return await link.count()===1&&await link.isVisible()&&await link.getAttribute('href')===href;}
function validCompanyLayout(value,expectedCount,viewport){
 if(!Number.isInteger(expectedCount)||expectedCount<1||![320,1440].includes(viewport)||!value||value.workspace!==true||value.stylesheet!==true||value.display!==(viewport===320?'block':'grid')||value.viewport!==viewport||!Number.isFinite(value.scrollWidth)||value.scrollWidth>viewport+1||value.scrollWidth<viewport||!Array.isArray(value.rects)||value.rects.length!==expectedCount)return false;
 const rects=value.rects;
 if(!rects.every(r=>r&&['x','y','width','height'].every(k=>Number.isFinite(r[k]))&&r.x>=0&&r.y>=0&&r.width>=44&&r.height>=44&&r.x+r.width<=viewport+1))return false;
 for(let i=0;i<rects.length;i++)for(let j=i+1;j<rects.length;j++){
  const a=rects[i],b=rects[j];
  if(Math.max(a.x-b.x-b.width,b.x-a.x-a.width,a.y-b.y-b.height,b.y-a.y-a.height)<8)return false;
 }
 return true;
}
function completeCompanyWorkspace(r){return ['company_layout_320','company_layout_desktop'].every(key=>r[key]===true)&&(r.policy_entry!==true||['payroll_company_layout_320','payroll_company_layout_desktop'].every(key=>r[key]===true));}
function completeNavigation(r){return ['payroll_nav_before_grant_absent','payroll_receipt_link','payroll_workspace_link','keyboard_payroll_navigation','reflow_payroll_workspace_320','payroll_nav_after_revoke_absent'].every(key=>r[key]===true);}
function expectedNativeHeaders(r){
 if(!validOrigin(r.header_origin)||!nonnil(r.org_id))throw Error('NATIVE_HEADER_IDENTITY');
 const w=`/companies/${r.org_id}`,p=w+'/policy/payroll-read',payroll=w+'/payroll';
 const initial=(phase,path=w)=>({phase,url:r.header_origin+path,paths:['/account',w,w+'/policy'],currentPath:path,boundTitle:true,deniedPrefixes:[w+'/people',payroll]});
 const first=initial('COMPANY_HEADER_FIRST');
 if(r.policy_entry!==true)return [first,initial('POLICY_ROOT_OPENED',w+'/policy'),initial('POLICY_ROOT_REOPENED',w+'/policy'),initial('COMPANY_POLICY_RETURNED')];
 const commands=r.policy?.mutations?.map(m=>m.command_id)||[];
 const policy=(phase,path,active=false)=>({phase,url:r.header_origin+path,paths:['/account',w,w+'/policy',...(active?[payroll]:[])],payrollPath:active?payroll:undefined,boundTitle:true});
 const read=phase=>({phase,url:r.header_origin+payroll,paths:['/account',payroll],payrollPath:payroll,currentPath:payroll,boundTitle:true});
 return [first,policy('POLICY_HEADER_PREFLIGHT',p+'/install'),
  policy('CATALOG_INSTALLED',p+'/requests/install/'+commands[0]),
  policy('GRANT_COMMITTED',p+'/requests/grant/'+commands[1],true),
  policy('GRANT_REOPENED',p+'/requests/grant/'+commands[1],true),
  {...policy('PAYROLL_COMPANY_HEADER',w,true),currentPath:w},
  read('PAYROLL_READ'),read('PAYROLL_REOPENED'),
  policy('REVOKE_COMMITTED',p+'/requests/revoke/'+commands[2]),
  policy('REVOKE_REOPENED',p+'/requests/revoke/'+commands[2]),
  initial('REVOKED_COMPANY_HEADER')];
}
function completeNativeHeaders(r){try{
 const expected=expectedNativeHeaders(r);
 return Array.isArray(r.native_headers)&&r.native_headers.length===expected.length&&r.native_headers.every((row,i)=>validHeaderEvidence(row,expected[i]))&&
  (r.people_entry!==true||r.people?.header_origin===r.header_origin);
}catch{return false;}}

function completeObservations(r){return completeNativeHeaders(r)&&(r.policy_entry===true||r.policy_root_journey===true)&&(r.people_entry!==true||(require('./people_journey.cjs').validEvidence(r.people)&&JSON.stringify(r.people_documents)===JSON.stringify(require('./people_journey.cjs').expectedDocuments(r.people))))&&completeCompanyWorkspace(r)&&completeDocuments(r)&&completeMutations(r)&&r.browser_version==='153.0.8010.12'&&r.unexpected_mutations===0&&r.relay_failure!==true&&r.tls_client_error!==true&&r.root_status===200&&r.registration_wire===true&&r.resident===true&&r.cookie_security===true&&r.literal_secret_absent===true&&r.external_requests===0&&r.checkpoints?.join(',')===(r.policy_entry===true?'ENROLLED,COMPANY_COMMITTED,COMPANY_REOPENED,POLICY_ENTRY_READY,POLICY_PREFLIGHT,CATALOG_INSTALLED,GRANT_COMMITTED,GRANT_REOPENED,PAYROLL_READ,PAYROLL_REOPENED,PAYROLL_JSON,REVOKE_COMMITTED,REVOKE_REOPENED,PAYROLL_DENIED,PAYROLL_JSON_DENIED,ADMIN_REOPENED':'ENROLLED,COMPANY_COMMITTED,COMPANY_REOPENED')&&(r.policy_entry!==true||(require('./policy_journey.cjs').validEvidence(r.policy)&&require('./policy_journey.cjs').validTaskEvidence(r.policy)&&completePolicyPlan(r)&&completeNavigation(r)&&['current_context','policy_entry_link','policy_preflight','reflow_policy_320','keyboard_policy_entry','keyboard_policy_submit','full_policy_journey','payroll_ssr','payroll_reopened','payroll_json','payroll_denied','payroll_json_denied','administration_reopened'].every(key=>r[key]===true)))&&MUTATION_PATHS.every(p=>r.posts?.[p]===1)&&['reflow_root_320','reflow_register_320','reflow_account_320','reflow_setup_320','reflow_result_320','reflow_company_320','keyboard_skip','keyboard_registration','keyboard_terms','keyboard_registration_submit','keyboard_company_entry','keyboard_company_submit','company_wire','company_result','company_reopen','company_workspace','company_back','invalid_input_preserved','recipient_consequence','business_input_not_stored'].every(key=>r[key]===true);}
function leafStatus(r){return !r.failure&&completeObservations(r)&&r.cleanup?.confirmed===true?'BROWSER_LEAF_PASSED':'BROWSER_LEAF_FAILED';}
function publicError(error){const allowed=new Set(['TIMEOUT','OWNER_PROTOCOL','OWNER_EOF','OWNER_REFUSED','PREREQUISITE','TLS_RELAY_FAILED','UI_PUBLIC_ENTRY_MISSING','UI_LINK_MISSING','TERMS_CONTROL','REGISTRATION_WIRE','REGISTRATION_EFFECT','RESIDENT','COOKIE_SECURITY','SECRET_DISCLOSURE','ACCOUNT_SSR','LOGOUT_EFFECT','LOGIN_WIRE','LOGIN_EFFECT','EXTERNAL_REQUEST','OBSERVATION_INCOMPLETE','REFLOW_320','KEYBOARD_FOCUS','KEYBOARD_DISCOVERY','KEYBOARD_SKIP','KEYBOARD_TERMS','COMPANY_ENTRY','COMPANY_FORM','COMPANY_WIRE','COMPANY_RESULT','COMPANY_WORKSPACE','COMPANY_REOPEN','INPUT_PRESERVATION','BUSINESS_STORAGE','CURRENT_CONTEXT','POLICY_ENTRY','POLICY_PREFLIGHT','NATIVE_HEADER','POLICY_ROOT','PEOPLE_ENTRY','PEOPLE_SEARCH_FORM_MISSING','POLICY_TASK_COMPREHENSION']);if(allowed.has(error?.code))return error.code;const network=String(error?.message??'').match(/\bnet::(ERR_[A-Z0-9_]{1,76})\b/);if(network)return network[1];if(error?.name==='TimeoutError')return 'BROWSER_TIMEOUT';if(error?.name==='SyntaxError')return 'INVALID_JSON';if(error?.name==='TypeError')return 'BROWSER_TYPE_ERROR';if(/No (?:resource with given identifier found|data found for resource with given identifier)/.test(String(error?.message??'')))return 'RESPONSE_BODY_UNAVAILABLE';if(String(error?.message??'').includes('Execution context was destroyed'))return 'BROWSER_CONTEXT_DESTROYED';return 'UNCLASSIFIED_FAILURE';}

// Same observer and teardown are used by the real driver and machinery-only controls.
function watchOwner(reader,cancel){
 reader.on('line',line=>{try{if(JSON.parse(line)?.kind==='ABORT')cancel('OWNER_REFUSED');}catch{cancel('OWNER_PROTOCOL');}});
 reader.on('close',()=>cancel('OWNER_EOF'));
}
async function closeOwnedBrowser(launchPromise){
 if(!launchPromise)return {confirmed:true,pid:null};
 let server,child;
 try{server=await delayLimit(launchPromise,11000);child=server.process();}catch{return {confirmed:false,pid:null};}
 if(!child)return {confirmed:false,pid:null};
 const exited=()=>child.exitCode!==null||child.signalCode!==null;
 let closeResolved=false;
 try{await delayLimit(server.close(),5000);closeResolved=true;}catch{}
 if(closeResolved&&!exited()){try{await delayLimit(once(child,'exit'),3000);}catch{}}
 // A resolved close is not exit evidence. Persist with the exact owned kill.
 if(!exited()){try{await delayLimit(server.kill(),5000);}catch{}}
 if(!exited()){try{await delayLimit(once(child,'exit'),3000);}catch{}}
 return {confirmed:exited(),pid:child.pid};
}

// Native browser memory retention keeps original responses available across navigation.
//1MiB total is configured; per-resource bound is requested, not independently certified.
function configureResponseRetention(cdp){return cdp.send('Network.configureDurableMessages',{maxTotalBufferSize:1048576,maxResourceBufferSize:262144});}

function certificateArgs(files){return ['req','-new','-x509','-newkey','ec','-pkeyopt','ec_paramgen_curve:P-256','-pkeyopt','ec_param_enc:named_curve','-nodes','-keyout',files[0],'-out',files[1],'-days','1','-config',files[2]];}

function installFormControlsSafe(f){return f.enctype==='application/x-www-form-urlencoded'&&f.target===''&&[...f.elements].filter(e=>e.tagName!=='FIELDSET').length===4&&[...f.elements].filter(e=>e.tagName==='BUTTON'&&e.type==='submit'&&!e.name&&!e.matches(':disabled')&&!e.hasAttribute('formaction')&&!e.hasAttribute('formmethod')&&!e.hasAttribute('formenctype')&&!e.hasAttribute('formtarget')).length===1&&[...f.ownerDocument.querySelectorAll('input[type="image" i]')].every(e=>e.form!==f);}

async function main(backendPort,out,mode){
 requireFact(mode===undefined||mode==='policy-entry'||mode==='people-entry','PREREQUISITE');
 requireFact(/^\d+$/.test(backendPort)&&Number(backendPort)>0&&Number(backendPort)<=65535&&path.isAbsolute(out),'PREREQUISITE');
 fs.mkdirSync(out,{mode:0o700});
 const result={kind:mode==='people-entry'?'REAL_NATIVE_PEOPLE_UI_BROWSER_LEAF':mode==='policy-entry'?'REAL_NATIVE_COMPANY_POLICY_ENTRY_BROWSER_LEAF':'REAL_NATIVE_COMPANY_UI_BROWSER_LEAF',people_entry:mode==='people-entry',policy_entry:mode==='policy-entry'||mode==='people-entry',node_version:process.version,checkpoints:[],screenshots:[],documents:[],document_failures:0,posts:Object.fromEntries(MUTATION_PATHS.map(p=>[p,0])),external_requests:0,unexpected_mutations:0,mutations:[],cleanup:{confirmed:false},limits:['Chromium153 virtual resident authenticator and automatic presence only; no human or physical-device picker proof.','Ephemeral-SPKI localhost TLS termination and unencrypted owned loopback upstream are test topology, not production TLS/HA certification.','Mutation census observes BrowserContext HTTP request events for pages/popups/workers; service workers are blocked. GET/HEAD/OPTIONS, WebSocket messages and operating-system egress are not certified.','Three independent DB acknowledgements plus actual parent-owned designation are required; parent remains final acceptance owner. Full policy mode proves install, grant, payroll access and revoke; dropped-response recovery remains a separate acceptance leaf.','Keyboard discovery and320px geometric reflow are observed on mounted pages; this is not human usability, screen-reader, contrast or complete WCAG certification.']};
 let relay,server,browserProcess,reader,finalized,launchPromise;let finalizing=false;let cancelled=false;const sockets=new Set();
 const files=['fixture.key','fixture.crt','fixture.cnf'].map(n=>path.join(out,n));
 const emit=value=>process.stdout.write(JSON.stringify(value)+'\n');
 let cleanupPromise;
 async function cleanup(){if(cleanupPromise)return cleanupPromise;cleanupPromise=(async()=>{
   let relayClosed=true;
   const closed=await closeOwnedBrowser(launchPromise);const browserClosed=closed.confirmed;
   if(closed.pid!==null)result.browser_pid=closed.pid;
   for(const s of sockets)s.destroy();
   if(relay){try{await delayLimit(new Promise(resolve=>relay.close(resolve)),2000);}catch{relayClosed=false;}}
   for(const file of files){try{fs.unlinkSync(file);}catch(e){if(e.code!=='ENOENT')relayClosed=false;}}
   reader?.close();result.cleanup={confirmed:browserClosed&&relayClosed,browser_process_exited:browserClosed,relay_closed:relayClosed};
 })();return cleanupPromise;}
 async function finish(){if(finalized)return finalized;finalizing=true;finalized=(async()=>{await cleanup();result.status=leafStatus(result);fs.writeFileSync(path.join(out,'result.json'),JSON.stringify(result,null,2)+'\n',{flag:'wx',mode:0o600});emit({kind:'RESULT',status:result.status,result_path:path.join(out,'result.json')});process.exitCode=result.status==='BROWSER_LEAF_PASSED'?0:2;})();return finalized;}
 function requestCancel(code){if(finalizing||cancelled)return;cancelled=true;result.failure={stage:'owner_cancel',code};void cleanup().catch(()=>{result.cleanup={confirmed:false};});}
 const watchdog=setTimeout(async()=>{requestCancel('TIMEOUT');await finish();process.exit(2);},mode==='people-entry'?300000:mode==='policy-entry'?180000:90000);
 let stage='prerequisites';
 try{
  const env=scrub(process.env);delete process.env.DEBUG;delete process.env.PWDEBUG;delete process.env.NODE_DEBUG;
  requireFact(crypto.createHash('sha256').update(fs.readFileSync(EXECUTABLE)).digest('hex')===EXECUTABLE_SHA,'PREREQUISITE');
  requireFact(JSON.parse(fs.readFileSync(path.join(PLAYWRIGHT,'package.json'),'utf8')).version==='1.63.0','PREREQUISITE');
  const {chromium}=require(PLAYWRIGHT);
  fs.writeFileSync(files[2],'[req]\nprompt=no\ndistinguished_name=dn\nx509_extensions=ext\n[dn]\nCN=localhost\n[ext]\nsubjectAltName=DNS:localhost\n',{mode:0o600,flag:'wx'});
  execFileSync('/usr/bin/openssl',certificateArgs(files),{stdio:'ignore',env,timeout:10000});
  const key=fs.readFileSync(files[0]);const cert=fs.readFileSync(files[1]);
  const spki=crypto.createHash('sha256').update(new crypto.X509Certificate(cert).publicKey.export({type:'spki',format:'der'})).digest('base64');
  relay=tls.createServer({key,cert,ALPNProtocols:['http/1.1']},downstream=>{
   if(!['127.0.0.1','::ffff:127.0.0.1','::1'].includes(downstream.remoteAddress)){downstream.destroy();return;}
   sockets.add(downstream);downstream.on('close',()=>sockets.delete(downstream));
   const upstream=net.createConnection({host:'127.0.0.1',port:Number(backendPort)});sockets.add(upstream);upstream.on('close',()=>sockets.delete(upstream));
   downstream.on('error',()=>upstream.destroy());upstream.on('error',()=>{result.relay_failure=true;downstream.destroy();});
   downstream.on('close',()=>upstream.destroy());upstream.on('close',()=>downstream.destroy());
   downstream.pipe(upstream);upstream.pipe(downstream);
  });
  relay.on('tlsClientError',error=>{result.tls_client_error=true;result.tls_client_error_code=/^[A-Z0-9_]{1,80}$/.test(error?.code??'')?error.code:'UNKNOWN';});
  await new Promise((resolve,reject)=>{relay.once('error',reject);relay.listen(0,'127.0.0.1',resolve);});
  const origin=`https://localhost:${relay.address().port}`;requireFact(validOrigin(origin),'PREREQUISITE');
  result.header_origin=origin;result.native_headers=[];
  for(const file of files)fs.unlinkSync(file); // Private test key stays memory-only after TLS initialization.
  reader=readline.createInterface({input:process.stdin,crlfDelay:Infinity});const input=reader[Symbol.asyncIterator]();
  watchOwner(reader,requestCancel);
  async function receive(){const line=await delayLimit(input.next(),20000);requireFact(!line.done,'OWNER_EOF');let value;try{value=JSON.parse(line.value);}catch{requireFact(false,'OWNER_PROTOCOL');}requireFact(value?.kind!=='ABORT','OWNER_REFUSED');return value;}
  emit({kind:'READY',origin,rp_id:'localhost',tls_spki_sha256:spki,upstream_port:Number(backendPort)});
  stage='owner_start';const start=await receive();requireFact(start.kind==='START'&&Object.keys(start).length===1,'OWNER_PROTOCOL');
  stage='browser_launch';requireFact(!cancelled,'OWNER_REFUSED');launchPromise=chromium.launchServer({headless:true,executablePath:EXECUTABLE,timeout:10000,env,args:['--disable-background-networking','--disable-component-update','--no-proxy-server',`--ignore-certificate-errors-spki-list=${spki}`]});server=await launchPromise;
  browserProcess=server.process();result.browser_pid=browserProcess.pid;emit({kind:'BROWSER_OWNED',pid:browserProcess.pid,executable_sha256:EXECUTABLE_SHA});requireFact(!cancelled,'OWNER_REFUSED');
  const browser=await chromium.connect(server.wsEndpoint(),{timeout:10000});result.browser_version=browser.version();requireFact(result.browser_version==='153.0.8010.12','PREREQUISITE');
  const context=await browser.newContext({serviceWorkers:'block',viewport:{width:320,height:900}});
  await context.route('**/*',route=>{if(new URL(route.request().url()).origin===origin)return route.continue();result.external_requests++;return route.abort('blockedbyclient');});
  observeMutations(context,origin,result);
  const page=await context.newPage();page.setDefaultTimeout(8000);observeDocuments(page,origin,result);
  const cdp=await context.newCDPSession(page);await configureResponseRetention(cdp);await cdp.send('WebAuthn.enable',{enableUI:false});
  const auth=await cdp.send('WebAuthn.addVirtualAuthenticator',{options:{protocol:'ctap2',transport:'internal',hasResidentKey:true,hasUserVerification:true,isUserVerified:true,automaticPresenceSimulation:true}});
  const api=(method,p)=>page.waitForResponse(r=>r.url()===origin+p&&r.request().method()===method,{timeout:10000}).catch(()=>null);
  async function reflow320(){return page.evaluate(()=>document.documentElement.clientWidth===320&&Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)<=321);}
  async function companyLayout(viewport,withPayroll){
   await page.setViewportSize({width:viewport,height:900});
   await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
   const company=`/companies/${result.org_id}`;
   const expected=[['권한 관리',company+'/policy']];
   if(result.policy_entry)expected.push(['급여 목록 열람 권한',company+'/policy/payroll-read/install']);
   if(withPayroll)expected.unshift(['급여',company+'/payroll']);
   const rects=[];
   for(const [name,href] of expected){
    const link=page.getByRole('main').getByRole('link',{name,exact:true});
    if(!await exactVisibleLink(link,href))return false;
    rects.push(await link.boundingBox());
   }
   const value=await page.evaluate(()=>({workspace:document.body.classList.contains('workspace')&&document.body.classList.contains('company-workspace'),stylesheet:[...document.styleSheets].some(sheet=>sheet.href===location.origin+'/assets/workspace.css'&&sheet.cssRules.length>0),display:getComputedStyle(document.body).display,viewport:document.documentElement.clientWidth,scrollWidth:Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)}));
   value.rects=rects;
   result.company_layout_observations??=[];result.company_layout_observations.push({withPayroll,...value});
   return validCompanyLayout(value,expected.length,viewport);
  }
  async function focusVisible(locator){return locator.evaluate(element=>{
   const rect=element.getBoundingClientRect(),style=getComputedStyle(element);
   return document.activeElement===element&&element.matches(':focus-visible')&&rect.width>0&&rect.height>0&&rect.left>=0&&rect.right<=innerWidth+1&&rect.top>=0&&rect.bottom<=innerHeight+1&&style.outlineStyle!=='none'&&parseFloat(style.outlineWidth)>=2&&style.outlineColor!=='transparent'&&!/rgba\([^)]*,\s*0\)/.test(style.outlineColor);
  });}
  async function tabTo(locator,maxTabs=12){
   requireFact(await locator.count()===1,'KEYBOARD_DISCOVERY');
   for(let count=0;count<maxTabs;count++){
    await page.keyboard.press('Tab');
    if(await locator.evaluate(element=>document.activeElement===element)){requireFact(await focusVisible(locator),'KEYBOARD_FOCUS');return;}
   }
   requireFact(false,'KEYBOARD_DISCOVERY');
  }
  async function nativeHeader(phase){
   const expected=expectedNativeHeaders(result).filter(row=>row.phase===phase);requireFact(expected.length===1,'NATIVE_HEADER');
   try{result.native_headers.push(await assertNativeHeader(page,tabTo,expected[0]));}
   catch(error){result.header_failure_phase=phase;error.code='NATIVE_HEADER';throw error;}
  }
  async function checkpoint(phase,account){emit({kind:'CHECKPOINT',phase,account_id:account});const command=await receive();requireFact(validCheckpointCommand(command,phase),'OWNER_PROTOCOL');result.checkpoints.push(phase);}
  async function secretFree(){const cookies=await context.cookies(origin);const secrets=cookies.filter(c=>c.name.startsWith('__Host-console_account_')).map(c=>c.value);const html=await page.content();const storage=await page.evaluate(()=>JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}));requireFact(secrets.every(s=>s.length>0&&!html.includes(s)&&!storage.includes(s)),'SECRET_DISCLOSURE');result.literal_secret_absent=true;return cookies;}
  async function capture(name){await page.screenshot({path:path.join(out,name),fullPage:true});result.screenshots.push(name);}
  stage='public_entry';const publicResponse=await page.goto(origin+'/',{waitUntil:'domcontentloaded',timeout:8000});result.root_status=publicResponse?.status();requireFact(result.root_status===200,'UI_PUBLIC_ENTRY_MISSING');result.reflow_root_320=await reflow320();requireFact(result.reflow_root_320,'REFLOW_320');await capture('01-public.png');
  requireFact(await page.getByRole('link',{name:'로그인',exact:true}).count()===1,'UI_LINK_MISSING');
  stage='keyboard_public_entry';const skip=page.getByRole('link',{name:'본문 바로가기',exact:true});await page.keyboard.press('Tab');requireFact(await skip.count()===1&&await focusVisible(skip),'KEYBOARD_FOCUS');await page.keyboard.press('Enter');
  result.keyboard_skip=await page.locator('#main-content').evaluate(element=>document.activeElement===element);requireFact(result.keyboard_skip,'KEYBOARD_SKIP');
  const registerLink=page.getByRole('link',{name:'계정 만들기',exact:true});await tabTo(registerLink,6);await page.keyboard.press('Enter');await page.waitForURL(origin+'/account/register');result.keyboard_registration=true;
  result.reflow_register_320=await reflow320();requireFact(result.reflow_register_320,'REFLOW_320');
  stage='keyboard_terms';for(const item of TERMS){const box=page.getByRole('checkbox',{name:item.title,exact:true});requireFact(await box.count()===1&&!(await box.isChecked()),'TERMS_CONTROL');await tabTo(box);await page.keyboard.press('Space');requireFact(await box.isChecked(),'KEYBOARD_TERMS');await page.keyboard.press('Space');requireFact(!(await box.isChecked()),'KEYBOARD_TERMS');await page.keyboard.press('Space');requireFact(await box.isChecked(),'KEYBOARD_TERMS');}result.keyboard_terms=true;
  await capture('02-terms.png');
  stage='keyboard_registration_submit';await tabTo(page.getByRole('button',{name:'패스키로 계정 만들기',exact:true}),8);
  stage='registration';const registrationStart=api('POST',MUTATION_PATHS[0]);const registrationFinish=api('POST',MUTATION_PATHS[1]);
  await page.keyboard.press('Enter');result.keyboard_registration_submit=true;
  stage='registration_start_response';const startedResponse=await registrationStart;requireFact(startedResponse?.status()===200,'REGISTRATION_WIRE');stage='registration_start_body';const started=await startedResponse.json();
  stage='registration_wire';const creation=started.public_key_options?.publicKey;result.registration_wire=creation?.authenticatorSelection?.residentKey==='required'&&creation?.authenticatorSelection?.requireResidentKey===true&&creation?.authenticatorSelection?.userVerification==='required'&&typeof creation?.user?.id==='string';requireFact(result.registration_wire,'REGISTRATION_WIRE');
  const handle=Buffer.from(creation.user.id,'base64url');requireFact(handle.length===16,'REGISTRATION_WIRE');const hex=handle.toString('hex');const account=`${hex.slice(0,8)}-${hex.slice(8,12)}-${hex.slice(12,16)}-${hex.slice(16,20)}-${hex.slice(20)}`;
  stage='registration_finish_response';const finishedResponse=await registrationFinish;requireFact(finishedResponse?.status()===201,'REGISTRATION_EFFECT');stage='registration_finish_body';const finished=await finishedResponse.json();stage='registration_finish_identity';requireFact(finished.account?.account_id===account,'REGISTRATION_EFFECT');
  stage='registration_account_navigation';await page.waitForURL(origin+'/account');stage='registration_account_active';await page.locator('[data-account-state="active"]').waitFor();stage='registration_account_state';requireFact(await page.locator('[data-account-state="active"]').count()===1&&await page.locator('[data-context-state="empty"]').count()===1,'ACCOUNT_SSR');
  result.reflow_account_320=await reflow320();requireFact(result.reflow_account_320,'REFLOW_320');
  stage='registration_credentials_read';const credentials=await cdp.send('WebAuthn.getCredentials',{authenticatorId:auth.authenticatorId});stage='registration_credentials_verify';result.resident=credentials.credentials.length===1&&credentials.credentials[0].isResidentCredential===true&&credentials.credentials[0].rpId==='localhost'&&Buffer.from(credentials.credentials[0].userHandle,'base64').equals(handle);credentials.credentials.length=0;requireFact(result.resident,'RESIDENT');
  stage='registration_secret_check';const cookies=await secretFree();const access=cookies.find(c=>c.name==='__Host-console_account_session');const refresh=cookies.find(c=>c.name==='__Host-console_account_refresh');result.cookie_security=!!access&&!!refresh&&access.httpOnly&&refresh.httpOnly&&access.secure&&refresh.secure&&access.path==='/'&&refresh.path==='/'&&access.sameSite==='Lax'&&refresh.sameSite==='Strict';requireFact(result.cookie_security,'COOKIE_SECURITY');
  stage='registration_checkpoint';await capture('03-account-enrolled.png');
  emit({kind:'CHECKPOINT',phase:'ENROLLED',account_id:account});
  requireFact(validCheckpointCommand(await receive(),'DESIGNATED'),'OWNER_PROTOCOL');result.checkpoints.push('ENROLLED');
  stage='company_entry';await page.reload({waitUntil:'domcontentloaded'});
  const setup=page.getByRole('link',{name:'회사 업무 공간 만들기',exact:true});await tabTo(setup,16);await page.keyboard.press('Enter');await page.waitForURL(origin+'/account/companies/new');result.keyboard_company_entry=true;
  requireFact(await page.getByText('기존 회사가 사용할 콘솔 업무 공간을 등록합니다.',{exact:true}).count()===1,'COMPANY_FORM');
  const companyName='브라우저로 만든 연결 회사 <연구 & 본사>';const slug=`browser-${account.replaceAll('-','')}`;
  const nameInput=page.getByRole('textbox',{name:'회사 이름',exact:true});const slugInput=page.getByRole('textbox',{name:'업무 공간 식별자',exact:true});
  requireFact(await nameInput.count()===1&&await slugInput.count()===1&&await page.getByText('내 계정',{exact:true}).count()>=1,'COMPANY_FORM');
  result.recipient_consequence=await page.getByText('이 계정에 이 회사의 정보 열람과 제한된 권한 관리 기능을 연결합니다. 급여·인사 권한은 포함되지 않습니다.',{exact:true}).count()===1;requireFact(result.recipient_consequence,'COMPANY_FORM');
  result.reflow_setup_320=await reflow320();requireFact(result.reflow_setup_320,'REFLOW_320');
  await nameInput.fill(companyName);await slugInput.fill('-invalid');
  const create=page.getByRole('button',{name:'회사 업무 공간 만들기',exact:true});await create.click();
  result.invalid_input_preserved=(await nameInput.inputValue())===companyName&&(await slugInput.inputValue())==='-invalid'&&(await slugInput.evaluate(input=>input.validity.valid))===false&&result.posts[MUTATION_PATHS[2]]===0;requireFact(result.invalid_input_preserved,'INPUT_PRESERVATION');
  await slugInput.fill(slug);await tabTo(create,8);result.keyboard_company_submit=true;
  stage='company_submit';const createResponse=api('POST',MUTATION_PATHS[2]);await page.keyboard.press('Enter');const response=await createResponse;requireFact(response?.status()===201,'COMPANY_WIRE');
  const sent=response.request().postDataJSON();requireFact(Object.keys(sent).sort().join(',')==='administrative_account_id,command_id,group_id,name,slug'&&nonnil(sent.command_id)&&sent.group_id===null&&sent.name===companyName&&sent.slug===slug&&sent.administrative_account_id===account,'COMPANY_WIRE');
  const body=await response.json();requireFact(Object.keys(body).sort().join(',')==='administrative_account_id,group_id,org_id,original_command_id,outcome,receipt_id,replayed,result_path'&&body.outcome==='COMMITTED'&&body.original_command_id===sent.command_id&&body.administrative_account_id===account&&body.replayed===false&&[body.org_id,body.group_id,body.receipt_id].every(nonnil)&&body.result_path===`/account/companies/requests/${sent.command_id}`,'COMPANY_WIRE');
  result.company_wire=true;result.command_id=sent.command_id;result.org_id=body.org_id;
  const committedPanel=page.locator('[data-company-outcome="committed"]');
  const openCompany=committedPanel.getByRole('link',{name:'업무 공간 열기',exact:true});
  const completionHeading=committedPanel.getByRole('heading',{name:'생성 완료',exact:true,level:1});
  const companyHeading=committedPanel.getByRole('heading',{name:companyName,exact:true,level:2});
  async function committedCompanyResult(){return await committedPanel.count()===1&&await completionHeading.count()===1&&await companyHeading.count()===1&&await openCompany.count()===1&&await completionHeading.isVisible()&&await companyHeading.isVisible()&&await openCompany.isVisible()&&await openCompany.getAttribute('href')===`/companies/${body.org_id}`;}
  stage='company_result';await page.waitForURL(origin+body.result_path);await page.getByText('생성 완료',{exact:true}).waitFor();
  result.company_result=await committedCompanyResult();requireFact(result.company_result,'COMPANY_RESULT');
  result.reflow_result_320=await reflow320();requireFact(result.reflow_result_320,'REFLOW_320');await secretFree();await capture('04-company-created.png');
  emit({kind:'CHECKPOINT',phase:'COMPANY_COMMITTED',account_id:account,command_id:sent.command_id,org_id:body.org_id,group_id:body.group_id,receipt_id:body.receipt_id});requireFact(validCheckpointCommand(await receive(),'COMPANY_COMMITTED'),'OWNER_PROTOCOL');result.checkpoints.push('COMPANY_COMMITTED');
  stage='company_reopen';await page.reload({waitUntil:'domcontentloaded'});result.company_reopen=page.url()===origin+body.result_path&&await page.getByText('생성 완료',{exact:true}).count()===1&&await committedCompanyResult();requireFact(result.company_reopen,'COMPANY_REOPEN');
  await openCompany.click();await page.waitForURL(origin+`/companies/${body.org_id}`);
  stage='company_header_first';await nativeHeader('COMPANY_HEADER_FIRST');
  result.company_workspace=await page.getByRole('main').getByRole('heading',{name:companyName,exact:true}).count()===1&&await page.getByRole('main').getByRole('link',{name:'권한 관리',exact:true}).count()===1;requireFact(result.company_workspace,'COMPANY_WORKSPACE');
  result.reflow_company_320=await reflow320();requireFact(result.reflow_company_320,'REFLOW_320');result.company_layout_320=await companyLayout(320,false);result.company_layout_desktop=await companyLayout(1440,false);await secretFree();await capture('company-workspace-desktop.png');await page.setViewportSize({width:320,height:900});await capture('05-company-workspace.png');requireFact(result.company_layout_320&&result.company_layout_desktop,'COMPANY_WORKSPACE');
  if(!result.policy_entry){
   const w=`/companies/${body.org_id}`,root=w+'/policy',main=page.getByRole('main');
   async function policyRootContents(response){
    requireFact(response.status()===200&&response.request().method()==='GET'&&response.request().redirectedFrom()===null,'POLICY_ROOT');
    const headers=await response.allHeaders();
    requireFact(headers['cache-control']==='no-store'&&headers['x-content-type-options']==='nosniff'&&headers['content-security-policy']?.includes("script-src 'none'"),'POLICY_ROOT');
    requireFact(await main.getByRole('heading',{name:'권한 관리',exact:true,level:1}).isVisible(),'POLICY_ROOT');
    for(const name of ['연결된 사용 조항','위임 상한에 포함된 조항'])requireFact(await main.getByRole('heading',{name,exact:true,level:2}).count()===1,'POLICY_ROOT');
    requireFact(await main.getByText(/실행.*현재.*정책/).count()>=1,'POLICY_ROOT');
    for(const name of ['사용할 수 있는 기능','다른 계정에 연결할 수 있는 범위'])requireFact(await main.getByRole('heading',{name,exact:true}).count()===0,'POLICY_ROOT');
    const raw=await main.locator('a[href]').evaluateAll(links=>links.map(a=>a.getAttribute('href')));
    requireFact(raw.length===1&&raw[0]===w&&await main.locator('form,button').count()===0,'POLICY_ROOT');
    await secretFree();
   }
   stage='policy_root_open';const policyCard=main.getByRole('link',{name:'권한 관리',exact:true});
   requireFact(await exactVisibleLink(policyCard,root),'POLICY_ROOT');await tabTo(policyCard,48);
   const opened=page.waitForResponse(r=>r.url()===origin+root&&r.request().isNavigationRequest());
   await page.keyboard.press('Enter');await page.waitForURL(origin+root);await nativeHeader('POLICY_ROOT_OPENED');
   await policyRootContents(await opened);await capture('company-policy-root-320.png');
   stage='policy_root_reopen';const reopened=await page.reload({waitUntil:'domcontentloaded'});await nativeHeader('POLICY_ROOT_REOPENED');
   await policyRootContents(reopened);await page.setViewportSize({width:1280,height:900});await capture('company-policy-root-desktop.png');await page.setViewportSize({width:320,height:900});
   stage='policy_root_return';const back=main.getByRole('link',{name:'회사 업무 공간으로',exact:true});
   requireFact(await exactVisibleLink(back,w),'POLICY_ROOT');await tabTo(back,48);
   const returned=page.waitForResponse(r=>r.url()===origin+w&&r.request().isNavigationRequest());
   await page.keyboard.press('Enter');await page.waitForURL(origin+w);requireFact((await returned).status()===200,'POLICY_ROOT');
   await nativeHeader('COMPANY_POLICY_RETURNED');requireFact(await main.getByRole('heading',{name:companyName,exact:true,level:1}).isVisible(),'COMPANY_WORKSPACE');
   result.policy_root_journey=true;
  }
  // A fresh real document read of the stable result URL models reopening; no back-forward cache is accepted as server evidence.
  await page.goto(origin+body.result_path,{waitUntil:'domcontentloaded'});result.company_back=page.url()===origin+body.result_path&&await page.getByText('생성 완료',{exact:true}).count()===1&&await committedCompanyResult();requireFact(result.company_back,'COMPANY_REOPEN');
  result.business_input_not_stored=await page.evaluate(({name,slug})=>!JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}).includes(name)&&!JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}).includes(slug),{name:companyName,slug});requireFact(result.business_input_not_stored,'BUSINESS_STORAGE');
  await checkpoint('COMPANY_REOPENED',account);
  if(result.policy_entry){
   stage='current_context';await page.goto(origin+'/account',{waitUntil:'domcontentloaded'});
   const currentCompany=page.getByRole('link',{name:companyName,exact:true});
   result.current_context=await currentCompany.count()===1&&await currentCompany.isVisible()&&await currentCompany.getAttribute('href')===`/companies/${body.org_id}`;
   requireFact(result.current_context,'CURRENT_CONTEXT');await tabTo(currentCompany,24);await page.keyboard.press('Enter');await page.waitForURL(origin+`/companies/${body.org_id}`);
   result.payroll_nav_before_grant_absent=await page.locator(`a[href="/companies/${body.org_id}/payroll"]`).count()===0;requireFact(result.payroll_nav_before_grant_absent,'POLICY_ENTRY');
   stage='policy_entry';const policy=page.getByRole('main').getByRole('link',{name:'급여 목록 열람 권한',exact:true});const install=`/companies/${body.org_id}/policy/payroll-read/install`;
   result.policy_entry_link=await policy.count()===1&&await policy.isVisible()&&await policy.getAttribute('href')===install;
   requireFact(result.policy_entry_link,'POLICY_ENTRY');await checkpoint('POLICY_ENTRY_READY',account);
   await tabTo(policy,24);const destination=page.waitForResponse(r=>r.url()===origin+install&&r.request().isNavigationRequest());await page.keyboard.press('Enter');result.keyboard_policy_entry=true;
   stage='policy_preflight';const preflight=await destination;await page.waitForURL(origin+install);
   requireFact(preflight.status()===200&&preflight.request().method()==='GET'&&preflight.request().redirectedFrom()===null,'POLICY_PREFLIGHT');
   const serverHtml=await preflight.text();requireFact(serverHtml.includes('data-policy-preflight-operation="install"')&&serverHtml.includes('data-policy-operation="InstallPayrollReadCatalogV1"'),'POLICY_PREFLIGHT');
   requireFact(await page.getByRole('heading',{name:'급여 목록 열람 권한',exact:true,level:1}).isVisible(),'POLICY_PREFLIGHT');
   requireFact(await page.locator('[data-policy-preflight-operation]').getAttribute('data-policy-preflight-operation')==='install','POLICY_PREFLIGHT');
   for(const [attribute,value] of [['data-policy-company',body.org_id],['data-policy-recipient',account],['data-policy-operator',account]])requireFact(await page.locator(`[${attribute}]`).getAttribute(attribute)===value,'POLICY_PREFLIGHT');
   // Conditional scope and initially closed Group identity are observed below;
   // existing exclusions, form admission and exact identity attributes stay immediate.
   for(const text of [companyName,'급여의 상세 내역·수정·지급·내보내기 권한은 연결되지 않습니다. 다른 회사에는 적용되지 않습니다.','설정을 준비해도 계정에 열람 권한이 연결되지는 않습니다.'])requireFact(await page.getByText(text,{exact:true}).first().isVisible(),'POLICY_PREFLIGHT');
   const form=page.locator('form[data-policy-operation="InstallPayrollReadCatalogV1"]');
   requireFact(await page.locator('form').count()===1&&await form.count()===1&&await form.getAttribute('method')==='post'&&await form.getAttribute('action')===`/companies/${body.org_id}/policy/payroll-read/catalog`,'POLICY_PREFLIGHT');
   requireFact(await form.evaluate(installFormControlsSafe),'POLICY_PREFLIGHT');
   const inputs=await form.evaluate(f=>[...f.elements].filter(e=>e.tagName==='INPUT').map(e=>({name:e.name,type:e.type,disabled:e.matches(':disabled'),valid:e.name==='csrf_proof'?e.value.length>0&&e.value.length<=4096:true,value:e.name==='csrf_proof'?null:e.value})));
   requireFact(inputs.length===3&&inputs.map(x=>x.name).sort().join(',')==='command_id,csrf_proof,expected_company_epoch'&&inputs.every(x=>x.type==='hidden'&&!x.disabled&&x.valid),'POLICY_PREFLIGHT');
   const command=inputs.find(x=>x.name==='command_id').value;const epoch=inputs.find(x=>x.name==='expected_company_epoch').value;
   requireFact(nonnil(command)&&command!==sent.command_id&&/^[1-9][0-9]*$/.test(epoch),'POLICY_PREFLIGHT');
   const submitInstall=form.getByRole('button',{name:'열람 권한 설정 준비',exact:true});requireFact(await submitInstall.count()===1&&await submitInstall.isVisible()&&await submitInstall.isEnabled(),'POLICY_PREFLIGHT');
   await tabTo(submitInstall,32);result.keyboard_policy_submit=true;result.policy_preflight=true;
   result.reflow_policy_320=await reflow320();requireFact(result.reflow_policy_320,'REFLOW_320');await secretFree();await capture('06-policy-install-preflight.png');
   await nativeHeader('POLICY_HEADER_PREFLIGHT');
   emit({kind:'CHECKPOINT',phase:'POLICY_PREFLIGHT',account_id:account,org_id:body.org_id,group_id:body.group_id,command_id:command,expected_company_epoch:epoch});requireFact(validCheckpointCommand(await receive(),'POLICY_PREFLIGHT'),'OWNER_PROTOCOL');result.checkpoints.push('POLICY_PREFLIGHT');
   stage='policy_actions';result.policy_documents=[];result.policy_expected_mutations=[];
   const expectDocument=(method,path,status,redirected)=>result.policy_documents.push({method,path,status,redirected});
   const exchange=async(event)=>{emit({kind:'CHECKPOINT',account_id:account,org_id:body.org_id,...event});const reply=await receive();requireFact(reply.kind==='CONTINUE'&&reply.phase===event.phase,'OWNER_PROTOCOL');return reply;};
   const payroll=`/companies/${body.org_id}/payroll`,payrollJson=`/api/v1/companies/${body.org_id}/payroll/runs`;
   const readPhase=async(phase,status,reload=false,open=()=>page.goto(origin+payroll))=>{
    expectDocument('GET',payroll,status,false);const response=reload?await page.reload():await open();
    requireFact(response.status()===status&&response.request().redirectedFrom()===null&&page.url()===origin+payroll,'POLICY_PREFLIGHT');
    const html=await response.text();
    if(status===200){requireFact(html.includes('data-screen="payroll"')&&html.includes('data-state="empty"'),'POLICY_PREFLIGHT');requireFact(await page.locator('[data-screen="payroll"] [data-state="empty"]').isVisible(),'POLICY_PREFLIGHT');}
    else {for(const secret of [account,body.org_id,body.group_id,companyName])requireFact(!html.includes(secret),'SECRET_DISCLOSURE');requireFact(await page.locator('[data-screen="payroll"], [data-run-id], form').count()===0,'SECRET_DISCLOSURE');}
    if(status===200)result.native_headers.push(await assertNativeHeader(page,tabTo,{phase,url:origin+payroll,paths:['/account',payroll],payrollPath:payroll,currentPath:payroll,boundTitle:true}));
    await exchange({phase});result.checkpoints.push(phase);await capture(phase.toLowerCase()+'.png');
   };
   const jsonPhase=async(phase,status)=>{
    const observed=page.waitForResponse(r=>r.url()===origin+payrollJson&&r.request().method()==='GET');
    const value=await page.evaluate(async path=>{const r=await fetch(path,{credentials:'same-origin'});return {status:r.status,text:await r.text()};},payrollJson);
    const response=await observed;requireFact(response.status()===status&&value.status===status&&!response.request().redirectedFrom(),'POLICY_PREFLIGHT');
    if(status===200){const valueJson=JSON.parse(value.text);requireFact(Object.keys(valueJson).sort().join(',')==='items,limit,offset,total'&&valueJson.items.length===0&&valueJson.total===0&&valueJson.limit===100&&valueJson.offset===0,'POLICY_PREFLIGHT');}
    else for(const secret of [account,body.org_id,body.group_id,companyName])requireFact(!value.text.includes(secret),'SECRET_DISCLOSURE');
    await exchange({phase});result.checkpoints.push(phase);
   };
   const {runPolicyJourney}=require('./policy_journey.cjs');
   result.policy=await runPolicyJourney({page,context,company:body.org_id,group:body.group_id,companyName,recipient:account,operator:account,
    expiresAtLocal:new Date(Date.now()+86400000+9*3600000).toISOString().slice(0,16),expectDocument,
    capture:async name=>capture(name+'.png'),
    beforeSubmit:async action=>{result.policy_expected_mutations.push(action.path);await exchange({phase:'POLICY_ACTION_READY',operation:action.operation,command_id:action.command_id,expected_company_epoch:action.expected_company_epoch,fields:action.fields,assignment_id:action.operation==='RevokePayrollReadV1'?action.path.split('/').at(-2):null});},
    checkpoint:async event=>{const reply=await exchange(event);result.checkpoints.push(event.phase);
     const active=['GRANT_COMMITTED','GRANT_REOPENED'].includes(event.phase);
     const kind=event.phase==='CATALOG_INSTALLED'?'install':active?'grant':'revoke';
     const w=`/companies/${body.org_id}`;
     result.native_headers.push(await assertNativeHeader(page,tabTo,{phase:event.phase,url:origin+`${w}/policy/payroll-read/requests/${kind}/${event.command_id}`,paths:['/account',w,w+'/policy',...(active?[payroll]:[])],payrollPath:active?payroll:undefined,boundTitle:true}));
     return reply.witness;},
    readPayroll:async()=>{const receiptPath=new URL(page.url()).pathname;
     const receiptPayroll=page.getByRole('link',{name:'급여',exact:true});
     result.payroll_receipt_link=await exactVisibleLink(receiptPayroll,payroll);
     requireFact(result.payroll_receipt_link,'POLICY_ENTRY');await secretFree();await capture('payroll-navigation-receipt-320.png');
     if(page.viewportSize().width<=680){
      const summary=page.getByRole('banner').locator('summary').filter({hasText:/^업무 탐색$/});
      requireFact(await summary.count()===1,'KEYBOARD_DISCOVERY');const menu=summary.locator('..');
      requireFact(await menu.evaluate(e=>e.open)===false,'KEYBOARD_DISCOVERY');
      await tabTo(summary,48);await page.keyboard.press('Enter');requireFact(await menu.evaluate(e=>e.open)===true,'KEYBOARD_DISCOVERY');
     }
     const workspacePath=`/companies/${body.org_id}`,workspace=page.getByRole('banner').getByRole('link',{name:'회사 업무 공간',exact:true});
     requireFact(await exactVisibleLink(workspace,workspacePath),'COMPANY_WORKSPACE');
     expectDocument('GET',workspacePath,200,false);await tabTo(workspace,32);
     const workspaceResponse=page.waitForResponse(r=>r.url()===origin+workspacePath&&r.request().isNavigationRequest());await page.keyboard.press('Enter');
     requireFact((await workspaceResponse).status()===200,'COMPANY_WORKSPACE');await page.waitForURL(origin+workspacePath);
     await nativeHeader('PAYROLL_COMPANY_HEADER');
     const payrollLink=page.getByRole('main').getByRole('link',{name:'급여',exact:true});
     result.payroll_workspace_link=await exactVisibleLink(payrollLink,payroll);
     requireFact(result.payroll_workspace_link,'POLICY_ENTRY');result.reflow_payroll_workspace_320=await reflow320();requireFact(result.reflow_payroll_workspace_320,'REFLOW_320');
     result.payroll_company_layout_320=await companyLayout(320,true);await secretFree();await capture('payroll-navigation-workspace-320.png');result.payroll_company_layout_desktop=await companyLayout(1440,true);await capture('payroll-navigation-workspace-desktop.png');await page.setViewportSize({width:320,height:900});requireFact(result.payroll_company_layout_320&&result.payroll_company_layout_desktop,'COMPANY_WORKSPACE');
     await readPhase('PAYROLL_READ',200,false,async()=>{await tabTo(payrollLink,32);const opened=page.waitForResponse(r=>r.url()===origin+payroll&&r.request().isNavigationRequest());await page.keyboard.press('Enter');const response=await opened;await page.waitForURL(origin+payroll);result.keyboard_payroll_navigation=true;return response;});result.payroll_ssr=true;await readPhase('PAYROLL_REOPENED',200,true);result.payroll_reopened=true;await jsonPhase('PAYROLL_JSON',200);result.payroll_json=true;expectDocument('GET',receiptPath,200,false);const back=await page.goto(origin+receiptPath);requireFact(back.status()===200,'POLICY_PREFLIGHT');}
   });
   result.full_policy_journey=true;
   requireFact(await page.locator(`a[href="/companies/${body.org_id}/payroll"]`).count()===0,'POLICY_ENTRY');
   await readPhase('PAYROLL_DENIED',404);result.payroll_denied=true;await jsonPhase('PAYROLL_JSON_DENIED',404);result.payroll_json_denied=true;
   expectDocument('GET','/account',200,false);const accountAgain=await page.goto(origin+'/account');requireFact(accountAgain.status()===200,'CURRENT_CONTEXT');
   const retainedCompany=page.getByRole('link',{name:companyName,exact:true});requireFact(await retainedCompany.count()===1&&await retainedCompany.isVisible()&&await retainedCompany.getAttribute('href')===`/companies/${body.org_id}`,'CURRENT_CONTEXT');
   expectDocument('GET',`/companies/${body.org_id}`,200,false);await tabTo(retainedCompany,24);await page.keyboard.press('Enter');await page.waitForURL(origin+`/companies/${body.org_id}`);
   await nativeHeader('REVOKED_COMPANY_HEADER');
   result.payroll_nav_after_revoke_absent=await page.locator(`a[href="/companies/${body.org_id}/payroll"]`).count()===0;requireFact(result.payroll_nav_after_revoke_absent,'POLICY_ENTRY');
   const retainedPolicy=page.getByRole('main').getByRole('link',{name:'급여 목록 열람 권한',exact:true});requireFact(await retainedPolicy.isVisible()&&await retainedPolicy.getAttribute('href')===install,'POLICY_ENTRY');
   await exchange({phase:'ADMIN_REOPENED'});result.checkpoints.push('ADMIN_REOPENED');result.administration_reopened=true;await capture('policy-administration-retained.png');
   result.policy_ui_findings=require('./policy_journey.cjs').taskEvidenceIssues(result.policy);
   requireFact(result.policy_ui_findings.length===0,'POLICY_TASK_COMPREHENSION');

  }
  if(result.people_entry){
   stage='people_journey';result.people_documents=[];result.people_expected_mutations=[];
   result.people=await require('./people_journey.cjs').runPeopleJourney({page,company:body.org_id,companyName,account,
    capture:async name=>capture(name+'.png'),tabTo,secretFree,
    expectDocument:(method,path,status,redirected)=>result.people_documents.push({method,path,status,redirected}),
    expectMutation:path=>result.people_expected_mutations.push(path),
    exchange:async event=>{emit({kind:'CHECKPOINT',account_id:account,org_id:body.org_id,...event});const reply=await receive();requireFact(reply.kind==='CONTINUE'&&reply.phase===event.phase,'OWNER_PROTOCOL');return reply.witness;}
   });
  }
  requireFact(!result.relay_failure&&!result.tls_client_error,'TLS_RELAY_FAILED');requireFact(result.external_requests===0,'EXTERNAL_REQUEST');requireFact(completeObservations(result),'OBSERVATION_INCOMPLETE');
  await cdp.send('WebAuthn.removeVirtualAuthenticator',{authenticatorId:auth.authenticatorId});await cdp.detach();
 }catch(error){result.failure??={stage,code:publicError(error)};}
 finally{clearTimeout(watchdog);await finish();}
}
module.exports={expectedNativeHeaders,completeNativeHeaders,validCompanyLayout,completeCompanyWorkspace,exactVisibleLink,completeNavigation,completePolicyPlan,expectedDocumentRows,completeDocumentsOriginal,completeMutationsOriginal,installFormControlsSafe,completeMutations,observeMutations,completeDocuments,observeDocuments,validOrigin,validCheckpointCommand,completeObservations,leafStatus,scrub,watchOwner,closeOwnedBrowser,certificateArgs,publicError,configureResponseRetention};
if(require.main===module)main(process.argv[2],process.argv[3],process.argv[4]).catch(()=>{process.stderr.write('Company UI browser producer initialization failed; no acceptance result.\n');process.exitCode=2;});
