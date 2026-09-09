"""FIX-S06/09 actual symbol hit geometry, scale guides and generated identities."""
from pathlib import Path
import sys,json,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(400.,200.).dpi(72).layout(c.layout_options().padding(0));key=9007199254741001
for kind,clipped in [('Circle',False),('Plus',False),('Circle',True),('Star',False)]:
    area=0. if kind=='Star' else math.pi*25. if kind=='Circle' else 256.
    d=c.Data.columns({'area':[area]},keys=[key]);layer=c.shape_symbol().symbol_kind(kind).shape_value('AreaSize',c.source_expr(d.field('area'))*1.)
    p=c.plot(d).aes(c.aes().x(-.02 if clipped else 2.).y(100.)).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_log(10.).domain(1.,10000.)).visible(False)).build();wire=p.to_json();assert json.loads(wire)['version']==7;c.Plot.from_json(wire).dispose();chart=p.chart();p.dispose();frame=chart.present(output,options);shapes=[i['primitive']['ShapePath'] for i in frame.scene()['items'] if 'ShapePath'in i['primitive']]
    if kind=='Star':assert not shapes and not chart.inspect(200.,100.,mode='Containment')['targets']
    else:
        assert len(shapes)==1 and len(shapes[0]['anchors'])==1
        x,y=(1.,100.) if clipped else (210.,100.) if kind=='Plus' else (204.,100.)
        hits=chart.inspect(x,y,mode='Containment')['targets'];assert len(hits)==1 and int(hits[0]['identity']['Source']['key'])==key;chart.focus(hits[0]);chart.present(output,options).dispose()
        if kind=='Plus':assert shapes[0]['fill'] is None and shapes[0]['stroke'] is not None and not chart.inspect(207.,107.,mode='Containment')['targets']
        elif not clipped:assert not chart.inspect(206.,100.,mode='Containment')['targets']
    assert not chart.inspect(-1.,100.,mode='Containment')['targets'];frame.dispose();chart.dispose()
d=c.Data.columns({'category':['a','b','b','c']});layer=c.shape_symbol().stat(c.count().group('category')).after_stat(c.stat_aes().x(2.).y(2.)).symbol_groups(['a','b','c'],['Circle','Square','Plus']).shape_value('AreaSize',{'Statistical':'Count'})
p=c.plot(d).layer(layer).build();request=output.request(p,options);f=request.prepare();targets=[t for group in f.scene()['targets'] for t in group if 'Aggregate'in t];assert [len(t['Aggregate']['members']) for t in targets]==[1,2,1];f.dispose();request.dispose();p.dispose()
d=c.Data.columns({'area':[0.,1.,2.]});scale=c.StandaloneScale('linear',domain=[0.,2.],range=[16.,256.]);layer=c.shape_symbol().symbol_kind('Square').numeric_scale('AreaSize',d.field('area'),scale).symbol_size_guide('Input',[0.,1.,2.]);p=c.plot(d).aes(c.aes().x(2.).y(2.)).layer(layer).build();request=output.request(p,options);f=request.prepare();scene=f.scene()
def width(geometry):
    points=[point for op in geometry['commands'] if isinstance(op,dict) for name,value in op.items() if name in ['MoveTo','LineTo'] for point in [value]]
    return max(p[0] for p in points)-min(p[0] for p in points)
marks=[i['primitive']['ShapePath']['geometry'] for i in scene['items'] if 'ShapePath'in i['primitive']];guides=[i['primitive']['VectorPath']['geometry'] for i in scene['items'] if 'VectorPath'in i['primitive']];assert len(marks)==len(guides)==3
for mark,guide,size in zip(marks,guides,[16.,136.,256.]):assert abs(width(mark)-math.sqrt(size))<1e-10 and abs(width(guide)-math.sqrt(size))<1e-10
f.dispose();request.dispose();p.dispose();scale.dispose()
print('PASS Python symbols: exact keys, field/expression area, log centers, stroke holes, zero size, clipping/focus, generated count identities and actual nonidentity size-guide geometry.')
