'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().padding(0)),keys=[9007199254741001n,9007199254741003n,9007199254741005n];
for(const clipped of [false,true]){
 const data=c.Data.columns({weight:[1,2,1],radius:[40,40,40]},{keys}),layer=c.shapePie().pieOrder('Input').shapeValue('PieValue',data.field('weight')).shapeValue('InnerRadius',20).shapeValue('OuterRadius',c.sourceExpr(data.field('radius')).mul(1));
 const p=c.plot(data).aes(c.aes().x(clipped?-.15:2).y(100)).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(c.scaleLog(10).domain(1,10000)).visible(false)).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,7);c.Plot.fromJson(wire).free();const chart=p.chart();p.free();const frame=chart.present(output,options),shapes=frame.scene().items.filter(i=>'ShapePath'in i.primitive).map(i=>i.primitive.ShapePath);assert.equal(shapes.length,3);assert.ok(shapes.every(s=>s.anchors.length===1));
 if(!clipped){assert.deepEqual(chart.inspect(200,100,{mode:'Containment'}).targets,[]);for(const [i,[x,y]]of [[222,78],[222,122],[178,78]].entries()){const hit=chart.inspect(x,y,{mode:'Containment'}).targets;assert.equal(hit.length,1);assert.equal(BigInt(hit[0].identity.Source.key),keys[i]);chart.focus(hit[0]);chart.present(output,options).free();}}
 else{const hit=chart.inspect(10,80,{mode:'Containment'}).targets;assert.equal(hit.length,1);assert.equal(BigInt(hit[0].identity.Source.key),keys[0]);chart.focus(hit[0]);chart.present(output,options).free();}
 assert.deepEqual(chart.inspect(-1,100,{mode:'Containment'}).targets,[]);chart.free();frame.free();
}
{
 const data=c.Data.columns({category:['a','b','b','c']}),layer=c.shapePie().stat(c.count().group('category')).afterStat(c.statAes().x(2).y(2)).pieGrouped(false).pieOrder('Input').shapeValue('PieValue',{Statistical:'Count'}),p=c.plot(data).layer(layer).build(),request=output.request(p,options),f=request.prepare(),targets=f.scene().targets.flat().filter(t=>'Aggregate'in t);assert.deepEqual(targets.map(t=>t.Aggregate.members.length),[1,2,1]);f.free();request.free();p.free();assert.throws(()=>c.shapePie().shapeValue('PieValue',{Statistical:'Count',unexpected:1}));
}
console.log('PASS WASM arc/pie interaction: exact slice identity, donut holes, clipped fill/focus, post-log center projection, native field/expression channels and v7 round trips.');
