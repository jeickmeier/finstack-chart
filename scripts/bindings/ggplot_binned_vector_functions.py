"""FIX-GG04: binned positional OOBs through actual hosts."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
joint='--joint' in sys.argv[3:]
cases=json.loads((ROOT/f"fixtures/parity/ggplot2/positional-pipeline-binned-{'compositions' if joint else 'vectors'}.json").read_text())['cases']
def configurations(t):return [('numeric',1)]
def mapping(t,m):return 'v'
def data_for(t,unit,m):
 values=[{'Infinity':float('inf'),'-Infinity':float('-inf')}.get(v,v) if isinstance(v,str) else v for v in t['inputs']];i=t['groups'] if t['kind']=='mean' else list(range(1,len(values)+1))
 return c.Data.columns({'v':c.column(values,kind='float64').nullable(True),'i':c.column(i,kind='int64')})
def layer(t,m):
 p=c.points().name('marks')
 return p.stat(c.summary().x(mapping(t,m)).group('i')).after_stat(c.stat_aes().x(1.).y('Mean')) if t['kind']=='mean' else p
def build(t,d,m):
 transform={'reverse':'Reverse','log10':{'Log':{'base':10.}}}.get(t['family'])
 scale=c.scale_binned({'bins':{'limits':[1.,10.] if t['limit_mode']=='explicit' else None,'breaks':{'Explicit':[1.,4.,10.]} if t['cuts']=='fixed' else {'Nice':3.},'right':True,'oob':'Squish'},'transform':transform})
 if joint and t['cuts']!='nice':scale=c.scale_binned({'bins':{'breaks':{'Nice':3.},'oob':'Squish','right':True},'transform':transform,'breaks_function':{'operation':{'id':'example.breaks_n','version':'1'},'parameters':'mixed_unnamed' if t['cuts']=='mixed' else 'domain'}})
 axis=(c.x_axis() if t['axis']=='x' else c.y_axis()).scale(scale).guide_geometry({'labels':'Preserve'}).oob_function({'operation':{'id':'example.scale_vector','version':'1'},'parameters':{'mode':'default' if t['mode']=='default' else 'oob_'+t['mode']}})
 if joint:axis=axis.limits_function({'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':t['control'],**({'bounds':[1.,10.]} if t['control']=='fixed' else {})}})
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x(mapping(t,m) if t['axis']=='x' else 'i').y('i' if t['axis']=='x' else mapping(t,m))).layer(layer(t,m))
 return (draft.x_axis(axis) if t['axis']=='x' else draft.y_axis(axis)).build()
def scalar(v):return v.get('number') if isinstance(v,dict) else v
def compare(a,b):
 if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert math.isclose(a,b,rel_tol=0,abs_tol=5e-14),(a,b)
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b),(a,b)
  for x,y in zip(a,b):compare(x,y)
 else:assert a==b,(a,b)
def check(t,chart,frame,m):
 semantics=chart.semantics();state=semantics['layers'][0];positions=state['point_positions'];dim=0 if t['axis']=='x' else 1
 compare([scalar(p[dim]) for p in positions],[v for v in t['result']['mapped'] if v is not None])
 compare([scalar(p[1-dim]) for p in positions],[1.]*len(positions) if t['kind']=='mean' else [i+1 for i,v in enumerate(t['result']['mapped']) if v is not None])
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Bottom' if dim==0 else 'Left'))['ticks']
 expected=[(v,label) for v,label in zip(t['result']['breaks'],t['result']['labels']) if isinstance(v,(int,float))]
 compare([tick['label'] for tick in ticks],[label or '' for _,label in expected])
 def tick_number(v):
  x=v['Number'];return -x if t['family']=='reverse' else math.log10(x) if t['family']=='log10' else x
 compare([tick_number(tick['value']) for tick in ticks],[v for v,_ in expected])
 return {'positions':positions,'ticks':ticks,'domains':state['domains']}
def sample(t,unit):
 return (t['cuts']=='domain' and t['control']=='fixed' if joint else t['cuts']=='nice') and t['population']=='ordinary' and t['kind'] in ('point_x','mean') and t['limit_mode']=='automatic' and t['mode'] in ('reverse','index')
for index,t in enumerate(cases):
 for unit,m in configurations(t):
  owned=[]
  try:
   d=data_for(t,unit,m);owned.append(d);p=build(t,d,m);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==40
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer(t,m)).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame);assert 'error' not in t['result'],t
    records.append({'index':index,'unit':unit,'state':state,**check(t,chart,frame,m)})
    if state=='original' and sample(t,unit):
     for fmt in ('svg','pdf','png'):(out/f"{t['family']}-{t['kind']}-{t['limit_mode']}-{t['mode']}.{fmt}").write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(index,unit,t,str(error));assert error.code in ('CHART_SCHEMA_CONFLICT','CHART_NUMERICAL_DOMAIN'),(index,error.code)
   records.append({'index':index,'unit':unit,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(2680 if joint else 1152),len(records)
def capture(t,chart,m,owned):
 try:
  request=output.request(chart,options);owned.append(request);frame=request.prepare();owned.append(frame)
  assert 'error' not in t['result'];return check(t,chart,frame,m)
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));return {'error':error.code}
for family in ('identity','reverse','log10'):
 for kind in ('point_x','mean'):
  selected={t['population']:t for t in cases if t['family']==family and t['kind']==kind and (t['control']=='fixed' if joint else t['limit_mode']=='explicit') and t['mode']=='index' and t['cuts']=='nice'}
  for unit,m in configurations(selected['ordinary']):
   owned=[]
   try:
    original=data_for(selected['ordinary'],unit,m);owned.append(original);p=build(selected['ordinary'],original,m);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
    for population in ('missing','empty'):
     t=selected[population];replacement=data_for(t,unit,m);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
     actual=capture(t,chart,m,owned)
     try:
      fresh=build(t,replacement,m);owned.append(fresh);batch=fresh.chart();owned.append(batch);expected=capture(t,batch,m,owned)
     except c.ChartError as error:
      assert 'error' in t['result'];expected={'error':error.code}
     assert actual==expected;assert p.to_json()==wire and held.scene()==scene
     records.append({'family':family,'kind':kind,'unit':unit,'replacement':population,**actual})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==(2692 if joint else 1164),len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));output.dispose();options.dispose();registry.dispose();print('PASS',len(records),'binned positional vector host states')
