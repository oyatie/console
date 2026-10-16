'use strict';
const assert=require('node:assert/strict');
const {completeObservations}=require('./company-preview.cjs');
const paths=['/','/account/register','/account','/account','/account/companies/new'];
const mutations=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish'];
const flags=['reflow_root_320','reflow_register_320','reflow_account_320','reflow_setup_320','keyboard_skip','keyboard_registration','keyboard_terms','keyboard_registration_submit','keyboard_company_entry','preview_has_no_form','preview_enter_preserved','business_input_not_stored','registration_wire','resident','cookie_security','literal_secret_absent'];
const base={documents:paths.map((path,i)=>({method:'GET',ordinal:i+1,path,redirected:false,status:200,url_exact:true})),document_failures:0,mutations:mutations.map((path,i)=>({method:'POST',ordinal:i+1,path})),posts:Object.fromEntries(mutations.map(p=>[p,1])),unexpected_mutations:0,browser_version:'151.0.7922.34',root_status:200,external_requests:0,checkpoints:['ENROLLED','PREVIEW_PRESERVED'],preview_enters:['name','slug'].map(field=>({field,no_requests:true,url_unchanged:true,history_unchanged:true,inputs_preserved:true})),...Object.fromEntries(flags.map(k=>[k,true]))};
assert.equal(completeObservations(base),true);
const corruptions=[r=>r.documents.pop(),r=>r.documents[4].path='/account',r=>r.documents[4].status=404,r=>r.documents[4].redirected=true,r=>r.mutations.push({ordinal:3,method:'POST',path:'/api/v2/companies/enroll'}),r=>r.preview_enters.pop(),r=>r.preview_enters[0].no_requests=false,r=>r.preview_enters[1].url_unchanged=false,r=>r.preview_enters[0].history_unchanged=false,r=>r.preview_enters[1].inputs_preserved=false,r=>r.preview_has_no_form=false,r=>r.checkpoints.pop()];
for(const change of corruptions){const candidate=structuredClone(base);change(candidate);assert.equal(completeObservations(candidate),false);}
console.log(JSON.stringify({kind:'preview-evidence-machinery-controls-only',positive:1,corruptions_detected:corruptions.length,browser_runs:0,database_runs:0}));
