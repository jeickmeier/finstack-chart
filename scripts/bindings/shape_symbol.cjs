// FIX-S06 actual WASM symbols, source palettes and bounded owned paths.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/symbol.json')));
function near(a,b){if(typeof a==='number'&&typeof b==='number')assert(Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>near(v,b[i]));}else if(a&&typeof a==='object'){assert.deepEqual(Object.keys(a).sort(),Object.keys(b).sort());for(const k of Object.keys(a))near(a[k],b[k]);}else assert.deepEqual(a,b);}
for(const q of corpus.cases){
 const cfg={kind:q.kind,size:q.size};if(q.size<0){assert.throws(()=>new c.ShapeSymbol(cfg),e=>e.code==='CHART_NUMERICAL_DOMAIN');continue;}
 const g=new c.ShapeSymbol(cfg),copy=g.copy();assert.deepEqual(g.config(),copy.config());g.free();
 const p=copy.generate(),saved=p.copy(),expected=new c.Path(3).apply_batch(q.operations);near(p.result().geometry,expected.result().geometry);assert.equal(p.to_svg(),q.svg['3'],q.id);
 const again=copy.generate();assert.deepEqual(again.result(),p.result());again.free();copy.free();const initial=saved.result();p.move_to(10,20);p.free();assert.deepEqual(saved.result(),initial);saved.free();expected.free();
}
const [fill,stroke]=c.ShapeSymbol.palettes();assert.deepEqual(fill,corpus.palettes.fill);assert.deepEqual(stroke,corpus.palettes.stroke);assert(Object.isFrozen(fill)&&Object.isFrozen(stroke));
const a=new c.ShapeSymbol({kind:'X'}),b=new c.ShapeSymbol({kind:'Times'}),p=a.generate(),q=b.generate();assert.deepEqual(p.result(),q.result());p.free();q.free();a.free();b.free();
for(const cfg of [{kind:'Unknown'},{bad:0},{digits:-1}])assert.throws(()=>new c.ShapeSymbol(cfg));
for(const cfg of [{limits:{max_points:0}},{limits:{path:{max_operations:0,max_commands:100,max_svg_bytes:1000,max_replay_commands:100,max_subdivisions:100}}}]){const g=new c.ShapeSymbol(cfg);assert.throws(()=>g.generate(),e=>e.code==='CHART_RESOURCE_LIMIT');g.free();}
console.log('PASS WASM symbols: 156 reference cases, palettes, aliases, repeated/copy/disposal ownership, independent numeric paths and bounded failures.');
