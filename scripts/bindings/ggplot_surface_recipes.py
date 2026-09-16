"""FIX-GG07: independent actual Python surface recipes and portable replay."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(9):
    if mode<4:
        data=c.Data.columns({'x':[0.,4.,4.,0.,1.,3.,3.,1.],'y':[0.,0.,4.,4.]+([3.,3.,1.,1.] if mode%2 else [1.,1.,3.,3.]),'sub':['outer']*4+['inner']*4})
        b=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points().recipe({'Polygon':{'rule':'EvenOdd' if mode<2 else 'NonZero'}}).recipe_value('Subgroup','sub'))
    elif mode==8:
        data=c.Data.columns({'g':['A','A','B']})
        b=c.plot(data).layer(c.points().stat(c.count().group('g')).after_stat(c.stat_aes().x('Group').y('Count')).recipe({'Tile':{'width':0.8,'height':0.8}}))
    else:
        data=c.Data.columns({'x':[1.,3.,1.,2.,3.],'y':[1.,1.,2.,2.,2.],'v':[1.,3.,4.,5.,6.]})
        recipe={'Tile':{'width':0.8 if mode==5 else None,'height':0.6 if mode==5 else None}} if mode<6 else {'Raster':{'hjust':0.5,'vjust':0.5,'interpolate':mode==7}}
        b=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points().recipe(recipe))
    p=b.x_axis(c.x_axis().visible(False)).y_axis(c.y_axis().visible(False)).build()
    wire=p.to_json();assert json.loads(wire)['version']==74
    restored=c.Plot.from_json(wire);request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare()
    assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    if mode in [6,7]:
        raster=next(i['primitive']['RasterImage'] for i in scene['items'] if 'RasterImage' in i['primitive'])
        assert len(raster['cells'])==5 and (raster['raster']['width'],raster['raster']['height'])==(3,2)
    (out/f'surface-{mode}.plot.json').write_text(wire);(out/f'surface-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'surface-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
# GG08 inherited boundary: field names, owner-scoped fields and source expressions.
data=c.Data.columns({'x':[1.,2.],'label':[3.,4.]})
for source,expected in [('label',['3','4']),(data.field('label'),['3','4']),(c.source_expr('label')+1.,['4','5'])]:
    p=c.plot(data).aes(c.aes().x('x').y(1.)).layer(c.points().text_geom({'size':12.}).text_label(source)).build()
    request=output.request(p,c.export_options(600.,360.));frame=request.prepare()
    labels=[i['primitive']['GlyphRun']['run']['text'] for i in frame.scene()['items'] if i.get('layer') and 'GlyphRun' in i['primitive']]
    assert labels==expected,labels
    frame.dispose();request.dispose();p.dispose()
data.dispose()
output.dispose()
print('PASS Python surfaces: nine independent authors, 27 publications.')
