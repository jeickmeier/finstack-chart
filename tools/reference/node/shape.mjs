// WP-S01 / FIX-S01-08: independent d3-shape 3.2.0 inventory and numeric contexts.
import * as d3 from 'd3-shape';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,retainLicense,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/shapes'));
const aliases={radialLine:'lineRadial',radialArea:'areaRadial',symbols:'symbolsFill',symbolX:'symbolTimes'};
const generators=['arc','area','areaRadial','line','lineRadial','link','linkHorizontal','linkVertical','linkRadial','pie','stack','symbol'];
const factory=n=>n==='link'?d3.link(d3.curveBumpX):d3[n]();
const owner=n=>n==='arc'||n==='pie'?'WP-S03':n.startsWith('stack')?'WP-S06':n.startsWith('symbol')?'WP-S05':/Radial|radial|^link|pointRadial/.test(n)?'WP-S04':'WP-S02';
class Context {
 constructor(){this.operations=[];}
 moveTo(...args){this.operations.push({op:'moveTo',args});}
 lineTo(...args){this.operations.push({op:'lineTo',args});}
 bezierCurveTo(...args){this.operations.push({op:'bezierCurveTo',args});}
 quadraticCurveTo(...args){this.operations.push({op:'quadraticCurveTo',args});}
 arc(x,y,r,a0,a1,anticlockwise=false){this.operations.push({op:'arc',args:{x,y,r,a0,a1,anticlockwise}});}
 arcTo(...args){this.operations.push({op:'arcTo',args});}
 rect(...args){this.operations.push({op:'rect',args});}
 closePath(){this.operations.push({op:'closePath'});}
}
const exports=Object.keys(d3).sort().map(name=>{
 const canonical=aliases[name]||name;let instance;
 if(generators.includes(canonical))instance=factory(canonical);
 else if(name.startsWith('curve'))instance=d3[name](new Context());
 const methods=instance?Object.keys(instance).filter(k=>typeof instance[k]==='function'):name.startsWith('symbol')&&!Array.isArray(d3[name])?['draw']:[];
 if(name.startsWith('curve'))for(const m of ['areaStart','areaEnd','lineStart','lineEnd','point'])if(typeof instance[m]==='function')methods.push(m);
 return {name,canonical,alias_of:aliases[name]||null,owner:owner(canonical),kind:Array.isArray(d3[name])?'palette':generators.includes(canonical)?'generator':name.startsWith('curve')?'curve':name.startsWith('symbol')?'symbol':name.startsWith('stackOrder')?'order':name.startsWith('stackOffset')?'offset':'point',methods:[...new Set(methods)].sort(),factory_controls:Object.keys(d3[name]).filter(k=>typeof d3[name][k]==='function').sort(),status:'open',adaptation:'Typed finite inputs, materialized columns or native/registered accessors; no JavaScript coercion or callback serialization.'};
});
function encoded(value) {
 if(typeof value==='function')return {kind:'function',export:Object.keys(d3).find(n=>d3[n]===value)||null,source:String(value)};
 if(value===null)return null;
 if(typeof value==='number'&&!Number.isFinite(value))return {number:String(value)};
 if(Array.isArray(value))return value.map(encoded);
 return value;
}
const generatorDefaults=generators.map(name=>{
 const g=factory(name),methods=Object.keys(g).sort(),boundaryMethods=methods.filter(m=>/^line[A-Z]/.test(m));
 return {name,methods,boundary_methods:boundaryMethods,defaults:Object.fromEntries(methods.filter(m=>!boundaryMethods.includes(m)&&m!=='centroid').map(m=>[m,encoded(g[m]())]))};
});
const protocols=[];
for(const family of ['line','area']){
 const events=[],context=new Context();let active=false;
 const curve=sink=>({areaStart(){events.push(['areaStart']);},areaEnd(){events.push(['areaEnd']);},lineStart(){active=false;events.push(['lineStart']);},lineEnd(){events.push(['lineEnd']);},point(x,y){events.push(['point',x,y]);(active?sink.lineTo.bind(sink):sink.moveTo.bind(sink))(x,y);active=true;}});
 const input=[[0,1],[1,2],[2,3],[3,4]],defined=[true,true,false,true];
 factory(family).defined((_,i)=>defined[i]).curve(curve).context(context)(input);
 protocols.push({family,input,defined,events,operations:context.operations});
}
const boundaries=[];
for(const family of ['area','areaRadial']){
 const context=new Context(),defined=()=>true,g=factory(family).curve(d3.curveBasis).digits(0).defined(defined).context(context);
 for(const method of Object.keys(g).filter(m=>/^line[A-Z]/.test(m))) {
  const helper=g[method]();boundaries.push({family,method,inheritance:{digits:helper.digits(),curve:helper.curve()===d3.curveBasis,defined:helper.defined()===defined,context:helper.context()===context},source:String(g[method])});
 }
}
const cases=[];
function shape(id,family,input,configure=()=>{},settings={}){
 const g=factory(family);configure(g);const c=new Context();g.context(c);g(input);
 const full=g.context(null).digits(null)(input),rounded={};
 for(const digits of [0,3,12])rounded[digits]=g.digits(digits)(input);
 cases.push({id,family,input,settings,operations:c.operations,svg_unrounded:full,svg_digits:rounded});
}
const points=[[0,0],[1,2],[2,-1],[3,3],[4,1],[6,2]];
for(const curve of Object.keys(d3).filter(n=>n.startsWith('curve')).sort()){
 for(const count of [0,1,2,3,4,6])shape(`line-${curve}-${count}`,'line',points.slice(0,count),g=>g.curve(d3[curve]),{curve});
 shape(`gap-${curve}`,'line',points,g=>g.curve(d3[curve]).defined((_,i)=>i!==2),{curve,defined:[true,true,false,true,true,true]});
 if(curve!=='curveBundle')for(const count of [0,1,2,3,4,6])shape(`area-${curve}-${count}`,'area',points.slice(0,count),g=>g.curve(d3[curve]),{curve});
 for(const control of ['alpha','beta','tension'])if(typeof d3[curve][control]==='function')for(const value of [0,1])shape(`parameter-${curve}-${value}`,'line',points,g=>g.curve(d3[curve][control](value)),{curve,[control]:value});
}
const arcCases=[['quarter',{innerRadius:0,outerRadius:10,startAngle:0,endAngle:Math.PI/2}],['hole',{innerRadius:5,outerRadius:10,startAngle:0,endAngle:Math.PI*2}],['reverse',{innerRadius:3,outerRadius:10,startAngle:Math.PI,endAngle:-Math.PI}],['swapped',{innerRadius:10,outerRadius:4,startAngle:0,endAngle:1}],['zero',{innerRadius:0,outerRadius:0,startAngle:0,endAngle:1}],['tiny',{innerRadius:4,outerRadius:10,startAngle:0,endAngle:1e-10}],['rounded',{innerRadius:4,outerRadius:10,startAngle:0,endAngle:1,padAngle:.1}],['negative-radius',{innerRadius:-3,outerRadius:-1,startAngle:0,endAngle:1}]];
for(const [id,input]of arcCases)shape(`arc-${id}`,'arc',input,g=>{if(id==='rounded')g.cornerRadius(3);},{cornerRadius:id==='rounded'?3:0,centroid:d3.arc().centroid(input)});
for(const type of Object.keys(d3).filter(n=>/^symbol[A-Z]/.test(n)&&!aliases[n]).sort())for(const size of [0,1,64,200])shape(`${type}-${size}`,'symbol',null,g=>g.type(d3[type]).size(size),{type,size});
for(const family of ['lineRadial','areaRadial'])shape(family,family,[[0,10],[Math.PI/2,20],[Math.PI,15]]);
for(const family of ['linkHorizontal','linkVertical','linkRadial'])shape(family,family,{source:[.3,5],target:[1.2,20]});
const layouts=[];
for(const values of [[],[1,1,2],[2,-1,0,3],[1,1,1]])for(const sort of ['default','none','ascending']){
 const p=d3.pie();if(sort==='none')p.sort(null);if(sort==='ascending')p.sortValues((a,b)=>a-b);
 layouts.push({family:'pie',values,sort,result:p(values)});
}
for(const order of Object.keys(d3).filter(n=>n.startsWith('stackOrder')).sort())for(const offset of Object.keys(d3).filter(n=>n.startsWith('stackOffset')).sort()){
 const data=[{a:2,b:-1,c:3},{a:0,b:4,c:1},{a:3,b:1,c:2}],result=d3.stack().keys(['a','b','c']).order(d3[order]).offset(d3[offset])(data);
 layouts.push({family:'stack',order,offset,data,keys:['a','b','c'],result:result.map(s=>({key:s.key,index:s.index,points:s.map(p=>({y0:p[0],y1:p[1],data:p.data}))}))});
}
writeJson(path.join(out,'inventory.json'),{schema_version:1,exports,generators:generatorDefaults,boundaries,palettes:{symbolsFill:d3.symbolsFill.map(s=>Object.keys(d3).find(n=>d3[n]===s)),symbolsStroke:d3.symbolsStroke.map(s=>Object.keys(d3).find(n=>d3[n]===s))}});
writeJson(path.join(out,'cases.json'),{schema_version:1,cases,layouts,protocols});
retainLicense('d3-shape',out);
writeJson(path.join(out,'manifest.json'),{schema_version:1,references:['d3-shape','d3-path'].map(provenance),exports:exports.length,path_cases:cases.length,layout_cases:layouts.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),inventory_sha256:digest(fs.readFileSync(path.join(out,'inventory.json'))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json')))});
console.log('PASS WP-S01 reference generation:',exports.length,'exports,',cases.length,'numeric-context cases,',layouts.length,'layouts; implementation acceptance remains open.');
