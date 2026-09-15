"""FIX-GG04 binned break functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
joint="--joint" in sys.argv[3:]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/('fixtures/parity/ggplot2/positional-binned-joint-dimensions.json' if joint else 'fixtures/parity/ggplot2/positional-binned-break-functions.json')).read_text())['cases']
def failed(t):return 'error' in t['result'] or isinstance(t['result'].get('positions'),dict) and 'error' in t['result']['positions']
def data_for(t):return c.Data.columns({'x':c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def build(t,d,route):
 transform={'sqrt':'Sqrt','reverse':'Reverse','log10':{'Log':{'base':10.}}}.get(t['transform'])
 breaks={'Nice':3. if t['count']=='three' else 0. if t['count']=='zero' else 10.}
 spec={'breaks_function':{'operation':{'id':'example.breaks_'+t['signature'],'version':'1'},'parameters':t['mode']},'bins':{'limits':[1.,10.] if t.get('limits')=='full' else None,'breaks':breaks,'oob':'Squish','right':True},'transform':transform,'show_limits':t['show_limits']}
 if route=='policy' and t['label_mode']!='automatic':spec['labels']={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 scale=c.scale_binned(spec)
 axis=c.x_axis().scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}})
 if route=='policy' or t['label_mode']=='automatic':axis=axis.tick_format(None)
 if joint:
  parameters={'mode':t['control']}
  if t['control']=='fixed':parameters['bounds']=[1.,10.]
  axis=axis.limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':parameters})
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if len(t['result']['range'])==2 and all(isinstance(v,(float,int)) for v in t['result']['range']):
  expected=[label or '' for v,label in zip(t['result']['values'],t['result']['labels']) if isinstance(v,(float,int))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
  lower,upper=t['result']['range']
  for tick,value in zip(ticks,[v for v in t['result']['values'] if isinstance(v,(float,int))]):assert abs((tick['position']-100)/440-(value-lower)/(upper-lower))<=3e-12,(t,tick,value)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 for route in (('axis','policy') if t['label_mode']=='indexed' else ('policy',)):
  owned=[]
  try:
   d=data_for(t);owned.append(d);p=build(t,d,route);owned.append(p);wire=p.to_json()
   assert json.loads(wire)['version']==35
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    assert not failed(t),t
    records.append({'index':index,'route':route,'state':state,**check(t,frame)})
    if state=='original' and route=='policy' and t['population']=='ordinary' and (t.get('control')=='fixed' if joint else t.get('limits')=='none') and t['signature']=='n' and t['count']=='three' and t['mode']=='mixed' and t['label_mode']=='automatic' and t['show_limits']:
     for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert failed(t),(t,str(error));assert (error.code in ('CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN','CHART_VALIDATION') if joint else error.code==('CHART_SCHEMA_CONFLICT' if 'labels' in t['result']['error'] else 'CHART_NUMERICAL_DOMAIN')),(t,error.code)
   records.append({'index':index,'route':route,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(2706 if joint else 14580),len(records)
for transform in ('identity','sqrt','log10','reverse'):
 for mode in ('domain','mixed'):
  selected={t['population']:t for t in cases if t['transform']==transform and (t.get('control')=='fixed' if joint else t.get('limits')=='full') and t['signature']=='n' and t['count']=='three' and t['mode']==mode and t['show_limits'] and t['label_mode']=='automatic'};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original,'policy');owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();assert json.loads(wire)['version']==35;request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('constant','missing','all_missing','empty','ordinary'):
    t=selected[population]
    if joint and failed(t):continue
    replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement,'policy');owned.append(fresh);batch=fresh.chart();owned.append(batch)
    a_request=output.request(chart,options);owned.append(a_request);a_frame=a_request.prepare();owned.append(a_frame);b_request=output.request(batch,options);owned.append(b_request);b_frame=b_request.prepare();owned.append(b_frame);actual=check(t,a_frame);assert actual==check(t,b_frame);assert held.scene()==scene and p.to_json()==wire
    records.append({'transform':transform,'mode':mode,'replacement':population,'semantics':actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(2744 if joint else 14620)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} binned positional break states; joint={joint}.')
