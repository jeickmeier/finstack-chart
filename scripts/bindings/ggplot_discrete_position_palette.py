"""FIX-GG04: materialized positional palettes through primary Python authoring."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-position-palettes.json').read_text())['cases'];records=[]
def key(v):return 'Null' if v is None else {'Text':v}
def number(v):return {'number':'NaN' if v is None else v} if v is None or isinstance(v,str) else v
for index,case in enumerate(cases):
 for family in ('band','point'):
  for mode in (('primary',) if 'error' in case['result'] else ('primary','secondary')):
   owned=[]
   try:
    values=case['inputs'];expected=case['result'] if mode=='primary' else case['result']['secondary']
    data=c.Data.columns({'x':c.categorical([v or '' for v in values]).validity([v is not None for v in values]),'y':c.column([1.]*len(values),kind='float64')},keys=list(range(100,100+len(values))));owned.append(data)
    axis=c.x_axis().scale(c.scale_band() if family=='band' else c.scale_point()).range(100,540).discrete_policy({'limits':None if case['limits'] is None else list(map(key,case['limits'])),'palette':list(map(number,case['palette_values'])),'na_translate':case['na_translate']}).guide_geometry({'labels':'Preserve'})
    builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis)
    if mode=='secondary':builder=builder.x_axis(c.x_axis().name('secondary').side('Top').secondary('x',1.,0.).guide_geometry({'labels':'Preserve'}))
    plot=builder.build();owned.append(plot);wire=plot.to_json();assert json.loads(wire)['version']==23
    restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
    request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame);assert 'error' not in expected,(index,mode,expected)
    guides=frame.guides()['guides'];ticks=next(g for g in guides if g['spec']['side']==('Bottom' if mode=='primary' else 'Top'))['ticks']
    if mode=='secondary':
     assert len(ticks)==len(expected['breaks'])
     for tick,value,label,position in zip(ticks,expected['breaks'],expected['labels'],expected['positions']):
      assert math.isclose(tick['value']['Number'],value,rel_tol=0,abs_tol=1e-12)
      assert tick['label']==('NA' if label is None else label)
      assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12),(index,mode,tick,position)
    else:
     assert len(ticks)==sum(v is not None for v in expected['major_positions'])
     for value,label,position in zip(expected['breaks'],expected['labels'],expected['major_positions']):
      semantic='MissingCategory' if value is None else {'Category':value};found=[t for t in ticks if t['value']==semantic]
      if position is None:assert not found
      else:
       assert len(found)==1;tick=found[0];assert tick['label']==('NA' if label is None else label)
       assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12)
    scene=frame.scene();positions=[None]*len(values)
    for item,targets in zip(scene['items'],scene['targets']):
     if targets:positions[int(targets[0]['Source']['key'])-100]=(item['primitive']['Point']['center']['x']-100)/440
    for actual,wanted in zip(positions,case['result']['positions']):
     assert (actual is None)==(wanted is None),(index,mode,actual,wanted)
     if actual is not None:assert math.isclose(actual,wanted,rel_tol=0,abs_tol=1e-12),(index,mode,actual,wanted)
    records.append({'index':index,'family':family,'mode':mode,'ticks':ticks,'positions':positions})
    if family=='band' and (index,mode) in ((10,'secondary'),(11,'secondary'),(14,'secondary'),(17,'primary')):
     for fmt in ('svg','pdf','png'):(out/f'position-palette-{case["palette_name"]}.{fmt}').write_bytes(frame.export(fmt))
   except c.ChartError as error:
    diagnostic=json.loads(str(error));assert 'error' in expected,(index,mode,diagnostic)
    records.append({'index':index,'family':family,'mode':mode,'error':diagnostic['code']})
   finally:
    for value in reversed(owned):value.dispose()
assert len(records)==520
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose()
print('PASS Python: 520 positional palette configurations and twelve publication files.')
