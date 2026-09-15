'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/vector-transform-statistic-styles.json'))).cases;
const retained=process.argv.includes('--retained');
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
function descriptor(t){
 let tr={Registered:{selection:{call:{operation:{id:'example.scale_transform_vector',version:'1'},parameters:{family:t.family}}}}};if(t.composed)tr={Compose:{transforms:[tr,'Reverse']}};
 const palette=t.route==='size'?{operation:'PowerRange',range:[1,6],exponent:.5,absolute:false}:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}};
 const result={training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family:{Ggplot:{transform:tr}},domain:[0,1],reverse:false,rescaler:'Range'}},output:{Interpolate:palette},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:null,oob:'Censor'}}};
 if(retained){const population=t.faceted?[1,1,1,1,2]:[2,1,3],mean=population.reduce((a,b)=>a+b,0)/population.length;let transformed=population.map(v=>t.family==='cardinality'?v+population.length:v-mean);if(t.composed)transformed=transformed.map(v=>-v);result.trained_transformed_bounds=[Math.min(...transformed),Math.max(...transformed)];}
 return result;
}
function layer(t){let l=c.points().name('marks').stat(c.count().group('x'));return t.route==='size'?l.numeric_scale('Size',{Statistical:'Count'},descriptor(t)):l.after_stat(c.stat_aes().x('Group').y('Count').color('Count').color_scale('paint'));}
function check(t,chart){
 const semantic=chart.semantics(),panels=t.faceted?semantic.panels:[semantic],expected=Object.values(t.result.panels),result=[];assert.equal(panels.length,expected.length);
 panels.forEach((panel,i)=>{const styles=panel.layers[0].styles??[],wanted=expected[i];assert.equal(styles.length,wanted.mapped.length);
 styles.forEach((style,j)=>{const v=wanted.mapped[j];if(t.route==='size')assert.ok(Math.abs(style.radius-v)<=4e-14*Math.max(1,Math.abs(v)),JSON.stringify({t,style,v}));else assert.deepEqual(style.color,{red:parseInt(v.slice(1,3),16),green:parseInt(v.slice(3,5),16),blue:parseInt(v.slice(5,7),16),alpha:255});});result.push(styles);});return result;
}
for(const [index,t] of cases.entries()){
 const owned=[];try{
  const d=c.Data.columns({x:c.categorical(['a','a','b','c','c','c']),panel:['a','b','a','a','b','b']});owned.push(d);
  let builder=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(layer(t));if(t.route==='paint')builder=builder.scale(c.color_mapped('paint',descriptor(t)));if(t.faceted)builder=builder.facet(c.facet_wrap('panel').columns(2));
  const p=builder.build();owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,retained?56:55);if(retained){const old=JSON.parse(wire);old.version=55;assert.throws(()=>c.Plot.from_json(JSON.stringify(old),registry),error=>error instanceof c.ChartError&&/version/i.test(String(error)));}const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer(t)).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);
   try{const chart=current.chart();owned.push(chart);const styles=check(t,chart),request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!t.result.error);records.push({index,state,styles});if(state==='original')for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${String(index).padStart(3,'0')}-${t.route}.${fmt}`),frame.export(fmt));}
   catch(error){assert.ok(error instanceof c.ChartError&&t.result.error&&error.code==='CHART_NUMERICAL_DOMAIN',`case${index}: ${error}`);records.push({index,state,error:error.code});}
   assert.equal(restored.to_json(),wire);
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error&&error.code==='CHART_NUMERICAL_DOMAIN',`case${index}: ${error}`);records.push({index,state:'build',error:error.code});}finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.deepEqual(records.filter(r=>r.state==='build').map(r=>r.index),[12,14]);assert.deepEqual(records.filter(r=>r.error).map(r=>[r.index,r.state]),[[12,"build"],[13,"original"],[13,"layer_edit"],[13,"theme_edit"],[14,"build"],[15,"original"],[15,"layer_edit"],[15,"theme_edit"]]);assert.equal(records.length,44);assert.equal(fs.readdirSync(out).filter(v=>/\.(svg|pdf|png)$/.test(v)).length,36);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS 44 generated style states and 36 publications');
