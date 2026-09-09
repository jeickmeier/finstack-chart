'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().padding(0)),keys=[9007199254741001n,9007199254741002n];
for(const area of [false,true]){
 const data=c.Data.columns({x:[0,4],y:area?[-10,-10]:[1,10000],x2:[0,4],y2:[10,10]},{keys});
 let p=c.plot(data).aes(c.aes().x('x').y('y').x2('x2').y2('y2')).layer(area?c.shapeArea():c.shapeLine().curve({kind:'BumpX'}).size(2));
 p=p.xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(area?c.scaleLinear().domain(0,4):c.scaleLog(10).domain(1,10000)).visible(false)).build();
 const wire=p.toJson();assert.equal(JSON.parse(wire).version,7);c.Plot.fromJson(wire).free();const chart=p.chart();p.free();const frame=chart.present(output,options),scene=frame.scene(),shape=scene.items.find(i=>'ShapePath'in i.primitive).primitive.ShapePath;
 assert.equal(shape.anchors.length,2);const hit=chart.inspect(200,100,{mode:'Containment'}).targets;assert.equal(hit.length,1);assert.ok(keys.includes(BigInt(hit[0].identity.Source.key)));assert.deepEqual(chart.inspect(200,-1,{mode:'Containment'}).targets,[]);
 if(!area){assert.deepEqual(shape.geometry.commands,[{MoveTo:[0,200]},{CubicTo:[200,200,200,0,400,0]}]);assert.deepEqual(chart.inspect(100,100,{mode:'Containment'}).targets,[]);assert.deepEqual(chart.selectRegion({Rectangle:[190,190,20,10]}).targets,[]);}
 else assert.deepEqual(chart.selectRegion({Rectangle:[0,0,5,5]}).targets,[]);
 chart.focus(hit[0]);chart.present(output,options).free();chart.free();frame.free();
}
console.log('PASS WASM presented shape containment, non-source controls, clipped interior focus, exact keys and v7 round trips.');
