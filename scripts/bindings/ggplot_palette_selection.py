"""FIX-GG04: actual host palette selection, immutable edits and publication."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
named='--named-palettes' in sys.argv
scalar='--scalar-names' in sys.argv
vectors=named or '--color-vectors' in sys.argv
cases=json.loads((ROOT/'fixtures/parity/ggplot2'/('named-theme-palettes.json' if named else 'theme-palette-values.json' if vectors else 'scale-palette-selection.json')).read_text())['cases']
if vectors:cases=[{**t,'channel':'colour','source':'builtin','theme_mode':'supplied'} for t in cases]
def matching(x,t):return all(x.get(k)==t.get(k) for k in ('lookup_variant','family','channel','source','theme_mode','palette','na_mode'))
def operation(t,mode):return {'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':mode,'channel':t['channel']}}
def descriptor(t):
 channel=t['channel'];discrete=t['family']=='discrete'
 output_palette={'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}} if channel=='colour' else {'operation':'PowerRange','range':[.1,1] if channel=='alpha' else [1,6],'exponent':.5 if channel=='size' else 1,'absolute':False}
 s={'missing_paint_is_na':channel=='colour','training':'Eligible','guide':'Hidden','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':output_palette},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':None,'oob':'Censor'}}}
 if discrete:
  s['function']={'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}}
  s['ggplot']={'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}} if channel=='colour' else {'NumericRange':{'range':[.1,1] if channel=='alpha' else [2,6],'area':channel=='size'}}}}
 if t['family']=='binned':s['ggplot']={'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':5},'right':True}}
 if t['source'] in ('explicit','fallback'):s['palette_function']=operation(t,'full' if t['source']=='explicit' else 'index')
 if t['source']!='explicit':s['palette_theme_aesthetics']=t.get('lookup_aesthetics',[channel])
 return s

def theme(t):
 if vectors:return c.theme().scale_palettes({f"palette.colour.{'discrete' if t['family']=='discrete' else 'continuous'}":t['palette'] if scalar else t['palette_values']})
 return c.theme().scale_palettes({key:operation(t,mode) for key,mode in t.get('theme_palettes',{f"palette.{t['channel']}.{'discrete' if t['family']=='discrete' else 'continuous'}":'first'}).items()}) if t['theme_mode']=='supplied' else c.theme()
def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def data_for(t):
 values=t['inputs'] if t['family']=='discrete' else [float(v) if isinstance(v,str) else v for v in t['inputs']]
 return c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.column(values,kind='string' if t['family']=='discrete' else 'float64').nullable(True)},name='data')
def build(t,data):
 draft=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry)
 if t['theme_mode']=='supplied':draft=draft.theme(theme(t))
 if t['channel']=='colour':
  color=c.color_mapped('v',descriptor(t))
  if t.get('na_mode')=='grey50':color=color.missing('#7F7F7F')
  return draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(color).layer(layer(t)).build()
 return draft.aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def paint(v):return dict(zip(('red','green','blue','alpha'),[int(v[i:i+2],16) for i in (1,3,5)]+[int(v[7:9],16) if len(v)==9 else 255]))
def number(v):return float(v['number']) if isinstance(v,dict) else float(v)
def check(t,chart):
 state=chart.semantics()['layers'][0];styles=state.get('styles',[])
 assert len(styles)==t['result']['point_count'],(t,styles)
 assert [s['color'] for s in styles]==[paint(v) for v in t['result']['point_colours']],t
 if t['channel']=='size':
  wanted=[number(v) for v in t['result']['mapped'] if v is not None];assert len(styles)==len(wanted),t
  for s,v in zip(styles,wanted):assert number(s['radius'])==v or math.isclose(number(s['radius']),v,rel_tol=0,abs_tol=2e-12),(t,s,v)
 return styles
for index,t in enumerate(cases):
 owned=[]
 try:
  data=data_for(t);owned.append(data);p=build(t,data);owned.append(p);wire=p.to_json()
  if t['source']!='explicit' or t['theme_mode']=='supplied':assert json.loads(wire)['version']==(47 if named or (vectors and len(t["palette_values"])==1) else 46 if vectors else 45)
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,'styles':check(t,chart)})
  if t['population']=='ordinary':
   alternate={**t,'palette':('hue' if t['palette']=='viridis' else 'viridis') if named else ('three' if t['palette']=='pair' else 'pair')} if vectors else {**t,'theme_mode':'absent' if t['theme_mode']=='supplied' else 'supplied'}
   expected=next(x for x in cases if matching(x,alternate) and x['population']=='ordinary')
   alternate=expected
   edited=restored.edit().theme(theme(alternate)).build();owned.append(edited);chart=edited.chart();owned.append(chart)
   records.append({'index':index,'state':'theme_edit','styles':check(expected,chart)});assert restored.to_json()==wire
  if (vectors and t['palette'] in (('hue','blues','viridis') if named else ('pair','missing','alpha')) and t['population'] in ('ordinary','missing') and t['na_mode']=='NA') or (not vectors and t['population']=='ordinary' and t['source']=='fallback' and 'lookup_variant' not in t):
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   name=f"{t['family']}-{t['palette']}-{t['population']}" if vectors else f"{t['family']}-{t['channel']}-{t['theme_mode']}"
   for fmt in ('svg','pdf','png'):(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
  if t['population']=='ordinary' and t['source'] in ('builtin','fallback'):
   chart=restored.chart();owned.append(chart);request=output.request(restored,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','empty','ordinary'):
    expected=next(x for x in cases if matching(x,t) and x['population']==population)
    replacement=data_for(expected);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(data,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    batch_plot=build(expected,replacement);owned.append(batch_plot);batch=batch_plot.chart();owned.append(batch)
    styles=check(expected,chart);assert styles==check(expected,batch);assert restored.to_json()==wire and held.scene()==scene
    records.append({'index':index,'state':'replacement','population':population,'styles':styles})
 except c.ChartError:
  if not vectors or 'error' not in t['result']:raise
  records.append({'index':index,'state':'reference_error'})
 else:assert 'error' not in t['result'],t
 finally:
  for obj in reversed(owned):obj.dispose()
for variant in (('theme_version','wire_version','lookup_name','theme_key') if vectors else ('missing_registration','native_only','invalid_parameters','theme_version','wire_version','lookup_name','theme_key')):
 owned=[]
 try:
  t=next(t for t in cases if t['source']==('builtin' if vectors else 'fallback') and t['theme_mode']=='supplied' and t['population']=='ordinary')
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=json.loads(p.to_json());definition=wire['definition'];palettes=definition['theme']['scale_palettes'];key=next(iter(palettes));call=palettes[key]
  if variant=='missing_registration':call['operation']['id']='example.absent_palette'
  if variant=='native_only':call['operation']['id']='example.native_scale_palette'
  if variant=='invalid_parameters':call['parameters']['mode']='invalid'
  if variant=='theme_version':definition['theme']['version']=4 if named else 3 if vectors else 2
  if variant=='wire_version':wire['version']=46 if named else 45 if vectors else 44
  if variant=='theme_key':definition['theme']['scale_palettes']={'palette..continuous':call}
  if variant=='lookup_name':
   def corrupt(value):
    if isinstance(value,dict):
     if 'palette_theme_aesthetics' in value:value['palette_theme_aesthetics']=['']
     for child in value.values():corrupt(child)
    elif isinstance(value,list):
     for child in value:corrupt(child)
   corrupt(definition)
  restored=c.Plot.from_json(json.dumps(wire),registry);owned.append(restored);chart=restored.chart();owned.append(chart);chart.semantics();raise AssertionError(variant)
 except c.ChartError as error:records.append({'rejection':variant,'code':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(4183 if named else 438 if vectors else 979),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));options.dispose();output.dispose();registry.dispose();print('PASS',len(records),'palette selection states and 54 publication files')
