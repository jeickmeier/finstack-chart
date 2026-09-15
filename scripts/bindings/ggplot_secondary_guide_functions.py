"""FIX-GG04: secondary callbacks through actual hosts, including timestamp units."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/secondary-guide-functions.json').read_text())['cases']
units=[('s',1),('ms',1000),('us',1000000),('ns',1000000000)]
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def temporal(t):return t['family'] in ('date','datetime')
def factor(t,m):return m*(86400 if t['family']=='date' else 1)
def data(t,unit,m):
 v=t['inputs'];x=c.categorical([x or '' for x in v]).validity([x is not None for x in v]) if t['family']=='discrete' else c.timestamps([round((x or 0)*factor(t,m)) for x in v],unit,'UTC').validity([x is not None for x in v]) if temporal(t) else c.column(v,kind='float64').nullable(True)
 return c.Data.columns({'x':x,'y':c.column([1.]*len(v),kind='float64')},keys=list(range(100,100+len(v))),name='data')
def build(t,d):
 scale=c.scale_date() if t['family']=='date' else c.scale_utc() if t['family']=='datetime' else c.scale_band() if t['family']=='discrete' else c.scale_linear()
 primary=c.x_axis().scale(scale).range(100,540)
 if not t['expand']:primary=primary.expansion({'mult':[0,0],'add':[0,0]})
 second=c.x_axis().name('secondary').side('Top')
 if t['conversion']=='square':
  transform={'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform','version':'1'},'parameters':{'family':'square','custom':False}}}}}
  second=second.secondary_transform('x',{'compatibility':'D3','family':{'Ggplot':{'transform':transform}},'domain':[0,20],'range':[0,400],'clamp':False,'round':False,'unknown':{'kind':'Missing'}})
 else:second=second.secondary('x',2. if t['conversion']=='affine' else 1.,3. if t['conversion']=='affine' else 2. if t['conversion']=='shift' else 0.)
 mode='empty' if t['mode']=='typed_empty' else 'untyped_empty' if t['mode']=='empty' and temporal(t) else t['mode']
 second=second.breaks_function({'operation':{'id':'example.breaks_limits','version':'1'},'parameters':mode}).guide_geometry({'labels':'Preserve'})
 if t['label_mode']=='function':second=second.tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'indexed'}})
 return c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).aes(c.aes().x({'field':'x','origin':'0'} if temporal(t) else 'x').y('y')).layer(c.points().name('marks')).x_axis(primary).x_axis(second).build()
def check(t,m,frame):
 e=t['result'];assert 'error' not in e,t
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Top')['ticks'];assert len(ticks)==len(e['positions']),(t,ticks)
 for i,tick in enumerate(ticks):
  assert math.isclose((tick['position']-100)/440,e['positions'][i],rel_tol=0,abs_tol=2e-12),(t,tick,e)
  assert tick['label']==('NA' if e['labels'][i] is None else e['labels'][i]),(t,tick,e)
  value=tick['value'];actual=int(value['Timestamp']['value'])/factor(t,m) if 'Timestamp' in value else value['Number']/factor(t,m)+(2. if t['conversion']=='shift' else 0.) if temporal(t) else value['Number']
  assert math.isclose(actual,e['user_values'][i],rel_tol=0,abs_tol=2e-12),(t,tick,actual,e)
 return {'ticks':ticks}
for index,t in enumerate(cases):
 for unit,m in units if temporal(t) else [('ms',1000)]:
  owned=[]
  try:
   d=data(t,unit,m);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==62
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for body,reg in ((wire,None),(json.dumps({**json.loads(wire),'version':61}),registry)):
    try:c.Plot.from_json(body,reg)
    except c.ChartError:pass
    else:raise AssertionError('missing registry or downgrade accepted')
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',c.points().name('marks')).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame);records.append({'index':index,'unit':unit,'state':state,**check(t,m,frame)});assert restored.to_json()==wire
    if state=='original' and unit=='ms' and t['label_mode']=='function' and t['mode']=='domain' and t['expand'] and (t['population']=='ordinary' or t['family']=='numeric' and t['conversion']=='square' and t['population']=='empty'):
     for fmt in ('svg','pdf','png'):(out/f"{t['family']}-{t['conversion']}-{t['population']}.{fmt}").write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(t,error);records.append({'index':index,'unit':unit,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==1190,len(records)
for family in ('numeric','date','datetime'):
 for conversion in (('identity','affine','square') if family=='numeric' else ('identity','shift')):
  selected={t['population']:t for t in cases if t['family']==family and t['conversion']==conversion and t['mode']=='domain' and t['label_mode']=='function' and t['expand']}
  for unit,m in units if family!='numeric' else [('ms',1000)]:
   owned=[]
   try:
    original=data(selected['ordinary'],unit,m);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart)
    request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene();wire=p.to_json()
    for population in ('constant','missing','empty','ordinary'):
     t=selected[population];replacement=data(t,unit,m);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
     fresh=build(t,replacement);owned.append(fresh);ar=output.request(chart,options);br=output.request(fresh,options);owned.extend([ar,br]);af=ar.prepare();bf=br.prepare();owned.extend([af,bf]);actual=check(t,m,af);assert actual==check(t,m,bf)
     assert held.scene()==scene and p.to_json()==wire;records.append({'family':family,'conversion':conversion,'unit':unit,'replacement':population,**actual})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==1266,len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS 1266 secondary callback states and 24 publication files')
