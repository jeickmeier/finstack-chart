"""FIX-GG04: continuous limit/vector/transform compositions in actual hosts."""
import copy,json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
family='continuous'
cases=json.loads((ROOT/'fixtures/parity/ggplot2/continuous-pipeline-compositions.json').read_text())['cases']
def descriptor(t):
 call={'operation':{'id':t.get('operation','example.scale_vector'),'version':'1'},'parameters':{'mode':t['mode'],'family':t['family']}}
 result={'missing_paint_is_na':t['channel']=='colour','training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[1,10],'reverse':False,'rescaler':'Range'}},'output':'Identity','unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[1,10],'oob':'Censor'}} if family=='continuous' else {'Binned':{'limits':[1,10],'oob':'Squish','breaks':{'Nice':5},'right':True}},'guide':'Hidden' if t['guide_mode']=='hidden' else ({'Continuous':{'labels':'Automatic'}} if family=='continuous' else {'BinnedLegend':'Automatic'}),'oob_function':call,'rescaler_function':call,'palette_function':{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':'full','channel':t['channel']}}}
 normal=result['function']['Interpolated']['normalization']['Ggplot'];normal['family']={'Log':{'base':10}} if t['transform']=='log10' else 'Linear';normal['reverse']=t['transform']=='reverse'
 result['ggplot']['Continuous']['limits']={'full':[1,10],'reversed':[10,1],'constant':[4,4]}.get(t['limit_mode'])
 if t['limit_mode'] in ('singleton','empty','null','missing'):
  result['limits_function']={'operation':{'id':'example.constant_numeric_limits','version':'1'},'parameters':{'values':{'singleton':[4],'empty':[],'null':None,'missing':[None,10]}[t['limit_mode']]}}
 return result

def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def data_for(t,name="data"):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column([float(v) if v in ('Infinity','-Infinity') else v for v in t['inputs']],kind='float64').nullable(True)},name=name)
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
 if t['channel']=='colour':compare([s['color'] for s in styles],[paint(v) for v in wanted if v is not None])
 if t['channel']=='alpha':compare([s['color']['alpha'] for s in styles],[0 if v in ('Infinity','-Infinity') else max(0,min(255,round((1 if v is None else v)*255))) for v in wanted])
 entries=(state.get('color_legend') or {}).get('numeric_breaks',[]) if t['channel']=='colour' else [e for group in state.get('numeric_value_guides',{}).values() for e in group]
 entries=[e for e in entries if e['visible']]
 def mapped(v):
  if v['kind'] in ('Missing','Null'):return None
  value=v['value'];return value.get('number') if isinstance(value,dict) else value
 keys=[{'values':[(-e['value'] if t['transform']=='reverse' else math.log10(e['value']) if t['transform']=='log10' else e['value']) for e in entries],'labels':[e.get('label') for e in entries],'mapped':[mapped(e['mapped']) for e in entries]}] if entries else []
 compare(keys,t['result']['keys'])
 if t['channel']=='size':compare([s['radius'].get('number') if isinstance(s['radius'],dict) else s['radius'] for s in styles],[v for v in wanted if v is not None])
 return {'styles':styles,'numeric_scales':{k:v['scale'] for k,v in state.get('numeric_scales',{}).items()},'numeric_value_guides':list(state.get('numeric_value_guides',{}).values()),'color_legend':None if state.get('color_legend') is None else {k:v for k,v in state['color_legend'].items() if k!='id'}}
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==(39 if t['family']=='binned' and t['guide_mode']!='hidden' else 38)
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual})
  sample=t['guide_mode']=='hidden' and t['population']=='ordinary' and ((t['limit_mode']=='full' and t['mode']=='both_index') or (t['channel']=='size' and t['transform']=='identity' and ((t['limit_mode']=='singleton' and t['mode']=='default') or (t['limit_mode'] in ('empty','missing') and t['mode']=='rescale_index'))))
  if sample:
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-{t['transform']}-{t['limit_mode']}-{t['mode']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code in ('CHART_VALIDATION','CHART_NUMERICAL_DOMAIN'),(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
for channel in ('colour','size','alpha'):
 for limit_mode in ('full','empty'):
  selected={t['population']:t for t in cases if t['guide_mode']=='hidden' and t['channel']==channel and t['mode']=='both_index' and t['transform']=='identity' and t['limit_mode']==limit_mode};owned=[]
  try:
   original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
   for population in ('missing','empty'):
    t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
    fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch);assert p.to_json()==wire and held.scene()==scene
    records.append({'channel':channel,'limit_mode':limit_mode,'replacement':population,**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==3012,len(records)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False))
output.dispose();registry.dispose();print('PASS',len(records),'continuous composition host states')
