'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/continuous-interval-label-functions.json'))).cases.filter(t=>t.kind==='interval');
function descriptor(t){const result=baseDescriptor(t);if(t.channel==='colour')result.function.Interpolated.output={Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}};result.guide={[t.guide==='bins'?'ContinuousBins':'ContinuousSteps']:result.guide.Continuous};return result;}
function baseDescriptor(t){const channel=t.channel,family=t.transform==='sqrt'?{Pow:{exponent:.5}}:t.transform==='log10'?{Log:{base:10}}:'Linear';return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family,domain:[0,1],reverse:t.transform==='reverse',rescaler:'Range'}},output:{Interpolate:{operation:'PowerRange',range:channel==='alpha'?[.1,1]:[1,6],exponent:channel==='size'?.5:1,absolute:false}},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:t.limits==='full'?[1,10]:null,oob:'Censor'}},guide:{Continuous:{breaks:t.break_mode==='auto'?null:t.break_mode==='empty'?[]:[-1,0,1,1,3,20,{number:'Infinity'},{number:'NaN'}],labels:{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}}}}};}
function layer(t){if(t.channel==='colour')return c.points().name('marks');const channel={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth',shape:'Shape',linetype:'LineType'}[t.channel],result=(['linewidth','linetype'].includes(t.channel)?c.line():c.points()).name('marks');return ['shape','linetype'].includes(t.channel)?result.value_scale(channel,'v',descriptor(t)):result.numeric_scale(channel,'v',descriptor(t));}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'float64'}).nullable(true)});}
function build(t,d){let draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3');if(t.channel==='colour')draft=draft.aes(c.aes().x('x').y(1).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)));else draft=draft.aes(c.aes().x('x').y(1));return draft.layer(layer(t)).build();}
function check(t,chart){const state=chart.semantics().layers[0],entries=(t.channel==='colour'?(state.color_legend?.numeric_breaks??[]):Object.values(state.numeric_value_guides??{}).flat()).filter(e=>e.visible),actual={values:entries.map(e=>e.transformed?.number??e.transformed),labels:entries.map(e=>e.label)},expected=t.result.keys?.[0]??{values:[],labels:[]};expected.values=t.result.boundaries.slice(0,expected.values.length);assert.deepEqual(actual.labels,expected.labels,JSON.stringify(t));assert.equal(actual.values.length,expected.values.length,JSON.stringify(t));actual.values.forEach((a,i)=>assert.ok(Number(a)===Number(expected.values[i])||Math.abs(a-Number(expected.values[i]))<=3e-12*Math.max(1,Math.abs(expected.values[i])),JSON.stringify(t)));return actual;}
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,58);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  const old=JSON.parse(wire);old.version=57;assert.throws(()=>c.Plot.from_json(JSON.stringify(old),registry),c.ChartError);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer(t)).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=check(t,chart);assert.ok(!t.result.error,JSON.stringify(t));records.push({index,state,...actual});assert.equal(restored.to_json(),wire);
  }
  if(t.label_mode==='indexed'&&((['ordinary','empty'].includes(t.population)&&t.limits==='full'&&t.break_mode==='auto')||(t.population==='constant'&&t.limits==='none'&&t.break_mode==='empty'))){const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.channel}-${t.guide}-${t.transform}-${t.population}.${fmt}`),frame.export(fmt));}
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_VALIDATION');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(fs.readdirSync(out).filter(f=>f.endsWith('.svg')).length,36);options.dispose();output.dispose();assert.equal(records.length,2720);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));registry.dispose();
console.log('PASS WASM: 2720 continuous interval label states: 820 successes in three states and 260 expected rejections.');
