"use strict";
// Evidence-corruption controls only; these fixtures never populate Console.
const test=require('node:test'),assert=require('node:assert/strict');
const {expectedDocuments,expectedMutations,validEvidence,storageControls}=require('./account_company_handoff.cjs');
const ids=Array.from({length:7},(_,i)=>`${i+1}0000000-0000-4000-8000-00000000000${i+1}`);
function positive(){
 const [a,b,o,command,org,group,receipt]=ids;
 const r={kind:'REAL_REACT_COMPANY_INFORMATION_MANAGER_CURRENT_V1',a,b,o,shared_reference:a,origin:'https://localhost:12345',command_id:command,org_id:org,company_name:'회사 정보 <연구 & 본사>',company_slug:'handoff-current',browser_version:'153.0.8010.12',observation_failures:0,external_requests:0,
  checkpoints:['A_ENROLLED','B_ENROLLED','O_ENROLLED','O_LOGGED_OUT','O_LOGGED_IN','HEALTHY_HANDOFF_ENTRY','COMPANY_COMMITTED','HANDOFF_REOPENED'],csrf_requests:['O','O'],react_mounts:['A_ACCOUNT','O_SETUP','O_HISTORY','O_HISTORY_REOPENED','A_ACCOUNT_REOPENED','B_ACCOUNT_REOPENED']};
 for(const key of ['a_registration','b_registration','o_registration','o_logout_login','own_reference','default_self','malformed_rejected','company_wire','o_history','o_company_denied','a_company_reopened','a_foreign_history_denied','a_operator_denied','b_company_denied','b_foreign_history_denied','b_operator_denied','input_not_stored','reflow_320'])r[key]=true;
 r.sent={administrative_account_id:a,command_id:command,group_id:null,name:r.company_name,slug:r.company_slug};
 r.receipt={administrative_account_id:a,group_id:group,org_id:org,original_command_id:command,outcome:'COMMITTED',receipt_id:receipt,replayed:false,result_path:`/account/companies/requests/${command}`};
 r.documents=expectedDocuments(r,true).map(([role,path,status],i)=>({ordinal:i+1,role,path,status,method:'GET',redirected:false,url_exact:true}));
 r.mutations=expectedMutations(true).map(([role,path],i)=>({ordinal:i+1,role,path,method:'POST'}));
 r.storage=['A','B','O'].map(role=>({role,installed_documents:r.documents.filter(row=>row.role===role).length,control_events:storageControls(),product_operations:0,observation_failures:0,state_preserved:true,forbidden_snapshot_absent:true}));
 return r;
}
test('three independent UI contexts have exact positive census',()=>{
 const r=positive();assert.equal(r.documents.length,28);assert.equal(r.mutations.length,10);assert.equal(r.checkpoints.length,8);assert.equal(validEvidence(r,true),true);assert.equal(validEvidence(r),false);
 assert.deepEqual(r.storage.map(s=>[s.role,s.installed_documents]),[['A',9],['B',7],['O',12]]);
});
test('every omitted document, mutation, checkpoint, mount and storage role fails',()=>{
 for(const key of ['documents','mutations','checkpoints','react_mounts','storage'])for(let i=0;i<positive()[key].length;i++){const r=positive();r[key].splice(i,1);assert.equal(validEvidence(r,true),false,`${key}:${i}`);}
});
test('duplicate, reordered, wrong-role and unobserved events fail',()=>{
 for(const key of ['documents','mutations','checkpoints','react_mounts','storage']){const r=positive();r[key].push(r[key][0]);assert.equal(validEvidence(r,true),false);const swapped=positive();[swapped[key][0],swapped[key][1]]=[swapped[key][1],swapped[key][0]];assert.equal(validEvidence(swapped,true),false);}
 for(const key of ['role','path','status','method','redirected','url_exact','ordinal']){const r=positive();r.documents[3][key]=null;assert.equal(validEvidence(r,true),false);}
});
test('B cannot substitute A/O, omit denial or become an operator',()=>{
 for(const value of [ids[0],ids[2],'00000000-0000-0000-0000-000000000000']){const r=positive();r.b=value;assert.equal(validEvidence(r,true),false);}
 for(const key of ['b_registration','b_company_denied','b_foreign_history_denied','b_operator_denied']){const r=positive();r[key]=false;assert.equal(validEvidence(r,true),false);}
 for(const i of [25,26,27]){const r=positive();assert.equal(r.documents[i].role,'B');r.documents[i].status=200;assert.equal(validEvidence(r,true),false);}
});
test('B document/storage omissions and transient writes cannot pass',()=>{
 for(const key of ['installed_documents','product_operations','observation_failures','forbidden_snapshot_absent','state_preserved']){const r=positive();r.storage[1][key]=key==='installed_documents'?6:key.endsWith('absent')||key==='state_preserved'?false:1;assert.equal(validEvidence(r,true),false);}
 const r=positive();r.storage[1].control_events.pop();assert.equal(validEvidence(r,true),false);
});
test('missing or extra CSRF and unobserved side effects fail',()=>{
 for(const mutate of [r=>r.csrf_requests.pop(),r=>r.csrf_requests.push('B'),r=>r.mutations.push({ordinal:11,role:'B',method:'POST',path:'/api/v2/companies/enroll'}),r=>{r.external_requests=1;},r=>{r.observation_failures=1;}]){const r=positive();mutate(r);assert.equal(validEvidence(r,true),false);}
});
test('administrator receipt, command and historical guards remain exact',()=>{
 for(const [key,value] of [['administrative_account_id',ids[1]],['original_command_id',ids[4]],['replayed',true],['outcome','PENDING']]){const r=positive();r.receipt[key]=value;assert.equal(validEvidence(r,true),false);}
 for(const key of ['malformed_rejected','o_company_denied','a_foreign_history_denied','a_operator_denied']){const r=positive();r[key]=false;assert.equal(validEvidence(r,true),false);}
});

test('wrong successor identity cannot certify a manager-current journey',()=>{const r=positive();r.kind='REAL_REACT_ACCOUNT_COMPANY_HANDOFF_V1';assert.equal(validEvidence(r,true),false);});
