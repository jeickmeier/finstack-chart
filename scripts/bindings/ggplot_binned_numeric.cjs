'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function descriptor(testcase){
 const kind=testcase.palette,control=testcase.control,area=kind==='area';
 const breaks=control==='explicit'?{Explicit:[1,3,5,7,9]}:['empty_breaks','null_breaks'].includes(control)?{Explicit:[]}:['count_eight','count_sixteen'].includes(control)?{Equal:control==='count_eight'?8:16}:{Nice:5};
 const limits=control==='limits'?[-1,5]:control==='partial_limits'?[null,5]:null;
 return {function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:area?'Maximum':'Range'}},output:{Interpolate:{operation:'PowerRange',range:area?[0,6]:kind==='alpha'?[0.1,1]:[1,6],exponent:['size','area'].includes(kind)?0.5:1,absolute:area}},unknown:{kind:'Missing'}}},training:'Eligible',ggplot:{Binned:{empty_population:false,nonfinite_population:false,limits,oob:'Squish',breaks,right:control!=='left'}},guide:{Binned:'Automatic'}};
}
const records=[];
for(const [index,testcase] of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/binned-numeric-palettes.json'))).cases.entries()){
 if(testcase.control==='left'&&['size','linewidth'].includes(testcase.palette))continue;
 const owned=[],expected=testcase.raw_result,failure=expected.error;
 try{
  const inputs=testcase.inputs,kind=testcase.palette,channel=kind==='linewidth'?'StrokeWidth':kind==='alpha'?'Alpha':'Size';
  const data=c.Data.columns({x:c.column(inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(inputs.map(v=>v??0),{kind:'float64'}).validity(inputs.map(v=>v!==null))});owned.push(data);
  const layer=()=>(kind==='linewidth'?c.rule():c.points()).name('marks').stroke('#000000').numeric_scale(channel,'v',descriptor(testcase));
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1).x2('x').y2(2)).layer(layer()).build();owned.push(plot);
  const wire=plot.to_json(),version=JSON.parse(wire).version,restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=chart.semantics().layers[0];assert.ok(!failure,`${index} ${failure}`);
   let wanted=expected.values;if(wanted.length===1)wanted=inputs.map(()=>wanted[0]);if(kind!=='alpha')wanted=wanted.filter(v=>v!==null);
   const styles=actual.styles??[];assert.equal(styles.length,wanted.length,`${index}`);
   styles.forEach((style,i)=>{const want=wanted[i];if(kind==='alpha')assert.equal(style.color.alpha,want===null?255:Math.floor(want*255+0.5),`${index}`);else assert.ok(Math.abs(style[kind==='linewidth'?'stroke_width':'radius']-want)<2e-12,`${index}: ${JSON.stringify(style)} ${want}`);});
   records.push({index,state,version,styles});
   if(testcase.population==='many'&&testcase.control==='count_eight'&&state==='original'){
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`numeric-${kind}.${fmt}`),frame.export(fmt));
   }
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&failure,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,372);fs.writeFileSync(path.join(out,'binned-numeric-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 372 binned numeric states; twelve publication files.');
