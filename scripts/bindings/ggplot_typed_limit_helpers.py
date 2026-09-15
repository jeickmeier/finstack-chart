"""GG2-03: typed endpoint helpers use the common positional scale engine."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
fixture='named-limit-dispatch.json' if '--named-limits' in sys.argv else 'scale-limit-helpers.json'
cases=json.loads((ROOT/'fixtures/parity/ggplot2'/fixture).read_text())['cases']
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,640).dpi(96).basis('current');records=[]
def temporal(t):return t['family'] in ('date','datetime')
def data_for(t,values=None):
 values=t['inputs'] if values is None else values
 v=c.timestamps([round(v*86400000) for v in values],'ms','UTC') if temporal(t) else c.column([str(int(v)) for v in values] if t['family']=='character' else values,kind='category' if t['family']=='character' else 'float64')
 return c.Data.columns({'v':v,'i':list(range(1,len(values)+1))},keys=list(range(1,len(values)+1)))
def axis_for(t):
 values=t['authored']
 if temporal(t):limits={'Date' if t['family']=='date' else 'Datetime':[None if v is None else {'Timestamp':{'value':str(round(v*86400)),'unit':'Seconds'}} for v in values]}
 elif t['family']=='character':limits={'Discrete':['Null' if v is None else {'Text':str(int(v))} for v in values]}
 else:limits={'Numeric':values}
 return (c.xlim(limits) if t['axis']=='x' else c.ylim(limits)).range(100. if t['axis']=='x' else 540.,540. if t['axis']=='x' else 100.).guide_geometry({'labels':'Preserve'})
def build(t,d,origin):
 v={'field':'v','origin':str(origin)} if temporal(t) else 'v'
 mapping=c.aes().x(v if t['axis']=='x' else 'i').y(v if t['axis']=='y' else 'i')
 p=c.plot(d).profile('Ggplot2_4_0_3').aes(mapping).layer(c.points().name('marks'))
 return (p.x_axis(axis_for(t)).y_axis(c.y_axis().visible(False)) if t['axis']=='x' else p.y_axis(axis_for(t)).x_axis(c.x_axis().visible(False))).build()
def checked(t,chart,frame,expected=True):
 scene=frame.scene();points=[None]*len(t['inputs'])
 for item,targets in zip(scene['items'],scene['targets']):
  if targets and 'Point' in item['primitive']:
   assert len(targets)==1 and 'Source' in targets[0]
   points[int(targets[0]['Source']['key'])-1]=((item['primitive']['Point']['center'][t['axis']]-100.) if t['axis']=='x' else (540.-item['primitive']['Point']['center'][t['axis']]))/440.
 ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']==('Bottom' if t['axis']=='x' else 'Left'))['ticks']
 if expected:
  for actual,wanted in zip(points,t['result']['point_positions']):
   assert (actual is None and wanted is None) or (actual is not None and wanted is not None and math.isclose(actual,wanted,abs_tol=1e-12,rel_tol=0)),(t,points)
  expected_ticks=sorted([(position,'NA' if label is None else label) for label,position in zip(t['result']['labels'],t['result']['break_positions']) if position is not None])
  actual_ticks=sorted([(((tick['position']-100.) if t['axis']=='x' else (540.-tick['position']))/440.,tick['label']) for tick in ticks])
  assert len(actual_ticks)==len(expected_ticks),(t,ticks,expected_ticks)
  for (actual,label),(wanted,expected_label) in zip(actual_ticks,expected_ticks):
   assert label==expected_label and math.isclose(actual,wanted,rel_tol=0,abs_tol=1e-12),(t,ticks,expected_ticks)
 return {'points':points,'ticks':ticks,'domains':chart.semantics()['layers'][0]['domains']}
for index,t in enumerate(cases):
 for origin in ([0,123456789] if temporal(t) and 'error' not in t['result'] else [0]):
  owned=[]
  try:
   d=data_for(t);owned.append(d)
   try:p=build(t,d,origin);owned.append(p)
   except c.ChartError as error:
    assert 'error' in t['result'],(t,error)
    records.append({'index':index,'origin':origin,'rejected':error.code});continue
   assert 'error' not in t['result'],t
   wire=p.to_json();assert json.loads(wire)['version']==(43 if temporal(t) else 31 if t['family']=='numeric' else 22)
   restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
   edit=restored.edit();owned.append(edit);edit=edit.x_axis(axis_for(t)) if t['axis']=='x' else edit.y_axis(axis_for(t));owned.append(edit);edited=edit.build();owned.append(edited)
   for state,current in [('original',p),('restored',restored),('edited',edited)]:
    chart=current.chart();request=output.request(current,options);frame=request.prepare();owned.extend([chart,request,frame]);held=frame.scene()
    records.append({'index':index,'origin':origin,'state':state,**checked(t,chart,frame)})
    if state=='original' and origin==0 and t['mode'] in ('ascending','descending'):
     for fmt in ['svg','pdf','png']:(out/f"{t['family']}-{t['axis']}-{t['mode']}.{fmt}").write_bytes(frame.export(fmt))
   if temporal(t) and t['mode']=='lower_missing':
    replacement=data_for(t,[-2,0,2,4,6]);owned.append(replacement)
    tb=chart.transaction();owned.append(tb);tb=tb.replace(d,replacement);owned.append(tb);tx=tb.build();owned.append(tx);assert 'Applied' in chart.commit(tx)
    fresh=build(t,replacement,origin);batch=fresh.chart();ar=output.request(chart,options);br=output.request(batch,options);af=ar.prepare();bf=br.prepare();owned.extend([fresh,batch,ar,br,af,bf])
    actual=checked(t,chart,af,False);assert actual==checked(t,batch,bf,False);assert af.export('png')==bf.export('png')
    assert frame.scene()==held and p.to_json()==wire
    records.append({'index':index,'origin':origin,'state':'replaced',**actual})
  finally:
   for obj in reversed(owned):obj.dispose()
# Exact host integers and authored endpoints must survive above 2**53.
owned=[]
try:
 epoch=9007199254740991
 def exact_data(values):return c.Data.columns({'v':c.timestamps([epoch+v for v in values],'ns','UTC')},keys=list(range(1,len(values)+1)))
 def exact_plot(data,endpoint):return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x({'field':'v','origin':str(epoch)}).y(1.)).layer(c.points()).x_axis(c.x_axis().scale(c.scale_utc()).temporal_limits([None,{'Timestamp':{'value':str(endpoint),'unit':'Nanoseconds'}}])).build()
 d=exact_data([-2,0,2,4]);p=exact_plot(d,epoch+2);wire=p.to_json();restored=c.Plot.from_json(wire);owned.extend([d,p,restored]);assert restored.to_json()==wire
 for state,current in [('original',p),('restored',restored)]:
  chart=current.chart();owned.append(chart);snapshot=chart.semantics();layer=snapshot['layers'][0]
  assert layer['domains']['x']=={'minimum':-2.,'maximum':2.} and len(layer['targets'])==3
  records.append({'index':'exact_ns','state':state,'domains':layer['domains'],'count':len(layer['targets'])})
 replacement=exact_data([-4,0,4]);owned.append(replacement);tb=chart.transaction();owned.append(tb);tb=tb.replace(d,replacement);owned.append(tb);tx=tb.build();owned.append(tx);assert 'Applied' in chart.commit(tx)
 layer=chart.semantics()['layers'][0];assert layer['domains']['x']=={'minimum':-4.,'maximum':0.} and len(layer['targets'])==2
 assert snapshot['layers'][0]['domains']['x']=={'minimum':-2.,'maximum':2.} and p.to_json()==wire
 records.append({'index':'exact_ns','state':'replaced','domains':layer['domains'],'count':len(layer['targets'])})
 for delta in [9007199254740993,-9007199254740993]:
  try:
   bad=exact_plot(d,epoch+delta);owned.append(bad);bc=bad.chart();owned.append(bc);bc.semantics();raise AssertionError('imprecise endpoints accepted')
  except c.ChartError as error:
   assert error.code=='CHART_PRECISION_LOSS',error
   records.append({'index':'exact_ns','delta':str(delta),'rejected':error.code})
finally:
 for obj in reversed(owned):obj.dispose()

assert len(records)==181,len(records)
(out/'records.json').write_text(json.dumps(records,allow_nan=False));options.dispose();output.dispose()
print('PASS',len(records),'typed helper states')
