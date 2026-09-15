'use strict';
// FIX-GG02 actual WASM authors, pinned R expectations and immutable capture/profile transitions.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..');
const c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const reference=Object.fromEntries(JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/stages.json'),'utf8')).cases.map(v=>[v.id,v.layers[0].columns]));
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const options=c.exportOptions(420,280).dpi(96).basis('current'),records=[];
const figure=data=>c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'));
const marks=frame=>frame.scene().items.filter(i=>i.primitive.Point&&i.layer!==null&&i.layer!==undefined).map(i=>i.primitive.Point);
const close=(a,b)=>assert(Math.abs(a-b)<=1e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);
function retain(name,plot){
  const request=output.request(plot,options),frame=request.prepare();
  fs.writeFileSync(path.join(out,name+'.plot.json'),plot.to_json());
  fs.writeFileSync(path.join(out,name+'.scene.json'),JSON.stringify(frame.scene(),null,2));
  for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,name+'.'+fmt),frame.export(fmt));
  return [request,frame];
}
for(const [name,axis] of [
 ['log_mean',c.yAxis().scale(c.scaleLog(10))],
 ['coordinate_log_mean',c.yAxis().coordinateScale(c.scaleLog(10))],
 ['scale_limit_mean',c.yAxis().scale(c.scaleLinear().domain(1,10))],
 ['coordinate_zoom_mean',c.yAxis().viewport(1,10)],
 ['squish_mean',c.yAxis().scale(c.scaleLinear().domain(1,10)).oob('Squish')],
 ['keep_mean',c.yAxis().scale(c.scaleLinear().domain(1,10)).oob('Keep')],
]){
 const data=c.Data.columns({x:new Float64Array([1,1,1]),y:new Float64Array([1,10,100])});
 const plot=figure(data).layer(c.points().stat(c.summary().x('y')).afterStat(c.statAes().x(1).y('Mean'))).yAxis(axis).build();
 const chart=plot.chart(),domain=chart.semantics().layers[0].domains.y;
 close(domain.minimum,reference[name].y[0]);close(domain.maximum,reference[name].y[0]);
 const [request,frame]=retain(name,plot);records.push({id:name,y:domain,point_count:marks(frame).length});
 for(const v of [frame,request,chart,plot,data,axis])v.free();
}
// Shared statistics and expression filters cross actual WASM component handles.
{
  const data=c.Data.columns({x:[1,1,1],y:[1,10,100]});
  for(const [axis,expected] of [[c.y_axis().scale(c.scale_log(10)),1],[c.y_axis().coordinate_scale(c.scale_log(10)),37]]) {
    const shared=c.transform('mean',c.summary().x('y'));
    const plot=figure(data).transform(shared).transform(c.transform('copy',c.identity_stat()).from_transform(shared))
      .layer(c.points().from_transform('copy').after_stat(c.stat_aes().x(1).y('Mean'))).y_axis(axis).build();
    const chart=plot.chart();assert.ok(Math.abs(chart.semantics().layers[0].domains.y.minimum-expected)<1e-12);
    chart.dispose();plot.dispose();
  }
  const plot=figure(data).layer(c.points().filter(c.filter(c.source_expr('y').mul(2)).maximum(20))).build();
  const chart=plot.chart();assert.equal(chart.semantics().layers[0].domains.y.maximum,10);
  chart.dispose();plot.dispose();data.dispose();
}
const histData=c.Data.columns({value:new Float64Array([1,2,5,20,50,200,500])});
{
 const plot=c.plot(histData).profile('Ggplot2_4_0_3').aes(c.aes().y('value')).layer(c.histogram().breaks([1,10,100,1000])).yAxis(c.yAxis().scale(c.scaleLog(10))).build();
 const chart=plot.chart(),rows=chart.semantics().layers[0].rows.Binned;assert.deepEqual(rows.map(r=>Number(r.count)),reference.horizontal_log_histogram.count);
 const [request,frame]=retain('horizontal_log_histogram',plot);assert.equal(frame.scene().items.filter(i=>i.layer!=null&&i.primitive.Rectangle).length,3);
 for(const v of [frame,request,chart,plot])v.free();
}
{
 const fraction=c.binExpr('Count').div(c.binExpr('Count').sum());
 const plot=c.plot(histData).profile('Ggplot2_4_0_3').aes(c.aes().x('value')).layer(c.histogram().breaks([1,10,100,1000]).afterBin(c.binAes().y(fraction))).build();
 const chart=plot.chart();close(chart.semantics().layers[0].domains.y.maximum,3/7);
 const [request,frame]=retain('after_stat_expression',plot);for(const v of [frame,request,chart,plot,histData,fraction])v.free();
}
{
 const data=c.Data.columns({category:c.categorical(['A','A','B'])});
 const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().y('category')).layer(c.bars().orientation('Horizontal')).yAxis(c.yAxis().scale(c.scaleBand())).build();
 const chart=plot.chart();assert.deepEqual(chart.semantics().layers[0].rows.Statistical.map(r=>Number(r.count)),reference.horizontal_count.count);
 const [request,frame]=retain('horizontal_count',plot);assert.equal(frame.scene().items.filter(i=>i.layer!=null&&i.primitive.Rectangle).length,2);
 for(const v of [frame,request,chart,plot,data])v.free();
}
const data=c.Data.columns({x:new Float64Array([1,1,1]),y:new Float64Array([1,10,100])});
let plot=figure(data).aes(c.aes().x(c.sourceExpr(data.field('x')).add(1)).y(c.sourceExpr('y').mul(2))).layer(c.points()).build();
let chart=plot.chart();let d=chart.semantics().layers[0].domains;
assert.deepEqual(d.x,{minimum:2,maximum:2});assert.deepEqual(d.y,{minimum:2,maximum:200});
let [request,frame]=retain('source_expression',plot);for(const v of [frame,request,chart,plot])v.free();
plot=figure(data).layer(c.points().afterScale(c.scaleAes().size(c.afterScaleExpr('Size').mul(2)))).build();
[request,frame]=retain('after_scale_expression',plot);
// R gg_par fontsize combines size (.pt) and stroke (.stroke/2); circle radius is 3/8 fontsize.
assert.equal(marks(frame).length,reference.after_scale_expression.size.length);
marks(frame).forEach((p,i)=>assert.ok(Math.abs(p.radius-(reference.after_scale_expression.size[i]*72.27+reference.after_scale_expression.stroke[i]*48)/25.4*.375)<1e-12));
for(const v of [frame,request,plot])v.free();
plot=figure(data).theme(c.theme().geometry({accent:'#1256ab'})).layer(c.points().afterScale(c.scaleAes().color(c.fromTheme('Accent')))).build();
[request,frame]=retain('theme_expression',plot);for(const p of marks(frame))assert.deepEqual(p.fill,{red:18,green:86,blue:171,alpha:255});
for(const v of [frame,request,plot])v.free();
plot=figure(data).layer(c.points().size(2).color('#1256ab').stat(c.summary().x('y')).afterStat(c.statAes().x(1).y(c.statExpr('Mean').mul(2)))).yAxis(c.yAxis().scale(c.scaleLog(10).domain(1,100))).build();
chart=plot.chart();d=chart.semantics().layers[0].domains.y;close(d.minimum,reference.log_after_stat_expression.y[0]);
const [staticRequest,staticFrame]=retain('log_after_stat_expression',plot);
const presented=chart.present(output,options),oldPoints=marks(presented),currentRequest=chart.request(output,options);
const legacy=plot.edit().profile('LibraryV1').build();assert(chart.applyPlot(legacy,0n));
const presentedOptions=options.basis('presented'),presentedRequest=chart.request(output,presentedOptions),presentedFrame=presentedRequest.prepare();
const newRequest=chart.request(output,options),newFrame=newRequest.prepare();
assert.deepEqual(marks(presentedFrame),oldPoints);assert.notDeepEqual(marks(newFrame),oldPoints);
const oldFrame=currentRequest.prepare();assert.deepEqual(marks(oldFrame),oldPoints);const oldBytes=oldFrame.export('svg');
for(const v of [chart,plot,legacy,data,output])v.dispose();
assert.deepEqual(oldFrame.export('svg'),oldBytes);assert.deepEqual(marks(staticFrame),oldPoints);
fs.writeFileSync(path.join(out,'captures.json'),JSON.stringify({old:oldPoints,new:marks(newFrame),retained_after_disposal:true},null,2));
for(const v of [staticRequest,staticFrame,presented,currentRequest,presentedOptions,presentedRequest,presentedFrame,newRequest,newFrame,oldFrame,options,chart,plot,legacy,data,output])v.free();
assert.throws(()=>c.aes().x(c.statExpr('Mean')),e=>e instanceof c.ChartError);
fs.writeFileSync(path.join(out,'results.json'),JSON.stringify(records,null,2));
console.log('PASS FIX-GG02 WASM: 13 independently authored stage figures, R numeric/style expectations, stage rejection, static/Presented/Current capture and disposal.');
