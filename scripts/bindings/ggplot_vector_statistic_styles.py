"""FIX-GG04: generated style vectors through real host execution and edits."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
cases=json.loads((ROOT/'fixtures/parity/ggplot2/vector-transform-statistic-styles.json').read_text())['cases']
retained='--retained' in sys.argv
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current');records=[]
def descriptor(t):
 tr={'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform_vector','version':'1'},'parameters':{'family':t['family']}}}}}
 if t['composed']:tr={'Compose':{'transforms':[tr,'Reverse']}}
 palette={'operation':'PowerRange','range':[1,6],'exponent':.5,'absolute':False} if t['route']=='size' else {'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}
 result={'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':{'Ggplot':{'transform':tr}},'domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':palette},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':None,'oob':'Censor'}}}
 if retained:
  population=[1.,1.,1.,1.,2.] if t['faceted'] else [2.,1.,3.]
  transformed=[v+len(population) for v in population] if t['family']=='cardinality' else [v-sum(population)/len(population) for v in population]
  if t['composed']:transformed=[-v for v in transformed]
  result['trained_transformed_bounds']=[min(transformed),max(transformed)]
 return result
def layer(t):
 l=c.points().name('marks').stat(c.count().group('x'))
 return l.numeric_scale('Size',{'Statistical':'Count'},descriptor(t)) if t['route']=='size' else l.after_stat(c.stat_aes().x('Group').y('Count').color('Count').color_scale('paint'))
def check(t,chart):
 semantic=chart.semantics();panels=semantic['panels'] if t['faceted'] else [semantic];expected=list(t['result']['panels'].values());assert len(panels)==len(expected)
 result=[]
 for panel,wanted in zip(panels,expected):
  styles=panel['layers'][0].get('styles',[]);assert len(styles)==len(wanted['mapped']),(t,styles)
  for style,value in zip(styles,wanted['mapped']):
   if t['route']=='size':assert math.isclose(style['radius'],value,rel_tol=4e-14,abs_tol=4e-14),(t,style,value)
   else:assert style['color']==dict(red=int(value[1:3],16),green=int(value[3:5],16),blue=int(value[5:7],16),alpha=255),(t,style,value)
  result.append(styles)
 return result
for index,t in enumerate(cases):
 owned=[]
 try:
  d=c.Data.columns({'x':c.categorical(['a','a','b','c','c','c']),'panel':['a','b','a','a','b','b']});owned.append(d)
  builder=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(layer(t))
  if t['route']=='paint':builder=builder.scale(c.color_mapped('paint',descriptor(t)))
  if t['faceted']:builder=builder.facet(c.facet_wrap('panel').columns(2))
  p=builder.build();owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==(56 if retained else 55)
  if retained:
   downgraded=json.loads(wire);downgraded['version']=55
   try:c.Plot.from_json(json.dumps(downgraded),registry)
   except c.ChartError as error:assert 'version' in str(error).lower()
   else:raise AssertionError('Retained metadata downgrade accepted')
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   try:
    chart=current.chart();owned.append(chart);styles=check(t,chart)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    assert 'error' not in t['result'];records.append(dict(index=index,state=state,styles=styles))
    if state=='original':
     for fmt in ('svg','pdf','png'):(out/f'{index:03d}-{t["route"]}.{fmt}').write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in t['result'] and error.code=='CHART_NUMERICAL_DOMAIN',(index,str(error));records.append(dict(index=index,state=state,error=error.code))
   assert restored.to_json()==wire
 except c.ChartError as error:
  assert 'error' in t['result'] and error.code=='CHART_NUMERICAL_DOMAIN',(index,str(error));records.append(dict(index=index,state='build',error=error.code))
 finally:
  for obj in reversed(owned):obj.dispose()
assert [r['index'] for r in records if r['state']=='build']==[12,14]
assert [(r['index'], r['state']) for r in records if 'error' in r]==[(12,'build'),(13,'original'),(13,'layer_edit'),(13,'theme_edit'),(14,'build'),(15,'original'),(15,'layer_edit'),(15,'theme_edit')]
assert len(records)==44,len(records)
assert sum(len(list(out.glob('*.'+fmt)))for fmt in ('svg','pdf','png'))==36
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose();print('PASS 44 generated style states and 36 publications')
