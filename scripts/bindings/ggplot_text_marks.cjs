// FIX-GG08: actual WASM source/stat labels and portable raster publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const color=(red,green,blue,alpha=255)=>({red,green,blue,alpha});
for(let mode=0;mode<7;mode++){
 let data=c.Data.columns({x:new Float64Array([1,2,3,4]),label:['Alpha','béta','','Delta']});
 let b=c.plot(data).aes(c.aes().x('x').y(1));
 const options={size:14,units:'Points',angle:mode===1?30:0,fill:mode===1?color(230,240,255):null,check_overlap:mode===2};
 if(mode<3){const layer=c.points().text_geom(options).text_label('label');b=b.layer(mode===2?layer.aes(c.aes().x(1).y(1)):layer);}
 else if(mode===3){data=c.Data.columns({group:['A','A','B']});b=c.plot(data).layer(c.points().stat(c.count().group('group')).after_stat(c.stat_aes().x('Group').y('Count')).text_geom(options).text_stat_label('Count'));}
 else {data=c.Data.columns({x:new Float64Array([1])});const content=mode===6?{Vector:{geometry:{commands:[{MoveTo:[-80,-60]},{LineTo:[80,-60]},{LineTo:[0,60]},'Close']},fill:color(20,90,120),stroke:null}}:{Raster:{raster:{width:2,height:2,pixels:[color(255,0,0),color(0,255,0),color(0,0,255),color(255,255,0,100)]},bounds:[-100,-60,200,120],interpolate:mode===5}};b=c.plot(data).aes(c.aes().x('x').y(1)).layer(c.points().annotation({units:'Points',content}));}
 const x=mode<3?c.x_axis().scale(c.scale_linear().domain(0,5)):c.x_axis();
 const p=b.x_axis(x.visible(false)).y_axis(c.y_axis().scale(c.scale_linear().domain(0,3)).visible(false)).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,73);
 const restored=c.Plot.fromJson(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const labels=scene.items.filter(i=>i.layer&&i.primitive.GlyphRun).map(i=>i.primitive.GlyphRun.run.text);
 if(mode<4)assert.deepEqual(labels,[['Alpha','béta','Delta'],['Alpha','béta','Delta'],['Alpha'],['2','1']][mode]);
 if(mode===4||mode===5){const images=scene.items.filter(i=>i.primitive.RasterImage);assert.equal(images.length,1);assert.equal(images[0].primitive.RasterImage.interpolate,mode===5);}
 fs.writeFileSync(path.join(out,`text-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`text-${mode}.scene.json`),JSON.stringify(scene));
 for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`text-${mode}.${format}`),frame.export(format));
 if(mode<4){const outlinedRequest=output.request(restored,c.export_options(600,360).dpi(144).text("outline")),outlined=outlinedRequest.prepare();for(const fmt of ["svg","pdf"])fs.writeFileSync(path.join(out,`text-${mode}.outline.${fmt}`),outlined.export(fmt));outlined.free();outlinedRequest.free();}
 frame.free();request.free();restored.free();p.free();data.free();
}
output.free();console.log('PASS WASM text/raster: seven authors, 21 standard and 8 outline publications, source/stat labels and replay.');
