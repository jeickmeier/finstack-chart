'use strict';
const fs=require('node:fs'), path=require('node:path'), assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),rust=path.resolve(process.argv[3]),out=path.resolve(process.argv[4]);
const c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
fs.mkdirSync(out,{recursive:true});
const corpus=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-path/cases.json'),'utf8'));
function draw(p,op){let args=op.args??[];if(op.op==='arc')args=['x','y','r','a0','a1','anticlockwise'].map(k=>args[k]);return p[op.op](...args);}
const records=[];
for(const test of corpus.cases){
  const p=new c.Path(test.digits),observations=[];let valid=true;
  for(const operation of test.operations){
    const before=p.result();let error=null;
    try{draw(p,operation);}catch(e){assert(e instanceof c.ChartError);error=e.code;valid=false;assert.deepEqual(p.result(),before);}
    observations.push({error,result:p.result()});
  }
  records.push({id:test.id,observations});
  const saved=p.result(),owned=p.copy();p.moveTo(777,888);assert.deepEqual(owned.result(),saved);
  if(valid){
    const batch=new c.Path(test.digits);batch.applyBatch(test.operations);assert.deepEqual(batch.result(),saved);batch.free();
    const request=c.Path.fromJson(JSON.stringify({version:1,digits:test.digits,operations:test.operations}));assert.deepEqual(request.result(),saved);request.free();
  }
  owned.free();p.free();
}
fs.writeFileSync(path.join(out,'paths.json'),JSON.stringify(records,null,2));
const round=c.pathRound();round.moveTo(1.23456,-1.23456);assert.equal(round.toString(),'M1.235,-1.235');round.free();
for(const [name,argc] of [['moveTo',2],['lineTo',2],['quadraticCurveTo',4],['bezierCurveTo',6],['arcTo',5],['arc',5],['rect',4]]){
  for(let i=0;i<argc;i++)for(const bad of [NaN,Infinity,-Infinity]){
    const p=c.path().moveTo(0,0),before=p.result(),args=Array(argc).fill(1);args[i]=bad;
    assert.throws(()=>p[name](...args),e=>e.code==='CHART_NUMERICAL_DOMAIN');assert.deepEqual(p.result(),before);p.lineTo(2,3);p.free();
  }
}
for(const request of ['{"version":2,"operations":[]}','{"version":1,"operations":[{"op":"ellipse"}]}'])assert.throws(()=>c.Path.fromJson(request),c.ChartError);
const p=c.path().rect(0,0,10,10),before=p.result(),accepted=[];
assert.throws(()=>p.replay(command=>{if(accepted.length===2)throw Error('stop');accepted.push(command);}));
assert.equal(accepted.length,2);assert.deepEqual(p.result(),before);
assert.throws(()=>p.applyBatch([{op:'moveTo',args:[20,20]},{op:'arcTo',args:[0,0,1,1,-1]}]),c.ChartError);assert.deepEqual(p.result(),before);
p.dispose();p.dispose();assert.throws(()=>p.toString(),e=>e.code==='CHART_DISPOSED_HANDLE');p.free();
const plot=c.Plot.fromJson(fs.readFileSync(path.join(rust,'figure.plot.json'),'utf8'));let draft=plot.edit();
const render=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/d3-path/render.json'),'utf8'));
function rebuild(draft){
for(const test of render.cases){
  const p=c.path().applyBatch(test.operations);
  let component=c.vectorPath(test.id,p);
  for(const [name,value] of [['anchor',test.anchor],['fill',test.fill],['stroke',test.stroke],['overflow',test.overflow]]){const next=component[name](value);component.free();component=next;}
  if(test.transform){const next=component.transform(test.transform,0.001,10000);component.free();component=next;}
  p.moveTo(999,999);p.free();const next=draft.annotation(component);draft.free();draft=next;component.free();
}
return draft;
}
draft=rebuild(draft);
const updated=draft.build(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
fs.writeFileSync(path.join(out,'figure.plot.json'),updated.toJson());
const repeatedDraft=rebuild(updated.edit()),repeated=repeatedDraft.build();
assert.equal(repeated.toJson(),updated.toJson(),'Rebuilding in the same runtime preserves exact definition identity');
repeated.free();repeatedDraft.free();
for(const dpi of [300,600]){
  const base=c.exportOptions(render.width,render.height),options=base.dpi(dpi);base.free();
  const request=output.request(updated,options),frame=request.prepare(),scene=frame.scene();assert.equal(scene.version,2);
  scene.items.forEach((item,i)=>{if(item.primitive.VectorPath)assert.deepEqual(scene.targets[i],[]);});
  fs.writeFileSync(path.join(out,`figure-${dpi}.scene.json`),JSON.stringify(scene,null,2));
  for(const format of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`figure-${dpi}.${format}`),frame.export(format));
  for(const value of [frame,request,options])value.free();
}
for(const value of [updated,draft,plot,output])value.free();
console.log('PASS FIX-P01–06 WASM: 86 sequences, batches/copies/errors/disposal and six actual exports.');
