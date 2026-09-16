'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const d=c.Data.columns({x:new Float64Array([.5,1.5]),w:new Float64Array([2,-3])}),records=[];
for(const kind of ['bin','count'])for(const expression of [false,true]){
 const weight=expression?c.sourceExpr(d.field('w')):d.field('w');
 const stat=kind==='bin'?c.bin().x('x').breaks([0,1,2]).ggplotBin({}).binWeight(weight):c.count().x('x').ggplotCount().countWeight(weight);
 const p=c.plot(d).layer(c.points().stat(stat)).build(),q=c.Plot.fromJson(p.toJson());
 for(const current of [p,q]){const chart=current.chart(),rows=chart.semantics().layers[0].rows,values=kind==='bin'?rows.Binned.map(r=>r.statistics.count):rows.Statistical.map(r=>r.values.find(v=>v.field==='WeightedCount').value);assert.deepEqual(values,[2,-3]);records.push(values);chart.free();}
 p.free();q.free();
}
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));d.free();console.log('PASS owned field/expression signed weights through actual host and replay.');
{
const data=c.Data.columns({x:new Float64Array([1,1,1]),y:new Float64Array([2,2,2]),v:new Float64Array([-1,1,2])});
const scale={training:'Eligible',function:{GgplotNumericIdentity:{transform:null,limits:null,guide:false,trained:null}}};
const p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points().recipe({Count:{}}).numericScale('Size',c.sourceExpr(data.field('v')).abs(),scale)).build(),q=c.Plot.fromJson(p.toJson()),expressionRecords=[];
for(const current of [p,q]){const chart=current.chart(),rows=chart.semantics().layers[0].rows.Statistical;assert.equal(rows.length,2);const values=rows.map(r=>[r.retained_numeric[0][1],r.values.find(v=>v.field==='WeightedCount').value]).sort((a,b)=>a[0]-b[0]);assert.deepEqual(values,[[1,2],[2,1]]);expressionRecords.push(values);chart.free();}
fs.writeFileSync(path.join(out,'count-expression.json'),JSON.stringify(expressionRecords));q.free();p.free();data.free();console.log('PASS implicit Count expression-result grouping and replay.');
}
