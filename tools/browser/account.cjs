'use strict';
// Real Console browser leaf. TLS relay forwards bytes to the parent's actual axum listener.
// Parent owns DB truth and must acknowledge every checkpoint. No mock routes or cookies.
const fs=require('node:fs');const path=require('node:path');const crypto=require('node:crypto');
const tls=require('node:tls');const net=require('node:net');const readline=require('node:readline');
const {execFileSync}=require('node:child_process');const {once}=require('node:events');
const browserPin={
 'darwin-arm64':['chrome-headless-shell-mac-arm64/chrome-headless-shell','a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
 'linux-x64':['chrome-headless-shell-linux64/chrome-headless-shell','ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e']
}[process.platform+'-'+process.arch];
if(!browserPin)throw Error('UNSUPPORTED_REVIEWED_BROWSER_PLATFORM');
const EXECUTABLE=path.join(__dirname,'runtime','browser',browserPin[0]);
const EXECUTABLE_SHA=browserPin[1];
const PLAYWRIGHT=path.join(__dirname,'runtime','node_modules','playwright');
const TERMS=[{kind:'test.account.service',title:'테스트 서비스 약관'},{kind:'test.account.privacy',title:'테스트 개인정보 안내'}];
const AUTH_PATHS=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish','/api/v2/auth/passkey/login/start','/api/v2/auth/passkey/login/finish','/api/v2/auth/logout'];
function scrub(env){return Object.fromEntries(Object.entries(env).filter(([k])=>!['DEBUG','PWDEBUG','NODE_DEBUG'].includes(k)));}
function requireFact(value,code){if(value!==true){const e=new Error(code);e.code=code;throw e;}}
function delayLimit(promise,ms){let timer;return Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>{const e=new Error('TIMEOUT');e.code='TIMEOUT';reject(e);},ms);})]).finally(()=>clearTimeout(timer));}
function validOrigin(origin){try{const u=new URL(origin);return u.protocol==='https:'&&u.hostname==='localhost'&&/^\d+$/.test(u.port)&&Number(u.port)>0&&Number(u.port)<=65535&&u.pathname==='/'&&!u.search&&!u.hash&&!u.username&&!u.password;}catch{return false;}}
function validCheckpointCommand(value,phase){return value?.kind==='CONTINUE'&&value.phase===phase&&Object.keys(value).sort().join(',')==='kind,phase';}
function completeObservations(r){return r.browser_version==='153.0.8010.12'&&r.unexpected_posts===0&&r.relay_failure!==true&&r.tls_client_error!==true&&r.root_status===200&&r.registration_wire===true&&r.resident===true&&r.cookie_security===true&&r.literal_secret_absent===true&&r.logout_cookie_clear===true&&r.login_wire===true&&r.same_account===true&&r.active_after_login===true&&r.external_requests===0&&r.checkpoints?.join(',')==='ENROLLED,LOGGED_OUT,LOGGED_IN'&&AUTH_PATHS.every(p=>r.posts?.[p]===1)&&r.reflow_root_320===true&&r.reflow_register_320===true&&r.reflow_account_320===true&&r.keyboard_skip===true&&r.keyboard_registration===true&&r.keyboard_terms===true&&r.keyboard_registration_submit===true;}
function leafStatus(r){return !r.failure&&completeObservations(r)&&r.cleanup?.confirmed===true?'BROWSER_LEAF_PASSED':'BROWSER_LEAF_FAILED';}
function publicError(error){const allowed=new Set(['TIMEOUT','OWNER_PROTOCOL','OWNER_EOF','OWNER_REFUSED','PREREQUISITE','TLS_RELAY_FAILED','UI_PUBLIC_ENTRY_MISSING','UI_LINK_MISSING','TERMS_CONTROL','REGISTRATION_WIRE','REGISTRATION_EFFECT','RESIDENT','COOKIE_SECURITY','SECRET_DISCLOSURE','ACCOUNT_SSR','LOGOUT_EFFECT','LOGIN_WIRE','LOGIN_EFFECT','EXTERNAL_REQUEST','OBSERVATION_INCOMPLETE','REFLOW_320','KEYBOARD_FOCUS','KEYBOARD_DISCOVERY','KEYBOARD_SKIP','KEYBOARD_TERMS']);if(allowed.has(error?.code))return error.code;const network=String(error?.message??'').match(/\bnet::(ERR_[A-Z0-9_]{1,76})\b/);if(network)return network[1];if(error?.name==='TimeoutError')return 'BROWSER_TIMEOUT';if(error?.name==='SyntaxError')return 'INVALID_JSON';if(error?.name==='TypeError')return 'BROWSER_TYPE_ERROR';if(/No (?:resource with given identifier found|data found for resource with given identifier)/.test(String(error?.message??'')))return 'RESPONSE_BODY_UNAVAILABLE';if(String(error?.message??'').includes('Execution context was destroyed'))return 'BROWSER_CONTEXT_DESTROYED';return 'UNCLASSIFIED_FAILURE';}

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

