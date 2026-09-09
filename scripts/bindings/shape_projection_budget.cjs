'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(const radial of [false,true])for(const faceted of [false,true])for(const inset of [false,true]){
 const labels=faceted?['A','B']:['A'],count=radial?2:1,d=c.Data.columns({a:labels.flatMap(()=>Array.from({length:count},(_,i)=>i*Math.PI/2)),facet:c.categorical(labels.flatMap(label=>Array(count).fill(label)))}),layer=radial?c.shapeLineRadial().shapeValue('Angle',d.field('a')).shapeValue('Radius',10):c.shapeArc().shapeValue('EndAngle',Math.PI/2);let draft=c.plot(d).layer(layer);
 if(faceted)draft=draft.facet(c.facetWrap('facet'));
 if(inset){let view=c.inset().id('repeat').rectangle(.6,.1,.3,.3).layer(layer).guides(false);if(faceted)view=view.panel({values:[{Text:'A'}]});draft=draft.inset(view);}
 const p=draft.build(),required=(radial?4:5)*(labels.length+Number(inset));
 for(const maximum of [required-1,required]){const request=output.request(p,c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().maxVertices(maximum)));if(maximum<required)assert.throws(()=>request.prepare(),e=>e.code==='CHART_RESOURCE_LIMIT');else request.prepare().free();request.free();}
 p.free();d.free();
}
console.log('PASS WASM projected shape work: exact single/facet/inset limits for atomic arcs and radial runs, with no budget reset per panel.');
