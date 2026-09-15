"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
joint="--joint" in sys.argv
major_kind="function_fixed" if joint else "automatic"
base_count=2858 if joint else 5716
cases=json.loads((ROOT/('fixtures/parity/ggplot2/positional-minor-break-joint-functions.json' if joint else 'fixtures/parity/ggplot2/positional-minor-break-functions.json')).read_text())['cases']
def data_for(t):return c.Data.columns({'x':c.column([v or 0. for v in t['inputs']],kind='float64').validity([v is not None for v in t['inputs']])},keys=list(range(100,100+len(t['inputs']))))
def layer():return c.points().name('marks')
def build(t,d):
 scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[t['transform']]()
 if t['limits']=='full':scale=scale.domain(1.,10.)
 axis=c.x_axis().scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).minor_breaks({'Registered':{'operation':{'id':'example.breaks_minor_'+t['signature'],'version':'1'},'parameters':t['mode']}})
 if t['major'].startswith('function_'):axis=axis.breaks_function({'operation':{'id':'example.breaks_limits','version':'1'},'parameters':t['major'].removeprefix('function_')})
 elif t['major']=='explicit':axis=axis.tick_values([{'Number':v} for v in [10.,5.,1.,1.,{'number':'NaN'},{'number':'Infinity'},{'number':'-Infinity'}]])
 elif t['major']=='empty':axis=axis.tick_values([])
 elif t['major']=='null':axis=axis.ticks([])
 if t['expand']=='zero':axis=axis.expansion({'mult':[0.,0.],'add':[0.,0.]})
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom');ticks=guide['ticks'];minor=guide.get('minor_ticks',[])
 finite=all(isinstance(v,(float,int)) for v in t['result']['range'])
 expected=[label or '' for v,label in zip(t['result']['major'],t['result']['labels']) if isinstance(v,(float,int)) and finite]
 assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 expected=[v for v in t['result']['minor'] if isinstance(v,(float,int)) and finite];assert len(minor)==len(expected),(t,minor,expected)
 forward={'identity':lambda x:x,'sqrt':math.sqrt,'log10':math.log10,'reverse':lambda x:-x}[t['transform']]
 for tick,want in zip(minor,expected):
  value=forward(tick['value']['Number']);assert abs(value-want)<=3e-12*max(1.,abs(want)),(t,tick,want)
  a,b=t['result']['range'];position=100.+440.*(want-a)/(b-a);assert abs(tick['position']-position)<1e-8,(t,tick,position)
 return {'ticks':ticks,'minor_ticks':minor}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==36
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
   assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**check(t,frame)})
   if state=='original' and t['population']=='spaced' and t['limits']=='none' and t['signature']=='two' and t['major']==major_kind and t['expand']=='default' and t['mode']=='mixed':
    for fmt in ('svg','pdf','png'):(out/f'sample-{index}.{fmt}').write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==base_count,len(records)
for transform in ('identity','sqrt','log10','reverse'):
 for mode in ('domain','mixed'):
  selected={t['population']:t for t in cases if t['transform']==transform and t['limits']=='full' and t['signature']=='two' and t['major']==major_kind and t['expand']=='default' and t['mode']==mode};owned=[]
  try:
   original=data_for(selected['spaced']);owned.append(original);p=build(selected['spaced'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();assert json.loads(wire)['version']==36;request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('constant','missing','all_missing','empty','spaced'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch)
    a_request=output.request(chart,options);owned.append(a_request);a_frame=a_request.prepare();owned.append(a_frame);b_request=output.request(batch,options);owned.append(b_request);b_frame=b_request.prepare();owned.append(b_frame);actual=check(t,a_frame);assert actual==check(t,b_frame);assert held.scene()==scene and p.to_json()==wire
    records.append({'transform':transform,'mode':mode,'replacement':population,'semantics':actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==base_count+40
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} numeric minor callback states.')
