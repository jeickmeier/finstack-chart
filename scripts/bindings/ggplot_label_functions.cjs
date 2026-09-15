'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/guide-label-functions.json'))).cases;
const key=v=>v===null?'Null':{Text:v};
function descriptor(t){
 const labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}},s={training:'Eligible'};
 if(t.kind==='continuous'){
  const family=t.transform==='sqrt'?{Pow:{exponent:.5}}:t.transform==='log10'?{Log:{base:10}}:'Linear';
  s.function={Interpolated:{normalization:{Ggplot:{family,domain:[0,1],reverse:t.transform==='reverse',rescaler:'Range'}},output:{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}},unknown:{kind:'Missing'}}};
  s.ggplot={Continuous:{limits:t.limits,oob:'Censor'}};
  s.guide={Continuous:{breaks:t.break_mode==='auto'?null:t.break_mode==='empty'?[]:[-1,0,.1,1,5,10,20,{number:'Infinity'},{number:'NaN'},{number:'NaN'}],labels}};
 }else{
  s.function={Ordinal:{domain:[],range:[],unknown:{Explicit:null}}};
  s.ggplot={Discrete:{limits:null,levels:null,drop:true,na_translate:true,palette:{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}}};
  s.guide={Discrete:{breaks:t.break_mode==='auto'?null:t.break_mode==='empty'?[]:['z','b','b',null,'a'].map(key),break_names:t.break_mode==='named'?['Z','B','B2','M','A']:null,labels}};
 }return s;
}
function dataFor(t){const values=t.kind==='continuous'?[1,4,10]:t.population==='empty'?[]:['a','b',t.population==='nullable'?null:'c'];return c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:t.kind==='continuous'?'float64':'string'}).nullable(true)},{keys:values.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d){return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1).color('v').colorScale('v')).scale(c.colorMapped('v',descriptor(t))).layer(layer()).build();}
function check(t,chart){const state=chart.semantics().layers[0],entries=state.color_legend?.entries??[],expected=(t.build.keys??[]).flatMap(g=>g.labels.map(v=>v??'NA'));assert.deepEqual(entries.map(e=>e[0]),expected,JSON.stringify(t));return {entries,styles:state.styles??[]};}
function sample(t){if(t.kind==='continuous')return ['identity','log10'].includes(t.transform)&&JSON.stringify(t.limits)==='[1,10]'&&t.break_mode==='explicit'&&t.label_mode==='indexed';return t.break_mode==='named'&&((t.population==='ordinary'&&['missing','short'].includes(t.label_mode))||(t.population==='nullable'&&t.label_mode==='indexed'));}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.toJson();assert.equal(JSON.parse(wire).version,32);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!('error' in t.build),JSON.stringify(t));records.push({index,state,...actual});
   if(state==='original'&&sample(t)){const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));}
  }
 }catch(error){assert.ok('error' in t.build,JSON.stringify(t)+' '+error);assert.equal(error.code,'CHART_VALIDATION');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,326);
for(const mode of ['indexed','missing','short']){
 const selected=Object.fromEntries(cases.filter(t=>t.kind==='discrete'&&t.break_mode==='auto'&&t.label_mode===mode).map(t=>[t.population,t])),owned=[];
 try{
  const original=dataFor(selected.ordinary);owned.push(original);const p=build(selected.ordinary,original);owned.push(p);const chart=p.chart();owned.push(chart);const wire=p.toJson(),request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['nullable','empty','ordinary','nullable']){
   const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(t,chart);assert.deepEqual(actual,check(t,batch));assert.deepEqual(held.scene(),scene);assert.equal(p.toJson(),wire);records.push({mode,replacement:population,...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,338);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 338 label callback states and 15 publication files.');
