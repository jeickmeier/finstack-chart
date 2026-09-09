'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/cartesian.json'))).cases;
for(const seed of JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/cases.json'))).cases){
 if(!['line','area'].includes(seed.family))continue;
 const config={...seed.settings},curve={kind:config.curve.slice(5)};
 for(const k of ['alpha','beta','tension'])if(k in config){curve[k]=config[k];delete config[k];}config.curve=curve;
 cases.push({id:seed.id,family:seed.family,input:seed.input,config,helper:null,finite:true,operations:seed.operations,svg:seed.svg_digits[3]});
}
function compare(a,b){
 if(typeof a==='number'&&typeof b==='number')assert.ok(Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);
 else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((x,i)=>compare(x,b[i]));}
 else if(a&&typeof a==='object'){assert.deepEqual(Object.keys(a),Object.keys(b));for(const k of Object.keys(a))compare(a[k],b[k]);}
 else assert.equal(a,b);
}
let negative=0;
for(const test of cases){
 let generator=new (test.family==='line'?c.ShapeLine:c.ShapeArea)(test.config);
 if(test.helper){const old=generator;generator=old.boundary(test.helper);old.free();}
 const copy=generator.copy();generator.free();
 if(!test.finite){assert.throws(()=>copy.generate(test.input),e=>e.code==='CHART_NUMERICAL_DOMAIN');negative++;copy.free();continue;}
 const p=copy.generate(test.input),original=p.result(),saved=p.copy(),expected=new c.Path(3);expected.applyBatch(test.operations);compare(original.geometry.commands,expected.result().geometry.commands);assert.equal(p.toSvg(),test.svg??'',test.id);
 const again=copy.generate(test.input);assert.deepEqual(again.result(),original);copy.free();p.moveTo(999,888);p.free();assert.deepEqual(saved.result(),original);again.free();saved.free();expected.free();
}
for(const [Type,config]of [[c.ShapeArea,{curve:{kind:'Bundle'}}],[c.ShapeLine,{curve:{kind:'Basis',tension:1}}],[c.ShapeLine,{x:{Column:-1}}]])assert.throws(()=>new Type(config));
console.log('PASS WASM Cartesian:',cases.length,'cases including',negative,'nonfinite-reference diagnostics; actual generator, boundary, copy, repeated output and independent Path ownership.');
