"""FIX-S05/09 all independent paths through actual Python charts and source identities."""
from pathlib import Path
import sys,json,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
corpus=json.loads((ROOT/'fixtures/shapes/radial.json').read_text());output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(400.,200.).dpi(72).layout(c.layout_options().padding(0));base=9007199254741001
C=lambda v:{'Constant':v};K=lambda i:{'Column':i}
def read(selector,row):return selector['Constant']if'Constant'in selector else row[selector['Column']]
def endpoint(cfg,name,datum):
 value=cfg.get(name,name)
 return datum[value.lower()]if isinstance(value,str)else value['Constant']
def compare(a,b):
 if isinstance(a,(int,float))and isinstance(b,(int,float)):assert math.isfinite(a)and abs(a-b)<=2e-12*max(1,abs(b)),(a,b)
 elif isinstance(a,list):
  assert len(a)==len(b),(len(a),len(b))
  for x,y in zip(a,b):compare(x,y)
 elif isinstance(a,dict):
  assert a.keys()==b.keys()
  for k in a:compare(a[k],b[k])
 else:assert a==b,(a,b)
def transformed(commands,radial):
 result=[]
 for command in commands:
  if isinstance(command,str):result.append(command);continue
  op,values=next(iter(command.items()));assert op in ['MoveTo','LineTo','QuadraticTo','CubicTo']
  result.append({op:[v*(1 if radial else 4 if i%2==0 else -2)+(200 if i%2==0 else 100)for i,v in enumerate(values)]})
 return result
def axes(draft,radial):return draft.x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)if radial else c.scale_linear().domain(-50.,50.)).visible(False)).y_axis(c.y_axis().scale(c.scale_log(10.).domain(1.,10000.)if radial else c.scale_linear().domain(-50.,50.)).visible(False))
for case in corpus['cases']:
 cfg=case['config'];family=case['family'];radial=family!='link';anchor_by_key={};curve=cfg.get('curve',{'kind':'BumpX'if family=='link'else'Linear'})
 if family.startswith('link'):
  s=endpoint(cfg,'source',case['input']);t=endpoint(cfg,'target',case['input']);xs=cfg.get('angle'if radial else'x',K(0));ys=cfg.get('radius'if radial else'y',K(1));x,y,x2,y2=read(xs,s),read(ys,s),read(xs,t),read(ys,t)
  anchor_by_key[base]=[{'x':200+(r*math.cos(a-math.pi/2)if radial else a*4),'y':100+(r*math.sin(a-math.pi/2)if radial else r*-2)}for a,r in [(x,y),(x2,y2)]]
  data=c.Data.columns({'value':[0.]},keys=[base]);layer=c.shape_link_radial().shape_value('StartAngle',x).shape_value('InnerRadius',y).shape_value('EndAngle',x2).shape_value('OuterRadius',y2).aes(c.aes().x(2.).y(100.))if radial else c.shape_link(curve).aes(c.aes().x(x).y(y).x2(x2).y2(y2))
 else:
  x0,y0,x1,y1=K(0),C(0.),None,K(1);area=family=='areaRadial'and case['helper']is None
  if family=='lineRadial':x0,y0=cfg.get('angle',K(0)),cfg.get('radius',K(1))
  else:
   x0,y0,x1,y1=cfg.get('start_angle',x0),cfg.get('inner_radius',y0),cfg.get('end_angle',x1),cfg.get('outer_radius',y1)
   if'angle'in cfg:x0,x1=cfg['angle'],None
   if'radius'in cfg:y0,y1=cfg['radius'],None
   if case['helper']=='EndAngle':x0=x1 or C(0.)
   if case['helper']=='OuterRadius':y0=y1 or C(0.)
  mask=cfg.get('defined',True);defined=lambda i:mask if isinstance(mask,bool)else mask[i]
  values=lambda selector:c.column([read(selector,row)if defined(i)else None for i,row in enumerate(case['input'])],kind='float64')
  data=c.Data.columns({'a':values(x0),'r':values(y0),'a2':values(x1 or x0),'r2':values(y1 or y0)},keys=[base+i for i in range(len(case['input']))])
  layer=c.shape_area_radial().shape_value('StartAngle',data.field('a')).shape_value('InnerRadius',data.field('r')).shape_value('EndAngle',data.field('a2')).shape_value('OuterRadius',data.field('r2'))if area else c.shape_line_radial().shape_value('Angle',data.field('a')).shape_value('Radius',data.field('r'))
  layer=layer.curve(curve).aes(c.aes().x(2.).y(100.))
  for i,row in enumerate(case['input']):
   if defined(i):
    a,r=read((x1 or x0)if area else x0,row),read((y1 or y0)if area else y0,row);anchor_by_key[base+i]=[{'x':200+r*math.sin(a),'y':100-r*math.cos(a)}]
 p=axes(c.plot(data).layer(layer),radial).build();wire=p.to_json();assert json.loads(wire)['version']==7;decoded=c.Plot.from_json(wire);assert decoded.to_json()==wire;decoded.dispose();request=output.request(p,options);p.dispose();f=request.prepare();scene=f.scene();commands=[]
 for i,item in enumerate(scene['items']):
  if'ShapePath'in item['primitive']:
   shape=item['primitive']['ShapePath'];commands.extend(shape['geometry']['commands']);assert len(shape['anchors'])==len(scene['targets'][i])
   for n,target in enumerate(scene['targets'][i]):
    key=int(target['Source']['key']);assert key in anchor_by_key;compare(shape['anchors'][n],anchor_by_key[key][n if family.startswith('link')else 0])
 expected=c.Path();expected.apply_batch(case['operations']);compare(commands,transformed(expected.result()['geometry']['commands'],radial));expected.dispose();f.dispose();request.dispose();data.dispose()
