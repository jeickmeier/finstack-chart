"""FIX-GG04: non-color binned guide labels through the actual Python adapter."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/noncolor-numeric-label-functions.json').read_text())['cases'] if t['kind']=='binned']
def descriptor(t):
 channel=t['channel'];family={'Pow':{'exponent':.5}} if t['transform']=='sqrt' else {'Log':{'base':10}} if t['transform']=='log10' else 'Linear'
 breaks={'Nice':5} if t['break_mode']=='auto' else {'Explicit':[] if t['break_mode']=='empty' else [-1,0,1,1,3,20,{'number':'Infinity'},{'number':'NaN'}]}
 labels='Automatic' if t['label_mode']=='default' else {'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':family,'domain':[0,1],'reverse':t['transform']=='reverse','rescaler':'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[.1,1.] if channel=='alpha' else [1.,6.],'exponent':.5 if channel=='size' else 1.,'absolute':False}},'unknown':{'kind':'Missing'}}},'ggplot':{'Binned':{'limits':[1,10] if t['limits']=='full' else None,'oob':'Squish','right':True,'breaks':breaks}},'guide':{'Binned':labels}}
def layer(t):
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth','shape':'Shape','linetype':'LineType'}[t['channel']]
 result=(c.line() if t['channel'] in ('linewidth','linetype') else c.points()).name('marks')
 return result.value_scale(channel,'v',descriptor(t)) if t['channel'] in ('shape','linetype') else result.numeric_scale(channel,'v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='float64').nullable(True)})
def build(t,d):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0];entries=[e for group in state.get('numeric_value_guides',{}).values() for e in group if e['visible']]
 actual={'values':[float(e['transformed']['number']) if isinstance(e['transformed'],dict) else e['transformed'] for e in entries],'labels':[e['label'] for e in entries]}
 expected={'values':t['result'].get('boundaries',[]),'labels':next(iter(t['result'].get('keys',[])),{'labels':[]})['labels']}
 assert actual['labels']==expected['labels'],(t,actual,expected)
 assert len(actual['values'])==len(expected['values']),(t,actual,expected)
 assert all(math.isclose(a,b,rel_tol=3e-12,abs_tol=3e-12) for a,b in zip(actual['values'],expected['values'])),(t,actual,expected)
 return actual
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert t['label_mode']=='default' or json.loads(wire)['version']==32
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual})
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code in ('CHART_VALIDATION','CHART_NUMERICAL_DOMAIN'),(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==2739,len(records)
for channel in ('size','alpha','linewidth'):
 selected={t['population']:t for t in cases if t['channel']==channel and t['limits']=='none' and t['transform']=='identity' and t['break_mode']=='auto' and t['label_mode']=='indexed'};owned=[]
 try:
  original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json()
  for population in ('missing','empty','ordinary'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire
   records.append({'channel':channel,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==2748
(out/'records.json').write_text(json.dumps(records,indent=2));registry.dispose()
print('PASS Python: 2748 non-color binned label states, including 9 replacements.')
