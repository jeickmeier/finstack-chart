'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);
fs.mkdirSync(path.dirname(out),{recursive:true});const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(480,260).dpi(72).basis('current'),base=9007199254741000n,records=[];
function data(rows){return c.Data.columns({x:Float64Array.from(rows.map(r=>r[1])),y:Float64Array.from(rows.map(r=>r[2])),x2:Float64Array.from(rows.map(r=>r[1]+.5)),y2:Float64Array.from(rows.map(r=>r[2]+2)),panel:c.categorical(rows.map(r=>r[3]))},{keys:rows.map(r=>r[0]),name:'live'});}
function author(rows,kind,area,facets){let p=c.plot(data(rows)).aes(c.aes().x('x').y('y').x2('x2').y2('y2')).layer((area?c.shapeArea():c.shapeLine()).curve({kind}).size(2)).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,10))).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,10)));if(facets)p=p.facet(c.facetWrap('panel').order([['A'],['B']]).columns(2));return p.build();}
for(const kind of ['Natural','CardinalClosed','MonotoneX','Step'])for(const area of [false,true])for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),i,i%3+1,i<4?'A':'B']);const p=author(rows,kind,area,facets),chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented')),frame=old.prepare(),png=frame.export('png');frame.free();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`shape-${step}`);
  if(step===0){const row=[base+9n,8,7,'B'];rows.push(row);const batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){const row=[base+2n,1,6,'A'];rows[1]=row;const batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else {rows=rows.slice(-5);tx=tx.retainCount('live',5);}
  tx=tx.build();assert.ok('Applied'in chart.commit(tx));tx.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();
  const current=chart.request(output,options),freshPlot=author(rows,kind,area,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();assert.deepEqual(a.export('png'),b.export('png'),`${kind} ${area} ${facets} ${step}`);
  for(const targets of a.scene().targets)for(const target of targets)if('Source'in target)assert.ok(rows.some(r=>r[0]===BigInt(target.Source.key)));
  a.free();b.free();current.free();fresh.free();
 }
 chart.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();old.free();records.push({kind,area,facets,updates:4,exact_keys:true,batch_png_equal:true,old_snapshot_retained:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS WASM Cartesian: 64 append/upsert/remove/retention checks across global/local curves, areas, facets and exact large keys; old exports retained.');
