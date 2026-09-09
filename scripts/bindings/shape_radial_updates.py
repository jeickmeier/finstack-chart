"""FIX-S09: all radial line/area curves and link routes under updates and retained output."""
from pathlib import Path
import json,sys,itertools
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480.,260.).dpi(72).basis('current');base=9007199254741000
corpus=json.loads((ROOT/'fixtures/shapes/radial.json').read_text());curves=list({json.dumps(t['config'].get('curve',{'kind':'Linear'}),sort_keys=True):t['config'].get('curve',{'kind':'Linear'})for t in corpus['cases']if t['family']=='lineRadial'}.values())
cases=[(family,curve)for family in ['line','area']for curve in curves if family!='area'or curve['kind']!='Bundle']+[(family,{'kind':'Linear'})for family in ['horizontal','vertical','step','radial']]
def data(rows):return c.Data.columns({'angle':c.column([r[1]for r in rows],kind='float64').nullable(True),'radius':[r[2]for r in rows],'panel':c.categorical([r[3]for r in rows])},keys=[r[0]for r in rows],name='live')
def author(rows,family,curve,facets):
    d=data(rows)
    if family in ['line','area']:
        layer=(c.shape_line_radial()if family=='line'else c.shape_area_radial()).curve(curve).shape_value('Angle',d.field('angle'))
        layer=layer.shape_value('Radius',d.field('radius'))if family=='line'else layer.shape_value('InnerRadius',4.).shape_value('OuterRadius',d.field('radius'))
    elif family=='radial':layer=c.shape_link_radial().shape_value('StartAngle',d.field('angle')).shape_value('EndAngle',1.).shape_value('InnerRadius',4.).shape_value('OuterRadius',d.field('radius'))
    else:layer=({'horizontal':c.shape_link_horizontal,'vertical':c.shape_link_vertical,'step':lambda:c.shape_link({'kind':'Step'})}[family])().aes(c.aes().x('angle').y('radius').x2(3.).y2(40.))
    p=c.plot(d).aes(c.aes().x(2.).y(100.)).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.))).y_axis(c.y_axis().scale(c.scale_log(10.).domain(1.,10000.)))
    if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
    return p.build()
for (family,curve),facets in itertools.product(cases,[False,True]):
    rows=[(base+i+1,None if i==3 else i*.4,10.+i,'A'if i<4 else'B')for i in range(8)]
    p=author(rows,family,curve,facets);chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));f=old.prepare();png=f.export('png');f.dispose()
    for step in range(4):
        tx=chart.transaction().id(f'radial-{step}')
        if step==0:row=(base+9,3.2,22.,'B');rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
        elif step==1:row=(base+2,.4,12. if family in ['horizontal','vertical','step']else-12.,'A');rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
        elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
        else:rows=rows[-5:];tx=tx.retain_count('live',5)
        tx=tx.build();result=chart.commit(tx);assert 'Applied'in result,(family,curve,facets,step,result);tx.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose()
        current=chart.request(output,options);fresh_plot=author(rows,family,curve,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose();a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(family,curve,facets,step)
        for targets in a.scene()['targets']:
            for t in targets:
                if'Source'in t:assert int(t['Source']['key'])in[r[0]for r in rows]
        a.dispose();b.dispose();current.dispose();fresh.dispose()
    chart.dispose();f=old.prepare();assert f.export('png')==png;f.dispose();old.dispose();records.append(dict(family=family,curve=curve,facets=facets,updates=4,batch_png_equal=True,exact_keys=True,old_snapshot_retained=True))
out.write_text(json.dumps(records,indent=2)+'\n');print(f'PASS Python radial/link updates: {len(records)*4} append/upsert/remove/retention comparisons, missing angles, signed-radius corrections, log centers, facets, exact keys and retained exports.')
