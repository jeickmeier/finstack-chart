'use strict';
// FIX-AUTH07 through real generated WASM classes and the primary syntax adapter.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../../..'),modulePath=path.resolve(process.argv[2]),out=path.resolve(process.argv[3]);
const c=require(path.join(modulePath,'authoring.cjs')),native=require(path.join(modulePath,'chart_wasm.js'));
fs.mkdirSync(out,{recursive:true});
const write=(name,value)=>fs.writeFileSync(path.join(out,name+'.json'),JSON.stringify(value,null,2));
const error=(call,code)=>{let failed=false;try{call();}catch(e){failed=true;if(code){assert(e instanceof c.ChartError);assert.equal(e.code,code);assert.equal(e.diagnostic.code,code);assert(e.context&&e.correction);}}assert(failed,'Expected rejected input');};
const data=(x,y,keys)=>c.Data.columns({x:new Float64Array(x),y:new Float64Array(y)},{keys});
async function main(){
{
const p=c.plot(data([1],[2],[1n])).aes(c.aes().x('x').y('y')).layer(c.points()).build();
const owner=p.chart();owner.legendVisible(false);const view=owner.externalView(),sibling=owner.externalView();
error(()=>view.transaction(),'CHART_UNSUPPORTED_CAPABILITY');
const tx=owner.transaction().append('data',data([2],[4],[2n])).build();owner.commit(tx);tx.free();
assert.equal(view.revisions().store,0n);view.acceptFrom(owner);
assert.equal(view.revisions().store,1n);assert.equal(sibling.revisions().store,0n);
const edited=p.edit().title(c.title('Independent view')).build();view.applyPlot(edited,0n);
assert.equal(view.revisions().definition,1n);assert.equal(owner.revisions().definition,0n);
owner.dispose();assert.equal(view.semantics().datasets[0].chunks.reduce((n,b)=>n+b.keys.length,0),2);
error(()=>view.acceptFrom(owner),'CHART_DISPOSED_HANDLE');
view.free();sibling.free();owner.free();p.free();edited.free();
}

const x=new Float64Array([1,2,3]),source=c.Data.columns({x,y:new Float64Array([2,4,3])},{keys:[1001n,1002n,1003n]});x[0]=-999;
const authored=c.plot(source).aes(c.aes().x(source.field('x')).y('y')).layer(c.line().name('prices').size(1.5)).layer(c.points().name('observations').size(3)).layer(c.labels().id('peak').at(2,4).text('Peak').offset(0,14)).title(c.title('Primary parity')).xAxis(c.xAxis().label('Time')).yAxis(c.yAxis().label('Value')).build();
const chart=authored.chart(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(400,260).dpi(96);
require('./families.cjs').run(c,root,output,options,write);
require('./replay.cjs').run(c,root,output,write);
require('./replay.cjs').runStream(c,root,output,write);
write('initial',chart.semantics());
error(()=>chart.request(output,options),'CHART_UNSUPPORTED_CAPABILITY');
{const r=chart.request(output,options.basis('current')),f=r.prepare();assert(f.export('svg').length);f.free();r.free();}
const frame=chart.present(output,options);write('scene',frame.scene());
for(const format of ['png','svg','pdf'])fs.writeFileSync(path.join(out,'primary.'+format),frame.export(format));
const selected=chart.selectSeries('observations',{limit:64});assert.deepEqual(selected.targets.map(t=>t.identity.Source.key),['1001','1002','1003']);
write('action',chart.select(selected.targets,'Replace',{expected:0n}));write('zoom',chart.zoom(200,130,2));
error(()=>chart.legendVisible(false,{expected:0n}),'CHART_REVISION_CONFLICT');error(()=>chart.layerVisible('absent',false),'CHART_MISSING_RESOURCE');
const editor=chart.editor(c.annotationEdit('peak')),proposal=editor.preview(0,0),original=editor.original().annotation;
assert.equal(proposal.id,original.id);assert.deepEqual(proposal.text,original.text);
for(const axis of ['x','y'])assert(Math.abs(proposal.anchor.Data[axis].Number-original.anchor.Data[axis].Number)<1e-12);
assert.notDeepEqual(editor.nudge().anchor,original.anchor);
const presented=chart.request(output,options),transaction=chart.transaction().id('primary-append').append('data',data([4],[8],[1004n])).build();
write('transaction',chart.commit(transaction));write('replay',chart.commit(transaction));
const edited=authored.edit().title(c.title('Edited primary')).build();assert(chart.applyPlot(edited,0n));write('final',chart.semantics());
const current=chart.request(output,options.basis('current')),matrix=[];
for(const basis of ['presented','current'])for(const view of ['visible','full_domain'])for(const selection of [false,true])matrix.push({basis,view,selection,request:chart.request(output,options.basis(basis).view(view).interaction({selection}))});
chart.dispose();chart.dispose();output.dispose();source.dispose();authored.dispose();edited.dispose();error(()=>chart.state(),'CHART_DISPOSED_HANDLE');
assert.equal(editor.preview(0,0).id,'peak');
function svg(request){const f=request.prepare();try{return new TextDecoder().decode(f.export('svg'));}finally{f.free();}}
assert(svg(presented).includes('Primary parity'));assert(svg(current).includes('Edited primary'));
for(const {basis,view,selection,request} of matrix){const f=request.prepare(),m=f.manifest();assert.equal(new TextDecoder().decode(f.export('svg')).includes('Edited primary'),basis==='current');assert.equal(m.origin_scene!==null,basis==='presented');assert.equal(m.profile.view,view==='visible'?'VisibleView':'FullDomain');assert.equal(m.interaction_policy.selection,selection);f.free();}
write('matrix',matrix.map(({basis,view,selection,request})=>({basis,view,selection,manifest:request.manifest()})));
const queue=new c.ExportQueue({maxJobs:1});let job=queue.submit(current,'svg');error(()=>queue.submit(current,'svg'),'CHART_RESOURCE_LIMIT');assert.equal(job.run()[0],60);error(()=>job.run(),'CHART_DISPOSED_HANDLE');job.free();job=queue.submit(current,'svg');assert(job.cancel());error(()=>job.run(),'CHART_CANCELLED');job.free();queue.dispose();error(()=>queue.submit(current,'svg'),'CHART_DISPOSED_HANDLE');
const exact=c.Data.columns({t:c.timestamps([(1n<<63n)-2n,(1n<<63n)-1n]),unsigned:[(1n<<64n)-2n,(1n<<64n)-1n],signed:[-(1n<<63n),(1n<<63n)-1n],nullable:c.column([1.25,null]),group:c.categorical(['b','a'])},{keys:[(1n<<64n)-2n,(1n<<64n)-1n]});
const exactPlot=c.plot(exact).aes(c.aes().x(1).y('nullable')).layer(c.points()).build();write('exact',JSON.parse(exactPlot.toJson()));
const roundtrip=c.Plot.fromJson(exactPlot.toJson());assert.deepEqual(JSON.parse(roundtrip.toJson()),JSON.parse(exactPlot.toJson()));
const statistics=c.plot(data([1,2,3,4],[2,4,6,8],[11n,12n,13n,14n])).aes(c.aes().x('x').y('y')).layer(c.points().name('summary').stat(c.summary().x('y')).afterStat(c.statAes().x(1).y('Mean'))).layer(c.line().name('fit').stat(c.fit()).afterStat(c.statAes().x('X').y('Y'))).build();write('statistics',statistics.chart().semantics());
const calls=[],rows=c.Data.rows([{a:1},{a:2}],{fields:{x:r=>(calls.push(r.a),r.a)},keys:r=>r.a});assert.deepEqual(calls,[1,2]);const rp=c.plot(rows).aes(c.aes().x('x').y('x')).layer(c.points()).build();rp.chart().semantics();assert.deepEqual(calls,[1,2]);
for(const call of [()=>c.column([true,1]),()=>c.column([1,null],{kind:'bool'}),()=>c.column([1n<<64n]),()=>c.column([-1],{kind:'uint64'}),()=>c.column([Number.MAX_SAFE_INTEGER+1,0.5]),()=>c.column([null]),()=>c.Data.columns({x:[1],y:[2,3]}),()=>c.Data.columns({x:[1]},{keys:[true]}),()=>c.Data.rows([{x:1},{y:2}]),()=>c.column([undefined]),()=>new c.ExportQueue({maxJobs:-1}),()=>new c.Output([256]),()=>c.column([1]).nullable('yes')])error(call);
error(()=>c.labels().title('Wrong'),'CHART_UNSUPPORTED_CAPABILITY');error(()=>c.plot(rows).title(c.labels()),'CHART_UNSUPPORTED_CAPABILITY');
const foreign=c.Data.columns({x:[1]});error(()=>c.plot(rows).aes(c.aes().x(foreign.field('x')).y('x')).layer(c.points()).build(),'CHART_SCHEMA_CONFLICT');
const times=c.Data.columns({t:c.timestamps([1n,2n,3n]),y:new Float64Array([1,2,3])},{keys:[1n,2n,3n]}),tc=c.plot(times).aes(c.aes().x(1).y('y')).layer(c.points()).build().chart();const receipt=tc.commit(tc.transaction().retainEventTime('data','t',1n,3n).build());assert.equal(receipt.Applied.operations[0].evicted,1);assert.deepEqual(tc.semantics().datasets[0].chunks[0].keys,['2','3']);
// Independent display/null metadata and explicit queue-vs-commit ownership.
const metadata=c.Data.columns({x:c.column([1.25,null]).formatted(['1.2500',null]).unit('USD').label('Net'),flag:c.column([false,null],{kind:'bool'})}),mp=c.plot(metadata).aes(c.aes().x(1).y('x')).layer(c.points()).build(),mb=JSON.parse(mp.toJson()).data[0].batch;
assert.equal(mb.fields[0].unit,'USD');assert.equal(mb.fields[0].label,'Net');assert.deepEqual(mb.columns[0].formatted,['1.2500',null]);assert.deepEqual(mb.columns[1].values.Boolean,[false,false]);assert.deepEqual(mb.columns[1].validity,[true,false]);
const base=c.plot(data([1,2,3],[2,4,3],[1001n,1002n,1003n])).aes(c.aes().x('x').y('y')).layer(c.line()).build(),live=base.chart().stream(c.streamOptions().transactions(1));
const t1=live.transaction().id('queue-append').append('data',data([4],[8],[1004n])).build(),t2=live.transaction().id('queue-stale').remove('data',[1001n]).build();
assert.equal(live.enqueue(t1),'Queued');assert.equal(live.enqueue(t1),'AlreadyQueued');assert.equal(live.enqueue(t2),'Backpressure');assert.equal(live.revisions().store,0n);
assert(live.commitNext().outcome.Applied);assert.equal(live.revisions().store,1n);assert.equal(live.enqueue(t2),'Queued');assert(live.commitNext().outcome.Conflict);assert.equal(live.queueStatus().committed,'1');assert.equal(live.queueStatus().failed,'1');
const batch=c.plot(data([1,2,3,4],[2,4,3,8],[1001n,1002n,1003n,1004n])).aes(c.aes().x('x').y('y')).layer(c.line()).build().chart();assert.deepEqual(live.semantics().layers[0].domains,batch.semantics().layers[0].domains);
assert.equal(live.commit(live.transaction().retainCount('data',2).build()).Applied.operations[0].evicted,2);assert.deepEqual(live.semantics().datasets[0].chunks.flatMap(b=>b.keys),['1003','1004']);
for(const v of [metadata,mp,base,live,t1,t2,batch])v.free();
// Explicitly compiled extension components preserve shared transforms and field ownership.
const ex=require(path.join(modulePath,'examples.cjs')),customSource=data([0.25,0.75,1.25,1.75],[1,1,1,1],[201n,202n,203n,204n]),node=c.transform('density',ex.densityHistogram(customSource.field('x'),[0,1,2]));
const customPlot=(native=false)=>ex.withExtensions(c.plot(customSource)).transform(node).layer(ex.chamferedBars(native).name('density-bars').fromTransform(node)).layer(c.points().name('density-points').fromTransform(node).afterStat(c.statAes().x({Custom:'left'}).y({Custom:'density'}))).build();
const custom=customPlot();write('extension',custom.chart().semantics());
const customOutput=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),cf=customOutput.request(custom,options).prepare();write('extension-scene',cf.scene());fs.writeFileSync(path.join(out,'extension.png'),cf.export('png'));
assert.deepEqual(JSON.parse(ex.loadPlot(custom.toJson()).toJson()),JSON.parse(custom.toJson()));error(()=>c.Plot.fromJson(custom.toJson()));
const nativeOnly=customPlot(true);error(()=>nativeOnly.chart(),'CHART_UNSUPPORTED_CAPABILITY');error(()=>nativeOnly.toJson(),'CHART_UNSUPPORTED_CAPABILITY');error(()=>customOutput.request(nativeOnly,options).prepare(),'CHART_UNSUPPORTED_CAPABILITY');
const foreignStat=ex.densityHistogram(c.Data.columns({x:[0.25]}).field('x'),[0,1,2]);error(()=>ex.withExtensions(c.plot(customSource)).layer(ex.chamferedBars().stat(foreignStat)).build(),'CHART_SCHEMA_CONFLICT');
for(const v of [customSource,node,custom,customOutput,cf,nativeOnly,foreignStat])v.free();
// Explicit frees release Rust payloads and wrapper allocations; JS GC timing is not an ownership guarantee.
for(const v of [chart,output,source,authored,edited,frame,editor,presented,current,queue,transaction,exact,exactPlot,roundtrip,statistics,rows,rp,foreign,times,tc])v.free();
for(const {request} of matrix)request.free();
assert(global.gc,'Run Node with --expose-gc for the allocator plateau proof');
const memory=[];
for(let batch=0;batch<6;batch++){
 for(let i=0;i<100;i++){
  const held=[],own=v=>(held.push(v),v);
  const d=own(c.Data.columns({x:new Float64Array(1000).fill(1),y:new Float64Array(1000).fill(2)}));
  const a=own(own(own(c.aes()).x('x')).y('y')),layer=own(c.line());
  const p=own(own(own(own(c.plot(d)).aes(a)).layer(layer)).build()),r=own(p.chart());
  r.dispose();r.dispose();
  for(const handle of held.reverse())handle.free();
 }
 global.gc();await new Promise(resolve=>setImmediate(resolve));global.gc();await new Promise(resolve=>setImmediate(resolve));
 memory.push(native.authoring_memory_bytes());
}
write('host_checks',{wasm_memory_bytes:memory,cycles:600,rows_per_cycle:1000,ownership:'All fluent intermediate handles explicitly freed; GC timing is not a lifetime guarantee'});
assert(Math.max(...memory.slice(3))-Math.min(...memory.slice(3))<=65536,JSON.stringify({memory}));
console.log('PASS primary WASM exact data, copies/accessors, actions/queries, stale fences, edits, capture matrix, queues, disposal, retention and memory plateau');
}
main().catch(error=>{console.error(error);process.exitCode=1;});
