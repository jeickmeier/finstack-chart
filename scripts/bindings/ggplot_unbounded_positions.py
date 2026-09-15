"""FIX-GG04 actual host proof for exact unbounded coordinate views."""
import json, math, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
registry = c.ExtensionRegistry.example()
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(640, 360).dpi(96).basis('current')
records = []
authored = len(sys.argv) > 3 and sys.argv[3] == "authored"
fixture = json.loads((ROOT / 'fixtures/parity/ggplot2/positional-unbounded-viewports-exact.json').read_text())
if authored:fixture['cases'] += json.loads((ROOT / 'fixtures/parity/ggplot2/positional-authored-limits.json').read_text())['cases']
def supported(t):
 return authored or t['kind']=='binned' or (t['population']!='empty' and not (t['transform'] in ('sqrt','log10') and t['control'] in ('both','lower','negative')))
cases = [(i,t) for i,t in enumerate(fixture['cases']) if supported(t)]
assert len(cases)==(2376 if authored else 1350)
def numeric(v): return float('nan') if v is None else float(v)
def wire_number(v): return {'number':v} if isinstance(v,str) else v
def data_for(t):
 return c.Data.columns({'x':c.column([numeric(v) if v is not None else None for v in t['inputs']],kind='float64').nullable(True)},keys=list(range(100,100+len(t['inputs']))))
def build(t,d):
 transform={'identity':None,'sqrt':'Sqrt','reverse':'Reverse','log10':{'Log':{'base':10.}}}[t['transform']]
 if t['kind']=='binned':
  scale=c.scale_binned({'transform':transform,'bins':{'limits':[wire_number(v) for v in t['limits']],'oob':t['oob'].capitalize(),'breaks':{'Nice':10.},'right':True}})
 else:
  scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 view=sorted(t['viewport'])
 if t['transform']=='reverse':view.reverse()
 axis=c.x_axis().scale(scale).viewport(*view).oob(t['oob'].capitalize()).guide_geometry({'labels':'Preserve'})
 if t['kind']=='continuous':
  bounds=[wire_number(v) for v in t['limits']]
  if authored:axis=axis.numeric_limits(bounds)
  else:
   if t['transform']=='reverse':bounds.reverse()
   axis=axis.limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':'fixed','bounds':bounds}})
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(c.points().name('marks')).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,chart,frame):
 scene=frame.scene(); points=[None]*len(t['inputs'])
 guide_frame=next(g['frame'] for g in frame.presentation() if g['frame']['side']=='Bottom')
 domain=guide_frame['domain'];numbers=domain['numbers'];assert domain['horizontal'] and guide_frame['translation'][0]==0.
 offset=numbers[2] if domain['caps'] else numbers[1]
 start=numbers[0]-offset;length=numbers[3 if domain['caps'] else 2]-offset-start
 assert length>0.

 for item,targets in zip(scene['items'],scene['targets']):
  if targets and 'Source' in targets[0] and 'Point' in item['primitive']:
   row=int(targets[0]['Source']['key'])-100
   assert 0<=row<len(points) and points[row] is None
   clip=item['clip'];assert abs(clip['origin']['x']-start)<=3e-12 and abs(clip['width']-length)<=3e-12
   points[row]=(item['primitive']['Point']['center']['x']-start)/length
 for actual,wanted in zip(points,t['result']['point_positions']):
  expected=numeric(wanted)
  if not math.isfinite(expected): assert actual is None,(t,points)
  else: assert actual is not None and abs(actual-expected)<=3e-12,(t,points)
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 expected=[(numeric(p),l or '') for p,l in zip(t['result']['positions'],t['result']['labels']) if math.isfinite(numeric(p))]
 expected.sort(key=lambda v:v[0])
 assert len(ticks)==len(expected),(t,ticks,expected)
 for tick,(position,label) in zip(sorted(ticks,key=lambda t:t['position']),expected):
  assert tick['label']==label and abs((tick['position']-start)/length-position)<=3e-12,(t,ticks,expected)
 semantics=chart.semantics()
 return {'points':points,'ticks':ticks,'domains':semantics['layers'][0]['domains'],'limits':semantics.get('positional_limits',{})}
def sample(t):
 if authored and t['kind']=='continuous' and t['control']=='both' and t['view']=='positive' and t['oob']=='keep' and (t['population']=='empty' or (t['population']=='finite' and t['transform'] in ('sqrt','log10'))):return True
 return t['view']=='positive' and ((t['control']=='upper' and t['population']=='infinite' and t['oob']=='keep') or (t['kind']=='binned' and (t['control']=='finite' or (t['transform']=='log10' and t['control']=='lower')) and t['population']=='finite' and t['oob']=='censor'))
for index,t in cases:
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p)
  wire=p.to_json();assert json.loads(wire)['version']==((31 if authored else 29) if t['kind']=='continuous' else 18)
  if authored and index==0:
   invalid=json.loads(wire);invalid['version']=30
   try:c.Plot.from_json(json.dumps(invalid),registry)
   except c.ChartError as error:assert error.code=='CHART_UNSUPPORTED_CAPABILITY'
   else:raise AssertionError('authored limits accepted a downgraded envelope')
   try:
    legacy=c.plot(d).aes(c.aes().x('x').y(1.)).layer(c.points()).x_axis(c.x_axis().numeric_limits([None,5.])).build();owned.append(legacy)
    legacy_chart=legacy.chart();owned.append(legacy_chart);legacy_chart.semantics()
   except c.ChartError as error:assert error.code=='CHART_UNSUPPORTED_CAPABILITY'
   else:raise AssertionError('legacy profile accepted authored ggplot limits')
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',c.points().name('marks')).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**check(t,chart,frame)})
   if state=='original' and sample(t):
    suffix='-'+t['population'] if t['control']=='both' else ''
    for fmt in ('svg','pdf','png'):(out/f"{t['kind']}-{t['transform']}-{t['control']}{suffix}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,error)
  assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(4392 if authored else 2394)
for kind in ('continuous','binned'):
 for transform in ('identity','sqrt','reverse','log10'):
  owned=[]
  try:
   selected={t['population']:t for index,t in cases if index<1728 and t['kind']==kind and t['transform']==transform and t['control']=='upper' and t['view']=='positive' and t['oob']=='keep'}
   original=data_for(selected['finite']);owned.append(original);p=build(selected['finite'],original);owned.append(p)
   chart=p.chart();owned.append(chart);request=output.request(chart,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
   populations=('infinite','missing','empty','finite','infinite') if authored and kind=='continuous' else ('infinite','missing','finite','infinite')
   for population in populations:
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);outcome=chart.commit(update);assert 'Applied' in outcome,(kind,transform,population,outcome)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    actual=chart.semantics();expected=batch.semantics();assert actual['layers'][0]['domains']==expected['layers'][0]['domains'];assert actual.get('positional_limits',{})==expected.get('positional_limits',{})
    current_request=output.request(chart,options);owned.append(current_request);current_frame=current_request.prepare();owned.append(current_frame)
    batch_request=output.request(batch,options);owned.append(batch_request);batch_frame=batch_request.prepare();owned.append(batch_frame)
    projection=check(t,chart,current_frame);assert projection==check(t,batch,batch_frame)
    assert held.scene()==scene and p.to_json()==wire
    records.append({'kind':kind,'transform':transform,'replacement':population,'projection':projection})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(4428 if authored else 2426)
(out/'unbounded-position-records.json').write_text(json.dumps(records,indent=2,allow_nan=False))
options.dispose();output.dispose();registry.dispose()
print('PASS Python:',len(records),'unbounded coordinate states.')
