// FIX-S09 symbol type/size corrections, facets and immutable export ownership.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(480,260).dpi(72).basis('current'),base=9007199254741000n,records=[],kinds=['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','Asterisk','Diamond2','Square2','Triangle2'];
function data(rows){return c.Data.columns({x:Float64Array.from(rows.map(r=>r[1])),y:Float64Array.from(rows.map(r=>r[2])),panel:c.categorical(rows.map(r=>r[3])),kind:c.categorical(rows.map(r=>r[4])),area:Float64Array.from(rows.map(r=>r[5]))},{keys:rows.map(r=>r[0]),name:'live'});}
function author(rows,facets){let p=c.plot(data(rows)).aes(c.aes().x('x').y('y')).layer(c.shapeSymbol().symbolTypes('kind',kinds,kinds).shapeValue('AreaSize','area').symbolSizeGuide('Area',[16,64,256])).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,10))).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,10)));if(facets)p=p.facet(c.facetWrap('panel').order([['A'],['B']]).columns(2));return p.build();}
for(const [offset,kind]of kinds.entries())for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),i,i%3+2,i<4?'A':'B',kinds[(offset+i)%13],16*(i%4+1)**2]);const p=author(rows,facets),chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented')),frame=old.prepare(),png=frame.export('png');frame.free();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`symbol-${step}`);
  if(step===0){const row=[base+9n,8,7,'B',kinds[(offset+5)%13],576];rows.push(row);const batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){const row=[base+2n,1,6,'A',kinds[(offset+9)%13],36];rows[1]=row;const batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else{rows=rows.slice(-5);tx=tx.retainCount('live',5);}
  tx=tx.build();assert('Applied'in chart.commit(tx));tx.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();
  const current=chart.request(output,options),freshPlot=author(rows,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();assert.deepEqual(a.export('png'),b.export('png'),`${kind} ${facets} ${step}`);for(const targets of a.scene().targets)for(const target of targets)if('Source'in target)assert(rows.some(r=>r[0]===BigInt(target.Source.key)));a.free();b.free();current.free();fresh.free();
 }
 chart.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();old.free();records.push({kind,facets,updates:4,type_and_area_corrections:true,batch_png_equal:true,exact_keys:true,old_snapshot_retained:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS WASM symbols: 104 append/upsert/remove/retention comparisons across 13 types and facets, exact large keys, type/area corrections and retained exports.');
