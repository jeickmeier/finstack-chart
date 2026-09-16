// FIX-GG07: independently authored WASM interval recipes and replay.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<17;mode++){
 const data=c.Data.columns({x:new Float64Array([1,2,3]),y:new Float64Array([2,-1,1]),lo:new Float64Array([1,-2,0]),hi:new Float64Array([3,0,2])});
 let b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'));
 if(mode<8){let layer=c.rule().recipe({Interval:{kind:['LineRange','PointRange','ErrorBar','Crossbar'][mode%4],width:0.5}}).recipe_value('Lower','lo').recipe_value('Upper','hi');if(mode>=4){b=b.aes(c.aes().x('y').y('x'));layer=layer.orientation('Horizontal');}b=b.layer(layer);}
 else if(mode<11)b=b.layer(c.step(['Hv','Vh','Mid'][mode-8]));
 else if(mode===11)b=b.layer(c.segment().aes(c.aes().x('x').y('y').x2(3.5).y2(3)).recipe({Segment:{arrow:{angle:30,length_mm:3,ends:'Both',closed:true}}}));
 else if(mode===15)b=b.layer(c.crossbar().recipe({Interval:{kind:'Crossbar',width:null,middle:{color:{red:255,green:0,blue:0,alpha:255},linewidth:2,line_type:'Dashed'},box_style:{color:{red:0,green:0,blue:255,alpha:255},linewidth:0.25,line_type:'Dotted'}}}).recipe_value('Lower','lo').recipe_value('Upper','hi').fill('#FFD700').alpha(0.2));
 else if(mode===16)b=b.layer(c.pointrange().recipe({Interval:{kind:'PointRange',width:null,fatten:2,point:{size:1,stroke:2,shape:21,fill:{red:255,green:215,blue:0,alpha:255}}}}).recipe_value('Lower','lo').recipe_value('Upper','hi').linewidth(0.25));
 else b=b.layer(c.points()).layer(mode===12?c.abline(0.5,0):mode===13?c.hline(0.5):c.vline(2));
 const p=b.x_axis(c.x_axis().visible(false)).y_axis(c.y_axis().visible(false)).build(),wire=p.to_json();assert.equal(JSON.parse(wire).version,74);
 const restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const directRequest=output.request(p,c.export_options(600,360).dpi(144)),directFrame=directRequest.prepare();assert.deepEqual(directFrame.scene(),scene);directFrame.dispose();directRequest.dispose();
 fs.writeFileSync(path.join(out,`interval-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`interval-${mode}.scene.json`),JSON.stringify(scene));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`interval-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM intervals: seventeen authors, 51 publications.');
