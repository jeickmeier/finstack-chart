// FIX-S05: pinned independent radial conversion, boundaries and two-point link curves.
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
function emit(family,input,config={},helper=null){
 let g=family==='link'?d3.link(d3.curveLinear):d3[family]();
 for(const [key,value]of Object.entries(config)){
  if(key==='curve'){
   let c=d3['curve'+value.kind];for(const k of ['alpha','beta','tension'])if(k in value)c=c[k](value[k]);
   if(family==='link')g=d3.link(c);else g.curve(c);
  }else if(key==='defined')g.defined(Array.isArray(value)?(_,i)=>value[i]:value);
  else if(key==='digits')g.digits(value);
  else if(key==='source'||key==='target')g[key](d=>d[value]);
  else g[key](value===null?null:'Column'in value?d=>d[value.Column]:value.Constant);
 }
 if(helper)g=g['line'+helper]();
 const context=new Context();g.context(context)(input);
 const svg=g.context(null)(input),finite=context.operations.every(o=>!o.args||o.args.every(Number.isFinite));
 cases.push({id:`radial-link-${cases.length}`,family,input,config,helper,finite,operations:context.operations,svg});
}
const runs=[[],[[0,10]],[[0,10],[Math.PI/2,20]],[[0,10],[.2,15],[2.8,4],[5.9,20],[Math.PI*2,10]],[[7,10],[4,-5],[1,0],[-2,12]],[[0,1],[0,1],[1,1],[2,1]],[[1e6,10],[1e6+.01,11],[1e6+.3,12]]];
for(const kind of Object.keys(d3).filter(n=>n.startsWith('curve')).map(n=>n.slice(5)).sort()){
 for(const input of runs){
  emit('lineRadial',input,{curve:{kind}});
  if(kind!=='Bundle')emit('areaRadial',input,{curve:{kind},innerRadius:{Constant:3}});
 }
 const input=runs[3],defined=[true,true,false,true,true];
 emit('lineRadial',input,{curve:{kind},defined});
 if(kind!=='Bundle')emit('areaRadial',input,{curve:{kind},defined});
 for(const param of ['alpha','beta','tension'])if(typeof d3['curve'+kind][param]==='function')for(const value of [-1,.25,.8,2])emit('lineRadial',runs[3],{curve:{kind,[param]:value}});
 for(const pair of [[[0,0],[10,20]],[[10,20],[0,0]],[[1,2],[1,2]],[[-4,2],[3,-5]]])emit('link',{source:pair[0],target:pair[1]},{curve:{kind}});
}
const input=[[.1,12,.3,20],[1.2,8,2,18],[2.8,3,4,9],[5.8,12,6,14]];
for(const config of [{},{startAngle:{Column:0},endAngle:{Column:2},innerRadius:{Column:1},outerRadius:{Column:3}},{startAngle:{Constant:1},endAngle:null,innerRadius:{Column:1},outerRadius:null}])for(const kind of ['Linear','Basis','Cardinal','Step']){
 const cfg={...config,curve:{kind},digits:0,defined:[true,true,false,true]};emit('areaRadial',input,cfg);
 for(const helper of ['StartAngle','EndAngle','InnerRadius','OuterRadius'])emit('areaRadial',input,cfg,helper);
}
for(const digits of [0,3,12,null]){
 emit('lineRadial',input,{angle:{Column:2},radius:{Constant:12.3456789},digits});
 emit('areaRadial',input,{angle:{Constant:.23456789},radius:{Column:3},digits});
 for(const family of ['linkHorizontal','linkVertical','linkRadial'])for(const pair of [[[0,10],[Math.PI/2,20]],[[2,20],[-2,-10]],[[1,2],[1,2]],[[0,0],[0,0]]]){
  emit(family,{source:pair[0],target:pair[1]},{digits});
  emit(family,{a:pair[0],b:pair[1]},{source:'a',target:'b',[family==='linkRadial'?'angle':'x']:{Column:1},[family==='linkRadial'?'radius':'y']:{Constant:2.3456789},digits});
 }
}
const points=[];for(const angle of [0,Math.PI/2,Math.PI,Math.PI*2,-.2,1e6])for(const radius of [-10,0,1,20])points.push({angle,radius,result:d3.pointRadial(angle,radius)});
writeJson(path.join(out,'radial-link.json'),JSON.parse(JSON.stringify({schema_version:1,cases,points},(_,v)=>typeof v==='number'&&!Number.isFinite(v)?{number:String(v)}:v)));
writeJson(path.join(out,'radial-link-manifest.json'),{schema_version:1,references:['d3-shape','d3-path'].map(provenance),cases:cases.length,points:points.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'radial-link.json')))});
console.log('PASS independent radial/link oracle:',cases.length,'generators and',points.length,'radial points; implementation acceptance is separate.');
