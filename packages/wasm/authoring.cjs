'use strict';
// Syntax and ownership only. Every fluent call immediately invokes a Rust builder.
const rawNative = require('./chart_wasm.js');
class ChartError extends Error {
  constructor(diagnostic) { super(diagnostic.message); this.name='ChartError'; Object.assign(this,diagnostic); this.diagnostic=diagnostic; }
}
function checked(call) {
  try { return call(); } catch (error) {
    let diagnostic;
    try { diagnostic=JSON.parse(error.message); } catch { throw error; }
    if (typeof diagnostic?.code !== 'string') throw error;
    throw new ChartError(diagnostic);
  }
}
const native = new Proxy(rawNative,{get(target,key){
  const Type=target[key];
  if(typeof key!=='string'||!key.startsWith('_')||typeof Type!=='function')return Type;
  return new Proxy(Type,{
    construct(T,args){return checked(()=>Reflect.construct(T,args));},
    get(T,key){const value=T[key];return typeof value==='function'?(...args)=>checked(()=>value.apply(T,args)):value;}
  });
}});
function encode(value) {
  return JSON.stringify(value, (_, v) => {
    if (typeof v === 'bigint') return v.toString();
    if (typeof v === 'number' && (!Number.isFinite(v) || (Number.isInteger(v) && !Number.isSafeInteger(v)))) throw new RangeError('Scalar options require finite, safe Numbers or exact BigInt.');
    return v;
  });
}
const decode = JSON.parse;
function integer(value, signed = false) {
  if (typeof value === 'number' && !Number.isSafeInteger(value)) throw new RangeError('Exact integers require safe Numbers or BigInt.');
  if (typeof value !== 'number' && typeof value !== 'bigint') throw new TypeError('Expected an exact integer.');
  const n = BigInt(value), min = signed ? -(1n << 63n) : 0n, max = signed ? (1n << 63n)-1n : (1n << 64n)-1n;
  if (n < min || n > max) throw new RangeError('Integer exceeds the declared 64-bit kind.');
  return n;
}
function integers(values, signed = false) { const checked = Array.from(values, v => integer(v,signed)); return signed ? new BigInt64Array(checked) : new BigUint64Array(checked); }
function count(value) {
  if(typeof value!=='number'||!Number.isInteger(value)||value<0||value>0xffffffff)throw new RangeError('Count requires an unsigned 32-bit integer Number.');
  return value;
}
function bytes(values) {
  const input=Array.from(values);
  if(input.some(v=>typeof v!=='number'||!Number.isInteger(v)||v<0||v>255))throw new RangeError('Byte payload requires integers in 0..255.');
  return Uint8Array.from(input);
}
class Owned {
  constructor(inner) { this._inner=new Proxy(inner,{get(target,key){const value=Reflect.get(target,key,target);return typeof value==='function'?(...args)=>checked(()=>value.apply(target,args)):value;}}); }
  dispose() { this._inner.dispose(); }
  free() { this._inner.free(); }
}
class Field extends Owned {}
class Column extends Owned {
  nullable(v=true) {if(typeof v!=='boolean')throw new TypeError('Nullable requires a boolean.');return new Column(this._inner.nullable(v));}
  validity(v) {if(Array.from(v).some(x=>typeof x!=='boolean'))throw new TypeError('Validity requires booleans.'); return new Column(this._inner.validity(Uint8Array.from(v,Number)));}
  formatted(v) {const values=Array.from(v); if(values.some(x=>x!==null&&typeof x!=='string'))throw new TypeError('Formatted values require strings or null.');return new Column(this._inner.formatted(values.map(x=>x??''),Uint8Array.from(values,x=>Number(x!==null))));}
  unit(v) {return new Column(this._inner.unit(v));}
  label(v) {return new Column(this._inner.label(v));}
}
function column(input,{kind,timezone='UTC'}={}) {
  const floating = input instanceof Float64Array || input instanceof Float32Array;
  const values=Array.from(input), present=values.filter(v=>v!==null), types=new Set(present.map(v=>typeof v));
  if(values.some(v=>v===undefined)) throw new TypeError('Use null for missing source values.');
  if(!kind) {
    if(floating)kind='float64';
    else if(input instanceof BigInt64Array)kind='int64';
    else if(input instanceof BigUint64Array)kind='uint64';
    else if(types.size===1&&types.has('boolean'))kind='bool';
    else if(types.size===1&&types.has('string'))kind='string';
    else if(types.size===1&&types.has('bigint'))kind=present.some(v=>v>(1n<<63n)-1n)&&present.every(v=>v>=0n)?'uint64':'int64';
    else if(types.size===1&&types.has('number'))kind=present.every(Number.isInteger)?'int64':'float64';
    else throw new TypeError('Empty, all-null or mixed columns require an explicit kind.');
  }
  let inner;
  if(kind==='float64') {
    if(present.some(v=>typeof v!=='number'||(!floating&&Number.isInteger(v)&&!Number.isSafeInteger(v))))throw new RangeError('Float64 accepts Numbers; wide exact integers require BigInt columns.');
    inner=native._Column.float64(Float64Array.from(values,v=>v??0));
  } else if(['int64','uint64','s','ms','us','ns'].includes(kind)) {
    const payload=integers(values.map(v=>v??0n),kind!=='uint64');
    inner=kind==='uint64'?native._Column.uint64(payload):kind==='int64'?native._Column.int64(payload):native._Column.timestamp(payload,kind,timezone);
  } else if(kind==='bool') {
    if(present.some(v=>typeof v!=='boolean'))throw new TypeError('Boolean column requires booleans.');
    inner=native._Column.boolean(Uint8Array.from(values,v=>Number(v??false)));
  } else if(kind==='string'||kind==='category') {
    if(present.some(v=>typeof v!=='string'))throw new TypeError('Text column requires strings.');
    inner=kind==='string'?native._Column.strings(values.map(v=>v??'')):native._Column.categorical(values.map(v=>v??''));
  } else throw new TypeError(`Unknown source column kind ${kind}`);
  if(present.length!==values.length) {const next=inner.validity(Uint8Array.from(values,v=>Number(v!==null)));inner.free();inner=next;}
  return new Column(inner);
}
const categorical=values=>column(values,{kind:'category'});
const timestamps=(values,unit='ns',timezone='UTC')=>column(values,{kind:unit,timezone});
class Data extends Owned {
  static columns(columns,{name='data',keys,limits,schemaVersion,identity}={}) {
    let b=new native._Columns();
    const update=next=>{b.free();b=next;};
    try {
      update(b.name(name));
      if(identity!==undefined)update(b.identity(integer(identity)));
      if(limits!==undefined)update(b.limits(encode(limits)));
      if(schemaVersion!==undefined)update(b.schema_version(integer(schemaVersion)));
      for(const [name,values] of Object.entries(columns)) {const c=values instanceof Column?values:column(values);try{update(b.column(name,c._inner));}finally{if(!(values instanceof Column))c.free();}}
      if(keys!==undefined)update(b.keys(integers(keys)));
      return new Data(b.build());
    } finally {b.free();}
  }
  static rows(rows,{fields,name='data',keys,...options}={}) {
    rows=Array.from(rows);let columns={};
    if(fields) {for(const [name,accessor] of Object.entries(fields))columns[name]=rows.map(accessor);}
    else {if(!rows.length)throw new TypeError('Empty rows require explicit fields.');const names=Object.keys(rows[0]);if(rows.some(r=>Object.keys(r).length!==names.length||names.some(n=>!Object.hasOwn(r,n))))throw new TypeError('Rows require identical named fields.');for(const n of names)columns[n]=rows.map(r=>r[n]);}
    return Data.columns(columns,{name,keys:typeof keys==='function'?rows.map(keys):keys,...options});
  }
  field(name){return new Field(this._inner.field(name));}
  get name(){return this._inner.name();}
}
function fluent(object, apply) {return new Proxy(object,{get(target,key,receiver){if(key==='then')return undefined;if(typeof key!=='string'||key in target||key.startsWith('_'))return Reflect.get(target,key,receiver);return (...args)=>apply(target,key.replace(/[A-Z]/g,c=>'_'+c.toLowerCase()),args);}});}
class Component extends Owned {
  constructor(inner){super(inner);return fluent(this,(t,name,args)=>{
    let next;
    if(args.length===1&&args[0] instanceof Component)next=t._inner.with_component(name,args[0]._inner);
    else if(args.length===1&&args[0] instanceof Field)next=t._inner.field(name,args[0]._inner);
    else if(name==='field_parameter'&&args.length===2&&args[1] instanceof Field)next=t._inner.field_parameter(args[0],args[1]._inner);
    else if(name==='data'&&args[0] instanceof Data)next=t._inner.data(args[0]._inner);
    else if(name==='layer'&&args.length===2)next=t._inner.theme_layer(...args.map(v=>v._inner));
    else if(name==='candle_volume')next=t._inner.candle_volume(...args.map(v=>v._inner));
    else if(name==='axis'&&args.length===2)next=t._inner.link_axis(...args.map(v=>v._inner));
    else next=t._inner.set(name,encode(args));
    return new t.constructor(next);
  });}
}
class PlotBuilder extends Owned {
  constructor(inner){super(inner);return fluent(this,(t,name,args)=>{
    let next;
    if(args.length===1&&args[0] instanceof Component)next=t._inner.with_component(name,args[0]._inner);
    else if(name==='data'&&args[0] instanceof Data)next=t._inner.dataset(args[0]._inner);
    else if(name==='layer'&&args.length===2)next=t._inner.layer(args[0],args[1]._inner);
    else next=t._inner.set(name,encode(args));
    return new t.constructor(next);
  });}
  build(){return new Plot(this._inner.build());}
}
class PlotEdit extends PlotBuilder {}
const plot=data=>new PlotBuilder(new native._Draft(data._inner));
class Plot extends Owned {
  edit(){return new PlotEdit(this._inner.edit());}
  chart(){return new Chart(this);}
  to_json(){return this._inner.to_json();}
  static from_json(value){return new Plot(native._Plot.from_json(value));}
  static _from_example_json(value){return new Plot(native._Plot.from_json_with_example_extensions(value));}
}
class ExportOptions extends Owned {
  constructor(inner){super(inner);return fluent(this,(t,name,args)=>new ExportOptions(name==='layout'?t._inner.layout(args[0]._inner):t._inner.set(name,encode(args))));}
}
const export_options=(width,height,unit='pt')=>new ExportOptions(new native._ExportOptions(width,height,unit));
class FigureRequest extends Owned {
  prepare(){return new FigureSnapshot(this._inner.prepare());}
  manifest(){return decode(this._inner.manifest());}
}
class FigureSnapshot extends Owned {
  scene(){return decode(this._inner.scene());}
  manifest(){return decode(this._inner.manifest());}
  export(format){return this._inner.export(format);}
}
class Output extends Owned {
  constructor(font){super(new native._Output(bytes(font)));}
  primary_font(){return decode(this._inner.primary_font());}
  register_font(font){return decode(this._inner.register_font(bytes(font)));}
  request(source,options){return source instanceof Chart?source.request(this,options):new FigureRequest(this._inner.request(source._inner,options._inner));}
  export(source,format,options){const request=this.request(source,options);try{const frame=request.prepare();try{return frame.export(format);}finally{frame.free();}}finally{request.free();}}
}
class ExportJob extends Owned {cancel(){return this._inner.cancel();}run(){return this._inner.run();}}
class ExportQueue extends Owned {
  constructor({maxJobs=2,maxInputBytes=536870912,maxRows=2000000}={}){super(new native._ExportQueue(count(maxJobs),count(maxInputBytes),count(maxRows)));}
  submit(request,format){return new ExportJob(this._inner.submit(request._inner,format));}
  metrics(){return decode(this._inner.metrics());}
}
const dataset=v=>v instanceof Data?v.name:v;
class Transaction extends Owned {}
class Updates extends Owned {
  id(v){return new Updates(this._inner.id(v));}
  append(target,data){return new Updates(this._inner.data('append',dataset(target),data._inner));}
  upsert(target,data){return new Updates(this._inner.data('upsert',dataset(target),data._inner));}
  replace(target,data){return new Updates(this._inner.data('replace',dataset(target),data._inner));}
  remove(target,keys){return new Updates(this._inner.remove(dataset(target),integers(keys)));}
  retain_count(target,value){return new Updates(this._inner.retain_count(dataset(target),value==null?undefined:count(value)));}
  retention(target,policy){return new Updates(this._inner.retention(dataset(target),encode(policy)));}
  retain_event_time(target,field,width,watermark,{allowedLateness=0n,late='Reject'}={}){return new Updates(this._inner.retain_event_time(dataset(target),field,encode({width:integer(width,true),watermark:integer(watermark,true),allowed_lateness:integer(allowedLateness,true),late})));}
  watermark(target,ticks){return new Updates(this._inner.watermark(dataset(target),integer(ticks,true)));}
  reset_categories(target,field){return new Updates(this._inner.reset_categories(dataset(target),field));}
  build(){return new Transaction(this._inner.build());}
}
class Editor extends Owned {
  original(){return decode(this._inner.original());}
  preview(dx,dy){return decode(this._inner.preview(dx,dy));}
  nudge({horizontal=true,forward=true,steps=1}={}){return decode(this._inner.nudge(horizontal,forward,count(steps)));}
}
class Chart extends Owned {
  constructor(plot,inner){super(inner ?? new native._Runtime(plot._inner));}
  external_view(){return new Chart(null,this._inner.external_view());}
  accept_from(source){return decode(this._inner.accept_from(source._inner));}
  _command(name,options={},expected){return decode(this._inner.command(name,encode(options),expected===undefined?undefined:integer(expected)));}
  _named_query(name,options,{gesture=false,stamp}={}){return decode(this._inner.named_query(name,encode(options),gesture,stamp===undefined?undefined:encode(stamp)));}
  layer_visible(layer,visible,{expected}={}){return this._command('layer_visible',{layer,visible},expected);}
  legend_visible(visible,{expected}={}){return this._command('legend_visible',{visible},expected);}
  follow(mode='FollowLatest',{expected}={}){return this._command('follow',{mode},expected);}
  freeze(options){return this.follow('FreezePresentation',options);}
  resume({expected}={}){return this._command('resume',{},expected);}
  reset({expected}={}){return this._command('reset',{},expected);}
  undo({expected}={}){return this._command('undo',{},expected);}
  redo({expected}={}){return this._command('redo',{},expected);}
  clear_inspection({expected}={}){return this._command('clear_inspection',{},expected);}
  select(targets,change='Replace',{expected}={}){return this._command('select',{targets,change},expected);}
  hover(targets,{expected}={}){return this._command('hover',{targets},expected);}
  focus(target=null,{expected}={}){return this._command('focus',{target},expected);}
  pin(target=null,{expected}={}){return this._command('pin',{target},expected);}
  set_annotation(annotation,{expected}={}){return this._command('annotation',{annotation},expected);}
  remove_annotation(id,{expected}={}){return this._command('remove_annotation',{id},expected);}
  set_windows(windows,{expected}={}){return this._command('windows',{windows},expected);}
  editor(options){return new Editor(this._inner.editor(options._inner));}
  describe({offset=0,limit=64,...fences}={}){return this.query({Describe:{offset,limit}},fences);}
  inspect(x,y,{radius=10,maxGrouped=32,mode='Auto',...fences}={}){return this.query({Inspect:{point:[x,y],radius,max_grouped:maxGrouped,mode}},fences);}
  select_region(region,{limit=4096,...fences}={}){return this.query({Select:{region,limit}},fences);}
  select_series(layer,{panel=null,limit=4096,...fences}={}){return this._named_query('Series',{layer,panel,limit},fences);}
  navigate(action,{axes=['x','y'],panel=null,boundary='ClampToDomain',...fences}={}){return this._named_query('Navigate',{axes,panel,action,boundary},fences);}
  zoom(x,y,factor,options){return this.navigate({Zoom:{anchor:[x,y],factor}},options);}
  pan(dx,dy,options){return this.navigate({Pan:{dx,dy}},options);}
  range(axis,window,{panel=null,...fences}={}){return this._named_query('SetRange',{axis,window,panel},fences);}
  revisions(){return Object.fromEntries(Object.entries(decode(this._inner.revisions())).map(([k,v])=>[k,BigInt(v)]));}
  semantics(){return decode(this._inner.semantics());}
  state(){return decode(this._inner.state());}
  apply_plot(plot,expected){return this._inner.apply_plot(plot._inner,integer(expected));}
  restore_state(state,expected){this._inner.restore_state(encode(state),integer(expected));}
  act(action,{origin='Programmatic',expected}={}){return decode(this._inner.act(encode(action),encode(origin),expected===undefined?undefined:integer(expected)));}
  query(operation,{gesture=false,stamp}={}){return decode(this._inner.query(encode(operation),gesture,stamp===undefined?undefined:encode(stamp)));}
  request(output,options){return new FigureRequest(this._inner.request(output._inner,options._inner));}
  present(output,options){return new FigureSnapshot(this._inner.present(output._inner,options._inner));}
  stream(options){this._inner.stream(options._inner);return this;}
  stream_status(){return decode(this._inner.stream_status());}
  pinned(){return decode(this._inner.pinned());}
  queue_status(){return decode(this._inner.queue_status());}
  commit_next(){return decode(this._inner.commit_next());}
  reset_epoch(){return decode(this._inner.reset_epoch());}
  transaction(){return new Updates(this._inner.transaction());}
  commit(transaction){return decode(this._inner.commit(transaction._inner));}
  enqueue(transaction){return decode(this._inner.enqueue(transaction._inner));}
  dense(frame,options){return decode(this._inner.dense(frame._inner,options._inner));}
  link_capture(component,event){return decode(this._inner.link_capture(component._inner,encode(event)));}
  link_resolve(component,message){return decode(this._inner.link_resolve(component._inner,encode(message)));}
}
const families = {"Aes": "aes", "Layer": "points line area ribbon bars volume ohlc rule rectangle cells histogram", "Stat": "identity_stat bin count summary fit custom_stat", "StatAes": "stat_aes", "BinAes": "bin_aes", "Position": "stack dodge jitter", "Filter": "filter", "Transform": "transform", "Scale": "scale_linear scale_log scale_symlog scale_band scale_point scale_utc scale_session", "Axis": "x_axis y_axis", "ColorScale": "color_discrete color_continuous", "Legend": "legend", "Facet": "facet_wrap facet_grid", "Style": "style", "Theme": "theme", "TextStyle": "text_style", "TextRun": "text_run", "RichText": "rich_text", "Title": "title", "Subtitle": "subtitle", "Caption": "caption", "SourceNote": "source_note", "Footnote": "footnote", "Labels": "labels", "Callout": "callout", "PanelLetter": "panel_letter", "Inset": "inset", "NumberFormat": "number_format", "LayoutOptions": "layout_options", "RenderOptions": "render_options", "StreamOptions": "stream_options", "AnnotationEdit": "annotation_edit", "Link": "link"};
module.exports={ChartError,LegacyChart:native.Chart,Editor,Column,Data,Field,Component,PlotBuilder,PlotEdit,Plot,Chart,Output,ExportOptions,FigureRequest,FigureSnapshot,ExportQueue,ExportJob,Updates,Transaction,column,categorical,timestamps,plot,export_options};
for(const [family,names] of Object.entries(families)) {
  const Type=class extends Component {};
  Object.defineProperty(Type,'name',{value:family});
  module.exports[family]=Type;
  for(const name of names.split(' '))module.exports[name]=name==='transform'?(name,stat)=>new Type(native._Component.transform(name,stat._inner)):(...args)=>new Type(new native._Component(name,encode(args)));
}

// Host-native camelCase aliases retain the documented snake_case compatibility spellings.
const camel = name => name.replace(/_([a-z])/g,(_,c)=>c.toUpperCase());
for(const [name,value] of Object.entries(module.exports)) {
  if(name.includes('_'))module.exports[camel(name)]=value;
  if(typeof value==='function'&&value.prototype) {
    for(const method of Object.getOwnPropertyNames(value)) {
      if(method.includes('_')&&!method.startsWith('_'))Object.defineProperty(value,camel(method),Object.getOwnPropertyDescriptor(value,method));
    }
    for(const method of Object.getOwnPropertyNames(value.prototype)) {
      if(method.includes('_')&&!method.startsWith('_'))Object.defineProperty(value.prototype,camel(method),Object.getOwnPropertyDescriptor(value.prototype,method));
    }
  }
}
