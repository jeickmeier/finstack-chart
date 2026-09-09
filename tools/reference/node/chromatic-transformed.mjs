// FIX-21 finite source values whose log/power normalization yields NaN or infinity.
import * as d3 from 'd3-scale';
import * as c from 'd3-scale-chromatic';
import {color} from 'd3-color';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale-chromatic')),cases=[];
for(const name of Object.keys(c).filter(n=>n.startsWith('interpolate')).sort())for(const reverse of [false,true]){
 for(const [family,options,input] of [['sequential_log',{domain:[1,100]},0],['sequential_log',{domain:[100,1]},0],['sequential_log',{domain:[1,100]},-1],['sequential_pow',{domain:[1,100],exponent:2},1e300]]){
  const ramp=c[name],f=reverse?t=>ramp(1-t):ramp;
  const scale=(family==='sequential_log'?d3.scaleSequentialLog():d3.scaleSequentialPow().exponent(options.exponent)).domain(options.domain).interpolator(f);
  const css=scale(input),value=color(css)?.rgb().clamp();
  cases.push({family,options,input,ramp:name.slice(11),reverse,reference_css:css??null,expected:value?[value.r,value.g,value.b,Math.round(value.opacity*255)]:null});
 }
}
fs.mkdirSync(out,{recursive:true});writeJson(path.join(out,'transformed.json'),{schema_version:1,cases,policy:'Finite source values preserve valid reference colors after exceptional normalization; invalid reference outputs diagnose. Direct non-finite ramp input still rejects; non-finite chart observations still use missing paint.'});
writeJson(path.join(out,'transformed-manifest.json'),{schema_version:1,references:['d3-scale','d3-scale-chromatic','d3-color'].map(provenance),cases:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'transformed.json')))});
console.log('PASS FIX-21 transformed parameters',cases.length,'cases');
