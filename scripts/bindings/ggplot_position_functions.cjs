'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),records=[],output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current');
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-limit-functions.json'))).cases.concat(JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/positional-limit-function-arguments.json'))).cases);
function rejected(t){return Boolean(t.result.error)||Boolean(t.result.positions?.error);}
function prefix(t){return t.kind==='binned'?'binned-':'';}
function dataFor(t){return c.Data.columns({x:c.column(t.inputs.map(v=>v??0),{kind:'float64'}).validity(t.inputs.map(v=>v!==null))},{keys:t.inputs.map((_,i)=>100+i)});}
function build(t,d){const scale=t.kind==='binned'?c.scale_binned({transform:{identity:null,sqrt:'Sqrt',reverse:'Reverse',log10:{Log:{base:10}}}[t.transform]}):{identity:c.scale_linear,sqrt:c.scaleSqrt,reverse:c.scaleReverse,log10:()=>c.scale_log(10)}[t.transform]();let axis=c.x_axis().scale(scale).limitsFunction({operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:t.control}}).range(100,540).guideGeometry({labels:'Preserve'});if(t.oob)axis=axis.oob(t.oob[0].toUpperCase()+t.oob.slice(1));return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1)).layer(c.points().name('marks')).x_axis(axis).y_axis(c.y_axis().visible(false)).build();}
function numeric(v){return v&&typeof v==='object'?numeric(v.number):v===null?NaN:Number(v);}
function equal(a,b){return a===b||Number.isNaN(a)&&Number.isNaN(b)||Math.abs(a-b)<=3e-12*Math.max(1,Math.abs(b));}
function check(t,chart,frame){
 const limits=chart.semantics().positional_limits['0'];assert.equal(limits.length,t.result.limits.length);assert.ok(limits.every((a,i)=>equal(numeric(a),numeric(t.result.limits[i]))),JSON.stringify({t,limits}));
 const scene=frame.scene(),points=t.inputs.map(()=>null);scene.items.forEach((item,i)=>{const targets=scene.targets[i];if(targets.length&&targets[0].Source&&item.primitive.Point){const row=Number(targets[0].Source.key)-100;points[row]=(item.primitive.Point.center.x-100)/440;}});
 points.forEach((actual,i)=>{const wanted=(t.result.coordinate_positions??t.result.point_positions)[i];if(wanted===null||!Number.isFinite(numeric(wanted)))assert.equal(actual,null,JSON.stringify({t,points}));else assert.ok(actual!==null&&equal(actual,numeric(wanted)),JSON.stringify({t,points,wanted}));});
 const ticks=frame.guides().guides.find(g=>g.spec.side==='Bottom').ticks,expected=t.result.positions.flatMap((position,i)=>position!==null&&Number.isFinite(numeric(position))?[[t.result.labels[i],numeric(position)]]:[]);assert.equal(ticks.length,expected.length,JSON.stringify({t,ticks,expected}));for(const [label,position]of expected)assert.ok(ticks.some(tick=>tick.label===label&&equal((tick.position-100)/440,position)),JSON.stringify({t,ticks,label,position}));return {limits,points,ticks};
}
for(const[index,t]of cases.entries()){
 const owned=[];
 try{const d=dataFor(t);owned.push(d);const p=build(t,d);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,29);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','edited']){const current=state==='original'?restored:restored.edit().layer('marks',c.points().name('marks')).build();if(current!==restored)owned.push(current);const chart=current.chart();owned.push(chart);const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);assert.ok(!rejected(t),JSON.stringify(t));records.push({index,state,...check(t,chart,frame)});
   if(state==='original'&&!t.oob&&t.population==='spaced'&&['identity','sqrt'].includes(t.transform)&&['identity','fixed','single'].includes(t.control))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${prefix(t)}${t.transform}-${t.control}.${fmt}`),frame.export(fmt));
   if(state==='original'&&!t.oob&&t.kind==='continuous'&&t.population==='all_missing'&&t.control==='missing_lower'&&['identity','reverse'].includes(t.transform)){assert.equal(frame.scene().version,16);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${t.transform}-infinite-guide.${fmt}`),frame.export(fmt));}
   if(state==='original'&&t.population==='spaced'&&t.transform==='log10'&&t.control==='identity'&&t.oob===(t.kind==='continuous'?'censor':'squish'))for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${prefix(t)}log10-identity.${fmt}`),frame.export(fmt));
  }
 }catch(error){assert.ok(error instanceof c.ChartError&&rejected(t),`${JSON.stringify(t)}: ${error.stack}`);assert.equal(error.code,'CHART_NUMERICAL_DOMAIN');records.push({index,error:error.code});}
 finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1559);for(const [kind,control] of [...['identity','fixed','single'].map(v=>['continuous',v]),...['identity','fixed'].map(v=>['binned',v])]){
 const owned=[];
 try{const selected=Object.fromEntries(cases.filter(t=>!t.oob&&t.kind===kind&&t.transform==='identity'&&t.control===control).map(t=>[t.population,t]));const original=dataFor(selected.spaced);owned.push(original);const p=build(selected.spaced,original);owned.push(p);const wire=p.to_json(),chart=p.chart();owned.push(chart);const request=output.request(p,options);owned.push(request);const held=request.prepare();owned.push(held);const scene=held.scene();
  for(const population of ['constant','missing','all_missing','empty','spaced']){const t=selected[population],replacement=dataFor(t);owned.push(replacement);const tx=chart.transaction();owned.push(tx);const builder=tx.replace(original,replacement);owned.push(builder);const update=builder.build();owned.push(update);assert.ok('Applied'in chart.commit(update));const fresh=build(t,replacement);owned.push(fresh);const batch=fresh.chart();owned.push(batch);const actual=chart.semantics(),expected=batch.semantics(),limits=actual.positional_limits;assert.deepEqual(limits,expected.positional_limits);assert.deepEqual(actual.layers[0].domains,expected.layers[0].domains);assert.deepEqual(held.scene(),scene);assert.equal(p.to_json(),wire);records.push({kind,control,replacement:population,limits});}
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
assert.equal(records.length,1584);fs.writeFileSync(path.join(out,'position-function-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM:',records.length,'positional callback states.');
