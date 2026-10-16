"use strict";
// Test-only producer: owned TLS forwards bytes to the parent's actual application.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const tls=require('node:tls'),net=require('node:net'),readline=require('node:readline');
const {execFileSync}=require('node:child_process');
const {scrub,watchOwner,closeOwnedBrowser,certificateArgs,validOrigin,publicError}=require('./company.cjs');
const {runHandoff,validEvidence}=require('./account_company_handoff.cjs');
const {runControllerControls}=require('./account_controller_controls.cjs');
const {validCompleteEvidence}=require('./account_controller_evidence.cjs');
const pin={
 'darwin-arm64':['chrome-headless-shell-mac-arm64/chrome-headless-shell','a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
 'linux-x64':['chrome-headless-shell-linux64/chrome-headless-shell','ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e']
}[process.platform+'-'+process.arch];
function fact(value,code){if(value!==true)throw Object.assign(new Error(code),{code});}
function bounded(promise,ms){let timer;return Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>reject(Object.assign(new Error('TIMEOUT'),{code:'TIMEOUT'})),ms);})]).finally(()=>clearTimeout(timer));}
// Failure-only fixed enums; never persist error text, inputs, identifiers or credentials.
const diagnosticStages=new Set(["prerequisites","owner_start","browser_launch","a_registration","o_registration","o_logout","o_login","healthy_handoff_entry","own_account_reference","separate_administrator","company_submit","handoff_reopen","owner_cancel","controller/native-first","controller/react-first","controller/keyboard-before-react","controller/autofill-before-react","controller/eventless-value-before-react","controller/ime-before-react","controller/preclaim-focus","controller/partial-bind-rollback","controller/capability-denied","controller/freeze-at-csrf","cleanup","evidence","result_write","result_emit","OTHER"]);
const diagnosticCodes=new Set(["TIMEOUT","OWNER_PROTOCOL","OWNER_EOF","OWNER_REFUSED","PREREQUISITE","TLS_RELAY_FAILED","EXTERNAL_REQUEST","ACCOUNT_SSR","ACCOUNT_REFERENCE_MISSING","ACCOUNT_REFERENCE_INVALID","SEPARATE_ADMIN_CONTROL_MISSING","ADMINISTRATOR_INPUT_INVALID","REACT_ACCOUNT_MOUNT_MISSING","HANDOFF_RECEIPT","HANDOFF_DISCLOSURE","HANDOFF_EVIDENCE","CONTROLLER_EVIDENCE","ACCOUNT_CONTROLLER_INVARIANT","BROWSER_TIMEOUT","INVALID_JSON","BROWSER_TYPE_ERROR","RESPONSE_BODY_UNAVAILABLE","BROWSER_CONTEXT_DESTROYED","UNCLASSIFIED_FAILURE","OTHER"]);
let diagnosticFile,diagnostic,finalizationStage='prerequisites',diagnosticCreated=false,diagnosticIdentity;
function writeFailureDiagnostic(original,finalization){
 if(!diagnosticFile)return;
 diagnostic??={kind:'ACCOUNT_CONTROLLER_PRODUCER_FAILURE_V1',original_stage:null,original_code:null,finalization_stage:null,finalization_code:null};
 for(const [prefix,failure] of [['original',original],['finalization',finalization]])if(failure){
  diagnostic[prefix+'_stage']=diagnosticStages.has(failure.stage)?failure.stage:'OTHER';
  diagnostic[prefix+'_code']=diagnosticCodes.has(failure.code)?failure.code:'OTHER';
 }
 let fd;
 try{
  const flags=fs.constants.O_WRONLY|fs.constants.O_NOFOLLOW|fs.constants.O_NONBLOCK|
   (diagnosticCreated?0:fs.constants.O_CREAT|fs.constants.O_EXCL);
  fd=fs.openSync(diagnosticFile,flags,0o600);const stat=fs.fstatSync(fd,{bigint:true});
  if(!stat.isFile()||(diagnosticCreated&&(stat.dev!==diagnosticIdentity.dev||stat.ino!==diagnosticIdentity.ino)))return;
  fs.ftruncateSync(fd,0);fs.writeFileSync(fd,JSON.stringify(diagnostic)+'\n');
  diagnosticIdentity={dev:stat.dev,ino:stat.ino};diagnosticCreated=true;
 }catch{}finally{if(fd!==undefined)try{fs.closeSync(fd);}catch{}}
}
async function main(port,out){
 fact(pin&&/^\d+$/.test(port)&&Number(port)>0&&Number(port)<=65535&&path.isAbsolute(out),'PREREQUISITE');
 fs.mkdirSync(out,{mode:0o700});diagnosticFile=path.join(out,'producer-diagnostic.json');
 const result={kind:'REAL_REACT_ACCOUNT_CONTROLLER_PARITY_V1',checkpoints:[],screenshots:[],documents:[],mutations:[],csrf_requests:[],observation_failures:0,external_requests:0,cleanup:{confirmed:false}};
 const files=['fixture.key','fixture.crt','fixture.cnf'].map(n=>path.join(out,n)),sockets=new Set();
 const emit=value=>process.stdout.write(JSON.stringify(value)+'\n');
 let relay,reader,launchPromise,cleanupPromise,finishPromise,cancelled=false,finishing=false,stage='prerequisites';
 async function cleanup(){return cleanupPromise??=(async()=>{
  const closed=await closeOwnedBrowser(launchPromise);let relayClosed=true;
  if(closed.pid!==null)result.browser_pid=closed.pid;
  for(const socket of sockets)socket.destroy();
  if(relay){try{await bounded(new Promise(resolve=>relay.close(resolve)),2000);}catch{relayClosed=false;}}
  for(const file of files){try{fs.unlinkSync(file);}catch(e){if(e.code!=='ENOENT')relayClosed=false;}}
  reader?.close();result.cleanup={confirmed:closed.confirmed&&relayClosed,browser_process_exited:closed.confirmed,relay_closed:relayClosed};
 })();}
 async function finish(){if(finishPromise)return finishPromise;finishing=true;return finishPromise=(async()=>{
  finalizationStage='cleanup';await cleanup();finalizationStage='evidence';result.status=!result.failure&&validCompleteEvidence(result)&&result.cleanup.confirmed?'BROWSER_LEAF_PASSED':'BROWSER_LEAF_FAILED';
  finalizationStage='result_write';fs.writeFileSync(path.join(out,'result.json'),JSON.stringify(result,null,2)+'\n',{flag:'wx',mode:0o600});
  finalizationStage='result_emit';emit({kind:'RESULT',status:result.status,result_path:path.join(out,'result.json'),failure_stage:result.failure?.stage??null,failure_code:result.failure?.code??null});
  process.exitCode=result.status==='BROWSER_LEAF_PASSED'?0:2;
 })();}
 function cancel(code){if(finishing||cancelled)return;cancelled=true;result.failure={stage:'owner_cancel',code};void cleanup().catch(()=>{result.cleanup={confirmed:false};});}
 const watchdog=setTimeout(async()=>{cancel('TIMEOUT');await finish();process.exit(2);},240000);
 try{
  const env=scrub(process.env);for(const name of ['DEBUG','PWDEBUG','NODE_DEBUG'])delete process.env[name];
  const executable=path.join(__dirname,'runtime','browser',pin[0]),playwright=path.join(__dirname,'runtime','node_modules','playwright');
  fact(crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex')===pin[1],'PREREQUISITE');
  fact(JSON.parse(fs.readFileSync(path.join(playwright,'package.json'),'utf8')).version==='1.63.0','PREREQUISITE');
  const {chromium}=require(playwright);
  fs.writeFileSync(files[2],'[req]\nprompt=no\ndistinguished_name=dn\nx509_extensions=ext\n[dn]\nCN=localhost\n[ext]\nsubjectAltName=DNS:localhost\n',{mode:0o600,flag:'wx'});
  execFileSync('/usr/bin/openssl',certificateArgs(files),{stdio:'ignore',env,timeout:10000});
  const key=fs.readFileSync(files[0]),cert=fs.readFileSync(files[1]);
  const spki=crypto.createHash('sha256').update(new crypto.X509Certificate(cert).publicKey.export({type:'spki',format:'der'})).digest('base64');
  relay=tls.createServer({key,cert,ALPNProtocols:['http/1.1']},downstream=>{
   if(!['127.0.0.1','::ffff:127.0.0.1','::1'].includes(downstream.remoteAddress)){downstream.destroy();return;}
   const upstream=net.createConnection({host:'127.0.0.1',port:Number(port)});
   for(const socket of [downstream,upstream]){sockets.add(socket);socket.on('close',()=>sockets.delete(socket));}
   downstream.on('error',()=>upstream.destroy());upstream.on('error',()=>{result.relay_failure=true;downstream.destroy();});
   downstream.on('close',()=>upstream.destroy());upstream.on('close',()=>downstream.destroy());
   downstream.pipe(upstream);upstream.pipe(downstream);
  });
  relay.on('tlsClientError',()=>{result.tls_client_error=true;});
  await new Promise((resolve,reject)=>{relay.once('error',reject);relay.listen(0,'127.0.0.1',resolve);});
  const origin=`https://localhost:${relay.address().port}`;fact(validOrigin(origin),'PREREQUISITE');result.origin=origin;
  for(const file of files)fs.unlinkSync(file);
  reader=readline.createInterface({input:process.stdin,crlfDelay:Infinity});const input=reader[Symbol.asyncIterator]();watchOwner(reader,cancel);
  async function receive(initial=false){const line=await (initial?input.next():bounded(input.next(),20000));fact(!line.done&&line.value.length<=8192,'OWNER_EOF');let value;try{value=JSON.parse(line.value);}catch{fact(false,'OWNER_PROTOCOL');}fact(value?.kind!=='ABORT','OWNER_REFUSED');return value;}
  emit({kind:'READY',origin,rp_id:'localhost',tls_spki_sha256:spki,upstream_port:Number(port)});
  stage='owner_start';const start=await receive(true);fact(start?.kind==='START'&&Object.keys(start).length===1,'OWNER_PROTOCOL');fact(!cancelled,'OWNER_REFUSED');
  stage='browser_launch';launchPromise=chromium.launchServer({headless:true,executablePath:executable,timeout:10000,env,args:['--disable-background-networking','--disable-component-update','--no-proxy-server',`--ignore-certificate-errors-spki-list=${spki}`]});
  const server=await launchPromise;result.browser_pid=server.process().pid;emit({kind:'BROWSER_OWNED',pid:result.browser_pid,executable_sha256:pin[1]});fact(!cancelled,'OWNER_REFUSED');
  const browser=await chromium.connect(server.wsEndpoint(),{timeout:10000});result.browser_version=browser.version();fact(result.browser_version==='153.0.8010.12','PREREQUISITE');
  await runHandoff({browser,origin,result,out,emit,receive,setStage:value=>{stage=value;}});
  fact(validEvidence(result),'HANDOFF_EVIDENCE'); result.handoff=JSON.parse(JSON.stringify(result));
  result.controller=await runControllerControls({browser,origin,result,out,emit,receive,setStage:value=>{stage=value;}});
  fact(!result.relay_failure&&!result.tls_client_error,'TLS_RELAY_FAILED');fact(result.external_requests===0,'EXTERNAL_REQUEST');fact(validCompleteEvidence(result),'CONTROLLER_EVIDENCE');
 }catch(error){
  const own=new Set(['ACCOUNT_REFERENCE_MISSING','ACCOUNT_REFERENCE_INVALID','SEPARATE_ADMIN_CONTROL_MISSING','ADMINISTRATOR_INPUT_INVALID','REACT_ACCOUNT_MOUNT_MISSING','HANDOFF_RECEIPT','HANDOFF_DISCLOSURE','HANDOFF_EVIDENCE']);
  result.failure??={stage,code:own.has(error?.code)?error.code:stage.startsWith('controller/')&&error?.code==='ERR_ASSERTION'?'ACCOUNT_CONTROLLER_INVARIANT':publicError(error)};
  writeFailureDiagnostic(result.failure,null);
 }finally{clearTimeout(watchdog);await finish();}
}
if(require.main===module)main(process.argv[2],process.argv[3]).catch(error=>{writeFailureDiagnostic(null,{stage:finalizationStage,code:publicError(error)});process.stderr.write('Account controller browser producer prerequisite failed; no acceptance result.\n');process.exitCode=2;});
