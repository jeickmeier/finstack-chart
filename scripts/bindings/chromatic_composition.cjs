'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-scale-chromatic/composition.json'))),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(320,160);let samples=0;
for(const test of corpus.cases){
 const s=test.selection,opts={...test.options};let ramp;
 if(s.ramp)opts.interpolator=ramp=c.chromatic(s.ramp,s.reverse);
 const scale=new c.StandaloneScale(test.family,opts);let mapping=c.colorMapped('named',scale).missing('#0b16212c');
 if(s.scheme)mapping=mapping.paletteScheme({id:s.scheme,size:s.size,reverse:s.reverse});
 const values=test.samples.map(s=>s.input&&typeof s.input==='object'?{NaN:NaN,Infinity:Infinity,'-Infinity':-Infinity}[s.input.number]:s.input);
 assert(test.samples.every(s=>s.expected!==null));
 const p=c.plot(c.Data.columns({x:values.map((_,i)=>i),value:values})).aes(c.aes().x('x').y(1).color('value').colorScale('named')).layer(c.points()).scale(mapping).build();
 const request=output.request(p,options),frame=request.prepare();const actual=frame.scene().items.filter(i=>'Point'in i.primitive&&i.layer!==null).map(i=>{const v=i.primitive.Point.fill;return [v.red,v.green,v.blue,v.alpha];});
 assert.deepEqual(actual,test.samples.map(s=>s.expected),test.id);samples+=actual.length;
 for(const v of [frame,request,p,scale])v.free();if(ramp)ramp.free();
}
assert.equal(samples,1268);fs.writeFileSync(process.argv[3],JSON.stringify({cases:90,samples,exact:'actual scene RGBA'},null,2)+'\n');
console.log('PASS FIX-21 WASM: 90 composed mappings / 1268 exact scene colors across all sequential/diverging/rank/classifier families.');
for(const [theme,mark,ends] of [['Editorial',[33,145,140],[[68,1,84],[253,231,37]]],['Terminal',[33,145,140],[[68,1,84],[253,231,37]]],['Grayscale',[121,121,121],[[21,21,21],[222,222,222]]]]){
 const ramp=c.chromatic('Viridis'),scale=new c.StandaloneScale('sequential',{domain:[0,1],interpolator:ramp});
 const p=c.plot(c.Data.columns({x:[.5]})).aes(c.aes().x('x').y(1).color('x').colorScale('named')).layer(c.points())
 .scale(c.colorMapped('named',scale)).legend(c.legend().scale('named')).theme(c.theme().preset(theme)).build();
 const request=output.request(p,options),frame=request.prepare(),scene=frame.scene().items,rgb=v=>[v.red,v.green,v.blue];
 const marks=scene.filter(i=>'Point'in i.primitive&&i.layer!==null).map(i=>rgb(i.primitive.Point.fill));
 const chips=scene.filter(i=>'Rectangle'in i.primitive&&i.clip!==null&&i.primitive.Rectangle.bounds.width===i.primitive.Rectangle.bounds.height).map(i=>rgb(i.primitive.Rectangle.fill));
 assert.deepEqual(marks,[mark],theme);assert.deepEqual(chips,ends,theme);for(const v of [frame,request,p,scale,ramp])v.free();
}
console.log('PASS FIX-21 WASM: editorial/terminal/grayscale preserve canonical evaluation and convert both marks and guide swatches.');
