"""FIX-GG04: registered count palettes for primary categorical positions."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-palette-functions.json').read_text())['cases']
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def key(v):return 'Null' if v is None else {'Text':v}
def data(t):
 v=t['inputs'];return c.Data.columns({'x':c.categorical([x or '' for x in v]).validity([x is not None for x in v]),'y':c.column([1.]*len(v),kind='float64')},keys=list(range(100,100+len(v))),name='data')
def axis(t,family):
 modes={'reverse':'reverse_count','spread':'square_count','short':'short_count','named':'named_square_count','missing':'missing_count','character':'reject','null':'null'}
 limits=None if t['limit_mode']=='automatic' else list(map(key,['c','b','a','d'])) if t['limit_mode']=='retained' else []
 return c.x_axis().scale(c.scale_band() if family=='band' else c.scale_point()).range(100,540).discrete_policy({'limits':limits,'palette_function':{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':modes[t['route']],'channel':'size'}}}).guide_geometry({'labels':'Preserve'})
def build(t,d,family):return c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points().name('marks')).x_axis(axis(t,family)).build()
def check(t,frame):
 expected=t['result'];assert 'error' not in expected,t
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 assert len(ticks)==sum(x is not None for x in expected['major_positions']),(t,ticks)
 for value,label,position in zip(expected['breaks'],expected['labels'],expected['major_positions']):
  semantic='MissingCategory' if value is None else {'Category':value};found=[x for x in ticks if x['value']==semantic]
  if position is None:assert not found
  else:
   assert len(found)==1;tick=found[0];assert tick['label']==('NA' if label is None else label)
   assert math.isclose((tick['position']-100)/440,position,rel_tol=0,abs_tol=1e-12),(t,tick,position)
 scene=frame.scene();positions=[None]*len(t['inputs'])
 for item,targets in zip(scene['items'],scene['targets']):
  if targets:positions[int(targets[0]['Source']['key'])-100]=(item['primitive']['Point']['center']['x']-100)/440
 for actual,wanted in zip(positions,expected['positions']):
  assert (actual is None)==(wanted is None),(t,actual,wanted)
  if actual is not None:assert math.isclose(actual,wanted,rel_tol=0,abs_tol=1e-12),(t,actual,wanted)
 return {'ticks':ticks,'positions':positions}
for index,t in enumerate(cases):
 for family in ('band','point'):
  owned=[]
  try:
   d=data(t);owned.append(d);p=build(t,d,family);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==61
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   stale=json.loads(wire);stale['version']=60
   try:c.Plot.from_json(json.dumps(stale),registry)
   except c.ChartError:pass
   else:raise AssertionError('downgrade accepted')
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',c.points().name('marks')).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    records.append({'index':index,'family':family,'state':state,**check(t,frame)});assert restored.to_json()==wire
    if state=='original' and family=='band' and t['population']=='ordinary' and t['route'] in ('reverse','spread','named') and t['limit_mode']!='empty':
     for fmt in ('svg','pdf','png'):(out/f"{t['route']}-{t['limit_mode']}.{fmt}").write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(t,error);records.append({'index':index,'family':family,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==266,len(records)
for route in ('reverse','spread','named','missing'):
 for family in ('band','point'):
  selected={t['population']:t for t in cases if t['route']==route and t['limit_mode']=='automatic'};owned=[]
  try:
   original=data(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original,family);owned.append(p);chart=p.chart();owned.append(chart)
   request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
   for population in ('constant','missing','empty','ordinary'):
    t=selected[population];replacement=data(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement,family);owned.append(fresh);ar=output.request(chart,options);br=output.request(fresh,options);owned.extend([ar,br]);af=ar.prepare();bf=br.prepare();owned.extend([af,bf]);actual=check(t,af);assert actual==check(t,bf)
    assert held.scene()==scene and p.to_json()==wire;records.append({'route':route,'family':family,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==298
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS 298 positional palette states and 18 publication files')
