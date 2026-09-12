'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current'),records=[];
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-continuous-limits.json')));
fixture.cases.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-order.json'))).cases);
const number=v=>v===null?{number:'NaN'}:(['Infinity','-Infinity','NaN'].includes(v)?{number:v}:v),close=(a,b)=>assert.ok(Math.abs(a-b)<1e-12,`${a} != ${b}`);
for(const [index,t] of fixture.cases.entries())for(const family of t.levels===null?['auto','band','point']:['band','point'])for(const reverse of [false,true]){
 const owned=[],expected=t.result;
 try{
  const n=t.inputs.length,data=c.Data.columns({x:c.categorical(t.inputs),y:c.column(Array(n).fill(1),{kind:'float64'})},{keys:Array.from({length:n},(_,i)=>BigInt(100+i))});owned.push(data);
  let axis=c.xAxis().range(reverse?540:100,reverse?100:540).guideGeometry({labels:'Preserve'});
  if(family!=='auto'){let scale=({band:c.scaleBand,point:c.scalePoint})[family]();if(t.levels!==null)scale=scale.categories(t.levels);axis=axis.scale(scale);}
  const expansion=({none:{mult:[0,0],add:[0,0]},asymmetric:{mult:[.1,.2],add:[.3,.7]},contract:{mult:[-.5,-.25],add:[-.2,-.1]}})[t.expansion]??null,limits=t.limits===null?null:t.limits.map(number);
  axis=axis.expansion(expansion).continuousLimits(limits).minorBreaks({Numeric:[-2,.5,1,1.5,2,3,4]});
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();owned.push(plot);
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,limits===null?19:20);const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!expected.error,`${index}`);
  const scene=frame.scene(),points=Array(n).fill(null);
  scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(!targets.length)return;assert.equal(targets.length,1);assert.ok(targets[0].Source&&item.primitive.Point);const row=Number(BigInt(targets[0].Source.key)-100n);assert.ok(row>=0&&row<n);assert.equal(points[row],null);points[row]=(item.primitive.Point.center.x-100)/440;});
  points.forEach((actual,i)=>{const wanted=expected.point_positions[i];if(typeof wanted!=='number')assert.equal(actual,null,`${index}`);else{assert.notEqual(actual,null);close(actual,reverse?1-wanted:wanted);}});
  const guide=frame.guides().guides.find(g=>g.spec.side==='Bottom'),ticks=guide.ticks,minor=guide.minor_ticks??[];
  assert.deepEqual(ticks.map(tick=>tick.label).sort(),[...expected.labels].sort(),`${index}`);
  expected.labels.forEach((label,i)=>{const tick=ticks.find(tick=>tick.label===label),p=expected.major_positions[i];close((tick.position-100)/440,reverse?1-p:p);});
  assert.equal(minor.length,expected.minor_values.length);minor.forEach((tick,i)=>{assert.equal(tick.value.Number,expected.minor_values[i]);const p=expected.minor_positions[i];close((tick.position-100)/440,reverse?1-p:p);});
  records.push({index,family,reversed:reverse,points,ticks,minor_ticks:minor});
  let sample;if(family==='band'&&!reverse&&t.expansion==='default'){
   if(t.population==='unused'&&t.limits_name==='wide')sample='unused-levels';else if(t.population==='three'&&t.limits_name==='right_inf')sample='one-infinite-endpoint';else if(t.population==='three'&&t.limits_name==='unbounded')sample='undefined-positions';else if(t.population==='all_outside'&&t.limits_name==='default')sample='excluded-population';
  }
  if(sample){fs.writeFileSync(path.join(out,sample+'.plot.json'),wire);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,sample+'.'+fmt),frame.export(fmt));}
 }catch(error){assert.ok(error instanceof c.ChartError&&expected.error,`${index} ${family} ${reverse}: ${error.stack}`);records.push({index,family,reversed:reverse,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
const updateData=labels=>c.Data.columns({x:c.categorical(labels),y:c.column(Array(labels.length).fill(1),{kind:'float64'})},{keys:labels.map((_,i)=>BigInt(100+i))});
const updatePlot=(data,family,limits)=>c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().scale(({band:c.scaleBand,point:c.scalePoint})[family]().categories(['a','b','c','d'])).continuousLimits(limits).guideGeometry({labels:'Preserve'})).yAxis(c.yAxis().visible(false)).build();
const selected=frame=>{const scene=frame.scene(),guide=frame.guides().guides.find(g=>g.spec.side==='Bottom');return {points:scene.items.filter((_,i)=>scene.targets[i].length).map(item=>item.primitive.Point.center),ticks:guide.ticks,minor_ticks:guide.minor_ticks??[]};};
for(const family of ['band','point'])for(const [limit_index,limits] of [[1.5,2.5],[0,{number:'Infinity'}],[{number:'-Infinity'},{number:'Infinity'}]].entries()){
 const owned=[];
 try{
  const originalData=updateData(['a','c']);owned.push(originalData);const original=updatePlot(originalData,family,limits);owned.push(original);const wire=original.toJson();
  const live=original.chart();owned.push(live);const capture=output.request(live,options);owned.push(capture);const held=capture.prepare();owned.push(held);const initial=selected(held);
  for(const [step,labels] of [['d'],['z'],[],['a','c']].entries()){
   const stepOwned=[];
   try{
    const replacement=updateData(labels);stepOwned.push(replacement);const updates=live.transaction();stepOwned.push(updates);const builder=updates.replace(originalData,replacement);stepOwned.push(builder);const tx=builder.build();stepOwned.push(tx);live.commit(tx);
    const request=output.request(live,options);stepOwned.push(request);const frame=request.prepare();stepOwned.push(frame);
    const freshPlot=updatePlot(replacement,family,limits);stepOwned.push(freshPlot);const freshRequest=output.request(freshPlot,options);stepOwned.push(freshRequest);const fresh=freshRequest.prepare();stepOwned.push(fresh);
    const actual=selected(frame);assert.deepEqual(actual,selected(fresh));const frozen=capture.prepare();stepOwned.push(frozen);assert.deepEqual(selected(frozen),initial);assert.deepEqual(selected(held),initial);assert.equal(original.toJson(),wire);
    records.push({kind:'replacement',family,limit_index,step,...actual});
   }finally{for(const value of stepOwned.reverse())value.dispose();}
  }
 }finally{for(const value of owned.reverse())value.dispose();}
}
output.dispose();assert.equal(records.length,3582);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));console.log('PASS WASM: 3558 positional limit/ordering configurations and 24 replacements.');
