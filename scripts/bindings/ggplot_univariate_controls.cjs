// FIX-GG09 independent WASM precomputed distribution geometry authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<8;mode++){
 const data=columns({x:[1,2,2,4,5,6],y:[1,3,3,2,5,4],w:[1,2,0,1,3,1]});let layer;
 if(mode===0)layer=c.density().stat(c.density_stat().input('x').weight('w'));
 else if(mode===1)layer=c.ecdf().stat(c.ecdf_stat().input('x').weight('w'));
 else if(mode===2)layer=c.qq().stat(c.qq_stat().input('y'));
 else if(mode===3)layer=c.qq_line().stat(c.qq_line_stat().input('y'));
 else if(mode===4||mode===5)layer=c.line().stat(c.univariate_stat({Function:{function:{Expression:{nodes:[{Read:'Value'},{Read:'Value'},{Binary:{op:'Multiply',left:0,right:1}}],output:2}},n:21,range:null}}).input('x'));
 else if(mode===6)layer=c.points().stat(c.unique_stat().x('x').y('y'));
 else layer=c.line().stat(c.connect_stat({Matrix:[[0,0],[.25,.75],[1,1]]}).x('x').y('y'));
 let builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer);if(mode===5)builder=builder.x_axis(c.x_axis().scale(c.scale_log(10)));
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`univariate-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`univariate-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`univariate-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM distribution geometry: eight authors, 24 publications.');