async function main(backendPort,out){
 requireFact(/^\d+$/.test(backendPort)&&Number(backendPort)>0&&Number(backendPort)<=65535&&path.isAbsolute(out),'PREREQUISITE');
 fs.mkdirSync(out,{mode:0o700});
 const result={kind:'REAL_NATIVE_ACCOUNT_UI_BROWSER_LEAF',node_version:process.version,checkpoints:[],screenshots:[],posts:Object.fromEntries(AUTH_PATHS.map(p=>[p,0])),external_requests:0,unexpected_posts:0,cleanup:{confirmed:false},limits:['Chromium153 virtual resident authenticator and automatic presence only; no human or physical-device picker proof.','Ephemeral-SPKI localhost TLS termination and unencrypted owned loopback upstream are test topology, not production TLS/HA certification.','Context routing controls page traffic, not operating-system egress.','Three independent DB acknowledgements are required; parent remains final acceptance owner.','Keyboard discovery and320px geometric reflow are observed on mounted pages; this is not human usability, screen-reader, contrast or complete WCAG certification.']};
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
 const watchdog=setTimeout(async()=>{requestCancel('TIMEOUT');await finish();process.exit(2);},90000);
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
  const page=await context.newPage();page.setDefaultTimeout(8000);
  page.on('request',request=>{const p=new URL(request.url()).pathname;if(request.method()==='POST'){if(Object.hasOwn(result.posts,p))result.posts[p]++;else result.unexpected_posts++;}});
  const cdp=await context.newCDPSession(page);await configureResponseRetention(cdp);await cdp.send('WebAuthn.enable',{enableUI:false});
  const auth=await cdp.send('WebAuthn.addVirtualAuthenticator',{options:{protocol:'ctap2',transport:'internal',hasResidentKey:true,hasUserVerification:true,isUserVerified:true,automaticPresenceSimulation:true}});
  const api=(method,p)=>page.waitForResponse(r=>r.url()===origin+p&&r.request().method()===method,{timeout:10000}).catch(()=>null);
  async function reflow320(){return page.evaluate(()=>document.documentElement.clientWidth===320&&Math.max(document.documentElement.scrollWidth,document.body.scrollWidth)<=321);}
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
  stage='registration';const registrationStart=api('POST',AUTH_PATHS[0]);const registrationFinish=api('POST',AUTH_PATHS[1]);
  await page.keyboard.press('Enter');result.keyboard_registration_submit=true;
  stage='registration_start_response';const startedResponse=await registrationStart;requireFact(startedResponse?.status()===200,'REGISTRATION_WIRE');stage='registration_start_body';const started=await startedResponse.json();
  stage='registration_wire';const creation=started.public_key_options?.publicKey;result.registration_wire=creation?.authenticatorSelection?.residentKey==='required'&&creation?.authenticatorSelection?.requireResidentKey===true&&creation?.authenticatorSelection?.userVerification==='required'&&typeof creation?.user?.id==='string';requireFact(result.registration_wire,'REGISTRATION_WIRE');
  const handle=Buffer.from(creation.user.id,'base64url');requireFact(handle.length===16,'REGISTRATION_WIRE');const hex=handle.toString('hex');const account=`${hex.slice(0,8)}-${hex.slice(8,12)}-${hex.slice(12,16)}-${hex.slice(16,20)}-${hex.slice(20)}`;
  stage='registration_finish_response';const finishedResponse=await registrationFinish;requireFact(finishedResponse?.status()===201,'REGISTRATION_EFFECT');stage='registration_finish_body';const finished=await finishedResponse.json();stage='registration_finish_identity';requireFact(finished.account?.account_id===account,'REGISTRATION_EFFECT');
  stage='registration_account_navigation';await page.waitForURL(origin+'/account');stage='registration_account_active';await page.locator('[data-account-state="active"]').waitFor();stage='registration_account_state';requireFact(await page.locator('[data-account-state="active"]').count()===1&&await page.locator('[data-context-state="empty"]').count()===1,'ACCOUNT_SSR');
  result.reflow_account_320=await reflow320();requireFact(result.reflow_account_320,'REFLOW_320');
  stage='registration_credentials_read';const credentials=await cdp.send('WebAuthn.getCredentials',{authenticatorId:auth.authenticatorId});stage='registration_credentials_verify';result.resident=credentials.credentials.length===1&&credentials.credentials[0].isResidentCredential===true&&credentials.credentials[0].rpId==='localhost'&&Buffer.from(credentials.credentials[0].userHandle,'base64').equals(handle);credentials.credentials.length=0;requireFact(result.resident,'RESIDENT');
  stage='registration_secret_check';const cookies=await secretFree();const access=cookies.find(c=>c.name==='__Host-console_account_session');const refresh=cookies.find(c=>c.name==='__Host-console_account_refresh');result.cookie_security=!!access&&!!refresh&&access.httpOnly&&refresh.httpOnly&&access.secure&&refresh.secure&&access.path==='/'&&refresh.path==='/'&&access.sameSite==='Lax'&&refresh.sameSite==='Strict';requireFact(result.cookie_security,'COOKIE_SECURITY');
  stage='registration_checkpoint';await capture('03-account-enrolled.png');await checkpoint('ENROLLED',account);
  stage='logout';const loggedOut=api('POST',AUTH_PATHS[4]);await page.getByRole('button',{name:'로그아웃',exact:true}).click();const logoutResponse=await loggedOut;requireFact(logoutResponse?.status()===200&&(await logoutResponse.json()).outcome==='COMMITTED','LOGOUT_EFFECT');await page.waitForURL(origin+'/');
  result.logout_cookie_clear=(await context.cookies(origin)).every(c=>!['__Host-console_account_session','__Host-console_account_refresh'].includes(c.name));requireFact(result.logout_cookie_clear,'LOGOUT_EFFECT');await checkpoint('LOGGED_OUT',account);
  stage='login_entry';await page.getByRole('link',{name:'로그인',exact:true}).click();await page.waitForURL(origin+'/account');const choice=page.getByRole('textbox',{name:'패스키 계정 선택',exact:true});requireFact(await choice.count()===1&&await choice.getAttribute('autocomplete')==='username webauthn','UI_LINK_MISSING');
  await capture('04-sign-in.png');
  const loginStart=api('POST',AUTH_PATHS[2]);const loginFinish=api('POST',AUTH_PATHS[3]);await page.getByRole('button',{name:'패스키로 로그인',exact:true}).click();
  const loginStarted=await loginStart;requireFact(loginStarted?.status()===200,'LOGIN_WIRE');const options=(await loginStarted.json()).public_key_options;result.login_wire=options?.mediation==='conditional'&&Array.isArray(options.publicKey?.allowCredentials)&&options.publicKey.allowCredentials.length===0&&options.publicKey.userVerification==='required';requireFact(result.login_wire,'LOGIN_WIRE');
  const loginFinished=await loginFinish;requireFact(loginFinished?.status()===200,'LOGIN_EFFECT');result.same_account=(await loginFinished.json()).account?.account_id===account;requireFact(result.same_account,'LOGIN_EFFECT');
  await page.waitForURL(origin+'/account');await page.locator('[data-account-state="active"]').waitFor();result.active_after_login=await page.locator('[data-context-state="empty"]').count()===1;await secretFree();await capture('05-account-returned.png');await checkpoint('LOGGED_IN',account);
  requireFact(!result.relay_failure&&!result.tls_client_error,'TLS_RELAY_FAILED');requireFact(result.external_requests===0,'EXTERNAL_REQUEST');requireFact(completeObservations(result),'OBSERVATION_INCOMPLETE');
  await cdp.send('WebAuthn.removeVirtualAuthenticator',{authenticatorId:auth.authenticatorId});await cdp.detach();
 }catch(error){result.failure??={stage,code:publicError(error)};}
 finally{clearTimeout(watchdog);await finish();}
}
module.exports={validOrigin,validCheckpointCommand,completeObservations,leafStatus,scrub,watchOwner,closeOwnedBrowser,certificateArgs,publicError,configureResponseRetention};
if(require.main===module)main(process.argv[2],process.argv[3]).catch(()=>{process.stderr.write('Native UI browser producer initialization failed; no acceptance result.\n');process.exitCode=2;});
