'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const mixedRoutes=process.argv.includes('--mixed-routes'),identityChain=mixedRoutes||process.argv.includes('--identity-chain');let route=0;
const registry=c.ExtensionRegistry.example(),records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function build(d,kind,free,shared,binned,mode,special){
 let target=special?(kind==='panels'?{Panels:['C','A'].map(v=>({values:[{Text:v}]}))}:'Match'):'Broadcast';if(route===3)target=special?'Broadcast':'Match';else if(route===4)target='Match';const scope=special&&kind==='chart'?'Chart':'Facet',presentation=shared&&kind==='chart'?'Broadcast':target;
 let layer=c.points().name('marks');if(shared)layer=layer.from_transform('mean').after_stat(c.stat_aes().x(1).y('Mean'));else if(identityChain&&special)layer=layer.from_transform('upper');else if(identityChain)layer=layer.filter(c.filter('v').minimum(0)).filter(c.filter('v').maximum(100));
 const scale=binned?c.scale_binned({bins:{limits:[0,5],oob:'Squish',right:true,breaks:{Nice:3}}}):c.scale_linear(),axis=c.y_axis().scale(scale).oob_function({operation:{id:'example.scale_vector',version:'1'},parameters:{mode:mode==='default'?mode:'oob_'+mode}});
 let draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('i').y('v')).layer(layer.facet_target(presentation).scope(scope)).y_axis(axis).facet(c.facet_wrap('f').order(((special&&kind==='panels')||[3,4].includes(route))?[['A'],['B'],['C']]:[['A'],['C']]).free_y(free));
 const ancestor=[1,4,5].includes(route)?'Broadcast':route===2?{Panels:['A','C'].map(v=>({values:[{Text:v}]}))}:route===3?'Match':target;
 if(identityChain&&special)draft=draft.transform(c.transform('lower',c.identity_stat()).facet_target(ancestor).scope(route===5?'Chart':scope).filter(c.filter('v').minimum(0))).transform(c.transform('upper',c.identity_stat()).from_transform('lower').facet_target(ancestor).scope(scope).filter(c.filter('v').maximum(100)));
 let mean=c.transform('mean',c.summary().x('v'));if(identityChain)mean=special?mean.from_transform('upper'):mean.filter(c.filter('v').minimum(0)).filter(c.filter('v').maximum(100));
 if(shared)draft=draft.transform(mean.facet_target(target).scope(scope)).layer(c.points().from_transform('mean').facet_target(presentation).scope(scope).after_stat(c.stat_aes().x(2).y('Mean')));
 return draft.build();
}
function data(values){return c.Data.columns({v:c.column(identityChain?[999,...values,-99]:values,{kind:'float64'}),i:c.column(identityChain?[99,1,2,-99]:[1,2],{kind:'float64'}),f:c.column(identityChain?['A','A','C','C']:['A','C'],{kind:'string'})});}
function state(chart){try{const result=[];for(const p of chart.semantics().panels){if(p.key.values[0].Text==='B')assert.ok(p.layers.every(l=>l.point_positions.length===0));else result.push(p.layers.map(l=>l.point_positions));}return {positions:result};}catch(e){assert.ok(e instanceof c.ChartError&&['CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN'].includes(e.code),e.stack);return {error:e.code};}}
for(route of (mixedRoutes?[1,2,3,4,5]:[0]))for(const kind of (mixedRoutes?['panels']:['panels','chart']))for(const free of [false,true])for(const shared of [false,true])for(const binned of [false,true])for(const mode of ['index','reverse','short','empty','null','default']){
 const owned=[];try{
  const d=data([10,20]);owned.push(d);const base=build(d,kind,free,shared,binned,mode,false);owned.push(base);const bc=base.chart();owned.push(bc);const expected=state(bc);
  const p=build(d,kind,free,shared,binned,mode,true);owned.push(p);const wire=p.to_json(),restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const [label,current] of [['original',p],['restored',restored]]){const chart=current.chart();owned.push(chart);const actual=state(chart);assert.deepEqual(actual,expected,JSON.stringify({kind,free,shared,binned,mode}));records.push({route,kind,free,shared,binned,mode,state:label,...actual});}
  if(mode==='index'){
   if(!expected.error){const request=output.request(p,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${mixedRoutes?route+'-':''}${kind}-${+free}-${+shared}-${+binned}.${fmt}`),frame.export(fmt));}
   const chart=p.chart();owned.push(chart);const replacement=data([4,8]);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(d,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied' in chart.commit(update));const fresh=build(replacement,kind,free,shared,binned,mode,true);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=state(chart);assert.deepEqual(actual,state(batch));assert.equal(p.to_json(),wire);records.push({route,kind,free,shared,binned,mode,state:'replaced',...actual});
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,mixedRoutes?520:208);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));output.dispose();options.dispose();registry.dispose();console.log('PASS',records.length,'panel scope vector states');
