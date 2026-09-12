'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-secondary.json'))).cases,records=[];
const key=v=>v===null?'Null':{Text:v},semantic=v=>v===null?{MissingCategory:null}:{Category:v};
for(const [index,testcase] of cases.entries())for(const family of ['band','point']){
 const owned=[];
 try{
  const values=testcase.inputs,control=testcase.control,expected=testcase.result;
  const data=c.Data.columns({x:c.categorical(values.map(v=>v??'')).validity(values.map(v=>v!==null)),y:c.column(values.map(()=>1),{kind:'float64'})});owned.push(data);
  let primary=c.xAxis().scale(family==='band'?c.scaleBand():c.scalePoint()).range(100,540).discretePolicy({limits:testcase.limits?.map(key)??null,na_translate:testcase.na_translate}).guideGeometry({labels:'Preserve'});
  if(['primary_breaks','primary_labels','primary_hidden'].includes(control)){const guide=control==='primary_hidden'?{labels:'Hidden'}:{breaks:[{Text:'b'},{Text:'a'}]};if(control==='primary_labels')guide.labels={Explicit:['Bee','Aye']};primary=primary.discretePolicy({limits:testcase.limits?.map(key)??null,na_translate:testcase.na_translate,guide});}
  let secondary=c.xAxis().name('secondary').side('Top').secondary('x',control==='transformed'?2:1,0).guideGeometry({labels:'Preserve'});
  if(control==='empty')secondary=secondary.tickValues([]);
  if(control==='numeric')secondary=secondary.tickValues([-1,0,.5,1,1.5,2,3,4,5].map(v=>({Number:v})));
  if(['character','explicit','bad_labels'].includes(control))secondary=secondary.tickValues((control==='character'?['b',null,'NA','absent','a']:control==='explicit'?['b',null,'NA','a']:['b','a']).map(semantic));
  if(['explicit','bad_labels','hidden'].includes(control))secondary=secondary.tickFormat({Labels:control==='explicit'?['Bee','Missing','Literal','A']:control==='bad_labels'?['wrong']:Array(expected.breaks?.length??0).fill('')});
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(primary).xAxis(secondary).build();owned.push(plot);
  const restored=c.Plot.from_json(plot.to_json());owned.push(restored);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!expected.error);
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Top').ticks;
  assert.equal(ticks.length,expected.breaks.length,`${index}`);
  ticks.forEach((tick,i)=>{assert.ok(Math.abs(tick.value.Number-expected.breaks[i])<1e-12);assert.equal(tick.label,['hidden','primary_hidden'].includes(control)?'':(expected.labels[i]??'NA'));assert.ok(Math.abs((tick.position-100)/440-expected.positions[i])<1e-12,`${index}: ${tick.position} ${expected.positions[i]}`);});
  records.push({index,family,ticks});
  if([11,14,16].includes(index)&&family==='band')for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`discrete-secondary-${control}.${format}`),frame.export(format));
 }catch(error){assert.ok(error instanceof c.ChartError&&testcase.result.error,`${index}: ${error.stack}`);records.push({index,family,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,352);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 352 discrete secondary configurations and nine publication files.');
