"""FIX-S09 symbol type/size corrections, facets and immutable export ownership."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True);records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480.,260.).dpi(72).basis('current');base=9007199254741000
kinds=['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','Asterisk','Diamond2','Square2','Triangle2']
def data(rows):return c.Data.columns({'x':[r[1] for r in rows],'y':[r[2] for r in rows],'panel':c.categorical([r[3] for r in rows]),'kind':c.categorical([r[4] for r in rows]),'area':[r[5] for r in rows]},keys=[r[0] for r in rows],name='live')
def author(rows,facets):
    layer=c.shape_symbol().symbol_types('kind',kinds,kinds).shape_value('AreaSize','area').symbol_size_guide('Area',[16.,64.,256.])
    p=c.plot(data(rows)).aes(c.aes().x('x').y('y')).layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,10.))).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,10.)))
    if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
    return p.build()
for offset,kind in enumerate(kinds):
 for facets in [False,True]:
    rows=[(base+i+1,float(i),float(i%3+2),'A' if i<4 else 'B',kinds[(offset+i)%13],float(16*(i%4+1)**2)) for i in range(8)]
    p=author(rows,facets);chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));frame=old.prepare();png=frame.export('png');frame.dispose()
    for step in range(4):
        tx=chart.transaction().id(f'symbol-{step}')
        if step==0:
            row=(base+9,8.,7.,'B',kinds[(offset+5)%13],576.);rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
        elif step==1:
            row=(base+2,1.,6.,'A',kinds[(offset+9)%13],36.);rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
        elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
        else:rows=rows[-5:];tx=tx.retain_count('live',5)
        tx=tx.build();assert 'Applied' in chart.commit(tx);tx.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose()
        current=chart.request(output,options);fresh_plot=author(rows,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose();a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(kind,facets,step)
        for targets in a.scene()['targets']:
            for target in targets:
                if 'Source'in target:assert int(target['Source']['key']) in [r[0] for r in rows]
        a.dispose();b.dispose();current.dispose();fresh.dispose()
    chart.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose();old.dispose();records.append(dict(kind=kind,facets=facets,updates=4,type_and_area_corrections=True,batch_png_equal=True,exact_keys=True,old_snapshot_retained=True))
out.write_text(json.dumps(records,indent=2)+'\n');print('PASS Python symbols: 104 append/upsert/remove/retention comparisons across 13 types and facets, exact large keys, type/area corrections and retained exports.')
