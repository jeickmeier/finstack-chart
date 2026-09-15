// FIX-GG04: binned shape defaults and nullable-solid theme/fallback selection.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});const records=[];
const cases=JSON.parse(fs.readFileSync(path.join(ROOT,'fixtures/parity/ggplot2/binned-style-defaults.json'))).cases,registry=c.ExtensionRegistry.example();
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function operation(mode){return {operation:{id:'example.scale_palette',version:'1'},parameters:{mode,channel:'size'}};}
function layer(t){
 const policy=t.route==='null'?{}:{palette:{Shape:{solid:t.route!=='hollow'}}};
 Object.assign(policy,{oob:'Squish',breaks:{Nice:5},right:true,limits:null});
 const scale={function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range'}},output:'Identity',unknown:{kind:'Missing'}}},training:'Eligible',ggplot:{Binned:policy},guide:{BinnedBins:'Automatic'}};
 if(t.route==='null')Object.assign(scale,{palette_function:operation('reject'),palette_theme_aesthetics:['shape']});
 return c.points().name('marks').value_scale('Shape','v',scale);
}
function theme(t){return t.theme_mode==='absent'?c.theme():c.theme().scale_palettes({'palette.shape.continuous':operation(t.theme_mode==='vector'?'constant':'repeat_count')});}
function check(t,chart){
 const data=chart.semantics().layers[0],wanted=t.result.mapped.filter(v=>v!==null),actual=data.aesthetics??[];
 assert.equal(actual.length,wanted.length,JSON.stringify(t));actual.forEach((row,i)=>assert.deepEqual(row.Shape,{kind:'Number',value:wanted[i]}));
 const keys=Object.values(data.numeric_value_guides??{}).flat().filter(v=>v.visible),expected=t.result.guides;
 assert.equal(keys.length,expected.length?expected[0].values.length:0,JSON.stringify(t));
 keys.forEach((key,i)=>{assert.equal(key.label??null,expected[0].labels[i]);const v=expected[0].mapped[i];assert.deepEqual(key.mapped,v===null?{kind:'Missing'}:{kind:'Number',value:v});});
 return {aesthetics:actual,keys};
}
for(const [index,t]of cases.entries()){
 const owned=[];
 try{
  let p;
  try{
   const d=c.Data.columns({x:c.column(t.inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(t.inputs,{kind:'float64'}).nullable(true)},{name:'data'});owned.push(d);
   p=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x('x').y(1)).layer(layer(t)).theme(theme(t)).build();owned.push(p);
   const chart=p.chart();owned.push(chart);chart.semantics();
  }catch(error){if(!error.code||!t.result.error)throw error;records.push({index,rejected:error.code});continue;}
  assert.equal(t.result.error,undefined,JSON.stringify(t));
  const wire=p.to_json(),restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer(t)).build():restored.edit().theme(theme(t)).build();if(state!=='original')owned.push(current);
   const chart=current.chart();owned.push(chart);records.push({index,state,...check(t,chart)});assert.equal(restored.to_json(),wire);
  }
  if(t.population==='ordinary'){
   const request=output.request(restored,options),frame=request.prepare();owned.push(request,frame);
   for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.route}-${t.theme_mode}.${fmt}`),frame.export(fmt));
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,134);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS 134 binned shape default states and 30 publication files');
