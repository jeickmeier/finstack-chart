'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
function descriptor(testcase){
 const kind=testcase.palette,control=testcase.control;
 const palette=kind==='linetype'?'LineType':kind==='brewer'?{Brewer:{id:'Blues',reverse:false}}:{Shape:{solid:kind==='solid'}};
 const breaks=control==='explicit'?{Explicit:[1,3,5,7,9]}:['empty_breaks','null_breaks'].includes(control)?{Explicit:[]}:['count_eight','count_sixteen'].includes(control)?{Equal:control==='count_eight'?8:16}:{Nice:5};
 const limits=control==='limits'?[-1,5]:control==='partial_limits'?[null,5]:null;
 return {function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range'}},output:'Identity',unknown:kind==='brewer'?{kind:'Color',value:{space:'Rgb',channels:{r:127,g:127,b:127,opacity:1}}}:{kind:'Missing'}}},training:'Eligible',ggplot:{Binned:{palette,empty_population:false,nonfinite_population:false,limits,oob:'Squish',breaks,right:control!=='left'}},guide:{Binned:'Automatic'}};
}
const records=[];
for(const [index,testcase] of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/binned-style-palettes.json'))).cases.entries()){
 const owned=[],expected=testcase.result,failure=expected.error||expected.draw_error;
 try{
  const inputs=testcase.inputs,kind=testcase.palette,channel=kind==='linetype'?'LineType':kind==='brewer'?'Stroke':'Shape';
  const data=c.Data.columns({x:c.column(inputs.map((_,i)=>i),{kind:'float64'}),v:c.column(inputs.map(v=>v??0),{kind:'float64'}).validity(inputs.map(v=>v!==null))});owned.push(data);
  const layer=()=>kind==='brewer'?c.points().name('marks'):(kind==='linetype'?c.rule():c.points()).name('marks').value_scale(channel,'v',descriptor(testcase));
  let mapping=c.aes().x('x').y(1).x2('x').y2(2);if(kind==='brewer')mapping=mapping.stroke('v').stroke_scale('bins').fill('v').fill_scale('bins');let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(layer());if(kind==='brewer')builder=builder.scale(c.color_mapped('bins',descriptor(testcase)));const plot=builder.build();owned.push(plot);
  const wire=plot.to_json();assert.equal(JSON.parse(wire).version,27);const restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const actual=chart.semantics().layers[0];assert.ok(!failure,`${index} ${failure}`);
   const wanted=expected.values.filter(v=>v!==null||kind==='linetype'),aesthetics=actual.aesthetics??[];assert.equal(aesthetics.length,wanted.length,`${index}`);
   aesthetics.forEach((row,i)=>{const got=row[channel];let want=wanted[i];if(['solid','hollow'].includes(kind))assert.deepEqual(got,{kind:'Number',value:want},`${index}`);else if(kind==='linetype')assert.deepEqual(got,want===null?{kind:'Missing'}:{kind:'Text',value:want},`${index}`);else{const rgb=actual.styles[i].stroke;want=want==='grey50'?'#7F7F7F':want;assert.deepEqual([rgb.red,rgb.green,rgb.blue],[1,3,5].map(i=>parseInt(want.slice(i,i+2),16)),`${index}`);}});
   records.push({index,state,version:27,styles:actual.styles??[],aesthetics});
   if(testcase.population==='many'&&testcase.control==='count_eight'&&state==='original'){
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${kind}.${fmt}`),frame.export(fmt));
   }
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&failure,`${index}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,392);fs.writeFileSync(path.join(out,'binned-style-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM:',records.length,'binned count palette states; twelve publication files.');
