'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),records=[];
function data(values,parents){return c.Data.columns({id:['root','a','b','c','d','e','f'],parent:parents,value:values},{keys:Array.from({length:7},(_,i)=>9007199254741001n+BigInt(i)),name:'live'});}
const parents=[null,'root','root','a','a','b','b'],layouts=[{Tree:{options:{}}},{Cluster:{options:{}}},{Partition:{}},{Pack:{options:{}}},{Treemap:{options:{},history:false}},{Treemap:{options:{tile:{Resquarify:1.6}},history:true}}];
for(const [index,layout]of layouts.entries()){
 const recipe={identity:'991',source:{Table:{id:'id',parent:'parent'}},aggregation:{Sum:'value'},label:null,layout};
 function author(values,parents){const d=data(values,parents),p=c.plot(d).layer(c.hierarchy(recipe)).build();d.free();return p;}
 const p=author([0,0,0,8,5,3,7],parents),wire=p.to_json(),charts=[p.chart(),c.Plot.from_json(wire).chart()];p.free();
 const opts=c.export_options(480,280).dpi(72).basis('current');for(const chart of charts)chart.present(output,opts).free();
 const old=charts[0].request(output,opts.basis('presented'));let f=old.prepare();const oldPng=f.export('png');f.free();
 const steps=[[[0,0,0,1,9,4,7],parents,[480,280]],[[0,0,0,1,9,4,7],parents,[280,480]],[[0,0,0,1,9,4,7],[null,'root','root','b','a','b','b'],[280,480]]];
 for(const [step,[values,par,size]]of steps.entries()){
  for(const chart of charts){const d=data(values,par),tx=chart.transaction().id(`hierarchy-${step}`).replace('live',d).build();d.free();assert.ok('Applied'in chart.commit(tx));assert.ok('AlreadyApplied'in chart.commit(tx));tx.free();}
  const frames=charts.map(chart=>chart.present(output,c.export_options(...size).dpi(72).basis('current'))),nodes=frames.map(f=>f.scene().hierarchies.snapshots[0].nodes);assert.deepEqual(nodes[0],nodes[1]);assert.deepEqual(frames[0].export('png'),frames[1].export('png'));
  if(index!==5){const batch=author(values,par),req=output.request(batch,c.export_options(...size).dpi(72)),fresh=req.prepare();assert.deepEqual(fresh.scene().hierarchies.snapshots[0].nodes,nodes[0]);assert.deepEqual(fresh.export('png'),frames[0].export('png'));fresh.free();req.free();batch.free();}
  const saved=old.prepare();assert.deepEqual(saved.export('png'),oldPng);saved.free();records.push({family:index,step,nodes:nodes[0],replay_equal:true,batch_equal:index!==5,old_snapshot_equal:true});for(const f of frames)f.free();
 }
 for(const chart of charts)chart.free();f=old.prepare();assert.deepEqual(f.export('png'),oldPng);f.free();old.free();
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');output.free();console.log('PASS WASM hierarchy: 18 update/resize/reparent traces, duplicate transaction rejection, equivalent histories, 15 stateless batch comparisons, retained exports after disposal.');
