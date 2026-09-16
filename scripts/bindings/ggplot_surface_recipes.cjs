// FIX-GG07: independently authored WASM surface recipes and replay.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<9;mode++){
 let data,b;
 if(mode<4){data=c.Data.columns({x:new Float64Array([0,4,4,0,1,3,3,1]),y:new Float64Array([0,0,4,4,...(mode%2?[3,3,1,1]:[1,1,3,3])]),sub:['outer','outer','outer','outer','inner','inner','inner','inner']});b=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points().recipe({Polygon:{rule:mode<2?'EvenOdd':'NonZero'}}).recipe_value('Subgroup','sub'));}
 else if(mode===8){data=c.Data.columns({g:['A','A','B']});b=c.plot(data).layer(c.points().stat(c.count().group('g')).after_stat(c.stat_aes().x('Group').y('Count')).recipe({Tile:{width:0.8,height:0.8}}));}
 else{data=c.Data.columns({x:new Float64Array([1,3,1,2,3]),y:new Float64Array([1,1,2,2,2]),v:new Float64Array([1,3,4,5,6])});const recipe=mode<6?{Tile:{width:mode===5?0.8:null,height:mode===5?0.6:null}}:{Raster:{hjust:0.5,vjust:0.5,interpolate:mode===7}};b=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points().recipe(recipe));}
 const p=b.x_axis(c.x_axis().visible(false)).y_axis(c.y_axis().visible(false)).build(),wire=p.to_json();assert.equal(JSON.parse(wire).version,74);
 const restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 if(mode===6||mode===7){const raster=scene.items.find(i=>i.primitive.RasterImage).primitive.RasterImage;assert.equal(raster.cells.length,5);assert.deepEqual([raster.raster.width,raster.raster.height],[3,2]);}
 fs.writeFileSync(path.join(out,`surface-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`surface-${mode}.scene.json`),JSON.stringify(scene));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`surface-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
const probe=c.Data.columns({x:new Float64Array([1,2]),label:new Float64Array([3,4])});
for(const [source,expected] of [['label',['3','4']],[probe.field('label'),['3','4']],[c.source_expr('label').add(1),['4','5']]]){
 const p=c.plot(probe).aes(c.aes().x('x').y(1)).layer(c.points().text_geom({size:12}).text_label(source)).build();
 const request=output.request(p,c.export_options(600,360)),frame=request.prepare();
 const labels=frame.scene().items.filter(i=>i.layer&&i.primitive.GlyphRun).map(i=>i.primitive.GlyphRun.run.text);assert.deepEqual(labels,expected);
 frame.dispose();request.dispose();p.dispose();
}
probe.dispose();output.dispose();console.log('PASS WASM surfaces: nine independent authors, 27 publications.');
