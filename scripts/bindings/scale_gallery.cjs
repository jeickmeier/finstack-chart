'use strict';
// SP-07 independently authored single-threaded WASM scale figures.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const options=c.export_options(720,420).dpi(144),figures=[];
const x=new c.StandaloneScale('linear',{domain:[0,10,100],range:[0,50,100]}),y=new c.StandaloneScale('sqrt',{domain:[0,16]});
figures.push(['numeric',c.plot(c.Data.columns({x:[0,5,10,55,100],y:[1,4,9,16,9]},{name:'numeric'}))
 .aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points().size(4))
 .x_axis(c.x_axis().scale(c.scale_numeric(x))).y_axis(c.y_axis().scale(c.scale_numeric(y)))
 .axis(c.x_axis().name('secondary').side('Top').secondary('x',.1,0).numeric_format({specifier:'.1f'}))
 .title(c.title('Piecewise x, square-root y')).theme(c.theme().preset('Editorial')).build()]);
const ordinal=new c.StandaloneScale('ordinal',{range:['#28587b','#c77c35']});
figures.push(['categorical',c.plot(c.Data.columns({category:c.categorical(['A','B','C','A','B','C']),region:c.categorical(['North','North','North','South','South','South']),value:[2,5,8,20,35,50]},{name:'categories'}))
 .aes(c.aes().x('category').y('value').color('region').color_scale('regions')).layer(c.points().size(7))
 .x_axis(c.x_axis().scale(c.scale_band_d3({padding_inner:.3,padding_outer:.15,align:.2,round:true})))
 .scale(c.color_mapped('regions',ordinal,'Eligible')).legend(c.legend().scale('regions').title('Region'))
 .facet(c.facet_wrap('region').columns(2).free_y(true)).title(c.title('Bands with free y facets')).theme(c.theme().preset('Editorial')).build()]);
const quantile=new c.StandaloneScale('quantile',{range:['#abc9d8','#4385a5','#163c55']}),size=new c.StandaloneScale('threshold',{domain:[5,10],range:[3,6,9]});
figures.push(['classifier',c.plot(c.Data.columns({x:[0,1,2,3,4,5,6,7,8],value:[0,0,1,2,3,5,8,13,21]},{name:'distribution'}))
 .aes(c.aes().x('x').y('value').color('value').color_scale('quantiles')).layer(c.points().numeric_scale('Size','value',size))
 .scale(c.color_mapped('quantiles',quantile,'Eligible')).legend(c.legend().scale('quantiles').title('Sample terciles'))
 .title(c.title('Exact quantiles and threshold size')).theme(c.theme().preset('Editorial')).build()]);
const diverging=new c.StandaloneScale('diverging',{domain:[-10,0,100],range:['#bb4a43','#f8f5ee','#28587b'],clamp:true}),opacity=new c.StandaloneScale('linear',{domain:[-10,100],range:[.4,1],clamp:true});
figures.push(['diverging',c.plot(c.Data.columns({x:[-10,-5,0,25,50,75,100],y:[1,2,3,2,1,2,3]},{name:'diverging'}))
 .aes(c.aes().x('x').y('y').color('x').color_scale('asymmetric')).layer(c.points().size(9).numeric_scale('Opacity','x',opacity))
 .scale(c.color_mapped('asymmetric',diverging)).legend(c.legend().scale('asymmetric').title('Center = 0'))
 .title(c.title('An asymmetric diverging guide')).theme(c.theme().preset('Editorial')).build()]);
const zone={Local:{version:1,zone:'America/New_York',revision:'42',tzdata:'2025c',coverage:{start:'1672531200000',end:'1735689600000'},initial_offset_seconds:-18000,transitions:[
 {at_millis:'1678604400000',offset_seconds:-14400},{at_millis:'1699164000000',offset_seconds:-18000},{at_millis:'1710054000000',offset_seconds:-14400},{at_millis:'1730613600000',offset_seconds:-18000}]}};
const start=1730606400000n,time=new c.StandaloneScale('local',{domain:[start,start+5n*3600000n],zone});
figures.push(['local-time',c.plot(c.Data.columns({time:c.timestamps(Array.from({length:6},(_,i)=>start+BigInt(i)*3600000n),'ms','UTC'),value:[1,2,1.5,3,2,4]},{name:'local-time'}))
 .aes(c.aes().x('time').y('value')).layer(c.line()).layer(c.points().size(3))
 .x_axis(c.x_axis().scale(c.scale_calendar(time).calendar_interval({unit:'Hour',step:1})).time_format({pattern:'%H:%M %Z'}))
 .y_axis(c.y_axis().visible(false)).title(c.title('One elapsed hour through the fold')).theme(c.theme().preset('Editorial')).build()]);
for(const [name,plot] of figures){
 const wire=plot.to_json();assert.equal(JSON.parse(wire).version,5);const copy=c.Plot.from_json(wire);assert.equal(copy.to_json(),wire);copy.dispose();
 fs.writeFileSync(path.join(out,`${name}.plot.json`),wire);const request=output.request(plot,options);plot.dispose();const frame=request.prepare();
 fs.writeFileSync(path.join(out,`${name}.scene.json`),JSON.stringify(frame.scene(),null,2));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${name}.${fmt}`),frame.export(fmt));frame.dispose();request.dispose();
}
const owned=c.Data.columns({x:[0,1],y:[1,2]});
for(const source of [owned.field('y'),c.source_expr(owned.field('y'))]){
 const p=c.plot(owned).aes(c.aes().x('x').y('y')).layer(c.points().numeric_scale('Size',source,new c.StandaloneScale('linear',{domain:[1,2],range:[3,6]}))).build();
 const frame=output.request(p,options).prepare();
 assert.deepEqual(frame.scene().items.filter(i=>'Point'in i.primitive&&i.layer!==null&&i.layer!==undefined).map(i=>i.primitive.Point.radius),[3,6]);frame.dispose();p.dispose();
}
console.log('PASS SP-07 WASM: five independently authored v5 scale figures, typed source routes and retained SVG/PDF/PNG.');
