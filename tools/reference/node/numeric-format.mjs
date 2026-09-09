// SP-05 / FIX-20: expanded pinned numeric formatting observations.
import {format, formatLocale, formatSpecifier, formatPrefix} from 'd3-format';
import {ticks, tickStep} from 'd3-array';
import {scaleLog, tickFormat} from 'd3-scale';
import fs from 'node:fs';
import path from 'node:path';
import {root, provenance, writeJson, digest} from './provenance.mjs';
const out = path.resolve(process.argv[2] || path.join(root,'fixtures/parity/d3-scale/numeric-format'));
const encode = x => Array.isArray(x) ? x.map(encode) : typeof x !== 'number' ? x : Object.is(x,-0) ? {number:'-0'} : Number.isFinite(x) ? x : {number:Number.isNaN(x)?'NaN':x>0?'Infinity':'-Infinity'};
const call = f => {try{return {value:encode(f())};}catch(e){return {error:String(e.message)};}};
const values=[-Infinity,-1e308,-1e25,-1e21,-1234567.89,-1234.5,-2.675,-1.25,-.0001,-0,0,Number.MIN_VALUE,1e-30,1e-25,1e-24,1e-8,1e-6,.1,.5,1.005,1.25,2.675,9.999999,99.95,999.5,1234.5,1e20,1e21,1e25,1e308,Infinity,NaN];
const specs=['','n','q','c','d','b','o','x','X','e','f','g','p','r','s','%',',.0f',',.2f','+.1f',' .1f','(.2f','$,.2f','($.2f','#b','#o','#x','#X','08.2f','020,.2f','0=+20,.2f','*>20.2f','*<20.2f','*^20.2f','.0g','.21g','.21r','.0s','.21s','.20f','.20e','.0%','.2p','.2~s','.2~g','.2~e','.99f','.99g','^12.3e'];
const locales=[
 {id:'default',value:{decimal:'.',thousands:',',grouping:[3],currency:['$',''],percent:'%',minus:'−',nan:'NaN'}},
 {id:'decimal-comma',value:{decimal:',',thousands:'\u202f',grouping:[3],currency:['',' €'],percent:' %',minus:'−',nan:'indéfini'}},
 {id:'indian-numerals',value:{decimal:'.',thousands:',',grouping:[3,2],currency:['₹',''],percent:' pct',minus:'minus',nan:'not a number',numerals:['⓪','①','②','③','④','⑤','⑥','⑦','⑧','⑨']}},
 {id:'ungrouped',value:{decimal:'.',thousands:'_',grouping:[],currency:['',''],percent:'%',minus:'-',nan:'N/A'}},
];
const formats=locales.flatMap(locale=>specs.map(specifier=>({locale:locale.id,specifier,values:encode(values),result:call(()=>{const f=formatLocale(locale.value).format(specifier);return values.map(f);})})));
const invalid=['.f','..2f','10.2ff','!','.','--f','2,2f'];
const parse=invalid.map(specifier=>({specifier,result:call(()=>String(formatSpecifier(specifier)))}));
const text=locales.flatMap(locale=>['c','>12c','*^12c','$>12c','$12c',',12c'].map(specifier=>({locale:locale.id,specifier,value:'hello 123',result:call(()=>formatLocale(locale.value).format(specifier)('hello 123'))})));
const prefix=locales.flatMap(locale=>[0,1e-30,1e-12,1,1000,1e27,Infinity].flatMap(reference=>['.2s','+.3s'].map(specifier=>({locale:locale.id,specifier,reference:encode(reference),values:encode(values),result:call(()=>values.map(formatLocale(locale.value).formatPrefix(specifier,reference)))}))));
const countCases=[0,-1,.1,.5,1,2.5,10,NaN,Infinity];
const tick= [[0,0],[1,1],[-0,-0],[0,1],[1,0],[-1,1],[1e-300,9e-300],[1e15,1e15+10],[Infinity,Infinity],[-Infinity,Infinity]].flatMap(([start,stop])=>countCases.map(count=>({start:encode(start),stop:encode(stop),count:encode(count),step:encode(tickStep(start,stop,count)),result:call(()=>ticks(start,stop,count))})));
const log=[2,Math.E,10,.5].flatMap(base=>[[.01,100],[-100,-.01],[100,.01]].flatMap(domain=>[0,1,4,10,Infinity].flatMap(count=>[null,',.2f','.3s'].map(specifier=>({base,domain,count:encode(count),specifier,values:encode([-.1,.01,.02,.03,.09,.1,.2,.5,1,2,3,5,9,10,20,50,100,NaN]),result:call(()=>{const s=scaleLog().domain(domain).base(base);const f=s.tickFormat(count,specifier);return [-.1,.01,.02,.03,.09,.1,.2,.5,1,2,3,5,9,10,20,50,100,NaN].map(f);})})))));
if (JSON.stringify(scaleLog().domain([.01,100]).ticks(1)) !== '[1]') throw Error('Independent authored log domain anchor');
writeJson(path.join(out,'cases.json'),{locales,formats,parse,text,prefix,tick,log});
writeJson(path.join(out,'manifest.json'),{schema_version:1,requirements:['SCL-07','LAY-01','LAY-02','THM-03','FIX-20'],oracle:['d3-format','d3-array','d3-scale'].map(provenance),generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),counts:{formats:formats.length,parse:parse.length,text:text.length,prefix:prefix.length,tick:tick.length,log:log.length},status:'Independent reference observations; compare against production before claiming parity.'});
console.log('PASS expanded numeric-format oracle',formats.length,'formats',tick.length,'ticks',log.length,'log formatters');
