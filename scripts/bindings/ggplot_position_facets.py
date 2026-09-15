"""FIX-GG04 faceted positional callback builds through the actual Python adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,640).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-facet-functions.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-facet-temporal-functions.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-facet-bin-functions.json').read_text())['cases']
authored=len(sys.argv)>3 and sys.argv[3]=='authored'
if authored:cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-facet-authored-limits.json').read_text())['cases']
automatic=len(sys.argv)>3 and sys.argv[3]=='automatic'
if automatic:cases=[t for t in cases if t.get('kind')=='datetime']
def rejected(t):return 'error' in t['result']
def mapping(t):return {'field':'v','origin':'1704067200000'} if t.get('kind') in ('date','datetime') else 'v'
def family(t):return 'binned_'+t['transform'] if t.get('kind')=='binned' else t.get('kind',t.get('transform'))
def data_for(t):
 values=c.column([v or 0. for v in t['inputs']],kind='float64')
 if t.get('kind') in ('date','datetime'):
  factor=86400000 if t['kind']=='date' else 1000
  values=c.timestamps([round(v*factor) if v is not None else 1704067200000000//1000 for v in t['inputs']],'ms','UTC')
 return c.Data.columns({'v':values.validity([v is not None for v in t['inputs']]),'panel':c.categorical(t['groups'])},keys=list(range(100,104)))
def layer_for(t):
 layer=c.points().name('marks')
 return layer.stat(c.summary().x(mapping(t))).after_stat(c.stat_aes().x(1.).y('Mean')) if t['context']=='summary' else layer
def build(t,d):
 summary=t['context']=='summary';free=t['policy']=='free'
 scale=c.scale_binned({'transform':{'identity':None,'sqrt':'Sqrt','reverse':'Reverse'}[t['transform']]}) if t.get('kind')=='binned' else {'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'date':c.scale_date,'datetime':c.scale_utc,'duration':c.scale_duration}[family(t)]()
 axis=(c.y_axis() if summary else c.x_axis())
 if not automatic:axis=axis.scale(scale)
 axis=axis.guide_geometry({'labels':'Preserve'}).oob(t['oob'].capitalize())
 if t.get('authored'):axis=axis.numeric_limits(t['limits'])
 elif t['control']!='automatic':axis=axis.limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':t['control']}})

 p=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x(1. if summary else mapping(t)).y(mapping(t) if summary else 1.)).layer(layer_for(t)).facet(c.facet_wrap('panel').columns(3).order([[v] for v in t['levels']]).empty('Keep').free_x(free and not summary).free_y(free and summary))
 return (p.y_axis(axis).x_axis(c.x_axis().visible(False)) if summary else p.x_axis(axis).y_axis(c.y_axis().visible(False))).build()
def numeric(v):return float('nan') if v is None else float(v)
def equal(a,b):return a==b or math.isnan(a) and math.isnan(b) or abs(a-b)<=3e-12*max(1.,abs(b))
def retained(chart):
 return [{'key':p['key'],'domains':[l['domains'] for l in p['layers']],'limits':p.get('positional_limits',{})} for p in chart.semantics()['panels']]
def check(t,chart,frame):
 summary=t['context']=='summary';dimension='y' if summary else 'x';scene=frame.scene();guides=frame.guides()['guides'];result=[]
 assert len(scene['panels'])==len(t['result']['panels'])
 for panel,expected in zip(scene['panels'],t['result']['panels']):
  key=panel['key'];assert key['values']==[{'Text':expected['key']}];plot=panel['plot'];start=plot['origin'][dimension]+(plot['height'] if summary else 0.);length=-plot['height'] if summary else plot['width']
  points=[(item['primitive']['Point']['center'][dimension]-start)/length for item,p in zip(scene['items'],scene['item_panels']) if p==key and item.get('layer') is not None and 'Point' in item['primitive']]
  wanted=[numeric(v) for v in expected['point_positions'] if v is not None and math.isfinite(numeric(v))]
  assert len(points)==len(wanted) and all(equal(a,b) for a,b in zip(points,wanted)),(t,key,points,wanted)
  ticks=next(g for g in guides if g['scope']==[{'Panel':key}] and g['spec']['side']==('Left' if summary else 'Bottom'))['ticks']
  wanted=[(label,numeric(position)) for label,position in zip(expected['labels'],expected['positions']) if position is not None and math.isfinite(numeric(position))]
  assert len(ticks)==len(wanted),(t,key,ticks,wanted)
  for tick,(label,position) in zip(ticks,wanted):assert tick['label']==label and equal((tick['position']-start)/length,position),(t,key,tick,label,position)
  result.append({'key':key,'points':points,'ticks':ticks})
 return {'panels':result,'retained':retained(chart)}
def sample(t):
 if t.get('authored'):return t['control']=='partial_lower' and t['population']=='missing' and t['oob']=='censor'
 if t.get('kind')=='binned':return t['control']=='automatic' and t['population']=='balanced' and t['oob']=='censor'
 if 'kind' in t:return t['control']=='identity' and t['oob']=='censor' and ((t['context']=='points' and t['population']=='balanced') or (t['context']=='summary' and t['population']=='fractional'))
 return t['control']=='identity' and t['oob']=='censor' and ((t['transform']=='identity' and t['population']=='balanced') or (t['transform']=='sqrt' and t['context']=='summary' and t['population']=='constant') or (t['transform']=='reverse' and t['context']=='points' and t['population']=='empty_panel'))
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  assert json.loads(wire)['version']==(31 if authored else 18 if t['control']=='automatic' else 29)
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer_for(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert not rejected(t),(t,'unexpected success');records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and sample(t):
    for fmt in ('svg','pdf','png'):(out/f"{t['context']}-{family(t)}-{t['policy']}-{t['population']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert rejected(t),(t,str(error));assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error.code);records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(1434 if authored else 649 if automatic else 5360),len(records)
for context in ('points','summary'):
 for transform in (('identity','sqrt','reverse') if authored else ('datetime',) if automatic else ('identity','sqrt','reverse','date','datetime','duration','binned_identity','binned_sqrt','binned_reverse')):
  for policy in ('fixed','free'):
   owned=[]
   try:
    selected={t['population']:t for t in cases if t['context']==context and family(t)==transform and t['policy']==policy and t['control']==('partial_lower' if authored else 'automatic' if transform.startswith('binned_') else 'identity') and t['oob']=='censor'}
    original=data_for(selected['balanced']);owned.append(original);p=build(selected['balanced'],original);owned.append(p);chart=p.chart();owned.append(chart);request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
    for population in ('constant','missing','balanced','constant'):
     t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
     def capture(current):
      try:return {'panels':retained(current)}
      except c.ChartError as error:return {'error':error.code}
     actual=capture(chart)
     try:
      fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);expected=capture(batch)
     except c.ChartError as error:expected={'error':error.code}
     assert actual==expected,(t,actual,expected)
     assert held.scene()==scene and p.to_json()==wire
     records.append({'context':context,'transform':transform,'policy':policy,'replacement':population,**actual})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==(1482 if authored else 665 if automatic else 5504),len(records)
(out/'position-facet-records.json').write_text(json.dumps(records,indent=2))
options.dispose();output.dispose();registry.dispose();print('PASS Python:',len(records),'faceted positional callback states.')
