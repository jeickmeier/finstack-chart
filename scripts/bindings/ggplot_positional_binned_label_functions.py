"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-binned-label-functions.json').read_text())['cases']
def data_for(t):return c.Data.columns({'x':c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def build(t,d,route):
 transform={'sqrt':'Sqrt','reverse':'Reverse','log10':{'Log':{'base':10.}}}.get(t['transform'])
 breaks={'Nice' if t['break_mode']=='nice' else 'Equal':5.}
 if t['break_mode'] in ('explicit','empty'):breaks={'Explicit':[-1,0,1,1,3,20,{'number':'Infinity'},{'number':'NaN'}] if t['break_mode']=='explicit' else []}
 spec={'bins':{'limits':[1.,10.] if t['limits']=='full' else None,'breaks':breaks,'oob':'Squish','right':True},'transform':transform,'show_limits':t['show_limits']}
 if route=='policy' and t['label_mode']!='default':spec['labels']={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 scale=c.scale_binned(spec)
 axis=c.x_axis().scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}})
 if route=='policy' or t['label_mode']=='default':axis=axis.tick_format(None)
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if all(isinstance(v,(float,int)) for v in t['result']['range']):
  expected=[label or '' for v,label in zip(t['result']['values'],t['result']['labels']) if isinstance(v,(float,int))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
  lower,upper=t['result']['range']
  for tick,value in zip(ticks,[v for v in t['result']['values'] if isinstance(v,(float,int))]):assert abs((tick['position']-100)/440-(value-lower)/(upper-lower))<=3e-12,(t,tick,value)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 for route in ('axis','policy'):
  owned=[]
  try:
   d=data_for(t);owned.append(d);p=build(t,d,route);owned.append(p);wire=p.to_json()
   if route=='policy' and t['label_mode']!='default':assert json.loads(wire)['version']==32
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    assert 'error' not in t['result'],t
    records.append({'index':index,'route':route,'state':state,**check(t,frame)})
    if state=='original' and route=='policy' and index in (0,1,30,324,1200,1545):
     for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(t,str(error));assert error.code==('CHART_SCHEMA_CONFLICT' if 'labels' in t['result']['error'] else 'CHART_NUMERICAL_DOMAIN'),(t,error.code)
   records.append({'index':index,'route':route,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==4944,len(records)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 4944 binned positional label states and 18 publications.')
