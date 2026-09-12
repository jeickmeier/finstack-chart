'use strict';
// AXIS-05: independently authored primary styles, retained roles and publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const formatter=mode=>({Registered:{operation:{id:'example.guide_format',version:1},parameters:{mode}}});
const styles={domain:{color:'#173f66',width:2,dashes:[8,3]},ticks:{color:'#77889999',width:1.5},labels:{color:'#174c75',font_size:12},per_tick:[{index:2,label:{color:'#cc4400',font_size:18}},{index:3,line:{visible:false}},{index:4,line:{color:'#cc4400',dashes:[2,3]}}]};
const data=c.Data.columns({x:new Float64Array([0,.25,.5,.75,1]),y:new Float64Array([1,2,3,2,1])},{keys:Array.from({length:5},(_,i)=>9007199254741001n+BigInt(i)),name:'ticks'});
const p=c.plot(data).withRegistry(registry).aes(c.aes().x('x').y('y')).layer(c.points())
 .xAxis(c.xAxis().scale(c.scaleLinear().domain(0,1)).range(500,100).guideComponents(styles).guideProfile('D3_3_0_0')
  .guideGeometry({inner:-180,outer:12,padding:8,clip_ticks:true})
  .tickValues([0,.25,.5,.5,.75,1]).tickFormat({Labels:['zero','','mid','','','one']}))
 .yAxis(c.yAxis().range(280,100).visible(false))
 .guide(c.axisGuide('top','x').side('Top').guideComponents({domain:{visible:false},ticks:{color:'#228844'},labels:{color:'#225533',font_size:11}})
  .tickSizeInner(-18).tickSizeOuter(-12).tickPadding(-8).tickOffset(0).guideProfile('D3_3_0_0').tickValues([0,.5,1]).tickFormat(formatter('Indexed')))
 .guide(c.axisGuide('lower','x').side('Bottom').translate(15,40).guideComponents({domain:{color:'#228844',width:2},labels:{color:'#225533',font_size:12},per_tick:[{index:2,label:{visible:false}}]})
  .tickSize(0).tickPadding(3).guideProfile('D3_3_0_0').tickValues([0,.5,1]).tickFormat(formatter('Same')))
 .title(c.title('Independent domain, tick and label styles')).build();
styles.domain.width=99;
const wire=p.toJson();assert.equal(JSON.parse(wire).version,14);const loaded=c.Plot.fromJson(wire,registry);assert.equal(loaded.toJson(),wire);fs.writeFileSync(path.join(out,'components.plot.json'),wire);registry.free();p.free();
for(const [name,text] of [['text','preserve'],['outline','outline']])for(const dpi of [300,600]){
 const request=output.request(loaded,c.exportOptions(600,400).dpi(dpi).text(text).basis('current')),frame=request.prepare(),scene=frame.scene();assert.equal(scene.version,14);
 const components=scene.items.filter(i=>'guide'in i);assert.equal(components.filter(i=>i.guide.role==='Domain').length,2);assert.equal(components.filter(i=>i.guide.role==='Line').length,11);assert.equal(components.filter(i=>i.guide.role==='Label').length,8);
 scene.items.forEach((item,i)=>{if('guide'in item)assert.equal(scene.targets[i].length,0)});
 for(const item of components)if(item.guide.role==='Domain'&&item.primitive.DashedPath)assert.equal(item.primitive.DashedPath.stroke.width,2);
 const prefix=`${name}-${dpi}`;fs.writeFileSync(path.join(out,`${prefix}.scene.json`),JSON.stringify(scene));fs.writeFileSync(path.join(out,`${prefix}.guides.json`),JSON.stringify(frame.guides()));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${prefix}.${fmt}`),frame.export(fmt));frame.free();request.free();
}
const typo={labels:{font_size:12,typography:{text:'ignored',size:1.25}},per_tick:[{index:1,label:{font_size:18,rotation:30,typography:{text:'ignored',size:1,weight:400,tabular:true}}}]};
const td=c.Data.columns({x:new Float64Array([0,1]),y:new Float64Array([0,1])}),tp=c.plot(td).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().guideProfile('D3_3_0_0').tickValues([0,.5,1]).guideComponents(typo)).yAxis(c.yAxis().visible(false)).build(),tr=output.request(tp,c.exportOptions(400,300).dpi(72).basis('current')),tf=tr.prepare(),ts=tf.scene();
const glyphs=ts.items.filter(i=>i.guide?.role==='Label').map(i=>i.primitive.GlyphRun);assert.deepEqual(glyphs.map(g=>g.run.font_size),[15,18,15]);assert.deepEqual(glyphs.map(g=>g.rotation),[0,30,0]);
fs.writeFileSync(path.join(out,'typography.plot.json'),tp.toJson());fs.writeFileSync(path.join(out,'typography.scene.json'),JSON.stringify(ts));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`typography.${fmt}`),tf.export(fmt));tf.free();tr.free();tp.free();td.free();
for(const bad of [{ticks:{width:0}},{labels:{font_size:-1}},{ticks:{dashes:[1,2,3]}},{per_tick:[{index:1},{index:1}]}])assert.throws(()=>c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().guideComponents(bad)).build());
const reset=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().guideComponents({labels:{font_size:18}}).guideComponents(null)).build();assert.ok(JSON.parse(reset.toJson()).version<14);reset.free();loaded.free();data.free();output.free();console.log('PASS WASM AX04: v14 styles/roles, owned input, invalid/reset controls, text/outline SVG/PDF/PNG at 300/600 DPI.');
