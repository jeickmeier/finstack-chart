"""FIX-GG10 independent Python weighted model authors."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for mode in range(10):
    data=c.Data.columns({'x':[float(i) for i in range(24)],'y':[1.+i%5+0.2*i for i in range(24)],'w':[float(1+i%3) for i in range(24)]})
    if mode in [0,7,8,9]:method='Linear'
    elif mode==1:method={'Glm':{'family':'Poisson','epsilon':1e-8,'iterations':25}}
    elif mode==2:method={'Loess':{'span':0.75,'degree':2,'cell':0.2,'surface':'Interpolate','family':'Gaussian','iterations':4,'normalize':True,'exact_statistics':False,'approximate_trace':False}}
    elif mode==3:method={'Gam':{'basis_dimension':10,'knots':None,'iterations':120,'tolerance':1e-9}}
    elif mode in [4,5]:method={'Quantile':{'solver':'Br' if mode==4 else 'Fn','probabilities':[0.25,0.5,0.75],'iterations':10000}}
    else:method='Auto'
    options={'method':method,'terms':[{'Power':0},{'Power':1},{'Power':2}] if mode==0 else None,'n':41,'xseq':None,'full_range':mode==9,'se':mode not in [4,5],'level':0.5 if mode==7 else 0.95}
    layer=c.smooth().stat(c.model_stat(options).x('x').y('y').weight('w'))
    if mode==8:layer=layer.orientation('Horizontal')
    builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(-2. if mode==8 else -1.,12. if mode==8 else 24.))).y_axis(c.y_axis().scale(c.scale_linear().domain(-1. if mode==8 else -2.,24. if mode==8 else 12.)))
    p=builder.build();wire=p.to_json();restored=c.Plot.from_json(wire)
    request=output.request(restored,c.export_options(600.,360.).dpi(144));frame=request.prepare();scene=frame.scene()
    original_request=output.request(p,c.export_options(600.,360.).dpi(144));original_frame=original_request.prepare();assert original_frame.scene()==scene
    original_frame.dispose();original_request.dispose()
    (out/f'model-{mode}.plot.json').write_text(wire);(out/f'model-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'model-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose()
output.dispose();print('PASS Python model controls: ten authors, 30 publications.')
