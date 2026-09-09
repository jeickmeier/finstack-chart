'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/radial.json')));
function compare(a,b){
 if(typeof a==='number'&&typeof b==='number')assert.ok(Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);
 else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((x,i)=>compare(x,b[i]));}
 else if(a&&typeof a==='object'){assert.deepEqual(Object.keys(a),Object.keys(b));for(const k of Object.keys(a))compare(a[k],b[k]);}
 else assert.equal(a,b);
}
for(const p of corpus.points)compare(c.point_radial(p.angle,p.radius),p.point);
const types={lineRadial:c.ShapeLineRadial,areaRadial:c.ShapeAreaRadial,link:c.ShapeLink,linkRadial:c.ShapeLinkRadial};
let negative=0,unrounded=0;
for(const test of corpus.cases){
 let generator=new types[test.family](test.config);
 const decoded=new types[test.family](generator.config());assert.deepEqual(decoded.config(),generator.config());decoded.free();
 if(test.helper){const old=generator;generator=old.boundary(test.helper);old.free();}
 const copy=generator.copy();generator.free();
 if(!test.finite){assert.throws(()=>copy.generate(test.input),e=>e.code==='CHART_NUMERICAL_DOMAIN');negative++;copy.free();continue;}
 const p=copy.generate(test.input),original=p.result(),saved=p.copy(),expected=new c.Path(3);expected.applyBatch(test.operations);compare(original.geometry.commands,expected.result().geometry.commands);
 if(test.config.digits===null&&!test.helper){
   const numbers=s=>(s.match(/[-+]?(?:\d+\.?\d*|\.\d+)(?:e[-+]?\d+)?/gi)||[]).map(Number);
   compare(numbers(p.toSvg()),numbers(test.svg??''));unrounded++;
 }else assert.equal(p.toSvg(),test.svg??'',test.id);
 const again=copy.generate(test.input);assert.deepEqual(again.result(),original);copy.free();p.moveTo(999,888);p.free();assert.deepEqual(saved.result(),original);again.free();saved.free();expected.free();
}
for(const [Type,config]of [[c.ShapeAreaRadial,{curve:{kind:'Bundle'}}],[c.ShapeLineRadial,{x:{Column:0}}],[c.ShapeLinkRadial,{curve:{kind:'Linear'}}],[c.ShapeLink,{source:'Node'}]])assert.throws(()=>new Type(config));
assert.throws(()=>c.point_radial(NaN,1));assert.throws(()=>c.point_radial(0,Infinity));
const bounded=new c.ShapeLinkRadial({limits:{max_points:1}});assert.throws(()=>bounded.generate({source:[0,1],target:[2,3]}));bounded.free();
console.log('PASS WASM radial:',corpus.points.length,'points;',corpus.cases.length,'paths;',unrounded,'unrounded numerical comparisons;',negative,'nonfinite diagnostics; copy, disposal and retained output.');
