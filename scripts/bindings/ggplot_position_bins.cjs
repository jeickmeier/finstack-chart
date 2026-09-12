'use strict';
// FIX-GG04 actual WASM positional-bin panels, v18 and publication proofs.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current'),records=[];
const close=(a,b)=>assert.ok(Math.abs(a-b)<=1e-11*Math.max(1,Math.abs(a),Math.abs(b)),`${a} != ${b}`);
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-bin-panels.json')));
for(const [index,testcase] of fixture.cases.entries()){
 const expected=testcase.result,owned=[],transform=({sqrt:'Sqrt',reverse:'Reverse',log10:{Log:{base:10}}})[testcase.transform]??null;
 let breaks={[testcase.mode==='equal'?'Equal':'Nice']:3};
 if(['explicit','empty'].includes(testcase.mode))breaks={Explicit:testcase.mode==='explicit'?[-1,1,2,4,10,20]:[]};
 if(testcase.mode==='none')breaks=null;
 const spec={bins:{limits:testcase.limits===null?null:testcase.limits.map(v=>typeof v==='string'?{number:v}:v),breaks,oob:'Squish',right:testcase.right},transform,show_limits:testcase.show_limits};
 try{
  const data=c.Data.columns({x:testcase.population==='constant'?[4,4]:[1,10],y:[1,2]});owned.push(data);
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().scale(c.scaleBinned(spec)).guideGeometry({labels:'Preserve'})).yAxis(c.yAxis().visible(false)).build();owned.push(plot);
  const wire=plot.toJson();assert.equal(JSON.parse(wire).version,18);
  const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
  const chart=restored.chart();owned.push(chart);const semantic=chart.semantics();
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);
  assert.ok(!expected.error,`${index}: unexpected success`);
  const finite=expected.x.filter(v=>typeof v==='number'),domain=semantic.layers[0].domains.x;
  if(finite.length){close(domain.minimum,Math.min(...finite));close(domain.maximum,Math.max(...finite));}
  const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks;
  assert.deepEqual(ticks.map(t=>t.label),expected.drawable_labels,`${index} labels`);
  assert.equal(ticks.length,expected.drawable_break_values.length);
  ticks.forEach((t,i)=>close(t.value.Number,expected.drawable_break_values[i]));
  records.push({index,domain,labels:ticks.map(t=>t.label)});
  let sample;
  if(testcase.mode==='nice'&&testcase.population==='finite'&&testcase.right&&!testcase.show_limits){
   if(testcase.limits===null&&['identity','sqrt','reverse'].includes(testcase.transform))sample=testcase.transform+'-positional-bins';
   else if(JSON.stringify(testcase.limits)==='[1,"Infinity"]'&&testcase.transform==='identity')sample='unbounded-positional-bins';
  }
  if(sample){
   fs.writeFileSync(path.join(out,sample+'.plot.json'),wire);
   for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,sample+'.'+fmt),frame.export(fmt));
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&expected.error,`${index}: ${error.stack}`);records.push({index,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
output.dispose();assert.equal(records.length,960);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));
console.log('PASS WASM: 960 positional-bin panels, v18 and four SVG/PDF/PNG samples.');
