"""Actual default-constructor theme selection, immutable edits and publication."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
temporal='--temporal' in sys.argv
ordinal='--ordinal' in sys.argv
numeric='--numeric-constructors' in sys.argv
cases=json.loads((ROOT/'fixtures/parity/ggplot2'/('numeric-paint-constructors.json' if numeric else 'default-ordinal-theme-palettes.json' if ordinal else 'default-temporal-theme-palettes.json' if temporal else 'default-palette-selection.json')).read_text())['cases'];records=[]
if numeric:cases=[{**t,'route':'area' if t['constructor'].endswith('_area') else 'radius' if t['constructor']=='radius' else 'constructor','family':'continuous','theme_mode':'absent'} for t in cases]
if ordinal:cases=[{**t,'route':'ordinal','family':'discrete'} for t in cases]
if temporal:cases=[{**t,'route':'automatic','family':'continuous'} for t in cases]
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def theme(t):
 channel=t['channel'];kind='colour' if channel in ('colour','fill') else 'size' if channel=='linewidth' else channel
 return c.theme().scale_palettes({f"palette.{channel}.{t['family']}":{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':'constant','channel':kind}}}) if t['theme_mode']=='supplied' else c.theme()
def descriptor(t):
 channel=t['channel'];route=t['route'];discrete=t['family']=='discrete';area=route=='area'
 s={'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Maximum' if area else 'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[.2,.8] if route=='range' else [0,6] if area else [.1,1] if channel=='alpha' else [1,6],'exponent':.5 if channel=='size' and route!='radius' else 1,'absolute':area}},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':None,'oob':'Censor'}}}
 if channel in ('colour','fill'):s['function']['Interpolated']['output']={'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}}
 if discrete:
  s['function']={'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}}
  s['ggplot']={'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}} if channel in ('colour','fill') else {'NumericRange':{'range':[.2,.8] if route=='range' else [.1,1] if channel=='alpha' else [2,6],'area':channel=='size'}}}}
 if ordinal:
  s['ggplot']['Discrete']['palette']={'Viridis':{'option':'Viridis','begin':0,'end':1,'reverse':False,'alpha':1}}
  s['guide']='Hidden'
  s['missing_paint_is_na']=True
 if numeric:
  s['guide']='Hidden'
  if t['configuration']!='default':s['function']['Interpolated']['output']['Interpolate']['range']=[0,t['args']['max_size']] if area else t['args']['range']
  if '_binned' in t['constructor']:s['ggplot']={'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':5},'right':True}}
 if route not in ('range','area','radius','ordinal') and (not numeric or t['configuration']=='default'):s['palette_theme_aesthetics']=[channel]
 return s
def layer(t):
 p=(c.rule() if (temporal or numeric) and t['channel']=='linewidth' else c.points()).name('marks')
 if ordinal:p=p.aesthetic_value('Shape',{'kind':'Number','value':21})
 if t['route']!='automatic' and t['channel'] not in ('colour','fill'):p=p.numeric_scale({'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth'}[t['channel']],'v',descriptor(t))
 return p
def build(t,data):
 channel=t['channel'];mapping=c.aes().x('x').y(1.);mapping=getattr(mapping,'color' if channel=='colour' else channel)('v')
 if (temporal or numeric) and channel=='linewidth':mapping=mapping.x2('end').y2(1.)
 p=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry).theme(theme(t))
 if t['route']!='automatic' and channel in ('colour','fill'):
  mapping=getattr(mapping,'color_scale' if channel=='colour' else 'fill_scale')('v');p=p.scale(c.color_mapped('v',descriptor(t)))
 return p.aes(mapping).layer(layer(t)).build()
def paint(v):
 if v is None:return {'red':0,'green':0,'blue':0,'alpha':0}
 v='#7F7F7F' if v=='grey50' else v
 return dict(zip(('red','green','blue','alpha'),[int(v[i:i+2],16) for i in (1,3,5)]+[int(v[7:9],16) if len(v)==9 else 255]))
def number(v):return float(v['number']) if isinstance(v,dict) else float(v)
def check(t,chart):
 styles=chart.semantics()['layers'][0].get('styles',[]);wanted=[v for v in t['result']['mapped'] if v is not None or t['channel'] not in (('size','linewidth','colour') if ordinal else ('size','linewidth'))];assert len(styles)==len(wanted),t
 for i,(style,v) in enumerate(zip(styles,wanted)):
  if t['channel']=='linewidth' and v is None:
   assert style['color']==paint(t['result']['mark_colours' if temporal or numeric else 'point_colours'][i]);continue
  if t['channel'] in ('colour','fill'):assert style.get('color' if t['channel']=='colour' else 'fill')==paint(v),(t,style,v)
  else:
   actual=style['color']['alpha'] if t['channel']=='alpha' else number(style['radius' if t['channel']=='size' else 'stroke_width'])
   expected=paint(t['result']['mark_colours' if temporal or numeric else 'point_colours'][i])['alpha'] if t['channel']=='alpha' else v
   assert math.isclose(actual,expected,rel_tol=0,abs_tol=2e-12),(t,actual,expected)
 return styles
for index,t in enumerate(cases):
 owned=[]
 try:
  d=c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),**({'end':c.column([float(i)+.5 for i in range(len(t['inputs']))],kind='float64')} if temporal or numeric else {}),'v':c.timestamps([int(v or 0)*(86400 if t['kind']=='date' else 1)*1000000000 for v in t['inputs']],'ns','UTC').validity([v is not None for v in t['inputs']]) if temporal else c.column(t['inputs'],kind='string' if t['family']=='discrete' else 'float64')},name='data');owned.append(d)
  p=build(t,d);owned.append(p);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  if numeric and 'error' in t['result']:
   failed=p.chart();owned.append(failed)
   failed.semantics()
   raise AssertionError(('expected reference rejection',t))
  for state in ('original','layer_edit','theme_edit'):
   expected=t
   if state=='original':current=restored
   elif state=='layer_edit':current=restored.edit().layer('marks',layer(t)).build();owned.append(current)
   else:
    expected=t if numeric else next(x for x in cases if all(x.get(k)==t.get(k) for k in ('channel','family','route','kind','population')) and x['theme_mode']!=t['theme_mode'])
    current=restored.edit().theme(theme(expected)).build();owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,'styles':check(expected,chart)});assert restored.to_json()==wire
  if numeric and t['population']=='ordinary':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"numeric-{t['constructor']}-{t['configuration']}.{fmt}").write_bytes(frame.export(fmt))
  if (t['route']=='automatic' or ordinal) and t['theme_mode']=='supplied' and (ordinal or t.get('population') in (None,'ordinary')):
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-{t.get('population') if ordinal else t.get('kind',t['family'])}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert numeric and 'error' in t['result'],(t,error)
  diagnostic=json.loads(str(error));assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(t,diagnostic)
  records.append({'index':index,'error':diagnostic['code']})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(484 if numeric else 60 if ordinal else 180 if temporal else 168)
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS',len(records),'palette states and',sum(len(list(out.glob('*.'+fmt))) for fmt in ('svg','pdf','png')),'publication files')
