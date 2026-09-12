"""FIX-GG03: corrections and retention match fresh independent aesthetic compilation."""
import json
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(480,280).dpi(72).basis('current');base=9007199254741000
records=[]
def data(rows):
 return c.Data.columns({'x':[r[1] for r in rows],'y':[r[2] for r in rows],'fill':c.categorical([r[3] for r in rows]),'stroke':c.categorical([r[4] for r in rows]),'alpha':[r[5] for r in rows],'area':[r[6] for r in rows],'width':[r[7] for r in rows],'panel':c.categorical([r[8] for r in rows])},keys=[r[0] for r in rows],name='live')
def author(rows,facets):
 d=data(rows)
 layer=c.shape_symbol().symbol_types('fill',['A','B'],[{'Ggplot':21},{'Ggplot':24}]).shape_value('AreaSize',d.field('area')).shape_value('StrokeWidth',d.field('width')).shape_value('Alpha',d.field('alpha'))
 p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').fill('fill').fill_scale('inside').stroke('stroke').stroke_scale('outside')).layer(layer)
 p=p.scale(c.color_discrete('inside').domain(['A','B']).palette(['#ff0000','#0000ff'])).scale(c.color_discrete('outside').domain(['A','B']).palette(['#000000','#008000']))
 p=p.x_axis(c.x_axis().scale(c.scale_linear().domain(0,10))).y_axis(c.y_axis().scale(c.scale_linear().domain(0,10)))
 if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2))
 result=p.build();d.dispose();layer.dispose();return result
for facets in [False,True]:
 rows=[(base+i+1,float(i+1),float(i%3+2),'A' if i%2 else 'B','B' if i%3 else 'A',.25*(i%4+1),float(16*(i%4+1)**2),float(i%3+1),'A' if i<4 else 'B') for i in range(8)]
 p=author(rows,facets);chart=p.chart();chart.present(output,options).dispose();p.dispose();old=chart.request(output,options.basis('presented'));frame=old.prepare();png=frame.export('png');frame.dispose()
 for step in range(4):
  tx=chart.transaction().id(f'aesthetics-{facets}-{step}')
  if step==0:
   row=(base+9,9.,7.,'A','B',.7,576.,4.,'B');rows.append(row);batch=data([row]);tx=tx.append('live',batch);batch.dispose()
  elif step==1:
   row=(base+2,2.,6.,'B','A',.9,36.,.5,'A');rows[1]=row;batch=data([row]);tx=tx.upsert('live',batch);batch.dispose()
  elif step==2:rows.pop(0);tx=tx.remove('live',[base+1])
  else:rows=rows[-5:];tx=tx.retain_count('live',5)
  tx=tx.build();assert 'Applied' in chart.commit(tx);tx.dispose();saved=old.prepare();assert saved.export('png')==png;saved.dispose()
  current=chart.request(output,options);fresh_plot=author(rows,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose();a=current.prepare();b=fresh.prepare()
  assert a.export('png')==b.export('png'),(facets,step)
  scene=a.scene();marks=[i['primitive']['ShapePath'] for i in scene['items'] if i.get('layer') is not None and 'ShapePath' in i['primitive']]
  assert len(marks)==len(rows);assert all(len(m['anchors'])==1 for m in marks)
  keys=[int(t['Source']['key']) for targets in scene['targets'] for t in targets if 'Source' in t]
  assert sorted(keys)==sorted(r[0] for r in rows)
  records.append({'facets':facets,'step':step,'marks':marks})
  a.dispose();b.dispose();current.dispose();fresh.dispose()
 old.dispose();chart.dispose()
out.write_text(json.dumps(records));output.dispose()
print('PASS Python GG-03: 8 append/correction/removal/retention states with independent fill/stroke/alpha/area/width, facets, exact fresh PNG and immutable presentation.')
