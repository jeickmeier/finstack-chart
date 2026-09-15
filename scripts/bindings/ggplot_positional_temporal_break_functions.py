"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
replacements_only='--replacements-only' in sys.argv[3:]
zero='--zero-range' in sys.argv[3:]
overrides='--overrides' in sys.argv[3:]
cases=json.loads((ROOT/('fixtures/parity/ggplot2/positional-temporal-break-zero.json' if zero else 'fixtures/parity/ggplot2/positional-temporal-break-overrides.json' if overrides else 'fixtures/parity/ggplot2/positional-temporal-break-functions.json')).read_text())['cases']
units=[('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)]
def zone(name):
 return 'Utc' if name=='UTC' else {'Local':{'version':1,'zone':name,'revision':'1','tzdata':'explicit-US-2024','coverage':{'start':'1704067200000','end':'1735689600000'},'initial_offset_seconds':-18000,'transitions':[{'at_millis':'1710054000000','offset_seconds':-14400},{'at_millis':'1730613600000','offset_seconds':-18000}]}}
def data_for(t,unit,multiplier):
 values=t['inputs'];factor=multiplier*(86400 if t['kind']=='date' else 1)
 return c.Data.columns({'x':c.timestamps([int(v or 0)*factor for v in values],unit,t['zone']).validity([v is not None for v in values])},keys=list(range(100,100+len(values))))
def layer():return c.points().name('marks')
def build(t,d,variant,multiplier):
 date=t['kind']=='date';factor=multiplier*(86400 if date else 3600);origin=t['epoch']*multiplier
 scale=c.scale_date() if date else c.scale_utc() if t['zone']=='UTC' else c.scale_calendar({'domain':[str(origin),str(origin+3*factor)] if t['limits']=='full' else [],'unit':variant,'zone':zone(t['zone']),'range':[{'kind':'Number','value':0},{'kind':'Number','value':1}],'factory':{'kind':'Value','gamma':None},'clamp':False,'unknown':{'kind':'Missing'}})
 if t['limits']=='full' and t['zone']=='UTC':scale=scale.time_domain(origin,origin+3*factor)
 axis=c.x_axis().scale(scale).range(100.,540.).guide_geometry({'labels':'Preserve'}).breaks_function({'operation':{'id':'example.breaks_'+t['signature'],'version':'1'},'parameters':t['mode']}).tick_arguments({'count':3 if t['count']=='three' else 0 if t['count']=='zero' else None})
 if t['label_mode']=='indexed':axis=axis.tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'indexed'}})
 if t.get('control') in ('width','both'):axis=axis.tick_arguments({'width':'1 day' if date else '1 hour'})
 if t.get('control') in ('format','both'):axis=axis.tick_format({'GgplotTime':{'pattern':'%Y-%m-%d' if date else '%H:%M','locale':None}})
 if zero:axis=axis.expansion({'mult':[0.,0.],'add':[0.,0.]})
 return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x({'field':'x','origin':str(origin)}).y(1.)).layer(layer()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def check(t,frame):
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['ticks']
 if all(isinstance(v,(float,int)) for v in t['result']['range']):
  expected=[label or '' for v,label in zip(t['result']['breaks'],t['result']['labels']) if isinstance(v,(float,int))]
  assert [tick['label'] for tick in ticks]==expected,(t,ticks,expected)
 return {'ticks':ticks}
def sample(t):
 return t['population']=='spaced' and t['limits']=='none' and t['signature']=='n' and t['count']=='three' and t['mode']=='mixed' and t['label_mode']=='automatic' and (t['kind']=='date' or t['epoch']==1710046800 or t['zone']=='America/New_York')
for index,t in ([] if replacements_only else enumerate(cases)):
 for unit,variant,multiplier in units:
  owned=[]
  try:
   d=data_for(t,unit,multiplier);owned.append(d);p=build(t,d,variant,multiplier);owned.append(p);wire=p.to_json()
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame);actual=check(t,frame);assert 'error' not in t['result'],t
    records.append({'index':index,'unit':unit,'state':state,**actual})
    if state=='original' and unit=='ms' and sample(t):
     for fmt in ('svg','pdf','png'):(out/f"sample-{index}.{fmt}").write_bytes(frame.export(fmt))
  except c.ChartError as error:
   assert 'error' in t['result'],(t,str(error));assert error.code==('CHART_SCHEMA_CONFLICT' if 'labels' in t['result']['error'] else 'CHART_VALIDATION'),(t,error.code)
   records.append({'index':index,'unit':unit,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(0 if replacements_only else 400 if zero else 1280 if overrides else 27000),len(records)
def comparable(chart):
 state=chart.semantics();domains=state['layers'][0]['domains'];space=domains.get('x_space')
 if space and 'Timestamp' in space:
  origin=int(space['Timestamp']['origin']);space['Timestamp']['origin']='0'
  if domains.get('x'):
   domains['x']={key:str(origin+int(value)) for key,value in domains['x'].items()}
 return {'limits':state.get('positional_limits'),'domains':domains}
for kind,zone_name,epoch in [('date','UTC',1704067200),('datetime','UTC',1710046800),('datetime','UTC',1730606400),('datetime','America/New_York',1710046800),('datetime','America/New_York',1730606400)]:
 if overrides or zero:break
 selected={t['population']:t for t in cases if t['kind']==kind and t['zone']==zone_name and t['epoch']==epoch and t['limits']=='none' and t['signature']=='n' and t['count']=='three' and t['mode']=='domain' and t['label_mode']=='indexed'};owned=[]
 try:
  original=data_for(selected['spaced'],'ms',1000);owned.append(original);p=build(selected['spaced'],original,'Milliseconds',1000);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('constant','missing','all_missing','empty','spaced'):
   t=selected[population];replacement=data_for(t,'ms',1000);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement,'Milliseconds',1000);owned.append(fresh);batch=fresh.chart();owned.append(batch)
   actual=comparable(chart);assert actual==comparable(batch),(kind,zone_name,population,actual,comparable(batch));assert held.scene()==scene and p.to_json()==wire
   def render(current):
    try:
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame);assert 'error' not in t['result'];return check(t,frame)
    except c.ChartError as error:
     assert 'error' in t['result'] and error.code=='CHART_SCHEMA_CONFLICT';return {'error':error.code}
   laid=render(chart);assert laid==render(batch)
   records.append({'kind':kind,'zone':zone_name,'epoch':epoch,'replacement':population,'semantics':actual,**laid})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(25 if replacements_only else 400 if zero else 1280 if overrides else 27025)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} temporal positional break states; overrides={overrides}.')
