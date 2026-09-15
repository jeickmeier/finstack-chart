"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-label-functions.json').read_text())['cases']
def data_for(t):return c.Data.columns({'x':c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def build(t,d):
 scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 if t['limits']=='full':scale=scale.domain(1.,10.)
 axis=c.x_axis().scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}})
 if t['break_mode']=='explicit':axis=axis.tick_values([-1,0,1,1,3,20,{'Number':{'number':'Infinity'}},{'Number':{'number':'NaN'}}])
 elif t['break_mode']=='empty':axis=axis.tick_values([])
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if all(isinstance(v,(float,int)) for v in t['result']['range']):
  expected=[label or '' for v,label in zip(t['result']['breaks'],t['result']['labels']) if isinstance(v,(float,int))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**check(t,frame)})
   if state=='original' and index in (0,5,120,240,360):
    for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_SCHEMA_CONFLICT',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==668,len(records)
for transform in ('identity','sqrt','log10','reverse'):
 for mode in ('indexed','missing'):
  selected={t['population']:t for t in cases if t['transform']==transform and t['limits']=='full' and t['break_mode']=='automatic' and t['label_mode']==mode};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('constant','missing','all_missing','empty','ordinary'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    a=chart.semantics();b=batch.semantics();actual={'limits':a.get('positional_limits'),'domains':a['layers'][0]['domains']};assert actual=={'limits':b.get('positional_limits'),'domains':b['layers'][0]['domains']};assert held.scene()==scene and p.to_json()==wire
    records.append({'transform':transform,'mode':mode,'replacement':population,'semantics':actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==708
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 708 positional label states and 15 publications.')
