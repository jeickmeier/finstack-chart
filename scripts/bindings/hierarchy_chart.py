"""HIR-07: independently authored primary chart recipes through the real Python facade."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
data=c.Data.columns({'id':['root','North','South','A','B','C','D','E','F'],'parent':[None,'root','root','North','North','North','South','South','South'],'value':[0.,0.,0.,8.,5.,3.,7.,4.,2.],'group':['root','North','South','North','North','North','South','South','South']},keys=[9007199254741001+i for i in range(9)],name='hierarchy')
resquarify={'Treemap':{'options':{'tile':{'Resquarify':1.618033988749895},'padding_inner':3.,'padding_top':5.,'padding_right':5.,'padding_bottom':5.,'padding_left':5.},'history':True}}
layers=[('tree',c.hierarchy_tree('id','parent')),('cluster-horizontal',c.hierarchy_cluster('id','parent').hierarchy_projection('Horizontal')),('tree-radial',c.hierarchy_tree('id','parent').hierarchy_projection('Radial')),('cluster-radial',c.hierarchy_cluster('id','parent').hierarchy_projection('Radial')),('icicle',c.hierarchy_icicle('id','parent')),('sunburst',c.hierarchy_sunburst('id','parent').hierarchy_projection({'Sunburst':{'inner_radius':18.,'radius':'Area'}})),('treemap',c.hierarchy_treemap('id','parent').hierarchy_layout(resquarify)),('pack',c.hierarchy_pack('id','parent')),('tree-node-size',c.hierarchy_tree('id','parent').hierarchy_layout({'Tree':{'options':{'mode':{'NodeSize':[30.,80.]}}}}))]
plots=[(name,layer,c.plot(data).layer(layer.name('hierarchy').hierarchy_value(data.field('value')).hierarchy_label(data.field('id')).aes(c.aes().color('id'))).title(c.title(name)).build()) for name,layer in layers]
for name,layer,p in plots:
 wire=p.to_json();assert json.loads(wire)['version']==15
 loaded=c.Plot.from_json(wire);assert loaded.to_json()==wire
 (out/f'{name}.plot.json').write_text(wire)
 for mode in ['preserve','outline']:
  request=output.request(loaded,c.export_options(500.,360.).dpi(144).text(mode));frame=request.prepare();scene=frame.scene()
  h=scene['hierarchies']['snapshots'][0]
  assert h['nodes'][0]['value']==29. and len(h['nodes'])==9
  assert sorted(h['source_keys'])==[str(9007199254741001+i) for i in range(9)]
  prefix=f'{name}-'+('text' if mode=='preserve' else mode)
  (out/f'{prefix}.scene.json').write_text(json.dumps(scene))
  for fmt in ['svg','pdf','png']:(out/f'{prefix}.{fmt}').write_bytes(frame.export(fmt))
  if name=='treemap':
   assert h['history_rows']>0
   resized_request=output.request(loaded,c.export_options(300.,500.).dpi(72)).with_hierarchy_history(frame)
   resized=resized_request.prepare();assert resized.scene()['hierarchies']['snapshots'][0]['history_members']==h['history_members']
   (out/f'{prefix}-resized.scene.json').write_text(json.dumps(resized.scene()))
   resized.dispose();resized_request.dispose()
  frame.dispose();request.dispose()
 loaded.dispose();p.dispose();layer.dispose()
# Registered chart operations resolve once into the captured core registry.
registry=c.ExtensionRegistry.example()
def op(mode, native=False): return {'operation':{'id':'example.native_hierarchy' if native else 'example.hierarchy','version':'1'},'parameters':{'mode':mode}}
layer=c.hierarchy_pack('id','parent').hierarchy_aggregation({'Registered':op('Value')}).hierarchy_order({'Registered':op('DescendingValue')}).hierarchy_layout({'Pack':{'options':{'radius':'Explicit'},'radius':{'Registered':op('DepthPadding')}}})
p=c.plot(data).with_registry(registry).layer(layer).build()
request=output.request(p,c.export_options(400.,300.));registry.dispose();p.dispose()
f=request.prepare();h=f.scene()['hierarchies']['snapshots'][0];assert h['nodes'][0]['value']==29.
assert all(n['geometry']['Circle']['r']==3. for n in h['nodes'] if not n['children'])
f.dispose();request.dispose();layer.dispose()
registry=c.ExtensionRegistry.example()
try:c.plot(data).with_registry(registry).layer(c.hierarchy_pack('id','parent').hierarchy_aggregation({'Registered':op('Value',True)})).build()
except c.ChartError:pass
else:raise AssertionError('native-only hierarchy callback accepted in portable chart')
registry.dispose()
# A path source includes inferred ancestors while retaining only actual source rows.
paths=c.Data.columns({'path':['a/b','a/c/d'],'v':[2.,3.]},keys=[71,72])
recipe={'identity':'991','source':{'Paths':'path'},'aggregation':{'Sum':'v'},'label':None,'layout':{'Partition':{}}}
p=c.plot(paths).layer(c.hierarchy(recipe)).build();f=output.request(p,c.export_options(300.,200.)).prepare();h=f.scene()['hierarchies']['snapshots'][0]
assert len(h['nodes'])==4 and len(h['source_keys'])==2
f.dispose();p.dispose();paths.dispose();data.dispose();output.dispose()
print('PASS Python HIR-07: nine projections, Field selectors, v15 round trip, source identities, retained resize history, synthetic ancestry, SVG/PDF/PNG.')
