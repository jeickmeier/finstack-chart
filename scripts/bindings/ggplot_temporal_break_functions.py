"""FIX-GG04 temporal break functions through the actual Python adapter."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
default_names='--default-names' in sys.argv[3:]
overrides=len(sys.argv)>3 and sys.argv[3]=='overrides'
cases=json.loads((ROOT/('fixtures/parity/ggplot2/temporal-break-default-overrides.json' if default_names and overrides else 'fixtures/parity/ggplot2/temporal-break-default-names.json' if default_names else 'fixtures/parity/ggplot2/temporal-break-overrides.json' if overrides else 'fixtures/parity/ggplot2/temporal-break-functions.json')).read_text())['cases']
units=[('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)]
def zone(name):
 return 'Utc' if name=='UTC' else {'Local':{'version':1,'zone':name,'revision':'1','tzdata':'explicit-US-2024','coverage':{'start':'1704067200000','end':'1735689600000'},'initial_offset_seconds':-18000,'transitions':[{'at_millis':'1710054000000','offset_seconds':-14400},{'at_millis':'1730613600000','offset_seconds':-18000}]}}
def descriptor(t,variant,multiplier):
 date=t['kind']=='date';origin=t['epoch']*multiplier;factor=multiplier*(86400 if date else 3600)
 labels='Automatic' if default_names else {'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'indexed'}}
 args={'date':date,'breaks':{'Width':'1 day' if date else '1 hour'} if t.get('control') in ('width','both') else 'Automatic','count':3 if t['count']=='three' else 0 if t['count']=='zero' else None,'labels':labels,'format':{'pattern':'%Y-%m-%d' if date else '%H:%M','locale':None} if t.get('control') in ('format','both') else None}
 palette={'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}} if t['channel']=='colour' else {'operation':'PowerRange','range':[1.,6.],'exponent':.5,'absolute':False}
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range','timestamp':{'origin':str(origin),'unit':variant,'date':date}}},'output':{'Interpolate':palette},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[0,3*factor] if t['limits']=='full' else None,'oob':'Censor'}},'breaks_function':{'operation':{'id':'example.breaks_'+t['signature'],'version':'1'},'parameters':t['mode']},'guide':{'Temporal':{'origin':str(origin),'unit':variant,'zone':zone(t['zone']),'arguments':args}}}
def data_for(t,unit,multiplier):
 values=t['inputs'];factor=multiplier*(86400 if t['kind']=='date' else 1)
 return c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.timestamps([int(v or 0)*factor for v in values],unit,t['zone']).validity([v is not None for v in values])},keys=list(range(100,100+len(values))))
def layer(t,variant,multiplier):
 if t['channel']=='colour':return c.points().name('marks')
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth'}[t['channel']]
 return (c.line() if t['channel']=='linewidth' else c.points()).name('marks').numeric_scale(channel,{'field':'v','origin':str(t['epoch']*multiplier)},descriptor(t,variant,multiplier))
def build(t,d,variant,multiplier):
 builder=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3')
 if t['channel']=='colour':return builder.aes(c.aes().x('x').y(1.).color({'field':'v','origin':str(t['epoch']*multiplier)}).color_scale('v')).scale(c.color_mapped('v',descriptor(t,variant,multiplier))).layer(layer(t,variant,multiplier)).build()
 return builder.aes(c.aes().x('x').y(1.)).layer(layer(t,variant,multiplier)).build()
def check(t,chart,multiplier):
 state=chart.semantics()['layers'][0]
 if t['channel']=='colour':
  legend=state.get('color_legend');entries=legend['entries'] if legend else []
  expected=[v for g in t['result'].get('keys',[]) for v in g['labels']]
  assert [e[0] for e in entries]==expected,(t,entries,expected)
  return {'entries':entries,'styles':state.get('styles',[])}
 entries=[e for group in state.get('numeric_value_guides',{}).values() for e in group if e['visible']]
 factor=multiplier*(86400 if t['kind']=='date' else 1);origin=t['epoch']*multiplier
 actual={'values':[origin/factor+(float(e['transformed']['number']) if isinstance(e['transformed'],dict) else e['transformed'])/factor for e in entries],'labels':[e['label'] for e in entries]}
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
 assert actual==expected,(t,actual,expected)
 return actual
for index,t in enumerate(cases):
 for unit,variant,multiplier in units:
  owned=[]
  try:
   d=data_for(t,unit,multiplier);owned.append(d);p=build(t,d,variant,multiplier);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==33
   restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer(t,variant,multiplier)).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);actual=check(t,chart,multiplier);assert 'error' not in t['result'],t
    records.append({'index':index,'unit':unit,'state':state,**actual})
  except c.ChartError as error:
   assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
   records.append({'index':index,'unit':unit,'error':error.code})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(240 if overrides else 800 if default_names else 30240),len(records)
for channel,kind,zone_name,epoch in [(channel,*scenario) for channel in ('colour','size') for scenario in [('date','UTC',1704067200),('datetime','UTC',1710046800),('datetime','UTC',1730606400),('datetime','America/New_York',1710046800),('datetime','America/New_York',1730606400)]]:
 if overrides or default_names:break
 selected={t['population']:t for t in cases if t['channel']==channel and t['kind']==kind and t['zone']==zone_name and t['epoch']==epoch and t['limits']=='none' and t['signature']=='n' and t['count']=='three' and t['mode']=='domain'};owned=[]
 try:
  original=data_for(selected['spaced'],'ms',1000);owned.append(original);p=build(selected['spaced'],original,'Milliseconds',1000);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('constant','missing','empty','spaced'):
   t=selected[population];replacement=data_for(t,'ms',1000);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement,'Milliseconds',1000);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart,1000);assert actual==check(t,batch,1000)
   assert held.scene()==scene and p.to_json()==wire;records.append({'channel':channel,'kind':kind,'zone':zone_name,'epoch':epoch,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(240 if overrides else 800 if default_names else 30280)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} temporal break function states; overrides={overrides}.')
