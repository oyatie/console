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

// Classifier-only Policy task controls. These records never stand in for the
// mounted browser, current policy or independent database-effect census.
const {COPY,validTaskEvidence,taskEvidenceIssues,taskIssues,collectTask}=require('./policy_journey.cjs');
function taskHistory(sameAccount=true) {
  const uuid=n=>`00000000-0000-4000-8000-${String(n).padStart(12,'0')}`;
  const e={company:uuid(1),group:uuid(2),recipient:uuid(3),operator:uuid(sameAccount?3:4),company_name:'연구 <회사 & 본사>',origin:'https://localhost:1234',
    mutations:[5,6,7].map(n=>({command_id:uuid(n)}))};
  const b=e.origin+'/companies/'+e.company+'/policy/payroll-read';
  const result=(kind,i)=>b+'/requests/'+kind+'/'+e.mutations[i].command_id;
  const sites=[['INSTALL_PREFLIGHT',b+'/install','NONE'],['CATALOG_INSTALLED',result('install',0),'NONE'],['GRANT_PREFLIGHT',b+'/grant','NONE'],
    ['GRANT_COMMITTED',result('grant',1),'ACTIVE'],['GRANT_REOPENED',result('grant',1),'ACTIVE'],['GRANT_RETURNED',result('grant',1),'ACTIVE'],
    ['REVOKE_PREFLIGHT',b+'/revoke','ACTIVE'],['REVOKE_COMMITTED',result('revoke',2),'REVOKED'],['REVOKE_REOPENED',result('revoke',2),'REVOKED']];
  e.ui_observations=sites.map(([site,url,current_state])=>({site,url,current_state,heading:COPY.heading,description:COPY.consequence,limit:COPY.scope,
    primary:[['회사',e.company_name],['권한 대상','회사 등록 시 지정된 관리 계정'],['작업 담당','현재 로그인 계정']],
    heading_visible:true,description_visible:true,limit_visible:true,consequence_text:[COPY.heading,COPY.consequence,COPY.scope].join(' '),primary_visible:true,
    primary_prose:['회사 등록 시 지정된 관리 계정에 한해 연결할 수 있습니다.',...(sameAccount?['현재 로그인 계정이 권한 대상입니다']:[])],
    details_count:1,details_native:true,details_title:'회사·계정 식별 정보',initially_closed:true,summary_tab_index:0,summary_visible:true,
    identifiers:[['회사',e.company],['그룹',e.group],['권한 대상',e.recipient],['현재 담당 계정',e.operator]],
    identity_attributes:[['data-policy-company',e.company],['data-policy-recipient',e.recipient],['data-policy-operator',e.operator]],
    attributes_in_details:true,identifiers_hidden:true,keyboard_opened:true,keyboard_closed:true,identifiers_visible:true,
    tab_discovered:true,focus_visible:true,network_requests:0,values_preserved:true,location_preserved:true,unique_ids:true,no_product_script:true}));
  for(const row of e.ui_observations)row.primary_text=row.primary.flat().concat(row.primary_prose).join(' ');
  return e;
}
for(const same of [true,false])test(`Policy task context positive, same Account=${same}`,()=>assert.equal(validTaskEvidence(taskHistory(same)),true));
test('Policy same-Account relationship is optional, never a human inference',()=>{
  const e=taskHistory();for(const row of e.ui_observations){row.primary_prose.pop();row.primary_text=row.primary.flat().concat(row.primary_prose).join(' ');}assert.equal(validTaskEvidence(e),true);
});
for(let i=0;i<9;i++) {
  test(`Policy omitted task observation ${i} refused`,()=>{const e=taskHistory();e.ui_observations.splice(i,1);assert.equal(validTaskEvidence(e),false);});
  test(`Policy duplicate task observation ${i} refused`,()=>{const e=taskHistory();e.ui_observations.splice(i,0,e.ui_observations[i]);assert.equal(validTaskEvidence(e),false);});
  test(`Policy wrong task URL/state ${i} refused`,()=>{
    for(const key of ['url','current_state']){const e=taskHistory();e.ui_observations[i][key]+='-wrong';assert.equal(validTaskEvidence(e),false);}
  });
  test(`Policy unconditional task disclosure ${i} refused`,()=>{
    const e=taskHistory();e.ui_observations[i].description='이 계정은 선택한 회사의 급여 목록과 목록에 포함된 모든 항목을 볼 수 있습니다. 근태 마감 증빙 전체와 결정 사유도 포함됩니다.';
    assert.equal(validTaskEvidence(e),false);assert.ok(taskEvidenceIssues(e).includes(e.ui_observations[i].site+':CONDITIONAL_SCOPE'));
  });
}
test('Policy reordered task history refused',()=>{const e=taskHistory();[e.ui_observations[0],e.ui_observations[1]]=[e.ui_observations[1],e.ui_observations[0]];assert.equal(validTaskEvidence(e),false);});
test('Policy absent task history refused',()=>{const e=taskHistory();delete e.ui_observations;assert.deepEqual(taskEvidenceIssues(e),['TASK_HISTORY_INCOMPLETE']);});
for(const key of ['heading_visible','description_visible','limit_visible','consequence_text','primary_visible','primary_text','tab_discovered','focus_visible','summary_tab_index','summary_visible','heading','description','limit','primary','primary_prose','details_count','details_native','details_title','initially_closed','identifiers','identity_attributes','attributes_in_details','identifiers_hidden','keyboard_opened','keyboard_closed','identifiers_visible','network_requests','values_preserved','location_preserved','unique_ids','no_product_script']) {
  test(`Policy missing task field ${key} refused`,()=>{const e=taskHistory();delete e.ui_observations[8][key];assert.equal(validTaskEvidence(e),false);});
}
for(let i=0;i<4;i++)for(const change of ['missing','duplicate','replaced'])test(`Policy ${change} identifier role ${i} refused`,()=>{
  const e=taskHistory(),rows=e.ui_observations[8].identifiers;
  if(change==='missing')rows.splice(i,1);else if(change==='duplicate')rows.splice(i,0,rows[i]);else rows[i]=[rows[i][0],'00000000-0000-4000-8000-000000000099'];
  assert.equal(validTaskEvidence(e),false);
});
for(const key of ['company','group','recipient','operator'])test(`Policy invalid canonical ${key} cannot support relationship claims`,()=>{
  for(const value of ['',null,'00000000-0000-0000-0000-000000000000']){const e=taskHistory();e[key]=value;assert.equal(validTaskEvidence(e),false);}
});
for(const [key,value] of [['heading_visible',false],['description_visible',false],['limit_visible',false],['primary_visible',false],['tab_discovered',false],['focus_visible',false],['summary_tab_index',-1],['summary_visible',false],['details_count',2],['details_native',false],['initially_closed',false],['identifiers_hidden',false],['attributes_in_details',false],['keyboard_opened',false],['keyboard_closed',false],['identifiers_visible',false],['network_requests',1],['network_requests','0'],['values_preserved',false],['location_preserved',false],['unique_ids',false],['no_product_script',false]])test(`Policy corrupt ${key}:${value} refused`,()=>{
  const e=taskHistory();e.ui_observations[8][key]=value;assert.equal(validTaskEvidence(e),false);
});
test('Policy different Accounts cannot claim the current Account is the target',()=>{
  const e=taskHistory(false);e.ui_observations[8].primary_prose.push('현재 로그인 계정이 권한 대상입니다');assert.equal(validTaskEvidence(e),false);
});
test('Policy Account equality cannot claim natural-person equality',()=>{
  const e=taskHistory();e.ui_observations[8].primary_prose.push('두 계정은 같은 사람입니다');assert.equal(validTaskEvidence(e),false);
});
test('Policy invented target name and prominent raw identities refused',()=>{
  for(const value of ['김관리자','invented@example.com',taskHistory().recipient]){const e=taskHistory();e.ui_observations[8].primary[1][1]=value;assert.equal(validTaskEvidence(e),false);}
});
test('Policy duplicated identity attribute cannot disappear from its census',()=>{
  const e=taskHistory();e.ui_observations[8].identity_attributes.push(e.ui_observations[8].identity_attributes[0]);assert.equal(validTaskEvidence(e),false);
});

