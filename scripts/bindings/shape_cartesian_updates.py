"""FIX-S09 Cartesian updates, exact row identities, facets and retained scenes."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480.,260.).dpi(72).basis('current')
base=9007199254741000
records=[]
def data(rows):return c.Data.columns({'x':[r[1] for r in rows],'y':[r[2] for r in rows],'x2':[r[1]+.5 for r in rows],'y2':[r[2]+2. for r in rows],'panel':c.categorical([r[3] for r in rows])},keys=[r[0] for r in rows],name='live')
def author(rows,kind,area,facets):
    p=c.plot(data(rows)).aes(c.aes().x('x').y('y').x2('x2').y2('y2')).layer((c.shape_area() if area else c.shape_line()).curve({'kind':kind}).size(2.))
    p=p.x_axis(c.x_axis().scale(c.scale_linear().domain(0.,10.))).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,10.)))
    if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
    return p.build()
for kind in ['Natural','CardinalClosed','MonotoneX','Step']:
 for area in [False,True]:
  for facets in [False,True]:
   rows=[(base+i+1,float(i),float(i%3+1),'A' if i<4 else 'B') for i in range(8)]
   p=author(rows,kind,area,facets);chart=p.chart();chart.present(output,options).dispose();p.dispose()
   old=chart.request(output,options.basis('presented'));frame=old.prepare();png=frame.export('png');frame.dispose()
   for step in range(4):
    tx=chart.transaction().id(f'shape-{step}')
    if step==0:
     row=(base+9,8.,7.,'B');rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
    elif step==1:
     row=(base+2,1.,6.,'A');rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
    elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
    else:rows=rows[-5:];tx=tx.retain_count('live',5)
    tx=tx.build();assert 'Applied' in chart.commit(tx);tx.dispose()
    old_frame=old.prepare();assert old_frame.export('png')==png;old_frame.dispose()
    current=chart.request(output,options);fresh_plot=author(rows,kind,area,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose()
    a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(kind,area,facets,step)
    for item in a.scene()['targets']:
     for target in item:
      if 'Source' in target:assert int(target['Source']['key']) in [r[0] for r in rows]
    a.dispose();b.dispose();current.dispose();fresh.dispose()
   chart.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose();old.dispose();records.append(dict(kind=kind,area=area,facets=facets,updates=4,exact_keys=True,batch_png_equal=True,old_snapshot_retained=True))
out.write_text(json.dumps(records,indent=2)+'\n');print('PASS Python Cartesian: 64 append/upsert/remove/retention checks across global/local curves, areas, facets and exact large keys; old exports retained.')
