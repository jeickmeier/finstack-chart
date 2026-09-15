"""FIX-GG04 primary colour/fill identity callback reference proof."""
import json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
fixture=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-identity-functions.json').read_text());cases=fixture['cases']
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current');records=[]
def descriptor(t):
 s={'training':'Eligible','function':{'GgplotDiscreteIdentity':{'limits':None,'levels':None if t['levels'] is None else [{'Text':v} for v in t['levels']],'drop':False,'na_translate':True,'guide':t['guide']=='legend','observed':[]}}}
 if t['limit_mode']!='none':s['limits_function']={'operation':{'id':'example.discrete_limits','version':'1'},'parameters':{'mode':t['limit_mode']}}
 if t['break_mode']!='auto':s['breaks_function']={'operation':{'id':'example.discrete_breaks','version':'1'},'parameters':t['break_mode']}
 return s
def layer():return c.points().name('marks').aesthetic_value('Shape',{'kind':'Number','value':21})
def build(t,data,spec=None):
 mapping=c.aes().x('x').y(1.)
 mapping=mapping.fill('v').fill_scale('identity') if t['aesthetic']=='fill' else mapping.color('v').color_scale('identity')
 return c.plot(data).profile('Ggplot2_4_0_3').with_registry(registry).aes(mapping).scale(c.color_mapped('identity',spec or descriptor(t))).layer(layer()).build()
def check_paint(paint,value):
 if value is None:assert paint['alpha']==0,paint
 else:assert paint==fixture['paints'][value],(paint,value)
for index,t in enumerate(cases):
 owned=[]
 try:
  values=t['inputs'];data=c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'v':c.column(values,kind='string').nullable(True)});owned.append(data)
  p=build(t,data);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==(33 if t['break_mode']!='auto' else 28 if t['limit_mode']!='none' else 17)
  restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','layer_edit','theme_edit'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build() if state=='layer_edit' else restored.edit().theme(c.theme()).build()
   if current is not restored:owned.append(current)
   try:
    chart=current.chart();owned.append(chart);semantics=chart.semantics()['layers'][0]
    assert 'error' not in t['result'],(index,t)
    legend=semantics.get('paint_legends',{}).get('Fill') if t['aesthetic']=='fill' else semantics.get('color_legend')
    entries=(legend or {}).get('entries',[]);expected=next(iter(t['result']['keys']),{'mapped':[],'labels':[]})
    assert len(entries)==len(expected['mapped']),(index,entries,expected)
    for (label,paint),value,text in zip(entries,expected['mapped'],expected['labels']):
     assert label==(text if text is not None else 'NA'),(index,label,text);check_paint(paint,value)
    paints=[s['fill'] if t['aesthetic']=='fill' else s['color'] for s in semantics.get('styles',[])]
    wanted=[v for v in t['result']['mapped'] if t['aesthetic']=='fill' or v is not None]
    assert len(paints)==len(wanted),(index,paints,wanted)
    for paint,value in zip(paints,wanted):check_paint(paint,value)
    records.append({'index':index,'state':state,'entries':entries,'paints':paints})
    sample=t['guide']=='legend' and ((t['population']=='factor' and t['break_mode']=='named_reverse' and t['limit_mode'] in ('none','reverse')) or (t['population']=='nullable' and t['break_mode']=='domain' and t['limit_mode']=='none'))
    if state=='original' and sample:
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
     for fmt in ('svg','pdf','png'):(out/f'{index:03d}-identity.{fmt}').write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in t['result'] and error.code=='CHART_VALIDATION',(index,t,str(error));records.append({'index':index,'state':state,'error':error.code})
   assert restored.to_json()==wire
 finally:
  for obj in reversed(owned):obj.dispose()
owned=[]
try:
 t=next(t for t in cases if t['aesthetic']=='colour' and t['population']=='ordinary' and t['guide']=='legend' and t['limit_mode']=='null' and t['break_mode']=='domain')
 data=c.Data.columns({'x':c.column([0.,1.],kind='float64'),'v':c.column(t['inputs'],kind='string')});owned.append(data)
 spec=descriptor(t);spec['resolved_discrete_limits_null']=True;spec['function']['GgplotDiscreteIdentity']['limits']=[]
 p=build(t,data,spec);owned.append(p);wire=p.to_json();assert json.loads(wire)['version']==63
 restored=c.Plot.from_json(wire,registry);owned.append(restored);assert restored.to_json()==wire
 chart=restored.chart();owned.append(chart);assert not (chart.semantics()['layers'][0].get('color_legend') or {}).get('entries',[])
 old=json.loads(wire);old['version']=62
 for payload,extensions in ((json.dumps(old),registry),(wire,None)):
  try:unexpected=c.Plot.from_json(payload,extensions)
  except c.ChartError:pass
  else:unexpected.dispose();raise AssertionError('Downgrade or missing registration accepted')
 records.append({'wire':63,'null_limits':True})
finally:
 for obj in reversed(owned):obj.dispose()
assert len(records)==577,len(records)
assert sum(len(list(out.glob('*.'+fmt))) for fmt in ('svg','pdf','png'))==18
(out/'records.json').write_text(json.dumps(records,sort_keys=True));options.dispose();output.dispose();registry.dispose()
print('PASS Python identity callbacks: 577 lifecycle/wire states and 18 publications')
