'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function descriptor(t){
 const s={training:'Eligible',limits_function:{operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:t.control}}};
 if(t.kind==='identity')s.function={GgplotNumericIdentity:{transform:({sqrt:'Sqrt',reverse:'Reverse'})[t.transform]??null,limits:null,guide:true,trained:null}};
 else{
  s.function={Interpolated:{normalization:{Ggplot:{family:t.transform==='sqrt'?{Pow:{exponent:0.5}}:'Linear',domain:[0,1],reverse:t.transform==='reverse',rescaler:t.rescaler==='maximum'?'Maximum':t.rescaler==='midpoint'?{Midpoint:2}:'Range'}},output:{Interpolate:{operation:'PowerRange',range:[1,6],exponent:0.5,absolute:false}},unknown:{kind:'Missing'}}};
  if('rescaler'in t)s.function.Interpolated.output='Identity';
  s.ggplot=t.kind==='binned'?{Binned:{limits:null,oob:'Squish',breaks:{Nice:5},right:true}}:{Continuous:{limits:null,oob:'Censor'}};
 }if('units'in t){
  const origin=(1704067200n*BigInt(t.units)).toString();s.function.Interpolated.normalization.Ggplot.timestamp={origin,unit:t.variant,date:t.kind==='date'};s.guide={Temporal:{origin,unit:t.variant,zone:'Utc',arguments:{date:t.kind==='date'}}};
 }return s;
}
function dataFor(t){if('units'in t){
 const factor=BigInt(t.units)*BigInt(t.kind==='date'?86400:1),stamp=v=>v===null?1704067200n*BigInt(t.units):BigInt(Math.trunc(v))*factor+BigInt(Math.round((v-Math.trunc(v))*Number(factor)));
 return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.timestamps(t.inputs.map(stamp),t.unit,'UTC').validity(t.inputs.map(v=>v!==null))});
}return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))});}
function layer(t){const p=c.points().name('marks');return t.kind==='identity'?p.numeric_scale('Alpha','v',descriptor(t)).after_scale(c.scale_aes().size(c.after_scale_expr('Alpha').coalesce(0).mul(.1).add(2))):p.numeric_scale('Size','units'in t?{field:'v',origin:(1704067200n*BigInt(t.units)).toString()}:'v',descriptor(t));}
function build(t,d){return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer(t)).build();}
function check(t,actual){
 let wanted=t.result.values;if(wanted.length===1)wanted=t.inputs.map(()=>wanted[0]);
 if(t.kind==='identity'){
  const rows=actual.styles??[];assert.equal(rows.length,wanted.length,JSON.stringify(t));
  rows.forEach((row,i)=>{const want=wanted[i],expectedSize=2+.1*(want??0);assert.ok(Math.abs(row.radius-expectedSize)<3e-12,JSON.stringify({t,row,want}));assert.equal(row.color.alpha,want===null?255:Math.floor(Math.min(1,Math.max(0,want))*255+.5),JSON.stringify({t,row,want}));});
 }else{
  wanted=wanted.filter(v=>v!==null);const rows=actual.styles??[];assert.equal(rows.length,wanted.length,JSON.stringify({t,rows,wanted}));rows.forEach((row,i)=>assert.ok(Math.abs(row.radius-wanted[i])<3e-12*Math.max(Math.abs(wanted[i]),1),JSON.stringify({t,row,want:wanted[i]})));
 }return {styles:actual.styles??[],aesthetics:actual.aesthetics??[]};
}
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/limit-functions.json'))).cases.filter(t=>'transform'in t);
cases.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/limit-rescalers.json'))).cases.map(t=>({...t,transform:'identity'})));
cases.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/temporal-limit-functions.json'))).cases.flatMap(t=>[['s','Seconds',1],['ms','Milliseconds',1000],['us','Microseconds',1000000],['ns','Nanoseconds',1000000000]].filter(([, ,units])=>!(t.kind==='datetime'&&t.population==='fractional'&&units===1)).map(([unit,variant,units])=>({...t,transform:'identity',unit,variant,units}))));
for(const [index,t] of cases.entries()){
 const owned=[],failure=t.result.error??('units'in t?t.result.guide?.error:undefined);
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,28);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);const chart=current.chart();owned.push(chart);const actual=chart.semantics().layers[0];assert.ok(!failure,`${index}: ${failure}`);records.push({index,state,...check(t,actual)});
   if((!('units'in t)||t.units===1000000)&&!('rescaler'in t)&&t.kind!=='identity'&&t.population==='spaced'&&t.transform==='identity'&&['reverse','fixed','single'].includes(t.control)&&state==='original'){const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.kind}-${t.control}.${fmt}`),frame.export(fmt));}
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&failure,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1362);
for(const kind of ['continuous','binned','identity','date','datetime'])for(const control of ['fixed','single']){
 const owned=[];
 try{
  const selected=Object.fromEntries(cases.filter(t=>t.kind===kind&&t.control===control&&t.transform==='identity'&&!('rescaler'in t)&&(!('units'in t)||t.units===1000000)).map(t=>[t.population,t]));
  const original=dataFor(selected.spaced);owned.push(original);const p=build(selected.spaced,original);owned.push(p);const wire=p.to_json(),chart=p.chart();owned.push(chart);const request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of (['date','datetime'].includes(kind)?['constant','missing','all_missing','spaced']:['constant','missing','all_missing','empty','spaced'])){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied'in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart.semantics().layers[0]);assert.deepEqual(actual,check(t,batch.semantics().layers[0]));assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({kind,control,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1408);fs.writeFileSync(path.join(out,'numeric-limit-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM:',records.length,'numeric limit states.');
