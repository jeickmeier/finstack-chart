// FIX-19: actual pinned D3 axes in a real SVG DOM. Reference data only; Rust consumes JSON offline.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {root,workspace,provenance,writeJson,digest,retainLicense} from './provenance.mjs';
if(!process.env.PLAYWRIGHT_MODULE||!process.env.CHROMIUM_EXECUTABLE)throw Error('Set the installed PLAYWRIGHT_MODULE and pinned CHROMIUM_EXECUTABLE.');
const {chromium}=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE));
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_EXECUTABLE});
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/axes'));
const packages=['d3-array','d3-color','d3-interpolate','d3-format','d3-time','d3-time-format','d3-scale','d3-selection','d3-dispatch','d3-timer','d3-ease','d3-transition','d3-axis'];
try{
 if(browser.version()!=='151.0.7922.34')throw Error('Require pinned Chromium 151.0.7922.34, got '+browser.version());
 const results=[];
 for(const dpr of [1,2]){
  const context=await browser.newContext({locale:'en-US',timezoneId:'UTC',deviceScaleFactor:dpr});const page=await context.newPage();
  await page.setContent('<!doctype html><meta charset="utf-8"><title>FIX-19 axis reference</title><svg xmlns="http://www.w3.org/2000/svg"></svg>');
  for(const name of packages)await page.addScriptTag({path:path.join(workspace,'node_modules',name,'dist',name+'.min.js')});
  results.push(await page.evaluate(dpr=>{
   const cases=[],svg=d3.select('svg'),sides=['Top','Right','Bottom','Left'];
   const encode=v=>v instanceof Date?{Timestamp:v.getTime()}:v;
   function scale(spec){
    const s=d3['scale'+spec.kind]();
    if(spec.domain)s.domain(['Utc','Time'].includes(spec.kind)?spec.domain.map(v=>new Date(v)):spec.domain);
    if(spec.range)s.range(spec.range);
    for(const k of ['base','constant','padding','paddingInner','paddingOuter','align','round','exponent'])if(k in spec)s[k](spec[k]);
    return s;
   }
   function configure(axis,spec){
    if('arguments'in spec)axis.tickArguments(spec.arguments);
    if('interval'in spec){const [kind,every]=spec.interval;axis.ticks(d3[kind].every(every));}
    if('values'in spec)axis.tickValues(spec.values===null?null:spec.values.map(v=>typeof v==='object'&&v!==null&&'Timestamp'in v?new Date(v.Timestamp):v));
    if('formatter'in spec)axis.tickFormat(spec.formatter===null?null:spec.formatter==='Blank'?()=>'':spec.formatter==='Same'?()=>'same':spec.formatter==='Indexed'?(v,i)=>i+':'+v:d3.format(spec.formatter));
    for(const k of ['tickSize','tickSizeInner','tickSizeOuter','tickPadding','offset'])if(k in spec)axis[k](spec[k]);
    return axis;
   }
   function capture(node,identity){
    const attrs=n=>Object.fromEntries(Array.from(n.attributes).sort((a,b)=>a.name.localeCompare(b.name,'en')).map(a=>[a.name,a.value]));
    return {attributes:attrs(node),domain:Array.from(node.querySelectorAll(':scope > .domain')).map(n=>({attributes:attrs(n)})),ticks:Array.from(node.querySelectorAll(':scope > .tick')).map(n=>({identity:identity(n),value:encode(n.__data__),attributes:attrs(n),line:attrs(n.querySelector('line')),text:{attributes:attrs(n.querySelector('text')),label:n.querySelector('text').textContent}}))};
   }
   function emit(id,side,scaleSpec,config={}){
    svg.selectAll('*').remove();const s=scale(scaleSpec),a=configure(d3['axis'+side](s),config),g=svg.append('g');let n=0;const ids=new WeakMap(),identity=node=>{if(!ids.has(node))ids.set(node,n++);return ids.get(node);};
    const states=[];g.call(a);states.push(capture(g.node(),identity));
    if(config.reset){configure(a,config.reset);g.call(a);states.push(capture(g.node(),identity));}
    cases.push({id:id+'-dpr'+dpr,device_scale:dpr,side,scale:scaleSpec,config,states});
   }
   const numeric=[
    {kind:'Linear',domain:[0,1],range:[0,100]},
    {kind:'Linear',domain:[1,0],range:[100,0]},
    {kind:'Linear',domain:[-7,13],range:[20,180]},
    {kind:'Linear',domain:[1e12,1e12+1],range:[0,100]},
    {kind:'Log',domain:[1,100],range:[0,100]},
    {kind:'Log',domain:[-100,-1],range:[100,0]},
    {kind:'Log',domain:[1,64],range:[0,100],base:2},
    {kind:'Log',domain:[1,20],range:[0,100],base:2.5},
    {kind:'Symlog',domain:[-10,10],range:[0,100],constant:2},
    {kind:'Pow',domain:[0,10],range:[0,100],exponent:2},
    {kind:'Sqrt',domain:[0,100],range:[0,100]},
    {kind:'Identity',domain:[0,100]},
    {kind:'Band',domain:['a','b','c'],range:[0,100]},
    {kind:'Point',domain:['a','b','c'],range:[100,0],padding:.25},
   ];
   for(const side of sides)for(const [i,s]of numeric.entries())emit('family-'+i+'-'+side,side,s);
   for(const side of sides)for(const range of [[0,100],[20,80],[100,0]])for(const config of [{offset:0},{offset:.5},{tickSize:0},{tickSizeInner:-40,tickSizeOuter:6},{tickSizeInner:6,tickSizeOuter:0,tickPadding:-3},{tickSizeInner:0,tickSizeOuter:9,tickPadding:0}])emit('geometry-'+cases.length,side,{kind:'Linear',domain:[0,1],range},config);
   for(const count of [0,1,2,5,10,2.5,-1])emit('count-'+count,'Bottom',numeric[0],{arguments:[count]});
   for(const config of [{values:[]},{values:[.8,.2,.8,0,1]},{values:[0,.5,1],formatter:'Blank'},{values:[0,.5,1],formatter:'Same'},{values:[0,.5,1],formatter:'Indexed'},{arguments:[5,'.1%']},{arguments:[5,'s']},{arguments:[5,',f']},{arguments:[5,'.2e']},{values:[],reset:{values:null,arguments:[3]}},{values:[0,.3,1],formatter:'Same',reset:{formatter:null,arguments:[5]}}])emit('selection-'+cases.length,'Bottom',numeric[0],config);
   for(const kind of ['Band','Point'])for(const round of [false,true])for(const range of [[0,2],[2,0],[0,101]])for(const align of [0,.5,1])emit('categorical-'+cases.length,'Bottom',{kind,domain:['a','b','c'],range,round,align,padding:.2});
   for(const spec of [{domain:[Date.UTC(2024,1,28),Date.UTC(2024,2,2)],interval:['utcDay',1]},{domain:[Date.UTC(2024,0,1),Date.UTC(2024,0,1,1)],interval:['utcMinute',15]},{domain:[Date.UTC(2024,0,1),Date.UTC(2024,0,1,0,0,0,10)]},{domain:[Date.UTC(2023,0,1),Date.UTC(2026,0,1)],interval:['utcYear',1]}])emit('utc-'+cases.length,'Bottom',{kind:'Utc',domain:spec.domain,range:[0,240]},spec.interval?{interval:spec.interval}:{});
   // Shared reference scale, independent guide configuration and placement; replacement affects one guide.
   svg.selectAll('*').remove();const shared=d3.scaleLinear().domain([0,1]).range([0,100]),top=d3.axisTop(shared).ticks(2),bottom=d3.axisBottom(shared).ticks(5),extra=d3.axisBottom(shared).ticks(1),groups=[svg.append('g').attr('transform','translate(10,20)'),svg.append('g').attr('transform','translate(10,80)'),svg.append('g').attr('transform','translate(40,120)')],axes=[top,bottom,extra];let sequence=0;const ids=new WeakMap(),id=node=>{if(!ids.has(node))ids.set(node,sequence++);return ids.get(node);};
   const stages=[];const render=()=>{groups.forEach((g,i)=>g.call(axes[i]));return groups.map(g=>capture(g.node(),id));};stages.push(render());shared.domain([0,2]);stages.push(render());top.scale(d3.scaleLinear().domain([-1,1]).range([20,80]));stages.push(render());
   const sample=d3.axisBottom(d3.scaleLinear()),defaults={tickArguments:sample.tickArguments(),tickValues:sample.tickValues(),tickFormat:sample.tickFormat(),tickSize:sample.tickSize(),tickSizeInner:sample.tickSizeInner(),tickSizeOuter:sample.tickSizeOuter(),tickPadding:sample.tickPadding(),offset:sample.offset()};sample.tickArguments([3,'s']);const argumentsCopy=sample.tickArguments();argumentsCopy.push('ignored');sample.tickValues([1,2]);const valuesCopy=sample.tickValues();valuesCopy.push(3);
   return {device_scale:dpr,exports:sides.map(s=>'axis'+s).sort(),methods:Object.keys(sample).sort(),defaults,copy_contract:{arguments:sample.tickArguments(),values:sample.tickValues()},cases,shared_guides:{placements:[[10,20],[10,80],[40,120]],stages,scale_identity_after_replacement:[top.scale()===shared,bottom.scale()===shared,extra.scale()===shared]}};
  },dpr));
  await context.close();
 }
 writeJson(path.join(out,'reference.json'),{schema_version:1,profiles:results});
 const first=results[0].cases.find(c=>c.id==='count-10-dpr1');if(first.states[0].ticks.length!==11)throw Error('Independent [0,1]/count10 invariant failed.');
 for(const r of results){if(r.defaults.offset!==(r.device_scale===1?.5:0))throw Error('Device offset invariant');if(JSON.stringify(r.shared_guides.scale_identity_after_replacement)!=='[false,true,true]')throw Error('Shared scale identity invariant');}
 writeJson(path.join(out,'manifest.json'),{schema_version:1,references:packages.map(provenance),browser:{engine:'Chromium',version:browser.version(),executable_sha256:digest(fs.readFileSync(process.env.CHROMIUM_EXECUTABLE)),os:os.platform(),release:os.release(),arch:os.arch()},cases:results.reduce((n,r)=>n+r.cases.length,0),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'reference.json'))),boundary:'Pinned static SVG DOM and rerender reference only. Timed transition sampling, local-zone DST and Rust/host axis parity remain open.'});
 writeJson(path.join(out,'inventory.json'),{schema_version:1,exports:results[0].exports,methods:results[0].methods,defaults:results.map(r=>({device_scale:r.device_scale,...r.defaults})),ownership:{scale:'Shared scale lane owns domain/map/ticks/format/band metrics.',guide:'Axis lane owns identity, placement, candidate precedence, component presentation and lifecycle.'},open:['timed transitions FIX-19-H','local zone DST FIX-19-E','registered provider descriptors','full native and cross-host acceptance']});
 for(const name of packages)retainLicense(name,path.join(out,'licenses',name));
 console.log('PASS FIX-19 static browser reference:',results.reduce((n,r)=>n+r.cases.length,0),'cases, 2 device scales, shared/replaced scale identity and independent guides.');
}finally{await browser.close();}
