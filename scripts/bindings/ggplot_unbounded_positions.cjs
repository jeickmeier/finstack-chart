'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-unbounded-viewports-exact.json')));
const authored=process.argv[4]==='authored';
if(authored)fixture.cases.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-authored-limits.json'))).cases);
function supported(t){return authored|| t.kind==='binned'||(t.population!=='empty'&&!(['sqrt','log10'].includes(t.transform)&&['both','lower','negative'].includes(t.control)));}
const cases=fixture.cases.map((t,i)=>[i,t]).filter(([,t])=>supported(t));assert.equal(cases.length,authored?2376:1350);
function numeric(v){return v===null?NaN:Number(v);}
function wireNumber(v){return typeof v==='string'?{number:v}:v;}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v===null?null:numeric(v)),{kind:'float64'}).nullable(true)},{keys:t.inputs.map((_,i)=>100+i)});}
function build(t,d){
 const transform={identity:null,sqrt:'Sqrt',reverse:'Reverse',log10:{Log:{base:10}}}[t.transform];
 const scale=t.kind==='binned'?c.scale_binned({transform,bins:{limits:t.limits.map(wireNumber),oob:t.oob[0].toUpperCase()+t.oob.slice(1),breaks:{Nice:10},right:true}}):{identity:c.scale_linear,sqrt:c.scaleSqrt,reverse:c.scaleReverse,log10:()=>c.scale_log(10)}[t.transform]();
 const view=[...t.viewport].sort((a,b)=>a-b);if(t.transform==='reverse')view.reverse();
 let axis=c.x_axis().scale(scale).viewport(...view).oob(t.oob[0].toUpperCase()+t.oob.slice(1)).guideGeometry({labels:'Preserve'});
 if(t.kind==='continuous'){const bounds=t.limits.map(wireNumber);if(authored)axis=axis.numericLimits(bounds);else{if(t.transform==='reverse')bounds.reverse();axis=axis.limitsFunction({operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:'fixed',bounds}});}}
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(c.points().name('marks')).x_axis(axis).y_axis(c.y_axis().visible(false)).build();
}
function check(t,chart,frame){
 const scene=frame.scene(),points=t.inputs.map(()=>null);
 const guideFrame=frame.presentation().find(g=>g.frame.side==='Bottom').frame,domain=guideFrame.domain,numbers=domain.numbers;assert.ok(domain.horizontal&&guideFrame.translation[0]===0);const offset=numbers[domain.caps?2:1],start=numbers[0]-offset,length=numbers[domain.caps?3:2]-offset-start;assert.ok(length>0);
 scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(targets.length&&targets[0].Source&&item.primitive.Point){const row=Number(targets[0].Source.key)-100;assert.ok(row>=0&&row<points.length&&points[row]===null);const clip=item.clip;assert.ok(Math.abs(clip.origin.x-start)<=3e-12&&Math.abs(clip.width-length)<=3e-12);points[row]=(item.primitive.Point.center.x-start)/length;}});
 points.forEach((actual,i)=>{const expected=numeric(t.result.point_positions[i]);if(!Number.isFinite(expected))assert.equal(actual,null,JSON.stringify({t,points}));else assert.ok(actual!==null&&Math.abs(actual-expected)<=3e-12,JSON.stringify({t,points}));});
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks,expected=t.result.positions.flatMap((p,i)=>Number.isFinite(numeric(p))?[[numeric(p),t.result.labels[i]??'']]:[]).sort((a,b)=>a[0]-b[0]);
 assert.equal(ticks.length,expected.length,JSON.stringify({t,ticks,expected}));[...ticks].sort((a,b)=>a.position-b.position).forEach((tick,i)=>{assert.equal(tick.label,expected[i][1],JSON.stringify(t));assert.ok(Math.abs((tick.position-start)/length-expected[i][0])<=3e-12,JSON.stringify({t,ticks,expected}));});
 const semantics=chart.semantics();return{points,ticks,domains:semantics.layers[0].domains,limits:semantics.positional_limits??{}};
}
function sample(t){if(authored&&t.kind==='continuous'&&t.control==='both'&&t.view==='positive'&&t.oob==='keep'&&(t.population==='empty'||(t.population==='finite'&&['sqrt','log10'].includes(t.transform))))return true;return t.view==='positive'&&((t.control==='upper'&&t.population==='infinite'&&t.oob==='keep')||(t.kind==='binned'&&(t.control==='finite'||(t.transform==='log10'&&t.control==='lower'))&&t.population==='finite'&&t.oob==='censor'));}
for(const[index,t]of cases){
 const owned=[];
 try{const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,t.kind==='continuous'?(authored?31:29):18);if(authored&&index===0){const invalid=JSON.parse(wire);invalid.version=30;assert.throws(()=>c.Plot.from_json(JSON.stringify(invalid),registry),e=>e instanceof c.ChartError&&e.code==='CHART_UNSUPPORTED_CAPABILITY');assert.throws(()=>{const legacy=c.plot(d).aes(c.aes().x('x').y(1)).layer(c.points()).x_axis(c.x_axis().numericLimits([null,5])).build();owned.push(legacy);const chart=legacy.chart();owned.push(chart);chart.semantics();},e=>e instanceof c.ChartError&&e.code==='CHART_UNSUPPORTED_CAPABILITY');}
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){const current=state==='original'?restored:restored.edit().layer('marks',c.points().name('marks')).build();if(current!==restored)owned.push(current);const chart=current.chart();owned.push(chart);const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...check(t,chart,frame)});
   if(state==='original'&&sample(t))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.kind}-${t.transform}-${t.control}${t.control==='both'?'-'+t.population:''}.${fmt}`),frame.export(fmt));}
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${JSON.stringify(t)}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,authored?4392:2394);
for(const kind of ['continuous','binned'])for(const transform of ['identity','sqrt','reverse','log10']){
 const owned=[];
 try{const selected=Object.fromEntries(cases.filter(([index,t])=>index<1728&&t.kind===kind&&t.transform===transform&&t.control==='upper'&&t.view==='positive'&&t.oob==='keep').map(([,t])=>[t.population,t]));const original=dataFor(selected.finite);owned.push(original);const p=build(selected.finite,original);owned.push(p);const chart=p.chart();owned.push(chart);const request=output.request(chart,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene(),wire=p.to_json();
  for(const population of (authored&&kind==='continuous'?['infinite','missing','empty','finite','infinite']:['infinite','missing','finite','infinite'])){const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);const outcome=chart.commit(update);assert.ok('Applied'in outcome,JSON.stringify({kind,transform,population,outcome}));const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=chart.semantics(),expected=batch.semantics();assert.deepEqual(actual.layers[0].domains,expected.layers[0].domains);assert.deepEqual(actual.positional_limits??{},expected.positional_limits??{});const currentRequest=output.request(chart,options);owned.push(currentRequest);const currentFrame=currentRequest.prepare();owned.push(currentFrame);const batchRequest=output.request(batch,options);owned.push(batchRequest);const batchFrame=batchRequest.prepare();owned.push(batchFrame);const projection=check(t,chart,currentFrame);assert.deepEqual(projection,check(t,batch,batchFrame));assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({kind,transform,replacement:population,projection});}
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,authored?4428:2426);fs.writeFileSync(path.join(out,'unbounded-position-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM:',records.length,'unbounded coordinate states.');
