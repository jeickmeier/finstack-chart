"""FIX-GG04: discrete palette values, lookup errors and immutable host publication."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/palette-functions.json').read_text())['cases'] if t['family']=='discrete']
cases+=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-palette-lookup.json').read_text())['cases']
cases+=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-palette-missing-paint.json').read_text())['cases']
def key(v):return 'Null' if v is None else {'Text':v}
def descriptor(t):
 s={'missing_paint_is_na':t['channel']=='colour','training':'Eligible','function':{'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}},'ggplot':{'Discrete':{'limits':list(map(key,['a','b','c','d'])) if t['limits']=='full' else None,'levels':None,'drop':True,'na_translate':True,'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}}}},'palette_function':{'operation':{'id':t.get('operation','example.scale_palette'),'version':'1'},'parameters':{'mode':t['mode'],'channel':t['channel']}}}
 if t.get('guide_mode')=='hidden':s['guide']='Hidden'
 if t.get('guide_mode')=='first':s['guide']={'Discrete':{'breaks':[key('a')]}}
 return s
def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='string').nullable(True)})
def build(t,d):
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3')
 if t['channel']=='colour':
  scale=c.color_mapped('v',descriptor(t))
  if t.get('na_mode','NA')!='NA':scale=scale.missing(paint('#00ff00' if t['na_mode']=='green' else '#00000000'))
  return draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(scale).layer(layer(t)).build()
 return draft.aes(c.aes().x('x').y(1.)).layer(layer(t)).build()
def compare(a,b):
 if isinstance(a,(int,float)) and isinstance(b,(int,float)):assert math.isclose(a,b,rel_tol=0,abs_tol=5e-14),(a,b)
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b),(a,b)
  for x,y in zip(a,b):compare(x,y)
 elif isinstance(a,dict) and isinstance(b,dict):
  assert a.keys()==b.keys(),(a,b)
  for k,v in a.items():compare(v,b[k])
 else:assert a==b,(a,b)
def paint(v):return dict(zip(('red','green','blue','alpha'),[0,0,0,0] if v is None else [int(v[i:i+2],16) for i in (1,3,5)]+[int(v[7:9],16) if len(v)==9 else 255]))
def check(t,chart):
 state=chart.semantics()['layers'][0]
 if t['channel']=='colour':
  legend=state.get('color_legend')
  compare([s['color'] for s in state.get('styles',[])],[paint(v) for v in t['result']['mapped'] if v is not None])
  if legend is None:
   assert not t['result'].get('keys'),t
   return {'mapped':None,'domain':None,'guide':[],'styles':state.get('styles',[])}
  spec=legend['mapping'];entries=legend['entries']
 else:
  spec=state['numeric_scales']['Size' if t['channel']=='size' else 'Alpha']['scale'];entries=[e for g in state.get('discrete_value_guides',{}).values() for e in g]
 ordinal=spec['function']['Ordinal'];domain=[k.get('Text') if isinstance(k,dict) else None for k in ordinal['domain']]
 def fallback():
  if t['channel']!='colour' or spec.get('missing_paint_is_na',False):return None
  missing=legend['missing'];return '#'+''.join(f'{missing[k]:02x}' for k in ('red','green','blue'))+(f"{missing['alpha']:02x}" if missing['alpha']!=255 else '')
 def sample(v):
  if v is None or v not in domain:return fallback()
  index=domain.index(v);raw=ordinal['range'][index]
  if raw['kind'] in ('Number','Text'):return raw.get('value')
  return fallback() if index in spec.get('palette_fallback_indices',[]) else None
 actual=[sample(v) for v in t['inputs']];compare(actual,t['result']['mapped'])
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[],'mapped':[]})
 if t['channel']=='colour':
  wanted=[['NA' if label is None else label,paint(value)] for label,value in zip(expected['labels'],expected['mapped'])];compare(entries,wanted)
 else:
  actual_keys={'values':[e['key'].get('Text') if isinstance(e['key'],dict) else None for e in entries],'labels':[e['label'] for e in entries],'mapped':[sample(e['key'].get('Text') if isinstance(e['key'],dict) else None) for e in entries]};compare(actual_keys,expected)
 return {'mapped':actual,'domain':domain,'guide':entries,'styles':state.get('styles',[])}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==37
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual})
  sample=None
  if not t.get('guide_mode') and not t.get('na_mode') and t['limits']=='none' and t['mode']=='named' and t['population'] in ('ordinary','missing'):sample=f"{t['channel']}-{t['population']}"
  if t.get('na_mode') and t['mode']=='missing' and t['population']=='missing':sample='na-'+t['na_mode'].lower()
  if sample:
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f'{sample}.{fmt}').write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
for channel in ('colour','size','alpha'):
 for mode in ('full','named'):
  selected={t['population']:t for t in cases if not t.get('guide_mode') and not t.get('na_mode') and t['channel']==channel and t['limits']=='none' and t['mode']==mode};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','all_missing','empty','ordinary'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire and held.scene()==scene
    records.append({'channel':channel,'mode':mode,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
for variant in ('missing_registration','native_only','invalid_parameters','downgrade'):
 t=copy.deepcopy(cases[0]);owned=[]
 if variant=='missing_registration':t['operation']='example.missing_palette'
 if variant=='native_only':t['operation']='example.native_scale_palette'
 if variant=='invalid_parameters':t['mode']='invalid'
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json()
  if variant=='downgrade':wire=json.loads(wire);wire['version']=36;restored=c.Plot.from_json(json.dumps(wire),registry);owned.append(restored)
  raise AssertionError('expected rejection: '+variant)
 except c.ChartError as error:records.append({'rejection':variant,'code':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
expected=sum(1 if 'error' in t['result'] else 2 for t in cases)+28
assert len(records)==expected,(len(records),expected)
(out/'records.json').write_text(json.dumps(records,indent=2));output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} discrete palette states including 24 replacements and four rejection contracts.')
