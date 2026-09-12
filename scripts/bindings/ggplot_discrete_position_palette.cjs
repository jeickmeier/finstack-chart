'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-position-palettes.json'))).cases,records=[];
const key=v=>v===null?'Null':{Text:v},number=v=>v===null||typeof v==='string'?{number:v??'NaN'}:v;
for(const [index,testcase] of cases.entries())for(const family of ['band','point'])for(const mode of testcase.result.error?['primary']:['primary','secondary']){
 const owned=[],expected=mode==='primary'?testcase.result:testcase.result.secondary;
 try{
  const values=testcase.inputs,data=c.Data.columns({x:c.categorical(values.map(v=>v??'')).validity(values.map(v=>v!==null)),y:c.column(values.map(()=>1),{kind:'float64'})},{keys:values.map((_,i)=>BigInt(100+i))});owned.push(data);
  const axis=c.xAxis().scale(family==='band'?c.scaleBand():c.scalePoint()).range(100,540).discretePolicy({limits:testcase.limits?.map(key)??null,palette:testcase.palette_values.map(number),na_translate:testcase.na_translate}).guideGeometry({labels:'Preserve'});
  let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis);
  if(mode==='secondary')builder=builder.xAxis(c.xAxis().name('secondary').side('Top').secondary('x',1,0).guideGeometry({labels:'Preserve'}));
  const plot=builder.build();owned.push(plot);const wire=plot.to_json();assert.equal(JSON.parse(wire).version,23);const restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
  const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!expected.error);
  const ticks=frame.guides().guides.find(g=>g.spec.side===(mode==='primary'?'Bottom':'Top')).ticks;
  if(mode==='secondary'){
   assert.equal(ticks.length,expected.breaks.length);
   ticks.forEach((tick,i)=>{assert.ok(Math.abs(tick.value.Number-expected.breaks[i])<1e-12);assert.equal(tick.label,expected.labels[i]??'NA');assert.ok(Math.abs((tick.position-100)/440-expected.positions[i])<1e-12,`${index} ${mode}: ${tick.position} ${expected.positions[i]}`);});
  }else{
   assert.equal(ticks.length,expected.major_positions.filter(v=>v!==null).length);
   expected.breaks.forEach((value,i)=>{const semantic=value===null?'MissingCategory':{Category:value},found=ticks.filter(t=>JSON.stringify(t.value)===JSON.stringify(semantic)),position=expected.major_positions[i];if(position===null)assert.equal(found.length,0);else{assert.equal(found.length,1);const tick=found[0];assert.equal(tick.label,expected.labels[i]??'NA');assert.ok(Math.abs((tick.position-100)/440-position)<1e-12);}});
  }
  const scene=frame.scene(),positions=Array(values.length).fill(null);scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(targets.length)positions[Number(targets[0].Source.key)-100]=(item.primitive.Point.center.x-100)/440;});
  positions.forEach((actual,i)=>{const wanted=testcase.result.positions[i];assert.equal(actual===null,wanted===null,`${index} ${mode}`);if(actual!==null)assert.ok(Math.abs(actual-wanted)<1e-12,`${index} ${mode}: ${actual} ${wanted}`);});
  records.push({index,family,mode,ticks,positions});
  if(family==='band'&&([[10,'secondary'],[11,'secondary'],[14,'secondary'],[17,'primary']].some(([i,m])=>i===index&&m===mode)))for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`position-palette-${testcase.palette_name}.${format}`),frame.export(format));
 }catch(error){assert.ok(error instanceof c.ChartError&&expected.error,`${index} ${mode}: ${error.stack}`);records.push({index,family,mode,error:error.code});}
 finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,520);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 520 positional palette configurations and twelve publication files.');
