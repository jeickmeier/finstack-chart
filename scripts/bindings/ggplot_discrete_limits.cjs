'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function descriptor(t){const s={training:'Eligible',limits_function:{operation:{id:'example.discrete_limits',version:"1"},parameters:{mode:t.control}}};if(t.kind==='identity')s.function={GgplotDiscreteIdentity:{limits:null,levels:null,drop:true,na_translate:true,guide:t.guide,observed:[]}};else Object.assign(s,{function:{Ordinal:{domain:[],range:[],unknown:{Explicit:null}}},ggplot:{Discrete:{limits:null,levels:null,drop:true,na_translate:true,palette:{Shape:{solid:true}}}}});return s;}
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/limit-functions.json'))).cases.filter(t=>!('transform' in t));
for(const [index,t] of cases.entries()){
 const owned=[],failure=t.result.error,identity=t.kind==='identity',channel=identity?'Label':'Shape';
 try{
  const inputs=t.inputs,data=c.Data.columns({x:c.column(inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(inputs,{kind:'string'}).nullable(true)});owned.push(data);
  const layer=()=>c.points().name('marks').value_scale(channel,'v',descriptor(t));
  const p=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).build();owned.push(p);
  const wire=p.to_json();assert.equal(JSON.parse(wire).version,28);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=chart.semantics().layers[0];assert.ok(!failure,`${index}: ${failure}`);
   const wanted=t.result.values.filter(v=>identity||v!==null),aesthetics=actual.aesthetics??[];assert.equal(aesthetics.length,wanted.length,`${index}`);
   aesthetics.forEach((row,i)=>assert.deepEqual(row[channel],wanted[i]===null?{kind:'Missing'}:{kind:identity?'Text':'Number',value:wanted[i]},`${index}`));
   records.push({index,state,aesthetics});
   if(t.guide&&t.population==='spaced'&&['reverse','fixed'].includes(t.control)&&state==='original'){const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.kind}-${t.control}.${fmt}`),frame.export(fmt));}
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&failure,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_VALIDATION');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
for(const kind of ['discrete','identity'])for(const control of ['append','fixed']){
 const owned=[];
 try{
  const selected=Object.fromEntries(cases.filter(t=>t.guide&&t.kind===kind&&t.control===control).map(t=>[t.population,t])),identity=kind==='identity',channel=identity?'Label':'Shape';
  const dataFor=t=>c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'string'}).nullable(true)});
  const build=d=>c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(c.points().name('marks').value_scale(channel,'v',descriptor(selected.spaced))).build();
  const original=dataFor(selected.spaced);owned.push(original);const p=build(original);owned.push(p);const wire=p.to_json(),chart=p.chart();owned.push(chart);
  const request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','spaced']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=chart.semantics().layers[0].aesthetics??[];assert.deepEqual(actual,batch.semantics().layers[0].aesthetics??[]);
   const expected=t.result.values.filter(v=>identity||v!==null);assert.equal(actual.length,expected.length);actual.forEach((row,i)=>assert.deepEqual(row[channel],expected[i]===null?{kind:'Missing'}:{kind:identity?'Text':'Number',value:expected[i]}));
   assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({kind,control,replacement:population,aesthetics:actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,198);fs.writeFileSync(path.join(out,'discrete-limit-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM:',records.length,'registered discrete limit states.');
