"""FIX-GG03: physical linewidths and hairlines through actual primary SVG export."""
import json,sys
from pathlib import Path
from xml.etree import ElementTree as E
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current');records=[]
for index,case in enumerate(json.loads((ROOT/'fixtures/parity/ggplot2/line-widths.json').read_text())['cases']):
 if case['kind']=='polygon':continue
 for mapped in (False,True):
  owned=[]
  try:
   kind=case['kind'];width=case['linewidth'];data=c.Data.columns({'x':[1.,2.,3.],'y':[1.,2.,1.],'xmax':[1.2,2.2,3.2],'ymax':[1.5,2.5,1.5],'width':[width]*3});owned.append(data)
   def layer():
    layer=(c.rule() if kind=='segment' else c.rectangle() if kind=='rect' else c.line().order('Authored' if kind=='path' else 'X')).name('marks').stroke('#ff0000')
    return layer.shape_value('StrokeWidth','width') if mapped else layer.linewidth(width)
   p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').x2('xmax').y2('ymax')).layer(layer()).build();owned.append(p)
   wire=p.to_json();restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',layer()).build()
    if current is not restored:owned.append(current)
    request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame);svg=frame.export('svg')
    widths=[float(e.attrib['stroke-width']) for e in E.fromstring(svg).iter() if e.attrib.get('stroke')=='#ff0000']
    expected=case['pdf_widths'][0] if width==0 else case['lwd'][0]*72/96
    assert widths and all(abs(v-expected)<2e-12 for v in widths),(index,mapped,widths,expected)
    records.append({'index':index,'mapped':mapped,'state':state,'widths':widths})
    if width==.5 and mapped and state=='original':
     (out/f'width-{kind}.svg').write_bytes(svg)
     for fmt in ('pdf','png'):(out/f'width-{kind}.{fmt}').write_bytes(frame.export(fmt))
  finally:
   for value in reversed(owned):value.dispose()
assert len(records)==96
(out/'line-width-records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose();print('PASS Python: 96 physical linewidth states and twelve publication files.')
