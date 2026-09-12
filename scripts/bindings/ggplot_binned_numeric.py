"""FIX-GG04: binned numeric scale mapping, separate from R guide-build failures."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
def descriptor(case):
 kind=case['palette'];control=case['control'];area=kind=='area'
 breaks={'Explicit':[1,3,5,7,9]} if control=='explicit' else {'Explicit':[]} if control in ('empty_breaks','null_breaks') else {'Equal':8 if control=='count_eight' else 16} if control in ('count_eight','count_sixteen') else {'Nice':5}
 limits=[-1,5] if control=='limits' else [None,5] if control=='partial_limits' else None
 return {'function':{'Interpolated':{'normalization':{'Ggplot':{'family':'Linear','domain':[0,1],'reverse':False,'rescaler':'Maximum' if area else 'Range'}},'output':{'Interpolate':{'operation':'PowerRange','range':[0,6] if area else [0.1,1] if kind=='alpha' else [1,6],'exponent':0.5 if kind in ('size','area') else 1,'absolute':area}},'unknown':{'kind':'Missing'}}},'training':'Eligible','ggplot':{'Binned':{'empty_population':False,'nonfinite_population':False,'limits':limits,'oob':'Squish','breaks':breaks,'right':control!='left'}},'guide':{'Binned':'Automatic'}}
records=[]
for index,case in enumerate(json.loads((ROOT/'fixtures/parity/ggplot2/binned-numeric-palettes.json').read_text())['cases']):
 if case['control']=='left' and case['palette'] in ('size','linewidth'):continue
 owned=[];expected=case['raw_result'];failure=expected.get('error')
 try:
  inputs=case['inputs'];kind=case['palette'];channel='StrokeWidth' if kind=='linewidth' else 'Alpha' if kind=='alpha' else 'Size'
  data=c.Data.columns({'x':c.column(list(map(float,range(len(inputs)))),kind='float64'),'v':c.column([v or 0 for v in inputs],kind='float64').validity([v is not None for v in inputs])});owned.append(data)
  def layer():return (c.rule() if kind=='linewidth' else c.points()).name('marks').stroke('#000000').numeric_scale(channel,'v',descriptor(case))
  plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(1.).x2('x').y2(2.)).layer(layer()).build();owned.append(plot)
  wire=plot.to_json();version=json.loads(wire)['version'];restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
  for state in ('original','edited'):
   current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
   if current is not restored:owned.append(current)
   chart=current.chart();owned.append(chart);actual=chart.semantics()['layers'][0];assert not failure,(index,failure)
   wanted=expected['values']
   if len(wanted)==1:wanted=wanted*len(inputs)
   if kind!='alpha':wanted=[v for v in wanted if v is not None]
   styles=actual.get('styles',[]);assert len(styles)==len(wanted),(index,styles,wanted)
   for style,want in zip(styles,wanted):
    if kind=='alpha':assert style['color']['alpha']==(255 if want is None else math.floor(want*255+0.5)),(index,style,want)
    else:assert abs(style['stroke_width' if kind=='linewidth' else 'radius']-want)<2e-12,(index,style,want)
   records.append({'index':index,'state':state,'version':version,'styles':styles})
   if case['population']=='many' and case['control']=='count_eight' and state=='original':
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
    for fmt in ('svg','pdf','png'):(out/f'numeric-{kind}.{fmt}').write_bytes(frame.export(fmt))
 except c.ChartError as error:
  diagnostic=json.loads(str(error));assert failure,(index,diagnostic);assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(index,diagnostic)
  records.append({'index':index,'error':diagnostic['code']})
 finally:
  for value in reversed(owned):value.dispose()
assert len(records)==372,len(records)
(out/'binned-numeric-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();print('PASS Python: 372 binned numeric states; twelve publication files.')
