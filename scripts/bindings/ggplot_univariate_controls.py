"""FIX-GG09 independent Python precomputed distribution geometry authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(8):
    data=c.Data.columns({'x':[1.,2.,2.,4.,5.,6.],'y':[1.,3.,3.,2.,5.,4.],'w':[1.,2.,0.,1.,3.,1.]})
    mapping=c.aes()
    if mode==0:layer=c.density().stat(c.density_stat().input('x').weight('w'))
    elif mode==1:layer=c.ecdf().stat(c.ecdf_stat().input('x').weight('w'))
    elif mode==2:layer=c.qq().stat(c.qq_stat().input('y'))
    elif mode==3:layer=c.qq_line().stat(c.qq_line_stat().input('y'))
    elif mode in [4,5]:layer=c.line().stat(c.univariate_stat({'Function':{'function':{'Expression':{'nodes':[{'Read':'Value'},{'Read':'Value'},{'Binary':{'op':'Multiply','left':0,'right':1}}],'output':2}},'n':21,'range':None}}).input('x'))
    elif mode==6:layer=c.points().stat(c.unique_stat().x('x').y('y'))
    else:layer=c.line().stat(c.connect_stat({'Matrix':[[0.,0.],[.25,.75],[1.,1.]]}).x('x').y('y'))
    builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer)
    if mode==5:builder=builder.x_axis(c.x_axis().scale(c.scale_log(10.)))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'univariate-{mode}.plot.json').write_text(wire);(out/f'univariate-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'univariate-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python distribution geometry: eight authors, 24 publications.')
