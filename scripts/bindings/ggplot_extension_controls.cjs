'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<10;mode++){
 let data,builder;
 if(mode===8){
  data=c.Data.columns({x:c.column([0,1,2],{kind:'float64'}),y:c.column([2,5,10],{kind:'float64'}),w:c.column([1,2,1],{kind:'float64'})});
  const options={method:{Registered:{operation:{id:'example.prescribed_slope',version:'1'},parameters:{slope:2,envelope:1.25}}},terms:null,n:80,full_range:false,se:true,xseq:[-1,0,1,2,3],level:.8};
  builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').layer(c.smooth().stat(c.model_stat(options).x('x').y('y').weight('w')));
 }
 else if(mode===4){data=registry.materialize('example.xy_recipe',1,{x:[0,1,2],y:[2,4,3]});builder=c.autoplot(data,'example.xy_recipe',1,{line:true},registry);}
 else {
  const x=[-3,-2,-1,0,1,2,3],groups=mode===1?c.cut_number(x,3):mode===2?c.cut_width(x,2,{center:0}):c.cut_interval(x,{n:3});
  const clone=groups.copy();assert.deepEqual(groups.value(),clone.value());groups.dispose();assert.throws(()=>groups.value(),c.ChartError);
  const levels=clone.value().levels,column=clone.column();clone.dispose();data=c.Data.columns({x:c.column(x,{kind:'float64'}),y:c.column([1,4,2,5,3,6,4],{kind:'float64'}),group:column});column.dispose();
  const layer=mode===3?c.points().key_glyph('example.diamond_key',1,{padding:.1}):c.points();
  builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').scale(c.color_discrete('groups').domain(levels)).aes(c.aes().x('x').y('y').color('group').color_scale('groups')).layer(layer);
 }
 if(mode===5)builder=builder.legend(c.legend().scale("groups").registered("example.strip_guide",1,{}));
 if(mode===6)builder=builder.facet(c.facet_wrap("group").registered("example.reverse_facets",1,{columns:2}));
 if(mode===7)builder=builder.registered_coordinate("example.wave_coordinate",1,{amplitude:.12}).layer(c.line()).layer(c.line().independent().data(c.Data.columns({x:c.column([-3,3],{kind:"float64"}),y:c.column([3.5,3.5],{kind:"float64"})},{name:"wave-span"})).aes(c.aes().x("x").y("y")));
 if(mode===9)builder=builder.facet(c.facet_wrap("group").reference({labeller:{registered:{operation:{id:"example.facet_labels",version:"1"},parameters:"context"}}}));
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire,registry),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),frame.scene());originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`extension-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`extension-${mode}.scene.json`),JSON.stringify(frame.scene()));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`extension-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
const records=[];
for(const fixture of JSON.parse(fs.readFileSync(path.join(ROOT,'fixtures/parity/ggplot2/vector-helper-controls.json'),'utf8')).cases){
 const x=Array.isArray(fixture.x)?fixture.x:[fixture.x],options={...fixture.options};
 if(fixture.operation==='resolution'){const value=c.resolution(x,{zero:options.zero,integer:fixture.integer});assert.ok(Math.abs(value-fixture.result.value)<1e-14);records.push(value);continue;}
 if('ordered_result' in options){options.ordered=options.ordered_result;delete options.ordered_result;}
 let value;
 try{
  let result;if(fixture.operation==='cut_number'){const {n,...rest}=options;result=c.cut_number(x,n,rest);}else if(fixture.operation==='cut_width'){const {width,...rest}=options;result=c.cut_width(x,width,rest);}else result=c.cut_interval(x,options);
  value=result.value();result.dispose();
 }catch(error){assert.ok('error' in fixture.result,fixture.name+': '+error.message);records.push({error:true});continue;}
 assert.ok(!('error' in fixture.result),fixture.name);for(const key of ['codes','levels','ordered'])assert.deepEqual(value[key],fixture.result[key],fixture.name);records.push(value);
}
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));registry.dispose();output.dispose();console.log('PASS WASM extension controls: ten authors, 30 publications and 35 vector cases');

const summary=c.summarize([1,null,2,3]);assert.equal(summary[0],2);
assert.ok(Math.abs(summary[1]-(2-Math.sqrt(1/3)))<1e-14);assert.deepEqual(c.summarize([null]),[null,null,null]);
const ownedRegistry=c.ExtensionRegistry.example(),kept=ownedRegistry.copy();ownedRegistry.dispose();
const d=c.Data.columns({x:c.column([0,1],{kind:'float64'}),y:c.column([1,2],{kind:'float64'}),g:['a','b']});
const base=c.plot(d).with_registry(kept).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('g').color_scale('groups')).scale(c.color_discrete('groups'));
for(const builder of [base.layer(c.points().key_glyph('example.native_diamond_key',1,{padding:.1})),base.layer(c.points()).legend(c.legend().scale('groups').registered('example.native_strip_guide',1,{})),base.layer(c.points()).facet(c.facet_wrap('g').registered('example.native_reverse_facets',1,{columns:1})),base.layer(c.points()).registered_coordinate('example.native_wave_coordinate',1,{amplitude:.1})]){
 const value=builder.build();assert.throws(()=>value.to_json(),c.ChartError);value.dispose();
}
assert.throws(()=>kept.materialize('example.native_xy_recipe',1,{x:[1],y:[2]}),c.ChartError);
const retained=base.layer(c.points()).registered_coordinate('example.wave_coordinate',1,{amplitude:.1}).build();kept.dispose();d.dispose();assert.ok(retained.to_json().includes('example.wave_coordinate'));retained.dispose();
console.log('PASS WASM native-only protocol rejection and registry copy/disposal retention');
{
 const r=c.ExtensionRegistry.example(),d=c.Data.columns({x:[0,1],y:[1,2],g:['a','b']});
 for(const [operation,mode] of [['example.facet_labels','short'],['example.facet_labels','oversize'],['example.native_facet_labels','context']]){
  const p=c.plot(d).with_registry(r).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).facet(c.facet_wrap('g').reference({labeller:{registered:{operation:{id:operation,version:'1'},parameters:mode}}})).build();
  const engine=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
  assert.throws(()=>operation.startsWith('example.native')?p.to_json():engine.request(p,c.export_options(600,360)).prepare());engine.dispose();p.dispose();
 }r.dispose();d.dispose();
}
