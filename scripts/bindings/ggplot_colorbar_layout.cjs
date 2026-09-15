'use strict';
// FIX-GG05: actual WASM colorbar geometry, lifecycle and immutable publication.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);
fs.mkdirSync(out,{recursive:true});
const sampling=process.argv.includes('--sampling'),demand=process.argv.includes('--demand'),constant=process.argv.includes('--constant'),orientation=process.argv.includes('--orientation'),presentation=process.argv.includes('--presentation'),boundaries=process.argv.includes('--steps-boundaries'),steps=process.argv.includes('--steps')||boundaries,alpha=process.argv.includes('--alpha'),display=process.argv.includes('--display')||alpha||steps;
let reference=JSON.parse(fs.readFileSync(path.join(root,sampling?'fixtures/parity/ggplot2/colorbar-boundaries.json':'fixtures/parity/ggplot2/colorbar-layout.json'))).cases;
if(sampling)reference=reference.filter(q=>q.display==='raster'&&!q.reverse&&q.direction==='vertical');
if(demand)reference=[];
if(constant)reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-constant.json'))).cases;
if(orientation)reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-boundaries.json'))).cases.filter(q=>q.display==='raster');
if(presentation){reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-presentation.json'))).cases;for(const q of reference)q.result.values=q.result.keys;}
if(display){reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-display.json'))).cases;for(const q of reference)q.result.values=q.result.keys;reference.push(...JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-boundaries.json'))).cases.filter(q=>q.display!=='raster'));}
if(alpha){reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-alpha.json'))).cases;for(const q of reference)q.result.values=q.result.keys;}
if(steps){reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorsteps-layout.json'))).cases.filter(q=>q.even_steps&&!q.show_limits);for(const q of reference){q.display='rectangles';q.nbin=null;const r=q.result;r.decor_colors=r.decor.colour.map((c,i)=>[Math.min(r.decor.min[i],r.decor.max[i]),c]).sort((a,b)=>a[0]-b[0]).map(v=>v[1]);r.values=r.key['.value'];r.labels=r.key['.label'];}}
if(boundaries){reference=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorsteps-boundaries.json'))).cases.filter(q=>!(q.family==='binned'&&q.mode==='null'));for(const q of reference){Object.assign(q,{palette:'asymmetric',display:'rectangles',nbin:null,direction:'vertical',reverse:false,constant:q.population==='constant'});const r=q.result;if(!r.error){r.decor_colors=r.decor?.colour??[];r.values=r.key?.['.value']??[];r.labels=r.key?.['.label']??[];}}}
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),records=[];
function paint(text){if(text==='grey50')return {red:127,green:127,blue:127,alpha:255};return {red:parseInt(text.slice(1,3),16),green:parseInt(text.slice(3,5),16),blue:parseInt(text.slice(5,7),16),alpha:text.length===9?parseInt(text.slice(7,9),16):255};}
function scale(palette,hidden=false,nbin=null,constant=false,controls=null){
 const [colors,values]={ordinary:[['#000000','#ffffff'],[0,1]],transparent:[['#0000ff20','#ffffff80','#ff0000e0'],[0,.2,1]],asymmetric:[['#0000ff','#ffffff','#ff0000'],[0,.2,1]],discontinuous:[['#ff0000','#ff0000','#0000ff','#0000ff'],[0,.499,.5,1]]}[palette];
 const result={...(nbin===null?{}:{colorbar_options:{nbin}}),training:'Eligible',function:{Interpolated:{normalization:{Sequential:{family:'Linear',domain:[-2,8],clamp:false}},output:{Interpolate:{operation:'GgplotPalette',spec:{Gradient:{colors:colors.map(paint),values}}}},unknown:{kind:'Missing'}}},ggplot:{Continuous:{limits:display&&constant?[2,2]:constant?[3,3]:[-2,8],oob:'Censor'}},guide:hidden?'Hidden':{Colorbar:{breaks:display&&constant?[2]:constant?[3]:[-2,0,3,8],labels:controls?.hidden_labels?'Hidden':'Automatic'}}};
 if(controls)result.colorbar_options??={};
 if(controls)for(const [key,value]of Object.entries(controls))if(!['hidden_labels','steps_family','steps_endpoints','steps_default','boundary'].includes(key))result.colorbar_options[key]=value;
 if(steps&&controls){const cuts=controls.steps_endpoints?[-2,0,3,8]:[0,3];if(controls.steps_family==='binned'){result.function.Interpolated.normalization={Ggplot:{family:'Linear',domain:[-2,8],reverse:false,rescaler:'Range'}};result.ggplot={Binned:{limits:[-2,8],breaks:{Explicit:cuts},oob:'Squish',right:true}};result.guide=hidden?'Hidden':{[controls.steps_default?'Binned':'BinnedSteps']:'Automatic'};}else result.guide=hidden?'Hidden':{ContinuousSteps:{breaks:cuts,labels:'Automatic'}};}
 if(boundaries&&controls){const q=controls.boundary,limits=q.limits,cuts=Array.isArray(q.breaks)?q.breaks.map(v=>v===null?{number:'NaN'}:typeof v==='string'?{number:v}:v):q.breaks;result.function.Interpolated.normalization={Ggplot:{family:'Linear',domain:limits,reverse:false,rescaler:'Range'}};if(q.family==='binned'){result.ggplot={Binned:{limits,breaks:cuts==='automatic'?{Nice:5}:{Explicit:cuts},oob:'Squish',right:true}};result.guide=hidden?'Hidden':{BinnedSteps:'Automatic'};}else{result.ggplot={Continuous:{limits,oob:'Censor'}};result.guide=hidden||q.mode==='null'?'Hidden':{ContinuousSteps:{breaks:cuts==='automatic'?null:cuts,labels:'Automatic'}};}}
 return result;
}
function author(palette,channel,facet,nbin=null,constant=false,controls=null){
 let data=c.Data.columns({x:new Float64Array([1,2,3,4,1,2,3,4]),v:new Float64Array(display&&constant?Array(8).fill(2):[-2,0,3,8,-2,0,3,8]),f:['A','A','A','A','B','B','B','B']});
 if(boundaries&&controls?.boundary.population==='empty'){data.free();data=c.Data.columns({x:new Float64Array([]),v:new Float64Array([]),f:c.column([],{kind:'string'})});}
 let aes=c.aes().x('x').y(1);aes=aes[channel]('v')[channel+'_scale']('v');
 let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(aes).scale(c.color_mapped('v',scale(palette,false,nbin,constant,controls))).layer(c.points());
 if(facet!=='single')builder=builder.facet(c.facet_wrap('f').collect_guides(facet==='collected'));
 const plot=builder.build();data.free();aes.free();builder.free();return plot;
}
function close(a,b){assert(Math.abs(a-b)<3e-12,`${a} != ${b}`);}
function inspect(plot,q,channel,facet,state){
 const chart=plot.chart(),semantic=chart.semantics();chart.free();assert.equal(semantic.layers.length,facet==='single'?1:2);const styles=semantic.layers.flatMap(layer=>layer.styles??[]);
 const expected=[...q.result.mapped,...q.result.mapped].map(paint);assert.deepEqual(styles.map(s=>s[channel]),expected);
 const request=output.request(plot,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const direction=q.direction[0].toUpperCase()+q.direction.slice(1),paired=scene.items.filter(i=>i.primitive.SampledGradientRectangle).map(i=>[i.primitive.SampledGradientRectangle,i.clip??null]),bars=state==='hidden'||(boundaries&&q.result.decor_colors.length===0)?0:facet==='local'?2:1;
 if((q.result.decor_colors.length===1||(display&&q.constant&&q.display==='gradient')))for(const item of scene.items){const r=item.primitive.Rectangle;if(r&&(Math.abs(r.bounds.height/r.bounds.width-20/3)<1e-10||Math.abs(r.bounds.width/r.bounds.height-20/3)<1e-10))paired.push([{bounds:r.bounds,direction,colors:[r.fill]},item.clip??null]);}
 assert.equal(paired.length,bars,`${q.palette} ${channel} ${facet} ${state}`);const normalized=[];
 const expectedTicks=(q.result.tick_positions??q.result.values).filter(v=>v!==null),expectedLabels=q.result.labels.filter((_,i)=>q.result.values[i]!==null);
 for(const [ramp,clip] of paired){
  assert.equal(ramp.direction,direction);let expectedColors=q.result.decor_colors.map(paint);if(display&&q.constant&&q.display==='gradient')expectedColors=expectedColors.slice(0,1);if(display&&q.display==='gradient'&&expectedColors.length>1)assert.equal(ramp.mode,'Endpoints');assert.deepEqual(ramp.colors,direction==='Horizontal'?expectedColors:expectedColors.reverse());
  if(display&&q.display==='rectangles'&&expectedColors.length>1)assert.equal(ramp.mode,'Steps');
  const b=ramp.bounds,bottom=b.origin.y+b.height,keys=[];
  for(const item of scene.items){
   const p=item.primitive.Path;if(!p||p.commands.length!==4)continue;
   const first=p.commands[0].MoveTo,third=p.commands[2].MoveTo;if(!first||!third)continue;
   let position;
   if(direction==='Horizontal'){if(first.y!==b.origin.y||!(b.origin.y<third.y&&third.y<bottom))continue;position=(first.x-b.origin.x)/b.width;}
   else{if(first.x!==b.origin.x||!(b.origin.x<third.x&&third.x<b.origin.x+b.width))continue;position=(bottom-first.y)/b.height;}
   if(position>=-1e-12&&position<=1+1e-12)keys.push(position);
  }
  assert.equal(keys.length,expectedTicks.length,JSON.stringify(q));keys.forEach((v,i)=>close(v,expectedTicks[i]));
  const labels=scene.items.filter(i=>JSON.stringify(i.clip??null)===JSON.stringify(clip)&&i.primitive.Text&&['-2','0','2','3','4','6','8','Inf','-Inf'].includes(i.primitive.Text.text)).map(i=>i.primitive.Text.text);
  assert.deepEqual(labels,expectedLabels);normalized.push(keys);
 }
 const record={palette:q.palette,channel,facet,state,styles,keys:normalized,gradients:paired.map(([p])=>p)};
 if(orientation||presentation||display)record.parameters=Object.fromEntries(['nbin','direction','reverse','lower','upper','labels',...(display?['display','constant']:[]),...(alpha?['alpha']:[]),...(steps?['family','endpoints','guide_kind']:[]),...(boundaries?['population','mode']:[])].map(k=>[k,q[k]??null]));
 records.push(record);return [request,frame,scene];
}
for(const [pi,q] of reference.entries()){
 const channels=orientation||display?[['color','fill','stroke'][pi%3]]:constant||presentation?['color']:['color','fill','stroke'];
 const facets=boundaries?['single']:orientation||display?[['single','collected','local'][Math.floor(pi/3)%3]]:presentation?[['single','collected','local'][pi%3]]:constant?['single']:['single','collected','local'];
 const controls=orientation||presentation||display?{direction:q.direction[0].toUpperCase()+q.direction.slice(1),reverse:q.reverse,draw_lower_limit:q.lower??true,draw_upper_limit:q.upper??true,hidden_labels:q.labels==='hidden'}:null;
 if(display&&!steps)controls.display=q.display[0].toUpperCase()+q.display.slice(1);
 if(alpha&&q.alpha!==null)controls.alpha=q.alpha;
 if(steps)Object.assign(controls,{steps_family:q.family,steps_endpoints:q.endpoints,steps_default:q.guide_kind==='default'});
 if(boundaries)controls.boundary=q;
 const isConstant=constant||(display&&q.constant===true);
 for(const channel of channels)for(const [fi,facet] of facets.entries()){
 if(boundaries&&q.result.error){assert.throws(()=>{const p=author(q.palette,channel,facet,q.nbin??null,isConstant,controls);output.request(p,c.export_options(600,360)).prepare();},e=>e instanceof c.ChartError);records.push({family:q.family,population:q.population,mode:q.mode,error:true});continue;}
 const plot=author(q.palette,channel,facet,q.nbin??null,isConstant,controls),wire=plot.to_json(),[request,frame,before]=inspect(plot,q,channel,facet,'initial');
 if(sampling||constant||orientation||presentation||display){const version=alpha&&q.alpha!==null?67:display&&!steps&&q.display!=='raster'?66:controls?65:64;assert.equal(JSON.parse(wire).version,version);const stale=JSON.parse(wire);stale.version=version-1;assert.throws(()=>c.Plot.from_json(JSON.stringify(stale)),e=>e instanceof c.ChartError);}
 const loaded=c.Plot.from_json(wire);let [rq,fr]=inspect(loaded,q,channel,facet,'roundtrip');fr.free();rq.free();loaded.free();
 const hidden=plot.edit().scale(c.color_mapped('v',scale(q.palette,true,q.nbin??null,isConstant,controls))).build();[rq,fr]=inspect(hidden,q,channel,facet,'hidden');fr.free();rq.free();
 const restored=hidden.edit().scale(c.color_mapped('v',scale(q.palette,false,q.nbin??null,isConstant,controls))).build();[rq,fr]=inspect(restored,q,channel,facet,'restored');fr.free();rq.free();restored.free();hidden.free();
 assert.equal(plot.to_json(),wire);assert.deepEqual(frame.scene(),before);
 const publish=steps?(!boundaries||(q.population==='ordinary'&&q.result.decor_colors.length>0)):alpha?pi%4===Math.floor(pi/4)%4:display?((pi<80&&(q.nbin===null||q.nbin===2.5))||(pi>=80&&q.palette==='discontinuous'&&q.nbin===5)):presentation?(q.nbin===5||(q.nbin>0&&q.lower&&q.upper&&q.labels==='automatic')):orientation||constant||channel===['color','fill','stroke'][(pi+fi)%3];
 if(publish){
  const name=orientation||presentation||display?`${boundaries?"step-boundaries":steps?"steps":alpha?"alpha":display?"display":orientation?"orientation":"presentation"}-${String(pi).padStart(3,"0")}`:`${q.palette}-${channel}-${facet}`+((sampling||constant)?`-nbin-${q.nbin}`:``);fs.writeFileSync(path.join(out,`${name}.scene.json`),JSON.stringify(before));fs.writeFileSync(path.join(out,`${name}.plot.json`),wire);
  for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`${name}.${fmt}`),frame.export(fmt));
 }
 frame.free();request.free();plot.free();
}
}
if(demand)for(const q of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/colorbar-demand.json'))).cases){
 const data=c.Data.columns({v:new Float64Array(q.population==='empty'?[]:[-2,0,3,8])}),aes=c.aes().x(1).y(1).fill('v').fill_scale('v');
 const spec=scale('ordinary',q.hidden,q.nbin==='NA'?{number:'NaN'}:q.nbin);
 if(!q.hidden){if(q.selection==='no_breaks')spec.guide.Colorbar.breaks=[];if(q.selection==='no_labels')spec.guide.Colorbar.labels='Hidden';}
 const builder=c.plot(data).profile('Ggplot2_4_0_3').aes(aes).scale(c.color_mapped('v',spec)).layer(c.points()),plot=builder.build(),loaded=c.Plot.from_json(plot.to_json()),request=output.request(loaded,c.export_options(600,360));
 let frame,result;
 try{frame=request.prepare();}catch(e){assert(e instanceof c.ChartError);assert('error' in q.result);result={error:true};}
 if(frame){assert(!('error' in q.result));const scene=frame.scene(),bars=scene.items.filter(i=>i.primitive.SampledGradientRectangle||(i.primitive.Rectangle&&Math.abs(i.primitive.Rectangle.bounds.height/i.primitive.Rectangle.bounds.width-20/3)<1e-10)).length;assert.equal(bars,q.result.guide_count);result={bars};frame.free();}
 records.push({nbin:q.nbin,population:q.population,selection:q.selection,hidden:q.hidden,result});
 request.free();loaded.free();plot.free();builder.free();aes.free();data.free();
}
output.free();assert.equal(records.length,boundaries?180:steps?72:alpha?384:display?704:orientation?192:presentation?512:demand?60:constant?48:sampling?432:108);fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));
console.log(`PASS WASM GG-05: ${records.length} geometry/lifecycle states and ${fs.readdirSync(out).filter(n=>n.endsWith('.png')).length*3} publication files.`);
