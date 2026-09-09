'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(720,420).dpi(144),figures=[];
for(const [name,id,size] of [['categorical','Category10',null],['brewer','Blues',5]]){
 const scale=new c.StandaloneScale('ordinal',{range:['black']});
 const p=c.plot(c.Data.columns({x:Array.from({length:10},(_,i)=>i),group:c.categorical(Array.from({length:10},(_,i)=>`C${i}`))}))
 .aes(c.aes().x('x').y(1).color('group').colorScale('catalog')).layer(c.points().size(13))
 .scale(c.colorMapped('catalog',scale,'Eligible').paletteScheme({id,size}))
 .legend(c.legend().scale('catalog').title(size?'Blues / exact k = 5':'Category10'))
 .yAxis(c.yAxis().visible(false)).title(c.title(`Named palette / ${name}`)).theme(c.theme().preset('Editorial')).build();figures.push([name,p]);scale.free();
}
for(const [name,id,domain,family] of [['lookup','Viridis',[0,1],'sequential'],['diverging','RdBu',[-10,0,100],'diverging'],['cyclic','Rainbow',[0,1],'sequential']]){
 const values=name==='diverging'?[-10,-8,-6,-4,-2,0,20,40,60,80,100]:Array.from({length:33},(_,i)=>i/32);
 const ramp=c.chromatic(id),scale=new c.StandaloneScale(family,{domain,interpolator:ramp});ramp.free();
 const p=c.plot(c.Data.columns({x:values})).aes(c.aes().x('x').y(1).color('x').colorScale('catalog')).layer(c.points().size(13))
 .scale(c.colorMapped('catalog',scale)).legend(c.legend().scale('catalog').title(id)).yAxis(c.yAxis().visible(false))
 .title(c.title(`Named ramp / ${name}`)).theme(c.theme().preset('Editorial')).build();figures.push([name,p]);scale.free();
}
for(const [name,plot] of figures){
 const wire=plot.toJson();assert.equal(JSON.parse(wire).version,6);const copy=c.Plot.fromJson(wire);assert.equal(copy.toJson(),wire);copy.free();fs.writeFileSync(path.join(out,`${name}.plot.json`),wire);
 const request=output.request(plot,options);plot.free();const frame=request.prepare();fs.writeFileSync(path.join(out,`${name}.scene.json`),JSON.stringify(frame.scene(),null,2));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${name}.${fmt}`),frame.export(fmt));frame.free();request.free();
}
console.log('PASS FIX-21 WASM: five independently authored v6 catalog figures, retained SVG/PDF/PNG.');
// Non-finite numeric categories cannot shift the named catalog's eligible population.
{
const scale=new c.StandaloneScale('ordinal',{range:['black']});
const p=c.plot(c.Data.columns({x:[0,1,2,3,4],key:[0,NaN,Infinity,-Infinity,1]}))
.aes(c.aes().x('x').y(1).color('key').colorScale('named')).layer(c.points())
.scale(c.colorMapped('named',scale,'Eligible').paletteScheme({id:'Category10'}).missing('#0b16212c')).build();
const request=output.request(p,options),frame=request.prepare();
const colors=frame.scene().items.filter(i=>'Point'in i.primitive&&i.layer!==null).map(i=>i.primitive.Point.fill);
const rgba=(red,green,blue,alpha=255)=>({red,green,blue,alpha});
assert.deepEqual(colors,[rgba(31,119,180),rgba(11,22,33,44),rgba(11,22,33,44),rgba(11,22,33,44),rgba(255,127,14)]);
for(const v of [frame,request,p,scale])v.free();
console.log('PASS CP-04 WASM: non-finite numeric categories retain missing paint and do not shift named palette training.');
}
