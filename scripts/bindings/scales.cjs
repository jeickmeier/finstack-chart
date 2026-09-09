'use strict';
// Every pinned FIX-20 operation through strict raw transport and the public JS facade.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),modulePath=path.resolve(process.argv[2]);
const native=require(path.join(modulePath,'chart_wasm.js')),c=require(path.join(modulePath,'authoring.cjs'));
const corpus=JSON.parse(fs.readFileSync(path.join(ROOT,'fixtures/parity/d3-scale/operations.json'),'utf8'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const n=v=>v&&typeof v==='object'?{NaN:NaN,Infinity:Infinity,'-Infinity':-Infinity,'-0':-0}[v.number]:v;
const en=v=>Number.isNaN(v)?{number:'NaN'}:v===Infinity?{number:'Infinity'}:v===-Infinity?{number:'-Infinity'}:Object.is(v,-0)?{number:'-0'}:v;
function k(v){if(v==='Null')return null;const [kind,p]=Object.entries(v)[0];return kind==='Number'?n(p):['Integer','Unsigned','Timestamp'].includes(kind)?new c.ScaleKey(kind,BigInt(p)):p;}
function ek(v){if(v===undefined)return null;if(v===null)return 'Null';if(v instanceof c.ScaleKey)return {[v.kind]:['Integer','Unsigned','Timestamp'].includes(v.kind)?String(v.value):v.value};if(typeof v==='boolean')return {Boolean:v};if(typeof v==='string')return {Text:v};if(typeof v==='bigint')return {Integer:String(v)};return {Number:en(v)};}
function v(x){const p=x.value;switch(x.kind){case'Missing':return undefined;case'Null':return null;case'Number':return n(p);case'Text':case'Boolean':return p;case'Array':return p.map(v);case'Record':return Object.fromEntries(Object.entries(p).map(([k,x])=>[k,v(x)]));case'Date':return c.dateValue(n(p));case'Color':return c.ColorValue.fromJson(JSON.stringify({version:1,value:p}));default:throw Error(x.kind);}}
function ev(x){if(x===undefined)return {kind:'Missing'};if(x===null)return {kind:'Null'};if(typeof x==='boolean')return {kind:'Boolean',value:x};if(typeof x==='string')return {kind:'Text',value:x};if(typeof x==='number')return {kind:'Number',value:en(x)};if(x instanceof Date)return {kind:'Date',value:en(x.getTime())};if(Array.isArray(x))return {kind:'Array',value:x.map(ev)};if(x instanceof c.ColorValue)return {kind:'Color',value:x.value()};return {kind:'Record',value:Object.fromEntries(Object.entries(x).map(([k,x])=>[k,ev(x)]))};}
function inp(x){if(x==='Missing')return undefined;const [kind,p]=Object.entries(x)[0];return kind==='Key'?k(p):kind==='Time'?p.includes('.')?Number(p):BigInt(p):n(p);}
const einp=(x,mode)=>({[mode]:mode==='Key'?ek(x):mode==='Time'?String(x):en(x)});
function opts(o){const r={...o};if('domain'in r)r.domain=r.domain.map(inp);if('range'in r)r.range=r.range.map(v);if('unknown'in r)r.unknown=v(r.unknown);if('interpolator'in r)r.interpolator=c.Interpolator.fromJson(JSON.stringify({version:1,spec:r.interpolator}));return r;}
function close(a,e,exact=false){
  if(typeof a==='number'&&typeof e==='number')return a===e||!exact&&Math.abs(a-e)<=1e-12+1e-12*Math.abs(e);
  if(Array.isArray(a)&&Array.isArray(e))return a.length===e.length&&a.every((v,i)=>close(v,e[i],exact));
  if(a&&e&&typeof a==='object'&&typeof e==='object')return Object.keys(a).length===Object.keys(e).length&&Object.keys(a).every(k=>Object.hasOwn(e,k)&&close(a[k],e[k],exact));
  return a===e;
}
function change(s,q,publicApi){
  if(!publicApi)return s.change(JSON.stringify(q));const [kind,p]=Object.entries(q)[0];
  switch(kind){case'Configure':return s.configure(opts(p));case'Reconfigure':return s.reconfigure(p);case'Train':return s.train(p.map(k));case'Nice':return s.nice(n(p));case'NiceTime':return 'Count'in p?s.nice(n(p.Count)):s.nice(10,{interval:p.Interval});default:throw Error(kind);}
}
function query(s,q,mode,publicApi){
  if(!publicApi)return JSON.parse(s.query(JSON.stringify(q)));const [name,p]=typeof q==='string'?[q,null]:Object.entries(q)[0];
  switch(name){
    case'Spec':return s.spec();case'Domain':return s.domain().map(x=>einp(x,mode));case'Range':return s.range().map(ev);
    case'Map':{const raw=s.mapValue(inp(p));assert(close(ev(s.map(inp(p))),raw));return raw;}
    case'Invert':{const result=s.invert(n(p));return mode==='Time'?{Time:String(result)}:{Value:ev(result)};}
    case'InvertExtent':{const r=s.invertExtent(v(p));return {found:r.found,lower:ek(r.lower),upper:ek(r.upper)};}
    case'Ticks':return s.ticks(n(p.count),{budget:p.budget}).map(en);
    case'TimeTicks':{const sel=p.selection,r='Count'in sel?s.ticks(n(sel.Count),{budget:p.budget}):s.ticks(10,{interval:sel.Interval,budget:p.budget});return r.map(x=>({Time:String(x)}));}
    case'Format':return s.format(n(p.value),{count:n(p.count),specifier:p.specifier,locale:p.locale});
    case'TimeFormat':return s.format(BigInt(p.value),p.format);
    case'Thresholds':return s.thresholds().map(x=>x===undefined?null:en(x));
    case'Quantiles':return s.quantiles(n(p)).map(x=>x===undefined?null:en(x));
    case'Step':return en(s.step());case'Bandwidth':return en(s.bandwidth());default:throw Error(name);
  }
}
function errorCode(e){if(e.code)return e.code;try{return JSON.parse(e.message).code;}catch{return /^CHART_[A-Z_]+/.exec(e.message)?.[0];}}
const records=[],failures=[];
for(const publicApi of [false,true]){
  const counts={cases:0,operations:0,adaptations:0,diagnostics:0,public:publicApi};
  for(const item of corpus.cases){
    counts.cases++;let scale;
    try{scale=publicApi?new c.StandaloneScale(item.family,opts(item.options)):native._Scale.create(JSON.stringify(item.family),JSON.stringify(item.options));}
    catch(e){if(!item.setup_error)failures.push([publicApi,item.id,'setup',e.message]);else counts.diagnostics++;continue;}
    if(item.setup_error){failures.push([publicApi,item.id,'setup','expected error']);scale.free();continue;}
    const copied=scale.copy();assert.equal(copied.to_json(),scale.to_json());copied.free();
    const restored=publicApi?c.StandaloneScale.fromJson(scale.to_json()):native._Scale.from_json(scale.to_json());assert.equal(restored.to_json(),scale.to_json());restored.free();
    for(const op of item.operations){
      counts.operations++;if(op.adaptation)counts.adaptations++;const before=scale.to_json();let selected=scale,changed,copy;
      try{
        if(op.copy){copy=scale.copy();selected=copy;}
        if(op.change){changed=change(selected,op.change,publicApi);selected=changed;}
        let actual=query(selected,op.query,item.mode,publicApi);
        if(op.select)for(const part of op.select.split('/').slice(1))actual=actual[part];
        if(op.diagnostic||!close(actual,op.expected,op.exact))failures.push([publicApi,item.id,op.name,actual,op.expected??op.diagnostic]);
        if(op.persist){scale.free();scale=changed;changed=undefined;}
      }catch(e){const allowed=op.diagnostic??op.allowed_diagnostic??[],code=errorCode(e);if(allowed.includes(code)||publicApi&&!code&&op.diagnostic&&op.adaptation&&(e instanceof TypeError||e instanceof RangeError))counts.diagnostics++;else failures.push([publicApi,item.id,op.name,e.message]);}
      finally{changed?.free();copy?.free();}
      if(!op.persist)assert.equal(scale.to_json(),before);
    }
    scale.dispose();assert.throws(()=>scale.to_json(),e=>errorCode(e)==='CHART_DISPOSED_HANDLE');scale.free();
  }
  records.push(counts);
}
assert.equal(failures.length,0,JSON.stringify(failures.slice(0,30),null,2));
assert(records.every(r=>r.cases===661&&r.operations===19562&&r.adaptations===1112));
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS WASM FIX-20 raw/public',records);
