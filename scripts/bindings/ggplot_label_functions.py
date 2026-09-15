"""FIX-GG04 vector label functions through actual Python authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/guide-label-functions.json').read_text())['cases']
def key(v):return 'Null' if v is None else {'Text':v}
def descriptor(t):
 labels={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
 s={'training':'Eligible'}
 if t['kind']=='continuous':
  family={'Pow':{'exponent':.5}} if t['transform']=='sqrt' else {'Log':{'base':10}} if t['transform']=='log10' else 'Linear'
  s['function']={'Interpolated':{'normalization':{'Ggplot':{'family':family,'domain':[0,1],'reverse':t['transform']=='reverse','rescaler':'Range'}},'output':{'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}},'unknown':{'kind':'Missing'}}}
  s['ggplot']={'Continuous':{'limits':t['limits'],'oob':'Censor'}}
  s['guide']={'Continuous':{'breaks':None if t['break_mode']=='auto' else [] if t['break_mode']=='empty' else [-1,0,.1,1,5,10,20,{'number':'Infinity'},{'number':'NaN'},{'number':'NaN'}],'labels':labels}}
 else:
  s['function']={'Ordinal':{'domain':[],'range':[],'unknown':{'Explicit':None}}}
  s['ggplot']={'Discrete':{'limits':None,'levels':None,'drop':True,'na_translate':True,'palette':{'Hue':{'h':[15,375],'chroma':100,'luminance':65,'start':0,'reverse':False}}}}
  s['guide']={'Discrete':{'breaks':None if t['break_mode']=='auto' else [] if t['break_mode']=='empty' else list(map(key,['z','b','b',None,'a'])),'break_names':['Z','B','B2','M','A'] if t['break_mode']=='named' else None,'labels':labels}}
 return s
def data_for(t):
 values=[1.,4.,10.] if t['kind']=='continuous' else [] if t['population']=='empty' else ['a','b',None if t['population']=='nullable' else 'c']
 return c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.column(values,kind='float64' if t['kind']=='continuous' else 'string').nullable(True)},keys=list(range(100,100+len(values))))
def layer():return c.points().name('marks')
def build(t,d):return c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t))).layer(layer()).build()
def check(t,chart):
 state=chart.semantics()['layers'][0];legend=state.get('color_legend');entries=legend['entries'] if legend else []
 expected=[v if v is not None else 'NA' for g in t['build'].get('keys',[]) for v in g['labels']]
 assert [e[0] for e in entries]==expected,(t,entries,expected)
 return {'entries':entries,'styles':state.get('styles',[])}
def sample(t):
 if t['kind']=='continuous':return t['transform'] in ('identity','log10') and t['limits']==[1,10] and t['break_mode']=='explicit' and t['label_mode']=='indexed'
 return t['break_mode']=='named' and ((t['population']=='ordinary' and t['label_mode'] in ('missing','short')) or (t['population']=='nullable' and t['label_mode']=='indexed'))
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==32
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['build'],t
   records.append({'index':index,'state':state,**actual})
   if state=='original' and sample(t):
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/f"sample-{index}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['build'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==326,len(records)
for mode in ('indexed','missing','short'):
 selected={t['population']:t for t in cases if t['kind']=='discrete' and t['break_mode']=='auto' and t['label_mode']==mode};owned=[]
 try:
  original=data_for(selected['ordinary']);owned.append(original);p=build(selected['ordinary'],original);owned.append(p);chart=p.chart();owned.append(chart);wire=p.to_json();request=output.request(p,options);owned.append(request);held=request.prepare();owned.append(held);scene=held.scene()
  for population in ('nullable','empty','ordinary','nullable'):
   t=selected[population];replacement=data_for(t);owned.append(replacement);tx=chart.transaction();owned.append(tx);builder=tx.replace(original,replacement);owned.append(builder);update=builder.build();owned.append(update);assert 'Applied' in chart.commit(update)
   fresh=build(t,replacement);owned.append(fresh);batch=fresh.chart();owned.append(batch);actual=check(t,chart);assert actual==check(t,batch)
   assert held.scene()==scene and p.to_json()==wire;records.append({'mode':mode,'replacement':population,**actual})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==338
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();registry.dispose()
print('PASS Python: 338 label callback states and 15 publication files.')
