// FIX-S06: independent built-in symbol geometries, palettes and source aliases.
import * as d3 from 'd3-shape';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/shapes'));
const names=['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','Asterisk','Diamond2','Square2','Triangle2'];
class Context {
 constructor(){this.operations=[];}
 moveTo(...args){this.operations.push({op:'moveTo',args});}
 lineTo(...args){this.operations.push({op:'lineTo',args});}
 arc(x,y,r,a0,a1,anticlockwise=false){this.operations.push({op:'arc',args:{x,y,r,a0,a1,anticlockwise}});}
 rect(...args){this.operations.push({op:'rect',args});}
 closePath(){this.operations.push({op:'closePath'});}
}
const cases=[];
for(const kind of names)for(const size of [0,1e-20,.125,1,8,14,21,64,256,10000,1e300,-1]){
 const type=d3['symbol'+kind],g=d3.symbol(type,size),context=new Context();let error=null;
 try{g.context(context)();}catch(e){error=String(e);}
 const svg={};for(const digits of [0,3,12,null])try{svg[String(digits)]=g.context(null).digits(digits)();}catch(e){svg[String(digits)]={error:String(e)};}
 const finite=context.operations.every(o=>!o.args||Object.values(o.args).every(v=>typeof v!=='number'||Number.isFinite(v)));
 cases.push({id:`symbol-${kind}-${size}`,kind,size,finite,error,operations:context.operations,svg});
}
const nameOf=t=>names.find(n=>d3['symbol'+n]===t),palettes={fill:d3.symbolsFill.map(nameOf),stroke:d3.symbolsStroke.map(nameOf)};
const aliases={symbols:d3.symbols===d3.symbolsFill,symbolX:d3.symbolX===d3.symbolTimes};
const defaultSvg=d3.symbol()();
writeJson(path.join(out,'symbol.json'),JSON.parse(JSON.stringify({schema_version:1,cases,palettes,aliases,default_svg:defaultSvg},(_,v)=>typeof v==='number'&&!Number.isFinite(v)?{number:String(v)}:v)));
writeJson(path.join(out,'symbol-manifest.json'),{schema_version:1,references:['d3-shape','d3-path'].map(provenance),cases:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'symbol.json')))});
console.log('PASS independent symbol oracle:',cases.length,'cases; implementation acceptance is separate.');
