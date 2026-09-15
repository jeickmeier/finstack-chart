'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
const named=process.argv.includes('--named-palettes'),scalar=process.argv.includes('--scalar-names'),vectors=named||process.argv.includes('--color-vectors');
let cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2',named?'named-theme-palettes.json':vectors?'theme-palette-values.json':'scale-palette-selection.json'))).cases;
if(vectors)cases=cases.map(t=>({...t,channel:'colour',source:'builtin',theme_mode:'supplied'}));
const matching=(x,t)=>['lookup_variant','family','channel','source','theme_mode','palette','na_mode'].every(k=>x[k]===t[k]);
const operation=(t,mode)=>({operation:{id:'example.scale_palette',version:'1'},parameters:{mode,channel:t.channel}});
function descriptor(t){
 const channel=t.channel,discrete=t.family==='discrete';
 const palette=channel==='colour'?{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}:{operation:'PowerRange',range:channel==='alpha'?[.1,1]:[1,6],exponent:channel==='size'?.5:1,absolute:false};
 const s={missing_paint_is_na:channel==='colour',training:'Eligible',guide:'Hidden',function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range'}},output:{Interpolate:palette},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:null,oob:'Censor'}}};
 if(discrete){s.function={Ordinal:{domain:[],range:[],unknown:{Explicit:null}}};s.ggplot={Discrete:{limits:null,levels:null,drop:true,na_translate:true,palette:channel==='colour'?{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}:{NumericRange:{range:channel==='alpha'?[.1,1]:[2,6],area:channel==='size'}}}};}
 if(t.family==='binned')s.ggplot={Binned:{limits:null,oob:'Squish',breaks:{Nice:5},right:true}};
 if(['explicit','fallback'].includes(t.source))s.palette_function=operation(t,t.source==='explicit'?'full':'index');
 if(t.source!=='explicit')s.palette_theme_aesthetics=t.lookup_aesthetics??[channel];
 return s;
}
const theme=t=>vectors?c.theme().scale_palettes({[`palette.colour.${t.family==='discrete'?'discrete':'continuous'}`]:scalar?t.palette:t.palette_values}):t.theme_mode==='supplied'?c.theme().scale_palettes(Object.fromEntries(Object.entries(t.theme_palettes??{[`palette.${t.channel}.${t.family==='discrete'?'discrete':'continuous'}`]:'first'}).map(([key,mode])=>[key,operation(t,mode)]))):c.theme();
const layer=t=>t.channel==='colour'?c.points().name('marks'):c.points().name('marks').numeric_scale(t.channel==='size'?'Size':'Alpha','v',descriptor(t));
function dataFor(t){const values=t.family==='discrete'?t.inputs:t.inputs.map(v=>typeof v==='string'?Number(v):v);return c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:t.family==='discrete'?'string':'float64'}).nullable(true)},{name:'data'});}
function build(t,data){
 let draft=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry);if(t.theme_mode==='supplied')draft=draft.theme(theme(t));
 if(t.channel==='colour'){let color=c.color_mapped('v',descriptor(t));if(t.na_mode==='grey50')color=color.missing('#7F7F7F');return draft.aes(c.aes().x('x').y(1).color('v').color_scale('v')).scale(color).layer(layer(t)).build();}
 return draft.aes(c.aes().x('x').y(1)).layer(layer(t)).build();
}
const paint=v=>({red:parseInt(v.slice(1,3),16),green:parseInt(v.slice(3,5),16),blue:parseInt(v.slice(5,7),16),alpha:v.length===9?parseInt(v.slice(7,9),16):255});
const number=v=>Number(typeof v==='object'?v.number:v);
function check(t,chart){
 const state=chart.semantics().layers[0],styles=state.styles??[];assert.equal(styles.length,t.result.point_count,JSON.stringify(t));assert.deepEqual(styles.map(s=>s.color),t.result.point_colours.map(paint),JSON.stringify(t));
 if(t.channel==='size'){const wanted=t.result.mapped.filter(v=>v!==null).map(number);assert.equal(styles.length,wanted.length);styles.forEach((s,i)=>{const a=number(s.radius),b=wanted[i];assert.ok(a===b||Math.abs(a-b)<2e-12,`${JSON.stringify(t)} ${a} ${b}`);});}
 return styles;
}
for(const [index,t] of cases.entries()){
 const owned=[];
 try{
  const data=dataFor(t);owned.push(data);const p=build(t,data);owned.push(p);const wire=p.to_json();if(t.source!=='explicit'||t.theme_mode==='supplied')assert.equal(JSON.parse(wire).version,named||(vectors&&t.palette_values.length===1)?47:vectors?46:45);
  const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){const current=state==='original'?restored:restored.edit().layer('marks',layer(t)).build();if(current!==restored)owned.push(current);const chart=current.chart();owned.push(chart);records.push({index,state,styles:check(t,chart)});}
  if(t.population==='ordinary'){
   const alternate=vectors?{...t,palette:named?(t.palette==='viridis'?'hue':'viridis'):(t.palette==='pair'?'three':'pair')}:{...t,theme_mode:t.theme_mode==='supplied'?'absent':'supplied'},expected=cases.find(x=>matching(x,alternate)&&x.population==='ordinary');
   const edited=restored.edit().theme(theme(expected)).build();owned.push(edited);const chart=edited.chart();owned.push(chart);records.push({index,state:'theme_edit',styles:check(expected,chart)});assert.equal(restored.to_json(),wire);
  }
  if((vectors&&(named?['hue','blues','viridis']:['pair','missing','alpha']).includes(t.palette)&&['ordinary','missing'].includes(t.population)&&t.na_mode==='NA')||(!vectors&&t.population==='ordinary'&&t.source==='fallback'&&!t.lookup_variant)){
   const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);const name=vectors?`${t.family}-${t.palette}-${t.population}`:`${t.family}-${t.channel}-${t.theme_mode}`;for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${name}.${fmt}`),frame.export(fmt));
  }
  if(t.population==='ordinary'&&['builtin','fallback'].includes(t.source)){
   const chart=restored.chart();owned.push(chart);const request=output.request(restored,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
   for(const population of ['missing','empty','ordinary']){
    const expected=cases.find(x=>matching(x,t)&&x.population===population),replacement=dataFor(expected);owned.push(replacement);
    const tx=chart.transaction();owned.push(tx);const builder=tx.replace(data,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok(JSON.stringify(chart.commit(update)).includes('Applied'));
    const batchPlot=build(expected,replacement);owned.push(batchPlot);const batch=batchPlot.chart();owned.push(batch);const styles=check(expected,chart);assert.deepEqual(styles,check(expected,batch));assert.equal(restored.to_json(),wire);assert.deepEqual(held.scene(),scene);records.push({index,state:'replacement',population,styles});
   }
  }
  assert.ok(!('error' in t.result),JSON.stringify(t));
 }catch(error){if(!vectors||!('error' in t.result)||!(error instanceof c.ChartError))throw error;records.push({index,state:'reference_error'});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
for(const variant of (vectors?['theme_version','wire_version','lookup_name','theme_key']:['missing_registration','native_only','invalid_parameters','theme_version','wire_version','lookup_name','theme_key'])){
 const owned=[];
 try{
  const t=cases.find(t=>t.source===(vectors?'builtin':'fallback')&&t.theme_mode==='supplied'&&t.population==='ordinary'),d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=JSON.parse(p.to_json()),definition=wire.definition,palettes=definition.theme.scale_palettes,call=Object.values(palettes)[0];
  if(variant==='missing_registration')call.operation.id='example.absent_palette';
  if(variant==='native_only')call.operation.id='example.native_scale_palette';
  if(variant==='invalid_parameters')call.parameters.mode='invalid';
  if(variant==='theme_version')definition.theme.version=named?4:vectors?3:2;
  if(variant==='wire_version')wire.version=named?46:vectors?45:44;
  if(variant==='theme_key')definition.theme.scale_palettes={'palette..continuous':call};
  if(variant==='lookup_name'){const corrupt=value=>{if(value&&typeof value==='object'){if(!Array.isArray(value)&&'palette_theme_aesthetics' in value)value.palette_theme_aesthetics=[''];for(const child of Object.values(value))corrupt(child);}};corrupt(definition);}
  const restored=c.Plot.from_json(JSON.stringify(wire),registry);owned.push(restored);const chart=restored.chart();owned.push(chart);chart.semantics();assert.fail(variant);
 }catch(error){assert.ok(error instanceof c.ChartError,`${variant}: ${error.stack}`);records.push({rejection:variant,code:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,named?4183:vectors?438:979);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS',records.length,'palette selection states and 54 publication files');
