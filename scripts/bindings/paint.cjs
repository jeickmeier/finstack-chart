'use strict';
// CLR-04: independently authored WASM figures and every retained paint route.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const background=c.hsl(210,.1,.97),options=c.export_options(440,340).dpi(96).background(background).basis('current');
const ink=()=>c.lab(28,7,-18),heading=text=>c.title(text).style(c.text_style().color(ink()).size(1.15));
const data=()=>c.Data.columns({x:[1,2,3],y:[1,2,1.5]});
const figures=[];
let swatches=c.plot(c.Data.columns({one:[1]})).title(heading('Color spaces at the paint boundary'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(0,6)).visible(false))
 .y_axis(c.y_axis().scale(c.scale_linear().domain(0,2)).visible(false))
 .theme(c.theme().preset('Editorial').style(c.style().gradient({direction:'Horizontal',start:c.hsl(40,.4,.98),end:c.hsl(210,.3,.94)})));
for(const [i,[name,value]] of [['RGB',c.rgb(230.25,55.75,80.5)],['HSL',c.hsl(155,.75,.38)],['Lab',c.lab(65,45,50)],['HCL',c.hcl(270,70,60)],['Cubehelix',c.cubehelix(210,1.3,.55)]].entries()) {
 const x=i+1;
 swatches=swatches.layer(c.points().aes(c.aes().x(x).y(1.15)).color(value).size(19)).layer(c.labels().id(name).at(x,.65).text(name).style(c.text_style().color(value)));
 value.dispose();
}
figures.push(['spaces',swatches.build()]);
const ramp=c.Data.columns({x:Array.from({length:11},(_,i)=>i/10),y:Array(11).fill(1)});
figures.push(['continuous',c.plot(ramp).aes(c.aes().x('x').y('y'))
 .layer(c.points().aes(c.aes().color('x').color_scale('ramp')).size(12))
 .scale(c.color_continuous('ramp',0,1).palette([c.lab(65,70,65),c.hcl(240,70,65)]).missing(c.hsl(0,0,.5)))
 .legend(c.legend().scale('ramp').title('Floating RGB')).title(heading('One ramp for marks and guide stops'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(-.1,1.1)))
 .y_axis(c.y_axis().scale(c.scale_linear().domain(0,2)).visible(false)).theme(c.theme().preset('Editorial')).build()]);
const candles=c.Data.columns({x:[1,2,3,4],open:[2,4,3,5],close:[4,3,5,4],low:[1,2,2,3],high:[5,5,6,6]});
figures.push(['candles',c.plot(candles).aes(c.aes().x('x').y('open').y2('close').low('low').high('high'))
 .layer(c.ohlc().width(12).color(c.rgb(40.5,45.5,50.5)).candle_colors({up:c.hcl(150,55,55),down:c.hcl(25,65,55)}))
 .title(heading('Directional colors retain their space'))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(.5,4.5))).y_axis(c.y_axis().scale(c.scale_linear().domain(0,7))).theme(c.theme().preset('Editorial').geometry({ink:ink(),paper:c.gray(98),accent:c.hcl(270,50,55),point_size:1.5,line_width:.5})).build()]);
const rich=text=>c.rich_text(text).style(c.text_style().color(ink()));
const vector=c.path().move_to(0,0).line_to(30,0).line_to(15,25).close_path();
figures.push(['furniture',c.plot(data()).aes(c.aes().x('x').y('y'))
 .layer(c.points().size(8).style(c.style().mark(c.hsl(210,.7,.5))))
 .layer(c.vector_path('triangle',vector).fill(c.lab(65,50,45,.55)).stroke({color:c.hcl(20,55,45),width:2}).anchor({Output:{x:310,y:155}}))
 .layer(c.labels().id('label').at(1.4,1.5).text('Retained annotation').style(c.text_style().color(c.hcl(300,40,40))))
 .title(c.title('Rich text and authored paint').rich(rich('Rich text and authored paint')))
 .subtitle(c.subtitle('Shared color descriptors').style(c.text_style().color(c.hsl(210,.5,.35))))
 .caption(c.caption('Caption / source / footnote').style(c.text_style().color(c.lab(40,20,-20))))
 .source_note(c.source_note('Source: supplied values').style(c.text_style().color(c.gray(40))))
 .footnote(c.footnote('Final scene uses sRGB8').style(c.text_style().color(c.hcl(100,20,35))))
 .x_axis(c.x_axis().scale(c.scale_linear().domain(.5,3.5)).rich_label(rich('Authored x'))).y_axis(c.y_axis().scale(c.scale_linear().domain(.5,2.5)).rich_label(rich('Authored y')))
 .theme(c.theme().preset('Editorial').style(c.style().foreground(ink()).annotation(ink()).focus(c.hcl(90,60,70)).selection(c.hcl(300,40,60)).grid(c.gray(90)).panel(c.gray(99)))).build()]);
