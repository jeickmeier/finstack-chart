'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/cases.json'))).cases;assert.equal(cases.length,333);
for(const test of cases){
 let reference=null;
 for(const digits of [0,3,12,null]){
  const p=new c.Path(digits);p.applyBatch(test.operations);const commands=[];p.replay(v=>commands.push(v));if(reference===null)reference=commands;assert.deepEqual(commands,reference);
  if(digits!==null)assert.equal(p.toString(),test.svg_digits[digits]??'',`${test.id} digits=${digits}`);
  const copy=p.copy();p.moveTo(777,888);p.free();const saved=[];copy.replay(v=>saved.push(v));assert.deepEqual(saved,reference);copy.free();
 }
}
let draft=c.plot(c.Data.columns({x:[0,1],y:[0,1]})).aes(c.aes().x('x').y('y')).layer(c.points().size(.1)).xAxis(c.xAxis().visible(false)).yAxis(c.yAxis().visible(false)).title(c.title('Shape contexts / shared path foundation'));
const color={red:35,green:97,blue:166,alpha:255};
for(const [id,x,scale,filled,label] of [['arc-quarter',85,5,true,'Circular sector'],['arc-hole',235,5,true,'Annular hole'],['line-curveBasis-6',370,14,false,'External sink / Bezier']]){
 const test=cases.find(v=>v.id===id),p=new c.Path(3);p.applyBatch(test.operations);const sink=[];p.replay(v=>sink.push(v));
 const component=c.vectorPath(id,p).transform([scale,0,0,scale,0,0],.001,10000).anchor({Output:{x,y:140}}).fill(filled?color:null).stroke({color,width:1.5});p.moveTo(777,888);p.free();draft=draft.layer(component);component.free();
 draft=draft.layer(c.labels().id('label-'+id).outputAt(x-40,220).text(label).style(c.textStyle().size(.85)));
}
const plot=draft.build();fs.writeFileSync(path.join(out,'figure.plot.json'),plot.toJson());const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),requests=[300,600].map(dpi=>[dpi,output.request(plot,c.exportOptions(540,280).dpi(dpi))]);plot.free();
for(const [dpi,request] of requests){
 const frame=request.prepare(),scene=frame.scene();assert.equal(scene.items.filter(i=>'VectorPath'in i.primitive).length,3);scene.items.forEach((item,i)=>{if('VectorPath'in item.primitive)assert.deepEqual(scene.targets[i],[]);});
 fs.writeFileSync(path.join(out,`figure-${dpi}.scene.json`),JSON.stringify(scene,null,2));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`figure-${dpi}.${fmt}`),frame.export(fmt));frame.free();request.free();
}
console.log('PASS FIX-S01 WASM: 333 contexts, exact digits, precision-independent external replay/copy ownership and independent sector/hole/Bezier publication at 300/600 DPI.');
