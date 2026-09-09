// SP-06 actual single-threaded Node WASM with the same supplied timezone revisions.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),modulePath=path.resolve(process.argv[2]),out=path.resolve(process.argv[3]),artifacts=path.resolve(process.argv[4]);
const native=require(path.join(modulePath,'chart_wasm.js')),c=require(path.join(modulePath,'authoring.cjs'));
fs.mkdirSync(out,{recursive:true});const corpus=JSON.parse(fs.readFileSync(path.join(ROOT,'fixtures/parity/d3-scale/calendar/cases.json')));
const number=x=>({kind:'Number',value:x});
function spec(z,domain){const r=z.resource,ts=r.transitions,zone=z.mode==='Utc'?'Utc':{Local:{version:1,zone:z.zone,revision:'7',tzdata:z.tzdata,coverage:{start:String(r.start),end:String(r.end)},initial_offset_seconds:ts[0].offset_seconds,transitions:ts.slice(1).map(t=>({at_millis:String(t.at),offset_seconds:t.offset_seconds}))}};return {domain:domain.map(String),unit:'Milliseconds',zone,range:[number(0),number(1)],factory:{kind:'Value'},clamp:false,unknown:{kind:'Missing'}};}
const every=(name,k)=>({unit:['Sunday','Monday'].includes(name)?{Week:name}:name,step:k});
// SP-07 public facade replay; adapters translate JSON only.
class PublicTime {
 constructor(spec){this.inner=c.StandaloneScale.fromSpec({Time:JSON.parse(spec)});}
 static owned(inner){const result=Object.create(PublicTime.prototype);result.inner=inner;return result;}
 static from_json(wire){return this.owned(c.StandaloneScale.fromJson(wire));}
 to_json(){return this.inner.toJson();}copy(){return PublicTime.owned(this.inner.copy());}
 dispose(){this.inner.dispose();}free(){this.inner.free();}
 map_json(value){return JSON.stringify(this.inner.mapValue(value));}invert(value){return this.inner.invert(value);}
 floor(value,interval){return this.inner.floor(value,JSON.parse(interval));}ceil(value,interval){return this.inner.ceil(value,JSON.parse(interval));}
 round(value,interval){return this.inner.roundTime(value,JSON.parse(interval));}offset(value,interval,steps){return this.inner.offset(value,JSON.parse(interval),steps);}
 format(value,fmt){return this.inner.format(value,JSON.parse(fmt));}
 ticks(selection,budget){const q=JSON.parse(selection);return this.inner.ticks(q.Count??10,{interval:q.Interval,budget});}
 nice(selection){const q=JSON.parse(selection);return PublicTime.owned(this.inner.nice(q.Count??10,{interval:q.Interval}));}
}
const Time=process.argv.includes('--public')?PublicTime:native._TimeScale;
const domain=scale=>{const spec=JSON.parse(scale.to_json()).spec;return (spec.Time??spec).domain;};
const counts={intervals:0,formats:0,automatic:0,mapping:0};
for(const z of corpus.zones){
 const s=new Time(JSON.stringify(spec(z,[1704067200000,1704153600000]))),copied=s.copy(),wire=s.to_json();const restored=Time.from_json(wire);assert.equal(restored.to_json(),wire);restored.dispose();restored.free();
 for(const q of z.intervals){counts.intervals++;const interval=JSON.stringify(every(q.name,q.every));for(const method of ['floor','ceil','round'])assert.equal(s[method](BigInt(q.value),interval),BigInt(q[method].value));for(const row of q.offset)assert.equal(s.offset(BigInt(q.value),interval,row.step),BigInt(row.result.value));}
 const locales=Object.fromEntries(z.locales.map(l=>[l.id,l.value]));
 for(const q of z.formats){counts.formats++;const fmt=JSON.stringify({pattern:q.pattern,locale:locales[q.locale]});assert.deepEqual(q.values.map(v=>s.format(BigInt(v),fmt)),q.result.value);}
 for(const q of z.automatic){counts.automatic++;const a=new Time(JSON.stringify(spec(z,q.domain))),selection=JSON.stringify({Count:q.count}),ticks=Array.from(a.ticks(selection,10000));assert.deepEqual(ticks,q.ticks.value.map(BigInt));const nice=a.nice(selection);assert.deepEqual(domain(nice),q.nice.value.map(String));nice.dispose();nice.free();assert.deepEqual(ticks.map(v=>a.format(v,JSON.stringify({pattern:null,locale:locales['en-US']}))),q.labels.value);a.dispose();a.free();}
 for(const q of z.mapping){counts.mapping++;const d=spec(z,q.domain);Object.assign(d,{range:q.range.map(number),clamp:q.clamp,factory:{kind:q.round?'Round':'Value'}});const a=new Time(JSON.stringify(d));q.values.forEach((v,i)=>{const x=JSON.parse(a.map_json(BigInt(v))).value,e=q.outputs[i];assert(Math.abs(x-e)<=Math.max(1e-10,Math.abs(e)*1e-12));});assert.deepEqual(q.positions.map(p=>a.invert(p)),q.inverse.map(BigInt));a.dispose();a.free();}
 s.dispose();assert.equal(copied.to_json(),wire);copied.dispose();copied.free();assert.throws(()=>s.to_json());s.free();
}
const d=spec(corpus.zones[0],[1700000000000000001n,1700000000000000011n]);d.unit='Nanoseconds';const a=new Time(JSON.stringify(d));assert.equal(a.invert(.3),1700000000000000004n);assert.equal(JSON.parse(a.map_json(1700000000000000004n)).value,.3);const bad=JSON.parse(a.to_json());bad.version=2;assert.throws(()=>Time.from_json(JSON.stringify(bad)));a.dispose();a.free();
const output=new c.Output(new Uint8Array(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf'))));
for(const name of ['spring','fall']){const p=c.Plot.from_json(fs.readFileSync(path.join(artifacts,name+'.plot.json'),'utf8')),wire=JSON.parse(p.to_json()),axis=wire.definition.axes.find(a=>a.scale.Calendar);assert.equal(axis.scale.Calendar.spec.zone.Local.revision,'42');const request=output.request(p,c.export_options(900,300)),frame=request.prepare(),svg=Buffer.from(frame.export('svg'));assert.deepEqual(svg,fs.readFileSync(path.join(artifacts,name+'.svg')));p.dispose();const next=request.prepare();assert.deepEqual(Buffer.from(next.export('svg')),svg);next.dispose();fs.writeFileSync(path.join(out,name+'.svg'),svg);frame.dispose();request.dispose();}
fs.writeFileSync(path.join(out,'checks.json'),JSON.stringify({counts,resource_revision:'7',chart_revision:'42',exact_big_integer:true,copied_and_disposed:true},null,2)+'\n');console.log('PASS SP-06 WASM',counts,'two retained publication fixtures and exact bigint lifecycle');
