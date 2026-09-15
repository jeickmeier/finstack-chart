"""FIX-GG04 positional missing-value builds through the actual Python adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,640).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-missing.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-duration-missing.json').read_text())['cases']
def rejected(t):return 'error' in t['result'] or isinstance(t['result'].get('positions'),dict)
def data_for(t):
 values=c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])
 return c.Data.columns({'x':c.column([1.]*len(t['inputs']),kind='float64'),'y':values} if t['context']=='summary' else {'x':values},keys=list(range(100,100+len(t['inputs']))))
def layer_for(t):
 layer=c.points().name('marks')
 return layer.stat(c.summary().x('y')).after_stat(c.stat_aes().x(1.).y('Mean')) if t['context']=='summary' else layer
def build(t,d):
 summary=t['context']=='summary'
 scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.),'duration':c.scale_duration}[t['transform']]()
 if t['limit_control']=='fixed':scale=scale.domain(1.,4.)
 axis=(c.y_axis() if summary else c.x_axis()).scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).oob(t['oob'].capitalize()).missing_value({'missing':None,'zero':0.,'five':5.,'negative':-1.}[t['replacement']])
 if t['limit_control']=='function':axis=axis.limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':'identity'}})
 p=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y' if summary else 1.)).layer(layer_for(t))
 return (p.y_axis(axis).x_axis(c.x_axis().visible(False)) if summary else p.x_axis(axis).y_axis(c.y_axis().visible(False))).build()
def numeric(v):return float('nan') if v is None else float(v)
def equal(a,b):return a==b or math.isnan(a) and math.isnan(b) or abs(a-b)<=3e-12*max(1.,abs(b))
def check(t,chart,frame):
 summary=t['context']=='summary';dimension='y' if summary else 'x'
 points=[(item['primitive']['Point']['center'][dimension]-100.)/440. for item in frame.scene()['items'] if item.get('layer') is not None and 'Point' in item['primitive']]
 expected=[numeric(v) for v in t['result'].get('coordinate_positions',t['result']['point_positions']) if v is not None and math.isfinite(numeric(v))]
 assert len(points)==len(expected) and all(equal(a,b) for a,b in zip(points,expected)),(t,points,expected)
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Left' if summary else 'Bottom'))['ticks']
 expected=[(label,numeric(position)) for label,position in zip(t['result']['labels'],t['result']['positions']) if position is not None and math.isfinite(numeric(position))]
 assert len(ticks)==len(expected),(t,ticks,expected)
 for label,position in expected:assert any(tick['label']==label and equal((tick['position']-100.)/440.,position) for tick in ticks),(t,ticks,label,position)
 semantics=chart.semantics()
 return {'points':points,'ticks':ticks,'domains':semantics['layers'][0]['domains'],'limits':semantics.get('positional_limits',{})}
def sample(t):
 return t['replacement']=='negative' and t['oob']=='censor' and ((t['population']=='mixed' and t['limit_control']=='auto' and t['transform'] in ('identity','sqrt','log10','duration')) or (t['population']=='all_missing' and t['limit_control']=='auto' and t['transform']=='sqrt' and t['context']=='points') or (t['population']=='mixed' and t['limit_control']=='function' and t['transform']=='log10' and t['context']=='summary'))
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  if t['replacement']!='missing':assert json.loads(wire)['version']==30
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer_for(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert not rejected(t),(t,'unexpected success');records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and sample(t):
    for fmt in ('svg','pdf','png'):(out/f"{t['context']}-{t['transform']}-{t['population']}-{t['limit_control']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert rejected(t),(t,str(error));assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error.code);records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==2520,len(records)
for context,transform in [('points','identity'),('points','sqrt'),('summary','log10'),('points','duration'),('summary','duration')]:
 owned=[]
 try:
  selected={t['population']:t for t in cases if t['context']==context and t['transform']==transform and t['replacement']=='negative' and t['limit_control']=='auto' and t['oob']=='censor'}
  original=data_for(selected['mixed']);owned.append(original);p=build(selected['mixed'],original);owned.append(p);chart=p.chart();owned.append(chart);request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
  for population in ('constant','all_missing','empty','mixed'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=chart.semantics();expected=batch.semantics();assert actual['layers'][0]['domains']==expected['layers'][0]['domains'];assert actual.get('positional_limits',{})==expected.get('positional_limits',{});assert held.scene()==scene;assert p.to_json()==wire
   records.append({'context':context,'transform':transform,'replacement':population,'domains':actual['layers'][0]['domains'],'limits':actual.get('positional_limits',{})})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==2540,len(records)
(out/'position-missing-records.json').write_text(json.dumps(records,indent=2))
options.dispose();output.dispose();registry.dispose();print('PASS Python:',len(records),'positional missing-value states.')