# Both edge endpoints select the same exact source identity; controls do not select.
d=c.Data.columns({'value':[0.]},keys=[base]);p=axes(c.plot(d).layer(c.shape_link_horizontal().aes(c.aes().x(-50.).y(-50.).x2(50.).y2(50.))),False).build();chart=p.chart();p.dispose();f=chart.present(output,options);shape=next(i['primitive']['ShapePath']for i in f.scene()['items']if'ShapePath'in i['primitive']);assert len(shape['anchors'])==2
first=chart.select_region({'Rectangle':[0.,190.,10.,10.]})['targets'];second=chart.select_region({'Rectangle':[390.,0.,10.,10.]})['targets'];assert len(first)==1 and first==second and int(first[0]['identity']['Source']['key'])==base
assert chart.select_region({'Rectangle':[190.,190.,20.,10.]})['targets']==[]
hit=chart.inspect(200.,100.,mode='Containment')['targets'];assert len(hit)==1;chart.focus(hit[0]);chart.present(output,options).dispose();assert chart.inspect(-1.,100.,mode='Containment')['targets']==[];chart.dispose();f.dispose()
# A complete annulus retains its hole; a clipped radial run keeps its real source anchors.
for clipped in [False,True]:
 d=c.Data.columns({'a':[i*math.tau/8 for i in range(9)]},keys=[base+i for i in range(9)])
 layer=c.shape_area_radial().shape_value('Angle',c.source_expr(d.field('a'))*1.).shape_value('InnerRadius',10.).shape_value('OuterRadius',30.)
 p=axes(c.plot(d).aes(c.aes().x(-.15 if clipped else 2.).y(100.)).layer(layer),True).build();chart=p.chart();p.dispose();f=chart.present(output,options)
 if not clipped:assert chart.inspect(200.,100.,mode='Containment')['targets']==[]
 hit=chart.inspect(10.,100.,mode='Containment')['targets']if clipped else chart.inspect(200.,80.,mode='Containment')['targets'];assert len(hit)==1;chart.focus(hit[0]);chart.present(output,options).dispose();assert chart.inspect(-1.,100.,mode='Containment')['targets']==[];chart.dispose();f.dispose()
# Same-schema aggregate rows remain one edge identity despite two endpoint anchors.
d=c.Data.columns({'category':['a','b','b']});p=c.plot(d).layer(c.shape_link_radial().stat(c.count().group('category')).after_stat(c.stat_aes().x(2.).y(100.)).shape_value('StartAngle',0.).shape_value('EndAngle',1.).shape_value('InnerRadius',10.).shape_value('OuterRadius',{'Statistical':'Count'})).build();f=output.request(p,options).prepare();targets=[group for group in f.scene()['targets']if group];assert len(targets)==2 and all(len(g)==2 and g[0]==g[1]for g in targets);assert [len(g[0]['Aggregate']['members'])for g in targets]==[1,2];f.dispose();p.dispose()
# A seven-vertex figure cannot admit two five-vertex quarter sectors.
d=c.Data.columns({'facet':c.categorical(['A','B'])});p=c.plot(d).facet(c.facet_wrap('facet')).layer(c.shape_arc().shape_value('EndAngle',math.pi/2)).compile_limits({'max_vertices':7}).build()
try:output.request(p,options).prepare()
except c.ChartError as e:assert e.code=='CHART_RESOURCE_LIMIT'
else:raise AssertionError('facet command/anchor budget')
p.dispose()
print('PASS Python radial/link charts: 697 reference paths, exact edge/source identities, endpoint selection, non-source controls, holes/clipped focus, generated aggregates, versioned round trips and cross-panel budgets.')
