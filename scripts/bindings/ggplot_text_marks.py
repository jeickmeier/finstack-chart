"""FIX-GG08: actual host text/raster authoring, immutable replay and publications."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
def color(r,g,b,a=255):return {'red':r,'green':g,'blue':b,'alpha':a}
for mode in range(7):
    data=c.Data.columns({'x':[1.,2.,3.,4.],'label':['Alpha','béta','','Delta']})
    b=c.plot(data).aes(c.aes().x('x').y(1.))
    options={'size':14.,'units':'Points','angle':30. if mode==1 else 0.,'fill':color(230,240,255) if mode==1 else None,'check_overlap':mode==2}
    if mode<3:
        layer=c.points().text_geom(options).text_label('label')
        b=b.layer(layer.aes(c.aes().x(1.).y(1.)) if mode==2 else layer)
    elif mode==3:
        data=c.Data.columns({'group':['A','A','B']})
        b=c.plot(data).layer(c.points().stat(c.count().group('group')).after_stat(c.stat_aes().x('Group').y('Count')).text_geom(options).text_stat_label('Count'))
    else:
        data=c.Data.columns({'x':[1.]})
        content={'Vector':{'geometry':{'commands':[{'MoveTo':[-80.,-60.]},{'LineTo':[80.,-60.]},{'LineTo':[0.,60.]},'Close']},'fill':color(20,90,120),'stroke':None}} if mode==6 else {'Raster':{'raster':{'width':2,'height':2,'pixels':[color(255,0,0),color(0,255,0),color(0,0,255),color(255,255,0,100)]},'bounds':[-100.,-60.,200.,120.],'interpolate':mode==5}}
        b=c.plot(data).aes(c.aes().x('x').y(1.)).layer(c.points().annotation({'units':'Points','content':content}))
    x=c.x_axis().scale(c.scale_linear().domain(0.,5.)) if mode<3 else c.x_axis()
    p=b.x_axis(x.visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,3.)).visible(False)).build()
    wire=p.to_json();assert json.loads(wire)['version']==73
    restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    labels=[i['primitive']['GlyphRun']['run']['text'] for i in scene['items'] if i.get('layer') and 'GlyphRun' in i['primitive']]
    if mode<4: assert labels==[['Alpha','béta','Delta'],['Alpha','béta','Delta'],['Alpha'],['2','1']][mode],labels
    if mode in (4,5):
        rasters=[i['primitive']['RasterImage'] for i in scene['items'] if 'RasterImage' in i['primitive']]
        assert len(rasters)==1 and rasters[0]['interpolate']==(mode==5)
    (out/f'text-{mode}.plot.json').write_text(wire)
    (out/f'text-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'text-{mode}.{fmt}').write_bytes(frame.export(fmt))
    if mode<4:
        outlined_request=output.request(restored,c.export_options(600.,360.).dpi(144).text("outline"));outlined=outlined_request.prepare()
        for fmt in ["svg","pdf"]:(out/f"text-{mode}.outline.{fmt}").write_bytes(outlined.export(fmt))
        outlined.dispose();outlined_request.dispose()
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose()
print('PASS Python text/raster: seven authors, 21 standard and 8 outline publications, source/stat labels and replay.')
