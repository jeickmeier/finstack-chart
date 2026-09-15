'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
let fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/scale-transform-contracts.json'))),registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
const positionMinor=process.argv.includes('--position-minors');
const positionExplicit=process.argv.includes('--position-explicit');
const positionFunctions=process.argv.includes('--position-functions')||positionExplicit||positionMinor;
const nullBreaks=process.argv.includes('--null-breaks');
const scaleFunctions=process.argv.includes('--scale-functions')||nullBreaks;
const identityFunctions=process.argv.includes('--identity-functions');
const identityVectors=process.argv.includes('--identity-vectors')||identityFunctions;
const vectors=process.argv.includes('--vectors')||identityVectors||scaleFunctions||positionFunctions,retained=process.argv.includes('--retained');assert.ok(!retained||vectors);
if(vectors){const source=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/vector-transforms.json')));fixture={cases:source.configurations,plots:source.cases};}
const minors=process.argv.includes('--minors');
const probability=process.argv.includes('--probability'),registered=process.argv.includes('--registered')||probability;
if(registered){const source=JSON.parse(fs.readFileSync(path.join(root,probability?'fixtures/parity/ggplot2/probability-transforms.json':'fixtures/parity/ggplot2/registered-transforms.json')));fixture={cases:source.configurations,plots:source.cases};}
if(minors){const source=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/registered-transform-minors.json')));fixture={cases:source.configurations,plots:source.cases.map(t=>({...t,route:'position'}))};}
const compositions=process.argv.includes('--compositions');
if(compositions)fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/transform-compositions.json')));
const secondary=process.argv.includes('--secondary'),labels=process.argv.includes('--labels'),guides=process.argv.includes('--guides')||labels;
if(guides)fixture.plots=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/transform-guide-controls.json'))).cases.filter(t=>labels||t.format==='default').map(t=>({...t,route:'position',population:'ordinary'}));
if(secondary)fixture.plots=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/transform-secondary.json'))).cases.map(t=>({...t,route:'position',population:'ordinary',count:3}));
const identityGuides=process.argv.includes('--identity-guides')||identityVectors;
if(identityGuides){const source=JSON.parse(fs.readFileSync(path.join(root,identityFunctions?'fixtures/parity/ggplot2/identity-vector-functions.json':identityVectors?'fixtures/parity/ggplot2/identity-vector-transforms.json':'fixtures/parity/ggplot2/identity-transform-guides.json')));fixture={cases:source.configurations,plots:source.cases.map(t=>({...t,route:'identity',break_mode:t.break_mode??'auto'}))};}
if(scaleFunctions){const source=JSON.parse(fs.readFileSync(path.join(root,nullBreaks?'fixtures/parity/ggplot2/vector-null-breaks.json':'fixtures/parity/ggplot2/vector-scale-functions.json')));fixture={cases:source.configurations,plots:source.cases};}
if(positionFunctions){const source=JSON.parse(fs.readFileSync(path.join(root,positionMinor?'fixtures/parity/ggplot2/vector-position-minors.json':positionExplicit?'fixtures/parity/ggplot2/vector-position-explicit.json':'fixtures/parity/ggplot2/vector-position-callbacks.json')));fixture={cases:source.configurations,plots:source.cases};}
const expectedStates=positionMinor?600:positionExplicit?360:positionFunctions?180:nullBreaks?1056:scaleFunctions?3168:identityFunctions?864:identityVectors?288:identityGuides?696:vectors?180:minors?360:registered?108:compositions?543:secondary?87:labels?348:guides?174:342,expectedFiles=positionMinor?36:positionExplicit?54:positionFunctions?27:nullBreaks?36:scaleFunctions?39:identityVectors?18:identityGuides?24:vectors?54:minors?36:registered?36:compositions?177:secondary?81:labels?342:171;
function transform(case_){
 if(vectors){const tr={Registered:{selection:{call:{operation:{id:'example.scale_transform_vector',version:'1'},parameters:{family:case_.family}}}}};return case_.composed?{Compose:{transforms:[tr,'Reverse']}}:tr;}
 if(minors){const tr={Registered:{selection:{call:{operation:{id:'example.scale_transform_minor',version:'1'},parameters:{family:case_.family,mode:case_.mode}}}}};return case_.composed?{Compose:{transforms:[tr,'Reverse']}}:tr;}
 if(probability)return {Registered:{selection:{call:{operation:{id:'example.probability_transform',version:'1'},parameters:case_.family==='unif'?{distribution:'Uniform',min:case_.custom?-2:0,max:case_.custom?3:1}:{distribution:'Exponential',rate:case_.custom?2:.5}}}}};
 if(registered)return {Registered:{selection:{call:{operation:{id:'example.scale_transform',version:'1'},parameters:{family:case_.family,custom:case_.custom}}}}};
 if(case_.transforms)return {Compose:{transforms:case_.transforms.map(p=>transform({constructor:p.name,args:p.args}))}};
 const name=case_.constructor,a=Array.isArray(case_.args)?{}:case_.args,simple={asinh:'Asinh',asn:'Asn',atanh:'Atanh',identity:'Identity',log1p:'Log1p',reciprocal:'Reciprocal',reverse:'Reverse',sqrt:'Sqrt'};
 if(simple[name])return simple[name];
 if(['log','log10','log2','exp'].includes(name))return {[name==='exp'?'Exp':'Log']:{base:a.base??(name==='log10'?10:name==='log2'?2:Math.E)}};
 if(['boxcox','modulus'].includes(name))return {[name==='boxcox'?'BoxCox':'Modulus']:{p:a.p,offset:a.offset??(name==='boxcox'?0:1)}};
 if(name==='yj')return {YeoJohnson:{p:a.p}};
 if(name==='pseudo_log')return {PseudoLog:{sigma:a.sigma??1,base:a.base??Math.E}};
 if(name==='probit'||a.distribution==='norm')return {Normal:{mean:a.mean??0,sd:a.sd??1}};
 if(name==='logit'||a.distribution==='logis')return {Logistic:{location:a.location??0,scale:a.scale??1}};
 throw Error(JSON.stringify(case_));
}
let identityCase,identityTransform;
const layer=()=>{let result=c.points().name('marks');if(scaleFunctions&&identityCase.route==='size')result=result.numeric_scale('Size','value',functionDescriptor(identityCase,identityTransform));if(identityGuides){const t=identityCase;const spec={training:'Eligible',function:{GgplotNumericIdentity:{transform:{Ggplot:{transform:identityTransform}},limits:t.limits==='fixed'?[0,10]:null,guide:t.guide==='legend',trained:null}},guide:{Continuous:{breaks:t.break_mode==='explicit'?t.inputs:null,labels:'Automatic'}}};if(identityFunctions){if(t.limit_mode==='reverse')spec.limits_function={operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:'reverse'}};if(t.break_mode!=='auto')spec.breaks_function={operation:{id:'example.breaks_limits',version:'1'},parameters:t.break_mode};}result=result.numeric_scale('Size','value',spec);}return result;};
const number=v=>typeof v==='number'?v:({Infinity:Infinity,'-Infinity':-Infinity,NA:NaN,NaN:NaN,'-0':-0})[v.number];
function paint(v){if(v==='grey50')v='#7F7F7F';return {red:parseInt(v.slice(1,3),16),green:parseInt(v.slice(3,5),16),blue:parseInt(v.slice(5,7),16),alpha:255};}
function descriptor(t){return {training:'Eligible',guide:'Hidden',function:{Interpolated:{normalization:{Ggplot:{family:{Ggplot:{transform:t}},domain:[0,1],reverse:false,rescaler:'Range'}},output:{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:null,oob:'Censor'}}};}
function functionDescriptor(t,tr){
 const spec=descriptor(tr);
 if(t.route==='size')spec.function.Interpolated.output={Interpolate:{operation:'PowerRange',range:[1,6],exponent:.5,absolute:false}};
 if(t.kind==='binned')spec.ggplot={Binned:{limits:null,oob:'Squish',breaks:{Nice:5},right:true}};
 spec.guide=t.guide==='none'?'Hidden':t.kind==='binned'?{BinnedLegend:'Automatic'}:{Continuous:{breaks:null,labels:'Automatic'}};
 if(t.limit_mode==='reverse')spec.limits_function={operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:'reverse'}};
 if(t.break_mode!=='auto')spec.breaks_function={operation:{id:'example.breaks_limits',version:'1'},parameters:t.break_mode};
 return spec;
}
for(const [index,t] of fixture.plots.entries()){
 const owned=[];try{
  const values=t.inputs.map(number),d=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),value:c.column(values,{kind:'float64'}).nullable(true)},{name:'data'});owned.push(d);
  const tr=transform(fixture.cases[t.configuration]);identityCase=t;identityTransform=tr;let builder=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).layer(layer()).y_axis(c.y_axis().visible(false));
  if(t.route==='position'){
   let axis=c.x_axis().scale(c.scale_transform({Ggplot:{transform:tr}})).range(100,540).breaks_function(positionFunctions&&!positionExplicit&&!positionMinor?{operation:{id:'example.breaks_limits',version:'1'},parameters:t.break_mode}:null).minor_breaks(minors&&t.override==='hidden'?'Hidden':minors&&t.override==='explicit'?{Numeric:[-1.5,0.5]}:null).expansion(secondary?{mult:[0,0],add:[0,0]}:null).tick_format(t.format==='two_digits'?{Registered:{operation:{id:'example.scale_labels',version:'1'},parameters:'fixed_two'}}:null).tick_arguments(t.count===undefined?null:{count:t.count}).guide_geometry({labels:'Preserve'});
   if(positionExplicit||(positionMinor&&t.break_values!==null)){const ticks=t.break_values.map(v=>typeof v==='object'?{Number:{number:v.number==='NA'?'NaN':v.number}}:v);axis=t.explicit_labels!=null?axis.ticks(ticks.map((v,i)=>[v,t.explicit_labels[i]])):axis.tick_values(ticks);}
   if(positionMinor)axis=axis.minor_breaks(t.minor_mode==='explicit'?{Numeric:[2,5]}:{Registered:{operation:{id:'example.breaks_minor_two',version:'1'},parameters:t.minor_mode}});
   builder=builder.aes(c.aes().x('value').y(1)).x_axis(axis);
  }
  else if(scaleFunctions){builder=builder.aes(t.route==='paint'?c.aes().x('x').y(1).color('value').color_scale('paint'):c.aes().x('x').y(1));if(t.route==='paint')builder=builder.scale(c.color_mapped('paint',functionDescriptor(t,tr)));}
  else if(identityGuides)builder=builder.aes(c.aes().x('x').y(1));
  else {const mapped=descriptor(tr);if(vectors){delete mapped.guide;if(t.route==='paint')mapped.guide={Continuous:{breaks:null,count:t.count,labels:'Automatic'}};}if(probability&&t.route==='binned_paint')delete mapped.guide;if(t.route==='binned_paint')mapped.ggplot={Binned:{limits:null,oob:'Squish',breaks:{Nice:vectors?(t.count??5):5},right:true}};if(retained){const case_=fixture.cases[t.configuration],present=values.filter(v=>!Number.isNaN(v)),offset=case_.family==='cardinality'?-values.length:present.reduce((a,b)=>a+b,0)/present.length,transformed=values.map(v=>(case_.composed?-1:1)*(v-offset)).filter(Number.isFinite);mapped.trained_transformed_bounds=transformed.length?[Math.min(...transformed),Math.max(...transformed)]:[0,1];}builder=builder.aes(c.aes().x('x').y(1).color('value').color_scale('paint')).scale(c.color_mapped('paint',mapped));}
  if((identityVectors||scaleFunctions||positionFunctions)&&t.layers==='two'){const values=t.second.map(number),d2=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),value:c.column(values,{kind:'float64'}).nullable(true)},{name:'second'});owned.push(d2);builder=builder.layer(layer().name('second').data(d2));}
  if(secondary)builder=builder.axis(c.x_axis().name('second').side('Top').secondary('x',2,1).guide_geometry({labels:'Preserve'}));
  const p=builder.build();owned.push(p);let wire=p.to_json();assert.equal(JSON.parse(wire).version,retained&&t.route!=='position'?56:(registered||minors||vectors)?55:compositions?54:53);if(retained&&t.route!=='position'){const old=JSON.parse(wire);old.version=55;assert.throws(()=>c.Plot.from_json(JSON.stringify(old),registry),error=>error instanceof c.ChartError&&/version/i.test(String(error)));}if((identityVectors||(scaleFunctions&&t.route==='size'))&&t.layers==='two'){const envelope=JSON.parse(wire);envelope.definition.layers[1].numeric_scales.Size.id=envelope.definition.layers[0].numeric_scales.Size.id;wire=JSON.stringify(envelope);}const restored=c.Plot.from_json(wire,registry);owned.push(restored);if((identityVectors||(scaleFunctions&&t.route==='size'))&&t.layers==='two')wire=restored.to_json();assert.equal(restored.to_json(),wire);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer()).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);
   try{
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!t.result.error,JSON.stringify(t));const chart=current.chart();owned.push(chart);const semantics=chart.semantics().layers[0];let result;
    if(scaleFunctions){
     const mapped=[],guideRows=[],expected=t.result.keys[0]??{values:[],labels:[]};
     for(const [i,layerState] of chart.semantics().layers.entries()){
      const actual=(layerState.styles??[]).map(s=>t.route==='paint'?s.color:s.radius),wanted=t.route==='paint'?t.result.mapped[i].map(paint):t.result.mapped[i].map(number).filter(v=>!Number.isNaN(v));
      assert.equal(actual.length,wanted.length,`case${index} ${state}`);
      if(t.route==='paint')assert.deepEqual(actual,wanted,`case${index} ${state}`);else actual.forEach((a,j)=>assert.ok(Math.abs(number(a)-wanted[j])<=4e-14*Math.max(1,Math.abs(wanted[j])),`case${index} ${state} mapped`));
      const entries=(t.route==='paint'?(layerState.color_legend?.numeric_breaks??[]):Object.values(layerState.numeric_value_guides??{}).flat()).filter(e=>e.visible),row={values:entries.map(e=>e.transformed),labels:entries.map(e=>e.label)};
      assert.deepEqual(row.labels,expected.labels,`case${index} ${state}`);assert.equal(entries.length,expected.values.length,`case${index} ${state}`);
      row.values.forEach((a,j)=>assert.ok(Math.abs(number(a)-number(expected.values[j]))<=4e-14*Math.max(1,Math.abs(number(expected.values[j]))),`case${index} ${state} guide`));
      mapped.push(actual);guideRows.push(row);
     }
     result={mapped,guides:guideRows};
    }else if(identityGuides){const entries=Object.values(semantics.numeric_value_guides??{}).flat().filter(e=>e.visible),expected=t.result.keys[0]??{values:[],labels:[]};result={values:entries.map(e=>e.transformed),labels:entries.map(e=>e.label)};assert.deepEqual(result.labels,expected.labels,`case${index}`);assert.equal(entries.length,expected.values.length,`case${index}`);result.values.forEach((a,i)=>{const b=number(expected.values[i]);assert.ok(Math.abs(number(a)-b)<=4e-14*Math.max(1,Math.abs(b)),`case${index} ${a} ${b}`);});if(identityVectors){const mapped=chart.semantics().layers.map(layer=>(layer.styles??[]).map(s=>s.radius));mapped.forEach((actual,i)=>{const wanted=t.result.mapped[i].map(number).filter(v=>!Number.isNaN(v));assert.equal(actual.length,wanted.length,`case${index}`);actual.forEach((a,j)=>assert.ok(Math.abs(number(a)-wanted[j])<=4e-14*Math.max(1,Math.abs(wanted[j])),`case${index} mapped ${a} ${wanted[j]}`));});result.mapped=mapped;}
    }else if(minors){
     const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks??[],expected=t.result.minor_positions;
     assert.equal(ticks.length,expected.length,`case${index}`);ticks.forEach((tick,i)=>assert.ok(Math.abs((tick.position-100)/440-expected[i])<4e-14,`case${index} minor position`));result={minor_ticks:ticks};
    }else if(t.route==='position'){
     const ticks=frame.guides().guides.find(g=>g.spec.side===(secondary?'Top':'Bottom')).ticks.sort((a,b)=>a.position-b.position),expected=t.result.positions.map((v,i)=>[v,(registered?(Array.isArray(t.result.panel_labels)?t.result.panel_labels:[]):t.result.labels)[i]]).filter(([v])=>typeof v==='number').sort((a,b)=>a[0]-b[0]);
     assert.equal(ticks.length,expected.length,`case${index}`);
     ticks.forEach((tick,i)=>{assert.equal(tick.label,expected[i][1],`case${index}`);assert.ok(Math.abs((tick.position-100)/440-expected[i][0])<(secondary?2e-12:4e-14),`case${index} position`);});result={ticks};if(positionMinor){const minorTicks=frame.guides().guides.find(g=>g.spec.side==='Bottom').minor_ticks??[],[lo,hi]=t.result.range.map(number),expectedMinor=t.result.minor.map(number).filter(Number.isFinite).map(v=>(v-lo)/(hi-lo));assert.equal(minorTicks.length,expectedMinor.length,`case${index}`);minorTicks.forEach((tick,i)=>assert.ok(Math.abs((tick.position-100)/440-expectedMinor[i])<4e-14,`case${index}`));result.minor_ticks=minorTicks;}
    }else{const styles=semantics.styles??[],expected=t.result.mapped.map(paint);assert.deepEqual(styles.map(s=>s.color),expected,`case${index}`);result={styles};}
    records.push({index,state,...result});if(state==='original'&&((positionFunctions&&t.population==='ordinary'&&t.layers==='one'&&(!positionMinor||['explicit','majors'].includes(t.minor_mode)))||(scaleFunctions&&t.population==='ordinary'&&t.guide==='legend'&&t.layers==='one'&&t.limit_mode==='none'&&(nullBreaks||t.break_mode==='mixed'||index===648))||(!positionFunctions&&!scaleFunctions&&(!identityFunctions||t.break_mode==='mixed')&&(identityVectors?([0,1,2].includes(t.configuration)&&t.guide==='legend'&&t.layers==='one'):(!identityGuides||([9,12,13,21].includes(t.configuration)&&t.guide==='legend')))&&(!minors||(t.count===null&&t.override==='default'))&&t.population===(compositions?'positive':'ordinary'))))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${String(index).padStart(3,'0')}-${t.route}.${fmt}`),frame.export(fmt));
   }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error&&(positionExplicit?['CHART_NUMERICAL_DOMAIN','CHART_VALIDATION','CHART_SCHEMA_CONFLICT']:(compositions||vectors)?['CHART_NUMERICAL_DOMAIN','CHART_VALIDATION']:['CHART_NUMERICAL_DOMAIN']).includes(error.code),`case${index}: ${error}`);records.push({index,state,error:error.code});}
   assert.equal(restored.to_json(),wire);
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error&&(positionExplicit?['CHART_NUMERICAL_DOMAIN','CHART_VALIDATION','CHART_SCHEMA_CONFLICT']:(compositions||vectors)?['CHART_NUMERICAL_DOMAIN','CHART_VALIDATION']:['CHART_NUMERICAL_DOMAIN']).includes(error.code),`case${index}: ${error}`);records.push({index,state:'build',error:error.code});}finally{for(const obj of owned.reverse())obj.dispose();}
}
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));if(vectors&&!identityVectors&&!scaleFunctions&&!positionFunctions)assert.deepEqual(records.filter(r=>r.state==='build').map(r=>r.index),Array.from({length:18},(_,i)=>54+i));if(scaleFunctions)assert.deepEqual(records.filter(r=>r.state==='build').map(r=>r.index),fixture.plots.flatMap((t,i)=>t.configuration===3&&t.route==='paint'?[i]:[]));if(positionFunctions)assert.deepEqual(records.filter(r=>r.state==='build').map(r=>r.index),fixture.plots.flatMap((t,i)=>t.configuration===3?[i]:[]));assert.equal(records.length,expectedStates);assert.equal(fs.readdirSync(out).filter(v=>/\.(svg|pdf|png)$/.test(v)).length,expectedFiles);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log(`PASS ${records.length} transform host states and ${expectedFiles} publications`);
