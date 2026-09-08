'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),{dataset}=require('./families.cjs');
function subset(actual,expected){
 if(Array.isArray(expected)){assert.equal(actual.length,expected.length);expected.forEach((v,i)=>subset(actual[i],v));}
 else if(expected!==null&&typeof expected==='object'){for(const [k,v]of Object.entries(expected))subset(actual[k],v);}
 else if(typeof expected==='number')assert(Math.abs(actual-expected)<=1e-12*Math.max(Math.abs(expected),1),`${actual} != ${expected}`);
 else assert.deepEqual(actual,expected);
}
function author(c,fixture,actions=false){
 const source=dataset(c,fixture);
 if(actions)return c.plot(source).aes(c.aes().x('x').y('y')).layer(c.histogram().breaks([0,1,2]).color('#b4c8d78c')).layer(c.line()).layer(c.points()).layer(c.labels().id('threshold').figureAt(.1,.1).text('Threshold')).build();
 const name=fixture.name,layer=name==='input-bars'?c.bars().width(20):['input-line','input-utc'].includes(name)?c.line():c.points();
 let p=c.plot(source).aes(c.aes().x('field_1').y('field_2')).layer(layer.name('observations').color('#1e7db4d2'));
 if(['input-category','input-host-category-link'].includes(name))p=p.xAxis(c.xAxis().visible(false).scale(c.scalePoint().categories(['Alpha','Beta','Gamma']).pointPadding(.5))).yAxis(c.yAxis().visible(false));
 else if(name==='input-utc')p=p.aes(c.aes().x({field:'field_1',origin:'1709164800000'}).y('field_2')).xAxis(c.xAxis().visible(false).scale(c.scaleUtc().interval({Days:1}))).yAxis(c.yAxis().visible(false));
 else p=p.xAxis(c.xAxis().visible(false).scale(c.scaleLinear().domain(0,4))).yAxis(c.yAxis().visible(false).scale(c.scaleLinear().domain(0,4)));
 if(name==='input-host-tools')p=p.layer(c.callout().label(c.labels().id('Threshold').at(.5,3).text('Threshold').offset(0,-20)).toData(3.5,3).connectorOrigin('Anchor'));
 return p.build();
}
function remapper(fixture,plot,chart){
 const envelope=JSON.parse(plot.toJson()),pairs={epoch:{[fixture.data.epoch]:chart.revisions().epoch.toString()},dataset:{[fixture.data.datasets[0].id]:envelope.data[0].id},layer:Object.fromEntries(fixture.chart.definition.layers.map((v,i)=>[v.id,envelope.definition.layers[i].id]))};
 const remap=(value,key='',reverse=false)=>{
  if(Array.isArray(value))return value.map(v=>remap(v,key,reverse));
  if(value!==null&&typeof value==='object')return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,remap(v,k,reverse)]));
  if(typeof value==='string'){
   let mapping=pairs[key]??{};if(reverse)mapping=Object.fromEntries(Object.entries(mapping).map(([a,b])=>[b,a]));value=mapping[value]??value;
   if(key==='description'){let[a,b]=Object.entries(pairs.dataset)[0];if(reverse)[a,b]=[b,a];value=value.replace(`in dataset ${a}`,`in dataset ${b}`);}
  }return value;
 };return remap;
}
exports.run=(c,root,output,write)=>{
 const read=p=>JSON.parse(fs.readFileSync(path.join(root,p),'utf8')),options=c.exportOptions(400,200).layout(c.layoutOptions().padding(0).fontSize(10)),inputs=[];
 const checked=(call,step)=>{try{const result=call();assert(!('error'in step),step.name);return result;}catch(e){assert(e instanceof c.ChartError);assert.equal(e.code,step.error,step.name);return{error:e.code};}};
 const present=chart=>{const f=chart.present(output,options);try{return f.scene().stamp;}finally{f.free();}};
 for(const fixture of [...read('fixtures/interaction/cases.json'),...read('fixtures/host-tools/cases.json')]){
  const plot=author(c,fixture),chart=plot.chart(),remap=remapper(fixture,plot,chart);let stamp=present(chart),basis=null;const initial=chart.semantics();
  for(const raw of fixture.queries){
   const step=remap(raw),before=chart.state();let result=null;
   if(step.query){const queryStamp={...(step.gesture?basis:stamp)};if(step.stale)queryStamp.layout='999999';result=checked(()=>chart.query(step.query,{gesture:step.gesture??false,stamp:queryStamp}),step);assert.deepEqual(chart.state(),before);if(step.expect)subset(result,step.expect);}
   let action=step.action;const apply=step.apply;
   if(apply==='annotation_preview')action={PreviewGesture:{id:step.id,preview:{Annotation:result.annotation}}};
   else if(apply==='linked')action=result.action;
   else if(apply==='windows')action={SetAxisWindows:result.windows};
   else if(apply==='preview')action={PreviewGesture:{id:step.id,preview:{AxisWindows:result.windows}}};
   else if(apply==='targets')action={Select:{change:step.change??'Replace',targets:result.targets}};
   if(action!==undefined){if(Object.hasOwn(action,'BeginGesture'))basis={...stamp};chart.act(action,{origin:apply==='linked'?result.origin:'Pointer'});if(Object.hasOwn(action,'CancelGesture')||Object.hasOwn(action,'CommitGesture'))basis=null;}
   const after=chart.state();if(step.state)subset(after,step.state);if(step.present)stamp=present(chart);
   const semantic=chart.semantics();assert.deepEqual(semantic.datasets,initial.datasets);semantic.layers.forEach((a,i)=>{assert.deepEqual(a.rows,initial.layers[i].rows);assert.deepEqual(a.domains,initial.layers[i].domains);});
   inputs.push(remap({case:fixture.name,name:step.name,result,state:after},'',true));
  }chart.free();plot.free();
 }
 const fixture={chart:read('fixtures/actions/chart.json'),data:read('fixtures/bindings/data.json')},plot=author(c,fixture,true),chart=plot.chart(),remap=remapper(fixture,plot,chart),actions=[];present(chart);
 for(const raw of read('fixtures/actions/trace.json')){
  const step=remap(raw),before=chart.state(),result=checked(()=>chart.act(step.action,{origin:step.origin??'Control',expected:step.expected_state===undefined?undefined:BigInt(step.expected_state)}),step),after=chart.state();
  if(step.expect)subset(result,step.expect);if(step.state)subset(after,step.state);if(step.error)assert.deepEqual(before,after);if(step.present)present(chart);
  actions.push(remap({name:step.name,result,state:after},'',true));
 }chart.free();plot.free();options.free();
 assert.equal(actions.length,23);assert.equal(inputs.length,47);write('runtime-actions',actions);write('runtime-input',inputs);
 console.log('PASS primary WASM 23 action transitions and 47 independent input steps');
};
exports.runStream=(c,root,output,write)=>{
 const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/streaming/replay.json'),'utf8'));
 const p=c.plot(dataset(c,fixture)).transform(c.transform('bins',c.bin().x('value').breaks([0,10,20,30,50])))
  .transform(c.transform('summary',c.summary().x('value').quantiles([0,.5,1]).emptySumZero(false)))
  .layer(c.points().aes(c.aes().x({field:'event_time',origin:'9007199254741001'}).y('value')).color('#1e7db4d2')).build();
 const chart=p.chart(),remap=remapper(fixture,p,chart),options=c.exportOptions(400,200).layout(c.layoutOptions().padding(0).fontSize(10));
 const present=()=>{const f=chart.present(output,options);f.dispose();f.free();};present();
 const cache=new Map(),trace=[];
 function transaction(wire){
  if(cache.has(wire.id))return cache.get(wire.id);
  let b=chart.transaction().id(wire.id);
  for(const op of wire.operations){
   const [kind,v]=Object.entries(op.mutation)[0];let next;
   if(['AppendBatch','UpsertByKey','ReplaceSnapshot'].includes(kind)){
    const d=dataset(c,{data:{datasets:[{id:'1',batch:v}]}});next=b[{AppendBatch:'append',UpsertByKey:'upsert',ReplaceSnapshot:'replace'}[kind]]('data_0',d);d.dispose();d.free();
   }else if(kind==='RemoveKeys')next=b.remove('data_0',v.map(BigInt));
   else if(kind==='AdvanceWatermark')next=b.watermark('data_0',BigInt(v));
   else if(kind==='SetRetention'){
    if(v.EventTime){const e=v.EventTime;next=b.retainEventTime('data_0','event_time',BigInt(e.width),BigInt(e.watermark),{allowedLateness:BigInt(e.allowed_lateness),late:e.late});}
    else next=b.retainCount('data_0',v.Count??null);
   }else throw Error(kind);
   b.free();b=next;
  }
  const result=b.build();b.free();cache.set(wire.id,result);return result;
 }
 for(const raw of fixture.steps){
  const step=remap(raw);let result;
  try{
   if(step.transaction)result=chart.commit(transaction(step.transaction));
   else if(step.stream){const s=step.stream;
    if(s.ConfigureQueue){const v=s.ConfigureQueue;chart.stream(c.streamOptions().transactions(v.transactions).rows(v.rows).bytes(v.bytes).overload(v.overload));result=chart.streamStatus().limits;}
    else if(s.Enqueue)result=chart.enqueue(transaction(s.Enqueue));
    else result=chart[{Status:'streamStatus',Pinned:'pinned',CommitNext:'commitNext'}[s]]();
   }else result=chart.act(step.action,{origin:'Control'});
   assert(!('error'in step),step.name);
  }catch(e){assert(e instanceof c.ChartError,`${step.name}: ${e.stack}`);assert.equal(e.code,step.error,`${step.name}: ${e.message}`);result={error:e.code};}
  if('expected'in step)subset(result,step.expected);
  const semantics=chart.semantics(),state=chart.state();assert.equal(semantics.store_revision,step.revision);
  if(step.present)present();
  trace.push({name:step.name,result:remap(result,'',true),state:remap(state,'',true),semantics});
 }
 for(const t of cache.values()){t.dispose();t.free();}chart.dispose();chart.free();p.dispose();p.free();assert.equal(trace.length,70);
 write('runtime-stream',trace);console.log('PASS primary WASM 70-step streaming replay');
};
