"""FIX-GG04: absolute datetime palette normalization through the actual Python host."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
def hide(value):
 if isinstance(value,dict):
  if isinstance(value.get('guide'),dict) and 'Temporal' in value['guide']:value['guide']='Hidden'
  for child in value.values():hide(child)
 elif isinstance(value,list):
  for child in value:hide(child)
records=[]
for index,case in enumerate(json.loads((ROOT/'fixtures/parity/ggplot2/temporal-aesthetic-precision.json').read_text())['cases']):
 owned=[]
 try:
  channel=case['channel'];stamps=[case['epoch']*1000000000+round(v*1e9) for v in case['offsets']]
  data=c.Data.columns({'x':c.column([0.,1.,2.,3.],kind='float64'),'y':c.column([0.,1.,2.,3.],kind='float64'),'when':c.timestamps(stamps,'ns','UTC')});owned.append(data)
  mapping=getattr(c.aes().x('x').y('y'),'color' if channel=='colour' else channel)('when')
  plot=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(c.points().stroke('#000000')).build();owned.append(plot)
  wire=json.loads(plot.to_json());assert wire['version']==25
  if case['hidden']:hide(wire)
  restored=c.Plot.from_json(json.dumps(wire));owned.append(restored)
  chart=restored.chart();owned.append(chart);layer=chart.semantics()['layers'][0];styles=layer['styles'];assert len(styles)==4
  for style,value in zip(styles,case['values']):
   if channel=='alpha':assert style['color']['alpha']==math.floor(value*255+0.5),(index,style,value)
   elif channel in ('size','linewidth'):assert abs(style['radius' if channel=='size' else 'stroke_width']-value)<2e-12,(index,style,value)
   else:
    paint=style['fill' if channel=='fill' else 'color'];assert [paint[k] for k in ('red','green','blue')]==[int(value[i:i+2],16) for i in (1,3,5)],(index,paint,value)
  records.append({'index':index,'styles':styles})
 finally:
  for value in reversed(owned):value.dispose()
assert len(records)==180
(out/'precision-records.json').write_text(json.dumps(records,indent=2));print('PASS Python: 180 temporal precision cases.')
