// FIX-S04: independent d3-shape 3.2.0 arc geometry, centroids and pie ordering/layout.
import * as d3 from 'd3-shape';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/shapes'));
class Context {
 constructor(){this.operations=[];}
 moveTo(...args){this.operations.push({op:'moveTo',args});}
 lineTo(...args){this.operations.push({op:'lineTo',args});}
 arc(x,y,r,a0,a1,anticlockwise=false){this.operations.push({op:'arc',args:{x,y,r,a0,a1,anticlockwise}});}
 closePath(){this.operations.push({op:'closePath'});}
}
const arcs=[];
function add(id,input,config={}){
 const g=d3.arc();for(const [key,value]of Object.entries(config))g[key](value);
 const context=new Context();let error=null;
 try{g.context(context)(input);}catch(e){error=String(e);}
 const centroid=g.centroid(input),svg={};
 for(const digits of [0,3,12,null]){try{svg[String(digits)]=g.context(null).digits(digits)(input);}catch(e){svg[String(digits)]={error:String(e)};}}
 const finite=context.operations.every(o=>!o.args||Object.values(o.args).every(v=>typeof v!=='number'||Number.isFinite(v)))&&centroid.every(Number.isFinite);
 arcs.push({id,input,config,operations:context.operations,centroid,svg,finite,error});
}
for(const [ri,ro] of [[0,10],[3,10],[9,10],[10,4],[-3,10],[-3,-1],[0,0]])for(const span of [0,1e-14,1e-8,.03,.5,Math.PI,Math.PI*1.7,Math.PI*2,Math.PI*3,-1.2])for(const corner of [0,.5,9]){
 add(`r${ri}-${ro}-a${span}-c${corner}`,{innerRadius:ri,outerRadius:ro,startAngle:.3,endAngle:.3+span,padAngle:.12},{cornerRadius:corner});
}
for(const start of [0,-Math.PI,7,1e6])for(const span of [.001,.2,1,-1,Math.PI*2])for(const pad of [0,.001,.4,2,-.1])for(const padRadius of [null,0,2,20]){
 add(`padding-${arcs.length}`,{innerRadius:5,outerRadius:10,startAngle:start,endAngle:start+span,padAngle:pad},{cornerRadius:2,padRadius});
}
for(const overrides of [{innerRadius:7},{outerRadius:2},{startAngle:-1},{endAngle:4},{padAngle:.7},{innerRadius:0,outerRadius:12,startAngle:0,endAngle:Math.PI/2,cornerRadius:2,padRadius:16}])add(`constant-${arcs.length}`,{innerRadius:3,outerRadius:10,startAngle:1,endAngle:2,padAngle:.1},overrides);
for(const input of [{innerRadius:0,outerRadius:10,startAngle:0,endAngle:Math.PI/2},{innerRadius:1e-10,outerRadius:1e-9,startAngle:0,endAngle:1},{innerRadius:1e100,outerRadius:2e100,startAngle:0,endAngle:1},{innerRadius:1e300,outerRadius:2e300,startAngle:0,endAngle:1}])add(`boundary-${arcs.length}`,input,{cornerRadius:1});
const pies=[];
for(const values of [[],[1,1,2],[2,-1,0,3],[0,0,0],[-1,-2],[1,1,1],[1e-300,2e-300],[1e300,2e300]])for(const order of ['default','none','valuesAscending','dataDescending'])for(const [start,end,pad]of [[0,Math.PI*2,0],[0,Math.PI*2,.2],[1,-2,.1],[0,100,10],[2,2,.2],[0,1,-.1]]){
 const data=values.map((value,i)=>({id:String(9007199254741001n+BigInt(i)),label:String.fromCharCode(90-i),value})),g=d3.pie().value(d=>d.value).startAngle(start).endAngle(end).padAngle(pad);
 if(order==='none')g.sort(null);if(order==='valuesAscending')g.sortValues((a,b)=>a-b);if(order==='dataDescending')g.sort((a,b)=>b.label.localeCompare(a.label,'en'));
 pies.push({id:`pie-${pies.length}`,data,order,start,end,pad,result:g(data)});
}
writeJson(path.join(out,'arc-pie.json'),JSON.parse(JSON.stringify({schema_version:1,arcs,pies},(_,v)=>typeof v==='number'&&!Number.isFinite(v)?{number:String(v)}:v)));
writeJson(path.join(out,'arc-pie-manifest.json'),{schema_version:1,references:['d3-shape','d3-path'].map(provenance),arc_cases:arcs.length,pie_cases:pies.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'arc-pie.json')))});
console.log('PASS independent arc/pie oracle:',arcs.length,'arcs and',pies.length,'pie layouts; implementation acceptance is separate.');
