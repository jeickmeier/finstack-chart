"""FIX-S09 curve dash gaps, exact source identities and retained host output."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(100.,100.).dpi(72).layout(c.layout_options().padding(0));keys=[9007199254740993,9007199254740997]
data=c.Data.columns({'x':[0.,1.],'y':[.5,.5]},keys=keys)
p=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.shape_line().curve({'kind':'BumpX'}).style(c.style().dashes([10.,10.]))).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,1.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,1.)).visible(False)).build()
wire=p.to_json();q=c.Plot.from_json(wire);assert q.to_json()==wire;q.dispose();chart=p.chart();p.dispose();f=chart.present(output,options);scene=f.scene();i=next(i for i,item in enumerate(scene['items'])if'ShapePath'in item['primitive']);shape=scene['items'][i]['primitive']['ShapePath'];assert shape['dashes']==[10.,10.];assert 'CubicTo'in shape['geometry']['commands'][1];assert [int(t['Source']['key'])for t in scene['targets'][i]]==keys
for x in [5.,25.,45.,65.,85.]:assert len(chart.inspect(x,50.,mode='Containment')['targets'])==1
for x in [15.,35.,55.,75.,95.]:assert chart.inspect(x,50.,mode='Containment')['targets']==[]
assert chart.inspect(-1.,50.,mode='Containment')['targets']==[]
saved={fmt:f.export(fmt)for fmt in ['svg','pdf','png']};assert b'stroke-dasharray="10,10"'in saved['svg'];chart.dispose()
for fmt,expected in saved.items():assert f.export(fmt)==expected
f.dispose();data.dispose();output.dispose();print('PASS Python retained curved dashes, five ink hits/five gap misses, clipping, exact source keys, wire round trip and exports after disposal.')
