'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],factors=process.argv.includes('--factors');
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,factors?'fixtures/parity/ggplot2/factor-discrete-label-functions.json':'fixtures/parity/ggplot2/noncolor-discrete-label-functions.json'))).cases;
const key=v=>v===null?'Null':{Text:v};
function descriptor(t){const channel=t.channel,palette=channel==='shape'?{Shape:{solid:true}}:channel==='linetype'?'LineType':{NumericRange:{range:channel==='alpha'?[.1,1]:[2,6],area:channel==='size'}};return {training:'Eligible',function:{Ordinal:{domain:[],range:[],unknown:{Explicit:null}}},ggplot:{Discrete:{limits:t.limit_mode==='explicit'?['c','b','a',null].map(key):null,levels:t.levels?.map(key)??null,drop:t.drop??true,na_translate:t.na_translate??true,palette}},guide:{Discrete:{breaks:t.break_mode==='auto'?null:t.break_mode==='empty'?[]:['z','b','b',null,'a'].map(key),break_names:t.break_mode==='named'?['Z','B','B2','M','A']:null,labels:{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}}}}};}
function layer(t){const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth',shape:'Shape',linetype:'LineType'}[t.channel],result=(['linewidth','linetype'].includes(t.channel)?c.line():c.points()).name('marks');return ['shape','linetype'].includes(t.channel)?result.value_scale(channel,'v',descriptor(t)):result.numeric_scale(channel,'v',descriptor(t));}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'string'}).nullable(true)});}
function build(t,d){return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer(t)).build();}
function check(t,chart){const state=chart.semantics().layers[0],entries=Object.values(state.discrete_value_guides??{}).flat(),actual={values:entries.map(e=>typeof e.key==='object'?e.key.Text??null:null),labels:entries.map(e=>e.label)},expected=t.result.keys?.[0]??{values:[],labels:[]};assert.deepEqual(actual,expected,JSON.stringify(t));return actual;}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,32);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...actual});
   if(factors&&state==='original'&&t.limit_mode==='trained'&&t.break_mode==='auto'&&t.label_mode==='indexed'&&!t.drop&&t.na_translate&&['ordinary','nullable','empty'].includes(t.population)&&!t.result.draw_error){const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.channel}-${t.population}.${fmt}`),frame.export(fmt));}
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_VALIDATION');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,factors?1280:1495);
for(const channel of ['size','alpha','linewidth','shape','linetype'])for(const drop of factors?[false,true]:[true])for(const translate of factors?[false,true]:[true]){
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.limit_mode==='trained'&&t.break_mode==='auto'&&t.label_mode==='indexed'&&(t.drop??true)===drop&&(t.na_translate??true)===translate).map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.to_json();let held,scene;if(factors){const request=output.request(p,options);owned.push(request);held=request.prepare();owned.push(held);scene=held.scene();}
  for(const population of ['nullable','all_missing','empty','ordinary']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart);assert.deepEqual(actual,check(t,batch));assert.equal(p.to_json(),wire);if(factors)assert.deepEqual(held.scene(),scene);records.push({channel,replacement:population,...(factors?{drop,translate}:{}),...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,factors?1360:1515);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();
console.log(`PASS WASM: ${records.length} non-color discrete label states; factors=${factors}.`);
