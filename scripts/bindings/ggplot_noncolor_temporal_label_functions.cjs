'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/noncolor-temporal-label-functions.json'))).cases;
const units=[['s','Seconds',1n],['ms','Milliseconds',1000n],['us','Microseconds',1000000n],['ns','Nanoseconds',1000000000n]];
function zone(name){return name==='UTC'?'Utc':{Local:{version:1,zone:name,revision:'1',tzdata:'explicit-US-2024',coverage:{start:'1704067200000',end:'1735689600000'},initial_offset_seconds:-18000,transitions:[{at_millis:'1710054000000',offset_seconds:-14400},{at_millis:'1730613600000',offset_seconds:-18000}]}};}
function descriptor(t,variant,multiplier){
 const date=t.kind==='date',origin=BigInt(t.epoch)*multiplier,factor=Number(multiplier)*(date?86400:3600),labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};
 const args={date,breaks:t.control==='explicit'?{Explicit:[-factor,0,factor,3*factor,4*factor,{number:'NaN'}]}:t.control==='empty'?{Explicit:[]}:'Automatic',labels,format:t.control==='format'?{pattern:date?'%Y-%m-%d':'%H:%M',locale:null}:null};
 return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range',timestamp:{origin:String(origin),unit:variant,date}}},output:{Interpolate:{operation:'PowerRange',range:t.channel==='alpha'?[.1,1]:[1,6],exponent:t.channel==='size'?.5:1,absolute:false}},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:null,oob:'Censor'}},guide:{Temporal:{origin:String(origin),unit:variant,zone:zone(t.zone),arguments:args}}};
}
function dataFor(t,unit,multiplier){const values=t.inputs,factor=multiplier*(t.kind==='date'?86400n:1n);return c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.timestamps(values.map(v=>BigInt(v??0)*factor),unit,t.zone).validity(values.map(v=>v!==null))},{keys:values.map((_,i)=>BigInt(100+i))});}
function layer(t,variant,multiplier){const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth'}[t.channel];return (t.channel==='linewidth'?c.line():c.points()).name('marks').numeric_scale(channel,{field:'v',origin:String(BigInt(t.epoch)*multiplier)},descriptor(t,variant,multiplier));}
function build(t,d,variant,multiplier){return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer(t,variant,multiplier)).build();}
function check(t,chart,multiplier){const state=chart.semantics().layers[0],entries=Object.values(state.numeric_value_guides??{}).flat().filter(e=>e.visible),factor=Number(multiplier)*(t.kind==='date'?86400:1),origin=Number(BigInt(t.epoch)*multiplier),actual={values:entries.map(e=>origin/factor+(typeof e.transformed==='object'?Number(e.transformed.number):e.transformed)/factor),labels:entries.map(e=>e.label)},expected=t.result.keys?.[0]??{values:[],labels:[]};assert.deepEqual(actual,expected,JSON.stringify(t));return actual;}
for(const [index,t] of cases.entries())for(const [unit,variant,multiplier] of units){
 const owned=[];
 try{
  const d=dataFor(t,unit,multiplier);owned.push(d);const p=build(t,d,variant,multiplier);owned.push(p);const wire=p.toJson();assert.equal(JSON.parse(wire).version,32);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t,variant,multiplier)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart,multiplier);assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,unit,state,...actual});
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,t.result.error.includes('labels')?'CHART_VALIDATION':'CHART_NUMERICAL_DOMAIN');records.push({index,unit,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,8340);
for(const channel of ['size','alpha','linewidth'])for(const [kind,zoneName,epoch] of [['date','UTC',1704067200],['datetime','UTC',1710046800],['datetime','UTC',1730606400],['datetime','America/New_York',1710046800],['datetime','America/New_York',1730606400]]){
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.kind===kind&&t.zone===zoneName&&t.epoch===epoch&&t.control==='automatic'&&t.label_mode==='indexed').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.spaced,'ms',1000n);owned.push(original);const p=build(selected.spaced,original,'Milliseconds',1000n);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','empty','spaced']){
   const t=selected[population],replacement=dataFor(t,'ms',1000n);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement,'Milliseconds',1000n);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart,1000n);assert.deepEqual(actual,check(t,batch,1000n));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({channel,kind,zone:zoneName,epoch,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,8400);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 8400 non-color temporal label states, including 60 replacements.');
