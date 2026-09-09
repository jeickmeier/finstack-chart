'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/stack.json'))).cases,key=9007199254741001n;
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().padding(0));
function num(v){return typeof v==='number'?v:({'NaN':NaN,'-0':-0})[v.number];}
function near(a,b){if(typeof a==='number'&&typeof b==='number')assert(Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>near(v,b[i]));}else if(a&&typeof a==='object'){assert.deepEqual(Object.keys(a).sort(),Object.keys(b).sort());for(const k of Object.keys(a))near(a[k],b[k]);}else assert.deepEqual(a,b);}
function plot(d,layer){return c.plot(d).aes(c.aes().x('x').x2('x').y('y').y2(0).group('g')).layer(layer).xAxis(c.xAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).yAxis(c.yAxis().scale(c.scaleLinear().domain(0,4)).visible(false)).build();}
let checked=0,precisionDiagnostics=0;
for(const test of cases){
 const n=test.keys.length;if(!n)continue;const rows=test.matrix.flatMap((row,i)=>row.flatMap((v,g)=>v===null?[]:[[i,g,v]])).reverse(),d=c.Data.columns({x:new Float64Array(rows.map(r=>r[0])),y:new Float64Array(rows.map(r=>r[2])),g:new BigInt64Array(rows.map(r=>BigInt(r[1])))},{keys:rows.map(r=>key+BigInt(r[0]*n+r[1]))});
 for(const area of [false,true]){
  const layer=(area?c.shapeArea():c.bars()).position(c.shapeStack(Array.from({length:n},(_,i)=>i)).stackOrder(test.order).stackOffset(test.offset).stackMissing(test.missing)),p=plot(d,layer);assert.equal(JSON.parse(p.toJson()).version,7);c.Plot.fromJson(p.toJson()).free();const request=output.request(p,options);p.free();let frame;try{frame=request.prepare();}catch(error){assert.equal(error.code,'CHART_PRECISION_LOSS');assert.ok(test.series.some(s=>s.points.some(p=>Math.abs(num(p.y0))>1e30||Math.abs(num(p.y1))>1e30)));precisionDiagnostics++;request.free();continue;}const scene=frame.scene(),observed=[];
  scene.items.forEach((item,index)=>{const targets=scene.targets[index];if(!targets.length)return;const ids=targets.map(t=>BigInt(t.Source.key));observed.push(...ids);const g=Number((ids[0]-key)%BigInt(n)),sample=Number((ids[0]-key)/BigInt(n));
   if(area){const shape=item.primitive.ShapePath,points=test.series[g].points,runs=[];let run=[];points.forEach((p,i)=>{if(Number.isNaN(num(p.y1))){if(run.length)runs.push(run);run=[];}else run.push(i);});if(run.length)runs.push(run);run=runs.find(r=>r.includes(sample));assert.deepEqual(ids,run.filter(i=>test.matrix[i][g]!==null).map(i=>key+BigInt(i*n+g)));
    const upper=run.map(i=>[i*100,200-num(points[i].y1)*50]),lower=[...run].reverse().map(i=>[i*100,200-num(points[i].y0)*50]);near(shape.geometry.commands,[{MoveTo:upper[0]},...[...upper.slice(1),...lower].map(p=>({LineTo:p})),'Close']);near(shape.anchors.map(p=>[p.x,p.y]),run.filter(i=>test.matrix[i][g]!==null).map(i=>[i*100,200-num(points[i].y1)*50]));
   }else{const p=test.series[g].points[sample],a=200-num(p.y0)*50,b=200-num(p.y1)*50;if(a===b)near(item.primitive.Rule.from.y,a);else{near(item.primitive.Rectangle.bounds.origin.y,Math.min(a,b));near(item.primitive.Rectangle.bounds.height,Math.abs(a-b));}}
  });assert.deepEqual(observed.sort(),rows.map(r=>key+BigInt(r[0]*n+r[1])).sort());checked++;frame.free();request.free();
 }d.free();
}
assert.equal(checked+precisionDiagnostics,810);
{
 const d=c.Data.columns({x:[0,1,2,0,2],y:[1,1,1,2,2],g:new BigInt64Array([1n,1n,1n,0n,0n])},{keys:Array.from({length:5},(_,i)=>key+BigInt(i))}),p=plot(d,c.shapeArea().position(c.shapeStack([0,1]).stackMissing('Zero'))),chart=p.chart();p.free();const f=chart.present(output,options);assert.equal(f.scene().targets.flat().length,5);const hits=chart.inspect(25,150,{mode:'Containment'}).targets;assert.ok(hits.length);assert.ok(hits.every(t=>BigInt(t.identity.Source.key)>=key&&BigInt(t.identity.Source.key)<key+5n));chart.focus(hits[0]);chart.present(output,options).free();assert.deepEqual(chart.inspect(-1,100,{mode:'Containment'}).targets,[]);f.free();chart.free();
}
{
 const d=c.Data.columns({group:['a','b','b']}),layer=c.bars().stat(c.count().group('group')).afterStat(c.statAes().x(2).y('Count').y2(0)).position(c.shapeStack(['a','b'])),p=c.plot(d).layer(layer).build(),request=output.request(p,options),f=request.prepare(),targets=f.scene().targets.flat().filter(t=>'Aggregate'in t);assert.deepEqual(targets.map(t=>t.Aggregate.members.length),[1,2]);f.free();request.free();p.free();
}
console.log(`PASS WASM stack: ${checked} reference chart comparisons plus ${precisionDiagnostics} explicit off-scale publication-precision diagnostics, exact sparse path commands/source anchors, signed bar extents, reversed input, wire roundtrip, clipping/focus without synthetic targets and generated aggregate membership.`);
