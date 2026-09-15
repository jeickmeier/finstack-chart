"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-other-minor-break-functions.json').read_text())['cases']
def data_for(t):
 values=c.categorical([v or '' for v in t['inputs']]) if t['family']=='discrete' else c.column([v or 0. for v in t['inputs']],kind='float64')
 return c.Data.columns({'x':values.validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def build(t,d,family):
 axis=c.x_axis().scale(c.scale_binned({}) if family=='binned' else c.scale_band() if family=='band' else c.scale_point())
 if family!='binned':axis=axis.discrete_policy({})
 axis=axis.range(100.,540.).guide_geometry({'labels':'Preserve'}).minor_breaks({'Registered':{'operation':{'id':'example.breaks_minor_'+t['signature'],'version':'1'},'parameters':'outside' if t['mode']=='mixed' else t['mode']}})
 if t['major']=='empty':axis=axis.tick_values([])
 elif t['major']=='null':axis=axis.ticks([])
 if t['expand']=='zero':axis=axis.expansion({'mult':[0.,0.],'add':[0.,0.]})
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom');ticks=guide['ticks'];minor=guide.get('minor_ticks',[])
 finite=all(isinstance(v,(float,int)) for v in t['result']['range'])
 expected=[v if v is not None else 'NA' for v in t['result']['labels']]
 assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 expected=[v for v in t['result']['minor'] if isinstance(v,(float,int)) and finite];assert len(minor)==len(expected),(t,minor,expected)
 for tick,want in zip(minor,expected):
  assert tick['value']['Number']==want,(t,tick,want)
  a,b=t['result']['range'];position=320. if a==b else 100.+440.*(want-a)/(b-a);assert abs(tick['position']-position)<1e-8,(t,tick,position)
 return {'ticks':ticks,'minor_ticks':minor}
for index,t in enumerate(cases):
 for family in (('band','point') if t['family']=='discrete' else ('binned',)):
  owned=[]
  try:
   d=data_for(t);owned.append(d);p=build(t,d,family);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==36
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    assert 'error' not in t['result'],t
    records.append({'index':index,'family':family,'state':state,**check(t,frame)})
    if state=='original' and t['population'] in ('spaced','constant') and t['signature']=='two' and t['major']=='automatic' and t['expand']=='default' and t['mode']=='mixed':
     for fmt in ('svg','pdf','png'):(out/f'sample-{family}-{index}.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_UNSUPPORTED_CAPABILITY',(t,error.code)
   records.append({'index':index,'family':family,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==1500,len(records)
for family in ('band','point'):
 for mode in ('domain','mixed'):
  selected={t['population']:t for t in cases if t['family']=='discrete' and t['signature']=='two' and t['major']=='automatic' and t['expand']=='default' and t['mode']==mode};owned=[]
  try:
   original=data_for(selected['spaced']);owned.append(original);p=build(selected['spaced'],original,family);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();assert json.loads(wire)['version']==36;request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('constant','missing','all_missing','empty','spaced'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement,family);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    a_request=output.request(chart,options);owned.append(a_request);a_frame=a_request.prepare();owned.append(a_frame);b_request=output.request(batch,options);owned.append(b_request);b_frame=b_request.prepare();owned.append(b_frame);actual=check(t,a_frame);assert actual==check(t,b_frame);assert held.scene()==scene and p.to_json()==wire
    records.append({'family':family,'mode':mode,'replacement':population,'semantics':actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==1520
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} discrete minor callback and binned exclusion states.')
