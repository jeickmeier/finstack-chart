"""FIX-GG04: continuous break functions through the actual Python adapter."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
default_names="--default-names" in sys.argv[3:]
cases=json.loads((ROOT/('fixtures/parity/ggplot2/numeric-break-default-names.json' if default_names else 'fixtures/parity/ggplot2/numeric-break-functions.json')).read_text())['cases']
def descriptor(t):
 family={'Pow':{'exponent':.5}} if t['transform']=='sqrt' else {'Log':{'base':10}} if t['transform']=='log10' else 'Linear'
 output={'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}} if t['channel']=='colour' else {'Interpolate':{'operation':'PowerRange','range':[1.,6.],'exponent':.5,'absolute':False}}
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':family,'domain':[0,1],'reverse':t['transform']=='reverse','rescaler':'Range'}},'output':output,'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[1,10] if t['limits']=='full' else None,'oob':'Censor'}},'breaks_function':{'operation':{'id':'example.breaks_'+t['signature'],'version':'1'},'parameters':t['mode']},'guide':{'Continuous':{'count':{'none':None,'three':3,'zero':0}[t['count']],'labels':('Automatic' if default_names else {'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'indexed'}})}}}
def layer(t):
 result=c.points().name('marks')
 return result if t['channel']=='colour' else result.numeric_scale('Size','v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='float64').nullable(True)})
def build(t,d):
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3')
 if t['channel']=='colour':return draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t))).layer(layer(t)).build()
 return draft.aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0]
 if t['channel']=='colour':
  entries=(state.get('color_legend') or {}).get('entries',[]);actual={'labels':[e[0] for e in entries]}
  expected={'labels':[v for g in t['result'].get('keys',[]) for v in g['labels']]}
 else:
  entries=[e for group in state.get('numeric_value_guides',{}).values() for e in group if e['visible']]
  actual={'values':[e['transformed'] for e in entries],'labels':[e['label'] for e in entries]}
  expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
  assert len(actual['values'])==len(expected['values']),(t,actual,expected)
  assert all(math.isclose(a,b,rel_tol=3e-12,abs_tol=3e-12) for a,b in zip(actual['values'],expected['values'])),(t,actual,expected)
 assert actual['labels']==expected['labels'],(t,actual,expected)
 return actual
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==33
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual})
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_NUMERICAL_DOMAIN',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(160 if default_names else 5328),len(records)
for channel in (() if default_names else ('size','colour')):
 selected={t['population']:t for t in cases if t['channel']==channel and t['limits']=='none' and t['transform']=='identity' and t['mode']=='mixed' and t['signature']=='n' and t['count']=='three'};owned=[]
 try:
  original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json()
  for population in ('missing','all_missing','empty','ordinary'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire
   records.append({'channel':channel,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(160 if default_names else 5336)
(out/'records.json').write_text(json.dumps(records,indent=2));registry.dispose()
print(f'PASS Python: {len(records)} numeric break states; default_names={default_names}.')
