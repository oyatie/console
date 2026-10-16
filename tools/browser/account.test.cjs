'use strict';
const assert = require('node:assert/strict');
const {test} = require('node:test');
const next = require('./account.cjs');
const paths = ['/api/v2/auth/registration/start','/api/v2/auth/registration/finish','/api/v2/auth/passkey/login/start','/api/v2/auth/passkey/login/finish','/api/v2/auth/logout'];
const oldFlags = ['registration_wire','resident','cookie_security','literal_secret_absent','logout_cookie_clear','login_wire','same_account','active_after_login','reflow_root_320','reflow_register_320','reflow_account_320','keyboard_skip','keyboard_registration','keyboard_terms','keyboard_registration_submit'];
const newFlags = ['keyboard_logout','keyboard_login_entry','keyboard_login_submit'];
function complete(){return {browser_version:'153.0.8010.12',unexpected_posts:0,root_status:200,external_requests:0,checkpoints:['ENROLLED','LOGGED_OUT','LOGGED_IN'],posts:Object.fromEntries(paths.map(p=>[p,1])),cleanup:{confirmed:true},...Object.fromEntries([...oldFlags,...newFlags].map(k=>[k,true]))};}
test('complete observation positive control accepts',()=>{const r=complete();assert.equal(next.leafStatus(r),'BROWSER_LEAF_PASSED');});
for(const flag of newFlags){
 test(`new ${flag} omission is rejected`,()=>{const r=complete();delete r[flag];assert.equal(next.completeObservations(r),false);assert.equal(next.leafStatus(r),'BROWSER_LEAF_FAILED');});
 for(const bad of [false,null,'true',1])test(`new ${flag} refuses ${JSON.stringify(bad)}`,()=>{const r=complete();r[flag]=bad;assert.equal(next.completeObservations(r),false);assert.equal(next.leafStatus(r),'BROWSER_LEAF_FAILED');});
}
for(const flag of oldFlags)test(`retained ${flag} omission still fails`,()=>{const r=complete();delete r[flag];assert.equal(next.completeObservations(r),false);});
for(const path of paths)test(`missing actual request count ${path} fails`,()=>{const r=complete();delete r.posts[path];assert.equal(next.leafStatus(r),'BROWSER_LEAF_FAILED');});
for(const phase of ['ENROLLED','LOGGED_OUT','LOGGED_IN'])test(`missing owner acknowledgement ${phase} fails`,()=>{const r=complete();r.checkpoints=r.checkpoints.filter(p=>p!==phase);assert.equal(next.leafStatus(r),'BROWSER_LEAF_FAILED');});
for(const [label,change] of [
 ['cleanup unconfirmed',r=>{r.cleanup.confirmed=false;}],
 ['explicit failure',r=>{r.failure={stage:'logout',code:'LOGOUT_EFFECT'};}],
 ['unexpected post',r=>{r.unexpected_posts=1;}],
 ['external request',r=>{r.external_requests=1;}],
 ['browser drift',r=>{r.browser_version='153.0.8010.13';}],
 ['relay failure',r=>{r.relay_failure=true;}],
 ['TLS error',r=>{r.tls_client_error=true;}]
])test(`retained ${label} prevents success`,()=>{const r=complete();change(r);assert.equal(next.leafStatus(r),'BROWSER_LEAF_FAILED');});
