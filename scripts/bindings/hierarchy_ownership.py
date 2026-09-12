"""HIR-08 owned values and explicit disposal; tracemalloc measures Python allocations only."""
from pathlib import Path
import gc,json,sys,tracemalloc
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
envelope={'version':1,'identity':'9007199254740999','input':{'Rows':[{'key':str(i+1),'parent':str((i-1)//2+1) if i else None,'data':{'value':i%7}} for i in range(1000)]}}
def cycle():
 tree=c.Hierarchy(envelope).count().layout({'Tree':{'options':{}}});saved=tree.nodes();root=saved[0]['handle'];copy=tree.copy();serialized=copy.to_json()
 saved[0]['data']['value']=999;assert tree.node(root)['data']['value']!=999
 tree.dispose();tree.dispose()
 try:tree.nodes()
 except c.ChartError as error:assert error.code=='CHART_DISPOSED_HANDLE'
 else:raise AssertionError('disposed hierarchy accepted')
 restored=c.Hierarchy.from_json(serialized);assert restored.to_json()==serialized
 copy.count();assert restored.to_json()==serialized;copy.dispose();restored.dispose()
 return saved
tracemalloc.start()
for _ in range(20):cycle()
gc.collect();warm=tracemalloc.get_traced_memory()[0];samples=[]
for _ in range(60):cycle();gc.collect();samples.append(tracemalloc.get_traced_memory()[0])
assert samples[-1]-warm<=4194304
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());data=c.Data.columns({'id':['r','a'],'parent':[None,'r'],'v':[0.,1.]})
p=c.plot(data).layer(c.hierarchy_pack('id','parent').hierarchy_value('v')).build();request=output.request(p,c.export_options(200.,200.));frame=request.prepare();payload=frame.export('svg');scene=frame.scene()
frame.dispose();request.dispose();p.dispose();data.dispose();output.dispose();assert payload.startswith(b'<svg') or b'<svg' in payload;assert scene['hierarchies']['snapshots'][0]['nodes'][0]['value']==1.
Path(sys.argv[2]).write_text(json.dumps({'version':1,'nodes':1000,'warmup':20,'cycles':60,'python_tracemalloc_warm_bytes':warm,'samples_bytes':samples,'retained_growth_bytes':samples[-1]-warm,'owned_scene_and_svg_after_disposal':True,'verdict':'PASS'},indent=2))
print('PASS Python hierarchy ownership: disposed/copied/restored sessions, bounded retained Python allocations and owned chart bytes/metadata.')
