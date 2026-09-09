'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().padding(0)),key=9007199254741001n;
for(const [kind,clipped]of [['Circle',false],['Plus',false],['Circle',true],['Star',false]]){
 const area=kind==='Star'?0:kind==='Circle'?Math.PI*25:256,d=c.Data.columns({area:[area]},{keys:[key]}),layer=c.shapeSymbol().symbolKind(kind).shapeValue('AreaSize',c.sourceExpr(d.field('area')).mul(1));
 const p=c.plot(d).aes(c.aes().x(clipped?-.02:2).y(100)).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(c.scaleLog(10).domain(1,10000)).visible(false)).build();assert.equal(JSON.parse(p.toJson()).version,7);c.Plot.fromJson(p.toJson()).free();const chart=p.chart();p.free();const frame=chart.present(output,options),shapes=frame.scene().items.filter(i=>'ShapePath'in i.primitive).map(i=>i.primitive.ShapePath);
 if(kind==='Star'){assert.equal(shapes.length,0);assert.deepEqual(chart.inspect(200,100,{mode:'Containment'}).targets,[]);}
 else{assert.equal(shapes.length,1);assert.equal(shapes[0].anchors.length,1);const [x,y]=clipped?[1,100]:kind==='Plus'?[210,100]:[204,100],hits=chart.inspect(x,y,{mode:'Containment'}).targets;assert.equal(hits.length,1);assert.equal(BigInt(hits[0].identity.Source.key),key);chart.focus(hits[0]);chart.present(output,options).free();if(kind==='Plus'){assert.equal(shapes[0].fill,null);assert.ok(shapes[0].stroke);assert.deepEqual(chart.inspect(207,107,{mode:'Containment'}).targets,[]);}else if(!clipped)assert.deepEqual(chart.inspect(206,100,{mode:'Containment'}).targets,[]);}
 assert.deepEqual(chart.inspect(-1,100,{mode:'Containment'}).targets,[]);frame.free();chart.free();
}
{
 const d=c.Data.columns({category:['a','b','b','c']}),layer=c.shapeSymbol().stat(c.count().group('category')).afterStat(c.statAes().x(2).y(2)).symbolGroups(['a','b','c'],['Circle','Square','Plus']).shapeValue('AreaSize',{Statistical:'Count'}),p=c.plot(d).layer(layer).build(),request=output.request(p,options),f=request.prepare(),targets=f.scene().targets.flat().filter(t=>'Aggregate'in t);assert.deepEqual(targets.map(t=>t.Aggregate.members.length),[1,2,1]);f.free();request.free();p.free();
}
{
 const d=c.Data.columns({area:[0,1,2]}),scale=new c.StandaloneScale('linear',{domain:[0,2],range:[16,256]}),layer=c.shapeSymbol().symbolKind('Square').numericScale('AreaSize',d.field('area'),scale).symbolSizeGuide('Input',[0,1,2]),p=c.plot(d).aes(c.aes().x(2).y(2)).layer(layer).build(),request=output.request(p,options),f=request.prepare(),scene=f.scene(),marks=scene.items.filter(i=>'ShapePath'in i.primitive).map(i=>i.primitive.ShapePath.geometry),guides=scene.items.filter(i=>'VectorPath'in i.primitive).map(i=>i.primitive.VectorPath.geometry);
 function width(g){const points=g.commands.flatMap(op=>Object.entries(op).filter(([name])=>name==='MoveTo'||name==='LineTo').map(([,p])=>p));return Math.max(...points.map(p=>p[0]))-Math.min(...points.map(p=>p[0]));}
 assert.equal(marks.length,3);assert.equal(guides.length,3);for(const [i,size]of [16,136,256].entries()){assert.ok(Math.abs(width(marks[i])-Math.sqrt(size))<1e-10);assert.ok(Math.abs(width(guides[i])-Math.sqrt(size))<1e-10);}f.free();request.free();p.free();scale.free();
}
console.log('PASS WASM symbols: exact keys, field/expression area, log centers, stroke holes, zero size, clipping/focus, generated count identities and actual nonidentity size-guide geometry.');
