'use strict';
// Host syntax and exact value transport only. All scale behavior executes in Rust.
module.exports=function(native,Owned,codec,Interpolator){
  const {pack,unpack,number,unnumber}=codec;
  class ScaleKey {
    constructor(kind,value=null){this.kind=kind;this.value=value;Object.freeze(this);}
  }
  function key(v){
    if(v instanceof ScaleKey)return v.kind==='Null'?'Null':{[v.kind]:['Integer','Unsigned','Timestamp'].includes(v.kind)?exact(v.value):v.kind==='Number'?number(v.value):v.value};
    if(v===null)return 'Null';
    switch(typeof v){case'boolean':return {Boolean:v};case'number':return {Number:number(v)};case'bigint':return {Integer:String(v)};case'string':return {Text:v};default:throw new TypeError('Categories require null, boolean, Number, BigInt, string or ScaleKey.');}
  }
  function unkey(v){if(v==='Null')return null;const [kind,payload]=Object.entries(v)[0];return kind==='Number'?unnumber(payload):kind==='Integer'?BigInt(payload):['Unsigned','Timestamp'].includes(kind)?new ScaleKey(kind,BigInt(payload)):payload;}
  function exact(v){if(typeof v!=='bigint')throw new TypeError('Time and exact integer keys require BigInt.');return String(v);}
  function input(v,mode){return v===undefined||v===null&&mode!=='Key'?'Missing':{[mode]:mode==='Key'?key(v):mode==='Time'?exact(v):number(v)};}
  function uninput(v){if(v==='Missing')return undefined;const [kind,payload]=Object.entries(v)[0];return kind==='Value'?unpack(payload):kind==='Key'?unkey(payload):kind==='Time'?BigInt(payload):unnumber(payload);}
  function options(o,mode){
    const r={...o};
    if(Object.hasOwn(r,'domain'))r.domain=Array.from(r.domain,v=>input(v,mode));
    if(Object.hasOwn(r,'range'))r.range=Array.from(r.range,v=>pack(v));
    if(Object.hasOwn(r,'unknown'))r.unknown=pack(r.unknown);
    if(Object.hasOwn(r,'interpolator')){if(!(r.interpolator instanceof Interpolator))throw new TypeError('Scale interpolator requires an owned Interpolator.');r.interpolator=JSON.parse(r.interpolator.to_json()).spec;}
    if(typeof r.factory==='function')r.factory=codec.descriptor(r.factory);
    return r;
  }
  const selection=(count,interval)=>interval===undefined?{Count:number(count)}:{Interval:interval};
  class StandaloneScale extends Owned {
    constructor(family='linear',config={}){
      const mode=['ordinal','band','point','threshold'].includes(family)?'Key':['utc','local'].includes(family)?'Time':'Number';
      super(native._Scale.create(JSON.stringify(family),JSON.stringify(options(config,mode))));this._readMode();
    }
    _readMode(){const family=Object.keys(this.spec())[0];this._mode=['Ordinal','Band','Point','Threshold'].includes(family)?'Key':family==='Time'?'Time':'Number';}
    static _wrap(inner){const s=new Owned(inner);Object.setPrototypeOf(s,StandaloneScale.prototype);s._readMode();return s;}
    static from_json(text){return StandaloneScale._wrap(native._Scale.from_json(text));}
    static from_spec(spec){return StandaloneScale._wrap(new native._Scale(JSON.stringify(spec)));}
    to_json(){return this._inner.to_json();}
    copy(){return StandaloneScale._wrap(this._inner.copy());}
    _query(value){return JSON.parse(this._inner.query(JSON.stringify(value)));}
    _change(value){return StandaloneScale._wrap(this._inner.change(JSON.stringify(value)));}
    spec(){return this._query('Spec');}
    mapped(training='Authored'){return this._query({Mapped:training});}
    configure(config){return this._change({Configure:options(config,this._mode)});}
    reconfigure(spec){return this._change({Reconfigure:spec});}
    domain(){return this._query('Domain').map(uninput);}
    range(){return this._query('Range').map(unpack);}
    map(value){return unpack(this._query({Map:input(value,this._mode)}));}
    map_value(value){return this._query({Map:input(value,this._mode)});}
    invert(value){return uninput(this._query({Invert:number(value)}));}
    invert_extent(value){const r=this._query({InvertExtent:pack(value)});return {found:r.found,lower:r.lower===null?undefined:unkey(r.lower),upper:r.upper===null?undefined:unkey(r.upper)};}
    ticks(count=10,{interval,budget=10000}={}){
      if(this._mode==='Time')return this._query({TimeTicks:{selection:selection(count,interval),budget}}).map(uninput);
      if(interval!==undefined)throw new TypeError('Calendar intervals require a time scale.');
      return this._query({Ticks:{count:number(count),budget}}).map(unnumber);
    }
    format(value,{count=10,specifier=null,pattern=null,locale={}}={}){
      if(this._mode==='Time'){if(specifier!==null)throw new TypeError('Time scales use pattern, not numeric specifier.');return this._query({TimeFormat:{value:exact(value),format:{pattern,locale}}});}
      if(pattern!==null)throw new TypeError('Numeric scales use specifier, not time pattern.');
      return this._query({Format:{value:number(value),count:number(count),specifier,locale}});
    }
    nice(count=10,{interval}={}){if(this._mode==='Time')return this._change({NiceTime:selection(count,interval)});if(interval!==undefined)throw new TypeError('Calendar intervals require a time scale.');return this._change({Nice:number(count)});}
    train(values){return this._change({Train:Array.from(values,key)});}
    thresholds(){return this._query('Thresholds').map(v=>v===null?undefined:unnumber(v));}
    quantiles(count){return this._query({Quantiles:number(count)}).map(v=>v===null?undefined:unnumber(v));}
    step(){return unnumber(this._query('Step'));}
    bandwidth(){return unnumber(this._query('Bandwidth'));}
    extent(value){const r=this._query({Extent:key(value)});return r===null?undefined:[r.start,r.end];}
    center(value){const r=this._query({Center:key(value)});return r===null?undefined:unnumber(r);}
    _calendar(method,value,interval,extra={}){return uninput(this._query({[method]:{value:exact(value),interval,...extra}}));}
    floor(value,interval){return this._calendar('Floor',value,interval);}
    ceil(value,interval){return this._calendar('Ceil',value,interval);}
    round_time(value,interval){return this._calendar('RoundTime',value,interval);}
    offset(value,interval,steps=1){return this._calendar('Offset',value,interval,{steps:number(steps)});}
  }
  return {ScaleKey,StandaloneScale};
};
