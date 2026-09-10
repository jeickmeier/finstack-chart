"""AXIS-01: actual registration, noninjective projection, independent guides and v10."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
def reject(fn,code):
 try:fn()
 except c.ChartError as e:assert e.code==code,(e,code)
 else:raise AssertionError('Expected '+code)
registry=c.ExtensionRegistry.example()
assert c.ExtensionRegistry is c.ShapeRegistry
base=9007199254741001
d=c.Data.columns({'x':[-10.,-2.,2.,10.],'y':[0.,1.,2.,3.]},keys=[base+i for i in range(4)],name='source')
def builder(name='example.fold',limit=10.,version=1):
 return (c.plot(d).with_registry(registry).aes(c.aes().x('x').y('y')).layer(c.points())
  .x_axis(c.x_axis().coordinate_scale(c.scale_registered(name,version,{'limit':limit})).range(100.,500.).visible(False))
  .y_axis(c.y_axis().range(300.,100.).visible(False)))
def guide(name,side,dx,dy):return c.axis_guide(name,'x').side(side).translate(dx,dy).ticks([(v,f'{name} {v:g}') for v in [0.,2.,10.]])
p=(builder().guide(guide('top','Top',0.,0.)).guide(guide('lower','Bottom',10.,30.)).title(c.title('Folded values / one scale / two guides')).build())
wire=p.to_json();payload=json.loads(wire);assert payload['version']==10
reject(lambda:c.Plot.from_json(wire),'CHART_UNSUPPORTED_CAPABILITY')
loaded=c.Plot.from_json(wire,registry);assert loaded.to_json()==wire
reject(lambda:builder(version=2).build(),'CHART_UNSUPPORTED_CAPABILITY')
reject(lambda:builder(limit=-1.).build(),'CHART_NUMERICAL_DOMAIN')
native=builder('example.native_fold').build();reject(native.to_json,'CHART_UNSUPPORTED_CAPABILITY')
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(600.,400.).dpi(72).basis('current')
reject(lambda:output.request(native,options).prepare(),'CHART_UNSUPPORTED_CAPABILITY');native.dispose()
request=output.request(p,options);copy=registry.copy();registry.dispose();p.dispose();frame=request.prepare();scene=frame.scene()
points=[i['primitive']['Point']['center'] for i in scene['items'] if 'Point' in i['primitive']]
assert [p['x'] for p in points]==[500.,180.,180.,500.],points
for name,offset in [('top',0.),('lower',10.)]:
 for i,item in enumerate(scene['items']):
  text=item['primitive'].get('Text')
  if text and text['text'].startswith(name+' '):
   v=float(text['text'].split()[1]);assert scene['items'][i-1]['primitive']['Rule']['from']['x']==100.+40.*v+offset
assert sorted(int(t['Source']['key']) for group in scene['targets'] for t in group)==[base+i for i in range(4)]
(out/'provider.plot.json').write_text(wire);(out/'provider.scene.json').write_text(json.dumps(scene))
for format in ['svg','pdf','png']:(out/f'provider.{format}').write_bytes(frame.export(format))
second=output.request(loaded,options).prepare();assert second.export('png')==frame.export('png');second.dispose();loaded.dispose()
frame.dispose();frame=request.prepare();assert frame.export('png')==(out/'provider.png').read_bytes();frame.dispose()
copy.dispose();request.dispose();output.dispose();d.dispose()
print('PASS Python provider: exact noninjective mark and guide positions, v10 registration, invalid versions/parameters, native-only rejection, copied registry and retained request after disposal, SVG/PDF/PNG.')
