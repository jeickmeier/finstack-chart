// SP-06 / FIX-20: filtered calendar intervals and complete time-format directives.
import * as time from 'd3-time';
import {timeFormatLocale} from 'd3-time-format';
import {scaleUtc,scaleTime} from 'd3-scale';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {root,workspace,provenance,writeJson,digest} from './provenance.mjs';
const encode=x=>Array.isArray(x)?x.map(encode):typeof x!=='number'?x:Number.isFinite(x)?x:{number:Number.isNaN(x)?'NaN':x>0?'Infinity':'-Infinity'};
const call=f=>{try{return {value:encode(f())};}catch(e){return {error:String(e.message)};}};
if(process.argv[2]==='--zone') {
 const zone=process.env.TZ;
 const utc=process.argv[3]==='utc';
 const types=[['Millisecond',time.timeMillisecond],['Second',time.timeSecond],['Minute',utc?time.utcMinute:time.timeMinute],['Hour',utc?time.utcHour:time.timeHour],['Day',utc?time.utcDay:time.timeDay],['UnixDay',time.unixDay],['Sunday',utc?time.utcSunday:time.timeSunday],['Monday',utc?time.utcMonday:time.timeMonday],['Month',utc?time.utcMonth:time.timeMonth],['Year',utc?time.utcYear:time.timeYear]];
 const samples=['2024-01-31T10:17:23.456Z','2024-02-29T10:17:23.456Z','2024-03-10T06:59:59.999Z','2024-03-10T07:00:00.001Z','2024-11-03T05:30:00Z','2024-11-03T06:30:00Z','2024-04-06T14:45:00Z','2024-04-06T15:15:00Z','2024-10-05T15:15:00Z','2024-10-05T15:45:00Z','2025-12-31T23:59:59.999Z'].map(s=>+new Date(s));
 const intervals=types.flatMap(([name,base])=>[1,2,3,7,13].flatMap(every=>samples.map(value=>{
  const interval=base.every(every);
  const span=name==='Millisecond'?31:name==='Second'?90000:name==='Minute'?4*3600000:name==='Hour'?4*86400000:35*86400000;
  return {name,every,value,floor:call(()=>+interval.floor(value)),ceil:call(()=>+interval.ceil(value)),round:call(()=>+interval.round(value)),offset:[-2.5,0,1,2.5].map(step=>({step,result:call(()=>+interval.offset(value,step))})),range:call(()=>interval.range(value,value+span).map(Number))};
 })));
 const locales=['en-US','fr-FR'].map(id=>({id,value:JSON.parse(fs.readFileSync(path.join(workspace,'node_modules/d3-time-format/locale',id+'.json')))}));
 const directives='aAbBcdefgGHIjLmMpqQsSuUVwWxXyYZ%'.split('');
 const specs=[...directives.flatMap(d=>['%'+d,'%_'+d,'%-'+d]),'%Y-%m-%d %H:%M:%S.%L %Z','%c | %x | %X | %Q','unknown %K / trailing %'];
 const formatValues=[...samples,Date.parse('2021-01-01'),Date.parse('2021-01-03'),Date.parse('2021-01-04'),Date.parse('2024-01-01')];
 if(utc)for(const year of [-1,0,1,99,100,9999,10000]){const date=new Date(0);date.setUTCFullYear(year,0,2);formatValues.push(+date);}
 if(utc)formatValues.push(-8640000000000000,8640000000000000);
 const formats=locales.flatMap(locale=>specs.map(pattern=>({locale:locale.id,pattern,values:formatValues,result:call(()=>{const f=timeFormatLocale(locale.value)[utc?'utcFormat':'format'](pattern);return formatValues.map(f);})})));
 const autoDomains=[['month-two-days','2024-01-29','2024-02-07'],['subsecond','2024-01-01T00:00:00.001Z','2024-01-01T00:00:00.019Z'],['century','2021-01-01','2029-01-01'],['one-month','2024-01-15','2024-02-15']];
 const automatic=autoDomains.flatMap(([id,start,stop])=>[false,true].flatMap(reverse=>[0,.5,1,4,10,24].map(count=>{const domain=[+new Date(start),+new Date(stop)];if(reverse)domain.reverse();const scale=(utc?scaleUtc:scaleTime)().domain(domain);return{id,domain,count,ticks:call(()=>scale.ticks(count).map(Number)),nice:call(()=>scale.copy().nice(count).domain().map(Number)),labels:call(()=>scale.ticks(count).map(scale.tickFormat(count)))};})));
 const mapping=autoDomains.flatMap(([id,start,stop])=>[false,true].flatMap(reverse=>[false,true].flatMap(clamp=>[false,true].map(round=>{
  const domain=[+new Date(start),+new Date(stop)];if(reverse)domain.reverse();
  const scale=(utc?scaleUtc:scaleTime)().domain(domain).range([-37,183]).clamp(clamp);if(round)scale.rangeRound([-37,183]);
  const values=[domain[0]-1000,domain[0],Math.trunc((domain[0]+domain[1])/2),domain[1],domain[1]+1000];
  const positions=[-100,-37,-36.9999,-1,0,1,42.424242,100,182.9999,183,200];
  return {id,domain,range:scale.range(),clamp,round,values,outputs:values.map(scale),positions,inverse:positions.map(x=>+scale.invert(x))};
 }))));
 const start=Date.parse('1970-01-01T00:00:00Z'),end=Date.parse('2070-01-01T00:00:00Z'),hour=3600000;
 let previous=-new Date(start).getTimezoneOffset()*60;
 const transitions=[{at:start,offset_seconds:previous}];
 for(let instant=start+hour;instant<=end;instant+=hour){
  const offset=-new Date(instant).getTimezoneOffset()*60;
  if(offset!==previous){let lo=instant-hour,hi=instant;while(hi-lo>1){const mid=Math.floor((hi+lo)/2);if(-new Date(mid).getTimezoneOffset()*60===previous)lo=mid;else hi=mid;}transitions.push({at:hi,offset_seconds:offset});previous=offset;}
 }
 console.log(JSON.stringify({zone,mode:utc?'Utc':'Local',node:process.versions.node,tzdata:process.versions.tz,resource:{start,end,transitions},locales,intervals,formats,automatic,mapping}));
} else {
 const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-scale/calendar'));
 const zones=[['UTC','utc'],['UTC','local'],['America/New_York','local'],['Europe/Berlin','local'],['Australia/Lord_Howe','local']].map(([zone,mode])=>{
  const p=spawnSync(process.execPath,[fileURLToPath(import.meta.url),'--zone',mode],{env:{...process.env,TZ:zone},encoding:'utf8',maxBuffer:64*1024*1024});if(p.status!==0)throw Error(p.stderr);return JSON.parse(p.stdout);
 });
 const anchors=zones.find(z=>z.mode==='Utc');
 const q=anchors.intervals.find(q=>q.name==='Month'&&q.every===1&&q.value===Date.parse('2024-01-31T10:17:23.456Z'));
 if(q.offset.find(x=>x.step===1).result.value!==Date.parse('2024-03-02T10:17:23.456Z'))throw Error('Independent month rollover anchor');
 const east=zones.find(z=>z.zone==='America/New_York');
 if(east.formats.find(f=>f.locale==='en-US'&&f.pattern==='%Z').result.value[2]!=='-0500'||east.formats.find(f=>f.locale==='en-US'&&f.pattern==='%Z').result.value[3]!=='-0400')throw Error('Independent DST offset anchor');
 writeJson(path.join(out,'cases.json'),{schema_version:1,zones});
 writeJson(path.join(out,'manifest.json'),{schema_version:1,requirements:['SCL-04','SCL-06','SCL-07','DAT-05','BND-01','FIX-20'],oracle:['d3-scale','d3-time','d3-time-format'].map(provenance),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),counts:{zones:zones.length,intervals:zones.reduce((n,z)=>n+z.intervals.length,0),formats:zones.reduce((n,z)=>n+z.formats.length,0),automatic:zones.reduce((n,z)=>n+z.automatic.length,0),mapping:zones.reduce((n,z)=>n+z.mapping.length,0)},status:'Independent reference observations; bounded timezone coverage is checked separately.'});
 console.log('PASS expanded calendar reference',zones.length,'modes/zones');
}
