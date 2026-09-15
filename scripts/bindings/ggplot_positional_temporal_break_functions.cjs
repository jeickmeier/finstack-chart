'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const replacementsOnly=process.argv.includes('--replacements-only');
const zero=process.argv.includes('--zero-range');
const overrides=process.argv.includes('--overrides');
const cases=JSON.parse(fs.readFileSync(path.join(root,zero?'fixtures/parity/ggplot2/positional-temporal-break-zero.json':overrides?'fixtures/parity/ggplot2/positional-temporal-break-overrides.json':'fixtures/parity/ggplot2/positional-temporal-break-functions.json'))).cases;
const units=[['s','Seconds',1n],['ms','Milliseconds',1000n],['us','Microseconds',1000000n],['ns','Nanoseconds',1000000000n]];
function zone(name){return name==='UTC'?'Utc':{Local:{version:1,zone:name,revision:'1',tzdata:'explicit-US-2024',coverage:{start:'1704067200000',end:'1735689600000'},initial_offset_seconds:-18000,transitions:[{at_millis:'1710054000000',offset_seconds:-14400},{at_millis:'1730613600000',offset_seconds:-18000}]}};}
function dataFor(t,unit,multiplier){const values=t.inputs,factor=multiplier*(t.kind==='date'?86400n:1n);return c.Data.columns({x:c.timestamps(values.map(v=>BigInt(v??0)*factor),unit,t.zone).validity(values.map(v=>v!==null))},{keys:values.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d,variant,multiplier){
 const date=t.kind==='date',factor=multiplier*(date?86400n:3600n),origin=BigInt(t.epoch)*multiplier;
 let scale=date?c.scaleDate():t.zone==='UTC'?c.scaleUtc():c.scaleCalendar({domain:t.limits==='full'?[String(origin),String(origin+3n*factor)]:[],unit:variant,zone:zone(t.zone),range:[{kind:'Number',value:0},{kind:'Number',value:1}],factory:{kind:'Value',gamma:null},clamp:false,unknown:{kind:'Missing'}});
 if(t.limits==='full'&&t.zone==='UTC')scale=scale.timeDomain(origin,origin+3n*factor);
 let axis=c.xAxis().scale(scale).range(100,540).guideGeometry({labels:'Preserve'}).breaksFunction({operation:{id:'example.breaks_'+t.signature,version:'1'},parameters:t.mode}).tickArguments({count:t.count==='three'?3:t.count==='zero'?0:null});
 if(t.label_mode==='indexed')axis=axis.tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'indexed'}});
 if(['width','both'].includes(t.control))axis=axis.tickArguments({width:date?'1 day':'1 hour'});
 if(['format','both'].includes(t.control))axis=axis.tickFormat({GgplotTime:{pattern:date?'%Y-%m-%d':'%H:%M',locale:null}});
 if(zero)axis=axis.expansion({mult:[0,0],add:[0,0]});
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x({field:'x',origin:String(origin)}).y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 if(t.result.range.every(v=>typeof v==='number')){const expected=t.result.labels.filter((_,i)=>typeof t.result.breaks[i]==='number').map(v=>v??'');assert.deepEqual(ticks.map(t=>t.label),expected,JSON.stringify(t));}
 return {ticks};
}
function sample(t){return t.population==='spaced'&&t.limits==='none'&&t.signature==='n'&&t.count==='three'&&t.mode==='mixed'&&t.label_mode==='automatic'&&(t.kind==='date'||t.epoch===1710046800||t.zone==='America/New_York');}
for(const [index,t] of (replacementsOnly?[]:cases.entries()))for(const [unit,variant,multiplier] of units){
 const owned=[];
 try{
  const d=dataFor(t,unit,multiplier);owned.push(d);const p=build(t,d,variant,multiplier);owned.push(p);const wire=p.toJson();
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);const actual=check(t,frame);assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,unit,state,...actual});
   if(state==='original'&&unit==='ms'&&sample(t)){for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));}
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,t.result.error.includes('labels')?'CHART_SCHEMA_CONFLICT':'CHART_VALIDATION');records.push({index,unit,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,replacementsOnly?0:zero?400:overrides?1280:27000);
function comparable(chart){
 const state=chart.semantics(),domains=state.layers[0].domains,space=domains.x_space;
 if(space?.Timestamp){const origin=BigInt(space.Timestamp.origin);space.Timestamp.origin='0';if(domains.x)domains.x=Object.fromEntries(Object.entries(domains.x).map(([key,value])=>[key,String(origin+BigInt(value))]));}
 return {limits:state.positional_limits??null,domains};
}
for(const [kind,zoneName,epoch] of [['date','UTC',1704067200],['datetime','UTC',1710046800],['datetime','UTC',1730606400],['datetime','America/New_York',1710046800],['datetime','America/New_York',1730606400]]){
 if(overrides||zero)break;
 const selected=Object.fromEntries(cases.filter(t=>t.kind===kind&&t.zone===zoneName&&t.epoch===epoch&&t.limits==='none'&&t.signature==='n'&&t.count==='three'&&t.mode==='domain'&&t.label_mode==='indexed').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.spaced,'ms',1000n);owned.push(original);const p=build(selected.spaced,original,'Milliseconds',1000n);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','spaced']){
   const t=selected[population],replacement=dataFor(t,'ms',1000n);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement,'Milliseconds',1000n);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=comparable(chart);assert.deepEqual(actual,comparable(batch),JSON.stringify({kind,zoneName,population}));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);function render(current){try{const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!('error' in t.result));return check(t,frame);}catch(error){assert.ok('error' in t.result);assert.equal(error.code,'CHART_SCHEMA_CONFLICT');return {error:error.code};}}const laid=render(chart);assert.deepEqual(laid,render(batch));records.push({kind,zone:zoneName,epoch,replacement:population,semantics:actual,...laid});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,replacementsOnly?25:zero?400:overrides?1280:27025);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log(`PASS WASM: ${records.length} temporal positional break states; overrides=${overrides}.`);
