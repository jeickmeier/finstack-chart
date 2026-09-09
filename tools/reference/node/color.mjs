// FIX-C01: d3-color 3.1.0 values and inherited operations; development-only oracle.
import * as d3 from 'd3-color';
import fs from 'node:fs';
import path from 'node:path';
import {root,workspace,provenance,writeJson,retainLicense,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-color'));
const scalar=v=>Object.is(v,-0)?{number:'-0'}:Number.isFinite(v)?v:{number:Number.isNaN(v)?'NaN':v>0?'Infinity':'-Infinity'};
const unscalar=v=>typeof v==='object'?v.number==='NaN'?NaN:v.number==='Infinity'?Infinity:v.number==='-0'?-0:-Infinity:v;
function value(v){
  if(v===null)return null;
  const space=v instanceof d3.rgb?'Rgb':v instanceof d3.hsl?'Hsl':v instanceof d3.lab?'Lab':v instanceof d3.hcl?'Hcl':'Cubehelix';
  return {space,channels:Object.fromEntries(Object.keys(v).map(k=>[k,scalar(v[k])]))};
}
const spaces=['rgb','hsl','lab','hcl','cubehelix'];
const methods=['copy','brighter','darker','rgb','displayable','formatHex','formatHex8','formatRgb','formatHsl','toString','hex'];
const parsedNames=[...fs.readFileSync(path.join(workspace,'node_modules/d3-color/src/color.js'),'utf8').matchAll(/^  ([a-z]+): 0x([0-9a-f]+),?$/gm)].map(m=>m[1]);
if(parsedNames.length!==148)throw Error('Unexpected upstream CSS name inventory');
const seeds=[];
function add(id,input){seeds.push({id,input});}
for(const css of parsedNames)add('name-'+css,{parse:css});
for(const [i,css] of ['transparent',' ReBeccAPurple\t','#fea2','#f00','#FfAABB','#ff00ff80','#0000','#ffffff00','#000000ff','rgb(10%,20%,30%)','rgba(1,2,3,0)','rgba(1,2,3,-.2)','rgb(-1,+256,3)','rgb(100%,0%,50%)','hsl(120,50%,20%)','hsla(120,50%,20%,0)','hsl(-360,0%,50%)','hsl(720,50%,100%)','hsl(30,50%,0%)','hsla(1e2,.5e2%,+50%,-1e-2)','rgba(+1,-2,03,+.5)','rgba(1%,2%,3%,.5)','\u00a0steelblue\ufeff'].entries())add('parse-'+i,{parse:css});
for(const [i,css] of ['', '#12','#12345','#1234567','#123456789','#ggg','currentColor','var(--color)','rgb(1 2 3)','rgb(1 2 3 / .5)','rgba(1,2,3)','rgb(1.5,2,3)','rgb(1e2,2,3)','rgb(1%,2,3%)','rgba(1,2,3,1.)','hsl(30deg,50%,50%)','hsl(30,50,50%)','hsl(30,50%,50%,1)','hsl(30,50 %,50%)','lab(50% 0 0)','oklch(50% .2 20)','none','rgb (1,2,3)','rgba(1,2,3,NaN)','red blue','#123\nred'].entries())add('invalid-'+i,{parse:css});
const direct={
 rgb:[[70,130,180],[25.5,51,76.5],[0,0,0],[255,255,255],[-.5,255.499999,128.5],[10,20,30,0],[10,20,30,.5],[1e-7,1e21,-1e21],[1,2,3,1e-7],[1,2,3,1e-6],[1,2,3,2**-25],[1,2,3,Number.MIN_VALUE]],
 hsl:[[120,.5,.2],[360,1,.5],[-360,1,.5],[-720,.4,.3],[765,1,.5],[45,0,.5],[NaN,.5,.2],[40,NaN,.4],[120,.5,0],[120,.5,1],[30,1.2,-.2,.5],[2**-25,1e-20,.5,2**-25]],
 lab:[[0,0,0],[100,0,0],[50,0,0],[50,30,40],[54.29173376861782,80.8124553179771,69.88504032350531],[50,NaN,NaN],[120,-200,200,.5]],
 hcl:[[0,0,0],[0,0,100],[NaN,0,50],[30,40,50],[390,40,50],[60,NaN,50],[NaN,100,50],[20,100,-10,.5]],
 cubehelix:[[0,0,0],[0,0,1],[NaN,0,.5],[30,.5,.5],[300,2,.3,.5],[NaN,1,.5],[120,NaN,.3]],
 gray:[[0],[50],[100],[50,.5]],lch:[[50,40,30],[50,0,NaN],[100,0,NaN],[50,40,390,.5]]
};
for(const [ctor,argsList]of Object.entries(direct))for(const [i,args]of argsList.entries())add(ctor+'-'+i,{constructor:ctor,args:args.map(scalar)});
for(const ctor of spaces){
 const normal=direct[ctor][Math.min(3,direct[ctor].length-1)].slice(0,3);normal.push(.5);
 for(let channel=0;channel<4;channel++)for(const [i,v]of [NaN,Infinity,-Infinity,-0].entries()){
  const args=normal.slice();args[channel]=v;add(`${ctor}-exception-${channel}-${i}`,{constructor:ctor,args:args.map(scalar)});
 }
 for(const css of ['steelblue','transparent','bogus','hsl(120,50%,20%)'])add(ctor+'-from-'+css,{constructor:ctor,css});
}
const cases=seeds.map(({id,input})=>{
 const color=input.parse!==undefined?d3.color(input.parse):input.css!==undefined?d3[input.constructor](input.css):d3[input.constructor](...input.args.map(unscalar));
 if(color===null)return {id,input,expected:null};
 const channels=Object.keys(color),copyPatch={[channels[0]]:13.25,opacity:.4};
 const expected={value:value(color),conversions:Object.fromEntries(spaces.map(s=>[s,value(d3[s](color))])),
   displayable:color.displayable(),formats:Object.fromEntries(['formatHex','formatHex8','formatRgb','formatHsl','toString','hex'].map(m=>[m,color[m]()])),
   copy:value(color.copy(copyPatch)),copyPatch,
   brightness:['brighter','darker'].flatMap(m=>[null,0,1,-1,.5,2,NaN,Infinity,-Infinity].map(k=>({method:m,k:k===null?null:scalar(k),value:value(k===null?color[m]():color[m](k))}))),
   clamp:typeof color.clamp==='function'?value(color.clamp()):null,
   sourceAfter:value(color)};
 return {id,input,expected};
});
writeJson(path.join(out,'cases.json'),{schema_version:1,requirement:'COL-01–06',reference:'d3-color 3.1.0',cases});
retainLicense('d3-color',out);
writeJson(path.join(out,'manifest.json'),{schema_version:1,...provenance('d3-color'),exports:Object.keys(d3).sort(),methods,spaceMethods:{Rgb:['clamp'],Hsl:['clamp']},cssNames:parsedNames,
 case_count:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),
 tolerances:{channels:{absolute:1e-10,relative:1e-10},tags_types_predicates_bytes_formats:'exact'},
 status:'Reference only; CLR-02–05 acceptance remains open.'});
console.log(`PASS FIX-C01 reference: ${cases.length} input cases, ${parsedNames.length} names, eight constructors, inherited methods and explicit exceptional tags.`);
