'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);
fs.mkdirSync(path.dirname(out),{recursive:true});const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(480,260).dpi(72).basis('current'),base=9007199254741000n,records=[];
function data(rows){return c.Data.columns({x:Float64Array.from(rows.map(r=>r[1])),y:Float64Array.from(rows.map(r=>r[2])),x2:Float64Array.from(rows.map(r=>r[1]+.5)),y2:Float64Array.from(rows.map(r=>r[2]+2)),panel:c.categorical(rows.map(r=>r[3])),slice:c.categorical(rows.map(r=>String(r[0])))},{keys:rows.map(r=>r[0]),name:'live'});}
// Fixed authored catalog matches live and fresh input semantics; removed categories are retained live.
function author(rows,kind,pie,facets){
 const [inner,corner,pad,order]={Solid:[0,0,0,'Input'],Donut:[12,0,0,'ValuesDescending'],Rounded:[10,5,.1,'ValuesAscending'],Reverse:[12,4,.04,'Input']}[kind];
 let layer=pie?c.shapePie().pieOrder(order).pieAngles({start_angle:kind==='Reverse'?1:0,end_angle:kind==='Reverse'?-4:Math.PI*2,pad_angle:pad}).shapeValue('PieValue','y'):c.shapeArc().shapeValue('StartAngle',kind==='Reverse'?6:0).shapeValue('EndAngle','y').shapeValue('PadAngle',pad);
 layer=layer.shapeValue('InnerRadius',inner).shapeValue('OuterRadius',28).shapeValue('CornerRadius',corner);
 let p=c.plot(data(rows)).aes(c.aes().x(pie?5:'x').y(5).color('slice').colorScale('slice-colors')).layer(layer).scale(c.colorDiscrete('slice-colors').domain(Array.from({length:9},(_,i)=>String(base+BigInt(i+1))))).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,10))).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,10)));
 if(facets)p=p.facet(c.facetWrap('panel').order([['A'],['B']]).columns(2));return p.build();
}
for(const kind of ['Solid','Donut','Rounded','Reverse'])for(const pie of [false,true])for(const facets of [false,true]){
 let rows=Array.from({length:8},(_,i)=>[base+BigInt(i+1),i,i%3+1,i<4?'A':'B']);const p=author(rows,kind,pie,facets),chart=p.chart();chart.present(output,options).free();p.free();const old=chart.request(output,options.basis('presented')),frame=old.prepare(),png=frame.export('png');frame.free();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`shape-${step}`);
  if(step===0){const row=[base+9n,8,7,'B'];rows.push(row);const batch=data([row]);tx=tx.append('live',batch);batch.free();}
  else if(step===1){const row=[base+2n,1,6,'A'];rows[1]=row;const batch=data([row]);tx=tx.upsert('live',batch);batch.free();}
  else if(step===2){rows.shift();tx=tx.remove('live',[base+1n]);}
  else {rows=rows.slice(-5);tx=tx.retainCount('live',5);}
  tx=tx.build();assert.ok('Applied'in chart.commit(tx));tx.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();
  const current=chart.request(output,options),freshPlot=author(rows,kind,pie,facets),fresh=output.request(freshPlot,options);freshPlot.free();const a=current.prepare(),b=fresh.prepare();const actualPng=a.export('png'),expectedPng=b.export('png');if(!Buffer.from(actualPng).equals(Buffer.from(expectedPng))){const failure=path.join(path.dirname(out),'update-mismatch');fs.mkdirSync(failure,{recursive:true});fs.writeFileSync(path.join(failure,'actual.json'),JSON.stringify(a.scene(),null,2));fs.writeFileSync(path.join(failure,'expected.json'),JSON.stringify(b.scene(),null,2));fs.writeFileSync(path.join(failure,'actual.png'),actualPng);fs.writeFileSync(path.join(failure,'expected.png'),expectedPng);}assert.deepEqual(actualPng,expectedPng,`${kind} ${pie} ${facets} ${step}`);
  for(const targets of a.scene().targets)for(const target of targets)if('Source'in target)assert.ok(rows.some(r=>r[0]===BigInt(target.Source.key)));
  a.free();b.free();current.free();fresh.free();
 }
 chart.free();const saved=old.prepare();assert.deepEqual(saved.export('png'),png);saved.free();old.free();records.push({kind,pie,facets,updates:4,exact_keys:true,batch_png_equal:true,old_snapshot_retained:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS WASM arc/pie: 64 append/upsert/remove/retention checks across solid/donut/rounded/reversed marks, facets and exact large keys; old exports retained.');
