"""GG07 actual host line ends/joins, alpha, explicit units and replay."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(6):
    data=c.Data.columns({'x':[1.,2.,2.1,3.],'y':[1.,3.,1.,2.]})
    layer=c.line().linewidth(8.).aesthetic_units('Points').alpha(.5).lineend(['Butt','Round','Square'][mode] if mode<3 else 'Butt').linejoin('Miter' if mode<3 else ['Miter','Round','Bevel'][mode-3]).line_type('Dashed' if mode<3 else 'Solid')
    p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).build()
    wire=p.to_json();assert json.loads(wire)['version']==74
    q=c.Plot.from_json(wire);scenes=[]
    for current in [p,q]:
        request=output.request(current,c.export_options(600.,360.).dpi(144));frame=request.prepare();scenes.append(frame.scene())
        if current is q:
            for fmt in ['svg','pdf','png']:(out/f'stroke-{mode}.{fmt}').write_bytes(frame.export(fmt))
        frame.dispose();request.dispose()
    assert scenes[0]==scenes[1]
    (out/f'stroke-{mode}.scene.json').write_text(json.dumps(scenes[0]));(out/f'stroke-{mode}.plot.json').write_text(wire)
    q.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python stroke controls: six authors,18 publications,original/replay scenes.')
