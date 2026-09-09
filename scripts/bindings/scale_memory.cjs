'use strict';
// SP-07: repeated owned scale creation/copy/reconfiguration/disposal must plateau.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const modulePath=path.resolve(process.argv[2]),c=require(path.join(modulePath,'authoring.cjs')),native=require(path.join(modulePath,'chart_wasm.js'));
async function main(){
 assert(global.gc,'Use --expose-gc');
 const numbers=Array.from({length:2000},(_,i)=>i),keys=numbers.map(i=>9007199254740992n+BigInt(i)),bytes=[];
 for(let batch=0;batch<6;batch++){
  for(let cycle=0;cycle<20;cycle++){
   for(const [family,options,input] of [
    ['linear',{domain:numbers,range:numbers},1000.5],
    ['ordinal',{domain:keys,range:['a','b','c']},keys[1000]],
    ['quantile',{domain:numbers,range:[0,1,2,3,4]},1000.5]]){
    const s=new c.StandaloneScale(family,options),copy=s.copy(),changed=s.configure({range:family==='ordinal'?['x','y']: [1,2,3,4,5]});
    s.map(input);copy.map(input);changed.map(input);
    for(const owned of [s,copy,changed]){owned.dispose();owned.dispose();owned.free();}
   }
  }
  global.gc();await new Promise(r=>setImmediate(r));global.gc();await new Promise(r=>setImmediate(r));
  bytes.push(native.authoring_memory_bytes());
 }
 assert(Math.max(...bytes.slice(3))-Math.min(...bytes.slice(3))<=65536,JSON.stringify(bytes));
 const report={cycles:120,families:3,owned_handles:1080,values_per_scale:2000,wasm_memory_bytes:bytes,plateau_tolerance_bytes:65536,scope:'WASM linear-memory capacity after explicit disposal and JS collection; not native allocation counts or a release RSS budget'};
 fs.writeFileSync(path.resolve(process.argv[3]),JSON.stringify(report,null,2)+'\n');console.log('PASS SP-07 owned scale allocation plateau',bytes);
}
main().catch(e=>{console.error(e);process.exitCode=1;});