test('Policy DOM collector controls under pinned Chromium, never business acceptance',async t=>{
  const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
  const driver=process.env.CONSOLE_COMPANY_BROWSER_DRIVER;
  assert.equal(typeof driver,'string','source-bound browser stage prerequisite required');assert.equal(path.isAbsolute(driver),true);
  const stage=path.dirname(driver),runtime=path.join(stage,'runtime');
  const pin={'darwin-arm64':['chrome-headless-shell-mac-arm64/chrome-headless-shell','a0bfe7b4da4787b66058477d696cd1d09065d25f06a548947722b9af77ee8282'],
    'linux-x64':['chrome-headless-shell-linux64/chrome-headless-shell','ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e']}[process.platform+'-'+process.arch];
  assert.ok(pin,'reviewed Chromium platform required');
  const executable=path.join(runtime,'browser',pin[0]);
  assert.equal(crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex'),pin[1]);
  assert.equal(require(path.join(runtime,'node_modules/playwright/package.json')).version,'1.63.0');
  assert.equal(require(path.join(runtime,'node_modules/playwright-core/package.json')).version,'1.63.0');
  const {chromium}=require(path.join(runtime,'node_modules/playwright'));
  const browser=await chromium.launch({headless:true,executablePath:executable});t.after(()=>browser.close());
  const page=await browser.newPage();
  const scope=taskHistory(),escape=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
  // Deliberately synthetic DOM fixtures test the collector only, in a separate
  // browser. No Company route, Account, source, command or database is seeded.
  const fixture=change=>`<section class="panel"><h2>대상과 업무 범위</h2><div class="policy-panel-body"><dl>
    <dt>회사</dt><dd>${escape(scope.company_name)}</dd><dt>권한 대상</dt><dd ${change==='hidden-role'?'hidden':''}>회사 등록 시 지정된 관리 계정</dd>
    <dt>작업 담당</dt><dd>현재 로그인 계정</dd></dl><p ${change==='hidden-restriction'?'hidden':''}>회사 등록 시 지정된 관리 계정에 한해 연결할 수 있습니다.</p>
    <p>현재 로그인 계정이 권한 대상입니다</p><details><summary ${change==='not-tab-accessible'?'tabindex="-1"':''}>회사·계정 식별 정보</summary><dl>
    <dt>회사</dt><dd data-policy-company="${scope.company}">${scope.company}</dd><dt>그룹</dt><dd>${scope.group}</dd>
    <dt>권한 대상</dt><dd data-policy-recipient="${scope.recipient}">${scope.recipient}</dd><dt>현재 담당 계정</dt><dd data-policy-operator="${scope.operator}">${scope.operator}</dd></dl></details></div></section>
    <section class="policy-consequences"><h2 ${change==='hidden-heading'?'hidden':''}>${COPY.heading}</h2>
    <p ${change==='hidden-copy'?'hidden':change==='transparent-copy'?'style="opacity:0"':''}>${COPY.consequence}</p>
    <p class="policy-limit">${COPY.scope}</p>${change==='extra-claim'||change==='hidden-copy'?'<span>이 계정은 지금 급여 목록을 볼 수 있습니다.</span>':''}</section>`;
  for(const [change,expected] of [['visible',[]],['hidden-heading',['CONDITIONAL_SCOPE']],['hidden-copy',['CONDITIONAL_SCOPE']],['transparent-copy',['CONDITIONAL_SCOPE']],
    ['extra-claim',['CONDITIONAL_SCOPE']],['hidden-role',['PRIMARY_CONTEXT']],['hidden-restriction',['PRIMARY_CONTEXT']],['not-tab-accessible',['IDENTIFIER_DETAILS']]])await t.test(change,async()=>{
    await page.setContent(fixture(change));const row=await page.evaluate(collectTask,'MACHINERY_DOM_CONTROL');
    // This noninteractive collector control cannot supply interaction evidence.
    assert.deepEqual(taskIssues(row,scope).filter(code=>code!=='DISCLOSURE_INTERACTION'),expected);
  });
});
