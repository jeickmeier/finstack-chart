'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<5;mode++){
 const d=c.Data.columns({x:new Float64Array([1,1,2,2,4,4]),y:new Float64Array([1,3,4,8,2,6]),w:new Float64Array([2,-1,0,3,0,-2]),g:['A','A','A','A','B','B']});
 let stat=mode===0?c.count().ggplotCount().x('x').group('g').countWeight(d.field('w')):c.summary().x('x').y('y').group('g').summaryHelper({MeanSe:{mult:1}});
 if(mode===2||mode===3)stat=stat.summaryBins({breaks:[0,2,5],bins:30,options:{closed:mode===2?'Right':'Left'}});
 if(mode===4)stat=stat.summaryHelper({MeanClBootSeeded:{}});
 const p=c.plot(d).layer(c.points().stat(stat)).xAxis(c.xAxis().scale(c.scaleLinear().domain(-1,6))).yAxis(c.yAxis().scale(c.scaleLinear().domain(-4,12))).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,72);const restored=c.Plot.fromJson(wire);d.free();const scenes=[];
 for(const candidate of [p,restored]){const request=output.request(candidate,c.exportOptions(600,360)),frame=request.prepare();scenes.push(frame.scene());if(candidate===restored)for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`summary-${mode}.${fmt}`),frame.export(fmt));frame.free();request.free();}
 assert.deepEqual(scenes[0],scenes[1]);fs.writeFileSync(path.join(out,`summary-${mode}.scene.json`),JSON.stringify(scenes[0]));fs.writeFileSync(path.join(out,`summary-${mode}.plot.json`),wire);p.free();restored.free();
}
output.free();
const expected=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/count-summary-controls.json'))).extra.kept_limits;
const d=c.Data.columns({x:new Float64Array([1,2,10])}),p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(c.histogram().stat(c.bin().bins(3).ggplotBin({}))).xAxis(c.xAxis().numericLimits([0,5]).oob('Keep')).build(),q=c.Plot.fromJson(p.toJson()),records=[];
for(const current of [p,q]){const chart=current.chart(),rows=chart.semantics().layers[0].rows.Binned;assert.equal(rows.length,expected.length);const record=rows.map((row,i)=>{const actual={xmin:row.start,xmax:row.end,count:row.statistics.count};for(const [key,value]of Object.entries(actual))assert.ok(Math.abs(value-expected[i][key])<1e-12);return actual;});records.push(record);chart.free();}
assert.deepEqual(records[0],records[1]);fs.writeFileSync(path.join(out,'kept-limits.json'),JSON.stringify(records));q.free();p.free();d.free();console.log('PASS explicit limits with OOB Keep: original/replay numeric R sentinel.');
console.log('PASS WASM five count/summary authors, original/replay scene equality and 15 publications.');

const reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/count-summary-controls.json'))).extra;
for(const mode of ['fixed','free_x','free_y','overlay']){
 const data=c.Data.columns(mode==='overlay'?{y:new Float64Array([1,2])}:{y:new Float64Array([0,1,100,101]),g:['A','A','B','B']});
 let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().y('y')).layer(c.histogram().orientation('Horizontal').stat(c.bin().x('y').bins(3).ggplotBin({}))),overlay;
 if(mode==='overlay'){overlay=c.Data.columns({x:new Float64Array([1,1]),y:new Float64Array([0,10])},{name:'overlay'});builder=builder.layer(c.points().data(overlay).aes(c.aes().x('x').y('y')));}
 else builder=builder.facet(c.facetWrap('g').freeX(mode==='free_x').freeY(mode==='free_y'));
 const original=builder.build(),replay=c.Plot.fromJson(original.toJson()),records=[];
 for(const current of [original,replay]){
  const chart=current.chart(),semantic=chart.semantics(),panels=semantic.panels.length?semantic.panels:[semantic],rows=panels.flatMap(panel=>panel.layers[0].rows.Binned),expected=reference['horizontal_'+mode];assert.equal(rows.length,expected.length);
  const record=rows.map((row,i)=>{const actual={ymin:row.start,ymax:row.end,count:row.count};for(const [key,value]of Object.entries(actual))assert.ok(Math.abs(Number(value)-expected[i][key])<1e-12,`${mode} ${key}: ${value} != ${expected[i][key]}`);return actual;});records.push(record);chart.free();
 }
 assert.deepEqual(records[0],records[1]);fs.writeFileSync(path.join(out,'horizontal-'+mode+'.json'),JSON.stringify(records));replay.free();original.free();data.free();if(overlay)overlay.free();
}
console.log('PASS horizontal physical-axis training: fixed/free_x/free_y facets and mixed-orientation overlay, original/replay R sentinels.');
