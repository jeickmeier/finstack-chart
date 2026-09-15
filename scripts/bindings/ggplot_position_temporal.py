"""FIX-GG04 positional temporal callback builds through the actual Python adapter."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,640).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-temporal-functions.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-duration-functions.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/positional-temporal-fractional-statistics.json').read_text())['cases']
automatic=len(sys.argv)>3 and sys.argv[3]=='automatic'
if automatic:cases=[t for t in cases if t['kind']=='datetime']
calendar=len(sys.argv)>3 and sys.argv[3]=='calendar'
if calendar:cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-calendar-functions.json').read_text())['cases']
def origin(t):return int(t.get('epoch',1704067200))*1000
def calendar_scale(t):
 zone='Utc' if t['zone']=='UTC' else {'Local':{'version':1,'zone':t['zone'],'revision':'0','tzdata':'explicit-2024-US-transitions','coverage':{'start':'1710000000000','end':'1740000000000'},'initial_offset_seconds':-18000,'transitions':[{'at_millis':'1710054000000','offset_seconds':-14400},{'at_millis':'1730613600000','offset_seconds':-18000}]}}
 return c.scale_calendar({'domain':[str(origin(t)),str(origin(t)+86400000)],'unit':'Milliseconds','zone':zone,'range':[{'kind':'Number','value':0},{'kind':'Number','value':1}],'factory':{'kind':'Value'},'clamp':False,'unknown':{'kind':'Missing'}})
def rejected(t):return 'error' in t['result']
def mapping(t):return 't' if t['kind']=='duration' else {'field':'t','origin':str(origin(t))}
def data_for(t):
 factor=86400000 if t['kind']=='date' else 1000
 values=(c.column([v or 0. for v in t['inputs']],kind='float64') if t['kind']=='duration' else c.timestamps([round(v*factor) if v is not None else 1704067200000 for v in t['inputs']],'ms','UTC')).validity([v is not None for v in t['inputs']])
 return c.Data.columns({'t':values},keys=list(range(100,100+len(t['inputs']))))
def layer_for(t):
 layer=c.points().name('marks')
 return layer.stat(c.summary().x(mapping(t))).after_stat(c.stat_aes().x(1.).y('Mean')) if t['context']=='summary' else layer
def build(t,d):
 summary=t['context']=='summary'
 axis=(c.y_axis() if summary else c.x_axis())
 if calendar:axis=axis.scale(calendar_scale(t))
 elif not automatic:axis=axis.scale(c.scale_duration() if t['kind']=='duration' else c.scale_date() if t['kind']=='date' else c.scale_utc())
 axis=axis.range(100.,540.).guide_geometry({'labels':'Preserve'}).oob(t['oob'].capitalize()).limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':t['control']}})
 p=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x(1. if summary else mapping(t)).y(mapping(t) if summary else 1.)).layer(layer_for(t))
 return (p.y_axis(axis).x_axis(c.x_axis().visible(False)) if summary else p.x_axis(axis).y_axis(c.y_axis().visible(False))).build()
def numeric(v):return float('nan') if v is None else float(v)
def equal(a,b):return a==b or math.isnan(a) and math.isnan(b) or abs(a-b)<=3e-12*max(1.,abs(b))
def check(t,chart,frame):
 summary=t['context']=='summary';dimension='y' if summary else 'x'
 points=[(item['primitive']['Point']['center'][dimension]-100.)/440. for item in frame.scene()['items'] if item.get('layer') is not None and 'Point' in item['primitive']]
 expected=[numeric(v) for v in t['result']['point_positions'] if v is not None and math.isfinite(numeric(v))]
 assert len(points)==len(expected) and all(equal(a,b) for a,b in zip(points,expected)),(t,points,expected)
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Left' if summary else 'Bottom'))['ticks']
 expected=[(label,numeric(position)) for label,position in zip(t['result']['labels'],t['result']['positions']) if position is not None and math.isfinite(numeric(position))]
 assert len(ticks)==len(expected),(t,ticks,expected)
 for label,position in expected:assert any(tick['label']==label and equal((tick['position']-100.)/440.,position) for tick in ticks),(t,ticks,label,position)
 semantics=chart.semantics()
 return {'points':points,'ticks':ticks,'domains':semantics['layers'][0]['domains'],'limits':semantics.get('positional_limits',{})}
def sample(t):
 if calendar:return t['context']=='points' and t['population']=='spaced' and t['control']=='identity' and t['oob']=='censor'
 return (t['oob']=='censor' and ((t['population']=='fractional' and t['control']=='identity') or (t['population']=='all_missing' and t['control']=='missing_lower' and t['context']=='points'))) or (t['oob']=='keep' and t['population']=='spaced' and t['control']=='single' and t['context']=='points') or (t['context']=='summary' and t['population'] in ('thirds','sevenths') and t['control']=='identity' and t['oob']=='censor')
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  assert json.loads(wire)['version']==29
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer_for(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert not rejected(t),(t,'unexpected success');records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and sample(t):
    for fmt in ('svg','pdf','png'):(out/f"{t['kind']}-{t['context']}-{t['population']}-{t['control']}{('-'+t['zone'].replace('/','-')+'-'+str(t['epoch'])) if calendar else ''}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert rejected(t),(t,str(error));assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error.code);records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(412 if calendar else 445 if automatic else 1292),len(records)
groups=[('datetime',zone,epoch) for zone in ('UTC','America/New_York') for epoch in (1710046800,1730606400)] if calendar else [(kind,None,None) for kind in (('datetime',) if automatic else ('date','datetime','duration'))]
for kind,zone,epoch in groups:
 for context in ('points','summary'):
  owned=[]
  try:
   selected={t['population']:t for t in cases if t['kind']==kind and t['context']==context and t['control']=='identity' and t['oob']=='censor' and (not calendar or t['zone']==zone and t['epoch']==epoch)}
   original=data_for(selected['spaced']);owned.append(original);p=build(selected['spaced'],original);owned.append(p);chart=p.chart();owned.append(chart);request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
   for population in (('constant','missing','spaced','constant') if calendar else ('constant','missing','all_missing','empty','fractional','spaced')):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    def capture(current):
     try:
      state=current.semantics();return {'domains':state['layers'][0]['domains'],'limits':state.get('positional_limits',{})}
     except c.ChartError as error:
      assert rejected(t);return {'error':error.code}
    actual=capture(chart)
    try:
     fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);expected=capture(batch)
    except c.ChartError as error:
     assert rejected(t);expected={'error':error.code}
    assert actual==expected
    assert held.scene()==scene and p.to_json()==wire
    records.append({'kind':kind,'context':context,'replacement':population,**({'zone':zone,'epoch':epoch} if calendar else {}),**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(444 if calendar else 457 if automatic else 1328),len(records)
(out/'position-temporal-records.json').write_text(json.dumps(records,indent=2))
options.dispose();output.dispose();registry.dispose();print('PASS Python:',len(records),'temporal positional callback states.')
