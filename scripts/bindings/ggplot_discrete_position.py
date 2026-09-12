"""FIX-GG04: actual nullable positional data, factor policies, guides and publication."""
import json
import math
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/position-null-categories.json').read_text())['cases'];cases += json.loads((ROOT/'fixtures/parity/ggplot2/position-null-continuous-limits.json').read_text())['cases'];cases += json.loads((ROOT/'fixtures/parity/ggplot2/position-null-minor-breaks.json').read_text())['cases'];records=[]
def key(v):return 'Null' if v is None else {'Text':v}
def policy(case):return {'limits':None if case['limits'] is None else list(map(key,case['limits'])),'levels':None if case['levels'] is None else list(map(key,case['levels'])),'drop':case['drop'],'na_translate':case['na_translate'],'guide':{'breaks':None if case['breaks'] is None else list(map(key,case['breaks']))}}
for index,case in enumerate(cases):
 for family in ('band','point'):
  owned=[]
  try:
   values=case['inputs'];n=len(values)
   data=c.Data.columns({'x':c.categorical([v or '' for v in values]).validity([v is not None for v in values]),'y':c.column([1.]*n,kind='float64')},keys=list(range(100,100+n)));owned.append(data)
   axis=c.x_axis().scale(c.scale_band() if family=='band' else c.scale_point()).range(100,540).discrete_policy(policy(case)).guide_geometry({'labels':'Preserve'})
   if 'expansion' in case:axis=axis.expansion(case['expansion']).continuous_limits(None if case['continuous_limits'] is None else [{'number':v} if isinstance(v,str) else v for v in case['continuous_limits']])
   if 'minor_breaks' in case:axis=axis.minor_breaks({'Numeric':[{'number':v} if isinstance(v,str) else v for v in case['minor_breaks']]})
   plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).build();owned.append(plot)
   wire=plot.to_json();assert json.loads(wire)['version']==22
   restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in case['result']
   scene=frame.scene();points=[None]*n
   for item,targets in zip(scene['items'],scene['targets']):
    if not targets:continue
    assert len(targets)==1 and 'Source' in targets[0] and 'Point' in item['primitive']
    row=int(targets[0]['Source']['key'])-100;assert points[row] is None
    points[row]=(item['primitive']['Point']['center']['x']-100)/440
   for actual,wanted in zip(points,case['result']['point_positions']):
    assert (actual is None)==(wanted is None),(index,actual,wanted)
    if actual is not None:assert math.isclose(actual,wanted,rel_tol=0,abs_tol=1e-12),(index,actual,wanted)
   assert sum(v is not None for v in points)==case['result']['point_count'],(index,points,case)
   ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
   for value,label,position in zip(case['result']['breaks'],case['result']['labels'],case['result']['major_positions']):
    semantic='MissingCategory' if value is None else {'Category':value}
    matches=[t for t in ticks if t['value']==semantic]
    if position is None:assert not matches
    else:
     assert len(matches)==1,(index,semantic,matches)
     tick=matches[0];assert tick['label']==('NA' if label is None else label)
     assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12)
   assert len(ticks)==sum(v is not None for v in case['result']['major_positions'])
   minor=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom').get('minor_ticks',[])
   if 'minor_values' in case['result']:
    assert len(minor)==len(case['result']['minor_values'])
    for tick,value,position in zip(minor,case['result']['minor_values'],case['result']['minor_positions']):
     assert tick['value']=={'Number':value}
     assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12)
   records.append({'index':index,'family':family,'points':points,'ticks':ticks,'minor_ticks':minor})
   if index < 576 and family=='band' and case['population']=='mixed' and case['level_name']=='character' and case['limits_name']=='auto' and case['drop'] and case['breaks_name']=='auto':
    for fmt in ('svg','pdf','png'):(out/f'null-positions-{case["na_translate"]}.{fmt}').write_bytes(frame.export(fmt))
   if index==652 and family=='band':
    for fmt in ('svg','pdf','png'):(out/f'null-positions-expanded.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   diagnostic=json.loads(str(error));assert 'error' in case['result'] and diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(index,diagnostic)
   records.append({'index':index,'family':family,'error':diagnostic['code']})
  finally:
   for value in reversed(owned):value.dispose()
assert len(records)==3840
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose()
print('PASS Python: 3840 positional nullable chart configurations and nine publication files.')
