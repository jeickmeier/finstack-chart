"""GG-04: continuous legend/NULL/empty selection through actual primary hosts."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();records=[]
colorbar='--colorbar' in sys.argv
transforms='--transforms' in sys.argv
intervals='--intervals' in sys.argv or transforms
cases=[t for t in json.loads((ROOT/('fixtures/parity/ggplot2/continuous-interval-transforms.json' if transforms else 'fixtures/parity/ggplot2/continuous-guide-selection.json')).read_text())['cases'] if t['guide'] in (('bins','coloursteps') if intervals else ('default','none','legend','colourbar') if colorbar else ('default','none','legend'))]
def descriptor(t):
 channel=t['channel']
 result={'missing_paint_is_na':channel=='colour','training':'Eligible',
  'guide':'Hidden' if t['breaks']=='null' or t['guide']=='none' else {({'colourbar':'Colorbar','bins':'ContinuousBins','coloursteps':'ContinuousSteps'}.get(t['guide'],'Continuous')):{'breaks':[0,.5,3,4] if t['breaks']=='uneven' else [-2,6] if t['breaks']=='outside' else [] if t['breaks']=='empty' else None}},
  'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,4],'reverse':False,'rescaler':'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[0,1],'exponent':1,'absolute':False}},'unknown':{'kind':'Missing'}}},
  'ggplot':{'Continuous':{'limits':[0,4],'oob':'Censor'}},
  'palette_function':{'operation':{'id':'example.scale_palette','version':'1'},'parameters':{'mode':t.get('palette_mode','full'),'channel':channel}}}
 if transforms:
  tr={'Log':{'base':10}} if t['transform']=='log10' else {'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform_vector','version':'1'},'parameters':{'family':t['transform']}}}}}
  result['function']['Interpolated']['normalization']['Ggplot'].update(family={'Ggplot':{'transform':tr}},domain=[1,16])
  result['ggplot']['Continuous']['limits']=[1,16]
  if t['breaks']=='uneven':result['guide']['ContinuousBins' if t['guide']=='bins' else 'ContinuousSteps']['breaks']=[1,2,8,16]
 return result
def layer(t):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha','v',descriptor(t))
def build(t):
 values=[float(v) if isinstance(v,str) else v for v in t['inputs']]
 data=c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.column(values,kind='float64').nullable(True)},name='data')
 draft=c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry)
 if t['channel']=='colour':draft=draft.aes(c.aes().x('x').y(1.).color('v').color_scale('v')).scale(c.color_mapped('v',descriptor(t)))
 else:draft=draft.aes(c.aes().x('x').y(1.))
 return data,draft.layer(layer(t)).build()
def check(t,chart):
 state=chart.semantics()['layers'][0]
 entries=(state.get('color_legend') or {}).get('numeric_breaks',[]) if t['channel']=='colour' else [e for group in state.get('numeric_value_guides',{}).values() for e in group]
 entries=[e for e in entries if e['visible']]
 actual={'values':[e['transformed'] if transforms else e['value'] for e in entries],'labels':[e['label'] for e in entries]}
 wanted=t['result']['guides']
 expected=({'values':wanted[0]['scale_values'] if transforms else wanted[0]['source_values'] if intervals else t['result']['raw_breaks'] if t['guide']=='colourbar' else wanted[0]['values'],'labels':wanted[0]['labels']} if wanted else {'values':[],'labels':[]})
 if transforms:
  assert actual['labels']==expected['labels'] and len(actual['values'])==len(expected['values']), (t,actual)
  assert all(math.isclose(a,b,rel_tol=4e-14,abs_tol=4e-14) for a,b in zip(actual['values'],expected['values'])),(t,actual)
  if t['palette_mode']=='index':
   styles=state.get('styles',[]);assert len(styles)==len(t['result']['mapped'])
   for style,mapped in zip(styles,t['result']['mapped']):
    if t['channel']=='size':assert math.isclose(style['radius'],mapped,rel_tol=4e-14,abs_tol=4e-14),(t,style,mapped)
    elif t['channel']=='alpha':assert style['color']['alpha']==math.floor(mapped*255+.5),(t,style,mapped)
    else:assert style['color']==dict(zip(('red','green','blue','alpha'),[int(mapped[i:i+2],16) for i in (1,3,5)]+[255]))
 else:assert actual==expected,(t,actual)
 ramp=(state.get('color_legend') or {}).get('colorbar',[])
 if t['guide']=='colourbar' and t['channel']=='colour' and wanted:
  assert len(ramp)==300
  for i,sample in enumerate(ramp):
   assert math.isclose(sample['value'],wanted[0]['decor_values'][i],rel_tol=0,abs_tol=8e-15)
   text=wanted[0]['decor_colors'][i];assert sample['color']==dict(zip(('red','green','blue','alpha'),[int(text[j:j+2],16) for j in (1,3,5)]+[255]))
 else:assert not ramp
 return {'keys':actual,'styles':state.get('styles',[]),**({'colorbar':ramp} if colorbar else {})}
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
for index,t in enumerate(cases):
 owned=[]
 try:
  data,p=build(t);owned.extend([data,p]);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  if t['guide']=='colourbar' and t['breaks']!='null':
   old=json.loads(wire);assert old['version']==57;old['version']=56
   try:c.Plot.from_json(json.dumps(old),registry)
   except c.ChartError:pass
   else:raise AssertionError('colorbar downgrade accepted')
  if intervals and t['breaks']!='null':
   old=json.loads(wire);assert old['version']==58;old['version']=57
   try:c.Plot.from_json(json.dumps(old),registry)
   except c.ChartError:pass
   else:raise AssertionError('interval downgrade accepted')
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer(t)).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);records.append({'index':index,'state':state,**check(t,chart)});assert restored.to_json()==wire
  if (t['population']=='ordinary' or t.get('palette_mode')=='index') and t['guide'] in ('default','colourbar','bins','coloursteps'):
   request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f"{t['transform'] + '-' + t['palette_mode'] + '-' if transforms else ''}{t['channel']}-{t['breaks']}{'-' + t['guide'] if t['guide']!='default' else ''}.{fmt}").write_bytes(frame.export(fmt))
 finally:
  for obj in reversed(owned):obj.dispose()
assert len(records)==(324 if transforms else 216 if intervals else 351 if colorbar else 243)
(out/'records.json').write_text(json.dumps(records,sort_keys=True,allow_nan=False));options.dispose();output.dispose();registry.dispose()
print('PASS Python:',len(records),'continuous guide-selection states;',len(list(out.glob('*.svg')))*3,'publications')
