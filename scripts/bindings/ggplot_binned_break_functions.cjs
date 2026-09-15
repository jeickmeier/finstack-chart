'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],defaultNames=process.argv.includes("--default-names"),constructorGuides=process.argv.includes("--constructor-guides");
const cases=JSON.parse(fs.readFileSync(path.join(root,defaultNames?'fixtures/parity/ggplot2/binned-break-default-names.json':'fixtures/parity/ggplot2/binned-break-functions.json'))).cases;
function descriptor(t){const family=t.transform==='sqrt'?{Pow:{exponent:.5}}:t.transform==='log10'?{Log:{base:10}}:'Linear',output=t.channel==='colour'?{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}}:{Interpolate:{operation:'PowerRange',range:[1,6],exponent:.5,absolute:false}};return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family,domain:[0,1],reverse:t.transform==='reverse',rescaler:'Range'}},output,unknown:{kind:'Missing'}}},ggplot:{Binned:{limits:t.limits==='full'?[1,10]:null,oob:'Squish',breaks:{Equal:{none:5,three:3,zero:0}[t.count]},right:true}},breaks_function:{operation:{id:'example.breaks_'+t.signature,version:'1'},parameters:t.mode},guide:{[constructorGuides?(t.channel==='colour'?'BinnedSteps':'BinnedBins'):'Binned']:(defaultNames?'Automatic':{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'indexed'}})}};}
function layer(t){const result=c.points().name('marks');return t.channel==='colour'?result:result.numeric_scale('Size','v',descriptor(t));}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'float64'}).nullable(true)});}
function build(t,d){const draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3');return t.channel==='colour'?draft.aes(c.aes().x('x').y(1).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t))).layer(layer(t)).build():draft.aes(c.aes().x('x').y(1)).layer(layer(t)).build();}
function check(t,chart){const state=chart.semantics().layers[0];let actual,expected;
 if(t.channel==='colour'){actual={labels:(state.color_legend?.numeric_breaks??[]).filter(e=>e.visible).map(e=>e.label)};expected={labels:(t.result.keys??[]).flatMap(g=>g.labels)};}
 else {const entries=Object.values(state.numeric_value_guides??{}).flat().filter(e=>e.visible);actual={values:entries.map(e=>e.transformed),labels:entries.map(e=>e.label)};expected={...(t.result.keys?.[0]??{values:[],labels:[]}),values:t.result.boundaries};if(expected.values.length===1)expected.values=[...expected.values,...expected.values];assert.equal(actual.values.length,expected.values.length,JSON.stringify(t));actual.values.forEach((a,i)=>assert.ok(Math.abs(a-expected.values[i])<=3e-12*Math.max(1,Math.abs(expected.values[i])),JSON.stringify(t)));}
 assert.deepEqual(actual.labels,expected.labels,JSON.stringify(t));return actual;}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,constructorGuides?44:33);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...actual});
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${index}: ${error.stack}`);assert.ok(['CHART_NUMERICAL_DOMAIN','CHART_VALIDATION'].includes(error.code),error.code);records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,defaultNames?116:4680);
for(const channel of (defaultNames?[]:['size','colour'])){
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.limits==='none'&&t.transform==='identity'&&t.mode==='domain'&&t.signature==='n'&&t.count==='three').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.to_json();
  for(const population of ['missing','empty','ordinary']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart);assert.deepEqual(actual,check(t,batch));assert.equal(p.to_json(),wire);records.push({channel,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,defaultNames?116:4686);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));registry.dispose();
console.log(`PASS WASM: ${records.length} binned break states; defaultNames=${defaultNames}.`);
