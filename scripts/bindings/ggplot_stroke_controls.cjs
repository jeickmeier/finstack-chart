'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<6;mode++){
 const data=c.Data.columns({x:new Float64Array([1,2,2.1,3]),y:new Float64Array([1,3,1,2])});
 const layer=c.line().linewidth(8).aestheticUnits('Points').alpha(.5).lineend(mode<3?['Butt','Round','Square'][mode]:'Butt').linejoin(mode<3?'Miter':['Miter','Round','Bevel'][mode-3]).lineType(mode<3?'Dashed':'Solid');
 const p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).build();
 const wire=p.toJson();assert.equal(JSON.parse(wire).version,74);const q=c.Plot.fromJson(wire),scenes=[];
 for(const current of [p,q]){const request=output.request(current,c.exportOptions(600,360).dpi(144)),frame=request.prepare();scenes.push(frame.scene());if(current===q)for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`stroke-${mode}.${fmt}`),frame.export(fmt));frame.free();request.free();}
 assert.deepEqual(scenes[0],scenes[1]);fs.writeFileSync(path.join(out,`stroke-${mode}.scene.json`),JSON.stringify(scenes[0]));fs.writeFileSync(path.join(out,`stroke-${mode}.plot.json`),wire);q.free();p.free();data.free();
}
output.free();console.log('PASS WASM stroke controls: six authors,18 publications,original/replay scenes.');
