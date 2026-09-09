// FIX-I01: pinned d3-interpolate outputs, captured before the next mutable sample.
import * as d3 from 'd3-interpolate';
import * as colors from 'd3-color';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,retainLicense,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-interpolate'));
export function scalar(v){return Number.isFinite(v)?Object.is(v,-0)?{number:'-0'}:v:{number:Number.isNaN(v)?'NaN':v>0?'Infinity':'-Infinity'};}
export function pack(v){
 if(v===undefined)return {kind:'Missing'};
 if(v===null)return {kind:'Null'};
 if(typeof v==='number')return {kind:'Number',value:scalar(v)};
 if(typeof v==='string')return {kind:'Text',value:v};
 if(typeof v==='boolean')return {kind:'Boolean',value:v};
 if(v instanceof Date)return {kind:'Date',value:scalar(+v)};
 if(v instanceof colors.color){const space=v instanceof colors.rgb?'Rgb':v instanceof colors.hsl?'Hsl':v instanceof colors.lab?'Lab':v instanceof colors.hcl?'Hcl':'Cubehelix';return {kind:'Color',value:{space,channels:Object.fromEntries(Object.entries(v).map(([k,n])=>[k,scalar(n)]))}};}
 if(ArrayBuffer.isView(v))return {kind:'NumericArray',value:{element:v.constructor.name,values:Array.from(v,scalar)}};
 if(Array.isArray(v))return {kind:'Array',value:v.map(pack)};
 return {kind:'Record',value:Object.fromEntries(Object.entries(v).map(([k,n])=>[k,pack(n)]))};
}
const times=[-.25,0,.25,.5,.75,1,1.25,.1,1,0,.5];
const cases=[];
function add(op,id,args,config={},adaptation=null){
 let expected,duration;
 try{
  let f;
  if(op==='piecewise')f=config.factory?d3.piecewise(d3[config.factory],args[0]):d3.piecewise(args[0]);
  else if(op==='quantize'){expected=pack(d3.quantize(d3[config.factory||'interpolateNumber'](...args),config.count));}
  else {let factory=d3[op];if(config.gamma!==undefined)factory=factory.gamma(config.gamma);if(config.rho!==undefined)factory=factory.rho(config.rho);f=factory(...args);}
  if(f){expected=times.map(t=>pack(f(t)));if('duration'in f)duration=scalar(f.duration);}
 }catch(e){expected={error:e.name};}
 cases.push({id,op,args:args.map(pack),config:Object.fromEntries(Object.entries(config).map(([k,v])=>[k,typeof v==='number'?scalar(v):v])),times:op==='quantize'?[]:times,expected,...(duration===undefined?{}:{duration}),...(adaptation?{adaptation}:{})});
}
for(const op of ['interpolateNumber','interpolateRound','interpolateHue'])for(const [i,args]of [[2,10],[10,-2],[-1,0],[-3,-2],[0,-0],[1e-300,2e-300],[1e308,-1e308],[NaN,10],[10,NaN],[Infinity,1],[-Infinity,Infinity],[350,10],[10,350],[0,180],[180,0],[-720,720]].entries())add(op,op+'-'+i,args);
for(const op of ['interpolateBasis','interpolateBasisClosed','interpolateDiscrete'])for(const [i,values]of [[],[7],[0,12],[0,12,-6,20],[0,NaN,2]].entries())add(op,op+'-'+i,[values],{},values.length<(op==='interpolateBasis'?2:1)?'RejectControlCount':null);
for(const [i,values]of [[],[7],[0,10],[0,10,-5,20]].entries())add('piecewise','piecewise-'+i,[values],{factory:'interpolateNumber'},values.length<2?'RejectControlCount':null);
add('piecewise','piecewise-round',[[0,3,-2]],{factory:'interpolateRound'});
add('piecewise','piecewise-default',[[{x:0,label:'0px'},{x:10,label:'10px'},{x:-5,label:'20px'}]]);
for(const n of [0,1,2,5])add('quantize','quantize-'+n,[2,10],{count:n},n<2?'RejectSampleCount':null);
for(const [i,args]of [['2px','10px'],['translate(1e2,-.5)','move(+2E2,1.5)'],['a1 b2','z3'],['a1','b2 c3'],['no numbers','4px'],['1.0','1.0'],['1','1.0'],['',''],['a-0','b0'],['opacity:0','opacity:'+2**-25],['x0 y1','x1e+21 y1e-8']].entries())add('interpolateString','string-'+i,args);
for(const [i,args]of [[0,100],[-2,1],[.5,2.5],[-8.64e15,8.64e15],[NaN,10]].entries())add('interpolateDate','date-'+i,args.map(n=>new Date(n)));
const structures=[
 [null,true],[9,false],[4,null],[4,undefined],['2',10],[null,10],[true,10],['no number',10],['2px','10px'],['red','blue'],['bad','steelblue'],
 [[0,10,'2px'],[10,30,'6px',true]],[[1,2,3],[3]],[[],[]],[null,[3,4]],
 [{x:0,old:9},{x:10,new:5}],[{nest:{x:0,a:[1,2]}},{nest:{x:8,a:[3,4,9]},flag:true}],[null,{}],[null,{x:1}],
 [new Date(0),new Date(1000)],[colors.hsl(350,1,.5),colors.hsl(10,1,.5)],
];
for(const [i,args]of structures.entries())add('interpolate','dispatch-'+i,args);
const mixed=[undefined,null,true,5,'2px',new Date(100),colors.hsl(40,.5,.5),new Float32Array([2,4]),[2,4],{'0':2,x:4}];
for(const [i,a]of mixed.entries())for(const [j,b]of mixed.entries()){
 const ak=pack(a).kind,bk=pack(b).kind;
 let adaptation=null;
 if(['Number','Date'].includes(bk)&&!['Missing','Null','Boolean','Number','Date','Text'].includes(ak))adaptation='RejectNumericCoercion';
 if(bk==='Text'&&!['Missing','Null','Boolean','Number','Text','Color'].includes(ak))adaptation='RejectTextCoercion';
 if(bk==='Color'&&!['Missing','Null','Text','Color'].includes(ak))adaptation='RejectColorCoercion';
 if(['Array','NumericArray'].includes(bk)&&!['Missing','Null','Array','NumericArray'].includes(ak))adaptation='RejectArrayInput';
 add('interpolate',`mixed-${i}-${j}`,[a,b],{},adaptation);
}
for(const [i,s]of ['','  +1.25e2  ','0xff','0b101','0o17','0x20000000000001','0x20000000000003','0x1000000000000081','-0','Infinity','-Infinity','inf','+0x1','1e','\ufeff12\u00a0','\u008512','0x'+'f'.repeat(270)].entries())add('interpolate','numeric-text-'+i,[s,10]);
for(const op of ['interpolateArray','interpolateObject'])for(const [i,args]of [[[],[]],[[1,2,9],[3,4]],[[1],[3,4]], [null,[3,4]],[{},{}],[{x:0,old:2},{x:10,new:4}],[null,{x:1}]].entries())add(op,op+'-'+i,args,{},op==='interpolateArray'&&i>=4?'RejectArrayInput':null);
for(const name of ['Float32Array','Float64Array','Int8Array','Uint8Array','Uint8ClampedArray','Int16Array','Uint16Array','Int32Array','Uint32Array']){
 const ctor=globalThis[name];
 const args=[[300,-300,1.5,2.5,4294967297],new ctor([0,0,0,0,0,9])];
 add('interpolateNumberArray','typed-'+name,args);
 add('interpolate','typed-dispatch-'+name,args);
}
for(const [i,args]of [[[],[]],[[1,2],[3]],[[1],[3,5]],[null,[2,3]]].entries())add('interpolateNumberArray','number-array-'+i,args);
const colorOps=['interpolateRgb','interpolateHsl','interpolateHslLong','interpolateLab','interpolateHcl','interpolateHclLong','interpolateCubehelix','interpolateCubehelixLong'];
for(const op of colorOps)for(const [i,args]of [['red','blue'],['transparent','steelblue'],['rgba(10,20,30,.25)','hsla(120,50%,20%,.75)'],['bogus','orange'],[colors.hsl(350,1,.5),colors.hsl(10,1,.5)],[colors.lab(50,80,-60),colors.hcl(60,90,80)],[colors.rgb(-20,280,2,.2),colors.rgb(300,0,256,.8)]].entries())add(op,op+'-'+i,args);
for(const op of ['interpolateRgb','interpolateCubehelix','interpolateCubehelixLong'])for(const gamma of [0,.5,1,2.2,-1,NaN,Infinity,-Infinity])add(op,op+'-gamma-'+gamma,['red','blue'],{gamma},gamma<=0||!Number.isFinite(gamma)?'RejectGamma':null);
for(const op of ['interpolateRgbBasis','interpolateRgbBasisClosed'])for(const [i,values]of [[],['red'],['red','blue'],['transparent','rgba(0,255,0,.3)','blue'],['bogus','orange','white']].entries())add(op,op+'-'+i,[values],{},values.length<(op==='interpolateRgbBasis'?2:1)?'RejectControlCount':null);
for(const [i,args]of [[[0,0,10],[0,0,10]],[[0,0,10],[0,0,1]],[[0,0,1],[0,0,10]],[[0,0,1],[10,-5,4]],[[10,-5,4],[0,0,1]],[[0,0,1],[1e-7,-1e-7,2]],[[0,0,1],[1e-6,0,2]],[[0,0,1e-4],[0,0,1e4]],[[0,0,0],[1,2,3]]].entries())add('interpolateZoom','zoom-'+i,args,{},i===8?'RejectZoomView':null);
for(const rho of [0,.5,1,2,4,NaN,Infinity,-Infinity])add('interpolateZoom','zoom-rho-'+rho,[[0,0,1],[4,3,2]],{rho},Number.isFinite(rho)?null:'RejectRho');
writeJson(path.join(out,'cases.json'),{schema_version:1,requirement:'ITP-01–08',cases});
retainLicense('d3-interpolate',out);
writeJson(path.join(out,'manifest.json'),{schema_version:1,...provenance('d3-interpolate'),color:provenance('d3-color'),exports:Object.keys(d3).sort(),configurations:['interpolateRgb.gamma','interpolateCubehelix.gamma','interpolateCubehelixLong.gamma','interpolateZoom.rho','interpolateZoom.duration'],case_count:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'cases.json'))),tolerances:{numeric:{absolute:1e-12,relative:1e-12},strings_integer_date_tags:'exact'},adaptations:{RejectControlCount:'Open basis and piecewise need two controls; closed basis/discrete need one.',RejectSampleCount:'Quantize requires an integer count greater than one.',RejectGamma:'Gamma must be positive finite.',RejectRho:'Rho must be finite; finite values are floored at 1e-3.',RejectArrayInput:'Array operations require array inputs or null/missing empty source.',RejectNumericCoercion:'Structured values have no primitive numeric conversion.',RejectTextCoercion:'Date/structured text requires explicit formatting.',RejectColorCoercion:'Color blending accepts colors, text or missing/null undefined sources.',RejectZoomView:'Zoom centers must be finite and widths positive finite.',owned_samples:'Each sample is captured before the next invocation; D3 mutable output identity is not copied into the typed API.',coercion:'Only explicit bounded primitive numeric conversions; no prototypes or callbacks.'},browser_transform_cases:'transforms.json',status:'Reference only; implementation gates remain open.'});
console.log(`PASS FIX-I01 numeric/value/color/zoom reference: ${cases.length} cases; 27 exports inventoried, transforms use separate real-browser runner.`);
