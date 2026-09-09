// CLR-05 independent WASM public color ramp authors and retained publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),modulePath=path.resolve(process.argv[2]),out=path.resolve(process.argv[3]);
const c=require(path.join(modulePath,'authoring.cjs'));fs.mkdirSync(out,{recursive:true});
const output=new c.Output(new Uint8Array(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf'))));
const options=c.export_options(720,420).dpi(144);
for(const [name,bg,fg,preset] of [['light','#ffffff','#202020','Editorial'],['dark','#18212a','#f0f0f0','Editorial'],['grayscale','#ffffff','#202020','Grayscale']]){
 let p=c.plot(c.Data.columns({x:Array.from({length:21},(_,i)=>i/20)}))
  .title(c.title(`Perceptual color / ${name}`).style(c.text_style().color(fg)))
  .x_axis(c.x_axis().scale(c.scale_linear().domain(-.05,1.05)))
  .y_axis(c.y_axis().scale(c.scale_linear().domain(.4,3.6)).visible(false))
  .theme(c.theme().preset(preset).style(c.style().background(bg).panel(bg).foreground(fg)));
 for(const [label,factory,y] of [['Lab','Lab',3],['HCL','Hcl',2],['Cubehelix','Cubehelix',1]]){
  const scale=new c.StandaloneScale('linear',{range:['rgba(255, 30, 30, 0.5)','rgba(30, 90, 255, 0.5)'],factory:{kind:factory}});
  p=p.layer(c.points().aes(c.aes().x('x').y(y).color('x').color_scale(label)).size(16))
   .layer(c.labels().id(label).at(.04,y+.32).text(label).style(c.text_style().color(fg)))
   .scale(c.color_mapped(label,scale)).legend(c.legend().scale(label).title(label));
 }
 p=p.legend(c.legend().scale('HCL').title('HCL / alpha 0.5')).build();
 const wire=p.to_json();assert.equal(JSON.parse(wire).version,5);fs.writeFileSync(path.join(out,name+'.plot.json'),wire);
 const request=output.request(p,options);p.dispose();const frame=request.prepare(),scene=frame.scene();
 const marks=scene.items.filter(i=>'Point'in i.primitive&&i.layer!==null&&i.layer!==undefined).map(i=>i.primitive.Point.fill);
 assert.equal(marks.length,63);assert(marks.every(m=>m.alpha===128));if(name==='grayscale')assert(marks.every(m=>m.red===m.green&&m.green===m.blue));
 fs.writeFileSync(path.join(out,name+'.scene.json'),JSON.stringify(scene,null,2));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,name+'.'+fmt),frame.export(fmt));frame.dispose();request.dispose();
}
console.log('PASS CLR-05 WASM: three independently authored perceptual/alpha/grayscale figures and retained publication.');
