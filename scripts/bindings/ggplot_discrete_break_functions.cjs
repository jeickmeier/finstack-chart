'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-break-functions.json'))).cases;
const key=v=>v===null?'Null':{Text:v};
function descriptor(t){const channel=t.channel,palette=channel==='colour'?{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}:channel==='shape'?{Shape:{solid:true}}:channel==='linetype'?'LineType':{NumericRange:{range:channel==='alpha'?[.1,1]:[2,6],area:channel==='size'}};return {training:'Eligible',function:{Ordinal:{domain:[],range:[],unknown:{Explicit:null}}},ggplot:{Discrete:{limits:t.limits==='explicit'?['c','b','a',null].map(key):null,levels:null,drop:true,na_translate:true,palette}},breaks_function:{operation:{id:'example.discrete_breaks',version:'1'},parameters:t.mode},guide:{Discrete:{labels:t.label_mode==='default'?'Automatic':{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'indexed'}}}}};}

function layer(t){if(t.channel==='colour')return c.points().name('marks');const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth',shape:'Shape',linetype:'LineType'}[t.channel],result=(['linewidth','linetype'].includes(t.channel)?c.line():c.points()).name('marks');return ['shape','linetype'].includes(t.channel)?result.value_scale(channel,'v',descriptor(t)):result.numeric_scale(channel,'v',descriptor(t));}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'string'}).nullable(true)});}
function build(t,d){const draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3');return t.channel==='colour'?draft.aes(c.aes().x('x').y(1).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t))).layer(layer(t)).build():draft.aes(c.aes().x('x').y(1)).layer(layer(t)).build();}

function check(t,chart){const state=chart.semantics().layers[0],entries=Object.values(state.discrete_value_guides??{}).flat();let actual={values:entries.map(e=>typeof e.key==='object'?e.key.Text??null:null),labels:entries.map(e=>e.label)},expected=t.result.keys?.[0]??{values:[],labels:[]};if(t.channel==='colour'){actual={labels:(state.color_legend?.entries??[]).map(e=>e[0])};expected={labels:expected.labels.map(v=>v??'NA')};}assert.deepEqual(actual,expected,JSON.stringify(t));return actual;}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,33);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...actual});
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_VALIDATION');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1200);
for(const channel of ['colour','size','alpha','linewidth','shape','linetype']){
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.limits==='trained'&&t.mode==='mixed'&&t.label_mode==='indexed').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.to_json();
  for(const population of ['nullable','all_missing','empty','numeric_text','ordinary']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart);assert.deepEqual(actual,check(t,batch));assert.equal(p.to_json(),wire);records.push({channel,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1230);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));registry.dispose();
console.log('PASS WASM: 1230 discrete break states, including 30 replacements.');
