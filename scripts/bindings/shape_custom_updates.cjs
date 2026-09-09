'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});const records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(480,260).dpi(72).basis('current'),base=9007199254741000n;
const families=['line','area','radial_line','radial_area','link','symbol','pie','stack_bars','stack_area'],op=(name,parameters)=>({operation:{id:'example.'+name,version:'1'},parameters});
function data(rows){return c.Data.columns({x:new Float64Array(rows.map(r=>r[1])),y:new Float64Array(rows.map(r=>r[2])),angle:new Float64Array(rows.map(r=>r[1]*.5)),radius:new Float64Array(rows.map(r=>10+r[2]*4)),area:new Float64Array(rows.map(r=>64+r[2]*32)),panel:c.categorical(rows.map(r=>r[3])),group:c.categorical(rows.map(r=>r[4]))},{keys:rows.map(r=>r[0]),name:'live'});}
function author(rows,family,facets){const d=data(rows),registry=c.ShapeRegistry.example();let layer;
 if(['line','area','link'].includes(family)){layer=({line:c.shapeLine,area:c.shapeArea,link:c.shapeLinkHorizontal}[family])().aes(c.aes().x('x').y('y').x2(5).y2(0));if(family==='area')layer=layer.aes(c.aes().x('x').y('y').x2('x').y2(0));}
 else if(family.startsWith('radial_')){layer=(family==='radial_line'?c.shapeLineRadial():c.shapeAreaRadial()).aes(c.aes().x(2).y(2)).shapeValue('Angle','angle');layer=family==='radial_line'?layer.shapeValue('Radius','radius'):layer.shapeValue('InnerRadius',8).shapeValue('OuterRadius','radius');}
 else if(family==='symbol')layer=c.shapeSymbol().shapeValue('AreaSize','area').symbolSizeGuide('Area',[64,128,192]).shapeProtocol('Symbol',op('rectangle_symbol',{amount:4}));
 else if(family==='pie')layer=c.shapePie().pieGrouped(false).aes(c.aes().x(2).y(2)).shapeValue('PieValue','y').shapeProtocol('PieComparator',op('field_comparator',{field:'key'}));
 else layer=(family==='stack_bars'?c.bars():c.shapeArea()).position(c.shapeStack(['a','b'])).shapeProtocol('StackOrder',op('first_value_order',{})).shapeProtocol('StackOffset',op('shift_offset',{amount:.25}));
 if(['line','area','radial_line','radial_area','link'].includes(family))layer=layer.shapeProtocol('Curve',op('shift_curve',{amount:5}));
 let p=c.plot(d).withShapeRegistry(registry).aes(c.aes().x('x').x2('x').y('y').y2(0).group('group').color('group')).layer(layer.name('custom')).scale(c.colorDiscrete('group').domain(['a','b'])).xAxis(c.xAxis().scale(c.scaleLinear().domain(-1,6))).yAxis(c.yAxis().scale(c.scaleLinear().domain(-1,5)));
 if(facets)p=p.facet(c.facetWrap('panel').order([['A'],['B']]).columns(2));p=p.build();registry.free();return p;
}
for(const family of families)for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),Math.floor(i/2),1+(i%3)*.5,i<4?'A':'B',i%2===0?'a':'b']);const p=author(rows,family,facets),wire=p.toJson();assert.equal(JSON.parse(wire).version,9);const registry=c.ShapeRegistry.example(),restored=c.Plot.fromJson(wire,registry);registry.free();assert.equal(restored.toJson(),wire);restored.free();const chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented'));let f=old.prepare();const png=f.export('png');f.free();
 for(let step=0;step<4;step++){let tx=chart.transaction().id('custom-'+step),row,batch;
  if(step===0){row=[base+9n,4,2,'B','a'];rows.push(row);batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){row=[base+2n,0,2.5,'A','b'];rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else{rows=rows.slice(-5);tx=tx.retainCount('live',5);}
  tx=tx.build();assert.ok('Applied'in chart.commit(tx));tx.free();let saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();const current=chart.request(output,options),freshPlot=author(rows,family,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();assert.deepEqual(a.export('png'),b.export('png'),JSON.stringify({family,facets,step}));for(const targets of a.scene().targets)for(const t of targets)if(t.Source)assert.ok(rows.some(r=>r[0]===BigInt(t.Source.key)));a.free();b.free();current.free();fresh.free();
 }
 chart.free();f=old.prepare();assert.deepEqual(f.export('png'),png);f.free();old.free();records.push({family,facets,updates:4,batch_png_equal:true,exact_keys:true,old_snapshot_retained:true,wire_roundtrip:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS WASM custom chart protocols: 72 append/upsert/remove/retention comparisons, all registered families, facets, size guides, exact keys, registry ownership and retained exports.');
