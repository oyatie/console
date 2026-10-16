'use strict';
const fs=require('node:fs');const path=require('node:path');const crypto=require('node:crypto');const readline=require('node:readline');const {once}=require('node:events');
const browserPin={
 'darwin-arm64':['chrome-headless-shell-mac-arm64/chrome-headless-shell','a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
 'linux-x64':['chrome-headless-shell-linux64/chrome-headless-shell','ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e']
}[process.platform+'-'+process.arch];
if(!browserPin)throw Error('UNSUPPORTED_REVIEWED_BROWSER_PLATFORM');
const executable=path.join(__dirname,'runtime','browser',browserPin[0]);
const executableSha=browserPin[1];
const runtime=path.join(__dirname,'runtime','node_modules','playwright');
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const emit=value=>process.stdout.write(JSON.stringify(value)+'\n');
const bound=(promise,ms)=>{let timer;return Promise.race([promise,new Promise((_,reject)=>timer=setTimeout(()=>reject(Error('BOUND')),ms))]).finally(()=>clearTimeout(timer));};
function requireFact(value,label){if(!value)throw Error(label);}
// ORACLE_START
function secretFreeUrl(url,tokens){return tokens.every(token=>!url.includes(token));}
function expectedJsDisabledBlock(result){
 const names=['anonymous','member','foreign-empty','authorized-js-off','authorized'];
 return result.request_failures===1&&Array.isArray(result.request_diagnostics)&&result.request_diagnostics.length===names.length&&result.request_diagnostics.every((entry,index)=>{
  if(!entry||entry.case!==names[index]||!Array.isArray(entry.failed)||!Array.isArray(entry.inflight_at_close)||entry.inflight_at_close.length!==0||!Array.isArray(entry.inflight_after_close)||entry.inflight_after_close.length!==0)return false;
  if(index!==3)return entry.failed.length===0;
  if(entry.failed.length!==1)return false;
  const failed=entry.failed[0];
  return failed&&Number.isSafeInteger(failed.ordinal)&&failed.ordinal>0&&failed.path==='/pkg/console_payroll_ui.js'&&failed.method==='GET'&&failed.resource==='script'&&failed.error==='csp'&&failed.started_phase==='navigation'&&failed.phase==='navigation';
 });
}
function completeEvidence(result){
 const names=['anonymous','member','foreign-empty','authorized-js-off','authorized'];
 return !result.failure&&result.cases?.length===5&&result.cases.every((c,i)=>c.name===names[i]&&c.document_requests===1&&c.document_responses===1&&c.status===200&&c.exact_url===true&&c.envelope===true&&c.credentials_absent===true&&c.expected_rows===true&&c.omission===true)&&result.fixture_statuses?.length===2&&new Set(result.fixture_statuses).size===2&&['BLOCKED_LEGAL_GATE','CALCULATED'].every(s=>result.fixture_statuses.includes(s))&&result.filters?.length===3&&result.filters.every((f,i)=>f.status===[...result.fixture_statuses,''][i]&&f.correct_visibility===true&&f.same_nodes===true&&f.no_requests===true)&&result.js_off_unfiltered===true&&result.assets?.js===true&&result.assets?.wasm===true&&result.external_requests===0&&result.unexpected_mutations===0&&result.page_errors===0&&result.console_errors===0&&expectedJsDisabledBlock(result)&&result.cleanup?.confirmed===true&&result.redirect_control?.source_302===true&&result.redirect_control?.foreign_hop_blocked===true&&result.interception_failures===0&&result.url_disclosures===0;
}
// ORACLE_END
// Diagnostics deliberately persist no headers, URL origin/query/fragment or arbitrary error text.
function sanitizedRequest(request,tokens){
 let url=null;try{const raw=request.url();if(secretFreeUrl(raw,tokens))url=new URL(raw);}catch{}
 const paths=new Set(['/payroll','/pkg/console_payroll_ui.js','/pkg/console_payroll_ui_bg.wasm']);
 const method=request.method();const resource=request.resourceType();
 return {path:url&&paths.has(url.pathname)?url.pathname:'[redacted-or-unrecognized]',method:['GET','POST','PUT','PATCH','DELETE','HEAD','OPTIONS'].includes(method)?method:'OTHER',resource:['document','stylesheet','image','media','font','script','texttrack','xhr','fetch','eventsource','websocket','manifest','other'].includes(resource)?resource:'other'};
}
function sanitizedFailure(request,tokens){
 let text=null;try{text=request.failure()?.errorText;}catch{}
 return typeof text==='string'&&secretFreeUrl(text,tokens)&&(text==='csp'||/^net::ERR_[A-Z0-9_]{1,80}$/.test(text))?text:'UNCLASSIFIED_FAILURE';
}
async function main(){
 const out=process.argv[2];requireFact(path.isAbsolute(out)&&!fs.existsSync(out),'OUTPUT');fs.mkdirSync(out,{mode:0o700});
 const reader=readline.createInterface({input:process.stdin});const lines=reader[Symbol.asyncIterator]();
 const first=await bound(lines.next(),10000);requireFact(!first.done,'INPUT');const input=JSON.parse(first.value);const base=new URL(input.origin);
 requireFact(base.protocol==='http:'&&base.hostname==='127.0.0.1'&&base.port&&base.pathname==='/'&&!base.search&&!base.hash&&!base.username&&!base.password,'ORIGIN');
 requireFact(input.runs.length===2&&input.runs[0].id!==input.runs[1].id&&input.runs[0].status!==input.runs[1].status,'RUN_FIXTURE');
 requireFact(JSON.stringify(input.runs.map(r=>r.status).sort())===JSON.stringify(['BLOCKED_LEGAL_GATE','CALCULATED']),'FIXTURE_STATUSES');
 const sink=new URL(input.redirect_sink);requireFact(sink.protocol==='http:'&&sink.hostname==='127.0.0.1'&&sink.port&&sink.origin!==base.origin&&sink.pathname==='/__test__/hydration/sink'&&!sink.search&&!sink.hash&&!sink.username&&!sink.password,'SINK_ORIGIN');
 const tokens=[input.admin,input.member,input.foreign];requireFact(tokens.every(t=>typeof t==='string'&&t.length>50),'TOKENS');
 const result={kind:'REAL_AUTHORIZED_HYDRATION_ARTIFACT_PROOF',cases:[],fixture_statuses:input.runs.map(r=>r.status),filters:[],screenshots:[],assets:{},external_requests:0,unexpected_mutations:0,page_errors:0,console_errors:0,request_failures:0,request_diagnostics:[],interception_failures:0,url_disclosures:0,redirect_control:{},cleanup:{confirmed:false},limits:['Fixture Bearer navigation only; not native Account entry acceptance.','TEST_ONLY second run status supplies presentation data, not accepted payroll lifecycle.','Isolated SQLx/current authorization and loopback HTTP; no production TLS, screen-reader, IME or representative-user qualification.']};
 let launch,server,browser,cancelled=false;
 const cancel=()=>{cancelled=true;void server?.kill().catch(()=>{});};reader.on('line',line=>{try{if(JSON.parse(line).kind==='ABORT')cancel();}catch{cancel();}});reader.on('close',()=>{if(!result.cleanup.confirmed)cancel();});
 const watchdog=setTimeout(cancel,90000);
 try{
  requireFact(sha(fs.readFileSync(executable))===executableSha,'BROWSER_CUSTODY');requireFact(require(runtime+'/package.json').version==='1.63.0','RUNTIME');
  const {chromium}=require(runtime);launch=chromium.launchServer({headless:true,executablePath:executable,env:{PATH:process.env.PATH},args:['--disable-background-networking','--disable-component-update','--disable-sync','--no-first-run']});server=await bound(launch,15000);emit({kind:'BROWSER_OWNED',pid:server.process().pid,executable_sha256:executableSha});
  browser=await chromium.connect(server.wsEndpoint());result.browser_version=browser.version();requireFact(result.browser_version==='153.0.8010.12','BROWSER_VERSION');
  for(const [name,token,js] of [['anonymous',null,true],['member',input.member,true],['foreign-empty',input.foreign,true],['authorized-js-off',input.admin,false],['authorized',input.admin,true]]){
   requireFact(!cancelled,'OWNER_CANCELLED');
   const context=await browser.newContext({javaScriptEnabled:js,serviceWorkers:'block',viewport:{width:1280,height:900},locale:'ko-KR'});
   // CDP request-stage interception examines every redirect hop. Header
   // overrides are per request, unlike Playwright route.continue headers.
   const page=await context.newPage();const cdp=await context.newCDPSession(page);
   cdp.on('Fetch.requestPaused',event=>{void(async()=>{
    if(!secretFreeUrl(event.request.url,tokens)){result.url_disclosures++;await cdp.send('Fetch.failRequest',{requestId:event.requestId,errorReason:'BlockedByClient'});return;}
    const url=new URL(event.request.url);
    if(url.origin!==base.origin){result.external_requests++;await cdp.send('Fetch.failRequest',{requestId:event.requestId,errorReason:'BlockedByClient'});return;}
    const headers=Object.entries(event.request.headers).filter(([name])=>name.toLowerCase()!=='authorization').map(([name,value])=>({name,value:String(value)}));
    if(token)headers.push({name:'Authorization',value:'Bearer '+token});
    await cdp.send('Fetch.continueRequest',{requestId:event.requestId,headers});
   })().catch(()=>result.interception_failures++);});
   await cdp.send('Fetch.enable',{patterns:[{urlPattern:'*',requestStage:'Request'}]});
   const requests=[],responses=[],tasks=[];
   let phase='navigation';const inflight=new Map();let requestOrdinal=0;const diagnostic={case:name,failed:[],inflight_at_close:[],inflight_after_close:[]};result.request_diagnostics.push(diagnostic);
   context.on('request',request=>{inflight.set(request,{ordinal:++requestOrdinal,...sanitizedRequest(request,tokens),started_phase:phase});if(!secretFreeUrl(request.url(),tokens)){result.url_disclosures++;return;}requests.push({url:request.url(),method:request.method(),document:request.isNavigationRequest(),resource:request.resourceType()});if(request.method()!=='GET')result.unexpected_mutations++;});
   context.on('response',response=>{tasks.push((async()=>{requireFact(secretFreeUrl(response.url(),tokens),'SECRET_URL_DISCLOSURE');const bytes=await response.body();responses.push({url:response.url(),status:response.status(),document:response.request().isNavigationRequest(),mime:response.headers()['content-type'],sha256:sha(bytes),credential_echo:tokens.some(t=>bytes.includes(Buffer.from(t)))});})());});
   context.on('requestfinished',request=>inflight.delete(request));
   context.on('requestfailed',request=>{result.request_failures++;diagnostic.failed.push({...(inflight.get(request)??{ordinal:null,started_phase:'unobserved'}),...sanitizedRequest(request,tokens),error:sanitizedFailure(request,tokens),phase});inflight.delete(request);});
   page.on('pageerror',()=>result.page_errors++);page.on('console',m=>{if(m.type()==='error')result.console_errors++;});
   const expectedUrl=base.origin+'/payroll';const response=await page.goto(expectedUrl,{waitUntil:'networkidle',timeout:15000});await Promise.all(tasks);phase='inspection';const headers=await response.allHeaders();const html=await response.text();
   const vary=new Set((headers.vary||'').toLowerCase().split(',').map(v=>v.trim()));
   const envelope=headers['cache-control']==='no-store'&&headers.pragma==='no-cache'&&!('set-cookie'in headers)&&headers['x-content-type-options']==='nosniff'&&headers['referrer-policy']==='no-referrer'&&headers['content-security-policy']?.includes("frame-ancestors 'none'")&&vary.has('authorization');
   requireFact(envelope,'PRIVATE_ENVELOPE');
   const authorized=name.startsWith('authorized');const expectedIds=authorized?input.runs.map(r=>r.id).sort():[];
   const ids=await page.locator('[data-run-id]').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-run-id')).sort());
   requireFact(JSON.stringify(ids)===JSON.stringify(expectedIds),'AUTHORIZED_ROWS');
   const credentialsAbsent=tokens.every(t=>!html.includes(t))&&await page.evaluate(tokens=>tokens.every(t=>!JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}).includes(t)),tokens)&&responses.every(r=>!r.credential_echo);
   requireFact(credentialsAbsent,'SECRET_DISCLOSURE');
   let omission=true;
   if(!authorized){omission=input.runs.every(r=>!html.includes(r.id))&&!html.includes('/pkg/')&&!html.includes('leptos-island')&&await page.locator('[data-run-status-filter]').count()===0&&requests.every(r=>!new URL(r.url).pathname.startsWith('/pkg/'));requireFact(omission,'DENY_OMISSION');}
   const documents=requests.filter(r=>r.document);const replies=responses.filter(r=>r.document);const exactUrl=page.url()===expectedUrl&&response.url()===expectedUrl&&response.request().redirectedFrom()===null;
   requireFact(documents.length===1&&replies.length===1&&response.status()===200&&exactUrl,'DOCUMENT_COUNT');
   result.cases.push({name,document_requests:documents.length,document_responses:replies.length,status:response.status(),exact_url:exactUrl,envelope,credentials_absent:credentialsAbsent,expected_rows:true,omission,requests:requests.map(r=>({path:new URL(r.url).pathname,method:r.method,resource:r.resource}))});
   if(authorized){
    requireFact(input.runs.every(r=>html.includes(`data-run-id="${r.id}"`)&&html.includes(`data-status="${r.status}"`)),'SSR_FIXTURE');
    const filter=page.locator('[data-run-status-filter]');requireFact(await filter.count()===1,'FILTER');
    const options=await filter.locator('option').evaluateAll(nodes=>nodes.map(n=>n.value).sort());requireFact(JSON.stringify(options)===JSON.stringify(['',...input.runs.map(r=>r.status)].sort()),'AUTHORIZED_OPTIONS');
    const originalNodes=await page.locator('[data-run-id]').elementHandles();
    const originalIds=await Promise.all(originalNodes.map(node=>node.getAttribute('data-run-id')));
    const sameOriginalNodes=async()=>{for(let i=0;i<originalNodes.length;i++){if(!await originalNodes[i].evaluate((node,id)=>node.isConnected&&document.querySelector(`[data-run-id="${id}"]`)===node,originalIds[i]))return false;}return true;};
    const visible=()=>page.locator('[data-run-id]').evaluateAll(nodes=>nodes.filter(n=>!n.closest('a').hidden&&getComputedStyle(n.closest('a')).display!=='none').map(n=>n.getAttribute('data-run-id')).sort());
    requireFact(JSON.stringify(await visible())===JSON.stringify(expectedIds),'INITIAL_VISIBILITY');
    if(!js){phase='js-off-control';await filter.selectOption(input.runs[0].status);requireFact(JSON.stringify(await visible())===JSON.stringify(expectedIds),'JS_DISABLED_CONTROL');result.js_off_unfiltered=true;}
    else{
     const jsAsset=responses.filter(r=>new URL(r.url).pathname==='/pkg/console_payroll_ui.js');const wasmAsset=responses.filter(r=>new URL(r.url).pathname==='/pkg/console_payroll_ui_bg.wasm');
     result.assets.js=jsAsset.length===1&&jsAsset[0].status===200&&jsAsset[0].mime.startsWith('text/javascript')&&jsAsset[0].sha256===input.assets.js;
     result.assets.wasm=wasmAsset.length===1&&wasmAsset[0].status===200&&wasmAsset[0].mime==='application/wasm'&&wasmAsset[0].sha256===input.assets.wasm;
     requireFact(result.assets.js&&result.assets.wasm,'ASSET_HASH');result.asset_responses=[...jsAsset,...wasmAsset];
     const beforeFile='authorized-before.png';await page.screenshot({path:path.join(out,beforeFile),fullPage:true});result.screenshots.push(beforeFile);
     phase='filters';
     for(const status of [input.runs[0].status,input.runs[1].status,'']){
      const start=requests.length;await filter.selectOption(status);
      const expected=input.runs.filter(r=>!status||r.status===status).map(r=>r.id).sort();
      await page.waitForFunction(expected=>JSON.stringify([...document.querySelectorAll('[data-run-id]')].filter(n=>!n.closest('a').hidden&&getComputedStyle(n.closest('a')).display!=='none').map(n=>n.getAttribute('data-run-id')).sort())===JSON.stringify(expected),expected,{timeout:10000});
      const current=await page.locator('[data-run-id]').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-run-id')).sort());const same=JSON.stringify(current)===JSON.stringify(expectedIds)&&await sameOriginalNodes();const correct=JSON.stringify(await visible())===JSON.stringify(expected);const noRequests=requests.length===start;
      requireFact(same&&correct&&noRequests,'FILTER_PRESENTATION');result.filters.push({status,correct_visibility:correct,same_nodes:same,no_requests:noRequests});
      const file='authorized-filter-'+result.filters.length+'.png';await page.screenshot({path:path.join(out,file),fullPage:true});result.screenshots.push(file);
     }
    }
   }
   await Promise.all(tasks);requireFact(requests.every(r=>!new URL(r.url).pathname.startsWith('/api/')),'NO_API_REQUESTS');phase='before-close';diagnostic.inflight_at_close=[...inflight.values()].map(row=>({...row}));phase='closing';await context.close();phase='closed';diagnostic.inflight_after_close=[...inflight.values()].map(row=>({...row}));
  }
  // Real redirect control uses a separate test-only route and owned foreign
  // sink. Parent independently verifies source Authorization and zero sink
  // requests/Authorization; this is not a substituted business response.
  const control=await browser.newContext({serviceWorkers:'block'});const page=await control.newPage();const cdp=await control.newCDPSession(page);let foreignBlocked=false;
  page.on('response',response=>{if(!secretFreeUrl(response.url(),tokens)){result.url_disclosures++;return;}if(response.url()===base.origin+'/__test__/hydration/redirect'&&response.status()===302)result.redirect_control.source_302=true;});
  cdp.on('Fetch.requestPaused',event=>{void(async()=>{
   if(!secretFreeUrl(event.request.url,tokens)){result.url_disclosures++;await cdp.send('Fetch.failRequest',{requestId:event.requestId,errorReason:'BlockedByClient'});return;}
    const url=new URL(event.request.url);
   if(url.origin!==base.origin){requireFact(url.href===input.redirect_sink,'REDIRECT_TARGET');foreignBlocked=true;await cdp.send('Fetch.failRequest',{requestId:event.requestId,errorReason:'BlockedByClient'});return;}
   const headers=Object.entries(event.request.headers).filter(([name])=>name.toLowerCase()!=='authorization').map(([name,value])=>({name,value:String(value)}));headers.push({name:'Authorization',value:'Bearer '+input.admin});
   await cdp.send('Fetch.continueRequest',{requestId:event.requestId,headers});
  })().catch(()=>result.interception_failures++);});
  await cdp.send('Fetch.enable',{patterns:[{urlPattern:'*',requestStage:'Request'}]});
  await page.goto(base.origin+'/__test__/hydration/redirect',{waitUntil:'load',timeout:10000}).catch(()=>{});
  result.redirect_control.foreign_hop_blocked=foreignBlocked;requireFact(result.redirect_control.source_302&&foreignBlocked,'REDIRECT_BARRIER_CONTROL');await control.close();

 }catch(error){result.failure={code:/^[A-Z_]+$/.test(error.message)?error.message:'DRIVER_FAILURE'};}
 finally{
  clearTimeout(watchdog);let exited=false;
  if(launch){try{server=await bound(launch,11000);const child=server.process();await bound(server.close(),5000).catch(()=>{});if(child.exitCode===null&&child.signalCode===null)await bound(server.kill(),5000).catch(()=>{});if(child.exitCode===null&&child.signalCode===null)await bound(once(child,'exit'),3000).catch(()=>{});exited=child.exitCode!==null||child.signalCode!==null;}catch{}}
  result.cleanup={confirmed:exited};reader.close();result.status=completeEvidence(result)?'PASSED':'FAILED';fs.writeFileSync(path.join(out,'result.json'),JSON.stringify(result,null,2)+'\n',{mode:0o600,flag:'wx'});emit({kind:'RESULT',status:result.status});process.exitCode=result.status==='PASSED'?0:2;
 }
}
if(require.main===module)main().catch(()=>{process.stderr.write('Hydration artifact harness prerequisite failure\n');process.exitCode=2;});
module.exports={completeEvidence,expectedJsDisabledBlock,secretFreeUrl,sanitizedRequest,sanitizedFailure};
