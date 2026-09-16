const {test,expect}=require('@playwright/test');
const {effectiveSubmissions,unexpectedMutations}=require('./effective-submissions.cjs');
// Evidence-machinery controls, not product UI tests. No HTTP action is submitted.
const cases=[
  ['ordinary read form', '<form action="/_ui/payroll"><button>read</button></form>', 0],
  ['permitted own logout', '<form method="post" action="/api/v2/auth/logout"><button>logout</button></form>', 0],
  ['renamed business form', '<form aria-label="unrelated" method="post" action="/business/write"><button>write</button></form>', 2],
  ['GET form with POST submit override', '<form id="x" method="get" action="/_ui/payroll"><button formmethod="post" formaction="/business/write">write</button></form>', 1],
  ['external associated submit override', '<form id="x" method="get" action="/_ui/payroll"></form><button form="x" formmethod="post" formaction="/business/write">write</button>', 1],
  ['logout form with hostile submit action', '<form method="post" action="/api/v2/auth/logout"><button formaction="https://example.invalid/leak">write</button></form>', 1],
];
for(const [name,html,count] of cases) test(name,async({page})=>{
  await page.setContent('<base href="http://127.0.0.1/">'+html);
  expect(unexpectedMutations(await page.evaluate(effectiveSubmissions),'http://127.0.0.1')).toHaveLength(count);
});

const {expectEssentialReflow}=require('./reflow.cjs');
test('reflow oracle permits vertical scrolling with reachable controls',async({page})=>{
  await page.setViewportSize({width:320,height:800});
  await page.setContent('<main><p>Company</p><div style="height:1200px"></div><button>save</button></main>');
  await expectEssentialReflow(page,[page.getByText('Company',{exact:true}),page.getByRole('button',{name:'save',exact:true})]);
});
test('reflow oracle rejects clipped offscreen control despite zero page overflow',async({page})=>{
  await page.setViewportSize({width:320,height:800});
  await page.setContent('<style>body{margin:0}.clip{width:320px;overflow:hidden}.wide{width:800px}button{margin-left:600px}</style><div class="clip"><form class="wide"><button>save</button></form></div>');
  const save=page.getByRole('button',{name:'save',exact:true});
  await expect(save).toBeVisible(); // The old oracle's positive false witness.
  expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBe(320);
  await expect(expectEssentialReflow(page,[save])).rejects.toThrow(/essential control ends within horizontal viewport/);
});

// Two real-browser controls for the instrumentation itself, using actual native
// WebAssembly on an owned loopback HTTP origin; no production proof claim.
const {observeWasm}=require('./wasm-observer.cjs');
const http=require('node:http');
const {once}=require('node:events');
async function wasmPage(page,work){
 const server=http.createServer((request,response)=>{response.writeHead(200,{'Content-Type':'text/html'});response.end('<!doctype html><title>WASM observer control</title>');});
 server.listen(0,'127.0.0.1');await once(server,'listening');
 try{await page.addInitScript(observeWasm);await page.goto(`http://127.0.0.1:${server.address().port}/`);await work();}
 finally{await new Promise((resolve,reject)=>server.close(error=>error?reject(error):resolve()));}
}
test('WASM observer binds real instantiated bytes and distinguishes another module',async({page})=>{
 await wasmPage(page,async()=>{
  const result=await page.evaluate(async()=>{
   const a=new Uint8Array([0,97,115,109,1,0,0,0]);const b=new Uint8Array([...a,0,2,1,120]);
   await WebAssembly.instantiate(a);const compiled=await WebAssembly.compile(b);await WebAssembly.instantiate(compiled);
   return window.__consoleObservedWasmInstances();
  });
  expect(result).toHaveLength(2);expect(result[0].sha256).not.toBe(result[1].sha256);
  const crypto=require('node:crypto');expect(result[0].sha256).toBe(crypto.createHash('sha256').update(Buffer.from([0,97,115,109,1,0,0,0])).digest('hex'));
 });
});
test('WASM observer does not accept failed native instantiation as hydration',async({page})=>{
 await wasmPage(page,async()=>{
  const result=await page.evaluate(async()=>{let failed=false;try{await WebAssembly.instantiate(new Uint8Array([1,2,3]));}catch{failed=true;}return {failed,events:window.__consoleObservedWasmInstances()};});
  expect(result).toEqual({failed:true,events:[]});
 });
});
