'use strict';
// Machinery-only controls, never browser/workflow acceptance evidence.
const test=require('node:test');const assert=require('node:assert/strict');
const {exactVisibleLink,completeNavigation,completePolicyPlan}=require(process.env.CONSOLE_NAVIGATION_DRIVER||'./company.cjs');
const flags=['payroll_nav_before_grant_absent','payroll_receipt_link','payroll_workspace_link','keyboard_payroll_navigation','reflow_payroll_workspace_320','payroll_nav_after_revoke_absent'];
const positive=()=>Object.fromEntries(flags.map(key=>[key,true]));
test('navigation observation positive control',()=>assert.equal(completeNavigation(positive()),true));
for(const key of flags){
 test(`missing ${key} is refused`,()=>{const value=positive();delete value[key];assert.equal(completeNavigation(value),false);});
 for(const wrong of [false,null,'true',1])test(`non-true ${key}:${JSON.stringify(wrong)} is refused`,()=>assert.equal(completeNavigation({...positive(),[key]:wrong}),false));
}
function history(){
 const company='/companies/c',b=company+'/policy/payroll-read',p=company+'/payroll';
 const install=b+'/requests/install/i',grant=b+'/requests/grant/g',revoke=b+'/requests/revoke/r';
 const mutations=[b+'/catalog',b+'/grants',b+'/grants/a/revoke'];
 const get=(path,status=200)=>({method:'GET',path,status,redirected:false});
 const post=path=>({method:'POST',path,status:303,redirected:false});
 const redirected=path=>({method:'GET',path,status:200,redirected:true});
 return {org_id:'c',policy:{mutations:[{command_id:'i'},{command_id:'g'},{command_id:'r'}],checkpoints:[{},{assignment_id:'a'}]},policy_expected_mutations:mutations,
  policy_documents:[post(mutations[0]),redirected(install),get(b+'/grant'),post(mutations[1]),redirected(grant),get(grant),get(company),get(p),get(p),get(grant),get(b+'/revoke'),post(mutations[2]),redirected(revoke),get(revoke),get(p,404),get('/account'),get(company)]};
}
test('exact navigation document history positive control',()=>assert.equal(completePolicyPlan(history()),true));
for(let index=0;index<17;index++)test(`omitted document ${index} is refused`,()=>{const r=history();r.policy_documents.splice(index,1);assert.equal(completePolicyPlan(r),false);});
for(const [key,value] of [['method','POST'],['path','/companies/wrong'],['status',404],['redirected',true]])test(`wrong workspace document ${key} is refused`,()=>{const r=history();r.policy_documents[6][key]=value;assert.equal(completePolicyPlan(r),false);});
test('unobserved extra document is refused',()=>{const r=history();r.policy_documents.splice(6,0,{...r.policy_documents[6]});assert.equal(completePolicyPlan(r),false);});
test('changed mutation plan is refused',()=>{const r=history();r.policy_expected_mutations[1]+='/wrong';assert.equal(completePolicyPlan(r),false);});

const link=(count=1,visible=true,href='/payroll')=>({count:async()=>count,isVisible:async()=>visible,getAttribute:async key=>{assert.equal(key,'href');return href;}});
test('link guard positive control',async()=>assert.equal(await exactVisibleLink(link(),'/payroll'),true));
for(const [label,value] of [['absent',link(0)],['duplicated',link(2)],['hidden',link(1,false)],['wrong target',link(1,true,'/wrong')]])test(`link guard rejects ${label}`,async()=>assert.equal(await exactVisibleLink(value,'/payroll'),false));
test('reordered navigation documents refused',()=>{const r=history();[r.policy_documents[6],r.policy_documents[7]]=[r.policy_documents[7],r.policy_documents[6]];assert.equal(completePolicyPlan(r),false);});
