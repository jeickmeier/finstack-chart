"""FIX-GG04: discrete identity duplicate axes through the actual Python adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-secondary.json').read_text())['cases'];records=[]
def key(v):return 'Null' if v is None else {'Text':v}
def semantic(v):return {'MissingCategory':None} if v is None else {'Category':v}
for index,case in enumerate(cases):
 for family in ('band','point'):
  owned=[]
  try:
   values=case['inputs'];control=case['control'];expected=case['result']
   data=c.Data.columns({'x':c.categorical([v or '' for v in values]).validity([v is not None for v in values]),'y':c.column([1.]*len(values),kind='float64')});owned.append(data)
   primary=c.x_axis().scale(c.scale_band() if family=='band' else c.scale_point()).range(100,540).discrete_policy({'limits':None if case['limits'] is None else list(map(key,case['limits'])),'na_translate':case['na_translate']}).guide_geometry({'labels':'Preserve'})
   if control in ('primary_breaks','primary_labels','primary_hidden'):
    guide={'breaks':[{'Text':'b'},{'Text':'a'}]} if control!='primary_hidden' else {'labels':'Hidden'}
    if control=='primary_labels':guide['labels']={'Explicit':['Bee','Aye']}
    primary=primary.discrete_policy({'limits':None if case['limits'] is None else list(map(key,case['limits'])),'na_translate':case['na_translate'],'guide':guide})
   secondary=c.x_axis().name('secondary').side('Top').secondary('x',2. if control=='transformed' else 1.,0.).guide_geometry({'labels':'Preserve'})
   if control=='empty':secondary=secondary.tick_values([])
   if control=='numeric':secondary=secondary.tick_values([{'Number':v} for v in [-1,0,.5,1,1.5,2,3,4,5]])
   if control in ('character','explicit','bad_labels'):
    keys=['b',None,'NA','absent','a'] if control=='character' else ['b',None,'NA','a'] if control=='explicit' else ['b','a']
    secondary=secondary.tick_values(list(map(semantic,keys)))
   if control in ('explicit','bad_labels','hidden'):
    labels=['Bee','Missing','Literal','A'] if control=='explicit' else ['wrong'] if control=='bad_labels' else ['']*len(expected.get('breaks',[]))
    secondary=secondary.tick_format({'Labels':labels})
   plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(primary).x_axis(secondary).build();owned.append(plot)
   restored=c.Plot.from_json(plot.to_json());owned.append(restored)
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in expected,(index,expected)
   ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Top')['ticks']
   assert len(ticks)==len(expected['breaks']),(index,ticks,expected)
   for tick,value,label,position in zip(ticks,expected['breaks'],(['']*len(ticks) if control in ('hidden','primary_hidden') else expected['labels']),expected['positions']):
    assert math.isclose(tick['value']['Number'],value,rel_tol=0,abs_tol=1e-12)
    assert tick['label']==('NA' if label is None else label),(index,tick,label)
    assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12),(index,tick,position)
   records.append({'index':index,'family':family,'ticks':ticks})
   if index in (11,14,16) and family=='band':
    for fmt in ('svg','pdf','png'):(out/f'discrete-secondary-{control}.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   diagnostic=json.loads(str(error));assert 'error' in case['result'],(index,diagnostic)
   records.append({'index':index,'family':family,'error':diagnostic['code']})
  finally:
   for value in reversed(owned):value.dispose()
assert len(records)==352
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose()
print('PASS Python: 352 discrete secondary configurations and nine publication files.')
