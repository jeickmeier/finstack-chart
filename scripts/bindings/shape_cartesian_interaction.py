"""Actual Python inspection of the presented curve/area; no controls become observations."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(400.,200.).dpi(72).layout(c.layout_options().padding(0))
keys=[9007199254741001,9007199254741002]
for area in [False,True]:
 data=c.Data.columns({'x':[0.,4.],'y':[-10.,-10.] if area else [1.,10000.],'x2':[0.,4.],'y2':[10.,10.]},keys=keys)
 p=c.plot(data).aes(c.aes().x('x').y('y').x2('x2').y2('y2')).layer(c.shape_area() if area else c.shape_line().curve({'kind':'BumpX'}).size(2.))
 p=p.x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.) if area else c.scale_log(10.).domain(1.,10000.)).visible(False)).build()
 wire=p.to_json();assert json.loads(wire)['version']==7;roundtrip=c.Plot.from_json(wire);roundtrip.dispose();chart=p.chart();p.dispose();frame=chart.present(output,options);scene=frame.scene()
 shape=next(i['primitive']['ShapePath'] for i in scene['items'] if 'ShapePath'in i['primitive'])
 assert len(shape['anchors'])==2
 hit=chart.inspect(200.,100.,mode='Containment')['targets'];assert len(hit)==1 and int(hit[0]['identity']['Source']['key']) in keys
 assert chart.inspect(200.,-1.,mode='Containment')['targets']==[]
 if not area:
  assert shape['geometry']['commands']==[{'MoveTo':[0.,200.]},{'CubicTo':[200.,200.,200.,0.,400.,0.]}]
  assert chart.inspect(100.,100.,mode='Containment')['targets']==[]
  assert chart.select_region({'Rectangle':[190.,190.,20.,10.]})['targets']==[]
 else:assert chart.select_region({'Rectangle':[0.,0.,5.,5.]})['targets']==[]
 chart.focus(hit[0]);chart.present(output,options).dispose();chart.dispose();frame.dispose()
print('PASS Python presented shape containment, non-source controls, clipped interior focus, exact keys and v7 round trips.')
