// Explicit-zone calendar oracle; launched by scale.mjs in a fresh Node process per zone.
import {scaleTime} from 'd3-scale';
import * as time from 'd3-time';
const zone=process.env.TZ;
if(!zone)throw Error('A timezone must be explicitly supplied');
const intervals=[['second',time.timeSecond,15],['minute',time.timeMinute,30],['hour',time.timeHour,3],['day',time.timeDay,2],['week',time.timeWeek,1],['month',time.timeMonth,3],['year',time.timeYear,1]];
const dates=[['spring','2024-03-09T00:00:00Z','2024-03-12T00:00:00Z'],['autumn','2024-11-02T00:00:00Z','2024-11-05T00:00:00Z'],['europe-spring','2024-03-30T00:00:00Z','2024-04-02T00:00:00Z'],['europe-autumn','2024-10-26T00:00:00Z','2024-10-29T00:00:00Z'],['half-hour-spring','2024-10-05T00:00:00Z','2024-10-08T00:00:00Z'],['half-hour-autumn','2024-04-06T00:00:00Z','2024-04-09T00:00:00Z'],['leap','2024-02-27T00:00:00Z','2024-03-03T00:00:00Z'],['year','2023-12-30T00:00:00Z','2024-01-04T00:00:00Z']];
const records=dates.map(([id,a,b])=>{
 const s=scaleTime([new Date(a),new Date(b)],[0,100]);
 return {id,domain:s.domain().map(Number),samples:[0,25,50,75,100].map(y=>({position:y,inverse:+s.invert(y)})),automatic:[1,4,10,24].map(count=>({count,ticks:s.ticks(count).map(Number),labels:s.ticks(count).map(s.tickFormat(count)),nice:s.copy().nice(count).domain().map(Number)})),intervals:intervals.map(([name,i,k])=>({name,every:k,floor:+i.floor(new Date(a)),ceil:+i.ceil(new Date(a)),offset:+i.offset(new Date(a),k),nice:s.copy().nice(i.every(k)).domain().map(Number),ticks:['second','minute'].includes(name)?null:s.ticks(i.every(k)).map(Number)}))};
});
const start=Date.parse('2020-01-01T00:00:00Z'),stop=Date.parse('2031-01-01T00:00:00Z'),hour=3600000;
let previous=-new Date(start).getTimezoneOffset()*60;
const transitions=[{at:start,offset_seconds:previous}];
for(let instant=start+hour;instant<=stop;instant+=hour){
 const offset=-new Date(instant).getTimezoneOffset()*60;
 if(offset!==previous){
  let lo=instant-hour,hi=instant;
  while(hi-lo>1){const mid=Math.floor((hi+lo)/2);if(-new Date(mid).getTimezoneOffset()*60===previous)lo=mid;else hi=mid;}
  transitions.push({at:hi,offset_seconds:offset});previous=offset;
 }
}
console.log(JSON.stringify({zone,node:process.versions.node,tzdata:process.versions.tz,icu:process.versions.icu,resource:{start,end:stop,transitions},records}));
