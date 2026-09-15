'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],joint=process.argv.includes("--joint");
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,joint?'fixtures/parity/ggplot2/positional-binned-joint-dimensions.json':'fixtures/parity/ggplot2/positional-binned-break-functions.json'))).cases;
const failed=t=>'error' in t.result||t.result.positions&&'error' in t.result.positions;
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d,route){
 const transform={sqrt:'Sqrt',reverse:'Reverse',log10:{Log:{base:10}}}[t.transform]??null;
 const breaks={Nice:t.count==='three'?3:t.count==='zero'?0:10};
 const spec={breaks_function:{operation:{id:'example.breaks_'+t.signature,version:'1'},parameters:t.mode},bins:{limits:t.limits==='full'?[1,10]:null,breaks,oob:'Squish',right:true},transform,show_limits:t.show_limits};
 if(route==='policy'&&t.label_mode!=='automatic')spec.labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};
 const scale=c.scaleBinned(spec);
 let axis=c.xAxis().scale(scale).range(100,540).guideGeometry({labels:'Preserve'}).tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}});
 if(route==='policy'||t.label_mode==='automatic')axis=axis.tickFormat(null);
 if(joint){const parameters={mode:t.control};if(t.control==='fixed')parameters.bounds=[1,10];axis=axis.limitsFunction({operation:{id:'example.numeric_limits',version:'1'},parameters});}
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 if(t.result.range.length===2&&t.result.range.every(v=>typeof v==='number')){
  const expected=t.result.labels.filter((_,i)=>typeof t.result.values[i]==='number').map(v=>v??'');
  assert.deepEqual(ticks.map(t=>t.label),expected,JSON.stringify(t));
  const [lower,upper]=t.result.range,values=t.result.values.filter(v=>typeof v==='number');ticks.forEach((tick,i)=>assert.ok(Math.abs((tick.position-100)/440-(values[i]-lower)/(upper-lower))<=3e-12,JSON.stringify({t,tick}))); 
 }return {ticks};
}
for(const [index,t] of cases.entries())for(const route of (t.label_mode==='indexed'?['axis','policy']:['policy'])){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d,route);owned.push(p);const wire=p.toJson();assert.equal(JSON.parse(wire).version,35);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);
   assert.ok(!failed(t),JSON.stringify(t));records.push({index,route,state,...check(t,frame)});
   if(state==='original'&&route==='policy'&&t.population==='ordinary'&&(joint?t.control==='fixed':t.limits==='none')&&t.signature==='n'&&t.count==='three'&&t.mode==='mixed'&&t.label_mode==='automatic'&&t.show_limits)for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok(failed(t),JSON.stringify(t)+' '+error);if(joint)assert.ok(['CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN','CHART_VALIDATION'].includes(error.code),error.code);else assert.equal(error.code,t.result.error.includes('labels')?'CHART_SCHEMA_CONFLICT':'CHART_NUMERICAL_DOMAIN');records.push({index,route,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,joint?2706:14580);
for(const transform of ['identity','sqrt','log10','reverse'])for(const mode of ['domain','mixed']){
 const selected=Object.fromEntries(cases.filter(t=>t.transform===transform&&(joint?t.control==='fixed':t.limits==='full')&&t.signature==='n'&&t.count==='three'&&t.mode===mode&&t.show_limits&&t.label_mode==='automatic').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original,'policy');owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','ordinary']){
   const t=selected[population];if(joint&&failed(t))continue;const replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement,'policy');owned.push(fresh);const batch=fresh.chart();owned.push(batch);const aRequest=output.request(chart,options);owned.push(aRequest);const aFrame=aRequest.prepare();owned.push(aFrame);const bRequest=output.request(batch,options);owned.push(bRequest);const bFrame=bRequest.prepare();owned.push(bFrame);const actual=check(t,aFrame);assert.deepEqual(actual,check(t,bFrame));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({transform,mode,replacement:population,semantics:actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,joint?2744:14620);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log(`PASS WASM: ${records.length} binned positional break states; joint=${joint}.`);
