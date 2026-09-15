'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<10;mode++) {
 const data=c.Data.columns({x:new Float64Array([1,4,10,100]),g:['A','B','C','D'],f:['one','one','two','two']});
 let b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1));
 if(mode<6) {
  const position=['Right','Left','Top','Bottom',{Inside:{x:.5,y:.5}},{Inside:{x:.5,y:.5}}][mode],options={position,ncol:2,reverse:mode%2===1,override_aes:{size:4}};
  b=b.aes(c.aes().x('x').y(1).color('g').shape('g')).layer(c.points().legend({})).legend(c.legend().aesthetic('Color').options(options)).legend(c.legend().aesthetic('Shape').options(options));
  if(mode===5)b=b.facet(c.facetWrap('f').collectGuides(false));
 } else if(mode>=8) {
  const scale={training:'Eligible',function:{Interpolated:{normalization:{Ggplot:{family:'Linear',domain:[0,1],reverse:false,rescaler:'Range'}},output:{Interpolate:{operation:'PowerRange',range:[1,6],exponent:.5,absolute:false}},unknown:{kind:'Missing'}}},ggplot:{Binned:{oob:'Squish',right:true,limits:[0,20],breaks:{Explicit:[5,10]}}},guide:{BinnedBins:'Automatic'},colorbar_options:{show_limits:mode===8}};
  b=b.layer(c.points().numericScale('Size','x',scale)).legend(c.legend().aesthetic('Size').options({position:mode===8?'Right':'Bottom',reverse:mode===9}));
 } else if(mode===7) b=b.layer(c.points()).legend(c.legend().custom({id:'999',bounds:[0,0,40,20],paths:[{geometry:{commands:[{MoveTo:[0,0]},{LineTo:[40,0]},{LineTo:[20,20]},'Close']},fill:'#123456',stroke:null}],options:{title:'Custom vector',position:'Left'}}));
 else b=b.layer(c.points()).xAxis(c.xAxis().scale(c.scaleLog(10).domain(1,100)).label('Primary logarithmic axis').ggplotAxis({n_dodge:2,check_overlap:true,cap:'Both',stack_order:0,stack_spacing:6})).guide(c.axisGuide('outer','x').side('Bottom').label('Stacked log ticks').ggplotAxis({stack_order:1,logticks:{expanded:false}}));
 const p=b.yAxis(c.yAxis().visible(false)).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,mode===7?71:mode===6?70:69);
 const restored=c.Plot.fromJson(wire);p.free();data.free();const request=output.request(restored,c.exportOptions(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 if(mode<6)assert.equal(scene.items.filter(i=>i.guide?.role==='LegendKey').length,4);else if(mode>=8){assert.equal(scene.items.filter(i=>i.guide?.role==='LegendKey').length,3);assert.equal(scene.items.filter(i=>i.guide?.role==='LegendTick').length,mode===8?4:2);}else if(mode===7)assert.equal(scene.items.filter(i=>JSON.stringify(i.guide?.scope)==='["legend","custom"]').length,2);else assert.equal(scene.items.filter(i=>JSON.stringify(i.guide?.scope)==='["logtick"]').length,19);
 fs.writeFileSync(path.join(out,`guide-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`guide-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`guide-${mode}.${fmt}`),frame.export(fmt));frame.free();request.free();restored.free();
}
output.free();console.log('PASS WASM guide composition: ten authors, 30 publications, owned controls and roundtrip.');