const records=[];
for(const [name,plot] of figures){
 const wire=plot.to_json();assert.equal(JSON.parse(wire).version,4);const copied=c.Plot.from_json(wire);assert.equal(copied.to_json(),wire);copied.dispose();
 const request=output.request(plot,options),frame=request.prepare(),scene=frame.scene();
 fs.writeFileSync(path.join(out,name+'.plot.json'),wire);fs.writeFileSync(path.join(out,name+'.scene.json'),JSON.stringify(scene,null,2));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,name+'.'+fmt),frame.export(fmt));
 plot.dispose();const repeated=request.prepare();assert.deepEqual(repeated.scene(),scene);repeated.dispose();
 records.push({id:name,version:4,retained_after_disposal:true});frame.dispose();request.dispose();
}
const value=c.hcl(-0,-0,50,.4),descriptor=JSON.parse(value.to_json());assert.deepEqual(descriptor.value.channels.h,{number:'-0'});
let style=c.style();for(const field of ['background','panel','foreground','grid','mark','annotation','focus','selection'])style=style[field](value);
const plot=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points().color('rebeccapurple')).theme(c.theme().style(style)).build();
assert.deepEqual(JSON.parse(plot.to_json()).definition.theme.plot.mark,descriptor);
for(const wide of [c.rgb(1e100,0,0),JSON.parse(c.rgb(1e100,0,0).to_json())]) {
 const p=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points().color(wide)).build();assert.equal(JSON.parse(p.to_json()).definition.layers[0].style.color.value.channels.r,1e100);p.dispose();
}
const layout=c.layout_options().host_style(c.style().background(value)).output_style(c.style().panel(value));
const request=output.request(plot,c.export_options(440,340).layout(layout).background(value)),frame=request.prepare();assert(frame.scene().items.length);
const d=c.Data.columns({x:[1,2,3],y:[1,2,3],group:c.categorical(['A','B','C'])});
const p=c.plot(d).aes(c.aes().x('x').y('y').color('group').color_scale('d')).layer(c.points()).scale(c.color_discrete('d').palette([c.lab(60,40,30),value]).domain(['A','B']).missing(c.rgb(1.2,2.3,3.4))).build();
const f=output.request(p,options).prepare();assert.equal(f.scene().items.filter(i=>i.primitive.Point&&i.layer!=null).length,3);
for(const bad of [{version:2,value:descriptor.value},{red:1.5,green:0,blue:0,alpha:255},'currentColor'])assert.throws(()=>c.points().color(bad));
value.dispose();assert.throws(()=>c.points().color(value));
const livePlot=c.plot(data()).aes(c.aes().x('x').y('y')).layer(c.points()).theme(c.theme().style(c.style().mark('red'))).build();
const live=livePlot.chart(),liveFrame=live.present(output,options),before=live.revisions(),old=live.request(output,options.basis('presented'));
const updated=livePlot.edit().theme(c.theme().style(c.style().mark(c.hsl(120,1,.5)))).build();
live.applyPlot(updated,BigInt(before.definition));const after=live.revisions();assert.equal(after.store,before.store);assert.equal(BigInt(after.definition),BigInt(before.definition)+1n);
const current=live.request(output,options.basis('current')),stillOld=live.request(output,options.basis('presented')),fresh=output.request(updated,options);
live.dispose();livePlot.dispose();updated.dispose();liveFrame.dispose();
function paints(request){const f=request.prepare();try{return f.scene().items.filter(i=>i.primitive.Point&&i.layer!=null).map(i=>i.primitive.Point.fill);}finally{f.dispose();}}
assert.deepEqual(paints(old),paints(stillOld));assert.deepEqual(paints(old),Array(3).fill({red:255,green:0,blue:0,alpha:255}));
assert.deepEqual(paints(current),paints(fresh));assert.deepEqual(paints(current),Array(3).fill({red:0,green:255,blue:0,alpha:255}));
records.push({id:'live-color-edit',retained_presented_and_current:true,data_revision_preserved:true});
fs.writeFileSync(path.join(out,'checks.json'),JSON.stringify(records,null,2)+'\n');
console.log('PASS CLR-04 WASM: four independently authored publication figures, every paint input, strict descriptors and retained ownership.');
