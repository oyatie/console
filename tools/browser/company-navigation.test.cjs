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

// Mandatory native policy/Payroll header classifier controls. No product fixtures.
const {expectedNativeHeaders,completeNativeHeaders}=require(process.env.CONSOLE_NAVIGATION_DRIVER||'./company.cjs');
// Classifier-only positive record; never published as browser acceptance evidence.
function headerWitness(expected) {
  return {kind: 'REAL_NATIVE_HEADER_BROWSER_CHECK', phase: expected.phase, url: expected.url,
    allowed_paths: [...expected.paths].sort(), current_path: expected.currentPath ?? null,
    payroll_path: expected.payrollPath ?? null, denied_prefixes: [...(expected.deniedPrefixes ?? [])].sort(),
    widths: [320, 680, 681, 1280].map(width => ({width, header_height: 90, main_top: 90,
      title_top: 160, no_overflow: true, open_no_overflow: width <= 680 ? true : null,
      routes_exact: true, current_exact: true, inactive_hidden: true, visible_landmarks_unique: true, denied_hrefs_absent: true})),
    enter_opened: true, space_closed: true, closed_focus_safe: true, resize_focus_safe: true,
    values_preserved: true, location_preserved: true, no_product_script: true,
    unique_ids: true, network_requests: 0};
}

function headerEvidence(mode = 'policy') {
  const r = {policy_entry: mode !== 'company', people_entry: mode === 'people', org_id: '00000000-0000-4000-8000-000000000001', header_origin: 'https://localhost:1234',
    policy: {mutations: [2, 3, 4].map(n => ({command_id: '00000000-0000-4000-8000-' + String(n).padStart(12, '0')}))}};
  r.native_headers = expectedNativeHeaders(r).map(headerWitness);
  if (r.people_entry) r.people = {header_origin: r.header_origin};
  return r;
}
test('policy Payroll mandatory header positive control', () => assert.equal(completeNativeHeaders(headerEvidence()), true));
test('Company-only mode without native header observations is refused', () => assert.equal(completeNativeHeaders({policy_entry: false}), false));
test('policy Payroll exact route and current expectations', () => {
  const r = headerEvidence(), w = '/companies/' + r.org_id, p = w + '/payroll';
  const e = expectedNativeHeaders(r);
  assert.deepEqual(e.map(row => row.phase), ['COMPANY_HEADER_FIRST', 'POLICY_HEADER_PREFLIGHT', 'CATALOG_INSTALLED',
    'GRANT_COMMITTED', 'GRANT_REOPENED', 'PAYROLL_COMPANY_HEADER', 'PAYROLL_READ', 'PAYROLL_REOPENED', 'REVOKE_COMMITTED', 'REVOKE_REOPENED', 'REVOKED_COMPANY_HEADER']);
  assert.deepEqual(e.map(row => row.currentPath ?? null), [w, null, null, null, null, w, p, p, null, null, w]);
  assert.deepEqual(e[0].paths, ['/account', w, w + '/policy']);
  assert.deepEqual(e[3].paths, ['/account', w, w + '/policy', p]);
  assert.deepEqual(e[6].paths, ['/account', p]);
  assert.deepEqual(e[8].paths, ['/account', w, w + '/policy']);
});
for (let i = 0; i < expectedNativeHeaders(headerEvidence()).length; i++) {
  test(`missing mandatory policy Payroll header ${i} refused`, () => {
    const r = headerEvidence(); r.native_headers.splice(i, 1); assert.equal(completeNativeHeaders(r), false);
  });
  test(`unpreserved form at policy Payroll header ${i} refused`, () => {
    const r = headerEvidence(); r.native_headers[i].values_preserved = false; assert.equal(completeNativeHeaders(r), false);
  });
  test(`unexpected request at policy Payroll header ${i} refused`, () => {
    const r = headerEvidence(); r.native_headers[i].network_requests = 1; assert.equal(completeNativeHeaders(r), false);
  });
  test(`wrong policy Payroll header ${i} location refused`, () => {
    const r = headerEvidence(); r.native_headers[i].url += '/wrong'; assert.equal(completeNativeHeaders(r), false);
  });
  test(`unpermitted policy Payroll header ${i} route refused`, () => {
    const r = headerEvidence(); r.native_headers[i].allowed_paths.push('/companies/foreign/payroll'); assert.equal(completeNativeHeaders(r), false);
  });
}
test('duplicate policy Payroll header refused', () => { const r = headerEvidence(); r.native_headers.push(r.native_headers[0]); assert.equal(completeNativeHeaders(r), false); });
test('reordered policy Payroll header refused', () => { const r = headerEvidence(); [r.native_headers[0], r.native_headers[1]] = [r.native_headers[1], r.native_headers[0]]; assert.equal(completeNativeHeaders(r), false); });
test('missing entire policy Payroll header history refused', () => { const r = headerEvidence(); delete r.native_headers; assert.equal(completeNativeHeaders(r), false); });
test('People and policy origin disagreement refused', () => { const r = headerEvidence(); r.people_entry = true; r.people = {header_origin: 'https://localhost:4321'}; assert.equal(completeNativeHeaders(r), false); });

