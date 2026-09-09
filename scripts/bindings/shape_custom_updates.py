"""FIX-S08/09: registered chart families, sparse facets, batch equivalence and retention."""
from pathlib import Path
import json,sys,itertools
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480.,260.).dpi(72).basis('current');base=9007199254741000
families=['line','area','radial_line','radial_area','link','symbol','pie','stack_bars','stack_area']
def op(name,parameters):return {'operation':{'id':'example.'+name,'version':'1'},'parameters':parameters}
def data(rows):return c.Data.columns({'x':[r[1]for r in rows],'y':[r[2]for r in rows],'angle':[r[1]*.5 for r in rows],'radius':[10.+r[2]*4. for r in rows],'area':[64.+r[2]*32. for r in rows],'panel':c.categorical([r[3]for r in rows]),'group':c.categorical([r[4]for r in rows])},keys=[r[0]for r in rows],name='live')
def author(rows,family,facets):
    d=data(rows);registry=c.ShapeRegistry.example()
    if family in ['line','area','link']:
        layer=({'line':c.shape_line,'area':c.shape_area,'link':c.shape_link_horizontal}[family])().aes(c.aes().x('x').y('y').x2(5.).y2(0.))
        if family=='area':layer=layer.aes(c.aes().x('x').y('y').x2('x').y2(0.))
    elif family.startswith('radial_'):
        layer=(c.shape_line_radial()if family=='radial_line'else c.shape_area_radial()).aes(c.aes().x(2.).y(2.)).shape_value('Angle','angle')
        layer=layer.shape_value('Radius','radius')if family=='radial_line'else layer.shape_value('InnerRadius',8.).shape_value('OuterRadius','radius')
    elif family=='symbol':layer=c.shape_symbol().shape_value('AreaSize','area').symbol_size_guide('Area',[64.,128.,192.]).shape_protocol('Symbol',op('rectangle_symbol',{'amount':4}))
    elif family=='pie':layer=c.shape_pie().pie_grouped(False).aes(c.aes().x(2.).y(2.)).shape_value('PieValue','y').shape_protocol('PieComparator',op('field_comparator',{'field':'key'}))
    else:
        layer=(c.bars()if family=='stack_bars'else c.shape_area()).position(c.shape_stack(['a','b'])).shape_protocol('StackOrder',op('first_value_order',{})).shape_protocol('StackOffset',op('shift_offset',{'amount':.25}))
    if family in ['line','area','radial_line','radial_area','link']:layer=layer.shape_protocol('Curve',op('shift_curve',{'amount':5}))
    p=c.plot(d).with_shape_registry(registry).aes(c.aes().x('x').x2('x').y('y').y2(0.).group('group').color('group')).layer(layer.name('custom')).scale(c.color_discrete('group').domain(['a','b'])).x_axis(c.x_axis().scale(c.scale_linear().domain(-1.,6.))).y_axis(c.y_axis().scale(c.scale_linear().domain(-1.,5.)))
    if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
    p=p.build();registry.dispose();return p
for family,facets in itertools.product(families,[False,True]):
    rows=[(base+i+1,float(i//2),1.+(i%3)*.5,'A'if i<4 else'B','a'if i%2==0 else'b')for i in range(8)]
    p=author(rows,family,facets);wire=p.to_json();assert json.loads(wire)['version']==9
    registry=c.ShapeRegistry.example();restored=c.Plot.from_json(wire,registry);registry.dispose();assert restored.to_json()==wire;restored.dispose()
    chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));f=old.prepare();png=f.export('png');f.dispose()
    for step in range(4):
        tx=chart.transaction().id(f'custom-{step}')
        if step==0:row=(base+9,4.,2.,'B','a');rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
        elif step==1:row=(base+2,0.,2.5,'A','b');rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
        elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
        else:rows=rows[-5:];tx=tx.retain_count('live',5)
        tx=tx.build();assert 'Applied'in chart.commit(tx);tx.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose()
        current=chart.request(output,options);fresh_plot=author(rows,family,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose();a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(family,facets,step)
        for targets in a.scene()['targets']:
            for t in targets:
                if'Source'in t:assert int(t['Source']['key'])in[r[0]for r in rows]
        a.dispose();b.dispose();current.dispose();fresh.dispose()
    chart.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose();old.dispose();records.append(dict(family=family,facets=facets,updates=4,batch_png_equal=True,exact_keys=True,old_snapshot_retained=True,wire_roundtrip=True))
out.write_text(json.dumps(records,indent=2)+'\n');print('PASS Python custom chart protocols: 72 append/upsert/remove/retention comparisons, all registered families, facets, size guides, exact keys, registry ownership and retained exports.')
