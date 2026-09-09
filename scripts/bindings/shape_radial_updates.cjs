'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(480,260).dpi(72).basis('current'),base=9007199254741000n,corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/radial.json'))),curves=[...new Map(corpus.cases.filter(t=>t.family==='lineRadial').map(t=>{const curve=t.config.curve??{kind:'Linear'};return [JSON.stringify(curve),curve];})).values()],cases=['line','area'].flatMap(family=>curves.filter(curve=>family!=='area'||curve.kind!=='Bundle').map(curve=>[family,curve])).concat(['horizontal','vertical','step','radial'].map(family=>[family,{kind:'Linear'}]));
function data(rows){return c.Data.columns({angle:c.column(rows.map(r=>r[1]),{kind:'float64'}).nullable(true),radius:rows.map(r=>r[2]),panel:c.categorical(rows.map(r=>r[3]))},{keys:rows.map(r=>r[0]),name:'live'});}
function author(rows,family,curve,facets){
 const d=data(rows);let layer;
 if(family==='line'||family==='area'){layer=(family==='line'?c.shapeLineRadial():c.shapeAreaRadial()).curve(curve).shapeValue('Angle',d.field('angle'));layer=family==='line'?layer.shapeValue('Radius',d.field('radius')):layer.shapeValue('InnerRadius',4).shapeValue('OuterRadius',d.field('radius'));}
 else if(family==='radial')layer=c.shapeLinkRadial().shapeValue('StartAngle',d.field('angle')).shapeValue('EndAngle',1).shapeValue('InnerRadius',4).shapeValue('OuterRadius',d.field('radius'));
 else layer=({horizontal:c.shapeLinkHorizontal,vertical:c.shapeLinkVertical,step:()=>c.shapeLink({kind:'Step'})}[family])().aes(c.aes().x('angle').y('radius').x2(3).y2(40));
 let p=c.plot(d).aes(c.aes().x(2).y(100)).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4))).yAxis(c.yAxis().scale(c.scaleLog(10).domain(1,10000)));if(facets)p=p.facet(c.facetWrap('panel').order([['A'],['B']]).columns(2));return p.build();
}
for(const [family,curve]of cases)for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),i===3?null:i*.4,10+i,i<4?'A':'B']);const p=author(rows,family,curve,facets),chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented'));let f=old.prepare();const png=f.export('png');f.free();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`radial-${step}`);
  if(step===0){const row=[base+9n,3.2,22,'B'];rows.push(row);const batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){const row=[base+2n,.4,['horizontal','vertical','step'].includes(family)?12:-12,'A'];rows[1]=row;const batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else{rows=rows.slice(-5);tx=tx.retainCount('live',5);}
  tx=tx.build();assert.ok('Applied'in chart.commit(tx));tx.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();const current=chart.request(output,options),freshPlot=author(rows,family,curve,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();assert.ok(Buffer.from(a.export('png')).equals(Buffer.from(b.export('png'))),JSON.stringify([family,curve,facets,step]));for(const targets of a.scene().targets)for(const t of targets)if(t.Source)assert.ok(rows.some(r=>r[0]===BigInt(t.Source.key)));a.free();b.free();current.free();fresh.free();
 }
 chart.free();f=old.prepare();assert.deepEqual(f.export('png'),png);f.free();old.free();records.push({family,curve,facets,updates:4,batch_png_equal:true,exact_keys:true,old_snapshot_retained:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log(`PASS WASM radial/link updates: ${records.length*4} append/upsert/remove/retention comparisons, missing angles, signed-radius corrections, log centers, facets, exact keys and retained exports.`);
