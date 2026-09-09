"""Independent Python arc and pie chart author; no local geometry/layout arithmetic."""
from pathlib import Path
import json,sys,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
base=c.Data.columns({'x':[0.,8.],'y':[0.,8.]})
draft=c.plot(base).aes(c.aes().x('x').y('y')).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,8.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,8.)).visible(False)).title(c.title('Arcs and pies / corners, padding and source identity'))
cases=[('Quarter',0.,48.,0.,math.pi/2,0.,0.,None),('Annulus',27.,48.,0.,math.tau,0.,0.,None),('Rounded',20.,48.,.1,2.3,9.,0.,None),('Padded',20.,48.,0.,1.2,4.,.18,60.),('Reverse',20.,48.,2.8,-1.2,6.,.1,None),('Swapped radii',48.,20.,0.,4.9,5.,0.,None),('Tiny wedge',20.,48.,0.,.08,20.,0.,None),('Signed inner',-10.,48.,0.,1.6,0.,0.,None)]
for i,(name,inner,outer,start,end,corner,pad,pad_radius) in enumerate(cases):
    x=i%4*2.+1.;y=6.5-i//4*2.3;data=c.Data.columns({'x':[x],'y':[y]},name=f'arc-{i}')
    parameters={'datum':{'inner_radius':inner,'outer_radius':outer,'start_angle':start,'end_angle':end,'pad_angle':pad},'corner_radius':corner,'pad_radius':pad_radius}
    draft=draft.layer(c.shape_arc().name(f'arc-{i}').data(data).arc_parameters(parameters).color('#2162a8')).layer(c.labels().id(f'arc-label-{i}').at(x-.8,y+1.1).text(name).style(c.text_style().size(.95)))
for i,(name,inner,pad,corner,start,end,order) in enumerate([('Pie / source order',0.,0.,0.,0.,math.tau,'Input'),('Donut / descending',32.,.06,7.,.4,.4+math.tau,'ValuesDescending')]):
    x=2.+i*4.;data=c.Data.columns({'x':[x]*4,'y':[1.45]*4,'weight':[1.,2.,3.,2.],'slice':['A','B','C','D']},name=f'pie-{i}',keys=[9007199254741001,9007199254741003,9007199254741005,9007199254741007])
    layer=c.shape_pie().name(f'pie-{i}').data(data).aes(c.aes().color('slice').color_scale('slices')).pie_order(order).pie_angles({'start_angle':start,'end_angle':end,'pad_angle':pad}).shape_value('PieValue','weight').shape_value('InnerRadius',inner).shape_value('OuterRadius',65.).shape_value('CornerRadius',corner)
    draft=draft.layer(layer).layer(c.labels().id(f'pie-label-{i}').at(x-1.,2.75).text(name).style(c.text_style().size(.95)))
plot=draft.scale(c.color_discrete('slices').palette(['#2162a8','#5b9cce','#e4ad4b','#b55037'])).legend(c.legend().scale('slices').title('Source slice')).build();(out/'figure.plot.json').write_text(plot.to_json());output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
requests=[(dpi,output.request(plot,c.export_options(680.,640.).dpi(dpi))) for dpi in [300,600]];plot.dispose()
for dpi,request in requests:
    frame=request.prepare();scene=frame.scene();assert sum('ShapePath' in item['primitive'] for item in scene['items'])==16
    for i,item in enumerate(scene['items']):
        if 'ShapePath' in item['primitive']:assert len(item['primitive']['ShapePath']['anchors'])==len(scene['targets'][i])==1
    (out/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
    for fmt in ['svg','pdf','png']:(out/f'figure-{dpi}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose()
print('PASS Python arc/pie gallery: eight arcs and two four-slice compositions at 300/600 DPI; immutable exports after plot disposal.')
