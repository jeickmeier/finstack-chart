'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/noncolor-numeric-label-functions.json'))).cases.filter(t=>t.kind==='binned');
function descriptor(t){const channel=t.channel,family=t.transform==='sqrt'?{Pow:{exponent:.5}}:t.transform==='log10'?{Log:{base:10}}:'Linear',breaks=t.break_mode==='auto'?{Nice:5}:{Explicit:t.break_mode==='empty'?[]:[-1,0,1,1,3,20,{number:'Infinity'},{number:'NaN'}]},labels=t.label_mode==='default'?'Automatic':{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family,domain:[0,1],reverse:t.transform==='reverse',rescaler:'Range'}},output:{Interpolate:{operation:'PowerRange',range:channel==='alpha'?[.1,1]:[1,6],exponent:channel==='size'?.5:1,absolute:false}},unknown:{kind:'Missing'}}},ggplot:{Binned:{limits:t.limits==='full'?[1,10]:null,oob:'Squish',right:true,breaks}},guide:{Binned:labels}};}
function layer(t){const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth',shape:'Shape',linetype:'LineType'}[t.channel],result=(['linewidth','linetype'].includes(t.channel)?c.line():c.points()).name('marks');return ['shape','linetype'].includes(t.channel)?result.value_scale(channel,'v',descriptor(t)):result.numeric_scale(channel,'v',descriptor(t));}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'float64'}).nullable(true)});}
function build(t,d){return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer(t)).build();}
function check(t,chart){const state=chart.semantics().layers[0],entries=Object.values(state.numeric_value_guides??{}).flat().filter(e=>e.visible),actual={values:entries.map(e=>typeof e.transformed==='object'?Number(e.transformed.number):e.transformed),labels:entries.map(e=>e.label)},expected={values:t.result.boundaries??[],labels:t.result.keys?.[0]?.labels??[]};assert.deepEqual(actual.labels,expected.labels,JSON.stringify(t));assert.equal(actual.values.length,expected.values.length,JSON.stringify(t));actual.values.forEach((a,i)=>assert.ok(Math.abs(a-expected.values[i])<=3e-12*Math.max(1,Math.abs(expected.values[i])),JSON.stringify(t)));return actual;}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.ok(t.label_mode==='default'||JSON.parse(wire).version===32);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...actual});
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${index}: ${error.stack}`);assert.ok(['CHART_VALIDATION','CHART_NUMERICAL_DOMAIN'].includes(error.code));records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,2739);
for(const channel of ['size','alpha','linewidth']){
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.limits==='none'&&t.transform==='identity'&&t.break_mode==='auto'&&t.label_mode==='indexed').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.to_json();
  for(const population of ['missing','empty','ordinary']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart);assert.deepEqual(actual,check(t,batch));assert.equal(p.to_json(),wire);records.push({channel,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,2748);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));registry.dispose();
console.log('PASS WASM: 2748 non-color binned label states, including 9 replacements.');
