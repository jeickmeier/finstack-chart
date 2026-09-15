"""FIX-GG04: built-in transform reference plots through the actual Python host."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
fixture=json.loads((ROOT/'fixtures/parity/ggplot2/scale-transform-contracts.json').read_text())
position_minor='--position-minors' in sys.argv
position_explicit='--position-explicit' in sys.argv
position_functions='--position-functions' in sys.argv or position_explicit or position_minor
null_breaks='--null-breaks' in sys.argv
scale_functions='--scale-functions' in sys.argv or null_breaks
identity_functions='--identity-functions' in sys.argv
identity_vectors='--identity-vectors' in sys.argv or identity_functions
vectors='--vectors' in sys.argv or identity_vectors or scale_functions or position_functions
retained='--retained' in sys.argv
assert not retained or vectors
if vectors:
 source=json.loads((ROOT/'fixtures/parity/ggplot2/vector-transforms.json').read_text());fixture={'cases':source['configurations'],'plots':source['cases']}
minors='--minors' in sys.argv
probability='--probability' in sys.argv
registered='--registered' in sys.argv or probability
if registered:
 source=json.loads((ROOT/('fixtures/parity/ggplot2/probability-transforms.json' if probability else 'fixtures/parity/ggplot2/registered-transforms.json')).read_text());fixture={'cases':source['configurations'],'plots':source['cases']}
if minors:
 source=json.loads((ROOT/'fixtures/parity/ggplot2/registered-transform-minors.json').read_text());fixture={'cases':source['configurations'],'plots':[dict(t,route='position') for t in source['cases']]}
compositions='--compositions' in sys.argv
if compositions: fixture=json.loads((ROOT/'fixtures/parity/ggplot2/transform-compositions.json').read_text())
secondary='--secondary' in sys.argv
labels='--labels' in sys.argv
guides='--guides' in sys.argv or labels
if guides:
 fixture['plots']=[dict(t,route='position',population='ordinary') for t in json.loads((ROOT/'fixtures/parity/ggplot2/transform-guide-controls.json').read_text())['cases'] if labels or t['format']=='default']
if secondary:
 fixture['plots']=[dict(t,route='position',population='ordinary',count=3) for t in json.loads((ROOT/'fixtures/parity/ggplot2/transform-secondary.json').read_text())['cases']]
identity_guides='--identity-guides' in sys.argv or identity_vectors
if identity_guides:
 source=json.loads((ROOT/('fixtures/parity/ggplot2/identity-vector-functions.json' if identity_functions else 'fixtures/parity/ggplot2/identity-vector-transforms.json' if identity_vectors else 'fixtures/parity/ggplot2/identity-transform-guides.json')).read_text());fixture={'cases':source['configurations'],'plots':[dict(t,route='identity',break_mode=t.get('break_mode','auto')) for t in source['cases']]}
if scale_functions:
 source=json.loads((ROOT/('fixtures/parity/ggplot2/vector-null-breaks.json' if null_breaks else 'fixtures/parity/ggplot2/vector-scale-functions.json')).read_text());fixture={'cases':source['configurations'],'plots':source['cases']}
if position_functions:
 source=json.loads((ROOT/('fixtures/parity/ggplot2/vector-position-minors.json' if position_minor else 'fixtures/parity/ggplot2/vector-position-explicit.json' if position_explicit else 'fixtures/parity/ggplot2/vector-position-callbacks.json')).read_text());fixture={'cases':source['configurations'],'plots':source['cases']}
expected_states,expected_files=(600,36) if position_minor else (360,54) if position_explicit else (180,27) if position_functions else (1056,36) if null_breaks else (3168,39) if scale_functions else (864,18) if identity_functions else (288,18) if identity_vectors else (696,24) if identity_guides else (180,54) if vectors else (360,36) if minors else (108,36) if registered else (543,177) if compositions else (87,81) if secondary else (348,342) if labels else (174,171) if guides else (342,171)
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current');records=[]
def transform(case):
 if vectors:
  tr={'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform_vector','version':'1'},'parameters':{'family':case['family']}}}}}
  return {'Compose':{'transforms':[tr,'Reverse']}} if case['composed'] else tr
 if minors:
  tr={'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform_minor','version':'1'},'parameters':{'family':case['family'],'mode':case['mode']}}}}}
  return {'Compose':{'transforms':[tr,'Reverse']}} if case['composed'] else tr
 if probability:return {'Registered':{'selection':{'call':{'operation':{'id':'example.probability_transform','version':'1'},'parameters':({'distribution':'Uniform','min':-2. if case['custom'] else 0.,'max':3. if case['custom'] else 1.} if case['family']=='unif' else {'distribution':'Exponential','rate':2. if case['custom'] else .5})}}}}
 if registered:return {'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform','version':'1'},'parameters':{'family':case['family'],'custom':case['custom']}}}}}
 if 'transforms' in case:return {'Compose':{'transforms':[transform({'constructor':p['name'],'args':p['args']}) for p in case['transforms']]}}
 name=case['constructor'];a=case['args'] if isinstance(case['args'],dict) else {}
 simple={'asinh':'Asinh','asn':'Asn','atanh':'Atanh','identity':'Identity','log1p':'Log1p','reciprocal':'Reciprocal','reverse':'Reverse','sqrt':'Sqrt'}
 if name in simple:return simple[name]
 if name in ('log','log10','log2','exp'):return {'Exp' if name=='exp' else 'Log':{'base':a.get('base',10 if name=='log10' else 2 if name=='log2' else math.e)}}
 if name in ('boxcox','modulus'):return {'BoxCox' if name=='boxcox' else 'Modulus':{'p':a['p'],'offset':a.get('offset',0 if name=='boxcox' else 1)}}
 if name=='yj':return {'YeoJohnson':{'p':a['p']}}
 if name=='pseudo_log':return {'PseudoLog':{'sigma':a.get('sigma',1),'base':a.get('base',math.e)}}
 if name=='probit' or a.get('distribution')=='norm':return {'Normal':{'mean':a.get('mean',0),'sd':a.get('sd',1)}}
 if name=='logit' or a.get('distribution')=='logis':return {'Logistic':{'location':a.get('location',0),'scale':a.get('scale',1)}}
 raise AssertionError(case)
def layer():
 result=c.points().name('marks')
 if scale_functions and t['route']=='size':result=result.numeric_scale('Size','value',function_descriptor(t,tr))
 if identity_guides:
  spec={'training':'Eligible','function':{'GgplotNumericIdentity':{'transform':{'Ggplot':{'transform':tr}},'limits':[0,10] if t.get('limits')=='fixed' else None,'guide':t['guide']=='legend','trained':None}},'guide':{'Continuous':{'breaks':t['inputs'] if t['break_mode']=='explicit' else None,'labels':'Automatic'}}}
  if identity_functions:
   if t['limit_mode']=='reverse':spec['limits_function']={'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':'reverse'}}
   if t['break_mode']!='auto':spec['breaks_function']={'operation':{'id':'example.breaks_limits','version':'1'},'parameters':t['break_mode']}
  result=result.numeric_scale('Size','value',spec)
 return result
def number(v):return v if isinstance(v,(int,float)) else {'Infinity':math.inf,'-Infinity':-math.inf,'NA':math.nan,'NaN':math.nan,'-0':-0.0}[v['number']]
def paint(v):
 if v=='grey50':v='#7F7F7F'
 return dict(red=int(v[1:3],16),green=int(v[3:5],16),blue=int(v[5:7],16),alpha=255)
def descriptor(t):
 return {'training':'Eligible','guide':'Hidden','function':{'Interpolated':{'normalization':{'Ggplot':{'family':{'Ggplot':{'transform':t}},'domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':None,'oob':'Censor'}}}
def function_descriptor(t,tr):
 spec=descriptor(tr)
 if t['route']=='size':spec['function']['Interpolated']['output']={'Interpolate':{'operation':'PowerRange','range':[1,6],'exponent':.5,'absolute':False}}
 if t['kind']=='binned':spec['ggplot']={'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':5},'right':True}}
 spec['guide']='Hidden' if t['guide']=='none' else {'BinnedLegend':'Automatic'} if t['kind']=='binned' else {'Continuous':{'breaks':None,'labels':'Automatic'}}
 if t['limit_mode']=='reverse':spec['limits_function']={'operation':{'id':'example.numeric_limits','version':'1'},'parameters':{'mode':'reverse'}}
 if t['break_mode']!='auto':spec['breaks_function']={'operation':{'id':'example.breaks_limits','version':'1'},'parameters':t['break_mode']}
 return spec
for index,t in enumerate(fixture['plots']):
 owned=[]
 try:
  values=[number(v) for v in t['inputs']];d=c.Data.columns({'x':c.column(list(range(len(values))),kind='float64'),'value':c.column(values,kind='float64').nullable(True)},name='data');owned.append(d)
  tr=transform(fixture['cases'][t['configuration']]);builder=c.plot(d).profile('Ggplot2_4_0_3').with_registry(registry).layer(layer()).y_axis(c.y_axis().visible(False))
  if t['route']=='position':
   axis=c.x_axis().scale(c.scale_transform({'Ggplot':{'transform':tr}})).range(100.,540.).breaks_function({'operation':{'id':'example.breaks_limits','version':'1'},'parameters':t['break_mode']} if position_functions and not position_explicit and not position_minor else None).minor_breaks('Hidden' if minors and t['override']=='hidden' else {'Numeric':[-1.5,0.5]} if minors and t['override']=='explicit' else None).expansion({'mult':[0.,0.],'add':[0.,0.]} if secondary else None).tick_format({'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':'fixed_two'}} if t.get('format')=='two_digits' else None).tick_arguments({'count':t['count']} if 'count' in t else None).guide_geometry({'labels':'Preserve'})
   if position_explicit or (position_minor and t['break_values'] is not None):
    ticks=[{'Number':{'number':'NaN' if v['number']=='NA' else v['number']}} if isinstance(v,dict) else v for v in t['break_values']]
    axis=axis.ticks(list(zip(ticks,t['explicit_labels']))) if t.get('explicit_labels') is not None else axis.tick_values(ticks)
   if position_minor:axis=axis.minor_breaks({'Numeric':[2,5]} if t['minor_mode']=='explicit' else {'Registered':{'operation':{'id':'example.breaks_minor_two','version':'1'},'parameters':t['minor_mode']}})
   builder=builder.aes(c.aes().x('value').y(1.)).x_axis(axis)
  elif scale_functions:
   builder=builder.aes(c.aes().x('x').y(1.).color('value').color_scale('paint') if t['route']=='paint' else c.aes().x('x').y(1.))
   if t['route']=='paint':builder=builder.scale(c.color_mapped('paint',function_descriptor(t,tr)))
  elif identity_guides:builder=builder.aes(c.aes().x('x').y(1.))
  else:
   mapped=descriptor(tr)
   if vectors:
    mapped.pop("guide")
    if t["route"]=="paint": mapped["guide"]={"Continuous":{"breaks":None,"count":t["count"],"labels":"Automatic"}}
   if probability and t['route']=='binned_paint':mapped.pop('guide')
   if t['route']=='binned_paint':mapped['ggplot']={'Binned':{'limits':None,'oob':'Squish','breaks':{'Nice':(t['count'] or 5) if vectors else 5},'right':True}}
   if retained:
    case=fixture['cases'][t['configuration']];present=[v for v in values if not math.isnan(v)]
    offset=-len(values) if case['family']=='cardinality' else sum(present)/len(present) if present else math.nan
    transformed=[(-1 if case['composed'] else 1)*(v-offset) for v in values];finite=[v for v in transformed if math.isfinite(v)]
    bounds=[min(finite),max(finite)] if finite else [0.,1.]
    mapped['trained_transformed_bounds']=bounds
   builder=builder.aes(c.aes().x('x').y(1.).color('value').color_scale('paint')).scale(c.color_mapped('paint',mapped))
  if (identity_vectors or scale_functions or position_functions) and t['layers']=='two':
   second=[number(v) for v in t['second']];d2=c.Data.columns({'x':c.column(list(range(len(second))),kind='float64'),'value':c.column(second,kind='float64').nullable(True)},name='second');owned.append(d2);builder=builder.layer(layer().name('second').data(d2))
  if secondary:builder=builder.axis(c.x_axis().name('second').side('Top').secondary('x',2.,1.).guide_geometry({'labels':'Preserve'}))
  p=builder.build();owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==(56 if retained and t['route']!='position' else 55 if registered or minors or vectors else 54 if compositions else 53)
  if retained and t['route']!='position':
   old=json.loads(wire);old['version']=55
   try:c.Plot.from_json(json.dumps(old),registry)
   except c.ChartError as error:assert 'version' in str(error).lower()
   else:raise AssertionError('Retained transform downgrade accepted')
  if (identity_vectors or scale_functions and t['route']=='size') and t['layers']=='two':
   envelope=json.loads(wire);envelope['definition']['layers'][1]['numeric_scales']['Size']['id']=envelope['definition']['layers'][0]['numeric_scales']['Size']['id'];wire=json.dumps(envelope)
  restored=c.Plot.from_json(wire,registry);owned.append(restored)
  if (identity_vectors or scale_functions and t['route']=='size') and t['layers']=='two':wire=restored.to_json()
  assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   try:
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    assert 'error' not in t['result'],t
    chart=current.chart();owned.append(chart);semantics=chart.semantics()['layers'][0]
    if scale_functions:
     mapped=[];guide_rows=[]
     expected=next(iter(t['result']['keys']),{'values':[],'labels':[]})
     for layer_state,wanted in zip(chart.semantics()['layers'],t['result']['mapped']):
      actual=[style['color'] if t['route']=='paint' else style['radius'] for style in layer_state.get('styles',[])]
      wanted=[paint(v) for v in wanted] if t['route']=='paint' else [number(v) for v in wanted if not math.isnan(number(v))]
      assert len(actual)==len(wanted),(index,state,actual,wanted)
      assert actual==wanted if t['route']=='paint' else all(math.isclose(number(a),b,rel_tol=4e-14,abs_tol=4e-14) for a,b in zip(actual,wanted)),(index,state,actual,wanted)
      entries=(layer_state.get('color_legend') or {}).get('numeric_breaks',[]) if t['route']=='paint' else [e for group in layer_state.get('numeric_value_guides',{}).values() for e in group]
      entries=[e for e in entries if e['visible']]
      row={'values':[e['transformed'] for e in entries],'labels':[e['label'] for e in entries]}
      assert row['labels']==expected['labels'],(index,state,row,expected)
      assert len(entries)==len(expected['values']),(index,state,row,expected)
      assert all(math.isclose(number(a),number(b),rel_tol=4e-14,abs_tol=4e-14) for a,b in zip(row['values'],expected['values'])),(index,state,row,expected)
      mapped.append(actual);guide_rows.append(row)
     result={'mapped':mapped,'guides':guide_rows}
    elif identity_guides:
     entries=[e for group in semantics.get('numeric_value_guides',{}).values() for e in group if e['visible']]
     expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
     result={'values':[e['transformed'] for e in entries],'labels':[e['label'] for e in entries]}
     assert result['labels']==expected['labels'],(index,state,result,expected)
     assert len(entries)==len(expected['values']),(index,result,expected)
     assert all(math.isclose(number(a),number(b),rel_tol=4e-14,abs_tol=4e-14) for a,b in zip(result['values'],expected['values'])),(index,result,expected)
     if identity_vectors:
      mapped=[[s['radius'] for s in layer_state.get('styles',[])] for layer_state in chart.semantics()['layers']]
      for actual,wanted in zip(mapped,t['result']['mapped']):
       wanted=[number(v) for v in wanted if not math.isnan(number(v))]
       assert len(actual)==len(wanted),(index,actual,wanted)
       assert all(math.isclose(number(a),b,rel_tol=4e-14,abs_tol=4e-14) for a,b in zip(actual,wanted)),(index,actual,wanted)
      result['mapped']=mapped
    elif minors:
     ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom').get('minor_ticks',[])
     expected=t['result']['minor_positions'];assert len(ticks)==len(expected),(index,ticks,expected)
     for tick,position in zip(ticks,expected):assert abs((tick['position']-100)/440-position)<4e-14,(index,tick,position)
     result={'minor_ticks':ticks}
    elif t['route']=='position':
     ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Top' if secondary else 'Bottom'))['ticks']
     ticks=sorted(ticks,key=lambda tick:tick['position'])
     expected=sorted([(v,label) for v,label in zip(t['result']['positions'],t['result']['panel_labels'] if registered and isinstance(t['result']['panel_labels'],list) else [] if registered else t['result']['labels']) if isinstance(v,(int,float))],key=lambda item:item[0])
     assert len(ticks)==len(expected),(index,ticks,expected)
     for tick,(position,label) in zip(ticks,expected):assert tick['label']==label and abs((tick['position']-100)/440-position)<(2e-12 if secondary else 4e-14),(index,tick,position,label)
     result={'ticks':ticks}
     if position_minor:
      minor_ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom').get('minor_ticks',[])
      lo,hi=map(number,t['result']['range']);expected_minor=[(number(v)-lo)/(hi-lo) for v in t['result']['minor'] if math.isfinite(number(v))]
      assert len(minor_ticks)==len(expected_minor),(index,minor_ticks,expected_minor)
      assert all(abs((tick['position']-100)/440-v)<4e-14 for tick,v in zip(minor_ticks,expected_minor)),(index,minor_ticks,expected_minor)
      result['minor_ticks']=minor_ticks
    else:
     styles=semantics.get('styles',[]);expected=[paint(v) for v in t['result']['mapped']]
     assert [s['color'] for s in styles]==expected,(index,styles,expected)
     result={'styles':styles}
    records.append({'index':index,'state':state,**result})
    if state=='original' and ((position_functions and t['population']=='ordinary' and t['layers']=='one' and (not position_minor or t['minor_mode'] in ('explicit','majors'))) or (scale_functions and t['population']=='ordinary' and t['guide']=='legend' and t['layers']=='one' and t['limit_mode']=='none' and (null_breaks or t['break_mode']=='mixed' or index==648)) or (not position_functions and not scale_functions and (not identity_functions or t['break_mode']=='mixed') and ((t['configuration'] in (0,1,2) and t['guide']=='legend' and t['layers']=='one') if identity_vectors else (not identity_guides or (t['configuration'] in (9,12,13,21) and t['guide']=='legend'))) and (not minors or (t['count'] is None and t['override']=='default')) and t['population']==('positive' if compositions else 'ordinary'))):
     for fmt in ('svg','pdf','png'):(out/f"{index:03d}-{t['route']}.{fmt}").write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in t['result'] and error.code in (('CHART_NUMERICAL_DOMAIN','CHART_VALIDATION','CHART_SCHEMA_CONFLICT') if position_explicit else ('CHART_NUMERICAL_DOMAIN','CHART_VALIDATION') if compositions or vectors else ('CHART_NUMERICAL_DOMAIN',)),(index,t,str(error),error.code)
    records.append({'index':index,'state':state,'error':error.code})
   assert restored.to_json()==wire
 except c.ChartError as error:
  assert 'error' in t['result'] and error.code in (('CHART_NUMERICAL_DOMAIN','CHART_VALIDATION','CHART_SCHEMA_CONFLICT') if position_explicit else ('CHART_NUMERICAL_DOMAIN','CHART_VALIDATION') if compositions or vectors else ('CHART_NUMERICAL_DOMAIN',)),(index,t,str(error),error.code)
  records.append({'index':index,'state':'build','error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
(out/'records.json').write_text(json.dumps(records,sort_keys=True))
if vectors and not identity_vectors and not scale_functions and not position_functions: assert [r['index'] for r in records if r['state']=='build']==list(range(54,72))
if scale_functions:assert [r['index'] for r in records if r['state']=='build']==[i for i,t in enumerate(fixture['plots']) if t['configuration']==3 and t['route']=='paint']
if position_functions:assert [r['index'] for r in records if r['state']=='build']==[i for i,t in enumerate(fixture['plots']) if t['configuration']==3]
assert len(records)==expected_states,len(records)
assert sum(len(list(out.glob('*.'+fmt))) for fmt in ('svg','pdf','png'))==expected_files
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose()
print(f'PASS {len(records)} transform host states and {expected_files} publications')
