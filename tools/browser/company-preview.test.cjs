'use strict';
// Machinery controls only; real browser/auth/DB evidence comes from the native leaf.
const assert=require('node:assert/strict');
const {test}=require('node:test');
const {EventEmitter}=require('node:events');
const {completeObservations,leafStatus,observeMutations,completeMutations}=require('./company-preview.cjs');
const account='11111111-1111-4111-8111-111111111111';
const paths=['/','/account/register','/account','/account','/account/companies/new'];
const posts=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish'];
function good(){return {
 contract_revision:'company-invalid-enter-v2',account_id:account,
 company_form:{count:1,account_id:account,submit_count:1,submit_type:'submit',submit_enabled:true,unexpected_actions:0},
 browser_version:'153.0.8010.12',document_failures:0,
 documents:paths.map((path,index)=>({ordinal:index+1,method:'GET',path,redirected:false,status:200,url_exact:true})),
 unexpected_mutations:0,mutations:posts.map((path,index)=>({ordinal:index+1,method:'POST',path})),posts:Object.fromEntries(posts.map(path=>[path,1])),
 root_status:200,registration_wire:true,resident:true,cookie_security:true,literal_secret_absent:true,external_requests:0,
 checkpoints:['ENROLLED','PREVIEW_PRESERVED'],
 reflow_root_320:true,reflow_register_320:true,reflow_account_320:true,reflow_setup_320:true,
 keyboard_skip:true,keyboard_registration:true,keyboard_terms:true,keyboard_registration_submit:true,keyboard_company_entry:true,
 company_form_ready:true,valid_correction_ready:true,preview_enter_preserved:true,business_input_not_stored:true,
 preview_enters:['name','slug'].map(field=>({field,no_requests:true,url_unchanged:true,history_unchanged:true,inputs_preserved:true,invalid_before:true,invalid_after:true,validation_message:true,submit_enabled:true,corrected_valid:true,inputs_not_stored:true})),
 cleanup:{confirmed:true},
};}
test('v2 requires both invalid Enter cases and complete unchanged observations',()=>{
 assert.equal(completeObservations(good()),true);assert.equal(leafStatus(good()),'BROWSER_LEAF_PASSED');
 for(const key of ['contract_revision','account_id','company_form','company_form_ready','valid_correction_ready','preview_enter_preserved','business_input_not_stored']){
  const r=good();delete r[key];assert.equal(completeObservations(r),false,key);
 }
 for(const field of ['name','slug']){
  const omitted=good();omitted.preview_enters=omitted.preview_enters.filter(row=>row.field!==field);assert.equal(completeObservations(omitted),false);
  for(const key of ['no_requests','url_unchanged','history_unchanged','inputs_preserved','invalid_before','invalid_after','validation_message','submit_enabled','corrected_valid','inputs_not_stored']){
   const r=good();delete r.preview_enters.find(row=>row.field===field)[key];assert.equal(completeObservations(r),false,field+'/'+key);
  }
 }
 const duplicate=good();duplicate.preview_enters[1]=duplicate.preview_enters[0];assert.equal(completeObservations(duplicate),false);
 for(const change of [r=>r.documents.pop(),r=>r.documents[4].status=503,r=>r.documents[4].redirected=true,r=>r.mutations.pop(),r=>r.unexpected_mutations++,r=>r.cleanup.confirmed=false]){
  const r=good();change(r);assert.equal(leafStatus(r),'BROWSER_LEAF_FAILED');
 }
});
test('a forged ready flag cannot cover missing disabled foreign or wrong form controls',()=>{
 for(const [key,value] of [['count',0],['count',2],['account_id','22222222-2222-4222-8222-222222222222'],['submit_count',0],['submit_count',2],['submit_type','button'],['submit_enabled',false],['unexpected_actions',1]]){
  const r=good();r.company_form[key]=value;r.company_form_ready=true;assert.equal(completeObservations(r),false,key);
 }
 const old=good();delete old.company_form;old.preview_has_no_form=true;assert.equal(completeObservations(old),false);
});
test('actual request observer rejects Company commands duplicates and omitted registration effects',()=>{
 const origin='https://localhost:12345';
 function observed(sequence){const context=new EventEmitter();const r={unexpected_mutations:0,mutations:[],posts:Object.fromEntries(posts.map(path=>[path,0]))};observeMutations(context,origin,r);for(const [method,path] of sequence)context.emit('request',{method:()=>method,url:()=>origin+path});return r;}
 const original=posts.map(path=>['POST',path]);assert.equal(completeMutations(observed(original)),true);
 for(const sequence of [original.slice(0,1),[...original,['POST','/api/v2/companies/enroll']],[...original,original[0]],[...original,['DELETE','/api/v2/companies/enroll']]])assert.equal(completeMutations(observed(sequence)),false);
});
