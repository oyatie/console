const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const net = require('node:net');
const {spawn, execFileSync} = require('node:child_process');
const runtime = '/private/tmp/console-root-entry-browser-smoke-20260919-round2/runtime/node_modules/playwright';
const {chromium} = require(runtime);
const root = '/private/tmp/console-client-release-20260919';
const binary = '/private/tmp/console-rust1981-owner-target/debug/console-app';
const executable = '/private/tmp/console-preparation-browsers/chromium_headless_shell-1243/chrome-headless-shell-mac-arm64/chrome-headless-shell';
const out = process.argv[2];
if (!out || !out.startsWith('/private/tmp/console-native-recovery-browser-') || fs.existsSync(out)) throw Error('new private output directory required');
fs.mkdirSync(out, {mode:0o700});
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
const hashFile = p => new Promise((resolve,reject) => {const h=crypto.createHash('sha256');fs.createReadStream(p).on('data',b=>h.update(b)).on('error',reject).on('end',()=>resolve(h.digest('hex')));});
const sourceFiles = ['backend/app/src/lib.rs','backend/crates/platform/auth-rest/src/account_entry.rs','backend/crates/payroll/ui/src/native_account.rs','backend/crates/payroll/ui/src/native_account.css','backend/crates/payroll/ui/src/native_account.js'];
const custody = async () => ({head:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),binary_sha256:await hashFile(binary),sources:Object.fromEntries(await Promise.all(sourceFiles.map(async p=>[p,await hashFile(path.join(root,p))])))});
const report = {kind:'real-binary-no-database-native-retry',started_at:new Date().toISOString(),checks:[],requests:[],responses:[],cases:[],screenshots:[],page_errors:[],console:[],request_failures:[],limitations:['No database or authenticated identity. Root fixture cookie only selects unavailable native routing.','Does not establish repaired authenticated browser workflow, Company setup, usability study or release acceptance.','Loopback HTTP is not deployment TLS; keyboard focus only, no screen reader/IME/actual zoom qualification.']};
const check = (name,ok,detail) => {report.checks.push({name,pass:!!ok,detail});if(!ok) throw Error(name);};
const delay = ms => new Promise(resolve=>setTimeout(resolve,ms));
let server,browser,logs,base,phase='startup';
const tasks=[];
async function envelope(response) {
 const headers=await response.allHeaders();
 const vary=new Set((headers.vary||'').toLowerCase().split(',').map(s=>s.trim()));
 check(`${phase}: complete private envelope`,vary.has('cookie')&&vary.has('origin')&&headers['x-content-type-options']==='nosniff'&&headers['referrer-policy']==='no-referrer'&&headers['content-security-policy']?.includes("frame-ancestors 'none'"));
 check(`${phase}: private503`,response.status()===503 && headers['cache-control']==='no-store' && headers.pragma==='no-cache' && !('set-cookie' in headers) && headers['content-security-policy']?.includes("base-uri 'none'"));
}
async function retry(page, keyboard=false) {
 const prior=new URL(page.url());
 const link=page.getByRole('link',{name:'다시 시도',exact:true});
 check(`${phase}: literal empty href`,await link.getAttribute('href')==='');
 const resolved=new URL(await link.evaluate(a=>a.href));
 check(`${phase}: same current destination`,resolved.origin===prior.origin && resolved.pathname===prior.pathname && resolved.search===prior.search);
 check(`${phase}: no base/form/island`,await page.locator('base,form,leptos-island').count()===0);
 await Promise.all(tasks);
 const start=report.requests.length;const responseStart=report.responses.length;
 const get=page.waitForRequest(r=>r.isNavigationRequest()&&r.frame()===page.mainFrame(),{timeout:10000});
 const reply=page.waitForResponse(r=>r.request().isNavigationRequest()&&r.request().frame()===page.mainFrame(),{timeout:10000});
 if(keyboard){
  await link.focus();
  const focus=await link.evaluate(a=>({active:document.activeElement===a,outline:getComputedStyle(a).outlineStyle,outlineWidth:getComputedStyle(a).outlineWidth,visible:a.getBoundingClientRect().width>0}));
  check(`${phase}: keyboard focus visible`,focus.active&&focus.visible&&focus.outline!=='none'&&focus.outlineWidth!=='0px',focus);
  const file='keyboard-retry-focus.png';await page.screenshot({path:path.join(out,file),fullPage:true});report.screenshots.push(file);
  await page.keyboard.press('Enter');
 } else await link.click();
 const [request,response]=await Promise.all([get,reply]);
 await page.waitForLoadState('networkidle');
 const sent=new URL(request.url());const final=new URL(page.url());
 check(`${phase}: actual fresh same-route GET`,request.method()==='GET'&&sent.origin===prior.origin&&sent.pathname===prior.pathname&&sent.search===prior.search);
 check(`${phase}: final route/query preserved`,final.origin===prior.origin&&final.pathname===prior.pathname&&final.search===prior.search);
 const navigations=report.requests.slice(start).filter(r=>r.navigation);
 check(`${phase}: exactly one new document request`,navigations.length===1,navigations);
 await envelope(response);
 await Promise.all(tasks);
 const replies=report.responses.slice(responseStart).filter(r=>r.navigation);
 check(`${phase}: exactly one new document response`,replies.length===1,replies.map(r=>({url:r.url,status:r.status})));
 check(`${phase}: current unavailable owner body`,await page.getByRole('heading',{name:'지금은 계정을 확인할 수 없습니다',exact:true}).isVisible());
 report.cases.push({phase,keyboard,before:prior.href,raw_href:'',resolved:resolved.href,request:request.url(),response:response.url(),status:response.status(),final:page.url(),new_document_requests:navigations.length,new_document_responses:replies.length});
}
(async()=>{
 try{
  report.custody_before=await custody();
  const listener=net.createServer();await new Promise((resolve,reject)=>listener.once('error',reject).listen(0,'127.0.0.1',resolve));const port=listener.address().port;await new Promise(resolve=>listener.close(resolve));base=`http://127.0.0.1:${port}`;
  const env={PATH:process.env.PATH,CONSOLE_APP_ROLE:'api',CONSOLE_HTTP_ADDR:`127.0.0.1:${port}`,RUST_LOG:'warn'};
  report.server={binary,base,environment:env};logs=fs.createWriteStream(path.join(out,'server.log'),{mode:0o600});server=spawn(binary,[],{cwd:out,env,stdio:['ignore','pipe','pipe']});server.stdout.pipe(logs);server.stderr.pipe(logs);
  let ready=false;for(let i=0;i<100;i++){if(server.exitCode!==null)throw Error('own server exited');try{if((await fetch(base+'/healthz')).status===200){ready=true;break;}}catch{}await delay(100);}check('own loopback server ready',ready);
  const listeners=execFileSync('/usr/sbin/lsof',['-nP','-a','-p',String(server.pid),'-iTCP','-sTCP:LISTEN'],{encoding:'utf8'});check('owned listener only loopback',listeners.includes(`127.0.0.1:${port}`)&&!listeners.includes('*:'));report.server.listeners=listeners;
  browser=await chromium.launch({headless:true,executablePath:executable,args:['--disable-background-networking','--disable-component-update','--disable-sync','--no-first-run']});report.browser={version:browser.version(),playwright:require(runtime+'/package.json').version,executable,sha256:await hashFile(executable)};
  for(const route of ['/','/account','/account/register']){
   const context=await browser.newContext({viewport:{width:1280,height:900},locale:'ko-KR',extraHTTPHeaders:route==='/'?{Cookie:'__Host-console_account_session=retry-unavailable-fixture'}:{}});
   await context.route('**/*',r=>new URL(r.request().url()).origin===base?r.continue():r.abort('blockedbyclient'));
   context.on('request',r=>report.requests.push({phase,url:r.url(),method:r.method(),resource:r.resourceType(),navigation:r.isNavigationRequest()}));
   context.on('requestfailed',r=>report.request_failures.push({phase,url:r.url(),error:r.failure()?.errorText}));
   context.on('response',r=>{const captured=phase;tasks.push((async()=>{const headers=await r.allHeaders();const body=await r.body();report.responses.push({phase:captured,url:r.url(),status:r.status(),navigation:r.request().isNavigationRequest(),headers,credential_echo:body.includes(Buffer.from('retry-unavailable-fixture')),body_sha256:sha(body),bytes:body.length});})());});
   const page=await context.newPage();page.on('pageerror',e=>report.page_errors.push(String(e)));page.on('console',m=>report.console.push({phase,type:m.type(),text:m.text(),location:m.location()}));
   for(const suffix of ['', '?retry_probe=1','?return_to=https%3A%2F%2Fexample.invalid%2Foutside','?retry_probe=fragment#main-content']){
    phase=route+suffix;const requested=new URL(base+route+suffix);const response=await page.goto(requested.href,{waitUntil:'networkidle'});
    const initial=new URL(page.url());const received=new URL(response.url());
    check(`${phase}: initial requested route/query preserved`,[initial,received].every(u=>u.origin===requested.origin&&u.pathname===requested.pathname&&u.search===requested.search));
    check(`${phase}: initial response is direct GET`,response.request().method()==='GET'&&response.request().redirectedFrom()===null);
    await envelope(response);await retry(page);
   }
   if(route==='/account/register'){phase='keyboard-register';await retry(page,true);}
   await Promise.all(tasks);await context.close();
  }
  check('all thirteen actual retries executed',report.cases.length===13);
  check('no POST API island or external requests',report.requests.every(r=>r.method==='GET'&&new URL(r.url).origin===base&&!/^\/(api|pkg)\//.test(new URL(r.url).pathname)));
  check('no Set-Cookie responses',report.responses.every(r=>!('set-cookie' in r.headers)));
  check('no credential echo',!report.responses.some(r=>r.credential_echo||Object.values(r.headers).some(v=>v.includes('retry-unavailable-fixture'))));
  check('no page errors',report.page_errors.length===0);
  check('only expected document503 console errors',report.console.filter(m=>m.type==='error').every(m=>m.text.includes('503')&&report.responses.some(r=>r.navigation&&r.status===503&&r.url===m.location.url)));
  check('no failed requests',report.request_failures.length===0);
  for(const asset of ['css','js']){const responses=report.responses.filter(r=>new URL(r.url).pathname===`/assets/native-account.${asset}`);check(`real ${asset} asset bytes`,responses.length>0&&responses.every(r=>r.status===200&&r.body_sha256===report.custody_before.sources[`backend/crates/payroll/ui/src/native_account.${asset}`]));}
 }catch(error){report.error=error.stack||String(error);report.checks.push({name:'completed harness',pass:false,detail:report.error});}
 finally{
  if(browser)await browser.close().catch(()=>{});
  if(server){if(server.exitCode===null){server.kill('SIGTERM');for(let i=0;i<60&&server.exitCode===null&&server.signalCode===null;i++)await delay(100);if(server.exitCode===null&&server.signalCode===null){server.kill('SIGKILL');await delay(200);}}report.server.stopped=server.exitCode!==null||server.signalCode!==null;}
  if(logs)await new Promise(resolve=>logs.end(resolve));
  const settled=await Promise.allSettled(tasks);report.checks.push({name:'response capture complete',pass:settled.every(r=>r.status==='fulfilled')});
  report.checks.push({name:'owned server stopped',pass:report.server?.stopped===true});
  report.custody_after=await custody();report.checks.push({name:'source and binary stable',pass:JSON.stringify(report.custody_before)===JSON.stringify(report.custody_after)});
  report.finished_at=new Date().toISOString();report.failed=report.checks.filter(c=>!c.pass).length;report.passed=report.checks.length-report.failed;
  fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2)+'\n',{mode:0o600});console.log(JSON.stringify({out,passed:report.passed,failed:report.failed,cases:report.cases.length,error:report.error}));process.exitCode=report.failed?1:0;
 }
})();
