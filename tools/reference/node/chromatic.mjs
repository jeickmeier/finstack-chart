// FIX-21: exact d3-scale-chromatic 3.1.0 catalog and sampled evaluation oracle.
import * as chromatic from 'd3-scale-chromatic';
import {color} from 'd3-color';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,retainLicense,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale-chromatic'));
const exports=Object.keys(chromatic).sort();
if(exports.length!==76)throw Error('Changed chromatic export inventory');
const scalar=x=>Number.isFinite(x)?Object.is(x,-0)?{number:'-0'}:x:{number:Number.isNaN(x)?'NaN':x>0?'Infinity':'-Infinity'};
const buffer=new ArrayBuffer(8),view=new DataView(buffer);
function adjacent(x,up){if(x===0)return up?Number.MIN_VALUE:-Number.MIN_VALUE;view.setFloat64(0,x);let bits=view.getBigUint64(0);bits+=((x>0)===up)?1n:-1n;view.setBigUint64(0,bits);return view.getFloat64(0);}
function bytes(css){const c=color(css);if(!c)return null;const r=c.rgb().clamp();return [r.r,r.g,r.b,Math.round(r.opacity*255)];}
const schemes=[];
for(const name of exports.filter(n=>n.startsWith('scheme'))){const values=chromatic[name];if(typeof values[0]==='string')schemes.push({name:name.slice(6),size:null,css:values,rgba:values.map(bytes)});else for(let k=0;k<values.length;k++)if(values[k])schemes.push({name:name.slice(6),size:k,css:values[k],rgba:values[k].map(bytes)});}
if(schemes.length!==218)throw Error('Changed scheme size inventory');
const ramps=[];
const special=[-Number.MAX_VALUE,-1e20,-1e6,-2.25,-1.25,-.25,-Number.MIN_VALUE,-0,0,.5,1,1.25,2.25,1e6,1e20,Number.MAX_VALUE,NaN,Infinity,-Infinity];
const lookup=new Set(['Viridis','Inferno','Magma','Plasma']);
const analytic=new Set(['Cividis','Turbo','CubehelixDefault','Warm','Cool','Rainbow','Sinebow']);
for(const name of exports.filter(n=>n.startsWith('interpolate'))){
 const id=name.slice(11),fn=chromatic[name],points=new Set(special);
 for(let i=0;i<=4096;i++)points.add(i/4096);
 const scheme=schemes.filter(s=>s.name===id).at(-1);
 const knots=lookup.has(id)?256:scheme?scheme.rgba.length-1:0;
 for(let i=0;i<=knots&&knots;i++){const t=i/knots;points.add(t);points.add(adjacent(t,false));points.add(adjacent(t,true));}
 // Independently find the first byte transition in each analytic 1/64 interval.
 // Adjacent binary64 sides expose exact output rounding without an epsilon waiver.
 if(analytic.has(id))for(let i=0;i<64;i++){
  let lo=i/64,hi=(i+1)/64;const initial=fn(lo);
  if(initial===fn(hi))continue;
  for(let step=0;step<60;step++){const mid=(lo+hi)/2;if(mid===lo||mid===hi)break;if(fn(mid)===initial)lo=mid;else hi=mid;}
  for(const t of [lo,hi,adjacent(lo,false),adjacent(hi,true)])points.add(t);
 }
 const samples=[...points].sort((a,b)=>Number.isNaN(a)?1:Number.isNaN(b)?-1:a-b).map(t=>{
  let css;try{css=fn(t);}catch(e){css={error:e.name};}
  return [scalar(t),css,typeof css==='string'?bytes(css):null];
 });
 ramps.push({name:id,kind:lookup.has(id)?'Lookup':analytic.has(id)?'Analytic':'Brewer',samples});
}
fs.mkdirSync(out,{recursive:true});
fs.writeFileSync(path.join(out,'cases.json'),JSON.stringify({schema_version:1,schemes,ramps})+'\n');
retainLicense('d3-scale-chromatic',out);
writeJson(path.join(out,'manifest.json'),{schema_version:1,requirements:['CHR-01','CHR-02','CHR-03','CHR-05','CHR-06'],references:['d3-scale-chromatic','d3-interpolate','d3-color','d3-scale'].map(provenance),exports,schemes:schemes.length,interpolators:ramps.length,samples:ramps.reduce((n,r)=>n+r.samples.length,0),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),comparison:'Exact RGBA; original CSS retained. Typed direct operations reject non-finite input; finite valid reference colors are required. Explicit reverse uses 1-t.',status:'CP-01 reference; implementation coverage remains open.'});
console.log(`PASS FIX-21: ${exports.length} exports, ${schemes.length} arrays, ${ramps.length} ramps, ${ramps.reduce((n,r)=>n+r.samples.length,0)} samples.`);
