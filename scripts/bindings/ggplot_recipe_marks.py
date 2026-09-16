"""FIX-GG07: actual host count/summary authoring, owned fields and portable replay."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(5):
    d=c.Data.columns({'x':[1.,1.,2.,3.],'y':[1.,1.,-1.,2.],'end_x':[3.,3.,4.,4.],'end_y':[2.,2.,2.,0.],'angle':[0.,0.,math.pi/2,math.pi],'radius':[1.,1.,-1.,2.],'w':[1.,2.,1.,4.]})
    if mode==0:layer=c.points().recipe({'Count':{}}).stat(c.count().sum_count().x('x').y('y').count_weight('w'))
    elif mode==1:layer=c.rectangle().recipe({'Column':{'width':.6,'just':.5}})
    elif mode==2:layer=c.points().recipe({'Rug':{'sides':'bltr','length':.04,'outside':False}})
    elif mode==3:layer=c.rule().aes(c.aes().x('x').y('y').x2('end_x').y2('end_y')).recipe({'Curve':{'curvature':.4,'angle':60.,'ncp':5,'arrow':{'angle':30.,'length_mm':3.,'ends':'Last','closed':True}}})
    else:layer=c.rule().recipe({'Spoke':{}}).recipe_value('Angle','angle').recipe_value('Radius','radius')
    p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,5.))).y_axis(c.y_axis().scale(c.scale_linear().domain(-2.,4.))).build();wire=p.to_json();assert json.loads(wire)['version']==74
    restored=c.Plot.from_json(wire);d.dispose()
    scenes=[]
    for candidate in [p,restored]:
        request=output.request(candidate,c.export_options(600.,360.));frame=request.prepare();scenes.append(frame.scene())
        if candidate is restored:
            for fmt in ['svg','pdf','png']:(out/f'mark-{mode}.{fmt}').write_bytes(frame.export(fmt))
        frame.dispose();request.dispose()
    assert scenes[0]==scenes[1]
    (out/f'mark-{mode}.scene.json').write_text(json.dumps(scenes[0]));(out/f'mark-{mode}.plot.json').write_text(wire)
    p.dispose();restored.dispose()
output.dispose()
print('PASS Python five recipe mark authors, original/replay scene equality and 15 publications.')
