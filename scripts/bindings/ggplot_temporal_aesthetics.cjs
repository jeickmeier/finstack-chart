'use strict';
// FIX-GG04: exact timestamp inputs, temporal default guides and edits in WASM.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/temporal-aesthetic-defaults.json'))).cases,records=[];
for(const [index,testcase] of cases.entries())for(const [unit,multiplier] of [['s',1n],['ms',1000n],['us',1000000n],['ns',1000000000n]]){
 const owned=[],expected=testcase.result;
 try{
  const values=testcase.inputs,factor=multiplier*(testcase.kind==='date'?86400n:1n),channel=testcase.channel;
  const data=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),y:c.column(values.map((_,i)=>i),{kind:'float64'}),when:c.timestamps(values.map(v=>BigInt(v??0)*factor),unit,'UTC').validity(values.map(v=>v!==null))});owned.push(data);
  const mapping=c.aes().x('x').y('y')[channel==='colour'?'color':channel]('when');
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(c.points().name('marks').stroke('#000000')).build();owned.push(plot);
  const wire=plot.to_json();assert.equal(JSON.parse(wire).version,25);const restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  let baseEncoding;
  for(const state of ['original','edited']){
   const current=state==='original'?restored:restored.edit().layer('marks',c.points().stroke('#000000')).build();if(current!==restored)owned.push(current);
   const chart=current.chart();owned.push(chart);const layer=chart.semantics().layers[0];assert.ok(!expected.error);
   const styles=layer.styles??[],target={size:'Size',alpha:'Alpha',linewidth:'StrokeWidth'}[channel];let wanted=expected.values;
   if(target&&values.length){const encoding=layer.numeric_scales[target];if(baseEncoding===undefined)baseEncoding=encoding;else assert.deepEqual(encoding,baseEncoding);}
   if(channel==='size'||channel==='linewidth')wanted=wanted.filter(v=>v!==null);assert.equal(styles.length,wanted.length,`${index} ${unit}`);
   styles.forEach((style,i)=>{const value=wanted[i];if(channel==='alpha'){assert.equal(style.color.alpha,value===null?255:Math.floor(value*255+0.5));}else if(target){const actual=style[{size:'radius',alpha:'alpha',linewidth:'stroke_width'}[channel]]??null;assert.equal(actual===null,value===null);if(value!==null)assert.ok(Math.abs(actual-value)<2e-12);}else{const paint=style[channel==='fill'?'fill':'color'],rgb=value==='grey50'?[127,127,127]:[1,3,5].map(i=>parseInt(value.slice(i,i+2),16));assert.deepEqual([paint.red,paint.green,paint.blue],rgb);assert.equal(paint.alpha,255);}});
   const legend=channel==='fill'?layer.paint_legends?.Fill:layer.color_legend,guides=legend?.numeric_breaks??[];
   if(['colour','fill'].includes(channel)&&expected.guide)assert.deepEqual(guides.map(v=>v.label),expected.guide.labels);
   records.push({index,unit,state,styles,guides});
   if(unit==='s'&&state==='original'&&testcase.kind==='datetime'&&testcase.population==='spaced'&&['size','alpha','colour','fill'].includes(channel)){
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`temporal-${channel}.${format}`),frame.export(format));
   }
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&expected.error,`${index} ${unit}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,unit,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,440);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 440 temporal states and twelve publication files.');
