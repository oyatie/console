"use strict";
// Executes the actual browser driver's owner-wait and handoff exchange with
// Readline and a virtual clock. No browser, database, or business data fixture.
const {test}=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const readline=require('node:readline'),{PassThrough}=require('node:stream');
const root=process.env.CONSOLE_CHECKPOINT_SOURCE_ROOT||path.resolve(__dirname,'../..');
assert.ok(root&&path.isAbsolute(root),'exact source root required');
const driver=fs.readFileSync(path.join(root,'tools/browser/account-company-handoff.cjs'),'utf8');
const handoff=fs.readFileSync(path.join(root,'tools/browser/account_company_handoff.cjs'),'utf8');
const {validCheckpointCommand}=require(path.join(root,'tools/browser/company.cjs'));
function exact(source,prefix){const lines=source.split('\n').filter(line=>line.startsWith(prefix));assert.equal(lines.length,1);return lines[0];}
function control(t,successor){
 t.mock.timers.enable({apis:['setTimeout']});
 const stream=new PassThrough(),reader=readline.createInterface({input:stream,crlfDelay:Infinity}),input=reader[Symbol.asyncIterator]();
 const r={checkpoints:[]},wire=[];
 const context=vm.createContext({successor,input,r,validCheckpointCommand,setTimeout,clearTimeout,emit:value=>wire.push(value)});
 const probe=vm.runInContext([exact(driver,'function fact('),exact(driver,'function bounded('),
  exact(driver,'  async function receive('),exact(handoff,' async function exchange('),'({receive,exchange})'].join('\n'),context);
 t.after(()=>{stream.end();reader.close();});
 return {probe,r,wire,send:value=>stream.write(JSON.stringify(value)+'\n'),end:()=>stream.end()};
}
function observed(promise){const r={status:'pending'};r.done=promise.then(value=>{r.status='fulfilled';r.value=value;},error=>{r.status='rejected';r.code=error.code;});return r;}
async function drain(){for(let i=0;i<12;i++)await Promise.resolve();await new Promise(resolve=>setImmediate(resolve));}
const account='10000000-0000-4000-8000-000000000001';
test('Manager terminal checkpoint survives ordinary deadline and accepts actual CONTINUE',async t=>{
 const h=control(t,true),r=observed(h.probe.exchange('HANDOFF_REOPENED',account));
 await drain();t.mock.timers.tick(20001);await drain();assert.equal(r.status,'pending');
 assert.deepEqual(h.r.checkpoints,[]);assert.equal(h.wire[0].phase,'HANDOFF_REOPENED');
 h.send({kind:'CONTINUE',phase:'HANDOFF_REOPENED'});await r.done;assert.equal(r.status,'fulfilled');
 assert.deepEqual(h.r.checkpoints,['HANDOFF_REOPENED']);
});
test('Manager terminal checkpoint still has a finite 120-second deadline',async t=>{
 const h=control(t,true),r=observed(h.probe.exchange('HANDOFF_REOPENED',account));
 await drain();t.mock.timers.tick(119999);await drain();assert.equal(r.status,'pending');
 t.mock.timers.tick(1);await r.done;assert.equal(r.code,'TIMEOUT');assert.deepEqual(h.r.checkpoints,[]);
});
for(const [successor,phase] of [[false,'HANDOFF_REOPENED'],[true,'COMPANY_COMMITTED'],[true,'A_ENROLLED'],[true,'NOT_A_CHECKPOINT']])
 test(`ordinary checkpoint deadline retained: ${successor}/${phase}`,async t=>{
  const h=control(t,successor),r=observed(h.probe.exchange(phase,account));await drain();
  t.mock.timers.tick(19999);await drain();assert.equal(r.status,'pending');
  t.mock.timers.tick(1);await r.done;assert.equal(r.code,'TIMEOUT');assert.deepEqual(h.r.checkpoints,[]);
 });
for(const [label,value,code] of [['wrong phase',{kind:'CONTINUE',phase:'COMPANY_COMMITTED'},'OWNER_PROTOCOL'],
 ['extra key',{kind:'CONTINUE',phase:'HANDOFF_REOPENED',extra:true},'OWNER_PROTOCOL'],
 ['abort',{kind:'ABORT'},'OWNER_REFUSED']])test(`Manager wait rejects ${label}`,async t=>{
 const h=control(t,true),r=observed(h.probe.exchange('HANDOFF_REOPENED',account));await drain();h.send(value);await r.done;
 assert.equal(r.code,code);assert.deepEqual(h.r.checkpoints,[]);
});
test('Manager wait detects real owner EOF without waiting for deadline',async t=>{
 const h=control(t,true),r=observed(h.probe.exchange('HANDOFF_REOPENED',account));await drain();h.end();await r.done;
 assert.equal(r.code,'OWNER_EOF');assert.deepEqual(h.r.checkpoints,[]);
});
test('START remains bounded by unchanged outer watchdog rather than checkpoint timer',async t=>{
 const h=control(t,true),r=observed(h.probe.receive(true));await drain();t.mock.timers.tick(120001);await drain();assert.equal(r.status,'pending');
 h.send({kind:'START'});await r.done;assert.equal(r.status,'fulfilled');assert.equal(r.value.kind,'START');
 assert.ok(driver.includes('},180000);'),'retained full-driver watchdog');
});
