'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const joint=process.argv.includes("--joint"),majorKind=joint?"function_fixed":"automatic",baseCount=joint?2858:5716;
const cases=JSON.parse(fs.readFileSync(path.join(root,joint?'fixtures/parity/ggplot2/positional-minor-break-joint-functions.json':'fixtures/parity/ggplot2/positional-minor-break-functions.json'))).cases;
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d){
 let scale={identity:c.scaleLinear,sqrt:c.scaleSqrt,reverse:c.scaleReverse,log10:()=>c.scaleLog(10)}[t.transform]();
 if(t.limits==='full')scale=scale.domain(1,10);
 let axis=c.xAxis().scale(scale).range(100,540).guideGeometry({labels:'Preserve'}).minorBreaks({Registered:{operation:{id:'example.breaks_minor_'+t.signature,version:'1'},parameters:t.mode}});
 if(t.major.startsWith('function_'))axis=axis.breaksFunction({operation:{id:'example.breaks_limits',version:'1'},parameters:t.major.slice('function_'.length)});
 else if(t.major==='explicit')axis=axis.tickValues([10,5,1,1,{number:'NaN'},{number:'Infinity'},{number:'-Infinity'}].map(v=>({Number:v})));
 else if(t.major==='empty')axis=axis.tickValues([]);
 else if(t.major==='null')axis=axis.ticks([]);
 if(t.expand==='zero')axis=axis.expansion({mult:[0,0],add:[0,0]});
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const guide=frame.guides().guides.find(g=>g.spec.side==='Bottom'),ticks=guide.ticks,minor=guide.minor_ticks??[],finite=t.result.range.every(v=>typeof v==='number');
 const labels=t.result.labels.filter((_,i)=>typeof t.result.major[i]==='number'&&finite).map(v=>v??'');assert.deepEqual(ticks.map(t=>t.label),labels,JSON.stringify(t));
 const expected=t.result.minor.filter(v=>typeof v==='number'&&finite);assert.equal(minor.length,expected.length,JSON.stringify(t));const forward={identity:x=>x,sqrt:Math.sqrt,log10:Math.log10,reverse:x=>-x}[t.transform];
 minor.forEach((tick,i)=>{const want=expected[i],value=forward(tick.value.Number);assert.ok(Math.abs(value-want)<=3e-12*Math.max(1,Math.abs(want)),JSON.stringify({t,tick,want}));const [a,b]=t.result.range,position=100+440*(want-a)/(b-a);assert.ok(Math.abs(tick.position-position)<1e-8,JSON.stringify({t,tick,position}));});
 return {ticks,minor_ticks:minor};
}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.toJson();assert.equal(JSON.parse(wire).version,36);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);
   assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,state,...check(t,frame)});
   if(state==='original'&&t.population==='spaced'&&t.limits==='none'&&t.signature==='two'&&t.major===majorKind&&t.expand==='default'&&t.mode==='mixed')for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,baseCount);
for(const transform of ['identity','sqrt','log10','reverse'])for(const mode of ['domain','mixed']){
 const selected=Object.fromEntries(cases.filter(t=>t.transform===transform&&t.limits==='full'&&t.signature==='two'&&t.major===majorKind&&t.expand==='default'&&t.mode===mode).map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.spaced);owned.push(original);const p=build(selected.spaced,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','spaced']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const aRequest=output.request(chart,options);owned.push(aRequest);const aFrame=aRequest.prepare();owned.push(aFrame);const bRequest=output.request(batch,options);owned.push(bRequest);const bFrame=bRequest.prepare();owned.push(bFrame);const actual=check(t,aFrame);assert.deepEqual(actual,check(t,bFrame));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({transform,mode,replacement:population,semantics:actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,baseCount+40);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log(`PASS WASM: ${records.length} numeric minor callback states.`);
