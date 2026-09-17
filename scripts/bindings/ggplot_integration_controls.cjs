const fs=require('fs'),path=require('path'),root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
function build(data,mode=0){
 const method=mode===3?{Registered:{operation:{id:'example.prescribed_slope',version:'1'},parameters:{slope:.3,envelope:.5}}}:'Linear';
 let model=c.smooth().stat(c.model_stat({method,n:25,terms:null,xseq:null,full_range:false,se:true,level:.95}).x('x').y('y').group('g'));if(mode===1)model=model.filter(c.filter('x').minimum(4));
 let builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('g').color_scale('groups')).scale(c.color_discrete('groups').domain(['a','b'])).layer(c.points().key_glyph('example.diamond_key',1,{padding:.1})).layer(model).theme(c.theme().reference_preset('Bw',{})).title(c.title(mode===3?'Geographic model':'Combined grammar')).subtitle(c.subtitle('').rich(c.math_text('frac(1,2)+sqrt(4)',{}))).tag(c.rich_text(mode===3?'B':'A')).legend(c.legend().scale('groups').registered('example.strip_guide',1,{})).facet(c.facet_wrap('g').registered('example.reverse_facets',1,{columns:2}));
 builder=mode===3?builder.coordinate({Geographic:{projection:{Crs:'WebMercator'},default_crs:'Wgs84'}}):builder.registered_coordinate('example.wave_coordinate',1,{amplitude:.06});
 if(mode===2)builder=builder.coordinate({Cartesian:{xlim:[{Number:4},{Number:11}]}});
return builder.build();}
for(let mode=0;mode<4;mode++){
 const indices=Array.from({length:24},(_,i)=>i),data=c.Data.columns({x:c.column(indices.map(i=>Math.floor(i/2)),{kind:'float64'}),y:c.column(indices.map(i=>2+.3*Math.floor(i/2)+Math.floor(i/2)%3+i%2),{kind:'float64'}),g:indices.map(i=>i%2===0?'a':'b')},{keys:indices.map(i=>9007199254740993n+BigInt(i))});
 const plot=build(data,mode),wire=plot.to_json(),restored=c.Plot.from_json(wire,registry),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare();
 const originalRequest=output.request(plot,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();require('assert').deepEqual(originalFrame.scene(),frame.scene());originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`integration-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`integration-${mode}.scene.json`),JSON.stringify(frame.scene()));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`integration-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();plot.dispose();data.dispose();
}
const assert=require('assert');
function materialize(rows){return c.Data.columns({x:c.column(rows.map(r=>r[1]),{kind:'float64'}),y:c.column(rows.map(r=>r[2]),{kind:'float64'}),g:rows.map(r=>r[3])},{keys:rows.map(r=>r[0])});}
let rows=Array.from({length:24},(_,i)=>[9007199254740993n+BigInt(i),Math.floor(i/2),2+.3*Math.floor(i/2)+Math.floor(i/2)%3+i%2,i%2===0?'a':'b']);
const source=materialize(rows),authored=build(source),live=authored.chart(),options=c.export_options(600,360).dpi(144).basis('current'),old=live.request(output,options),oldFrame=old.prepare(),oldSvg=oldFrame.export('svg');oldFrame.dispose();
function checkBatch(){const d=materialize(rows),p=build(d),r=live.request(output,options),s=output.request(p,options),a=r.prepare(),b=s.prepare();assert.deepEqual(a.scene().items.map(i=>i.primitive),b.scene().items.map(i=>i.primitive));a.dispose();b.dispose();r.dispose();s.dispose();p.dispose();d.dispose();}
const added=[[9007199254741017n,12,9,'a'],[9007199254741018n,12,10,'b']];assert.ok(live.commit(live.transaction().append(source,materialize(added)).build()).Applied);rows.push(...added);checkBatch();
rows[0]=[rows[0][0],0,20,'a'];assert.ok(live.commit(live.transaction().upsert(source,materialize(rows.slice(0,1))).build()).Applied);checkBatch();
assert.ok(live.commit(live.transaction().remove(source,[9007199254740994n]).retain_count(source,20).build()).Applied);rows=rows.filter(r=>r[0]!==9007199254740994n).slice(-20);checkBatch();
assert.ok(live.commit(live.transaction().append(source,materialize(rows.slice(0,1))).build()).Rejected);checkBatch();
live.act({SetViewport:{x:[4,8],y:null}});
let presented=live.present(output,options);
const targets=live.select_region({Rectangle:[0,0,600,360]}).targets,derived=targets.find(t=>t.identity.Derived),sourceTarget=targets.slice().reverse().find(t=>t.identity.Source);assert.ok(derived&&sourceTarget);
live.select([derived,sourceTarget]);live.hover([derived]);presented.dispose();presented=live.present(output,options);
const painted=presented.manifest();assert.equal(painted.state.interaction.selection.length,2);assert.equal(painted.state.hover.length,1);
const extra=materialize([[9007199254741019n,13,11,'a']]);assert.ok(live.commit(live.transaction().append(source,extra).build()).Applied);extra.dispose();
const edited=authored.edit().title(c.title('Pending title')).build();live.apply_plot(edited,live.revisions().definition);edited.dispose();live.act({SetViewport:{x:[5,9],y:null}});
const currentRequest=live.request(output,options),currentFrame=currentRequest.prepare(),current=currentFrame.manifest();currentFrame.dispose();currentRequest.dispose();assert.ok(BigInt(current.stamp.store)>BigInt(painted.stamp.store));assert.ok(BigInt(current.stamp.definition)>BigInt(painted.stamp.definition));
const matrix=[];
for(const basis of ['presented','current'])for(const view of ['visible','full_domain'])for(const selected of [false,true])matrix.push([basis,view,selected,live.request(output,options.basis(basis).view(view).interaction({selection:selected,hover:selected}))]);
live.dispose();source.dispose();authored.dispose();presented.dispose();output.dispose();registry.dispose();const again=old.prepare();assert.deepEqual(again.export('svg'),oldSvg);again.dispose();old.dispose();
const records=[];
for(const [basis,view,selected,request] of matrix){
 const frame=request.prepare(),manifest=frame.manifest(),expected=basis==='presented'?painted:current;
 assert.equal(manifest.origin_scene!==null,basis==='presented');assert.equal(manifest.stamp.store,expected.stamp.store);assert.equal(manifest.stamp.definition,expected.stamp.definition);assert.deepEqual(manifest.state,expected.state);
 assert.ok(manifest.state.interaction.selection.some(t=>t.identity.Derived));assert.ok(manifest.state.interaction.selection.some(t=>t.identity.Source));
 const effective=manifest.effective_state;assert.equal(effective.interaction.selection.length,selected?2:0);assert.equal(effective.hover.length,selected?1:0);assert.deepEqual(effective.viewport.x,view==='visible'?expected.state.viewport.x:null);assert.equal(Buffer.from(frame.export('svg')).toString().includes('Pending title'),basis==='current');
 records.push({basis,view,include:selected,store:manifest.stamp.store,definition:manifest.stamp.definition,selected:effective.interaction.selection.length,hover:effective.hover.length,viewport:effective.viewport});frame.dispose();request.dispose();
}
fs.writeFileSync(path.join(out,'capture-matrix.json'),JSON.stringify(records));console.log('PASS WASM integration: publications, updates/batch, rollback, derived/source selection and hover, revised capture matrix and disposal');
