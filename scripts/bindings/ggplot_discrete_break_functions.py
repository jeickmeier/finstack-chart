"""FIX-GG04: non-color discrete guide labels through the actual Python adapter."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-break-functions.json').read_text())['cases']
def key(v):return 'Null' if v is None else {'Text':v}
def descriptor(t):
 channel=t['channel']
 palette={'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}} if channel=='colour' else {'Shape':{'solid':True}} if channel=='shape' else 'LineType' if channel=='linetype' else {'NumericRange':{'range':[.1,1.] if channel=='alpha' else [2.,6.],'area':channel=='size'}}
 return {'training':'Eligible','function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'ggplot':{'Discrete':{'limits':list(map(key,['c','b','a',None])) if t['limits']=='explicit' else None,'levels':None,'drop':True,'na_translate':True,'palette':palette}},'breaks_function':{'operation':{'id':'example.discrete_breaks','version':'1'},'parameters':t['mode']},'guide':{'Discrete':{'labels':'Automatic' if t['label_mode']=='default' else {'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'indexed'}}}}}

def layer(t):
 if t['channel']=='colour':return c.points().name('marks')
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth','shape':'Shape','linetype':'LineType'}[t['channel']]
 result=(c.line() if t['channel'] in ('linewidth','linetype') else c.points()).name('marks')
 return result.value_scale(channel,'v',descriptor(t)) if t['channel'] in ('shape','linetype') else result.numeric_scale(channel,'v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='string').nullable(True)})
def build(t,d):
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3')
 if t['channel']=='colour':return draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t))).layer(layer(t)).build()
 return draft.aes(c.aes().x('x').y(1.)).layer(layer(t)).build()

def check(t,chart):
 state=chart.semantics()['layers'][0];entries=[e for group in state.get('discrete_value_guides',{}).values() for e in group]
 actual={'values':[e['key'].get('Text') if isinstance(e['key'],dict) else None for e in entries],'labels':[e['label'] for e in entries]}
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
 if t['channel']=='colour':
  actual={'labels':[e[0] for e in (state.get('color_legend') or {}).get('entries',[])]};expected={'labels':['NA' if v is None else v for v in expected['labels']]}
 assert actual==expected,(t,actual,expected)
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
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==1200,len(records)
for channel in ('colour','size','alpha','linewidth','shape','linetype'):
 selected={t['population']:t for t in cases if t['channel']==channel and t['limits']=='trained' and t['mode']=='mixed' and t['label_mode']=='indexed'};owned=[]
 try:
  original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json()
  for population in ('nullable','all_missing','empty','numeric_text','ordinary'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire
   records.append({'channel':channel,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==1230
(out/'records.json').write_text(json.dumps(records,indent=2));registry.dispose()
print('PASS Python: 1230 discrete break states, including 30 replacements.')
