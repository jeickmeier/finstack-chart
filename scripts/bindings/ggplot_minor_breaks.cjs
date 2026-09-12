'use strict';
// FIX-GG04 actual numeric minor selection through WASM and immutable guide snapshots.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).basis('current'),records=[];
const close=(a,b)=>assert.ok(Math.abs(a-b)<=1e-11*Math.max(1,Math.abs(a),Math.abs(b)),`${a} != ${b}`);
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/minor-breaks.json')));
for(const [index,testcase] of fixture.cases.entries()){
 const owned=[];
 try{
  const values=({finite:[1,10],constant:[4,4],negative:[-4,10],zero_wide:[0,10]})[testcase.population];
  const data=c.Data.columns({x:values,y:[1,2]});owned.push(data);
  const scale=({identity:c.scaleLinear,sqrt:c.scaleSqrt,reverse:c.scaleReverse,log10:()=>c.scaleLog(10)})[testcase.transform]();
  let axis=c.xAxis().scale(scale).range(0,100);
  if(testcase.population==='zero_wide')axis=axis.expansion({mult:[1,1],add:[0,0]});
  const major=({regular:[1,2,4,10],descending:[10,4,2,1],outside:[-5,0,2,20],one:[4],empty:[],none:[]})[testcase.major];
  if(major!==undefined)axis=axis.tickValues(major);
  const minor=({auto:'Automatic',none:'Hidden',empty:{Numeric:[]},explicit:{Numeric:[{number:'-Infinity'},-2,0,.5,1,3,6,10,12,{number:'Infinity'},{number:'NaN'}]}})[testcase.minor];
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis.minorBreaks(minor)).build();owned.push(plot);
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,19);
  const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);
  const guide=frame.guides().guides.find(g=>g.spec.side==='Bottom'),ticks=guide.minor_ticks??[],expected=testcase.result;
  assert.ok(!expected.error);assert.equal(ticks.length,expected.minor_values.length,`${index}`);
  ticks.forEach((tick,i)=>{if(expected.minor_values[i]===null)assert.equal(tick.value,null);else close(tick.value.Number,expected.minor_values[i]);close(tick.position/100,expected.minor_positions[i]);});
  records.push({index,minor_ticks:ticks});
 }finally{for(const value of owned.reverse())value.dispose();}
}
const temporalOptions=c.exportOptions(640,360).basis('current').layout(c.layoutOptions().maxTicks(4096));
const temporal=['minor-time-widths.json','minor-time-auto.json','minor-time-values.json'].flatMap(name=>JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2',name))).cases);
for(const [index,testcase] of temporal.entries()){
 const owned=[];
 try{
  const kind=testcase.kind,multiplier=kind==='date'?86400000000:1000000;
  const values=kind==='duration'?testcase.inputs:c.timestamps(testcase.inputs.map(v=>BigInt(Math.round(v*multiplier))),'us','UTC');
  const data=c.Data.columns({x:values,y:[1,2]});owned.push(data);
  const scale=({date:c.scaleDate,datetime:c.scaleUtc,duration:c.scaleDuration})[kind]();
  let policy=testcase.width!==undefined?{TimeWidth:testcase.width}:'Automatic';
  if(testcase.minor_values!==undefined)policy={Timestamps:testcase.minor_values.map(v=>v===null?null:{Timestamp:{value:String(Math.round(v*multiplier)),unit:'Microseconds'}})};
  let axis=c.xAxis().scale(scale).range(0,100).minorBreaks(policy);
  if(testcase.major!==undefined){
   const ratios=({regular:[0,.25,.75,1],descending:[1,.75,.25,0],one:[0],empty:[]})[testcase.major];
   if(ratios!==undefined){
    const selected=ratios.map(r=>testcase.inputs[0]+r*(testcase.inputs[1]-testcase.inputs[0]));
    axis=axis.tickValues(kind==='duration'?selected:selected.map(v=>({Timestamp:{value:String(Math.round(v*multiplier)),unit:'Microseconds'}})));
   }
  }
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis).build();owned.push(plot);
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,19);
  const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,temporalOptions);owned.push(request);
  let frame;
  try{frame=request.prepare();}catch(error){assert.ok(testcase.result.error,`${index}: ${error}`);records.push({kind,index,error:error.code});continue;}
  owned.push(frame);assert.ok(!testcase.result.error);
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks??[];
  assert.equal(ticks.length,testcase.result.values.length,`${index}`);
  ticks.forEach((tick,i)=>{
   const expected=testcase.result.values[i];
   if(kind==='duration')assert.ok(Math.abs(tick.value.Number-expected)<1e-10);
   else{assert.equal(tick.value.Timestamp.unit,'Microseconds');assert.equal(BigInt(tick.value.Timestamp.value),BigInt(Math.round(expected*multiplier)));}
   assert.ok(Math.abs(tick.position/100-testcase.result.positions[i])<1e-12);
  });records.push({kind,index,minor_ticks:ticks});
 }finally{for(const value of owned.reverse())value.dispose();}
}
for(const [unit,factor,nextUnit] of [['s',1n,'Milliseconds'],['ms',1000n,'Microseconds'],['us',1000000n,'Nanoseconds'],['ns',1000000000n,null]]){
 const owned=[];
 try{
  const origin=1700000000n*factor;
  const data=c.Data.columns({x:c.timestamps([origin,origin+1n],unit,'UTC'),y:[1,2]});owned.push(data);
  const name=({s:'Seconds',ms:'Milliseconds',us:'Microseconds',ns:'Nanoseconds'})[unit];
  const axis=c.xAxis().scale(c.scaleUtc()).range(0,100).expansion({mult:[0,0],add:[0,0]}).tickValues([origin,origin+1n].map(v=>({Timestamp:{value:String(v),unit:name}}))).tickFormat({GgplotTime:{pattern:'%Y'}});
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis).build();owned.push(plot);
  const wire=plot.toJson(),restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks;
  assert.deepEqual(ticks.map(t=>t.position),[0,50,100]);
  if(nextUnit===null)assert.equal(ticks[1].value,null);
  else{assert.equal(ticks[1].value.Timestamp.unit,nextUnit);assert.equal(BigInt(ticks[1].value.Timestamp.value),origin*1000n+500n);}
  records.push({kind:'precision',unit,minor_ticks:ticks});
 }finally{for(const value of owned.reverse())value.dispose();}
}
const discrete=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/minor-discrete.json')));
for(const [index,testcase] of discrete.cases.entries())for(const family of ['auto','band','point'])for(const reversed of [false,true]){
 const owned=[];
 try{
  const n=testcase.count,data=c.Data.columns({x:c.categorical(['a','b','c'].slice(0,n)),y:c.column(Array(n).fill(1),{kind:'float64'})});owned.push(data);
  let axis=c.xAxis().range(reversed?100:0,reversed?0:100);
  if(family!=='auto')axis=axis.scale(({band:c.scaleBand,point:c.scalePoint})[family]());
  const expansion=({none:{mult:[0,0],add:[0,0]},wide:{mult:[.2,.5],add:[1,2]}})[testcase.expansion];
  if(expansion!==undefined)axis=axis.expansion(expansion);
  const policy=({auto:'Automatic',hidden:'Hidden',empty:{Numeric:[]},explicit:{Numeric:[{number:'-Infinity'},-1,0,.5,1,1.5,2.5,3.5,4,{number:'Infinity'},{number:'NaN'}]}})[testcase.minor];
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis.minorBreaks(policy)).build();owned.push(plot);
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,19);
  const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks??[];
  assert.equal(ticks.length,testcase.result.values.length,`${index} ${family}`);
  ticks.forEach((tick,i)=>{assert.equal(tick.value.Number,testcase.result.values[i]);const p=testcase.result.positions[i];assert.ok(Math.abs(tick.position/100-(reversed?1-p:p))<1e-12);});
  records.push({kind:'discrete',index,family,reversed,minor_ticks:ticks});
 }finally{for(const value of owned.reverse())value.dispose();}
}
for(const [index,values] of [[{Number:0}],[{Timestamp:{value:'0',unit:'Milliseconds'}}],Array(257).fill(null)].entries()){
 const owned=[];
 try{
  const data=c.Data.columns({x:c.timestamps([0n,86400n],'s','UTC'),y:[1,2]});owned.push(data);
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().scale(c.scaleUtc()).minorBreaks({Timestamps:values})).build();owned.push(plot);
  const request=output.request(plot,options);owned.push(request);let failure;
  try{const frame=request.prepare();owned.push(frame);}catch(error){failure=error;}
  assert.ok(failure,'invalid timestamp minors accepted');records.push({kind:'timestamp_rejection',index,error:failure.code});
 }finally{for(const value of owned.reverse())value.dispose();}
}
output.dispose();assert.equal(records.length,761);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));
console.log('PASS WASM: 448 numeric, 90 temporal, 216 discrete, four exact-origin and three invalid-input minor cases.');
