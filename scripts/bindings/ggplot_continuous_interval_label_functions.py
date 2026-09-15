"""FIX-GG04: continuous interval guide labels through the actual Python adapter."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
cases=[t for t in json.loads((ROOT/'fixtures/parity/ggplot2/continuous-interval-label-functions.json').read_text())['cases'] if t['kind']=='interval']
def descriptor(t):
 result=base_descriptor(t)
 if t['channel']=='colour':result['function']['Interpolated']['output']={'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}}
 result['guide']={('ContinuousBins' if t['guide']=='bins' else 'ContinuousSteps'):result['guide']['Continuous']}
 return result
def base_descriptor(t):
 channel=t['channel'];family={'Pow':{'exponent':.5}} if t['transform']=='sqrt' else {'Log':{'base':10}} if t['transform']=='log10' else 'Linear'
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':family,'domain':[0,1],'reverse':t['transform']=='reverse','rescaler':'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[.1,1.] if channel=='alpha' else [1.,6.],'exponent':.5 if channel=='size' else 1.,'absolute':False}},'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[1,10] if t['limits']=='full' else None,'oob':'Censor'}},'guide':{'Continuous':{'breaks':None if t['break_mode']=='auto' else [] if t['break_mode']=='empty' else [-1,0,1,1,3,20,{'number':'Infinity'},{'number':'NaN'}],'labels':{'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}}}}
def layer(t):
 if t['channel']=='colour':return c.points().name('marks')
 channel={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth','shape':'Shape','linetype':'LineType'}[t['channel']]
 result=(c.line() if t['channel'] in ('linewidth','linetype') else c.points()).name('marks')
 return result.value_scale(channel,'v',descriptor(t)) if t['channel'] in ('shape','linetype') else result.numeric_scale(channel,'v',descriptor(t))
def data_for(t):return c.Data.columns({'x':c.column(list(map(float,range(len(t['inputs'])))),kind='float64'),'v':c.column(t['inputs'],kind='float64').nullable(True)})
def build(t,d):
 draft=c.plot(d).with_registry(registry).profile('Ggplot2_4_0_3')
 if t['channel']=='colour':draft=draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)))
 else:draft=draft.aes(c.aes().x('x').y(1.))
 return draft.layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0];entries=(state.get('color_legend') or {}).get('numeric_breaks',[]) if t['channel']=='colour' else [e for group in state.get('numeric_value_guides',{}).values() for e in group]
 entries=[e for e in entries if e['visible']]
 actual={'values':[e['transformed']['number'] if isinstance(e['transformed'],dict) else e['transformed'] for e in entries],'labels':[e['label'] for e in entries]}
 expected=next(iter(t['result'].get('keys',[])),{'values':[],'labels':[]})
 expected={**expected,'values':t['result']['boundaries'][:len(expected['values'])]}
 assert actual['labels']==expected['labels'],(t,actual,expected)
 assert len(actual['values'])==len(expected['values']),(t,actual,expected)
 assert all(math.isclose(float(a),float(b),rel_tol=3e-12,abs_tol=3e-12) for a,b in zip(actual['values'],expected['values'])),(t,actual,expected)
 return actual
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
for index,t in enumerate(cases):
 owned=[]
 try:
  d=data_for(t);owned.append(d);p=build(t,d);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==58
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  old=json.loads(wire);old['version']=57
  try:c.Plot.from_json(json.dumps(old),registry)
  except c.ChartError:pass
  else:raise AssertionError('interval downgrade accepted')
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=check(t,chart);assert 'error' not in t['result'],t
   records.append({'index':index,'state':state,**actual});assert restored.to_json()==wire
  if t['label_mode']=='indexed' and ((t['population'] in ('ordinary','empty') and t['limits']=='full' and t['break_mode']=='auto') or (t['population']=='constant' and t['limits']=='none' and t['break_mode']=='empty')):
   request=output.request(restored,options);owned.append(request)
   if 'error' in t['draw']:
    try:request.prepare()
    except c.ChartError as error:assert error.code=='CHART_VALIDATION'
    else:raise AssertionError(('reference draw rejected',t))
    continue
   frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['channel']}-{t['guide']}-{t['transform']}-{t['population']}.{fmt}").write_bytes(frame.export(fmt))
 except c.ChartError as error:
  assert 'error' in t['result'],(t,str(error));assert error.code=='CHART_VALIDATION',(t,error.code)
  records.append({'index':index,'error':error.code})
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==2720,len(records)
assert len(list(out.glob('*.svg')))==34
options.dispose();output.dispose()
(out/'records.json').write_text(json.dumps(records,indent=2));registry.dispose()
print('PASS Python: 2720 continuous interval label states: 820 successes in three states and 260 expected rejections.')
