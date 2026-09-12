'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const Instance=WebAssembly.Instance;let memory;
WebAssembly.Instance=class extends Instance {constructor(module,imports){super(module,imports);memory=this.exports.memory;}};
let c;try{c=require(path.resolve(process.argv[2],'authoring.cjs'));}finally{WebAssembly.Instance=Instance;}
const envelope={version:1,identity:'9007199254740999',input:{Rows:Array.from({length:1000},(_,i)=>({key:String(i+1),parent:i?String(Math.floor((i-1)/2)+1):null,data:{value:i%7}}))}};
function cycle(){
 const tree=new c.Hierarchy(envelope).count().layout({Tree:{options:{}}}),saved=tree.nodes(),root=saved[0].handle,copy=tree.copy(),serialized=copy.toJson();
 saved[0].data.value=999;assert.notEqual(tree.node(root).data.value,999);
 tree.dispose();tree.dispose();assert.throws(()=>tree.nodes(),e=>e.code==='CHART_DISPOSED_HANDLE');tree.free();
 const restored=c.Hierarchy.fromJson(serialized);assert.equal(restored.toJson(),serialized);
 copy.count();assert.equal(restored.toJson(),serialized);copy.free();restored.free();return saved;
}
for(let i=0;i<20;i++)cycle();const warm=memory.buffer.byteLength,samples=[];let saved;
for(let i=0;i<60;i++){saved=cycle();samples.push(memory.buffer.byteLength);}
assert.ok(samples.at(-1)-warm<=4194304,'unbounded hierarchy memory growth');
const retained=JSON.stringify(saved),old=memory.buffer;memory.grow(1);assert.equal(old.byteLength,0);assert.equal(JSON.stringify(saved),retained);
const root=path.resolve(__dirname,'../..'),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),data=c.Data.columns({id:['r','a'],parent:[null,'r'],v:new Float64Array([0,1])});
const p=c.plot(data).layer(c.hierarchy_pack('id','parent').hierarchy_value('v')).build(),request=output.request(p,c.export_options(200,200)),frame=request.prepare(),bytes=frame.export('svg'),copy=Uint8Array.from(bytes),scene=frame.scene();
frame.free();request.free();p.free();data.free();output.free();memory.grow(1);assert.deepEqual(bytes,copy);assert.equal(scene.hierarchies.snapshots[0].nodes[0].value,1);
fs.writeFileSync(process.argv[3],JSON.stringify({version:1,nodes:1000,warmup:20,cycles:60,warm_bytes:warm,samples_bytes:samples,retained_growth_bytes:samples.at(-1)-warm,forced_memory_growth:true,owned_scene_and_svg_after_disposal:true,verdict:'PASS'},null,2));
console.log('PASS WASM hierarchy ownership: disposed/copied/restored sessions, bounded allocator reuse, forced memory growth and retained chart bytes/metadata.');
