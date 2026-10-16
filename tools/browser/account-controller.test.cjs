'use strict';
// Evidence machinery controls only; no browser or Console objects are created here.
const assert = require('node:assert/strict'), {test} = require('node:test');
const {NAMES, validRecords, expectedSuffix, validCompleteEvidence} = require('./account_controller_evidence.cjs');
const {expectedDocuments, expectedMutations, storageControls} = require('./account_company_handoff.cjs');
const ids = ['10000000-0000-4000-8000-000000000001','20000000-0000-4000-8000-000000000002',
  '30000000-0000-4000-8000-000000000003','40000000-0000-4000-8000-000000000004',
  '50000000-0000-4000-8000-000000000005','60000000-0000-4000-8000-000000000006'];
function records() {return NAMES.map(name => ({name,asset_bytes_unchanged:true,native_sha256:'1'.repeat(64),react_sha256:'2'.repeat(64),
  react_mounted:['native-first','react-first','freeze-at-csrf'].includes(name),
  exact_original_nodes:!['native-first','react-first','freeze-at-csrf'].includes(name),input_preserved:true,
  late_callbacks:name==='native-first'?3:0,late_callbacks_inert:true,
  csrf_requests:['native-first','react-first','partial-bind-rollback','freeze-at-csrf'].includes(name)?1:0,
  post_requests:name==='freeze-at-csrf'?1:0,detached_render_hits:1,autofill_events:name==='autofill-before-react'?1:0,
  ime_events:name==='ime-before-react'?2:0,rollback_faults:name==='partial-bind-rollback'?1:0,
  capability_calls:name==='capability-denied'?1:0,selected_input_frozen:name==='freeze-at-csrf'}));}
function evidence() {
  const [a,o,command,org,group,receipt] = ids;
  const h={a,o,shared_reference:a,origin:'https://localhost:12345',command_id:command,org_id:org,
    company_name:'실제 브라우저만 생성하는 회사',company_slug:'handoff-test',browser_version:'153.0.8010.12',
    observation_failures:0,external_requests:0,
    checkpoints:['A_ENROLLED','O_ENROLLED','O_LOGGED_OUT','O_LOGGED_IN','HEALTHY_HANDOFF_ENTRY','COMPANY_COMMITTED','HANDOFF_REOPENED'],
    csrf_requests:['O','O'],react_mounts:['A_ACCOUNT','O_SETUP','O_HISTORY','O_HISTORY_REOPENED','A_ACCOUNT_REOPENED']};
  for (const key of ['a_registration','o_registration','o_logout_login','own_reference','default_self','malformed_rejected',
    'company_wire','o_history','o_company_denied','a_company_reopened','a_foreign_history_denied','a_operator_denied','input_not_stored','reflow_320']) h[key]=true;
  h.storage=['A','O'].map((role,i)=>({role,installed_documents:i===0?9:12,control_events:storageControls(),product_operations:0,
    observation_failures:0,state_preserved:true,forbidden_snapshot_absent:true}));
  h.sent={administrative_account_id:a,command_id:command,group_id:null,name:h.company_name,slug:h.company_slug};
  h.receipt={administrative_account_id:a,group_id:group,org_id:org,original_command_id:command,outcome:'COMMITTED',
    receipt_id:receipt,replayed:false,result_path:`/account/companies/requests/${command}`};
  h.documents=expectedDocuments(h).map(([role,path,status],i)=>({ordinal:i+1,role,path,status,method:'GET',redirected:false,url_exact:true}));
  h.mutations=expectedMutations().map(([role,path],i)=>({ordinal:i+1,role,path,method:'POST'}));
  const next='70000000-0000-4000-8000-000000000007', company='80000000-0000-4000-8000-000000000008';
  const r=structuredClone(h); r.handoff=h; r.controller={records:records(),operator_storage_operations:0,storage_positive_controls:4,
    sent:{...h.sent,command_id:next,name:'고정된 관리자 회사 <원본 & 입력>',slug:`controller-${o.replaceAll('-','')}`},
    receipt:{...h.receipt,original_command_id:next,org_id:company,result_path:`/account/companies/requests/${next}`}};
  r.documents.push(...expectedSuffix(r).map(([role,path,status],i)=>({ordinal:22+i,role,path,status,method:'GET',redirected:false,url_exact:true})));
  r.mutations.push({ordinal:9,role:'O',method:'POST',path:'/api/v2/companies/enroll'}); r.csrf_requests.push('O','O','O','O');
  r.checkpoints.push(...NAMES.flatMap(name=>name==='freeze-at-csrf'?['CONTROLLER_READY','CONTROLLER_PRE_DISPATCH','CONTROLLER_COMMITTED','CONTROLLER_CHECKED']:['CONTROLLER_READY','CONTROLLER_CHECKED']));
  return r;
}
test('complete additive evidence positive control',()=>assert.equal(validCompleteEvidence(evidence()),true));
for (const name of NAMES) test(`omitted ${name} control rejected`,()=>{
  const r=evidence(); r.controller.records=r.controller.records.filter(row=>row.name!==name); assert.equal(validCompleteEvidence(r),false);
});
for (const name of NAMES) for (const flag of ['asset_bytes_unchanged','input_preserved','late_callbacks_inert'])
  test(`${name} false ${flag} rejected`,()=>{const r=evidence();r.controller.records.find(row=>row.name===name)[flag]=false;assert.equal(validCompleteEvidence(r),false);});
for (const [name,key,bad] of [['native-first','late_callbacks',0],['native-first','csrf_requests',2],['react-first','detached_render_hits',0],
  ['autofill-before-react','autofill_events',0],['ime-before-react','ime_events',1],['partial-bind-rollback','rollback_faults',0],
  ['capability-denied','capability_calls',0],['freeze-at-csrf','selected_input_frozen',false],['freeze-at-csrf','post_requests',2]])
  test(`${name} missing/duplicate ${key} rejected`,()=>{const r=evidence();r.controller.records.find(row=>row.name===name)[key]=bad;assert.equal(validCompleteEvidence(r),false);});
for (const name of NAMES) test(`${name} omitted field rejected`,()=>{const r=records();delete r.find(row=>row.name===name).input_preserved;assert.equal(validRecords(r),false);});
for (const key of ['documents','mutations','csrf_requests','checkpoints']) test(`additive ${key} omissions/duplicates rejected`,()=>{
  for (const change of [rows=>rows.pop(),rows=>rows.push(rows.at(-1))]) {const r=evidence();change(r[key]);assert.equal(validCompleteEvidence(r),false);}
});
test('wrong selected Account under same frozen command rejected',()=>{const r=evidence();r.controller.sent.administrative_account_id=r.handoff.o;assert.equal(validCompleteEvidence(r),false);});
test('original handoff cannot be weakened by additive evidence',()=>{const r=evidence();r.handoff.documents.pop();assert.equal(validCompleteEvidence(r),false);});
test('runtime substitution, storage writes and source errors rejected',()=>{
  for (const change of [r=>{r.controller.records[1].react_sha256='3'.repeat(64);},r=>{r.controller.operator_storage_operations=1;},
    r=>{r.observation_failures=1;},r=>{r.external_requests=1;}]) {const r=evidence();change(r);assert.equal(validCompleteEvidence(r),false);}
});
