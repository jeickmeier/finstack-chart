'use strict';
// FIX-21 explicit ownership stress; WASM capacity is not a native allocation counter.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const dir=path.resolve(process.argv[2]),c=require(path.join(dir,'authoring.cjs')),native=require(path.join(dir,'chart_wasm.js'));
async function main(){
 assert(global.gc,'Use --expose-gc');const bytes=[];let handles=0;
 for(let batch=0;batch<6;batch++){
  for(let cycle=0;cycle<50;cycle++)for(const id of ['Viridis','Blues','Rainbow','Sinebow']){
   const ramp=c.chromatic(id),copy=ramp.copy(),scale=new c.StandaloneScale('sequential',{interpolator:ramp});handles+=3;
   for(const v of [ramp.sample(.12345),copy.sample(.87654),scale.map(.5)]){v.free();handles++;}
   for(const v of [ramp,copy,scale]){v.dispose();v.dispose();v.free();}
   for(const v of c.chromaticScheme('Blues',9)){v.free();handles++;}
  }
  global.gc();await new Promise(r=>setImmediate(r));global.gc();await new Promise(r=>setImmediate(r));bytes.push(native.authoring_memory_bytes());
 }
 assert(Math.max(...bytes.slice(3))-Math.min(...bytes.slice(3))<=65536,JSON.stringify(bytes));
 fs.writeFileSync(process.argv[3],JSON.stringify({batches:6,cycles:300,ramps:4,owned_handles:handles,wasm_memory_bytes:bytes,plateau_tolerance_bytes:65536,scope:'WASM linear memory after explicit disposal and collection; native allocation counts and release RSS budget not measured'},null,2)+'\n');console.log('PASS FIX-21 owned catalog/ramp/scale plateau',bytes);
}
main().catch(e=>{console.error(e);process.exitCode=1;});
