"""Independently authored all-curve chart publication using the actual Python primary API."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
names=[v['name'][5:] for v in json.loads((ROOT/'fixtures/shapes/inventory.json').read_text())['exports'] if v['kind']=='curve']
base=c.Data.columns({'x':[0.,32.],'y':[0.,20.]})
draft=c.plot(base).aes(c.aes().x('x').y('y')).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,32.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,20.)).visible(False)).title(c.title('Cartesian curves / projected lines and general areas'))
xy=[[0.,0.],[1.,2.],[2.,-1.],[3.,3.],[4.,1.],[6.,2.]]
for i,name in enumerate(names):
    curve={'kind':name};ox=i%4*8.+.8;oy=16.-i//4*4.
    data=c.Data.columns({'x':[ox+x for x,y in xy],'y':[oy+2.+y*.3 for x,y in xy]},name='line-'+name)
    draft=draft.layer(c.shape_line().name('line-'+name).data(data).curve(curve).color('#2162a8').size(1.5)).layer(c.points().name('sources-'+name).data(data).color('#d85b24').size(1.5))
    if name!='Bundle':
        data=c.Data.columns({'x':[ox+x for x,y in xy],'y':[oy+.25+y*.15 for x,y in xy],'x2':[ox+x+y*.2 for x,y in xy],'y2':[oy+.9+y*.2 for x,y in xy]},name='area-'+name)
        draft=draft.layer(c.shape_area().name('area-'+name).data(data).aes(c.aes().x2('x2').y2('y2')).curve(curve).color('rgba(33,98,168,0.55)'))
    draft=draft.layer(c.labels().id('label-'+name).at(ox,oy+3.6).text(name).style(c.text_style().size(.85)))
plot=draft.build();(out/'figure.plot.json').write_text(plot.to_json());output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
requests=[(dpi,output.request(plot,c.export_options(680.,640.).dpi(dpi))) for dpi in [300,600]];plot.dispose()
for dpi,request in requests:
    frame=request.prepare();scene=frame.scene();assert sum('ShapePath' in i['primitive'] for i in scene['items'])==39
    for i,item in enumerate(scene['items']):
        if 'ShapePath' in item['primitive']:assert len(item['primitive']['ShapePath']['anchors'])==len(scene['targets'][i])
    (out/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
    for fmt in ['svg','pdf','png']:(out/f'figure-{dpi}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose()
print('PASS Python Cartesian publication: 20 curves, 19 areas and retained source anchors at 300/600 DPI.')
