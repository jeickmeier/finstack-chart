'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let i=0;i<8;i++) {
 const d=c.Data.columns({x:new Float64Array([1,1,2]),y:new Float64Array([2,3,-1]),g:['a','b','a'],lo:new Float64Array([.6,.8,1.6]),hi:new Float64Array([1.4,1.2,2.4]),panel:['one','one','two']});
 const positions=[c.ggplot_stack().vjust(.5),c.ggplot_fill().reverse(true),c.ggplot_dodge().width(.8).preserve('Single'),c.dodge2().width(.8).padding(.2),c.nudge(.2,-.1),c.jitter_dodge(42).displacement(.2,.1),c.ggplot_stack().reverse(true),c.dodge2().width(.8).reverse(true).preserve('Single')];
 const interval=[1,3,6,7].includes(i),mapping=[1,6].includes(i)?c.aes().x(.6).x2(1.4).y('y').y2(0).group('g'):interval?c.aes().x('lo').x2('hi').y('y').y2(0).group('g'):c.aes().x('x').y('y').group('g');
 let builder=c.plot(d).profile('Ggplot2_4_0_3').aes(mapping).layer((interval?c.rectangle():c.points()).position(positions[i]));if(i===7)builder=builder.facet(c.facetWrap('panel'));
 const p=builder.build(),wire=p.toJson();assert(JSON.parse(wire).version>=72);
 const q=c.Plot.fromJson(wire),request=output.request(q,c.exportOptions(480,320)),frame=request.prepare();fs.writeFileSync(path.join(out,`position-${i}.scene.json`),JSON.stringify(frame.scene()));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`position-${i}.${fmt}`),frame.export(fmt));frame.free();request.free();q.free();p.free();d.free();
}
output.free();console.log('PASS WASM GG06 positions: 8 authors, roundtrips, 24 publications.');
