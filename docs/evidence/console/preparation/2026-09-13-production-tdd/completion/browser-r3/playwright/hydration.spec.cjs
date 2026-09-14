const {test,expect}=require('@playwright/test');
const {fixture,open,submittedCommand:command,writeObservation:write}=require('./fixture.cjs');
const fs=require('node:fs');
const crypto=require('node:crypto');
const amount=p=>p.getByLabel('기본급',{exact:true});
const saved=p=>p.getByText('서버에 저장됨',{exact:true});
const unsaved=p=>p.getByText('미저장 입력 있음',{exact:true});
const post=(page,f,status)=>page.waitForResponse(r=>r.request().method()==='POST'&&new URL(r.url()).pathname===f.pathname&&r.status()===status);
async function openHydrated(page,f){
 const wasm=page.waitForResponse(r=>new URL(r.url()).origin===f.origin&&new URL(r.url()).pathname.endsWith('.wasm'));
 await open(page,f);const response=await wasm;expect(response.status()).toBe(200);
 expect(response.headers()['content-type']).toContain('application/wasm');
 const bytes=await response.body();expect([...bytes.subarray(0,4)]).toEqual([0,97,115,109]);
 await expect(page.getByText('자동 저장 사용 중',{exact:true})).toBeVisible();
 return {url:response.url(),sha256:crypto.createHash('sha256').update(bytes).digest('hex')};
}
async function acknowledgement(response){
 expect(response.status()).toBe(200);
 expect(['fetch','xhr']).toContain(response.request().resourceType());
 expect(response.headers()['content-type']).toContain('application/json');
 expect(response.request().headers().accept).toContain('application/json');
 // This is the actual ordinary DraftAck transport projection. Rust deserializes
 // it as DraftAck and matches its receipt to independent persisted history.
 return {command_id:command(response.request()),ack:await response.json()};
}
test('hydrated edit autosaves an ordinary acknowledged draft receipt',async({page},info)=>{
 const f=fixture(info);const wasm=await openHydrated(page,f);
 await expect(amount(page)).toHaveValue('1200000');
 const response=post(page,f,200);await amount(page).fill('1234567');
 const ack=await acknowledgement(await response);await expect(saved(page)).toBeVisible();
 await expect(amount(page)).toBeFocused();
 await page.reload();await expect(amount(page)).toHaveValue('1234567');
 write({kind:'autosave',wasm,acknowledgements:[ack],manual_commands:[],final_amount:1234567});
});

test('hydrated stale autosave preserves local input and exposes current receipt recovery',async({page,browser},info)=>{
 const f=fixture(info);const wasm=await openHydrated(page,f);
 // Hold the original expected revision through a real transport outage while
 // another authenticated browser makes a genuine competing owner commit.
 await page.context().setOffline(true);await amount(page).fill('1234567');
 await expect(unsaved(page)).toBeVisible();await expect(saved(page)).toHaveCount(0);
 const otherContext=await browser.newContext({javaScriptEnabled:true});
 let winning;
 try{
  const other=await otherContext.newPage();await openHydrated(other,f);
  const response=post(other,f,200);await amount(other).fill('1250000');
  winning=await acknowledgement(await response);await expect(saved(other)).toBeVisible();
 }finally{await otherContext.close();}
 const conflict=post(page,f,409);await page.context().setOffline(false);const refused=await conflict;
 await expect(amount(page)).toHaveValue('1234567');await expect(page.getByRole('alert')).toBeVisible();
 await expect(saved(page)).toHaveCount(0);
 await page.getByRole('link',{name:'현재 내용 확인',exact:true}).click();
 await expect(amount(page)).toHaveValue('1250000');
 write({kind:'stale',wasm,acknowledgements:[winning],manual_commands:[],refused_command:command(refused.request()),refusal:await refused.json(),final_amount:1250000});
});

test('WASM load failure labels autosave unavailable and retains real manual save',async({page},info)=>{
 const f=fixture(info);let blocked=0;
 // Network failure injection only: never fulfill a route with a fake response.
 await page.route('**/*.wasm',route=>{blocked+=1;return route.abort('failed');});
 await open(page,f);
 await expect.poll(()=>blocked).toBeGreaterThan(0);
 await expect(page.getByText('자동 저장을 사용할 수 없음',{exact:true})).toBeVisible();
 await amount(page).fill('1234567');await expect(saved(page)).toHaveCount(0);
 const response=post(page,f,303);await page.getByRole('button',{name:'저장',exact:true}).click();
 const real=await response;await expect(saved(page)).toBeVisible();
 await page.reload();await expect(amount(page)).toHaveValue('1234567');
 write({kind:'wasm-fallback',blocked_wasm_requests:blocked,acknowledgements:[],manual_commands:[command(real.request())],final_amount:1234567});
});

test('hydrated offline input remains unacknowledged and reconnect saves exactly once',async({page},info)=>{
 const f=fixture(info);const wasm=await openHydrated(page,f);
 await page.context().setOffline(true);await amount(page).fill('1234567');
 await expect(unsaved(page)).toBeVisible();await expect(saved(page)).toHaveCount(0);
 await expect(amount(page)).toHaveValue('1234567');await expect(amount(page)).toBeFocused();
 const response=post(page,f,200);await page.context().setOffline(false);
 const ack=await acknowledgement(await response);await expect(saved(page)).toBeVisible();
 await page.reload();await expect(amount(page)).toHaveValue('1234567');
 write({kind:'reconnect',wasm,acknowledgements:[ack],manual_commands:[],final_amount:1234567});
});
