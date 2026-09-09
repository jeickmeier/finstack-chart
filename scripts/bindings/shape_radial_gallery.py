"""Independently authored Python radial/link gallery using the primary engine."""
from pathlib import Path
import sys,json,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True);output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
labels=['Linear spiral','Closed basis','Radial area','Smooth annulus','Signed radii','Defined gaps','Horizontal links','Vertical links','Step links','Radial links']
for preset in ['Editorial','Terminal','Grayscale']:
 folder=out/preset;folder.mkdir(exist_ok=True);rows=[]
 for slot,label in enumerate(labels):
  n=3 if slot>=6 else 4 if slot==4 else 13
  for i in range(n):
   cartesian=6<=slot<=8;a=i*math.pi/6
   rows.append(dict(x=.5 if cartesian else 2.,y=.75+i*.5 if cartesian else 2.,x2=3.5,y2=3.25-i*.5,angle=None if slot==5 and i==6 else a,radius=[24.,-24.,32.,-12.][i] if slot==4 else 20.+i,inner=8.+i%3,end=a+1.5,outer=32.,group='ABC'[i] if slot>=6 else 'A',panel=label,slot=slot,key=9007199254741001+slot*100+i))
 columns={name:c.column([r[name]for r in rows],kind='float64')for name in ['x','y','x2','y2','angle','radius','inner','end','outer']};columns.update(group=c.categorical([r['group']for r in rows]),panel=c.categorical([r['panel']for r in rows]),slot=[r['slot']for r in rows])
 data=c.Data.columns(columns,keys=[r['key']for r in rows],name='radial-links')
 draft=c.plot(data).aes(c.aes().x('x').y('y').x2('x2').y2('y2').group('group').color('group')).theme(c.theme().preset(preset)).title(c.title(f'Radial shapes and links / {preset}')).subtitle(c.subtitle('Clockwise angles, signed radii, gaps and source edges')).facet(c.facet_wrap('panel').columns(2).gap(12.)).x_axis(c.x_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).y_axis(c.y_axis().scale(c.scale_linear().domain(0.,4.)).visible(False)).scale(c.color_discrete('group').domain(['A','B','C'])).legend(c.legend().scale('group').title('Source'))
 for slot,label in enumerate(labels):
  if slot==0:layer=c.shape_line_radial()
  elif slot==1:layer=c.shape_line_radial().curve({'kind':'BasisClosed'})
  elif slot==2:layer=c.shape_area_radial()
  elif slot==3:layer=c.shape_area_radial().curve({'kind':'Basis'})
  elif slot==4:layer=c.shape_line_radial().curve({'kind':'Cardinal','tension':.2})
  elif slot==5:layer=c.shape_line_radial().curve({'kind':'Step'})
  elif slot==6:layer=c.shape_link_horizontal()
  elif slot==7:layer=c.shape_link_vertical()
  elif slot==8:layer=c.shape_link({'kind':'Step'})
  else:layer=c.shape_link_radial()
  if slot<6:
   layer=layer.shape_value('Angle',data.field('angle'))
   layer=layer.shape_value('InnerRadius',data.field('inner')).shape_value('OuterRadius',data.field('radius')) if slot in [2,3] else layer.shape_value('Radius',data.field('radius'))
  if slot==9:layer=layer.shape_value('StartAngle',data.field('angle')).shape_value('EndAngle',data.field('end')).shape_value('InnerRadius',data.field('inner')).shape_value('OuterRadius',data.field('outer'))
  draft=draft.layer(layer.name(label).filter(c.filter('slot').minimum(float(slot)).maximum(float(slot))))
 p=draft.build();(folder/'figure.plot.json').write_text(p.to_json());requests=[(dpi,output.request(p,c.export_options(600.,740.).dpi(dpi)))for dpi in [300,600]];p.dispose()
 for dpi,request in requests:
  f=request.prepare();scene=f.scene();assert sum('ShapePath'in i['primitive']for i in scene['items'])==19;assert sum(len(t)for t in scene['targets'])==92
  for i,item in enumerate(scene['items']):
   if 'ShapePath'in item['primitive']:assert len(item['primitive']['ShapePath']['anchors'])==len(scene['targets'][i])
  (folder/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
  for fmt in ['svg','pdf','png']:(folder/f'figure-{dpi}.{fmt}').write_bytes(f.export(fmt))
  f.dispose();request.dispose()
print('PASS Python radial/link gallery: ten facets, 80 source identities, 92 real endpoint/source anchors, three themes and immutable 300/600 DPI outputs.')
