'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);
fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/shared-paint-aesthetics.json'))).cases;
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
function descriptor(family){
 const s={training:'Eligible',guide:'Hidden',function:{Ordinal:{domain:[],range:[],unknown:{Explicit:null}}},ggplot:{Discrete:{limits:null,levels:null,drop:true,na_translate:true,palette:{Hue:{h:[15,375],chroma:100,luminance:65,start:0,reverse:false}}}}};
 if(['continuous','binned'].includes(family)){
  s.function={Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range'}},output:{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}},unknown:{kind:'Missing'}}};
  s.ggplot=family==='continuous'?{Continuous:{limits:null,oob:'Censor'}}:{Binned:{limits:null,oob:'Squish',breaks:{Nice:5},right:true}};
 }
 if(family==='manual')s.ggplot.Discrete.palette={Manual:{values:['#FF0000','#008000','#0000FF'].map(value=>({kind:'Text',value})),names:null}};
 if(family==='identity'){delete s.ggplot;s.function={GgplotDiscreteIdentity:{limits:null,levels:null,drop:true,na_translate:true,guide:false,observed:[]}};}
 return s;
}
const layer=()=>c.points().name('marks').size(4).aesthetic_value('Shape',{kind:'Number',value:21});
function paint(raw){if(raw===null)return {red:0,green:0,blue:0,alpha:0};const v=raw==='grey50'?'#7F7F7F':raw;return {red:parseInt(v.slice(1,3),16),green:parseInt(v.slice(3,5),16),blue:parseInt(v.slice(5,7),16),alpha:v.length===9?parseInt(v.slice(7,9),16):255};}
for(const [index,t] of cases.entries()){
 const owned=[];try{
  const kind=['continuous','binned'].includes(t.family)?'float64':'string',d=c.Data.columns({x:c.column(t.colour.map((_,i)=>i),{kind:'float64'}),colour:c.column(t.colour,{kind}),fill:c.column(t.fill,{kind})},{name:'data'});owned.push(d);
  const p=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x('x').y(1).color('colour').fill('fill').color_scale('shared').fill_scale('shared')).scale(c.color_mapped('shared',descriptor(t.family))).layer(layer()).build();owned.push(p);
  const wire=p.to_json(),restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer()).build():restored.edit().theme(c.theme()).build();if(state!=='original')owned.push(current);
   const chart=current.chart();owned.push(chart);const styles=chart.semantics().layers[0].styles??[],wanted=t.result.colour.map((a,i)=>[a,t.result.fill[i]]).filter(([a])=>a!==null);
   assert.equal(styles.length,wanted.length,JSON.stringify(t));assert.equal(styles.length,t.result.mark_count);
   styles.forEach((s,i)=>{assert.deepEqual(s.color,paint(wanted[i][0]));assert.deepEqual(s.fill,paint(wanted[i][1]));});
   records.push({index,state,styles});assert.equal(restored.to_json(),wire);
  }
  if(t.population==='ordinary'){const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.family}.${fmt}`),frame.export(fmt));}
 }finally{for(const o of owned.reverse())o.dispose();}
}
assert.equal(records.length,45);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS 45 joint paint states and 15 publication files');
