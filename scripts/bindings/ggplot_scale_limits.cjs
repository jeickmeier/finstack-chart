'use strict';
// FIX-GG04 actual WASM exceptional limits, hidden bins and fractional counts.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(640,360).dpi(96).basis('current'),records=[];
const wireNumber=value=>typeof value==='string'?{number:value}:value;
function descriptor(testcase,hidden){
 const limits=testcase.limits===null?null:testcase.limits.map(wireNumber);
 const family=({sqrt:{Pow:{exponent:.5}},log10:{Log:{base:10}}})[testcase.transform]??'Linear';
 const kind=testcase.kind??(testcase.nice?'binned_nice':'binned_equal');
 const cuts=testcase.mode==='explicit'?{Explicit:testcase.cuts.map(wireNumber)}:{[kind==='binned_equal'?'Equal':'Nice']:testcase.count??5};
 const policy=kind==='continuous'?{Continuous:{limits,oob:'Censor'}}:{Binned:{limits,oob:'Squish',breaks:cuts,right:true}};
 return {training:'Eligible',guide:hidden?'Hidden':{Binned:'Automatic'},ggplot:policy,function:{Interpolated:{normalization:{Ggplot:{family,domain:[1,10],reverse:testcase.transform==='reverse',rescaler:'Range'}},output:{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:[{red:19,green:43,blue:67,alpha:255},{red:86,green:177,blue:247,alpha:255}],values:null}}}},unknown:{kind:'Missing'}}}};
}
function paint(value){
 if(value==='grey50')return {red:127,green:127,blue:127,alpha:255};
 assert.match(value,/^#[0-9A-Fa-f]{6}$/);return {red:parseInt(value.slice(1,3),16),green:parseInt(value.slice(3,5),16),blue:parseInt(value.slice(5,7),16),alpha:255};
}
for(const [kind,filename] of [['authored','authored-limit-populations.json'],['fractional','binned-fractional-counts.json']]){
 const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2',filename)));
 for(const [index,testcase] of fixture[kind==='authored'?'primary':'cases'].entries()){
  const hidden=kind==='authored',values=({finite:[1,10],empty:[],missing:[null,null],infinite:[Infinity,-Infinity]})[testcase.population]??testcase.domain;
  const expected=hidden||testcase.result.error?testcase.result:testcase.result.mapping,owned=[];
  try{
   const data=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:'float64'})});owned.push(data);
   const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1).color('v').colorScale('v')).scale(c.colorMapped('v',descriptor(testcase,hidden))).layer(c.points()).build();owned.push(plot);
   const wire=plot.toJson();assert.equal(JSON.parse(wire).version,17);
   const restored=c.Plot.fromJson(wire);owned.push(restored);assert.equal(restored.toJson(),wire);
   const chart=restored.chart();owned.push(chart);const semantic=chart.semantics();assert.ok(!expected.error,`${kind} ${index}: unexpected success`);
   const layer=semantic.layers[0],colors=(layer.styles??[]).map(s=>s.color);let wanted=expected.colors.map(paint);
   if(wanted.length===1)wanted=Array(values.length).fill(wanted[0]);assert.deepEqual(colors,wanted,`${kind} ${index}`);
   const record={kind,index,colors};
   if(hidden)assert.equal(layer.color_legend??null,null);
   else{
    const labels=testcase.result.labels,entries=layer.color_legend?.numeric_breaks??[],actual=entries.map(e=>e.label);
    assert.deepEqual(entries.map(e=>e.visible),testcase.result.visible,`${kind} ${index} visibility`);
    assert.deepEqual(actual,labels,`${kind} ${index} labels`);record.labels=actual;
   }
   records.push(record);let sample;
   if(hidden&&testcase.transform==='identity'&&testcase.population==='finite'&&testcase.mode==='auto'){
    if(testcase.kind==='binned_nice'&&testcase.limits===null)sample='hidden-bins';
    if(testcase.kind==='continuous'&&JSON.stringify(testcase.limits)==='[1,"Infinity"]')sample='infinite-limit';
   }
   if(!hidden&&testcase.transform==='identity'&&JSON.stringify(testcase.domain)==='[1,10]'&&testcase.limits===null&&testcase.count===2.5&&testcase.nice)sample='fractional-bins';
   if(sample){
    const request=output.request(restored,options);owned.push(request);const frame=request.prepare();owned.push(frame);
    fs.writeFileSync(path.join(out,sample+'.plot.json'),wire);fs.writeFileSync(path.join(out,sample+'.scene.json'),JSON.stringify(frame.scene(),null,2));
    for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,sample+'.'+fmt),frame.export(fmt));
   }
  }catch(error){
   assert.ok(error instanceof c.ChartError&&expected.error,`${kind} ${index}: ${error.stack}`);records.push({kind,index,error:error.code});
  }finally{for(const value of owned.reverse())value.dispose();}
 }
}
output.dispose();assert.equal(records.length,2304);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records,null,2));
console.log('PASS WASM GG04: 2016 authored-limit chart cases, 288 fractional-bin records, v17 and three SVG/PDF/PNG samples.');
