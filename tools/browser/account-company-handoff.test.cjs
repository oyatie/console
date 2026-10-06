"use strict";
// Machinery-only controls. These never create Console business data or replace real browser observations.
const test=require('node:test'),assert=require('node:assert/strict');
const {expectedDocuments,expectedMutations,validReceipt,validEvidence,deniedProjectionSafe,storageSnapshotSafe,storageControls,storageTrace,validStorageEvidence}=require('./account_company_handoff.cjs');
const ids=['10000000-0000-4000-8000-000000000001','20000000-0000-4000-8000-000000000002','30000000-0000-4000-8000-000000000003','40000000-0000-4000-8000-000000000004','50000000-0000-4000-8000-000000000005','60000000-0000-4000-8000-000000000006'];
function positive(){
 const [a,o,command,org,group,receipt]=ids;
 const r={a,o,shared_reference:a,origin:'https://localhost:12345',command_id:command,org_id:org,company_name:'자료 보존 회사 <연구 & 본사>',company_slug:'handoff-test',browser_version:'153.0.8010.12',observation_failures:0,external_requests:0,
  checkpoints:['A_ENROLLED','O_ENROLLED','O_LOGGED_OUT','O_LOGGED_IN','HEALTHY_HANDOFF_ENTRY','COMPANY_COMMITTED','HANDOFF_REOPENED'],csrf_requests:['O','O'],react_mounts:['A_ACCOUNT','O_SETUP','O_HISTORY','O_HISTORY_REOPENED','A_ACCOUNT_REOPENED']};
 for(const k of ['a_registration','o_registration','o_logout_login','own_reference','default_self','malformed_rejected','company_wire','o_history','o_company_denied','a_company_reopened','a_foreign_history_denied','a_operator_denied','input_not_stored','reflow_320'])r[k]=true;
 r.storage=['A','O'].map((role,i)=>({role,installed_documents:i===0?9:12,control_events:storageControls(),product_operations:0,observation_failures:0,state_preserved:true,forbidden_snapshot_absent:true}));
 r.sent={administrative_account_id:a,command_id:command,group_id:null,name:r.company_name,slug:r.company_slug};
 r.receipt={administrative_account_id:a,group_id:group,org_id:org,original_command_id:command,outcome:'COMMITTED',receipt_id:receipt,replayed:false,result_path:`/account/companies/requests/${command}`};
 r.documents=expectedDocuments(r).map(([role,path,status],i)=>({ordinal:i+1,role,path,status,method:'GET',redirected:false,url_exact:true}));
 r.mutations=expectedMutations().map(([role,path],i)=>({ordinal:i+1,role,path,method:'POST'}));return r;
}
test('complete bounded evidence positive control',()=>assert.equal(validEvidence(positive()),true));
test('every omitted document, mutation, checkpoint and mount is rejected',()=>{
 for(const key of ['documents','mutations','checkpoints','react_mounts'])for(let i=0;i<positive()[key].length;i++){
  const r=positive();r[key].splice(i,1);assert.equal(validEvidence(r),false,`${key}:${i}`);
 }
});
test('duplicate events and missing or extra CSRF admissions are rejected',()=>{
 for(const key of ['documents','mutations','checkpoints','csrf_requests']){
  const duplicate=positive();duplicate[key].push(duplicate[key][0]);assert.equal(validEvidence(duplicate),false);
  const missing=positive();missing[key].pop();assert.equal(validEvidence(missing),false);
 }
});
test('wrong identity, altered receipt and retargeted command are rejected',()=>{
 for(const [key,value] of [['administrative_account_id',ids[1]],['original_command_id',ids[3]],['replayed',true],['outcome','PENDING'],['result_path','/account']]){
  const r=positive();r.receipt[key]=value;assert.equal(validEvidence(r),false);
 }
 const r=positive();r.sent.administrative_account_id=r.o;assert.equal(validReceipt(r.sent,r.receipt,r.a,r.company_name,r.company_slug),false);
});
test('unauthorized document success, redirect, wrong role and extra keys are rejected',()=>{
 for(const [key,value] of [['status',200],['redirected',true],['role','A'],['url_exact',false],['extra',true]]){
  const r=positive();r.documents[14][key]=value;assert.equal(validEvidence(r),false);
 }
});
test('stale runtime, loss of negative guards and observer failures are rejected',()=>{
 for(const [key,value] of [['browser_version','152.0.0.0'],['origin','http://localhost:12345'],['a',ids[1]],['malformed_rejected',false],['o_company_denied',false],['a_operator_denied',false],['observation_failures',1],['external_requests',1],['relay_failure',true]]){
  const r=positive();r[key]=value;assert.equal(validEvidence(r),false);
 }
});

