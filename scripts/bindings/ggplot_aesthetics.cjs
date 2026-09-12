'use strict';
// FIX-GG03: independently authored actual WASM styles and immutable publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/aesthetics.json'))),records=[];
for(const q of reference.symbols){
 const symbol=new c.ShapeSymbol({kind:{Ggplot:q.pch},size:Math.PI*(.375*q.device_size)**2}),copy=symbol.copy();symbol.free();
 const p=copy.generate(),saved=p.copy(),before=saved.result(),again=copy.generate();assert.deepEqual(again.result(),before);
 p.move_to(99,99);p.free();copy.free();assert.deepEqual(saved.result(),before);records.push({pch:q.pch,device_size:q.device_size,geometry:before.geometry});saved.free();again.free();
}
for(const code of [26,31,255])assert.throws(()=>new c.ShapeSymbol({kind:{Ggplot:code}}).generate(),e=>e instanceof c.ChartError);
fs.writeFileSync(path.join(out,'symbols.json'),JSON.stringify(records));
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const data=c.Data.columns({x:new Float64Array([1,3]),y:new Float64Array([2,2]),inside:['A','B'],outside:['B','A'],area:new Float64Array([1,4]),alpha:new Float64Array([.5,1])},{keys:[9007199254741001n,9007199254741003n]});
const builder=()=>c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').fill('inside').fill_scale('inside').stroke('outside').stroke_scale('outside')).scale(c.color_discrete('inside').domain(['A','B']).palette(['#ff0000','#0000ff'])).scale(c.color_discrete('outside').domain(['A','B']).palette(['#000000','#008000']));
const p=builder().layer(c.points().name('points').radius(5).linewidth(2).shape_value('Alpha',data.field('alpha'))).build(),wire=p.to_json();assert.equal(JSON.parse(wire).version,16);
const loaded=c.Plot.from_json(wire);assert.equal(loaded.to_json(),wire);const chart=loaded.chart(),semantic=chart.semantics(),styles=semantic.layers[0].styles;
assert.deepEqual(styles.map(s=>s.fill),[{red:255,green:0,blue:0,alpha:128},{red:0,green:0,blue:255,alpha:255}]);assert.deepEqual(styles.map(s=>s.stroke.green),[128,0]);assert.equal(Object.keys(semantic.layers[0].paint_legends).length,2);assert(styles.every(s=>s.stroke_width===2&&s.radius===5));
fs.writeFileSync(path.join(out,'independent.semantics.json'),JSON.stringify(semantic));
const edited=p.edit().layer('points',c.points().fill('#123456')).build(),ec=edited.chart();assert.equal(Object.keys(ec.semantics().layers[0].paint_legends).length,1);assert.equal(p.to_json(),wire);ec.free();edited.free();
const valueScale=new c.StandaloneScale('linear'),valuePlot=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points().value_scale('TextSize',c.source_expr(data.field('area')).sum(),valueScale).aesthetic_value('FontFace',{kind:'Text',value:'bold'})).build(),valueChart=valuePlot.chart();
assert.deepEqual(valueChart.semantics().layers[0].aesthetics,Array.from({length:2},()=>({FontFace:{kind:'Text',value:'bold'},TextSize:{kind:'Number',value:5}})));valueChart.free();valuePlot.free();valueScale.free();
for(const layer of [c.points().alpha(1.1),c.points().line_type({Custom:0}),c.points().aesthetic_value('FontFace',{kind:'Text',value:'unknown'})]){assert.throws(()=>c.plot(data).aes(c.aes().x('x').y('y')).layer(layer).build(),e=>e instanceof c.ChartError);layer.free();}
for(const [name,layer] of [['area',c.points().shape_value('AreaSize',data.field('area')).stroke('#000000').linewidth(2)],['radius',c.points().aes(c.aes().size('area')).stroke('#000000')]]){
 const q=c.plot(data).aes(c.aes().x('x').y('y')).layer(layer).build(),qc=q.chart(),s=qc.semantics().layers[0].styles;
 assert(Math.abs((s[1].radius/s[0].radius)**2-(name==='area'?4:16))<1e-12);qc.free();q.free();layer.free();
}
let gallery=c.plot(data).aes(c.aes().x('x').y('y')).x_axis(c.x_axis().scale(c.scale_linear().domain(-.5,6.5))).y_axis(c.y_axis().scale(c.scale_linear().domain(-.5,3.5)));
for(let code=0;code<26;code++){
 const d=c.Data.columns({x:new Float64Array([code%7]),y:new Float64Array([3-Math.floor(code/7)])},{keys:[9007199254741001n+BigInt(code)],name:`glyph-${code}`});
 const layer=c.shape_symbol().data(d).symbol_kind({Ggplot:code}).symbol_size(100).fill('#ff0000').stroke('#0000ff').linewidth(1);gallery=gallery.layer(layer);d.free();layer.free();
}
const g=gallery.title(c.title('R point glyphs 0–25: independent red fill and blue outline')).build();
for(const [name,plot] of [['independent',loaded],['glyphs',g]]){
 const request=output.request(plot,c.export_options(640,400).dpi(144)),frame=request.prepare(),scene=frame.scene();fs.writeFileSync(path.join(out,`${name}.scene.json`),JSON.stringify(scene));fs.writeFileSync(path.join(out,`${name}.plot.json`),plot.to_json());
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${name}.${fmt}`),frame.export(fmt));
 if(name==='glyphs'){
  const marks=scene.items.filter(i=>i.layer!=null&&i.primitive.ShapePath).map(i=>i.primitive.ShapePath);assert.equal(marks.length,26);
  marks.forEach((m,code)=>{assert.equal(m.fill!=null,code>=15);assert.equal(m.stroke!=null,code<=14||code>=19);assert.equal(m.anchors.length,1);});
 }
 frame.free();request.free();
}
chart.free();p.free();loaded.free();g.free();data.free();output.free();
console.log('PASS WASM GG-03: 104 glyph ownership cases, independent paints/alpha/metadata, area/radius ratios, v16/edit capture and 26-glyph SVG/PDF/PNG gallery.');
