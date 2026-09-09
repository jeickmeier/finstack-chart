'use strict';
// Independent actual-public WASM constructor and lifecycle execution against pinned fixtures.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-interpolate/cases.json'))),transforms=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-interpolate/transforms.json')));
const scalar=v=>typeof v==='object'?({'NaN':NaN,'Infinity':Infinity,'-Infinity':-Infinity,'-0':-0})[v.number]:v;
const tagged=v=>Number.isNaN(v)?{number:'NaN'}:v===Infinity?{number:'Infinity'}:v===-Infinity?{number:'-Infinity'}:Object.is(v,-0)?{number:'-0'}:v;
function decode(v,owned){switch(v.kind){case'Missing':return undefined;case'Null':return null;case'Boolean':case'Text':return v.value;case'Number':return scalar(v.value);case'Date':return c.dateValue(scalar(v.value));case'Color':{const color=c.ColorValue.fromJson(JSON.stringify({version:1,value:v.value}));owned.push(color);return color;}case'NumericArray':return c.numericArray(v.value.element,v.value.values.map(scalar));case'Array':return v.value.map(v=>decode(v,owned));case'Record':return Object.fromEntries(Object.entries(v.value).map(([k,v])=>[k,decode(v,owned)]));default:throw Error(v.kind);}}
function encode(v){if(v===undefined)return {kind:'Missing'};if(v===null)return {kind:'Null'};if(typeof v==='boolean')return {kind:'Boolean',value:v};if(typeof v==='number')return {kind:'Number',value:tagged(v)};if(typeof v==='string')return {kind:'Text',value:v};if(v instanceof Date)return {kind:'Date',value:tagged(+v)};if(v instanceof c.ColorValue)return {kind:'Color',value:v.value()};if(ArrayBuffer.isView(v))return {kind:'NumericArray',value:{element:v.constructor.name,values:Array.from(v,tagged)}};if(Array.isArray(v))return {kind:'Array',value:v.map(encode)};return {kind:'Record',value:Object.fromEntries(Object.entries(v).map(([k,v])=>[k,encode(v)]))};}
function compare(a,e,id,exact=false){if(typeof e==='number'){assert.equal(typeof a,'number',id);assert.ok(exact?a===e:Math.abs(a-e)<=1e-12+1e-12*Math.abs(e),`${id}: ${a} != ${e}`);}else if(Array.isArray(e)){assert.ok(Array.isArray(a),id);assert.equal(a.length,e.length,id);e.forEach((e,i)=>compare(a[i],e,id+'/'+i,exact));}else if(e!==null&&typeof e==='object'){assert.deepEqual(Object.keys(a).sort(),Object.keys(e).sort(),id);exact=exact||e.kind==='Date'||(e.element!==undefined&&e.element!=='Float64Array');for(const k of Object.keys(e))compare(a[k],e[k],id+'/'+k,exact);}else assert.equal(a,e,id);}
function construct(test,owned){const args=test.args.map(v=>decode(v,owned)),options=test.config;if(test.op==='quantize')return c[options.factory||'interpolateNumber'](...args);if(test.op==='piecewise')return options.factory?c.piecewise(c[options.factory],args[0]):c.piecewise(args[0]);let factory=c[test.op];if(Object.hasOwn(options,'gamma'))factory=factory.gamma(scalar(options.gamma));if(Object.hasOwn(options,'rho'))factory=factory.rho(scalar(options.rho));return factory(...args);}
const results=[];
for(const test of fixture.cases){const owned=[];let f;
 try{
  f=construct(test,owned);
  if(test.op==='quantize'){const actual=encode(f.quantize(test.config.count));assert.equal(test.adaptation,undefined,test.id);compare(actual,test.expected,test.id);results.push({id:test.id,samples:actual});continue;}
  assert.equal(test.adaptation,undefined,test.id);
  const samples=test.times.map((t,i)=>{const actual=f.sampleValue(t);compare(actual,test.expected[i],test.id);compare(encode(f.sample(t)),test.expected[i],test.id+'/public');return actual;});
  if(Object.hasOwn(test,'duration')){compare(f.duration,test.duration,test.id+'/duration');assert.equal(f.schedulingDuration,Math.abs(f.duration));}
  const restored=c.Interpolator.fromJson(f.toJson()),copy=f.copy(),held=f.sampleValue(.25);f.dispose();
  compare(restored.sampleValue(.25),held,test.id+'/restored');compare(copy.sampleValue(.25),held,test.id+'/copy');restored.free();copy.free();
  results.push({id:test.id,samples,duration:test.duration??null});
 }catch(error){if(error instanceof assert.AssertionError)throw error;assert.ok(test.adaptation,`${test.id}: ${error.stack}`);results.push({id:test.id,adaptation:test.adaptation});}
 finally{if(f)f.free();for(const value of owned)value.free();}
}
function canonical(a,e,id){const pattern=/[-+]?(?:\d+\.?\d*|\.?\d+)(?:[eE][-+]?\d+)?/g;assert.equal(a.replace(pattern,'#'),e.replace(pattern,'#'),id);compare(Array.from(a.matchAll(pattern),v=>+v[0]),Array.from(e.matchAll(pattern),v=>+v[0]),id);}
const motion=[];
for(const test of transforms.cases){const factory=test.syntax==='css'?c.interpolateTransformCss:c.interpolateTransformSvg;let raw;
 try{raw=factory(test.a,test.b);}catch(error){assert.ok(test.adaptation,`${test.id}: ${error}`);motion.push({id:test.id,adaptation:test.adaptation});continue;}
 assert.equal(test.adaptation,undefined,test.id);const resolved=factory(...test.matrices),samples=[];
 for(let i=0;i<test.times.length;i++){const t=test.times[i],e=test.expected[i];canonical(resolved.sample(t),e.text,test.id);for(const f of [raw,resolved]){const m=f.sampleTransform(t);m.forEach((a,j)=>assert.ok(Math.abs(a-e.matrix[j])<=1e-5+1e-6*Math.abs(e.matrix[j]),`${test.id}: matrix ${j} ${a} != ${e.matrix[j]}`));}samples.push({text:resolved.sample(t),matrix:resolved.sampleTransform(t)});}
 raw.free();resolved.free();motion.push({id:test.id,samples});
}
let f=c.interpolateArray([0,[0]],[10,[20]]),held=f.sample(.25),samples=f.quantize(3);samples[0][1][0]=999;assert.deepEqual(held,[2.5,[5]]);assert.deepEqual(samples[2],[10,[20]]);const copy=f.copy();f.dispose();assert.throws(()=>f.sample(.5));assert.deepEqual(copy.sample(.5),[5,[10]]);copy.free();f.free();
for(const invalid of [{version:2,spec:{operation:'Discrete',values:[{kind:'Null'}]}},{version:1,spec:{operation:'Unknown'}},{version:1,extra:1,spec:{operation:'Discrete',values:[{kind:'Null'}]}}])assert.throws(()=>c.Interpolator.fromJson(JSON.stringify(invalid)));
f=c.interpolateNumber(0,1);assert.throws(()=>f.sample(NaN));assert.throws(()=>f.sample('0.5'));assert.throws(()=>c.quantize(f,1));assert.throws(()=>c.quantize(f,2.5));f.free();assert.throws(()=>c.piecewise((a,b)=>a,[0,1]));assert.throws(()=>c.interpolate(new BigInt64Array([1n]),new BigInt64Array([2n])));assert.throws(()=>c.interpolate(new DataView(new ArrayBuffer(8)),new Float64Array([1])));
const out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});fs.writeFileSync(out,JSON.stringify({cases:results,transforms:motion},null,2)+'\n');
console.log(`PASS FIX-I01 WASM: ${results.length} value/configuration cases, ${motion.length} browser transforms, owned samples, descriptors and disposal.`);
