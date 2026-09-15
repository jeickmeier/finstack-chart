'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-label-functions.json'))).cases;
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d){
 let scale={identity:c.scaleLinear,sqrt:c.scaleSqrt,reverse:c.scaleReverse,log10:()=>c.scaleLog(10)}[t.transform]();
 if(t.limits==='full')scale=scale.domain(1,10);
 let axis=c.xAxis().scale(scale).range(100,540).guideGeometry({labels:'Preserve'}).tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}});
 if(t.break_mode==='explicit')axis=axis.tickValues([-1,0,1,1,3,20,{Number:{number:'Infinity'}},{Number:{number:'NaN'}}]);
 else if(t.break_mode==='empty')axis=axis.tickValues([]);
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 if(t.result.range.every(v=>typeof v==='number')){
  const expected=t.result.labels.filter((_,i)=>typeof t.result.breaks[i]==='number').map(v=>v??'');
  assert.deepEqual(ticks.map(t=>t.label),expected,JSON.stringify(t));
 }return {ticks};
}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.toJson();
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);
   assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,state,...check(t,frame)});
   if(state==='original'&&[0,5,120,240,360].includes(index))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,'CHART_SCHEMA_CONFLICT');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,668);
for(const transform of ['identity','sqrt','log10','reverse'])for(const mode of ['indexed','missing']){
 const selected=Object.fromEntries(cases.filter(t=>t.transform===transform&&t.limits==='full'&&t.break_mode==='automatic'&&t.label_mode===mode).map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','ordinary']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const a=chart.semantics(),b=batch.semantics(),actual={limits:a.positional_limits??null,domains:a.layers[0].domains};assert.deepEqual(actual,{limits:b.positional_limits??null,domains:b.layers[0].domains});assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({transform,mode,replacement:population,semantics:actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,708);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 708 positional label states and 15 publications.');
