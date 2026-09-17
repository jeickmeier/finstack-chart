// FIX-GG13 independent WASM coordinate authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<14;mode++){
 let data,builder;
 if(mode<6){
 data=columns({x:[0,1,2,3,4],y:[1,2,3,2,1]});const radial={expand:false};
 if(mode===0)radial.mode='Polar';
 else if(mode===1)Object.assign(radial,{start:Math.PI/4,end:1.5*Math.PI,inner_radius:0.3});
 else if(mode===2)Object.assign(radial,{reverse:'Both',radial_axis:'Inside',inner_radius:0.3});
 else if(mode===4)radial.inner_radius=0.4;
 const coordinate=mode===5?{Cartesian:{ratio:1}}:{Radial:radial};
 let x=c.x_axis().scale(c.scale_linear().domain(0,4)).ticks(Array.from({length:5},(_,i)=>[{Number:i},String(i)]));
 if(mode===3)x=x.guide_components({labels:{rotation:0}});
 builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points()).x_axis(x).y_axis(c.y_axis().scale(c.scale_linear().domain(0,4))).coordinate(coordinate);
 if(mode===4)builder=builder.guide(c.axis_guide('theta-inner','x').side('Top')).guide(c.axis_guide('r-secondary','y').side('Right'));
 }else{
 const values=mode===10?{x:[1,3,1,2,3],y:[1,1,2,2,2]}:mode===13?{x:['A','B','C','D','E'],y:[1,2,3,2,1]}:{x:[0,1,2,3,4],y:[1,2,3,2,1],lo:[.5,1,2,1,.5],hi:[1.5,3,4,3,1.5]};
 data=mode===13?c.Data.columns({x:c.categorical(values.x),y:new Float64Array(values.y)}):columns(values);builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'));let coordinate;
 if(mode===6){builder=builder.layer(c.errorbar().recipe_value('Lower','lo').recipe_value('Upper','hi'));coordinate={Cartesian:{flip:true}};}
 else if(mode===7){builder=builder.layer(c.line()).layer(c.segment().aes(c.aes().x('x').y('y').x2(3.8).y2(3.5)).recipe({Segment:{arrow:{closed:true,length_mm:3,angle:30,ends:'Last'}}}));coordinate={Transformed:{y:'Sqrt'}};}
 else if(mode===8){builder=builder.layer(c.rectangle().recipe({Column:{}}));coordinate={Radial:{mode:'Polar',expand:false}};}
 else if(mode===9){builder=builder.layer(c.line()).layer(c.points().radius(5));coordinate={Radial:{inner_radius:.45,clip:'On',expand:false}};}
 else if(mode===10){builder=builder.layer(c.points().recipe({Raster:{interpolate:true,hjust:.5,vjust:.5}}));coordinate={Radial:{inner_radius:.5,clip:'On',expand:false}};}
 else if(mode===11){builder=builder.layer(c.points()).theme(c.theme().style(c.style().gradient({direction:'Horizontal',start:c.rgb(255,220,100),end:c.rgb(40,90,180)})));coordinate={Radial:{start:Math.PI/4,end:1.5*Math.PI,inner_radius:.35,clip:'On',expand:false}};}
 else if(mode===12){builder=builder.layer(c.rectangle().recipe({Column:{}}).stat(c.count().ggplot_count().x('x')));coordinate={Cartesian:{xlim:[{Number:1},{Number:3}],expand:[false,false,false,false]}};}
 else {builder=builder.layer(c.points());coordinate={Cartesian:{flip:true,reverse:'Y'}};}
 builder=builder.coordinate(coordinate);
 }
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`coordinate-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`coordinate-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`coordinate-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM coordinate controls: 14 authors, 42 publications.');
