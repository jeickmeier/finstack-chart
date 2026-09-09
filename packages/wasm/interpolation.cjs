'use strict';
// Wire classification and host ownership only; all interpolation executes in Rust.
module.exports=function(native,Owned,ColorValue,count){
  const utf8=new TextEncoder();
  const constructors={Float32Array,Float64Array,Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array};
  const number=v=>{if(typeof v!=='number')throw new TypeError('Interpolation numbers require Number.');return Number.isNaN(v)?{number:'NaN'}:v===Infinity?{number:'Infinity'}:v===-Infinity?{number:'-Infinity'}:Object.is(v,-0)?{number:'-0'}:v;};
  const unnumber=v=>typeof v==='object'?({'NaN':NaN,'Infinity':Infinity,'-Infinity':-Infinity,'-0':-0})[v.number]:v;
  function pack(v,depth=0,budget={nodes:0,bytes:0}){
    if(depth>32||++budget.nodes>200000)throw new RangeError('Interpolation nesting/node budget exceeded.');
    if(v===undefined)return {kind:'Missing'};
    if(v===null)return {kind:'Null'};
    if(typeof v==='boolean')return {kind:'Boolean',value:v};
    if(typeof v==='number')return {kind:'Number',value:number(v)};
    if(typeof v==='string'){budget.bytes+=utf8.encode(v).length;if(budget.bytes>4*1024*1024)throw new RangeError('Interpolation text budget exceeded.');return {kind:'Text',value:v};}
    if(v instanceof ColorValue)return {kind:'Color',value:v.value()};
    if(v instanceof Date)return {kind:'Date',value:number(v.getTime())};
    if(ArrayBuffer.isView(v)){
      const element=v.constructor.name;
      if(!Object.hasOwn(constructors,element)||!(v instanceof constructors[element]))throw new TypeError('Only Number-based typed arrays are supported.');
      budget.nodes+=v.length;if(budget.nodes>200000)throw new RangeError('Interpolation node budget exceeded.');
      return {kind:'NumericArray',value:{element,values:Array.from(v,number)}};
    }
    if(Array.isArray(v))return {kind:'Array',value:Array.from(v,x=>pack(x,depth+1,budget))};
    if(typeof v==='object'&&(Object.getPrototypeOf(v)===Object.prototype||Object.getPrototypeOf(v)===null)){
      budget.bytes+=Object.keys(v).reduce((n,k)=>n+utf8.encode(k).length,0);if(budget.bytes>4*1024*1024)throw new RangeError('Interpolation record key budget exceeded.');
      return {kind:'Record',value:Object.fromEntries(Object.entries(v).map(([k,v])=>[k,pack(v,depth+1,budget)]))};
    }
    throw new TypeError('Unsupported interpolation value; use typed values without callbacks or BigInt.');
  }
  function unpack(v){switch(v.kind){
    case'Missing':return undefined;case'Null':return null;case'Boolean':case'Text':return v.value;case'Number':return unnumber(v.value);
    case'Date':return new Date(unnumber(v.value));
    case'Color':return ColorValue.from_json(JSON.stringify({version:1,value:v.value}));
    case'NumericArray':return new constructors[v.value.element](v.value.values.map(unnumber));
    case'Array':return v.value.map(unpack);
    case'Record':return Object.fromEntries(Object.entries(v.value).map(([k,v])=>[k,unpack(v)]));
    default:throw new TypeError('Unknown core interpolation value kind.');
  }}
  class Interpolator extends Owned{
    static from_json(v){return new Interpolator(native._Interpolator.from_json(v));}
    to_json(){return this._inner.to_json();}
    copy(){return new Interpolator(this._inner.copy());}
    sample(t){if(typeof t!=='number')throw new TypeError('Interpolation parameter requires Number.');return unpack(JSON.parse(this._inner.sample_json(t)));}
    sample_value(t){if(typeof t!=='number')throw new TypeError('Interpolation parameter requires Number.');return JSON.parse(this._inner.sample_json(t));}
    sample_color(t){if(typeof t!=='number')throw new TypeError('Interpolation parameter requires Number.');return new ColorValue(this._inner.sample_color(t));}
    sample_transform(t){if(typeof t!=='number')throw new TypeError('Interpolation parameter requires Number.');return JSON.parse(this._inner.sample_transform_json(t));}
    quantize(n){return unpack(JSON.parse(this._inner.quantize_json(count(n))));}
    get duration(){return this._inner.duration_ms();}
    get scheduling_duration(){return this._inner.scheduling_duration_ms();}
  }
  const metadata=new WeakMap();
  function factory(name,options={}){
    const f=(...args)=>{const budget={nodes:0,bytes:0};return new Interpolator(new native._Interpolator(name,JSON.stringify(args.map(v=>pack(v,0,budget))),JSON.stringify(options)));};
    metadata.set(f,{name,options});
    if(['interpolateRgb','interpolateCubehelix','interpolateCubehelixLong'].includes(name))f.gamma=value=>{native._Interpolator.factory(name,value);return factory(name,{gamma:number(value)});};
    if(name==='interpolateZoom')f.rho=value=>factory(name,{rho:number(value)});
    return f;
  }
  const exports={Interpolator,MISSING:undefined,
    date_value:ms=>{number(ms);return unpack(JSON.parse(native._Interpolator.date_value(ms)));},
    numeric_array:(kind,values)=>unpack(JSON.parse(native._Interpolator.normalize_value(JSON.stringify({kind:'NumericArray',value:{element:kind,values:Array.from(values,number)}})))),
    quantize:(f,n)=>{if(!(f instanceof Interpolator))throw new TypeError('quantize requires an owned Interpolator.');return f.quantize(n);},
    piecewise:(...args)=>{
      let f,values;if(args.length===1){f=exports.interpolate;values=args[0];}else if(args.length===2){[f,values]=args;}else throw new TypeError('piecewise expects values or a built-in factory and values.');
      const meta=metadata.get(f);if(!meta||meta.options.rho!==undefined)throw new TypeError('Portable piecewise needs a registered built-in factory.');
      const descriptor=JSON.parse(native._Interpolator.factory(meta.name,meta.options.gamma===undefined?undefined:unnumber(meta.options.gamma)));
      return factory('piecewise',{factory:descriptor})(values);
    },
  };
  for(const name of ['interpolate','interpolateArray','interpolateBasis','interpolateBasisClosed','interpolateDate','interpolateDiscrete','interpolateHue','interpolateNumber','interpolateNumberArray','interpolateObject','interpolateRound','interpolateString','interpolateTransformCss','interpolateTransformSvg','interpolateZoom','interpolateRgb','interpolateRgbBasis','interpolateRgbBasisClosed','interpolateHsl','interpolateHslLong','interpolateLab','interpolateHcl','interpolateHclLong','interpolateCubehelix','interpolateCubehelixLong'])exports[name.replace(/[A-Z]/g,c=>'_'+c.toLowerCase())]=factory(name);
  Object.defineProperty(exports,'_valueCodec',{value:{pack,unpack,number,unnumber,descriptor:f=>{const m=metadata.get(f);if(!m)throw new TypeError('A registered interpolation factory is required.');return JSON.parse(native._Interpolator.factory(m.name,m.options.gamma===undefined?undefined:unnumber(m.options.gamma)));}}});
  exports.chromatic_catalog=()=>JSON.parse(native._Interpolator.chromatic_catalog());
  exports.chromatic_scheme=(name,size=null,reverse=false)=>{if(typeof name!=='string'||typeof reverse!=='boolean'||(size!==null&&(!Number.isSafeInteger(size)||size<0)))throw new TypeError('Chromatic scheme requires a name, optional nonnegative integer size and boolean reversal.');return unpack(JSON.parse(native._Interpolator.chromatic_scheme(JSON.stringify({version:1,id:name,size,reverse}))));};
  exports.chromatic=(name,reverse=false)=>{if(typeof name!=='string'||typeof reverse!=='boolean')throw new TypeError('Chromatic interpolator requires a name and boolean reversal.');return new Interpolator(native._Interpolator.chromatic(name,reverse));};
  return exports;
};
