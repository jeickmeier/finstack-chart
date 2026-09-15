"""FIX-GG04 non-color temporal labels through the actual Python adapter."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/noncolor-temporal-label-functions.json').read_text())['cases']
units=[('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)]
def zone(name):
 return 'Utc' if name=='UTC' else {'Local':{'version':1,'zone':name,'revision':'1','tzdata':'explicit-US-2024','coverage':{'start':'1704067200000','end':'1735689600000'},'initial_offset_seconds':-18000,'transitions':[{'at_millis':'1710054000000','offset_seconds':-14400},{'at_millis':'1730613600000','offset_seconds':-18000}]}}
def descriptor(t,variant,multiplier):
 date=t['kind']=='date';origin=t['epoch']*multiplier;factor=multiplier*(86400 if date else 3600)
 labels={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 args={'date':date,'breaks':{'Explicit':[-factor,0,factor,3*factor,4*factor,{'number':'NaN'}]} if t['control']=='explicit' else {'Explicit':[]} if t['control']=='empty' else 'Automatic','labels':labels,'format':{'pattern':'%Y-%m-%d' if date else '%H:%M','locale':None} if t['control']=='format' else None}
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range','timestamp':{'origin':str(origin),'unit':variant,'date':date}}},'output':{'Interpolate':{'operation':'PowerRange','range':[.1,1.] if t['channel']=='alpha' else [1.,6.],'exponent':.5 if t['channel']=='size' else 1.,'absolute':False}},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':None,'oob':'Censor'}},'guide':{'Temporal':{'origin':str(origin),'unit':variant,'zone':zone(t['zone']),'arguments':args}}}
def data_for(t,unit,multiplier):
 values=t['inputs'];factor=multiplier*(86400 if t['kind']=='date' else 1)
 return c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.timestamps([int(v or 0)*factor for v in values],unit,t['zone']).validity([v is not None for v in values])},keys=list(range(100,100+len(values))))
def layer(t,variant,multiplier):
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth'}[t['channel']]
 return (c.line() if t['channel']=='linewidth' else c.points()).name('marks').numeric_scale(channel,{'field':'v','origin':str(t['epoch']*multiplier)},descriptor(t,variant,multiplier))
def build(t,d,variant,multiplier):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer(t,variant,multiplier)).build()
def check(t,chart,multiplier):
 state=chart.semantics()['layers'][0];entries=[e for group in state.get('numeric_value_guides',{}).values() for e in group if e['visible']]
 factor=multiplier*(86400 if t['kind']=='date' else 1);origin=t['epoch']*multiplier
 actual={'values':[origin/factor+(float(e['transformed']['number']) if isinstance(e['transformed'],dict) else e['transformed'])/factor for e in entries],'labels':[e['label'] for e in entries]}
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
 assert actual==expected,(t,actual,expected)
 return actual
for index,t in enumerate(cases):
 for unit,variant,multiplier in units:
  owned=[]
  try:
   d=data_for(t,unit,multiplier);owned.append(d);p=build(t,d,variant,multiplier);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==32
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer(t,variant,multiplier)).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);actual=check(t,chart,multiplier);assert 'error' not in t['result'],t
    records.append({'index':index,'unit':unit,'state':state,**actual})
  except c.ChartError as error:
   assert 'error' in t['result'],(t,str(error));assert error.code==('CHART_VALIDATION' if 'labels' in t['result']['error'] else 'CHART_NUMERICAL_DOMAIN'),(t,error.code)
   records.append({'index':index,'unit':unit,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==8340,len(records)
for channel,kind,zone_name,epoch in [(channel,*scenario) for channel in ('size','alpha','linewidth') for scenario in [('date','UTC',1704067200),('datetime','UTC',1710046800),('datetime','UTC',1730606400),('datetime','America/New_York',1710046800),('datetime','America/New_York',1730606400)]]:
 selected={t['population']:t for t in cases if t['channel']==channel and t['kind']==kind and t['zone']==zone_name and t['epoch']==epoch and t['control']=='automatic' and t['label_mode']=='indexed'};owned=[]
 try:
  original=data_for(selected['spaced'],'ms',1000);owned.append(original);p=build(selected['spaced'],original,'Milliseconds',1000);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('constant','missing','empty','spaced'):
   t=selected[population];replacement=data_for(t,'ms',1000);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement,'Milliseconds',1000);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart,1000);assert actual==check(t,batch,1000)
   assert held.scene()==scene and p.to_json()==wire;records.append({'channel':channel,'kind':kind,'zone':zone_name,'epoch':epoch,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==8400
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 8400 non-color temporal label states, including 60 replacements.')
