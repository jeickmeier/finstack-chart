'use strict';
// CP-04 independently exercises every frozen sample through actual WASM owners.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-scale-chromatic/cases.json')));
const catalog=c.chromaticCatalog();assert.equal(catalog.version,1);assert.equal(catalog.schemes.length,38);assert.equal(catalog.interpolators.length,38);
catalog.schemes[0].sizes.push(999);assert(!c.chromaticCatalog().schemes[0].sizes.includes(999));
const hex=v=>{try{return v.formatHex8();}finally{v.free();}},expected=v=>'#'+v.map(x=>x.toString(16).padStart(2,'0')).join('');
for(const row of corpus.schemes){assert.deepEqual(c.chromaticScheme(row.name,row.size).map(hex),row.rgba.map(expected));assert.deepEqual(c.chromaticScheme(row.name,row.size,true).map(hex),row.rgba.toReversed().map(expected));assert.throws(()=>c.chromaticScheme(row.name,0));}
let n=0;
for(const row of corpus.ramps){
 const ramp=c.chromatic(row.name),wire=ramp.toJson(),copy=c.Interpolator.fromJson(wire),reverse=c.chromatic(row.name,true);
 assert.throws(()=>ramp.sample(null));for(const count of [0,1,200001])assert.throws(()=>ramp.quantize(count));
 const anchors=new Map(row.samples.filter(s=>[0,.5,1].includes(s[0])).map(s=>[s[0],s[2]]));assert.deepEqual(ramp.quantize(3).map(hex),[0,.5,1].map(t=>expected(anchors.get(t))));
 for(let [t,css,rgba] of row.samples){n++;if(typeof t==='object')t={NaN:NaN,Infinity:Infinity,'-Infinity':-Infinity,'-0':-0}[t.number];
  if(!Number.isFinite(t)||rgba===null){assert.throws(()=>ramp.sample(t));continue;}
  assert.equal(hex(ramp.sample(t)),expected(rgba),`${row.name} ${t}`);
 }
 for(const t of [0,.123456789,.5,1])assert.equal(hex(reverse.sample(t)),hex(copy.sample(1-t)));
 ramp.dispose();assert.equal(copy.toJson(),wire);assert.throws(()=>ramp.sample(.5));ramp.free();copy.free();reverse.free();
}
assert.equal(n,160666);
for(const call of [()=>c.chromatic('bad'),()=>c.chromaticScheme('Viridis'),()=>c.chromaticScheme('Blues'),()=>c.chromaticScheme('Category10',10),()=>c.chromaticScheme('Blues',3.5),()=>c.chromaticScheme('Blues',NaN),()=>c.chromatic('Blues','yes')])assert.throws(call);
fs.mkdirSync(path.dirname(path.resolve(process.argv[3])),{recursive:true});fs.writeFileSync(process.argv[3],JSON.stringify({schemes:218,samples:n,catalog:c.chromaticCatalog(),exact:'sRGB8 RGBA',ownership:'copies, disposal, reversal and isolated results'},null,2)+'\n');
console.log(`PASS CP-04 WASM: 218 exact scheme arrays and ${n} ramp rows, catalog metadata, reversal, invalid inputs, copies and disposal.`);
