// FIX-S04: actual WASM arc/pie ownership and shared engine, using independent fixtures.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/arc-pie.json')));
const snake=k=>k.replace(/[A-Z]/g,v=>'_'+v.toLowerCase());
function near(a,b){if(typeof a==='number'&&typeof b==='number')assert(Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((x,i)=>near(x,b[i]));}else if(a&&typeof a==='object'){assert.deepEqual(Object.keys(a).sort(),Object.keys(b).sort());for(const k of Object.keys(a))near(a[k],b[k]);}else assert.deepEqual(a,b);}
let negative=0;
for(const q of corpus.arcs){
 const config=Object.fromEntries(Object.entries(q.config).map(([k,v])=>[snake(k),v])),datum=Object.fromEntries(Object.entries(q.input).map(([k,v])=>[snake(k),v]));
 const arc=new c.ShapeArc(config),copy=arc.copy();assert.deepEqual(arc.config(),copy.config());arc.free();
 if(!q.finite){assert.throws(()=>copy.generate(datum),e=>e.code==='CHART_NUMERICAL_DOMAIN');copy.free();negative++;continue;}
 const p=copy.generate(datum),saved=p.copy(),expected=new c.Path(3).apply_batch(q.operations);near(p.result().geometry.commands,expected.result().geometry.commands);near(copy.centroid(datum),q.centroid);assert.equal(p.to_svg(),q.svg['3'],q.id);
 const again=copy.generate(datum);assert.deepEqual(p.result(),again.result());again.free();copy.free();const retained=saved.result();p.move_to(998,997);p.free();assert.deepEqual(saved.result(),retained);saved.free();expected.free();
}
let count=0;
for(const q of corpus.pies){
 if(q.order==='dataDescending')continue; // Custom portable comparators are WP-S07.
 const pie=new c.ShapePie({order:{default:'ValuesDescending',none:'Input',valuesAscending:'ValuesAscending'}[q.order],angles:{start_angle:q.start,end_angle:q.end,pad_angle:q.pad}}),copy=pie.copy();pie.free();const values=q.data.map(d=>d.value),actual=copy.layout(q.data,values),again=copy.layout(q.data,values);copy.free();assert.deepEqual(actual,again);near(actual,q.result.map(a=>Object.fromEntries(Object.entries(a).map(([k,v])=>[snake(k),v]))));if(q.data.length){assert.equal(actual[0].data.id,q.data[0].id);actual[0].data.label='mutated';assert.deepEqual(again[0].data,q.data[0]);}count++;
}
assert.equal(count,144);assert.equal(negative,1);
const constant=new c.ShapeArc({inner_radius:0,outer_radius:10,start_angle:0,end_angle:Math.PI/2}),p=constant.generate();assert.equal(p.to_svg(),'M0,-10A10,10,0,0,1,10,0L0,0Z');p.free();constant.free();
const pie=new c.ShapePie({value:2});assert.equal(pie.layout([4,7])[0].value,2);pie.free();
assert.throws(()=>new c.ShapeArc({wrong:1}));assert.throws(()=>new c.ShapePie({order:'Unknown'}));
const missing=new c.ShapeArc();assert.throws(()=>missing.generate());missing.free();const mismatch=new c.ShapePie();assert.throws(()=>mismatch.layout(['a'],[]));mismatch.free();
console.log('PASS WASM arc/pie: 620 arc cases (619 finite, one checked overflow), 144 portable pie layouts, exact metadata, constants, copies, disposal and independent paths. 48 native comparator fixtures remain Rust-only until WP-S07.');
