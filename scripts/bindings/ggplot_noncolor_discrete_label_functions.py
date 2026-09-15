"""FIX-GG04: non-color discrete guide labels through the actual Python adapter."""
import json,sys,itertools
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
factors='--factors' in sys.argv
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/('fixtures/parity/ggplot2/factor-discrete-label-functions.json' if factors else 'fixtures/parity/ggplot2/noncolor-discrete-label-functions.json')).read_text())['cases']
def key(v):return 'Null' if v is None else {'Text':v}
def descriptor(t):
 channel=t['channel']
 palette={'Shape':{'solid':True}} if channel=='shape' else 'LineType' if channel=='linetype' else {'NumericRange':{'range':[.1,1.] if channel=='alpha' else [2.,6.],'area':channel=='size'}}
 return {'training':'Eligible','function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'ggplot':{'Discrete':{'limits':list(map(key,['c','b','a',None])) if t['limit_mode']=='explicit' else None,'levels':list(map(key,t['levels'])) if 'levels' in t else None,'drop':t.get('drop',True),'na_translate':t.get('na_translate',True),'palette':palette}},'guide':{'Discrete':{'breaks':None if t['break_mode']=='auto' else [] if t['break_mode']=='empty' else list(map(key,['z','b','b',None,'a'])),'break_names':['Z','B','B2','M','A'] if t['break_mode']=='named' else None,'labels':{'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}}}}
def layer(t):
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth','shape':'Shape','linetype':'LineType'}[t['channel']]
 result=(c.line() if t['channel'] in ('linewidth','linetype') else c.points()).name('marks')
 return result.value_scale(channel,'v',descriptor(t)) if t['channel'] in ('shape','linetype') else result.numeric_scale(channel,'v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='string').nullable(True)})
def build(t,d):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0];entries=[e for group in state.get('discrete_value_guides',{}).values() for e in group]
 actual={'values':[e['key'].get('Text') if isinstance(e['key'],dict) else None for e in entries],'labels':[e['label'] for e in entries]}
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
 assert actual==expected,(t,actual,expected)
 return actual
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==32
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual})
   if factors and state=='original' and t['limit_mode']=='trained' and t['break_mode']=='auto' and t['label_mode']=='indexed' and not t['drop'] and t['na_translate'] and t['population'] in ('ordinary','nullable','empty') and not t['result'].get('draw_error'):
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/(t['channel']+'-'+t['population']+'.'+fmt)).write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(1280 if factors else 1495),len(records)
for channel,drop,translate in itertools.product(('size','alpha','linewidth','shape','linetype'),(False,True) if factors else (True,),(False,True) if factors else (True,)):
 selected={t['population']:t for t in cases if t['channel']==channel and t['limit_mode']=='trained' and t['break_mode']=='auto' and t['label_mode']=='indexed' and t.get('drop',True)==drop and t.get('na_translate',True)==translate};owned=[]
 try:
  original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json()
  if factors:
   request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('nullable','all_missing','empty','ordinary'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire
   if factors:assert held.scene()==scene
   records.append({'channel':channel,'replacement':population,**({'drop':drop,'translate':translate} if factors else {}),**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(1360 if factors else 1515)
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} non-color discrete label states; factors={factors}.')
