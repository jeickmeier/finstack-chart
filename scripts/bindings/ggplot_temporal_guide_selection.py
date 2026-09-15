"""GG-04: actual temporal legend/colorbar selection, replay, edits and publication."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
label_cases='--interval-labels' in sys.argv[3:]
fixture='temporal-interval-label-functions.json' if label_cases else 'temporal-guide-selection.json'
cases=json.loads((ROOT/'fixtures/parity/ggplot2'/fixture).read_text())['cases']
registry=c.ExtensionRegistry.example()
records=[]
def colorbar(t):return t['guide']=='colourbar' or t['guide']=='default' and t['channel']=='colour'
def descriptor(t,origin,variant,factor):
 channel=t['channel'];date=t['kind']=='date'
 output={'Interpolate':{'operation':'GgplotPalette','spec':{'Gradient':{'colors':[{'red':19,'green':43,'blue':67,'alpha':255},{'red':86,'green':177,'blue':247,'alpha':255}],'values':None}}}} if channel=='colour' else {'Interpolate':{'operation':'PowerRange','range':[.1,1] if channel=='alpha' else [1,6],'exponent':.5 if channel=='size' else 1,'absolute':False}}
 args={'date':date,'breaks':'None' if t['breaks']=='null' else {'Explicit':[]} if t['breaks']=='empty' else {'Explicit':[-factor,0,factor,factor,3*factor,20*factor,{'number':'NaN'}]} if t['breaks']=='explicit' else 'Automatic'}
 if label_cases:
  args['labels']={'Registered':{'operation':{'id':'example.scale_labels','version':'1'},'parameters':t['label_mode']}}
  if t['format_mode']=='explicit':args['format']={'pattern':'%d/%m' if date else '%Hh%M','locale':None}
 return {'training':'Eligible','function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,10*factor],'reverse':False,'rescaler':'Range','timestamp':{'origin':str(origin),'unit':variant,'date':date}}},'output':output,'unknown':{'kind':'Missing'}}},'ggplot':{'Continuous':{'limits':[0,10*factor] if t['limits']=='full' else None,'oob':'Censor'}},'guide':'Hidden' if t['guide']=='none' else {('TemporalColorbar' if colorbar(t) else 'TemporalBins' if t['guide']=='bins' else 'TemporalSteps' if t['guide']=='coloursteps' else 'Temporal'):{'origin':str(origin),'unit':variant,'zone':'Utc','arguments':args}}}
def layer(t,source,scale):return c.points().name('marks') if t['channel']=='colour' else c.points().name('marks').numeric_scale('Size' if t['channel']=='size' else 'Alpha',source,scale)
def check(t,chart,factor,multiplier):
 state=chart.semantics()['layers'][0];colour=t['channel']=='colour';date=t['kind']=='date'
 entries=(state.get('color_legend') or {}).get('numeric_breaks',[]) if colour else [e for group in state.get('numeric_value_guides',{}).values() for e in group]
 entries=[e for e in entries if e['visible']];guides=t['result']['guides']
 wanted=[v for v in t['result']['raw_breaks'] if isinstance(v,(int,float)) and t['result']['limits'][0]<=v<=t['result']['limits'][1]] if guides else []
 if guides and t['guide'] in ('bins','coloursteps'):wanted=guides[0]['source_values'][:len(guides[0]['values'])]
 actual={'values':[(18262+e['transformed']/factor) if date else (1577836800+e['transformed']/multiplier) for e in entries],'labels':[e['label'] for e in entries]}
 assert actual=={'values':wanted,'labels':guides[0]['labels'] if guides else []},(t,actual)
 assert len(wanted)==(len(guides[0]['values']) if guides else 0),t
 if guides and t['guide'] in ('bins','coloursteps'):
  for entry,value in zip(entries,guides[0]['mapped']):
   mapped=entry['mapped'];actual_value=None if mapped['kind'] in ('Missing','Null') else mapped['value']
   if mapped['kind']=='Color':
    assert actual_value['space']=='Rgb';channels=actual_value['channels'];assert channels==dict(zip(('r','g','b','opacity'),[int(value[i:i+2],16) for i in (1,3,5)]+[1])),(t,entry,value)
   elif isinstance(value,(int,float)):assert math.isclose(actual_value,value,rel_tol=3e-12,abs_tol=3e-12),(t,entry,value)
   else:assert actual_value==value,(t,entry,value)
 styles=state.get('styles',[]);assert len(styles)==len(t['result']['mapped']),t
 def paint(text):return dict(zip(('red','green','blue','alpha'),[int(text[i:i+2],16) for i in (1,3,5)]+[255]))
 for style,value in zip(styles,t['result']['mapped']):
  if colour:assert style['color']==paint(value),(t,style,value)
  elif t['channel']=='size':assert math.isclose(style['radius'],value,rel_tol=2e-12,abs_tol=2e-12),(t,style,value)
  else:assert style['color']['alpha']==round(value*255),(t,style,value)
 ramp=(state.get('color_legend') or {}).get('colorbar',[]);expected=guides[0]['decor_values'] if guides else []
 assert len(ramp)==len(expected),(t,len(ramp),len(expected))
 for i,(sample,value) in enumerate(zip(ramp,expected)):
  assert abs(sample['value']-value)<=4*sys.float_info.epsilon*max(1,abs(value)),(t,i,sample,value)
  assert sample['color']==paint(guides[0]['decor_colors'][i]),(t,i,sample)
 return {'keys':actual,'styles':styles,'colorbar':ramp}
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
for index,t in enumerate(cases):
 for unit,variant,multiplier in [('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)]:
  owned=[]
  try:
   factor=multiplier*(86400 if t['kind']=='date' else 3600);origin=1577836800*multiplier;values=t['inputs']
   data=c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.timestamps([origin+int(v*factor) for v in values],unit,'UTC')});owned.append(data)
   source={'field':'v','origin':str(origin)};scale=descriptor(t,origin,variant,factor);draft=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3')
   draft=draft.aes(c.aes().x('x').y(1.).color(source).color_scale('v')).scale(c.color_mapped('v',scale)) if t['channel']=='colour' else draft.aes(c.aes().x('x').y(1.))
   p=draft.layer(layer(t,source,scale)).build();owned.append(p);wire=p.to_json();restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
   if colorbar(t) or t['guide'] in ('bins','coloursteps'):
    version=59 if colorbar(t) else 60
    old=json.loads(wire);assert old['version']==version;old['version']=version-1
    try:c.Plot.from_json(json.dumps(old),registry)
    except c.ChartError:pass
    else:raise AssertionError('temporal colorbar downgrade accepted')
   if 'error' in t['result']:
    try:
     rejected=restored.chart();owned.append(rejected);rejected.semantics()
    except c.ChartError:records.append({'index':index,'unit':unit,'state':'rejected'})
    else:raise AssertionError(('reference rejected',t))
    continue
   for state in ('original','layer_edit','theme_edit'):
    current=restored if state=='original' else restored.edit().layer('marks',layer(t,source,scale)).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);records.append({'index':index,'unit':unit,'state':state,**check(t,chart,factor,multiplier)});assert restored.to_json()==wire
   selected=(t['guide']=='default' and t['breaks']=='auto' and t['limits']==('full' if t['population']=='ordinary' else 'none')) or (t['population']=='ordinary' and t['limits']=='full' and ((t['guide']=='colourbar' and t['breaks']=='auto') or (t['guide']=='default' and t['breaks']=='null')))
   selected=selected or (t['guide'] in ('bins','coloursteps') and t['limits']=='full' and t['population']=='ordinary')
   if label_cases:selected=t['label_mode']=='indexed' and t['limits']=='full' and ((t['population']=='ordinary' and t['breaks'] in ('auto','explicit')) or (t['population']=='empty' and t['breaks']=='auto'))
   if unit=='s' and selected:
    request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
    suffix=f"-format-{t['format_mode']}" if label_cases else ''
    for fmt in ('svg','pdf','png'):(out/f"{t['kind']}-{t['channel']}-{t['population']}-{t['guide']}-{t['breaks']}{suffix}.{fmt}").write_bytes(frame.export(fmt))
  finally:
   for obj in reversed(owned):obj.dispose()
assert len(records)==(21872 if label_cases else 7680),len(records);assert len(list(out.glob('*.svg')))==(60 if label_cases else 66)
(out/'records.json').write_text(json.dumps(records,allow_nan=False));options.dispose();output.dispose()
print(f'PASS Python: {len(records)} temporal guide-selection states and {len(list(out.glob("*.svg")))*3} publications.')
