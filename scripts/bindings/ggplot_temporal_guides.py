"""FIX-GG04: temporal guide arguments through primary Python authoring and publication."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def descriptor(case,origin,unit,factor):
 control=case['control'];date=case['kind']=='date'
 args={'date':date,'breaks':'Automatic','count':None,'labels':'Automatic','format':None}
 if control=='null_breaks':args['breaks']='None'
 if control=='empty_breaks':args['breaks']={'Explicit':[]}
 if control in ('explicit_breaks','explicit_labels','bad_labels'):args['breaks']={'Explicit':[v*factor for v in (-1,0,2,5)]}
 if control=='width':args['breaks']={'Width':case.get('width', '2 days' if date else '2 secs')}
 if control=='count_two':args['count']=2
 if control=='null_labels':args['labels']='Hidden'
 if control=='explicit_labels':args['labels']={'Explicit':['Before','Start','Two','After']}
 if control=='bad_labels':args['labels']={'Explicit':['A','B']}
 if control=='format':args['format']={'pattern':'%Y-%m-%d' if date else '%H:%M:%S','locale':None}
 limits=[-factor,5*factor] if control=='limits' else [None,5*factor] if control=='partial_limits' else None
 return {'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range','timestamp':{'origin':str(origin),'unit':unit,'date':date}}},'output':{'Interpolate':{'operation':'PowerRange','range':[1,6],'exponent':0.5,'absolute':False}},'unknown':{'kind':'Missing'}}},'training':'Eligible','ggplot':{'Continuous':{'empty_population':False,'nonfinite_population':False,'limits':limits,'oob':'Censor'}},'guide':'Hidden' if control=='hidden_guide' else {'Temporal':{'origin':str(origin),'unit':unit,'zone':'Utc','arguments':args}}}
records=[]
cases=json.loads((ROOT/'fixtures/parity/ggplot2/temporal-aesthetic-guides.json').read_text())['cases']+json.loads((ROOT/'fixtures/parity/ggplot2/temporal-aesthetic-widths.json').read_text())['cases']
for index,case in enumerate(cases):
 units=[('us','Microseconds',1000000)] if 'width' in case else [('s','Seconds',1),('ms','Milliseconds',1000),('us','Microseconds',1000000),('ns','Nanoseconds',1000000000)]
 for unit,variant,multiplier in units:
  owned=[];expected=case['result'];failure=expected.get('error') or expected.get('draw_error')
  try:
   factor=multiplier*(86400 if case['kind']=='date' else 1);origin=1704067200*multiplier;inputs=case['inputs']
   def stamp(value):
    value=(19723 if case['kind']=='date' else 1704067200) if value is None else value
    whole=math.trunc(value);return whole*factor+round((value-whole)*factor)
   data=c.Data.columns({'x':c.column(list(map(float,range(len(inputs)))),kind='float64'),'y':c.column(list(map(float,range(len(inputs)))),kind='float64'),'when':c.timestamps([stamp(v) for v in inputs],unit,'UTC').validity([v is not None for v in inputs])});owned.append(data)
   scale=descriptor(case,origin,variant,factor);source={'field':'when','origin':str(origin)}
   def layer():return c.points().name('marks').stroke('#000000').numeric_scale('Size',source,scale)
   plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer()).build();owned.append(plot)
   wire=plot.to_json();version=json.loads(wire)['version'];assert version in (25,26)
   restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);actual=chart.semantics()['layers'][0];assert not failure,(index,failure)
    styles=actual.get('styles',[]);wanted=[v for v in expected['values'] if v is not None]
    assert len(styles)==len(wanted),(index,unit,styles,wanted)
    for style,value in zip(styles,wanted):assert abs(style['radius']-value)<2e-12,(index,unit,style,value)
    encoding=actual.get('numeric_scales',{}).get('Size');retained=(encoding or json.loads(current.to_json())['definition']['layers'][0]['numeric_scales']['Size'])['scale'];assert retained['function']['Interpolated']['normalization']['Ggplot']['timestamp']['origin']==str(origin)
    if case['population']=='short' and case['control']=='default':
     standalone=c.StandaloneScale.from_json(json.dumps({'version':5 if case['kind']=='date' else 4,'spec':retained['function']}));owned.append(standalone)
     copied=c.StandaloneScale.from_json(standalone.to_json());owned.append(copied);assert copied.to_json()==standalone.to_json()
     for value,expected_value in zip(inputs,expected['values']):assert abs(copied.map(float(value*factor-origin))-expected_value)<2e-12
    records.append({'index':index,'unit':unit,'state':state,'version':version,'styles':styles,'scale':retained})
    if unit=='s' and state=='original' and ((case['kind'],case['population'],case['control']) in [('date','short','default'),('datetime','short','width'),('date','constant','null_breaks')]):
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
     for fmt in ('svg','pdf','png'):(out/f"{case['kind']}-{case['population']}-{case['control']}.{fmt}").write_bytes(frame.export(fmt))
  except c.ChartError as error:
   diagnostic=json.loads(str(error));assert failure,(index,unit,diagnostic)
   expected_code='CHART_VALIDATION' if 'width' in case or 'labels' in failure else 'CHART_NUMERICAL_DOMAIN';assert diagnostic['code']==expected_code,(index,diagnostic,failure)
   records.append({'index':index,'unit':unit,'error':diagnostic['code']})
  finally:
   for value in reversed(owned):value.dispose()
assert len(records)==1236,len(records)
(out/'guide-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();print('PASS Python: 1236 temporal guide states and nine publication files.')
