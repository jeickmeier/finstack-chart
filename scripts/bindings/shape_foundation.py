"""FIX-S01: reference shape contexts through actual checked Python path owners."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
cases=json.loads((ROOT/'fixtures/shapes/cases.json').read_text())['cases'];assert len(cases)==333
for test in cases:
    reference=None
    for digits in [0,3,12,None]:
        p=c.Path(digits).apply_batch(test['operations']);commands=[];p.replay(commands.append)
        if reference is None:reference=commands
        assert commands==reference
        if digits is not None:assert p.to_svg()==(test['svg_digits'][str(digits)] or ''),(test['id'],digits)
        copy=p.copy();p.move_to(777.,888.);p.dispose();saved=[];copy.replay(saved.append);assert saved==reference;copy.dispose()
draft=(c.plot(c.Data.columns({'x':[0.,1.],'y':[0.,1.]})).aes(c.aes().x('x').y('y')).layer(c.points().size(.1))
 .x_axis(c.x_axis().visible(False)).y_axis(c.y_axis().visible(False)).title(c.title('Shape contexts / shared path foundation')))
color=dict(red=35,green=97,blue=166,alpha=255)
for id,x,scale,filled,label in [('arc-quarter',85.,5.,True,'Circular sector'),('arc-hole',235.,5.,True,'Annular hole'),('line-curveBasis-6',370.,14.,False,'External sink / Bezier')]:
    test=next(v for v in cases if v['id']==id);p=c.Path(3.).apply_batch(test['operations']);sink=[];p.replay(sink.append)
    component=c.vector_path(id,p).transform([scale,0.,0.,scale,0.,0.],max_error=.001,max_commands=10000).anchor({'Output':{'x':x,'y':140.}}).fill(color if filled else None).stroke({'color':color,'width':1.5})
    p.move_to(777.,888.);p.dispose();draft=draft.layer(component);component.dispose()
    draft=draft.layer(c.labels().id('label-'+id).output_at(x-40.,220.).text(label).style(c.text_style().size(.85)))
plot=draft.build();(out/'figure.plot.json').write_text(plot.to_json());output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
requests=[(dpi,output.request(plot,c.export_options(540.,280.).dpi(dpi))) for dpi in [300,600]];plot.dispose()
for dpi,request in requests:
    frame=request.prepare();scene=frame.scene();assert sum('VectorPath'in i['primitive'] for i in scene['items'])==3
    assert all(scene['targets'][n]==[] for n,i in enumerate(scene['items']) if 'VectorPath'in i['primitive'])
    (out/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
    for fmt in ['svg','pdf','png']:(out/f'figure-{dpi}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose()
print('PASS FIX-S01 Python: 333 contexts, exact digits, precision-independent external replay/copy ownership and independent sector/hole/Bezier publication at 300/600 DPI.')
