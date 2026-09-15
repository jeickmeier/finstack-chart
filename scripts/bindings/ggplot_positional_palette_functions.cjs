'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-palette-functions.json'))).cases,records=[];
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const key=v=>v===null?'Null':{Text:v};
function data(t){const v=t.inputs;return c.Data.columns({x:c.categorical(v.map(x=>x??'')).validity(v.map(x=>x!==null)),y:c.column(v.map(()=>1),{kind:'float64'})},{keys:v.map((_,i)=>BigInt(100+i)),name:'data'});}
function axis(t,family){
 const modes={reverse:'reverse_count',spread:'square_count',short:'short_count',named:'named_square_count',missing:'missing_count',character:'reject',null:'null'};
 const limits=t.limit_mode==='automatic'?null:t.limit_mode==='retained'?['c','b','a','d'].map(key):[];
 return c.xAxis().scale(family==='band'?c.scaleBand():c.scalePoint()).range(100,540).discretePolicy({limits,palette_function:{operation:{id:'example.scale_palette',version:'1'},parameters:{mode:modes[t.route],channel:'size'}}}).guideGeometry({labels:'Preserve'});
}
function build(t,d,family){return c.plot(d).profile('Ggplot2_4_0_3').withRegistry(registry).aes(c.aes().x('x').y('y')).layer(c.points().name('marks')).xAxis(axis(t,family)).build();}
function check(t,frame){
 const e=t.result;assert.ok(!e.error,JSON.stringify(t));const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 assert.equal(ticks.length,e.major_positions.filter(x=>x!==null).length,JSON.stringify(t));
 e.breaks.forEach((v,i)=>{const semantic=v===null?'MissingCategory':{Category:v},found=ticks.filter(x=>JSON.stringify(x.value)===JSON.stringify(semantic)),p=e.major_positions[i];
  if(p===null)assert.equal(found.length,0);else{assert.equal(found.length,1);const tick=found[0];assert.equal(tick.label,e.labels[i]??'NA');assert.ok(Math.abs((tick.position-100)/440-p)<1e-12,JSON.stringify(t));}
 });
 const scene=frame.scene(),positions=Array(t.inputs.length).fill(null);
 scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(targets.length)positions[Number(targets[0].Source.key)-100]=(item.primitive.Point.center.x-100)/440;});
 positions.forEach((a,i)=>{const wanted=e.positions[i];assert.equal(a===null,wanted===null,JSON.stringify(t));if(a!==null)assert.ok(Math.abs(a-wanted)<1e-12,JSON.stringify(t));});
 return {ticks,positions};
}
for(const [index,t] of cases.entries())for(const family of ['band','point']){
 const owned=[];
 try{
  const d=data(t);owned.push(d);const p=build(t,d,family);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,61);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);const stale=JSON.parse(wire);stale.version=60;assert.throws(()=>c.Plot.from_json(JSON.stringify(stale),registry),c.ChartError);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',c.points().name('marks')).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);records.push({index,family,state,...check(t,frame)});assert.equal(restored.to_json(),wire);
   if(state==='original'&&family==='band'&&t.population==='ordinary'&&['reverse','spread','named'].includes(t.route)&&t.limit_mode!=='empty')for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.route}-${t.limit_mode}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${JSON.stringify(t)} ${error.stack}`);records.push({index,family,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,266);
for(const route of ['reverse','spread','named','missing'])for(const family of ['band','point']){
 const selected=Object.fromEntries(cases.filter(t=>t.route===route&&t.limit_mode==='automatic').map(t=>[t.population,t])),owned=[];
 try{
  const original=data(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original,family);owned.push(p);const chart=p.chart();owned.push(chart);
  const request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene(),wire=p.to_json();
  for(const population of ['constant','missing','empty','ordinary']){
   const t=selected[population],replacement=data(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement,family);owned.push(fresh);const ar=output.request(chart,options),br=output.request(fresh,options);owned.push(ar,br);const af=ar.prepare(),bf=br.prepare();owned.push(af,bf);const actual=check(t,af);assert.deepEqual(actual,check(t,bf));assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({route,family,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,298);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS 298 positional palette states and 18 publication files');
