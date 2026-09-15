'use strict';
// GG-04: actual temporal legend/colorbar selection, replay, edits and publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const labelCases=process.argv.slice(4).includes('--interval-labels'),fixture=labelCases?'temporal-interval-label-functions.json':'temporal-guide-selection.json',cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2',fixture))).cases,records=[],registry=c.ExtensionRegistry.example();
function colorbar(t){return t.guide==='colourbar'||t.guide==='default'&&t.channel==='colour';}
function descriptor(t,origin,unit,factor){
 const channel=t.channel,date=t.kind==='date',output=channel==='colour'?{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}}:{Interpolate:{operation:'PowerRange',range:channel==='alpha'?[.1,1]:[1,6],exponent:channel==='size'?.5:1,absolute:false}},args={date,breaks:t.breaks==='null'?'None':t.breaks==='empty'?{Explicit:[]}:t.breaks==='explicit'?{Explicit:[-factor,0,factor,factor,3*factor,20*factor,{number:'NaN'}]}:'Automatic'};
 if(labelCases){args.labels={Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:t.label_mode}};if(t.format_mode==='explicit')args.format={pattern:date?'%d/%m':'%Hh%M',locale:null};}
 return {training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,10*factor],reverse:false,rescaler:'Range',timestamp:{origin:String(origin),unit,date}}},output,unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:t.limits==='full'?[0,10*factor]:null,oob:'Censor'}},guide:t.guide==='none'?'Hidden':{[colorbar(t)?'TemporalColorbar':t.guide==='bins'?'TemporalBins':t.guide==='coloursteps'?'TemporalSteps':'Temporal']:{origin:String(origin),unit,zone:'Utc',arguments:args}}};
}
function layer(t,source,scale){return t.channel==='colour'?c.points().name('marks'):c.points().name('marks').numeric_scale(t.channel==='size'?'Size':'Alpha',source,scale);}
function paint(text){return Object.fromEntries(['red','green','blue','alpha'].map((k,i)=>[k,i===3?255:parseInt(text.slice(1+2*i,3+2*i),16)]));}
function roundEven(v){const n=Math.floor(v);return v-n===.5?n+n%2:Math.round(v);}
function check(t,chart,factor,multiplier){
 const state=chart.semantics().layers[0],colour=t.channel==='colour',date=t.kind==='date',entries=(colour?(state.color_legend?.numeric_breaks??[]):Object.values(state.numeric_value_guides??{}).flat()).filter(e=>e.visible),guides=t.result.guides,wanted=guides.length?( ['bins','coloursteps'].includes(t.guide)?guides[0].source_values.slice(0,guides[0].values.length):t.result.raw_breaks.filter(v=>typeof v==='number'&&v>=t.result.limits[0]&&v<=t.result.limits[1])):[],actual={values:entries.map(e=>date?18262+e.transformed/factor:1577836800+e.transformed/multiplier),labels:entries.map(e=>e.label)};
 assert.deepEqual(actual,{values:wanted,labels:guides[0]?.labels??[]},JSON.stringify(t));assert.equal(wanted.length,guides[0]?.values.length??0);
 if(guides.length&&['bins','coloursteps'].includes(t.guide))entries.forEach((entry,i)=>{const mapped=entry.mapped,value=guides[0].mapped[i],actual=['Missing','Null'].includes(mapped.kind)?null:mapped.value;if(mapped.kind==='Color'){assert.equal(actual.space,'Rgb');assert.deepEqual(actual.channels,{r:parseInt(value.slice(1,3),16),g:parseInt(value.slice(3,5),16),b:parseInt(value.slice(5,7),16),opacity:1});}else if(typeof value==='number')assert.ok(Math.abs(actual-value)<=3e-12*Math.max(1,Math.abs(value)),JSON.stringify([t,entry,value]));else assert.deepEqual(actual,value,JSON.stringify([t,entry,value]));});
 const styles=state.styles??[];assert.equal(styles.length,t.result.mapped.length);
 styles.forEach((style,i)=>{const value=t.result.mapped[i];if(colour)assert.deepEqual(style.color,paint(value));else if(t.channel==='size')assert.ok(Math.abs(style.radius-value)<=2e-12*Math.max(1,Math.abs(value)));else assert.equal(style.color.alpha,roundEven(value*255));});
 const ramp=state.color_legend?.colorbar??[],expected=guides[0]?.decor_values??[];assert.equal(ramp.length,expected.length,JSON.stringify(t));
 ramp.forEach((sample,i)=>{assert.ok(Math.abs(sample.value-expected[i])<=4*Number.EPSILON*Math.max(1,Math.abs(expected[i])),JSON.stringify([t,i,sample]));assert.deepEqual(sample.color,paint(guides[0].decor_colors[i]));});
 return {keys:actual,styles,colorbar:ramp};
}
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
for(const [index,t] of cases.entries()){
 for(const [unit,variant,multiplier] of [['s','Seconds',1],['ms','Milliseconds',1000],['us','Microseconds',1000000],['ns','Nanoseconds',1000000000]]){
  const owned=[];
  try{
   const factor=multiplier*(t.kind==='date'?86400:3600),origin=1577836800n*BigInt(multiplier),values=t.inputs,data=c.Data.columns({x:c.column(Float64Array.from(values,(_,i)=>i),{kind:'float64'}),v:c.timestamps(values.map(v=>origin+BigInt(v)*BigInt(factor)),unit,'UTC')});owned.push(data);
   const source={field:'v',origin:String(origin)},scale=descriptor(t,origin,variant,factor);let draft=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3');draft=t.channel==='colour'?draft.aes(c.aes().x('x').y(1).color(source).color_scale('v')).scale(c.color_mapped('v',scale)):draft.aes(c.aes().x('x').y(1));
   const p=draft.layer(layer(t,source,scale)).build();owned.push(p);const wire=p.to_json(),restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
   if(colorbar(t)||['bins','coloursteps'].includes(t.guide)){const version=colorbar(t)?59:60,old=JSON.parse(wire);assert.equal(old.version,version);old.version=version-1;assert.throws(()=>c.Plot.from_json(JSON.stringify(old),registry),c.ChartError);}
   if(t.result.error){assert.throws(()=>{const rejected=restored.chart();owned.push(rejected);rejected.semantics();},c.ChartError);records.push({index,unit,state:'rejected'});continue;}
   for(const state of ['original','layer_edit','theme_edit']){
    const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer(t,source,scale)).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);
    const chart=current.chart();owned.push(chart);records.push({index,unit,state,...check(t,chart,factor,multiplier)});assert.equal(restored.to_json(),wire);
   }
   let selected=(t.guide==='default'&&t.breaks==='auto'&&t.limits===(t.population==='ordinary'?'full':'none'))||(t.population==='ordinary'&&t.limits==='full'&&((t.guide==='colourbar'&&t.breaks==='auto')||(t.guide==='default'&&t.breaks==='null')));
   selected=labelCases?t.label_mode==='indexed'&&t.limits==='full'&&((t.population==='ordinary'&&['auto','explicit'].includes(t.breaks))||(t.population==='empty'&&t.breaks==='auto')):(selected||(['bins','coloursteps'].includes(t.guide)&&t.limits==='full'&&t.population==='ordinary'));
   if(unit==='s'&&selected){const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.kind}-${t.channel}-${t.population}-${t.guide}-${t.breaks}${labelCases?'-format-'+t.format_mode:''}.${fmt}`),frame.export(fmt));}
  }finally{for(const obj of owned.reverse())obj.dispose();}
 }
}
assert.equal(records.length,labelCases?21872:7680);assert.equal(fs.readdirSync(out).filter(f=>f.endsWith('.svg')).length,labelCases?60:66);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();
console.log(`PASS WASM: ${records.length} temporal guide-selection states and ${fs.readdirSync(out).filter(f=>f.endsWith('.svg')).length*3} publications.`);
