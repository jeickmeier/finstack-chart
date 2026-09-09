'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const labels=['Linear spiral','Closed basis','Radial area','Smooth annulus','Signed radii','Defined gaps','Horizontal links','Vertical links','Step links','Radial links'];
for(const preset of ['Editorial','Terminal','Grayscale']){
 const folder=path.join(out,preset);fs.mkdirSync(folder,{recursive:true});const rows=[];
 for(const [slot,label]of labels.entries())for(let i=0,n=slot>=6?3:slot===4?4:13;i<n;i++){
  const cartesian=slot>=6&&slot<=8,a=i*Math.PI/6;
  rows.push({x:cartesian?.5:2,y:cartesian?.75+i*.5:2,x2:3.5,y2:3.25-i*.5,angle:slot===5&&i===6?null:a,radius:slot===4?[24,-24,32,-12][i]:20+i,inner:8+i%3,end:a+1.5,outer:32,group:slot>=6?'ABC'[i]:'A',panel:label,slot,key:9007199254741001n+BigInt(slot*100+i)});
 }
 const columns=Object.fromEntries(['x','y','x2','y2','angle','radius','inner','end','outer'].map(name=>[name,c.column(rows.map(r=>r[name]),{kind:'float64'})]));Object.assign(columns,{group:c.categorical(rows.map(r=>r.group)),panel:c.categorical(rows.map(r=>r.panel)),slot:rows.map(r=>r.slot)});
 const data=c.Data.columns(columns,{keys:rows.map(r=>r.key),name:'radial-links'});
 let draft=c.plot(data).aes(c.aes().x('x').y('y').x2('x2').y2('y2').group('group').color('group')).theme(c.theme().preset(preset)).title(c.title(`Radial shapes and links / ${preset}`)).subtitle(c.subtitle('Clockwise angles, signed radii, gaps and source edges')).facet(c.facetWrap('panel').columns(2).gap(12)).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).scale(c.colorDiscrete('group').domain(['A','B','C'])).legend(c.legend().scale('group').title('Source'));
 for(const [slot,label]of labels.entries()){
  let layer;
  switch(slot){case 0:layer=c.shapeLineRadial();break;case 1:layer=c.shapeLineRadial().curve({kind:'BasisClosed'});break;case 2:layer=c.shapeAreaRadial();break;case 3:layer=c.shapeAreaRadial().curve({kind:'Basis'});break;case 4:layer=c.shapeLineRadial().curve({kind:'Cardinal',tension:.2});break;case 5:layer=c.shapeLineRadial().curve({kind:'Step'});break;case 6:layer=c.shapeLinkHorizontal();break;case 7:layer=c.shapeLinkVertical();break;case 8:layer=c.shapeLink({kind:'Step'});break;default:layer=c.shapeLinkRadial();}
  if(slot<6){layer=layer.shapeValue('Angle',data.field('angle'));layer=slot===2||slot===3?layer.shapeValue('InnerRadius',data.field('inner')).shapeValue('OuterRadius',data.field('radius')):layer.shapeValue('Radius',data.field('radius'));}
  if(slot===9)layer=layer.shapeValue('StartAngle',data.field('angle')).shapeValue('EndAngle',data.field('end')).shapeValue('InnerRadius',data.field('inner')).shapeValue('OuterRadius',data.field('outer'));
  draft=draft.layer(layer.name(label).filter(c.filter('slot').minimum(slot).maximum(slot)));
 }
 const p=draft.build();fs.writeFileSync(path.join(folder,'figure.plot.json'),p.toJson());const requests=[300,600].map(dpi=>[dpi,output.request(p,c.exportOptions(600,740).dpi(dpi))]);p.free();
 for(const [dpi,request]of requests){const f=request.prepare(),scene=f.scene();assert.equal(scene.items.filter(i=>'ShapePath'in i.primitive).length,19);assert.equal(scene.targets.flat().length,92);scene.items.forEach((i,n)=>{if('ShapePath'in i.primitive)assert.equal(i.primitive.ShapePath.anchors.length,scene.targets[n].length);});fs.writeFileSync(path.join(folder,`figure-${dpi}.scene.json`),JSON.stringify(scene,null,2));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(folder,`figure-${dpi}.${fmt}`),f.export(fmt));f.free();request.free();}
}
console.log('PASS WASM radial/link gallery: ten facets, 80 source identities, 92 real endpoint/source anchors, three themes and immutable 300/600 DPI outputs.');
