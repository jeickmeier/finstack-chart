"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-discrete-label-functions.json').read_text())['cases']
def data_for(t):return c.Data.columns({'x':c.categorical([v or '' for v in t['inputs']]).validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def key(v):return 'Null' if v is None else {'Text':v}
def build(t,d,family,route):
 scale=c.scale_band() if family=='band' else c.scale_point()
 policy={'levels':list(map(key,['b','a','c','unused'])),'limits':list(map(key,['c','b','a',None])) if t['limits']=='full' else None,'drop':t['drop'],'na_translate':t['translate']}
 if t['break_mode']=='named':policy['guide']={'breaks':list(map(key,['outside','a','a',None,'c'])),'break_names':['off','first','duplicate','missing','last']}
 if route=='policy':policy.setdefault('guide',{})['labels']={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 axis=c.x_axis().scale(scale).discrete_policy(policy).range(100.,540.).guide_geometry({'labels':'Preserve'}).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}})
 if t['break_mode']=='explicit':axis=axis.tick_values([{'Category':v} if v is not None else {'MissingCategory':None} for v in ['outside','a','a',None,'c']])
 elif t['break_mode']=='empty':axis=axis.tick_values([])
 if route=='policy':axis=axis.tick_format(None)
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if all(isinstance(v,(float,int)) for v in t['result']['range']):
  raw=t['result']['labels'];expected=[raw[i%len(raw)] or '' for i in range(len(t['result']['values']))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 for family in ('band','point'):
  for route in ('axis','policy'):
   owned=[]
   try:
    d=data_for(t);owned.append(d);p=build(t,d,family,route);owned.append(p);wire=p.to_json()
    if route=='policy' and t['label_mode']!='default':assert json.loads(wire)['version']==32
    restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
    for state in ('original','edited'):
     current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
     if current is not restored:owned.append(current)
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
     assert 'error' not in t['result'],t
     records.append({'index':index,'family':family,'route':route,'state':state,**check(t,frame)})
     if state=='original' and route=='policy' and family=='band' and index in (0,1,2,261):
      for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
    records.append({'index':index,'family':family,'route':route,'error':error.code})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==4700,len(records)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 4700 discrete positional label states and 12 publications.')
