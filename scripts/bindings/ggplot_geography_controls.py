"""FIX-GG15 independent Python geography authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
def ring(x,y,w,h):return [[x,y],[x+w,y],[x+w,y+h],[x,y+h],[x,y]]
for mode in range(19):
    polygons=[[ring(-20.,10.,30.,40.),ring(-10.,20.,10.,15.)],[ring(20.,15.,15.,20.)]]
    if mode==3:polygons=[[[[p[1],p[0]] for p in r] for r in polygon] for polygon in polygons]
    collection={'features':[{'id':{'Text':'land'},'geometry':{'MultiPolygon':polygons},'crs':None}],'crs':'Wgs84','axis_order':'YX' if mode==3 else 'XY'}
    if mode==12:collection['features'][0]['geometry']={'Polygon':[[[170.,10.],[-170.,10.],[-170.,40.],[170.,40.],[170.,10.]],[[160.,20.],[-160.,20.],[-160.,30.],[160.,30.],[160.,20.]]]}
    if mode==16:collection['features'].append({'id':{'Text':'city'},'geometry':{'Point':[1113194.9079327357,5621521.486192066]},'crs':'WebMercator'})
    data=c.Data.columns({'id':['land','city' if mode==16 else 'unmatched'],'value':[1.,2.],'x':[-30.,45.],'y':[5.,60.]})
    layer=c.points().geography(collection,'id')
    operations={4:'Centroid',5:'PointOnSurface',6:'Borders',7:'Coordinates',14:'PointOnSurface',15:'PointOnSurface'}
    if mode in operations:layer=layer.geography_operation(operations[mode])
    if mode in [14,15]:layer=layer.text_geom({'fill':{'red':255,'green':255,'blue':220,'alpha':255}} if mode==15 else {}).text_label('id')
    if mode==11:layer=layer.fill('#40a0d0').color('#603030').linewidth(0.8).alpha(0.5)
    if mode in [1,4,5,10,11,12,13,14,15]:projection={'Crs':'WebMercator'}
    elif mode==2:projection={'Crs':{'Proj':'+proj=utm +zone=31 +datum=WGS84 +units=m +no_defs'}}
    elif mode in [8,9]:projection={'Mapproj':{'method':'mollweide' if mode==8 else 'tetra','parameters':[],'orientation':[90.,0.,0.]}}
    else:projection={'Crs':'Wgs84'}
    coordinate={'projection':projection,'default_crs':'Wgs84'}
    if mode==10:coordinate.update(limits_method='GeometryBounds',view={'xlim':[{'Number':-5.},{'Number':5.}]})
    if mode==17:coordinate['graticule']={'longitude':[-20.,0.,20.],'latitude':[10.,30.,50.],'label_axes':['Longitude','Latitude','Longitude','Latitude']}
    if mode==18:coordinate['graticule']={'datum':None}
    builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer).coordinate({'Geographic':coordinate})
    if mode==13:builder=builder.layer(c.line().aes(c.aes().x('x').y('y')).color('red'))
    if mode>=17:builder=builder.theme(c.theme().reference_preset('Grey',{}))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(300 if mode==12 else 144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(300 if mode==12 else 144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'geography-{mode}.plot.json').write_text(wire);(out/f'geography-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'geography-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python geography controls: 19 authors, 57 publications.')
