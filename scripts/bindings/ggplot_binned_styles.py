"""FIX-GG04: binned count palettes through primary authoring and publication."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def descriptor(case):
 kind=case['palette'];control=case['control']
 palette='LineType' if kind=='linetype' else {'Brewer':{'id':'Blues','reverse':False}} if kind=='brewer' else {'Shape':{'solid':kind=='solid'}}
 breaks={'Explicit':[1,3,5,7,9]} if control=='explicit' else {'Explicit':[]} if control in ('empty_breaks','null_breaks') else {'Equal':8 if control=='count_eight' else 16} if control in ('count_eight','count_sixteen') else {'Nice':5}
 limits=[-1,5] if control=='limits' else [None,5] if control=='partial_limits' else None
 return {'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Range'}},'output':'Identity','unknown':{'kind':'Color','value':{'space':'Rgb','channels':{'r':127,'g':127,'b':127,'opacity':1}}} if kind=='brewer' else {'kind':'Missing'}}},'training':'Eligible','ggplot':{'Binned':{'palette':palette,'empty_population':False,'nonfinite_population':False,'limits':limits,'oob':'Squish','breaks':breaks,'right':control!='left'}},'guide':{'Binned':'Automatic'}}
records=[]
for index,case in enumerate(json.loads((ROOT/'fixtures/parity/ggplot2/binned-style-palettes.json').read_text())['cases']):
 owned=[];expected=case['result'];failure=expected.get('error') or expected.get('draw_error')
 try:
  inputs=case['inputs'];kind=case['palette'];channel='LineType' if kind=='linetype' else 'Stroke' if kind=='brewer' else 'Shape'
  data=c.Data.columns({'x':c.column(list(map(float,range(len(inputs)))),kind='float64'),'v':c.column([v or 0 for v in inputs],kind='float64').validity([v is not None for v in inputs])});owned.append(data)
  def layer():return c.points().name('marks') if kind=='brewer' else (c.rule() if kind=='linetype' else c.points()).name('marks').value_scale(channel,'v',descriptor(case))
  mapping=c.aes().x('x').y(1.).x2('x').y2(2.)
  if kind=='brewer':mapping=mapping.stroke('v').stroke_scale('bins').fill('v').fill_scale('bins')
  builder=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(layer())
  if kind=='brewer':builder=builder.scale(c.color_mapped('bins',descriptor(case)))
  plot=builder.build();owned.append(plot)
  wire=plot.to_json();assert json.loads(wire)['version']==27
  restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=chart.semantics()['layers'][0];assert not failure,(index,failure)
   wanted=[v for v in expected['values'] if v is not None or kind=='linetype'];aesthetics=actual.get('aesthetics',[]);styles=actual.get('styles',[])
   assert len(aesthetics)==len(wanted),(index,aesthetics,wanted)
   for row_index,(row,want) in enumerate(zip(aesthetics,wanted)):
    got=row.get(channel)
    if kind in ('solid','hollow'):assert got=={'kind':'Number','value':want},(index,got,want)
    elif kind=='linetype':assert got==({'kind':'Missing'} if want is None else {'kind':'Text','value':want}),(index,got,want)
    else:
     rgb=styles[row_index]['stroke'];want='#7F7F7F' if want=='grey50' else want;assert [rgb[k] for k in ('red','green','blue')]==[int(want[i:i+2],16) for i in (1,3,5)],(index,rgb,want)
   records.append({'index':index,'state':state,'version':27,'styles':actual.get('styles',[]),'aesthetics':aesthetics})
   if case['population']=='many' and case['control']=='count_eight' and state=='original':
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/f'{kind}.{fmt}').write_bytes(frame.export(fmt))
 except c.ChartError as error:
  diagnostic=json.loads(str(error));assert failure,(index,diagnostic)
  assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(index,diagnostic)
  records.append({'index':index,'error':diagnostic['code']})
 finally:
  for value in reversed(owned):value.dispose()
assert len(records)==392,len(records)
(out/'binned-style-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();print('PASS Python:',len(records),'binned count palette states; twelve publication files.')
