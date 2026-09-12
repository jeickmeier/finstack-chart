'use strict';
// AXIS-06 independent WASM authoring, explicit host fractions and retained ownership.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const axis=(domain,values,labels)=>c.xAxis().scale(c.scaleLinear().domain(0,domain)).range(50,450).guideProfile('D3_3_0_0').tickValues(values).tickFormat({Labels:labels});
const data=c.Data.columns({x:new Float64Array([0,.5,1]),y:new Float64Array([0,1,0])});
const a=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(axis(1,[0,.5,1],['zero','half','one'])).yAxis(c.yAxis().visible(false)).build();
const b=a.edit().xAxis(axis(2,[0,1,2],['ZERO','ONE','TWO'])).build(),cp=b.edit().xAxis(axis(4,[0,2,4],['zero again','two again','four'])).build(),plots=[a,b,cp];
plots.forEach((p,i)=>fs.writeFileSync(path.join(out,`plot-${i}.json`),p.toJson()));
for(const mode of ['text','outline']){
 const figures=plots.map(p=>output.request(p,c.exportOptions(900,300).text(mode==='text'?'preserve':'outline').dpi(144)).prepare());
 const plan=figures[1].guideTransition(figures[0]),mid=plan.sample(.5),interrupt=figures[2].guideTransition(mid);
 for(const [name,frame] of [['start',plan.sample(0)],['mid',mid],['end',plan.sample(1)],['interrupt-start',interrupt.sample(0)],['interrupt-mid',interrupt.sample(.5)],['interrupt-end',interrupt.sample(1)]]){
  const prefix=`${mode}-${name}`;
  for(const [ext,value] of [['scene',frame.scene()],['guides',frame.guides()],['presentation',frame.presentation()]])fs.writeFileSync(path.join(out,`${prefix}.${ext}.json`),JSON.stringify(value));
  for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${prefix}.${fmt}`),frame.export(fmt));
  frame.free();
 }
 assert.deepEqual(plan.sample(1).scene(),figures[1].scene());figures.forEach(f=>f.free());
 assert.equal(plan.sample(.5).presentation()[0].frame.ticks.length,4);
 for(const fraction of [NaN,Infinity,-.1,1.1])assert.throws(()=>plan.sample(fraction));
 plan.dispose();assert.throws(()=>plan.sample(.5));plan.free();interrupt.free();
}
const chart=a.chart(),opts=c.exportOptions(900,300).dpi(144),initial=chart.present(output,opts);
chart.applyPlot(b,chart.revisions().definition);
const target=chart.present(output,opts),plan=target.guideTransition(initial),mid=plan.sample(.5);
chart.acknowledgeFrame(mid);const frozen=chart.request(output,opts.basis('displayed')).prepare();
assert.deepEqual(frozen.scene(),mid.scene());assert.deepEqual(frozen.guides(),mid.guides());assert.throws(()=>chart.acknowledgeFrame(initial));
chart.dispose();assert.throws(()=>chart.acknowledgeFrame(mid));output.free();assert.equal(plan.sample(.5).presentation()[0].frame.ticks.length,4);
console.log('PASS WASM AX05: sampled publications, interruption, final/reduced motion, capture, stale/disposed handles and retained resources');
