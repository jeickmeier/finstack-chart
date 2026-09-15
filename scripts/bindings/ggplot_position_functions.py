"""FIX-GG04 primary positional callbacks through the rebuilt Python adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-limit-functions.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-limit-function-arguments.json').read_text())['cases']
def rejected(t):return 'error' in t['result'] or isinstance(t['result'].get('positions'),dict)
def prefix(t):return 'binned-' if t['kind']=='binned' else ''
def data_for(t):return c.Data.columns({'x':c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def build(t,d):
 scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 if t['kind']=='binned':scale=c.scale_binned({'transform':{'identity':None,'sqrt':'Sqrt','reverse':'Reverse','log10':{'Log':{'base':10.}}}[t['transform']]})
 axis=c.x_axis().scale(scale).limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':t['control']}}).range(100.,540.).guide_geometry({'labels':'Preserve'})
 if 'oob' in t:axis=axis.oob(t['oob'].capitalize())
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(c.points().name('marks')).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def numeric(v):
 if isinstance(v,dict):return numeric(v['number'])
 return float('nan') if v is None else float(v)
def equal(a,b):return a==b or math.isnan(a) and math.isnan(b) or abs(a-b)<=3e-12*max(1.,abs(b))
def check(t,chart,frame):
 limits=chart.semantics()['positional_limits']['0'];assert len(limits)==len(t['result']['limits'])
 assert all(equal(numeric(a),numeric(b)) for a,b in zip(limits,t['result']['limits'])),(t,limits)
 scene=frame.scene();points=[None]*len(t['inputs'])
 for item,targets in zip(scene['items'],scene['targets']):
  if targets and 'Source' in targets[0] and 'Point' in item['primitive']:
   row=int(targets[0]['Source']['key'])-100;points[row]=(item['primitive']['Point']['center']['x']-100.)/440.
 for actual,wanted in zip(points,t['result'].get('coordinate_positions',t['result']['point_positions'])):
  if wanted is None or not math.isfinite(numeric(wanted)):assert actual is None,(t,points)
  else:assert actual is not None and equal(actual,numeric(wanted)),(t,points,wanted)
 guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom');ticks=guide['ticks']
 expected=[(label,numeric(position)) for label,position in zip(t['result']['labels'],t['result']['positions']) if position is not None and math.isfinite(numeric(position))]
 assert len(ticks)==len(expected),(t,ticks,expected)
 for label,position in expected:assert any(tick['label']==label and equal((tick['position']-100.)/440.,position) for tick in ticks),(t,ticks,label,position)
 return {'limits':limits,'points':points,'ticks':ticks}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==29
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',c.points().name('marks')).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert not rejected(t),(t,'unexpected successful layout')
   records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and 'oob' not in t and t['population']=='spaced' and t['transform'] in ('identity','sqrt') and t['control'] in ('identity','fixed','single'):
    for fmt in ('svg','pdf','png'):(out/f"{prefix(t)}{t['transform']}-{t['control']}.{fmt}").write_bytes(frame.export(fmt))
   if state=='original' and 'oob' not in t and t['kind']=='continuous' and t['population']=='all_missing' and t['control']=='missing_lower' and t['transform'] in ('identity','reverse'):
    assert frame.scene()['version']==16
    for fmt in ('svg','pdf','png'):(out/f"{t['transform']}-infinite-guide.{fmt}").write_bytes(frame.export(fmt))
   if state=='original' and t['population']=='spaced' and t['transform']=='log10' and t['control']=='identity' and t.get('oob')==('censor' if t['kind']=='continuous' else 'squish'):
    for fmt in ('svg','pdf','png'):(out/f"{prefix(t)}log10-identity.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  diagnostic=json.loads(str(error));assert rejected(t),(t,diagnostic);assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(t,diagnostic)
  records.append({'index':index,'error':diagnostic['code']})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==1559,len(records)
for kind,control in [('continuous',v) for v in ('identity','fixed','single')]+[('binned',v) for v in ('identity','fixed')]:
 owned=[]
 try:
  selected={t['population']:t for t in cases if 'oob' not in t and t['kind']==kind and t['transform']=='identity' and t['control']==control}
  original=data_for(selected['spaced']);owned.append(original);p=build(selected['spaced'],original);owned.append(p);wire=p.to_json();chart=p.chart();owned.append(chart)
  request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('constant','missing','all_missing','empty','spaced'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
   actual=chart.semantics();expected=batch.semantics();limits=actual['positional_limits'];assert limits==expected['positional_limits']
   assert actual['layers'][0]['domains']==expected['layers'][0]['domains']
   assert held.scene()==scene;assert p.to_json()==wire
   records.append({'kind':kind,'control':control,'replacement':population,'limits':limits})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==1584,len(records)
(out/'position-function-records.json').write_text(json.dumps(records,indent=2))
options.dispose();output.dispose();registry.dispose()
print('PASS Python:',len(records),'positional callback states.')
