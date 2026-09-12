'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-null-domains.json'))).cases.filter(v=>v.kind==='hue'),records=[];
cases.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/identity-null-paints.json'))).cases);
const key=v=>v===null?'Null':{Text:v};
function descriptor(testcase){if(testcase.kind==='identity')return {training:'Eligible',function:{GgplotDiscreteIdentity:{limits:testcase.limits===null?null:testcase.limits.map(key),levels:testcase.levels===null?null:testcase.levels.map(key),drop:testcase.drop,na_translate:testcase.na_translate,guide:testcase.guide,observed:[]}}};return {training:'Eligible',function:{Ordinal:{domain:[],range:[],unknown:{Explicit:null}}},ggplot:{Discrete:{limits:testcase.limits===null?null:testcase.limits.map(key),levels:testcase.levels===null?null:testcase.levels.map(key),drop:testcase.drop,na_translate:testcase.na_translate,palette:{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}}}};}
function dataFor(testcase){const values=testcase.inputs;return c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:'string'}).nullable(true)},{keys:values.map((_,i)=>BigInt(100+i))});}
function build(data,testcase){return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1).color('v').colorScale('v')).scale(c.colorMapped('v',descriptor(testcase))).layer(c.points()).build();}
function paint(v){
 const named={red4:[139,0,0,255],blue:[0,0,255,255],green:[0,255,0,255],orange:[255,165,0,255],NA:[255,255,255,0]};if(named[v])return Object.fromEntries(['red','green','blue','alpha'].map((k,i)=>[k,named[v][i]]));
 if(v===null)return {red:0,green:0,blue:0,alpha:0};if(v==='grey50')return {red:127,green:127,blue:127,alpha:255};
 assert.ok(/^#[0-9A-Fa-f]{6}$/.test(v),v);return {red:parseInt(v.slice(1,3),16),green:parseInt(v.slice(3,5),16),blue:parseInt(v.slice(5,7),16),alpha:255};
}
function check(chart,testcase){
 const layer=chart.semantics().layers[0],colors=(layer.styles??[]).map(v=>v.color),wanted=testcase.result.training_paints.filter(v=>v!==null).map(paint);assert.deepEqual(colors,wanted,JSON.stringify(testcase));
 const identity=testcase.kind==='identity';if(identity)assert.equal(colors.length,testcase.result.point_count);
 const legend=layer.color_legend,entries=legend?legend.entries:[],domain=legend?(legend.mapping.function.Ordinal?.domain??null):[],wantedEntries=testcase.result.labels.map((label,i)=>[label===null?'NA':label,paint(identity&&testcase.result.guide_paints[i]===null?'NA':testcase.result.guide_paints[i])]);
 if(identity&&!testcase.guide)wantedEntries.length=0;
 assert.deepEqual(entries,wantedEntries,JSON.stringify(testcase));if(!identity)assert.deepEqual(domain,testcase.result.breaks.map(key),JSON.stringify(testcase));return {colors,entries,domain};
}
for(const [index,testcase] of cases.entries()){
 const owned=[];
 try{
  const data=dataFor(testcase);owned.push(data);const plot=build(data,testcase);owned.push(plot);const wire=plot.toJson(),restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const chart=restored.chart();owned.push(chart);records.push({index,...check(chart,testcase)});
  let sample=null;
  if(testcase.kind==='identity'&&testcase.population==='mixed'&&testcase.levels===null&&testcase.drop&&testcase.na_translate&&testcase.limits_name==='first'&&testcase.guide)sample='identity-missing-first';
  if(testcase.kind==='hue'&&testcase.population==='mixed'&&testcase.drop){
   if(testcase.levels===null&&testcase.na_translate&&testcase.limits_name==='first')sample='authored-missing-first';
   if(testcase.levels!==null&&testcase.na_translate&&testcase.limits_name==='auto')sample='factor-missing-middle';
   if(testcase.levels===null&&!testcase.na_translate&&testcase.limits_name==='first')sample='suppressed-missing';
   if(testcase.levels===null&&!testcase.na_translate&&testcase.limits_name==='only')sample='empty-palette';
  }
  if(sample){const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${sample}.${fmt}`),frame.export(fmt));fs.writeFileSync(path.join(out,`${sample}.scene.json`),JSON.stringify(frame.scene(),null,2));}
 }finally{for(const value of owned.reverse())value.dispose();}
}
for(const kind of ['hue','identity'])for(const limitsName of ['auto','first'])for(const translate of [false,true]){
 const selected=Object.fromEntries(cases.filter(v=>v.kind===kind&&(kind==='hue'||v.guide)&&v.levels===null&&v.limits_name===limitsName&&v.na_translate===translate&&v.drop).map(v=>[v.population,v])),owned=[];
 try{
  const original=dataFor(selected.mixed);owned.push(original);const plot=build(original,selected.mixed);owned.push(plot);const wire=plot.toJson(),chart=plot.chart();owned.push(chart);const request=output.request(plot,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['finite','missing','empty','mixed']){
   const testcase=selected[population],replacement=dataFor(testcase);owned.push(replacement);const updates=chart.transaction();owned.push(updates);const builder=updates.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));
   const fresh=build(replacement,testcase);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=check(chart,testcase);assert.deepEqual(actual,check(batch,testcase));assert.deepEqual(held.scene(),scene);assert.equal(plot.toJson(),wire);records.push({kind,replacement:population,limits:limitsName,translate,...actual});
  }
 }finally{for(const value of owned.reverse())value.dispose();}
}
const positionCases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/missing-paint-positions.json'))).cases;
for(const [index,testcase] of positionCases.entries()){
 const owned=[];
 try{
  const columns=Object.fromEntries(['x','y'].map(axis=>[axis,testcase[axis+'_kind']==='category'?c.categorical(testcase[axis]):c.column(testcase[axis],{kind:'float64'})]));columns.v=c.column(testcase.values,{kind:'string'}).nullable(true);
  const data=c.Data.columns(columns,{keys:[100n,101n,102n,103n]});owned.push(data);const spec=descriptor({...testcase,drop:true,levels:null});spec.guide='Hidden';
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('v').colorScale('v')).scale(c.colorMapped('v',spec)).layer(c.points()).xAxis(c.xAxis().range(100,540)).yAxis(c.yAxis().range(300,60)).build();owned.push(plot);
  const request=output.request(plot,options);owned.push(request);const frame=request.prepare();owned.push(frame);const scene=frame.scene(),points=Array(4).fill(null);
  scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(!targets.length||!('Source' in targets[0]))return;const point=item.primitive.Point,row=Number(targets[0].Source.key)-100;assert.equal(points[row],null);points[row]=[(point.center.x-100)/440,(300-point.center.y)/240];assert.deepEqual(point.fill,paint(testcase.result.colors[row]));});
  points.forEach((point,row)=>{if(testcase.result.colors[row]===null)assert.equal(point,null);else{assert.ok(point);['x','y'].forEach((axis,i)=>assert.ok(Math.abs(point[i]-testcase.result[axis+'_positions'][row])<1e-12,JSON.stringify(testcase)));}});
  const guides=frame.guides().guides,labels={};
  for(const [axis,side,start,span] of [['x','Bottom',100,440],['y','Left',300,-240]]){const guide=guides.find(g=>g.spec.side===side);labels[axis]=guide.ticks.map(t=>t.label).sort();assert.deepEqual(labels[axis],[...testcase.result[axis+'_labels']].sort());testcase.result[axis+'_labels'].forEach((label,i)=>{const tick=guide.ticks.find(t=>t.label===label);assert.ok(Math.abs((tick.position-start)/span-testcase.result[axis+'_major_positions'][i])<1e-12,JSON.stringify(testcase));});}
  records.push({position_index:index,points,labels});
  if(testcase.x_kind==='category'&&testcase.y_kind==='numeric'&&testcase.population==='mixed'&&testcase.limits_name==='first'&&!testcase.na_translate)for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`unpainted-categories.${fmt}`),frame.export(fmt));
 }finally{for(const value of owned.reverse())value.dispose();}
}
const emptyCase=cases.find(testcase=>testcase.population==='empty'&&testcase.limits_name==='auto'&&testcase.levels===null);
for(const retained of [false,true]){
 const owned=[];
 try{
  const data=dataFor(emptyCase);owned.push(data);const spec=descriptor(emptyCase);spec.ggplot.Discrete.empty_population=retained;
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1).color('v').colorScale('v')).scale(c.colorMapped('v',spec)).layer(c.points()).build();owned.push(plot);const wire=plot.toJson(),encoded=JSON.parse(wire),version=retained?21:17;assert.equal(encoded.version,version);const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  if(retained){encoded.version=20;assert.throws(()=>c.Plot.fromJson(JSON.stringify(encoded)));}else assert.ok(!wire.includes('empty_population'));
  records.push({retained_empty:retained,version});
 }finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,874);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();
console.log('PASS WASM: 256 hue and 512 identity nullable domains, 32 replacements, 72 position panels, two wire cases and six SVG/PDF/PNG samples.');
