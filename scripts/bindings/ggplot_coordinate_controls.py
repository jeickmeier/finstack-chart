"""FIX-GG13 independent Python coordinate authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(14):
    if mode<6:
        data=c.Data.columns({'x':[0.,1.,2.,3.,4.],'y':[1.,2.,3.,2.,1.]})
        radial={'expand':False}
        if mode==0:radial['mode']='Polar'
        elif mode==1:radial.update(start=math.pi/4,end=1.5*math.pi,inner_radius=0.3)
        elif mode==2:radial.update(reverse='Both',radial_axis='Inside',inner_radius=0.3)
        elif mode==4:radial['inner_radius']=0.4
        coordinate={'Cartesian':{'ratio':1.}} if mode==5 else {'Radial':radial}
        x=c.x_axis().scale(c.scale_linear().domain(0.,4.)).ticks([({'Number':float(i)},str(i)) for i in range(5)])
        if mode==3:x=x.guide_components({'labels':{'rotation':0.}})
        builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points()).x_axis(x).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.))).coordinate(coordinate)
        if mode==4:builder=builder.guide(c.axis_guide('theta-inner','x').side('Top')).guide(c.axis_guide('r-secondary','y').side('Right'))
    else:
        values={'x':[1.,3.,1.,2.,3.],'y':[1.,1.,2.,2.,2.]} if mode==10 else {'x':['A','B','C','D','E'],'y':[1.,2.,3.,2.,1.]} if mode==13 else {'x':[0.,1.,2.,3.,4.],'y':[1.,2.,3.,2.,1.],'lo':[0.5,1.,2.,1.,0.5],'hi':[1.5,3.,4.,3.,1.5]}
        data=c.Data.columns({**values,"x":c.categorical(values["x"])}) if mode==13 else c.Data.columns(values)
        builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y'))
        if mode==6:
            builder=builder.layer(c.errorbar().recipe_value('Lower','lo').recipe_value('Upper','hi'));coordinate={'Cartesian':{'flip':True}}
        elif mode==7:
            builder=builder.layer(c.line()).layer(c.segment().aes(c.aes().x('x').y('y').x2(3.8).y2(3.5)).recipe({'Segment':{'arrow':{'closed':True,'length_mm':3.,'angle':30.,'ends':'Last'}}}));coordinate={'Transformed':{'y':'Sqrt'}}
        elif mode==8:
            builder=builder.layer(c.rectangle().recipe({'Column':{}}));coordinate={'Radial':{'mode':'Polar','expand':False}}
        elif mode==9:
            builder=builder.layer(c.line()).layer(c.points().radius(5.));coordinate={'Radial':{'inner_radius':0.45,'clip':'On','expand':False}}
        elif mode==10:
            builder=builder.layer(c.points().recipe({'Raster':{'interpolate':True,'hjust':0.5,'vjust':0.5}}));coordinate={'Radial':{'inner_radius':0.5,'clip':'On','expand':False}}
        elif mode==11:
            builder=builder.layer(c.points()).theme(c.theme().style(c.style().gradient({'direction':'Horizontal','start':c.rgb(255,220,100),'end':c.rgb(40,90,180)})));coordinate={'Radial':{'start':math.pi/4,'end':1.5*math.pi,'inner_radius':0.35,'clip':'On','expand':False}}
        elif mode==12:
            builder=builder.layer(c.rectangle().recipe({'Column':{}}).stat(c.count().ggplot_count().x('x')));coordinate={'Cartesian':{'xlim':[{'Number':1.},{'Number':3.}],'expand':[False]*4}}
        else:
            builder=builder.layer(c.points());coordinate={'Cartesian':{'flip':True,'reverse':'Y'}}
        builder=builder.coordinate(coordinate)
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'coordinate-{mode}.plot.json').write_text(wire);(out/f'coordinate-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'coordinate-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python coordinate controls: 14 authors, 42 publications.')
