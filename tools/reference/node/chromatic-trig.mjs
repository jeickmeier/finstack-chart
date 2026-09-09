// Independent Node/V8 trigonometric anchors for reproduced color byte transitions.
import fs from 'node:fs';import path from 'node:path';
import {root,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale-chromatic'));
const s=x=>Object.is(x,-0)?{number:'-0'}:Number.isFinite(x)?x:{number:Number.isNaN(x)?'NaN':x>0?'Infinity':'-Infinity'};
const inputs=new Set([-Number.MAX_VALUE,-1e20,-1e6,-0,0,1e6,1e20,Number.MAX_VALUE,NaN,Infinity,-Infinity]);
for(let i=-1024;i<=1024;i++)inputs.add(i/8);
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-scale-chromatic/cases.json')));
for(const row of corpus.ramps.filter(r=>['CubehelixDefault','Warm','Cool','Rainbow','Sinebow'].includes(r.name)))for(const q of row.samples){
 const t=q[0];if(typeof t!=='number'||!Number.isFinite(t))continue;
 if(row.name==='Sinebow'){const a=(.5-t)*Math.PI;for(const d of [0,Math.PI/3,Math.PI*2/3])inputs.add(a+d);}
 else {const h=row.name==='CubehelixDefault'?300+t*(-540):row.name==='Warm'?-100+t*180:row.name==='Cool'?260+t*(-180):360*((t<0||t>1)?t-Math.floor(t):t)-100;inputs.add((h+120)*(Math.PI/180));}
}
inputs.add(-0);
const cases=[...inputs].map(x=>[s(x),s(Math.sin(x)),s(Math.cos(x))]);cases.push([s(-0),s(Math.sin(-0)),s(Math.cos(-0))]);
writeJson(path.join(out,'trig.json'),{node:process.versions.node,v8:process.versions.v8,source:'https://github.com/v8/v8/blob/13.6.233/src/base/ieee754.cc',generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),comparison:'Exact binary64 values and exceptional/signed-zero tags',cases});
console.log(`PASS ${cases.length} pinned trigonometric anchors`);
