import assert from 'node:assert/strict';
import {spawnSync,spawn} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,copyFileSync,chmodSync,rmSync} from 'node:fs';
import {randomBytes,createHash} from 'node:crypto';
import {createServer} from 'node:net';
import {homedir} from 'node:os';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const [rootArg,binaryArg,expectedBinaryHash]=process.argv.slice(2);
if(!rootArg||!binaryArg||!/^([a-f0-9]{64})$/.test(expectedBinaryHash??''))throw new Error('usage: node runtime-smoke.mjs REPO FROZEN_BINARY SHA256');
const root=resolve(rootArg), sourceBinary=resolve(binaryArg);
const {prepareLocalCustody,retainLocalCustodyInventory}=await import(pathToFileURL(root+'/scripts/lib/dev-account-custody.mjs'));
const sha=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
assert.equal(sha(sourceBinary),expectedBinaryHash,'independently frozen build binary changed');
const evidence=mkdtempSync('/private/tmp/lc07-runtime-evidence-');
mkdirSync(homedir()+'/.cache',{recursive:true});
const stage=mkdtempSync(homedir()+'/.cache/console-lc07-stack-');
const project='console-lc07-'+randomBytes(6).toString('hex');
const passwords=Object.fromEntries(['ADMIN','APP','RT','LEAVE_COMMAND','ONTOLOGY_COMMAND','PLATFORM_FORCE_COMMAND'].map(k=>[k,randomBytes(24).toString('hex')]));
const report={project,evidence,cases:[],commands:[],scope:'Actual local Compose PostgreSQL/topology/operator image and host-built API; not published app image or Kubernetes'};
const redact=s=>Object.values(passwords).reduce((s,p)=>s.replaceAll(p,'[REDACTED]'),String(s));
let api, operatorImageId;
const sourcePaths=['.dockerignore','ops/compose.yml','ops/account-custody.Dockerfile','ops/postgres-reconcile-topology.sh','ops/postgres-finalize-account-custody.sh','ops/postgres-finalize-account-custody.sql','ops/account-custody-migrations.sha384','scripts/lib/dev-account-custody.mjs'];
const sources=()=>Object.fromEntries(sourcePaths.map(p=>[p,sha(root+'/'+p)]));
const beforeSources=sources();
function run(bin,args,opts={}) {
 const result=spawnSync(bin,args,{cwd:root,env:process.env,encoding:'utf8',timeout:120000,...opts});
 const output=redact((result.stdout??'')+(result.stderr??''));
 const log=`${report.commands.length+1}.log`;writeFileSync(`${evidence}/${log}`,output);
 report.commands.push({bin,args:args.map(redact),status:result.status,log});
 if(!opts.allowFailure&&(result.error||result.status!==0))throw new Error(`${bin} failed ${result.status}: ${log}`);
 return {...result,output};
}
function compose(args,opts){return run('docker',['compose','-p',project,'-f',root+'/ops/compose.yml','-f',stage+'/override.yml',...args],opts);}
function sql(query){return compose(['exec','-T','postgres','psql','-X','-At','-v','ON_ERROR_STOP=1','-U','console_cluster_admin','-d','console_lc07','-c',query]).stdout.trim();}
function absentResources(){
 const remaining={};const failures=[];
 for(const [kind,args] of [['containers',['ps','-aq']],['volumes',['volume','ls','-q']],['networks',['network','ls','-q']]]){
  const result=run('docker',[...args,'--filter',`label=com.docker.compose.project=${project}`],{allowFailure:true});
  if(result.error||result.status!==0){failures.push('cannot inspect owned '+kind);remaining[kind]='UNKNOWN';}
  else remaining[kind]=result.stdout.trim();
 }
 report.resource_inspection_errors=failures;return remaining;
}
function pass(name){report.cases.push({name,status:'PASS'});console.log(name+': PASS');}
try {
 Object.assign(process.env,{CONSOLE_POSTGRES_DB:'console_lc07',CONSOLE_POSTGRES_ADMIN_PASSWORD:passwords.ADMIN,CONSOLE_APP_POSTGRES_PASSWORD:passwords.APP,CONSOLE_RT_POSTGRES_PASSWORD:passwords.RT,CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD:passwords.LEAVE_COMMAND,CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD:passwords.ONTOLOGY_COMMAND,CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD:passwords.PLATFORM_FORCE_COMMAND});
 const descriptor=prepareLocalCustody(stage+'/custody','docker',passwords.ADMIN);
 copyFileSync(root+'/ops/postgres-reconcile-topology.sh',stage+'/topology.sh');
 copyFileSync(sourceBinary,stage+'/console-app');chmodSync(stage+'/console-app',0o500);
 report.binary_sha256=sha(stage+'/console-app');assert.equal(report.binary_sha256,expectedBinaryHash);
 assert.equal(sha(sourceBinary),expectedBinaryHash);
 assert.equal(run('git',['status','--porcelain']).stdout.trim(),'','source must be clean');
 report.sources=beforeSources;
 report.source_head=run('git',['rev-parse','HEAD']).stdout.trim();
 writeFileSync(stage+'/override.yml',`services:\n  postgres:\n    ports: ["127.0.0.1::5432"]\n  postgres-topology:\n    volumes:\n      - ${stage}/topology.sh:/usr/local/bin/postgres-reconcile-topology:ro\n  account-finalize:\n    image: ${project}-operator:local\n    build:\n      labels:\n        console.lc07.owner: ${project}\n`);
 compose(['config','--quiet']);compose(['up','-d','--wait','--wait-timeout','90','postgres']);
 compose(['run','--rm','postgres-topology']);
 const identity=sql("SELECT system_identifier::text || '|' || (SELECT oid FROM pg_catalog.pg_database WHERE datname=current_database()) FROM pg_catalog.pg_control_system()");
 retainLocalCustodyInventory(descriptor,identity,'console_cluster_admin','console_lc07');
 pass('independent-local-socket-inventory-before-migration');
 const port=compose(['port','postgres','5432']).stdout.trim().split(':').at(-1);
 const url=(role,password)=>`postgresql://${role}:${password}@localhost:${port}/console_lc07?sslmode=verify-full&sslrootcert=${encodeURIComponent(process.env.ACCOUNT_CUSTODY_CA_FILE)}`;
 const baseEnv=Object.fromEntries(Object.entries(process.env).filter(([k])=>!k.startsWith('CONSOLE_')&&!k.startsWith('ACCOUNT_CUSTODY_')&&!k.startsWith('PG')&&!k.endsWith('DATABASE_URL')));
 const migrate=()=>run(stage+'/console-app',[],{env:{...baseEnv,CONSOLE_APP_ROLE:'migrate',DATABASE_URL:url('console_app',passwords.APP)},timeout:300000});
 migrate();
 const roster="SELECT string_agg(relname || ':' || pg_get_userbyid(relowner), ',' ORDER BY relname) FROM pg_class WHERE relnamespace='public'::regnamespace AND relname IN ('accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts')";
 const before=sql(roster);assert.equal(before.split(',').length,6);assert.ok(before.split(',').every(s=>s.endsWith(':console_app')));pass('actual-migrator-creates-six-dormant-staging-tables');
 compose(['build','account-finalize']);
 const imageInfo=JSON.parse(run('docker',['image','inspect',`${project}-operator:local`]).stdout)[0];
 assert.equal(imageInfo.Config.Labels['console.lc07.owner'],project);operatorImageId=imageInfo.Id;report.operator_image_id=operatorImageId;
 const inventory=readFileSync(descriptor,'utf8');writeFileSync(descriptor,inventory.replace(/ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=\d+/, 'ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=1'));
 const denied=compose(['run','--rm','--no-deps','account-finalize'],{allowFailure:true});assert.notEqual(denied.status,0);assert.match(denied.output,/target_mismatch/);assert.equal(sql(roster),before);writeFileSync(descriptor,inventory);pass('wrong-inventory-no-owner-changes');
 compose(['run','--rm','--no-deps','account-finalize']);const after=sql(roster);assert.equal(after.split(',').length,6);assert.deepEqual(Object.fromEntries(after.split(',').map(s=>s.split(':'))),{accounts:'console_account_owner',account_security:'console_account_owner',account_security_events:'console_account_owner',account_terms_acceptances:'console_account_owner',account_terms_head:'console_terms_owner',account_terms_release_receipts:'console_terms_owner'});pass('readonly-unprivileged-operator-image-finalizes-over-verified-TLS');
 compose(['run','--rm','--no-deps','account-finalize']);assert.equal(sql(roster),after);pass('repeated-finalization-preserves-custody');
 migrate();compose(['run','--rm','postgres-topology']);assert.equal(sql(roster),after);pass('repeat-migration-and-topology-preserve-finalized-custody');
 // The same actual executable boot, with TLS runtime/command pools, owns readiness.
 const reservation=createServer();await new Promise((resolve,reject)=>{reservation.once('error',reject);reservation.listen(0,'127.0.0.1',resolve);});
 const apiPort=reservation.address().port;await new Promise(resolve=>reservation.close(resolve));
 const env={...baseEnv,CONSOLE_APP_ROLE:'api',DATABASE_URL:url('console_rt',passwords.RT),LEAVE_COMMAND_DATABASE_URL:url('console_leave_cmd',passwords.LEAVE_COMMAND),ONTOLOGY_COMMAND_DATABASE_URL:url('console_ontology_cmd',passwords.ONTOLOGY_COMMAND),PLATFORM_FORCE_COMMAND_DATABASE_URL:url('console_platform_force_cmd',passwords.PLATFORM_FORCE_COMMAND),CONSOLE_HTTP_ADDR:`127.0.0.1:${apiPort}`};
 let apiOutput='',apiError;api=spawn(stage+'/console-app',[],{cwd:root,env,stdio:['ignore','pipe','pipe']});api.once('error',error=>apiError=error);api.stdout.on('data',x=>apiOutput+=redact(x));api.stderr.on('data',x=>apiOutput+=redact(x));
 let ready=false;for(let i=0;i<100;i++){if(api.exitCode!==null||apiError)break;try{const r=await fetch(`http://127.0.0.1:${apiPort}/readyz`,{signal:AbortSignal.timeout(1000)});if(r.status===200){ready=true;break;}}catch{}await new Promise(r=>setTimeout(r,100));}
 writeFileSync(evidence+'/api.log',apiOutput);assert.equal(apiError,undefined);assert.equal(ready,true,'actual app readiness: see api.log');assert.equal(api.exitCode,null,'launched API must still own its listener');pass('actual-host-api-readiness-after-finalization');
 assert.deepEqual(sources(),beforeSources,'runtime source drift');assert.equal(run('git',['rev-parse','HEAD']).stdout.trim(),report.source_head);assert.equal(run('git',['status','--porcelain']).stdout.trim(),'');assert.equal(sha(sourceBinary),expectedBinaryHash);assert.equal(sha(stage+'/console-app'),expectedBinaryHash);
 report.status='PASS';
} catch(error){report.status='FAIL';report.reason=String(error);console.error(String(error));process.exitCode=1;}
finally {
 const cleanupErrors=[];
 if(api?.pid && api.exitCode===null && api.signalCode===null){
  const settled=()=>api.exitCode!==null||api.signalCode!==null;
  const wait=async(ms)=>{const end=Date.now()+ms;while(!settled()&&Date.now()<end)await new Promise(r=>setTimeout(r,50));};
  try{api.kill('SIGTERM');await wait(5000);if(!settled()){api.kill('SIGKILL');await wait(5000);}if(!settled())throw new Error('owned API stop unverified');}
  catch(e){cleanupErrors.push(String(e));}
 }
 try{compose(['down','--volumes','--remove-orphans']);}catch(e){cleanupErrors.push(String(e));}
 try{report.remaining=absentResources();assert.deepEqual(report.remaining,{containers:'',volumes:'',networks:''});}catch(e){cleanupErrors.push(String(e));}
 try{
  const inspected=run('docker',['image','inspect',`${project}-operator:local`],{allowFailure:true});
  if(!inspected.error&&inspected.status===0){
   const info=JSON.parse(inspected.stdout)[0];
   if(operatorImageId)assert.equal(info.Id,operatorImageId);
   assert.equal(info.Config.Labels['console.lc07.owner'],project);
   try{run('docker',['image','rm',`${project}-operator:local`]);}
   finally{const after=run('docker',['image','inspect',`${project}-operator:local`],{allowFailure:true});assert.ok(!after.error);assert.notEqual(after.status,0);assert.match(after.output,new RegExp(`No such image: ${project}-operator:local`));}
  }else{assert.ok(!inspected.error);assert.match(inspected.output,new RegExp(`No such image: ${project}-operator:local`));}
 }catch(e){cleanupErrors.push(String(e));}
 report.cleanup=cleanupErrors.length?cleanupErrors:'owned containers, volumes, networks and operator image absent';
 if(cleanupErrors.length){report.status='FAIL';process.exitCode=1;}
 rmSync(stage,{recursive:true,force:true});writeFileSync(evidence+'/result.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({evidence,status:report.status,passed:report.cases.length,cleanup:report.cleanup}));
}
