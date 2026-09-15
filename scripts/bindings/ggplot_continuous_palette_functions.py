"""FIX-GG04: continuous vector palette values, lookup errors and immutable host publication."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/palette-functions.json').read_text())['cases'] if t['family']=='continuous']
cases += [t for t in json.loads((ROOT/'fixtures/parity/ggplot2/vector-palette-lookup.json').read_text())['cases'] if t['family']=='continuous']
cases += [t for t in json.loads((ROOT/'fixtures/parity/ggplot2/vector-palette-missing-paint.json').read_text())['cases'] if t['family']=='continuous']
def descriptor(t):
 return {'missing_paint_is_na':t['channel']=='colour','training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':'Identity','unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[1,10] if t['limits']=='full' else None,'oob':'Censor'}},'guide':'Hidden' if t.get('guide_mode')=='hidden' else {'Continuous':{'breaks':[1] if t.get('guide_mode')=='first' else ([10,1,4] if t.get('guide_mode')=='restricted' else None),'labels':'Automatic'}},'palette_function':{'operation':{'id':t.get('operation','example.scale_palette'),'version':'1'},'parameters':{'mode':t['mode'],'channel':t['channel']}}}
def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def data_for(t,name="data"):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='float64').nullable(True)},name=name)
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
def paint(v):
 if v=='black':v='#000000'
 return dict(zip(('red','green','blue','alpha'),[0,0,0,0] if v is None else [int(v[i:i+2],16) for i in (1,3,5)]+[int(v[7:9],16) if len(v)==9 else 255]))
def check(t,chart,index=0):
 state=chart.semantics()['layers'][index];styles=state.get('styles',[]);wanted=t['result']['mapped']
 if t['channel']=='colour': compare([s['color'] for s in styles],[paint(v) for v in wanted if v is not None])
 elif t['channel']=='size': compare([s['radius'] for s in styles],[v for v in wanted if v is not None])
 else: compare([s['color']['alpha'] for s in styles],[255 if v is None else round(max(0,min(1,v))*255) for v in wanted])
 return {'styles':styles,'numeric_scales':{k:v['scale'] for k,v in state.get('numeric_scales',{}).items()},'numeric_value_guides':list(state.get('numeric_value_guides',{}).values()),'color_legend':None if state.get('color_legend') is None else {k:v for k,v in state['color_legend'].items() if k!='id'}}
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
  assert 'error' in t['result'],(t,str(error));assert error.code in ('CHART_VALIDATION','CHART_NUMERICAL_DOMAIN'),(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
for channel in ('colour','size','alpha'):
 for mode in ('full','named'):
  selected={t['population']:t for t in cases if not t.get('guide_mode') and not t.get('na_mode') and t['channel']==channel and t['limits']=='full' and t['mode']==mode};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','all_missing','empty','ordinary'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire and held.scene()==scene
    records.append({'channel':channel,'mode':mode,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
batch_cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/vector-palette-batches.json').read_text())['cases'] if t['family']=='continuous']
assert len(batch_cases)==81
for index,source in enumerate(batch_cases):
 owned=[];t={**source,'limits':'full'}
 try:
  datasets=[data_for({**t,'inputs':v},name=f'layer-{i}') for i,v in enumerate(t['inputs'])];owned.extend(datasets)
  draft=c.plot(datasets[0]).with_registry(registry).profile('Ggplot2_4_0_3');owned.append(draft)
  if t['channel']=='colour':draft=draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)))
  else:draft=draft.aes(c.aes().x('x').y(1.))
  p=draft.layer(layer(t)).layer(layer(t).name('second').data(datasets[1])).build();owned.append(p)
  wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);chart=restored.chart();owned.append(chart)
  states=[check({**t,'result':{'mapped':t['result']['mapped'][i]}},chart,i) for i in range(2)]
  records.append({'batch':index,'layers':states})
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
expected=sum(1 if 'error' in t['result'] else 2 for t in cases)+28+81
assert len(records)==expected,(len(records),expected)
(out/'records.json').write_text(json.dumps(records,indent=2));output.dispose();registry.dispose()
print(f'PASS Python: {len(records)} continuous vector palette states including 24 replacements and four rejection contracts.')
