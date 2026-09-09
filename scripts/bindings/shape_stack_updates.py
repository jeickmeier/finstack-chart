"""FIX-S09 all stack policies through corrections, sparse facets and retained export."""
from pathlib import Path
import json,sys,itertools
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480.,260.).dpi(72).basis('current');base=9007199254741000
orders=['None','Reverse','Ascending','Descending','Appearance','InsideOut'];offsets=['None','Expand','Diverging','Silhouette','Wiggle']
def data(rows):return c.Data.columns({'x':[r[1]for r in rows],'y':[r[2]for r in rows],'panel':c.categorical([r[3]for r in rows]),'group':c.categorical([r[4]for r in rows])},keys=[r[0]for r in rows],name='live')
def author(rows,order,offset,area,facets,missing):
    layer=(c.shape_area()if area else c.bars()).position(c.shape_stack(['a','b','c']).stack_order(order).stack_offset(offset).stack_missing(missing))
    p=c.plot(data(rows)).aes(c.aes().x('x').x2('x').y('y').y2(0.).group('group').color('group')).layer(layer).scale(c.color_discrete('group').domain(['a','b','c'])).x_axis(c.x_axis().scale(c.scale_linear().domain(-1.,7.))).y_axis(c.y_axis().scale(c.scale_linear().domain(-8.,14.)))
    if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
    return p.build()
for oi,order in enumerate(orders):
 for offset,area,facets in itertools.product(offsets,[False,True],[False,True]):
    missing='Zero'if oi%2 else'Gap';rows=[(base+i*3+g+1,float(i),float((i+g)%4+1),'A'if i<3 else'B',group)for i in range(6)for g,group in enumerate(['a','b','c'])]
    p=author(rows,order,offset,area,facets,missing);chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));f=old.prepare();png=f.export('png');f.dispose()
    for step in range(4):
        tx=chart.transaction().id(f'stack-{step}')
        if step==0:
            row=(base+19,6.,4.,'B','b');rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
        elif step==1:
            row=(base+2,0.,-6.,'A','b');rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
        elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
        else:rows=rows[-11:];tx=tx.retain_count('live',11)
        tx=tx.build();assert 'Applied'in chart.commit(tx);tx.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose()
        current=chart.request(output,options);fresh_plot=author(rows,order,offset,area,facets,missing);fresh=output.request(fresh_plot,options);fresh_plot.dispose();a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(order,offset,area,facets,step)
        for targets in a.scene()['targets']:
            for target in targets:
                if 'Source'in target:assert int(target['Source']['key'])in [r[0]for r in rows]
        a.dispose();b.dispose();current.dispose();fresh.dispose()
    chart.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose();old.dispose();records.append(dict(order=order,offset=offset,area=area,facets=facets,missing=missing,updates=4,batch_png_equal=True,exact_keys=True,old_snapshot_retained=True))
out.write_text(json.dumps(records,indent=2)+'\n');print('PASS Python stacks: 480 append/upsert/remove/retention comparisons across all 30 policies, bars/areas, sparse facets, exact keys and retained exports.')
