"""FIX-GG04: temporal positional OOBs through actual hosts in four timestamp units."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/positional-pipeline-temporal.json').read_text())['cases']
units=[('s',1),('ms',1000),('us',1000000),('ns',1000000000)]
def configurations(t):return units[:1] if t['family']=='duration' else units
def factor(t,m):return m*(86400 if t['family']=='date' else 1)
def origin(t,m):return 0 if t['family']=='duration' else t['origin']*factor(t,m)
def mapping(t,m):return 'v' if t['family']=='duration' else {'field':'v','origin':str(origin(t,m))}
def data_for(t,unit,m):
 values=t['inputs'];i=t['groups'] if t['kind']=='mean' else list(range(1,len(values)+1))
 v=c.column(values,kind='float64').nullable(True) if t['family']=='duration' else c.timestamps([int(v or 0)*factor(t,m) for v in values],unit,'UTC').validity([v is not None for v in values])
 return c.Data.columns({'v':v,'i':c.column(i,kind='int64')})
def layer(t,m):
 p=c.points().name('marks')
 return p.stat(c.summary().x(mapping(t,m)).group('i')).after_stat(c.stat_aes().x(1.).y('Mean')) if t['kind']=='mean' else p
def build(t,d,m):
 scale={'duration':c.scale_duration,'date':c.scale_date,'datetime':c.scale_utc}[t['family']]()
 if t['limit_mode']=='explicit':
  scale=scale.domain(t['origin']+1.,t['origin']+10.) if t['family']=='duration' else scale.time_domain((t['origin']+1)*factor(t,m),(t['origin']+10)*factor(t,m))
 axis=(c.x_axis() if t['axis']=='x' else c.y_axis()).scale(scale).guide_geometry({'labels':'Preserve'}).oob_function({'operation':{'id':'example.scale_vector','version':'1'},'parameters':{'mode':'default' if t['mode']=='default' else 'oob_'+t['mode']}})
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x(mapping(t,m) if t['axis']=='x' else 'i').y('i' if t['axis']=='x' else mapping(t,m))).layer(layer(t,m))
 return (draft.x_axis(axis) if t['axis']=='x' else draft.y_axis(axis)).build()
def scalar(v):return v.get('number') if isinstance(v,dict) else v
def absolute(t,m,v):return v if t['family']=='duration' else origin(t,m)/factor(t,m)+v/factor(t,m)
def compare(a,b):
 if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert math.isclose(a,b,rel_tol=0,abs_tol=5e-14),(a,b)
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b),(a,b)
  for x,y in zip(a,b):compare(x,y)
 else:assert a==b,(a,b)
def check(t,chart,frame,m):
 semantics=chart.semantics();state=semantics['layers'][0];positions=state['point_positions'];dim=0 if t['axis']=='x' else 1
 compare([absolute(t,m,scalar(p[dim])) for p in positions],[v for v in t['result']['mapped'] if v is not None])
 compare([scalar(p[1-dim]) for p in positions],[1.]*len(positions) if t['kind']=='mean' else [i+1 for i,v in enumerate(t['result']['mapped']) if v is not None])
 limits=semantics['positional_limits'][str(dim)];compare([absolute(t,m,scalar(v)) for v in limits],t['result']['limits'])
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Bottom' if dim==0 else 'Left'))['ticks']
 expected=[(v,label) for v,label in zip(t['result']['breaks'],t['result']['labels']) if isinstance(v,(int,float))]
 compare([tick['label'] for tick in ticks],[label or '' for _,label in expected])
 def tick_number(v):
  if 'Number' in v:return absolute(t,m,v['Number'])
  stamp=v['Timestamp'];return int(stamp['value'])/({'Seconds':1,'Milliseconds':1000,'Microseconds':1e6,'Nanoseconds':1e9}[stamp['unit']]*(86400 if t['family']=='date' else 1))
 compare([tick_number(tick['value']) for tick in ticks],[v for v,_ in expected])
 return {'positions':positions,'limits':limits,'ticks':ticks,'domains':state['domains']}
def sample(t,unit):
 return unit==('s' if t['family']=='duration' else 'ms') and t['population']=='ordinary' and ((t['kind']=='point_x' and ((t['limit_mode']=='explicit' and t['mode']=='default') or (t['limit_mode']=='automatic' and t['mode']=='index'))) or (t['kind']=='mean' and t['limit_mode']=='automatic' and t['mode'] in ('reverse','index')))
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
assert len(records)==1691,len(records)
for family in ('duration','date','datetime'):
 for kind in ('point_x','mean'):
  selected={t['population']:t for t in cases if t['family']==family and t['kind']==kind and t['limit_mode']=='explicit' and t['mode']=='index'}
  for unit,m in configurations(selected['ordinary']):
   owned=[]
   try:
    original=data_for(selected['ordinary'],unit,m);owned.append(original);p=build(selected['ordinary'],original,m);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
    for population in ('missing','empty'):
     t=selected[population];replacement=data_for(t,unit,m);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
     fresh=build(t,replacement,m);owned.append(fresh);batch=fresh.chart();owned.append(batch);ar=output.request(chart,options);owned.append(ar);af=ar.prepare();owned.append(af);br=output.request(batch,options);owned.append(br);bf=br.prepare();owned.append(bf);actual=check(t,chart,af,m);assert actual==check(t,batch,bf,m);assert p.to_json()==wire and held.scene()==scene
     records.append({'family':family,'kind':kind,'unit':unit,'replacement':population,**actual})
   finally:
    for obj in reversed(owned):obj.dispose()
assert len(records)==1727,len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));output.dispose();options.dispose();registry.dispose();print('PASS',len(records),'temporal positional vector host states')
