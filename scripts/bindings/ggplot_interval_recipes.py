"""FIX-GG07: independent actual Python interval recipes and portable replay."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(17):
    data=c.Data.columns({'x':[1.,2.,3.],'y':[2.,-1.,1.],'lo':[1.,-2.,0.],'hi':[3.,0.,2.]})
    b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'))
    if mode<8:
        layer=c.rule().recipe({'Interval':{'kind':['LineRange','PointRange','ErrorBar','Crossbar'][mode%4],'width':0.5}}).recipe_value('Lower','lo').recipe_value('Upper','hi')
        if mode>=4:b=b.aes(c.aes().x('y').y('x'));layer=layer.orientation('Horizontal')
        b=b.layer(layer)
    elif mode<11:b=b.layer(c.step(['Hv','Vh','Mid'][mode-8]))
    elif mode==11:b=b.layer(c.segment().aes(c.aes().x('x').y('y').x2(3.5).y2(3.)).recipe({'Segment':{'arrow':{'angle':30.,'length_mm':3.,'ends':'Both','closed':True}}}))
    elif mode==15:b=b.layer(c.crossbar().recipe({'Interval':{'kind':'Crossbar','width':None,'middle':{'color':{'red':255,'green':0,'blue':0,'alpha':255},'linewidth':2.,'line_type':'Dashed'},'box_style':{'color':{'red':0,'green':0,'blue':255,'alpha':255},'linewidth':0.25,'line_type':'Dotted'}}}).recipe_value('Lower','lo').recipe_value('Upper','hi').fill('#FFD700').alpha(0.2))
    elif mode==16:b=b.layer(c.pointrange().recipe({'Interval':{'kind':'PointRange','width':None,'fatten':2.,'point':{'size':1.,'stroke':2.,'shape':21,'fill':{'red':255,'green':215,'blue':0,'alpha':255}}}}).recipe_value('Lower','lo').recipe_value('Upper','hi').linewidth(0.25))
    else:b=b.layer(c.points()).layer(c.abline(0.5,0.) if mode==12 else c.hline(0.5) if mode==13 else c.vline(2.))
    p=b.x_axis(c.x_axis().visible(False)).y_axis(c.y_axis().visible(False)).build()
    wire=p.to_json();assert json.loads(wire)['version']==74
    restored=c.Plot.from_json(wire);request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    direct_request=output.request(p,c.export_options(600.,360.).dpi(144));direct_frame=direct_request.prepare()
    assert direct_frame.scene()==scene
    direct_frame.dispose();direct_request.dispose()
    (out/f'interval-{mode}.plot.json').write_text(wire);(out/f'interval-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'interval-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose()
print('PASS Python intervals: seventeen authors, 51 publications.')
