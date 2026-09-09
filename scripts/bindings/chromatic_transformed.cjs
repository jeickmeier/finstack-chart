'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-scale-chromatic/transformed.json'))).cases,output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(320,160);let errors=0;
for(const test of cases){
 const ramp=c.chromatic(test.ramp,test.reverse),scale=new c.StandaloneScale(test.family,{...test.options,interpolator:ramp}),expected=test.expected;
 if(expected===null){assert.throws(()=>scale.map(test.input),c.ChartError);errors++;}
 else{const result=scale.map(test.input);assert.equal(result.formatHex8(),'#'+expected.map(n=>n.toString(16).padStart(2,'0')).join(''),JSON.stringify(test));result.free();}
 const p=c.plot(c.Data.columns({x:[0],value:new Float64Array([test.input])})).aes(c.aes().x('x').y(1).color('value').colorScale('named')).layer(c.points()).scale(c.colorMapped('named',scale)).build(),request=output.request(p,options);
 if(expected===null)assert.throws(()=>request.prepare(),c.ChartError);
 else{const frame=request.prepare(),actual=frame.scene().items.filter(i=>'Point'in i.primitive&&i.layer!==null).map(i=>{const v=i.primitive.Point.fill;return [v.red,v.green,v.blue,v.alpha];});assert.deepEqual(actual,[expected],JSON.stringify(test));frame.free();}
 for(const v of [request,p,scale,ramp])v.free();
}
assert.equal(cases.length,304);assert.equal(errors,12);fs.writeFileSync(process.argv[3],JSON.stringify({cases:304,expected_diagnostics:errors,surfaces:['standalone','actual scene'],exact:'RGBA'},null,2)+'\n');
console.log('PASS FIX-21 WASM: 304 exceptional normalization cases; valid reference colors and twelve explicit undefined-color diagnostics.');