function material(extra={}){return {html:'<main>이 요청을 열 수 없습니다</main>',visible:'이 요청을 열 수 없습니다',text:'이 요청을 열 수 없습니다',attributes:[],...extra};}
test('safe actual-material shape positive control and raw/escaped Company leaks',()=>{
 const name=positive().company_name,escaped=name.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
 assert.equal(deniedProjectionSafe(material(),[name,ids[1]]),true);
 assert.equal(deniedProjectionSafe(material({html:`<main>${name}</main>`}),[name]),false);
 assert.equal(deniedProjectionSafe(material({html:`<main>${escaped}</main>`}),[name]),false);
});
test('decoded visible, hidden text and attribute leaks are rejected',()=>{
 const name=positive().company_name;
 for(const key of ['visible','text'])assert.equal(deniedProjectionSafe(material({[key]:name,html:'&#60;encoded&#62;'}),[name]),false);
 assert.equal(deniedProjectionSafe(material({attributes:[name]}),[name]),false);
 assert.equal(deniedProjectionSafe(material({attributes:[ids[1]]}),[ids[1]]),false);
 assert.equal(deniedProjectionSafe(material({html:42}),[name]),false);
});
test('snapshot guard covers own/operator/shared/malformed refs and business inputs in keys or values',()=>{
 const forbidden=[ids[0],ids[1],'not-an-account',positive().company_name,positive().company_slug];
 assert.equal(storageSnapshotSafe({local:[],session:[]},forbidden),true);
 assert.equal(storageSnapshotSafe({local:[],session:[]},null),false);
 for(const store of ['local','session'])for(const value of forbidden)for(const row of [[value,'unrelated'],['unrelated',value]]){
  const snapshot={local:[],session:[]};snapshot[store]=[row];assert.equal(storageSnapshotSafe(snapshot,forbidden),false);
 }
});
const controlKeys={set:'observer-set',property:'observer-property',clear:'observer-clear'};
function nativeTrace(){
 const trace=storageTrace('https://localhost:12345',controlKeys);
 for(const isLocalStorage of [true,false]){
  const storageId={securityOrigin:'https://localhost:12345',isLocalStorage};
  for(const [operation,purpose] of [['add','set'],['update','set'],['remove','set'],['add','property'],['remove','property'],['add','clear'],['clear',null]])trace.observe(operation,{storageId,key:purpose?controlKeys[purpose]:undefined,newValue:operation==='update'?'changed observer only':'observer only'});
 }
 return trace;
}
function storageEvidence(trace,extra={}){return {role:'A',installed_documents:9,control_events:trace.controls,product_operations:trace.product_operations,observation_failures:trace.observation_failures,state_preserved:true,forbidden_snapshot_absent:true,...extra};}
test('native setter/update/remove/property/delete/clear trace positive control for both stores',()=>{
 const trace=nativeTrace();assert.deepEqual(trace.controls,storageControls());assert.equal(validStorageEvidence(storageEvidence(trace),'A',9),true);
});
test('native reference write followed by clear remains a detected product operation',()=>{
 for(const isLocalStorage of [true,false]){
  const trace=nativeTrace(),storageId={securityOrigin:'https://localhost:12345',isLocalStorage};
  trace.observe('add',{storageId,key:'account-reference',newValue:ids[0]});trace.observe('clear',{storageId});
  assert.equal(trace.product_operations,2);assert.equal(validStorageEvidence(storageEvidence(trace),'A',9),false);
 }
});
test('missing native controls/document coverage and persisted ref despite event omission fail',()=>{
 for(const mutate of [e=>e.control_events.pop(),e=>e.installed_documents--,e=>{e.forbidden_snapshot_absent=false;},e=>{e.state_preserved=false;}]){
  const e=storageEvidence(nativeTrace());mutate(e);assert.equal(validStorageEvidence(e,'A',9),false);
 }
 const r=positive();r.storage[0].product_operations=1;assert.equal(validEvidence(r),false);
 const missing=positive();missing.storage.pop();assert.equal(validEvidence(missing),false);
});
test('foreign origin and malformed native storage identity cannot certify storage safety',()=>{
 for(const storageId of [{securityOrigin:'https://other.invalid',isLocalStorage:true},{securityOrigin:'https://localhost:12345',isLocalStorage:'yes'}]){
  const trace=nativeTrace();trace.observe('add',{storageId,key:'unrelated',newValue:'value'});assert.equal(trace.observation_failures,1);assert.equal(validStorageEvidence(storageEvidence(trace),'A',9),false);
 }
});

test('current Rust JSON-script encoding cannot hide a Company field disclosure',()=>{
 for(const name of [positive().company_name,'직접   "인용" \\ & <본사> ']){
  const json=JSON.stringify({company_name:name}).replaceAll('&','\\u0026').replaceAll('<','\\u003c').replaceAll('>','\\u003e').replaceAll('\u2028','\\u2028').replaceAll('\u2029','\\u2029');
  assert.equal(JSON.parse(json).company_name,name);
  const encoded=material({html:`<script type="application/json">${json}</script>`,visible:'이 요청을 열 수 없습니다',text:json});
  assert.equal(encoded.html.includes(name),false);assert.equal(deniedProjectionSafe(encoded,[name]),false);
 }
});