// Same first mounted Company observation is mandatory in every real launch mode.
for (const mode of ['company', 'policy', 'people']) {
  test(`${mode} first Company header positive control`, () => assert.equal(completeNativeHeaders(headerEvidence(mode)), true));
  for (const [label, change] of [
    ['missing first', r => r.native_headers.shift()],
    ['missing history', r => delete r.native_headers],
    ['duplicate first', r => r.native_headers.splice(1, 0, r.native_headers[0])],
    ['reordered first', r => [r.native_headers[0], r.native_headers[1]] = [r.native_headers[1], r.native_headers[0]]],
    ['wrong Company URL', r => r.native_headers[0].url += '/wrong'],
    ['wrong current route', r => r.native_headers[0].current_path += '/policy'],
    ['stale Payroll rights', r => r.native_headers[0].allowed_paths.push(`/companies/${r.org_id}/payroll`)],
    ['stale People rights', r => r.native_headers[0].allowed_paths.push(`/companies/${r.org_id}/people`)],
    ['missing denied-route census', r => delete r.native_headers[0].denied_prefixes],
    ['duplicate visible landmarks', r => r.native_headers[0].widths[0].visible_landmarks_unique = false],
    ['hidden denied anchor', r => r.native_headers[0].widths[0].denied_hrefs_absent = false],
  ]) test(`${mode} ${label} Company header refused`, () => {
    const r = headerEvidence(mode); change(r); assert.equal(completeNativeHeaders(r), false);
  });
}
const {completeDocumentsOriginal, observeDocuments} = require('./company.cjs');
function companyDocuments() {
  const command = '00000000-0000-4000-8000-000000000090', company = '00000000-0000-4000-8000-000000000001';
  const w = '/companies/' + company, saved = '/account/companies/requests/' + command;
  return {command_id: command, org_id: company, policy_entry: false, document_failures: 0,
    documents: ['/', '/account/register', '/account', '/account', '/account/companies/new', saved, saved,
      w, w + '/policy', w + '/policy', w, saved].map((path, i) =>
        ({method: 'GET', path, status: 200, redirected: false, ordinal: i + 1, url_exact: true}))};
}
test('Company Policy-root open/reload/return exact twelve-document history', () => assert.equal(completeDocumentsOriginal(companyDocuments()), true));
for (let i = 0; i < 12; i++) {
  test(`Company omitted document ${i} refused`, () => {const r = companyDocuments(); r.documents.splice(i, 1); assert.equal(completeDocumentsOriginal(r), false);});
  test(`Company duplicated document ${i} refused`, () => {const r = companyDocuments(); r.documents.splice(i, 0, r.documents[i]); assert.equal(completeDocumentsOriginal(r), false);});
}
for (const index of [8, 9, 10]) for (const [key, wrong] of [['method', 'POST'], ['path', '/companies/wrong/policy'], ['status', 404], ['redirected', true], ['url_exact', false]]) {
  test(`Company Policy-root document ${index} wrong ${key} refused`, () => {const r = companyDocuments(); r.documents[index][key] = wrong; assert.equal(completeDocumentsOriginal(r), false);});
}
test('Company Policy-root reordered return refused', () => {const r = companyDocuments(); [r.documents[9], r.documents[10]] = [r.documents[10], r.documents[9]]; assert.equal(completeDocumentsOriginal(r), false);});
test('Company observer accepts actual Policy-root and rejects foreign Policy-root', () => {
  const {EventEmitter} = require('node:events');
  for (const foreign of [false, true]) {
    const r = companyDocuments(); r.documents = [];
    const page = new EventEmitter(), frame = {}; page.mainFrame = () => frame;
    observeDocuments(page, 'https://localhost:1234', r);
    const route = '/companies/' + (foreign ? '00000000-0000-4000-8000-000000000099' : r.org_id) + '/policy';
    const request = {frame: () => frame, resourceType: () => 'document', isNavigationRequest: () => true,
      url: () => 'https://localhost:1234' + route, method: () => 'GET', redirectedFrom: () => null};
    page.emit('request', request);
    assert.equal(r.documents[0].path, foreign ? '<unexpected>' : route);
  }
});
