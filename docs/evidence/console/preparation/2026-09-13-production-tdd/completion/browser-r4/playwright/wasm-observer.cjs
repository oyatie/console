// Browser-machinery observation only. Every successful record follows the real
// native WebAssembly API resolving to an actual Instance for exact input bytes.
// No app event, endpoint, returned instance or authority object is fabricated.
function observeWasm(){
 const modules=new WeakMap();const instances=[];
 const digest=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),x=>x.toString(16).padStart(2,'0')).join('');
 for(const name of ['compile','compileStreaming','instantiate','instantiateStreaming']){
  const native=WebAssembly[name];if(typeof native!=='function')continue;
  WebAssembly[name]=async function(input,...rest){
   let actual=input;let identity;
   if(name.endsWith('Streaming')){
    actual=await input;const copy=actual.clone();identity=copy.arrayBuffer().then(digest);
   }else if(input instanceof WebAssembly.Module){identity=Promise.resolve(modules.get(input));}
   else{
    const copy=ArrayBuffer.isView(input)?new Uint8Array(input.buffer,input.byteOffset,input.byteLength).slice():new Uint8Array(input).slice();
    identity=digest(copy);
   }
   // Start real native work; observe only after it succeeds. Rejections are
   // propagated, not converted into witness records or replacement results.
   const result=await Reflect.apply(native,WebAssembly,[actual,...rest]);const sha256=await identity;
   const module=result instanceof WebAssembly.Module?result:result.module||(input instanceof WebAssembly.Module?input:null);
   if(module&&sha256)modules.set(module,sha256);
   const instance=result instanceof WebAssembly.Instance?result:result.instance;
   if(instance instanceof WebAssembly.Instance&&sha256)instances.push({method:name,sha256});
   return result;
  };
 }
 Object.defineProperty(window,'__consoleObservedWasmInstances',{value:()=>instances.map(x=>({...x})),writable:false,configurable:false});
}
module.exports={observeWasm};
