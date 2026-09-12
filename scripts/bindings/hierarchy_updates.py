"""HIR-08 actual-host updates, deterministic replay, batch and snapshot ownership."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
records=[]
def data(values,parents):
 return c.Data.columns({'id':['root','a','b','c','d','e','f'],'parent':parents,'value':values},keys=list(range(9007199254741001,9007199254741008)),name='live')
parents=[None,'root','root','a','a','b','b']
layouts=[{'Tree':{'options':{}}},{'Cluster':{'options':{}}},{'Partition':{}},{'Pack':{'options':{}}},{'Treemap':{'options':{},'history':False}},{'Treemap':{'options':{'tile':{'Resquarify':1.6}},'history':True}}]
for index,layout in enumerate(layouts):
 recipe={'identity':'991','source':{'Table':{'id':'id','parent':'parent'}},'aggregation':{'Sum':'value'},'label':None,'layout':layout}
 def author(values,parents):
  d=data(values,parents);p=c.plot(d).layer(c.hierarchy(recipe)).build();d.dispose();return p
 initial=[0.,0.,0.,8.,5.,3.,7.];p=author(initial,parents);wire=p.to_json();charts=[p.chart(),c.Plot.from_json(wire).chart()];p.dispose()
 opts=c.export_options(480.,280.).dpi(72).basis('current')
 for chart in charts:chart.present(output,opts).dispose()
 old=charts[0].request(output,opts.basis('presented'));f=old.prepare();old_png=f.export('png');f.dispose()
 for step,(values,par,size) in enumerate([([0.,0.,0.,1.,9.,4.,7.],parents,(480.,280.)),([0.,0.,0.,1.,9.,4.,7.],parents,(280.,480.)),([0.,0.,0.,1.,9.,4.,7.],[None,'root','root','b','a','b','b'],(280.,480.))]):
  for chart in charts:
   d=data(values,par);tx=chart.transaction().id(f'hierarchy-{step}').replace('live',d).build();d.dispose();assert 'Applied' in chart.commit(tx);assert 'AlreadyApplied' in chart.commit(tx);tx.dispose()
  frames=[chart.present(output,c.export_options(*size).dpi(72).basis('current')) for chart in charts]
  nodes=[f.scene()['hierarchies']['snapshots'][0]['nodes'] for f in frames];assert nodes[0]==nodes[1]
  assert frames[0].export('png')==frames[1].export('png')
  if index!=5:
   batch=author(values,par);req=output.request(batch,c.export_options(*size).dpi(72));fresh=req.prepare();assert fresh.scene()['hierarchies']['snapshots'][0]['nodes']==nodes[0];assert fresh.export('png')==frames[0].export('png');fresh.dispose();req.dispose();batch.dispose()
  saved=old.prepare();assert saved.export('png')==old_png;saved.dispose()
  records.append({'family':index,'step':step,'nodes':nodes[0],'replay_equal':True,'batch_equal':index!=5,'old_snapshot_equal':True})
  for f in frames:f.dispose()
 for chart in charts:chart.dispose()
 f=old.prepare();assert f.export('png')==old_png;f.dispose();old.dispose()
out.write_text(json.dumps(records,indent=2)+'\n');output.dispose()
print('PASS Python hierarchy: 18 update/resize/reparent traces, duplicate transaction rejection, equivalent histories, 15 stateless batch comparisons, retained exports after disposal.')
