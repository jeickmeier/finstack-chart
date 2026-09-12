'use strict';
// FIX-GG03: independently authored corrections/retention versus fresh WASM compilation.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(480,280).dpi(72).basis('current'),base=9007199254741000n,records=[];
function data(rows){return c.Data.columns({x:new Float64Array(rows.map(r=>r[1])),y:new Float64Array(rows.map(r=>r[2])),fill:c.categorical(rows.map(r=>r[3])),stroke:c.categorical(rows.map(r=>r[4])),alpha:new Float64Array(rows.map(r=>r[5])),area:new Float64Array(rows.map(r=>r[6])),width:new Float64Array(rows.map(r=>r[7])),panel:c.categorical(rows.map(r=>r[8]))},{keys:rows.map(r=>r[0]),name:'live'});}
function author(rows,facets){
 const d=data(rows),layer=c.shape_symbol().symbol_types('fill',['A','B'],[{Ggplot:21},{Ggplot:24}]).shape_value('AreaSize',d.field('area')).shape_value('StrokeWidth',d.field('width')).shape_value('Alpha',d.field('alpha'));
 let p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').fill('fill').fill_scale('inside').stroke('stroke').stroke_scale('outside')).layer(layer).scale(c.color_discrete('inside').domain(['A','B']).palette(['#ff0000','#0000ff'])).scale(c.color_discrete('outside').domain(['A','B']).palette(['#000000','#008000'])).x_axis(c.x_axis().scale(c.scale_linear().domain(0,10))).y_axis(c.y_axis().scale(c.scale_linear().domain(0,10)));
 if(facets)p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2));const result=p.build();d.free();layer.free();return result;
}
for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),i+1,i%3+2,i%2?'A':'B',i%3?'B':'A',.25*(i%4+1),16*(i%4+1)**2,i%3+1,i<4?'A':'B']);
 const p=author(rows,facets),chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented')),frame=old.prepare(),png=frame.export('png');frame.free();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`aesthetics-${facets}-${step}`);
  if(step===0){const row=[base+9n,9,7,'A','B',.7,576,4,'B'];rows.push(row);const batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){const row=[base+2n,2,6,'B','A',.9,36,.5,'A'];rows[1]=row;const batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else {rows=rows.slice(-5);tx=tx.retain_count('live',5);}
  tx=tx.build();assert('Applied' in chart.commit(tx));tx.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();
  const current=chart.request(output,options),freshPlot=author(rows,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();assert.deepEqual(a.export('png'),b.export('png'));
  const scene=a.scene(),marks=scene.items.filter(i=>i.layer!=null&&i.primitive.ShapePath).map(i=>i.primitive.ShapePath);assert.equal(marks.length,rows.length);assert(marks.every(m=>m.anchors.length===1));
  assert.deepEqual(scene.targets.flat().filter(t=>t.Source).map(t=>t.Source.key).sort(),rows.map(r=>r[0].toString()).sort());
  records.push({facets,step,marks});a.free();b.free();current.free();fresh.free();
 }
 old.free();chart.free();
}
fs.writeFileSync(out,JSON.stringify(records));output.free();console.log('PASS WASM GG-03: 8 append/correction/removal/retention states with independent fill/stroke/alpha/area/width, facets, exact fresh PNG and immutable presentation.');
