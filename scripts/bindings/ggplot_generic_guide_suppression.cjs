'use strict';
// GG-04: actual host continuous legend and break suppression semantics.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/generic-guide-suppression.json'))).cases;
function descriptor(t){const discrete=t.family==='discrete';const s={missing_paint_is_na:t.channel==='colour',training:'Eligible',
 guide:t.breaks==='null'||t.guide==='none'?'Hidden':discrete?{Discrete:{breaks:t.breaks==='empty'?[]:null,labels:'Automatic'}}:{[t.guide==='legend'?'BinnedLegend':'BinnedBins']:'Automatic'},
 function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,4],reverse:false,rescaler:'Range'}},output:{Interpolate:{operation:'PowerRange',range:[0,1],exponent:1,absolute:false}},unknown:{kind:'Missing'}}},
 ggplot:{Binned:{limits:t.limit_mode==='fixed'?[0,4]:null,oob:'Squish',breaks:t.breaks==='default'?{Nice:5}:{Explicit:[]},right:true}},palette_function:{operation:{id:'example.scale_palette',version:'1'},parameters:{mode:discrete?'endpoints':'full',channel:t.channel}}};
 if(discrete){s.function={Ordinal:{domain:[],range:[],unknown:{Explicit:null}}};s.ggplot={Discrete:{limits:t.limit_mode==='fixed'?['a','b','c'].map(Text=>({Text})):null,levels:null,drop:true,na_translate:true,palette:{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}}};}return s;}
const layer=t=>t.channel==='colour'?c.points().name('marks'):c.points().name('marks').numeric_scale(t.channel==='size'?'Size':'Alpha','v',descriptor(t));
function build(t){const values=t.family==='discrete'?t.inputs:t.inputs.map(v=>typeof v==='string'?Number(v):v),data=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:t.family==='discrete'?'string':'float64'}).nullable(true)},{name:'data'});let draft=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry);
 if(t.channel==='colour')draft=draft.aes(c.aes().x('x').y(1).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)));else draft=draft.aes(c.aes().x('x').y(1));return [data,draft.layer(layer(t)).build()];}
function check(t,chart){const state=chart.semantics().layers[0],entries=t.channel==='colour'?(state.color_legend?.entries??[]):Object.values(state.numeric_value_guides??{}).flat().filter(e=>e.visible);
 if(!t.result.guides.length)assert.equal(entries.length,0,JSON.stringify(t));return {keys:entries,styles:state.styles??[]};}
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
for(const [index,t] of cases.entries()){const owned=[];try{
 const [data,p]=build(t);owned.push(data,p);const wire=p.to_json(),restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
 for(const state of ['original','layer_edit','theme_edit']){const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer(t)).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);const chart=current.chart();owned.push(chart);records.push({index,state,...check(t,chart)});assert.equal(restored.to_json(),wire);}
 if(t.population==='ordinary'&&t.guide==='default'){const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.family}-${t.channel}-${t.limit_mode}-${t.breaks}.${fmt}`),frame.export(fmt));}
 }finally{for(const obj of owned.reverse())obj.dispose();}}
assert.equal(records.length,972);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM: 972 generic guide-suppression states and 108 publications');
