'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function descriptor(testcase,origin,unit,factor){
 const control=testcase.control,date=testcase.kind==='date',args={date,breaks:'Automatic',count:null,labels:'Automatic',format:null};
 if(control==='null_breaks')args.breaks='None';if(control==='empty_breaks')args.breaks={Explicit:[]};if(['explicit_breaks','explicit_labels','bad_labels'].includes(control))args.breaks={Explicit:[-1,0,2,5].map(v=>v*factor)};
 if(control==='width')args.breaks={Width:testcase.width??(date?'2 days':'2 secs')};if(control==='count_two')args.count=2;if(control==='null_labels')args.labels='Hidden';if(control==='explicit_labels')args.labels={Explicit:['Before','Start','Two','After']};if(control==='bad_labels')args.labels={Explicit:['A','B']};if(control==='format')args.format={pattern:date?'%Y-%m-%d':'%H:%M:%S',locale:null};
 const limits=control==='limits'?[-factor,5*factor]:control==='partial_limits'?[null,5*factor]:null;
 return {function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range',timestamp:{origin:String(origin),unit,date}}},output:{Interpolate:{operation:'PowerRange',range:[1,6],exponent:0.5,absolute:false}},unknown:{kind:'Missing'}}},training:'Eligible',ggplot:{Continuous:{empty_population:false,nonfinite_population:false,limits,oob:'Censor'}},guide:control==='hidden_guide'?'Hidden':{Temporal:{origin:String(origin),unit,zone:'Utc',arguments:args}}};
}
const records=[];
const cases=[...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/temporal-aesthetic-guides.json'))).cases,...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/temporal-aesthetic-widths.json'))).cases];
for(const [index,testcase] of cases.entries())for(const [unit,variant,multiplier] of (Object.hasOwn(testcase,'width')?[['us','Microseconds',1000000n]]:[['s','Seconds',1n],['ms','Milliseconds',1000n],['us','Microseconds',1000000n],['ns','Nanoseconds',1000000000n]])){
 const owned=[],expected=testcase.result,failure=expected.error||expected.draw_error;
 try{
  const factor=multiplier*(testcase.kind==='date'?86400n:1n),origin=1704067200n*multiplier,inputs=testcase.inputs;
  const stamp=value=>{value??=testcase.kind==='date'?19723:1704067200;const whole=Math.trunc(value);return BigInt(whole)*factor+BigInt(Math.round((value-whole)*Number(factor)));};
  const data=c.Data.columns({x:c.column(inputs.map((_,i)=>i),{kind:'float64'}),y:c.column(inputs.map((_,i)=>i),{kind:'float64'}),when:c.timestamps(inputs.map(stamp),unit,'UTC').validity(inputs.map(v=>v!==null))});owned.push(data);
  const scale=descriptor(testcase,origin,variant,Number(factor)),source={field:'when',origin:String(origin)},layer=()=>c.points().name('marks').stroke('#000000').numeric_scale('Size',source,scale);
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer()).build();owned.push(plot);
  const wire=plot.to_json(),version=JSON.parse(wire).version;assert.ok([25,26].includes(version));const restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=chart.semantics().layers[0];assert.ok(!failure,`${index} ${failure}`);const styles=actual.styles??[],wanted=expected.values.filter(v=>v!==null);assert.equal(styles.length,wanted.length,`${index} ${unit}`);styles.forEach((style,i)=>assert.ok(Math.abs(style.radius-wanted[i])<2e-12,`${index} ${unit} ${style.radius} ${wanted[i]}`));
   const retained=(actual.numeric_scales?.Size??JSON.parse(current.to_json()).definition.layers[0].numeric_scales.Size).scale;assert.equal(retained.function.Interpolated.normalization.Ggplot.timestamp.origin,String(origin));if(testcase.population==='short'&&testcase.control==='default'){
    const standalone=c.StandaloneScale.fromJson(JSON.stringify({version:testcase.kind==='date'?5:4,spec:retained.function}));owned.push(standalone);const copied=c.StandaloneScale.fromJson(standalone.to_json());owned.push(copied);assert.equal(copied.to_json(),standalone.to_json());inputs.forEach((value,i)=>assert.ok(Math.abs(copied.map(Number(BigInt(value)*factor-origin))-expected.values[i])<2e-12));
   }
   records.push({index,unit,state,version,styles,scale:retained});
   if(unit==='s'&&state==='original'&&['date-short-default','datetime-short-width','date-constant-null_breaks'].includes(`${testcase.kind}-${testcase.population}-${testcase.control}`)){
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${testcase.kind}-${testcase.population}-${testcase.control}.${fmt}`),frame.export(fmt));
   }
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&failure,`${index} ${unit}: ${error.stack}`);assert.equal(error.code,(Object.hasOwn(testcase,'width')||failure.includes('labels'))?'CHART_VALIDATION':'CHART_NUMERICAL_DOMAIN');records.push({index,unit,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,1236);fs.writeFileSync(path.join(out,'guide-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 1236 temporal guide states and nine publication files.');
