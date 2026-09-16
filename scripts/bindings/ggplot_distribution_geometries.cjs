// FIX-GG09 independent WASM precomputed distribution geometry authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const red={red:255,green:0,blue:0,alpha:255},gold={red:255,green:215,blue:0,alpha:255};
for(let mode=0;mode<15;mode++){
 let data,mapping,layer;
 if(mode>=11){data=columns({x:[0,0,.2,1,1.2,2,2,2]});mapping=c.aes();layer=c.density().stat(c.density_stat().input('x')).recipe({Density:{outline:['Upper','Lower','Both','Full'][mode-11]}}).fill(gold).alpha(.2);}
 else if(mode>=8){data=columns({x:Array(8).fill(1),y:[0,1,1,2,3,4,5,20]});mapping=c.aes();layer=mode===8?c.boxplot().stat(c.boxplot_stat().input(data.field('y')).x('x')):mode===9?c.violin().stat(c.violin_stat().input(c.source_expr(data.field('y'))).x('x')):c.dotplot().stat(c.dotplot_stat().input('y'));}
 else if(mode<4){data=columns({x:[1,2],middle:[2.5,3],lo:[.75,2],hi:[3.25,4],min:[0,1],max:[4,5],nl:[1.1034641071565687,1.5868050382201329],nu:[3.8965358928434313,4.413194961779867],relative:[Math.sqrt(8),Math.sqrt(5)]},{keys:[10,20]});
 const spec={width:.75,notch:mode===1,notch_width:.3,staple_width:mode===0?0:.7,variable_width:mode===2,source_outliers:[{row:'10',values:[20]}]};
 if(mode===2)Object.assign(spec,{median:{color:red,linewidth:1},whisker:{line_type:'Dashed'},outlier:{shape:21,fill:gold,size:3}});
 layer=c.rule().recipe({Boxplot:spec});for(const[channel,field]of[['Lower','lo'],['Upper','hi'],['Middle','middle'],['WhiskerLower','min'],['WhiskerUpper','max'],['NotchLower','nl'],['NotchUpper','nu'],['RelativeWidth','relative']])layer=layer.recipe_value(channel,field);
 mapping=c.aes().x('x').y('middle').group('x');if(mode===3){mapping=c.aes().x('middle').y('x').group('x');layer=layer.orientation('Horizontal');}
 }else if(mode===4){data=columns({x:[1,1,1,1,1],y:[0,1,2,3,4],width:[.1,.7,1,.5,.1],q:[null,.25,.5,.75,null]},{keys:[1,2,3,4,5]});mapping=c.aes().x('x').y('y');layer=c.rule().recipe({Violin:{width:.8,quantile:{line_type:'Dashed',color:red}}}).recipe_value('ViolinWidth','width').recipe_value('QuantileFlag','q');
 }else{data=columns({bin:[0,1,2],count:[3,1,4]},{keys:[1,2,3]});mapping=mode===6?c.aes().x(1).y('bin'):c.aes().x('bin').y(0);layer=c.rule().recipe({Dotplot:{bin_axis:mode===6?'Y':'X',stack:mode===7?'CenterWhole':'Up',stack_ratio:.8,dot_size:.7}}).recipe_value('Count','count').recipe_value('BinWidth',.5);}
 const p=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(layer).build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`distribution-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`distribution-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`distribution-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM distribution geometry: fifteen authors, 45 publications.');
