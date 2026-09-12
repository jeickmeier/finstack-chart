// FIX-H01: the complete locked d3-hierarchy surface; no production JS dependency.
import * as d3 from 'd3-hierarchy';
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {root,provenance,writeJson,retainLicense,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/hierarchy'));
const cases=[];
const finite=v=>typeof v==='number'&&!Number.isFinite(v)?{number:String(v)}:v;
const clean=v=>JSON.parse(JSON.stringify(v,(_,x)=>finite(x)));
function nodes(r){return r.descendants().map(n=>({name:n.data?.name??null,...(n.id===undefined?{}:{id:n.id}),parent:n.parent?(n.parent.data?.name??n.parent.id??null):null,children:n.children?.map(c=>c.data?.name??c.id??null)??[],depth:n.depth,height:n.height,...Object.fromEntries(['value','x','y','r','x0','y0','x1','y1'].filter(k=>k in n).map(k=>[k,finite(n[k])]))}));}
function add(op,id,input,config,run){let expected;try{expected=clean(run());}catch(e){expected={error:e.message};}cases.push({id,op,input,config,expected});}
const balanced={name:'R',value:5,children:[{name:'A',value:2,children:[{name:'C',value:4},{name:'D',value:6}]},{name:'B',value:3}]};
const trees={balanced,asymmetric:{name:'R',children:[{name:'A',children:[{name:'C',children:[{name:'D',value:1}]}]},{name:'B',value:3}]},singleton:{name:'R',value:1},two:{name:'R',children:[{name:'A',value:1},{name:'B',value:3}]},zero:{name:'R',children:[{name:'A',value:0},{name:'B',value:0}]},wide:{name:'R',children:Array.from({length:20},(_,i)=>({name:`N${i}`,value:(i%5)+1}))},skew:{name:'R',children:[1e-6,1,1e3,1e6].map((v,i)=>({name:`N${i}`,value:v}))}};
const make=data=>d3.hierarchy(structuredClone(data)).sum(d=>d.value??0);
add('Node','node', {name:'leaf',value:7},{},()=>nodes(new d3.Node({name:'leaf',value:7})));
for(const [id,input] of Object.entries(trees))add('hierarchy',`nested-${id}`,input,{},()=>nodes(d3.hierarchy(input)));
add('hierarchy','custom-iterable',balanced,{children:'Set'},()=>nodes(d3.hierarchy(balanced,d=>d.children?new Set(d.children):null)));
const grouped=[['a',[['x',2],['y',3]]],['b',[['z',5]]]];
add('hierarchy','ordered-grouped',grouped,{children:'Map'},()=>{const r=d3.hierarchy(new Map(grouped.map(([k,v])=>[k,new Map(v)])));return r.descendants().map(n=>({data:Array.isArray(n.data)?[n.data[0]??null,n.data[1] instanceof Map?'Map':n.data[1]]:n.data,depth:n.depth,height:n.height,children:n.children?.length??0}));});
const tables={shuffled:[{id:'C',parentId:'A'},{id:'R'},{id:'A',parentId:'R'},{id:'B',parentId:'R'}],anonymous:[{id:'R'},{parentId:'R'},{id:'',parentId:'R'}],duplicateLeaves:[{id:'R'},{id:'A',parentId:'R'},{id:'A',parentId:'R'}],ambiguous:[{id:'R'},{id:'A',parentId:'R'},{id:'A',parentId:'R'},{id:'B',parentId:'A'}],missing:[{id:'A',parentId:'X'}],multiple:[{id:'A'},{id:'B'}],noRoot:[{id:'A',parentId:'B'},{id:'B',parentId:'A'}],cycle:[{id:'R'},{id:'A',parentId:'B'},{id:'B',parentId:'A'}],empty:[],nullParent:[{id:'R',parentId:null}],emptyParent:[{id:'R',parentId:''}],large:[{id:'9007199254740993'},{id:'18446744073709551615',parentId:'9007199254740993'}]};
for(const [id,input] of Object.entries(tables))add('stratify',`table-${id}`,input,{},()=>nodes(d3.stratify()(input)));
for(const [id,input]of [['impute',['a/b/c','a/b/d','a/e']],['root',['/']],['slashes',['a/b/','a/c']],['double',['a//b','a//c']],['escape',['a/b\\/c/d','a/e']],['backslashes',['a/b\\\\/c','a/d']],['single',['a/b/c']],['duplicates',['a/b','a/b']],['empty',[]]])add('stratify',`path-${id}`,input,{path:true},()=>nodes(d3.stratify().id(()=> 'ignored').parentId(()=> 'ignored').path(d=>d)(input)));
add('operations','operations-all',balanced,{},()=>{
 const r=make(balanced),before=[],after=[],each=[];r.eachBefore((n,i,root)=>before.push([n.data.name,i,root===r]));r.eachAfter((n,i,root)=>after.push([n.data.name,i,root===r]));r.each((n,i,root)=>each.push([n.data.name,i,root===r]));const c=r.find(n=>n.data.name==='C'),b=r.find(n=>n.data.name==='B');
 const copied=r.children[0].copy(),sorted=r.copy().sort((a,b)=>b.value-a.value),counted=r.copy().count();
 return {sum:nodes(r),count:nodes(counted),before,after,each,iterator:[...r].map(n=>n.data.name),ancestors:c.ancestors().map(n=>n.data.name),leaves:r.leaves().map(n=>n.data.name),path:c.path(b).map(n=>n.data.name),selfPath:c.path(c).map(n=>n.data.name),links:r.links().map(l=>[l.source.data.name,l.target.data.name]),find:r.find(n=>n.depth===1).data.name,noMatch:r.find(()=>false)===undefined,copy:nodes(copied),copyShared:copied.data===r.children[0].data,sort:nodes(sorted)};
});
const separation=(a,b)=> (a.parent===b.parent?1:2)/Math.max(1,a.depth);
for(const op of ['tree','cluster'])for(const [id,input]of Object.entries(trees))for(const config of [{},{size:[8,4]},{nodeSize:[2,3]},{size:[8,4],separation:'depth'},{size:[0,0]},{size:[8,4],separation:'constant',distance:3}])add(op,`${op}-${id}-${cases.length}`,input,config,()=>{const f=d3[op]();if(config.size)f.size(config.size);if(config.nodeSize)f.nodeSize(config.nodeSize);if(config.separation)f.separation(config.separation==='depth'?separation:()=>config.distance);return nodes(f(make(input)));});
for(const [id,input]of Object.entries(trees))for(const config of [{},{size:[8,4]},{size:[8,4],padding:1},{size:[8,4],padding:20},{size:[7.5,4.5],round:true},{size:[0,0]}])add('partition',`partition-${id}-${cases.length}`,input,config,()=>{const f=d3.partition();for(const [k,v]of Object.entries(config))f[k](v);return nodes(f(make(input)));});
const tilers=['treemapBinary','treemapDice','treemapSlice','treemapSliceDice','treemapSquarify','treemapResquarify'];
for(const tile of tilers)for(const [id,input]of Object.entries(trees))for(const bounds of [[0,0,8,4],[0,0,4,8],[0,0,4,4],[2,3,2,3]])add('tile',`${tile}-${id}-${cases.length}`,input,{tile,bounds},()=>{const r=make(input);d3[tile](r,...bounds);return nodes(r);});
for(const tile of tilers)for(const config of [{},{size:[80,40],padding:2},{size:[80,40],paddingInner:3,paddingTop:7,paddingRight:2,paddingBottom:4,paddingLeft:5},{size:[7.5,4.5],round:true},{size:[8,4],padding:20},{size:[80,40],paddingAccessor:true},{size:[80,40],ratio:0},{size:[80,40],ratio:3}])add('treemap',`treemap-${tile}-${cases.length}`,balanced,{tile,...config},()=>{const f=d3.treemap().tile(config.ratio!==undefined&&d3[tile].ratio?d3[tile].ratio(config.ratio):d3[tile]);for(const [k,v]of Object.entries(config))if(!['ratio','paddingAccessor'].includes(k))f[k](v);if(config.paddingAccessor)f.padding(n=>n.depth+1);return nodes(f(make(balanced)));});
add('treemap','custom-tiler',trees.two,{size:[8,4],tile:'customEqual'},()=>nodes(d3.treemap().size([8,4]).tile((p,x0,y0,x1,y1)=>{p.children.forEach((n,i)=>Object.assign(n,{x0:x0+(x1-x0)*i/p.children.length,y0,x1:x0+(x1-x0)*(i+1)/p.children.length,y1}));})(make(trees.two))));
for(const ratio of [1,(1+Math.sqrt(5))/2,3])add('history',`resquarify-${ratio}`,trees.wide,{ratio,steps:[{size:[80,40]},{size:[40,80],weights:'reverse'},{size:[100,100],weights:'alternating'},{size:[100,100],reset:true}]},()=>{let r=make(trees.wide);const f=d3.treemap().tile(d3.treemapResquarify.ratio(ratio));return [[80,40],[40,80],[100,100],[100,100]].map((size,i)=>{if(i===1)r.each(n=>{if(!n.children)n.data.value=21-Number(n.data.name.slice(1));});if(i===2)r.each(n=>{if(!n.children)n.data.value=Number(n.data.name.slice(1))%2?20:1;});if(i===3)r=r.copy();r.sum(d=>d.value??0);return nodes(f.size(size)(r));});});
for(const [id,input]of Object.entries(trees))for(const config of [{},{size:[80,40]},{size:[80,40],padding:2},{size:[80,40],paddingAccessor:true},{size:[80,40],radius:'value'},{size:[80,40],radius:'value',padding:2},{size:[0,0]}])add('pack',`pack-${id}-${cases.length}`,input,config,()=>{const f=d3.pack();if(config.size)f.size(config.size);if(config.radius)f.radius(n=>n.data.value??0);if(config.paddingAccessor)f.padding(n=>n.depth+1);if(config.padding!==undefined)f.padding(config.padding);return nodes(f(make(input)));});
for(const [id,radii]of [['empty',[]],['one',[1]],['two',[1,1]],['three',[1,1,1]],['zero',[0,0,0]],['unequal',[1e-6,1,1000,2,0.2]],['many',Array.from({length:30},(_,i)=>(i%7)+1)]])add('packSiblings',`siblings-${id}`,radii,{},()=>d3.packSiblings(radii.map(r=>({r}))));
for(const [id,input]of [['empty',[]],['one',[{x:3,y:4,r:2}]],['two',[{x:0,y:0,r:1},{x:4,y:0,r:1}]],['contained',[{x:0,y:0,r:10},{x:1,y:1,r:1}]],['coincident',[{x:0,y:0,r:1},{x:0,y:0,r:1}]],['triangle',[{x:0,y:0,r:1},{x:2,y:0,r:1},{x:1,y:Math.sqrt(3),r:1}]],['near-collinear',[{x:0,y:0,r:1},{x:2,y:1e-12,r:1},{x:4,y:0,r:1}]]])add('packEnclose',`enclose-${id}`,input,{},()=>d3.packEnclose(input)??null);
// Deterministic asymmetric forests exercise Buchheim threads and both normalization modes.
let randomState=107;const random=()=>((randomState=Math.imul(randomState,1664525)+1013904223>>>0)/4294967296);
for(let sample=0;sample<64;sample++){
 const list=Array.from({length:3+sample%38},(_,i)=>({name:`N${i}`,value:1+i%5}));
 for(let i=1;i<list.length;i++){const parent=list[Math.floor(random()*i)];(parent.children??=[]).push(list[i]);}
 for(const op of ['tree','cluster'])for(const mode of ['size','nodeSize']){
  const config={[mode]:mode==='size'?[80,40]:[2,3]};add(op,`random-${op}-${mode}-${sample}`,list[0],config,()=>nodes(d3[op]()[mode](config[mode])(make(list[0]))));
 }
}
// Topology changes compare to rebuilt reference objects, never stale private caches.
const keyed=structuredClone(balanced);let occurrence=1;(function assign(n){n.key=String(occurrence++);for(const c of n.children??[])assign(c);})(keyed);
const topologySteps=[{tree:structuredClone(keyed),ratio:GOLDEN()}];
function GOLDEN(){return (1+Math.sqrt(5))/2;}
let edited=structuredClone(keyed);edited.children.reverse();topologySteps.push({tree:structuredClone(edited),ratio:GOLDEN()});
edited.children.find(n=>n.name==='A').children.push({name:'E',key:'99',value:7});topologySteps.push({tree:structuredClone(edited),ratio:GOLDEN()});
edited.children=edited.children.filter(n=>n.name!=='B');topologySteps.push({tree:structuredClone(edited),ratio:GOLDEN()});
const a=edited.children[0],d=a.children.find(n=>n.name==='D');a.children=a.children.filter(n=>n!==d);edited.children.push(d);topologySteps.push({tree:structuredClone(edited),ratio:GOLDEN()});topologySteps.push({tree:structuredClone(edited),ratio:3});
add('historyTopology','resquarify-topology-rebuilds',keyed,{size:[80,40],steps:topologySteps},()=>topologySteps.map(step=>nodes(d3.treemap().size([80,40]).tile(d3.treemapResquarify.ratio(step.ratio))(make(step.tree)))));
// Extra independent seeded packing/enclosure geometry beyond the entry fixtures.
let packState=193;const packRandom=()=>((packState=Math.imul(packState,1664525)+1013904223>>>0)/4294967296);
const randomInputs=cases.filter(c=>c.id.startsWith('random-tree-size-')).slice(0,32).map(c=>c.input);
for(let sample=0;sample<32;sample++){
 const radii=Array.from({length:3+sample%25},()=>0.1+packRandom()*12);
 add('packSiblings',`random-siblings-${sample}`,radii,{},()=>d3.packSiblings(radii.map(r=>({r}))));
 const circles=radii.map(r=>({x:packRandom()*100-50,y:packRandom()*100-50,r}));
 add('packEnclose',`random-enclose-${sample}`,circles,{},()=>d3.packEnclose(circles));
 for(const explicit of [false,true]){const input=randomInputs[sample],config={size:[80,40],padding:2,...(explicit?{radius:'value'}:{})};add('pack',`random-pack-${sample}-${explicit}`,input,config,()=>{const f=d3.pack().size(config.size).padding(2);if(explicit)f.radius(n=>n.data.value??0);return nodes(f(make(input)));});}
}
// Independently calculated seed assertions guard the fixture generator itself.
const ops=cases.find(c=>c.id==='operations-all').expected;
assert.deepEqual(ops.iterator,['R','A','B','C','D']);assert.deepEqual(ops.leaves,['C','D','B']);assert.equal(ops.sum[0].value,20);assert.equal(ops.count[0].value,3);assert.deepEqual(ops.path,['C','A','R','B']);
const part=cases.find(c=>c.op==='partition'&&c.id.startsWith('partition-two-')&&JSON.stringify(c.config)==='{"size":[8,4]}').expected;
assert.deepEqual(part.map(n=>[n.x0,n.y0,n.x1,n.y1]),[[0,0,8,2],[0,2,2,4],[2,2,8,4]]);
assert.deepEqual(cases.find(c=>c.id==='enclose-two').expected,{x:2,y:0,r:3});
const inventory={schema_version:1,exports:Object.keys(d3).sort(),node_methods:Reflect.ownKeys(d3.Node.prototype).filter(k=>k!=='constructor').map(String).sort(),factories:{}};
for(const op of ['stratify','tree','cluster','partition','pack','treemap']){
 const f=d3[op](),methods=Object.keys(f).sort(),defaults={};for(const k of methods){const v=f[k]();defaults[k]=typeof v==='function'?{callable:true,...(['padding','paddingInner','paddingOuter','paddingTop','paddingRight','paddingBottom','paddingLeft'].includes(k)?{sample:v(make(balanced))}:{})}:v??null;}
 inventory.factories[op]={methods,defaults};
}
inventory.tiler_factories={treemapSquarify:['ratio'],treemapResquarify:['ratio']};
// Readback and reset sequences record their actual state, not inferred documentation.
const controls={};for(const op of ['tree','cluster']){const f=d3[op]();controls[op]=[{size:f.size(),nodeSize:f.nodeSize()}];f.nodeSize([2,3]);controls[op].push({size:f.size(),nodeSize:f.nodeSize()});f.size([8,4]);controls[op].push({size:f.size(),nodeSize:f.nodeSize()});}
for(const op of ['stratify','pack']){const f=d3[op](),k=op==='pack'?'radius':'path';f[k](d=>d);assert.equal(typeof f[k](),'function');f[k](null);controls[op]={[k]:f[k]()??null};}
// Every factory setter is invoked, read back, and restored through the public API.
controls.setters=[];
for(const op of ['stratify','tree','cluster','partition','pack','treemap'])for(const method of inventory.factories[op].methods){
 const f=d3[op](),old=f[method]();
 const value=['size','nodeSize'].includes(method)?[7,9]:method==='round'?true:method==='tile'?d3.treemapDice:method==='separation'?(()=>3):['id','parentId','path','radius'].includes(method)?(d=>d.value??'key'):4;
 assert.equal(f[method](value),f);
 const got=f[method]();
 let expected=typeof got==='function'?['padding','paddingInner','paddingOuter','paddingTop','paddingRight','paddingBottom','paddingLeft'].includes(method)?got(make(balanced)):'same callable':got;
 if(typeof value==='function')assert.equal(got,value);else if(Array.isArray(value)){assert.deepEqual(got,value);value[0]=999;assert.deepEqual(f[method](),[7,9]);}else if(method==='round')assert.equal(got,true);else assert.equal(expected,value);
 // size/nodeSize restore the original active mode, not a null setter coercion.
 if(['size','nodeSize'].includes(method))f.size([1,1]);else f[method](old);
 const reset=f[method]();controls.setters.push({factory:op,method,set:expected,reset:typeof reset==='function'?'callable':reset??null});
}
const owner=op=>['Node','hierarchy','stratify','operations'].includes(op)?'WP-H02':['tree','cluster'].includes(op)?'WP-H03':op==='partition'?'WP-H04':['tile','treemap','history'].includes(op)?'WP-H05':'WP-H06';
inventory.coverage=inventory.exports.map(name=>({export:name,owner:owner(name.startsWith('treemap')?'treemap':name),case_ids:cases.filter(c=>c.op===name||(name==='Node'&&c.op==='operations')||(name.startsWith('treemap')&&(c.config.tile===name||['history','historyTopology'].includes(c.op)))).map(c=>c.id),production_verdict:'NOT IMPLEMENTED'}));
writeJson(path.join(out,'inventory.json'),inventory);writeJson(path.join(out,'controls.json'),controls);writeJson(path.join(out,'reference.json'),{schema_version:1,cases});retainLicense('d3-hierarchy',path.join(out,'licenses/d3-hierarchy'));
const identity=provenance('d3-hierarchy');assert.equal(identity.version,'3.1.2');const lock=JSON.parse(fs.readFileSync(path.join(root,'tools/reference/node/package-lock.json')));
writeJson(path.join(out,'manifest.json'),{...identity,source_commit:'7bea49efdc1093b28a8d60d2f8717d8a17803564',integrity:lock.packages['node_modules/d3-hierarchy'].integrity,generator:'tools/reference/node/hierarchy.mjs',generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),reference_sha256:digest(fs.readFileSync(path.join(out,'reference.json'))),inventory_sha256:digest(fs.readFileSync(path.join(out,'inventory.json'))),case_count:cases.length});
console.log(`PASS hierarchy oracle: ${inventory.exports.length} exports, ${cases.length} cases, independent traversal/partition/enclosure seeds.`);
