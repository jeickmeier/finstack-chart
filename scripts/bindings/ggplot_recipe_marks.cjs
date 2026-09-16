'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<5;mode++){
 const d=c.Data.columns({x:new Float64Array([1,1,2,3]),y:new Float64Array([1,1,-1,2]),end_x:new Float64Array([3,3,4,4]),end_y:new Float64Array([2,2,2,0]),angle:new Float64Array([0,0,Math.PI/2,Math.PI]),radius:new Float64Array([1,1,-1,2]),w:new Float64Array([1,2,1,4])});
 let layer;
 if(mode===0)layer=c.points().recipe({Count:{}}).stat(c.count().sumCount().x('x').y('y').countWeight('w'));
 else if(mode===1)layer=c.rectangle().recipe({Column:{width:.6,just:.5}});
 else if(mode===2)layer=c.points().recipe({Rug:{sides:'bltr',length:.04,outside:false}});
 else if(mode===3)layer=c.rule().aes(c.aes().x('x').y('y').x2('end_x').y2('end_y')).recipe({Curve:{curvature:.4,angle:60,ncp:5,arrow:{angle:30,length_mm:3,ends:'Last',closed:true}}});
 else layer=c.rule().recipe({Spoke:{}}).recipeValue('Angle','angle').recipeValue('Radius','radius');
 const p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,5))).yAxis(c.yAxis().scale(c.scaleLinear().domain(-2,4))).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,74);const restored=c.Plot.fromJson(wire);d.free();const scenes=[];
 for(const candidate of [p,restored]){const request=output.request(candidate,c.exportOptions(600,360)),frame=request.prepare();scenes.push(frame.scene());if(candidate===restored)for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`mark-${mode}.${fmt}`),frame.export(fmt));frame.free();request.free();}
 assert.deepEqual(scenes[0],scenes[1]);fs.writeFileSync(path.join(out,`mark-${mode}.scene.json`),JSON.stringify(scenes[0]));fs.writeFileSync(path.join(out,`mark-${mode}.plot.json`),wire);p.free();restored.free();
}
output.free();
console.log('PASS WASM five recipe mark authors, original/replay scene equality and 15 publications.');
