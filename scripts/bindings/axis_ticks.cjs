'use strict';
// AXIS-02/03: independently authored actual primary WASM replay and publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registry=c.ExtensionRegistry.example(),output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.exportOptions(600,400).dpi(72).basis('current');
const formatter=(mode,name='example.guide_format',version=1)=>({Registered:{operation:{id:name,version},parameters:{mode}}});
function configure(axis,config){
 for(const key of ['tickSize','tickSizeInner','tickSizeOuter','tickPadding','offset'])if(key in config)axis=axis[key==='offset'?'tickOffset':key](config[key]);
 if('arguments'in config){const a=config.arguments;axis=axis.tickArguments({count:a[0]??null,specifier:a[1]??null});}
 if('interval'in config){const [name,step]=config.interval;axis=axis.tickArguments({interval:{unit:{utcDay:'Day',utcMinute:'Minute',utcYear:'Year'}[name],step}});}
 if('values'in config)axis=axis.tickValues(config.values);
 if('formatter'in config)axis=axis.tickFormat(config.formatter===null?null:formatter(config.formatter));
 return axis;
}
const records=[];
for(const profile of JSON.parse(fs.readFileSync(path.join(root,'fixtures/axes/reference.json'))).profiles)for(const testcase of profile.cases){
 const s=testcase.scale,kind=s.kind,domain=s.domain,horizontal=['Top','Bottom'].includes(testcase.side);
 const data=c.Data.columns({value:['Band','Point'].includes(kind)?c.categorical(domain):kind==='Utc'?c.timestamps(domain,'ms','UTC'):domain,other:domain.map(()=>1)});
 let axis=(horizontal?c.xAxis():c.yAxis()).side(testcase.side).guideProfile('D3_3_0_0'),owned=null,scale;
 if(['Band','Point'].includes(kind)){
  const spec={domain,align:s.align??.5,round:s.round??false};
  if(kind==='Band'){spec.padding_inner=spec.padding_outer=s.padding??0;scale=c.scaleBandD3(spec);}else{spec.padding=s.padding??0;scale=c.scalePointD3(spec);}
 }else if(kind==='Utc')scale=c.scaleUtc().timeDomain(...domain);
 else{const spec={domain};for(const k of ['base','constant','exponent'])if(k in s)spec[k]=s[k];owned=new c.StandaloneScale(kind.toLowerCase(),spec);scale=c.scaleNumeric(owned);}
 axis=axis.scale(scale);if(s.range)axis=axis.range(...s.range);else if(kind==='Identity')axis=axis.range(...domain);axis=configure(axis,testcase.config);
 for(let index=0;index<testcase.states.length;index++){
  if(index)axis=configure(axis,testcase.config.reset);
  const state=testcase.states[index],builder=c.plot(data).withRegistry(registry).aes(c.aes().x(horizontal?'value':'other').y(horizontal?'other':'value')).layer(c.points());
  const p=(horizontal?builder.xAxis(axis):builder.yAxis(axis)).build();assert.equal(JSON.parse(p.toJson()).version,['tickSize','tickSizeInner','tickSizeOuter','tickPadding','offset'].some(k=>k in testcase.config)?13:11);
  const request=output.request(p,options.layout(c.layoutOptions().deviceScale(profile.device_scale))),frame=request.prepare(),guides=frame.guides().guides.filter(g=>g.spec.profile==='D3_3_0_0');assert.equal(guides.length,1);
  const actual=guides[0].ticks.map(t=>({value:t.value,label:t.label})),expected=state.ticks.map(t=>({value:['Band','Point'].includes(kind)?{Category:t.value}:kind==='Utc'?{Timestamp:{value:String(t.value.Timestamp),unit:'Milliseconds'}}:{Number:t.value},label:t.text.label}));
  assert.deepEqual(actual,expected,`${testcase.id} ${index}`);
  for(let i=0;i<guides[0].ticks.length;i++){const coordinate=Number(state.ticks[i].attributes.transform.slice(10,-1).split(',')[horizontal?0:1]);assert.ok(Math.abs(guides[0].ticks[i].position-coordinate)<=1e-9,`${testcase.id}: ${guides[0].ticks[i].position} != ${coordinate}`);}
records.push({case:testcase.id,state:index,ticks:actual});frame.free();request.free();p.free();
 }
 if(owned)owned.free();data.free();
}
assert.equal(records.length,376);fs.writeFileSync(path.join(out,'reference-records.json'),JSON.stringify(records,null,2));
const base=9007199254740993n,timeData=c.Data.columns({x:c.timestamps([base,base+1000000000n],'ns','UTC'),y:[0,1]});
const timeAxis=c.xAxis().guideProfile('D3_3_0_0').tickArguments({interval:{unit:'Millisecond',step:1}}).tickValues([{Timestamp:{value:String(base+1n),unit:'Nanoseconds'}}]).tickFormat(formatter('Context'));
const timePlot=c.plot(timeData).withRegistry(registry).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(timeAxis).build(),timeRequest=output.request(timePlot,options),timeFrame=timeRequest.prepare(),tick=timeFrame.guides().guides.find(g=>g.spec.profile==='D3_3_0_0').ticks[0];assert.equal(tick.value.Timestamp.value,String(base+1n));assert.equal(tick.label,`1:0:${base+1n}`);timeFrame.free();timeRequest.free();timePlot.free();timeData.free();
const reject=(fn,code)=>assert.throws(fn,e=>e.code===code),invalidData=c.Data.columns({x:[0,1],y:[0,1]});
function invalidFormat(name='example.guide_format',version=1,mode='Same'){return c.plot(invalidData).withRegistry(registry).aes(c.aes().x('x').y('y')).layer(c.points()).xAxis(c.xAxis().guideProfile('D3_3_0_0').tickFormat(formatter(mode,name,version))).build();}
reject(()=>invalidFormat('example.guide_format',2),'CHART_UNSUPPORTED_CAPABILITY');reject(()=>invalidFormat('example.guide_format',1,'Unknown'),'CHART_VALIDATION');const native=invalidFormat('example.native_guide_format');reject(()=>native.toJson(),'CHART_UNSUPPORTED_CAPABILITY');reject(()=>output.request(native,options).prepare(),'CHART_UNSUPPORTED_CAPABILITY');native.free();invalidData.free();
const data=c.Data.columns({x:[0,.25,.5,.75,1],y:[1,2,3,2,1]},{keys:Array.from({length:5},(_,i)=>9007199254741001n+BigInt(i)),name:'ticks'});
let p=c.plot(data).withRegistry(registry).aes(c.aes().x('x').y('y')).layer(c.points())
 .xAxis(c.xAxis().scale(c.scaleLinear().domain(0,1)).range(100,500).guideProfile('D3_3_0_0').tickValues([0,.25,.5,.5,.75,1]).tickFormat({Labels:['zero','','mid','','','one']}))
 .yAxis(c.yAxis().range(280,100).visible(false))
 .guide(c.axisGuide('top','x').side('Top').guideProfile('D3_3_0_0').tickValues([0,.5,1]).tickFormat(formatter('Indexed')))
 .guide(c.axisGuide('lower','x').side('Bottom').translate(0,32).guideProfile('D3_3_0_0').tickValues([0,.5,1]).tickFormat(formatter('Same')))
 .title(c.title('Independent values and labels')).build();
