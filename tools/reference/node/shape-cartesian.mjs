// FIX-S02/03: independently evaluated Cartesian controls, helpers and irregular inputs.
import * as d3 from 'd3-shape';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/shapes'));
class Context {
 constructor(){this.operations=[];}
 moveTo(...args){this.operations.push({op:'moveTo',args});}
 lineTo(...args){this.operations.push({op:'lineTo',args});}
 bezierCurveTo(...args){this.operations.push({op:'bezierCurveTo',args});}
 closePath(){this.operations.push({op:'closePath'});}
}
const cases=[];
function emit(id,family,input,config={},helper=null){
 const g=d3[family]();
 for(const [key,value]of Object.entries(config)){
  if(key==='curve'){
   let c=d3['curve'+value.kind];
   for(const k of ['alpha','beta','tension'])if(k in value)c=c[k](value[k]);
   g.curve(c);
  }else if(key==='defined')g.defined(Array.isArray(value)?(_,i)=>value[i]:value);
  else if(key==='digits')g.digits(value);
  else g[key](value===null?null:'Column' in value?d=>d[value.Column]:value.Constant);
 }
 const generator=helper?g['line'+helper]():g,context=new Context();
 generator.context(context)(input);
 const svg=generator.context(null)(input);
 const finite=context.operations.every(op=>!op.args||op.args.every(Number.isFinite));
 const operations=JSON.parse(JSON.stringify(context.operations,(_,v)=>typeof v==='number'&&!Number.isFinite(v)?{number:String(v)}:v));
 cases.push({id,family,input,config,helper,finite,operations,svg});
}
const runs={
 duplicate:[[0,0],[0,0],[1,1],[1,1],[2,-1],[2,-1],[4,2]],
 vertical:[[0,0],[0,2],[0,-1],[0,3],[0,0]],
 reverse:[[4,2],[3,-1],[2,3],[1,1],[0,0]],
 irregular:Array.from({length:31},(_,i)=>[i*i/11,Math.sin(i*1.7)*10+i/3]),
};
for(const kind of Object.keys(d3).filter(n=>n.startsWith('curve')).map(n=>n.slice(5)).sort()){
 for(const [label,input]of Object.entries(runs)){
  emit(`${kind}-${label}-line`,'line',input,{curve:{kind}});
  if(kind!=='Bundle')emit(`${kind}-${label}-area`,'area',input,{curve:{kind},x0:{Column:1},y0:{Constant:1.23456},x1:{Column:0},y1:{Column:1}});
 }
 for(const n of [0,1,2,3,4]){
  const input=runs.irregular.slice(0,n+2),defined=input.map((_,i)=>i<n);
  emit(`${kind}-defined-${n}`,'line',input,{curve:{kind},defined});
  if(kind!=='Bundle')emit(`${kind}-area-gap-${n}`,'area',input,{curve:{kind},defined});
 }
 for(const param of ['alpha','beta','tension'])if(typeof d3['curve'+kind][param]==='function')for(const value of [-1,0.2,0.75,2]){
  emit(`${kind}-${param}-${value}`,'line',runs.irregular,{curve:{kind,[param]:value}});
  if(kind!=='Bundle')emit(`${kind}-${param}-${value}-area`,'area',runs.irregular,{curve:{kind,[param]:value}});
 }
}
const input=[[0,4,2,1],[1,3,3,-1],[2,-2,5,6],[5,8,9,2],[6,0,8,3]];
const configs=[{}, {x0:{Constant:1.234567},y0:{Constant:2.345678},x1:null,y1:null},
 {x0:{Column:1},y0:{Column:0},x1:{Column:2},y1:{Column:3}},
 {x0:{Column:2},y0:{Column:3},x1:{Constant:-2.34567},y1:{Constant:1.23456}},
 {x0:{Constant:3},y0:{Column:1},x1:null,y1:{Column:0}},
 {x0:{Column:3},y0:{Constant:-2},x1:{Column:2},y1:null}];
for(const [i,c]of configs.entries())for(const kind of ['Linear','Basis','Cardinal','Natural','Step']){
 const config={...c,curve:{kind},digits:0,defined:[true,true,false,true,true]};
 emit(`controls-${i}-${kind}`,'area',input,config);
 for(const helper of ['X0','X1','Y0','Y1'])emit(`helper-${i}-${kind}-${helper}`,'area',input,config,helper);
}
for(const config of [{x:{Constant:1.23456},y:{Column:3}}, {x:{Column:2},y:{Constant:-1.34567}}])for(const digits of [0,3,12,null])emit(`line-controls-${cases.length}`,'line',input,{...config,digits});
writeJson(path.join(out,'cartesian.json'),{schema_version:1,cases});
writeJson(path.join(out,'cartesian-manifest.json'),{schema_version:1,references:['d3-shape','d3-path'].map(provenance),cases:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cartesian.json')))});
console.log('PASS independent Cartesian oracle:',cases.length,'cases');
