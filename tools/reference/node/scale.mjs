// FIX-20 / SP-01: pinned method catalog and behavioral observations, never production code.
import * as d3 from 'd3-scale';
import * as interpolation from 'd3-interpolate';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {root,workspace,provenance,writeJson,retainLicense,digest} from './provenance.mjs';
process.env.TZ='UTC';
const functionNames=new Map(Object.entries(interpolation).map(([name,value])=>[value,name]));
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale'));
function encode(v){
 if(v===undefined)return {kind:'Undefined'};
 if(typeof v==='number')return Object.is(v,-0)?{number:'-0'}:Number.isFinite(v)?v:{number:Number.isNaN(v)?'NaN':v>0?'Infinity':'-Infinity'};
 if(v instanceof Date)return {date:encode(+v)};
 if(v===d3.scaleImplicit)return {kind:'Implicit'};
 if(typeof v==='function')return {kind:'Function',name:functionNames.get(v)||v.name||'anonymous'};
 if(Array.isArray(v))return v.map(encode);
 if(v&&typeof v==='object')return Object.fromEntries(Object.entries(v).map(([k,x])=>[k,encode(x)]));
 return v;
}
const call=f=>{try{return {value:encode(f())};}catch(e){return {error:{name:e.name,message:e.message}};}};
const factories=Object.keys(d3).filter(k=>k.startsWith('scale')&&k!=='scaleImplicit').sort();
const getterNames=['domain','range','clamp','unknown','base','exponent','constant','round','padding','paddingInner','paddingOuter','align','bandwidth','step','interpolate','interpolator'];
const inventory=factories.map(factory=>{
 const s=d3[factory]();
 return {factory,methods:Object.keys(s).sort(),defaults:Object.fromEntries(getterNames.filter(k=>typeof s[k]==='function').map(k=>[k,call(()=>s[k]())])),owner:factory.includes('Time')||factory==='scaleUtc'?'SP-06':['scaleBand','scalePoint','scaleOrdinal'].includes(factory)?'SP-03':/Sequential|Diverging|Quantile|Quantize|Threshold/.test(factory)?'SP-04':'SP-02'};
});
const cases=[];
function observe(id,factory,config={},inputs=[-1,0,.25,.5,1,2],opts={}){
 let s=d3[factory]();
 const setup=call(()=>{for(const [method,args]of Object.entries(config))s[method](...args);return true;});
 const row={id,factory,config:encode(config),inputs:encode(inputs),setup,requirement:'SCL-06/07',fixture:'FIX-20',tolerance:{absolute:1e-12,relative:1e-12,exact:['tags','strings','category','rounded','timestamp']},owner:inventory.find(x=>x.factory===factory).owner};
 if(setup.error){cases.push(row);return;}
 row.output=inputs.map(x=>call(()=>s(x)));
 row.getters=Object.fromEntries(getterNames.filter(k=>typeof s[k]==='function').map(k=>[k,call(()=>s[k]())]));
 const queries={};
 if(s.invert)queries.invert=(opts.inverse||inputs).map(x=>({input:encode(x),...call(()=>s.invert(x))}));
 if(s.invertExtent)queries.invertExtent=[...s.range(),'absent'].map(x=>({input:encode(x),...call(()=>s.invertExtent(x))}));
 for(const method of ['quantiles','thresholds'])if(s[method]&&!(factory==='scaleSequentialQuantile'&&method==='quantiles'))queries[method]=call(()=>s[method]());
 if(factory==='scaleSequentialQuantile')queries.quantiles=[0,1,2,4,2.5].map(n=>({count:n,...call(()=>s.quantiles(n))}));
 if(s.ticks){
  const counts=opts.counts||[0,1,4,5.5,10];
  queries.ticks=counts.map(n=>({count:n,...call(()=>s.ticks(n))}));
  queries.labels=counts.map(n=>({count:n,...call(()=>{const f=s.tickFormat(n);return s.ticks(n).map(x=>[x,f(x)]);})}));
  if(s.nice)queries.nice=counts.map(n=>({count:n,...call(()=>s.copy().nice(n).domain())}));
 }
 // Every scale must copy its configuration independently.
 const copied=s.copy(),original=encode(s.domain());copied.domain([]);
 row.copy={original,after:encode(s.domain()),copyDomain:encode(copied.domain())};
 row.queries=queries;cases.push(row);
}
for(const factory of factories)observe(factory+'-defaults',factory);
const numeric=['scaleLinear','scalePow','scaleSqrt','scaleIdentity','scaleRadial','scaleLog','scaleSymlog'];
for(const factory of numeric){
 for(const [name,domain,range] of [
  ['ascending',[0,10],[0,100]],['descending',[10,0],[100,0]],['piecewise',[0,10,100],[0,50,100]],['piecewise-reversed',[100,10,0],[100,50,0]],['constant',[3,3],[10,20]],['range-constant',[0,1],[5,5]],['repeated',[0,0,1],[0,10,20]],['short-range',[0,10,100],[0,50]],['long-range',[0,1],[0,50,100]],['empty-domain',[],[0,1]],['empty-range',[0,1],[]],['single-domain',[3],[0,1]],['single-range',[0,1],[7]],['negative',[-100,-1],[0,100]],['tiny',[1e-12,1e-11],[0,1]],['large',[1e15,1e15+10],[0,1]]]){
  observe(factory+'-'+name,factory,{domain:[domain],range:[range]},[-101,-10,-1,-0,0,.25,.5,1,3,5,10,50,100,101,null,NaN]);
 }
 if(d3[factory]().clamp)observe(factory+'-clamp',factory,{domain:[[1,10]],range:[[0,100]],clamp:[true],unknown:['missing']},[null,NaN,-1,0,1,5,10,11],{inverse:[-100,0,50,100,200]});
 if(d3[factory]().rangeRound)observe(factory+'-round',factory,{domain:[[1,10]],rangeRound:[[-2,3]]},[0,1,1.9,3.7,5.5,7.3,9.1,10,11]);
}
for(const base of [.1,.5,1,2,Math.E,10,-2,0])observe('log-base-'+base,'scaleLog',{domain:[[.01,100]],base:[base]},[.01,.1,1,2,10,100]);
for(const exponent of [-2,-1,0,.5,1,2,3])observe('pow-exponent-'+exponent,'scalePow',{domain:[[-10,10]],exponent:[exponent]},[-10,-2,-1,-0,0,1,2,10]);
for(const constant of [-1,0,.1,1,10])observe('symlog-constant-'+constant,'scaleSymlog',{domain:[[-10,10]],constant:[constant]},[-10,-1,0,1,10]);
for(const factory of ['scaleBand','scalePoint'])for(const n of [0,1,2,5])for(const reversed of [false,true])for(const align of [0,.5,1])for(const round of [false,true]){
 const domain=Array.from({length:n},(_,i)=>'k'+i),config={domain:[domain],range:[reversed?[103,0]:[0,103]],align:[align],round:[round],padding:[.5]};
 observe(`${factory}-${n}-${reversed}-${align}-${round}`,factory,config,[...domain,'absent']);
}
observe('band-singleton-anchor','scaleBand',{domain:[['a']],range:[[0,100]],paddingInner:[.5],paddingOuter:[0]},['a']);
for(const inner of [0,1,1.5,-.5])for(const outer of [0,.5,-.5])observe(`band-padding-${inner}-${outer}`,'scaleBand',{domain:[['a','b','a']],range:[[0,100]],paddingInner:[inner],paddingOuter:[outer]},['a','b','missing']);
for(const domain of [[],['a','b','a'],[1,'1',true,false,null]])for(const range of [[],['red'],['red','green']])for(const implicit of [true,false])observe(`ordinal-${cases.length}`,'scaleOrdinal',{domain:[domain],range:[range],...(implicit?{}:{unknown:['missing']})},['a','b','a',1,'1',true,false,null,'unknown']);
for(const factory of factories.filter(k=>/Sequential|Diverging/.test(k))){
 const diverging=factory.includes('Diverging');
 for(const [name,domain]of [['ascending',diverging?[-10,0,100]:[0,100]],['descending',diverging?[100,0,-10]:[100,0]],['constant',diverging?[1,1,1]:[1,1]],['log-positive',diverging?[.1,1,100]:[.1,100]],['log-negative',diverging?[-100,-1,-.1]:[-100,-.1]]]){
  const config={domain:[domain]};if(d3[factory]().range&&factory!=='scaleSequentialQuantile')config.range=[diverging?['red','white','blue']:['red','blue']];
  observe(factory+'-'+name,factory,config,[-100,-10,-1,0,.1,1,10,50,100,null,NaN]);
 }
 if(d3[factory]().clamp)observe(factory+'-clamp',factory,{domain:[diverging?[1,5,10]:[1,10]],clamp:[true],unknown:['missing']},[null,NaN,0,1,5,10,11]);
}
for(const factory of ['scaleQuantile','scaleQuantize','scaleThreshold'])for(const domain of [[],[0],[0,1],[0,0,1,2,10],[10,0],[-10,0,100]])for(const range of [[],['a'],['a','b','a','c']])observe(`${factory}-${cases.length}`,factory,{domain:[domain],range:[range],unknown:['missing']},[null,NaN,-11,-1,0,.25,.5,1,2,5,10,100,101]);
observe('threshold-text','scaleThreshold',{domain:[['b','m']],range:[['low','middle','high']]},['a','b','c','m','z']);
for(const factory of ['scaleUtc','scaleTime'])for(const [id,a,b] of [['leap','2024-02-28','2024-03-02'],['year','2023-12-29','2024-01-03'],['month','2024-01-15','2024-06-15'],['millisecond','2024-01-01T00:00:00.001Z','2024-01-01T00:00:00.019Z']])observe(factory+'-'+id,factory,{domain:[[new Date(a),new Date(b)]],range:[[0,100]]},[new Date(a),new Date((+new Date(a)+ +new Date(b))/2),new Date(b)]);
for(const factory of numeric.filter(k=>typeof d3[k]().interpolate==='function'))for(const method of ['interpolateNumber','interpolateRound','interpolateLab','interpolateHclLong','interpolateCubehelix'])observe(factory+'-'+method,factory,{domain:[[1,10]],range:[method.includes('Number')||method.includes('Round')?[0,100]:['red','blue']],interpolate:[interpolation[method]]},[1,3.25,5.5,7.75,10]);
for(const factory of factories.filter(k=>typeof d3[k]().interpolator==='function')){
 const f=interpolation.interpolateRgb.gamma(2)('red','blue');functionNames.set(f,'interpolateRgb.gamma(2)(red,blue)');
 observe(factory+'-interpolator',factory,{interpolator:[f]},[0,.25,.5,.75,1]);
}
const localTime=['UTC','America/New_York','Europe/Berlin','Australia/Lord_Howe'].map(zone=>{
 const child=spawnSync(process.execPath,[path.join(workspace,'scale-time.mjs')],{env:{...process.env,TZ:zone},encoding:'utf8',maxBuffer:16*1024*1024});
 if(child.status!==0)throw Error(child.stderr);return JSON.parse(child.stdout);
});
writeJson(path.join(out,'local-time.json'),{schema_version:1,zones:localTime});
const formatCases=[];
for(const [a,b]of [[0,1],[-1,1],[0,1e-8],[0,1e9],[990,1100]])for(const count of [0,1,4,10])for(const specifier of [undefined,'',',.2f','+.1%','.3s','~g','(.2f','#x','08.2f','^12.3e'])formatCases.push({start:a,stop:b,count,specifier:encode(specifier),result:call(()=>{const f=d3.tickFormat(a,b,count,specifier);return [a,(a+b)/2,b,-0].map(x=>f(x));})});
// Independently derived anchors catch reference harness/configuration mistakes.
const find=id=>cases.find(c=>c.id===id);
const mapAt=(id,x)=>{const c=find(id);return c.output[c.inputs.indexOf(x)].value;};
if(mapAt('scaleLinear-piecewise',10)!==50||mapAt('scaleLinear-constant',3)!==15)throw Error('Independent piecewise/constant anchor');
if(Math.abs(mapAt('scaleLog-negative',-10)-50)>1e-12)throw Error('Independent negative log anchor');
const band=find('band-singleton-anchor');if(band.output[0].value!==25||band.getters.bandwidth.value!==50)throw Error('Independent aligned singleton anchor');
const clamp=find('scaleLinear-clamp').queries.invert.find(v=>v.input===200);if(clamp.value!==10)throw Error('Independent inverse clamp anchor');
if(JSON.stringify(find('scaleLinear-defaults').queries.ticks.find(v=>v.count===4).value)!=='[0,0.2,0.4,0.6,0.8,1]')throw Error('Independent tick anchor');
const lock=JSON.parse(fs.readFileSync(path.join(workspace,'package-lock.json')));
const identities=['d3-scale','d3-array','d3-interpolate','d3-format','d3-time','d3-time-format','d3-color','internmap'].map(name=>({...provenance(name),integrity:lock.packages['node_modules/'+name].integrity,resolved:lock.packages['node_modules/'+name].resolved}));
for(const identity of identities)retainLicense(identity.name,path.join(out,'licenses',identity.name));
writeJson(path.join(out,'inventory.json'),{schema_version:1,exports:Object.keys(d3).sort(),factories:inventory,helperMethods:['scaleImplicit','tickFormat']});
writeJson(path.join(out,'cases.json'),{schema_version:1,cases,format_cases:formatCases});
retainLicense('d3-scale',out);
writeJson(path.join(out,'manifest.json'),{schema_version:1,requirements:['SCL-06','SCL-07','SCL-08','FIX-20'],oracle:identities,cases:cases.length,format_cases:formatCases.length,inventory_sha256:digest(fs.readFileSync(path.join(out,'inventory.json'))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),time_generator_sha256:digest(fs.readFileSync(path.join(workspace,'scale-time.mjs'))),local_time_sha256:digest(fs.readFileSync(path.join(out,'local-time.json'))),source_commit:'83555bd759c7314420bd4240642beda5e258db9e',timezone:'UTC',locale:'en-US',status:'Reference observations only; Rust/hosts are not certified.'});
console.log(`PASS scale reference: ${factories.length} factories, ${cases.length} observations, ${formatCases.length} format cases.`);
