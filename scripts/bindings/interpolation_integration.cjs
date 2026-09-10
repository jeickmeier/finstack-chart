'use strict';
// IP06 independently authored registered consumers and explicit sampled states.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example();
const factory=(mode,native=false)=>c.registeredInterpolation(registry,native?'example.native_interpolation':'example.interpolation',1,{mode});
const reject=fn=>assert.throws(fn);
const f=factory('SquaredNumber'),i=f(0,100),copy=f.copy();f.dispose();reject(()=>f(0,1));assert.deepEqual(i.quantize(5),[0,6.25,25,56.25,100]);
const wire=i.toJson();assert.equal(JSON.parse(wire).version,2);reject(()=>c.Interpolator.fromJson(wire));const restored=c.Interpolator.fromJson(wire,registry);assert.equal(restored.sample(.5),25);
const piece=c.piecewise(copy,[0,100,200]);assert.deepEqual(piece.quantize(5),[0,25,100,125,200]);
for(const version of [2,9007199254740993n,true,1.5,9007199254740992])reject(()=>c.registeredInterpolation(registry,'example.interpolation',version,{mode:'SquaredNumber'}));
reject(()=>c.registeredInterpolation(registry,'example.interpolation',1,{mode:'Bad'}));
for(const extra of [undefined,()=>{},NaN,new Date()])reject(()=>c.registeredInterpolation(registry,'example.interpolation',1,{mode:'SquaredNumber',extra}));
const native=factory('SquaredNumber',true),nativeI=native(0,100);reject(()=>nativeI.toJson());
for(const family of ['linear','utc']){
 const domain=family==='linear'?[0,10]:[9007199254740993n,9007199254741993n];
 const s=new c.StandaloneScale(family,{registry,domain,range:[0,100],factory:copy,...(family==='utc'?{unit:'Nanoseconds'}:{})});
 assert.equal(s.map(family==='linear'?5:9007199254741493n),25);const r=c.StandaloneScale.fromJson(s.toJson(),registry);s.dispose();assert.equal(r.map(domain[0]),0);r.dispose();
}
for(const [family,domain,x,expected] of [['sequential',[0,10],5,25],['diverging',[-10,0,100],50,56.25],['sequential_quantile',[0,1,2,3,4],2,25]]){const s=new c.StandaloneScale(family,{registry,domain,interpolator:i,...(family==='sequential_quantile'?{}:{unknown:null,clamp:true})});assert.equal(s.map(x),expected);assert.equal(s.map(),family==='sequential_quantile'?undefined:null);assert.equal(s.map(200),100);s.dispose();}
const ns=new c.StandaloneScale('linear',{registry,factory:native});reject(()=>ns.toJson());ns.dispose();
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(600,400).dpi(72).basis('current');
function figure(t,native=false,rows=Array.from({length:21},(_,j)=>[9007199254741001n+BigInt(j),j/20])){
 const colorFactory=factory('LabColor',native),sizeFactory=factory('SquaredNumber',native);
 const color=new c.StandaloneScale('linear',{registry,factory:colorFactory,range:['#d54a3a','#28689b']}),size=new c.StandaloneScale('linear',{registry,factory:sizeFactory,range:[5,13]});
 const data=c.Data.columns({x:rows.map(r=>r[1])},{keys:rows.map(r=>r[0]),name:'interpolation'});
 const transform=c.interpolateTransformSvg('translate(0,0) rotate(-25) scale(0.7)','translate(40,10) rotate(65) scale(1.3)'),zoom=c.interpolateZoom([0,0,100],[40,20,50]),view=zoom.sample(t);
 const triangle=new c.Path().move_to(-24,18).line_to(0,-25).line_to(24,18).close_path(),camera=new c.Path().rect(-35,-20,70,40);
 const p=c.plot(data).withRegistry(registry).aes(c.aes().x('x').y(1).color('x').color_scale('perceptual'))
  .layer(c.points().numeric_scale('Size','x',size)).scale(c.color_mapped('perceptual',color)).legend(c.legend().scale('perceptual').title('Registered Lab ramp'))
  .x_axis(c.x_axis().scale(c.scale_linear().domain(0,1)).range(65,415))
  .y_axis(c.y_axis().scale(c.scale_linear().domain(0,2)).range(155,65).visible(false))
  .layer(c.vector_path('sampled-transform',triangle).transform(transform.sampleTransform(t),.001,10000).anchor({Output:{x:155,y:255}}).fill({red:213,green:74,blue:58,alpha:220}))
  .layer(c.vector_path('sampled-zoom',camera).transform([100/view[2],0,0,100/view[2],-view[0],-view[1]],.001,10000).anchor({Output:{x:395,y:255}}).fill(null).stroke({color:{red:40,green:104,blue:155,alpha:255},width:2}))
  .layer(c.labels().id('transform-label').output_at(135,330).text('Sampled transform'))
  .layer(c.labels().id('zoom-label').output_at(340,330).text('Sampled zoom'))
  .title(c.title(`Shared interpolation / t = ${t.toFixed(2)}`)).theme(c.theme().preset('Editorial')).build();
 for(const v of [colorFactory,sizeFactory,color,size,data,transform,zoom,triangle,camera])v.dispose();return p;
}
for(const t of [0,.5,1]){
 const folder=path.join(out,`frame-${t}`);fs.mkdirSync(folder,{recursive:true});const p=figure(t),wire=p.toJson();assert.equal(JSON.parse(wire).version,12);
 const loaded=c.Plot.fromJson(wire,registry),request=output.request(p,options);p.dispose();const frame=request.prepare(),other=output.request(loaded,options).prepare();assert.deepEqual(frame.export('png'),other.export('png'));
 const rampFactory=factory('LabColor'),ramp=rampFactory('#d54a3a','#28689b'),marks=frame.scene().items.filter(v=>v.layer!=null&&v.primitive.Point).map(v=>v.primitive.Point);assert.equal(marks.length,21);
 marks.forEach((mark,j)=>{const sample=ramp.sample(j/20),hex=sample.formatHex8();sample.dispose();assert.equal(hex,'#'+['red','green','blue','alpha'].map(k=>mark.fill[k].toString(16).padStart(2,'0')).join(''));assert.ok(Math.abs(mark.radius-(5+8*(j/20)**2))<1e-12);});ramp.dispose();rampFactory.dispose();
 fs.writeFileSync(path.join(folder,'interpolation.plot.json'),wire);fs.writeFileSync(path.join(folder,'interpolation.scene.json'),JSON.stringify(frame.scene()));fs.writeFileSync(path.join(folder,'interpolation.guides.json'),JSON.stringify(frame.guides()));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(folder,`interpolation.${fmt}`),frame.export(fmt));
 for(const v of [frame,other,request,loaded])v.dispose();
}
const p=figure(.5,true);reject(()=>p.toJson());reject(()=>output.request(p,options).prepare());p.dispose();
const rows=[[9007199254741001n,0],[9007199254741002n,.5],[9007199254741003n,1]],initial=figure(.5,false,rows),chart=initial.chart();chart.present(output,options).dispose();initial.dispose();const old=chart.request(output,options.basis('presented'));let held=old.prepare();const png=held.export('png');held.dispose();
for(let step=0;step<4;step++){
 let tx=chart.transaction().id(`ip-${step}`);
 if(step<2){const row=step===0?[9007199254741004n,.75]:[9007199254741002n,.25];if(step===0)rows.push(row);else rows[1]=row;const batch=c.Data.columns({x:[row[1]]},{keys:[row[0]],name:'interpolation'});tx=step===0?tx.append('interpolation',batch):tx.upsert('interpolation',batch);batch.dispose();}
 else if(step===2){rows.shift();tx=tx.remove('interpolation',[9007199254741001n]);}
 else {rows.shift();tx=tx.retain_count('interpolation',2);}
 tx=tx.build();assert.ok('Applied' in chart.commit(tx));tx.dispose();const fresh=figure(.5,false,rows),a=chart.request(output,options).prepare(),b=output.request(fresh,options).prepare();assert.deepEqual(a.export('png'),b.export('png'));for(const v of [a,b,fresh])v.dispose();
}
chart.dispose();registry.dispose();held=old.prepare();assert.deepEqual(held.export('png'),png);held.dispose();old.dispose();const last=copy(0,100);assert.equal(last.sample(.5),25);last.dispose();
for(const v of [copy,i,restored,piece,native,nativeI,output])v.dispose();
console.log('PASS IP06 WASM: registered/piecewise/time factories, v2/v12, native-only rejection, three retained frames, four update/fresh PNG comparisons and disposal.');
