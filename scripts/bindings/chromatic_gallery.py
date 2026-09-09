"""FIX-21 independently authored named palettes and retained v6 publication."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(720.,420.).dpi(144)
figures=[]
for name,id,size in [('categorical','Category10',None),('brewer','Blues',5)]:
    scale=c.StandaloneScale('ordinal',range=['black'])
    p=(c.plot(c.Data.columns({'x':[float(i) for i in range(10)],'group':c.categorical([f'C{i}' for i in range(10)])}))
       .aes(c.aes().x('x').y(1.).color('group').color_scale('catalog')).layer(c.points().size(13.))
       .scale(c.color_mapped('catalog',scale,'Eligible').palette_scheme({'id':id,'size':size}))
       .legend(c.legend().scale('catalog').title('Blues / exact k = 5' if size else 'Category10'))
       .y_axis(c.y_axis().visible(False)).title(c.title(f'Named palette / {name}')).theme(c.theme().preset('Editorial')).build())
    figures.append((name,p))
for name,id,domain,family in [('lookup','Viridis',[0.,1.],'sequential'),('diverging','RdBu',[-10.,0.,100.],'diverging'),('cyclic','Rainbow',[0.,1.],'sequential')]:
    values=[-10.,-8.,-6.,-4.,-2.,0.,20.,40.,60.,80.,100.] if name=='diverging' else [i/32. for i in range(33)]
    ramp=c.chromatic(id);scale=c.StandaloneScale(family,domain=domain,interpolator=ramp);ramp.dispose()
    p=(c.plot(c.Data.columns({'x':values})).aes(c.aes().x('x').y(1.).color('x').color_scale('catalog')).layer(c.points().size(13.))
       .scale(c.color_mapped('catalog',scale)).legend(c.legend().scale('catalog').title(id)).y_axis(c.y_axis().visible(False))
       .title(c.title(f'Named ramp / {name}')).theme(c.theme().preset('Editorial')).build());figures.append((name,p))
for name,plot in figures:
    wire=plot.to_json();assert json.loads(wire)['version']==6
    copy=c.Plot.from_json(wire);assert copy.to_json()==wire;copy.dispose();(out/f'{name}.plot.json').write_text(wire)
    request=output.request(plot,options);plot.dispose();frame=request.prepare();(out/f'{name}.scene.json').write_text(json.dumps(frame.scene(),indent=2))
    for fmt in ['svg','pdf','png']:(out/f'{name}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose()
print('PASS FIX-21 Python: five independently authored v6 catalog figures, retained SVG/PDF/PNG.')
# Non-finite categorical numbers are missing observations, not palette-training keys.
scale=c.StandaloneScale('ordinal',range=['black'])
p=(c.plot(c.Data.columns({'x':[0.,1.,2.,3.,4.],'key':[0.,float('nan'),float('inf'),-float('inf'),1.]}))
 .aes(c.aes().x('x').y(1.).color('key').color_scale('named')).layer(c.points())
 .scale(c.color_mapped('named',scale,'Eligible').palette_scheme({'id':'Category10'}).missing('#0b16212c')).build())
request=output.request(p,options);frame=request.prepare()
colors=[i['primitive']['Point']['fill'] for i in frame.scene()['items'] if 'Point'in i['primitive'] and i.get('layer') is not None]
rgba=lambda r,g,b,a=255:dict(red=r,green=g,blue=b,alpha=a)
assert colors==[rgba(31,119,180),rgba(11,22,33,44),rgba(11,22,33,44),rgba(11,22,33,44),rgba(255,127,14)],colors
frame.dispose();request.dispose();p.dispose();scale.dispose()
print('PASS CP-04 Python: non-finite numeric categories retain missing paint and do not shift named palette training.')
