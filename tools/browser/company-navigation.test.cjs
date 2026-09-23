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

// Company layout evidence must observe actual, nonempty, separated controls.
const {validCompanyLayout,completeCompanyWorkspace}=require(process.env.CONSOLE_NAVIGATION_DRIVER||'./company.cjs');
function layout(){return {workspace:true,stylesheet:true,display:'block',viewport:320,scrollWidth:320,rects:[{x:16,y:200,width:120,height:44},{x:16,y:256,width:120,height:44}]};}
test('Company geometry positive control',()=>assert.equal(validCompanyLayout(layout(),2,320),true));
test('Company desktop geometry positive control',()=>assert.equal(validCompanyLayout({...layout(),display:'grid',viewport:1440,scrollWidth:1440},2,1440),true));
for(const key of ['workspace','stylesheet','display','viewport','scrollWidth','rects'])test(`Company geometry missing ${key} refused`,()=>{const v=layout();delete v[key];assert.equal(validCompanyLayout(v,2,320),false);});
for(const [key,values] of [['workspace',[false,'true']],['stylesheet',[false,'true']],['display',['none','flex','grid']],['viewport',[319,1440]],['scrollWidth',[NaN,Infinity,319,322]],['rects',[[],null]]])for(const value of values)test(`Company invalid ${key}:${JSON.stringify(value)} refused`,()=>assert.equal(validCompanyLayout({...layout(),[key]:value},2,320),false));
for(const key of ['x','y','width','height'])for(const value of [undefined,NaN,Infinity,-1])test(`Company invalid rectangle ${key}:${String(value)} refused`,()=>{const v=layout();v.rects[0][key]=value;assert.equal(validCompanyLayout(v,2,320),false);});
for(const [key,value] of [['width',0],['height',0],['width',43],['height',43],['x',250]])test(`Company undersized/offpage ${key}:${value} refused`,()=>{const v=layout();v.rects[0][key]=value;assert.equal(validCompanyLayout(v,2,320),false);});
for(const y of [200,220,244,251])test(`Company overlapping/touching/close targets ${y} refused`,()=>{const v=layout();v.rects[1].y=y;assert.equal(validCompanyLayout(v,2,320),false);});
test('Company exact8px target gap accepted',()=>{const v=layout();v.rects[1].y=252;assert.equal(validCompanyLayout(v,2,320),true);});
test('Company horizontal target gap accepted',()=>{const v=layout();v.rects[1]={...v.rects[0],x:144};assert.equal(validCompanyLayout(v,2,320),true);});
for(const count of [0,1,3,NaN])test(`Company wrong cardinality ${count} refused`,()=>assert.equal(validCompanyLayout(layout(),count,320),false));
const companyPositive=()=>({policy_entry:true,company_layout_320:true,company_layout_desktop:true,payroll_company_layout_320:true,payroll_company_layout_desktop:true});
test('Company completion positive control',()=>assert.equal(completeCompanyWorkspace(companyPositive()),true));
for(const key of ['company_layout_320','company_layout_desktop','payroll_company_layout_320','payroll_company_layout_desktop']){
 test(`Company missing observation ${key} refused`,()=>{const v=companyPositive();delete v[key];assert.equal(completeCompanyWorkspace(v),false);});
 for(const value of [false,null,1,'true'])test(`Company false observation ${key}:${value} refused`,()=>assert.equal(completeCompanyWorkspace({...companyPositive(),[key]:value}),false));
}
