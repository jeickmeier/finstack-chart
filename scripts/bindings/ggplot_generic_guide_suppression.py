"""GG-04: generic guide suppression through actual primary hosts."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/generic-guide-suppression.json').read_text())['cases']]
def descriptor(t):
 channel=t['channel'];discrete=t['family']=='discrete'
 s={'missing_paint_is_na':channel=='colour','training':'Eligible',
  'guide':'Hidden' if t['breaks']=='null' or t['guide']=='none' else ({'Discrete':{'breaks':[] if t['breaks']=='empty' else None,'labels':'Automatic'}} if discrete else {'BinnedLegend' if t['guide']=='legend' else 'BinnedBins':'Automatic'}),
  'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,4],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[0,1],'exponent':1,'absolute':False}},'unknown':{'kind':'Missing'}}},
  'ggplot':{'Binned':{'limits':[0,4] if t['limit_mode']=='fixed' else None,'oob':'Squish','breaks':{'Nice':5} if t['breaks']=='default' else {'Explicit':[]},'right':True}},
  'palette_function':{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':'endpoints' if discrete else 'full','channel':channel}}}
 if discrete:
  s['function']={'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}}
  s['ggplot']={'Discrete':{'limits':[{'Text':v} for v in ('a','b','c')] if t['limit_mode']=='fixed' else None,'levels':None,'drop':True,'na_translate':True,'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}}}}
 return s
def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def build(t):
 values=t['inputs'] if t['family']=='discrete' else [float(v) if isinstance(v,str) else v for v in t['inputs']]
 data=c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.column(values,kind='string' if t['family']=='discrete' else 'float64').nullable(True)},name='data')
 draft=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry)
 if t['channel']=='colour':draft=draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)))
 else:draft=draft.aes(c.aes().x('x').y(1.))
 return data,draft.layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0]
 entries=(state.get('color_legend') or {}).get('entries',[]) if t['channel']=='colour' else [e for group in state.get('numeric_value_guides',{}).values() for e in group if e['visible']]
 if not t['result']['guides']:assert not entries,(t,entries)
 return {'keys':entries,'styles':state.get('styles',[])}

output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
for index,t in enumerate(cases):
 owned=[]
 try:
  data,p=build(t);owned.extend([data,p]);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,**check(t,chart)});assert restored.to_json()==wire
  if t['population']=='ordinary' and t['guide']=='default':
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['family']}-{t['channel']}-{t['limit_mode']}-{t['breaks']}.{fmt}").write_bytes(frame.export(fmt))
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==972
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 972 generic guide-suppression states and 108 publications')
