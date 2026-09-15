'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const defaultNames=process.argv.includes('--default-names');
const overrides=process.argv[4]==='overrides';
const cases=JSON.parse(fs.readFileSync(path.join(root,defaultNames&&overrides?'fixtures/parity/ggplot2/temporal-break-default-overrides.json':defaultNames?'fixtures/parity/ggplot2/temporal-break-default-names.json':overrides?'fixtures/parity/ggplot2/temporal-break-overrides.json':'fixtures/parity/ggplot2/temporal-break-functions.json'))).cases;
const units=[['s','Seconds',1n],['ms','Milliseconds',1000n],['us','Microseconds',1000000n],['ns','Nanoseconds',1000000000n]];
function zone(name){return name==='UTC'?'Utc':{Local:{version:1,zone:name,revision:'1',tzdata:'explicit-US-2024',coverage:{start:'1704067200000',end:'1735689600000'},initial_offset_seconds:-18000,transitions:[{at_millis:'1710054000000',offset_seconds:-14400},{at_millis:'1730613600000',offset_seconds:-18000}]}};}
function descriptor(t,variant,multiplier){
 const date=t.kind==='date',origin=BigInt(t.epoch)*multiplier,factor=Number(multiplier)*(date?86400:3600),labels=defaultNames?'Automatic':{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'indexed'}};
 const args={date,breaks:['width','both'].includes(t.control)?{Width:date?'1 day':'1 hour'}:'Automatic',count:t.count==='three'?3:t.count==='zero'?0:null,labels,format:['format','both'].includes(t.control)?{pattern:date?'%Y-%m-%d':'%H:%M',locale:null}:null};
 const palette=t.channel==='colour'?{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}:{operation:'PowerRange',range:[1,6],exponent:.5,absolute:false};
 return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range',timestamp:{origin:String(origin),unit:variant,date}}},output:{Interpolate:palette},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:t.limits==='full'?[0,3*factor]:null,oob:'Censor'}},breaks_function:{operation:{id:'example.breaks_'+t.signature,version:'1'},parameters:t.mode},guide:{Temporal:{origin:String(origin),unit:variant,zone:zone(t.zone),arguments:args}}};
}
function dataFor(t,unit,multiplier){const values=t.inputs,factor=multiplier*(t.kind==='date'?86400n:1n);return c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.timestamps(values.map(v=>BigInt(v??0)*factor),unit,t.zone).validity(values.map(v=>v!==null))},{keys:values.map((_,i)=>BigInt(100+i))});}
function layer(t,variant,multiplier){if(t.channel==='colour')return c.points().name('marks');const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth'}[t.channel];return (t.channel==='linewidth'?c.line():c.points()).name('marks').numeric_scale(channel,{field:'v',origin:String(BigInt(t.epoch)*multiplier)},descriptor(t,variant,multiplier));}
function build(t,d,variant,multiplier){const builder=c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3');if(t.channel==='colour')return builder.aes(c.aes().x('x').y(1).color({field:'v',origin:String(BigInt(t.epoch)*multiplier)}).color_scale('v')).scale(c.color_mapped('v',descriptor(t,variant,multiplier))).layer(layer(t,variant,multiplier)).build();return builder.aes(c.aes().x('x').y(1)).layer(layer(t,variant,multiplier)).build();}
function check(t,chart,multiplier){const state=chart.semantics().layers[0];if(t.channel==='colour'){const legend=state.color_legend,entries=legend?.entries??[],expected=(t.result.keys??[]).flatMap(g=>g.labels);assert.deepEqual(entries.map(e=>e[0]),expected,JSON.stringify(t));return {entries,styles:state.styles??[]};}const entries=Object.values(state.numeric_value_guides??{}).flat().filter(e=>e.visible),factor=Number(multiplier)*(t.kind==='date'?86400:1),origin=Number(BigInt(t.epoch)*multiplier),actual={values:entries.map(e=>origin/factor+(typeof e.transformed==='object'?Number(e.transformed.number):e.transformed)/factor),labels:entries.map(e=>e.label)},expected=t.result.keys?.[0]??{values:[],labels:[]};assert.deepEqual(actual,expected,JSON.stringify(t));return actual;}
for(const [index,t] of cases.entries())for(const [unit,variant,multiplier] of units){
 const owned=[];
 try{
  const d=dataFor(t,unit,multiplier);owned.push(d);const p=build(t,d,variant,multiplier);owned.push(p);const wire=p.toJson();assert.equal(JSON.parse(wire).version,33);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t,variant,multiplier)).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart,multiplier);assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,unit,state,...actual});
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,'CHART_VALIDATION');records.push({index,unit,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,overrides?240:defaultNames?800:30240);
for(const channel of ['colour','size'])for(const [kind,zoneName,epoch] of [['date','UTC',1704067200],['datetime','UTC',1710046800],['datetime','UTC',1730606400],['datetime','America/New_York',1710046800],['datetime','America/New_York',1730606400]]){
 if(overrides||defaultNames)break;
 const selected=Object.fromEntries(cases.filter(t=>t.channel===channel&&t.kind===kind&&t.zone===zoneName&&t.epoch===epoch&&t.limits==='none'&&t.signature==='n'&&t.count==='three'&&t.mode==='domain').map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.spaced,'ms',1000n);owned.push(original);const p=build(selected.spaced,original,'Milliseconds',1000n);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','empty','spaced']){
   const t=selected[population],replacement=dataFor(t,'ms',1000n);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement,'Milliseconds',1000n);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart,1000n);assert.deepEqual(actual,check(t,batch,1000n));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({channel,kind,zone:zoneName,epoch,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,overrides?240:defaultNames?800:30280);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log(`PASS WASM: ${records.length} temporal break function states; overrides=${overrides}.`);
