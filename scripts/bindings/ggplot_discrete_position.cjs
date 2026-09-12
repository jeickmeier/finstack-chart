'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/position-null-categories.json'))).cases.concat(JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/position-null-continuous-limits.json'))).cases,JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/position-null-minor-breaks.json'))).cases),records=[];
const key=v=>v===null?'Null':{Text:v};
const policy=v=>({limits:v.limits?.map(key)??null,levels:v.levels?.map(key)??null,drop:v.drop,na_translate:v.na_translate,guide:{breaks:v.breaks?.map(key)??null}});
for(const [index,testcase] of cases.entries())for(const family of ['band','point']){
 const owned=[];
 try{
  const values=testcase.inputs,n=values.length;
  const data=c.Data.columns({x:c.categorical(values.map(v=>v??'')).validity(values.map(v=>v!==null)),y:c.column(values.map(()=>1),{kind:'float64'})},{keys:values.map((_,i)=>BigInt(100+i))});owned.push(data);
  let axis=c.xAxis().scale(family==='band'?c.scaleBand():c.scalePoint()).range(100,540).discretePolicy(policy(testcase)).guideGeometry({labels:'Preserve'});
  if(testcase.expansion)axis=axis.expansion(testcase.expansion).continuousLimits(testcase.continuous_limits?.map(v=>typeof v==='string'?{number:v}:v)??null);
  if(testcase.minor_breaks)axis=axis.minorBreaks({Numeric:testcase.minor_breaks.map(v=>typeof v==='string'?{number:v}:v)});
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis).build();owned.push(plot);
  const wire=plot.to_json();assert.equal(JSON.parse(wire).version,22);
  const restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!testcase.result.error);
  const scene=frame.scene(),points=Array(n).fill(null);
  scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(!targets.length)return;assert.equal(targets.length,1);assert.ok(targets[0].Source&&item.primitive.Point);const row=Number(targets[0].Source.key)-100;assert.equal(points[row],null);points[row]=(item.primitive.Point.center.x-100)/440;});
  points.forEach((actual,i)=>{const wanted=testcase.result.point_positions[i];assert.equal(actual===null,wanted===null,`${index}`);if(actual!==null)assert.ok(Math.abs(actual-wanted)<1e-12,`${index}: ${actual} ${wanted}`);});
  assert.equal(points.filter(v=>v!==null).length,testcase.result.point_count);
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
  testcase.result.breaks.forEach((value,i)=>{const semantic=value===null?'MissingCategory':{Category:value},matches=ticks.filter(t=>JSON.stringify(t.value)===JSON.stringify(semantic)),position=testcase.result.major_positions[i];if(position===null)assert.equal(matches.length,0);else{assert.equal(matches.length,1,`${index}`);const tick=matches[0];assert.equal(tick.label,testcase.result.labels[i]??'NA');assert.ok(Math.abs((tick.position-100)/440-position)<1e-12);}});
  assert.equal(ticks.length,testcase.result.major_positions.filter(v=>v!==null).length);
  const minor=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks??[];
  if(testcase.result.minor_values){assert.equal(minor.length,testcase.result.minor_values.length);minor.forEach((tick,i)=>{assert.deepEqual(tick.value,{Number:testcase.result.minor_values[i]});assert.ok(Math.abs((tick.position-100)/440-testcase.result.minor_positions[i])<1e-12);});}
  records.push({index,family,points,ticks,minor_ticks:minor});
  if(index<576&&family==='band'&&testcase.population==='mixed'&&testcase.level_name==='character'&&testcase.limits_name==='auto'&&testcase.drop&&testcase.breaks_name==='auto')for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`null-positions-${testcase.na_translate?'True':'False'}.${format}`),frame.export(format));
  if(index===652&&family==='band')for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`null-positions-expanded.${format}`),frame.export(format));
 }catch(error){assert.ok(error instanceof c.ChartError&&testcase.result.error&&error.code==='CHART_NUMERICAL_DOMAIN',`${index}: ${error.stack}`);records.push({index,family,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,3840);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 3840 positional nullable chart configurations and nine publication files.');