// Authored guide IDs do not depend on prior reference-case allocations.
const descriptor=JSON.parse(p.toJson());['top','lower'].forEach((name,i)=>{descriptor.definition.guides[i].id=String(1026+i);descriptor.guides[name]=String(1026+i);});p.free();p=c.Plot.fromJson(JSON.stringify(descriptor),registry);
const wire=p.toJson(),loaded=c.Plot.fromJson(wire,registry);assert.equal(loaded.toJson(),wire);const request=output.request(p,options);registry.free();p.free();const frame=request.prepare();fs.writeFileSync(path.join(out,'ticks.plot.json'),wire);fs.writeFileSync(path.join(out,'ticks.guides.json'),JSON.stringify(frame.guides(),null,2));fs.writeFileSync(path.join(out,'ticks.scene.json'),JSON.stringify(frame.scene()));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`ticks.${fmt}`),frame.export(fmt));const secondRequest=output.request(loaded,options),second=secondRequest.prepare();assert.deepEqual(second.guides(),frame.guides());assert.deepEqual(second.export('png'),frame.export('png'));second.free();secondRequest.free();loaded.free();frame.free();request.free();data.free();output.free();console.log('PASS WASM AX02: 372 reference cases / 376 states, exact values/order/labels, independent resets, typed timestamps above 2^53, v11 registry round trip and retained SVG/PDF/PNG.');
