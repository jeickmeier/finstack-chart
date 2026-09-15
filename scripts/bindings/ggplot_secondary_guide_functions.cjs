'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/secondary-guide-functions.json'))).cases,records=[],units=[['s',1],['ms',1000],['us',1000000],['ns',1000000000]];
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const temporal=t=>['date','datetime'].includes(t.family),factor=(t,m)=>m*(t.family==='date'?86400:1);
function data(t,unit,m){const v=t.inputs,x=t.family==='discrete'?c.categorical(v.map(x=>x??'')).validity(v.map(x=>x!==null)):temporal(t)?c.timestamps(v.map(x=>BigInt(Math.round((x??0)*factor(t,m)))),unit,'UTC').validity(v.map(x=>x!==null)):c.column(v,{kind:'float64'}).nullable(true);return c.Data.columns({x,y:c.column(v.map(()=>1),{kind:'float64'})},{keys:v.map((_,i)=>BigInt(100+i)),name:'data'});}
function build(t,d){
 const scale=t.family==='date'?c.scaleDate():t.family==='datetime'?c.scaleUtc():t.family==='discrete'?c.scaleBand():c.scaleLinear();let primary=c.xAxis().scale(scale).range(100,540);if(!t.expand)primary=primary.expansion({mult:[0,0],add:[0,0]});
 let second=c.xAxis().name('secondary').side('Top');
 if(t.conversion==='square'){
  const transform={Registered:{selection:{call:{operation:{id:'example.scale_transform',version:'1'},parameters:{family:'square',custom:false}}}}};
  second=second.secondaryTransform('x',{compatibility:'D3',family:{Ggplot:{transform}},domain:[0,20],range:[0,400],clamp:false,round:false,unknown:{kind:'Missing'}});
 }else second=second.secondary('x',t.conversion==='affine'?2:1,t.conversion==='affine'?3:t.conversion==='shift'?2:0);
 const mode=t.mode==='typed_empty'?'empty':t.mode==='empty'&&temporal(t)?'untyped_empty':t.mode;
 second=second.breaksFunction({operation:{id:'example.breaks_limits',version:'1'},parameters:mode}).guideGeometry({labels:'Preserve'});
 if(t.label_mode==='function')second=second.tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'indexed'}});
 return c.plot(d).profile('Ggplot2_4_0_3').withRegistry(registry).aes(c.aes().x(temporal(t)?{field:'x',origin:'0'}:'x').y('y')).layer(c.points().name('marks')).xAxis(primary).xAxis(second).build();
}
function check(t,m,frame){
 const e=t.result;assert.ok(!e.error,JSON.stringify(t));const ticks=frame.guides().guides.find(g=>g.spec.side==='Top').ticks;assert.equal(ticks.length,e.positions.length,JSON.stringify(t));
 ticks.forEach((tick,i)=>{assert.ok(Math.abs((tick.position-100)/440-e.positions[i])<2e-12,JSON.stringify({t,tick}));assert.equal(tick.label,e.labels[i]??'NA');const v=tick.value,a=v.Timestamp?Number(v.Timestamp.value)/factor(t,m):temporal(t)?v.Number/factor(t,m)+(t.conversion==='shift'?2:0):v.Number;assert.ok(Math.abs(a-e.user_values[i])<2e-12,JSON.stringify({t,tick,a,e}));});return {ticks};
}
for(const [index,t] of cases.entries())for(const [unit,m] of temporal(t)?units:[['ms',1000]]){
 const owned=[];
 try{
  const d=data(t,unit,m);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,62);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  assert.throws(()=>c.Plot.from_json(wire),c.ChartError);const stale=JSON.parse(wire);stale.version=61;assert.throws(()=>c.Plot.from_json(JSON.stringify(stale),registry),c.ChartError);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',c.points().name('marks')).build();if(current!==restored)owned.push(current);const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);records.push({index,unit,state,...check(t,m,frame)});assert.equal(restored.to_json(),wire);
   if(state==='original'&&unit==='ms'&&t.label_mode==='function'&&t.mode==='domain'&&t.expand&&(t.population==='ordinary'||t.family==='numeric'&&t.conversion==='square'&&t.population==='empty'))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.family}-${t.conversion}-${t.population}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${JSON.stringify(t)} ${error.stack}`);records.push({index,unit,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1190);
for(const family of ['numeric','date','datetime'])for(const conversion of family==='numeric'?['identity','affine','square']:['identity','shift']){
 const selected=Object.fromEntries(cases.filter(t=>t.family===family&&t.conversion===conversion&&t.mode==='domain'&&t.label_mode==='function'&&t.expand).map(t=>[t.population,t]));
 for(const [unit,m] of family!=='numeric'?units:[['ms',1000]]){
  const owned=[];
  try{
   const original=data(selected.ordinary,unit,m);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene(),wire=p.to_json();
   for(const population of ['constant','missing','empty','ordinary']){
    const t=selected[population],replacement=data(t,unit,m);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));const fresh=build(t,replacement);owned.push(fresh);const ar=output.request(chart,options),br=output.request(fresh,options);owned.push(ar,br);const af=ar.prepare(),bf=br.prepare();owned.push(af,bf);const actual=check(t,m,af);assert.deepEqual(actual,check(t,m,bf));assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({family,conversion,unit,replacement:population,...actual});
   }
  }finally{for(const obj of owned.reverse())obj.dispose();}
 }
}
assert.equal(records.length,1266);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS 1266 secondary callback states and 24 publication files');
