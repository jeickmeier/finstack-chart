'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/discrete-identity-functions.json'))),cases=fixture.cases;
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
function descriptor(t){const s={training:'Eligible',function:{GgplotDiscreteIdentity:{limits:null,levels:t.levels===null?null:t.levels.map(v=>({Text:v})),drop:false,na_translate:true,guide:t.guide==='legend',observed:[]}}};if(t.limit_mode!=='none')s.limits_function={operation:{id:'example.discrete_limits',version:'1'},parameters:{mode:t.limit_mode}};if(t.break_mode!=='auto')s.breaks_function={operation:{id:'example.discrete_breaks',version:'1'},parameters:t.break_mode};return s;}
const layer=()=>c.points().name('marks').aesthetic_value('Shape',{kind:'Number',value:21});
function build(t,data,spec){let mapping=c.aes().x('x').y(1);mapping=t.aesthetic==='fill'?mapping.fill('v').fill_scale('identity'):mapping.color('v').color_scale('identity');return c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry).aes(mapping).scale(c.color_mapped('identity',spec??descriptor(t))).layer(layer()).build();}
function checkPaint(paint,value){if(value===null)assert.equal(paint.alpha,0);else assert.deepEqual(paint,fixture.paints[value]);}
for(const [index,t] of cases.entries()){
 const owned=[];try{
  const values=t.inputs,data=c.Data.columns({x:c.column(values.map((_,i)=>i),{kind:'float64'}),v:c.column(values,{kind:'string'}).nullable(true)});owned.push(data);
  const p=build(t,data);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,t.break_mode!=='auto'?33:t.limit_mode!=='none'?28:17);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);
  for(const state of ['original','layer_edit','theme_edit']){
   const current=state==='original'?restored:state==='layer_edit'?restored.edit().layer('marks',layer()).build():restored.edit().theme(c.theme()).build();if(current!==restored)owned.push(current);
   try{
    const chart=current.chart();owned.push(chart);const semantics=chart.semantics().layers[0];assert.ok(!t.result.error,`case${index}`);
    const legend=t.aesthetic==='fill'?(semantics.paint_legends??{}).Fill:semantics.color_legend,entries=legend?.entries??[],expected=t.result.keys[0]??{mapped:[],labels:[]};assert.equal(entries.length,expected.mapped.length,`case${index}`);entries.forEach(([label,paint],i)=>{assert.equal(label,expected.labels[i]??'NA',`case${index}`);checkPaint(paint,expected.mapped[i]);});
    const paints=(semantics.styles??[]).map(s=>t.aesthetic==='fill'?s.fill:s.color),wanted=t.result.mapped.filter(v=>t.aesthetic==='fill'||v!==null);assert.equal(paints.length,wanted.length,`case${index}`);paints.forEach((paint,i)=>checkPaint(paint,wanted[i]));records.push({index,state,entries,paints});
    const sample=t.guide==='legend'&&((t.population==='factor'&&t.break_mode==='named_reverse'&&['none','reverse'].includes(t.limit_mode))||(t.population==='nullable'&&t.break_mode==='domain'&&t.limit_mode==='none'));
    if(state==='original'&&sample){const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${String(index).padStart(3,'0')}-identity.${fmt}`),frame.export(fmt));}
   }catch(error){assert.ok(error instanceof c.ChartError&&t.result.error&&error.code==='CHART_VALIDATION',`case${index}: ${error}`);records.push({index,state,error:error.code});}
   assert.equal(restored.to_json(),wire);
  }
 }finally{for(const obj of owned.reverse())obj.dispose();}
}
const owned=[];try{
 const t=cases.find(t=>t.aesthetic==='colour'&&t.population==='ordinary'&&t.guide==='legend'&&t.limit_mode==='null'&&t.break_mode==='domain'),data=c.Data.columns({x:c.column([0,1],{kind:'float64'}),v:c.column(t.inputs,{kind:'string'})});owned.push(data);
 const spec=descriptor(t);spec.resolved_discrete_limits_null=true;spec.function.GgplotDiscreteIdentity.limits=[];const p=build(t,data,spec);owned.push(p);const wire=p.to_json();assert.equal(JSON.parse(wire).version,63);const restored=c.Plot.from_json(wire,registry);owned.push(restored);assert.equal(restored.to_json(),wire);const chart=restored.chart();owned.push(chart);assert.equal(chart.semantics().layers[0].color_legend?.entries?.length??0,0);
 const old=JSON.parse(wire);old.version=62;assert.throws(()=>c.Plot.from_json(JSON.stringify(old),registry),error=>error instanceof c.ChartError);assert.throws(()=>c.Plot.from_json(wire),error=>error instanceof c.ChartError);records.push({wire:63,null_limits:true});
}finally{for(const obj of owned.reverse())obj.dispose();}
assert.equal(records.length,577);assert.equal(fs.readdirSync(out).filter(f=>/\.(svg|pdf|png)$/.test(f)).length,18);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));options.dispose();output.dispose();registry.dispose();console.log('PASS WASM identity callbacks: 577 lifecycle/wire states and 18 publications');
