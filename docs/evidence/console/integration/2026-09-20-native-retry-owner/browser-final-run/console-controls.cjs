const fs=require('node:fs');const vm=require('node:vm');const assert=require('node:assert/strict');
const source=fs.readFileSync(__dirname+'/retry.cjs','utf8');const code=source.split('// CONSOLE_ORACLE_START\n')[1].split('// CONSOLE_ORACLE_END')[0];assert(code);
const sandbox={URL};vm.createContext(sandbox);vm.runInContext(code,sandbox);const oracle=sandbox.expectedDocument503;
const report=JSON.parse(fs.readFileSync('/private/tmp/console-native-recovery-browser-run-20260920/report.json','utf8'));
const errors=report.console.filter(m=>m.type==='error');assert(errors.length>0);for(const error of errors)assert.equal(oracle(error,report.responses),true);
const error=errors.find(m=>m.location.url.includes('#main-content'));assert(error);const url=new URL(error.location.url);url.hash='';const response=report.responses.find(r=>r.url===url.href&&r.status===503&&r.navigation);assert(response);
const attacks=[([m,r])=>{m.location.url='http://foreign.invalid'+new URL(m.location.url).pathname;},([m,r])=>{const u=new URL(m.location.url);u.pathname='/wrong';m.location.url=u.href;},([m,r])=>{const u=new URL(m.location.url);u.search='?wrong=1';m.location.url=u.href;},([m,r])=>{r.status=200;},([m,r])=>{r.navigation=false;},([m,r])=>{m.text='Unrelated execution failure 503';},([m,r])=>{m.location.url='bad URL';},([m,r])=>{r.url+='?different';}];
for(const attack of attacks){const pair=structuredClone([error,response]);attack(pair);assert.equal(oracle(pair[0],[pair[1]]),false);}
console.log(JSON.stringify({actual_console_errors_matched:errors.length,fragment_positive:true,corruptions_detected:attacks.length,new_browser_runs:0}));
