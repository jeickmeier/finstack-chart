'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const names=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/inventory.json'))).exports.filter(v=>v.kind==='curve').map(v=>v.name.slice(5));
let draft=c.plot(c.Data.columns({x:[0,32],y:[0,20]})).aes(c.aes().x('x').y('y')).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,32)).visible(false)).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,20)).visible(false)).title(c.title('Cartesian curves / projected lines and general areas'));
const xy=[[0,0],[1,2],[2,-1],[3,3],[4,1],[6,2]];
for(const [i,name]of names.entries()){
 const curve={kind:name},ox=i%4*8+.8,oy=16-Math.floor(i/4)*4,data=c.Data.columns({x:xy.map(([x,y])=>ox+x),y:xy.map(([x,y])=>oy+2+y*.3)},{name:'line-'+name});
 draft=draft.layer(c.shapeLine().name('line-'+name).data(data).curve(curve).color('#2162a8').size(1.5)).layer(c.points().name('sources-'+name).data(data).color('#d85b24').size(1.5));
 if(name!=='Bundle'){
  const data=c.Data.columns({x:xy.map(([x,y])=>ox+x),y:xy.map(([x,y])=>oy+.25+y*.15),x2:xy.map(([x,y])=>ox+x+y*.2),y2:xy.map(([x,y])=>oy+.9+y*.2)},{name:'area-'+name});
  draft=draft.layer(c.shapeArea().name('area-'+name).data(data).aes(c.aes().x2('x2').y2('y2')).curve(curve).color('rgba(33,98,168,0.55)'));
 }
 draft=draft.layer(c.labels().id('label-'+name).at(ox,oy+3.6).text(name).style(c.textStyle().size(.85)));
}
const plot=draft.build();fs.writeFileSync(path.join(out,'figure.plot.json'),plot.toJson());const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),requests=[300,600].map(dpi=>[dpi,output.request(plot,c.exportOptions(680,640).dpi(dpi))]);plot.free();
for(const [dpi,request] of requests){const frame=request.prepare(),scene=frame.scene();assert.equal(scene.items.filter(i=>'ShapePath'in i.primitive).length,39);scene.items.forEach((item,i)=>{if('ShapePath'in item.primitive)assert.equal(item.primitive.ShapePath.anchors.length,scene.targets[i].length);});fs.writeFileSync(path.join(out,`figure-${dpi}.scene.json`),JSON.stringify(scene,null,2));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`figure-${dpi}.${fmt}`),frame.export(fmt));frame.free();request.free();}
console.log('PASS WASM Cartesian publication: 20 curves, 19 areas and retained source anchors at 300/600 DPI.');
