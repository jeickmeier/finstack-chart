"""CP-05 actual Python scale retraining during keyed updates and retained captures."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(480.,260.).dpi(72).basis('current')
base=9007199254740992
records=[]
def data(rows):return c.Data.columns({'key':c.column([r[1] for r in rows],kind='uint64'),'value':[r[2] for r in rows],'panel':c.categorical([r[3] for r in rows])},keys=[r[0] for r in rows],name='live')
def author(rows,classifier,facets):
 d=data(rows);scale=c.StandaloneScale(classifier,**({'interpolator':c.chromatic('Viridis')} if classifier=='sequential_quantile' else {'range':['black']}))
 p=c.plot(d).aes(c.aes().x('value').y('value').color('key' if classifier=='ordinal' else 'value').color_scale('shared')).layer(c.points())
 mapping=c.color_mapped('shared',scale,'Eligible')
 if classifier!='sequential_quantile':mapping=mapping.palette_scheme({'id':'Blues','size':3})
 p=p.scale(mapping).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,100.))).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,100.)))
 if facets:p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2).free_y(True))
 result=p.build();d.dispose();scale.dispose();return result
for classifier in ['ordinal','quantile','sequential_quantile']:
 for facets in [False,True]:
  rows=[(1,base+1,0.,'A'),(2,base+2,2.,'B'),(3,base+1,4.,'A')]
  p=author(rows,classifier,facets);chart=p.chart();chart.present(output,options).dispose();p.dispose()
  old=chart.request(output,options.basis('presented'));frame=old.prepare();png=frame.export('png');frame.dispose()
  for step in range(4):
   tx=chart.transaction().id(f'scale-{step}')
   if step==0:
    r=(4,base+3,100.,'B');rows.append(r);batch=data([r]);tx=tx.append('live',batch);batch.dispose()
   elif step==1:
    r=(2,base+3,8.,'B');rows[1]=r;batch=data([r]);tx=tx.upsert('live',batch);batch.dispose()
   elif step==2:rows.pop(0);tx=tx.remove('live',[1])
   else:rows.pop(0);tx=tx.retain_count('live',2)
   tx=tx.build();assert 'Applied'in chart.commit(tx);tx.dispose()
   before=old.prepare();assert before.export('png')==png;before.dispose()
   current=chart.request(output,options);fresh_plot=author(rows,classifier,facets);fresh=output.request(fresh_plot,options);fresh_plot.dispose()
   a=current.prepare();b=fresh.prepare();assert a.export('png')==b.export('png'),(classifier,facets,step)
   a.dispose();b.dispose();current.dispose();fresh.dispose()
  chart.dispose();frame=old.prepare();assert frame.export('png')==png;frame.dispose();old.dispose()
  records.append({'family':classifier,'facets':facets,'updates':4,'fresh_batch_rgba_equal':True,'retained_capture':True})
out.write_text(json.dumps(records,indent=2)+'\n')
print('PASS CP-05 Python: twenty-four append/upsert/remove/retention steps match fresh batch PNG; exact uint64 categories and old captures survive.')
