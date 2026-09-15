'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-binned-label-functions.json'))).cases;
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
function build(t,d,route){
 const transform={sqrt:'Sqrt',reverse:'Reverse',log10:{Log:{base:10}}}[t.transform]??null;
 let breaks={[t.break_mode==='nice'?'Nice':'Equal']:5};
 if(['explicit','empty'].includes(t.break_mode))breaks={Explicit:t.break_mode==='explicit'?[-1,0,1,1,3,20,{number:'Infinity'},{number:'NaN'}]:[]};
 const spec={bins:{limits:t.limits==='full'?[1,10]:null,breaks,oob:'Squish',right:true},transform,show_limits:t.show_limits};
 if(route==='policy'&&t.label_mode!=='default')spec.labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};
 const scale=c.scaleBinned(spec);
 let axis=c.xAxis().scale(scale).range(100,540).guideGeometry({labels:'Preserve'}).tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}});
 if(route==='policy'||t.label_mode==='default')axis=axis.tickFormat(null);
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 if(t.result.range.every(v=>typeof v==='number')){
  const expected=t.result.labels.filter((_,i)=>typeof t.result.values[i]==='number').map(v=>v??'');
  assert.deepEqual(ticks.map(t=>t.label),expected,JSON.stringify(t));
  const [lower,upper]=t.result.range,values=t.result.values.filter(v=>typeof v==='number');ticks.forEach((tick,i)=>assert.ok(Math.abs((tick.position-100)/440-(values[i]-lower)/(upper-lower))<=3e-12,JSON.stringify({t,tick}))); 
 }return {ticks};
}
for(const [index,t] of cases.entries())for(const route of ['axis','policy']){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d,route);owned.push(p);const wire=p.toJson();if(route==='policy'&&t.label_mode!=='default')assert.equal(JSON.parse(wire).version,32);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);
   assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,route,state,...check(t,frame)});
   if(state==='original'&&route==='policy'&&[0,1,30,324,1200,1545].includes(index))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,t.result.error.includes('labels')?'CHART_SCHEMA_CONFLICT':'CHART_NUMERICAL_DOMAIN');records.push({index,route,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,4944);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 4944 binned positional label states and 18 publications.');
