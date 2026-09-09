'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/shapes/radial.json'))),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,200).dpi(72).layout(c.layoutOptions().padding(0)),base=9007199254741001n;
const C=v=>({Constant:v}),K=i=>({Column:i}),read=(s,row)=>'Constant'in s?s.Constant:row[s.Column];
function endpoint(cfg,name,datum){const v=cfg[name]??name;return typeof v==='string'?datum[v.toLowerCase()]:v.Constant;}
function compare(a,b){if(typeof a==='number'&&typeof b==='number')assert.ok(Number.isFinite(a)&&Math.abs(a-b)<=2e-12*Math.max(1,Math.abs(b)),`${a} != ${b}`);else if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>compare(v,b[i]));}else if(a!==null&&typeof a==='object'){assert.deepEqual(Object.keys(a).sort(),Object.keys(b).sort());for(const k in a)compare(a[k],b[k]);}else assert.deepEqual(a,b);}
function transformed(commands,radial){return commands.map(op=>typeof op==='string'?op:Object.fromEntries(Object.entries(op).map(([k,vs])=>[k,vs.map((v,i)=>v*(radial?1:i%2?-2:4)+(i%2?100:200))])));}
function axes(p,radial){return p.xAxis(c.xAxis().scale(radial?c.scaleLinear().domain(0,4):c.scaleLinear().domain(-50,50)).visible(false)).yAxis(c.yAxis().scale(radial?c.scaleLog(10).domain(1,10000):c.scaleLinear().domain(-50,50)).visible(false));}
for(const test of corpus.cases){
 const cfg=test.config,family=test.family,radial=family!=='link',anchors=new Map(),curve=cfg.curve??{kind:family==='link'?'BumpX':'Linear'};let data,layer;
 if(family.startsWith('link')){
  const s=endpoint(cfg,'source',test.input),t=endpoint(cfg,'target',test.input),xs=cfg[radial?'angle':'x']??K(0),ys=cfg[radial?'radius':'y']??K(1),x=read(xs,s),y=read(ys,s),x2=read(xs,t),y2=read(ys,t);
  anchors.set(base.toString(),[[x,y],[x2,y2]].map(([a,r])=>({x:200+(radial?r*Math.cos(a-Math.PI/2):a*4),y:100+(radial?r*Math.sin(a-Math.PI/2):r*-2)})));
  data=c.Data.columns({value:[0]},{keys:[base]});layer=radial?c.shapeLinkRadial().shapeValue('StartAngle',x).shapeValue('InnerRadius',y).shapeValue('EndAngle',x2).shapeValue('OuterRadius',y2).aes(c.aes().x(2).y(100)):c.shapeLink(curve).aes(c.aes().x(x).y(y).x2(x2).y2(y2));
 }else{
  let x0=K(0),y0=C(0),x1=null,y1=K(1);const area=family==='areaRadial'&&test.helper===null;
  if(family==='lineRadial'){x0=cfg.angle??K(0);y0=cfg.radius??K(1);}else{
   x0=cfg.start_angle??x0;y0=cfg.inner_radius??y0;x1=cfg.end_angle??x1;y1='outer_radius'in cfg?cfg.outer_radius:y1;
   if('angle'in cfg){x0=cfg.angle;x1=null;}if('radius'in cfg){y0=cfg.radius;y1=null;}
   if(test.helper==='EndAngle')x0=x1??C(0);if(test.helper==='OuterRadius')y0=y1??C(0);
  }
  const mask=cfg.defined??true,defined=i=>typeof mask==='boolean'?mask:mask[i],values=s=>c.column(test.input.map((row,i)=>defined(i)?read(s,row):null),{kind:'float64'});
  data=c.Data.columns({a:values(x0),r:values(y0),a2:values(x1??x0),r2:values(y1??y0)},{keys:test.input.map((_,i)=>base+BigInt(i))});
  layer=area?c.shapeAreaRadial().shapeValue('StartAngle',data.field('a')).shapeValue('InnerRadius',data.field('r')).shapeValue('EndAngle',data.field('a2')).shapeValue('OuterRadius',data.field('r2')):c.shapeLineRadial().shapeValue('Angle',data.field('a')).shapeValue('Radius',data.field('r'));
  layer=layer.curve(curve).aes(c.aes().x(2).y(100));test.input.forEach((row,i)=>{if(defined(i)){const a=read(area?(x1??x0):x0,row),r=read(area?(y1??y0):y0,row);anchors.set((base+BigInt(i)).toString(),[{x:200+r*Math.sin(a),y:100-r*Math.cos(a)}]);}});
 }
 const p=axes(c.plot(data).layer(layer),radial).build(),wire=p.toJson();assert.equal(JSON.parse(wire).version,7);const decoded=c.Plot.fromJson(wire);assert.equal(decoded.toJson(),wire);decoded.free();const request=output.request(p,options);p.free();const f=request.prepare(),scene=f.scene(),commands=[];
 scene.items.forEach((item,i)=>{if('ShapePath'in item.primitive){const shape=item.primitive.ShapePath;commands.push(...shape.geometry.commands);assert.equal(shape.anchors.length,scene.targets[i].length);scene.targets[i].forEach((t,n)=>{const expected=anchors.get(BigInt(t.Source.key).toString());assert.ok(expected);compare(shape.anchors[n],expected[family.startsWith('link')?n:0]);});}});
 const expected=new c.Path();expected.applyBatch(test.operations);compare(commands,transformed(expected.result().geometry.commands,radial));expected.free();f.free();request.free();data.free();
}
{
 const d=c.Data.columns({value:[0]},{keys:[base]}),p=axes(c.plot(d).layer(c.shapeLinkHorizontal().aes(c.aes().x(-50).y(-50).x2(50).y2(50))),false).build(),chart=p.chart();p.free();const f=chart.present(output,options),first=chart.selectRegion({Rectangle:[0,190,10,10]}).targets,second=chart.selectRegion({Rectangle:[390,0,10,10]}).targets;assert.equal(first.length,1);assert.deepEqual(first,second);assert.equal(BigInt(first[0].identity.Source.key),base);assert.deepEqual(chart.selectRegion({Rectangle:[190,190,20,10]}).targets,[]);const hit=chart.inspect(200,100,{mode:'Containment'}).targets;assert.equal(hit.length,1);chart.focus(hit[0]);chart.present(output,options).free();assert.deepEqual(chart.inspect(-1,100,{mode:'Containment'}).targets,[]);chart.free();f.free();d.free();
}
for(const clipped of [false,true]){
 const d=c.Data.columns({a:Array.from({length:9},(_,i)=>i*Math.PI/4)},{keys:Array.from({length:9},(_,i)=>base+BigInt(i))}),layer=c.shapeAreaRadial().shapeValue('Angle',c.sourceExpr(d.field('a')).mul(1)).shapeValue('InnerRadius',10).shapeValue('OuterRadius',30),p=axes(c.plot(d).aes(c.aes().x(clipped?-.15:2).y(100)).layer(layer),true).build(),chart=p.chart();p.free();const f=chart.present(output,options);if(!clipped)assert.deepEqual(chart.inspect(200,100,{mode:'Containment'}).targets,[]);const hit=chart.inspect(clipped?10:200,clipped?100:80,{mode:'Containment'}).targets;assert.equal(hit.length,1);chart.focus(hit[0]);chart.present(output,options).free();assert.deepEqual(chart.inspect(-1,100,{mode:'Containment'}).targets,[]);chart.free();f.free();d.free();
}
{
 const d=c.Data.columns({category:['a','b','b']}),p=c.plot(d).layer(c.shapeLinkRadial().stat(c.count().group('category')).afterStat(c.statAes().x(2).y(100)).shapeValue('StartAngle',0).shapeValue('EndAngle',1).shapeValue('InnerRadius',10).shapeValue('OuterRadius',{Statistical:'Count'})).build(),request=output.request(p,options),f=request.prepare(),targets=f.scene().targets.filter(g=>g.length);assert.equal(targets.length,2);for(const g of targets){assert.equal(g.length,2);assert.deepEqual(g[0],g[1]);}assert.deepEqual(targets.map(g=>g[0].Aggregate.members.length),[1,2]);f.free();request.free();p.free();d.free();
}
{
 const d=c.Data.columns({facet:c.categorical(['A','B'])}),p=c.plot(d).facet(c.facetWrap('facet')).layer(c.shapeArc().shapeValue('EndAngle',Math.PI/2)).compileLimits({max_vertices:7}).build(),request=output.request(p,options);assert.throws(()=>request.prepare(),e=>e.code==='CHART_RESOURCE_LIMIT');request.free();p.free();d.free();
}
console.log('PASS WASM radial/link charts: 697 reference paths, exact edge/source identities, endpoint selection, non-source controls, holes/clipped focus, generated aggregates, versioned round trips and cross-panel budgets.');
