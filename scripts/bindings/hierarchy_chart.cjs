'use strict';
// HIR-07: independently authored primary chart recipes through actual WASM.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const data=c.Data.columns({id:['root','North','South','A','B','C','D','E','F'],parent:[null,'root','root','North','North','North','South','South','South'],value:new Float64Array([0,0,0,8,5,3,7,4,2]),group:['root','North','South','North','North','North','South','South','South']},{keys:Array.from({length:9},(_,i)=>9007199254741001n+BigInt(i)),name:'hierarchy'});
const resquarify={Treemap:{options:{tile:{Resquarify:1.618033988749895},padding_inner:3,padding_top:5,padding_right:5,padding_bottom:5,padding_left:5},history:true}};
const layers=[['tree',c.hierarchy_tree('id','parent')],['cluster-horizontal',c.hierarchy_cluster('id','parent').hierarchy_projection('Horizontal')],['tree-radial',c.hierarchy_tree('id','parent').hierarchy_projection('Radial')],['cluster-radial',c.hierarchy_cluster('id','parent').hierarchy_projection('Radial')],['icicle',c.hierarchy_icicle('id','parent')],['sunburst',c.hierarchy_sunburst('id','parent').hierarchy_projection({Sunburst:{inner_radius:18,radius:'Area'}})],['treemap',c.hierarchy_treemap('id','parent').hierarchy_layout(resquarify)],['pack',c.hierarchy_pack('id','parent')],['tree-node-size',c.hierarchy_tree('id','parent').hierarchy_layout({Tree:{options:{mode:{NodeSize:[30,80]}}}})]];
const plots=layers.map(([name,layer])=>[name,layer,c.plot(data).layer(layer.name('hierarchy').hierarchy_value(data.field('value')).hierarchy_label(data.field('id')).aes(c.aes().color('id'))).title(c.title(name)).build()]);
for(const [name,layer,p] of plots){
 const wire=p.to_json();assert.equal(JSON.parse(wire).version,15);const loaded=c.Plot.from_json(wire);assert.equal(loaded.to_json(),wire);fs.writeFileSync(path.join(out,`${name}.plot.json`),wire);
 for(const mode of ['preserve','outline']){
  const request=output.request(loaded,c.export_options(500,360).dpi(144).text(mode)),frame=request.prepare(),scene=frame.scene(),h=scene.hierarchies.snapshots[0];
  assert.equal(h.nodes[0].value,29);assert.equal(h.nodes.length,9);assert.deepEqual([...h.source_keys].sort(),Array.from({length:9},(_,i)=>(9007199254741001n+BigInt(i)).toString()));
  const prefix=name+'-'+(mode==='preserve'?'text':mode);fs.writeFileSync(path.join(out,`${prefix}.scene.json`),JSON.stringify(scene));
  for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${prefix}.${fmt}`),frame.export(fmt));
  if(name==='treemap'){
   assert.ok(h.history_rows>0);const resizedRequest=output.request(loaded,c.export_options(300,500).dpi(72)).with_hierarchy_history(frame),resized=resizedRequest.prepare();
   assert.equal(resized.scene().hierarchies.snapshots[0].history_members,h.history_members);fs.writeFileSync(path.join(out,`${prefix}-resized.scene.json`),JSON.stringify(resized.scene()));resized.free();resizedRequest.free();
  }
  frame.free();request.free();
 }
 loaded.free();p.free();layer.free();
}
const op=(mode,native=false)=>({operation:{id:native?'example.native_hierarchy':'example.hierarchy',version:'1'},parameters:{mode}});
let registry=c.ExtensionRegistry.example();
const registeredLayer=c.hierarchy_pack('id','parent').hierarchy_aggregation({Registered:op('Value')}).hierarchy_order({Registered:op('DescendingValue')}).hierarchy_layout({Pack:{options:{radius:'Explicit'},radius:{Registered:op('DepthPadding')}}});
const registeredPlot=c.plot(data).with_registry(registry).layer(registeredLayer).build(),registeredRequest=output.request(registeredPlot,c.export_options(400,300));registry.free();registeredPlot.free();
const registeredFrame=registeredRequest.prepare(),registeredHierarchy=registeredFrame.scene().hierarchies.snapshots[0];assert.equal(registeredHierarchy.nodes[0].value,29);for(const n of registeredHierarchy.nodes)if(!n.children.length)assert.equal(n.geometry.Circle.r,3);
registeredFrame.free();registeredRequest.free();registeredLayer.free();registry=c.ExtensionRegistry.example();assert.throws(()=>c.plot(data).with_registry(registry).layer(c.hierarchy_pack('id','parent').hierarchy_aggregation({Registered:op('Value',true)})).build(),e=>e instanceof c.ChartError);registry.free();
const paths=c.Data.columns({path:['a/b','a/c/d'],v:new Float64Array([2,3])},{keys:[71n,72n]});
const recipe={identity:'991',source:{Paths:'path'},aggregation:{Sum:'v'},label:null,layout:{Partition:{}}};const p=c.plot(paths).layer(c.hierarchy(recipe)).build(),f=output.request(p,c.export_options(300,200)).prepare(),h=f.scene().hierarchies.snapshots[0];assert.equal(h.nodes.length,4);assert.equal(h.source_keys.length,2);f.free();p.free();paths.free();data.free();output.free();
console.log('PASS WASM HIR-07: nine projections, Field selectors, v15 round trip, source identities, retained resize history, synthetic ancestry, SVG/PDF/PNG.');
