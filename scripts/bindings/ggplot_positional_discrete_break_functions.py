"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-discrete-break-functions.json').read_text())['cases']
def data_for(t):return c.Data.columns({'x':c.categorical([v or '' for v in t['inputs']]).validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def key(v):return 'Null' if v is None else {'Text':v}
def build(t,d,family,route):
 axis=c.x_axis()
 if family!='auto':axis=axis.scale(c.scale_band() if family=='band' else c.scale_point())
 policy={'levels':list(map(key,['b','a','c','unused'])),'limits':list(map(key,['c','b','a',None])) if t['limits']=='full' else None,'drop':t['drop'],'na_translate':t['translate']}
 if route=='policy':policy.setdefault('guide',{})['labels']={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 axis=axis.discrete_policy(policy).range(100.,540.).guide_geometry({'labels':'Preserve'}).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}})
 axis=axis.breaks_function({'operation':{'id':'example.positional_discrete_breaks','version':'1'},'parameters':t['mode']})
 if route=='policy' or t['label_mode']=='automatic':axis=axis.tick_format(None)
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if all(isinstance(v,(float,int)) for v in t['result']['range']):
  raw=t['result']['labels'];expected=[raw[i%len(raw)] if raw[i%len(raw)] is not None else ('NA' if t['label_mode']=='automatic' else '') for i in range(len(t['result']['values']))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 for family in ('auto','band','point'):
  for route in (('axis','policy') if t['label_mode']=='indexed' else ('axis',)):
   owned=[]
   try:
    d=data_for(t);owned.append(d);p=build(t,d,family,route);owned.append(p);wire=p.to_json()
    assert json.loads(wire)['version']==34
    restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
    for state in ('original','edited'):
     current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
     if current is not restored:owned.append(current)
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
     assert 'error' not in t['result'],t
     records.append({'index':index,'family':family,'route':route,'state':state,**check(t,frame)})
     if state=='original' and route=='axis' and family=='band' and t['population']=='ordinary' and t['limits']=='full' and t['drop'] and t['translate'] and t['mode'] in ('domain','mixed'):
      for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
    records.append({'index':index,'family':family,'route':route,'error':error.code})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==3600,len(records)
for family in ('auto','band','point'):
 for mode in ('domain','mixed'):
  for label_mode in ('indexed','automatic'):
   selected={t['population']:t for t in cases if t['limits']=='full' and t['drop'] and t['translate'] and t['mode']==mode and t['label_mode']==label_mode};owned=[]
   try:
    original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original,family,'axis');owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
    for population in ('constant','missing','all_missing','empty','ordinary'):
     t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
     fresh=build(t,replacement,family,'axis');owned.append(fresh);batch=fresh.chart();owned.append(batch);a_request=output.request(chart,options);owned.append(a_request);a_frame=a_request.prepare();owned.append(a_frame);b_request=output.request(batch,options);owned.append(b_request);b_frame=b_request.prepare();owned.append(b_frame);actual=check(t,a_frame);assert actual==check(t,b_frame);assert held.scene()==scene and p.to_json()==wire
     records.append({'family':family,'mode':mode,'label_mode':label_mode,'replacement':population,**actual})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==3660,len(records)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 3660 discrete positional break states and 12 publications.')
