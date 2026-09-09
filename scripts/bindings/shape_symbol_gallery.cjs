// Independent primary WASM symbol gallery across every built-in theme.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),kinds=['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','Asterisk','Diamond2','Square2','Triangle2'];
for(const preset of ['Editorial','Terminal','Grayscale']){
 const folder=path.join(out,preset);fs.mkdirSync(folder,{recursive:true});
 let draft=c.plot(c.Data.columns({x:[0,5],y:[0,15]})).aes(c.aes().x('x').y('y')).theme(c.theme().preset(preset)).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,5)).visible(false)).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,15)).visible(false)).title(c.title(`Symbols / area and stroke size / ${preset}`));
 for(const [i,kind]of kinds.entries()){
  const y=13.5-i,d=c.Data.columns({x:[2,3,4],y:[y,y,y],size:[16,64,256]},{name:`symbol-${i}`,keys:[9007199254741001n+BigInt(i*10),9007199254741003n+BigInt(i*10),9007199254741005n+BigInt(i*10)]});
  draft=draft.layer(c.shapeSymbol().name(`symbol-${i}`).data(d).symbolKind(kind).symbolPaint(i<7?'Fill':'Stroke').shapeValue('AreaSize','size').color(i<7?'#2162a8':'#b55037')).layer(c.labels().id(`symbol-label-${i}`).at(.1,y).text(kind).style(c.textStyle().size(.85)));
 }
 for(const [i,label]of ['16','64','256'].entries())draft=draft.layer(c.labels().id(`size-${i}`).at(2+i,14.4).text(label).style(c.textStyle().size(.85)));
 const d=c.Data.columns({x:[2,3,4],y:[.4,.4,.4],size:[16,64,256],kind:['A','B','C']},{name:'mapped'});
 draft=draft.layer(c.shapeSymbol().name('mapped').data(d).symbolTypes('kind',['A','B','C'],['Circle','Square','Plus']).symbolTitle('Type').shapeValue('AreaSize','size').symbolSizeGuide('Area',[16,64,256]).color('#2162a8')).layer(c.labels().id('mapped-label').at(.1,.4).text('Mapped').style(c.textStyle().size(.85)));
 const p=draft.build();fs.writeFileSync(path.join(folder,'figure.plot.json'),p.toJson());const requests=[300,600].map(dpi=>[dpi,output.request(p,c.exportOptions(600,740).dpi(dpi))]);p.free();
 for(const [dpi,request]of requests){const frame=request.prepare(),scene=frame.scene();assert.equal(scene.items.filter(i=>'ShapePath'in i.primitive).length,42);scene.items.forEach((item,i)=>{if('ShapePath'in item.primitive){assert.equal(item.primitive.ShapePath.anchors.length,1);assert.equal(scene.targets[i].length,1);}});fs.writeFileSync(path.join(folder,`figure-${dpi}.scene.json`),JSON.stringify(scene,null,2));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(folder,`figure-${dpi}.${fmt}`),frame.export(fmt));frame.free();request.free();}
}
console.log('PASS WASM symbol gallery: 13 types at three sizes, type/size guides and all three themes at 300/600 DPI; immutable exports after plot disposal.');
