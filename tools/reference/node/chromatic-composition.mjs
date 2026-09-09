// FIX-21 composition observes pinned d3-scale and d3-scale-chromatic directly.
import * as d3 from 'd3-scale';
import * as c from 'd3-scale-chromatic';
import {color} from 'd3-color';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale-chromatic'));
const scalar=x=>typeof x!=='number'||Number.isFinite(x)?x:{number:Number.isNaN(x)?'NaN':x>0?'Infinity':'-Infinity'};
const rgba=x=>{const v=color(x);if(!v)return null;const r=v.rgb().clamp();return [r.r,r.g,r.b,Math.round(r.opacity*255)];};
const missing=[11,22,33,44],cases=[];
const add=(family,options,scale,inputs,selection)=>{
 const samples=inputs.map(x=>{let css,diagnostic=false;try{css=scale(x);}catch{diagnostic=true;}
  return {input:scalar(x),reference_css:css??null,expected:x===null||typeof x==='number'&&!Number.isFinite(x)?missing:rgba(css),diagnostic};});
 cases.push({id:`${family}-${cases.length}`,family,options,selection,samples});
};
for(const [family,factory] of [['sequential','scaleSequential'],['sequential_log','scaleSequentialLog'],['sequential_pow','scaleSequentialPow'],['sequential_sqrt','scaleSequentialSqrt'],['sequential_symlog','scaleSequentialSymlog'],['diverging','scaleDiverging'],['diverging_log','scaleDivergingLog'],['diverging_pow','scaleDivergingPow'],['diverging_sqrt','scaleDivergingSqrt'],['diverging_symlog','scaleDivergingSymlog']]){
 const diverging=family.startsWith('diverging'),log=family.endsWith('_log');
 for(const descending of [false,true])for(const reverse of [false,true])for(const clamp of [false,true]){
  const domain=log?(diverging?[.1,1,100]:[.1,100]):diverging?[-10,0,100]:[-10,100];if(descending)domain.reverse();
  const id=diverging?'RdBu':'Viridis',interpolate=c['interpolate'+id];
  const scale=d3[factory]().domain(domain).interpolator(reverse?t=>interpolate(1-t):interpolate).clamp(clamp),options={domain,clamp};
  if(family.endsWith('_pow')){scale.exponent(2);options.exponent=2;}if(family.endsWith('_symlog')){scale.constant(3);options.constant=3;}
  add(family,options,scale,[...(log?[.001,.1,1,2,10,50,100,1000]:[-20,-10,-5,-1,0,1,5,10,50,100,150]),null,NaN,Infinity,-Infinity],{ramp:id,reverse});
 }
}
for(const reverse of [false,true]){
 const domain=[0,0,1,2,8,13],ramp=c.interpolateBlues;
 add('sequential_quantile',{domain},d3.scaleSequentialQuantile(domain,reverse?t=>ramp(1-t):ramp),[-1,0,.5,1,2,5,8,13,20,null,NaN,Infinity,-Infinity],{ramp:'Blues',reverse});
 for(const [family,factory,domain] of [['quantile','scaleQuantile',[0,0,1,2,8,13]],['quantize','scaleQuantize',[0,10]],['threshold','scaleThreshold',[2,5,8,10]]]){
  const values=reverse?c.schemeBlues[5].toReversed():c.schemeBlues[5],scale=d3[factory]().domain(domain).range(values);
  add(family,{domain,range:values},scale,[-1,0,1.9999999999999998,2,4.999999999999999,5,8,10,11,null,NaN,Infinity,-Infinity],{scheme:'Blues',size:5,reverse});
 }
 const categories=['A','B','C'],values=reverse?c.schemeCategory10.toReversed():c.schemeCategory10;
 add('ordinal',{domain:categories,range:values,unknown:'#0b16212c'},d3.scaleOrdinal(categories,values).unknown('#0b16212c'),['A','C','A','B','missing',null],{scheme:'Category10',size:null,reverse});
}
fs.mkdirSync(out,{recursive:true});writeJson(path.join(out,'composition.json'),{schema_version:1,missing,adaptation:'Chart null/NaN/Inf colors use the declared missing paint; finite legal-domain samples compare exact reference RGBA.',cases});
writeJson(path.join(out,'composition-manifest.json'),{schema_version:1,references:['d3-scale','d3-scale-chromatic','d3-color'].map(provenance),cases:cases.length,samples:cases.reduce((n,c)=>n+c.samples.length,0),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'composition.json')))});
console.log('PASS FIX-21 composition',cases.length,'cases',cases.reduce((n,c)=>n+c.samples.length,0),'samples');
