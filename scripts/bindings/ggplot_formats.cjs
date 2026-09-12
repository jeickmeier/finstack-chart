'use strict';
// FIX-GG04 actual WASM date/duration consumers over pinned R records.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const options=c.exportOptions(1200,360).dpi(96).basis('current'),records=[];
for(const [kind,filename] of [['datetime','time-formats.json'],['duration','duration-formats.json']]){
 const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2',filename))).cases;
 for(const [index,testcase] of cases.entries()){
  let data,values,expected,axis;
  if(kind==='datetime'){
   const value=BigInt(testcase.microseconds);
   data=c.Data.columns({x:c.timestamps([value-1000000n,value+1000000n],'us','UTC'),y:[0,1]});
   values=[{Timestamp:{value:String(value),unit:'Microseconds'}}];expected=[testcase.label];axis=c.xAxis();
  }else{
   const seconds=testcase.seconds.map(Number);data=c.Data.columns({x:seconds,y:[0,1]});
   values=seconds.map(Number=>({Number}));expected=testcase.labels;axis=c.xAxis().scale(c.scaleDuration());
  }
  axis=axis.tickValues(values).tickFormat({GgplotTime:{pattern:testcase.pattern}}).guideGeometry({labels:'Preserve'});
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,17);
  const restored=c.Plot.fromJson(wire);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,options);
  if(expected.some(label=>label.includes('\t'))){
   assert.throws(()=>request.prepare(),error=>error.code==='CHART_MISSING_RESOURCE');
   records.push({kind,index,unsupported:'tab glyph'});
   for(const value of [request,restored,plot,data])value.dispose();continue;
  }
  const frame=request.prepare(),guides=frame.guides().guides.filter(g=>g.spec.side==='Bottom');
  assert.equal(guides.length,1);const ticks=guides[0].ticks;
  assert.deepEqual(ticks.map(t=>t.label),expected,`${kind} ${index}`);assert.deepEqual(ticks.map(t=>t.value),values,`${kind} ${index}`);
  records.push({kind,index,values,labels:expected});
  if((kind==='datetime'&&[0,21].includes(index))||(kind==='duration'&&index===1)){
   const name=kind+'-format-'+index;fs.writeFileSync(path.join(out,name+'.plot.json'),wire);
   fs.writeFileSync(path.join(out,name+'.guides.json'),JSON.stringify(frame.guides(),null,2));
   for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,name+'.'+fmt),frame.export(fmt));
  }
  for(const value of [frame,request,restored,plot,data])value.dispose();
 }
}
output.dispose();assert.equal(records.length,260);assert.equal(records.filter(record=>record.unsupported).length,14);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));
console.log('PASS WASM GG04: 246 R date/duration cases with exact values/v17/publication; 14 tab-glyph rejections.');
