"""Actual Python presented slice identity, holes, clipping and named field/expression channels."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(400.,200.).dpi(72).layout(c.layout_options().padding(0));keys=[9007199254741001,9007199254741003,9007199254741005]
for clipped in [False,True]:
    data=c.Data.columns({'weight':[1.,2.,1.],'radius':[40.,40.,40.]},keys=keys)
    layer=c.shape_pie().pie_order('Input').shape_value('PieValue',data.field('weight')).shape_value('InnerRadius',20.).shape_value('OuterRadius',c.source_expr(data.field('radius'))*1.)
    p=c.plot(data).aes(c.aes().x(-.15 if clipped else 2.).y(100.)).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_log(10.).domain(1.,10000.)).visible(False)).build()
    wire=p.to_json();assert json.loads(wire)['version']==7;c.Plot.from_json(wire).dispose();chart=p.chart();p.dispose();frame=chart.present(output,options);scene=frame.scene();shapes=[i['primitive']['ShapePath'] for i in scene['items'] if 'ShapePath'in i['primitive']];assert len(shapes)==3 and all(len(s['anchors'])==1 for s in shapes)
    if not clipped:
        assert chart.inspect(200.,100.,mode='Containment')['targets']==[]
        for key,(x,y) in zip(keys,[(222.,78.),(222.,122.),(178.,78.)]):
            hit=chart.inspect(x,y,mode='Containment')['targets'];assert len(hit)==1 and int(hit[0]['identity']['Source']['key'])==key
            chart.focus(hit[0]);chart.present(output,options).dispose()
    else:
        hit=chart.inspect(10.,80.,mode='Containment')['targets'];assert len(hit)==1 and int(hit[0]['identity']['Source']['key'])==keys[0];chart.focus(hit[0]);chart.present(output,options).dispose()
    assert chart.inspect(-1.,100.,mode='Containment')['targets']==[]
    chart.dispose();frame.dispose()
data=c.Data.columns({'category':['a','b','b','c']})
layer=c.shape_pie().stat(c.count().group('category')).after_stat(c.stat_aes().x(2.).y(2.)).pie_grouped(False).pie_order('Input').shape_value('PieValue',{'Statistical':'Count'})
p=c.plot(data).layer(layer).build();f=output.request(p,options).prepare();targets=[t for group in f.scene()['targets'] for t in group if 'Aggregate' in t];assert [len(t['Aggregate']['members']) for t in targets]==[1,2,1];f.dispose();p.dispose()
try:c.shape_pie().shape_value('PieValue',{'Statistical':'Count','unexpected':1});assert False
except c.ChartError:pass
print('PASS Python arc/pie interaction: exact slice identity, donut holes, clipped fill/focus, post-log center projection, native field/expression channels and v7 round trips.')
