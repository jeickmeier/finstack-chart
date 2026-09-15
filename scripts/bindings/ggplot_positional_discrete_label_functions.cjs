'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-discrete-label-functions.json'))).cases;
function dataFor(t){return c.Data.columns({x:c.categorical(t.inputs.map(v=>v??'')).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>BigInt(100+i))});}
const layer=()=>c.points().name('marks');
const key=v=>v===null?'Null':{Text:v};
function build(t,d,family,route){
 const scale=family==='band'?c.scaleBand():c.scalePoint(),policy={levels:['b','a','c','unused'].map(key),limits:t.limits==='full'?['c','b','a',null].map(key):null,drop:t.drop,na_translate:t.translate};
 if(t.break_mode==='named')policy.guide={breaks:['outside','a','a',null,'c'].map(key),break_names:['off','first','duplicate','missing','last']};
 if(route==='policy')(policy.guide??={}).labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};
 let axis=c.xAxis().scale(scale).discretePolicy(policy).range(100,540).guideGeometry({labels:'Preserve'}).tickFormat({Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}});
 if(t.break_mode==='explicit')axis=axis.tickValues(['outside','a','a',null,'c'].map(v=>v===null?{MissingCategory:null}:{Category:v}));
 else if(t.break_mode==='empty')axis=axis.tickValues([]);
 if(route==='policy')axis=axis.tickFormat(null);
 return c.plot(d).withRegistry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(layer()).xAxis(axis).yAxis(c.yAxis().visible(false)).build();
}
function check(t,frame){
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
 if(t.result.range.every(v=>typeof v==='number')){
  const raw=t.result.labels,expected=t.result.values.map((_,i)=>raw[i%raw.length]??'');
  assert.deepEqual(ticks.map(t=>t.label),expected,JSON.stringify(t));
 }return {ticks};
}
for(const [index,t] of cases.entries())for(const family of ['band','point'])for(const route of ['axis','policy']){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d,family,route);owned.push(p);const wire=p.toJson();if(route==='policy'&&t.label_mode!=='default')assert.equal(JSON.parse(wire).version,32);
  const restored=c.Plot.fromJson(wire,registry);owned.push(restored);assert.equal(restored.toJson(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);
   assert.ok(!('error' in t.result),JSON.stringify(t));records.push({index,family,route,state,...check(t,frame)});
   if(state==='original'&&route==='policy'&&family==='band'&&[0,1,2,261].includes(index))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`sample-${index}.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok('error' in t.result,JSON.stringify(t)+' '+error);assert.equal(error.code,'CHART_VALIDATION');records.push({index,family,route,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,4700);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 4700 discrete positional label states and 12 publications.');
