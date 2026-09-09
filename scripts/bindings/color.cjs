'use strict';
// FIX-C01 operations are independently authored through actual WASM color owners.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-color/cases.json'))).cases;
const scalar=v=>typeof v==='object'?{NaN:NaN,Infinity:Infinity,'-Infinity':-Infinity,'-0':-0}[v.number]:v;
const snapshot=value=>{try{return value.value();}finally{value.free();}};
function compare(a,e,label){
  if(typeof a==='number'&&typeof e==='number')assert(Math.abs(a-e)<=1e-10*Math.max(1,Math.abs(e)),`${label}: ${a} != ${e}`);
  else if(e!==null&&typeof e==='object'){
    assert.deepEqual(Object.keys(a).sort(),Object.keys(e).sort(),label);
    for(const k of Object.keys(e))compare(a[k],e[k],label+'/'+k);
  }else assert.equal(a,e,label);
}
const results=[];
for(const test of cases){
 const i=test.input,e=test.expected;
 const value=i.parse!==undefined?c.color(i.parse):i.css!==undefined?c[i.constructor](i.css):c[i.constructor](...i.args.map(scalar));
 if(value===null){assert.equal(e,null);results.push({id:test.id,result:null});continue;}
 const result={value:value.value(),conversions:Object.fromEntries(['rgb','hsl','lab','hcl','cubehelix'].map(s=>[s,snapshot(c[s](value))])),displayable:value.displayable(),
  formats:{formatHex:value.formatHex(),formatHex8:value.formatHex8(),formatRgb:value.formatRgb(),formatHsl:value.formatHsl(),toString:value.toString(),hex:value.hex()},
  copy:snapshot(value.copy(Object.fromEntries(Object.entries(e.copyPatch).map(([k,v])=>[k,scalar(v)])))),copyPatch:e.copyPatch,
  brightness:e.brightness.map(op=>({method:op.method,k:op.k,value:snapshot(op.k===null?value[op.method]():value[op.method](scalar(op.k)))})),
  clamp:['Rgb','Hsl'].includes(value.space())?snapshot(value.clamp()):null,sourceAfter:value.value()};
 compare(result,e,test.id);
 const wire=value.toJson(),copied=c.ColorValue.fromJson(wire);assert.equal(copied.toJson(),wire);
 value.dispose();assert.equal(copied.toJson(),wire);value.free();copied.free();
 results.push({id:test.id,result});
}
assert.throws(()=>c.color('x'.repeat(4097)),c.ChartError);
assert.throws(()=>c.rgb(1,2),c.ChartError);
assert.throws(()=>c.ColorValue.fromJson('{"version":2,"value":{"space":"Rgb","channels":{"r":1,"g":2,"b":3,"opacity":1}}}'),c.ChartError);
const v=c.rgb(1,2,3);assert.throws(()=>v.copy({h:4}),c.ChartError);assert.equal(v.channel('r'),1);
v.dispose();assert.throws(()=>v.formatHex(),c.ChartError);v.free();
fs.mkdirSync(path.dirname(path.resolve(process.argv[3])),{recursive:true});fs.writeFileSync(process.argv[3],JSON.stringify(results,null,2)+'\n');
console.log(`PASS FIX-C01 WASM: ${cases.length} cases, all constructor/methods, exceptional tags, copies, strict descriptors and disposal.`);
