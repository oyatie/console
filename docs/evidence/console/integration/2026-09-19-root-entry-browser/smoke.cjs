const fs = require('node:fs');
const fsp = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');
const net = require('node:net');
const {spawn, execFileSync} = require('node:child_process');
const {chromium} = require('/private/tmp/console-root-entry-browser-smoke-20260919-round2/runtime/node_modules/playwright');
const root = '/private/tmp/console-client-release-20260919';
const out = '/private/tmp/console-root-entry-browser-smoke-20260919-round4';
const binary = '/private/tmp/console-rust1981-owner-target/debug/console-app';
const browserExecutable = '/private/tmp/console-preparation-browsers/chromium_headless_shell-1243/chrome-headless-shell-mac-arm64/chrome-headless-shell';
const sourceFiles = ['backend/app/src/lib.rs','backend/crates/platform/auth-rest/src/account_browser.rs','backend/crates/platform/auth-rest/src/account_entry.rs','backend/crates/platform/auth-rest/src/lib.rs','backend/crates/payroll/ui/src/native_account.rs','backend/crates/payroll/ui/src/native_account.js','backend/crates/payroll/ui/src/native_account.css'];
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
const hashFile = p => new Promise((resolve,reject) => {const h=crypto.createHash('sha256');fs.createReadStream(p).on('data',b=>h.update(b)).on('error',reject).on('end',()=>resolve(h.digest('hex')));});
const delay = ms => new Promise(r=>setTimeout(r,ms));
const report = {kind:'private-loopback-public-entry-browser-smoke', started_at:new Date().toISOString(), checks:[], requests:[], responses:[], request_failures:[], console:[], page_errors:[], screenshots:[], cases:[], limitations:['Public entry smoke only; no database, no Account session/workflow acceptance.','Loopback HTTP is not production TLS/deployment qualification.','200% approximation uses 720x500 CSS viewport and deviceScaleFactor2 (1440x1000 output); it is not actual browser/OS zoom.','No screen reader, Korean IME, live authorization, workload, or independent visual acceptance is established by this run.']};
function check(name, ok, detail) { report.checks.push({name,pass:!!ok,detail}); }
let server, browser, phase='startup', base, logStream;
const responseTasks=[];
async function snapshot(page,label) {
 const state = await page.evaluate(()=>({title:document.title,language:document.documentElement.lang,viewport:document.querySelector('meta[name=viewport]')?.content,innerWidth,innerHeight,devicePixelRatio,scrollWidth:document.documentElement.scrollWidth,clientWidth:document.documentElement.clientWidth,bodyScrollWidth:document.body.scrollWidth,mainCount:document.querySelectorAll('main').length,forms:document.querySelectorAll('form').length,islands:document.querySelectorAll('leptos-island,[data-hk]').length,links:[...document.querySelectorAll('a')].map(a=>({text:a.textContent.trim(),href:a.getAttribute('href'),rect:{x:a.getBoundingClientRect().x,y:a.getBoundingClientRect().y,width:a.getBoundingClientRect().width,height:a.getBoundingClientRect().height}})),styleSheets:[...document.styleSheets].map(s=>({href:s.href,rules:s.cssRules.length})),card:document.querySelector('.entry-card')?{fontFamily:getComputedStyle(document.querySelector('.entry-card')).fontFamily,background:getComputedStyle(document.querySelector('.entry-card')).backgroundColor,borderRadius:getComputedStyle(document.querySelector('.entry-card')).borderRadius}:null}));
 report.cases.push({label,state});
 check(`${label}: no horizontal document overflow`,state.scrollWidth<=state.clientWidth && state.bodyScrollWidth<=state.clientWidth,{scrollWidth:state.scrollWidth,bodyScrollWidth:state.bodyScrollWidth,clientWidth:state.clientWidth});
 check(`${label}: Korean SSR landmark and no forms/islands`,state.language==='ko'&&state.mainCount===1&&state.forms===0&&state.islands===0);
 check(`${label}: CSS stylesheet applied`,state.styleSheets.some(s=>s.href?.endsWith('/assets/native-account.css')&&s.rules>0)&&state.card?.borderRadius!=='0px',state.card);
 const file=`${label}.png`; await page.screenshot({path:path.join(out,file),fullPage:true}); report.screenshots.push(file);
 return state;
}
async function instrument(context) {
 await context.route('**/*',route=>{ const u=new URL(route.request().url());if(u.origin===base)return route.continue();report.requests.push({phase,method:route.request().method(),url:u.href,blocked_external:true});return route.abort('blockedbyclient');});
 context.on('request',req=>report.requests.push({phase,method:req.method(),url:req.url(),resource_type:req.resourceType()}));
 context.on('requestfailed',req=>report.request_failures.push({phase,url:req.url(),error:req.failure()?.errorText}));
 context.on('response',res=>{const captured=phase; responseTasks.push((async()=>{let body;try{body=await res.body();}catch{} report.responses.push({phase:captured,url:res.url(),status:res.status(),headers:await res.allHeaders(),body_sha256:body?sha(body):null,body_bytes:body?.length});})());});
 context.on('page',page=>{page.on('console',msg=>report.console.push({phase,type:msg.type(),text:msg.text(),location:msg.location()}));page.on('pageerror',err=>report.page_errors.push({phase,message:err.message}));});
}
(async()=>{
 try {
  const before={git_head:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),git_status:execFileSync('git',['status','--short'],{cwd:root,encoding:'utf8'}),source_hashes:Object.fromEntries(await Promise.all(sourceFiles.map(async p=>[p,await hashFile(path.join(root,p))]))),binary_sha256:await hashFile(binary),binary_bytes:fs.statSync(binary).size,binary_mtime:fs.statSync(binary).mtime.toISOString()};
  report.custody_before=before;
  const listener=net.createServer(); await new Promise((res,rej)=>listener.once('error',rej).listen(0,'127.0.0.1',res)); const port=listener.address().port;await new Promise(res=>listener.close(res));base=`http://127.0.0.1:${port}`;
  const env={PATH:process.env.PATH,CONSOLE_APP_ROLE:'api',CONSOLE_HTTP_ADDR:`127.0.0.1:${port}`,RUST_LOG:'warn'};
  report.server={binary,base,argv:[],environment:env,environment_note:'Explicit allowlist; no inherited database/auth/cloud/telemetry credentials.'};
  logStream=fs.createWriteStream(path.join(out,'server.log'));
  server=spawn(binary,[],{cwd:out,env,stdio:['ignore','pipe','pipe']});server.stdout.pipe(logStream);server.stderr.pipe(logStream);report.server.pid=server.pid;
  let ready=false;for(let i=0;i<100;i++){if(server.exitCode!==null)throw Error(`server exited ${server.exitCode}`);try{const r=await fetch(`${base}/healthz`);if(r.status===200){ready=true;break;}}catch{}await delay(100);}check('own server ready',ready);if(!ready)throw Error('loopback server readiness deadline');
  try {report.server.listeners=execFileSync('/usr/sbin/lsof',['-nP','-a','-p',String(server.pid),'-iTCP','-sTCP:LISTEN'],{encoding:'utf8'});check('own listener only loopback',report.server.listeners.includes(`127.0.0.1:${port}`)&&!report.server.listeners.includes('*:'));}catch(e){report.server.listener_error=String(e);check('own listener identity captured',false);}
  browser=await chromium.launch({headless:true,executablePath:browserExecutable,args:['--disable-background-networking','--disable-component-update','--disable-sync','--no-first-run']});report.browser={version:browser.version(),playwright_version:require('/private/tmp/console-root-entry-browser-smoke-20260919-round2/runtime/node_modules/playwright/package.json').version,executable:browserExecutable,executable_sha256:await hashFile(browserExecutable)};
  phase='desktop-root';const context=await browser.newContext({viewport:{width:1440,height:1000},deviceScaleFactor:1,locale:'ko-KR'});await instrument(context);const page=await context.newPage();
  let response=await page.goto(base,{waitUntil:'networkidle'});check('desktop root status200',response.status()===200);const headers=await response.allHeaders();check('root privacy and frame envelope',headers['cache-control']==='no-store'&&headers.pragma==='no-cache'&&headers['content-security-policy']?.includes("frame-ancestors 'none'")&&headers.vary?.includes('Cookie')&&headers.vary?.includes('Origin'),headers);await fsp.writeFile(path.join(out,'desktop-root.html'),await page.content());
  await snapshot(page,'desktop-root');check('login/register destinations visible',await page.getByRole('link',{name:'로그인',exact:true}).isVisible()&&await page.getByRole('link',{name:'계정 만들기',exact:true}).isVisible());
  phase='keyboard';await page.keyboard.press('Tab');let focus=await page.evaluate(()=>({text:document.activeElement.textContent.trim(),href:document.activeElement.getAttribute('href'),outline:getComputedStyle(document.activeElement).outlineStyle,rect:document.activeElement.getBoundingClientRect().toJSON()}));report.keyboard=[{step:'Tab1',...focus}];check('keyboard skip link first and visible',focus.href==='#main-content'&&focus.rect.y>=0&&focus.outline!=='none',focus);await page.keyboard.press('Enter');check('skip link moves focus to main',await page.evaluate(()=>document.activeElement.id==='main-content'));await page.goto(base,{waitUntil:'networkidle'});
  for(let i=1;i<=4;i++){await page.keyboard.press('Tab');focus=await page.evaluate(()=>({text:document.activeElement.textContent.trim(),href:document.activeElement.getAttribute('href'),outline:getComputedStyle(document.activeElement).outlineStyle}));report.keyboard.push({step:`fresh Tab${i}`,...focus});}
  check('keyboard register action receives focus',focus.href==='/account/register'&&focus.outline!=='none',focus);await page.screenshot({path:path.join(out,'desktop-keyboard-focus.png'),fullPage:true});report.screenshots.push('desktop-keyboard-focus.png');
  phase='unrelated-cookie-root';await context.addCookies([{name:'theme',value:'root-smoke-theme',url:base}]);response=await page.goto(base,{waitUntil:'networkidle'});check('unrelated cookie root still Public',response.status()===200&&await page.getByRole('link',{name:'계정 만들기',exact:true}).isVisible());
  phase='account-unavailable';await Promise.all([page.waitForURL(`${base}/account`),page.getByRole('link',{name:'로그인',exact:true}).click()]);await page.waitForLoadState('networkidle');check('login link reaches actual unavailable document',await page.getByRole('heading',{name:'지금은 계정을 확인할 수 없습니다'}).isVisible());await page.screenshot({path:path.join(out,'account-unavailable.png'),fullPage:true});report.screenshots.push('account-unavailable.png');
  phase='register-root-return';await page.goto(base,{waitUntil:'networkidle'});phase='register-unavailable';await Promise.all([page.waitForURL(`${base}/account/register`),page.getByRole('link',{name:'계정 만들기',exact:true}).click()]);await page.waitForLoadState('networkidle');check('register link reaches actual unavailable document',await page.getByRole('heading',{name:'지금은 계정을 확인할 수 없습니다'}).isVisible());await page.screenshot({path:path.join(out,'register-unavailable.png'),fullPage:true});report.screenshots.push('register-unavailable.png');await context.close();
  for(const [label,viewport,deviceScaleFactor] of [['mobile-320',{width:320,height:900},1],['desktop-200pct-approximation',{width:720,height:500},2]]){phase=label;const ctx=await browser.newContext({viewport,deviceScaleFactor,locale:'ko-KR'});await instrument(ctx);const p=await ctx.newPage();const r=await p.goto(base,{waitUntil:'networkidle'});check(`${label}: root200`,r.status()===200);await snapshot(p,label);check(`${label}: both actions visible`,await p.getByRole('link',{name:'로그인',exact:true}).isVisible()&&await p.getByRole('link',{name:'계정 만들기',exact:true}).isVisible());await ctx.close();}
  await Promise.all(responseTasks);
  for(const asset of ['native-account.css','native-account.js']){const observed=report.responses.filter(r=>new URL(r.url).pathname===`/assets/${asset}`);const source=asset.replace('native-account','native_account');check(`${asset}: actual asset loads and equals source`,observed.length>0&&observed.every(r=>r.status===200&&r.body_sha256===before.source_hashes[`backend/crates/payroll/ui/src/${source}`]),{requests:observed.length,statuses:observed.map(r=>r.status)});}
  check('account links return expected503',report.responses.some(r=>new URL(r.url).pathname==='/account'&&r.status===503)&&report.responses.some(r=>new URL(r.url).pathname==='/account/register'&&r.status===503));
  check('no incidental mutation requests',report.requests.every(r=>['GET','HEAD'].includes(r.method)),report.requests.filter(r=>!['GET','HEAD'].includes(r.method)));
  check('no incidental API or island resources',report.requests.every(r=>!new URL(r.url).pathname.startsWith('/api/')&&!new URL(r.url).pathname.startsWith('/pkg/')),report.requests.map(r=>new URL(r.url).pathname));
  check('no external requests',report.requests.every(r=>new URL(r.url).origin===base));check('no page JS exceptions',report.page_errors.length===0,report.page_errors);
  const rootErrors=report.console.filter(m=>m.type==='error'&&!['account-unavailable','register-unavailable'].includes(m.phase));check('no console errors on public entry',rootErrors.length===0,rootErrors);
  report.expected_destination_console_errors=report.console.filter(m=>m.type==='error'&&['account-unavailable','register-unavailable'].includes(m.phase));
  check('destination console errors only expected503 resource reports',report.expected_destination_console_errors.every(m=>m.text.includes('503')),report.expected_destination_console_errors);
  check('no failed browser requests',report.request_failures.length===0,report.request_failures);
 } catch(error){report.fatal_error=error.stack||String(error);check('smoke completed without harness exception',false,report.fatal_error);}
 finally {
  if(browser)await browser.close().catch(()=>{});
  if(server){if(server.exitCode===null){server.kill('SIGTERM');for(let i=0;i<60&&server.exitCode===null;i++)await delay(100);if(server.exitCode===null){server.kill('SIGKILL');await delay(200);}}report.server.exit_code=server.exitCode;report.server.signal_code=server.signalCode;report.server.stopped=server.exitCode!==null||server.signalCode!==null;check('own server stopped',report.server.stopped);}
  if(logStream)await new Promise(r=>logStream.end(r));
  report.custody_after={source_hashes:Object.fromEntries(await Promise.all(sourceFiles.map(async p=>[p,await hashFile(path.join(root,p))]))),binary_sha256:await hashFile(binary),git_head:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim()};
  check('binary and source bytes stable during smoke',JSON.stringify(report.custody_before.source_hashes)===JSON.stringify(report.custody_after.source_hashes)&&report.custody_before.binary_sha256===report.custody_after.binary_sha256);
  report.finished_at=new Date().toISOString();report.passed=report.checks.filter(c=>c.pass).length;report.failed=report.checks.filter(c=>!c.pass).length;
  await fsp.writeFile(path.join(out,'report.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({passed:report.passed,failed:report.failed,fatal_error:report.fatal_error,server_stopped:report.server?.stopped,screenshots:report.screenshots,failed_checks:report.checks.filter(c=>!c.pass)},null,2));process.exitCode=report.failed?1:0;
 }
})();
